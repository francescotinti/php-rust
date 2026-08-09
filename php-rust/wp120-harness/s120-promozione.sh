#!/bin/bash
# s120-promozione.sh <cand_hash16> — gate di promozione L-RE1 (criterio
# s120-criterio-re1.md p.8; bersaglio+guardie già a verbale, rc=0 dal full).
# Ordine REGOLE §6: commit leva → build ricetta (DEVE riprodurre il candidato)
# → batteria (rc in FILE, inventario per NOME vs pin s119) → re-build ricetta
# → pin-phpr.sh s120 → corpus-gate CANONICO → fixture chain → micro R=5 sul pin.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp120-harness"
BIN="$HOME/Claude/php-rust-output/release/phpr"
RUNNER="$HOME/Claude/php-rust-output/release/phpt-runner"
OUT="$H/promo-out"; mkdir -p "$OUT"
VERD="$H/s120-re1-verdetto.out"
CAND_EXP="${1:?uso: s120-promozione.sh <cand_hash16>}"
PIN_OLD="350582e55d21533d"
note(){ echo "$1"; echo "$1" >> "$VERD"; }
stop(){ note "$1"; echo 1 > "$OUT/rc"; exit 1; }

cd "$SRC" || exit 4
H0=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$H0" = "$PIN_OLD" ] || stop "PRE: release=$H0 != pin $PIN_OLD — STOP"
git diff --quiet -- crates/ || stop "PRE: crates/ sporco — STOP"

git apply "$H/re1.patch" || stop "patch NON applica"
git add -u crates/ || stop "git add -u fallito"
printf 'S-120 L-RE1: preg_match hot path — 17->10 alloc/iter sul giudice re\n\nQuattro siti (classifica C-lite S-119, re +12 vs Zend): (a) borrow di\npattern/subject in ho_preg_match (2 alloc + memcpy O(len)); (b) fast-path\nsenza nomi in captures_array (Vec dei nomi ricostruito a ogni chiamata);\n(c) il testo dei gruppi MUOVE nel PhpStr (via la seconda copia);\n(d) scratch CaptureLocations riusato per Engine::Regex (wrapper Deref,\ncaptures_read_at). Criterio PRE a57595a; census: re 17,00->10,00,\nnon-bersaglio identiche; A/B full R=5: re D_med +100,00 ns/iter (5/5,\nsoglia +20,00), guardie 6/6 tengono. Admission 6/6 output + 6/6 dump\ninteri byte-id (deroga p.6 dichiarata).\n\nCo-Authored-By: Claude Fable 5 <noreply@anthropic.com>\n' > /tmp/cmsg-re1
git commit -F /tmp/cmsg-re1 > "$OUT/commit.log" 2>&1 || stop "commit leva fallito"
HEADL=$(git rev-parse --short HEAD)
note "promozione: leva committata @ $HEADL"

SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build.log" 2>&1
rc=$?; echo "$rc" > "$OUT/build.rc"
[ "$rc" = 0 ] || stop "build rc=$rc"
HB=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$HB" = "$CAND_EXP" ] || stop "build NON riproduce il candidato giudicato ($HB != $CAND_EXP) — STOP"
note "promozione: build ricetta riproduce il candidato $HB"

CARGO_INCREMENTAL=0 cargo test --release > "$OUT/batteria.log" 2>&1
brc=$?; echo "$brc" > "$OUT/batteria.rc"
cnt=$(awk '/^test result:/{p+=$4; f+=$6; ig+=$8} END{printf "%d/%d/%d", p, f, ig}' "$OUT/batteria.log")
grep -E '^test .* \.\.\. ' "$OUT/batteria.log" | sed 's/^test //; s/ \.\.\..*//' | sort > "$OUT/batteria-nomi.txt"
if diff -q "$OUT/batteria-nomi.txt" "$SRC/wp119-harness/promo-out/batteria-nomi.txt" > /dev/null; then INV=IDENTICO; else
  INV="DIVERGE (promo-out/batteria-nomi.diff)"; diff "$SRC/wp119-harness/promo-out/batteria-nomi.txt" "$OUT/batteria-nomi.txt" > "$OUT/batteria-nomi.diff"
fi
note "promozione batteria: rc=$brc (da promo-out/batteria.rc) · $cnt · inventario vs pin s119: $INV"
[ "$brc" = 0 ] || stop "batteria rc=$brc"
[ "$INV" = IDENTICO ] || stop "inventario batteria DIVERGE"

SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build2.log" 2>&1
rc=$?; echo "$rc" > "$OUT/build2.rc"
[ "$rc" = 0 ] || stop "build2 rc=$rc"
H2=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$H2" = "$CAND_EXP" ] || stop "re-hash post-batteria $H2 != $CAND_EXP — STOP"
note "promozione: churn batteria neutralizzato (build ricetta → $H2 al byte)"

"$SRC/scripts/pin-phpr.sh" s120 > "$OUT/pin.log" 2>&1
prc=$?; echo "$prc" > "$OUT/pin.rc"
[ "$prc" = 0 ] || stop "pin-phpr.sh rc=$prc"
note "promozione: $(tail -1 "$OUT/pin.log")"

"$SRC/scripts/corpus-gate.sh" "$RUNNER" "$OUT/corpus"
crc=$?; echo "$crc" > "$OUT/corpus-rc"
[ "$crc" = 0 ] || stop "corpus-gate rc=$crc (vedi promo-out/corpus/corpus-gate.out)"
note "promozione corpus-gate: rc=0 — nomi==congelato, CONTENUTO==golden, off↔on zero"

PHPR_PIN_ATTESO="$CAND_EXP" "$SRC/wp109-harness/s109-fixture-chain.sh" > "$OUT/fixture-chain.out" 2>&1
frc=$?; echo "$frc" > "$OUT/fixture-chain.rc"
[ "$frc" = 0 ] || stop "fixture chain rc=$frc"
note "promozione fixture chain: rc=0 ($(grep -c '^-- .* rc=0' "$OUT/fixture-chain.out")/6 gate verdi)"

PHPR="$BIN" R=5 "$SRC/wp97-harness/micro/run-micro.sh" > "$OUT/micro-pin-s120.out" 2>&1
note "promozione micro pin s120: $(grep -E '^rapporto_' "$OUT/micro-pin-s120.out" | tr '\n' ' ')"
echo 0 > "$OUT/rc"
note "PROMOZIONE COMPLETA rc=0 (da promo-out/rc): pin s120 = $H2 @ $HEADL"
exit 0
