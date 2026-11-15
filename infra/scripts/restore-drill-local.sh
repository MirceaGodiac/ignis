#!/usr/bin/env bash
# Local, non-destructive restore drill (M4). Populates a scratch state directory through a real
# `ignis serve` (fake Codex runtime, local auth, loopback only), takes a live `ignis state backup`,
# writes more data after the backup, restores the snapshot into a fresh directory, starts a second
# instance on it and checks that the restored instance replays every backed-up episode's event log
# identically and serves every backed-up artifact byte-identical. Prints RTO and RPO figures.
#
# Never touches the VM or any live state: everything lives under $WORK (default target/drill).
# Usage: infra/scripts/restore-drill-local.sh [WORK]
#   IGNIS=<path to ignis>  FAKE_CODEX=<path to ignis-server-fake-codex>  (defaults: target/debug)
set -euo pipefail

WORK=${1:-target/drill}
EXE=""; case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) EXE=".exe" ;; esac
IGNIS=${IGNIS:-target/debug/ignis$EXE}
FAKE_CODEX=${FAKE_CODEX:-target/debug/ignis-server-fake-codex$EXE}
EPISODES=${EPISODES:-6}
PORT_A=${PORT_A:-18741}
PORT_B=${PORT_B:-18742}

rm -rf "$WORK"
mkdir -p "$WORK"
WORK=$(cd "$WORK" && pwd)
A=http://127.0.0.1:$PORT_A/api/v1
B=http://127.0.0.1:$PORT_B/api/v1
now() { date +%s.%N; }
secs() { awk "BEGIN { printf \"%.2f\", $2 - $1 }"; }
pids=()
cleanup() { for p in "${pids[@]}"; do kill "$p" 2>/dev/null || true; done; }
trap cleanup EXIT

serve() { # port state-dir log
  "$IGNIS" serve --port "$1" --host 127.0.0.1 --dev --auth local --state-dir "$2" \
    --codex-bin "$FAKE_CODEX" --data "$WORK/none" --cohorts "$WORK/none" --splits "$WORK/none" \
    --raw-samples "$WORK/none" --evidence-db "$WORK/none.sqlite" --web-dist "$WORK/none" \
    >"$3" 2>&1 &
  pids+=($!)
}
healthy() { # port; polls /livez (readiness, not a fixed sleep), fails after 120 s
  for _ in $(seq 1 1200); do
    curl -sf "http://127.0.0.1:$1/livez" >/dev/null && return 0
    sleep 0.1
  done
  echo "instance on port $1 never became healthy" >&2; return 1
}
field() { sed -n "s/.*\"$1\":\"\\{0,1\\}\\([^\",}]*\\).*/\\1/p" | head -n1; }
json_post() { curl -sf -X POST "$1" -H 'content-type: application/json' -d "$2"; }

episode() { # base goal tag -> episode id, after its turn completed
  local id st
  id=$(json_post "$1/episodes" "{\"goal\":\"$2\",\"client_request_id\":\"c-$3\"}" | field episode_id)
  json_post "$1/episodes/$id/messages" \
    "{\"text\":\"search with a tool for $2\",\"client_request_id\":\"m-$3\"}" >/dev/null
  for _ in $(seq 1 600); do
    st=$(curl -sf "$1/episodes/$id" | field status)
    [ "$st" = completed ] && { echo "$id"; return 0; }
    sleep 0.1
  done
  echo "episode $id did not complete (status $st)" >&2; return 1
}
upload() { # base file tag -> artifact id
  local sha len t put up
  sha=sha256:$(sha256sum "$2" | cut -d' ' -f1); len=$(wc -c <"$2" | tr -d ' ')
  t=$(json_post "$1/uploads/initiate" \
    "{\"media_type\":\"text/csv\",\"byte_length\":$len,\"sha256\":\"$sha\",\"client_request_id\":\"u-$3\"}")
  put=$(echo "$t" | field put_path); up=$(echo "$t" | field upload_id)
  curl -sf -X PUT "$1/$put" --data-binary "@$2" >/dev/null
  json_post "$1/uploads/complete" "{\"upload_id\":\"$up\"}" | field artifact_id
}
events() { # base id -> the episode's event log (data lines), read from seq 0 up to last_seq
  local last
  last=$(curl -sf "$1/episodes/$2" | field last_seq)
  echo "$last" >"$WORK/last-$2"
  # The stream stays open (heartbeats); awk stops at last_seq, which ends curl with SIGPIPE.
  ( set +o pipefail
    curl -sN --max-time 30 "$1/episodes/$2/events?after=0" 2>/dev/null \
      | awk -v last="$last" '/^data:/ { print; if (index($0, "\"seq\":" last ",")) exit }' )
}

