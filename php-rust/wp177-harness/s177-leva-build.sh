#!/bin/bash
# s177-leva-build.sh — braccio C della leva L-CM1 (criterio s177-criterio-cm1.md p.3 e p.6): COPIA DICHIARATA di
# ../wp176-harness/s176-flag-build.sh (manifest s177-leva-build-copia.diff) coi SOLI adattamenti: (1) UNA sola build
# (C = tree + L-CM1; NESSUN mutante: la leva non ha predicato da mordere, p.6); (2) tag s177 (out ab-out/s177-leva,
# sorgente/target /Volumes/Extreme Pro/Claude/s177-leva{,-tgt}); (3) parità: fx-sw2-gc C == pin (invarianza, blocchi
# rotti per etichetta se diverge), fx-sw1 C == oracle byte-id, PIÙ fx-sl1/fx-sl2/fx-sl3 (wp172-harness) C == oracle
# byte-id con le opzioni oracle della promozione (-d log_errors=0 -d display_errors=1) e marcatori FX-SLn DONE;
# (4) disasm di run_loop del braccio C agli atti (istr/bl/blr/sp_refs vs pin, p.7). rc (ab-out/s177-leva.done):
# 0 = C a parità ⇒ braccio pronto · 2 = C diverge · 4 = build fallita · 6 = binario == pin · 7 = file/commit ·
# 8 = disco · 9 = lock/pin.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment"
H="$REPO/php-rust/wp177-harness"; OUT="$H/ab-out/s177-leva"; mkdir -p "$OUT"
VERD="$H/s177-leva-build-verdetto.out"; DONE="$H/ab-out/s177-leva.done"; rm -f "$DONE"
LOCK=/private/tmp/phpr-measure.lock
PIN="$HOME/Claude/php-rust-output/release/phpr"; PIN_ATTESO="5de14d6856d760a8"
ORACLE=/opt/homebrew/opt/php/bin/php
COMMIT="${COMMIT:?COMMIT della leva (hash)}"
FX2="$REPO/php-rust/wp174-harness/fixtures/fx-sw2-gc.php"
FX1="$REPO/php-rust/wp174-harness/fixtures/fx-sw1.php"
H2="$REPO/php-rust/wp172-harness"
SRC="/Volumes/Extreme Pro/Claude/s177-leva"; TGT="/Volumes/Extreme Pro/Claude/s177-leva-tgt"
: > "$VERD"
fin(){ echo "rc=$1 $(date +%T)" > "$DONE"; exit "$1"; }
note(){ echo "$*" >> "$VERD"; }

grep -qw s177 "$LOCK" 2>/dev/null || { note "rc=9 lock s177 assente (per TOKEN)"; fin 9; }
for f in "$FX2" "$FX1" "$H2/fx-sl1.php" "$H2/fx-sl2.php" "$H2/fx-sl3.php"; do [ -s "$f" ] || { note "rc=7 fixture assente: $f"; fin 7; }; done
PH=$(shasum -a 256 "$PIN" | cut -c1-16)
[ "$PH" = "$PIN_ATTESO" ] || { note "rc=9 pin $PH ≠ atteso $PIN_ATTESO"; fin 9; }
cd "$REPO" || fin 7
SHA0=$(git rev-parse --verify "$COMMIT^{commit}") || { note "rc=7 commit $COMMIT inesistente"; fin 7; }
AVX=$(df -k "/Volumes/Extreme Pro" | awk 'NR>1{printf "%.0f", $4/1048576}')
note "== s177 braccio C leva L-CM1 — riferimento A = pin s175 $PH, sorgente = commit ${SHA0:0:12}, fixture fx-sw2-gc $(wc -l < "$FX2" | tr -d ' ') righe + fx-sw1 $(wc -l < "$FX1" | tr -d ' ') righe + fx-sl1/fx-sl2/fx-sl3, Extreme ${AVX}G $(date '+%F %T') =="
awk -v a="$AVX" 'BEGIN{exit !(a+0 < 15)}' && { note "rc=8 Extreme ${AVX}G < 15G: niente build"; fin 8; }

# riferimento: pin su fx-sw2-gc (marcatore) e gate bilaterale fx-sw1 sul pin (sanità del gate: passato alla promozione)
perl -e 'alarm 120; exec @ARGV or die' -- "$PIN" "$FX2" > "$OUT/ref.out" 2>&1
grep -q "FX-SW2 DONE" "$OUT/ref.out" || { note "rc=7 riferimento: marcatore FX-SW2 assente"; fin 7; }
"$ORACLE" "$FX1" > "$OUT/sw1-oracle.out" 2>&1
perl -e 'alarm 120; exec @ARGV or die' -- "$PIN" "$FX1" > "$OUT/sw1-pin.out" 2>&1
grep -q "FX-SW1 DONE" "$OUT/sw1-pin.out" || { note "rc=7 riferimento: marcatore FX-SW1 assente"; fin 7; }
diff -q "$OUT/sw1-oracle.out" "$OUT/sw1-pin.out" > /dev/null || { note "rc=7 fx-sw1: pin ≠ oracle (gate rotto a monte)"; fin 7; }
note "RIFERIMENTO: pin fx-sw2-gc $(tr '\n' ' ' < "$OUT/ref.out" | cut -c1-160) · fx-sw1 pin==oracle BYTE-ID"

