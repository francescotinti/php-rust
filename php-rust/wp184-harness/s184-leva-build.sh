#!/bin/bash
# s184-leva-build.sh — bracci Z (gemello del pin s181) e B (tree = L-RT1 + L-RT2) + MUTANTE (criterio s184-criterio-rt2.md p.3/p.6):
# COPIA DICHIARATA di ../wp182-harness/s183-leva-build.sh (manifest s184-leva-build-copia.diff) coi SOLI adattamenti: token s184,
# tag s184-leva, fixture fx-rt2 (NUOVA) in riferimento + parità + bersaglio del mutante, MUTANTE = «ammissione del Ret fuso SENZA i bit
# di forma» (shape/flags/ret_cell tolti dall'ammissione; esito esatto: M diverge dall'oracle su fx-rt2 su ≥1 riga e TUTTE le righe
# cambiate sono RT2- o righe del fatal/perse dopo l'abort — emenda p.7a), DISASM come GATE (az.rev. S-183 #2, emenda p.7b forma 2):
# istr(B) < istr(tree-RT1 9ef70777) e sp_refs(B) ≤ +40 (ab-out/s183-leva/phpr-B), altrimenti rc=5 «meccanismo da rileggere». Testo S-183/S-181 conservato sotto:
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment"
H="$REPO/php-rust/wp184-harness"; H1="$REPO/php-rust/wp181-harness"; H3="$REPO/php-rust/wp182-harness"; OUT="$H/ab-out/s184-leva"; mkdir -p "$OUT"
VERD="$H/s184-leva-build-verdetto.out"; DONE="$H/ab-out/s184-leva.done"; rm -f "$DONE"
LOCK=/private/tmp/phpr-measure.lock
PIN="$HOME/Claude/php-rust-output/release/phpr"; PIN_ATTESO="19a2faa83a492745"
ORACLE=/opt/homebrew/opt/php/bin/php
COMMIT="${COMMIT:?COMMIT della leva (hash)}"; ZCOMMIT="${ZCOMMIT:?ZCOMMIT del sorgente del pin (hash)}"
FX2="$REPO/php-rust/wp174-harness/fixtures/fx-sw2-gc.php"
FX1="$REPO/php-rust/wp174-harness/fixtures/fx-sw1.php"
H2="$REPO/php-rust/wp172-harness"
FXCR="$H1/fx-cr1.php"
# EMENDA S-183 (dichiarata dopo il rc=3 delle 20:16: fx-cr1/fx-sw2-gc non distinguono B da M): fixture NUOVA fx-rt1.php (cicli locali +
# gc_collect_cycles) = gate bilaterale in più E bersaglio del mutante; forma del morso = sole righe «dtor …» o «collectN: …» cambiate
FXRT="$H3/fx-rt1.php"; FXRT2="$H/fx-rt2.php"; TREE_RT1="$H3/ab-out/s183-leva/phpr-B"
SRC="/Volumes/Extreme Pro/Claude/s184-leva"; BUNDLE_MP="$HOME/Claude/phpr-target"; TGT="$BUNDLE_MP/s184-leva-tgt"
: > "$VERD"
fin(){ echo "rc=$1 $(date +%T)" > "$DONE"; exit "$1"; }
note(){ echo "$*" >> "$VERD"; }

grep -qw s184 "$LOCK" 2>/dev/null || { note "rc=9 lock s184 assente (per TOKEN)"; fin 9; }
/sbin/mount | grep -q " $BUNDLE_MP " || { note "rc=8 bundle NON montata ($BUNDLE_MP)"; fin 8; }
for f in "$FX2" "$FX1" "$H2/fx-sl1.php" "$H2/fx-sl2.php" "$H2/fx-sl3.php" "$FXCR" "$FXRT" "$FXRT2" "$TREE_RT1"; do [ -s "$f" ] || { note "rc=7 fixture assente: $f"; fin 7; }; done
PH=$(shasum -a 256 "$PIN" | cut -c1-16)
[ "$PH" = "$PIN_ATTESO" ] || { note "rc=9 pin $PH ≠ atteso $PIN_ATTESO"; fin 9; }
cd "$REPO" || fin 7
SHA0=$(git rev-parse --verify "$COMMIT^{commit}") || { note "rc=7 commit $COMMIT inesistente"; fin 7; }
SHAZ=$(git rev-parse --verify "$ZCOMMIT^{commit}") || { note "rc=7 commit $ZCOMMIT inesistente"; fin 7; }
AVX=$(df -k "/Volumes/Extreme Pro" | awk 'NR>1{printf "%.0f", $4/1048576}')
note "== s184 bracci Z/B/M leva COMPOSTA L-RT1+L-RT2 — riferimento A = pin s181 $PH, tree-RT1 $(shasum -a 256 "$TREE_RT1" | cut -c1-8), sorgente B = commit ${SHA0:0:12}, Z = commit ${SHAZ:0:12}, fixture fx-sw2-gc $(wc -l < "$FX2" | tr -d ' ') righe + fx-sw1 $(wc -l < "$FX1" | tr -d ' ') righe + fx-sl1/fx-sl2/fx-sl3 + fx-cr1 $(wc -l < "$FXCR" | tr -d ' ') righe, Extreme ${AVX}G $(date '+%F %T') =="
awk -v a="$AVX" 'BEGIN{exit !(a+0 < 15)}' && { note "rc=8 Extreme ${AVX}G < 15G: niente build"; fin 8; }

