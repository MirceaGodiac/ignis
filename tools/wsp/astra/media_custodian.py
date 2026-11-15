"""Private TRAIN-only media routing. No references, predictions or outcome series.

Standalone workflow artifact, not an IGNIS shared contract. Fetches require --execute.
Blind manifests are strict allowlists. Deletion requires verified retained evidence.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import re
import shutil
import urllib.request
import uuid
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
PRIVATE = ROOT / "runs/dev/data_astra/work/media_routing"


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def contained(root: Path, value: str | Path) -> Path:
    root = root.resolve(strict=True)
    target = (root / value).resolve()
    if target == root or root not in target.parents:
        raise ValueError("path escapes designated store")
    return target


def train_gate(route: dict) -> None:
    """Open split metadata, verify membership, then permit any media bytes."""
    split_path = contained(ROOT, route["split_path"])
    if sha256(split_path) != route["split_sha256"]:
        raise ValueError("split changed")
    split = json.loads(split_path.read_text(encoding="utf-8"))["split"]
    group = split["run_to_group"].get(route["run_id"])
    if (not group or group != route["group_id"] or group not in split["train"]
            or group in split.get("validation", []) or group in split.get("test", [])
            or route["partition"] != "train"):
        raise ValueError("explicit uncontested TRAIN membership required")
    atlas = contained(ROOT, route["atlas_path"])
    if sha256(atlas) != route["atlas_sha256"]:
        raise ValueError("condition alias metadata changed")
    matches = []
    with atlas.open(encoding="utf-8") as stream:
        for line in stream:
            row = json.loads(line)
            if row["atlas_id"] == route["run_id"]:
                matches.append(row)
    if len(matches) != 1:
        raise ValueError("ambiguous condition identity")
    row = matches[0]
    if (row["project_partition"] != "train" or row["group_id"] != group
            or {"scheme": "cir_test_point", "name": route["alias"]} not in row["aliases"]):
        raise ValueError("source alias does not match frozen train identity")
    for source in route["sources"]:
        if (source["investigation"] != "PSI-69" or source["test_id"] != route["alias"]
                or source.get("test_id_alternative") or not source["latest_version"]):
            raise ValueError("source routing unresolved")
    catalogue = contained(ROOT, route["file_catalogue_path"])
    if sha256(catalogue) != route["file_catalogue_sha256"]:
        raise ValueError("file catalogue changed")
    selected = {x["psi_file_id"]: x for x in route["sources"]}
    matched = set()
    with catalogue.open(encoding="utf-8") as stream:
        for line in stream:
            item = json.loads(line)
            if item["psi_file_id"] in selected and item == selected[item["psi_file_id"]]:
                matched.add(item["psi_file_id"])
    if matched != set(selected) or len(selected) != len(route["sources"]):
        raise ValueError("source metadata differs from pinned catalogue")


def fetch(route: dict, execute: bool = False) -> list[dict]:
    train_gate(route)
    if not execute:
        return [{"size_bytes": x["size_bytes"]} for x in route["sources"]]
    source_dir = PRIVATE / "sources"
    source_dir.mkdir(exist_ok=True)
    receipts = []
    for source in sorted(route["sources"], key=lambda x: (x["extension"] == "zip", x["file_name"])):
        if shutil.disk_usage(PRIVATE).free < 10 * 1024**3 + source["size_bytes"]:
            raise ValueError("disk reserve would be breached")
        path = contained(source_dir, source["psi_file_id"] + "." + source["extension"])
        if path.exists():
            previous = json.loads((PRIVATE / "fetch_receipts.json").read_text())
            pin = next(x for x in previous if x["path"] == str(path))
            if path.stat().st_size != pin["bytes"] or sha256(path) != pin["sha256"]:
                raise ValueError("retained source changed")
            receipts.append(pin)
            continue
        partial = path.with_suffix(path.suffix + ".part")
        if partial.exists():
            raise ValueError("unfinished fetch requires custodian review")
        count = 0
        req = urllib.request.Request(source["url"], headers={"User-Agent": "IGNIS-source-custodian/1"})
        with urllib.request.urlopen(req, timeout=120) as response, partial.open("xb") as output:
            for chunk in iter(lambda: response.read(1024 * 1024), b""):
                count += len(chunk)
                if count > source["size_bytes"]:
                    raise ValueError("source exceeded listed bytes")
                output.write(chunk)
        if count != source["size_bytes"]:
            raise ValueError("source byte count differs from catalogue")
        digest = sha256(partial)
        partial.rename(path)
        receipts.append({"path": str(path), "bytes": count, "sha256": digest,
                         "psi_file_id": source["psi_file_id"], "url": source["url"],
                         "retention": "kept pending measurement and evidence verification"})
        (PRIVATE / "fetch_receipts.json").write_text(json.dumps(receipts, indent=2), encoding="utf-8")
    return receipts


def validate_blind_manifest(manifest: dict) -> None:
    fields = {"schema", "packet_id", "method_version", "method_sha256", "instrument",
              "observable", "distance_unit", "time_axis", "calibration_status", "partial",
              "frames", "coverage", "pixel_transform", "full_source_ref"}
    if set(manifest) != fields or manifest["schema"] != "astra.blind-media/1":
        raise ValueError("blind manifest field allowlist mismatch")
    if not re.fullmatch(r"[0-9a-f]{32}", manifest["packet_id"]):
        raise ValueError("opaque packet identity required")
    if not re.fullmatch(r"media_[0-9a-f]{32}", manifest["full_source_ref"]):
        raise ValueError("opaque full-artifact reference required")
    choices = {
        "instrument": {"backlit_camera"}, "observable": {"liquid_silhouette_horizontal_vertical_extents"},
        "distance_unit": {"pixel"}, "time_axis": {"native_frame_index"},
        "calibration_status": {"dimensional_and_clock_unqualified"},
        "pixel_transform": {"none; metadata removed; lossless PNG"},
        "method_version": {"manual-silhouette-pixel-v1"},
        "coverage": {"systematic native-index sample; full source retained privately"},
    }
    for field, allowed in choices.items():
        if manifest[field] not in allowed:
            raise ValueError("unapproved instrument-only value")
    if manifest["partial"] is not True:
        raise ValueError("sampled packet must be explicitly partial")
    if not re.fullmatch(r"[0-9a-f]{64}", manifest["method_sha256"]):
        raise ValueError("method digest required")
    if not isinstance(manifest["frames"], list) or not manifest["frames"]:
        raise ValueError("nonempty frame list required")
    previous_index = -1
    for frame in manifest["frames"]:
        if set(frame) != {"path", "sha256", "frame_index", "width", "height", "bit_depth"}:
            raise ValueError("blind frame field allowlist mismatch")
        if not re.fullmatch(r"frame_[0-9]{6}\.png", frame["path"]):
            raise ValueError("unsafe or identifying frame path")
        if not re.fullmatch(r"[0-9a-f]{64}", frame["sha256"]):
            raise ValueError("frame digest required")
        if any(type(frame[k]) is not int or frame[k] < 0 for k in ("frame_index", "width", "height", "bit_depth")):
            raise ValueError("invalid geometry")
        if frame["width"] == 0 or frame["height"] == 0 or frame["bit_depth"] not in (8, 16):
            raise ValueError("unsupported image geometry")
        if frame["frame_index"] <= previous_index:
            raise ValueError("duplicate or unordered native frame index")
        previous_index = frame["frame_index"]


def verify_retention_then_delete(root: Path, source: str, source_sha256: str,
                                 evidence: list[dict], receipts: list[dict], execute=False):
    """No actual archive deletion is invoked by this task. Testable fail-closed primitive."""
    source_path = contained(root, source)
    if sha256(source_path) != source_sha256 or not evidence or len(receipts) != 2:
        raise ValueError("source identity or doubled measurement receipt missing")
    for receipt in receipts:
        if receipt.get("state") != "sealed" or receipt.get("source_sha256") != source_sha256:
            raise ValueError("measurement not sealed against this source")
    if len({x.get("measurer_id") for x in receipts}) != 2 or any(not x.get("measurer_id") for x in receipts):
        raise ValueError("independent identities required")
    for item in evidence:
        path = contained(root, item["path"])
        if path == source_path or sha256(path) != item["sha256"]:
            raise ValueError("retained evidence verification failed")
    if execute:
        source_path.unlink()
        if source_path.exists():
            raise ValueError("source deletion not verified")
    return {"verified": True, "deleted": execute}


METHOD = """# Manual silhouette observation, version manual-silhouette-pixel-v1

