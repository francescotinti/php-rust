#!/bin/bash
# s190-leva-build.sh — S-190 cura O(1) v2: costruisce il SOLO braccio B3 (tree con la v2, B3COMMIT) nella target separata
# della bundle, da ARCHIVIO (mai il tree), con la ricetta del pin (`cargo build --release -p php-cli`, SOURCE_DATE_EPOCH=0,
# CARGO_INCREMENTAL=0). Il gemello Z è RIUSATO da ../wp186-harness/ab-out/s186-leva/phpr-Z (23c0a94c, dichiarato). Script NUOVO
# col pattern di ../wp186-harness/s186-leva-build.sh (archivio(), build(), bilat(), dis() ripresi alla lettera; tolti M, C, xctrace):
# gate = pin canonico s183 per hash, Data ≥10G, Extreme ≥15G, bundle montata; B3 ≠ pin; 9 fixture bilaterali vs oracle (fx-mc2
# contro wp186-harness/fx-mc2.expected-diff VUOTO); fx-sw2-gc vs pin (invarianza, a verbale); DISASM run_loop B3 vs pin:
# Δsp_refs ≤ +97 E |Δbl| ≤ 2 (regola irrigidita S-186) = gate rc=5. Esiti SOLO in ab-out/s190-leva.done; verdetto
# s190-leva-build-verdetto.out. Generato con assistenza AI (Claude Fable 5).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment"
P="$REPO/php-rust"; H="$P/wp190-harness"
OUT="$H/ab-out/s190-leva"; mkdir -p "$OUT"
VERD="$H/s190-leva-build-verdetto.out"; DONE="$H/ab-out/s190-leva.done"; rm -f "$DONE"
PIN="$HOME/Claude/php-rust-output/release/phpr"; PIN_ATTESO="9a6ae11bf7e900ef"
Z="$P/wp186-harness/ab-out/s186-leva/phpr-Z"; Z_ATTESO="23c0a94c26db62eb"
ORACLE=/opt/homebrew/opt/php/bin/php
B3COMMIT="${B3COMMIT:?B3COMMIT = commit del tree con la cura O(1) v2}"
FX1="$P/wp174-harness/fixtures/fx-sw1.php"; FX2="$P/wp174-harness/fixtures/fx-sw2-gc.php"; H2="$P/wp172-harness"
FXCR="$P/wp181-harness/fx-cr1.php"; FXRT="$P/wp182-harness/fx-rt1.php"; FXRT2="$P/wp184-harness/fx-rt2.php"; FXMC2="$P/wp185-harness/fx-mc2.php"
SRC="/Volumes/Extreme Pro/Claude/s190-leva"; BUNDLE_MP="$HOME/Claude/phpr-target"; TGT="$BUNDLE_MP/s190-leva-tgt"
: > "$VERD"
note(){ echo "$(date '+%H:%M:%S') $1" | tee -a "$VERD"; }
fin(){ echo "rc=$1 $(date '+%F %T')" > "$DONE"; exit "$1"; }

PH=$(shasum -a 256 "$PIN" | cut -c1-16)
[ "$PH" = "$PIN_ATTESO" ] || { note "rc=9 pin canonico $PH != atteso $PIN_ATTESO"; fin 9; }
ZH=$(shasum -a 256 "$Z" 2>/dev/null | cut -c1-16)
[ "$ZH" = "$Z_ATTESO" ] || { note "rc=9 gemello Z $ZH != atteso $Z_ATTESO ($Z)"; fin 9; }
/sbin/mount | grep -q "$BUNDLE_MP" || { note "rc=8 bundle target NON montata ($BUNDLE_MP)"; fin 8; }
cd "$REPO" || { note "rc=7 repo"; fin 7; }
SHA3=$(git rev-parse --verify "$B3COMMIT^{commit}") || { note "rc=7 commit $B3COMMIT inesistente"; fin 7; }
AVX=$(df -k "/Volumes/Extreme Pro" | awk 'NR>1{printf "%.0f", $4/1048576}')
DATAG=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}')
note "== s190 braccio B3 — A = pin s183 $PH · Z (riuso S-186) $ZH · B3 = ${SHA3:0:12} · Extreme ${AVX}G Data ${DATAG}G"
awk -v a="$AVX" 'BEGIN{exit !(a+0 < 15)}' && { note "rc=8 Extreme ${AVX}G < 15G: niente build"; fin 8; }
[ "${DATAG:-0}" -ge 10 ] || { note "rc=8 Data ${DATAG}G < 10G: niente build"; fin 8; }