# riferimento: pin su fx-sw2-gc (marcatore) e gate bilaterale fx-sw1 + fx-cr1 sul pin (sanità del gate)
perl -e 'alarm 120; exec @ARGV or die' -- "$PIN" "$FX2" > "$OUT/ref.out" 2>&1
grep -q "FX-SW2 DONE" "$OUT/ref.out" || { note "rc=7 riferimento: marcatore FX-SW2 assente"; fin 7; }
"$ORACLE" "$FX1" > "$OUT/sw1-oracle.out" 2>&1
perl -e 'alarm 120; exec @ARGV or die' -- "$PIN" "$FX1" > "$OUT/sw1-pin.out" 2>&1
grep -q "FX-SW1 DONE" "$OUT/sw1-pin.out" || { note "rc=7 riferimento: marcatore FX-SW1 assente"; fin 7; }
diff -q "$OUT/sw1-oracle.out" "$OUT/sw1-pin.out" > /dev/null || { note "rc=7 fx-sw1: pin ≠ oracle (gate rotto a monte)"; fin 7; }
"$ORACLE" -d log_errors=0 -d display_errors=1 "$FXCR" > "$OUT/fxcr1-oracle.out" 2>&1
perl -e 'alarm 120; exec @ARGV or die' -- "$PIN" "$FXCR" > "$OUT/fxcr1-pin.out" 2>&1
grep -q "FX-CR1 DONE" "$OUT/fxcr1-pin.out" || { note "rc=7 riferimento: marcatore FX-CR1 assente sul pin"; fin 7; }
diff -q "$OUT/fxcr1-oracle.out" "$OUT/fxcr1-pin.out" > /dev/null || { note "rc=7 fx-cr1: pin ≠ oracle (fixture non bilaterale a monte)"; fin 7; }
"$ORACLE" -d log_errors=0 -d display_errors=1 "$FXRT" > "$OUT/fxrt1-oracle.out" 2>&1
perl -e 'alarm 120; exec @ARGV or die' -- "$PIN" "$FXRT" > "$OUT/fxrt1-pin.out" 2>&1
grep -q "FX-RT1 DONE" "$OUT/fxrt1-pin.out" || { note "rc=7 riferimento: marcatore FX-RT1 assente sul pin"; fin 7; }
diff -q "$OUT/fxrt1-oracle.out" "$OUT/fxrt1-pin.out" > /dev/null || { note "rc=7 fx-rt1: pin ≠ oracle (fixture non bilaterale a monte)"; fin 7; }
"$ORACLE" -d log_errors=0 -d display_errors=1 "$FXRT2" > "$OUT/fxrt2-oracle.out" 2>&1
perl -e 'alarm 120; exec @ARGV or die' -- "$PIN" "$FXRT2" > "$OUT/fxrt2-pin.out" 2>&1
grep -q "FX-RT2 DONE" "$OUT/fxrt2-pin.out" || { note "rc=7 riferimento: marcatore FX-RT2 assente sul pin"; fin 7; }
diff -q "$OUT/fxrt2-oracle.out" "$OUT/fxrt2-pin.out" > /dev/null || { note "rc=7 fx-rt2: pin ≠ oracle (fixture non bilaterale a monte)"; fin 7; }
NACE=$(grep -c '^ACE' "$OUT/fxcr1-oracle.out")
note "RIFERIMENTO: pin fx-sw2-gc $(tr '\n' ' ' < "$OUT/ref.out" | cut -c1-160) · fx-sw1 pin==oracle BYTE-ID · fx-cr1 pin==oracle BYTE-ID (righe ACE: $NACE) · fx-rt1/fx-rt2 pin==oracle BYTE-ID"

