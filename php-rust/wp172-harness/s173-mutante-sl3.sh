#!/bin/bash
# s173-mutante-sl3.sh — fetta 3 (criterio s173-criterio.md p.6): MUTANTE ABORTIVO dei due fast
# path P3/P4 contro fx-sl3 (forme in loop a 2 = IC calda; la 1ª iterazione è SEMPRE storica: il
# probe si vede solo dalla 2ª). Prova che il fast path è PRESO su righe NOMINATE e MAI fuori dominio.
# COPIA DICHIARATA di s173-mutante-sl2.sh (manifest s173-mutante-sl3-copia.diff); adattamenti:
# binario di riferimento = braccio CANDIDATO (ab-out/phpr-C, P3+P4: il pin s172 non ha i fast path),
# fixture fx-sl3 (marcatore FX-SL3 DONE), mutanti MP3/MP4 dal COMMIT del braccio C, target esterno.
#   MP3 (P3, peephole PropGetSlot+BinarySTDst): `long_arith_i64(*b2, *lv, *y)`
#       → `long_arith_i64(*b2, *lv, *y).map(|v| v.wrapping_add(1))`
#   MP4 (P4, borrow unico recv==slot): `if let Some(r) = long_arith_i64(*b2, *y, *k) {`
#       → `if let Some(r) = long_arith_i64(*b2, *y, *k).map(|v| v.wrapping_add(1)) {`
# ATTESA PRE-registrata (blocchi ROTTI):
#   MP3 ⊇ {p3-add p3-sub p3-mul p3-and p3-or p3-xor p3-shl p3-shr p3-shl-64 p3-shr-64 p3-shl-neg-l
#          p3-shr-neg-l p3-loop100 p3-loop-overflow-step p3-loop-overflow p3-twice prop-micro-1000}
#   MP4 ⊇ {p4-same p4-same-mul p4-same-shr-neg p4-self p4-self-y p4-other-class prop-micro-1000}
# A VERDETTO: MP3 p3-typed-src (IC get su prop tipizzata) · p3-magic-class-plain-src (prop dichiarata
#   su classe con __get) · p3-dynamic-src (stdClass) · MP4 p4-this-private (forma ThisProp*, attesa
#   NON abbassata al bigramma) · p4-dynamic (stdClass) · p4-self-rhs (rhs non-const: attesa intatta).
# DEVONO restare INTATTE: MP3 = {p3-shl-neg#0 p3-shl-neg#1 p3-div p3-div-exact p3-mod p3-pow p3-concat
#   p3-div-zero#0 p3-div-zero#1 p3-add-overflow p3-sub-overflow p3-mul-overflow p3-double-dst
#   p3-numstr-dst p3-null-dst p3-bool-dst p3-double-src p3-numstr-src p3-null-src p3-dst-ref
#   p3-dst-ref-alias p3-src-ref p3-typed-ref p3-typed-ref-overflow#0 p3-typed-ref-overflow#1
#   p3-magic-get-src p3-hook-get-src p3-enum-src p3-no-peephole p3-rhs-expr p3-dst-is-obj#0
#   p3-dst-is-obj#1} + TUTTI i blocchi p4-*;
#   MP4 = {p4-overflow p4-dst-double p4-dst-str p4-dst-ref p4-dst-ref-alias p4-src-ref p4-src-double
#   p4-two-objs p4-typed p4-typed-float-dst p4-readonly#0 p4-readonly#1 p4-enum#0
#   p4-enum#1} + TUTTI i blocchi p3-*.
# Esiti: VERD (committato) + ab-out/s173-mut3/*; rc SOLO da ab-out/s173-mut3.done.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp172-harness"; OUT="$H/ab-out/s173-mut3"; mkdir -p "$OUT"
VERD="$H/s173-mutante3-verdetto.out"; DONE="$H/ab-out/s173-mut3.done"; rm -f "$DONE"
LOCK=/private/tmp/phpr-measure.lock
REF="${REF:-$H/ab-out/phpr-C}"; REF_ATTESO="${REF_ATTESO:?hash16 atteso del braccio C}"
COMMIT="${COMMIT:?commit sorgente del braccio C}"
FX="$H/fx-sl3.php"
SRC="/Volumes/Extreme Pro/Claude/s173-mut3"; TGT="/Volumes/Extreme Pro/Claude/s173-mut3-tgt"
RS=crates/php-runtime/src/vm/run.rs
: > "$VERD"
fin(){ echo "rc=$1 $(date +%T)" > "$DONE"; exit "$1"; }
note(){ echo "$*" >> "$VERD"; }

