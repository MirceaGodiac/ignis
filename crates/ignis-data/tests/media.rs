//! R11 media indexes on synthetic inputs (`synthetic_fixture`; no NASA bytes, no network):
//! keyframe index of an MPEG-TS stream (every keyframe and sampled), ZIP central directory and
//! one entry read by byte range, TIFF → PNG/WebP with the content-addressed cache.

use std::io::{Cursor, Write};

use ignis_core::contracts::{KeyframeCoverage, Origin, SourceId, VideoCodec};
use ignis_data::media::frames::{self, FrameCache, FrameFormat};
use ignis_data::media::range::{MemRange, RangeRead};
use ignis_data::media::synth::{self, SynthSpec};
use ignis_data::media::ts::{self, TsOptions};
use ignis_data::media::zipdir::{self, DirCache};

fn sid() -> SourceId {
    SourceId::new("sha256:a9585e18f0f87f9c8c3c75e8603342ce19b4c72944ebd69dd91e1e89a910cc16")
        .unwrap()
}

fn all() -> TsOptions {
    TsOptions {
        sample_every_s: None,
        ..Default::default()
    }
}

#[test]
fn every_keyframe_is_found_with_its_access_unit_and_unwrapped_pts() {
    let spec = SynthSpec {
        trailing: 100,
        ..Default::default()
    };
    let s = synth::stream(&spec);
    let r = MemRange::new(s.bytes.clone());
    let ix = ts::index(&r, sid(), all(), Origin::SyntheticFixture).unwrap();
    assert_eq!(ix.codec, VideoCodec::H264);
    assert_eq!(ix.video_pid, 0x100);
    assert_eq!(ix.coverage, KeyframeCoverage::All);
    assert_eq!(ix.init_bytes, s.init_bytes, "init = PAT + PMT");
    assert_eq!(ix.keyframes.len(), s.keyframes.len());
    for (got, want) in ix.keyframes.iter().zip(&s.keyframes) {
        assert_eq!(got.offset, want.offset);
        assert_eq!(
            got.length, want.length,
            "keyframe at {}: access unit",
            want.offset
        );
        assert!((got.pts_s.get() - want.pts_ticks as f64 / 90_000.0).abs() < 1e-9);
    }
    // The PTS wraps 10 s in; the index keeps counting.
    assert!(ix
        .keyframes
        .windows(2)
        .all(|w| w[1].pts_s.get() > w[0].pts_s.get()));
    assert!((ix.end_pts_s.get() - s.end_ticks as f64 / 90_000.0).abs() < 1e-9);
    assert!((ix.duration_s() - 60.0).abs() < 1e-9);
    // A keyframe's range ends where the next frame starts, not at the next keyframe; the partial
    // trailing packet is outside the media.
    assert!(ix
        .keyframes
        .windows(2)
        .all(|w| w[0].offset + w[0].length < w[1].offset));
    assert_eq!(ix.size_bytes - ix.media_end(), 100);
    ix.validate().unwrap();
}

#[test]
fn sampled_index_reads_a_fraction_and_lists_real_keyframes() {
    // 4 minutes, 1 s GOPs of ~260 kB: ~62 MB.
    let spec = SynthSpec {
        gops: 240,
        key_packets: 200,
        frame_packets: 40,
        ..Default::default()
    };
    let s = synth::stream(&spec);
    let r = MemRange::new(s.bytes.clone());
    let ix = ts::index(
        &r,
        sid(),
        TsOptions {
            sample_every_s: Some(4.0),
            ..Default::default()
        },
        Origin::SyntheticFixture,
    )
    .unwrap();
    assert_eq!(ix.coverage, KeyframeCoverage::Sampled);
    let truth: std::collections::HashMap<u64, _> =
        s.keyframes.iter().map(|k| (k.offset, *k)).collect();
    for k in &ix.keyframes {
        let t = truth
            .get(&k.offset)
            .expect("a listed keyframe is a real keyframe");
        assert_eq!(k.length, t.length);
    }
    let gaps: Vec<f64> = ix
        .keyframes
        .windows(2)
        .map(|w| w[1].pts_s.get() - w[0].pts_s.get())
        .collect();
    assert!(gaps.iter().all(|g| (3.0..=6.0).contains(g)), "{gaps:?}");
    assert!(ix.keyframes.len() >= 50, "{}", ix.keyframes.len());
    assert_eq!(ix.bytes_read, r.meter().bytes());
    assert!(
        (ix.bytes_read as f64) < 0.4 * ix.size_bytes as f64,
        "read {} of {}",
        ix.bytes_read,
        ix.size_bytes
    );
    ix.validate().unwrap();
}

#[test]
fn open_gop_i_slices_count_as_keyframes() {
    let s = synth::stream(&SynthSpec {
        gops: 8,
        i_slices: true,
        ..Default::default()
    });
    let ix = ts::index(
        &MemRange::new(s.bytes),
        sid(),
        all(),
        Origin::SyntheticFixture,
    )
    .unwrap();
    assert_eq!(ix.keyframes.len(), 8);
}

