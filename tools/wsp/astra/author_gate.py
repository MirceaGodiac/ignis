"""Author-boundary policy: screened conditions and committed split metadata adapters.

The fixture wire format below is private test scaffolding, NOT an atlas schema.
The host adapter reads Fable's authoritative metadata, never outcome payloads at
construction. Authors must never supply identities, classifications or grants.
"""

from dataclasses import dataclass
from hashlib import sha256
import json
import math
from pathlib import Path
import re
import subprocess
from types import MappingProxyType
from uuid import uuid4


ATLAS_VERSION = "ignis.atlas.test/1"
SPLIT_VERSION = "ignis.atlas.split/1"
# Exposure policy, not a replacement vocabulary: values/types come from Fable's schema.
CONDITION_PATHS = frozenset((
    "hardware.rack", "hardware.insert", "fuel.phase", "geometry.configuration",
    "geometry.flame_orientation", "atmosphere.x_o2", "atmosphere.pressure",
    "atmosphere.temperature", "atmosphere.x_fuel_ambient", "oxidizer_stream.x_o2",
    "flow.speed", "flow.direction", "injection.x_fuel_injected",
    "injection.fuel_mass_flow", "injection.oxidizer_mass_flow",
    "injection.inert_mass_flow", "injection.total_mass_flow", "gravity", "fuel.name",
))


def _condition_path(path):
    return isinstance(path, str) and (path in CONDITION_PATHS or re.fullmatch(
        r"(?:fuel\.components\.(?:0|[1-9][0-9]*)\.(?:species|role|fraction)"
        r"|atmosphere\.diluents\.items\.(?:0|[1-9][0-9]*)\.(?:species|fraction)"
        r"|geometry\.sizes\.(?:0|[1-9][0-9]*)\.(?:dimension|value))", path) is not None)


def _species_path(path):
    return path == "fuel.name" or path.endswith(".species")


def _canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


@dataclass(frozen=True)
class ConditionsApproval:
    """Trusted-host screening receipt, never accepted from an author request.

    Host asserts identity and selected fields are pre-outcome conditions in these
    exact bytes. This class is not a cryptographic signature or partition grant.
    """

    row_sha256: str
    identity: tuple[str, str, str, str]  # atlas id, investigation id, test id, level
    fields: tuple[str, ...]
    group_id: str | None = None
    catalogue_sha256: str | None = None
    approved_species: tuple[str, ...] = ()


def conditions_approval_bytes(approvals, schema_pin, prose_pin):
    """Canonical host mapping bytes to independently pin after screening."""
    return _canonical({
        "version": "author-conditions-approvals/1",
        "schema_sha256": schema_pin.sha256, "prose_sha256": prose_pin.sha256,
        "schema_version": schema_pin.version,
        "rows": {key: {"row_sha256": value.row_sha256,
                        "identity": value.identity, "fields": value.fields,
                        "group_id": value.group_id, "catalogue_sha256": value.catalogue_sha256,
                        "approved_species": value.approved_species}
                 for key, value in approvals.items()},
    })


@dataclass(frozen=True)
class ConditionsRead:
    decision: "Decision"
    projection: dict | None = None


