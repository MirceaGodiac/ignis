//! Essential vision checks: container PTS (synthetic VFR), causal mask dependencies, homography,
//! units kept apart (pixel runs are ImageContourObservation bundles, metric runs ContourObservation),
//! and the scope checks for serving visual bundles.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ignis_core::contracts::{
    Capability, ContourObservation, ContourStatus, ErrorCode, EvidenceClass, ExecutionScope,
    ImageContourObservation, Origin, Partition, PrincipalId, Resolution, RunId, RunIdentity,
    ScopeId, ScopeRole, Split, UnitsStatus, VisualObservationBundle, WorkspaceId, IMAGE_CONTOURS,
    METRIC_CONTOURS,
};
use ignis_vision::bundle::{resolve_identity, ClipMapEntry, RunPartitions, VisualBundleStore};
use ignis_vision::calibration::{apply_h, calibrate, mat_inv, CalibrationSpec};
use ignis_vision::pipeline::{run_contours, RunParams, Segmentation};
use ignis_vision::segment::{causal_dependencies, ClassicalSegmenter};
use ignis_vision::video::{iter_frames, probe};

/// Irregular presentation times (ms): a VFR clip where index/fps would be wrong.
const PTS_MS: [u64; 10] = [0, 40, 70, 150, 160, 260, 300, 420, 450, 600];
const W: usize = 160;
const H: usize = 128;

fn split_annexb(b: &[u8]) -> Vec<&[u8]> {
    let mut starts = vec![];
    let mut i = 0;
    while i + 3 <= b.len() {
        if b[i] == 0 && b[i + 1] == 0 && b[i + 2] == 1 {
            starts.push(i + 3);
            i += 3;
        } else {
            i += 1;
        }
    }
    starts
        .iter()
        .enumerate()
        .map(|(k, &s)| {
            let mut e = starts.get(k + 1).map(|&n| n - 3).unwrap_or(b.len());
            while e > s && b[e - 1] == 0 {
                e -= 1;
            }
            &b[s..e]
        })
        .collect()
}