grep -qw s173 "$LOCK" 2>/dev/null || { note "rc=9 lock s173 assente (per TOKEN)"; fin 9; }
[ -s "$FX" ] || { note "rc=7 fixture assente"; fin 7; }
PH=$(shasum -a 256 "$REF" | cut -c1-16)
[ "$PH" = "$REF_ATTESO" ] || { note "rc=9 riferimento $PH ≠ atteso $REF_ATTESO"; fin 9; }
cd "$REPO" || fin 4
SHA0=$(git rev-parse --verify "$COMMIT^{commit}") || { note "rc=7 commit $COMMIT inesistente"; fin 7; }
RS_H0=$(git show "$SHA0:./$RS" | shasum -a 256 | cut -c1-16)
SOLO="${SOLO:-0}"
AVX=$(df -k "/Volumes/Extreme Pro" | awk 'NR>1{printf "%.0f", $4/1048576}')
note "== s173 mutante abortivo fetta 3 (P3/P4) — riferimento braccio C $PH (sorgente = commit ${SHA0:0:12}, run.rs $RS_H0), fixture $(wc -l < "$FX" | tr -d ' ') righe, Extreme ${AVX}G $(date '+%F %T') =="
awk -v a="$AVX" 'BEGIN{exit !(a+0 < 15)}' && { note "rc=8 Extreme ${AVX}G < 15G: niente build"; fin 8; }

perl -e 'alarm 60; exec @ARGV or die' -- "$REF" "$FX" > "$OUT/ref.out" 2>&1
grep -q "FX-SL3 DONE" "$OUT/ref.out" || { note "rc=7 riferimento: marcatore assente"; fin 7; }

if [ "$SOLO" = 1 ]; then
  note "SOLO=1: binari della corsa precedente riusati (MP3 $(shasum -a 256 "$OUT/phpr-MP3" | cut -c1-16), MP4 $(shasum -a 256 "$OUT/phpr-MP4" | cut -c1-16)); nessuna build"
else
rm -rf "$SRC"; mkdir -p "$SRC/php-rust"
git archive "$SHA0" crates Cargo.toml Cargo.lock rust-toolchain.toml .cargo 2>/dev/null | tar -x -C "$SRC/php-rust" \
  || { note "rc=7 archivio del commit fallito"; fin 7; }
[ -s "$SRC/php-rust/$RS" ] || { note "rc=7 archivio senza run.rs"; fin 7; }
/usr/bin/find "$SRC" -type f -exec touch {} +
cp "$SRC/php-rust/$RS" "$SRC/run.rs.orig"
fi

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

