//! NTRS client, offline: the real response shape (saved responses), rights from NTRS's own
//! determination, metadata-only identity, the provider's rate limit, and duplicate candidates with
//! PSI. No network.

use std::time::Duration;

use ignis_core::contracts::{AccessMode, IdBasis, RightsStatus};
use ignis_data::sources::ntrs::{
    entries, ntrs_id_in, page_total, parse_record, record_id, rights, source_record,
    wait_before_next, MIN_INTERVAL,
};
use serde_json::{json, Value};

fn fixture(name: &str) -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

#[test]
fn records_parse_with_absolute_links_and_public_use_rights() {
    let cit = fixture("ntrs_citation_20150023456.json");
    let r = parse_record(&cit).unwrap();
    assert_eq!(r.ntrs_id, "20150023456");
    assert_eq!(r.sti_type.as_deref(), Some("TECHNICAL_PUBLICATION"));
    assert_eq!(
        r.pdf_url.as_deref(),
        Some("https://ntrs.nasa.gov/api/citations/20150023456/downloads/20150023456.pdf")
    );
    assert_eq!(
        r.fulltext_url.as_deref(),
        Some("https://ntrs.nasa.gov/api/citations/20150023456/downloads/20150023456.txt")
    );
    // Both spellings NTRS uses for public use are verified.
    assert_eq!(r.determination.as_deref(), Some("PUBLIC_USE_PERMITTED"));
    assert_eq!(rights(&r).status, RightsStatus::Verified);
    let search = fixture("ntrs_search_flame_extinguishment.json");
    let first = parse_record(&search["results"][0]).unwrap();
    assert_eq!(
        first.determination.as_deref(),
        Some("GOV_PUBLIC_USE_PERMITTED")
    );
    assert_eq!(rights(&first).status, RightsStatus::Verified);
}

#[test]
fn rights_follow_distribution_and_determination() {
    let mut v = fixture("ntrs_citation_20150023456.json");
    v["distribution"] = json!("RESTRICTED");
    assert_eq!(
        rights(&parse_record(&v).unwrap()).status,
        RightsStatus::Restricted
    );
    let mut v = fixture("ntrs_citation_20150023456.json");
    v["copyright"]["determinationType"] = json!("PERMISSION_REQUIRED");
    assert_eq!(
        rights(&parse_record(&v).unwrap()).status,
        RightsStatus::Unverified
    );
    // No downloads offered: no links, even if the record lists files.
    let mut v = fixture("ntrs_citation_20150023456.json");
    v["downloadsAvailable"] = json!(false);
    let r = parse_record(&v).unwrap();
    assert!(r.pdf_url.is_none() && r.fulltext_url.is_none());
}

#[test]
fn a_record_is_a_metadata_source_without_investigation_identity() {
    let search = fixture("ntrs_search_flame_extinguishment.json");
    let es = entries(
        "flame extinguishment experiment",
        search["results"].as_array().unwrap(),
        "2026-11-14T00:00:00+00:00",
    )
    .unwrap();
    assert_eq!(es.iter().map(|e| e.rank).collect::<Vec<_>>(), vec![1, 2]);
    for e in &es {
        assert_eq!(e.source.id_basis, IdBasis::MetadataRecord);
        assert_eq!(
            e.source.investigation_id, None,
            "a query match is not identity"
        );
        assert_eq!(e.access_mode, AccessMode::Proxy);
        assert_eq!(
            e.source.canonical_url.as_deref(),
            Some(format!("https://ntrs.nasa.gov/citations/{}", e.record.ntrs_id).as_str())
        );
    }
    // The id names the record version: query-specific fields do not change it, an edit does.
    let mut v = search["results"][0].clone();
    let id = record_id(&v);
    v["_meta"] = json!({"score": 1.0});
    assert_eq!(record_id(&v), id);
    v["modified"] = json!("2027-01-01T00:00:00.0000000+00:00");
    assert_ne!(record_id(&v), id);
    let r = parse_record(&v).unwrap();
    assert_eq!(
        source_record(&v, &r, "t").provider_version.as_deref(),
        Some("2027-01-01T00:00:00.0000000+00:00")
    );
}

#[test]
fn the_gate_keeps_within_the_providers_limit() {
    // Spread: 500 requests per 15 minutes.
    assert_eq!(wait_before_next(None, Some(498), None, 0), Duration::ZERO);
    assert_eq!(
        wait_before_next(Some(Duration::from_millis(300)), Some(10), Some(900), 0),
        MIN_INTERVAL - Duration::from_millis(300)
    );
    // Exhausted window: wait for the reset the provider announced.
    assert_eq!(
        wait_before_next(Some(Duration::from_secs(5)), Some(0), Some(1_000), 940),
        Duration::from_secs(61)
    );
    assert_eq!(MIN_INTERVAL * 500, Duration::from_secs(15 * 60));
}

#[test]
fn psi_names_flag_duplicate_candidates_only() {
    // The two PSI documents of the atlas that carry an NTRS id in their name.
    assert_eq!(
        ntrs_id_in("PSI-69_Presentations_20130010979.pdf").as_deref(),
        Some("20130010979")
    );
    assert_eq!(
        ntrs_id_in("PSI-23_Presentations_20120015928.pdf").as_deref(),
        Some("20120015928")
    );
    for no in [
        "PSI-69_Reports_2015.pdf",
        "PSI-69_Raw Data_201300109790.txt",
        "PSI-69_x_30130010979.pdf",
    ] {
        assert_eq!(ntrs_id_in(no), None, "{no}");
    }
}

#[test]
fn a_page_total_that_is_absent_estimated_or_over_the_cap_is_refused() {
    use serde_json::json;
    assert_eq!(
        page_total(&json!({"stats": {"total": 18, "estimate": false}})).unwrap(),
        18
    );
    // Absent is not zero: paging on it would stop after one page and truncate the listing.
    assert!(page_total(&json!({"stats": {}})).is_err());
    assert!(page_total(&json!({"stats": {"total": 18, "estimate": true}})).is_err());
    assert!(page_total(&json!({"stats": {"total": 10_001, "estimate": false}})).is_err());
}
