#!/bin/bash
# s120-census-verdetto.sh — riduce i .census di s120-census-re1.sh alla riga di
# meccanismo (criterio p.2): alloc/iter netto per categoria, confronto con la
# colonna C-lite S-119 (pin s119). R=2: conteggi attesi identici, altrimenti
# si pubblica lo spread. Append al verdetto.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
OUT="$H/census-out"
M="$H/../wp97-harness/micro"
VERD="$H/s120-re1-verdetto.out"

python3 - "$OUT" "$M" > "$OUT/census-verdetto.txt" <<'PY'
import sys, re, os
out, micro = sys.argv[1], sys.argv[2]
# C-lite S-119 (s119-clite-verdetto.out), alloc/iter phpr sul PIN s119:
S119 = {"arith": 0.00, "prop": 0.00, "calls": 0.00, "str": 5.00, "arr": 4.016, "re": 17.00}
ZEND = {"arith": 0.00, "prop": 0.00, "calls": 0.00, "str": 2.00, "arr": 2.00, "re": 5.00}
def galloc(path):
    for line in open(path, errors="replace"):
        m = re.search(r"gacensus .*galloc_n=(\d+)", line)
        if m: return int(m.group(1))
    raise SystemExit(f"galloc_n assente in {path}")
def niter(cat):
    src = open(os.path.join(micro, cat + ".php")).read()
    m = re.search(r"\$i<(\d+)", src)
    return int(m.group(1))
emp = [galloc(os.path.join(out, f"phpr-empty-r{r}.census")) for r in (1, 2)]
print(f"empty galloc_n R=2: {emp[0]} / {emp[1]}")
for cat in ["arith", "prop", "calls", "str", "arr", "re"]:
    g = [galloc(os.path.join(out, f"phpr-{cat}-r{r}.census")) for r in (1, 2)]
    n = niter(cat)
    per = [(gi - ei) / n for gi, ei in zip(g, emp)]
    spread = abs(per[0] - per[1])
    tag = "" if spread < 0.005 else f"  SPREAD={spread:.3f} (R=2 NON identici)"
    print(f"{cat}: alloc/iter={per[0]:.2f}/{per[1]:.2f}  s119={S119[cat]:.2f}  zend={ZEND[cat]:.2f}{tag}")
re_now = (galloc(os.path.join(out, "phpr-re-r1.census")) - emp[0]) / niter("re")
verdict = "MECCANISMO CONFERMATO" if re_now <= 10.0 else "MECCANISMO MANCATO (atteso <=10,00)"
worse = []
for cat in ["arith", "prop", "calls", "str", "arr"]:
    per = (galloc(os.path.join(out, f"phpr-{cat}-r1.census")) - emp[0]) / niter(cat)
    if per > S119[cat] + 0.005: worse.append(f"{cat}:{per:.2f}>{S119[cat]:.2f}")
if worse: verdict += " · AUMENTI NON-BERSAGLIO: " + ",".join(worse)
print(f"CENSUS_RE1: re {S119['re']:.2f} -> {re_now:.2f} alloc/iter (zend 5,00) => {verdict}")
PY
rc=$?
cat "$OUT/census-verdetto.txt"
{
  echo ""
  echo "MECCANISMO census (s120-census-re1.sh + verdetto, identity: $(grep census_bin "$OUT/identity.txt")):"
  cat "$OUT/census-verdetto.txt"
} >> "$VERD"
exit "$rc"
