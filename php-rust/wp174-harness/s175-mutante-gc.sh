#!/bin/bash
# s175-mutante-gc.sh — mutante abortivo della gamba «pressione GC» del predicato sweep_idle (az.rev. S-174 (a),
# revisione S-174 azione 3). COPIA DICHIARATA di s174-mutante-sw.sh (manifest s175-mutante-gc-copia.diff);
# DIVERGENZE DICHIARATE: (1) fixture fx-sw2-gc.php (marcatore FX-SW2 DONE), cicli a OGGETTI (`$o->self = $o`:
# un ciclo via `$a[] = &$a` NON lascia radici, sondato S-175), gc_enable() INLINE nel rhs dell'op fuso;
# (2) giudice di INVARIANZA pin/candidato == stash s173 byte-id (NON bilaterale: l'oracle raccoglie lazy —
# all'inserimento di una radice a buffer pieno — phpr allo Sweep di fine statement: divergenza a catalogo,
# ho_gc_status) + oracle == riferimento a meno dei soli conteggi «d=»/«gcp-loop:» (normalizzati);
# (3) mutanti: MS = predicato sempre vero nel helper (needle INVARIATO da S-174) e M3 = 3ª clausola di sweep_idle
# (`!gc_enabled || radici < gc_sweep_bound`) forzata VERA (`&& true)`): tocca il testo UNICO del predicato, letto
# dal handler Op::Sweep E dai siti fusi ⇒ atteso ROTTO ovunque la 3ª clausola decide (fused, ctl, loop, scsc);
# MC non pertinente (nessun back-edge in gioco: omesso); (4) tag s175 (lock per TOKEN s175, out ab-out/s175-mutgc,
# verdetto s175-mutante-gc-verdetto.out, target/sorgente s175-mutgc*).
# ATTESE: MS ROTTE = {gcp-fused gcp-loop} (Sweep scavalcato dopo l'op fuso: la raccolta slitta allo statement
#   dopo la lettura); MS INTATTE per NOME (discriminanti: si romperebbero SOLO se `$s = $s + …` / `$z = …` fossero
#   fusi) = {gcp-ctl gcp-scsc}; a verdetto (NON discriminanti: nessuna raccolta in nessun binario) = {gcp-under gcp-off}.
#   M3 ROTTE = {gcp-fused gcp-ctl gcp-loop gcp-scsc}; M3 intatte attese: nessuna; a verdetto = {gcp-under gcp-off}.
# REGOLA (S-174, azione 4): un blocco NON discriminante va «a verdetto», mai «intatto atteso».
# Esiti: VERD (committato) + ab-out/s175-mutgc/*; rc SOLO da ab-out/s175-mutgc.done.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp174-harness"; OUT="$H/ab-out/s175-mutgc"; mkdir -p "$OUT"
VERD="$H/s175-mutante-gc-verdetto.out"; DONE="$H/ab-out/s175-mutgc.done"; rm -f "$DONE"
LOCK=/private/tmp/phpr-measure.lock
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
REF="${REF:-$STASH/phpr-s174-sw-C}"; REF_ATTESO="${REF_ATTESO:-b6c4b5876971d76d}"
S173="$STASH/phpr-s173"; S173_ATTESO="da4921a52eba0187"
ORACLE=/opt/homebrew/opt/php/bin/php
COMMIT="${COMMIT:-883cb598}"
FX="$H/fixtures/fx-sw2-gc.php"
SRC="/Volumes/Extreme Pro/Claude/s175-mutgc"; TGT="/Volumes/Extreme Pro/Claude/s175-mutgc-tgt"
RS=crates/php-runtime/src/vm/run.rs
: > "$VERD"
fin(){ echo "rc=$1 $(date +%T)" > "$DONE"; exit "$1"; }
note(){ echo "$*" >> "$VERD"; }

