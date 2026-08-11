#!/bin/bash
# s128-probes-census.sh — census differenziale dei passi dell'insert
# (criterio admission-seg2 p.6: «forme candidate DA CENSIRE»). Usa il binario
# census gia' costruito dall'admission (stesso target, stessa patch); parita'
# output vs pin per ogni sonda; R=2 identici; per-iter=(sonda-empty)/3e6.
# rc autoritativo = SOLO admission-out/probes.rc scritto QUI (az.rev. #2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp128-harness"
OUT="$H/admission-out"
CTL="/Volumes/Extreme Pro/Claude/phpr-census-l1-target"
LBIN="$CTL/release/phpr"
PIN="$HOME/Claude/php-rust-output/release/phpr"
VERD="$H/s128-probes-verdetto.out"
fail(){ echo "$1" | tee -a "$VERD"; echo 1 > "$OUT/probes.rc"; exit 1; }
[ "$(shasum -a 256 "$PIN" | cut -c1-16)" = "ccb63dcaf565cffc" ] || fail "pin!=s127b"
[ -x "$LBIN" ] || fail "binario census assente (admission non eseguita?)"
for c in p2-append p3-local p4-int p5-two p6-overwrite; do
  "$LBIN" "$H/probe/$c.php" > "$OUT/prb-$c-census.out" 2>&1
  "$PIN"  "$H/probe/$c.php" > "$OUT/prb-$c-pin.out" 2>&1
  diff -q "$OUT/prb-$c-census.out" "$OUT/prb-$c-pin.out" > /dev/null \
    || fail "probe: output DIVERGE su $c"
  for r in 1 2; do
    f="$OUT/cen-$c-r$r.census"; rm -f "$f"
    PHPR_MEM_CENSUS="$f" MIMALLOC_PURGE_DELAY=0 "$LBIN" "$H/probe/$c.php" > /dev/null 2>&1
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
print("== s128 probes census: passi dell'insert su proprieta' (REGISTRAZIONE, R=2 identici) ==")
print("grade=VERDICT  # conteggi deterministici; per-iter=(sonda-empty)/3e6; rc autoritativo = admission-out/probes.rc")
print("# riferimenti dallo stesso census: objalloc=7.00, objdatains=12.00 (s128-admission-verdetto.out)")
rc = 0
for cat in ["p2-append", "p3-local", "p4-int", "p5-two", "p6-overwrite"]:
    per = []
    for r in (1, 2):
        c, e = counters(cat, r), counters("empty", r)
        per.append((round((c["galloc_n"]-e["galloc_n"])/N, 2),
                    round((c["gfree_n"]-e["gfree_n"])/N, 2)))
    if per[0] != per[1]:
        print(f"{cat}: R=2 NON identici {per} — NON ACQUISITA"); rc = 3; continue
    a, f = per[0]
    print(f"{cat:12s} alloc/iter={a:.2f} free/iter={f:.2f}  (delta vs objalloc: {a-7.0:+.2f})")
print("PROBES-CENSUS: " + ("ACQUISITA" if rc == 0 else "NON ACQUISITA"))
sys.exit(rc)
PY
prc=$?
echo "$prc" > "$OUT/probes.rc"
exit "$prc"
