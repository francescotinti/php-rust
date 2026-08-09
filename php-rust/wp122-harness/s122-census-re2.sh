#!/bin/bash
# s122-census-re2.sh — MECCANISMO L-RE2 (criterio s122-criterio-re2.md p.2):
# build census (HEAD + census-clite.patch wp119 + re2.patch) nel target
# SEPARATO; R=2 su empty + 6 categorie; alloc/iter netto vs baseline s119.
# Bersaglio: re 10,00→9,00 ESATTO; predizioni secondarie VERIFICATE e a
# verbale ANCHE SE MANCATE (az. rev. S-121 #3). SOLO CONTEGGI, zero tempo.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp122-harness"
H119="$REPO/wp119-harness"
MICRO="$REPO/wp97-harness/micro"
OUT="$H/census-out"; mkdir -p "$OUT"
CTD="/Volumes/Extreme Pro/Claude/phpr-census-target"
CENSUS_BIN="$CTD/release/phpr"
VERD="$H/s122-re2-verdetto.out"
fail(){ echo "$1" | tee -a "$VERD"; echo 1 > "$OUT/census.rc"; git -C "$REPO" checkout -- . 2>/dev/null; exit 1; }

cd "$REPO" || exit 2
git diff --quiet || fail "PRE: tree sporco — STOP"
git apply "$H119/census-clite.patch" || fail "census-clite.patch NON applica"
git apply "$H/re2.patch" || fail "re2.patch NON applica sopra il census"

CARGO_TARGET_DIR="$CTD" SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 \
  cargo build --release -p php-cli --features mem-census,zval-census \
  > "$OUT/build-census.log" 2>&1
brc=$?; echo "$brc" > "$OUT/build-census.rc"
[ "$brc" = 0 ] || fail "build census rc=$brc"

{
  echo "census_bin=$(shasum -a 256 "$CENSUS_BIN" | cut -c1-16)  # build strumentata, NON il pin"
  echo "head=$(git -C "$REPO" rev-parse --short HEAD)  patches=census-clite(wp119)+re2"
} > "$OUT/identity.txt"

for c in empty arith prop calls str arr re; do
  for r in 1 2; do
    f="$OUT/phpr-$c-r$r.census"; rm -f "$f"
    PHPR_MEM_CENSUS="$f" MIMALLOC_PURGE_DELAY=0 "$CENSUS_BIN" "$MICRO/$c.php" > /dev/null 2>"$OUT/phpr-$c-r$r.err"
  done
done
git checkout -- . || fail "ripristino tree fallito"

python3 - "$OUT" "$H119/clite-out" "$MICRO" > "$OUT/census-verdetto.txt" <<'PY'
import sys, re, os
out, old, micro = sys.argv[1], sys.argv[2], sys.argv[3]
def galloc(path):
    for line in open(path, errors="replace"):
        m = re.search(r"gacensus .*galloc_n=(\d+)", line)
        if m: return int(m.group(1))
    raise SystemExit(f"galloc_n assente in {path}")
def niter(cat):
    src = open(os.path.join(micro, cat + ".php")).read()
    return int(re.search(r"\$i<(\d+)", src).group(1))
def per_iter(base, cat, r):
    e = galloc(os.path.join(base, f"phpr-empty-r{r}.census"))
    g = galloc(os.path.join(base, f"phpr-{cat}-r{r}.census"))
    return (g - e) / niter(cat)
worse, pred = [], []
for cat in ["arith", "prop", "calls", "str", "arr", "re"]:
    now = [round(per_iter(out, cat, r), 2) for r in (1, 2)]
    old_v = round(per_iter(old, cat, 1), 2)
    tag = "" if now[0] == now[1] else f"  SPREAD R=2: {now[0]:.2f}/{now[1]:.2f}"
    print(f"{cat}: alloc/iter={now[0]:.2f} (s119={old_v:.2f}, N={niter(cat)}){tag}")
    if cat != "re":
        # predizione secondaria del criterio p.2: INVARIATA — esito a verbale anche se mancato
        pred.append(f"{cat}:{'OK' if now[0] == old_v else f'MANCATA {old_v:.2f}->{now[0]:.2f}'}")
        if now[0] > old_v: worse.append(f"{cat}:{now[0]:.2f}>{old_v:.2f}")
re_now = round(per_iter(out, "re", 1), 2)
re_old = round(per_iter(old, "re", 1), 2)
ok = (re_now == 9.00) and not worse
verdict = "MECCANISMO CONFERMATO (9,00 ESATTO)" if re_now == 9.00 else f"MECCANISMO MANCATO (atteso 9,00, misurato {re_now:.2f})"
if worse: verdict += " · AUMENTI NON-BERSAGLIO: " + ",".join(worse)
print(f"predizioni secondarie (invarianza): {' '.join(pred)}")
print(f"CENSUS_RE2: re {re_old:.2f} -> {re_now:.2f} alloc/iter => {verdict}")
print(f"RCV={0 if ok else 1}")
PY
sed '/^RCV=/d' "$OUT/census-verdetto.txt"
RC=$(awk -F= '/^RCV=/{print $2}' "$OUT/census-verdetto.txt")
{
  echo "MECCANISMO census L-RE2 (rc=$RC, identity: $(grep census_bin "$OUT/identity.txt")):"
  sed '/^RCV=/d' "$OUT/census-verdetto.txt"
} >> "$VERD"
echo "$RC" > "$OUT/census.rc"
exit "$RC"
