#!/bin/bash
# s117-promozione.sh — criterio s117-criterio-la.md p.7: L-A PROMOSSA ⇒
# cherry-pick 2c18b2e COMMITTATO + §6 PIENO sul tree promosso, ricetta A′:
# build (output canonico) → hash → batteria → re-hash → stash via pin-phpr.sh
# → corpus ×2 vs congelato + off↔on → micro R=5 + held-out R=5. rc in FILE.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
export SOURCE_DATE_EPOCH=0
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
ROOT="/Volumes/Extreme Pro/Claude/php-rust-experiment"
SRC="$ROOT/php-rust"
BIN="$HOME/Claude/php-rust-output/release/phpr"
RUNNER="$HOME/Claude/php-rust-output/release/phpt-runner"
CORPUS="/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests"
FROZ="$H/../wp109-harness/corpus-gate"
REF_NOMI="$H/../wp114-harness/batteria-out/la-batteria-nomi.txt"
OUT="$H/promo-out"; mkdir -p "$OUT/corpus"
VERD="$H/s117-la-verdetto.out"
note(){ echo "$1"; echo "$1" >> "$VERD"; }

cd "$ROOT" || exit 4
git diff --quiet && git diff --cached --quiet || { note "promozione: tree NON pulito — STOP"; echo 4 > "$OUT/rc"; exit 4; }
git cherry-pick 2c18b2e > "$OUT/cherry.log" 2>&1 || { note "promozione: cherry-pick FALLITO"; git cherry-pick --abort; echo 4 > "$OUT/rc"; exit 4; }
note "promozione: cherry-pick 2c18b2e committato ($(git rev-parse --short HEAD))"

# §6: build → hash
cd "$SRC"
CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build.log" 2>&1
rc=$?; echo "$rc" > "$OUT/build.rc"
[ "$rc" = 0 ] || { note "promozione: build FALLITA rc=$rc"; echo 4 > "$OUT/rc"; exit 4; }
H1=$(shasum -a 256 "$BIN" | cut -c1-16)
note "promozione: build rc=0, phpr=$H1"

# batteria
CARGO_INCREMENTAL=0 cargo test --release > "$OUT/batteria.log" 2>&1
brc=$?; echo "$brc" > "$OUT/batteria.rc"
cnt=$(awk '/^test result:/{p+=$4; f+=$6; ig+=$8} END{printf "%d/%d/%d", p, f, ig}' "$OUT/batteria.log")
grep -E '^test .* \.\.\. ' "$OUT/batteria.log" | sed 's/^test //; s/ \.\.\..*//' | sort > "$OUT/batteria-nomi.txt"
if diff -q "$OUT/batteria-nomi.txt" "$REF_NOMI" > /dev/null; then INV=IDENTICO; else INV="DIVERGE"; diff "$REF_NOMI" "$OUT/batteria-nomi.txt" > "$OUT/batteria-nomi.diff"; fi
note "promozione: batteria rc=$brc (da promo-out/batteria.rc) · $cnt · inventario: $INV"
[ "$brc" = 0 ] && [ "$INV" = IDENTICO ] || { note "promozione: BATTERIA FALLITA — pin NON eseguito"; echo 1 > "$OUT/rc"; exit 1; }

# re-hash (build dopo stash = gate nullo: qui re-hash PRIMA dello stash)
H2=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$H1" = "$H2" ] || { note "promozione: re-hash DIVERSO ($H1 vs $H2) — churn non dichiarato, STOP"; echo 1 > "$OUT/rc"; exit 1; }
note "promozione: re-hash OK ($H2)"

# stash via pin-phpr.sh (smoke parità nell'atto)
"$SRC/scripts/pin-phpr.sh" s117 > "$OUT/pin.log" 2>&1
prc=$?; echo "$prc" > "$OUT/pin.rc"
[ "$prc" = 0 ] || { note "promozione: pin-phpr.sh FALLITO rc=$prc"; echo 1 > "$OUT/rc"; exit 1; }
note "promozione: $(tail -1 "$OUT/pin.log")"

# corpus ×2 + off↔on col runner del pin nuovo
GRC=0
for mode in off on; do
  case "$mode" in off) reg=0 ;; on) reg=1 ;; esac
  /usr/bin/env PHPR_REG_LOWER="$reg" "$RUNNER" --isolate --list-fails "$CORPUS" > "$OUT/corpus/$mode.log" 2>&1
  echo $? > "$OUT/corpus/$mode.rc"
  tr -d '\0' < "$OUT/corpus/$mode.log" > "$OUT/corpus/$mode.norm"
  grep -E '^--- .+ ---$' "$OUT/corpus/$mode.norm" | sed 's/^--- //; s/ ---$//' | sort > "$OUT/corpus/$mode.fails"
  nf=$(wc -l < "$OUT/corpus/$mode.fails" | tr -d ' ')
  if diff -q "$OUT/corpus/$mode.fails" "$FROZ/corpus-s109-$mode.fails" > /dev/null; then
    note "promozione corpus($mode): fail-set per NOME == congelato s109 ($nf)"
  else
    diff "$FROZ/corpus-s109-$mode.fails" "$OUT/corpus/$mode.fails" > "$OUT/corpus/$mode-vs-s109.diff"
    note "promozione corpus($mode): DIVERGE dal congelato ($nf vs 1415)"; GRC=1
  fi
done
if diff -q "$OUT/corpus/off.fails" "$OUT/corpus/on.fails" > /dev/null; then
  note "promozione corpus: insieme nomi off==on"
else
  note "promozione corpus: nomi off≠on"; GRC=1
fi
echo "$GRC" > "$OUT/corpus-rc"
[ "$GRC" = 0 ] || { note "promozione: CORPUS FALLITO"; echo 1 > "$OUT/rc"; exit 1; }

# micro R=5 + held-out R=5 sul pin nuovo (nuove baseline ufficiali)
PHPR="$BIN" R=5 "$SRC/wp97-harness/micro/run-micro.sh" > "$OUT/micro-pin-s117.out" 2>&1
PHPR="$BIN" R=5 "$SRC/wp111-harness/heldout/run-heldout.sh" > "$OUT/heldout-pin-s117.out" 2>&1
note "promozione micro pin s117: $(grep -E '^rapporto_' "$OUT/micro-pin-s117.out" | tr '\n' ' ')"
note "promozione held-out pin s117: $(grep -E '^rapporto_' "$OUT/heldout-pin-s117.out" | tr '\n' ' ')"
echo 0 > "$OUT/rc"
note "PROMOZIONE COMPLETA rc=0 (da promo-out/rc): pin s117 = $H2"
exit 0