grep -qw s175 "$LOCK" 2>/dev/null || { note "rc=9 lock s175 assente (per TOKEN)"; fin 9; }
[ -s "$FX" ] || { note "rc=7 fixture assente"; fin 7; }
PH=$(shasum -a 256 "$REF" | cut -c1-16)
[ "$PH" = "$REF_ATTESO" ] || { note "rc=9 riferimento $PH ≠ atteso $REF_ATTESO"; fin 9; }
[ "$(shasum -a 256 "$S173" | cut -c1-16)" = "$S173_ATTESO" ] || { note "rc=9 stash s173 ≠ $S173_ATTESO"; fin 9; }
cd "$REPO" || fin 4
SHA0=$(git rev-parse --verify "$COMMIT^{commit}") || { note "rc=7 commit $COMMIT inesistente"; fin 7; }
RS_H0=$(git show "$SHA0:./$RS" | shasum -a 256 | cut -c1-16)
SOLO="${SOLO:-0}"
AVX=$(df -k "/Volumes/Extreme Pro" | awk 'NR>1{printf "%.0f", $4/1048576}')
note "== s175 mutante abortivo «pressione GC» (MS/M3) — riferimento braccio C $PH (sorgente = commit ${SHA0:0:12}, run.rs $RS_H0), stash s173 $S173_ATTESO, fixture $(wc -l < "$FX" | tr -d ' ') righe, Extreme ${AVX}G $(date '+%F %T') =="
awk -v a="$AVX" 'BEGIN{exit !(a+0 < 15)}' && { note "rc=8 Extreme ${AVX}G < 15G: niente build"; fin 8; }

perl -e 'alarm 120; exec @ARGV or die' -- "$REF" "$FX" > "$OUT/ref.out" 2>&1
grep -q "FX-SW2 DONE" "$OUT/ref.out" || { note "rc=7 riferimento: marcatore assente"; fin 7; }
perl -e 'alarm 120; exec @ARGV or die' -- "$S173" "$FX" > "$OUT/s173.out" 2>&1
if diff -q "$OUT/ref.out" "$OUT/s173.out" > /dev/null; then
  note "INVARIANZA: candidato C == stash s173 BYTE-ID ($(tr '\n' ' ' < "$OUT/ref.out" | cut -c1-160))"
else
  diff "$OUT/ref.out" "$OUT/s173.out" > "$OUT/invarianza.diff" || true
  note "rc=2 INVARIANZA: candidato C ≠ stash s173 (ab-out/s175-mutgc/invarianza.diff)"; fin 2
fi
"$ORACLE" -d log_errors=0 -d display_errors=1 "$FX" > "$OUT/oracle.out" 2>&1
norm(){ sed -E 's/d=-?[0-9]+/d=_/; s/^gcp-loop: [-0-9,]+/gcp-loop: _/' "$1"; }
if diff -q <(norm "$OUT/ref.out") <(norm "$OUT/oracle.out") > /dev/null; then
  note "ORACLE (dichiarato): == riferimento a meno dei conteggi di raccolta; oracle grezzo: $(tr '\n' ' ' < "$OUT/oracle.out" | cut -c1-160) — divergenza di MOMENTO della raccolta (lazy vs Sweep di fine statement), a catalogo"
else
  diff "$OUT/ref.out" "$OUT/oracle.out" > "$OUT/oracle.diff" || true
  note "rc=2 ORACLE: diverge OLTRE i conteggi di raccolta (ab-out/s175-mutgc/oracle.diff)"; fin 2
fi

