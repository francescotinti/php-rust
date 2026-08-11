#!/bin/bash
# s129-tempo-passi.sh — sonde per-passo su build EMENDATA (criterio p.4).
# Applica time-probes.patch (modulo wp129-harness/timeprobes.rs via #[path]),
# builda su target separato, ripristina il tree, parità stdout vs pin, poi R=3
# run con PHPR_TIME_PROBES per {objalloc, objdatains, p2-append, p5-two,
# p6-overwrite}. Quote = MODELLO (mai cifre promuovibili). CAL sottratta.
# rc autoritativo = SOLO tempo-out/tempo-passi.rc scritto QUI.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp129-harness"
M="$REPO/wp127-harness/micro-orm"
P="$REPO/wp128-harness/probe"
OUT="$H/tempo-out"; mkdir -p "$OUT"
TTL="/Volumes/Extreme Pro/Claude/phpr-time-l1-target"
PIN="$HOME/Claude/php-rust-output/release/phpr"
VERD="$H/s129-tempo-passi-verdetto.out"
fail(){ echo "$1" | tee -a "$VERD"; echo 1 > "$OUT/tempo-passi.rc"; git -C "$REPO" checkout -- . 2>/dev/null; exit 1; }

cd "$REPO" || exit 2
git diff --quiet || fail "PRE: tree sporco — STOP"
[ "$(shasum -a 256 "$PIN" | cut -c1-16)" = "ccb63dcaf565cffc" ] || fail "pin!=s127b"
[ -f "$H/timeprobes.rs" ] || fail "timeprobes.rs assente"
git apply wp129-harness/time-probes.patch || fail "time-probes.patch NON applica"
CARGO_TARGET_DIR="$TTL" SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 \
  cargo build --release -p php-cli > "$OUT/build-tp.log" 2>&1
brc=$?
git checkout -- . || fail "ripristino tree fallito"
git diff --quiet || fail "POST: tree NON pulito — STOP"
[ "$brc" = 0 ] || fail "build time-probes FALLITA (build-tp.log)"

TBIN="$TTL/release/phpr"
src_of(){ case "$1" in p2-append|p3-local|p4-int|p5-two|p6-overwrite) echo "$P/$1.php";; *) echo "$M/$1.php";; esac; }
CATS="objalloc objdatains p2-append p5-two p6-overwrite"
for c in $CATS; do
  s="$(src_of "$c")"
  "$TBIN" "$s" > "$OUT/tp-$c.out" 2>&1
  "$PIN"  "$s" > "$OUT/tp-$c-pin.out" 2>&1
  diff -q "$OUT/tp-$c.out" "$OUT/tp-$c-pin.out" > /dev/null || fail "stdout DIVERGE su $c"
  for r in 1 2 3; do
    f="$OUT/tp-$c-r$r.tprobes"; rm -f "$f"
    PHPR_TIME_PROBES="$f" MIMALLOC_PURGE_DELAY=0 "$TBIN" "$s" > /dev/null 2>&1
    if ! grep -q "^tprobes " "$f" 2>/dev/null; then
      # EMENDA dichiarata (morso d'apparato S-129): una categoria con ZERO
      # statement FieldAssign non chiama mai note() ⇒ l'atexit non si
      # registra e il dump manca. SOLO objalloc (atteso 0) riceve zeri
      # sintetici; per ogni altra categoria il file assente resta FATALE.
      [ "$c" = objalloc ] || fail "tprobes ASSENTE in $c-r$r"
      echo "tprobes pid=0 freq=24000000 s0=0 c0=0 s1=0 c1=0 s2=0 c2=0 s3=0 c3=0 s4=0 c4=0 s5=0 c5=0 s6=0 c6=0 s7=0 c7=0" > "$f"
    fi
  done
done