class AtlasConditions:
    """Host-only reader for screened conditions; no live outcome route.

    Creation reads only the two explicitly pinned metadata/code files, never rows.
    The reader callback is invoked only for a row in the approved host mapping.
    All validation errors are generic, so raw row content cannot leak in errors.
    """

    def __init__(self, schema_pin, prose_pin, approvals, approvals_pin):
        self._reason = "conditions_metadata_unavailable"
        self._approvals = {}
        self._verified_identities = {}
        try:
            if schema_pin.version != ATLAS_VERSION or prose_pin.version != ATLAS_VERSION:
                raise ValueError("unsupported schema")
            if approvals_pin.version != "author-conditions-approvals/1":
                raise ValueError("unsupported approval mapping")
            # Copy immutable receipts; changing the caller's dict cannot grant access.
            receipts = dict(approvals)
            for key, receipt in receipts.items():
                _text(key)
                if not isinstance(receipt, ConditionsApproval):
                    raise ValueError("invalid receipt")
                _hash(receipt.row_sha256)
                if type(receipt.identity) is not tuple or len(receipt.identity) != 4:
                    raise ValueError("invalid identity")
                for part in receipt.identity:
                    _text(part)
                if receipt.group_id is not None:
                    _text(receipt.group_id)
                if receipt.catalogue_sha256 is not None:
                    _hash(receipt.catalogue_sha256)
                if (type(receipt.approved_species) is not tuple
                        or len(set(receipt.approved_species)) != len(receipt.approved_species)
                        or any(not isinstance(token, str) or not re.fullmatch(r"[a-z][a-z0-9_]*", token)
                               for token in receipt.approved_species)):
                    raise ValueError("invalid host-reviewed species set")
                if (type(receipt.fields) is not tuple
                        or len(set(receipt.fields)) != len(receipt.fields)
                        or not all(_condition_path(path) for path in receipt.fields)
                        or (any(_species_path(path) for path in receipt.fields) and not receipt.approved_species)):
                    raise ValueError("unscreenable field requested")
            if digest(conditions_approval_bytes(receipts, schema_pin, prose_pin)) != approvals_pin.sha256:
                raise ValueError("approval mapping mismatch")
            root = Path(__file__).resolve().parents[3]
            source = (root / "tools/wsp/atlas/atlas_schema.py").read_bytes()
            prose = (root / "docs/wsp/atlas/SCHEMA.md").read_bytes()
            if digest(source) != schema_pin.sha256 or digest(prose) != prose_pin.sha256:
                raise ValueError("schema changed")
            # Execute the same inspected, hash-pinned metadata-only source bytes.
            # No caller-selected path or code is accepted.
            namespace = {}
            exec(compile(source, "atlas_schema.py", "exec"), namespace)
            if namespace["SCHEMA_ID"] != ATLAS_VERSION or ATLAS_VERSION.encode() not in prose:
                raise ValueError("schema identity mismatch")
            from jsonschema import Draft202012Validator
            schema = namespace["row_schema"]()
            Draft202012Validator.check_schema(schema)
            self._validator = Draft202012Validator(schema)
            self._schema = schema
            self._walk = namespace["walk_values"]
            self._problems = namespace["value_problems"]
            self._approvals = MappingProxyType(receipts)
            self._schema_pin = schema_pin
            self._prose_pin = prose_pin
            self._reason = None
        except (ValueError, TypeError, KeyError, AttributeError, OSError, ImportError):
            pass

    def _project_condition(self, row, path, receipt):
        if path.startswith("fuel.components.") and path.endswith(".fraction"):
            parent = row["fuel"]["name"]
            if parent["status"] != "resolved" or parent["flags"] or parent["alternatives"]:
                raise ValueError("component fraction has unresolved parent identity")
        value, schema = row, self._schema
        for segment in path.split("."):
            if isinstance(value, list):
                value, schema = value[int(segment)], schema["items"]
            else:
                value, schema = value[segment], schema["properties"][segment]
        # The complete row has already passed the authoritative schema, including
        # role/dimension enums. Open species identifiers additionally need exact
        # membership in the pinned host-reviewed set; a regex alone is no grant.
        if isinstance(value, str):
            if _species_path(path) and value not in receipt.approved_species:
                raise ValueError("species not reviewed")
            return {"status": "resolved", "value_si": value, "unit_si": None}
        if value["flags"] or value["alternatives"]:
            raise ValueError("condition requires further screening")
        if value["status"] == "resolved":
            if path == "fuel.name" and any(token not in receipt.approved_species for token in value["value_si"].split("+")):
                raise ValueError("fuel identity not reviewed")
            return {name: value[name] for name in ("status", "value_si", "unit_si", "basis", "scope")}
        if value["status"] != "multiple" or value["basis"] != "nominal":
            raise ValueError("only planned nominal sets may be projected")
        allowed_type = schema["properties"]["value_si"].get("type", [])
        if isinstance(allowed_type, str):
            allowed_type = [allowed_type]
        values = value["set"]["values"]
        if ("number" not in allowed_type or not values
                or any(type(number) not in (int, float)
                       or (type(number) is float and not math.isfinite(number)) for number in values)
                or (value["set"]["kind"] == "range" and (len(values) != 2 or values[0] > values[1]))):
            raise ValueError("invalid planned numeric set")
        projected = {name: value[name] for name in ("status", "value_si", "unit_si", "basis", "scope")}
        projected["set"] = {"kind": value["set"]["kind"], "values": list(values)}
        return projected

    def read(self, key, reader):
        if self._reason:
            return ConditionsRead(Decision("unavailable", self._reason))
        receipt = self._approvals.get(key)
        if receipt is None:
            return ConditionsRead(Decision("sealed", "row_not_screened"))
        raw = reader(key)
        if not isinstance(raw, bytes) or digest(raw) != receipt.row_sha256:
            return ConditionsRead(Decision("sealed", "row_identity_mismatch"))
        try:
            def finite(value):
                number = float(value)
                if not math.isfinite(number):
                    raise ValueError("nonfinite number")
                return number

            row = json.loads(raw, object_pairs_hook=_unique_object,
                             parse_float=finite, parse_constant=finite)
            if not self._validator.is_valid(row):
                raise ValueError("schema violation")
            if any(self._problems(path, value) for path, value in self._walk(row)):
                raise ValueError("status violation")
            identity = tuple(row[name] for name in ("atlas_id", "investigation_id", "test_id", "level"))
            if identity != receipt.identity or row["atlas_id"] != row["investigation_id"] + ":" + row["test_id"]:
                raise ValueError("identity mismatch")
            if receipt.group_id is not None and row["group_id"] != receipt.group_id:
                raise ValueError("group identity mismatch")
            conditions = {}
            for path in receipt.fields:
                conditions[path] = self._project_condition(row, path, receipt)
            if receipt.group_id is not None and receipt.catalogue_sha256 is not None:
                verified = (identity[1], receipt.group_id, receipt.row_sha256, receipt.catalogue_sha256)
                previous = self._verified_identities.get(identity[0])
                if previous is not None and previous != verified:
                    raise ValueError("conflicting catalogue identity")
                self._verified_identities[identity[0]] = verified
            projection = {
                "source_schema": ATLAS_VERSION, "schema_sha256": self._schema_pin.sha256,
                "row_sha256": receipt.row_sha256, "partial": True,
                "omission_reason": "only_host_approved_fields_are_projected",
                "atlas_id": identity[0], "investigation_id": identity[1],
                "test_id": identity[2], "level": identity[3], "conditions": conditions,
            }
            if receipt.group_id is not None and receipt.catalogue_sha256 is not None:
                projection.update(group_id=receipt.group_id, catalogue_sha256=receipt.catalogue_sha256)
            return ConditionsRead(Decision("allowed", "screened_conditions_only"), projection)
        except (ValueError, TypeError, KeyError, IndexError, RecursionError):
            return ConditionsRead(Decision("sealed", "row_not_safe_for_conditions_projection"))