build_run(){ # $1=nome $2=needle-perl $3=repl-perl
  local n="$1" needle="$2" repl="$3"
  if [ "$SOLO" = 1 ]; then
    [ -x "$OUT/phpr-$n" ] || { note "rc=7 SOLO: binario $n assente"; fin 7; }
  else
    cp "$SRC/run.rs.orig" "$SRC/php-rust/$RS"
    perl -0pi -e "s/\Q$needle\E/$repl/" "$SRC/php-rust/$RS"
    local nrep; nrep=$(diff "$SRC/run.rs.orig" "$SRC/php-rust/$RS" | grep -c '^>')
    [ "$nrep" = 1 ] || { note "rc=6 $n: il mutante ha toccato $nrep righe (attesa 1)"; fin 6; }
    diff -u "$SRC/run.rs.orig" "$SRC/php-rust/$RS" > "$OUT/$n.diff"
    /usr/bin/touch "$SRC/php-rust/$RS"
    ( cd "$SRC/php-rust" && SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TGT" \
        cargo build --release -p php-cli ) > "$OUT/build-$n.log" 2>&1 \
      || { note "rc=4 $n: build FALLITA (ab-out/s173-mut3/build-$n.log)"; fin 4; }
    local mh; mh=$(shasum -a 256 "$TGT/release/phpr" | cut -c1-16)
    [ "$mh" != "$PH" ] || { note "rc=6 $n: binario == riferimento (mutante NON entrato)"; fin 6; }
    cp "$TGT/release/phpr" "$OUT/phpr-$n"
  fi
  perl -e 'alarm 60; exec @ARGV or die' -- "$OUT/phpr-$n" "$FX" > "$OUT/$n.out" 2>&1
  local rrc=$?
  note "$n: binario $(shasum -a 256 "$OUT/phpr-$n" | cut -c1-16) rc_fixture=$rrc"
  rotte "$OUT/ref.out" "$OUT/$n.out" | sort -u > "$OUT/$n.rotte"
  note "$n: blocchi ROTTI ($(wc -l < "$OUT/$n.rotte" | tr -d ' ')): $(tr '\n' ' ' < "$OUT/$n.rotte")"
}

verdetto(){ # $1=nome $2=attese $3=intatte
  local n="$1" bad=0 miss=""; local a
  for a in $2; do grep -qxF "$a" "$OUT/$n.rotte" || miss="$miss $a"; done
  local viol=""
  for a in $3; do grep -qxF "$a" "$OUT/$n.rotte" && viol="$viol $a"; done
  [ -z "$miss" ] && note "$n: attese TUTTE rotte -> fast path PRESO su ogni blocco atteso" || { note "$n: attese NON rotte:$miss -> fast path NON preso (o forma non abbassata) su questi"; bad=1; }
  [ -z "$viol" ] && note "$n: blocchi fuori-dominio INTATTI (nessuna presa fuori dominio)" || { note "$n: VIOLAZIONE dominio — rotti:$viol (fast path preso FUORI dominio)"; bad=2; }
  return $bad
}

build_run MP3 'long_arith_i64(*b2, *lv, *y)' 'long_arith_i64(*b2, *lv, *y).map(|v| v.wrapping_add(1))'
build_run MP4 'if let Some(r) = long_arith_i64(*b2, *y, *k) {' 'if let Some(r) = long_arith_i64(*b2, *y, *k).map(|v| v.wrapping_add(1)) {'

P3ALL="p3-add p3-sub p3-mul p3-and p3-or p3-xor p3-shl p3-shr p3-shl-64 p3-shr-64 p3-shl-neg-l p3-shr-neg-l p3-shl-neg#0 p3-shl-neg#1 p3-div p3-div-exact p3-mod p3-pow p3-concat p3-div-zero#0 p3-div-zero#1 p3-add-overflow p3-sub-overflow p3-mul-overflow p3-double-dst p3-numstr-dst p3-null-dst p3-bool-dst p3-double-src p3-numstr-src p3-null-src p3-dst-ref p3-dst-ref-alias p3-src-ref p3-typed-ref p3-typed-ref-overflow#0 p3-typed-ref-overflow#1 p3-typed-src p3-magic-get-src p3-magic-class-plain-src p3-hook-get-src p3-dynamic-src p3-enum-src p3-loop100 p3-loop-overflow-step p3-loop-overflow p3-no-peephole p3-rhs-expr p3-twice p3-dst-is-obj#0 p3-dst-is-obj#1"
P4ALL="p4-same p4-same-mul p4-same-shr-neg p4-self-rhs p4-self p4-self-y p4-overflow p4-dst-double p4-dst-str p4-dst-ref p4-dst-ref-alias p4-src-ref p4-src-double p4-two-objs p4-other-class p4-typed p4-typed-float-dst p4-readonly#0 p4-readonly#1 p4-this-private p4-dynamic p4-enum#0 p4-enum#1"
ATT3="p3-add p3-sub p3-mul p3-and p3-or p3-xor p3-shl p3-shr p3-shl-64 p3-shr-64 p3-shl-neg-l p3-shr-neg-l p3-loop100 p3-loop-overflow-step p3-loop-overflow p3-twice prop-micro-1000"
INT3="p3-shl-neg#0 p3-shl-neg#1 p3-div p3-div-exact p3-mod p3-pow p3-concat p3-div-zero#0 p3-div-zero#1 p3-add-overflow p3-sub-overflow p3-mul-overflow p3-double-dst p3-numstr-dst p3-null-dst p3-bool-dst p3-double-src p3-numstr-src p3-null-src p3-dst-ref p3-dst-ref-alias p3-src-ref p3-typed-ref p3-typed-ref-overflow#0 p3-typed-ref-overflow#1 p3-magic-get-src p3-hook-get-src p3-enum-src p3-no-peephole p3-rhs-expr p3-dst-is-obj#0 p3-dst-is-obj#1 $P4ALL"
VER3="p3-typed-src p3-magic-class-plain-src p3-dynamic-src"
ATT4="p4-same p4-same-mul p4-same-shr-neg p4-self p4-self-y p4-other-class prop-micro-1000"
INT4="p4-overflow p4-dst-double p4-dst-str p4-dst-ref p4-dst-ref-alias p4-src-ref p4-src-double p4-two-objs p4-typed p4-typed-float-dst p4-readonly#0 p4-readonly#1 p4-enum#0 p4-enum#1 $P3ALL"
VER4="p4-this-private p4-dynamic p4-self-rhs"
python3 - "$OUT/ref.out" "$ATT3 $INT3 $VER3" "$ATT4 $INT4 $VER4" <<'PY' >> "$VERD" || fin 7
import sys, re
labels = []
for line in open(sys.argv[1], encoding='utf-8', errors='replace'):
    m = re.match(r"^([A-Za-z0-9_().'#=-]+): ", line)
    if m and m.group(1) not in labels: labels.append(m.group(1))
rc = 0
for nome, lst in (("MP3", sys.argv[2]), ("MP4", sys.argv[3])):
    s = set(lst.split()); miss = [l for l in labels if l not in s]
    print(f"copertura {nome}: {len(labels)} etichette nel riferimento, non classificate: {' '.join(miss) or 'nessuna'}")
    if miss: rc = 7
sys.exit(rc)
PY
RC=0
verdetto MP3 "$ATT3" "$INT3" || RC=$?
for v in $VER3; do grep -qxF "$v" "$OUT/MP3.rotte" && note "MP3 a verdetto: $v ROTTO -> peephole PRESO (IC get riempita su questa forma)" || note "MP3 a verdetto: $v INTATTO -> peephole NON preso (IC get non riempita o forma diversa)"; done
verdetto MP4 "$ATT4" "$INT4" || RC=$?
for v in $VER4; do grep -qxF "$v" "$OUT/MP4.rotte" && note "MP4 a verdetto: $v ROTTO -> borrow unico PRESO" || note "MP4 a verdetto: $v INTATTO -> borrow unico NON preso (forma non abbassata al bigramma o IC non riempita)"; done
[ "$RC" = 2 ] && RC=5
if [ "$SOLO" != 1 ]; then
  RS_H1=$(shasum -a 256 "$SRC/run.rs.orig" | cut -c1-16)
  [ "$RS_H0" = "$RS_H1" ] && note "SORGENTE: run.rs del mutante (prima dell'edit) $RS_H1 == commit ${SHA0:0:12}" || { note "SORGENTE: run.rs.orig $RS_H1 ≠ commit $RS_H0 — INCIDENTE"; RC=3; }
fi
rm -rf "$TGT" "$SRC"
note "ESITO rc=$RC (0 = fast path preso su tutte le attese, dominio rispettato; 1 = attese non rotte; 5 = presa fuori dominio; 7 = copertura) fine $(date '+%F %T')"
fin $RC
