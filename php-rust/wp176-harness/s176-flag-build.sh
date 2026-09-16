#!/bin/bash
# s176-flag-build.sh — bracci della leva «flag gc-idle» (criterio s176-criterio-flag.md p.2 e p.5): COPIA DICHIARATA
# di ../wp174-harness/s175-mutante-gc.sh (manifest s176-flag-build-copia.diff) + `bilat` di s174-promozione.sh, coi SOLI
# adattamenti: (1) sorgente = COMMIT della leva (dichiarato per hash nel verdetto); (2) DUE build dalla stessa ricetta:
# B = leva intatta (nessun perl), MF = mutante «flag mai aggiornato» (needle = la riga di `gc_idle_set` in mod.rs: lo
# store diventa no-op, il flag resta [true; 2] dall'init ⇒ Sweep sempre saltato; 1 riga); (3) riferimento A = pin s175
# (5de14d6856d760a8) al posto del braccio C; (4) fx-sw2-gc: B == A BYTE-ID (invarianza; rotte per etichetta se diverge)
# + fx-sw1 BILATERALE: B == oracle byte-id con marcatore FX-SW1 DONE; (5) MF: fx-sw2-gc DEVE rompere gcp-fused e
# gcp-loop (come MS S-175), fx-sw1 DEVE divergere dall'oracle; (6) tag s176: out ab-out/s176-flag, sorgente/target
# /Volumes/Extreme Pro/Claude/s176-flag{,-tgt}; RS = crates/php-runtime/src/vm/mod.rs (il needle vive lì).
# rc (ab-out/s176-flag.done): 0 = B a parità E MF morde ⇒ bracci pronti · 1 = MF non rompe le attese (flag non letto)
# · 2 = B diverge (fx-sw2-gc o fx-sw1) · 4 = build fallita · 6 = mutante non entrato / righe ≠ 1 · 7 = file/commit
# · 8 = disco · 9 = lock/pin.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment"
H="$REPO/php-rust/wp176-harness"; OUT="$H/ab-out/s176-flag"; mkdir -p "$OUT"
VERD="$H/s176-flag-build-verdetto.out"; DONE="$H/ab-out/s176-flag.done"; rm -f "$DONE"
LOCK=/private/tmp/phpr-measure.lock
PIN="$HOME/Claude/php-rust-output/release/phpr"; PIN_ATTESO="5de14d6856d760a8"
ORACLE=/opt/homebrew/opt/php/bin/php
COMMIT="${COMMIT:?COMMIT della leva (hash)}"
FX2="$REPO/php-rust/wp174-harness/fixtures/fx-sw2-gc.php"
FX1="$REPO/php-rust/wp174-harness/fixtures/fx-sw1.php"
SRC="/Volumes/Extreme Pro/Claude/s176-flag"; TGT="/Volumes/Extreme Pro/Claude/s176-flag-tgt"
RS=crates/php-runtime/src/vm/mod.rs
: > "$VERD"
fin(){ echo "rc=$1 $(date +%T)" > "$DONE"; exit "$1"; }
note(){ echo "$*" >> "$VERD"; }

grep -qw s176 "$LOCK" 2>/dev/null || { note "rc=9 lock s176 assente (per TOKEN)"; fin 9; }
[ -s "$FX2" ] && [ -s "$FX1" ] || { note "rc=7 fixture assente"; fin 7; }
PH=$(shasum -a 256 "$PIN" | cut -c1-16)
[ "$PH" = "$PIN_ATTESO" ] || { note "rc=9 pin $PH ≠ atteso $PIN_ATTESO"; fin 9; }
cd "$REPO" || fin 7
SHA0=$(git rev-parse --verify "$COMMIT^{commit}") || { note "rc=7 commit $COMMIT inesistente"; fin 7; }
RS_H0=$(git show "$SHA0:php-rust/$RS" | shasum -a 256 | cut -c1-16)
AVX=$(df -k "/Volumes/Extreme Pro" | awk 'NR>1{printf "%.0f", $4/1048576}')
note "== s176 bracci leva «flag gc-idle» — riferimento A = pin s175 $PH, sorgente = commit ${SHA0:0:12} (mod.rs $RS_H0), fixture fx-sw2-gc $(wc -l < "$FX2" | tr -d ' ') righe + fx-sw1 $(wc -l < "$FX1" | tr -d ' ') righe, Extreme ${AVX}G $(date '+%F %T') =="
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
[ -s "$SRC/php-rust/$RS" ] || { note "rc=7 archivio senza mod.rs"; fin 7; }
/usr/bin/find "$SRC" -type f -exec touch {} +
cp "$SRC/php-rust/$RS" "$SRC/mod.rs.orig"

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

