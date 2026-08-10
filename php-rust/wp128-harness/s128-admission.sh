#!/bin/bash
# s128-admission.sh — ARBITRO admission L-OL1 seg.2 (criterio
# s128-criterio-admission-seg2.md p.2, committato PRIMA di questo codice).
# ATTENDE compoff.done (run sequenziali, criterio p.5), poi: build census
# (patch clite wp119, target separato, tree ripristinato), R=2 conteggi per
# categoria micro-ORM al pin s127b, per-iter=(cat-empty)/N; collaudo differito
# della predizione F1 (-1,00 vs s127-admission). REGISTRAZIONE.
# rc autoritativo = SOLO admission-out/admission.rc scritto QUI (az.rev. #2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp128-harness"
M="$REPO/wp127-harness/micro-orm"
OUT="$H/admission-out"; mkdir -p "$OUT"
CTL="/Volumes/Extreme Pro/Claude/phpr-census-l1-target"
VERD="$H/s128-admission-verdetto.out"
fail(){ echo "$1" | tee -a "$VERD"; echo 1 > "$OUT/admission.rc"; git -C "$REPO" checkout -- . 2>/dev/null; exit 1; }

i=0; while [ ! -f "$H/compoff-out/compoff.done" ] && [ $i -lt 1080 ]; do /bin/sleep 20; i=$((i+1)); done
[ -f "$H/compoff-out/compoff.done" ] || fail "timeout attesa compoff.done"

cd "$REPO" || exit 2
git diff --quiet || fail "PRE: tree sporco — STOP"
[ "$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)" = "ccb63dcaf565cffc" ] || fail "pin!=s127b"
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
# atteso post-F1 = colonna "leva" di s127-admission2-verdetto.out (modello -2+w,
# misurato sul candidato S-127); il pin s127b deve riprodurlo ESATTO (collaudo differito)
EXP = {"objalloc": 7.0, "objallocni": 5.0, "objdatains": 12.0,
       "objdropdef": 8.0, "objmap": 1.0, "objchurn": 13.0}
def counters(cat, r):
    for line in open(f"{out}/cen-{cat}-r{r}.census"):
        if line.startswith("gacensus "):
            return {k: int(v) for k, v in
                    (kv.split("=") for kv in re.findall(r"\w+=\d+", line))}
    raise SystemExit(f"gacensus assente: {cat}-{r}")
print("== s128 admission L-OL1 seg.2: census alloc/free per-iter al pin s127b (REGISTRAZIONE, R=2 identici) ==")
print("grade=VERDICT  # conteggi deterministici; per-iter=(cat-empty)/3e6; rc autoritativo = admission-out/admission.rc")
rc = 0
got = {}
for cat in ["objalloc", "objallocni", "objdatains", "objdropdef", "objmap", "objchurn"]:
    per = []
    for r in (1, 2):
        c, e = counters(cat, r), counters("empty", r)
        per.append((round((c["galloc_n"]-e["galloc_n"])/N, 2),
                    round((c["gfree_n"]-e["gfree_n"])/N, 2)))
    if per[0] != per[1]:
        print(f"{cat}: R=2 NON identici {per} — NON ACQUISITA"); rc = 3; continue
    a, f = per[0]
    got[cat] = a
    exp = EXP[cat]
    tag = "F1-OK" if abs(a-exp) < 0.005 else f"F1-SCARTO(atteso {exp:.2f})"
    print(f"{cat:10s} alloc/iter={a:.2f} free/iter={f:.2f}  [{tag}]")
if rc == 0 and "objdatains" in got and "objalloc" in got:
    print(f"Dins_alloc = datains-alloc = {got['objdatains']-got['objalloc']:.2f} alloc/insert (atteso post-F1: 5.00; pre-F1 era 4.00)")
scarti = [c for c in got if abs(got[c]-EXP[c]) >= 0.005]
if scarti: print(f"F1-SCARTI: {scarti} — indagine PRIMA di procedere (criterio p.2)"); rc = rc or 4
print("ADMISSION-CENSUS: " + ("ACQUISITA" if rc == 0 else "NON ACQUISITA"))
sys.exit(rc)
PY
prc=$?
echo "$prc" > "$OUT/admission.rc"
exit "$prc"
