-- IGNIS evidence catalog, layout version 2 (crates/ignis-evidence, compiled in by catalog.rs; read by
-- crates/ignis-server). All JSON columns hold UTF-8 JSON text. Pages are 1-based. Char spans index the page
-- text defined in ignis_evidence::pdf::page_text (concatenated MuPDF text blocks, counted in Unicode scalar
-- values); NULL spans for table chunks.
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL                       -- catalog_version, created_at, parser_version, ...
);

CREATE TABLE IF NOT EXISTS publications (
  id                TEXT PRIMARY KEY,       -- 'pub-<sha256[:16]>'
  source_sha256     TEXT NOT NULL UNIQUE,   -- bytes of the PDF
  title             TEXT,
  authors_json      TEXT NOT NULL DEFAULT '[]',
  year              INTEGER,
  ntrs_id           TEXT,
  doi               TEXT,
  canonical_url     TEXT,
  investigation_ids_json TEXT NOT NULL DEFAULT '[]',
  families_json     TEXT NOT NULL DEFAULT '[]',   -- evidence families: 'flex','bass','acme',...
  file_name         TEXT,
  n_pages           INTEGER NOT NULL,
  rights_status     TEXT NOT NULL,          -- verified | unverified | restricted | noncommercial
  license           TEXT,
  describes_runs_json TEXT NOT NULL DEFAULT '[]',
  outcome_bearing   INTEGER NOT NULL DEFAULT 1,
  ingested_at       TEXT NOT NULL,
  parser_version    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS chunks (
  rowid             INTEGER PRIMARY KEY,    -- FTS5 content_rowid
  id                TEXT NOT NULL UNIQUE,   -- '<publication_id>:p<page>:<ordinal>'
  publication_id    TEXT NOT NULL REFERENCES publications(id) ON DELETE CASCADE,
  ordinal           INTEGER NOT NULL,
  kind              TEXT NOT NULL,          -- text | table | caption
  page              INTEGER NOT NULL,
  char_start        INTEGER,
  char_end          INTEGER,
  section           TEXT,
  text              TEXT NOT NULL,          -- exact page_text[char_start:char_end]
  text_norm         TEXT NOT NULL,          -- NFKC(text): what FTS indexes (ligatures such as U+FB02 split)
  context_json      TEXT NOT NULL DEFAULT '{}',  -- captions/tables on the same page, table bbox
  locator_json      TEXT NOT NULL,          -- SourceLocator (asset_sha256, path, page, char_start, char_end)
  describes_runs_json TEXT NOT NULL DEFAULT '[]',
  outcome_bearing   INTEGER NOT NULL DEFAULT 1,
  evidence_class    TEXT NOT NULL,
  continues_runs_json TEXT NOT NULL DEFAULT '[]'  -- last run named before the chunk (withhold-only)
);
CREATE INDEX IF NOT EXISTS chunks_pub_page ON chunks(publication_id, page);

-- Parse view: the exact text of every physical page (1-based) of a publication, as defined by the
-- publication's parser_version. Chunk char spans index these texts. The view id is
-- 'pv-<sha16>@<parser_version>' (ignis_evidence::records::parse_view_id). Pages without text are ''.
-- Several views of one publication coexist (catalog v3): re-parsing with another parser adds a view
-- and makes it active (`publications.parser_version`); older views stay readable, so citations made
-- against them still resolve. Search covers the active view only.
CREATE TABLE IF NOT EXISTS pages (
  publication_id    TEXT NOT NULL REFERENCES publications(id) ON DELETE CASCADE,
  parse_view_id     TEXT NOT NULL,
  page              INTEGER NOT NULL,
  text              TEXT NOT NULL,
  PRIMARY KEY (publication_id, parse_view_id, page)
);

-- Where this host keeps the bytes of an asset (a cache hint, not an identity: readers re-hash).
CREATE TABLE IF NOT EXISTS asset_locations (
  sha256            TEXT PRIMARY KEY,
  rel_path          TEXT NOT NULL           -- relative to the working directory of the ingest
);

-- External-content FTS5 index over chunks; kept in sync by triggers.
CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
  text, section, title UNINDEXED,
  content='chunks_fts_src', content_rowid='rowid',
  tokenize='porter unicode61 remove_diacritics 2'
);
-- View providing the FTS columns (title from the publication) for external content reads.
CREATE VIEW IF NOT EXISTS chunks_fts_src AS
  SELECT c.rowid AS rowid, c.text_norm AS text, c.section AS section, p.title AS title
  FROM chunks c JOIN publications p ON p.id = c.publication_id;

