//! R6 `sources adopt` and `coverage`: bytes that arrived by another route are recorded only when
//! their size equals NASA's listing (a copy that is 1 MiB off is refused, never recorded as NASA's),
//! are hashed from disk, and nothing is downloaded; a dry run writes nothing; coverage counts what
//! is recorded, unrecorded and missing.

use std::path::PathBuf;

use ignis_data::sources::adopt::{adopt, coverage_of};
use ignis_data::sources::manifest as mf;
use ignis_data::sources::storage::flight;
use serde_json::json;

const INV: &str = "PSI-26";

static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn root(tag: &str) -> PathBuf {
    let r = std::env::temp_dir().join(format!("ignis-adopt-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&r);
    std::fs::create_dir_all(r.join("manifests")).unwrap();
    std::fs::create_dir_all(r.join("raw/samples").join(INV)).unwrap();
    r
}

fn listed(name: &str, category: &str, size: i64) -> serde_json::Value {
    listed_v(name, category, size, 1)
}

fn listed_v(name: &str, category: &str, size: i64, v: i64) -> serde_json::Value {
    json!({"investigation": INV, "category": category, "key": format!("psi-data/{INV}/{category}/{name}"),
           "file_name": name, "size_bytes": size, "file_version": v, "psi_file_id": null,
           "updated": null, "restricted": false})
}

#[test]
fn adoption_records_only_intact_copies_and_coverage_counts_them() {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let r = root("intact");
    std::env::set_var("IGNIS_DATA", &r);
    let raw = r.join("raw/samples").join(INV);
    let good = b"Synthetic test bytes standing in for a NASA document.".to_vec();
    let bad = vec![b'x'; 64];
    std::fs::write(raw.join("good.pdf"), &good).unwrap();
    std::fs::write(raw.join("damaged.csv"), &bad).unwrap();
    std::fs::write(raw.join("frame.tif"), b"media stays at NASA").unwrap();
    let rows = [
        listed("good.pdf", "Reports", good.len() as i64),
        // listed 1 MiB smaller than the copy on disk: the signature of a doubled chunk
        listed(
            "damaged.csv",
            "Raw Data",
            bad.len() as i64 - 1_048_576 + 2_000_000,
        ),
        listed("absent.txt", "Raw Data", 10),
        listed("frame.tif", "Raw Data", 19),
    ];
    let body: String = rows.iter().map(|v| v.to_string() + "\n").collect();
    std::fs::write(
        r.join("manifests").join(format!("{INV}.listing.jsonl")),
        body,
    )
    .unwrap();
    std::fs::write(
        r.join("manifests")
            .join(format!("{INV}.investigation.json")),
        json!({"investigation": INV, "investigation_version": "4", "doi": "10.0/test",
               "license": "CC0-1.0", "metadata_url_sha256": "00"})
        .to_string(),
    )
    .unwrap();

    // Dry run: counts, writes nothing.
    let dry = adopt(INV, false).unwrap();
    assert_eq!(
        (dry.stored_files, dry.adopted, dry.missing),
        (3, 1, 1),
        "frame.tif is media"
    );
    assert_eq!(dry.size_mismatch.len(), 1);
    assert!(mf::read_manifest(INV).unwrap().is_empty());

    // Execute: the intact copy is recorded with the hash of its bytes; the damaged one is not.
    let run = adopt(INV, true).unwrap();
    assert_eq!(run.adopted, 1);
    assert_eq!(run.size_mismatch[0].file_name, "damaged.csv");
    let recs = mf::read_manifest(INV).unwrap();
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].file_name.as_deref(), Some("good.pdf"));
    assert_eq!(
        recs[0].id,
        format!(
            "sha256:{}",
            ignis_core::hashing::file_sha256(&raw.join("good.pdf")).unwrap()
        )
    );
    assert_eq!(recs[0].size_bytes, Some(good.len() as i64));
    assert_eq!(recs[0].rights.license.as_deref(), Some("CC0-1.0"));

    // A second run finds it recorded and adopts nothing new.
    let again = adopt(INV, true).unwrap();
    assert_eq!((again.already_recorded, again.adopted), (1, 0));

    // Coverage: 3 stored files, 1 recorded, the refused copy present but unrecorded, 1 missing,
    // the media file at NASA; BASS documents are under the lockbox.
    let c = coverage_of(flight(INV).unwrap()).unwrap();
    assert_eq!(
        (c.stored_files, c.recorded, c.unrecorded_present),
        (3, 1, 1)
    );
    assert_eq!(c.missing, vec!["absent.txt".to_string()]);
    assert_eq!(c.manifest_only, 1);
    assert!(c.lockbox);

    // A listing erratum explains the size only with the exact served size AND the bytes' hash.
    let sha = ignis_core::hashing::file_sha256(&raw.join("damaged.csv")).unwrap();
    let erratum = |sha: &str| {
        json!({"investigation": INV, "file_name": "damaged.csv", "file_version": 1,
               "listed_size": bad.len() as i64 - 1_048_576 + 2_000_000, "served_size": bad.len(),
               "sha256": sha, "checked_at": "2026-11-14T23:00:00Z", "note": "test"})
        .to_string()
            + "\n"
    };
    let errata = r.join("manifests").join("listing_errata.jsonl");
    std::fs::write(&errata, erratum(&"0".repeat(64))).unwrap();
    let wrong = adopt(INV, true).unwrap();
    assert_eq!(
        (wrong.adopted, wrong.size_mismatch.len()),
        (0, 1),
        "hash differs: refused"
    );
    std::fs::write(&errata, erratum(&sha)).unwrap();
    let fixed = adopt(INV, true).unwrap();
    assert_eq!((fixed.adopted, fixed.size_mismatch.len()), (1, 0));
    let recs = mf::read_manifest(INV).unwrap();
    let d = recs
        .iter()
        .find(|s| s.file_name.as_deref() == Some("damaged.csv"))
        .unwrap();
    assert_eq!(d.id, format!("sha256:{sha}"));
    assert_eq!(
        d.size_bytes,
        Some(bad.len() as i64),
        "the record states the served size"
    );
}

