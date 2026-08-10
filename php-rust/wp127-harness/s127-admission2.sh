#!/bin/bash
# s127-admission2.sh — ARBITRO admission della FORMA F1 (criterio s127-criterio-ab.md
# p.1, committato PRIMA del run): build census sul tree-LEVA (clean), R=2 conteggi,
# delta vs baseline s127-admission-verdetto.out. Predizioni: alloc/free −1,00 ESATTO
# (tol 0,05) su objalloc/objallocni/objdatains/objdropdef/objchurn; objmap 0,00.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp127-harness"
M="$H/micro-orm"
OUT="$H/admission-out"; mkdir -p "$OUT"
CTL="/Volumes/Extreme Pro/Claude/phpr-census-l1-target"
VERD="$H/s127-admission2-verdetto.out"
LEVER="${LEVER:?path binario leva}"          # release/phpr con la leva
LEXP="${LEXP:?hash8 atteso leva}"
fail(){ echo "$1" | tee -a "$VERD"; echo 1 > "$OUT/admission2.rc"; git -C "$REPO" checkout -- . 2>/dev/null; exit 1; }

cd "$REPO" || exit 2
git diff --quiet || fail "PRE: tree sporco — STOP"
[ "$(shasum -a 256 "$LEVER" | cut -c1-8)" = "$LEXP" ] || fail "LEVER != $LEXP"
git apply wp119-harness/census-clite.patch || fail "census-clite.patch NON applica"
CARGO_TARGET_DIR="$CTL" SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 \
  cargo build --release -p php-cli --features mem-census \
  > "$OUT/build2.log" 2>&1
brc=$?
git checkout -- . || fail "ripristino tree fallito"
git diff --quiet || fail "POST: tree NON pulito — STOP"
[ "$brc" = 0 ] || fail "build census-leva FALLITA (build2.log)"

LBIN="$CTL/release/phpr"
for c in empty objalloc objallocni objdatains objdropdef objmap objchurn; do
  "$LBIN" "$M/$c.php" > "$OUT/adm2-$c-census.out" 2>&1
  "$LEVER" "$M/$c.php" > "$OUT/adm2-$c-lever.out" 2>&1
  diff -q "$OUT/adm2-$c-census.out" "$OUT/adm2-$c-lever.out" > /dev/null \
    || fail "admission2: output census DIVERGE da leva su $c"
  diff -q "$OUT/adm2-$c-lever.out" "$OUT/adm-$c-pin.out" > /dev/null \
    || fail "admission2: output leva DIVERGE dal PIN su $c — SCARTATA"
  for r in 1 2; do
    f="$OUT/cen2-$c-r$r.census"; rm -f "$f"
    PHPR_MEM_CENSUS="$f" MIMALLOC_PURGE_DELAY=0 "$LBIN" "$M/$c.php" > /dev/null 2>&1
    grep -q "^gacensus " "$f" || fail "gacensus ASSENTE in $c-r$r"
  done
done

python3 - "$OUT" "$H/s127-admission-verdetto.out" > "$VERD" <<'PY'
import re, sys
out, basef = sys.argv[1], sys.argv[2]
N = 3_000_000
base = {}
for line in open(basef):
    m = re.match(r"(\w+)\s+alloc/iter=([\d.]+) free/iter=([\d.]+)", line)
    if m:
        base[m.group(1)] = (float(m.group(2)), float(m.group(3)))
def counters(cat, r):
    for line in open(f"{out}/cen2-{cat}-r{r}.census"):
        if line.startswith("gacensus "):
            return {k: int(v) for k, v in
                    (kv.split("=") for kv in re.findall(r"\w+=\d+", line))}
    raise SystemExit(f"gacensus assente: {cat}-{r}")
print("== s127 admission2 FORMA F1: census leva vs baseline (pred -1,00; objmap 0,00) ==")
print("grade=VERDICT  # conteggi deterministici; per-iter = (cat-empty)/3e6")
PRED = {"objmap": 0.00}
rc = 0
for cat in ["objalloc", "objallocni", "objdatains", "objdropdef", "objmap", "objchurn"]:
    per = []
    for r in (1, 2):
        c, e = counters(cat, r), counters("empty", r)
        per.append((round((c["galloc_n"]-e["galloc_n"])/N, 2),
                    round((c["gfree_n"]-e["gfree_n"])/N, 2)))
    if per[0] != per[1]:
        print(f"{cat}: R=2 NON identici {per} — NON ACQUISITA"); rc = 3; continue
    a, f = per[0]
    ba, bf = base[cat]
    da, df = round(a-ba, 2), round(f-bf, 2)
    pred = PRED.get(cat, -1.00)
    ok = abs(da-pred) <= 0.05 and abs(df-pred) <= 0.05
    if not ok: rc = 2
    print(f"{cat:10s} alloc/it base={ba:.2f} leva={a:.2f} D={da:+.2f} | free/it base={bf:.2f} leva={f:.2f} D={df:+.2f} | pred={pred:+.2f} -> {'OK' if ok else 'FUORI PREDIZIONE'}")
print("ADMISSION2: " + ("PASS" if rc == 0 else "FUORI PREDIZIONE — diagnosi PRIMA dell'A/B"))
sys.exit(rc)
PY
prc=$?
echo "$prc" > "$OUT/admission2.rc"
exit "$prc"
