"""Compare sealed manual pixel observations without mutating either submission.

This is descriptive repeatability on one authorized TRAIN pilot, not NASA accuracy.
The private protocol is frozen before execution. No reference series is opened.
"""
from __future__ import annotations
import json
import math
from pathlib import Path
from statistics import mean
from media_custodian import PRIVATE, contained, sha256, train_gate, validate_blind_manifest

OBSERVABLES = ("left", "right", "top", "bottom", "width", "height")


def verify_seal(seal):
    for packet in seal["packets"]:
        root = Path(packet["path"])
        expected = {item["path"] for item in packet["files"]}
        output_root = root / "output"
        if output_root.exists():
            actual = {str(path.relative_to(root)) for path in output_root.rglob("*") if path.is_file()}
            if actual != expected:
                raise ValueError("sealed output inventory changed")
        for item in packet["files"]:
            path = contained(root, item["path"])
            if path.stat().st_size != item["bytes"] or sha256(path) != item["sha256"]:
                raise ValueError("sealed submission mutated")


def checked_value(value, interval):
    if value is None:
        if interval is not None:
            raise ValueError("missing value carries interval")
        return None, None
    if (type(value) not in (int, float) or not math.isfinite(value)
            or not isinstance(interval, list) or len(interval) != 2
            or any(type(v) not in (int, float) or not math.isfinite(v) for v in interval)
            or not interval[0] <= value <= interval[1]):
        raise ValueError("invalid finite operator bounds")
    return value, interval


def normalize_frame(frame):
    candidates = frame["candidates"]
    if len(candidates) > 1:
        return {name: {"value": None, "bounds": None, "status": "unresolved_multiple_candidates"}
                for name in OBSERVABLES}
    candidate = candidates[0] if candidates else frame.get("unavailable_observation", frame)
    if set(candidate["edges"]) != set(OBSERVABLES[:4]) or set(candidate["extents"]) != set(OBSERVABLES[4:]):
        raise ValueError("observable set differs from frozen method")
    result = {}
    for name in OBSERVABLES:
        is_edge = name in OBSERVABLES[:4]
        entry = candidate["edges" if is_edge else "extents"].get(name)
        if entry is None:
            states = candidate.get("edge_statuses" if is_edge else "extent_statuses", {})
            status = states.get(name, {}).get("status")
            if status is None:
                status = candidate.get("missing_edges", {}).get(name, {}).get("reason", "unavailable")
            value, bounds = None, None
        else:
            value = entry.get("coordinate_px" if is_edge else "value_px")
            bounds = (entry.get("ambiguity_interval_px", entry.get("coordinate_interval_px"))
                      if is_edge else entry.get("interval_px"))
            status = entry.get("status", "observed" if value is not None else "unavailable")
            if is_edge and entry["axis"] != ("x" if name in ("left", "right") else "y"):
                raise ValueError("coordinate axis mismatch")
        value, bounds = checked_value(value, bounds)
        if status not in {"observed", "occluded", "ambiguous", "out_of_field", "unavailable"}:
            raise ValueError("unsupported observation status")
        if (value is None) == (status == "observed"):
            raise ValueError("status and availability disagree")
        result[name] = {"value": value, "bounds": bounds, "status": status}
    for extent, low, high in (("width", "left", "right"), ("height", "top", "bottom")):
        a, b, e = result[low], result[high], result[extent]
        if a["value"] is not None and b["value"] is not None:
            if e["value"] != b["value"] - a["value"] or e["bounds"] != [b["bounds"][0] - a["bounds"][1], b["bounds"][1] - a["bounds"][0]]:
                raise ValueError("extent is not declared edge difference")
        elif e["value"] is not None:
            raise ValueError("extent reconstructed through unavailable edge")
    return result


