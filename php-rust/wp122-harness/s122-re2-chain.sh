#!/bin/bash
# s122-re2-chain.sh — ordine criterio p.10: census → build+admission →
# smoke → full. rc SEMPRE da file; stadio fallito ferma (fail-closed).
set -u
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp122-harness"

echo "== stadio 1: census meccanismo =="
bash "$H/s122-census-re2.sh"
[ "$(cat "$H/census-out/census.rc" 2>/dev/null)" = 0 ] || { echo "CHAIN STOP: census.rc != 0 (meccanismo mancato o aumenti)"; exit 1; }

echo "== stadio 2: build candidato + admission =="
bash "$H/s122-re2-build.sh"
[ "$(cat "$H/re2-out/build.rc" 2>/dev/null)" = 0 ] || { echo "CHAIN STOP: build.rc != 0"; exit 2; }

echo "== stadio 3: smoke R=2 (re+str) =="
bash "$H/s122-ab-re2.sh" smoke
src=$(cat "$H/re2-out/smoke-rc" 2>/dev/null)
[ "$src" = 0 ] || { echo "CHAIN STOP: smoke early-stop (rc=$src) — niente full, criterio p.8"; exit 3; }

echo "== stadio 4: full A/B R=5 =="
bash "$H/s122-ab-re2.sh" full
echo "CHAIN DONE: full-rc=$(cat "$H/re2-out/full-rc" 2>/dev/null) (0=promozione-ai-gate 1=guardia 3=invalida 4=sotto-soglia)"
echo done > "$H/re2-out/chain.done"