/// Dark clip; a bright bar grows from the left (a front moving +x), H.264 in MP4 with VFR `stts`.
fn synthetic_vfr(path: &Path) {
    use openh264::encoder::{Encoder, EncoderConfig};
    use openh264::formats::{RgbSliceU8, YUVBuffer};
    let cfg = EncoderConfig::new().skip_frames(false);
    let mut enc = Encoder::with_api_config(openh264::OpenH264API::from_source(), cfg).unwrap();
    let mut samples = vec![];
    let (mut sps, mut pps) = (vec![], vec![]);
    for i in 0..PTS_MS.len() {
        let mut rgb = vec![0u8; W * H * 3];
        for y in 40..80 {
            for x in 10..(30 + 10 * i) {
                rgb[(y * W + x) * 3..(y * W + x) * 3 + 3].copy_from_slice(&[255, 255, 255]);
            }
        }
        let yuv = YUVBuffer::from_rgb8_source(RgbSliceU8::new(&rgb, (W, H)));
        let bs = enc.encode(&yuv).unwrap().to_vec();
        let mut avcc = vec![];
        for nal in split_annexb(&bs) {
            match nal[0] & 0x1f {
                7 => sps = nal.to_vec(),
                8 => pps = nal.to_vec(),
                _ => {
                    avcc.extend_from_slice(&(nal.len() as u32).to_be_bytes());
                    avcc.extend_from_slice(nal);
                }
            }
        }
        samples.push((avcc, i == 0));
    }
    let f = std::fs::File::create(path).unwrap();
    let cfg = mp4::Mp4Config {
        major_brand: "isom".parse().unwrap(),
        minor_version: 512,
        compatible_brands: vec!["isom".parse().unwrap(), "avc1".parse().unwrap()],
        timescale: 1000,
    };
    let mut w = mp4::Mp4Writer::write_start(std::io::BufWriter::new(f), &cfg).unwrap();
    w.add_track(&mp4::TrackConfig {
        track_type: mp4::TrackType::Video,
        timescale: 1000,
        language: "und".into(),
        media_conf: mp4::MediaConfig::AvcConfig(mp4::AvcConfig {
            width: W as u16,
            height: H as u16,
            seq_param_set: sps,
            pic_param_set: pps,
        }),
    })
    .unwrap();
    for (i, (bytes, sync)) in samples.into_iter().enumerate() {
        let dur = if i + 1 < PTS_MS.len() {
            PTS_MS[i + 1] - PTS_MS[i]
        } else {
            40
        };
        let s = mp4::Mp4Sample {
            start_time: PTS_MS[i],
            duration: dur as u32,
            rendering_offset: 0,
            is_sync: sync,
            bytes: bytes.into(),
        };
        w.write_sample(1, &s).unwrap();
    }
    w.write_end().unwrap();
}

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ignis-vision-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn pts_come_from_the_container() {
    let d = tmp("pts");
    let clip = d.join("vfr.mp4");
    synthetic_vfr(&clip);
    let pr = probe(&clip).unwrap();
    assert_eq!(pr.n_frames, PTS_MS.len());
    assert!(pr.variable_frame_rate);
    let got: Vec<f64> = iter_frames(&clip, 1, 0, None, false)
        .unwrap()
        .map(|f| f.unwrap().pts_s)
        .collect();
    let want: Vec<f64> = PTS_MS.iter().map(|&t| t as f64 / 1000.0).collect();
    assert_eq!(got, want);
    let by_rate: Vec<f64> = (0..PTS_MS.len()).map(|i| i as f64 / 25.0).collect();
    assert_ne!(got, by_rate, "index/fps would be wrong");

    // Real BASS-II clip (B-frames, edit list): decoded PTS are the container's sample PTS.
    let sample = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/raw/samples/psi25/BASS_GMT178_SAMPLE_136_TEST_T12.mp4");
    if sample.exists() {
        let pr = probe(&sample).unwrap();
        assert_eq!(
            (pr.n_frames, pr.time_base.as_str(), pr.first_pts_s),
            (1374, "1/30000", Some(0.0))
        );
        let pts: Vec<i64> = iter_frames(&sample, 1, 0, Some(60), false)
            .unwrap()
            .map(|f| f.unwrap().pts)
            .collect();
        assert_eq!(pts, (0..60).map(|i| i * 1001).collect::<Vec<i64>>());
    }
}