#[test]
fn one_path_holds_the_latest_version_and_duplicate_claims_are_pruned() {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let r = root("versions");
    std::env::set_var("IGNIS_DATA", &r);
    let raw = r.join("raw/samples").join(INV);
    let bytes = b"same bytes listed under two versions".to_vec();
    std::fs::write(raw.join("tp.txt"), &bytes).unwrap();
    let n = bytes.len() as i64;
    let rows = [
        listed_v("tp.txt", "Raw Data", n, 1),
        listed_v("tp.txt", "Raw Data", n, 2),
    ];
    let body: String = rows.iter().map(|v| v.to_string() + "\n").collect();
    std::fs::write(
        r.join("manifests").join(format!("{INV}.listing.jsonl")),
        body,
    )
    .unwrap();
    std::fs::write(
        r.join("manifests")
            .join(format!("{INV}.investigation.json")),
        json!({"investigation": INV, "investigation_version": "4", "license": "CC0-1.0",
               "metadata_url_sha256": "00"})
        .to_string(),
    )
    .unwrap();
    // A duplicate claim from before the fix: v1 recorded with the very bytes on the path.
    let sha = format!(
        "sha256:{}",
        ignis_core::hashing::file_sha256(&raw.join("tp.txt")).unwrap()
    );
    let rec = |v: &str| {
        json!({"id": sha, "kind": "dataset_file", "investigation_id": INV, "file_name": "tp.txt",
               "file_version": v, "size_bytes": n, "rights": {"status": "verified"}})
        .to_string()
    };
    std::fs::write(
        r.join("manifests").join(format!("{INV}.jsonl")),
        rec("1") + "\n",
    )
    .unwrap();

    let rep = adopt(INV, true).unwrap();
    assert_eq!(rep.stored_files, 1, "one path, one (latest) version");
    assert_eq!((rep.adopted, rep.pruned_duplicate_versions), (1, 1));
    let recs = mf::read_manifest(INV).unwrap();
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].file_version.as_deref(), Some("2"));
    let c = coverage_of(flight(INV).unwrap()).unwrap();
    assert_eq!(
        (c.stored_files, c.recorded, c.unrecorded_present),
        (1, 1, 0)
    );
}

#[test]
fn the_committed_corpus_identity_is_reproducible_from_the_committed_manifests() {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    std::env::set_var("IGNIS_DATA", &data);
    let cov: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(data.join("manifests/coverage.json")).unwrap(),
    )
    .unwrap();
    let invs: Vec<&str> = ignis_data::sources::storage::FLIGHTS
        .iter()
        .map(|f| f.accession)
        .collect();
    assert_eq!(
        mf::manifest_hash(&invs).unwrap(),
        cov["manifest_hash"].as_str().unwrap(),
        "a manifest changed without `ignis sources coverage --out data/manifests/coverage.json`"
    );
    // Every flight investigation has a coverage row, and every gap is named.
    let rows = cov["investigations"].as_array().unwrap();
    assert_eq!(rows.len(), 19);
    for r in rows {
        let stored = r["stored_files"].as_u64().unwrap();
        let recorded = r["recorded"].as_u64().unwrap();
        let missing = r["missing"].as_array().unwrap().len() as u64;
        let unrecorded = r["unrecorded_present"].as_u64().unwrap();
        assert_eq!(
            stored,
            recorded + missing + unrecorded,
            "{}",
            r["investigation"]
        );
    }
}
