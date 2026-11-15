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

### Retrieval evaluation and the τ/w sweep (R12b)

```bash
ignis evidence retrieval-eval --tasks benchmarks/t10/dev/tasks.json \
  --db artifacts/evidence/evidence.sqlite --pack artifacts/evidence/expansion-llm \
  [--expected-terms <task_id → ExpectedTerm[] json>] [--tau 0.01,0.02,0.05,0.1,0.2] [--w 0.1,...,4] \
  [--out artifacts/evidence/retrieval_eval]
```

nDCG@10 and recall@10 per question type (single-hop, multi-hop, figure) for `bm25`,
`query_expansion`, `doc_expansion` and `query_doc_expansion`, ranked as the builder scope sees the
corpus (the lockbox applies). The input catalog is never written: each τ re-enriches a working copy.
Writes `report.json`, `report.md` and `proposal.json`; the proposal is never applied to `sira.rs`.
Dev split only: it refuses `holdout/` paths, any file holding a non-dev task and any scope that may
read the test partition. The relevance rule (span, page, table row, path) and the metric definition
are in `src/retrieval_eval.rs`. The fixture in `tests/fixtures/retrieval_eval/` is synthetic.

## Tests

```bash
cargo test --release -p ignis-evidence
```

`rrf_uses_ranks_only`, `fts_rrf_returns_known_chunk_with_page_locator` (only with `--features pdf`),
`firewall_passes_on_current_workspace`, `firewall_detects_leaks`,
`sealed_runs_strip_outcome_bearing_evidence`.

See also: [GUIDE](../../docs/GUIDE.md#9-evidence-search) · [REPOMAP](../../docs/REPOMAP.md)
