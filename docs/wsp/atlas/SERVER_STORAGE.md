# What the application's server would store from NASA's archive

Built by a script from NASA's own file listings of the 19 flight combustion investigations in PSI
(retrieved 14 November 2026). Nothing here was downloaded to make this list, and no file content was
read: names, categories, sizes and addresses only. NASA's listings give some files twice
(5,701 repeated rows, most of them in SLICE and CLD Flame); every file is counted once here. It is a proposal from the hand-written-models
lane for the lanes that own the corpus and the server; it changes nothing by itself.

## In one table

| What | Files | Size | Stored on the server? |
|---|---|---|---|
| Documents, tables and analysed data | 636 | 1.63 GB | yes, all |
| Sensor and settings text files of the tests | 24,482 | 12.26 GB | yes, all (or per investigation as it is taken up) |
| Images, video and archives of them | 35,446 | 13444.97 GB | **no**: they stay at NASA; the application keeps their name, size and address and links to them |
| Other archives outside Raw Data (calibration images, zipped presentations) | 236 | 12.66 GB | no |

So the server holds about 13.89 GB of NASA's 13471.52 GB,
plus the file listings themselves (about 30 MB) and what the project derives from them (search
index, labels, measurements, model releases and forecasts).

## The rule

- **Documents:** the PSI categories Experimental table, Science Documents, Reports, Presentations, Engineering Documents, Analyzed Data, Reference Data, Calibration, Investigation Metadata Files, restricted to the file types
  pdf, doc, docx, ppt, pptx, xls, xlsx, csv, txt, rtf, pxp, log, json, xml, md. The full list is [server_documents.csv](server_documents.csv), one row per
  file with investigation, category, name, size, PSI file id, version, date and address.
- **Sensor and settings text:** the PSI category Raw Data, restricted to the same file types. The
  full list is `server_sensor_text.csv.gz` (same columns).
- **Not stored:** every other file of Raw Data (image archives, video, camera formats), and the
  archives and video outside Raw Data.
- **The two stored groups are split by PSI category, not by content.** CLD Flame, FLEX-2 and SLICE
  file everything under Raw Data, so some of their documents are in the sensor-text list. Both
  lists are stored, so nothing is lost; the per-file kind is in the lane's file catalogue.
- A file can be fetched one at a time by its address, or with the data lane's command:
  `ignis sources fetch PSI-<id> --category '<category>' --pattern '<regex>' --execute`.

## By investigation

| Id | Name | Documents | MB | Sensor text files | MB | Images and video (not stored) | GB | Licence |
|---|---|---|---|---|---|---|---|---|
| PSI-69 | FLEX | 107 | 21 | 4742 | 600 | 1580 | 915.29 | CC0-1.0 |
| PSI-68 | FLEX-2 | 12 | 13 | 7947 | 524 | 2055 | 1216.96 | CC0-1.0 |
| PSI-39 | CFI | 40 | 37 | 1838 | 1193 | 933 | 717.66 | CC0-1.0 |
| PSI-159 | ACME CFI-G | 4 | 11 | 1814 | 1615 | 1935 | 2083.97 | CC0-1.0 |
| PSI-10 | ACME/Flame Design | 9 | 15 | 1733 | 1373 | 2059 | 2179.05 |  |
| PSI-23 | ACME s-Flame | 39 | 110 | 1609 | 796 | 2449 | 780.00 | CC0-1.0 |
| PSI-20 | ACME BRE | 42 | 142 | 1059 | 1156 | 1238 | 2033.98 | CC0-1.0 |
| PSI-21 | ACME CLD Flame | 1 | 0 | 1928 | 1604 | 2204 | 1996.70 | CC0-1.0 |
| PSI-22 | ACME E-FIELD Flames | 58 | 178 | 1103 | 1539 | 1387 | 947.49 | CC0-1.0 |
| PSI-107 | SPICE | 15 | 48 | 52 | 945 | 120 | 5.95 | CC0-1.0 |
| PSI-106 | SLICE | 1 | 0 | 355 | 129 | 18350 | 60.66 | CC0-1.0 |
| PSI-25 | BASS-II | 8 | 52 | 125 | 123 | 560 | 274.46 | CC0-1.0 |
| PSI-26 | BASS | 38 | 40 | 0 | 0 | 173 | 163.51 | CC0-1.0 |
| PSI-98 | SAFFIRE-I | 7 | 15 | 7 | 57 | 61 | 12.18 | CC0-1.0 |
| PSI-99 | SAFFIRE-II | 5 | 22 | 22 | 218 | 186 | 16.59 | CC0-1.0 |
| PSI-100 | SAFFIRE-III | 11 | 28 | 8 | 67 | 67 | 35.12 | CC0-1.0 |
| PSI-102 | SAME | 95 | 310 | 0 | 0 | 0 | 0.00 | CC0-1.0 |
| PSI-101 | SAME-R | 119 | 549 | 134 | 317 | 89 | 5.40 | CC0-1.0 |
| PSI-47 | DAFT / DAFT-2 | 25 | 40 | 6 | 0 | 0 | 0.00 | CC0-1.0 |
| | **Total** | **636** | **1630** | **24,482** | **12256** | **35,446** | **13444.97** | |

## Documents by PSI category

| Category | Files | MB |
|---|---|---|
| Analyzed Data | 189 | 740 |
| Presentations | 86 | 353 |
| Engineering Documents | 126 | 202 |
| Science Documents | 76 | 188 |
| Reports | 29 | 125 |
| Reference Data | 11 | 11 |
| Calibration | 98 | 10 |
| Experimental table | 21 | 0 |

By file type: pdf 314, xls 141, txt 97, xlsx 29, csv 21, docx 11, ppt 11, pptx 7, doc 4, pxp 1.

## What the server has to take care of

- **Results of protected tests.** Documents of FLEX (PSI-69), ACME CFI-G (PSI-159), BASS-II (PSI-25)
  and BASS (PSI-26) contain outcomes of tests that are held out for the evaluation of the models.
  The search service already withholds such passages from a model-authoring scope; that has to stay
  true for every document added. Users of the application see them normally.
- **The same will hold for the 15 other investigations** once their evaluation groups are frozen.
- **Sizes and versions are NASA's**, as listed on the retrieval date; a file can change at NASA
  (the listing carries a version and a date per file). Store the hash of what was fetched.
- **Licence:** as stated per investigation in the table, from NASA's metadata.
- **Today** the application's corpus manifest holds six documents and the data sets of four
  investigations; this list is what would be added.
