#!/bin/bash
# s123-giudice-v2.sh <TAG> <TARGET> — criterio s123-criterio-metro.md p.6-7:
# giudice a DUE lati SIMMETRICI sul bersaglio, guardie a solo-regressione.
# Legge FAIL-CLOSED metro-out/layout-bande-v2.txt (SOGLIA_LAYOUT) e il TSV
# di s123-ab-v2.sh (8 colonne, ord in coda). rc in metro-out/giudice-<TAG>.rc:
# 0=PROMOZIONE-ai-gate 1=REFUTAZIONE-CONFERMATA 2=NON-DISTINGUIBILE 3=INVALIDA.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
TAG="${1:?TAG}"; TARGET="${2:?TARGET}"
OUT="$H/metro-out"; mkdir -p "$OUT"
TSV="$OUT/$TAG-runs.tsv"
BANDE="$OUT/layout-bande-v2.txt"
VERD="$H/s123-$TAG-verdetto.out"
RC="$OUT/giudice-$TAG.rc"

ABRC="$OUT/$TAG-ab.rc"
[ -s "$ABRC" ] && [ "$(cat "$ABRC")" = 0 ] || { echo "giudice $TAG: A/B rc != 0 — STOP" | tee -a "$VERD"; echo 3 > "$RC"; exit 3; }
for f in "$TSV" "$BANDE"; do
  [ -s "$f" ] || { echo "giudice $TAG: input MANCANTE $f — STOP" | tee -a "$VERD"; echo 3 > "$RC"; exit 3; }
done

python3 - "$TSV" "$BANDE" "$TARGET" "$TAG" > "$OUT/giudice-$TAG.txt" <<'PY'
import sys
from statistics import median
BSTOR = {"arith": 0.80, "prop": 3.33, "calls": 0.50, "str": 7.50, "arr": 6.67, "re": 10.00}
ZAVORRA = {"str": 2.50}
FLOOR_PROMO = 4.00
target, tag = sys.argv[3], sys.argv[4]
SL = {}
for line in open(sys.argv[2]):
    k, c, v = line.split()
    if k == "SOGLIA_LAYOUT": SL[c] = float(v)
rows, NN = {}, {}
for line in open(sys.argv[1]):
    c, n, i, ta, tb, fa, fb, ord_ = line.split('\t')
    n, ta, tb, fa, fb = float(n), float(ta), float(tb), float(fa), float(fb)
    NN[c] = n
    rows.setdefault(c, []).append((int(i), (ta-fa)/n*1e9, (tb-fb)/n*1e9))
cats = [c for c in ["arith", "prop", "calls", "str", "arr", "re"] if c in rows]
if target not in rows or any(c not in SL for c in cats):
    print(f"INPUT INCOMPLETO (target o SOGLIA_LAYOUT mancante)"); print("RCV=3"); sys.exit()
verdict = None; guardie_sfondate = []
for c in cats:
    na = [r[1] for r in rows[c]]; nb = [r[2] for r in rows[c]]
    tA, tB = 1.3*min(na), 1.3*min(nb)
    fam = [(i, a, b) for i, a, b in rows[c] if a <= tA and b <= tB]
    for i, a, b in rows[c]:
        if not (a <= tA and b <= tB):
            print(f"{c}: ESCLUSA coppia{i} (A={a:.2f} B={b:.2f}; fam A<={tA:.2f} B<={tB:.2f})")
    if len(fam) < 5:
        print(f"{c}: fam={len(fam)}<5 — MISURA INVALIDA"); print("RCV=3"); sys.exit()
    As = [a for _, a, _ in fam]; Ds = [a-b for _, a, b in fam]
    dmed = round(median(Ds), 2); pos = sum(1 for d in Ds if round(d, 2) > 0)
    sa = round(max(As)-min(As), 2)
    quanto = 0.001/NN[c]*1e9
    zav = ZAVORRA.get(c, 0.0)
    if c == target:
        s_ref = round(max(SL[c], zav, 2*sa, 4*quanto), 2)
        s_pro = round(max(FLOOR_PROMO, zav, SL[c], 2*sa, 4*quanto), 2)
        if dmed <= -s_ref: verdict = ("REFUTAZIONE CONFERMATA", 1)
        elif dmed >= s_pro: verdict = ("PROMOZIONE-ai-gate", 0)
        else: verdict = ("NON-DISTINGUIBILE (né refutazione né promozione)", 2)
        print(f"{c} [BERSAGLIO]: fam={len(Ds)} D_med={dmed:+.2f} segni +{pos}/{len(Ds)} spread_A={sa:.2f} "
              f"4quanto={4*quanto:.3f} SL={SL[c]:.2f} soglia_ref=-{s_ref:.2f} soglia_promo=+{s_pro:.2f} -> {verdict[0]}")
    else:
        soglia = round(-max(2*sa, BSTOR[c], SL[c], 4*quanto), 2)
        sf = dmed < soglia
        if sf: guardie_sfondate.append(c)
        print(f"{c}: fam={len(Ds)} D_med={dmed:+.2f} segni +{pos}/{len(Ds)} spread_A={sa:.2f} "
              f"4quanto={4*quanto:.3f} SL={SL[c]:.2f} soglia_guardia={soglia:.2f} -> {'SFONDATA' if sf else 'tiene'} (tie: = -> tiene)")
if guardie_sfondate:
    print(f"guardie SFONDATE: {guardie_sfondate}")
label, rc = verdict
if rc == 0 and guardie_sfondate:
    label, rc = "bersaglio OLTRE soglia ma guardie SFONDATE — NIENTE promozione", 2
print(f"VERDETTO_S123_{tag} = {label}")
print(f"RCV={rc}")
PY
sed '/^RCV=/d' "$OUT/giudice-$TAG.txt"
R=$(awk -F= '/^RCV=/{print $2}' "$OUT/giudice-$TAG.txt")
echo "$R" > "$RC"
{
  echo ""
  echo "GIUDICE S-123 $TAG bersaglio=$TARGET (rc=$R da metro-out/giudice-$TAG.rc):"
  sed '/^RCV=/d' "$OUT/giudice-$TAG.txt"
} >> "$VERD"
exit "$R"
