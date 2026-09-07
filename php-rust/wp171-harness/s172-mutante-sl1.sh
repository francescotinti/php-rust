#!/bin/bash
# s172-mutante-sl1.sh — az.rev. S-171 rilievo 1 (wp171-harness/revisione.md): MUTANTE
# ABORTIVO del fast path L-SL1 contro fx-sl1 (ESTESA in S-172: typed-prop-ref, shl/shr su
# l negativo). Prova che il fast path è PRESO: un mutante che altera SOLO il ramo veloce
# deve rompere righe NOMINATE; le righe che devono cadere al corpo esatto restano
# byte-identiche al pin (se una di esse si rompe = fast path preso FUORI dominio: rc=5).
# Due mutanti SEPARATI (attribuzione per riga; un mutante unico confonderebbe: il cmp
# negato spegne i loop e nasconderebbe l'arith):
#   M1 arith = `.and_then(|r| long_arith_i64(*opd, *xl, r))` → `.map(|v| v.wrapping_add(1))`
#              (= «Some(r)→Some(r+1)» del rilievo, sul risultato del fast path BinarySCSCDst)
#   M2 cmp   = `long_cmp_i64`: `Some(match b {` → `Some(!match b {` (confronto NEGATO)
# ATTESA PRE-registrata (righe ROTTE, per etichetta): M1 ⊇ {dq100, bitops, shr-64,
# shr-70-neg, shl-64, shl-neg-l, shl-neg-l-62, shr-neg-l} (a VERDETTO, non attese: dst==src e
# assign-form = abbassamento a BinarySCSCDst da leggere; shl-63 = `21 − MIN` overflow su Sub
# ⇒ corpo esatto PER COSTRUZIONE, dichiarato dopo la corsa 1); M2 ⊇ {dq100, lt-loop, ge-loop,
# ne-loop, nid-loop, lt-max, dec-loop, inc-loop-neg, inc-max-loop, cmp(2), cmp(3), cmp(4)}.
# CORSA 1 (s172-mutante-verdetto-corsa1.out, rc=1): le righe dopo `dst-ref`/`src-ref` NON si
# rompevano perché `unset($r)` lascia `$s`/`$i` come slot `Ref` (miss per TAG): difetto di
# DISEGNO della fixture, curato (variabili dedicate `$rs`/`$ri`), non del fast path.
# CORSA 2: sorgente del mutante = `git archive HEAD` (nasce da un COMMIT: il repo può
# avanzare intanto), binari conservati in ab-out/s172-mut/phpr-M{1,2}.
# DEVONO restare INTATTE (dominio non-Long / Ref / overflow / Div-Mod-Pow): M1 =
# {mul-overflow add-overflow-dst sub-overflow-neg shr-neg shl-neg div-exact-mod div-inexact
# div-zero mod-zero mod-min-neg1 pow pow-overflow double-slot double-dst numeric-string
# numeric-string-dst null-slot bool-slot dst-ref dst-ref-alias src-ref typed-ref
# typed-ref-overflow typed-prop-ref typed-prop-ref-alias typed-prop-ref-overflow};
# M2 = {cmp(2.5) cmp(3.0) cmp('3') cmp('3.0') cmp('abc') cmp(NULL) cmp(true) cmp(false)
# lt-double-loop lt-numstr-loop ref-lt ref-ge}.
# Il REPO NON viene toccato: i mutanti nascono in una COPIA dell'albero (/private/tmp/
# s172-mut, come i census S-161..164); «revert al byte» = `git diff --quiet crates` PRIMA e
# DOPO + hash di run.rs invariato; i diff dei mutanti sono archiviati in ab-out/.
# Ricetta build = quella del pin (SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build
# --release -p php-cli), target dedicato /private/tmp/s172-mut-tgt (riusato M1→M2),
# rimosso in epilogo (il mutante NON si conserva: non è un braccio).
# Esiti: VERD (committato) + ab-out/s172-mut/*; rc SOLO da ab-out/s172-mut.done.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp171-harness"; OUT="$H/ab-out/s172-mut"; mkdir -p "$OUT"
VERD="$H/s172-mutante-verdetto.out"; DONE="$H/ab-out/s172-mut.done"; rm -f "$DONE"
LOCK=/private/tmp/phpr-measure.lock
PIN="$HOME/Claude/php-rust-output/release/phpr"; PIN_ATTESO=b360b2933eddfe18
FX="$H/fx-sl1.php"
SRC=/private/tmp/s172-mut; TGT=/private/tmp/s172-mut-tgt
RS=crates/php-runtime/src/vm/run.rs
: > "$VERD"
fin(){ echo "rc=$1 $(date +%T)" > "$DONE"; exit "$1"; }
note(){ echo "$*" >> "$VERD"; }

