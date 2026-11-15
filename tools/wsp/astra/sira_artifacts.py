"""Private offline SIRA-inspired artifacts; no retrieval, network, or scope grants.

Only a trusted custodian may call the Python ingestion API. The CLI has no author
export mode. All payloads and corpus-derived statistics remain under WORK.
"""
from __future__ import annotations

import argparse
from dataclasses import asdict, dataclass
from fractions import Fraction
import hashlib
import json
import os
from pathlib import Path
import re
import sqlite3
import sys
import unicodedata

REPO = Path(__file__).resolve().parents[3]
PRIVATE = REPO / "runs/dev/data_astra/work"
WORK = PRIVATE / "sira_artifacts"
VERSION = "astra.private.sira-artifacts/1"
UPSTREAM_COMMIT = "57f9cbbe5d0a00c91a60f14f3532c8268edd6332"
TOKENIZER = "nfkc-casefold-unicode-letter-number-mark/1"


def require(condition, code):
    if not condition:
        raise ValueError(code)


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False,
                      separators=(",", ":"), allow_nan=False).encode("utf-8")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def file_sha(path):
    value = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def strict_json(raw):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, "duplicate_json_key")
            result[key] = value
        return result

    def constant(_):
        raise ValueError("nonfinite_json")

    result = json.loads(raw, object_pairs_hook=unique, parse_constant=constant)
    canonical(result)
    return result


def jsonl(path):
    with path.open("rb") as stream:
        for line in stream:
            require(line.endswith(b"\n") and line.strip(), "incomplete_or_empty_jsonl_row")
            yield strict_json(line)


def valid_hash(value):
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) is not None


def private_path(path, root=PRIVATE):
    """Resolve before opening; paths are custody restrictions, never permissions."""
    path, root = Path(path), Path(root).resolve()
    resolved = path.resolve()
    require(resolved != root and resolved.is_relative_to(root), "outside_private_root")
    require(not any(x.casefold() in {"holdout", "holdout_ext", "val_sealed", "validation",
                                   "tests_outcomes.jsonl"} for x in resolved.parts), "sealed_path")
    return resolved


def tokens(text):
    """NFKC then casefold; L/N runs with following combining M characters.

    All other code points delimit tokens. There is no stemming, stopword list,
    diacritic removal, or sentence reset. Normalization never edits stored text.
    """
    result, current = [], []
    for char in unicodedata.normalize("NFKC", text).casefold():
        category = unicodedata.category(char)[0]
        if category in ("L", "N") or (category == "M" and current):
            current.append(char)
        elif current:
            result.append("".join(current))
            current = []
    if current:
        result.append("".join(current))
    return result


def ngrams(sequence, max_n):
    for n in range(1, min(max_n, len(sequence)) + 1):
        for start in range(len(sequence) - n + 1):
            yield " ".join(sequence[start:start + n])


def contains(sequence, phrase):
    return any(sequence[i:i + len(phrase)] == phrase
               for i in range(len(sequence) - len(phrase) + 1))


@dataclass(frozen=True)
class Config:
    upstream_commit: str = UPSTREAM_COMMIT
    max_n: int = 4
    max_df_ratio: str = "0.05"

    def identity(self):
        require(self.upstream_commit == UPSTREAM_COMMIT, "unreviewed_upstream_pin")
        require(type(self.max_n) is int and self.max_n > 0, "invalid_max_n")
        require(isinstance(self.max_df_ratio, str), "ratio_must_be_exact_string")
        try:
            ratio = Fraction(self.max_df_ratio)
        except (ValueError, ZeroDivisionError):
            raise ValueError("invalid_df_ratio") from None
        require(0 <= ratio <= 1, "invalid_df_ratio")
        return {**asdict(self), "version": VERSION, "tokenizer": TOKENIZER,
                "unicode_version": unicodedata.unidata_version,
                "code_sha256": file_sha(__file__),
                "label_records_code_sha256": file_sha(Path(__file__).with_name("label_records.py")),
                "df_population": "unique_physical_page_ids",
                "df_basis": "original_exact_contiguous_normalized_phrase",
                "injection": "retained_phrase_sliding_ngrams_1_to_max_n",
                "visibility": "custodian_private", "generated_cues_are_evidence": False}


