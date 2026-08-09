#!/bin/bash
# s125-cbargs-chain.sh — catena DETACHED leva L-HD2 (criterio
# s125-criterio-cbargs.md, committato PRIMA): attende p2-chain.done, poi:
# commit patch (perimetro dichiarato) → admission census (arbitro p.4) →
# build candidato ricetta + stash cb1 → disasm admission (s105) → smoke R=2
# early-stop a segno opposto → A/B R=5 alternato → giudice (banda v2-s125).
# NON promossa ⇒ revert del commit leva + build ricetta + pin al byte.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp125-harness"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
BIN="$HOME/Claude/php-rust-output/release/phpr"
PIN_EXP="c5ba2573a23adf69"
MARKER="S-125 L-HD2 cbargs: patch (perimetro criterio p.2)"
p(){ echo "$(date +%H:%M:%S) $1" >> "$H/cbargs-progress.txt"; }
: > "$H/cbargs-progress.txt"
rm -f "$H/cbargs-chain.done"
fine(){ echo "rc=$1 fase=$2 $(date +%T)" > "$H/cbargs-chain.done"; p "DONE rc=$1 fase=$2"; exit "$1"; }

p "attesa p2-chain.done (poll 60s)"
while [ ! -f "$H/p2-chain.done" ]; do sleep 60; done
p "p.2 concluso: $(cat "$H/p2-chain.done")"
[ "$(cat "$H/census-out/run.rc" 2>/dev/null)" = 0 ] || fine 1 "baseline-A-census-mancante"
[ "$(cat "$H/banda-out/band.rc" 2>/dev/null)" = 0 ] || fine 1 "banda-v2-s125-mancante"

cd "$REPO" || fine 1 cd
git diff --quiet || fine 1 "tree-sporco-pre-patch"
[ "$(shasum -a 256 "$BIN" | cut -c1-16)" = "$PIN_EXP" ] || fine 1 "release-non-pin-pre-patch"
git apply "$H/s125-cbargs.patch" || fine 1 "patch-non-applica"
git add -u || fine 1 "git-add"
printf '%s\n' "$MARKER" > /tmp/cbargs-cmsg && git commit -F /tmp/cbargs-cmsg >> "$H/cbargs-progress.txt" 2>&1 || fine 1 "commit-patch"
p "patch committata $(git rev-parse --short HEAD)"

p "admission census START"
"$H/s125-cbargs-census.sh" >> "$H/cbargs-progress.txt" 2>&1
crc=$?
p "admission census rc=$crc"
if [ "$crc" != 0 ]; then
  git revert --no-edit HEAD >> "$H/cbargs-progress.txt" 2>&1
  SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$H/census-out/build-revert.log" 2>&1
  [ "$(shasum -a 256 "$BIN" | cut -c1-16)" = "$PIN_EXP" ] && p "revert: pin al byte OK" || p "revert: PIN NON RIPRODOTTO — INDAGARE"
  fine 2 "admission-census-fuori-predizione"
fi

p "build candidato ricetta START"
SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$H/census-out/build-cb1.log" 2>&1 || fine 1 "build-candidato"
CB=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$CB" != "$PIN_EXP" ] || fine 1 "candidato==pin (leva non entrata?)"
cp "$BIN" "$STASH/phpr-s125-cb1" && chmod +x "$STASH/phpr-s125-cb1"
[ "$(shasum -a 256 "$STASH/phpr-s125-cb1" | cut -c1-16)" = "$CB" ] || fine 1 "stash-hash-diverso"
p "candidato cb1 = $CB stash phpr-s125-cb1"

p "disasm admission (bl-count run_loop pin vs cb1)"
SCRATCH="$H/census-out" "$REPO/wp105-harness/s105-admission-disasm.sh" "$STASH/phpr-s124" pin-s124 > "$H/census-out/disasm-pin.txt" 2>&1
SCRATCH="$H/census-out" "$REPO/wp105-harness/s105-admission-disasm.sh" "$STASH/phpr-s125-cb1" cb1 > "$H/census-out/disasm-cb1.txt" 2>&1
{ echo "== disasm admission (criterio p.6) =="
  grep -E "run_loop_size_B|bl_totali" "$H/census-out/disasm-pin.txt" | sed 's/^/pin: /'
  grep -E "run_loop_size_B|bl_totali" "$H/census-out/disasm-cb1.txt" | sed 's/^/cb1: /'
} >> "$H/s125-cbargs-verdetto.out"

p "smoke R=2 early-stop"
"$H/s125-ab.sh" "$STASH/phpr-s125-cb1" "${CB:0:8}" cb1smoke 2 0 str >> "$H/cbargs-progress.txt" 2>&1
src=$?
if [ "$src" = 0 ]; then
  SMOKE=$(python3 - "$H/metro-out/cb1smoke-runs.tsv" <<'PY'
import sys
ds = []
for l in open(sys.argv[1]):
    c, n, i, ta, tb, fa, fb, o = l.split('\t')
    n = float(n); ds.append(((float(ta)-float(fa))/n - (float(tb)-float(fb))/n)*1e9)
print("OPPOSTO" if all(d < 0 for d in ds) else "OK", " ".join(f"{d:+.2f}" for d in ds))
PY
)
  p "smoke: $SMOKE"
  echo "smoke R=2: $SMOKE" >> "$H/s125-cbargs-verdetto.out"
  case "$SMOKE" in OPPOSTO*)
    git revert --no-edit HEAD >> "$H/cbargs-progress.txt" 2>&1
    SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$H/census-out/build-revert.log" 2>&1
    [ "$(shasum -a 256 "$BIN" | cut -c1-16)" = "$PIN_EXP" ] && p "revert: pin al byte OK" || p "revert: PIN NON RIPRODOTTO — INDAGARE"
    fine 3 "smoke-segno-opposto" ;;
  esac
else
  p "smoke rc=$src — STOP"; fine 1 "smoke-ab"
fi

p "A/B R=5 START (6 categorie)"
"$H/s125-ab.sh" "$STASH/phpr-s125-cb1" "${CB:0:8}" cb1 5 5 arith prop calls str arr re >> "$H/cbargs-progress.txt" 2>&1 || fine 1 "ab-r5"
"$H/s125-giudice.sh" cb1 str >> "$H/cbargs-progress.txt" 2>&1
grc=$?
p "giudice rc=$grc (0=promo 1=refut 2=nondist 3=invalida)"
if [ "$grc" != 0 ]; then
  git revert --no-edit HEAD >> "$H/cbargs-progress.txt" 2>&1
  SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$H/census-out/build-revert.log" 2>&1
  [ "$(shasum -a 256 "$BIN" | cut -c1-16)" = "$PIN_EXP" ] && p "revert: pin al byte OK" || p "revert: PIN NON RIPRODOTTO — INDAGARE"
fi
git push >> "$H/cbargs-progress.txt" 2>&1 || p "push FALLITO (rimandare al mattino)"
fine "$grc" "giudice"
