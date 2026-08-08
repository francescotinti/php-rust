#!/bin/bash
# s117-aprime-gates.sh — gate PIENI sul candidato A′ (criterio s117-criterio-aprime.md §3).
# Ordine: smoke parità (ricetta pin-phpr.sh, 2 modi vs oracle) → conserva candidato →
# batteria cargo test --release (rc in FILE, inventario per NOME vs S-114/116) →
# corpus --isolate --list-fails ×2 modi col phpt-runner A′: (a) fail-set per NOME
# == congelato s109 (1415) per modo; (b) chunk off↔on byte-id (macchina s102,
# carve-out per NOME lista chiusa). Esiti appesi al verbale DALLO script.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
TGT="/Volumes/Extreme Pro/Claude/phpr-s117-aprime-target"
BIN="$TGT/release/phpr"; RUNNER="$TGT/release/phpt-runner"
ORACLE=/opt/homebrew/opt/php/bin/php
MICRO="$H/../wp97-harness/micro/arith_small.php"
CORPUS="/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests"
FROZ="$H/../wp109-harness/corpus-gate"
REF_NOMI="$H/../wp114-harness/batteria-out/la-batteria-nomi.txt"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
OUT="$H/aprime-out"; mkdir -p "$OUT/corpus"
VERD="$H/s117-aprime-verdetto.out"
note(){ echo "$1"; echo "$1" >> "$VERD"; }

# --- 1. smoke parità (2 modi vs oracle) ---
EXP=$("$ORACLE" "$MICRO")
ON=$(env -u PHPR_REG_LOWER "$BIN" "$MICRO")
OFF=$(PHPR_REG_LOWER=0 "$BIN" "$MICRO")
if [ "$ON" != "$EXP" ] || [ "$OFF" != "$EXP" ]; then
  note "gate SMOKE: FALLITO (oracle='$EXP' on='$ON' off='$OFF') rc=1"; echo 1 > "$OUT/gates-rc"; exit 1
fi
HB=$(shasum -a 256 "$BIN" | cut -c1-16)
note "gate SMOKE: parità arith 2 modi OK (candidato phpr $HB)"

# --- conserva il candidato (NON è un pin: etichetta candidato) ---
cp "$BIN" "$STASH/phpr-s117-aprime"; cp "$RUNNER" "$STASH/phpt-runner-s117-aprime"
HS=$(shasum -a 256 "$STASH/phpr-s117-aprime" | cut -c1-16)
[ "$HB" = "$HS" ] || { note "conserva: hash diverso ($HB vs $HS) rc=1"; echo 1 > "$OUT/gates-rc"; exit 1; }
note "candidato CONSERVATO: phpr-s117-aprime + phpt-runner-s117-aprime ($HB)"

# --- 2. batteria ---
cd "$SRC" || { echo 4 > "$OUT/gates-rc"; exit 4; }
CARGO_TARGET_DIR="$TGT" CARGO_INCREMENTAL=0 cargo test --release > "$OUT/batteria.log" 2>&1
brc=$?
echo "$brc" > "$OUT/batteria.rc"
cnt=$(awk '/^test result:/{p+=$4; f+=$6; ig+=$8} END{printf "%d/%d/%d", p, f, ig}' "$OUT/batteria.log")
grep -E '^test .* \.\.\. ' "$OUT/batteria.log" | sed 's/^test //; s/ \.\.\..*//' | sort > "$OUT/batteria-nomi.txt"
if diff -q "$OUT/batteria-nomi.txt" "$REF_NOMI" > /dev/null; then INV=IDENTICO; else
  INV="DIVERGE (aprime-out/batteria-nomi.diff)"; diff "$REF_NOMI" "$OUT/batteria-nomi.txt" > "$OUT/batteria-nomi.diff"
fi
note "gate BATTERIA: rc=$brc (da aprime-out/batteria.rc) · $cnt · inventario vs S-114: $INV"
GRC=0; [ "$brc" != 0 ] && GRC=1; [ "$INV" != IDENTICO ] && GRC=1

