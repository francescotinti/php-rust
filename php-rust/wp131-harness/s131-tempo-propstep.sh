#!/bin/bash
# s131-tempo-propstep.sh — modello prop_step su build EMENDATA (criterio
# s131-criterio-propstep.md, committato PRIMA). Ricetta = s130-tempo-e1a.sh
# (adattamento dichiarato: patch time-probes-propstep.patch, NSEG=12,
# segmenti 0/1/2/3/4/5/6/7/8/9/10/11, quiescenza gate SEPARATO con rc in
# header, overhead sonde annidate sottratto dai conteggi).
# Quote = MODELLO (mai promuovibili). rc autoritativo = SOLO
# tempo-out/tempo-propstep.rc scritto QUI.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp131-harness"
M="$REPO/wp127-harness/micro-orm"
P="$REPO/wp128-harness/probe"
OUT="$H/tempo-out"; mkdir -p "$OUT"
TTL="/Volumes/Extreme Pro/Claude/phpr-time-l1-target"
PIN="$HOME/Claude/php-rust-output/release/phpr"
VERD="$H/s131-tempo-propstep-verdetto.out"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
fail(){ echo "$1" | tee -a "$VERD"; echo 1 > "$OUT/tempo-propstep.rc"; git -C "$REPO" checkout -- crates/ 2>/dev/null; exit 1; }

cd "$REPO" || exit 2
# Quiescenza gate SEPARATO (criterio p.6): rc nel suo file, citato in header.
"$QUIESCE" "$OUT/quiesce-propstep.rc" > "$OUT/quiesce-propstep.log" 2>&1 \
  || fail "quiescenza FALLITA (vedi tempo-out/quiesce-propstep.log)"
git diff --quiet -- crates/ || fail "PRE: tree sporco — STOP"
[ "$(shasum -a 256 "$PIN" | cut -c1-16)" = "0fdf1c49b16c24ba" ] || fail "pin!=s130"
[ -f "$H/timeprobes.rs" ] || fail "timeprobes.rs assente"
git apply "$H/time-probes-propstep.patch" || fail "time-probes-propstep.patch NON applica"
CARGO_TARGET_DIR="$TTL" SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 \
  cargo build --release -p php-cli > "$OUT/build-propstep.log" 2>&1
brc=$?
git checkout -- crates/ || fail "ripristino tree fallito"
git diff --quiet -- crates/ || fail "POST: tree NON pulito — STOP"
[ "$brc" = 0 ] || fail "build time-probes FALLITA (build-propstep.log)"

TBIN="$TTL/release/phpr"
src_of(){ case "$1" in p2-append|p3-local|p4-int|p5-two|p6-overwrite) echo "$P/$1.php";; *) echo "$M/$1.php";; esac; }
CATS="objalloc objdatains p2-append p5-two p6-overwrite"
for c in $CATS; do
  s="$(src_of "$c")"
  "$TBIN" "$s" > "$OUT/ps-$c.out" 2>&1
  "$PIN"  "$s" > "$OUT/ps-$c-pin.out" 2>&1
  diff -q "$OUT/ps-$c.out" "$OUT/ps-$c-pin.out" > /dev/null || fail "stdout DIVERGE su $c"
  for r in 1 2 3; do
    f="$OUT/ps-$c-r$r.tprobes"; rm -f "$f"
    PHPR_TIME_PROBES="$f" MIMALLOC_PURGE_DELAY=0 "$TBIN" "$s" > /dev/null 2>&1
    if ! grep -q "^tprobes " "$f" 2>/dev/null; then
      # objalloc: 0 statement ⇒ zeri sintetici SOLO se anche il ctor non nota
      # (qui il ctor NOTA su seg11 ⇒ atexit presente; il fallback resta per
      # coerenza di ricetta, altrove FATALE).
      [ "$c" = objalloc ] || fail "tprobes ASSENTE in $c-r$r"
      printf 'tprobes pid=0 freq=1000000000' > "$f"
      for i in 0 1 2 3 4 5 6 7 8 9 10 11; do printf ' s%d=0 c%d=0' "$i" "$i" >> "$f"; done
      printf '\n' >> "$f"
    fi
  done
done

python3 - "$OUT" > "$VERD" <<'PY'
import re, sys
out = sys.argv[1]
N = 3_000_000
NSEG = 12
SEGS = {0: "TOT", 4: "E_fieldset", 5: "E2_walk", 1: "prop_step_INTERO",
        2: "borrow", 3: "guardie", 6: "defer_check", 8: "key+container_op",
        9: "resolve@prop_key", 10: "resolve@prop_key_read", 11: "resolve_TUTTE"}
