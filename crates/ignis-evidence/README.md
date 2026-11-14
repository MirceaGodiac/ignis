# ignis-evidence

Scientific retrieval (V3.1 §18): PDF parsing with page/character locators, a SQLite FTS5 catalog,
dense embeddings, reciprocal-rank fusion, bounded reranking, and the leak firewall.

> Updated with the DeepInfra lane. Dense retrieval and reranking are moving from a local candle model
> to the **DeepInfra API** with `Qwen/Qwen3-Embedding-8B` and `Qwen/Qwen3-Reranker-8B`. The key is read
> only from the environment variable `DEEPINFRA_API_KEY`; paid calls are dry-runs unless `--execute`.
> Flag names may change with that lane; `ignis evidence --help` is authoritative.

Evidence content is explanation, **never a predictor input**. Two mechanisms enforce it:

- `firewall::scan` checks the cargo dependency graph: the protected crates listed in
  `configs/evidence/firewall.toml` (`ignis-science`, `-flex`, `-acme`, `-bass`, `-accel`, `-train`) must
  not depend on this crate, directly or transitively.
- `firewall::strip_for_sealed` removes outcome-bearing chunks for runs under sealed evaluation.

## Modules

`pdf` (MuPDF parsing behind the `pdf` feature; page, char spans, captions) · `catalog` (SQLite + FTS5,
schema in `schemas/evidence_catalog.sql`) · `dense` (embedding / reranker client and vector files) ·
`search` (FTS5 + dense → RRF → rerank, with declared fallbacks) · `records` · `firewall`.

## Commands

```bash
ignis evidence ingest <pdf-or-dir> [--investigation PSI-69]   # needs: cargo build ... --features pdf
ignis evidence embed [--execute]
ignis evidence search "<query>" [--family flex] [--limit 5] [--no-dense] [--no-rerank] [--sealed-run <id>]
ignis evidence firewall
```

Catalog: `artifacts/evidence/evidence.sqlite` (override with `--db`). Retrieval settings:
`configs/evidence/retrieval.toml` (fusion `rrf_k = 60`, lexical/dense depth, rerank `top_n`, timeout
with fallback to the RRF order).

## Tests

```bash
cargo test --release -p ignis-evidence
```

`rrf_uses_ranks_only`, `fts_rrf_returns_known_chunk_with_page_locator` (only with `--features pdf`),
`firewall_passes_on_current_workspace`, `firewall_detects_leaks`,
`sealed_runs_strip_outcome_bearing_evidence`.

See also: [GUIDE](../../docs/GUIDE.md#9-evidence-search) · [REPOMAP](../../docs/REPOMAP.md)