build_run(){ # $1=nome $2=needle-perl("" = nessuna mutazione) $3=repl-perl
  local n="$1" needle="$2" repl="$3"
  cp "$SRC/mod.rs.orig" "$SRC/php-rust/$RS"
  if [ -n "$needle" ]; then
    perl -0pi -e "s/\Q$needle\E/$repl/" "$SRC/php-rust/$RS"
    local nrep; nrep=$(diff "$SRC/mod.rs.orig" "$SRC/php-rust/$RS" | grep -c '^>')
    [ "$nrep" = 1 ] || { note "rc=6 $n: il mutante ha scritto $nrep righe (attesa 1)"; fin 6; }
    diff -u "$SRC/mod.rs.orig" "$SRC/php-rust/$RS" > "$OUT/$n.diff"
  fi
  /usr/bin/touch "$SRC/php-rust/$RS"
  ( cd "$SRC/php-rust" && SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TGT" \
      cargo build --release -p php-cli ) > "$OUT/build-$n.log" 2>&1 \
    || { note "rc=4 $n: build FALLITA (ab-out/s176-flag/build-$n.log)"; fin 4; }
  local mh; mh=$(shasum -a 256 "$TGT/release/phpr" | cut -c1-16)
  [ "$mh" != "$PH" ] || { note "rc=6 $n: binario == riferimento (sorgente NON entrata)"; fin 6; }
  cp "$TGT/release/phpr" "$OUT/phpr-$n"
  perl -e 'alarm 120; exec @ARGV or die' -- "$OUT/phpr-$n" "$FX2" > "$OUT/$n.out" 2>&1
  local rrc=$?
  perl -e 'alarm 120; exec @ARGV or die' -- "$OUT/phpr-$n" "$FX1" > "$OUT/$n-sw1.out" 2>&1
  note "$n: binario $(shasum -a 256 "$OUT/phpr-$n" | cut -c1-16) rc_fixture=$rrc"
  rotte "$OUT/ref.out" "$OUT/$n.out" | sort -u > "$OUT/$n.rotte"
  note "$n: fx-sw2-gc blocchi ROTTI vs pin ($(wc -l < "$OUT/$n.rotte" | tr -d ' ')): $(tr '\n' ' ' < "$OUT/$n.rotte")"
  if diff -q "$OUT/sw1-oracle.out" "$OUT/$n-sw1.out" > /dev/null; then note "$n: fx-sw1 == oracle BYTE-ID"; else diff "$OUT/sw1-oracle.out" "$OUT/$n-sw1.out" > "$OUT/$n-sw1.diff" || true; note "$n: fx-sw1 DIVERGE dall'oracle ($(wc -l < "$OUT/$n-sw1.diff" | tr -d ' ') righe di diff)"; fi
}

build_run B "" ""
build_run MF 'fn gc_idle_set(&mut self, v: [bool; 2]) { self.gc_idle = v; }' 'fn gc_idle_set(&mut self, v: [bool; 2]) { let _ = v; }'

RC=0
# B: parità PRIMA della misura (p.5)
if diff -q "$OUT/ref.out" "$OUT/B.out" > /dev/null; then note "B: fx-sw2-gc == pin BYTE-ID (invarianza)"; else diff "$OUT/ref.out" "$OUT/B.out" > "$OUT/B-invarianza.diff" || true; note "B: fx-sw2-gc ≠ pin (ab-out/s176-flag/B-invarianza.diff) -> rc=2"; RC=2; fi
[ -e "$OUT/B-sw1.diff" ] && { note "B: fx-sw1 diverge -> rc=2"; RC=2; }
# MF: il mutante deve mordere (p.5)
miss=""; for a in gcp-fused gcp-loop; do grep -qxF "$a" "$OUT/MF.rotte" || miss="$miss $a"; done
if [ -z "$miss" ]; then note "MF: attese ROTTE (gcp-fused gcp-loop) -> il flag è letto dai siti fusi/handler"; else note "MF: attese NON rotte:$miss -> il flag NON decide -> rc=1"; [ "$RC" = 0 ] && RC=1; fi
[ -e "$OUT/MF-sw1.diff" ] && note "MF: fx-sw1 diverge dall'oracle (atteso)" || { note "MF: fx-sw1 == oracle (INATTESO: il mutante non morde su fx-sw1) -> rc=1"; [ "$RC" = 0 ] && RC=1; }
RS_H1=$(shasum -a 256 "$SRC/mod.rs.orig" | cut -c1-16)
[ "$RS_H0" = "$RS_H1" ] && note "SORGENTE: mod.rs archiviato $RS_H1 == commit ${SHA0:0:12}" || { note "SORGENTE: mod.rs.orig $RS_H1 ≠ commit $RS_H0 — INCIDENTE"; RC=3; }
rm -rf "$TGT" "$SRC"
note "ESITO rc=$RC (0 = bracci pronti: B a parità e MF morde; 1 = MF non morde; 2 = B diverge; 3 = sorgente) fine $(date '+%F %T')"
fin $RC
