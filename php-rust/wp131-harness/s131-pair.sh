#!/bin/bash
# s131-pair.sh — S-131 p.1: rimisura full/media WP sul pin s130, criterio
# s131-criterio-pair.md (committato PRIMA di questo codice, commit distinti).
# WARM-UP media-only bilaterale (MAI giudicata) + 4 gambe INTERCALATE
# off1→on1→off2→on2 (pair109 INVARIATA); quiescenza gate SEPARATO per gamba
# (rc per-gamba in file propri, citati nell'header del verdetto — az.rev.
# S-130 #3); gate ictx/s a mediana PER MOTORE (addendum rev. S-129).
# rc autoritativo = SOLO pair-out/pair131.done scritto QUI.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp131-harness"
H109="$REPO/wp109-harness"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
OUT="$H/pair-out"; mkdir -p "$OUT"
VERD="$H/s131-pair-verdetto.out"
WPDEV="/Users/francescotinti/Claude/wpdev"
ORACLE=/opt/homebrew/opt/php/bin/php
PHPR="$HOME/Claude/php-rust-output/release/phpr"
GUARD="/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
step(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
: > "$OUT/progress.txt"
rm -f "$OUT/pair131.done"

PIN_ATTESO="0fdf1c49b16c24ba"
SRV_ATTESO="7fb7906988bb7292"
PIN="$(shasum -a 256 "$PHPR" | cut -c1-16)"
SRV="$(shasum -a 256 "$HOME/Claude/php-rust-output/release/php-server" | cut -c1-16)"
if [ "$PIN" != "$PIN_ATTESO" ] || [ "$SRV" != "$SRV_ATTESO" ]; then
  step "ABORT pin: phpr=$PIN (atteso $PIN_ATTESO) server=$SRV (atteso $SRV_ATTESO)"
  echo "rc=9 pin-mismatch" > "$OUT/pair131.done"; exit 9
fi

quiesce_gate(){ # $1=etichetta — gate SEPARATO, rc nel suo file (mai nel comando del lancio)
  "$QUIESCE" "$OUT/quiesce-$1.rc" > "$OUT/quiesce-$1.log" 2>&1
  local q=$?
  step "quiescenza $1 rc=$q (file: pair-out/quiesce-$1.rc)"
  if [ "$q" != 0 ]; then
    echo "rc=8 quiescenza-fail su $1" > "$OUT/pair131.done"; exit 8
  fi
}

reset_env(){
  mysql -h 127.0.0.1 -u root -e "DROP DATABASE IF EXISTS wptests; CREATE DATABASE wptests; GRANT ALL ON wptests.* TO 'wp'@'%';" || exit 3
  find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
}

# ---- WARM-UP dichiarato (criterio p.2): media-only bilaterale, MAI giudicata
quiesce_gate warmup
mkdir -p "$OUT/warmup"
"$GUARD" backup-wipe >> "$OUT/progress.txt" 2>&1 || { echo "rc=4 guard-backup" > "$OUT/pair131.done"; exit 4; }
cd "$WPDEV" || { echo "rc=2 wpdev" > "$OUT/pair131.done"; exit 2; }
step "warmup media oracle"
reset_env
MIMALLOC_PURGE_DELAY=0 /usr/bin/time -l "$ORACLE" vendor/bin/phpunit --group media \
  > "$OUT/warmup/media-oracle.txt" 2> "$OUT/warmup/media-oracle.time"
step "warmup media oracle rc=$?"
step "warmup media phpr (off)"
reset_env
MIMALLOC_PURGE_DELAY=0 PHPR_REG_LOWER=0 /usr/bin/time -l "$PHPR" vendor/bin/phpunit --group media \
  > "$OUT/warmup/media-phpr.txt" 2> "$OUT/warmup/media-phpr.time"
step "warmup media phpr rc=$?"
find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
"$GUARD" restore >> "$OUT/progress.txt" 2>&1
step "warmup DONE (mai giudicata)"

# ---- 4 gambe intercalate off1→on1→off2→on2 (pair109 INVARIATA)
RC=0
for leg in 1 2; do
  for mode in off on; do
    quiesce_gate "leg$leg-$mode"
    step "gamba $mode-$leg START"
    "$H109/pair109.sh" "$mode" >> "$OUT/progress.txt" 2>&1
    lrc=$?
    step "gamba $mode-$leg rc=$lrc"
    [ "$lrc" = 0 ] || RC=1
    rm -rf "$OUT/leg$leg-$mode"
    mv "$H109/pair-out-$mode" "$OUT/leg$leg-$mode"
    mv "$H109/pair109-ratios-$mode.out" "$OUT/leg$leg-$mode/ratios.out" 2>/dev/null
  done
done

# ---- giudice meccanico dai .time: header con rc quiescenza, ictx/s mediana PER MOTORE
python3 - "$OUT" > "$VERD" <<'PY'
import re, sys, statistics, os
out = sys.argv[1]
def t(f):
    d = {}
    for l in open(f, errors="replace"):
        m = re.search(r"([\d.]+)\s+real", l);  d.update(real=float(m.group(1))) if m else None
        m = re.search(r"([\d.]+)\s+user", l);  d.update(user=float(m.group(1))) if m else None
        m = re.search(r"([\d.]+)\s+sys", l);   d.update(sys=float(m.group(1))) if m else None
        m = re.search(r"^\s*(\d+)\s+involuntary context switches", l); d.update(ictx=int(m.group(1))) if m else None
        m = re.search(r"^\s*(\d+)\s+peak memory footprint", l); d.update(pf=int(m.group(1))) if m else None
    return d
legs = ["leg1-off", "leg1-on", "leg2-off", "leg2-on"]
print("== s131 coppia WP full+media sul pin s130 (criterio s131-criterio-pair.md) ==")
print("grade=VERDICT  # derivazione meccanica dai .time; rc autoritativo = pair-out/pair131.done")
print("# full: cpu=user+sys (etichettato); media: CANONICA=user-only + companion (az.rev. S-128 #3)")
for tag in ["warmup"] + legs:
    p = f"{out}/quiesce-{tag}.rc"
    v = open(p).read().strip() if os.path.exists(p) else "MANCANTE"
    print(f"quiescenza {tag}: rc={v} (file: pair-out/quiesce-{tag}.rc)")
O, P, OM, PM = {}, {}, {}, {}
for leg in legs:
    d = f"{out}/{leg}"
    O[leg] = t(f"{d}/full-oracle.time"); P[leg] = t(f"{d}/full-phpr.time")
    OM[leg] = t(f"{d}/media-oracle.time"); PM[leg] = t(f"{d}/media-phpr.time")
ictx_s = {}
for leg in legs:
    for side, d in (("oracle", O[leg]), ("phpr", P[leg])):
        ictx_s[f"{leg}-{side}"] = d["ictx"] / d["real"] if d.get("real") else -1.0
med = {side: statistics.median([ictx_s[f"{l}-{side}"] for l in legs if ictx_s[f"{l}-{side}"] >= 0])
       for side in ("oracle", "phpr")}
flag_legs = set()
for leg in legs:
    fo, fp = ictx_s[f"{leg}-oracle"], ictx_s[f"{leg}-phpr"]
    flagged = (med["oracle"] > 0 and fo > 1.5 * med["oracle"]) or (med["phpr"] > 0 and fp > 1.5 * med["phpr"])
    if flagged: flag_legs.add(leg)
    print(f"{leg}: ictx/s oracle={fo:.0f} phpr={fp:.0f}" + ("  SEGNALATA(>1,5x med MOTORE)" if flagged else "  contesa ok"))
print(f"ictx_s_mediana_oracle={med['oracle']:.0f} ictx_s_mediana_phpr={med['phpr']:.0f}  # PER MOTORE (addendum rev. S-129)")
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
    pu = {l: P[l]["user"]/O[l]["user"] for l in clean}
    print(f"coppie_proprie_user_only={min(pu.values()):.3f}-{max(pu.values()):.3f}")
for leg in legs:
    mu = PM[leg]["user"] / OM[leg]["user"] if OM[leg].get("user") else -1
    mc = (PM[leg]["user"]+PM[leg]["sys"]) / (OM[leg]["user"]+OM[leg]["sys"])
    star = " (SEGNALATA)" if leg in flag_legs else ""
    print(f"media_{leg}: user_only_CANONICA={mu:.3f} user+sys_companion={mc:.3f}{star}")
# parità per NOME (criterio p.7): media diff vuoto; full diff == SOLO wp_is_stream #2
exp = "> Tests_Functions::test_wp_is_stream with data set #2 ('ftp://example.com', true)"
for leg in legs:
    md = open(f"{out}/{leg}/media.failnames.diff", errors="replace").read().strip()
    fd = [l for l in open(f"{out}/{leg}/full.failnames.diff", errors="replace").read().splitlines()
          if l[:1] in "<>"]
    m_ok = "OK(vuoto)" if md == "" else "DIVERSO!"
    f_ok = "OK(solo wp_is_stream #2)" if fd == [exp] else f"DIVERSO! {fd}"
    print(f"parita_{leg}: media={m_ok} full={f_ok}")
PY
echo "rc=$RC" > "$OUT/pair131.done"
step "DONE rc=$RC (fonte rc: $OUT/pair131.done)"
exit "$RC"