archivio(){ # $1=commit → sorgente pulito in $SRC (mai il tree)
  rm -rf "$SRC"; mkdir -p "$SRC/php-rust"
  git archive "$1" php-rust/crates php-rust/Cargo.toml php-rust/Cargo.lock php-rust/rust-toolchain.toml php-rust/.cargo 2>/dev/null | tar -x -C "$SRC" || return 1
  [ -s "$SRC/php-rust/crates/php-runtime/src/vm/run.rs" ] || return 1
  /usr/bin/find "$SRC" -type f -exec touch {} +
}
build(){ # $1=etichetta → stampa hash16; log in $OUT/build-$1.log
  ( cd "$SRC/php-rust" && SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TGT" cargo build --release -p php-cli ) > "$OUT/build-$1.log" 2>&1 || return 1
  cp "$TGT/release/phpr" "$OUT/phpr-$1"; shasum -a 256 "$OUT/phpr-$1" | cut -c1-16
}
if [ "${SKIP_BUILD:-0}" = 1 ] && [ -s "$OUT/phpr-B3" ]; then
  B3H=$(shasum -a 256 "$OUT/phpr-B3" | cut -c1-16); note "SKIP_BUILD=1: riuso dichiarato di phpr-B3 $B3H"
else
  archivio "$SHA3" || { note "rc=7 archivio B3 fallito"; fin 7; }
  B3H=$(build B3) || { note "rc=4 B3: build FALLITA (ab-out/s190-leva/build-B3.log)"; fin 4; }
fi
[ "$B3H" != "$PH" ] || { note "rc=6 B3: binario == pin (sorgente NON entrata)"; fin 6; }
note "B3: binario $B3H (commit ${SHA3:0:12}, cura O(1) v2)"

RC=0
perl -e 'alarm 300; exec @ARGV or die' -- "$PIN" "$FX2" > "$OUT/ref.out" 2>&1
perl -e 'alarm 300; exec @ARGV or die' -- "$OUT/phpr-B3" "$FX2" > "$OUT/B3-sw2.out" 2>&1
if diff -q "$OUT/ref.out" "$OUT/B3-sw2.out" > /dev/null; then note "B3: fx-sw2-gc == pin BYTE-ID (invarianza)"; else diff "$OUT/ref.out" "$OUT/B3-sw2.out" > "$OUT/B3-invarianza.diff" || true; note "B3: fx-sw2-gc ≠ pin — $(grep -c '^[<>]' "$OUT/B3-invarianza.diff") righe — a verbale"; fi
bilat(){ # $1=nome $2=file $3=marcatore $4..=opzioni oracle → B3 == oracle (fx-mc2: diff == expected-diff)
  local n="$1" f="$2" m="$3"; shift 3
  "$ORACLE" "$@" "$f" > "$OUT/$n-oracle.out" 2>&1
  perl -e 'alarm 300; exec @ARGV or die' -- "$OUT/phpr-B3" "$@" "$f" > "$OUT/$n-B3.out" 2>&1
  [ -s "$OUT/$n-B3.out" ] || { note "B3: $n output VUOTO -> rc=2"; RC=2; return; }
  [ -z "$m" ] || grep -q "$m" "$OUT/$n-B3.out" || { note "B3: $n marcatore $m ASSENTE -> rc=2"; RC=2; return; }
  if [ "$n" = fxmc2 ]; then
    diff "$OUT/$n-oracle.out" "$OUT/$n-B3.out" > "$OUT/$n-B3.diff" || true
    if cmp -s "$OUT/$n-B3.diff" "$P/wp186-harness/fx-mc2.expected-diff"; then note "B3: fx-mc2 diff == expected-diff (vuoto ⇒ byte-id)"; else note "B3: fx-mc2 diff ≠ expected-diff ($(grep -c '^[<>]' "$OUT/$n-B3.diff") righe) -> rc=2"; RC=2; fi
  elif diff -q "$OUT/$n-oracle.out" "$OUT/$n-B3.out" > /dev/null; then note "B3: $n == oracle BYTE-ID"
  else diff "$OUT/$n-oracle.out" "$OUT/$n-B3.out" > "$OUT/$n-B3.diff" || true; note "B3: $n DIVERGE dall'oracle ($(grep -c '^[<>]' "$OUT/$n-B3.diff") righe) -> rc=2"; RC=2; fi
}
bilat fxsw1 "$FX1" "FX-SW1 DONE"
bilat fxsl1 "$H2/fx-sl1.php" "FX-SL1 DONE" -d log_errors=0 -d display_errors=1
bilat fxsl2 "$H2/fx-sl2.php" "FX-SL2 DONE" -d log_errors=0 -d display_errors=1
bilat fxsl3 "$H2/fx-sl3.php" "FX-SL3 DONE" -d log_errors=0 -d display_errors=1
bilat fxcr1 "$FXCR" "FX-CR1 DONE" -d log_errors=0 -d display_errors=1
bilat fxrt1 "$FXRT" "FX-RT1 DONE" -d log_errors=0 -d display_errors=1
bilat fxrt2 "$FXRT2" "FX-RT2 DONE" -d log_errors=0 -d display_errors=1
bilat fxmc2 "$FXMC2" "FX-MC2 DONE" -d log_errors=0 -d display_errors=1

# disasm: istr/bl/blr/sp_refs di run_loop di B3 vs il PIN (gate Δsp_refs ≤ +97 e |Δbl| ≤ 2)
dis(){ # $1=binario $2=etichetta
  local sym; sym=$(nm -n "$1" | awk '{print $3}' | grep -iE '8run_loop17h|2Vm8run_loop$' | head -n 1)
  [ -n "$sym" ] || { echo "simbolo run_loop non trovato"; return; }
  objdump -d --no-show-raw-insn --disassemble-symbols="$sym" "$1" > "$OUT/disasm-$2-run_loop.s" 2>/dev/null
  echo "istr=$(grep -cE '^ *[0-9a-f]+:' "$OUT/disasm-$2-run_loop.s") bl=$(grep -cE '[[:space:]]bl[[:space:]]' "$OUT/disasm-$2-run_loop.s") blr=$(grep -cE '[[:space:]]blr[[:space:]]' "$OUT/disasm-$2-run_loop.s") sp_refs=$(grep -c '\[sp' "$OUT/disasm-$2-run_loop.s")"
}
num(){ echo "$1" | sed -n "s/.*$2=\([0-9]*\).*/\1/p"; }
DPIN=$(dis "$PIN" pin); DB=$(dis "$OUT/phpr-B3" B3)
note "DISASM run_loop: pin s183 $DPIN · B3 $DB"
IB=$(num "$DB" istr); IT=$(num "$DPIN" istr); SPB=$(num "$DB" sp_refs); SPT=$(num "$DPIN" sp_refs); BLB=$(num "$DB" bl); BLT=$(num "$DPIN" bl)
if [ -n "$IB" ] && [ -n "$IT" ] && [ -n "$SPB" ] && [ -n "$SPT" ] && [ -n "$BLB" ] && [ -n "$BLT" ]; then
  DI=$((IB-IT)); DSP=$((SPB-SPT)); DBL=$((BLB-BLT)); ABL=${DBL#-}
  if [ "$DSP" -le 97 ] && [ "$ABL" -le 2 ]; then note "GATE disasm B3 vs pin: Δsp_refs=$DSP (≤ +97) Δbl=$DBL (|Δbl| ≤ 2) ⇒ RISPETTATO · Δistr=$DI"
  else note "GATE disasm B3: Δsp_refs=$DSP Δbl=$DBL FUORI tolleranza (≤ +97, |Δbl| ≤ 2) Δistr=$DI -> rc=5"; [ "$RC" -eq 0 ] && RC=5; fi
else note "GATE disasm B3: conteggi non leggibili (B3='$DB' pin='$DPIN') -> rc=5"; [ "$RC" -eq 0 ] && RC=5; fi
rm -rf "$TGT" "$SRC"
note "ESITO rc=$RC (0 = B3 a parità e disasm in tolleranza: braccio pronto; 2 = fixture divergono; 5 = disasm fuori) fine $(date '+%F %T')"
fin $RC