grep -qw s172 "$LOCK" 2>/dev/null || { note "rc=9 lock s172 assente (per TOKEN)"; fin 9; }
[ -s "$FX" ] || { note "rc=7 fixture assente"; fin 7; }
PH=$(shasum -a 256 "$PIN" | cut -c1-16)
[ "$PH" = "$PIN_ATTESO" ] || { note "rc=9 pin $PH ≠ atteso $PIN_ATTESO"; fin 9; }
cd "$REPO" || fin 4
SHA0=$(git rev-parse HEAD)
# la RADICE git è la cartella padre (php-rust-experiment/): il path va dato RELATIVO alla cwd
# (`:./`), altrimenti `git show` fallisce a vuoto (corsa 2: hash e3b0… dell'input vuoto = falso incidente)
RS_H0=$(git show "$SHA0:./$RS" | shasum -a 256 | cut -c1-16)
SOLO="${SOLO:-0}"   # SOLO=1: riusa i binari ab-out/s172-mut/phpr-M{1,2} della corsa precedente (nessuna build)
AV=$(df -k /System/Volumes/Data | awk 'NR>1{printf "%.1f", $4/1048576}')
note "== s172 mutante abortivo L-SL1 (corsa 2) — pin $PH (sorgente = commit ${SHA0:0:12}, run.rs $RS_H0), fixture $(wc -l < "$FX" | tr -d ' ') righe, Data ${AV}G $(date '+%F %T') =="
awk -v a="$AV" 'BEGIN{exit !(a+0 < 8)}' && { note "rc=8 Data ${AV}G < 8G: niente build"; fin 8; }

# --- riferimento: pin su fixture (già oracle==pin byte-id nel gate S-172 pre-run) ---
perl -e 'alarm 60; exec @ARGV or die' -- "$PIN" "$FX" > "$OUT/pin.out" 2>&1
grep -q "FX-SL1 DONE" "$OUT/pin.out" || { note "rc=7 pin: marcatore assente"; fin 7; }

# --- copia albero ---
if [ "$SOLO" = 1 ]; then
  note "SOLO=1: binari della corsa precedente riusati (M1 $(shasum -a 256 "$OUT/phpr-M1" | cut -c1-16), M2 $(shasum -a 256 "$OUT/phpr-M2" | cut -c1-16)); diff archiviati ab-out/s172-mut/M{1,2}.diff; nessuna build"
else
rm -rf "$SRC"; mkdir -p "$SRC/php-rust"
git archive "$SHA0" crates Cargo.toml Cargo.lock rust-toolchain.toml .cargo 2>/dev/null | tar -x -C "$SRC/php-rust" \
  || { note "rc=7 archivio del commit fallito"; fin 7; }
[ -s "$SRC/php-rust/$RS" ] || { note "rc=7 archivio senza run.rs"; fin 7; }
cp "$SRC/php-rust/$RS" "$SRC/run.rs.orig"
fi