def digest(data):
    return sha256(data).hexdigest()


def _hash(value):
    if not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value):
        raise ValueError("invalid digest")
    return value


def _text(value):
    if not isinstance(value, str) or not value.strip():
        raise ValueError("missing identity")
    return value


def _keys(value, expected):
    if not isinstance(value, dict) or set(value) != set(expected):
        raise ValueError("unexpected metadata fields")


def _strings(value):
    if not isinstance(value, list) or len(set(value)) != len(value):
        raise ValueError("invalid dependency list")
    return tuple(_text(item) for item in value)


def _unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate metadata key")
        result[key] = value
    return result


@dataclass(frozen=True)
class Pin:
    """Expected identity, supplied by the host independently of incoming data."""

    version: str
    sha256: str

    def __post_init__(self):
        _text(self.version)
        _hash(self.sha256)


@dataclass(frozen=True)
class Decision:
    state: str  # allowed, sealed, unavailable
    reason: str


@dataclass(frozen=True)
class AuthorRead:
    decision: Decision
    payload: bytes | None = None


@dataclass(frozen=True)
class _Payload:
    sha256: str
    kind: str
    screened: bool
    dependencies_complete: bool
    runs: tuple[str, ...]
    pages: tuple[str, ...]


def _payload_records(rows):
    if not isinstance(rows, dict):
        raise ValueError("invalid mapping")
    payloads = {}
    for key, row in rows.items():
        _text(key)
        _keys(row, ("sha256", "kind", "screened", "dependencies_complete", "runs", "pages"))
        if row["kind"] not in ("conditions", "public_method", "measurement", "test_label", "outcome_page", "interpretation"):
            raise ValueError("unknown content kind")
        if type(row["screened"]) is not bool or type(row["dependencies_complete"]) is not bool:
            raise ValueError("invalid screening flag")
        payloads[key] = _Payload(
            _hash(row["sha256"]), row["kind"], row["screened"],
            row["dependencies_complete"], _strings(row["runs"]), _strings(row["pages"]),
        )
    return payloads