CREATE TRIGGER IF NOT EXISTS chunks_ai AFTER INSERT ON chunks BEGIN
  INSERT INTO chunks_fts(rowid, text, section, title)
  VALUES (new.rowid, new.text_norm, new.section, (SELECT title FROM publications WHERE id = new.publication_id));
END;
CREATE TRIGGER IF NOT EXISTS chunks_ad AFTER DELETE ON chunks BEGIN
  INSERT INTO chunks_fts(chunks_fts, rowid, text, section, title)
  VALUES ('delete', old.rowid, old.text_norm, old.section, (SELECT title FROM publications WHERE id = old.publication_id));
END;

CREATE TABLE IF NOT EXISTS findings (
  id                TEXT PRIMARY KEY,
  publication_id    TEXT NOT NULL REFERENCES publications(id) ON DELETE CASCADE,
  chunk_id          TEXT REFERENCES chunks(id) ON DELETE SET NULL,
  claim             TEXT NOT NULL,
  population        TEXT NOT NULL,
  conditions_json   TEXT NOT NULL DEFAULT '{}',
  result            TEXT NOT NULL,
  evidence_type     TEXT NOT NULL,          -- observed | author_conclusion | independent_calculation | ...
  locator_json      TEXT NOT NULL,
  describes_runs_json TEXT NOT NULL DEFAULT '[]',
  outcome_bearing   INTEGER NOT NULL DEFAULT 1,
  derivation_json   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS embedding_sets (
  id                TEXT PRIMARY KEY,       -- sha256 of vectors.f32
  model             TEXT NOT NULL,          -- 'Qwen/Qwen3-Embedding-8B'
  revision          TEXT NOT NULL,          -- 'api:<date>' (provider-served) or a resolved commit hash
  dim               INTEGER NOT NULL,
  n                 INTEGER NOT NULL,
  instruction       TEXT,                   -- query instruction template (documents: none)
  vectors_path      TEXT NOT NULL,          -- relative to the catalog directory
  ids_path          TEXT NOT NULL,
  created_at        TEXT NOT NULL,
  provider          TEXT NOT NULL DEFAULT 'local'   -- 'deepinfra'; vectors of different provider/model/dim are never mixed
);

-- Document-side expansion (LANES_V2 R12b, SIRA): per chunk, vocabulary a reader would type that the
-- text may lack (concepts, synonyms, abbreviations, phrasings, instruments, materials), generated
-- offline per page and DF-filtered (crates/ignis-evidence/src/enrich.rs). Indexed in its own FTS5
-- table with the same tokenizer, rowid = chunks.rowid, and scored as a separate column with its
-- own weight. Never shown as a snippet, quote or citation. The pack it came from is recorded in
-- meta('expansion').
CREATE VIRTUAL TABLE IF NOT EXISTS expansion_fts USING fts5(
  terms,
  tokenize='porter unicode61 remove_diacritics 2'
);
CREATE TRIGGER IF NOT EXISTS chunks_ad_expansion AFTER DELETE ON chunks BEGIN
  DELETE FROM expansion_fts WHERE rowid = old.rowid;
END;

-- Corpus statistics (the term table of SIRA, after Corpus's nationalterms): for every indexed term
-- (porter-stemmed, as FTS5 stores it), the chunks and publications holding it and its instances,
-- over the chunk text only. Rebuilt by `ignis evidence terms build`; meta('term_stats') holds the
-- totals and the catalog generation it was built on, so a stale table is never read.
CREATE TABLE IF NOT EXISTS term_stats (
  term         TEXT PRIMARY KEY,
  chunks       INTEGER NOT NULL,
  publications INTEGER NOT NULL,
  instances    INTEGER NOT NULL
);
