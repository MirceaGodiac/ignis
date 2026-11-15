# R8 gold set

The set of documents that measures the R8 document labeller (`SourceLabel`) before its labels are
used. A person labels it; code never fills a label.

| File | What |
|---|---|
| `gold_template.jsonl` | 100 documents chosen by `ignis evidence gold-select --n 100 --seed 2026`, all label fields empty. Never edit it. |
| `labelling_sheet.html` | The same 100 documents with their file name, title, storage folder, page count, a link to the PDF and the section outline (R7b). The page lets you fill the fields and exports `gold_labels.jsonl`. It shows no model output. |
| `gold_labels.jsonl` | (to come) the labelled rows, exported from the sheet and committed by the labeller. |
| `build_sheet.py`, `sheet_template.html` | Rebuild the sheet from the template and a catalog. |

## Selection

- **Catalog:** built on 2026-11-15 from the local corpus. `import-views` of
  `data/work/views` (PDFium first, then Docling) against `data/raw/samples`. It holds 1,242 unique
  PDFs, with Docling active for 284 of them; 3 views were refused for failed pages.
- **Strata:** (storage investigation × a kind guessed from the file name). There are 69 strata in
  the catalog: 19 investigations × up to 7 kinds (`report_or_paper`, `presentation`,
  `requirements`, `operations`, `data_description`, `thesis`, `other`).
- **Coverage:** with 100 documents, every one of the 69 strata has at least one document. The 31
  second picks go to the strata that sort first as strings (PSI-10, PSI-100 … PSI-26). Per
  investigation the set has 2 (PSI-98, PSI-99) to 11 (PSI-20) documents. By kind it has:
  - `other` 30, `report_or_paper` 22, `requirements` 17;
  - `presentation` 13, `operations` 11;
  - `data_description` 5, `thesis` 2.

  That is 3,782 pages in all; the longest document has 181.
- **Document types:** the kinds are file-name guesses, not the 12 `doc_type` values. Coverage of
  `journal_article`, `conference_paper`, `engineering_document`, `data_table` and `outreach` is
  only known once the set is labelled.

## Labelling

1. Open `labelling_sheet.html` from the main checkout (the PDF links are relative:
   `../../raw/samples/<inv>/<file>`). Enter your name.
2. For each document, fill `doc_type`, `families`, `investigation_ids`, `runs`,
   `conditions_mentioned` (and `notes` for edge cases), then tick **done**. The rules are at the
   top of the sheet. They are the same rules the model is given.
3. Export and save the file as `data/labels/gold/gold_labels.jsonl`. You can import it back to
   continue later. Only rows marked done are scored.
4. `ignis evidence gold-eval --gold data/labels/gold/gold_labels.jsonl --labels <catalog dir>/source_labels.jsonl`
   prints:
   - document-type accuracy;
   - investigation-set exact match;
   - F1 for families, runs and conditions;
   - the same over resolved labels only;
   - a confusion table.