def observer_identity(doc, runtime_binding=None):
    """Keep observer prose separate from controller-attested execution identity."""
    identity = doc.get("observer", doc.get("operator"))
    declaration = doc["exposure_declaration"]
    if runtime_binding is None:
        if declaration.get("task_context") != identity["task_context"]:
            raise ValueError("runtime context mismatch")
    else:
        if (runtime_binding["packet_id"] != doc["packet_id"]
                or runtime_binding["observer_declaration"] != identity
                or runtime_binding["exposure_declaration"] != declaration
                or not runtime_binding["task_context"]
                or runtime_binding["fork_turns"] != "none"):
            raise ValueError("controller runtime binding mismatch")
        identity = dict(identity, task_context=runtime_binding["task_context"],
                        original_task_context=identity["task_context"])
    familiarity = [declaration[k] for k in ("prior_familiarity_with_pictured_acquisition", "prior_familiarity") if k in declaration]
    if not familiarity or any(value is not False for value in familiarity):
        raise ValueError("observer exposure declaration missing or affirmative")
    if any(declaration.get(k, False) is not False for k in ("outside_packet_scientific_context_accessed", "other_observer_work_accessed", "external_context_sought")):
        raise ValueError("observer declares outside exposure")
    return identity


def load_submission(packet, seal_packet, runtime_binding=None):
    root = Path(packet["path"])
    if root != Path(seal_packet["path"]) or packet["packet_id"] != seal_packet["packet_id"]:
        raise ValueError("sealed packet identity mismatch")
    manifest = json.loads((root / "manifest.json").read_text())
    validate_blind_manifest(manifest)
    doc = json.loads((root / "output/measurements.json").read_text())
    if (sha256(root / "manifest.json") != packet["manifest_sha256"]
            or doc["manifest_sha256"] != packet["manifest_sha256"]
            or doc["packet_id"] != manifest["packet_id"]
            or doc["method_sha256"] != manifest["method_sha256"]
            or sha256(root / "METHOD.md") != doc["method_sha256"]
            or doc["method_version"] != manifest["method_version"]):
        raise ValueError("input or method identity mismatch")
    if doc["distance_unit"] != "pixel" or doc["time_axis"] != "native_frame_index":
        raise ValueError("unit or time-axis mismatch")
    if doc["qualification"] != "unqualified_pixel_only" or doc["partial"] is not True:
        raise ValueError("unsupported qualification")
    expected = {f["frame_index"]: f for f in manifest["frames"]}
    rows = {}
    for frame in doc["frames"]:
        index = frame.get("native_frame_index", frame.get("frame_index"))
        if index in rows or index not in expected:
            raise ValueError("duplicate or unexpected frame")
        pin = expected[index]
        source = contained(root, pin["path"])
        if (frame["input_path"] != pin["path"] or frame["input_sha256"] != pin["sha256"]
                or sha256(source) != pin["sha256"]):
            raise ValueError("source frame identity mismatch")
        if frame["inspection_status"] not in ("complete", "completed"):
            raise ValueError("frame inspection incomplete")
        evidence = contained(root / "output", frame["evidence_png"])
        evidence_sha = sha256(evidence)
        if "evidence_sha256" in frame and frame["evidence_sha256"] != evidence_sha:
            raise ValueError("evidence digest mismatch")
        rows[index] = {"source_sha256": pin["sha256"], "evidence_sha256": evidence_sha,
                       "observables": normalize_frame(frame)}
    if set(rows) != set(expected):
        raise ValueError("frame coverage incomplete")
    if runtime_binding is not None and sha256(root / "output/measurements.json") != runtime_binding["measurements_sha256"]:
        raise ValueError("runtime binding submission digest mismatch")
    identity = observer_identity(doc, runtime_binding)
    return {"method_sha256": doc["method_sha256"], "identity": identity,
            "unit": doc["distance_unit"], "time_axis": doc["time_axis"], "rows": rows}


