# ignis-data

The data lane: NASA PSI sources (read-only connector, manifests, bounded samples), the international
adapters, and curation: family ingest, field rules, cohorts, frozen grouped splits, Parquet views and
the semantic review queue. `views::DataStore` implements `ignis_core::views::ViewSource` and is the
data source the binary hands to `fit`, `decide`, `eval` and `lab`.

## Modules

| Module | Contents |
|---|---|
| `sources::psi`, `sources::manifest`, `sources::samples` | PSI metadata/listing client, `Source` manifests (sha256, version, rights), the bounded sample selection |
| `sources::intl` | JAXA DARTS (GCEM), Zenodo (ZARM FLARE), ESA HREDA (metadata only), Mendeley |
| `families::{flex,acme,bass}` | parsers from NASA files to typed records, observations, endpoints, review items |
| `review` | export items, import adjudications, apply them on ingest |
| `cohort` | declared criteria → cohort + flow table; grouped 60/20/20 split freezing with content hash |
| `views` | Parquet views (physical SI scaling only), rebuilt when stale |
| `store`, `paths` | `data/work/records/<family>/` persistence and the `IGNIS_DATA` layout |

Families: `flex` (PSI-69), `acme_normal`, `acme_inverse` (PSI-159), `bass` (PSI-25; PSI-26 kept
separate).

## Commands

```bash
ignis sources inspect [INV...]          # metadata + listing (read-only)
ignis sources samples [INV...]          # bounded sample set (< ~150 MB)
ignis sources fetch <INV> [--pattern RE] [--category C] [--execute]
ignis sources manifest <INV> [--verify]
ignis sources intl list|inspect|manifest|fetch
ignis data ingest [family|all]
ignis data rules <family>
ignis data cohort build <family>
ignis data split freeze <cohort-id> --seed <n>
ignis data views <cohort-id> [--time-load]
ignis data review export <family>
ignis data review import <family> <adjudications.jsonl>
```

## Status (real CC0 NASA samples)

Ingest is byte-identical to the previous Python implementation. Cohorts: FLEX 114 runs (27/9/9
groups), ACME normal 129, ACME inverse 131, BASS planar SIBAL candidate 25 (2/1/1 groups, not
qualified). Review queue: 173 items, none adjudicated yet. FLEX views reload in ~10 ms.

## Tests

```bash
cargo test --release -p ignis-data
```

`flex_real_record`, `acme_real_record`, `bass_real_record` parse verbatim CC0 NASA files in
`tests/fixtures/` (each file's sha256 equals its manifest record); `split_groups_never_cross_partitions`.

See also: [GUIDE](../../docs/GUIDE.md#2-sources-and-the-guarded-fetch) · [REPOMAP](../../docs/REPOMAP.md) ·
[TEAM](../../docs/TEAM.md)
