//! Stdio framing: the full turn scenario with the fake app-server writing every message in
//! bursts (many messages in one OS read).
//! Its own test binary, so the environment variable does not leak into other tests.

mod common;

#[test]
fn full_turn_survives_merged_reads() {
    std::env::set_var("IGNIS_FAKE_CODEX_FRAMING", "merged");
    let dir = tempfile::tempdir().unwrap();
    let h = common::Harness::new(dir.path());
    common::full_turn(&h);
}
