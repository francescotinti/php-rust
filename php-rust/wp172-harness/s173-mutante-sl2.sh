#!/bin/bash
# s173-mutante-sl2.sh — az.rev. S-172 (wp172-harness/revisione.md rilievo 4, NEXT §S-173 p.2a):
# MUTANTE ABORTIVO dei due fast path della leva L-SL2 contro fx-sl2 (S-172, forme P1 in loop
# ≥2 = IC calda). Prova che il fast path è PRESO: un mutante che altera SOLO il ramo veloce deve
# rompere righe NOMINATE; le righe che devono cadere al corpo esatto restano byte-identiche al
# pin (se una di esse si rompe = fast path preso FUORI dominio: rc=5).
# COPIA DICHIARATA di wp171-harness/s172-mutante-sl1.sh (manifest s173-mutante-sl2-copia.diff);
# adattamenti: pin s172, lock TOKEN s173, fixture fx-sl2 (marcatore FX-SL2 DONE), due mutanti
# NUOVI (MP1/MP2), confronto per BLOCCO-etichetta (fx-sl2 stampa array multi-riga: il diff per
# riga di S-172 avrebbe dato etichette spurie), target di build sul disco ESTERNO (Data ~10G).
# Due mutanti SEPARATI (attribuzione per riga):
#   MP1 (P1, bigramma fuso sigillato, run.rs ~4762): `long_arith_i64(*b2, *y, *k)`
#       → `long_arith_i64(*b2, *y, *k).map(|v| v.wrapping_add(1))`  (Some(r)→Some(r+1))
#   MP2 (P2, BinarySTDst fast path, run.rs ~2102): `=> long_arith_i64(*b, *lv, *rv),`
#       → `=> long_arith_i64(*b, *lv, *rv).map(|v| v.wrapping_add(1)),`
# ATTESA PRE-registrata (blocchi ROTTI, per etichetta; letta dal sorgente delle guardie
# run.rs 4751-4802 e 2099-2116):
#   MP1 ⊇ {p1-loop100 p1-loop-overflow p1-add p1-sub p1-mul p1-and p1-or p1-xor p1-shl p1-shr
#          p1-shl-64 p1-shr-70 p1-shl-neg-l p1-shr-neg-l p1-shr-70-neg p1-double-dst p1-str-dst
#          p1-null-dst p1-arr-dst p1-dst-ref p1-dst-ref-alias p1-self p1-two-objs p1-inherited
#          p1-private-inscope prop-micro-1000}
#       (dst non-Long ⇒ write_property_at(Long(r)): il probe copre il DST di ogni tag; dst Ref
#        con typed_refs vuoto idem; il DOMINIO è sul prop LETTO e sul const)
#   MP2 ⊇ {p2-add p2-sub p2-mul p2-and p2-or p2-xor p2-shl p2-shr p2-shl-64 p2-shr-64
#          p2-shl-neg-l p2-shr-neg-l p2-loop100 p2-arr-src p2-call-src p2-loop-overflow
#          prop-micro-1000}
# A VERDETTO (non attese, da leggere): MP1 p1-shl-63 (7<<63: dominio dello shift a 63) ·
#   p1-typed-int / p1-typed-float-dst / p1-typed-float-from-long (IC set su prop TIPIZZATA:
#   riempita o no?) · p1-dynamic / p1-dynamic-self (stdClass: slot o dyn_entries) ·
#   p1-readonly-inscope#1 (rilievo 4: se ROTTA il probe scrive in place su readonly = BUG
#   rc=5) · p1-hook-set / p1-magic-set (attese intatte per guardia IC).
# DEVONO restare INTATTE (dominio non-Long / Ref sul letto / overflow / Div-Mod-Pow-Concat /
# errori): MP1 = {p1-div-inexact p1-div-exact p1-mod p1-pow p1-pow-overflow p1-concat
#   p1-div-zero#0 p1-div-zero#1 p1-mod-zero#0 p1-mod-zero#1 p1-mod-min-neg1 p1-add-overflow
#   p1-mul-overflow p1-sub-overflow p1-double-src p1-numstr-src p1-null-src p1-bool-src
#   p1-src-ref p1-typed-int-overflow#0 p1-typed-int-overflow#1 p1-typed-int-from-float
#   p1-readonly#0 p1-readonly#1 p1-readonly-inscope#0 p1-readonly-inscope#1
#   p1-private-outscope#0 p1-private-outscope#1 p1-hook-set p1-magic-set}
#   + TUTTI i blocchi p2-* (MP1 non tocca BinarySTDst).
#   MP2 = {p2-shl-neg p2-div p2-div-exact p2-mod p2-pow p2-concat p2-div-zero p2-add-overflow
#   p2-sub-overflow p2-mul-overflow p2-double-dst p2-numstr-dst p2-null-dst p2-bool-dst
#   p2-double-src p2-numstr-src p2-null-src p2-dst-ref p2-dst-ref-alias p2-typed-ref
#   p2-typed-ref-overflow p2-call-double-src} + TUTTI i blocchi p1-* (MP2 non tocca il bigramma).
# Il REPO NON viene toccato: i mutanti nascono da `git archive HEAD` in una COPIA dell'albero;
# «revert al byte» = hash di run.rs del commit == run.rs.orig. Diff dei mutanti in ab-out/.
# Ricetta build = quella del pin (SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build
# --release -p php-cli), target dedicato "$TGT" (riusato MP1→MP2), rimosso in epilogo.
# CORSA 1 (s173-mutante-verdetto-corsa1.out, rc=1): 4 attese non rotte, tutte LETTE dai file — p1-loop-overflow e
# p2-loop-overflow stampano il float SATURO finale (il +1 del mutante sparisce nell'overflow: fixture non
# discriminante ⇒ righe `-step` aggiunte, S-173); p2-and = coincidenza aritmetica della cascata (85&7+1 == 70&7
# = 6; p2-or subito dopo ROTTA ⇒ reset `$s = 70`); p1-private-inscope = DOMINIO: la IC set non si riempie su
# classi non-plain (private/typed: p1-typed-* INTATTI coerenti) ⇒ probe NON preso, corretto: spostata a VERDETTO
# (perimetro dichiarato: il probe sigillato copre SOLO classi plain_set_props — leva futura «typed»).
# CORSA 2 (SOLO=1, stessi binari): p1/p2-loop-overflow a VERDETTO (saturazione per costruzione), presidio = righe -step.
# Esiti: VERD (committato) + ab-out/s173-mut/*; rc SOLO da ab-out/s173-mut.done.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp172-harness"; OUT="$H/ab-out/s173-mut"; mkdir -p "$OUT"
VERD="$H/s173-mutante-verdetto.out"; DONE="$H/ab-out/s173-mut.done"; rm -f "$DONE"
LOCK=/private/tmp/phpr-measure.lock
PIN="$HOME/Claude/php-rust-output/release/phpr"; PIN_ATTESO=5f2dff7d17ebed79
FX="$H/fx-sl2.php"
SRC=/private/tmp/s173-mut; TGT="/Volumes/Extreme Pro/Claude/s173-mut-tgt"
RS=crates/php-runtime/src/vm/run.rs
: > "$VERD"
fin(){ echo "rc=$1 $(date +%T)" > "$DONE"; exit "$1"; }
note(){ echo "$*" >> "$VERD"; }

