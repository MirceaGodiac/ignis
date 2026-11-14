# ignis-core — rules for agents

This crate is the single source of truth for every cross-lane type. It depends on no workspace crate
and must never depend on HTTP, MCP, SQLite, Tokio or any expert/service crate.

- Edit only the contract family you own (`docs/implementation/CONTRACTS.md` table). A contract PR
  contains no lane implementation.
- New struct: `#[serde(deny_unknown_fields)]`; optional fields
  `#[serde(default, skip_serializing_if = "Option::is_none")]`; finite numbers as `Finite`; ids as the
  typed newtypes in `contracts/common.rs`; top-level documents carry their family `schema_version` tag.
- Never add a field to a legacy type in `contracts.rs` that serializes when absent: it changes content
  hashes (release ids, prefix `input_hash`). Use `skip_serializing_if`.
- An invariant goes in a `validate()` next to the type, with one focused case in `tests/contracts.rs`.
- Export new top-level types in `schema::all()`; add or update a fixture in
  `fixtures/contracts/manifest.json`; then:

```bash
cargo run -p ignis-cli -- schemas export
npm --prefix web run gen:types
cargo test -p ignis-core
npm --prefix web run check:types
```

- Fixture hashes are produced by Rust; when they drift, `fixtures_are_mutually_consistent` prints the
  correct values. Never hand-edit a hash to make a different payload pass.
