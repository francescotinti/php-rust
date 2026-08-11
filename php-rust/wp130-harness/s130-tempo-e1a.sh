#!/bin/bash
# s130-tempo-e1a.sh — sonda E1a su build EMENDATA (criterio s130-criterio-e1a.md,
# committato PRIMA). Ricetta = s129-tempo-passi.sh (adattamento dichiarato:
# patch time-probes-e1a.patch sul tree F4, pin s130, segmenti 0/1/4/5/7).
# Applica la patch, builda su target separato, ripristina il tree, parità
# stdout vs pin, R=3 con PHPR_TIME_PROBES. Quote = MODELLO (mai promuovibili).
# rc autoritativo = SOLO tempo-out/tempo-e1a.rc scritto QUI.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp130-harness"
M="$REPO/wp127-harness/micro-orm"
P="$REPO/wp128-harness/probe"
OUT="$H/tempo-out"; mkdir -p "$OUT"
TTL="/Volumes/Extreme Pro/Claude/phpr-time-l1-target"
PIN="$HOME/Claude/php-rust-output/release/phpr"
VERD="$H/s130-tempo-e1a-verdetto.out"
fail(){ echo "$1" | tee -a "$VERD"; echo 1 > "$OUT/tempo-e1a.rc"; git -C "$REPO" checkout -- crates/ 2>/dev/null; exit 1; }

cd "$REPO" || exit 2
git diff --quiet -- crates/ || fail "PRE: tree sporco — STOP"
[ "$(shasum -a 256 "$PIN" | cut -c1-16)" = "0fdf1c49b16c24ba" ] || fail "pin!=s130"
[ -f "$H/timeprobes.rs" ] || fail "timeprobes.rs assente"
git apply "$H/time-probes-e1a.patch" || fail "time-probes-e1a.patch NON applica"
CARGO_TARGET_DIR="$TTL" SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 \
  cargo build --release -p php-cli > "$OUT/build-e1a.log" 2>&1
brc=$?
git checkout -- crates/ || fail "ripristino tree fallito"
git diff --quiet -- crates/ || fail "POST: tree NON pulito — STOP"
[ "$brc" = 0 ] || fail "build time-probes FALLITA (build-e1a.log)"

TBIN="$TTL/release/phpr"
src_of(){ case "$1" in p2-append|p3-local|p4-int|p5-two|p6-overwrite) echo "$P/$1.php";; *) echo "$M/$1.php";; esac; }
CATS="objalloc objdatains p2-append p5-two p6-overwrite"
for c in $CATS; do
  s="$(src_of "$c")"
  "$TBIN" "$s" > "$OUT/e1a-$c.out" 2>&1
  "$PIN"  "$s" > "$OUT/e1a-$c-pin.out" 2>&1
  diff -q "$OUT/e1a-$c.out" "$OUT/e1a-$c-pin.out" > /dev/null || fail "stdout DIVERGE su $c"
  for r in 1 2 3; do
    f="$OUT/e1a-$c-r$r.tprobes"; rm -f "$f"
    PHPR_TIME_PROBES="$f" MIMALLOC_PURGE_DELAY=0 "$TBIN" "$s" > /dev/null 2>&1
    if ! grep -q "^tprobes " "$f" 2>/dev/null; then
      # Lezione S-129 (a verbale): categoria a zero eventi non registra
      # l'atexit — SOLO objalloc riceve zeri sintetici, altrove FATALE.
      [ "$c" = objalloc ] || fail "tprobes ASSENTE in $c-r$r"
      echo "tprobes pid=0 freq=24000000 s0=0 c0=0 s1=0 c1=0 s2=0 c2=0 s3=0 c3=0 s4=0 c4=0 s5=0 c5=0 s6=0 c6=0 s7=0 c7=0" > "$f"
    fi
  done
done

python3 - "$OUT" > "$VERD" <<'PY'
import re, sys
out = sys.argv[1]
N = 3_000_000
SEGS = {0: "TOT", 1: "E1a_resolve", 4: "E_fieldset", 5: "E2_walk"}
STMT = {"objalloc": 0, "objdatains": 1, "p2-append": 1, "p5-two": 2, "p6-overwrite": 2}
def load(cat, r):
    line = open(f"{out}/e1a-{cat}-r{r}.tprobes").readline()
    freq = int(re.search(r"freq=(\d+)", line).group(1))
    s = [int(re.search(rf"s{i}=(\d+)", line).group(1)) for i in range(8)]
    c = [int(re.search(rf"c{i}=(\d+)", line).group(1)) for i in range(8)]
    return freq, s, c
print("== s130 sonda E1a (criterio s130-criterio-e1a.md; build EMENDATA: quote=MODELLO, nessuna cifra promuovibile) ==")
print("grade=MODEL  # rc autoritativo = tempo-out/tempo-e1a.rc; CAL sottratta per nota; R=3 mediane; pin s130")
rc = 0
for cat in STMT:
    per = []
    for r in (1, 2, 3):
        freq, s, c = load(cat, r)
        ns = 1e9 / freq
        cal = (s[7] / c[7]) if c[7] else 0.0          # ticks per coppia di note
        segns = {}
        for i in SEGS:
            raw = s[i] * ns / N
            over = (c[i] / N) * cal * ns              # 1 coppia di letture per nota
            segns[i] = raw - over
        per.append((segns, c))
    cnts = [tuple(x[1]) for x in per]
    if len(set(cnts)) != 1:
        print(f"{cat}: CONTEGGI NON deterministici tra R — indagine"); rc = 3
    cnt = per[0][1]
    k_att = STMT[cat]
    k_mis = round(cnt[0] / N, 4)
    tag = "CNT-OK" if abs(k_mis - k_att) < 0.001 else f"CNT-SCARTO(atteso {k_att})"
    if "SCARTO" in tag: rc = rc or 4
    k1 = cnt[1] / N                                    # resolve/iter (con eps di setup)
    med = {i: sorted(x[0][i] for x in per)[1] for i in SEGS}
    spread = {i: max(x[0][i] for x in per) - min(x[0][i] for x in per) for i in SEGS}
    print(f"-- {cat} (stmt/iter={k_mis:g} [{tag}]; resolve/iter k={k1:.4f}) --")
    freq, s, c = load(cat, 1)
    print(f"   cal_ticks/nota={(s[7]/c[7]) if c[7] else 0:.2f} freq={freq}")
    for i in SEGS:
        print(f"   {SEGS[i]:12s} {med[i]:8.1f} ns/iter  (spread R=3 {spread[i]:.1f})")
    if med[0] > 0:
        e1 = med[4] - med[5]
        print(f"   E-E2 (dispatch+prop_step) = {e1:.1f} ns/iter; E1a/(E-E2) = {med[1]/e1*100 if e1>0 else 0:.0f}%")
        if k_att > 0 and k1 > k_att:
            ub = med[1] * (k1 - k_att) / k1
            print(f"   UB resolve-once (collasso a UNA resolve/statement: {k1:.2f}->{k_att}) = {ub:.1f} ns/iter")
print("SONDA-E1A: " + ("ACQUISITA" if rc == 0 else f"NON ACQUISITA (rc={rc})"))
sys.exit(rc)
PY
prc=$?
echo "$prc" > "$OUT/tempo-e1a.rc"
echo "done rc=$prc $(date +%T)" > "$OUT/tempo-e1a.done"
exit "$prc"