You are an independent instrument observer. Read only this packet. Do not search
the repository, browse, infer source conditions or inspect any other observer's
work. Declare prior familiarity with the pictured acquisition before beginning;
if familiar, stop and return only an exposure-declaration receipt.

Instrument: backlit camera, native 1024 by 1024 array, 16-bit stored grayscale
values copied losslessly from TIFF into PNG. Acquisition settings report 12-bit
camera sampling; the TIFF values occupy the 16-bit range with low four bits zero.
No established dimensional length calibration
or qualified acquisition-time map is supplied. The coordinate origin is the
upper-left pixel center (0,0); x increases right, y increases down. Use native
pixel coordinates and native frame index only. Do not infer millimetres or seconds.
For display only use the fixed mapping display = floor(stored_value / 256), clipped to
0..255, with nearest-neighbour magnification; do not normalize each frame.

Observable: projected horizontal and vertical extents of the visible liquid
silhouette. Inspect each supplied frame independently. Mark left, right, top and
bottom visible liquid boundaries and an inner/outer ambiguity interval for every
edge. Report the corresponding width/height intervals in pixels. Report every
plausible candidate separately if identity is ambiguous. A support, needle,
shadow or obscured edge is not a reconstructed liquid boundary. If an edge cannot
be distinguished, return null for that edge and its dependent extent, with reason
occluded, ambiguous, out_of_field or unavailable. Do not fit circles, interpolate
missing perimeters, force symmetry or temporal monotonicity, or infer a zero from
non-detection. Pixel ambiguity intervals are operator bounds, not a claimed
confidence level; dimensional scale and clock systematic uncertainty are unknown.