build_run(){ # $1=nome $2=needle-perl $3=repl-perl
  local n="$1" needle="$2" repl="$3"
  if [ "$SOLO" = 1 ]; then
    [ -x "$OUT/phpr-$n" ] || { note "rc=7 SOLO: binario $n assente"; fin 7; }
    perl -e 'alarm 60; exec @ARGV or die' -- "$OUT/phpr-$n" "$FX" > "$OUT/$n.out" 2>&1
    note "$n: binario $(shasum -a 256 "$OUT/phpr-$n" | cut -c1-16) (riusato) rc_fixture=$?"
    diff "$OUT/pin.out" "$OUT/$n.out" > "$OUT/$n-vs-pin.diff" || true
    grep '^<' "$OUT/$n-vs-pin.diff" | sed -E 's/^< //; s/:.*//; s/ ok$//' | sort -u > "$OUT/$n.rotte"
    note "$n: righe ROTTE ($(wc -l < "$OUT/$n.rotte" | tr -d ' ')): $(tr '\n' ' ' < "$OUT/$n.rotte")"
    return 0
  fi
  cp "$SRC/run.rs.orig" "$SRC/php-rust/$RS"
  perl -0pi -e "s/\Q$needle\E/$repl/" "$SRC/php-rust/$RS"
  local nrep; nrep=$(diff "$SRC/run.rs.orig" "$SRC/php-rust/$RS" | grep -c '^>')
  [ "$nrep" = 1 ] || { note "rc=6 $n: il mutante ha toccato $nrep righe (attesa 1)"; fin 6; }
  diff -u "$SRC/run.rs.orig" "$SRC/php-rust/$RS" > "$OUT/$n.diff"
  ( cd "$SRC/php-rust" && SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TGT" \
      cargo build --release -p php-cli ) > "$OUT/build-$n.log" 2>&1 \
    || { note "rc=4 $n: build FALLITA (ab-out/s172-mut/build-$n.log)"; fin 4; }
  local mh; mh=$(shasum -a 256 "$TGT/release/phpr" | cut -c1-16)
  [ "$mh" != "$PH" ] || { note "rc=6 $n: binario == pin (mutante NON entrato)"; fin 6; }
  cp "$TGT/release/phpr" "$OUT/phpr-$n"
  perl -e 'alarm 60; exec @ARGV or die' -- "$OUT/phpr-$n" "$FX" > "$OUT/$n.out" 2>&1
  local rrc=$?
  note "$n: binario $mh rc_fixture=$rrc"
  diff "$OUT/pin.out" "$OUT/$n.out" > "$OUT/$n-vs-pin.diff" || true
  # etichette ROTTE = righe del pin che nel mutante differiscono (o mancano)
  grep '^<' "$OUT/$n-vs-pin.diff" | sed -E 's/^< //; s/:.*//; s/ ok$//' | sort -u > "$OUT/$n.rotte"
  note "$n: righe ROTTE ($(wc -l < "$OUT/$n.rotte" | tr -d ' ')): $(tr '\n' ' ' < "$OUT/$n.rotte")"
}

verdetto(){ # $1=nome $2=attese(spazio) $3=intatte(spazio)
  local n="$1" bad=0 miss=""; local a
  for a in $2; do grep -qxF "$a" "$OUT/$n.rotte" || miss="$miss $a"; done
  local viol=""
  for a in $3; do grep -qxF "$a" "$OUT/$n.rotte" && viol="$viol $a"; done
  [ -z "$miss" ] && note "$n: attese TUTTE rotte -> fast path PRESO su ogni riga attesa" || { note "$n: attese NON rotte:$miss -> fast path NON preso (o forma non abbassata) su queste"; bad=1; }
  [ -z "$viol" ] && note "$n: righe fuori-dominio INTATTE (nessuna presa fuori dominio)" || { note "$n: VIOLAZIONE dominio — rotte:$viol (fast path preso FUORI dominio)"; bad=2; }
  return $bad
}

build_run M1 '.and_then(|r| long_arith_i64(*opd, *xl, r))' '.and_then(|r| long_arith_i64(*opd, *xl, r).map(|v| v.wrapping_add(1)))'
build_run M2 'fn long_cmp_i64(b: BinOp, l: i64, r: i64) -> Option<bool> {
    use BinOp::*;
    Some(match b {' 'fn long_cmp_i64(b: BinOp, l: i64, r: i64) -> Option<bool> {
    use BinOp::*;
    Some(!match b {'

ATT1="dq100 bitops shr-64 shr-70-neg shl-64 shl-neg-l shl-neg-l-62 shr-neg-l"
INT1="mul-overflow add-overflow-dst sub-overflow-neg shr-neg shl-neg div-exact-mod div-inexact div-zero mod-zero mod-min-neg1 pow pow-overflow double-slot double-dst numeric-string numeric-string-dst null-slot bool-slot dst-ref dst-ref-alias src-ref typed-ref typed-ref-overflow typed-prop-ref typed-prop-ref-alias typed-prop-ref-overflow"
ATT2="dq100 lt-loop ge-loop ne-loop nid-loop lt-max dec-loop inc-loop-neg inc-max-loop cmp(2) cmp(3) cmp(4)"
INT2="cmp(2.5) cmp(3.0) cmp('3') cmp('3.0') cmp('abc') cmp(NULL) cmp(true) cmp(false) lt-double-loop ref-lt ref-ge"
# lt-numstr-loop NON è intatta per costruzione: `$i = "1"; $i++` dà int(2) alla prima iterazione,
# dalla seconda il confronto è Long vs Int = dominio fast (corsa 2: rotta = corretto). A verdetto.
RC=0
verdetto M1 "$ATT1" "$INT1" || RC=$?
for v in assign-form dst==src shl-63; do
  grep -qxF "$v" "$OUT/M1.rotte" && note "M1 a verdetto: $v ROTTA -> forma abbassata a BinarySCSCDst e fast path preso" || note "M1 a verdetto: $v INTATTA -> $( [ "$v" = shl-63 ] && echo 'atteso: 21−MIN overflow su Sub ⇒ corpo esatto per costruzione' || echo 'forma NON abbassata a BinarySCSCDst o dst aliasato (da leggere in reg_lower)')"
done
verdetto M2 "$ATT2" "$INT2" || RC=$?
grep -qxF lt-numstr-loop "$OUT/M2.rotte" && note "M2 a verdetto: lt-numstr-loop ROTTA -> tag misto per iterazione (\"1\"++ = int 2): fast path preso dalla 2ª iterazione, in dominio" || note "M2 a verdetto: lt-numstr-loop INTATTA -> il Long dopo \"1\"++ non ha preso il fast path (da leggere)"
[ "$RC" = 2 ] && RC=5

# --- epilogo: repo intatto al byte, mutante non conservato ---
if [ "$SOLO" = 1 ]; then
  note "SORGENTE: mutanti della corsa 2 nati dall'archivio del commit 5d87222b89cc (run.rs 95a662073ec13202 == HEAD di allora; il falso incidente della corsa 2 era il path di git show, emendato)"
else
  RS_H1=$(shasum -a 256 "$SRC/run.rs.orig" | cut -c1-16)
  [ "$RS_H0" = "$RS_H1" ] && note "SORGENTE: run.rs del mutante (prima dell'edit) $RS_H1 == commit ${SHA0:0:12} (il repo non è stato toccato: mutanti nati dall'archivio del commit)" || { note "SORGENTE: run.rs.orig $RS_H1 ≠ commit $RS_H0 — INCIDENTE"; RC=3; }
fi
rm -rf "$TGT" "$SRC"
AV=$(df -k /System/Volumes/Data | awk 'NR>1{printf "%.1f", $4/1048576}')
note "ESITO rc=$RC (0 = fast path preso su tutte le attese, dominio rispettato; 1 = attese non rotte; 5 = presa fuori dominio) — Data ${AV}G fine $(date '+%F %T')"
fin $RC