@dataclass(frozen=True)
class BlindTask:
    """Only manifest may be sent to a measurer; host_routes stays on the host."""

    manifest: dict
    host_routes: dict


def _committed_metadata(commit, path):
    """Read fixed safe metadata/code paths from an exact Git commit, never data."""
    safe = {
        "tools/wsp/atlas/atlas_schema.py", "docs/wsp/atlas/SCHEMA.md",
        "tools/wsp/atlas/normalize_tests.py", "tools/wsp/atlas/freeze_groups.py",
    }
    if (not re.fullmatch(r"[0-9a-f]{40}", commit)
            or (path not in safe and not re.fullmatch(r"docs/wsp/atlas/splits/PSI-[0-9]+\.json", path))):
        raise ValueError("invalid committed metadata address")
    result = subprocess.run(["git", "show", f"{commit}:{path}"],
                            cwd=Path(__file__).resolve().parents[3], capture_output=True, check=True)
    return result.stdout


def _split_groups(raw, pin, investigation):
    if pin.version != SPLIT_VERSION or digest(raw) != pin.sha256:
        raise ValueError("split identity mismatch")
    doc = json.loads(raw, object_pairs_hook=_unique_object)
    _keys(doc, ("schema", "investigation_id", "acronym", "rule", "seed", "group_key",
                "catalogue_level", "strata", "frozen_on", "use", "note"))
    if doc["schema"] != SPLIT_VERSION or doc["investigation_id"] != investigation:
        raise ValueError("split version or investigation mismatch")
    for field in ("acronym", "rule", "group_key", "frozen_on", "note"):
        _text(doc[field])
    if type(doc["seed"]) is not int or not re.fullmatch(r"\d{4}-\d{2}-\d{2}", doc["frozen_on"]):
        raise ValueError("invalid freeze metadata")
    _keys(doc["use"], ("train", "validation", "test", "held"))
    if (not isinstance(doc["catalogue_level"], dict) or not doc["catalogue_level"]
            or not set(doc["catalogue_level"]) <= {"test", "test_point", "test_day", "matrix_row"}
            or any(type(n) is not int or n < 1 for n in doc["catalogue_level"].values())):
        raise ValueError("invalid catalogue counts")
    if not isinstance(doc["strata"], dict) or not doc["strata"]:
        raise ValueError("missing strata")
    groups, row_count = {}, 0
    for name, stratum in doc["strata"].items():
        _text(name)
        _keys(stratum, ("status", "groups", "rows_in_catalogue"))
        status = stratum["status"]
        if status not in ("split", "held_too_few_groups", "held_until_test_list"):
            raise ValueError("unknown split status")
        parts = ("train", "validation", "test") if status == "split" else ("held",)
        _keys(stratum["groups"], parts)
        _keys(stratum["rows_in_catalogue"], parts)
        for part, members in stratum["groups"].items():
            members = _strings(members)
            count = stratum["rows_in_catalogue"][part]
            if not members or type(count) is not int or count < len(members):
                raise ValueError("invalid group counts")
            row_count += count
            for group in members:
                if not group.startswith(investigation + ":") or group in groups:
                    raise ValueError("group reused across partitions or strata")
                groups[group] = part
    if row_count != sum(doc["catalogue_level"].values()):
        raise ValueError("inconsistent catalogue counts")
    return groups


