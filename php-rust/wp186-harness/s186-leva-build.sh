#!/bin/bash
# s186-leva-build.sh — S-186 (fetta 0 + guardia a buffer pieno, concilio S-186 Q2/Q3): cinque bracci nella target separata
#   Z  = gemello del pin s183 (sorgente ZCOMMIT 483e970f; se byte-id col pin lo si dichiara)
#   B  = tree «cura 22/10 com'è» (BCOMMIT = feature census-time SPENTA, gc_sweep_after_ret O(|gc_buf|))
#   B2 = tree + cura O(1) (CCOMMIT, census-time SPENTA) — guardia RI-ESEGUITA (ordine (3) del concilio)
#   C  = CCOMMIT con `--features census-time` (braccio di calibrazione della fetta 0: tempi per classe, Instant 1/64)
#   M  = sorgente del pin (ZCOMMIT) + MUTANTE «spin ~200 ns in Op::CallBuiltin» = strumento vivo della fetta 0
#        (atteso +2,38 s su 11,91M CallBuiltin ORM: wall in [2,1;2,7] s E ≥75 % del Δ nella classe builtin, criterio p.6)
# COPIA DICHIARATA di ../wp185-harness/s185-leva-build.sh (manifest s186-leva-build-copia.diff) coi SOLI adattamenti: token s186,
# tag s186-leva, pin atteso s183 9a6ae11bf7e900ef, cinque bracci (B2 e C in più) con tre archivi (Z/M da ZCOMMIT, B da BCOMMIT,
# B2/C da CCOMMIT), MUTANTE temporale (DEVE restare byte-identico al pin su TUTTE le fixture: il morso è di TEMPO, letto nella
# fetta 0), C byte-identico a B2 su stdout + ≥1 riga `census-time` con count>0 su stderr, fx-mc2 di B/B2 vs oracle a diff
# DICHIARATO (emenda Pedersen: `fxmc2-B.diff` = candidato expected-diff, C1 timing §3.35), DISASM run_loop di B e B2 vs il PIN
# s183: Δsp_refs ≤ +97 (gate, soglia derivata S-184), Δbl e Δistr a verbale (la cura 22/10 aggiunge l'hook: non gate).
# Testo S-185 conservato nel manifest. Esiti SOLO in ab-out/s186-leva.done; verdetto in s186-leva-build-verdetto.out.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment"
H="$REPO/php-rust/wp186-harness"; H1="$REPO/php-rust/wp181-harness"; H3="$REPO/php-rust/wp182-harness"; H4="$REPO/php-rust/wp184-harness"; H5="$REPO/php-rust/wp185-harness"
OUT="$H/ab-out/s186-leva"; mkdir -p "$OUT"
VERD="$H/s186-leva-build-verdetto.out"; DONE="$H/ab-out/s186-leva.done"; rm -f "$DONE"
PIN="$HOME/Claude/php-rust-output/release/phpr"; PIN_ATTESO="9a6ae11bf7e900ef"
ORACLE=/opt/homebrew/opt/php/bin/php
ZCOMMIT="${ZCOMMIT:?ZCOMMIT del sorgente del pin s183 (483e970f)}"
BCOMMIT="${BCOMMIT:?BCOMMIT del tree con la cura 22/10 come sta (census-time spenta)}"
CCOMMIT="${CCOMMIT:?CCOMMIT del tree con la cura O(1) (B2 e C)}"
FX2="$REPO/php-rust/wp174-harness/fixtures/fx-sw2-gc.php"
FX1="$REPO/php-rust/wp174-harness/fixtures/fx-sw1.php"
H2="$REPO/php-rust/wp172-harness"
FXCR="$H1/fx-cr1.php"; FXRT="$H3/fx-rt1.php"; FXRT2="$H4/fx-rt2.php"; FXMC2="$H5/fx-mc2.php"
SRC="/Volumes/Extreme Pro/Claude/s186-leva"; BUNDLE_MP="$HOME/Claude/phpr-target"; TGT="$BUNDLE_MP/s186-leva-tgt"
: > "$VERD"
note(){ echo "$(date '+%H:%M:%S') $1" | tee -a "$VERD"; }
fin(){ echo "rc=$1 $(date '+%F %T')" > "$DONE"; exit "$1"; }

