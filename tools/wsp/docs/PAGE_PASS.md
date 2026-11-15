# Document lane, step 2: the whole-document page pass

The owner's decision of 14 November 2026: Claude Sonnet 5.5 sub-agents at high effort read every
NASA document and write, in one pass, the card of every page and its search enrichment. They read
a document **whole**: what a test id, an abbreviation or a table means is often said pages earlier.

- Inputs: `tools/wsp/docs/prepare.py` writes one text file per distinct document under
  `runs/dev/data_docs/inputs/` (every page marked, the NASA file names, the test ids of the
  catalogue for that investigation) and groups documents into batches without splitting one.
  Documents with the same text under two NASA files are read once.
- Each sub-agent gets one batch, reads each document in full, and writes
  `runs/dev/data_docs/cards/<document sha256>.jsonl`: one document row, then one row per page. A
  long document is written in parts, `<sha256>.p01.jsonl`, `<sha256>.p02.jsonl`, of at most 40
  pages each, after the whole document has been read.
- `tools/wsp/docs/validate.py` checks every file against the schema below, that every page came
  back exactly once, and lists what to send back.
- **Protection.** These cards say how tests ended. The folder is custody data: no model author
  (Fable for FLEX, CFI-G and BASS-II; Astra for everything) reads a card. Authors receive only
  what the export of step 9 releases for train groups. The coordinator reads aggregates by script.
- **Check.** A strong model labels a random 200 pages (seed 20261114, drawn by `prepare.py`)
  blind, with the same schema and the whole document as context; agreement is computed by script
  per field. A field is kept from the Sonnet pass only where agreement is nine in ten or better.

## Rows

Document row:

```json
{"schema": "ignis.docs.document_card/1", "document_sha256": "...", "title": "...",
 "doc_type": "<one of DOC_TYPES>", "investigations": ["PSI-.."],
 "abstract": "<at most 80 words: what the document is and what it says>",
 "glossary": [{"short": "PMT", "long": "photomultiplier tube"}],
 "test_id_scheme": "<how this document names tests, or null>",
 "summary_pages": [3, 4], "outcome_pages": [12, 13]}
```

Page row:

```json
{"schema": "ignis.docs.page_card/1", "document_sha256": "...", "page": 12,
 "kind": "<one of PAGE_KINDS>", "summary": "<at most 40 words>",
 "tests": ["<test ids the page speaks about, as the catalogue writes them when it can be matched, else as written>"],
 "quantities": [{"name": "flame standoff ratio", "unit": "-"}],
 "flame_type": ["<values of the closed list labels.flame_type>"],
 "studied": ["<values of labels.studied>"],
 "fire_safety_need": ["prevention" | "detection" | "suppression"],
 "reveals_test_outcome": "yes" | "no" | "unsure",
 "is_summary_or_conclusion": true | false,
 "needs_image": true | false,
 "confidence": "high" | "medium" | "low",
 "enrichment": {"concepts": ["..."], "synonyms": ["PMT = photomultiplier tube"],
                "queries": ["<3 to 6 phrasings a reader would search with>"],
                "instruments": ["..."], "materials": ["..."]}}
```

DOC_TYPES: `requirements`, `design_or_engineering`, `operations_or_procedures`, `test_matrix`,
`data_description`, `results_report`, `paper_or_presentation`, `calibration`, `other`.

PAGE_KINDS: `front_matter`, `contents_or_lists`, `requirements`, `methods`, `apparatus`,
`procedure_or_script`, `test_matrix`, `results_table`, `results_figure`, `discussion`,
`conclusions_or_summary`, `drawing_or_schematic`, `data_listing`, `references`, `administrative`,
`blank_or_unreadable`.

FLAME_TYPES (`flame_type`, the phenomenon the page is about): `spherical_diffusion_flame`,
`flat_burner_diffusion_flame`, `jet_diffusion_flame`, `coflow_jet_diffusion_flame`,
`droplet_flame`, `solid_surface_flame`, `overheat_smoke` (smoke from an overheated sample, no
flame), `none` (the page is about no combustion phenomenon: hardware, operations, lists). Several
may apply; an empty list only when the page says nothing that decides it.

STUDIED (`studied`, what the page studies or reports, when it does): `cool_flame`, `smoke_point`,
`extinction`, `electric_field`, `liftoff`, `burning_rate`, `aerosol`. Empty when none applies.

FIRE_SAFETY_NEED: `prevention` (flammability, ignition, spread, material selection), `detection`
(smoke, aerosol, detectors), `suppression` (extinction by flow, diluent, oxygen reduction,
extinguishers). Empty when the page bears on none.

`reveals_test_outcome` is `yes` when the page states or shows how an individual test ended or what
it measured (a burn time, an extinction, a diameter series, a spread rate, whether it ignited);
`unsure` whenever in doubt. Planned conditions, the apparatus and methods are not outcomes.