def outcome_approval_bytes(conditions, payloads, split_pins, catalogue_pin, code_pins, commit):
    """Host screening mapping binds exact identities; no outcome payload values."""
    return _canonical({
        "version": "author-outcome-approvals/1", "commit": commit,
        "schema_sha256": conditions._schema_pin.sha256,
        "prose_sha256": conditions._prose_pin.sha256,
        "catalogue_sha256": catalogue_pin.sha256,
        "code": {name: {"version": pin.version, "sha256": pin.sha256} for name, pin in code_pins.items()},
        "splits": {name: {"version": pin.version, "sha256": pin.sha256} for name, pin in split_pins.items()},
        "identities": dict(conditions._verified_identities), "payloads": payloads,
    })


class AuthorGate:
    def __init__(self):
        self._reason = "authoritative_metadata_not_configured"
        self._runs = MappingProxyType({})
        self._payloads = MappingProxyType({})
        self._pins = MappingProxyType({})
        self._origin = "synthetic_fixture"
        self._allowed_reason = "synthetic_fixture_policy_only"
        self._is_live = False
        self._reveal_recorder = None

    @classmethod
    def live(cls):
        """Fail closed without opening schema, splits, mapping or payload files.

        Deliberately accepts no path or caller-supplied authorization override.
        """
        return cls()

    @classmethod
    def from_atlas(cls, conditions, split_metadata, split_pins, catalogue_pin,
                   code_pins, mapping, mapping_pin, commit, reveal_recorder=None):
        """Real metadata adapter. No catalogue/outcome payload files are opened.

        conditions contains only host-screened identities verified by read().
        split_metadata contains Fable's actual ignis.atlas.split/1 bytes. Every
        authority code/split byte must match both host pins and a committed blob.
        Caller-supplied request bodies must never populate these host arguments.
        A reveal recorder must durably log each decision before live bytes return.
        """
        gate = cls()
        gate._reason = "committed_atlas_authority_unavailable"
        try:
            if not isinstance(conditions, AtlasConditions) or conditions._reason:
                raise ValueError("conditions authority unavailable")
            if not re.fullmatch(r"[0-9a-f]{40}", commit):
                raise ValueError("uncommitted authority")
            if catalogue_pin.version != ATLAS_VERSION:
                raise ValueError("catalogue version mismatch")
            _keys(code_pins, ("catalogue", "freeze"))
            if code_pins["catalogue"].version != ATLAS_VERSION or code_pins["freeze"].version != SPLIT_VERSION:
                raise ValueError("producer version mismatch")
            _keys(split_metadata, split_pins)
            if not split_metadata:
                raise ValueError("missing splits")
            if mapping_pin.version != "author-outcome-approvals/1" or digest(mapping) != mapping_pin.sha256:
                raise ValueError("mapping identity mismatch")
            doc = json.loads(mapping, object_pairs_hook=_unique_object)
            expected = outcome_approval_bytes(conditions, doc["payloads"], split_pins,
                                              catalogue_pin, code_pins, commit)
            if mapping != expected:
                raise ValueError("mapping authority bindings mismatch")
            payloads = _payload_records(doc["payloads"])
            paths = {
                "tools/wsp/atlas/atlas_schema.py": conditions._schema_pin,
                "docs/wsp/atlas/SCHEMA.md": conditions._prose_pin,
                "tools/wsp/atlas/normalize_tests.py": code_pins["catalogue"],
                "tools/wsp/atlas/freeze_groups.py": code_pins["freeze"],
            }
            groups = {}
            for investigation, raw in split_metadata.items():
                if not re.fullmatch(r"PSI-[0-9]+", investigation):
                    raise ValueError("invalid investigation identity")
                groups[investigation] = _split_groups(raw, split_pins[investigation], investigation)
                paths[f"docs/wsp/atlas/splits/{investigation}.json"] = split_pins[investigation]
            for path, pin in paths.items():
                if digest(_committed_metadata(commit, path)) != pin.sha256:
                    raise ValueError("uncommitted or changed authority")
            runs = {}
            for atlas_id, (investigation, group, row_hash, catalogue_hash) in conditions._verified_identities.items():
                if catalogue_hash != catalogue_pin.sha256:
                    raise ValueError("row catalogue identity mismatch")
                # Exact join only. No case-folding, trimming, alternate ids or aliases.
                runs[atlas_id] = groups.get(investigation, {}).get(group, "held")
            gate._runs = MappingProxyType(runs)
            gate._payloads = MappingProxyType(payloads)
            gate._pins = MappingProxyType({**paths, "catalogue": catalogue_pin, "mapping": mapping_pin})
            gate._is_live = True
            gate._origin = "host_screened"
            gate._allowed_reason = "all_dependencies_in_committed_train_groups"
            gate._reveal_recorder = reveal_recorder
            gate._commit = commit
            gate._reason = None
        except (ValueError, TypeError, KeyError, AttributeError, OSError, subprocess.SubprocessError):
            pass
        return gate

    @classmethod
    def synthetic_fixture(cls, metadata, pins):
        """Exercise policy with already supplied SYNTHETIC metadata bytes only.

        This is not a loader for production atlas files. The three fixture blobs
        (schema, split, mapping) must all say origin=synthetic_fixture. Hashes and
        versions must match independent pins, and mapping binds both other blobs.
        Invalid or missing inputs return unavailable; they cannot invoke a reader.
        """
        gate = cls()
        try:
            _keys(metadata, ("schema", "split", "mapping"))
            _keys(pins, metadata)
            docs = {}
            for name, raw in metadata.items():
                pin = pins[name]
                if not isinstance(pin, Pin) or not isinstance(raw, bytes):
                    raise ValueError("invalid pinned metadata")
                if digest(raw) != pin.sha256:
                    raise ValueError("metadata hash mismatch")
                doc = json.loads(raw, object_pairs_hook=_unique_object)
                if doc.get("origin") != "synthetic_fixture" or doc.get("version") != pin.version:
                    raise ValueError("metadata origin or version mismatch")
                docs[name] = doc
            _keys(docs["schema"], ("origin", "version"))
            split = docs["split"]
            _keys(split, ("origin", "version", "frozen_by", "schema_sha256", "runs"))
            if split["frozen_by"] != "Fable" or split["schema_sha256"] != pins["schema"].sha256:
                raise ValueError("split identity mismatch")
            runs = split["runs"]
            if not isinstance(runs, dict):
                raise ValueError("invalid runs")
            for run, row in runs.items():
                _text(run)
                _keys(row, ("group", "partition"))
                _text(row["group"])
                if row["partition"] not in ("train", "validation", "test"):
                    raise ValueError("unknown partition")
            groups = {}
            for row in runs.values():
                if groups.setdefault(row["group"], row["partition"]) != row["partition"]:
                    raise ValueError("group crosses partitions")
            mapping = docs["mapping"]
            _keys(mapping, ("origin", "version", "schema_sha256", "split_sha256", "payloads"))
            if (mapping["schema_sha256"] != pins["schema"].sha256
                    or mapping["split_sha256"] != pins["split"].sha256):
                raise ValueError("mapping identity mismatch")
            payloads = _payload_records(mapping["payloads"])
            gate._runs = MappingProxyType({key: row["partition"] for key, row in runs.items()})
            gate._payloads = MappingProxyType(payloads)
            gate._pins = MappingProxyType(dict(pins))
            gate._reason = None
        except (ValueError, TypeError, KeyError, AttributeError):
            gate._reason = "synthetic_metadata_invalid_or_unpinned"
        return gate

    def decide(self, key):
        """Resolve the complete dependency graph before any payload is opened."""
        if self._reason:
            return Decision("unavailable", self._reason)

        def visit(current, active, force_train=False):
            item = self._payloads.get(current)
            if item is None or current in active:
                return None
            if not item.screened or not item.dependencies_complete:
                return None
            outcome = item.kind not in ("conditions", "public_method")
            train_only = force_train or outcome
            if outcome and not item.runs and not item.pages:
                return None
            dependencies = set(item.runs)
            for run in item.runs:
                if run not in self._runs or (train_only and self._runs[run] != "train"):
                    return None
            for page in item.pages:
                child = visit(page, active | {current}, train_only)
                if child is None:
                    return None
                dependencies.update(child)
            if outcome and not dependencies:
                return None
            return dependencies

        try:
            resolved = visit(key, set())
        except (TypeError, RecursionError):
            resolved = None
        if resolved is None:
            return Decision("sealed", "unapproved_or_unresolved_dependency")
        return Decision("allowed", self._allowed_reason)

    def read_author(self, key, reader):
        """Reader takes an opaque key; it is never called for denied metadata.

        Return the SAME bytes whose hash was checked, avoiding a second path read.
        Live adapters require a durable host reveal recorder. Staging is separate.
        """
        decision = self.decide(key)
        if self._is_live:
            if not callable(self._reveal_recorder):
                return AuthorRead(Decision("unavailable", "reveal_recorder_unavailable"))
            try:
                self._reveal_recorder({"phase": "authorization", "payload_key": key, "state": decision.state,
                                       "reason": decision.reason, "commit": self._commit,
                                       "mapping_sha256": self._pins["mapping"].sha256})
            except Exception:
                # A logging failure must not open the payload or leak an exception
                # containing internal labels/paths into the author response.
                return AuthorRead(Decision("unavailable", "reveal_record_failed"))
        if decision.state != "allowed":
            return AuthorRead(decision)
        raw = reader(key)
        if not isinstance(raw, bytes) or digest(raw) != self._payloads[key].sha256:
            return AuthorRead(Decision("sealed", "payload_identity_mismatch"))
        return AuthorRead(decision, raw)

    def blind_task(self, input_keys, method, operation):
        """Plan a blind synthetic task; never open, copy or emit source payloads.

        Input keys and run identities stay in host_routes. No free-text model,
        hypothesis, expected outcome or partition field exists in the manifest.
        The host must also screen the pinned method and strip embedded identifiers
        when staging; opaque filenames alone cannot make source bytes blind.
        """
        if self._reason:
            raise ValueError(self._reason)
        if operation not in ("measure", "annotate") or not isinstance(method, Pin):
            raise ValueError("invalid blind task")
        keys = _strings(list(input_keys))
        if not keys or any(key not in self._payloads for key in keys):
            raise ValueError("unknown blind input")
        # Version names can encode hypotheses; workers receive only the hash.
        task_id = uuid4().hex
        paths = [f"tasks/{task_id}/inputs/{i:08d}.bin" for i in range(len(keys))]
        output = f"tasks/{task_id}/output.json"
        return BlindTask(
            manifest={
                "origin": self._origin, "task_id": task_id, "operation": operation,
                "method_sha256": method.sha256,
                "inputs": [{"path": path, "sha256": self._payloads[key].sha256}
                           for path, key in zip(paths, keys)],
                "output": output,
            },
            host_routes={
                "task_id": task_id, "method_version": method.version,
                "inputs": dict(zip(paths, keys)), "output": output,
                "metadata_pins": dict(self._pins),
            },
        )
