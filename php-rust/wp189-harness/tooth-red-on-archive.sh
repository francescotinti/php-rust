#!/bin/bash
# tooth-red-on-archive.sh — S-189, azione sulla revisione (rilievo 1): prova che il dente di CI COMMITTATO
# (`no_idle_worker_while_queue_nonempty` in php-server/src/axum_web, HEAD) è ROSSO sul tree C2 (5306ee88,
# dispatcher round-robin). Regola shell-harness: la patch si applica all'ARCHIVIO (`git archive`), mai al tree;
# target separata sulla bundle. Il test è copiato dal file di HEAD; la riga del lock `DEPTH_TEST_LOCK` (re-export
# nato in C3) è TOLTA nell'archivio e il test gira da solo (filtro): dichiarato. Esito appeso a s189-tooth-red-c2.out.
# Generato con assistenza AI (Claude Fable 5).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment"
H="$REPO/php-rust/wp189-harness"
OUT="$H/s189-tooth-red-c2.out"
BASE="5306ee88"
TMP="$HOME/Claude/phpr-target/archive-$BASE"
export CARGO_TARGET_DIR="$HOME/Claude/phpr-target/dev-output-red"
rm -rf "$TMP"; mkdir -p "$TMP"
git -C "$REPO" archive "$BASE" php-rust | tar -x -C "$TMP"
SRC_HEAD="$REPO/php-rust/crates/php-server/src/axum_web.rs"
DST="$TMP/php-rust/crates/php-server/src/axum_web.rs"
python3 - "$SRC_HEAD" "$DST" <<'PY'
import sys, re
head = open(sys.argv[1], encoding="utf-8").read()
dst_path = sys.argv[2]
dst = open(dst_path, encoding="utf-8").read()
m = re.search(r"\n    /// S-189 \(Council, signed 2026-11-03; Klabnik R5, Pedersen R9\).*?\n    #\[test\]\n    fn no_idle_worker_while_queue_nonempty\(\) \{.*?\n    \}\n", head, re.S)
assert m, "test non trovato in HEAD"
test = m.group(0)
lock = "        // the census counters are process-global: serialize with the pool tests (A-SK8)\n        let _guard = crate::worker_pool::DEPTH_TEST_LOCK.lock().unwrap();\n"
assert lock in test, "riga del lock non trovata"
test = test.replace(lock, "        // (archivio C2: il re-export DEPTH_TEST_LOCK non esiste ancora; il test gira da solo)\n")
anchor = "\n    #[test]\n    fn two_workers_no_request_bleed() {"
assert dst.count(anchor) == 1, "ancora non unica nell'archivio"
i = dst.index(anchor)
# insert BEFORE two_workers_no_request_bleed (end of previous item)
dst = dst[:i] + test.rstrip("\n") + "\n" + dst[i:]
open(dst_path, "w", encoding="utf-8").write(dst)
print("test inserito nell'archivio", dst_path)
PY
cd "$TMP/php-rust" || exit 2
{
  echo ""
  echo "# PROVA 2 (azione sulla revisione S-189 rilievo 1): il dente COMMITTATO (axum_web tests di HEAD $(git -C "$REPO" rev-parse --short HEAD)) sul tree C2 $BASE via git archive + target separata — $(date +%F_%H:%M:%S)"
  echo "cmd: CARGO_TARGET_DIR=$CARGO_TARGET_DIR cargo test --profile dev-release -p php-server --features axum-server no_idle_worker (nell'archivio; lock DEPTH_TEST_LOCK tolto: nato in C3)"
  cargo test --profile dev-release -p php-server --features axum-server no_idle_worker 2>&1 | awk '/^test |panicked|C answered|test result|^error/'
  echo "rc=${PIPESTATUS[0]} (atteso: 101 = ROSSO)"
} | tee -a "$OUT"