echo "== populate (instance A on $PORT_A)"
serve "$PORT_A" "$WORK/state-a" "$WORK/a.log"; healthy "$PORT_A"
ids=(); arts=(); files=()
for i in $(seq 1 "$EPISODES"); do ids+=("$(episode "$A" "drill episode $i" "e$i")"); done
for size in 1024 1048576 8388608 33554432; do
  f="$WORK/upload-$size.csv"
  # Random numeric CSV of exactly $size bytes (generated a little long, then cut to size; no
  # pipe into `head`, which would end the generator with SIGPIPE under pipefail).
  { echo "t_s,d_m"; head -c "$size" /dev/urandom | od -An -tu4 -v | awk '{print NR*0.001 "," $1}'; } \
    >"$f.full"
  head -c "$size" "$f.full" >"$f"
  rm -f "$f.full"
  files+=("$f"); arts+=("$(upload "$A" "$f" "a$size")")
done
echo "episodes: ${#ids[@]}, artifacts: ${#arts[@]}"

echo "== live backup while A serves"
t0=$(now)
"$IGNIS" state backup --state-dir "$WORK/state-a" --out "$WORK/snap" >"$WORK/backup.log" 2>&1
t1=$(now); backup_s=$(secs "$t0" "$t1"); backup_at=$t1
"$IGNIS" state verify "$WORK/snap" >"$WORK/verify.log" 2>&1
snap_files=$(grep -o '"path"' "$WORK/snap/manifest.json" | wc -l | tr -d ' ')
snap_bytes=$(du -sb "$WORK/snap" | cut -f1)

echo "== written after the backup (inside the RPO window)"
late=$(episode "$A" "after the backup" late)
late_at=$(now)

echo "== restore into a fresh directory + start instance B (RTO)"
t2=$(now)
"$IGNIS" state restore "$WORK/snap" --target "$WORK/state-b" >"$WORK/restore.log" 2>&1
t3=$(now)
serve "$PORT_B" "$WORK/state-b" "$WORK/b.log"; healthy "$PORT_B"
t4=$(now)
restore_s=$(secs "$t2" "$t3"); start_s=$(secs "$t3" "$t4"); rto_s=$(secs "$t2" "$t4")

echo "== verify"
fail=0
for id in "${ids[@]}"; do
  events "$A" "$id" >"$WORK/ev-a-$id"; events "$B" "$id" >"$WORK/ev-b-$id"
  n=$(wc -l <"$WORK/ev-a-$id" | tr -d ' ')
  [ "$n" = "$(cat "$WORK/last-$id")" ] || { echo "FAIL $id: replayed $n events, last_seq $(cat "$WORK/last-$id")"; fail=1; }
  if [ "$n" -gt 0 ] && cmp -s "$WORK/ev-a-$id" "$WORK/ev-b-$id"; then
    seqs=$(sed 's/.*"seq":\([0-9]*\).*/\1/' "$WORK/ev-b-$id" | tr '\n' ' ')
    [ "$seqs" = "$(seq 1 "$n" | tr '\n' ' ')" ] || { echo "FAIL $id: seqs not contiguous: $seqs"; fail=1; }
  else
    echo "FAIL $id: event log differs or is empty ($n events on A)"; fail=1
  fi
done
total_events=$(cat "$WORK"/ev-b-* | wc -l | tr -d ' ')
for k in "${!arts[@]}"; do
  got=$(curl -sf "$B/artifacts/${arts[$k]}" | sha256sum | cut -d' ' -f1)
  want=$(sha256sum "${files[$k]}" | cut -d' ' -f1)
  [ "$got" = "$want" ] || { echo "FAIL artifact ${arts[$k]}: $got != $want"; fail=1; }
done
st_late=$(curl -s -o /dev/null -w '%{http_code}' "$B/episodes/$late")
[ "$st_late" = 404 ] || { echo "FAIL: the post-backup episode should be lost on B (got $st_late)"; fail=1; }
listed_a=$(curl -sf "$A/episodes" | grep -o '"episode_id"' | wc -l | tr -d ' ')
listed_b=$(curl -sf "$B/episodes" | grep -o '"episode_id"' | wc -l | tr -d ' ')

cat <<EOF
== result ($( [ $fail = 0 ] && echo PASS || echo FAIL ))
snapshot:            $snap_files files, $snap_bytes bytes, live backup $backup_s s, verify ok
restored instance:   $listed_b episodes listed (A: $listed_a), ${#ids[@]} event logs replayed identically ($total_events events, seq contiguous), ${#arts[@]} artifacts byte-identical
RTO (local):         $rto_s s = restore $restore_s s + start to healthy $start_s s
RPO (local):         the episode written $(secs "$backup_at" "$late_at") s after the backup is absent on B (404): everything after the last snapshot is lost
EOF
exit $fail
