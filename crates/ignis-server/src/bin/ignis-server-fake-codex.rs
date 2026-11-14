//! Test double of `codex app-server` for the server conformance tests (same program as
//! `ignis-research`'s `ignis-fake-codex`; a crate's tests can only locate its own binaries).

fn main() {
    ignis_research::runtime::fake::main();
}