grep -qw s173 "$LOCK" 2>/dev/null || { note "rc=9 lock s173 assente (per TOKEN)"; fin 9; }
[ -s "$FX" ] || { note "rc=7 fixture assente"; fin 7; }
PH=$(shasum -a 256 "$PIN" | cut -c1-16)
[ "$PH" = "$PIN_ATTESO" ] || { note "rc=9 pin $PH ≠ atteso $PIN_ATTESO"; fin 9; }
cd "$REPO" || fin 4
SHA0=$(git rev-parse HEAD)
# la RADICE git è la cartella padre (php-rust-experiment/): path RELATIVO alla cwd (`:./`)
RS_H0=$(git show "$SHA0:./$RS" | shasum -a 256 | cut -c1-16)
SOLO="${SOLO:-0}"   # SOLO=1: riusa i binari ab-out/s173-mut/phpr-MP{1,2} (nessuna build)
AV=$(df -k /System/Volumes/Data | awk 'NR>1{printf "%.1f", $4/1048576}')
AVX=$(df -k "/Volumes/Extreme Pro" | awk 'NR>1{printf "%.0f", $4/1048576}')
note "== s173 mutante abortivo L-SL2 (P1/P2) — pin $PH (sorgente = commit ${SHA0:0:12}, run.rs $RS_H0), fixture $(wc -l < "$FX" | tr -d ' ') righe, Data ${AV}G, Extreme ${AVX}G (target di build) $(date '+%F %T') =="
awk -v a="$AVX" 'BEGIN{exit !(a+0 < 15)}' && { note "rc=8 Extreme ${AVX}G < 15G: niente build"; fin 8; }

