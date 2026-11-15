# Contributing to IGNIS

This guide covers how a change gets from a branch to `main`, and from `main` to production. It is
binding for people and coding agents alike. The agent-specific operating guide is
[`AGENTS.md`](AGENTS.md); ownership and merge gates are in
[`docs/implementation/INTEGRATION.md`](docs/implementation/INTEGRATION.md); contract rules are in
[`docs/implementation/CONTRACTS.md`](docs/implementation/CONTRACTS.md). Where this file and those
documents differ, they win.

> **Merging to `main` is releasing.** The origin VM deploys the tip of `main` within about a minute,
> and the UI ships with it ([`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md) §4). There is no CI between
> your merge and production: your own checks are the only gate.

---

## 1. Workflow

1. **Find your lane and task.** `docs/implementation/WORKBOARD.md`, your `HANDOFF_<NAME>.md`, and the
   current phase in `docs/implementation/LANES_V2.md`. Edit only what your lane owns
   (`docs/implementation/INTEGRATION.md` §ownership map).
2. **Branch from a fresh `main`.** One short-lived branch per task, named `feat/<lane>-<task>` (or
   `fix/<lane>-<topic>`), deleted on merge. Separate worktrees are the norm:

   ```bash
   git fetch origin
   git worktree add -b feat/<lane>-<task> ../ignis-<you> origin/main
   ```

3. **Keep PRs small and focused.** One workboard task per PR. Do not mix refactors, formatting
   sweeps or unrelated fixes into a feature PR.
4. **Rebase on `main` before merging**, then re-run your checks.
5. **Run the targeted checks yourself** (§3) and list the commands you actually ran, with their
   results, in the PR description.
6. **Merge with squash** once the integrator has agreed (`INTEGRATION.md` §merge gates):

   ```bash
   gh pr merge <number> --squash --delete-branch
   ```

   The squash commit title follows the house style (§4); GitHub appends `(#<number>)`.
7. **Watch the deploy.** For server changes, confirm the release came up
   (`docs/DEPLOYMENT.md` §8) or ask an operator to.

### What a PR description contains

- task id; contracts and versions touched; paths changed;
- behaviour added or changed, and the fixtures or artifacts that prove it;
- **the commands you ran and their results** (never write PASS for something you did not run);
- limitations and what is not done; the command to reproduce.

---

## 2. No CI

The GitHub Actions workflow `ci` is disabled (since 2026-11-14, until the integrator decides
otherwise). Do not re-enable it, add or edit workflows, wait for checks, or cite CI in a PR. A PR is
mergeable on the strength of the checks its author ran and reported.

---

## 3. Checks before merging

Run what applies to your change. "Targeted" means the crates and packages you touched plus their
direct consumers, not the whole world, and not less than that.

### Rust

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test -p ignis-core                  # contracts, schema drift, fixtures, isolation, registry
cargo test -p <crate-you-changed>         # e.g. ignis-server, ignis-research, ignis-data
```

`cargo clippy -p <crate> --all-targets` is useful but is not a gate: the pinned toolchain
([`rust-toolchain.toml`](rust-toolchain.toml), channel `1.99.1`) installs only `rustfmt`; add it with
`rustup component add clippy` if you want it. Do not land clippy-driven rewrites of code you do not own.

Live tests (real model turns, the container runner, R2) are `#[ignore]` and need credentials and
explicit authorization; they are not part of the pre-merge set.

### Web (`web/`)

```bash
npm ci --prefix web                       # once, and after package-lock changes
npm --prefix web run check:types          # generated.ts matches schemas/
npm --prefix web run typecheck            # tsc --noEmit
npm --prefix web test                     # node --test "tests/**/*.test.ts"
npm --prefix web run build                # typecheck + vite build (what production runs)
```

The tests run TypeScript directly under Node's type stripping; use the Node version in
[`web/.nvmrc`](web/.nvmrc).

### Edge Worker (`infra/worker/`)

```bash
npm ci --prefix infra/worker
npm --prefix infra/worker run typecheck
npm --prefix infra/worker test            # vitest
```

### Linux build check for platform-specific code

Production builds on Linux (`infra/server/Dockerfile`, Debian bookworm, `rust:1.99.1-bookworm`), and
much of the team develops on Windows. If your change touches `#[cfg(windows)]`, `#[cfg(unix)]`,
`target_os`, file paths, process spawning, signals or permissions (current examples:
`crates/ignis-core/src/resources.rs`, `crates/ignis-research/src/sandbox.rs`,
`crates/ignis-server/src/lib.rs`), check it on Linux before merging: on a Linux machine or WSL,

```bash
cargo check --workspace --all-targets
cargo test -p <crate-you-changed>
```

A Linux compile failure does not take production down (the deploy fails closed and keeps the running
release), but it blocks every later release until someone fixes `main`.

### Shell scripts

Scripts under `infra/scripts/` run as root on the origin. They must stay LF (`.gitattributes`
enforces `*.sh text eol=lf`). Never edit a script while it is running on the VM; changes to
`autodeploy.sh` and `infra/deploy/systemd/*` install themselves after the next good deploy.

---

## 4. Commit messages

House style, as in `git log --oneline`:

```text
Area: one plain sentence saying what is now true
```

- **Area** is a capitalized component name. The common ones in history are `Web`, `Infra` and
  `Data`; use another component name (a crate, the lab, access) when it is clearer, capitalized the
  same way. Examples from history:
  - `Infra: build-data rebuilds the conditions table (#127)`
  - `Web: Sources opens empty with three tested starter searches (#125)`
  - `Data: BASS oxygen and pressure for the atmosphere map (#124)`
- The sentence describes the behaviour, in the present tense, for a reader who has not seen the
  diff. Parentheses for the short "how" are fine: `Infra: faster deploys (local snapshot, web-only
  skip, 1-minute poll) (#126)`.
- No trailing period, no ticket prefixes, no emoji. The body (optional) explains why and what was
  measured.

---

## 5. Data and science rules

These are non-negotiable (full text: [`AGENTS.md`](AGENTS.md) §Non-negotiable rules).

- **Data, results and interpretation stay separate.** An original value keeps its raw value, raw
  unit, SI value and locator. IGNIS computations are artifacts with inputs and code identity. Model
  outputs are `model_output`, never measurements. LLM interpretation is a claim of kind
  `interpretation`; what the model states from its own knowledge is `background`, labelled as such.
- **Absence is explicit.** Missing, conflicting and not-applicable are statuses or documented absent
  fields, never zero. Experimental times are seconds since an explicit `time_origin`; system times
  are RFC 3339 UTC.
- **Never claim "validated".** Do not claim "validated", "superior harness" or "complete corpus"
  because tests passed. Do not state latency or throughput you did not measure. Fixtures are
  `origin = synthetic_fixture` and are never presented as NASA results.
- **Protect the test partition.** Never pass `--final-test`, read `holdout/`, score or compare on the
  test partition, or show final-test outcomes to an author. Only an evaluation scope may read it.
  Builder bundles never contain test outcomes. A change guided by test results turns the test into
  development; renaming or reseeding does not undo it.
- **Releases are immutable.** Never edit `artifacts/models/`; a change is a new release.
- **Scope is host-derived.** Never accept a scope, workspace id or partition grant from a request
  body, tool argument, prompt or document.
- **Literal WSP.** FLEX/ACME/BASS weight-space releases follow the `HandwrittenWeightPatch` rules in
  `AGENTS.md`; the registry refuses anything else.

---

## 6. Shared contracts and generated types

A type that crosses a lane boundary lives in `crates/ignis-core/src/contracts/`, never as a local
copy, a hand-written TypeScript interface or a parallel JSON Schema. A contract change ships producer,
fixture, consumer and regenerated schemas and TypeScript **in the same PR**.

```bash
cargo run -p ignis-cli -- schemas export   # writes schemas/*.json and schemas/index.json
npm --prefix web run gen:types             # web/scripts/gen-types.mjs -> web/src/contracts/generated.ts
npm --prefix web run check:types           # fails if generated.ts is stale
```

Never edit `web/src/contracts/generated.ts` by hand. A merge conflict in `schemas/` or
`generated.ts` is resolved by regenerating from the merged Rust, not by editing either side.

---

## 7. Code style

### Rust

- `cargo fmt` is the formatter; do not hand-format against it.
- Correctness by construction: typed ids, `Finite` numbers, closed enums, `deny_unknown_fields`,
  invariants in `validate()` next to the type. Fail closed; never skip invalid input silently.
- Domain logic lives in a library crate behind a service trait (`ignis_core::services`); HTTP, MCP
  and CLI are thin adapters over the same call.
- No CPU work on Tokio reactor threads (`spawn_blocking`/rayon, bounded). Move references
  (`ArtifactRef`), not payloads; stream large bytes.
- No arbitrary limits on documents, tokens, steps or result size: paginate, stream, checkpoint.
- No new framework, microservice or abstraction without a concrete consumer.

### TypeScript and React (`web/`)

- Strict TypeScript; contract types come only from `src/contracts/generated.ts`.
- Each module opens with a short comment saying what it is for; keep that up to date.
- UI follows [`DESIGN.md`](DESIGN.md): colour only where it carries meaning, sentence-case labels, no
  explanatory copy under titles. See [`docs/WEB.md`](docs/WEB.md).

### Tests

Integration and end-to-end first; unit tests only for hard invariants (contract `validate()` rules,
schema drift, test-partition isolation, citation and unit correctness, release construction,
event-log replay). No getter tests, line-by-line restatements, unexplained snapshots, heavy mocks or
coverage padding. E2E tests run offline on fixtures and fakes; live tests stay opt-in.

### Documentation

Code is the authority for what works now: never document a planned component as existing. When you
change operations, update [`infra/README.md`](infra/README.md) and
[`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md) in the same PR.
