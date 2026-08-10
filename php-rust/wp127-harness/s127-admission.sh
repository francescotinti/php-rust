#!/bin/bash
# s127-admission.sh — ARBITRO admission L-OL1 (s126-leva-nominata.md: census
# malloc/free per `new` PRIMA della forma). Build census (patch clite wp119,
# target separato, tree ripristinato), R=2 conteggi per categoria micro-ORM,
# per-iter = (cat - empty)/N. REGISTRAZIONE: nessuna predizione.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp127-harness"
M="$H/micro-orm"
OUT="$H/admission-out"; mkdir -p "$OUT"
CTL="/Volumes/Extreme Pro/Claude/phpr-census-l1-target"
VERD="$H/s127-admission-verdetto.out"
fail(){ echo "$1" | tee -a "$VERD"; echo 1 > "$OUT/admission.rc"; git -C "$REPO" checkout -- . 2>/dev/null; exit 1; }

cd "$REPO" || exit 2
git diff --quiet || fail "PRE: tree sporco — STOP"
[ "$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)" = "002e6cc12047ab9f" ] || fail "pin!=s125"
git apply wp119-harness/census-clite.patch || fail "census-clite.patch NON applica"
CARGO_TARGET_DIR="$CTL" SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 \
  cargo build --release -p php-cli --features mem-census \
  > "$OUT/build.log" 2>&1
brc=$?
git checkout -- . || fail "ripristino tree fallito"
git diff --quiet || fail "POST: tree NON pulito — STOP"
[ "$brc" = 0 ] || fail "build census FALLITA (build.log)"

LBIN="$CTL/release/phpr"
PIN="$HOME/Claude/php-rust-output/release/phpr"
for c in empty objalloc objallocni objdatains objdropdef objmap objchurn; do
  # parita' output censimento vs pin (admission: il binario census non diverge)
  "$LBIN" "$M/$c.php" > "$OUT/adm-$c-census.out" 2>&1
  "$PIN"  "$M/$c.php" > "$OUT/adm-$c-pin.out" 2>&1
  diff -q "$OUT/adm-$c-census.out" "$OUT/adm-$c-pin.out" > /dev/null \
    || fail "admission: output DIVERGE su $c"
  for r in 1 2; do
    f="$OUT/cen-$c-r$r.census"; rm -f "$f"
    PHPR_MEM_CENSUS="$f" MIMALLOC_PURGE_DELAY=0 "$LBIN" "$M/$c.php" > /dev/null 2>&1
    grep -q "^gacensus " "$f" || fail "gacensus ASSENTE in $c-r$r"
  done
done

python3 - "$OUT" > "$VERD" <<'PY'
import re, sys
out = sys.argv[1]
N = 3_000_000
def counters(cat, r):
    for line in open(f"{out}/cen-{cat}-r{r}.census"):
        if line.startswith("gacensus "):
            return {k: int(v) for k, v in
                    (kv.split("=") for kv in re.findall(r"\w+=\d+", line))}
    raise SystemExit(f"gacensus assente: {cat}-{r}")
print("== s127 admission L-OL1: census alloc/free per-iter (REGISTRAZIONE, R=2 identici) ==")
print("grade=VERDICT  # conteggi deterministici; per-iter = (cat-empty)/3e6")
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
    print(f"{cat:10s} alloc/iter={a:.2f} free/iter={f:.2f}")
print("ADMISSION-CENSUS: " + ("ACQUISITA" if rc == 0 else "NON ACQUISITA"))
sys.exit(rc)
PY
prc=$?
echo "$prc" > "$OUT/admission.rc"
exit "$prc"
