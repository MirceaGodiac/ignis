//! TIFF frames → PNG / WebP on demand (R11), cached by content.
//!
//! The PSI image archives hold TIFF frames a browser cannot show. A frame is converted when a
//! viewer asks for it, never in bulk, and the result is cached under its input's SHA-256 and the
//! converter version, so the same frame is converted once and a changed converter never serves a
//! stale rendition.
//!
//! * **PNG** keeps every sample: 8- and 16-bit grey, grey+alpha, RGB and RGBA are written at their
//!   own bit depth (16-bit PNG is big-endian). PNG is the faithful rendition.
//! * **WebP** (lossless) is a display rendition: WebP has 8 bits per channel, so 16-bit samples
//!   are reduced to their high byte. It is never used for measurement.
//!
//! The work is CPU-bound: callers on an async runtime run it in `spawn_blocking`.

use std::io::Cursor;
use std::path::PathBuf;

use anyhow::{bail, Context};
use sha2::{Digest, Sha256};
use tiff::decoder::{Decoder, DecodingResult};
use tiff::ColorType;

use super::zipdir::write_atomic;

/// Changes whenever a conversion would produce different bytes.
pub const CONVERTER: &str = "frames-1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameFormat {
    Png,
    Webp,
}

impl FrameFormat {
    pub fn ext(self) -> &'static str {
        match self {
            FrameFormat::Png => "png",
            FrameFormat::Webp => "webp",
        }
    }
    pub fn content_type(self) -> &'static str {
        match self {
            FrameFormat::Png => "image/png",
            FrameFormat::Webp => "image/webp",
        }
    }
    pub fn parse(s: &str) -> Option<FrameFormat> {
        match s {
            "png" => Some(FrameFormat::Png),
            "webp" => Some(FrameFormat::Webp),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
enum Layout {
    Gray,
    GrayA,
    Rgb,
    Rgba,
}

impl Layout {
    fn channels(self) -> usize {
        match self {
            Layout::Gray => 1,
            Layout::GrayA => 2,
            Layout::Rgb => 3,
            Layout::Rgba => 4,
        }
    }
}

/// Decoded samples: 8-bit or 16-bit, interleaved.
enum Samples {
    U8(Vec<u8>),
    U16(Vec<u16>),
}

struct Image {
    width: u32,
    height: u32,
    layout: Layout,
    samples: Samples,
}

fn decode(tiff_bytes: &[u8]) -> anyhow::Result<Image> {
    // The decoder's default limits stay: a hostile file cannot exhaust memory.
    let mut d = Decoder::new(Cursor::new(tiff_bytes)).context("not a TIFF")?;
    let (width, height) = d.dimensions()?;
    let (layout, bits) = match d.colortype()? {
        ColorType::Gray(b) => (Layout::Gray, b),
        ColorType::GrayA(b) => (Layout::GrayA, b),
        ColorType::RGB(b) => (Layout::Rgb, b),
        ColorType::RGBA(b) => (Layout::Rgba, b),
        c => bail!("TIFF colour type {c:?} is not converted"),
    };
    let samples = match (d.read_image()?, bits) {
        (DecodingResult::U8(v), 8) => Samples::U8(v),
        (DecodingResult::U16(v), 16) => Samples::U16(v),
        (_, b) => bail!("TIFF with {b}-bit samples is not converted"),
    };
    let n = width as usize * height as usize * layout.channels();
    let got = match &samples {
        Samples::U8(v) => v.len(),
        Samples::U16(v) => v.len(),
    };
    anyhow::ensure!(got == n, "TIFF decoded {got} samples, expected {n}");
    Ok(Image {
        width,
        height,
        layout,
        samples,
    })
}

fn png(img: &Image) -> anyhow::Result<Vec<u8>> {
    let mut out = Vec::new();
    {
        let mut e = png::Encoder::new(&mut out, img.width, img.height);
        e.set_color(match img.layout {
            Layout::Gray => png::ColorType::Grayscale,
            Layout::GrayA => png::ColorType::GrayscaleAlpha,
            Layout::Rgb => png::ColorType::Rgb,
            Layout::Rgba => png::ColorType::Rgba,
        });
        match &img.samples {
            Samples::U8(v) => {
                e.set_depth(png::BitDepth::Eight);
                let mut w = e.write_header()?;
                w.write_image_data(v)?;
                w.finish()?;
            }
            Samples::U16(v) => {
                e.set_depth(png::BitDepth::Sixteen);
                let mut w = e.write_header()?;
                let be: Vec<u8> = v.iter().flat_map(|s| s.to_be_bytes()).collect();
                w.write_image_data(&be)?;
                w.finish()?;
            }
        }
    }
    Ok(out)
}

fn webp(img: &Image) -> anyhow::Result<Vec<u8>> {
    let bytes: Vec<u8> = match &img.samples {
        Samples::U8(v) => v.clone(),
        Samples::U16(v) => v.iter().map(|s| (s >> 8) as u8).collect(),
    };
    let color = match img.layout {
        Layout::Gray => image_webp::ColorType::L8,
        Layout::GrayA => image_webp::ColorType::La8,
        Layout::Rgb => image_webp::ColorType::Rgb8,
        Layout::Rgba => image_webp::ColorType::Rgba8,
    };
    let mut out = Vec::new();
    image_webp::WebPEncoder::new(&mut out).encode(&bytes, img.width, img.height, color)?;
    Ok(out)
}

/// Converts one TIFF frame.
pub fn convert(tiff_bytes: &[u8], fmt: FrameFormat) -> anyhow::Result<Vec<u8>> {
    let img = decode(tiff_bytes)?;
    match fmt {
        FrameFormat::Png => png(&img),
        FrameFormat::Webp => webp(&img),
    }
}

/// Content-addressed cache of converted frames: `<sha256 of the TIFF>.<converter>.<ext>`.
pub struct FrameCache {
    dir: PathBuf,
}

impl FrameCache {
    pub fn new(dir: impl Into<PathBuf>) -> FrameCache {
        FrameCache { dir: dir.into() }
    }

    pub fn path(&self, input_sha256: &str, fmt: FrameFormat) -> PathBuf {
        self.dir
            .join(format!("{input_sha256}.{CONVERTER}.{}", fmt.ext()))
    }

    /// The converted frame and whether it came from the cache.
    pub fn get(&self, tiff_bytes: &[u8], fmt: FrameFormat) -> anyhow::Result<(Vec<u8>, bool)> {
        let key = hex::encode(Sha256::digest(tiff_bytes));
        let p = self.path(&key, fmt);
        if let Ok(b) = std::fs::read(&p) {
            return Ok((b, true));
        }
        let out = convert(tiff_bytes, fmt)?;
        std::fs::create_dir_all(&self.dir)?;
        write_atomic(&p, &out)?;
        Ok((out, false))
    }
}

/// Is `name` a TIFF by its extension?
pub fn is_tiff(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.ends_with(".tif") || n.ends_with(".tiff")
}