#[test]
fn contour_masks_are_causal() {
    let d = tmp("causal");
    let clip = d.join("vfr.mp4");
    synthetic_vfr(&clip);
    let pr = probe(&clip).unwrap();
    let spec = CalibrationSpec {
        applies_to: vec!["synthetic".into()],
        pixel_only: true,
        image_size_px: Some([W as u32, H as u32]),
        propagation_sign: 1,
        point_sigma_px: 1.0,
        ..Default::default()
    };
    let cal = calibrate(&spec, Some(&pr.sha256), 0, 0).unwrap();
    let params = RunParams {
        run_id: "synthetic:vfr".into(),
        run: RunIdentity {
            run_id: None,
            resolution: Resolution::NotApplicable,
        },
        camera_id: "synthetic-camera".into(),
        time_origin: "container PTS 0 s (synthetic clip)".into(),
        every: 1,
        start: 0,
        stop: None,
        time_origin_s: 0.0,
        k: 16,
        evidence_class: EvidenceClass::Synthetic,
        out_root: d.join("out"),
    };
    let pool = ignis_core::batch::bounded_pool(4);
    let res = run_contours(
        &clip,
        &pr,
        &cal,
        Segmentation::Classical(ClassicalSegmenter::default()),
        &params,
        &pool,
    )
    .unwrap();
    let out = Path::new(res["out"].as_str().unwrap()).to_path_buf();
    // a pixel-only run never writes metric records
    assert!(!out.join("contours.jsonl").exists());
    let text = std::fs::read_to_string(out.join("image_contours.jsonl")).unwrap();
    let obs: Vec<ImageContourObservation> = text
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(obs.len(), PTS_MS.len());
    for o in &obs {
        o.validate().unwrap();
        assert_eq!(o.units, UnitsStatus::PixelOnly);
        assert_eq!(o.origin, Origin::SyntheticFixture);
        assert_eq!(
            o.locator.pts_s,
            Some(PTS_MS[o.frame as usize] as f64 / 1000.0)
        );
        // the metric contract refuses these records outright
        assert!(
            serde_json::from_str::<ContourObservation>(&serde_json::to_string(o).unwrap()).is_err()
        );
    }
    let b: VisualObservationBundle =
        serde_json::from_str(&std::fs::read_to_string(out.join("visual_bundle.json")).unwrap())
            .unwrap();
    b.validate().unwrap();
    assert_eq!(b.units_status, UnitsStatus::PixelOnly);
    assert_eq!(b.contours.contract.as_deref(), Some(IMAGE_CONTOURS));
    assert_eq!(
        b.contours.sha256,
        ignis_core::contracts::Digest::of_bytes(text.as_bytes())
    );
    assert!(
        b.calibration_id.is_none(),
        "a pixel spec is not a metric calibration"
    );
    assert!(b.require_metric().is_err());
    let lead: Vec<f64> = obs
        .iter()
        .filter(|o| o.status == ContourStatus::SingleValued)
        .map(|o| {
            o.f.iter()
                .flatten()
                .map(|v| v.get())
                .fold(f64::MIN, f64::max)
        })
        .collect();
    assert!(
        lead.len() >= 5 && lead.windows(2).all(|w| w[0] <= w[1]) && lead[lead.len() - 1] > lead[0],
        "front moves forward: {lead:?}"
    );
    // a recurrent tracker's dependencies never reach the future; a future prompt is refused
    assert_eq!(causal_dependencies(&[3], 7).unwrap(), vec![3, 4, 5, 6, 7]);
    assert!(causal_dependencies(&[3, 9], 7).is_err());
    let mut bad = obs[2].clone();
    bad.mask_dependency_frames = vec![2, 3];
    assert!(bad.validate().is_err());
}

