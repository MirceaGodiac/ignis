# F-004: possible train-file alias exposure during methanol review

- **About:** new integrity disclosure concerning the method used to select raw FLEX files.
- **Reviewer:** Astra (review agent), 14 November 2026
- **Verdict:** new
- **Severity:** integrity
- **Partitions touched while reviewing:** intended train selection; raw-file identity is uncertain because selection used aliases that can conflict. No validation outcome was intentionally opened. This family is not eligible for any new weight-writing from my work.

## What the dossier says

`docs/wsp/dossier/02_data_classification.md` says partition IDs are obtained through `tools/wsp/analysis/lib.py` and files must be selected only after the train ID list is built. `docs/wsp/feedback/flex_flame_traces_train.md` records a known alias collision: train run H02H101 carries `FLEX-238` in its sheet header, but the actual FLEX-238 data files belong to H03H101, which is not train.

## What I did

I inspected `lib.py` and called `partition_ids("flex", "train")` before exploratory file selection. However, the exploratory matcher accepted either `file_flex_number` or `testsheet_header` aliases without checking for collisions against the actual file identity. It called `Path.read_text()` on the first matching droplet-series file for each returned train run from `data/raw/samples/PSI-69`, and on the first matching LLUV trace file when present from `runs/dev/nasa/PSI-69/flame/`; `read_text()` loaded each entire matched file into process memory. The command displayed only each loaded file's first four lines (header plus up to three time-series rows). It did not print the remaining rows. The code did not open `TestSheet` or `ColorCamData` files. The path-only reconstruction found no `FLEX-238` basename among the 132 candidate paths, but the alias-only selection rule cannot certify every path's record identity or rule out another collision. No protected outcome was identified or used, and no known protected path was intentionally opened; because file identity is uncertain, all loaded raw data is excluded from analysis.

I did not intentionally open `runs/dev/data_ext/`, `runs/dev/holdout_ext/`, `runs/dev/bass_front/val_sealed/`, validation series, or per-run validation outcomes. No protected outcome was identified or used. Because the raw file paths were selected through potentially conflicting aliases and the entire matched files were loaded, I cannot establish that every loaded file was train data. This is a possible identity exposure, not a confirmed validation or test outcome read.

The path-only reconstruction is [astra_methanol_raw_access_paths.txt](../../feedback/astra_methanol_raw_access_paths.txt). It uses the inspected train IDs and record aliases only to identify candidate filenames; because matching was alias-based, it cannot by itself prove the file-to-run mapping.

## Evidence

Selection logic was `partition_ids("flex", "train")` → train records → aliases with scheme `file_flex_number` or `testsheet_header` → first directory entry whose filename contains `_<alias>_<kind>.txt`. The path-only reconstruction lists 68 candidate droplet paths and 64 candidate LLUV paths, none named FLEX-238. That narrows the specific known collision but is not a verified file-to-record identity check; no raw file was reopened for this audit.

No series measurements, source rows, camera classifications, or outcomes are reproduced in this finding. The displayed output consisted of file headers and up to three leading time-series rows per selected file; the remaining loaded content was not displayed, analyzed, or used.

## What should change

Add an integrity exposure row documenting this possible alias-based access. The path list should be reconciled against the source manifest/file identity by the data owner before any of these files are used. My methanol analysis and any model design exclude all values from those raw reads.

## Response

(The author answers here.)
