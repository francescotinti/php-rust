#!/bin/bash
# s125-cbargs-census.sh — ARBITRO admission leva cbargs (criterio p.4,
# committato PRIMA del run): build census+leva (tree corrente CON la leva
# committata), R=2 conteggi, confronto vs gamba A del controllo ±zval
# (stesso head SENZA leva). Predizioni: str alloc −1,00 ESATTO, resto 0,00.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp125-harness"
H119="$REPO/wp119-harness"
MICRO="$REPO/wp97-harness/micro"
OUT="$H/census-out"; mkdir -p "$OUT"
CTL="/Volumes/Extreme Pro/Claude/phpr-census-l1-target"
PIN="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s124"
VERD="$H/s125-cbargs-verdetto.out"
fail(){ echo "$1" | tee -a "$VERD"; echo 1 > "$OUT/cbargs-census.rc"; git -C "$REPO" checkout -- . 2>/dev/null; exit 1; }

cd "$REPO" || exit 2
git diff --quiet || fail "PRE: tree sporco — STOP"
[ "$(cat "$OUT/run.rc" 2>/dev/null)" = 0 ] || fail "controllo ±zval non concluso (run.rc) — baseline A mancante"
git apply "$H119/census-clite.patch" || fail "census-clite.patch NON applica"
CARGO_TARGET_DIR="$CTL" SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 \
  cargo build --release -p php-cli --features mem-census \
  > "$OUT/build-L.log" 2>&1 || fail "build census+leva FALLITA"
git checkout -- . || fail "ripristino tree fallito"
git diff --quiet || fail "POST: tree NON pulito — STOP"

LBIN="$CTL/release/phpr"
# parità output 7/7 vs pin (la leva non cambia la semantica)
for c in empty arith prop calls str arr re; do
  "$LBIN" "$MICRO/$c.php" > "$OUT/adm-$c-L.out" 2>&1
  diff -q "$OUT/adm-$c-L.out" "$OUT/adm-$c-pin.out" > /dev/null \
    || fail "admission cbargs: output DIVERGE su $c — SCARTATA"
  for r in 1 2; do
    f="$OUT/L-$c-r$r.census"; rm -f "$f"
    PHPR_MEM_CENSUS="$f" MIMALLOC_PURGE_DELAY=0 "$LBIN" "$MICRO/$c.php" > /dev/null 2>&1
    grep -q "^gacensus " "$f" || fail "gacensus ASSENTE in L-$c-r$r"
  done
done

python3 - "$OUT" >> "$VERD" <<'PY'
import re, sys
out = sys.argv[1]
N = {"arith": 50_000_000, "prop": 30_000_000, "calls": 20_000_000,
     "str": 4_000_000, "arr": 6_000_000, "re": 2_000_000}
def counters(side, cat, r):
    for line in open(f"{out}/{side}-{cat}-r{r}.census"):
        if line.startswith("gacensus "):
            return {k: int(v) for k, v in
                    (kv.split("=") for kv in re.findall(r"\w+=\d+", line))}
    raise SystemExit(f"gacensus assente: {side}-{cat}-r{r}")
def per_iter(side, cat):
    per = []
    for r in (1, 2):
        c, e = counters(side, cat, r), counters(side, "empty", r)
        per.append((round((c["galloc_n"]-e["galloc_n"])/N[cat], 2),
                    round((c["gfree_n"]-e["gfree_n"])/N[cat], 2)))
    if per[0] != per[1]:
        print(f"{side} {cat}: R=2 NON identici {per} — NON ACQUISITA"); sys.exit(3)
    return per[0]
print("== s125 admission cbargs: census L (leva) vs A (baseline stesso head) ==")
PRED = {"str": -1.00}   # alloc e free; altrove 0.00
TOL = {"arr": 0.10}
rc = 0
for cat in ["arith", "prop", "calls", "str", "arr", "re"]:
    la, lf = per_iter("L", cat)
    aa, af = per_iter("A", cat)
    da, df = round(la-aa, 2), round(lf-af, 2)
    tol = TOL.get(cat, 0.05)
    pred = PRED.get(cat, 0.00)
    ok = abs(da-pred) <= tol and abs(df-pred) <= tol
    if not ok: rc = 2
    print(f"{cat:6s} alloc/it A={aa:.2f} L={la:.2f} D={da:+.2f} | free/it A={af:.2f} L={lf:.2f} D={df:+.2f} | pred={pred:+.2f} tol={tol:.2f} -> {'OK' if ok else 'FUORI PREDIZIONE'}")
print("ADMISSION: " + ("PASS — str a PARITA' alloc con l'oracle (2,00)" if rc == 0 else "FUORI PREDIZIONE — diagnosi PRIMA dell'A/B (criterio p.4)"))
sys.exit(rc)
PY
prc=$?
echo "$prc" > "$OUT/cbargs-census.rc"
exit "$prc"