Retain per-frame coordinates, ambiguity bounds, evidence PNG names, input hashes,
method hash, actual tool/runtime identifiers when exposed, and every intervention.
Write measurements.json and an evidence overlay per measured frame under this
packet's output/ directory. The submission must declare its partial frame-index
coverage and unqualified pixel-only status. Include an exposure declaration and
UTC completion timestamp. Preserve originals. Never print coordinates or scientific
outcomes into chat; return only completion status and opaque output path.

The supplied frames form a systematic native-index sample. Observations between
these frames are not implied. Manual point/area selection follows the instrument
procedure concept in NASA Spotlight-8 (NASA/TM-2006-214084, pp. 3-7); no reference
values or accuracy claim are supplied. No external reading is needed or permitted.
"""


def stage_samples(route: dict) -> dict:
    """Prepare candidates for custodian overlay review; never release automatically."""
    import io
    import numpy as np
    from PIL import Image
    import PIL

    train_gate(route)
    receipts = json.loads((PRIVATE / "fetch_receipts.json").read_text())
    source = next(x for x in route["sources"] if x["extension"] == "zip")
    receipt = next(x for x in receipts if x["psi_file_id"] == source["psi_file_id"])
    archive = contained(PRIVATE, receipt["path"])
    if archive.stat().st_size != source["size_bytes"] or sha256(archive) != receipt["sha256"]:
        raise ValueError("archive pin verification failed")
    candidate = PRIVATE / "blind_candidates"
    candidate.mkdir(exist_ok=False)
    maps = []
    with zipfile.ZipFile(archive) as zip_source:
        index_source = next(x for x in route["sources"] if x["file_name"].endswith(route["alias"] + "A.txt"))
        index_receipt = next(x for x in receipts if x["psi_file_id"] == index_source["psi_file_id"])
        embedded_index = zip_source.read(route["alias"] + "A.txt")
        if hashlib.sha256(embedded_index).hexdigest() != index_receipt["sha256"]:
            raise ValueError("archive index differs from pinned instrument index")
        members = []
        for info in zip_source.infolist():
            if info.is_dir():
                continue
            if "/" in info.filename or "\\" in info.filename or ":" in info.filename:
                raise ValueError("unexpected archive member path")
            match = re.fullmatch(re.escape(route["alias"]) + r"A_([0-9]+)\.tif", info.filename)
            if match:
                members.append((int(match[1]), info))
            elif info.filename != route["alias"] + "A.txt":
                raise ValueError("unexpected archive content")
        members.sort(key=lambda x: x[0])
        if not members or len({x[0] for x in members}) != len(members):
            raise ValueError("missing or duplicate native frame indices")
        # Initial manual workload: eleven evenly spaced native-order positions.
        # This is a partial first packet, never a claimed whole-source measurement.
        selected = sorted({round(i * (len(members) - 1) / 10) for i in range(11)})
        frames = []
        for seq, pos in enumerate(selected):
            frame_index, info = members[pos]
            raw = zip_source.read(info)  # zipfile verifies member CRC
            with Image.open(io.BytesIO(raw)) as im:
                values = np.asarray(im)
                if values.shape != (1024, 1024) or values.dtype != np.uint16 or np.any(values % 16):
                    raise ValueError("source pixels do not match instrument settings")
                image_path = candidate / f"frame_{seq:06d}.png"
                Image.fromarray(values).save(image_path)
                with Image.open(image_path) as decoded:
                    if not np.array_equal(values, np.asarray(decoded)):
                        raise ValueError("lossless conversion failed")
            frames.append({"path": image_path.name, "sha256": sha256(image_path),
                           "frame_index": frame_index, "width": 1024, "height": 1024, "bit_depth": 16})
            maps.append({"path": image_path.name, "archive_member": info.filename,
                         "member_sha256": hashlib.sha256(raw).hexdigest(), "frame_index": frame_index})
    (candidate / "METHOD.md").write_text(METHOD, encoding="utf-8")
    manifest = {"schema": "astra.blind-media/1", "packet_id": uuid.uuid4().hex,
                "full_source_ref": "media_" + uuid.uuid4().hex,
                "method_version": "manual-silhouette-pixel-v1", "method_sha256": sha256(candidate / "METHOD.md"),
                "instrument": "backlit_camera", "observable": "liquid_silhouette_horizontal_vertical_extents",
                "distance_unit": "pixel", "time_axis": "native_frame_index",
                "calibration_status": "dimensional_and_clock_unqualified", "partial": True,
                "coverage": "systematic native-index sample; full source retained privately",
                "pixel_transform": "none; metadata removed; lossless PNG", "frames": frames}
    validate_blind_manifest(manifest)
    (candidate / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    (PRIVATE / "sample_routing.json").write_text(json.dumps({"archive_sha256": receipt["sha256"],
        "full_source_ref": manifest["full_source_ref"], "archive_path": str(archive),
        "native_frame_count": len(members), "selection_rule": "round(i*(N-1)/10), i=0..10, native order",
        "frames": maps, "status": "awaiting custodian overlay review"}, indent=2), encoding="utf-8")
    (PRIVATE / "processing_receipt.json").write_text(json.dumps({
        "created_utc": datetime.now(timezone.utc).isoformat(),
        "code_sha256": sha256(Path(__file__)), "numpy_version": np.__version__,
        "pillow_version": PIL.__version__, "archive_sha256": receipt["sha256"],
        "method_sha256": manifest["method_sha256"],
        "conversion": "uint16 array copied to new lossless PNG; all decoded samples equality-checked",
        "metadata": "source TIFF tags not copied", "geometric_transform": "none",
        "intensity_transform_on_evidence": "none", "source_retention": "kept"}, indent=2), encoding="utf-8")
    return {"candidate_path": str(candidate), "frame_count": len(frames), "release": "pending_overlay_review"}


def release_reviewed_samples(route: dict) -> dict:
    """Release only the exact reviewed PNGs; the review remains custodian-private."""
    train_gate(route)
    candidate = PRIVATE / "blind_candidates"
    manifest = json.loads((candidate / "manifest.json").read_text())
    validate_blind_manifest(manifest)
    review = json.loads((PRIVATE / "overlay_review.json").read_text())
    if (review["state"] != "approved" or review["condition_overlays_present"] is not False
            or review["reviewed_sha256"] != [x["sha256"] for x in manifest["frames"]]):
        raise ValueError("overlay review incomplete or mismatched")
    if sha256(candidate / "METHOD.md") != manifest["method_sha256"]:
        raise ValueError("method changed after staging")
    for frame in manifest["frames"]:
        if sha256(contained(candidate, frame["path"])) != frame["sha256"]:
            raise ValueError("reviewed frame changed")
    blind_root = ROOT / "runs/dev/data_astra/work/blind_tasks"
    blind_root.mkdir(exist_ok=True)
    packets = []
    for _ in range(2):
        packet_id = uuid.uuid4().hex
        destination = blind_root / packet_id
        destination.mkdir()
        (destination / "output").mkdir()
        for frame in manifest["frames"]:
            shutil.copyfile(candidate / frame["path"], destination / frame["path"])
        shutil.copyfile(candidate / "METHOD.md", destination / "METHOD.md")
        delivered = dict(manifest, packet_id=packet_id, full_source_ref="media_" + uuid.uuid4().hex)
        validate_blind_manifest(delivered)
        (destination / "manifest.json").write_text(json.dumps(delivered, indent=2), encoding="utf-8")
        packets.append({"packet_id": packet_id, "path": str(destination),
                        "full_source_ref": delivered["full_source_ref"],
                        "manifest_sha256": sha256(destination / "manifest.json")})
    receipt = {"state": "ready_for_independent_blind_measurement", "packets": packets,
               "qualification": "pixel_frame_index_only; no dimensional calibration", "sources_deleted": False}
    (PRIVATE / "release_receipt.json").write_text(json.dumps(receipt, indent=2), encoding="utf-8")
    (PRIVATE / "full_artifact_aliases.json").write_text(json.dumps({
        "source": json.loads((PRIVATE / "sample_routing.json").read_text())["archive_path"],
        "aliases": [x["full_source_ref"] for x in packets]}, indent=2), encoding="utf-8")
    return receipt


def native_inventory(route: dict) -> dict:
    """Complete member inventory without interpreting source pixels or reference data."""
    train_gate(route)
    receipts = json.loads((PRIVATE / "fetch_receipts.json").read_text())
    source = next(x for x in route["sources"] if x["extension"] == "zip")
    pin = next(x for x in receipts if x["psi_file_id"] == source["psi_file_id"])
    archive = contained(PRIVATE, pin["path"])
    if sha256(archive) != pin["sha256"]:
        raise ValueError("full native archive identity changed")
    clocks = {}
    with (PRIVATE / "native_frame_clock.jsonl").open() as stream:
        for line in stream:
            row = json.loads(line)
            if row["native_frame_index"] in clocks:
                raise ValueError("duplicate clock index")
            clocks[row["native_frame_index"]] = row
    members = []
    with zipfile.ZipFile(archive) as source_zip:
        for info in source_zip.infolist():
            match = re.fullmatch(re.escape(route["alias"]) + r"A_([0-9]+)\.tif", info.filename)
            if match:
                index = int(match[1])
                if index not in clocks or clocks[index]["source_member"] != info.filename:
                    raise ValueError("clock and native member mismatch")
                members.append({"native_frame_index": index, "member": info.filename,
                                "uncompressed_bytes": info.file_size, "crc32": info.CRC})
            elif info.filename != route["alias"] + "A.txt":
                raise ValueError("unexpected native archive entry")
    members.sort(key=lambda x: x["native_frame_index"])
    if len(members) != len(clocks) or len({r["native_frame_index"] for r in members}) != len(members):
        raise ValueError("incomplete native inventory")
    return {"schema": "astra.native-member-inventory/1", "archive_path": str(archive),
            "archive_sha256": pin["sha256"], "clock_sha256": sha256(PRIVATE / "native_frame_clock.jsonl"),
            "members": members, "selection": "all native members, ascending native index; no quality exclusions"}


def release_continuation(route: dict) -> dict:
    train_gate(route)
    base = PRIVATE / "continuation"
    state = json.loads((base / "checkpoint.json").read_text())
    batch = contained(base, state["active_batch"])
    receipt_path = contained(batch, state.get("active_release_receipt", "release_receipt.json"))
    if receipt_path.exists():
        return json.loads(receipt_path.read_text())
    manifest = json.loads((batch / "manifest.json").read_text())
    validate_blind_manifest(manifest)
    review = json.loads((batch / "overlay_review.json").read_text())
    if review["state"] != "approved" or review["condition_overlays_present"] is not False or review["reviewed_sha256"] != [f["sha256"] for f in manifest["frames"]]:
        raise ValueError("continuation overlay review incomplete")
    if sha256(batch / "METHOD.md") != state["method_sha256"]:
        raise ValueError("frozen method changed")
    packets = []
    for _ in range(2):
        packet_id = uuid.uuid4().hex
        destination = ROOT / "runs/dev/data_astra/work/blind_tasks" / packet_id
        destination.mkdir()
        (destination / "output").mkdir()
        for frame in manifest["frames"]:
            source = contained(batch, frame["path"])
            if sha256(source) != frame["sha256"]:
                raise ValueError("reviewed continuation frame changed")
            shutil.copyfile(source, destination / frame["path"])
            if sha256(destination / frame["path"]) != frame["sha256"]:
                raise ValueError("delivered continuation frame changed")
        shutil.copyfile(batch / "METHOD.md", destination / "METHOD.md")
        delivered = dict(manifest,packet_id=packet_id,full_source_ref="media_"+uuid.uuid4().hex)
        validate_blind_manifest(delivered)
        (destination / "manifest.json").write_text(json.dumps(delivered,indent=2))
        packets.append({"packet_id":packet_id,"path":str(destination),"full_source_ref":delivered["full_source_ref"],
                        "manifest_sha256":sha256(destination / "manifest.json")})
    receipt = {"state":"ready_for_independent_blind_measurement","packets":packets,
               "batch_id":state["active_batch"],"method_sha256":state["method_sha256"],"source_retained":True}
    with receipt_path.open("x") as stream:json.dump(receipt,stream,indent=2)
    return receipt


def budget_indices(members: list[dict], covered: set[int], byte_budget: int) -> list[dict]:
    """Choose contiguous pending indices by decoded storage budget, never observed quality."""
    if type(byte_budget) is not int or byte_budget <= 0:
        raise ValueError("positive resource budget required")
    chosen, used = [], 0
    for member in members:
        if member["native_frame_index"] in covered:
            continue
        # PNG deflate overhead bound is conservatively covered by 64 KiB beyond
        # the native fixed array; this is a byte resource bound, not a frame cap.
        allocation = 1024 * 1024 * 2 + 65536
        if used + allocation > byte_budget:
            if not chosen:
                raise ValueError("resource budget cannot hold one native frame")
            break
        chosen.append(member)
        used += allocation
    return chosen


def stage_continuation(route: dict, byte_budget: int) -> dict:
    """Stage one resource-bounded batch. Never advance over an unsealed active batch."""
    checkpoint_path = PRIVATE / "continuation/checkpoint.json"
    if checkpoint_path.exists() and json.loads(checkpoint_path.read_text()).get("paused_by_controller"):
        raise ValueError("continuation paused by controller; explicit resume required")
    import io
    import numpy as np
    from PIL import Image
    from compare_blind_measurements import verify_seal, load_submission, compare

    if byte_budget > route["workspace_limit_bytes"]:
        raise ValueError("batch exceeds reserved workspace")
    if shutil.disk_usage(PRIVATE).free < 10 * 1024**3 + byte_budget:
        raise ValueError("disk reserve would be breached")
    continuation = PRIVATE / "continuation"
    continuation.mkdir(exist_ok=True)
    storage_roots = [PRIVATE, ROOT / "runs/dev/data_astra/work/blind_tasks"]
    retained = sum(p.stat().st_size for base in storage_roots for p in base.rglob("*") if p.is_file())
    if retained + 8 * byte_budget > route["reservation_bytes"]:
        raise ValueError("reserved total cannot cover next batch and both observer artifacts")
    checkpoint_path = continuation / "checkpoint.json"
    inventory_path = continuation / "native_inventory.json"
    pilot = json.loads((PRIVATE / "blind_candidates/manifest.json").read_text())
    covered = {x["frame_index"] for x in pilot["frames"]}
    if checkpoint_path.exists():
        checkpoint = json.loads(checkpoint_path.read_text())
        if checkpoint.get("paused_by_controller"):
            raise ValueError("continuation paused by controller; explicit resume required")
        if sha256(inventory_path) != checkpoint["inventory_sha256"]:
            raise ValueError("native inventory changed")
        inventory = json.loads(inventory_path.read_text())
        train_gate(route)
        if sha256(Path(inventory["archive_path"])) != inventory["archive_sha256"]:
            raise ValueError("retained source changed")
        covered = set(checkpoint["sealed_covered_indices"])
        if checkpoint.get("active_batch"):
            active = contained(continuation, checkpoint["active_batch"])
            active_release = contained(active, checkpoint.get("active_release_receipt", "release_receipt.json"))
            if not (active / "submission_seal.json").exists() or not active_release.exists():
                raise ValueError("active batch awaits sealed measurement and evidence retention")
            seal = json.loads((active / "submission_seal.json").read_text())
            release = json.loads(active_release.read_text())
            verify_seal(seal)
            binding_path = active / "runtime_context_bindings.json"
            bindings = (json.loads(binding_path.read_text())["packets"] if binding_path.exists()
                        else [None] * len(release["packets"]))
            a, b = [load_submission(p, s, binding) for p, s, binding in
                    zip(release["packets"], seal["packets"], bindings, strict=True)]
            expected = {f["frame_index"]: f["sha256"] for f in json.loads((active / "manifest.json").read_text())["frames"]}
            if any({i:r["source_sha256"] for i,r in submission["rows"].items()} != expected for submission in (a,b)):
                raise ValueError("batch submissions do not cover exact staged pixels")
            result = compare(a,b)
            verify_seal(seal)
            complete = active / "completion_receipt.json"
            if not complete.exists():
                with complete.open("x") as stream:
                    json.dump({"state":"sealed_and_evidence_verified","frame_indices":sorted(expected),
                               "seal_sha256":sha256(active / "submission_seal.json"),
                               "comparison":result,"originals_retained":True},stream,indent=2)
            covered.update(expected)
            checkpoint.update(active_batch=None,sealed_covered_indices=sorted(covered),batch_status="ready_for_next_batch")
            checkpoint_path.write_text(json.dumps(checkpoint,indent=2))
    else:
        inventory = native_inventory(route)
    verify_seal(json.loads((PRIVATE / "submission_seal.json").read_text()))
    chosen = budget_indices(inventory["members"], covered, byte_budget)
    if not chosen:
        return {"state":"all_native_frames_sealed; calibration/reference qualification unresolved", "frames":len(covered)}
    batch_id = uuid.uuid4().hex
    batch = continuation / batch_id
    batch.mkdir()
    frames = []
    with zipfile.ZipFile(inventory["archive_path"]) as archive:
        for seq, row in enumerate(chosen):
            raw = archive.read(row["member"])
            with Image.open(io.BytesIO(raw)) as im:
                values = np.asarray(im)
                if values.dtype != np.uint16 or values.shape != (1024, 1024) or np.any(values % 16):
                    raise ValueError("unexpected native instrument format")
                path = batch / f"frame_{seq:06d}.png"
                Image.fromarray(values).save(path)
                with Image.open(path) as check:
                    if not np.array_equal(values, np.asarray(check)):
                        raise ValueError("native batch conversion changed pixels")
            frames.append({"path": path.name, "sha256": sha256(path), "frame_index": row["native_frame_index"],
                           "width": 1024, "height": 1024, "bit_depth": 16})
    method_source = PRIVATE / "blind_candidates/METHOD.md"
    shutil.copyfile(method_source, batch / "METHOD.md")
    manifest = dict(pilot, packet_id=batch_id, full_source_ref="media_" + uuid.uuid4().hex, frames=frames)
    validate_blind_manifest(manifest)
    (batch / "manifest.json").write_text(json.dumps(manifest, indent=2))
    if not inventory_path.exists():
        with inventory_path.open("x") as stream:
            json.dump(inventory, stream, indent=2)
    protocol = {"schema": "astra.native-continuation-protocol/1", "method_sha256": pilot["method_sha256"],
        "method_change": False, "all_native_frames": len(inventory["members"]),
        "retained_pilot_indices": sorted(covered), "pending_initial": len(inventory["members"])-len(covered),
        "order": "ascending native index, reuse sealed pilot observations; no outcome filtering",
        "batch_budget_bytes": byte_budget, "memory": "one decoded native frame at a time",
        "gate": "custodian screens every batch frame for overlays before independently aliased A/B release",
        "advance": "requires both immutable measurement receipts and retained evidence hashes for active batch; original archive retained throughout",
        "reference_comparison": "none until independent spatial calibration and clock qualification; no expected diameter allowed",
        "source_deletion": "forbidden while whole source coverage or evidence retention unresolved"}
    if not (continuation / "protocol.json").exists():
        (continuation / "protocol.json").write_text(json.dumps(protocol, indent=2))
    state = {"active_batch": batch_id, "batch_status": "staged; overlay review and A/B dispatch pending",
             "sealed_covered_indices": sorted(covered), "staged_indices": [r["native_frame_index"] for r in chosen],
             "inventory_sha256": sha256(continuation / "native_inventory.json"),
             "method_sha256": pilot["method_sha256"], "source_retained": True}
    checkpoint_path.write_text(json.dumps(state, indent=2))
    return {"state": state["batch_status"], "candidate_path": str(batch), "frames": len(frames),
            "all_native_frames": len(inventory["members"]), "remaining_unmeasured": protocol["pending_initial"]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", action="store_true")
    parser.add_argument("--stage", action="store_true")
    parser.add_argument("--release-reviewed", action="store_true")
    parser.add_argument("--stage-continuation", action="store_true")
    parser.add_argument("--release-continuation-reviewed", action="store_true")
    parser.add_argument("--workspace-bytes", type=int)
    args = parser.parse_args()
    route = json.loads((PRIVATE / "route.json").read_text())
    if args.release_continuation_reviewed:
        if not args.execute:raise ValueError("release requires --execute")
        print(json.dumps(release_continuation(route)))
        return
    if args.stage_continuation:
        if not args.execute or args.workspace_bytes is None:
            raise ValueError("continuation requires --execute and explicit byte resource budget")
        print(json.dumps(stage_continuation(route, args.workspace_bytes)))
        return
    if args.release_reviewed:
        if not args.execute:
            raise ValueError("release requires --execute")
        print(json.dumps(release_reviewed_samples(route)))
        return
    if args.stage:
        if not args.execute:
            raise ValueError("staging requires --execute")
        print(json.dumps(stage_samples(route)))
        return
    result = fetch(route, args.execute)
    print(json.dumps({"sources": len(result), "execute": args.execute,
                      "bytes": sum(x.get("bytes", x.get("size_bytes", 0)) for x in result)}))


if __name__ == "__main__":
    main()