rm -rf "$SRC"; mkdir -p "$SRC/php-rust"
git archive "$SHA0" php-rust/crates php-rust/Cargo.toml php-rust/Cargo.lock php-rust/rust-toolchain.toml php-rust/.cargo 2>/dev/null | tar -x -C "$SRC" \
  || { note "rc=7 archivio del commit fallito"; fin 7; }
[ -s "$SRC/php-rust/crates/php-runtime/src/vm/run.rs" ] || { note "rc=7 archivio senza run.rs"; fin 7; }
/usr/bin/find "$SRC" -type f -exec touch {} +

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

( cd "$SRC/php-rust" && SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TGT" \
    cargo build --release -p php-cli ) > "$OUT/build-C.log" 2>&1 \
  || { note "rc=4 C: build FALLITA (ab-out/s177-leva/build-C.log)"; fin 4; }
CH=$(shasum -a 256 "$TGT/release/phpr" | cut -c1-16)
[ "$CH" != "$PH" ] || { note "rc=6 C: binario == pin (sorgente NON entrata)"; fin 6; }
cp "$TGT/release/phpr" "$OUT/phpr-C"
note "C: binario $CH (commit ${SHA0:0:12})"

RC=0
perl -e 'alarm 120; exec @ARGV or die' -- "$OUT/phpr-C" "$FX2" > "$OUT/C.out" 2>&1
rotte "$OUT/ref.out" "$OUT/C.out" | sort -u > "$OUT/C.rotte"
if diff -q "$OUT/ref.out" "$OUT/C.out" > /dev/null; then note "C: fx-sw2-gc == pin BYTE-ID (invarianza)"; else diff "$OUT/ref.out" "$OUT/C.out" > "$OUT/C-invarianza.diff" || true; note "C: fx-sw2-gc ≠ pin — blocchi ROTTI ($(wc -l < "$OUT/C.rotte" | tr -d ' ')): $(tr '\n' ' ' < "$OUT/C.rotte") -> rc=2"; RC=2; fi
bilat(){ # $1=nome $2=file $3=marcatore $4..=opzioni oracle
  local n="$1" f="$2" m="$3"; shift 3
  "$ORACLE" "$@" "$f" > "$OUT/$n-oracle.out" 2>&1
  perl -e 'alarm 300; exec @ARGV or die' -- "$OUT/phpr-C" "$f" > "$OUT/$n-C.out" 2>&1
  [ -s "$OUT/$n-C.out" ] || { note "C: $n output VUOTO -> rc=2"; RC=2; return; }
  [ -z "$m" ] || grep -q "$m" "$OUT/$n-C.out" || { note "C: $n marcatore $m ASSENTE -> rc=2"; RC=2; return; }
  if diff -q "$OUT/$n-oracle.out" "$OUT/$n-C.out" > /dev/null; then note "C: $n == oracle BYTE-ID"; else diff "$OUT/$n-oracle.out" "$OUT/$n-C.out" > "$OUT/$n.diff" || true; note "C: $n DIVERGE dall'oracle ($(wc -l < "$OUT/$n.diff" | tr -d ' ') righe, ab-out/s177-leva/$n.diff) -> rc=2"; RC=2; fi
}
bilat fxsw1 "$FX1" "FX-SW1 DONE"
bilat fxsl1 "$H2/fx-sl1.php" "FX-SL1 DONE" -d log_errors=0 -d display_errors=1
bilat fxsl2 "$H2/fx-sl2.php" "FX-SL2 DONE" -d log_errors=0 -d display_errors=1
bilat fxsl3 "$H2/fx-sl3.php" "FX-SL3 DONE" -d log_errors=0 -d display_errors=1

# disasm agli atti (p.7): istr/bl/blr/sp_refs di run_loop del braccio C vs pin (S-104: ogni leva su run_loop pretende il disasm)
SYM=$(nm -n "$OUT/phpr-C" | awk '{print $3}' | grep -i '8run_loop17h' | head -n 1)
if [ -n "$SYM" ]; then
  objdump -d --no-show-raw-insn --disassemble-symbols="$SYM" "$OUT/phpr-C" > "$OUT/disasm-C-run_loop.s" 2>/dev/null
  NI=$(grep -cE '^ *[0-9a-f]+:' "$OUT/disasm-C-run_loop.s"); NBL=$(grep -cE '[[:space:]]bl[[:space:]]' "$OUT/disasm-C-run_loop.s"); NBLR=$(grep -cE '[[:space:]]blr[[:space:]]' "$OUT/disasm-C-run_loop.s"); NSP=$(grep -c '\[sp' "$OUT/disasm-C-run_loop.s")
  note "DISASM C run_loop: istr=$NI bl=$NBL blr=$NBLR sp_refs=$NSP (pin s175: istr=71745 bl=5993 blr=6 sp_refs=11270 — wp177-harness/disasm-pin-run_loop-stats.txt)"
else
  note "DISASM C: simbolo run_loop non trovato (dichiarato)"
fi
rm -rf "$TGT" "$SRC"
note "ESITO rc=$RC (0 = C a parità: braccio pronto; 2 = C diverge) fine $(date '+%F %T')"
fin $RC