if [ "$SOLO" = 1 ]; then
  note "SOLO=1: binari della corsa precedente riusati (MS $(shasum -a 256 "$OUT/phpr-MS" | cut -c1-16), M3 $(shasum -a 256 "$OUT/phpr-M3" | cut -c1-16)); nessuna build"
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
    [ "$nrep" = 1 ] || { note "rc=6 $n: il mutante ha scritto $nrep righe (attesa 1)"; fin 6; }
    diff -u "$SRC/run.rs.orig" "$SRC/php-rust/$RS" > "$OUT/$n.diff"
    /usr/bin/touch "$SRC/php-rust/$RS"
    ( cd "$SRC/php-rust" && SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TGT" \
        cargo build --release -p php-cli ) > "$OUT/build-$n.log" 2>&1 \
      || { note "rc=4 $n: build FALLITA (ab-out/s175-mutgc/build-$n.log)"; fin 4; }
    local mh; mh=$(shasum -a 256 "$TGT/release/phpr" | cut -c1-16)
    [ "$mh" != "$PH" ] || { note "rc=6 $n: binario == riferimento (mutante NON entrato)"; fin 6; }
    cp "$TGT/release/phpr" "$OUT/phpr-$n"
  fi
  perl -e 'alarm 120; exec @ARGV or die' -- "$OUT/phpr-$n" "$FX" > "$OUT/$n.out" 2>&1
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
  [ -z "$miss" ] && note "$n: attese TUTTE rotte -> il predicato mutato decide su ogni blocco atteso" || { note "$n: attese NON rotte:$miss -> il predicato mutato NON decide su questi"; bad=1; }
  [ -z "$viol" ] && note "$n: blocchi intatti per NOME (discriminanti, nessuna presa fuori dominio): $(for a in $3; do grep -qxF "$a" "$OUT/$n.rotte" || printf '%s ' "$a"; done)" || { note "$n: VIOLAZIONE dominio — rotti:$viol"; bad=2; }
  return $bad
}

N3=$'&& (!self.gc_enabled\n                    || self.gc_cycle_roots.len() + self.gc_ctr_roots.len() < self.gc_sweep_bound))'
build_run MS 'matches!(func.ops.get(at), Some(Op::Sweep { main }) if self.sweep_idle(top, *main))' 'matches!(func.ops.get(at), Some(Op::Sweep { .. }))'
build_run M3 "$N3" '&& true)'

ATTS="gcp-fused gcp-loop"; INTS="gcp-ctl gcp-scsc"; VERS="gcp-under gcp-off"
ATT3="gcp-fused gcp-ctl gcp-loop gcp-scsc"; INT3=""; VER3="gcp-under gcp-off"
python3 - "$OUT/ref.out" "$ATTS $INTS $VERS" "$ATT3 $INT3 $VER3" <<'PY' >> "$VERD" || fin 7
import sys, re
labels = []
for line in open(sys.argv[1], encoding='utf-8', errors='replace'):
    m = re.match(r"^([A-Za-z0-9_().'#=-]+): ", line)
    if m and m.group(1) not in labels: labels.append(m.group(1))
rc = 0
for nome, lst in (("MS", sys.argv[2]), ("M3", sys.argv[3])):
    s = set(lst.split()); miss = [l for l in labels if l not in s]
    print(f"copertura {nome}: {len(labels)} etichette nel riferimento, non classificate: {' '.join(miss) or 'nessuna'}")
    if miss: rc = 7
sys.exit(rc)
PY
RC=0
verdetto MS "$ATTS" "$INTS" || RC=$?
for v in $VERS; do grep -qxF "$v" "$OUT/MS.rotte" && note "MS a verdetto: $v ROTTO (inatteso: nessuna raccolta prevista)" || note "MS a verdetto: $v INTATTO -> nessuna raccolta in nessun binario (3ª clausola inerte per gc off / radici < bound)"; done
verdetto M3 "$ATT3" "$INT3" || RC=$?
for v in $VER3; do grep -qxF "$v" "$OUT/M3.rotte" && note "M3 a verdetto: $v ROTTO (inatteso)" || note "M3 a verdetto: $v INTATTO -> la 3ª clausola non decide (gc off / radici < bound)"; done
[ "$RC" = 2 ] && RC=5
if [ "$SOLO" != 1 ]; then
  RS_H1=$(shasum -a 256 "$SRC/run.rs.orig" | cut -c1-16)
  [ "$RS_H0" = "$RS_H1" ] && note "SORGENTE: run.rs del mutante (prima dell'edit) $RS_H1 == commit ${SHA0:0:12}" || { note "SORGENTE: run.rs.orig $RS_H1 ≠ commit $RS_H0 — INCIDENTE"; RC=3; }
fi
rm -rf "$TGT" "$SRC"
note "ESITO rc=$RC (0 = 3ª clausola VIVA nel predicato unico e letta dai siti fusi; 1 = attese non rotte; 5 = presa fuori dominio; 7 = copertura) fine $(date '+%F %T')"
fin $RC
