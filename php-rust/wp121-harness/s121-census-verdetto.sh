#!/bin/bash
# s121-census-verdetto.sh — riduce i .census di s121-census-st1.sh alla riga
# di meccanismo (criterio p.2): alloc/iter netto per categoria. BASELINE s119
# lette dai RAW di wp119-harness/clite-out con lo STESSO codice e lo STESSO
# denominatore (lezione verdetto v2 di S-120). Confronti a 2 decimali.
# R=2: conteggi attesi identici, altrimenti spread. Bersaglio: str 5,00→4,00
# ESATTO; nessuna non-bersaglio in AUMENTO (cali si REGISTRANO, non promuovono).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
OUT="$H/census-out"
OLD="$H/../wp119-harness/clite-out"
M="$H/../wp97-harness/micro"
VERD="$H/s121-st1-verdetto.out"

python3 - "$OUT" "$OLD" "$M" > "$OUT/census-verdetto.txt" <<'PY'
import sys, re, os
out, old, micro = sys.argv[1], sys.argv[2], sys.argv[3]
ZEND_STR = 2.00  # C-lite S-119, per-iter
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
emp = [galloc(os.path.join(out, f"phpr-empty-r{r}.census")) for r in (1, 2)]
print(f"empty galloc_n R=2: {emp[0]} / {emp[1]}")
worse, cali = [], []
for cat in ["arith", "prop", "calls", "str", "arr", "re"]:
    now = [round(per_iter(out, cat, r), 2) for r in (1, 2)]
    old_v = round(per_iter(old, cat, 1), 2)
    tag = "" if now[0] == now[1] else f"  SPREAD R=2: {now[0]:.2f}/{now[1]:.2f}"
    print(f"{cat}: alloc/iter={now[0]:.2f} (s119={old_v:.2f}, stesso denominatore N={niter(cat)}){tag}")
    if cat != "str":
        if now[0] > old_v: worse.append(f"{cat}:{now[0]:.2f}>{old_v:.2f}")
        elif now[0] < old_v: cali.append(f"{cat}:{old_v:.2f}->{now[0]:.2f}")
str_now = round(per_iter(out, "str", 1), 2)
str_old = round(per_iter(old, "str", 1), 2)
verdict = "MECCANISMO CONFERMATO (atteso 4,00 ESATTO)" if str_now == 4.00 else "MECCANISMO MANCATO (atteso 4,00 ESATTO)"
if worse: verdict += " · AUMENTI NON-BERSAGLIO: " + ",".join(worse)
if cali: verdict += " · cali registrati (non promuovono): " + ",".join(cali)
print(f"CENSUS_ST1: str {str_old:.2f} -> {str_now:.2f} alloc/iter (zend {ZEND_STR:.2f}) => {verdict}")
PY
rc=$?
cat "$OUT/census-verdetto.txt"
{
  echo ""
  echo "MECCANISMO census (s121-census-st1.sh + verdetto, identity: $(grep census_bin "$OUT/identity.txt")):"
  cat "$OUT/census-verdetto.txt"
} >> "$VERD"
exit "$rc"