archivio(){ # $1=sha
  rm -rf "$SRC"; mkdir -p "$SRC/php-rust"
  git archive "$1" php-rust/crates php-rust/Cargo.toml php-rust/Cargo.lock php-rust/rust-toolchain.toml php-rust/.cargo 2>/dev/null | tar -x -C "$SRC" || return 1
  [ -s "$SRC/php-rust/crates/php-runtime/src/vm/run.rs" ] || return 1
  /usr/bin/find "$SRC" -type f -exec touch {} +
}
build(){ # $1=etichetta → stampa hash16; log in $OUT/build-$1.log
  ( cd "$SRC/php-rust" && SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TGT" \
      cargo build --release -p php-cli ) > "$OUT/build-$1.log" 2>&1 || return 1
  cp "$TGT/release/phpr" "$OUT/phpr-$1"; shasum -a 256 "$OUT/phpr-$1" | cut -c1-16
}
rotte(){ # $1=ref.out $2=mut.out → stdout etichette rotte (blocchi per etichetta)
  python3 - "$1" "$2" <<'PY'
import sys, re
def blocks(p):
    d, cur = {}, None
    for line in open(p, encoding='utf-8', errors='replace'):
        m = re.match(r"^([A-Za-z0-9_().'#=-]+): ", line)
        if m: cur = m.group(1); d.setdefault(cur, [])
        if cur is not None: d[cur].append(line)
    return d
a, b = blocks(sys.argv[1]), blocks(sys.argv[2])
for k in a:
    if a[k] != b.get(k): print(k)
PY
}

if [ "${SKIP_BUILD:-0}" = 1 ] && [ -s "$OUT/phpr-Z" ] && [ -s "$OUT/phpr-B" ] && [ -s "$OUT/phpr-M" ]; then
  ZH=$(shasum -a 256 "$OUT/phpr-Z" | cut -c1-16); BH=$(shasum -a 256 "$OUT/phpr-B" | cut -c1-16); MH=$(shasum -a 256 "$OUT/phpr-M" | cut -c1-16)
  note "SKIP_BUILD=1 (emenda 5): riuso dei binari della corsa 2 — Z $ZH$( [ "$ZH" = "$PH" ] && echo ' == pin' || echo " ≠ pin $PH (gemello a contenuto)") · B $BH · M $MH"
  [ "$BH" != "$PH" ] || { note "rc=6 B: binario == pin"; fin 6; }
  [ "$MH" != "$BH" ] || { note "rc=7 M: binario == B"; fin 7; }
else
# Z = sorgente del pin nella target separata (gemello: se byte-id col pin lo si dichiara)
archivio "$SHAZ" || { note "rc=7 archivio Z fallito"; fin 7; }
ZH=$(build Z) || { note "rc=4 Z: build FALLITA (ab-out/s184-leva/build-Z.log)"; fin 4; }
if [ "$ZH" = "$PH" ]; then note "Z: binario $ZH == pin BYTE-ID (gemello byte-id: SL in-run NON stimabile — dichiarato; |A−Z| misura il solo rumore)"; else note "Z: binario $ZH ≠ pin $PH (gemello a contenuto: SL in-run stimabile)"; fi

# B = leva
archivio "$SHA0" || { note "rc=7 archivio B fallito"; fin 7; }
BH=$(build B) || { note "rc=4 B: build FALLITA (ab-out/s184-leva/build-B.log)"; fin 4; }
[ "$BH" != "$PH" ] || { note "rc=6 B: binario == pin (sorgente NON entrata)"; fin 6; }
note "B: binario $BH (commit ${SHA0:0:12})"

# M = B + mutante: ammissione del Ret FUSO senza i bit di FORMA (shape/flags/ret_cell) — sostituzione sull'ARCHIVIO, mai sul tree — DEVE divergere su fx-rt2 (righe RT2-)
python3 - "$SRC/php-rust/crates/php-runtime/src/vm/run.rs" <<'PY' || { echo "mutante non applicato" >> "$OUT/mutante.err"; }
import sys
p = sys.argv[1]; s = open(p).read()
old = ('                        f.func.ret_shape == 0\n'
       '                            && f.flags.bits() == 0\n'
       '                            && f.ret_cell.is_none()\n'
       '                            && f.this.is_none()\n')
assert s.count(old) == 1, 'sito ammissione Ret fuso non unico'
s = s.replace(old, '                        // MUTANTE s184: ammissione SENZA shape/flags/ret_cell\n                        f.this.is_none()\n')
open(p, 'w').write(s)
print('mutante applicato')
PY
[ -e "$OUT/mutante.err" ] && { note "rc=7 mutante non applicabile (sito non trovato)"; fin 7; }
grep -q 'MUTANTE s184' "$SRC/php-rust/crates/php-runtime/src/vm/run.rs" || { note "rc=7 mutante NON scritto (verifica grep dopo la patch, lezione S-183 #3)"; fin 7; }
MH=$(build M) || { note "rc=4 M: build FALLITA (ab-out/s184-leva/build-M.log)"; fin 4; }
[ "$MH" != "$BH" ] || { note "rc=7 M: binario == B (mutante NON entrato)"; fin 7; }
note "M: binario $MH (B + mutante: ammissione del Ret fuso senza shape/flags/ret_cell)"
fi

RC=0
perl -e 'alarm 120; exec @ARGV or die' -- "$OUT/phpr-B" "$FX2" > "$OUT/B.out" 2>&1
rotte "$OUT/ref.out" "$OUT/B.out" | sort -u > "$OUT/B.rotte"
if diff -q "$OUT/ref.out" "$OUT/B.out" > /dev/null; then note "B: fx-sw2-gc == pin BYTE-ID (invarianza)"; else diff "$OUT/ref.out" "$OUT/B.out" > "$OUT/B-invarianza.diff" || true; note "B: fx-sw2-gc ≠ pin — blocchi ROTTI ($(wc -l < "$OUT/B.rotte" | tr -d ' ')): $(tr '\n' ' ' < "$OUT/B.rotte") -> rc=2"; RC=2; fi
bilat(){ # $1=nome $2=file $3=marcatore $4..=opzioni oracle
  local n="$1" f="$2" m="$3"; shift 3
  "$ORACLE" "$@" "$f" > "$OUT/$n-oracle.out" 2>&1
  perl -e 'alarm 300; exec @ARGV or die' -- "$OUT/phpr-B" "$f" > "$OUT/$n-B.out" 2>&1
  [ -s "$OUT/$n-B.out" ] || { note "B: $n output VUOTO -> rc=2"; RC=2; return; }
  [ -z "$m" ] || grep -q "$m" "$OUT/$n-B.out" || { note "B: $n marcatore $m ASSENTE -> rc=2"; RC=2; return; }
  if diff -q "$OUT/$n-oracle.out" "$OUT/$n-B.out" > /dev/null; then note "B: $n == oracle BYTE-ID"; else diff "$OUT/$n-oracle.out" "$OUT/$n-B.out" > "$OUT/$n.diff" || true; note "B: $n DIVERGE dall'oracle ($(wc -l < "$OUT/$n.diff" | tr -d ' ') righe, ab-out/s184-leva/$n.diff) -> rc=2"; RC=2; fi
}
bilat fxsw1 "$FX1" "FX-SW1 DONE"
bilat fxsl1 "$H2/fx-sl1.php" "FX-SL1 DONE" -d log_errors=0 -d display_errors=1
bilat fxsl2 "$H2/fx-sl2.php" "FX-SL2 DONE" -d log_errors=0 -d display_errors=1
bilat fxsl3 "$H2/fx-sl3.php" "FX-SL3 DONE" -d log_errors=0 -d display_errors=1
bilat fxcr1 "$FXCR" "FX-CR1 DONE" -d log_errors=0 -d display_errors=1
bilat fxrt1 "$FXRT" "FX-RT1 DONE" -d log_errors=0 -d display_errors=1
bilat fxrt2 "$FXRT2" "FX-RT2 DONE" -d log_errors=0 -d display_errors=1

# mutante: DEVE divergere dall'oracle su fx-rt2 con ≥1 riga cambiata e TUTTE le righe cambiate etichettate RT2- (esito esatto, criterio-rt2 p.3)
perl -e 'alarm 120; exec @ARGV or die' -- "$OUT/phpr-M" "$FXRT2" > "$OUT/fxrt2-M.out" 2>&1
diff "$OUT/fxrt2-oracle.out" "$OUT/fxrt2-M.out" > "$OUT/fxrt2-M.diff" || true
MCH=$(grep -c '^[<>]' "$OUT/fxrt2-M.diff"); MBAD=$(grep '^[<>]' "$OUT/fxrt2-M.diff" | grep -vcE '^[<>] (RT2-|FX-RT2 DONE|Fatal error|Stack trace|#[0-9]|  thrown)|^[<>] ?$')
if [ "$MCH" -gt 0 ] && [ "$MBAD" -eq 0 ]; then
  note "MUTANTE: morde — fx-rt2 righe cambiate $MCH, tutte RT2- (non RT2- $MBAD): etichette $(grep '^[<>]' "$OUT/fxrt2-M.diff" | sed 's/^[<>] \(RT2-[a-z-]*\).*/\1/' | sort -u | tr '\n' ' ') — la fixture presidia l'ammissione"
else
  note "MUTANTE: NON morde nella forma attesa (fx-rt2 cambiate $MCH, non RT2- $MBAD; ab-out/s184-leva/fxrt2-M.diff) -> rc=3"; [ "$RC" -eq 0 ] && RC=3
fi

# disasm agli atti (p.6): istr/bl/blr/sp_refs di run_loop di B vs pin s180 (S-104: ogni leva su run_loop pretende il disasm)
dis(){ # $1=binario $2=etichetta
  local sym; sym=$(nm -n "$1" | awk '{print $3}' | grep -iE '8run_loop17h|2Vm8run_loop$' | head -n 1)
  [ -n "$sym" ] || { echo "simbolo run_loop non trovato"; return; }
  objdump -d --no-show-raw-insn --disassemble-symbols="$sym" "$1" > "$OUT/disasm-$2-run_loop.s" 2>/dev/null
  echo "istr=$(grep -cE '^ *[0-9a-f]+:' "$OUT/disasm-$2-run_loop.s") bl=$(grep -cE '[[:space:]]bl[[:space:]]' "$OUT/disasm-$2-run_loop.s") blr=$(grep -cE '[[:space:]]blr[[:space:]]' "$OUT/disasm-$2-run_loop.s") sp_refs=$(grep -c '\[sp' "$OUT/disasm-$2-run_loop.s")"
}
DPIN=$(dis "$PIN" pin); DZ=$(dis "$OUT/phpr-Z" Z); DB=$(dis "$OUT/phpr-B" B); DT=$(dis "$TREE_RT1" treeRT1)
note "DISASM run_loop: pin s181 $DPIN · Z $DZ · tree-RT1 9ef70777 $DT · B $DB"
num(){ echo "$1" | sed -n "s/.*$2=\([0-9]*\).*/\1/p"; }
IB=$(num "$DB" istr); IT=$(num "$DT" istr); SPB=$(num "$DB" sp_refs); SPT=$(num "$DT" sp_refs); BLB=$(num "$DB" bl); BLT=$(num "$DT" bl)
if [ -n "$IB" ] && [ -n "$IT" ] && [ -n "$SPB" ] && [ -n "$SPT" ]; then
  DI=$((IB-IT)); DSP=$((SPB-SPT)); DBL=$((BLB-BLT))
  if [ "$DI" -lt 0 ] && [ "$DSP" -le 40 ]; then note "GATE disasm (criterio-rt2 p.7 forma 2): Δistr=$DI (<0) Δsp_refs=$DSP (≤40) vs tree-RT1 ⇒ RISPETTATO · Δbl=$DBL a verbale"; else note "GATE disasm (forma 2): Δistr=$DI Δsp_refs=$DSP FUORI attesa (istr<0, sp_refs≤40) ⇒ meccanismo da rileggere, niente misura -> rc=5 · Δbl=$DBL"; [ "$RC" -eq 0 ] && RC=5; fi
else
  note "GATE disasm: conteggi non leggibili (B='$DB' tree='$DT') -> rc=5"; [ "$RC" -eq 0 ] && RC=5
fi
rm -rf "$TGT" "$SRC"
note "ESITO rc=$RC (0 = B a parità, mutante che morde, disasm in tolleranza: bracci pronti; 2 = B diverge; 3 = mutante non morde; 5 = disasm fuori tolleranza) fine $(date '+%F %T')"
fin $RC
