#!/bin/bash
# s122-st1-giudice.sh — criterio s122-criterio-layout.md p.5-7: ri-giudizio
# del full A/B L-ST1 (TSV di s122-ab-st1.sh) con le bande-LAYOUT lette
# FAIL-CLOSED da layout-out/layout-bande.txt. Verdetto a DUE lati sul
# bersaglio str; guardie a solo-regressione con banda-layout inclusa.
# rc in st1-out/giudice.rc: 0=PROMOZIONE-ai-gate 1=REFUTAZIONE-CONFERMATA
# 2=NON-DISTINGUIBILE-dal-layout 3=MISURA-INVALIDA/input-mancante.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp122-harness"
OUT="$H/st1-out"; mkdir -p "$OUT"
TSV="$OUT/st1-full-runs.tsv"
BANDE="$H/layout-out/layout-bande.txt"
VERD="$H/s122-st1-verdetto.out"

for f in "$TSV" "$BANDE"; do
  [ -s "$f" ] || { echo "giudice: input MANCANTE $f — STOP" | tee -a "$VERD"; echo 3 > "$OUT/giudice.rc"; exit 3; }
done

python3 - "$TSV" "$BANDE" > "$OUT/giudice-verdetto.txt" <<'PY'
import sys
from statistics import median
BANDA_V2 = {"arith": 0.80, "prop": 3.33, "calls": 0.50, "arr": 6.67, "re": 10.00}
ZAVORRA_STR = 2.50
FLOOR_PROMO = 4.00
BL = {}
for line in open(sys.argv[2]):
    _, c, v = line.split()
    BL[c] = float(v)
rows = {}; NN = {}
for line in open(sys.argv[1]):
    c, n, i, ta, tb, fa, fb = line.split('\t')
    n, ta, tb, fa, fb = float(n), float(ta), float(tb), float(fa), float(fb)
    NN[c] = n
    rows.setdefault(c, []).append((int(i), (ta-fa)/n*1e9, (tb-fb)/n*1e9))
missing = [c for c in ["arith","prop","calls","str","arr","re"] if c not in rows or c not in BL]
if missing:
    print(f"INPUT INCOMPLETO: {missing}"); print("RCV=3"); sys.exit()
verdict = None; guardie_sfondate = []
for c in ["arith", "prop", "calls", "str", "arr", "re"]:
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
    sa = round(max(As)-min(As), 2); quanto = round(0.01/NN[c]*1e9, 2)
    if c == "str":
        s_ref = round(max(BL[c], ZAVORRA_STR, 2*quanto), 2)
        s_pro = round(max(FLOOR_PROMO, ZAVORRA_STR, BL[c], 2*sa, 2*quanto), 2)
        if dmed <= -s_ref: verdict = ("REFUTAZIONE CONFERMATA", 1)
        elif dmed >= s_pro: verdict = ("PROMOZIONE-ai-gate", 0)
        else: verdict = ("NON-DISTINGUIBILE dal layout (refutazione S-121 DECADE a non-provata)", 2)
        print(f"str: fam={len(Ds)} D_med={dmed:+.2f} segni +{pos}/{len(Ds)} spread_A={sa:.2f} "
              f"quanto={quanto:.2f} BL={BL[c]:.2f} soglia_ref=-{s_ref:.2f} soglia_promo=+{s_pro:.2f} -> {verdict[0]}")
    else:
        soglia = round(-max(2*sa, BANDA_V2[c], BL[c], 2*quanto), 2)
        sf = dmed < soglia
        if sf: guardie_sfondate.append(c)
        print(f"{c}: fam={len(Ds)} D_med={dmed:+.2f} segni +{pos}/{len(Ds)} spread_A={sa:.2f} "
              f"quanto={quanto:.2f} BL={BL[c]:.2f} soglia_guardia={soglia:.2f} -> {'SFONDATA' if sf else 'tiene'} (tie: = -> tiene)")
if guardie_sfondate:
    print(f"guardie SFONDATE (informativo per p.6; bloccanti SOLO sul ramo promozione): {guardie_sfondate}")
label, rc = verdict
if rc == 0 and guardie_sfondate:
    label, rc = "bersaglio OLTRE soglia ma guardie SFONDATE — NIENTE promozione", 2
print(f"VERDETTO_S122_L-ST1 = {label}")
print(f"RCV={rc}")
PY
sed '/^RCV=/d' "$OUT/giudice-verdetto.txt"
RC=$(awk -F= '/^RCV=/{print $2}' "$OUT/giudice-verdetto.txt")
echo "$RC" > "$OUT/giudice.rc"
{
  echo ""
  echo "GIUDICE S-122 L-ST1 (rc=$RC da st1-out/giudice.rc, criterio p.6-7):"
  sed '/^RCV=/d' "$OUT/giudice-verdetto.txt"
} >> "$VERD"
exit "$RC"