PH=$(shasum -a 256 "$PIN" | cut -c1-16)
[ "$PH" = "$PIN_ATTESO" ] || { note "rc=9 pin canonico $PH != atteso $PIN_ATTESO"; fin 9; }
/sbin/mount | grep -q "$BUNDLE_MP" || { note "rc=8 bundle target NON montata ($BUNDLE_MP)"; fin 8; }
cd "$REPO" || { note "rc=7 repo"; fin 7; }
SHAZ=$(git rev-parse --verify "$ZCOMMIT^{commit}") || { note "rc=7 commit $ZCOMMIT inesistente"; fin 7; }
SHAB=$(git rev-parse --verify "$BCOMMIT^{commit}") || { note "rc=7 commit $BCOMMIT inesistente"; fin 7; }
SHAC=$(git rev-parse --verify "$CCOMMIT^{commit}") || { note "rc=7 commit $CCOMMIT inesistente"; fin 7; }
AVX=$(df -k "/Volumes/Extreme Pro" | awk 'NR>1{printf "%.0f", $4/1048576}')
DATAG=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}')
note "== s186 bracci Z/B/B2/C/M — riferimento A = pin s183 $PH · Z/M = ${SHAZ:0:12} · B = ${SHAB:0:12} · B2/C = ${SHAC:0:12} · Extreme ${AVX}G Data ${DATAG}G"
awk -v a="$AVX" 'BEGIN{exit !(a+0 < 15)}' && { note "rc=8 Extreme ${AVX}G < 15G: niente build"; fin 8; }
[ "${DATAG:-0}" -ge 10 ] || { note "rc=8 Data ${DATAG}G < 10G: niente build"; fin 8; }

# riferimento: pin su fx-sw2-gc (marcatore) e gate bilaterale pin == oracle sulle fixture (sanità del gate a monte)
perl -e 'alarm 120; exec @ARGV or die' -- "$PIN" "$FX2" > "$OUT/ref.out" 2>&1
grep -q "FX-SW2 DONE" "$OUT/ref.out" || { note "rc=7 riferimento: marcatore FX-SW2 assente"; fin 7; }
"$ORACLE" "$FX1" > "$OUT/sw1-oracle.out" 2>&1
perl -e 'alarm 120; exec @ARGV or die' -- "$PIN" "$FX1" > "$OUT/sw1-pin.out" 2>&1
diff -q "$OUT/sw1-oracle.out" "$OUT/sw1-pin.out" > /dev/null || { note "rc=7 fx-sw1: pin ≠ oracle (gate rotto a monte)"; fin 7; }
for pair in "fxcr1:$FXCR:FX-CR1 DONE" "fxrt1:$FXRT:FX-RT1 DONE" "fxrt2:$FXRT2:FX-RT2 DONE" "fxmc2:$FXMC2:FX-MC2 DONE"; do
  n="${pair%%:*}"; rest="${pair#*:}"; f="${rest%%:*}"; m="${rest#*:}"
  "$ORACLE" -d log_errors=0 -d display_errors=1 "$f" > "$OUT/$n-oracle.out" 2>&1
  perl -e 'alarm 120; exec @ARGV or die' -- "$PIN" "$f" > "$OUT/$n-pin.out" 2>&1
  grep -q "$m" "$OUT/$n-pin.out" || { note "rc=7 riferimento: marcatore $m assente sul pin"; fin 7; }
  diff -q "$OUT/$n-oracle.out" "$OUT/$n-pin.out" > /dev/null || { note "rc=7 $n: pin ≠ oracle (fixture non bilaterale a monte)"; fin 7; }
