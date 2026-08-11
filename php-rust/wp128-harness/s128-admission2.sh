#!/bin/bash
# s128-admission2.sh — ARBITRO census della FORMA F2 (predizioni pre-registrate
# in s128-admission-lettura.md c9f5950, PRIMA del codice della forma 31ceca9).
# Tree = HEAD con F2; patch census applicata sopra e RIMOSSA a fine atto.
# rc autoritativo = SOLO admission-out/admission2.rc scritto QUI (az.rev. #2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp128-harness"
M="$REPO/wp127-harness/micro-orm"
OUT="$H/admission-out"; mkdir -p "$OUT"
CTL="/Volumes/Extreme Pro/Claude/phpr-census-l1-target"
VERD="$H/s128-admission2-verdetto.out"
fail(){ echo "$1" | tee -a "$VERD"; echo 1 > "$OUT/admission2.rc"; git -C "$REPO" checkout -- . 2>/dev/null; exit 1; }

cd "$REPO" || exit 2
git diff --quiet || fail "PRE: tree sporco — STOP"
[ "$(git rev-parse --short=7 HEAD)" = "31ceca9" ] || fail "HEAD != forma F2 31ceca9"
git apply wp119-harness/census-clite.patch || fail "census-clite.patch NON applica"
CARGO_TARGET_DIR="$CTL" SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 \
  cargo build --release -p php-cli --features mem-census \
  > "$OUT/build2.log" 2>&1
brc=$?
git checkout -- . || fail "ripristino tree fallito"
git diff --quiet || fail "POST: tree NON pulito — STOP"
[ "$brc" = 0 ] || fail "build census F2 FALLITA (build2.log)"

LBIN="$CTL/release/phpr"
PIN="$HOME/Claude/php-rust-output/release/phpr"
for c in empty objalloc objallocni objdatains objdropdef objmap objchurn; do
  "$LBIN" "$M/$c.php" > "$OUT/adm2-$c-census.out" 2>&1
  "$PIN"  "$M/$c.php" > "$OUT/adm2-$c-pin.out" 2>&1
  diff -q "$OUT/adm2-$c-census.out" "$OUT/adm2-$c-pin.out" > /dev/null \
    || fail "admission2: output DIVERGE su $c"
  for r in 1 2; do
    f="$OUT/cen2-$c-r$r.census"; rm -f "$f"
    PHPR_MEM_CENSUS="$f" MIMALLOC_PURGE_DELAY=0 "$LBIN" "$M/$c.php" > /dev/null 2>&1
    grep -q "^gacensus " "$f" || fail "gacensus ASSENTE in $c-r$r"
  done
done
for c in p2-append p3-local p4-int p5-two p6-overwrite; do
  "$LBIN" "$H/probe/$c.php" > "$OUT/adm2-$c-census.out" 2>&1
  "$PIN"  "$H/probe/$c.php" > "$OUT/adm2-$c-pin.out" 2>&1
  diff -q "$OUT/adm2-$c-census.out" "$OUT/adm2-$c-pin.out" > /dev/null \
    || fail "admission2: output DIVERGE su $c"
  for r in 1 2; do
    f="$OUT/cen2-$c-r$r.census"; rm -f "$f"
    PHPR_MEM_CENSUS="$f" MIMALLOC_PURGE_DELAY=0 "$LBIN" "$H/probe/$c.php" > /dev/null 2>&1
    grep -q "^gacensus " "$f" || fail "gacensus ASSENTE in $c-r$r"
  done
done

python3 - "$OUT" > "$VERD" <<'PY'
import re, sys
out = sys.argv[1]
N = 3_000_000
# predizioni F2 pre-registrate (s128-admission-lettura.md, commit c9f5950)
EXP = {"objalloc": 7.0, "objallocni": 5.0, "objdatains": 11.0, "objdropdef": 8.0,
       "objmap": 1.0, "objchurn": 12.0, "p2-append": 11.0, "p3-local": 10.0,
       "p4-int": 11.0, "p5-two": 13.0, "p6-overwrite": 14.0}
def counters(cat, r):
    for line in open(f"{out}/cen2-{cat}-r{r}.census"):
        if line.startswith("gacensus "):
            return {k: int(v) for k, v in
                    (kv.split("=") for kv in re.findall(r"\w+=\d+", line))}
    raise SystemExit(f"gacensus assente: {cat}-{r}")
print("== s128 admission2 FORMA F2: census leva vs predizioni (pre-registrate c9f5950) ==")
print("grade=VERDICT  # conteggi deterministici; per-iter=(cat-empty)/3e6; rc autoritativo = admission-out/admission2.rc")
rc = 0
for cat in ["objalloc", "objallocni", "objdatains", "objdropdef", "objmap", "objchurn",
            "p2-append", "p3-local", "p4-int", "p5-two", "p6-overwrite"]:
    per = []
    for r in (1, 2):
        c, e = counters(cat, r), counters("empty", r)
        per.append((round((c["galloc_n"]-e["galloc_n"])/N, 2),
                    round((c["gfree_n"]-e["gfree_n"])/N, 2)))
    if per[0] != per[1]:
        print(f"{cat}: R=2 NON identici {per} — NON ACQUISITA"); rc = 3; continue
    a, f = per[0]
    tag = "PRED-OK" if abs(a-EXP[cat]) < 0.005 else f"FUORI PREDIZIONE (atteso {EXP[cat]:.2f})"
    if abs(a-EXP[cat]) >= 0.005: rc = rc or 4
    print(f"{cat:12s} alloc/iter={a:.2f} free/iter={f:.2f}  [{tag}]")
print("ADMISSION2-F2: " + ("PREDIZIONI CONFERMATE — A/B AMMESSO" if rc == 0 else "FUORI PREDIZIONE — diagnosi PRIMA dell'A/B"))
sys.exit(rc)
PY
prc=$?
echo "$prc" > "$OUT/admission2.rc"
exit "$prc"