def compare(a, b):
    if (a["method_sha256"] != b["method_sha256"] or a["unit"] != "pixel" or b["unit"] != "pixel"
            or a["time_axis"] != "native_frame_index" or b["time_axis"] != "native_frame_index"):
        raise ValueError("incompatible method or units")
    if a["identity"]["task_context"] == b["identity"]["task_context"]:
        raise ValueError("independent observer contexts required")
    if set(a["rows"]) != set(b["rows"]):
        raise ValueError("unmatched frame coverage")
    records = []
    for index in sorted(a["rows"]):
        ar, br = a["rows"][index], b["rows"][index]
        if ar["source_sha256"] != br["source_sha256"]:
            raise ValueError("different source pixels")
        for name in OBSERVABLES:
            x, y = ar["observables"][name], br["observables"][name]
            item = {"frame_index": index, "observable": name, "unit": "pixel", "a": x, "b": y,
                    "status_agreement": x["status"] == y["status"], "difference_b_minus_a": None,
                    "difference_bounds": None, "bounds_overlap": None}
            if x["value"] is not None and y["value"] is not None:
                lo, hi = y["bounds"][0] - x["bounds"][1], y["bounds"][1] - x["bounds"][0]
                item.update(difference_b_minus_a=y["value"] - x["value"], difference_bounds=[lo, hi], bounds_overlap=lo <= 0 <= hi)
            records.append(item)
    summaries = {}
    for name in OBSERVABLES:
        rows = [r for r in records if r["observable"] == name]
        paired = [r for r in rows if r["difference_b_minus_a"] is not None]
        diffs = [r["difference_b_minus_a"] for r in paired]
        summaries[name] = {"frames": len(rows), "available_pairs": len(paired),
            "both_missing": sum(r["a"]["value"] is None and r["b"]["value"] is None for r in rows),
            "one_sided_missing": sum((r["a"]["value"] is None) != (r["b"]["value"] is None) for r in rows),
            "status_agreements": sum(r["status_agreement"] for r in rows),
            "mean_signed_difference_px": mean(diffs) if diffs else None,
            "mean_absolute_difference_px": mean(map(abs, diffs)) if diffs else None,
            "rms_difference_px": math.sqrt(mean(d*d for d in diffs)) if diffs else None,
            "maximum_absolute_difference_px": max(map(abs, diffs)) if diffs else None,
            "overlapping_operator_bounds": sum(r["bounds_overlap"] for r in paired),
            "mean_difference_bounds_px": [mean(r["difference_bounds"][i] for r in paired) for i in (0, 1)] if paired else None,
            "mean_absolute_difference_bounds_px": [mean(0 if r["bounds_overlap"] else min(map(abs, r["difference_bounds"])) for r in paired),
                mean(max(map(abs, r["difference_bounds"])) for r in paired)] if paired else None}
    return {"records": records, "summary": summaries}


def main():
    route = json.loads((PRIVATE / "route.json").read_text())
    train_gate(route)
    protocol = json.loads((PRIVATE / "comparison_protocol.json").read_text())
    seal = json.loads((PRIVATE / "submission_seal.json").read_text())
    release = json.loads((PRIVATE / "release_receipt.json").read_text())
    verify_seal(seal)
    a, b = [load_submission(p, s) for p, s in zip(release["packets"], seal["packets"], strict=True)]
    if protocol["method_sha256"] != a["method_sha256"]:
        raise ValueError("frozen protocol method mismatch")
    result = compare(a, b)
    verify_seal(seal)
    result.update(schema="astra.pixel-repeatability/1", protocol_sha256=sha256(PRIVATE / "comparison_protocol.json"),
                  submission_seal_sha256=sha256(PRIVATE / "submission_seal.json"),
                  qualification=protocol["qualification"], independent_contexts=[a["identity"]["task_context"], b["identity"]["task_context"]],
                  exact_model_builds="unavailable; declared model identity identical; context separation supported by controller inventory",
                  observations_modified=False, nasa_reference_accessed=False)
    with (PRIVATE / "pixel_repeatability.json").open("x", encoding="utf-8") as stream:
        json.dump(result, stream, indent=2, allow_nan=False)
    print(json.dumps({"state": "comparison_complete", "summary": result["summary"],
                      "observations_modified": False, "nasa_reference_accessed": False}))


if __name__ == "__main__":
    main()
