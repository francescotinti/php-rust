#!/bin/bash
# s181-leva-build.sh — bracci Z (gemello del pin s180) e B (leva L-CR1) + MUTANTE (criterio s181-criterio-cr1.md p.2/p.3/p.6):
# COPIA DICHIARATA di ../wp177-harness/s177-leva-build.sh (manifest s181-leva-build-copia.diff) coi SOLI adattamenti:
# (1) TRE build nella stessa target separata: Z = sorgente del pin (ZCOMMIT, atteso == pin al byte ⇒ SL in-run NON
# stimabile, dichiarato), B = commit della leva (COMMIT), M = B + mutante «ip=1 anche sul cammino LENTO di Call»
# (sostituzione sull'archivio, mai sul tree) che DEVE rompere fx-cr1 sulle righe ACE (esito ESATTO); (2) pin atteso s180
# 884399fc52277119, tag s181 (out ab-out/s181-leva, sorgente/target /Volumes/Extreme Pro/Claude/s181-leva{,-tgt});
# (3) parità: fx-sw2-gc B == pin a blocchi, fx-sw1/fx-sl1/fx-sl2/fx-sl3 B == oracle byte-id, PIÙ fx-cr1 (nuova, S-181)
# B == oracle byte-id con marcatore FX-CR1 DONE; (4) disasm run_loop di B E del pin s180 agli atti (istr/bl/blr/sp_refs).
# rc (ab-out/s181-leva.done): 0 = B a parità e mutante che morde ⇒ bracci pronti · 2 = B diverge · 3 = mutante NON morde
# (fixture non presidia: niente misura) · 4 = build fallita · 6 = B == pin · 7 = file/commit · 8 = disco · 9 = lock/pin.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment"
H="$REPO/php-rust/wp181-harness"; OUT="$H/ab-out/s181-leva"; mkdir -p "$OUT"
VERD="$H/s181-leva-build-verdetto.out"; DONE="$H/ab-out/s181-leva.done"; rm -f "$DONE"
LOCK=/private/tmp/phpr-measure.lock
PIN="$HOME/Claude/php-rust-output/release/phpr"; PIN_ATTESO="884399fc52277119"
ORACLE=/opt/homebrew/opt/php/bin/php
COMMIT="${COMMIT:?COMMIT della leva (hash)}"; ZCOMMIT="${ZCOMMIT:?ZCOMMIT del sorgente del pin (hash)}"
FX2="$REPO/php-rust/wp174-harness/fixtures/fx-sw2-gc.php"
FX1="$REPO/php-rust/wp174-harness/fixtures/fx-sw1.php"
H2="$REPO/php-rust/wp172-harness"
FXCR="$H/fx-cr1.php"
SRC="/Volumes/Extreme Pro/Claude/s181-leva"; TGT="/Volumes/Extreme Pro/Claude/s181-leva-tgt"
: > "$VERD"
fin(){ echo "rc=$1 $(date +%T)" > "$DONE"; exit "$1"; }
note(){ echo "$*" >> "$VERD"; }

grep -qw s181 "$LOCK" 2>/dev/null || { note "rc=9 lock s181 assente (per TOKEN)"; fin 9; }
for f in "$FX2" "$FX1" "$H2/fx-sl1.php" "$H2/fx-sl2.php" "$H2/fx-sl3.php" "$FXCR"; do [ -s "$f" ] || { note "rc=7 fixture assente: $f"; fin 7; }; done
PH=$(shasum -a 256 "$PIN" | cut -c1-16)
[ "$PH" = "$PIN_ATTESO" ] || { note "rc=9 pin $PH ≠ atteso $PIN_ATTESO"; fin 9; }
cd "$REPO" || fin 7
SHA0=$(git rev-parse --verify "$COMMIT^{commit}") || { note "rc=7 commit $COMMIT inesistente"; fin 7; }
SHAZ=$(git rev-parse --verify "$ZCOMMIT^{commit}") || { note "rc=7 commit $ZCOMMIT inesistente"; fin 7; }
AVX=$(df -k "/Volumes/Extreme Pro" | awk 'NR>1{printf "%.0f", $4/1048576}')
note "== s181 bracci Z/B/M leva L-CR1 — riferimento A = pin s180 $PH, sorgente B = commit ${SHA0:0:12}, Z = commit ${SHAZ:0:12}, fixture fx-sw2-gc $(wc -l < "$FX2" | tr -d ' ') righe + fx-sw1 $(wc -l < "$FX1" | tr -d ' ') righe + fx-sl1/fx-sl2/fx-sl3 + fx-cr1 $(wc -l < "$FXCR" | tr -d ' ') righe, Extreme ${AVX}G $(date '+%F %T') =="
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
NACE=$(grep -c '^ACE' "$OUT/fxcr1-oracle.out")
note "RIFERIMENTO: pin fx-sw2-gc $(tr '\n' ' ' < "$OUT/ref.out" | cut -c1-160) · fx-sw1 pin==oracle BYTE-ID · fx-cr1 pin==oracle BYTE-ID (righe ACE: $NACE)"

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

# Z = sorgente del pin nella target separata (gemello: se byte-id col pin lo si dichiara)
archivio "$SHAZ" || { note "rc=7 archivio Z fallito"; fin 7; }
ZH=$(build Z) || { note "rc=4 Z: build FALLITA (ab-out/s181-leva/build-Z.log)"; fin 4; }
if [ "$ZH" = "$PH" ]; then note "Z: binario $ZH == pin BYTE-ID (gemello byte-id: SL in-run NON stimabile — dichiarato; |A−Z| misura il solo rumore)"; else note "Z: binario $ZH ≠ pin $PH (gemello a contenuto: SL in-run stimabile)"; fi

