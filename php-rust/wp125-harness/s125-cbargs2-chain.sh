#!/bin/bash
# s125-cbargs2-chain.sh — catena DETACHED leva L-HD2 FORMA-2 (criterio
# s125-criterio-cbargs2.md, committato PRIMA). Nessuna attesa (macchina
# libera): commit patch → admission census → build candidato + stash cb2 →
# disasm → smoke R=2 MINFAM=2 con lettore a segno → A/B R=5 → giudice.
# NON promossa ⇒ revert verificato al byte. rc finale in cbargs2-chain.done.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp125-harness"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
BIN="$HOME/Claude/php-rust-output/release/phpr"
PIN_EXP="c5ba2573a23adf69"
MARKER="S-125 L-HD2 cbargs2: patch forma-2 (perimetro criterio p.7)"
p(){ echo "$(date +%H:%M:%S) $1" >> "$H/cbargs2-progress.txt"; }
: > "$H/cbargs2-progress.txt"
rm -f "$H/cbargs2-chain.done"
revert_pin(){
  git revert --no-edit HEAD >> "$H/cbargs2-progress.txt" 2>&1
  SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$H/census-out/build-revert2.log" 2>&1
  [ "$(shasum -a 256 "$BIN" | cut -c1-16)" = "$PIN_EXP" ] && p "revert: pin al byte OK" || p "revert: PIN NON RIPRODOTTO — INDAGARE"
}
fine(){ echo "rc=$1 fase=$2 $(date +%T)" > "$H/cbargs2-chain.done"; p "DONE rc=$1 fase=$2"; git push >> "$H/cbargs2-progress.txt" 2>&1 || p "push FALLITO"; exit "$1"; }

cd "$REPO" || exit 2
git diff --quiet || fine 1 "tree-sporco-pre-patch"
[ "$(shasum -a 256 "$BIN" | cut -c1-16)" = "$PIN_EXP" ] || fine 1 "release-non-pin"
[ "$(cat "$H/banda-out/band.rc" 2>/dev/null)" = 0 ] || fine 1 "banda-mancante"
git apply "$H/s125-cbargs2.patch" || fine 1 "patch-non-applica"
git add -u && printf '%s\n' "$MARKER" > /tmp/cbargs2-cmsg && git commit -F /tmp/cbargs2-cmsg >> "$H/cbargs2-progress.txt" 2>&1 || fine 1 "commit-patch"
p "patch committata $(git rev-parse --short HEAD)"

p "admission census START"
"$H/s125-cbargs2-census.sh" >> "$H/cbargs2-progress.txt" 2>&1; crc=$?
p "admission census rc=$crc"
[ "$crc" = 0 ] || { revert_pin; fine 2 "admission-fuori-predizione"; }

p "build candidato ricetta START"
SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$H/census-out/build-cb2.log" 2>&1 || { revert_pin; fine 1 "build-candidato"; }
CB=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$CB" != "$PIN_EXP" ] || { revert_pin; fine 1 "candidato==pin"; }
cp "$BIN" "$STASH/phpr-s125-cb2" && chmod +x "$STASH/phpr-s125-cb2"
[ "$(shasum -a 256 "$STASH/phpr-s125-cb2" | cut -c1-16)" = "$CB" ] || { revert_pin; fine 1 "stash-hash"; }
p "candidato cb2 = $CB stash phpr-s125-cb2"

SCRATCH="$H/census-out" "$REPO/wp105-harness/s105-admission-disasm.sh" "$STASH/phpr-s125-cb2" cb2 > "$H/census-out/disasm-cb2.txt" 2>&1
{ echo "== disasm admission (criterio p.6; pin da census-out/disasm-pin.txt della forma-1) =="
  grep -E "run_loop_size_B|bl_totali" "$H/census-out/disasm-pin.txt" | sed 's/^/pin: /'
  grep -E "run_loop_size_B|bl_totali" "$H/census-out/disasm-cb2.txt" | sed 's/^/cb2: /'
} >> "$H/s125-cbargs2-verdetto.out"

p "smoke R=2 MINFAM=2"
MINFAM=2 "$H/s125-ab.sh" "$STASH/phpr-s125-cb2" "${CB:0:8}" cb2smoke 2 0 str >> "$H/cbargs2-progress.txt" 2>&1; src=$?
[ "$src" = 0 ] || { p "smoke rc=$src"; revert_pin; fine 1 "smoke-ab"; }
SMOKE=$(python3 - "$H/metro-out/cb2smoke-runs.tsv" <<'PY'
import sys
ds = []
for l in open(sys.argv[1]):
    c, n, i, ta, tb, fa, fb, o = l.split('\t')
    n = float(n); ds.append(((float(ta)-float(fa))/n - (float(tb)-float(fb))/n)*1e9)
print(("OPPOSTO" if all(d < 0 for d in ds) else "OK") + " " + " ".join(f"{d:+.2f}" for d in ds))
PY
)
p "smoke: $SMOKE"
echo "smoke R=2 MINFAM=2: $SMOKE" >> "$H/s125-cbargs2-verdetto.out"
case "$SMOKE" in OPPOSTO*) revert_pin; fine 3 "smoke-segno-opposto" ;; esac

p "A/B R=5 START (6 categorie)"
"$H/s125-ab.sh" "$STASH/phpr-s125-cb2" "${CB:0:8}" cb2 5 5 arith prop calls str arr re >> "$H/cbargs2-progress.txt" 2>&1 || { revert_pin; fine 1 "ab-r5"; }
"$H/s125-giudice.sh" cb2 str >> "$H/cbargs2-progress.txt" 2>&1; grc=$?
p "giudice rc=$grc (0=promo 1=refut 2=nondist 3=invalida)"
[ "$grc" = 0 ] || revert_pin
fine "$grc" "giudice"
