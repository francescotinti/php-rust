#!/bin/bash
# phaseR-supplement.sh — S-82.0 p7: Phase R re-run with the instrument
# ARMED. Campaign-4's Phase R ran the mem-census binary WITHOUT
# PHPR_MEM_CENSUS=<file>: the feature compiles the instrument, the env
# arms it (WP-45) — full-body PASS was real, the figure was mute.
# Phase R is driver-independent (no measure78, no matrix identity): this
# supplement re-runs it standalone at the current rev; the binary hash must
# EQUAL campaign-4's mem_hash (crates untouched) or the run is VOID.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
OUT="$REPO/wp78-harness/measure-out"
OUTBIN="$HOME/Claude/php-rust-output/release"
ORACLE="/opt/homebrew/opt/php/bin/php"
FIXDIR="$REPO/wp78-harness/gate-axum/fixtures"
EXPECT_MEM_HASH="${1:?usage: phaseR-supplement.sh <campaign-4 mem_hash>}"
FAILS=0
fail() { echo "FAIL: $*"; FAILS=$((FAILS+1)); }

( cd "$REPO" && cargo build --release -p php-server --features mem-census ) \
  > "$OUT/m82r.build.log" 2>&1 || { echo "FAIL: build"; exit 1; }
MEM_HASH=$(shasum -a 256 "$OUTBIN/php-server" | cut -c1-16)
if [ "$MEM_HASH" != "$EXPECT_MEM_HASH" ]; then
  echo "FAIL: mem-census hash $MEM_HASH != campaign-4 $EXPECT_MEM_HASH — crates moved, supplement VOID"
  exit 1
fi
HFIX="hello.php include_gate.php include_heavy.php bare.php"
PORT=8299
MCFILE="$OUT/m82r.retained.memcensus"
: > "$MCFILE"
# CLI-SERVER arm (no --axum): the tag=unitcache mem-census dump lives in
# run_module_with_hir's request teardown — the axum worker teardown does
# not emit it (campaign-4 + supplement-1 both mute). The cli-server is the
# declared honest A/B twin (design79 §2) with the SAME acquire/park/publish.
PHPR_MEM_CENSUS="$MCFILE" "$OUTBIN/php-server" --port $PORT -t "$FIXDIR" \
  > /dev/null 2> "$OUT/m82r.retained.log" &
MPID=$!
up=0; for _ in $(seq 1 100); do curl -s -o /dev/null -m 1 "http://127.0.0.1:$PORT/hello.php" && { up=1; break; }; sleep 0.1; done
[ $up = 1 ] || { fail "server not up"; kill -TERM $MPID 2>/dev/null; exit 1; }
RBODYFAIL=0
for f in $HFIX; do
  want=$(cd "$FIXDIR" && "$ORACLE" -n "$f" 2>/dev/null)
  for _ in 1 2 3; do
    got=$(curl -s -m 10 "http://127.0.0.1:$PORT/$f")
    [ "$got" = "$want" ] || { RBODYFAIL=1; fail "full-body $f != oracle (KS-AH-83-1)"; break; }
  done
done
[ $RBODYFAIL = 0 ] && echo "OK full-body battery vs oracle PASS on mem-census binary ($MEM_HASH)"
for f in $HFIX; do for _ in $(seq 1 5); do curl -s -m 10 -o /dev/null "http://127.0.0.1:$PORT/$f"; done; done
kill -TERM "$MPID" 2>/dev/null; wait "$MPID" 2>/dev/null
echo "mem_hash=$MEM_HASH git=$(git -C "$REPO" rev-parse --short HEAD) supplement=phaseR" >> "$MCFILE"
N=$(grep -c "tag=unitcache" "$MCFILE" || true)
echo "unitcache rows: $N"
[ "$N" -ge 1 ] || fail "no unitcache row even with the instrument armed"
[ "$FAILS" = 0 ] && echo "== PHASE-R SUPPLEMENT DONE ==" || echo "== PHASE-R SUPPLEMENT FAIL($FAILS) =="
exit $FAILS