# --- riferimento: pin su fixture (oracle==pin byte-id nel gate di promozione S-172) ---
perl -e 'alarm 60; exec @ARGV or die' -- "$PIN" "$FX" > "$OUT/pin.out" 2>&1
grep -q "FX-SL2 DONE" "$OUT/pin.out" || { note "rc=7 pin: marcatore assente"; fin 7; }

# --- copia albero ---
if [ "$SOLO" = 1 ]; then
  note "SOLO=1: binari della corsa precedente riusati (MP1 $(shasum -a 256 "$OUT/phpr-MP1" | cut -c1-16), MP2 $(shasum -a 256 "$OUT/phpr-MP2" | cut -c1-16)); nessuna build"
else
rm -rf "$SRC"; mkdir -p "$SRC/php-rust"
git archive "$SHA0" crates Cargo.toml Cargo.lock rust-toolchain.toml .cargo 2>/dev/null | tar -x -C "$SRC/php-rust" \
  || { note "rc=7 archivio del commit fallito"; fin 7; }
[ -s "$SRC/php-rust/$RS" ] || { note "rc=7 archivio senza run.rs"; fin 7; }
cp "$SRC/php-rust/$RS" "$SRC/run.rs.orig"
fi

# blocchi per ETICHETTA: ogni riga `label: …` apre un blocco che dura fino alla prossima etichetta
# (gli array di var_dump stanno su più righe); ROTTO = blocco diverso o assente nel mutante.
rotte(){ # $1=pin.out $2=mut.out → stdout etichette rotte
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

build_run(){ # $1=nome $2=needle-perl $3=repl-perl
  local n="$1" needle="$2" repl="$3"
  if [ "$SOLO" = 1 ]; then
    [ -x "$OUT/phpr-$n" ] || { note "rc=7 SOLO: binario $n assente"; fin 7; }
    perl -e 'alarm 60; exec @ARGV or die' -- "$OUT/phpr-$n" "$FX" > "$OUT/$n.out" 2>&1
    note "$n: binario $(shasum -a 256 "$OUT/phpr-$n" | cut -c1-16) (riusato) rc_fixture=$?"
    rotte "$OUT/pin.out" "$OUT/$n.out" | sort -u > "$OUT/$n.rotte"
    note "$n: blocchi ROTTI ($(wc -l < "$OUT/$n.rotte" | tr -d ' ')): $(tr '\n' ' ' < "$OUT/$n.rotte")"
    return 0
  fi
  cp "$SRC/run.rs.orig" "$SRC/php-rust/$RS"
  perl -0pi -e "s/\Q$needle\E/$repl/" "$SRC/php-rust/$RS"
  local nrep; nrep=$(diff "$SRC/run.rs.orig" "$SRC/php-rust/$RS" | grep -c '^>')
  [ "$nrep" = 1 ] || { note "rc=6 $n: il mutante ha toccato $nrep righe (attesa 1)"; fin 6; }
  diff -u "$SRC/run.rs.orig" "$SRC/php-rust/$RS" > "$OUT/$n.diff"
  ( cd "$SRC/php-rust" && SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TGT" \
      cargo build --release -p php-cli ) > "$OUT/build-$n.log" 2>&1 \
    || { note "rc=4 $n: build FALLITA (ab-out/s173-mut/build-$n.log)"; fin 4; }
  local mh; mh=$(shasum -a 256 "$TGT/release/phpr" | cut -c1-16)
  [ "$mh" != "$PH" ] || { note "rc=6 $n: binario == pin (mutante NON entrato)"; fin 6; }
  cp "$TGT/release/phpr" "$OUT/phpr-$n"
  perl -e 'alarm 60; exec @ARGV or die' -- "$OUT/phpr-$n" "$FX" > "$OUT/$n.out" 2>&1
  local rrc=$?
  note "$n: binario $mh rc_fixture=$rrc"
  rotte "$OUT/pin.out" "$OUT/$n.out" | sort -u > "$OUT/$n.rotte"
  note "$n: blocchi ROTTI ($(wc -l < "$OUT/$n.rotte" | tr -d ' ')): $(tr '\n' ' ' < "$OUT/$n.rotte")"
}

verdetto(){ # $1=nome $2=attese(spazio) $3=intatte(spazio)
  local n="$1" bad=0 miss=""; local a
  for a in $2; do grep -qxF "$a" "$OUT/$n.rotte" || miss="$miss $a"; done
  local viol=""
  for a in $3; do grep -qxF "$a" "$OUT/$n.rotte" && viol="$viol $a"; done
  [ -z "$miss" ] && note "$n: attese TUTTE rotte -> fast path PRESO su ogni blocco atteso" || { note "$n: attese NON rotte:$miss -> fast path NON preso (o forma non abbassata) su questi"; bad=1; }
  [ -z "$viol" ] && note "$n: blocchi fuori-dominio INTATTI (nessuna presa fuori dominio)" || { note "$n: VIOLAZIONE dominio — rotti:$viol (fast path preso FUORI dominio)"; bad=2; }
  return $bad
}

build_run MP1 'long_arith_i64(*b2, *y, *k)' 'long_arith_i64(*b2, *y, *k).map(|v| v.wrapping_add(1))'
build_run MP2 '=> long_arith_i64(*b, *lv, *rv),' '=> long_arith_i64(*b, *lv, *rv).map(|v| v.wrapping_add(1)),'

P1ALL="p1-loop100 p1-loop-overflow p1-loop-overflow-step p1-add p1-sub p1-mul p1-and p1-or p1-xor p1-shl p1-shr p1-shl-64 p1-shr-70 p1-shl-63 p1-shl-neg-l p1-shr-neg-l p1-shr-70-neg p1-div-inexact p1-div-exact p1-mod p1-pow p1-pow-overflow p1-concat p1-div-zero#0 p1-div-zero#1 p1-mod-zero#0 p1-mod-zero#1 p1-mod-min-neg1 p1-add-overflow p1-mul-overflow p1-sub-overflow p1-double-src p1-numstr-src p1-null-src p1-bool-src p1-double-dst p1-str-dst p1-null-dst p1-arr-dst p1-dst-ref p1-dst-ref-alias p1-src-ref p1-self p1-two-objs p1-inherited p1-typed-int p1-typed-int-overflow#0 p1-typed-int-overflow#1 p1-typed-float-dst p1-typed-int-from-float p1-typed-float-from-long p1-readonly#0 p1-readonly#1 p1-readonly-inscope#0 p1-readonly-inscope#1 p1-private-inscope p1-private-outscope#0 p1-private-outscope#1 p1-hook-set p1-magic-set p1-dynamic p1-dynamic-self"
P2ALL="p2-loop-overflow-step p2-add p2-sub p2-mul p2-and p2-or p2-xor p2-shl p2-shr p2-shl-64 p2-shr-64 p2-shl-neg-l p2-shr-neg-l p2-shl-neg p2-div p2-div-exact p2-mod p2-pow p2-concat p2-div-zero p2-add-overflow p2-sub-overflow p2-mul-overflow p2-double-dst p2-numstr-dst p2-null-dst p2-bool-dst p2-double-src p2-numstr-src p2-null-src p2-dst-ref p2-dst-ref-alias p2-typed-ref p2-typed-ref-overflow p2-loop100 p2-arr-src p2-call-src p2-call-double-src p2-loop-overflow"
ATT1="p1-loop100 p1-loop-overflow-step p1-add p1-sub p1-mul p1-and p1-or p1-xor p1-shl p1-shr p1-shl-64 p1-shr-70 p1-shl-neg-l p1-shr-neg-l p1-shr-70-neg p1-double-dst p1-str-dst p1-null-dst p1-arr-dst p1-dst-ref p1-dst-ref-alias p1-self p1-two-objs p1-inherited prop-micro-1000"
INT1="p1-div-inexact p1-div-exact p1-mod p1-pow p1-pow-overflow p1-concat p1-div-zero#0 p1-div-zero#1 p1-mod-zero#0 p1-mod-zero#1 p1-mod-min-neg1 p1-add-overflow p1-mul-overflow p1-sub-overflow p1-double-src p1-numstr-src p1-null-src p1-bool-src p1-src-ref p1-typed-int-overflow#0 p1-typed-int-overflow#1 p1-typed-int-from-float p1-readonly#0 p1-readonly#1 p1-readonly-inscope#0 p1-readonly-inscope#1 p1-private-outscope#0 p1-private-outscope#1 p1-hook-set p1-magic-set $P2ALL"
VER1="p1-loop-overflow p1-private-inscope p1-shl-63 p1-typed-int p1-typed-float-dst p1-typed-float-from-long p1-dynamic p1-dynamic-self"
ATT2="p2-loop-overflow-step p2-add p2-sub p2-mul p2-and p2-or p2-xor p2-shl p2-shr p2-shl-64 p2-shr-64 p2-shl-neg-l p2-shr-neg-l p2-loop100 p2-arr-src p2-call-src prop-micro-1000"
VER2="p2-loop-overflow"
INT2="p2-shl-neg p2-div p2-div-exact p2-mod p2-pow p2-concat p2-div-zero p2-add-overflow p2-sub-overflow p2-mul-overflow p2-double-dst p2-numstr-dst p2-null-dst p2-bool-dst p2-double-src p2-numstr-src p2-null-src p2-dst-ref p2-dst-ref-alias p2-typed-ref p2-typed-ref-overflow p2-call-double-src $P1ALL"
# copertura: ogni etichetta del pin deve stare in ATT/INT/VER del proprio mutante (fixture ≠ copione ⇒ rc=7)
python3 - "$OUT/pin.out" "$ATT1 $INT1 $VER1" "$ATT2 $INT2 $VER2" <<'PY' >> "$VERD" || fin 7
import sys, re
labels = []
for line in open(sys.argv[1], encoding='utf-8', errors='replace'):
    m = re.match(r"^([A-Za-z0-9_().'#=-]+): ", line)
    if m and m.group(1) not in labels: labels.append(m.group(1))
rc = 0
for nome, lst in (("MP1", sys.argv[2]), ("MP2", sys.argv[3])):
    s = set(lst.split()); miss = [l for l in labels if l not in s]
    print(f"copertura {nome}: {len(labels)} etichette nel pin, non classificate: {' '.join(miss) or 'nessuna'}")
    if miss: rc = 7
sys.exit(rc)
PY
RC=0
verdetto MP1 "$ATT1" "$INT1" || RC=$?
for v in $VER1; do
  [ "$v" = p1-loop-overflow ] && { grep -qxF "$v" "$OUT/MP1.rotte" && note "MP1 a verdetto: $v ROTTO (inatteso: il float finale satura)" || note "MP1 a verdetto: $v INTATTO -> atteso per costruzione: il float finale SATURA (non discriminante; presidio = p1-loop-overflow-step)"; continue; }
  grep -qxF "$v" "$OUT/MP1.rotte" && note "MP1 a verdetto: $v ROTTO -> probe sigillato PRESO (IC set riempita su questa forma)" || note "MP1 a verdetto: $v INTATTO -> probe NON preso (IC set non riempita, slot assente o fuori dominio)"
done
verdetto MP2 "$ATT2" "$INT2" || RC=$?
for v in $VER2; do grep -qxF "$v" "$OUT/MP2.rotte" && note "MP2 a verdetto: $v ROTTO (inatteso: il float finale satura)" || note "MP2 a verdetto: $v INTATTO -> atteso per costruzione: il float finale SATURA (non discriminante; presidio = p2-loop-overflow-step)"; done
[ "$RC" = 2 ] && RC=5

# --- epilogo: repo intatto al byte, mutante non conservato ---
if [ "$SOLO" != 1 ]; then
  RS_H1=$(shasum -a 256 "$SRC/run.rs.orig" | cut -c1-16)
  [ "$RS_H0" = "$RS_H1" ] && note "SORGENTE: run.rs del mutante (prima dell'edit) $RS_H1 == commit ${SHA0:0:12} (repo non toccato: mutanti nati dall'archivio del commit)" || { note "SORGENTE: run.rs.orig $RS_H1 ≠ commit $RS_H0 — INCIDENTE"; RC=3; }
fi
rm -rf "$TGT" "$SRC"
AV=$(df -k /System/Volumes/Data | awk 'NR>1{printf "%.1f", $4/1048576}')
note "ESITO rc=$RC (0 = fast path preso su tutte le attese, dominio rispettato; 1 = attese non rotte; 5 = presa fuori dominio; 7 = copertura) — Data ${AV}G fine $(date '+%F %T')"
fin $RC
