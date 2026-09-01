#!/bin/bash
# s167-f0d.sh — FETTA 0 braccio (d): drop-census su arith (criterio
# s167-criterio-f0.md). Probe = TREE==pin s166 (nessuna patch) build
# --features zval-census in target dedicato; driver arith-census.php N=2M;
# MUTANTE arith-dropmut.php (N=200k, +1 stringa/iter): DropS deve spostarsi
# di +200000 ESATTO. Repliche r1==r2. rc autoritativo = f0-out/f0d.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp167-harness"; OUT="$H/f0-out"; mkdir -p "$OUT"
VERD="$H/s167-f0d-verdetto.out"; RC="$OUT/f0d.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE" >&2; exit 7; }
SRC=/private/tmp/s167-census-f0
rm -rf "$SRC"; mkdir -p "$SRC/php-rust"
for f in crates Cargo.toml Cargo.lock rust-toolchain.toml .cargo; do
  cp -R "$REPO/$f" "$SRC/php-rust/" || { echo "rc=7 copia ($f)" >> "$VERD"; echo 7 > "$RC"; exit 7; }
done
( cd "$SRC/php-rust" && SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$SRC/tgt" \
  cargo build --release -p php-cli --features zval-census ) > "$OUT/f0d-build.log" 2>&1 \
  || { echo "rc=7 build probe (log f0-out/f0d-build.log)" >> "$VERD"; echo 7 > "$RC"; exit 7; }
P="$SRC/tgt/release/phpr"
{
echo "== s167 F0 braccio (d) drop-census (probe $(shasum -a 256 "$P" | cut -c1-8) da tree==pin; driver N=2M; mutante +200000 DropS atteso) =="
for tag in main-r1 main-r2 mut-r1 mut-r2; do
  case "$tag" in main-*) D="$H/arith-census.php";; mut-*) D="$H/arith-dropmut.php";; esac
  RAW="$OUT/f0d-$tag.raw"; rm -f "$RAW"
  PHPR_ZVAL_CENSUS="$RAW" "$P" "$D" > "$OUT/f0d-$tag.out" 2>&1
  [ -s "$RAW" ] || { echo "probe muto ($tag)"; echo 8 > "$RC"; exit 8; }
  tr -d '\0' < "$RAW" > "$OUT/f0d-$tag.txt"
done
python3 - "$OUT" <<'PY'
import sys, re
out = sys.argv[1]
def tot(f):
    t = {}
    for l in open(f"{out}/f0d-{f}.txt"):
        if l.startswith("stackcensus"):
            for k, v in re.findall(r"(\w+)=(\d+)", l):
                t[k] = t.get(k, 0) + int(v)
    return t
m1, m2, u1, u2 = tot("main-r1"), tot("main-r2"), tot("mut-r1"), tot("mut-r2")
print(f"repliche main: {'r1==r2' if m1==m2 else 'DIVERGONO'} · mutante: {'r1==r2' if u1==u2 else 'DIVERGONO'}")
bad = 0 if (m1==m2 and u1==u2) else 5
N = 2_000_000; NM = 200_000
per = {k: v/N for k, v in sorted(m1.items()) if v}
print("conteggi/iter (main, N=2M): " + " ".join(f"{k}={x:.3f}" for k, x in per.items() if x >= 0.001))
ds_main = m1.get("DropS", 0); ds_mut = u1.get("DropS", 0)
base_at_nm = ds_main * NM / N
delta = ds_mut - base_at_nm
print(f"MUTANTE DropS: mut={ds_mut} attesa=base({base_at_nm:.0f})+{NM} Delta={delta:.0f} -> {'MORDE ESATTO' if abs(delta-NM) < 1 else 'FUORI ATTESA'}")
if abs(delta-NM) >= 1: bad = bad or 5
print("ESITO: rc=%d" % bad)
sys.exit(bad)
PY
prc=$?
echo "$prc" > "$RC"; rm -rf "$SRC/tgt"; exit "$prc"
} >> "$VERD" 2>&1