done
note "riferimento: pin == oracle su fx-sw1/fx-cr1/fx-rt1/fx-rt2/fx-mc2 (gate sano)"

archivio(){ # $1=commit → sorgente pulito in $SRC (mai il tree)
  rm -rf "$SRC"; mkdir -p "$SRC/php-rust"
  git archive "$1" php-rust/crates php-rust/Cargo.toml php-rust/Cargo.lock php-rust/rust-toolchain.toml php-rust/.cargo 2>/dev/null | tar -x -C "$SRC" || return 1
  [ -s "$SRC/php-rust/crates/php-runtime/src/vm/run.rs" ] || return 1
  /usr/bin/find "$SRC" -type f -exec touch {} +
}
build(){ # $1=etichetta $2=feature flags (opz.) → stampa hash16; log in $OUT/build-$1.log
  ( cd "$SRC/php-rust" && SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TGT" \
      cargo build --release -p php-cli ${2:-} ) > "$OUT/build-$1.log" 2>&1 || return 1
  cp "$TGT/release/phpr" "$OUT/phpr-$1"; shasum -a 256 "$OUT/phpr-$1" | cut -c1-16
}

build_m(){ # M = sorgente del pin + MUTANTE temporale: spin ~200 ns in Op::CallBuiltin, dopo la lookup nel registry (archivio, mai il tree)
archivio "$SHAZ" || { note "rc=7 archivio M fallito"; fin 7; }
rm -f "$OUT/mutante.err"
python3 - "$SRC/php-rust/crates/php-runtime/src/vm/run.rs" "${SPIN_ITER:-65}" <<'PY' || { echo "mutante non applicato" >> "$OUT/mutante.err"; }
import sys
p = sys.argv[1]; s = open(p).read(); iters = int(sys.argv[2])
# corsa 1 (29/10): spin chiamato DENTRO il braccio CallBuiltin di run_loop ⇒ +100 ns/chiamata finivano nel self di run_loop (layout
# del dispatcher perturbato: quota builtin 0,73 < 0,75). Corsa 2: il sito è l'ingresso di `value_builtin_call` (corpo FUORI dal
# dispatcher, L-HD2 forma-2): run_loop resta al byte, il self dello spin cade nel suo simbolo (classe builtin).
old = ('    ) -> Result<Zval, PhpError> {\n'
       '        // A whole-object exporter initializes a lazy argument first\n'
       '        // (PHP 8.4, init_trigger_var_export); the pure builtin then\n')
assert s.count(old) == 1, 'ingresso di value_builtin_call non unico'
new = ('    ) -> Result<Zval, PhpError> {\n'
       '        builtin_spin_mutant_s186(); // MUTANTE s186: spin ~200 ns per CallBuiltin, fuori linea e fuori dal dispatcher\n'
       '        // A whole-object exporter initializes a lazy argument first\n'
       '        // (PHP 8.4, init_trigger_var_export); the pure builtin then\n')
s = s.replace(old, new)
# EMENDA rev. S-186 #1: spin SENZA clock in una funzione #[inline(never)] dal nome catturato dalla regola `builtin` di xc-classi.py
# (uno spin inline su Instant::now/elapsed cadeva nei frame del clock e nel self di run_loop: strumento MUTO per costruzione).
# corsa 1 (29/10 22:28): 120 iterazioni = 371 ns/CallBuiltin (3,1 ns/iter: black_box forza lo store) ⇒ si RIFÀ la build con 65 (soglia invariata)
s += ('\n/// MUTANTE s186 (strumento vivo della fetta 0): ~200 ns di lavoro dipendente, mai inline, nessun clock.\n'
      '#[inline(never)]\n'
      'pub fn builtin_spin_mutant_s186() {\n'
      '    let mut x = 0u64;\n'
      '    for i in 0..%du64 {\n'
      '        x = std::hint::black_box(x.wrapping_mul(6364136223846793005).wrapping_add(i));\n'
      '    }\n'
      '    std::hint::black_box(x);\n'
      '}\n' % iters)
open(p, 'w').write(s)
print('mutante applicato, iterazioni', iters)
PY
[ -e "$OUT/mutante.err" ] && { note "rc=7 mutante non applicabile (sito non trovato)"; fin 7; }
grep -q 'MUTANTE s186' "$SRC/php-rust/crates/php-runtime/src/vm/run.rs" || { note "rc=7 mutante NON scritto (verifica grep dopo la patch, lezione S-183 #3)"; fin 7; }
MH=$(build M) || { note "rc=4 M: build FALLITA (ab-out/s186-leva/build-M.log)"; fin 4; }
[ "$MH" != "$ZH" ] || { note "rc=7 M: binario == Z (mutante NON entrato)"; fin 7; }
note "M: binario $MH (sorgente del pin + spin ${SPIN_ITER:-65} iterazioni in CallBuiltin)"
}

if [ "${SKIP_BUILD:-0}" = 1 ] && [ -s "$OUT/phpr-Z" ] && [ -s "$OUT/phpr-B" ] && [ -s "$OUT/phpr-B2" ] && [ -s "$OUT/phpr-C" ] && [ -s "$OUT/phpr-M" ]; then
  ZH=$(shasum -a 256 "$OUT/phpr-Z" | cut -c1-16); BH=$(shasum -a 256 "$OUT/phpr-B" | cut -c1-16); B2H=$(shasum -a 256 "$OUT/phpr-B2" | cut -c1-16)
  CH=$(shasum -a 256 "$OUT/phpr-C" | cut -c1-16); MH=$(shasum -a 256 "$OUT/phpr-M" | cut -c1-16)
  note "SKIP_BUILD=1 (emenda 5 S-184): riuso dichiarato dei binari — Z $ZH$( [ "$ZH" = "$PH" ] && echo ' == pin' || echo " ≠ pin $PH") · B $BH · B2 $B2H · C $CH · M $MH"
  # REBUILD_M=1: si rifà SOLO M (taratura fuori finestra nella corsa precedente: la build si rifà, non la soglia)
  if [ "${REBUILD_M:-0}" = 1 ]; then OLDM="$MH"; build_m; [ "$MH" != "$OLDM" ] || { note "rc=7 M: binario identico alla corsa precedente (mutante NON cambiato)"; fin 7; }; fi
else
# Z = sorgente del pin nella target separata (gemello: se byte-id col pin lo si dichiara)
archivio "$SHAZ" || { note "rc=7 archivio Z fallito"; fin 7; }
ZH=$(build Z) || { note "rc=4 Z: build FALLITA (ab-out/s186-leva/build-Z.log)"; fin 4; }
if [ "$ZH" = "$PH" ]; then note "Z: binario $ZH == pin BYTE-ID (gemello byte-id: |A−Z| misura il solo rumore — dichiarato)"; else note "Z: binario $ZH ≠ pin $PH (gemello a contenuto: SL in-run stimabile)"; fi
build_m

# B = tree con la cura 22/10 com'è (census-time spenta)
archivio "$SHAB" || { note "rc=7 archivio B fallito"; fin 7; }
BH=$(build B) || { note "rc=4 B: build FALLITA (ab-out/s186-leva/build-B.log)"; fin 4; }
[ "$BH" != "$PH" ] || { note "rc=6 B: binario == pin (sorgente NON entrata)"; fin 6; }
note "B: binario $BH (commit ${SHAB:0:12}, cura 22/10 com'è)"

# B2 = tree + cura O(1) (census-time spenta) · C = stesso archivio con --features census-time
archivio "$SHAC" || { note "rc=7 archivio B2/C fallito"; fin 7; }
B2H=$(build B2) || { note "rc=4 B2: build FALLITA (ab-out/s186-leva/build-B2.log)"; fin 4; }
[ "$B2H" != "$BH" ] || { note "rc=6 B2: binario == B (cura O(1) NON entrata)"; fin 6; }
note "B2: binario $B2H (commit ${SHAC:0:12}, cura O(1))"
CH=$(build C "--features census-time") || { note "rc=4 C: build FALLITA (ab-out/s186-leva/build-C.log)"; fin 4; }
[ "$CH" != "$B2H" ] || { note "rc=6 C: binario == B2 (feature census-time NON entrata)"; fin 6; }
note "C: binario $CH (commit ${SHAC:0:12}, --features census-time)"
fi

RC=0
# B e B2 su fx-sw2-gc vs pin (invarianza del marcatore GC; blocchi rotti a verbale, non gate — come S-185)
for arm in B B2; do
  perl -e 'alarm 300; exec @ARGV or die' -- "$OUT/phpr-$arm" "$FX2" > "$OUT/$arm.out" 2>&1
  if diff -q "$OUT/ref.out" "$OUT/$arm.out" > /dev/null; then note "$arm: fx-sw2-gc == pin BYTE-ID (invarianza)"; else diff "$OUT/ref.out" "$OUT/$arm.out" > "$OUT/$arm-invarianza.diff" || true; note "$arm: fx-sw2-gc ≠ pin — $(grep -c '^[<>]' "$OUT/$arm-invarianza.diff") righe (ab-out/s186-leva/$arm-invarianza.diff) — a verbale"; fi
done

bilat(){ # $1=braccio $2=nome $3=file $4=marcatore $5..=opzioni oracle → B/B2 == oracle (fx-mc2: diff DICHIARATO, non rc)
  local arm="$1" n="$2" f="$3" m="$4"; shift 4
  [ -s "$OUT/$n-oracle.out" ] || "$ORACLE" "$@" "$f" > "$OUT/$n-oracle.out" 2>&1
  perl -e 'alarm 300; exec @ARGV or die' -- "$OUT/phpr-$arm" "$f" > "$OUT/$n-$arm.out" 2>&1
  [ -s "$OUT/$n-$arm.out" ] || { note "$arm: $n output VUOTO -> rc=2"; RC=2; return; }
  [ -z "$m" ] || grep -q "$m" "$OUT/$n-$arm.out" || { note "$arm: $n marcatore $m ASSENTE -> rc=2"; RC=2; return; }
  if diff -q "$OUT/$n-oracle.out" "$OUT/$n-$arm.out" > /dev/null; then note "$arm: $n == oracle BYTE-ID"
  else
    diff "$OUT/$n-oracle.out" "$OUT/$n-$arm.out" > "$OUT/$n-$arm.diff" || true
    if [ "$n" = fxmc2 ]; then note "$arm: fx-mc2 DIVERGE dall'oracle ($(grep -c '^[<>]' "$OUT/$n-$arm.diff") righe: C1 timing §3.35, emenda Pedersen — ab-out/s186-leva/$n-$arm.diff = candidato expected-diff, NON gate)"
    else note "$arm: $n DIVERGE dall'oracle ($(grep -c '^[<>]' "$OUT/$n-$arm.diff") righe, ab-out/s186-leva/$n-$arm.diff) -> rc=2"; RC=2; fi
  fi
}
for arm in B B2; do
  bilat $arm fxsw1 "$FX1" "FX-SW1 DONE"
  bilat $arm fxsl1 "$H2/fx-sl1.php" "FX-SL1 DONE" -d log_errors=0 -d display_errors=1
  bilat $arm fxsl2 "$H2/fx-sl2.php" "FX-SL2 DONE" -d log_errors=0 -d display_errors=1
  bilat $arm fxsl3 "$H2/fx-sl3.php" "FX-SL3 DONE" -d log_errors=0 -d display_errors=1
  bilat $arm fxcr1 "$FXCR" "FX-CR1 DONE" -d log_errors=0 -d display_errors=1
  bilat $arm fxrt1 "$FXRT" "FX-RT1 DONE" -d log_errors=0 -d display_errors=1
  bilat $arm fxrt2 "$FXRT2" "FX-RT2 DONE" -d log_errors=0 -d display_errors=1
  bilat $arm fxmc2 "$FXMC2" "FX-MC2 DONE" -d log_errors=0 -d display_errors=1
done
# B2 vs B: la cura O(1) NON deve cambiare byte sulle fixture (controesempio (b) del concilio = forma (2): se compare, a verbale, rc=2)
for n in fxsw1 fxsl1 fxsl2 fxsl3 fxcr1 fxrt1 fxrt2 fxmc2; do
  if ! diff -q "$OUT/$n-B.out" "$OUT/$n-B2.out" > /dev/null; then diff "$OUT/$n-B.out" "$OUT/$n-B2.out" > "$OUT/$n-B2-vs-B.diff" || true; note "B2 ≠ B su $n ($(grep -c '^[<>]' "$OUT/$n-B2-vs-B.diff") righe: la cura O(1) cambia byte — controesempio (b), a verbale) -> rc=2"; RC=2; fi
done
[ "$RC" -eq 0 ] && note "B2 == B BYTE-ID su tutte le fixture (la cura O(1) non cambia byte)"

# M: byte-identico al PIN su TUTTE le fixture (il morso del mutante è di TEMPO, letto nella fetta 0) — altrimenti rc=3
MB=0
for pair in "fxsw1:$FX1" "fxsl1:$H2/fx-sl1.php" "fxsl2:$H2/fx-sl2.php" "fxsl3:$H2/fx-sl3.php" "fxcr1:$FXCR" "fxrt1:$FXRT" "fxrt2:$FXRT2" "fxmc2:$FXMC2" "fxsw2:$FX2"; do
  n="${pair%%:*}"; f="${pair#*:}"
  perl -e 'alarm 300; exec @ARGV or die' -- "$OUT/phpr-M" "$f" > "$OUT/$n-M.out" 2>&1
  perl -e 'alarm 300; exec @ARGV or die' -- "$PIN" "$f" > "$OUT/$n-pinref.out" 2>&1
  cmp -s "$OUT/$n-M.out" "$OUT/$n-pinref.out" || { MB=$((MB+1)); note "M ≠ pin su $n (il mutante cambia byte: NON ammesso) -> rc=3"; }
done
if [ "$MB" -eq 0 ]; then note "M == pin BYTE-ID su 9 fixture (mutante puramente temporale)"; else [ "$RC" -eq 0 ] && RC=3; fi
# TARATURA del mutante (gate, emenda rev. S-186 #1): builtin-dq.php (N CallBuiltin dal sorgente), user CPU pin vs M, R=3 interleaved,
# ns/chiamata = (med M − med pin)/N; PRE-registrato [120;320] ns (atteso ~200), altrimenti rc=3; la cifra va in mutante-ns.txt
# e la fetta 0 la legge per la finestra di ΔM (0,85-1,15 × 11,91M × ns).
ucpu(){ { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$@" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
BD="$H/builtin-dq.php"; NBD=$(awk 'match($0, /\$i < [0-9]+/) {print substr($0, RSTART+5, RLENGTH-5); exit}' "$BD")
TP=""; TM=""
for r in 1 2 3; do TP="$TP $(ucpu "$PIN" "$BD")"; TM="$TM $(ucpu "$OUT/phpr-M" "$BD")"; done
NSM=$(python3 -c 'import sys
p=sorted(map(float,sys.argv[1].split())); m=sorted(map(float,sys.argv[2].split())); n=float(sys.argv[3])
print("%.1f" % ((m[1]-p[1])/n*1e9))' "$TP" "$TM" "$NBD")
echo "$NSM" > "$OUT/mutante-ns.txt"
if python3 -c 'import sys; v=float(sys.argv[1]); sys.exit(0 if 120.0 <= v <= 320.0 else 1)' "$NSM"; then
  note "TARATURA mutante: $NSM ns/CallBuiltin (N=$NBD; pin [$TP ] · M [$TM ] s) ∈ [120;320] ⇒ OK (ab-out/s186-leva/mutante-ns.txt)"
else
  note "TARATURA mutante: $NSM ns/CallBuiltin FUORI [120;320] (pin [$TP ] · M [$TM ]) -> rc=3"; [ "$RC" -eq 0 ] && RC=3
fi
# ATTRIBUZIONE del mutante (gate, emenda rev. S-186 #1): xctrace Time Profiler su builtin-dq.php per pin e M, classi via xc-classi.py;
# ΔM nella classe builtin ≥ 75 % di ΔM totale, altrimenti lo strumento è MUTO per costruzione -> rc=3.
XC="$H/xc-classi.py"; KT="$(getconf DARWIN_USER_TEMP_DIR)"
for arm in pin M; do
  case "$arm" in pin) bin="$PIN";; M) bin="$OUT/phpr-M";; esac
  rm -rf "$OUT/attr-$arm.trace"
  xcrun xctrace record --template 'Time Profiler' --output "$OUT/attr-$arm.trace" --target-stdout /dev/null --launch -- "$bin" "$BD" > "$OUT/attr-$arm.rec.log" 2>&1
  xcrun xctrace export --input "$OUT/attr-$arm.trace" --xpath '//trace-toc/run[@number="1"]/data/table[@schema="time-profile"]' > "$OUT/attr-$arm.xml" 2>/dev/null
  python3 -I "$XC" "$OUT/attr-$arm.xml" "$OUT/attr-$arm.classi.tsv" "$OUT/attr-$arm.top.tsv" > "$OUT/attr-$arm.classi.log" 2>&1
  rm -rf "$OUT/attr-$arm.trace" "$KT"/instruments*.ktrace
done
ATTR=$(python3 -c 'import sys
def load(p):
    d={}
    for l in open(p):
        f=l.rstrip("\n").split("\t")
        if len(f)>=2:
            try: d[f[0]]=float(f[1])
            except ValueError: pass
    return d
p=load(sys.argv[1]); m=load(sys.argv[2])
dt=m.get("TOTAL",0)-p.get("TOTAL",0); db=m.get("builtin",0)-p.get("builtin",0)
q=db/dt if dt>0 else 0.0
print("%.2f %.2f %.2f" % (dt, db, q))' "$OUT/attr-pin.classi.tsv" "$OUT/attr-M.classi.tsv" 2>/dev/null)
set -- $ATTR
if [ -n "${3:-}" ] && python3 -c 'import sys; sys.exit(0 if float(sys.argv[1]) >= 0.75 and float(sys.argv[2]) > 0 else 1)' "$3" "$1"; then
  note "ATTRIBUZIONE mutante: ΔM totale $1 s, in classe builtin $2 s = quota $3 (≥ 0,75) ⇒ strumento VIVO per costruzione (attr-M.classi.tsv)"
else
  note "ATTRIBUZIONE mutante: ΔM totale ${1:-n/d} s, builtin ${2:-n/d} s, quota ${3:-n/d} < 0,75 o export fallito ⇒ strumento MUTO per costruzione -> rc=3"; [ "$RC" -eq 0 ] && RC=3
fi

# C: stdout byte-identico a B2 su tutte le fixture (stderr porta le righe census-time) + strumento vivo (count>0 su call_fast)
CB=0
for pair in "fxsw1:$FX1" "fxcr1:$FXCR" "fxrt1:$FXRT" "fxrt2:$FXRT2" "fxmc2:$FXMC2"; do
  n="${pair%%:*}"; f="${pair#*:}"
  perl -e 'alarm 300; exec @ARGV or die' -- "$OUT/phpr-C" "$f" > "$OUT/$n-C.out" 2> "$OUT/$n-C.err"
  perl -e 'alarm 300; exec @ARGV or die' -- "$OUT/phpr-B2" "$f" > "$OUT/$n-B2so.out" 2> /dev/null
  cmp -s "$OUT/$n-C.out" "$OUT/$n-B2so.out" || { CB=$((CB+1)); note "C ≠ B2 su stdout di $n -> rc=3"; }
done
# strumento vivo = conteggio ret_fast > 0 (fx-mc2 è tutta metodi: call_fast può essere 0 — corsa 1 rc=3 per questo, emenda dichiarata)
CC=$(awk -F'\t' '$1=="census-time" && $2=="ret_fast" {print $3}' "$OUT/fxmc2-C.err" 2>/dev/null)
if [ "$CB" -eq 0 ] && [ "${CC:-0}" -gt 0 ]; then note "C: stdout == B2 su 5 fixture; census-time vivo (ret_fast count=$CC su fx-mc2; righe: $(grep -c '^census-time' "$OUT/fxmc2-C.err"))"; else note "C: stdout diverso ($CB) o census MUTO (ret_fast count='${CC:-}') -> rc=3"; [ "$RC" -eq 0 ] && RC=3; fi

# disasm agli atti: istr/bl/blr/sp_refs di run_loop di B e B2 vs il PIN s183 (gate Δsp_refs ≤ +97; Δbl/Δistr a verbale: l'hook della cura 22/10 è noto)
dis(){ # $1=binario $2=etichetta
  local sym; sym=$(nm -n "$1" | awk '{print $3}' | grep -iE '8run_loop17h|2Vm8run_loop$' | head -n 1)
  [ -n "$sym" ] || { echo "simbolo run_loop non trovato"; return; }
  objdump -d --no-show-raw-insn --disassemble-symbols="$sym" "$1" > "$OUT/disasm-$2-run_loop.s" 2>/dev/null
  echo "istr=$(grep -cE '^ *[0-9a-f]+:' "$OUT/disasm-$2-run_loop.s") bl=$(grep -cE '[[:space:]]bl[[:space:]]' "$OUT/disasm-$2-run_loop.s") blr=$(grep -cE '[[:space:]]blr[[:space:]]' "$OUT/disasm-$2-run_loop.s") sp_refs=$(grep -c '\[sp' "$OUT/disasm-$2-run_loop.s")"
}
num(){ echo "$1" | sed -n "s/.*$2=\([0-9]*\).*/\1/p"; }
DPIN=$(dis "$PIN" pin); DZ=$(dis "$OUT/phpr-Z" Z)
note "DISASM run_loop: pin s183 $DPIN · Z $DZ"
for arm in B B2; do
  DB=$(dis "$OUT/phpr-$arm" $arm)
  IB=$(num "$DB" istr); IT=$(num "$DPIN" istr); SPB=$(num "$DB" sp_refs); SPT=$(num "$DPIN" sp_refs); BLB=$(num "$DB" bl); BLT=$(num "$DPIN" bl)
  if [ -n "$IB" ] && [ -n "$IT" ] && [ -n "$SPB" ] && [ -n "$SPT" ]; then
    DI=$((IB-IT)); DSP=$((SPB-SPT)); DBL=$((BLB-BLT))
    if [ "$DSP" -le 97 ]; then note "GATE disasm $arm vs pin: Δsp_refs=$DSP (≤ +97) ⇒ RISPETTATO · Δbl=$DBL Δistr=$DI a verbale ($DB)"; else note "GATE disasm $arm: Δsp_refs=$DSP > +97 FUORI tolleranza (Δbl=$DBL Δistr=$DI) -> rc=5"; [ "$RC" -eq 0 ] && RC=5; fi
  else
    note "GATE disasm $arm: conteggi non leggibili ($arm='$DB' pin='$DPIN') -> rc=5"; [ "$RC" -eq 0 ] && RC=5
  fi
done
rm -rf "$TGT" "$SRC"
note "ESITO rc=$RC (0 = B/B2 a parità, M byte-id col pin, C vivo, disasm in tolleranza: bracci pronti; 2 = B/B2 divergono; 3 = M cambia byte o C muto; 5 = disasm fuori tolleranza) fine $(date '+%F %T')"
fin $RC
