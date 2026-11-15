"""One metadata-only PSI availability probe, with durable backoff and no body reads."""
import argparse
from datetime import datetime, timezone, timedelta
import json
import msvcrt
from urllib.parse import urlsplit

import requests

from fetch_public import ROOT, now, session, write_json


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    state_path = ROOT / "logs/availability_state.json"
    state_path.parent.mkdir(parents=True, exist_ok=True)
    if not args.execute:
        print(json.dumps(dict(execute=False, state=json.loads(state_path.read_text()) if state_path.exists() else None)))
        return 0
    # Do not let a second caller race the persisted backoff or overlap the probe.
    with (ROOT / "logs/availability_probe.lock").open("a+b") as lock:
        if lock.tell() == 0:
            lock.write(b"0")
            lock.flush()
        lock.seek(0)
        try:
            msvcrt.locking(lock.fileno(), msvcrt.LK_NBLCK, 1)
        except OSError:
            print(json.dumps(dict(state="another_probe_running", probe_calls_started=0)))
            return 0
        previous = json.loads(state_path.read_text()) if state_path.exists() else {}
        current = datetime.now(timezone.utc)
        next_allowed = previous.get("next_check_not_before")
        if next_allowed and current < datetime.fromisoformat(next_allowed.replace("Z", "+00:00")):
            print(json.dumps(dict(state="backoff", next_check_not_before=next_allowed, probe_calls_started=0)))
            return 0
        rows = [json.loads(line) for line in (ROOT / "metadata/selected_files.jsonl").read_text().splitlines()]
        row = next(row for row in rows if row["file_name"] == "PSI-69_Calibration_20CAL13_Readme.txt")
        record = dict(checked_at=now(), investigation=row["investigation"], file_name=row["file_name"],
                      identity=row["identity"], stable_url=row["stable_url"], body_bytes_read=0,
                      declared_bytes=row["size_bytes"], probe_calls_started=1,
                      redirect_policy="sequential; no parallel HTTP requests")
        healthy, retry_after = False, 0
        try:
            with session().get(row["stable_url"], stream=True, timeout=(10, 20)) as response:
                host = urlsplit(response.url).hostname or ""
                record.update(http_status=response.status_code, final_host=host, http_response_chain_length=len(response.history) + 1,
                              response_headers={key: response.headers[key] for key in
                                ("Content-Length", "Content-Type", "ETag", "Retry-After") if key in response.headers})
                healthy = (response.status_code == 200 and host.endswith(".amazonaws.com") and
                           response.headers.get("Content-Length") == str(row["size_bytes"]))
                if response.headers.get("Retry-After", "").isdigit():
                    retry_after = int(response.headers["Retry-After"])
        except requests.RequestException as error:
            record["error_type"] = type(error).__name__
        failures = 0 if healthy else previous.get("consecutive_failures", 0) + 1
        delay = 60 if healthy else (60, 120, 240, 300)[min(failures - 1, 3)]
        delay = max(delay, retry_after)
        record.update(state="healthy" if healthy else "unavailable", consecutive_failures=failures,
                      next_check_not_before=(datetime.now(timezone.utc) + timedelta(seconds=delay)).isoformat())
        write_json(state_path, record)
        with (ROOT / "logs/availability_checks.jsonl").open("a", encoding="utf-8") as ledger:
            ledger.write(json.dumps(record, sort_keys=True) + "\n")
        print(json.dumps(record), flush=True)
        return 0 if healthy else 1


if __name__ == "__main__":
    raise SystemExit(main())