# --- 3. corpus ×2 modi ---
for mode in off on; do
  case "$mode" in off) reg=0 ;; on) reg=1 ;; esac
  /usr/bin/env PHPR_REG_LOWER="$reg" "$RUNNER" --isolate --list-fails "$CORPUS" \
    > "$OUT/corpus/$mode.log" 2>&1
  echo $? > "$OUT/corpus/$mode.rc"
  tr -d '\0' < "$OUT/corpus/$mode.log" > "$OUT/corpus/$mode.norm"
  grep -E '^--- .+ ---$' "$OUT/corpus/$mode.norm" | sed 's/^--- //; s/ ---$//' | sort > "$OUT/corpus/$mode.fails"
  nf=$(wc -l < "$OUT/corpus/$mode.fails" | tr -d ' ')
  decl=$(grep -Eo '^failures: [0-9]+' "$OUT/corpus/$mode.norm" | awk '{print $2}' | tail -1)
  if [ "$nf" != "${decl:--1}" ]; then note "gate CORPUS($mode): VOID — estratti $nf != dichiarati ${decl:-n/d}"; GRC=1; fi
  if diff -q "$OUT/corpus/$mode.fails" "$FROZ/corpus-s109-$mode.fails" > /dev/null; then
    note "gate CORPUS($mode): fail-set per NOME == congelato s109 ($nf)"
  else
    diff "$FROZ/corpus-s109-$mode.fails" "$OUT/corpus/$mode.fails" > "$OUT/corpus/$mode-vs-s109.diff"
    note "gate CORPUS($mode): fail-set DIVERGE dal congelato s109 ($nf vs 1415; aprime-out/corpus/$mode-vs-s109.diff)"; GRC=1
  fi
done

# --- 3b. chunk off↔on byte-id (macchina s102, carve-out lista chiusa) ---
perl - "$OUT/corpus/off.norm" "$OUT/corpus/on.norm" > "$OUT/corpus/offon.txt" 2>&1 <<'PERL'
use strict; use warnings;
my ($f_off, $f_on) = @ARGV;
sub chunks {
  my ($f) = @_;
  open my $h, '<', $f or die "$f: $!";
  local $/; my $all = <$h>;
  my ($decl) = $all =~ /^failures: (\d+)$/m;
  my %c;
  while ($all =~ /^--- (.+?) ---\n(.*?)(?=^--- .+? ---$|\z)/msg) { $c{$1} = $2; }
  return (\%c, $decl // -1);
}
my ($off, $n_off) = chunks($f_off);
my ($on,  $n_on)  = chunks($f_on);
my %nondet = map { $_ => 1 } (
  '/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests/type_coercion/settype/settype_bool_nan_with_error_handler3.phpt',
  '/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests/type_coercion/settype/settype_int_nan_with_error_handler3.phpt',
  '/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests/type_coercion/settype/settype_string_nan_with_error_handler3.phpt',
);
my $void = 0;
for ([$n_off, scalar keys %$off, 'off'], [$n_on, scalar keys %$on, 'on']) {
  my ($decl, $extr, $m) = @$_;
  if ($decl != $extr) { print "GATE VOID ($m): chunk estratti $extr != dichiarati $decl\n"; $void = 1; }
}
exit 65 if $void;
my @only_off = sort grep { !exists $on->{$_} } keys %$off;
my @only_on  = sort grep { !exists $off->{$_} } keys %$on;
my @raw_diff = sort grep { exists $on->{$_} && $off->{$_} ne $on->{$_} } keys %$off;
my @differ   = grep { !$nondet{$_} } @raw_diff;
print "fail solo OFF: ", scalar @only_off, "\n";  print "  $_\n" for @only_off;
print "fail solo ON: ",  scalar @only_on,  "\n";  print "  $_\n" for @only_on;
print "diff per-test DIVERSO: ", scalar @differ, "\n";  print "  $_\n" for @differ;
if (@only_off + @only_on + @differ) { print "CORPUS-DIFF: DIVERSO\n"; exit 1 }
print "CORPUS-DIFF: ZERO differenze per-test off<->on fuori carve-out\n";
PERL
orc=$?
echo "$orc" > "$OUT/corpus/offon.rc"
note "gate CORPUS off↔on: rc=$orc (da aprime-out/corpus/offon.rc) — $(tail -1 "$OUT/corpus/offon.txt")"
[ "$orc" != 0 ] && GRC=1

echo "$GRC" > "$OUT/gates-rc"
note "GATES A′ COMPLESSIVO: rc=$GRC (da aprime-out/gates-rc)"
exit "$GRC"