STMT = {"objalloc": 0, "objdatains": 1, "p2-append": 1, "p5-two": 2, "p6-overwrite": 2}
# nesting statico (note INTERNE per statement, cammino intermedio; criterio p.5):
# seg3⊃{9,11}=2 · seg6⊃{9,11,10,11}=4 · seg8⊃{9,11}=2 · seg9⊃{11}=1 · seg10⊃{11}=1
# seg1⊃{2,3,6,8,5, 9x3,10, 11x5}=14 · seg4⊃seg1+{1}=15 · seg0⊃seg4+{4,7}=17
INNER_PER_STMT = {3: 2, 6: 4, 8: 2, 9: 3, 10: 1, 1: 14, 4: 15, 0: 17, 2: 0, 5: 0, 11: 0}
def load(cat, r):
    line = open(f"{out}/ps-{cat}-r{r}.tprobes").readline()
    freq = int(re.search(r"freq=(\d+)", line).group(1))
    s = [int(re.search(rf"s{i}=(\d+)", line).group(1)) for i in range(NSEG)]
    c = [int(re.search(rf"c{i}=(\d+)", line).group(1)) for i in range(NSEG)]
    return freq, s, c
print("== s131 modello prop_step (criterio s131-criterio-propstep.md; build EMENDATA: quote=MODELLO) ==")
print("grade=MODEL  # rc autoritativo = tempo-out/tempo-propstep.rc; overhead annidato sottratto dai conteggi; R=3 mediane; pin s130")
qrc = open(f"{out}/quiesce-propstep.rc").read().strip()
print(f"quiescenza: rc={qrc} (file: tempo-out/quiesce-propstep.rc)")
rc = 0
ctor_k = None
for cat in STMT:
    per = []
    for r in (1, 2, 3):
        freq, s, c = load(cat, r)
        ns = 1e9 / freq
        cal = (s[7] / c[7]) if c[7] else 0.0
        k_st = STMT[cat]
        segns = {}
        for i in SEGS:
            raw = s[i] * ns / N
            own = (c[i] / N) * cal * ns
            inner = INNER_PER_STMT.get(i, 0) * k_st * cal * ns if i != 9 else 1 * (c[9]/N) * cal * ns
            segns[i] = raw - own - inner
        per.append((segns, c))
    cnts = [tuple(x[1]) for x in per]
    if len(set(cnts)) != 1:
        print(f"{cat}: CONTEGGI NON deterministici tra R — indagine"); rc = 3
    cnt = per[0][1]
    k_att = STMT[cat]
    k_mis = round(cnt[0] / N, 4)
    tag = "CNT-OK" if abs(k_mis - k_att) < 0.001 else f"CNT-SCARTO(atteso {k_att})"
    if "SCARTO" in tag: rc = rc or 4
    k11 = cnt[11] / N
    if cat == "objalloc": ctor_k = k11
    med = {i: sorted(x[0][i] for x in per)[1] for i in SEGS}
    spread = {i: max(x[0][i] for x in per) - min(x[0][i] for x in per) for i in SEGS}
    freq, s, c = load(cat, 1)
    print(f"-- {cat} (stmt/iter={k_mis:g} [{tag}]; resolve TUTTE k={k11:.4f}; "
          f"c9/iter={cnt[9]/N:.4f} c10/iter={cnt[10]/N:.4f}) --")
    print(f"   cal_ticks/nota={(s[7]/c[7]) if c[7] else 0:.2f} freq={freq}")
    for i in (0, 4, 5, 1, 2, 3, 6, 8, 9, 10, 11):
        print(f"   {SEGS[i]:22s} {med[i]:8.1f} ns/iter  (spread R=3 {spread[i]:.1f})")
    if ctor_k is not None and cat != "objalloc":
        altri = k11 - ctor_k - cnt[9]/N - cnt[10]/N
        print(f"   quota call-site/iter: statement(c9+c10)={cnt[9]/N + cnt[10]/N:.2f} "
              f"ctor={ctor_k:.2f} ALTRI={altri:.2f}" + ("  <-- ENUMERARE per NOME" if altri > 0.01 else ""))
    if med[0] > 0:
        ee2 = med[4] - med[5]
        blocchi = med[2] + med[3] + med[6] + med[8]
        interno = med[1] - med[5]
        print(f"   E-E2={ee2:.1f} ns/iter; prop_step interno (1-5)={interno:.1f}; "
              f"somma blocchi (2+3+6+8)={blocchi:.1f}; chiusura={blocchi/interno*100 if interno>0 else 0:.0f}% "
              f"(resto prop_step altro={interno-blocchi:.1f}); dispatch fuori prop_step={ee2-interno:.1f}")
        nonres = blocchi - med[9] - med[10]
        print(f"   non-resolve nei blocchi = {nonres:.1f} ns/iter (resolve@siti statement = {med[9]+med[10]:.1f})")
print("SONDA-PROPSTEP: " + ("ACQUISITA" if rc == 0 else f"NON ACQUISITA (rc={rc})"))
sys.exit(rc)
PY
prc=$?
echo "$prc" > "$OUT/tempo-propstep.rc"
echo "done rc=$prc $(date +%T)" > "$OUT/tempo-propstep.done"
exit "$prc"
