//! Minimal PNG I/O for the frame cache and the SAM2 sidecar exchange.

use std::io::BufWriter;
use std::path::Path;

pub fn write_rgb_png(path: &Path, w: usize, h: usize, rgb: &[u8]) -> anyhow::Result<()> {
    write_png(path, w, h, rgb, png::ColorType::Rgb)
}

/// A binary mask as an 8-bit grey PNG (0 / 255).
pub fn write_mask_png(path: &Path, w: usize, h: usize, mask: &[bool]) -> anyhow::Result<()> {
    let g: Vec<u8> = mask.iter().map(|&m| if m { 255 } else { 0 }).collect();
    write_png(path, w, h, &g, png::ColorType::Grayscale)
}

fn write_png(
    path: &Path,
    w: usize,
    h: usize,
    data: &[u8],
    ct: png::ColorType,
) -> anyhow::Result<()> {
    let tmp = path.with_extension("png.part");
    {
        let f = std::fs::File::create(&tmp)?;
        let mut enc = png::Encoder::new(BufWriter::new(f), w as u32, h as u32);
        enc.set_color(ct);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_compression(png::Compression::Fast);
        enc.write_header()?.write_image_data(data)?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Read an 8-bit grey (or RGB/RGBA, first channel) PNG as a mask: value > 127 is luminous.
pub fn read_mask_png(path: &Path) -> anyhow::Result<(usize, usize, Vec<bool>)> {
    let dec = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(path)?));
    let mut r = dec.read_info()?;
    let mut buf = vec![0; r.output_buffer_size().unwrap_or(0)];
    let info = r.next_frame(&mut buf)?;
    let ch = info.color_type.samples();
    if info.bit_depth != png::BitDepth::Eight {
        anyhow::bail!("mask PNG must be 8-bit");
    }
    let (w, h) = (info.width as usize, info.height as usize);
    Ok((w, h, (0..w * h).map(|i| buf[i * ch] > 127).collect()))
}
