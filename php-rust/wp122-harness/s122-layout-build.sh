#!/bin/bash
# s122-layout-build.sh — criterio s122-criterio-layout.md p.1-2: tre build
# P1..P3 dello STESSO sorgente-pin s120 + no-op FIRMATO di taglia crescente
# (probe in coda a crates/php-cli/src/main.rs), admission fail-closed
# (output 6/6 + dump byte-id + hash != pin + simbolo presente), stash
# phpr-s122-lay1..3, ripristino del release al pin AL BYTE. Nessuna misura.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp122-harness"
BIN="$HOME/Claude/php-rust-output/release/phpr"
P0="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s120-re1"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
M="$REPO/wp97-harness/micro"
SRC="$REPO/crates/php-cli/src/main.rs"
OUT="$H/layout-out"; mkdir -p "$OUT"
VERD="$H/s122-layout-verdetto.out"
PIN_EXP="885d2c646ac7ff4c"
note(){ echo "$1"; echo "$1" >> "$VERD"; }
fail(){ note "$1"; echo 1 > "$OUT/build.rc"; exit 1; }

cd "$REPO" || exit 2
H0=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$H0" = "$PIN_EXP" ] || fail "PRE: release/phpr=$H0 != pin $PIN_EXP — STOP"
git diff --quiet -- crates/ || fail "PRE: crates/ sporco — STOP"
HP0=$(shasum -a 256 "$P0" | cut -c1-16)
[ "$HP0" = "$PIN_EXP" ] || fail "PRE: stash P0=$HP0 != pin — STOP"

V=0
for K in 17 173 1731; do
  V=$((V+1))
  git checkout -- crates/ || fail "checkout pre-P$V fallito"
  python3 - "$SRC" "$K" <<'PY' || fail "append probe P$V fallito"
import sys
src, k = sys.argv[1], int(sys.argv[2])
body = "\n".join(
    f"    x = x.wrapping_mul(6364136223846793005).wrapping_add({i});"
    for i in range(k))
code = f'''

// s122 layout probe (no-op FIRMATO, s122-criterio-layout.md p.1): mai chiamata,
// esiste solo per spostare il layout .text di questa build di misura.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __phpr_layout_probe_s122() -> u64 {{
    let mut x: u64 = std::hint::black_box({k}u64);
{body}
    std::hint::black_box(x)
}}
'''
open(src, "a").write(code)
PY
  SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build-p$V.log" 2>&1
  brc=$?; echo "$brc" > "$OUT/build-p$V.rc"
  [ "$brc" = 0 ] || { git checkout -- crates/; fail "build P$V rc=$brc (tree ripristinato)"; }
  HB=$(shasum -a 256 "$BIN" | cut -c1-16)
  [ "$HB" != "$PIN_EXP" ] || fail "P$V riproduce il pin: probe NON entrato — build NULLA"
  nm "$BIN" 2>/dev/null | grep -q "__phpr_layout_probe_s122" \
    || fail "P$V: simbolo probe ASSENTE (dead-strip?) — build NULLA"
  ADM=0
  for C in arith prop calls str arr re; do
    "$P0"  "$M/$C.php" > "$OUT/adm-$C-p0.out" 2>&1
    "$BIN" "$M/$C.php" > "$OUT/adm-$C-p$V.out" 2>&1
    cmp -s "$OUT/adm-$C-p0.out" "$OUT/adm-$C-p$V.out" \
      || { note "P$V admission $C: OUTPUT DIVERGE"; ADM=1; }
    PHPR_DUMP_OPS=1 "$P0"  "$M/$C.php" 2> "$OUT/adm-$C-p0.dump"  > /dev/null
    PHPR_DUMP_OPS=1 "$BIN" "$M/$C.php" 2> "$OUT/adm-$C-p$V.dump" > /dev/null
    cmp -s "$OUT/adm-$C-p0.dump" "$OUT/adm-$C-p$V.dump" \
      || { note "P$V admission $C: DUMP DIVERGE (emissione NON invariata)"; ADM=1; }
  done
  [ "$ADM" = 0 ] || { git checkout -- crates/; fail "P$V admission VIOLATA — build SCARTATA"; }
  cp "$BIN" "$STASH/phpr-s122-lay$V"
  HS=$(shasum -a 256 "$STASH/phpr-s122-lay$V" | cut -c1-16)
  [ "$HB" = "$HS" ] || fail "stash P$V hash diverso ($HB vs $HS)"
  note "P$V (K=$K) = $HB CONSERVATA phpr-s122-lay$V (admission 6/6+6/6, simbolo presente)"
done

git checkout -- crates/ || fail "checkout ripristino finale fallito"
SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build-restore.log" 2>&1
rrc=$?; echo "$rrc" > "$OUT/build-restore.rc"
[ "$rrc" = 0 ] || fail "build ripristino rc=$rrc"
H1=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$H1" = "$PIN_EXP" ] || fail "RIPRISTINO NON AL BYTE: release=$H1 != $PIN_EXP — STOP (determinismo violato)"
note "ripristino: release/phpr == pin $PIN_EXP AL BYTE; layout P1..P3 pronti"
echo 0 > "$OUT/build.rc"