#[test]
fn non_ts_input_is_refused() {
    let r = MemRange::new(vec![0u8; 188 * 10]);
    let e = ts::index(&r, sid(), all(), Origin::SyntheticFixture).unwrap_err();
    assert!(e.to_string().contains("sync"), "{e}");
}

/// A 16-bit grey TIFF frame with a gradient.
fn tiff16(w: u32, h: u32) -> (Vec<u8>, Vec<u16>) {
    let px: Vec<u16> = (0..w * h).map(|i| (i * 977 % 65536) as u16).collect();
    let mut out = Cursor::new(Vec::new());
    tiff::encoder::TiffEncoder::new(&mut out)
        .unwrap()
        .write_image::<tiff::encoder::colortype::Gray16>(w, h, &px)
        .unwrap();
    (out.into_inner(), px)
}

fn zip_of(entries: &[(&str, &[u8], bool)]) -> Vec<u8> {
    let mut w = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data, deflate) in entries {
        let o = zip::write::SimpleFileOptions::default().compression_method(if *deflate {
            zip::CompressionMethod::Deflated
        } else {
            zip::CompressionMethod::Stored
        });
        w.start_file(*name, o).unwrap();
        w.write_all(data).unwrap();
    }
    w.finish().unwrap().into_inner()
}

#[test]
fn a_zip_entry_is_read_by_range_from_the_cached_directory() {
    let (tif, _) = tiff16(64, 48);
    let filler = vec![7u8; 3_000_000];
    let z = zip_of(&[
        ("run1/notes.txt", b"ignition at t=0", true),
        ("run1/big.bin", &filler, false),
        ("run1/frame_0001.tif", &tif, true),
    ]);
    let size = z.len() as u64;
    let r = MemRange::new(z);
    let dir = tempfile::tempdir().unwrap();
    let cache = DirCache::new(dir.path());
    let d = cache.get("sha256:x", &r).unwrap();
    assert_eq!(d.entries.len(), 3);
    // Tail + directory: two reads, a few kB.
    assert_eq!(r.meter().requests(), 2);
    assert!(r.meter().bytes() < 70_000);

    let before = r.meter().bytes();
    let e = d.entry("run1/frame_0001.tif").unwrap();
    let got = zipdir::read_entry(&r, e).unwrap();
    assert_eq!(got, tif);
    assert!(
        r.meter().bytes() - before < e.compressed_size + 2_000,
        "one entry costs about its compressed size, not the archive"
    );
    let stored = zipdir::read_entry(&r, d.entry("run1/big.bin").unwrap()).unwrap();
    assert_eq!(stored, filler);

    // The cached directory is used for the same archive version, not for another size.
    let again = cache.load("sha256:x", size).unwrap().unwrap();
    assert_eq!(again.entries, d.entries);
    assert!(cache.load("sha256:x", size + 1).unwrap().is_none());

    // A corrupted range never passes as the entry (CRC-32).
    let mut bad = r.bytes.clone();
    let at = (e.local_header_offset + 30 + e.name.len() as u64 + 5) as usize;
    bad[at] ^= 0xff;
    assert!(zipdir::read_entry(&MemRange::new(bad), e).is_err());
}

#[test]
fn tiff_frames_convert_to_faithful_png_and_display_webp_cached_by_content() {
    let (tif, px) = tiff16(40, 30);
    let png = frames::convert(&tif, FrameFormat::Png).unwrap();
    let mut dec = png::Decoder::new(Cursor::new(png.as_slice()))
        .read_info()
        .unwrap();
    let mut buf = vec![0u8; dec.output_buffer_size().unwrap()];
    let info = dec.next_frame(&mut buf).unwrap();
    assert_eq!((info.width, info.height), (40, 30));
    assert_eq!(
        info.bit_depth,
        png::BitDepth::Sixteen,
        "PNG keeps 16-bit samples"
    );
    let back: Vec<u16> = buf[..info.buffer_size()]
        .chunks_exact(2)
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect();
    assert_eq!(back, px, "every sample survives");

    let webp = frames::convert(&tif, FrameFormat::Webp).unwrap();
    assert_eq!(&webp[..4], b"RIFF");
    assert_eq!(&webp[8..12], b"WEBP");

    let dir = tempfile::tempdir().unwrap();
    let cache = FrameCache::new(dir.path());
    let (a, hit_a) = cache.get(&tif, FrameFormat::Png).unwrap();
    let (b, hit_b) = cache.get(&tif, FrameFormat::Png).unwrap();
    assert!(!hit_a && hit_b);
    assert_eq!(a, b);
    assert_eq!(a, png);
    assert!(frames::convert(b"not a tiff", FrameFormat::Png).is_err());
}

#[test]
fn http_content_range_is_parsed() {
    use ignis_data::media::range::parse_content_range;
    assert_eq!(
        parse_content_range("bytes 0-0/188864800"),
        Some((0, 0, 188864800))
    );
    assert_eq!(parse_content_range("bytes */100"), None);
}
