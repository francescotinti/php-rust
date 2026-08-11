#!/bin/bash
# s129-census-f4.sh — ARBITRO census F4 (criterio s129-criterio-f4.md p.2,
# committato PRIMA della forma): clite.patch sull'albero CON F4 committata
# (f4143a6), build census, R=2 identici; predizioni ESATTE = colonna B del
# s129-census-verdetto. rc autoritativo = SOLO census-out/census-f4.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp129-harness"
M="$REPO/wp127-harness/micro-orm"
P="$REPO/wp128-harness/probe"
OUT="$H/census-out"; mkdir -p "$OUT"
CTL="/Volumes/Extreme Pro/Claude/phpr-census-l1-target"
PIN="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s127b"
VERD="$H/s129-census-f4-verdetto.out"
fail(){ echo "$1" | tee -a "$VERD"; echo 1 > "$OUT/census-f4.rc"; git -C "$REPO" checkout -- . 2>/dev/null; exit 1; }

cd "$REPO" || exit 2
git diff --quiet || fail "PRE: tree sporco — STOP"
[ "$(shasum -a 256 "$PIN" | cut -c1-8)" = "ccb63dca" ] || fail "stash A != s127b"
git apply wp119-harness/census-clite.patch || fail "clite.patch NON applica su F4"
CARGO_TARGET_DIR="$CTL" SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 \
  cargo build --release -p php-cli --features mem-census > "$OUT/build-f4.log" 2>&1
brc=$?
git checkout -- . || fail "ripristino tree fallito"
git diff --quiet || fail "POST: tree NON pulito — STOP"
[ "$brc" = 0 ] || fail "build census-F4 FALLITA (build-f4.log)"

LBIN="$CTL/release/phpr"
src_of(){ case "$1" in p2-append|p3-local|p4-int|p5-two|p6-overwrite) echo "$P/$1.php";; *) echo "$M/$1.php";; esac; }
for c in empty objalloc objallocni objdatains objdropdef objmap objchurn p2-append p3-local p4-int p5-two p6-overwrite; do
  s="$(src_of "$c")"
  "$LBIN" "$s" > "$OUT/F4-$c.out" 2>&1
  "$PIN"  "$s" > "$OUT/F4pin-$c.out" 2>&1
  diff -q "$OUT/F4-$c.out" "$OUT/F4pin-$c.out" > /dev/null || fail "F4: stdout DIVERGE su $c"
  for r in 1 2; do
    f="$OUT/cen-F4-$c-r$r.census"; rm -f "$f"
    PHPR_MEM_CENSUS="$f" MIMALLOC_PURGE_DELAY=0 "$LBIN" "$s" > /dev/null 2>&1
    grep -q "^gacensus " "$f" || fail "gacensus ASSENTE in F4-$c-r$r"
  done
done

python3 - "$OUT" > "$VERD" <<'PY'
import re, sys
out = sys.argv[1]
N = 3_000_000
VAR = {"objalloc": 7.0, "objallocni": 5.0, "objdatains": 10.0, "objdropdef": 8.0,
       "objmap": 1.0, "objchurn": 11.0, "p2-append": 9.0, "p3-local": 10.0,
       "p4-int": 10.0, "p5-two": 11.0, "p6-overwrite": 12.0}
def counters(cat, r):
    for line in open(f"{out}/cen-F4-{cat}-r{r}.census"):
        if line.startswith("gacensus "):
            return {k: int(v) for k, v in (kv.split("=") for kv in re.findall(r"\w+=\d+", line))}
    raise SystemExit(f"gacensus assente: F4-{cat}-{r}")
print("== s129 census F4 prelude-gate (criterio f4 p.2; predizioni = colonna B name-borrow) ==")
print("grade=VERDICT  # conteggi deterministici; rc autoritativo = census-out/census-f4.rc")
rc = 0
for cat in VAR:
    per = []
    for r in (1, 2):
        c, e = counters(cat, r), counters("empty", r)
        per.append((round((c["galloc_n"]-e["galloc_n"])/N, 2), round((c["gfree_n"]-e["gfree_n"])/N, 2)))
    if per[0] != per[1]:
        print(f"{cat}: R=2 NON identici {per} — NON ACQUISITA"); rc = 3; continue
    a, f = per[0]
    tag = "PRED-OK" if abs(a-VAR[cat]) < 0.005 else f"PRED-SCARTO(atteso {VAR[cat]:.2f})"
    if "SCARTO" in tag: rc = rc or 4
    print(f"{cat:12s} alloc/iter={a:.2f} free/iter={f:.2f}  [{tag}]")
print("CENSUS-F4: " + ("11/11 PRED-OK — A/B AMMESSO" if rc == 0 else "SCARTO — STOP (criterio p.2)"))
sys.exit(rc)
PY
prc=$?
echo "$prc" > "$OUT/census-f4.rc"
echo "done rc=$prc $(date +%T)" > "$OUT/census-f4.done"
exit "$prc"