@dataclass(frozen=True)
class VerifiedPage:
    """Host-validated v3 record and full UTF-8 text, never an authorization token.

    The caller must validate record/source/producer receipts before constructing
    this object. This builder repeats integrity, provenance and novelty checks.
    It cannot turn any input's audience, review flag, or dependencies into scope.
    """
    record: dict
    text: bytes
    text_sha256: str
    source_aliases: tuple = ()


class WriterLease:
    """OS-held lock: process death releases it, so interrupted builds can resume."""
    def __init__(self, path):
        self.stream = private_path(path, WORK).open("a+b")
        try:
            if self.stream.tell() == 0:
                self.stream.write(b"0")
                self.stream.flush()
            self.stream.seek(0)
            if os.name == "nt":
                import msvcrt
                msvcrt.locking(self.stream.fileno(), msvcrt.LK_NBLCK, 1)
            else:
                import fcntl
                fcntl.flock(self.stream.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        except OSError:
            self.stream.close()
            raise ValueError("build_already_has_writer") from None

    def close(self):
        if not self.stream.closed:
            self.stream.close()


class PrivateBuilder:
    def __init__(self, work, pins, config=Config(), *, host_check=None, synthetic_fixture=False):
        self.work = private_path(work, WORK)
        require(type(synthetic_fixture) is bool, "invalid_fixture_mode")
        require(synthetic_fixture or callable(host_check), "independent_host_qualification_required")
        self.synthetic_fixture = synthetic_fixture
        require(isinstance(pins, dict) and pins, "missing_host_input_pins")
        require(all(isinstance(k, str) and valid_hash(v) for k, v in pins.items()),
                "invalid_host_input_pins")
        self.config = config
        self.host_check = host_check
        self.identity = {"configuration": {**config.identity(), "synthetic_fixture_only": synthetic_fixture},
                         "input_pins": pins}
        self.build_id = sha(canonical(self.identity))
        self.work.mkdir(parents=True, exist_ok=True)
        self.lease = WriterLease(self.work / "writer.lock")
        self.db = None
        try:
            self._initialize_database()
        except Exception:
            self.close()
            raise

    def _initialize_database(self):
        self.db = sqlite3.connect(private_path(self.work / "state.sqlite", WORK))
        self.db.execute("PRAGMA foreign_keys=ON")
        self.db.execute("PRAGMA temp_store=FILE")
        self.db.execute("PRAGMA cache_size=-8192")
        self.db.executescript("""
            CREATE TABLE IF NOT EXISTS metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS pages (
                page_id TEXT PRIMARY KEY, record_sha256 TEXT NOT NULL, text_sha256 TEXT NOT NULL,
                document_sha256 TEXT NOT NULL, ordinal INTEGER NOT NULL, record BLOB NOT NULL,
                original BLOB NOT NULL, producer_sha256 TEXT NOT NULL,
                source_aliases BLOB NOT NULL,
                UNIQUE(document_sha256,ordinal));
            CREATE TABLE IF NOT EXISTS original_postings (
                term TEXT NOT NULL, page_id TEXT NOT NULL REFERENCES pages(page_id),
                tf INTEGER NOT NULL, PRIMARY KEY(term,page_id));
            CREATE TABLE IF NOT EXISTS proposals (
                page_id TEXT NOT NULL REFERENCES pages(page_id), cue INTEGER NOT NULL,
                ordinal INTEGER NOT NULL, raw TEXT NOT NULL, normalized TEXT NOT NULL,
                df INTEGER, reason TEXT, PRIMARY KEY(page_id,cue,ordinal));
            CREATE TABLE IF NOT EXISTS enriched_postings (
                term TEXT NOT NULL, page_id TEXT NOT NULL REFERENCES pages(page_id),
                PRIMARY KEY(term,page_id));
            CREATE TABLE IF NOT EXISTS injection_links (
                term TEXT NOT NULL, page_id TEXT NOT NULL, cue INTEGER NOT NULL, ordinal INTEGER NOT NULL,
                PRIMARY KEY(term,page_id,cue,ordinal),
                FOREIGN KEY(page_id,cue,ordinal) REFERENCES proposals(page_id,cue,ordinal));
        """)
        row = self.db.execute("SELECT value FROM metadata WHERE key='identity'").fetchone()
        if row:
            if row[0] != canonical(self.identity).decode():
                raise ValueError("changed_build_pins")
        else:
            with self.db:
                self.db.execute("INSERT INTO metadata VALUES ('identity',?)",
                                (canonical(self.identity).decode(),))
        self.frozen = self.db.execute("SELECT value FROM metadata WHERE key='frozen'").fetchone() is not None

    def close(self):
        if self.db is not None:
            self.db.close()
        self.lease.close()

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()

    def ingest(self, page):
        import label_records as labels
        record, raw = page.record, page.text
        require(record.get("origin") in ("synthetic_fixture", "source_evidence"), "invalid_origin")
        require(not self.synthetic_fixture or record["origin"] == "synthetic_fixture", "source_page_in_fixture_build")
        if record["origin"] == "source_evidence":
            require(callable(self.host_check), "independent_host_qualification_required")
            require({"qualification_review_sha256", "cards_sha256", "census_sha256",
                     "runtime_receipts_sha256", "document_registry_sha256"} <= set(self.identity["input_pins"]),
                    "missing_qualification_pins")
            require(bool(page.source_aliases), "source_filename_join_required")
        if callable(self.host_check):
            # The callback is trusted host code, never loaded from a JSON file or
            # prompt. Its checks must include free-form cue grounding/error review.
            require(self.host_check(page, self.identity["input_pins"]) is True,
                    "host_qualification_not_accepted")
        require(isinstance(raw, bytes) and sha(raw) == page.text_sha256, "page_text_hash_mismatch")
        text = raw.decode("utf-8", errors="strict")
        require(record.get("envelope_version") == "astra.private.labels/3" and record.get("kind") == "page",
                "v3_page_required")
        require(labels.content_hash(record) == record.get("content_sha256"), "page_record_hash_mismatch")
        body, producer = record["body"], record["producer"]
        source, ordinal, page_id = body["document_sha256"], body["page_number"], body["page_id"]
        require(valid_hash(source) and type(ordinal) is int and ordinal > 0, "invalid_page_identity")
        require(page_id == f"sha256:{source}:page:{ordinal:06d}", "noncanonical_page_id")
        aliases = sorted({canonical(alias) for alias in page.source_aliases})
        for alias in page.source_aliases:
            require(alias.get("document_sha256") == source and isinstance(alias.get("file_name"), str)
                    and bool(alias["file_name"]) and isinstance(alias.get("source_identity"), str)
                    and bool(alias["source_identity"]), "invalid_source_alias")
        alias_bytes = canonical([strict_json(alias) for alias in aliases])
        enrichment = body["search_enrichment"]
        labels.validate_enrichment(enrichment, producer)
        require(enrichment["status"] != "unresolved", "unresolved_enrichment_not_complete")
        for key in ("prompt_sha256", "tool_sha256", "config_sha256"):
            require(valid_hash(producer.get(key)), "missing_producer_hash")
        require(all(isinstance(record.get(key), list) for key in ("dependencies", "test_dependencies")),
                "missing_dependency_metadata")
        locators = list(record["locators"])
        locators.extend(labels.validate_enrichment(enrichment))
        locators.extend(loc for label in record.get("labels", []) for loc in label["locators"])
        locators.extend(item["locator"] for field in ("quantity_evidence", "condition_evidence")
                        for item in body.get(field, []))
        require(all(loc["source_id"] in record["dependencies"] for loc in locators),
                "protected_dependency_omitted")
        require(any(loc["source_sha256"] == source and loc["page"] == ordinal for loc in locators),
                "page_locator_mismatch")
        for loc in locators:
            selector = loc["selector"]
            if loc["source_sha256"] == source and loc["page"] == ordinal and selector["kind"] == "text":
                require(selector["extraction_sha256"] == page.text_sha256
                        and 0 <= selector["start"] < selector["end"] <= len(text)
                        and text[selector["start"]:selector["end"]] == selector["quote"],
                        "source_text_locator_mismatch")
        if "definition_schema_sha256" in self.identity["input_pins"]:
            require(record["schema_sha256"] == self.identity["input_pins"]["definition_schema_sha256"],
                    "definition_schema_pin_mismatch")
        prior = self.db.execute("SELECT record_sha256,text_sha256,source_aliases FROM pages WHERE page_id=?", (page_id,)).fetchone()
        if prior:
            require(prior == (record["content_sha256"], page.text_sha256, alias_bytes), "conflicting_duplicate_page")
            return False
        require(not self.frozen, "immutable_completed_build")
        sequence = tokens(text)
        with self.db:
            self.db.execute("INSERT INTO pages VALUES (?,?,?,?,?,?,?,?,?)",
                            (page_id, record["content_sha256"], page.text_sha256, source, ordinal,
                             canonical(record), raw, sha(canonical(producer)), alias_bytes))
            # One page at a time, SQLite handles corpus size and spill-to-disk.
            for term in ngrams(sequence, self.config.max_n):
                self.db.execute("INSERT INTO original_postings VALUES (?,?,1) "
                                "ON CONFLICT(term,page_id) DO UPDATE SET tf=tf+1", (term, page_id))
            for cue_index, cue in enumerate(enrichment["retrieval_cues"]):
                for term_index, term in enumerate(cue["new_terms"]):
                    norm = " ".join(tokens(term))
                    self.db.execute("INSERT INTO proposals VALUES (?,?,?,?,?,NULL,NULL)",
                                    (page_id, cue_index, term_index, term, norm))
        return True

    def original_df(self, normalized):
        phrase = normalized.split()
        if not phrase:
            return 0
        if len(phrase) <= self.config.max_n:
            return self.db.execute("SELECT COUNT(*) FROM original_postings WHERE term=?", (normalized,)).fetchone()[0]
        # Long phrases are tested exactly, never as an intersection of word bags.
        prefix = " ".join(phrase[:self.config.max_n])
        rows = self.db.execute("SELECT p.original FROM original_postings o JOIN pages p USING(page_id) "
                               "WHERE o.term=?", (prefix,))
        return sum(contains(tokens(bytes(row[0]).decode("utf-8")), phrase) for row in rows)

    def finalize(self, input_rows):
        count = self.db.execute("SELECT COUNT(*) FROM pages").fetchone()[0]
        require(type(input_rows) is int and input_rows >= count and count > 0, "incomplete_page_stream")
        previous = self.db.execute("SELECT value FROM metadata WHERE key='input_rows'").fetchone()
        if self.frozen:
            require(previous is not None and int(previous[0]) == input_rows, "changed_stream_cardinality")
            return
        threshold = Fraction(self.config.max_df_ratio) * count
        with self.db:
            self.db.execute("DELETE FROM injection_links")
            self.db.execute("DELETE FROM enriched_postings")
            self.db.execute("CREATE TEMP TABLE IF NOT EXISTS seen_terms "
                            "(page_id TEXT, term TEXT, PRIMARY KEY(page_id,term))")
            self.db.execute("DELETE FROM seen_terms")
            proposals = self.db.execute("SELECT page_id,cue,ordinal,normalized FROM proposals ORDER BY page_id,cue,ordinal")
            for page_id, cue, ordinal, term in proposals:
                df = self.original_df(term)
                own = self.db.execute("SELECT original FROM pages WHERE page_id=?", (page_id,)).fetchone()[0]
                duplicate = self.db.execute("INSERT OR IGNORE INTO seen_terms VALUES (?,?)", (page_id, term)).rowcount == 0
                if not term:
                    reason = "rejected_empty_normalization"
                elif duplicate:
                    reason = "rejected_duplicate_normalized_proposal"
                elif contains(tokens(bytes(own).decode("utf-8")), term.split()):
                    reason = "rejected_already_in_source"
                elif df > threshold:
                    reason = "rejected_common_original_phrase"
                else:
                    reason = "retained_absent_original_corpus" if df == 0 else "retained_rare_original_phrase"
                    for atom in ngrams(term.split(), self.config.max_n):
                        self.db.execute("INSERT OR IGNORE INTO enriched_postings VALUES (?,?)", (atom, page_id))
                        self.db.execute("INSERT OR IGNORE INTO injection_links VALUES (?,?,?,?)", (atom, page_id, cue, ordinal))
                self.db.execute("UPDATE proposals SET df=?,reason=? WHERE page_id=? AND cue=? AND ordinal=?",
                                (df, reason, page_id, cue, ordinal))
            self.db.execute("INSERT INTO metadata VALUES ('input_rows',?)", (str(input_rows),))
            self.db.execute("INSERT INTO metadata VALUES ('frozen','true')")
        self.frozen = True

    def _rows(self, name):
        if name == "pages":
            for page_id, record, raw, text_hash, producer_hash, aliases in self.db.execute(
                    "SELECT page_id,record,original,text_sha256,producer_sha256,source_aliases FROM pages ORDER BY page_id"):
                yield {"page_id": page_id, "record": json.loads(record), "original_text": bytes(raw).decode("utf-8"),
                       "original_utf8_sha256": text_hash, "producer_sha256": producer_hash,
                       "source_aliases": strict_json(aliases),
                       "visibility": "custodian_private"}
        elif name == "proposals":
            for page_id, cue, ordinal, raw, normalized, df, reason, record_hash, source_hash, producer_hash in self.db.execute(
                    "SELECT q.*,p.record_sha256,p.document_sha256,p.producer_sha256 FROM proposals q "
                    "JOIN pages p USING(page_id) ORDER BY page_id,cue,ordinal"):
                yield dict(page_id=page_id, cue_index=cue, term_index=ordinal, raw=raw, normalized=normalized,
                           original_df=df, decision=reason, record_sha256=record_hash,
                           document_sha256=source_hash, producer_sha256=producer_hash,
                           role="generated_retrieval_cue", evidence=False)
        elif name in ("original_postings", "enriched_postings"):
            for row in self.db.execute(f"SELECT * FROM {name} ORDER BY term,page_id"):
                yield dict(zip(("term", "page_id", "tf") if name == "original_postings" else ("term", "page_id"), row))
        elif name == "injection_links":
            for row in self.db.execute("SELECT * FROM injection_links ORDER BY term,page_id,cue,ordinal"):
                yield dict(zip(("term", "page_id", "cue_index", "term_index"), row))
        elif name == "document_frequencies":
            for term, original_df, enriched_df in self.db.execute("""
                WITH combined AS (
                    SELECT term,page_id,1 original FROM original_postings UNION ALL
                    SELECT term,page_id,0 original FROM enriched_postings)
                SELECT term,COUNT(DISTINCT CASE WHEN original=1 THEN page_id END),COUNT(DISTINCT page_id)
                FROM combined GROUP BY term ORDER BY term
            """):
                yield dict(term=term, original_df=original_df, enriched_df=enriched_df,
                           visibility="custodian_private")

    def export(self):
        require(self.frozen, "unfinished_build")
        target = private_path(self.work / "exports" / self.build_id, WORK)
        target.mkdir(parents=True, exist_ok=True)
        existing_manifest = private_path(target / "manifest.json", WORK)
        if existing_manifest.exists():
            return verify_export(target)
        files = {}
        for name in ("pages", "proposals", "original_postings", "enriched_postings", "injection_links", "document_frequencies"):
            path = private_path(target / (name + ".jsonl"), WORK)
            temporary = private_path(path.with_suffix(".jsonl.partial"), WORK)
            checksum, count, size = hashlib.sha256(), 0, 0
            with temporary.open("wb") as stream:
                for row in self._rows(name):
                    data = canonical(row) + b"\n"
                    stream.write(data)
                    checksum.update(data)
                    count += 1
                    size += len(data)
                stream.flush()
                os.fsync(stream.fileno())
            require(not path.exists() or file_sha(path) == checksum.hexdigest(), "immutable_export_conflict")
            if path.exists():
                temporary.unlink()
            else:
                temporary.replace(path)
            files[path.name] = dict(sha256=checksum.hexdigest(), records=count, bytes=size)
        input_rows = int(self.db.execute("SELECT value FROM metadata WHERE key='input_rows'").fetchone()[0])
        manifest = {"schema": VERSION, "build_id": self.build_id, **self.identity,
                    "visibility": "custodian_private", "scope_granted": False,
                    "coverage": "complete_supplied_card_stream_only",
                    "complete": True, "input_rows": input_rows,
                    "duplicate_page_rows": input_rows - files["pages.jsonl"]["records"], "files": files}
        data = canonical(manifest) + b"\n"
        temporary = private_path(target / "manifest.json.partial", WORK)
        temporary.write_bytes(data)
        temporary.replace(existing_manifest)
        return operational_summary(manifest, sha(data))


def build_terra_private(*, work, cards_path, census_path, documents_path, text_root,
                       runtime_receipts_path, qualification_review_path,
                       definitions, pins, host_check, config=Config()):
    """Trusted-host adapter; no CLI/JSON mechanism can supply host_check.

    host_check must independently establish the qualification review, accepting
    each page's free-form grounding/error review and source/dependency integrity.
    Closed-label agreement alone is insufficient. It grants no ExecutionScope.
    The host must prevent concurrent modification of input snapshots.
    """
    import label_records as labels
    require(callable(host_check), "independent_host_qualification_required")
    inputs = {"cards_sha256": private_path(cards_path), "census_sha256": private_path(census_path),
              "document_registry_sha256": private_path(documents_path),
              "runtime_receipts_sha256": private_path(runtime_receipts_path),
              "qualification_review_sha256": private_path(qualification_review_path)}
    for key, path in inputs.items():
        require(pins.get(key) == file_sha(path), "input_snapshot_pin_mismatch")
    require(pins.get("definition_schema_sha256") == definitions.schema_sha256
            and pins.get("definition_vocabulary_sha256") == definitions.vocabulary_sha256,
            "definition_pin_mismatch")
    text_root = private_path(Path(text_root) / ".anchor").parent
    receipts = strict_json(inputs["runtime_receipts_sha256"].read_bytes())
    with PrivateBuilder(work, pins, config, host_check=host_check) as builder:
        builder.db.execute("CREATE TEMP TABLE census (page_id TEXT PRIMARY KEY, payload BLOB NOT NULL)")
        builder.db.execute("CREATE TEMP TABLE aliases (document_sha256 TEXT, alias BLOB, PRIMARY KEY(document_sha256,alias))")
        for row in jsonl(inputs["document_registry_sha256"]):
            if row["status"] != "complete":
                continue
            require(row["receipt_state"] in ("verified_download", "verified_existing"), "unverified_document_registry")
            source = row["source"]
            alias = {"document_sha256": row["document_sha256"], "file_name": source["file_name"],
                     "source_identity": source["identity"], "investigation": source["investigation"],
                     "psi_file_id": source["psi_file_id"], "file_version": source["file_version"]}
            builder.db.execute("INSERT OR IGNORE INTO aliases VALUES (?,?)", (row["document_sha256"], canonical(alias)))
        for row in jsonl(inputs["census_sha256"]):
            encoded = canonical(row)
            prior = builder.db.execute("SELECT payload FROM census WHERE page_id=?", (row["page_id"],)).fetchone()
            require(prior is None or bytes(prior[0]) == encoded, "conflicting_census_page")
            builder.db.execute("INSERT OR IGNORE INTO census VALUES (?,?)", (row["page_id"], encoded))
        builder.db.commit()
        count = 0
        for record in jsonl(inputs["cards_sha256"]):
            labels.validate_record(record, definitions, receipts)
            require(record["kind"] == "page", "nonpage_in_card_stream")
            row = builder.db.execute("SELECT payload FROM census WHERE page_id=?",
                                     (record["body"]["page_id"],)).fetchone()
            require(row is not None, "card_missing_census_page")
            metadata = strict_json(row[0])
            require(metadata["document_sha256"] == record["body"]["document_sha256"]
                    and metadata["page_ordinal"] == record["body"]["page_number"], "census_identity_mismatch")
            require(metadata["text_status"] != "extraction_error", "original_extraction_failed")
            relative = Path(metadata["text_path"])
            require(not relative.is_absolute() and ".." not in relative.parts, "unsafe_text_reference")
            source = private_path(text_root / relative, text_root)
            raw = source.read_bytes()
            require(sha(raw) == metadata["text_sha256"], "page_text_hash_mismatch")
            aliases = tuple(strict_json(row[0]) for row in builder.db.execute(
                "SELECT alias FROM aliases WHERE document_sha256=? ORDER BY alias", (metadata["document_sha256"],)))
            require(bool(aliases), "source_filename_join_required")
            builder.ingest(VerifiedPage(record, raw, metadata["text_sha256"], aliases))
            count += 1
        for key, path in inputs.items():
            require(pins[key] == file_sha(path), "input_snapshot_changed_during_build")
        builder.finalize(count)
        return builder.export()


def operational_summary(manifest, manifest_sha256):
    """No terms, paths, source identifiers, labels, or per-term DF escape here."""
    return {"build_id": manifest["build_id"], "manifest_sha256": manifest_sha256,
            "pages": manifest["files"]["pages.jsonl"]["records"],
            "proposals": manifest["files"]["proposals.jsonl"]["records"],
            "duplicate_page_rows": manifest["duplicate_page_rows"],
            "export_bytes": sum(x["bytes"] for x in manifest["files"].values())}


def verify_export(path):
    path = private_path(path, WORK)
    raw = private_path(path / "manifest.json", WORK).read_bytes()
    manifest = strict_json(raw)
    require(manifest["schema"] == VERSION and manifest["visibility"] == "custodian_private"
            and manifest["scope_granted"] is False and manifest["complete"] is True, "invalid_export_manifest")
    identity = {key: manifest[key] for key in ("configuration", "input_pins")}
    require(sha(canonical(identity)) == manifest["build_id"] == path.name, "export_identity_mismatch")
    expected = {name + ".jsonl" for name in
                ("pages", "proposals", "original_postings", "enriched_postings", "injection_links", "document_frequencies")}
    require(set(manifest["files"]) == expected, "invalid_export_file_set")
    for name, info in manifest["files"].items():
        payload = private_path(path / name, WORK)
        require(payload.stat().st_size == info["bytes"] and file_sha(payload) == info["sha256"],
                "export_checksum_mismatch")
    return operational_summary(manifest, sha(raw))


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verify-private-export", type=Path)
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args(argv)
    if not args.execute:
        print(json.dumps({"execute_required": True, "network_calls": 0}))
        return 0
    try:
        require(args.verify_private_export is not None, "private_export_path_required")
        print(json.dumps(verify_export(args.verify_private_export), sort_keys=True))
        return 0
    except Exception:
        # Even parser, SQLite and filesystem exceptions can contain protected terms.
        print(json.dumps({"status": "failed", "details": "custodian_inspection_required"}))
        return 1


if __name__ == "__main__":
    sys.exit(main())