# B = leva
archivio "$SHA0" || { note "rc=7 archivio B fallito"; fin 7; }
BH=$(build B) || { note "rc=4 B: build FALLITA (ab-out/s181-leva/build-B.log)"; fin 4; }
[ "$BH" != "$PH" ] || { note "rc=6 B: binario == pin (sorgente NON entrata)"; fin 6; }
note "B: binario $BH (commit ${SHA0:0:12})"

# M = B + mutante: ip=1 anche sul cammino LENTO di Op::Call (dopo bind_params) — sostituzione sull'ARCHIVIO
python3 - "$SRC/php-rust/crates/php-runtime/src/vm/run.rs" <<'PY' || { echo "mutante non applicato" >> "$OUT/mutante.err"; }
import sys
p = sys.argv[1]; s = open(p).read()
i = s.index('Op::Call { func, argc } => {')
j = s.index('bind_params(&mut frame, args);\n', i)
assert j - i < 4000, 'sito lento troppo lontano'
ins = 'bind_params(&mut frame, args);\n                        if matches!(callee.ops.first(), Some(Op::CheckArity { .. })) { frame.ip = 1; } // MUTANTE s181\n'
s = s[:j] + ins + s[j + len('bind_params(&mut frame, args);\n'):]
open(p, 'w').write(s)
print('mutante applicato')
PY
[ -e "$OUT/mutante.err" ] && { note "rc=7 mutante non applicabile (sito non trovato)"; fin 7; }
MH=$(build M) || { note "rc=4 M: build FALLITA (ab-out/s181-leva/build-M.log)"; fin 4; }
[ "$MH" != "$BH" ] || { note "rc=7 M: binario == B (mutante NON entrato)"; fin 7; }
note "M: binario $MH (B + mutante ip=1 sul cammino lento)"

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
  if diff -q "$OUT/$n-oracle.out" "$OUT/$n-B.out" > /dev/null; then note "B: $n == oracle BYTE-ID"; else diff "$OUT/$n-oracle.out" "$OUT/$n-B.out" > "$OUT/$n.diff" || true; note "B: $n DIVERGE dall'oracle ($(wc -l < "$OUT/$n.diff" | tr -d ' ') righe, ab-out/s181-leva/$n.diff) -> rc=2"; RC=2; fi
}
bilat fxsw1 "$FX1" "FX-SW1 DONE"
bilat fxsl1 "$H2/fx-sl1.php" "FX-SL1 DONE" -d log_errors=0 -d display_errors=1
bilat fxsl2 "$H2/fx-sl2.php" "FX-SL2 DONE" -d log_errors=0 -d display_errors=1
bilat fxsl3 "$H2/fx-sl3.php" "FX-SL3 DONE" -d log_errors=0 -d display_errors=1
bilat fxcr1 "$FXCR" "FX-CR1 DONE" -d log_errors=0 -d display_errors=1

# mutante: DEVE divergere dall'oracle su fx-cr1 ESATTAMENTE sulle righe ACE (esito esatto, mai «diverso da»)
perl -e 'alarm 120; exec @ARGV or die' -- "$OUT/phpr-M" "$FXCR" > "$OUT/fxcr1-M.out" 2>&1
MACE=$(grep -c '^ACE' "$OUT/fxcr1-M.out")
NONACE_DIFF=$(diff <(grep -v '^ACE' "$OUT/fxcr1-oracle.out") <(grep -v '^ACE' "$OUT/fxcr1-M.out") | grep -c '^[<>]')
if [ "$MACE" -lt "$NACE" ] && [ "$NONACE_DIFF" -eq 0 ]; then
  note "MUTANTE: morde — righe ACE oracle $NACE vs M $MACE (le altre righe identiche): la fixture presidia il rischio (b)"
else
  diff "$OUT/fxcr1-oracle.out" "$OUT/fxcr1-M.out" > "$OUT/fxcr1-M.diff" || true
  note "MUTANTE: NON morde nella forma attesa (ACE oracle $NACE vs M $MACE, righe non-ACE diverse $NONACE_DIFF; ab-out/s181-leva/fxcr1-M.diff) -> rc=3"; [ "$RC" -eq 0 ] && RC=3
fi

# disasm agli atti (p.6): istr/bl/blr/sp_refs di run_loop di B vs pin s180 (S-104: ogni leva su run_loop pretende il disasm)
dis(){ # $1=binario $2=etichetta
  local sym; sym=$(nm -n "$1" | awk '{print $3}' | grep -i '8run_loop17h' | head -n 1)
  [ -n "$sym" ] || { echo "simbolo run_loop non trovato"; return; }
  objdump -d --no-show-raw-insn --disassemble-symbols="$sym" "$1" > "$OUT/disasm-$2-run_loop.s" 2>/dev/null
  echo "istr=$(grep -cE '^ *[0-9a-f]+:' "$OUT/disasm-$2-run_loop.s") bl=$(grep -cE '[[:space:]]bl[[:space:]]' "$OUT/disasm-$2-run_loop.s") blr=$(grep -cE '[[:space:]]blr[[:space:]]' "$OUT/disasm-$2-run_loop.s") sp_refs=$(grep -c '\[sp' "$OUT/disasm-$2-run_loop.s")"
}
note "DISASM run_loop: pin s180 $(dis "$PIN" pin) · B $(dis "$OUT/phpr-B" B)"
rm -rf "$TGT" "$SRC"
note "ESITO rc=$RC (0 = B a parità e mutante che morde: bracci pronti; 2 = B diverge; 3 = mutante non morde) fine $(date '+%F %T')"
fin $RC