python3 - "$OUT" > "$VERD" <<'PY'
import re, sys
out = sys.argv[1]
N = 3_000_000
SEGS = ["TOT", "B_byref", "C_indirect", "D_lazy", "E_fieldset", "E2_walk", "A_popkeys", "CAL"]
STMT = {"objalloc": 0, "objdatains": 1, "p2-append": 1, "p5-two": 2, "p6-overwrite": 2}
def load(cat, r):
    line = open(f"{out}/tp-{cat}-r{r}.tprobes").readline()
    freq = int(re.search(r"freq=(\d+)", line).group(1))
    s = [int(re.search(rf"s{i}=(\d+)", line).group(1)) for i in range(8)]
    c = [int(re.search(rf"c{i}=(\d+)", line).group(1)) for i in range(8)]
    return freq, s, c
print("== s129 sonde per-passo (criterio p.4; build EMENDATA: quote=MODELLO, nessuna cifra promuovibile) ==")
print("grade=MODEL  # rc autoritativo = tempo-out/tempo-passi.rc; CAL sottratta per nota; R=3 mediane")
res = {}
rc = 0
for cat in STMT:
    per = []
    for r in (1, 2, 3):
        freq, s, c = load(cat, r)
        ns = 1e9 / freq
        cal = (s[7] / c[7]) if c[7] else 0.0          # ticks per coppia di note
        segns = []
        for i in range(7):
            raw = s[i] * ns / N
            over = (c[i] / N) * cal * ns              # 1 coppia di letture per nota
            segns.append(raw - over)
        per.append((segns, [c[i] for i in range(8)]))
    cnts = [tuple(x[1]) for x in per]
    if len(set(cnts)) != 1:
        print(f"{cat}: CONTEGGI NON deterministici tra R — indagine"); rc = 3
    cnt = per[0][1]
    k_att = STMT[cat]
    k_mis = round(cnt[0] / N, 4)
    tag = "CNT-OK" if abs(k_mis - k_att) < 0.001 else f"CNT-SCARTO(atteso {k_att})"
    if "SCARTO" in tag: rc = rc or 4
    med = [sorted(x[0][i] for x in per)[1] for i in range(7)]
    spread = [max(x[0][i] for x in per) - min(x[0][i] for x in per) for i in range(7)]
    res[cat] = med
    print(f"-- {cat} (stmt/iter={k_mis:g} [{tag}]) --")
    freq, s, c = load(cat, 1)
    print(f"   cal_ticks/nota={ (s[7]/c[7]) if c[7] else 0:.2f} freq={freq}")
    for i in range(7):
        print(f"   {SEGS[i]:11s} {med[i]:8.1f} ns/iter  (spread R=3 {spread[i]:.1f})")
    somma = med[6] + med[1] + med[2] + med[3] + med[4]
    if med[0] > 0:
        print(f"   chiusura: A+B+C+D+E={somma:.1f} vs TOT={med[0]:.1f} ({somma/med[0]*100:.0f}% — banda 85-115%)")
        if not (0.85 <= somma / med[0] <= 1.15): print("   MODELLO NON CHIUSO su questa categoria"); rc = rc or 5
print("-- differenze per-statement (stessa build, ns/iter) --")
def diff(a, b, tag):
    if a in res and b in res:
        d = [res[a][i] - res[b][i] for i in range(7)]
        print(f"{tag}: " + " ".join(f"{SEGS[i]}={d[i]:+.1f}" for i in range(7)))
diff("objdatains", "objalloc", "insert'k' (datains-alloc)")
diff("p5-two", "objdatains", "2o-insert (p5-datains)")
diff("p6-overwrite", "objdatains", "overwrite (p6-datains)")
diff("p2-append", "objalloc", "append    (p2-alloc)")
print("MODELLO-PASSI: " + ("ACQUISITO" if rc == 0 else f"NON ACQUISITO (rc={rc})"))
sys.exit(rc)
PY
prc=$?
echo "$prc" > "$OUT/tempo-passi.rc"
echo "done rc=$prc $(date +%T)" > "$OUT/tempo-passi.done"
exit "$prc"