#[test]
fn a_metric_run_writes_contour_observations_in_metres() {
    let d = tmp("metric");
    let clip = d.join("vfr.mp4");
    synthetic_vfr(&clip);
    let pr = probe(&clip).unwrap();
    // 1 mm per pixel over the frame: four declared corners
    let px = [[10.0, 10.0], [150.0, 10.0], [150.0, 118.0], [10.0, 118.0]];
    let m = |p: [f64; 2]| [(p[0] - 10.0) * 1e-3, (p[1] - 10.0) * 1e-3];
    let spec = CalibrationSpec {
        applies_to: vec!["synthetic".into()],
        specimen_length_m: Some(0.14),
        specimen_width_m: Some(0.108),
        image_points_px: px.to_vec(),
        plane_points_m: px.iter().map(|p| m(*p)).collect(),
        point_sigma_px: 0.5,
        camera_distance_m: Some(0.5),
        flame_standoff_max_m: Some(0.002),
        propagation_sign: 1,
        image_size_px: Some([W as u32, H as u32]),
        ..Default::default()
    };
    let cal = calibrate(&spec, Some(&pr.sha256), 50, 0).unwrap();
    let params = RunParams {
        run_id: "synthetic:vfr".into(),
        run: RunIdentity {
            run_id: None,
            resolution: Resolution::NotApplicable,
        },
        camera_id: "synthetic-camera".into(),
        time_origin: "container PTS 0 s (synthetic clip)".into(),
        every: 1,
        start: 0,
        stop: None,
        time_origin_s: 0.0,
        k: 16,
        evidence_class: EvidenceClass::Synthetic,
        out_root: d.join("out"),
    };
    let pool = ignis_core::batch::bounded_pool(2);
    let res = run_contours(
        &clip,
        &pr,
        &cal,
        Segmentation::Classical(ClassicalSegmenter::default()),
        &params,
        &pool,
    )
    .unwrap();
    let out = Path::new(res["out"].as_str().unwrap()).to_path_buf();
    assert!(!out.join("image_contours.jsonl").exists());
    let obs: Vec<ContourObservation> = std::fs::read_to_string(out.join("contours.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(obs.len(), PTS_MS.len());
    // metres: every coordinate inside the 0.14 m x 0.108 m plane
    assert!(obs
        .iter()
        .flat_map(|o| o.f_m.iter().flatten())
        .all(|v| (-0.01..0.16).contains(v)));
    let b: VisualObservationBundle =
        serde_json::from_str(&std::fs::read_to_string(out.join("visual_bundle.json")).unwrap())
            .unwrap();
    b.validate().unwrap();
    assert_eq!(b.units_status, UnitsStatus::Metric);
    assert_eq!(b.contours.contract.as_deref(), Some(METRIC_CONTOURS));
    assert_eq!(b.calibration_id.as_deref(), Some(cal.record.id.as_str()));
}

#[test]
fn clip_identity_is_cross_checked_with_the_clip_map() {
    let map = vec![ClipMapEntry {
        file_name: "T5.mp4".into(),
        run_id: Some("PSI-25:T5".into()),
        resolution: "resolved".into(),
    }];
    let ok = resolve_identity("T5.mp4", None, &["PSI-25:T5".into()], &map).unwrap();
    assert_eq!(
        (ok.run_id.unwrap().as_str(), ok.resolution),
        ("PSI-25:T5", Resolution::Resolved)
    );
    let conflict = resolve_identity("T5.mp4", Some("PSI-25:T6"), &[], &map).unwrap();
    assert_eq!(conflict.resolution, Resolution::UnresolvedConflict);
    let unmapped = resolve_identity("other.mp4", None, &["PSI-25:T9".into()], &map).unwrap();
    assert_eq!(
        unmapped.resolution,
        Resolution::Unknown,
        "declared but not cross-checked"
    );
    let none = resolve_identity("other.mp4", None, &[], &map).unwrap();
    assert_eq!((none.run_id, none.resolution), (None, Resolution::Unknown));
}

fn scope(role: ScopeRole, parts: &[Partition], caps: &[Capability]) -> ExecutionScope {
    ExecutionScope {
        scope_id: ScopeId::new("s").unwrap(),
        workspace_id: WorkspaceId::new("w").unwrap(),
        principal: PrincipalId::new("p").unwrap(),
        role,
        partitions: parts.iter().copied().collect(),
        capabilities: caps.iter().copied().collect(),
        corpus_snapshot: None,
        parent_scope_id: None,
    }
}

#[test]
fn visual_bundles_are_served_only_within_the_scope() {
    let split = Split {
        id: "split".into(),
        cohort_id: "cohort".into(),
        group_key: "session".into(),
        rule: "fixture".into(),
        seed: 0,
        run_to_group: BTreeMap::from([
            ("PSI-25:T5".to_string(), "g1".to_string()),
            ("PSI-25:T12".to_string(), "g2".to_string()),
        ]),
        train: vec!["g1".into()],
        validation: vec![],
        test: vec!["g2".into()],
        frozen_at: "2026-11-14T00:00:00Z".into(),
    };
    let store = VisualBundleStore::open(tmp("store"), RunPartitions::new(&[split]));
    let fixture: VisualObservationBundle = serde_json::from_str(
        &std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/contracts/data/visual_bundle_pixel_only.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let with = |run: Option<&str>, res: Resolution| {
        let mut b = fixture.clone();
        b.origin = Origin::System;
        b.run = RunIdentity {
            run_id: run.map(|r| RunId::new(r).unwrap()),
            resolution: res,
        };
        b
    };
    let rd = [Capability::ReadData];
    let research = scope(
        ScopeRole::Research,
        &[Partition::Train, Partition::Validation],
        &rd,
    );
    let eval = scope(
        ScopeRole::Evaluation,
        &[Partition::Train, Partition::Validation, Partition::Test],
        &rd,
    );
    let train_run = with(Some("PSI-25:T5"), Resolution::Resolved);
    let test_run = with(Some("PSI-25:T12"), Resolution::Resolved);
    let unknown = with(Some("PSI-25:T5"), Resolution::Unknown);
    let conflict = with(Some("PSI-25:T5"), Resolution::UnresolvedConflict);
    assert!(store.admit(&research, &train_run).is_ok());
    assert_eq!(
        store.admit(&research, &test_run).unwrap_err().code,
        ErrorCode::ScopeDenied
    );
    assert!(store.admit(&eval, &test_run).is_ok());
    // an unresolved clip could show a sealed run: evaluation only
    for b in [&unknown, &conflict] {
        assert_eq!(
            store.admit(&research, b).unwrap_err().code,
            ErrorCode::ScopeDenied
        );
        assert!(store.admit(&eval, b).is_ok());
    }
    // no read_data, or a malformed scope, gets nothing
    let no_cap = scope(
        ScopeRole::Research,
        &[Partition::Train],
        &[Capability::ReadSources],
    );
    assert_eq!(
        store.admit(&no_cap, &train_run).unwrap_err().code,
        ErrorCode::ScopeDenied
    );
    let forged = scope(
        ScopeRole::Research,
        &[Partition::Train, Partition::Test],
        &rd,
    );
    assert_eq!(
        store.admit(&forged, &train_run).unwrap_err().code,
        ErrorCode::ScopeDenied
    );
}

#[test]
fn homography_maps_declared_points() {
    let h_true = [
        [2.1e-4, 1.5e-5, -0.012],
        [-8e-6, 1.9e-4, -0.020],
        [2e-5, -3e-5, 1.0],
    ];
    let plane = [
        [0.0, 0.0],
        [0.1, 0.0],
        [0.1, 0.05],
        [0.0, 0.05],
        [0.05, 0.025],
        [0.02, 0.04],
    ];
    let inv = mat_inv(&h_true).unwrap();
    let px: Vec<[f64; 2]> = plane.iter().map(|&p| apply_h(&inv, p)).collect();
    let spec = CalibrationSpec {
        applies_to: vec!["synthetic".into()],
        specimen_length_m: Some(0.1),
        specimen_width_m: Some(0.05),
        image_points_px: px.clone(),
        plane_points_m: plane.to_vec(),
        point_sigma_px: 0.5,
        camera_distance_m: Some(0.5),
        flame_standoff_max_m: Some(0.005),
        propagation_sign: 1,
        ..Default::default()
    };
    let pc = calibrate(&spec, None, 400, 0).unwrap();
    for (p, q) in px.iter().zip(&plane) {
        let m = pc.px_to_plane(*p);
        assert!(
            (m[0] - q[0]).abs() < 1e-6 && (m[1] - q[1]).abs() < 1e-6,
            "{m:?} vs {q:?}"
        ); // within 1 µm
    }
    assert!(pc.residual_rms_m.unwrap() < 1e-8);
    let s = pc.scale_rel.unwrap();
    assert!(s > 0.0 && s < 0.05);
    assert!((pc.parallax_rel.unwrap() - 0.01).abs() < 1e-12);
    assert_eq!(pc.record.uncertainty["scale_rel"], s);
    // lens distortion round-trips through the plane mapping
    let lens = CalibrationSpec {
        camera_matrix: Some([[800.0, 0.0, 360.0], [0.0, 800.0, 240.0], [0.0, 0.0, 1.0]]),
        distortion: Some(vec![-0.12, 0.03, 1e-3, -5e-4, 0.0]),
        ..spec.clone()
    };
    let pl = calibrate(&lens, None, 10, 0).unwrap();
    let back = pl.px_to_plane(pl.plane_to_px([0.03, 0.02]));
    assert!((back[0] - 0.03).abs() < 1e-9 && (back[1] - 0.02).abs() < 1e-9);
    // without a stand-off bound the limitation is explicit
    let unbounded = calibrate(
        &CalibrationSpec {
            flame_standoff_max_m: None,
            ..spec
        },
        None,
        50,
        0,
    )
    .unwrap();
    assert!(unbounded.parallax_rel.is_none());
    assert!(unbounded
        .record
        .derivation
        .notes
        .unwrap_or_default()
        .contains("out_of_plane_unbounded"));
}
