#!/bin/bash
# s129-pair-legoff.sh — az.rev. S-128 #1+#2: UNA gamba off pulita (pair109
# ricetta INVARIATA) + gate contesa ictx/s PER GAMBA su tutte le gambe (le 4
# S-128 archiviate + leg3-off nuova); riferimento ripubblicato dalle gambe
# PULITE, medie ETICHETTATE (az.rev. #3: canonica = user-only).
# rc autoritativo = SOLO pair-out/legoff.done scritto QUI.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp129-harness"
H109="$REPO/wp109-harness"
H128="$REPO/wp128-harness"
OUT="$H/pair-out"; mkdir -p "$OUT"
VERD="$H/s129-pair-legoff-verdetto.out"
rm -f "$OUT/legoff.done"
PIN="$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)"
[ "$PIN" = "ccb63dcaf565cffc" ] || { echo "rc=9 pin-mismatch" > "$OUT/legoff.done"; exit 9; }

"$H109/pair109.sh" off > "$OUT/leg3-progress.txt" 2>&1
lrc=$?
rm -rf "$OUT/leg3-off"
mv "$H109/pair-out-off" "$OUT/leg3-off"
mv "$H109/pair109-ratios-off.out" "$OUT/leg3-off/ratios.out" 2>/dev/null
[ "$lrc" = 0 ] || { echo "rc=$lrc pair109-off fallita" > "$OUT/legoff.done"; exit "$lrc"; }

python3 - "$H128/pair-out" "$OUT" > "$VERD" <<'PY'
import re, sys, statistics
h128, out = sys.argv[1], sys.argv[2]
def t(f):
    d = {}
    for l in open(f, errors="replace"):
        m = re.search(r"([\d.]+)\s+real", l);  d.update(real=float(m.group(1))) if m else None
        m = re.search(r"([\d.]+)\s+user", l);  d.update(user=float(m.group(1))) if m else None
        m = re.search(r"([\d.]+)\s+sys", l);   d.update(sys=float(m.group(1))) if m else None
        m = re.search(r"^\s*(\d+)\s+involuntary context switches", l); d.update(ictx=int(m.group(1))) if m else None
        m = re.search(r"^\s*(\d+)\s+peak memory footprint", l); d.update(pf=int(m.group(1))) if m else None
    return d
legs = {"leg1-off": h128, "leg1-on": h128, "leg2-off": h128, "leg2-on": h128, "leg3-off": out}
O, P, OM, PM = {}, {}, {}, {}
for leg, base in legs.items():
    d = f"{base}/{leg}"
    O[leg] = t(f"{d}/full-oracle.time"); P[leg] = t(f"{d}/full-phpr.time")
    OM[leg] = t(f"{d}/media-oracle.time"); PM[leg] = t(f"{d}/media-phpr.time")
print("== s129 gamba off pulita + gate ictx/s per gamba (az.rev. S-128 #1-#3) ==")
print("grade=VERDICT  # derivazione meccanica dai .time; rc autoritativo = pair-out/legoff.done")
print("# full: cpu=user+sys (etichettato); media: CANONICA=user-only (az.rev. #3)")
ictx_s = {}
for leg in legs:
    for side, d in (("oracle", O[leg]), ("phpr", P[leg])):
        k = f"{leg}-{side}"
        ictx_s[k] = d["ictx"] / d["real"] if d.get("real") else -1.0
med = statistics.median([v for v in ictx_s.values() if v >= 0])
flag_legs = set()
for leg in legs:
    fo, fp = ictx_s[f"{leg}-oracle"], ictx_s[f"{leg}-phpr"]
    flagged = med > 0 and (fo > 1.5 * med or fp > 1.5 * med)
    if flagged: flag_legs.add(leg)
    print(f"{leg}: ictx/s oracle={fo:.0f} phpr={fp:.0f}" + ("  SEGNALATA(>1,5x med)" if flagged else "  contesa ok"))
print(f"ictx_s_mediana={med:.0f}")
clean = [l for l in legs if l not in flag_legs]
print(f"gambe pulite: {clean}")
for leg in legs:
    oc = O[leg]["user"] + O[leg]["sys"]; pc = P[leg]["user"] + P[leg]["sys"]
    print(f"{leg}: full_oracle_cpu={oc:.2f} full_phpr_cpu={pc:.2f} proprio={pc/oc:.3f} peak_phpr_MiB={P[leg].get('pf',0)/1048576:.1f}")
ocs = {l: O[l]["user"] + O[l]["sys"] for l in clean}
pcs = {l: P[l]["user"] + P[l]["sys"] for l in clean}
print("matrice full (gambe PULITE) phpr(riga)/oracle(colonna):")
for pl in clean:
    print("  " + "  ".join(f"{pl}/{ol}={pcs[pl]/ocs[ol]:.3f}" for ol in clean))
if clean:
    print(f"full_intervallo_PULITO={min(pcs.values())/max(ocs.values()):.3f}-{max(pcs.values())/min(ocs.values()):.3f}")
    props = [pcs[l]/ocs[l] for l in clean]
    print(f"coppie_proprie_PULITE={min(props):.3f}-{max(props):.3f} (N={len(props)})")
for leg in legs:
    mu = PM[leg]["user"] / OM[leg]["user"] if OM[leg].get("user") else -1
    mc = (PM[leg]["user"]+PM[leg]["sys"]) / (OM[leg]["user"]+OM[leg]["sys"])
    star = " (SEGNALATA)" if leg in flag_legs else ""
    print(f"media_{leg}: user_only_CANONICA={mu:.3f} user+sys_companion={mc:.3f}{star}")
PY
echo "rc=0" > "$OUT/legoff.done"
exit 0
