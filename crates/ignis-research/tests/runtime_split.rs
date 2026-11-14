//! Stdio framing: the full turn scenario with the fake app-server writing every message in
//! 3-byte pieces (one message over many OS reads).
//! Its own test binary, so the environment variable does not leak into other tests.

mod common;

#[test]
fn full_turn_survives_split_reads() {
    std::env::set_var("IGNIS_FAKE_CODEX_FRAMING", "split");
    let dir = tempfile::tempdir().unwrap();
    let h = common::Harness::new(dir.path());
    common::full_turn(&h);
}
