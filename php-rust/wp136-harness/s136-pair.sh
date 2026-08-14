#!/bin/bash
# s136-pair.sh <tentativo> — S-136 p.1: coppia full/media WP sul pin s135 (OBBLIGO pin nuovo),
# criterio s132-criterio-pair.md RIUSATO INVARIATO (per-config, firma per gamba;
# gia committato) + s136-criterio-pair.md (confronto formale col rif 1,769 su
# banda off 0,041). COPIA DICHIARATA di wp134-harness/s134-pair.sh (collaudo:
# copia-gate + manifest s136-pair-copia.diff) coi SOLI
# adattamenti: pin s135 (phpr+server); N>=3 ON-ONLY (az.rev. S-134 #4: TRE
# gambe per configurazione, off1->on1->off2->on2->off3->on3); giudice esteso
# col confronto formale al riferimento 1,769 (banda = variabilita' off 0,041).
# Tutto il resto INVARIATO: warm-up media-only MAI giudicata, gambe
# intercalate (pair109 INVARIATA), quiescenza gate
# SEPARATO per gamba, gate ictx/s a mediana PER MOTORE.
# rc autoritativo = SOLO pair-out/pair136-<tentativo>.done scritto QUI.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp136-harness"
H109="$REPO/wp109-harness"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
T="${1:?tentativo (es. t1)}"
OUT="$H/pair-out"; mkdir -p "$OUT"
VERD="$H/s136-pair-verdetto-$T.out"
if [ -e "$VERD" ]; then
  echo "verdetto $VERD ESISTE — tentativo nuovo = file nuovo (az.rev. S-131 #5)" >&2
  exit 7
fi
DONE="$OUT/pair136-$T.done"
WPDEV="/Users/francescotinti/Claude/wpdev"
ORACLE=/opt/homebrew/opt/php/bin/php
PHPR="$HOME/Claude/php-rust-output/release/phpr"
GUARD="/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
step(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress-$T.txt"; }
: > "$OUT/progress-$T.txt"
rm -f "$DONE"

PIN_ATTESO="6518a1e14a266d52"
SRV_ATTESO="e2efdf15bf74d8c9"
PIN="$(shasum -a 256 "$PHPR" | cut -c1-16)"
SRV="$(shasum -a 256 "$HOME/Claude/php-rust-output/release/php-server" | cut -c1-16)"
if [ "$PIN" != "$PIN_ATTESO" ] || [ "$SRV" != "$SRV_ATTESO" ]; then
  step "ABORT pin: phpr=$PIN (atteso $PIN_ATTESO) server=$SRV (atteso $SRV_ATTESO)"
  echo "rc=9 pin-mismatch" > "$DONE"; exit 9
fi

quiesce_gate(){ # $1=etichetta — gate SEPARATO, rc nel suo file (mai nel comando del lancio)
  # EMENDA DICHIARATA (t3, dopo rc=8 t1 E t2 allo STESSO confine leg2-off):
  # attesa di ASSESTAMENTO per mediaanalysisd (digerisce il churn immagini
  # delle gambe media, flare ~25' dopo l'avvio del ciclo) PRIMA del gate —
  # il gate resta AUTORITATIVO e invariato (stesso modello del quiet_wait
  # del runner CI). Max ~22 min, poi si lascia parlare il gate.
  local w=0 c1 c2
  while :; do
    c1=$(ps -Ao %cpu,comm | awk 'index($0,"mediaanalysisd"){s+=$1} END{printf "%.1f", s+0}')
    /bin/sleep 5
    c2=$(ps -Ao %cpu,comm | awk 'index($0,"mediaanalysisd"){s+=$1} END{printf "%.1f", s+0}')
    if awk -v a="$c1" -v b="$c2" 'BEGIN{exit !(a<5.0 && b<5.0)}'; then break; fi
    w=$((w+1)); step "assestamento $1: mediaanalysisd $c1/$c2 (attesa $((w*15))s)"
    [ "$w" -gt 90 ] && break
    /bin/sleep 10
  done
  "$QUIESCE" "$OUT/quiesce-$T-$1.rc" > "$OUT/quiesce-$T-$1.log" 2>&1
  local q=$?
  step "quiescenza $1 rc=$q (file: pair-out/quiesce-$T-$1.rc)"
  if [ "$q" != 0 ]; then
    echo "rc=8 quiescenza-fail su $1" > "$DONE"; exit 8
  fi
}

reset_env(){
  mysql -h 127.0.0.1 -u root -e "DROP DATABASE IF EXISTS wptests; CREATE DATABASE wptests; GRANT ALL ON wptests.* TO 'wp'@'%';" || exit 3
  find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
}

# ---- WARM-UP dichiarato (criterio p.2): media-only bilaterale, MAI giudicata
quiesce_gate warmup
mkdir -p "$OUT/warmup-$T"
"$GUARD" backup-wipe >> "$OUT/progress-$T.txt" 2>&1 || { echo "rc=4 guard-backup" > "$DONE"; exit 4; }
cd "$WPDEV" || { echo "rc=2 wpdev" > "$DONE"; exit 2; }
step "warmup media oracle"
reset_env
MIMALLOC_PURGE_DELAY=0 /usr/bin/time -l "$ORACLE" vendor/bin/phpunit --group media \
  > "$OUT/warmup-$T/media-oracle.txt" 2> "$OUT/warmup-$T/media-oracle.time"
step "warmup media oracle rc=$?"
step "warmup media phpr (off)"
reset_env
MIMALLOC_PURGE_DELAY=0 PHPR_REG_LOWER=0 /usr/bin/time -l "$PHPR" vendor/bin/phpunit --group media \
  > "$OUT/warmup-$T/media-phpr.txt" 2> "$OUT/warmup-$T/media-phpr.time"
step "warmup media phpr rc=$?"
find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
"$GUARD" restore >> "$OUT/progress-$T.txt" 2>&1
step "warmup DONE (mai giudicata)"

# ---- 6 gambe intercalate off1->on1->off2->on2->off3->on3 (pair109 INVARIATA)
RC=0
for leg in 1 2 3; do
  for mode in off on; do
    quiesce_gate "leg$leg-$mode"
    step "gamba $mode-$leg START"
    "$H109/pair109.sh" "$mode" >> "$OUT/progress-$T.txt" 2>&1
    lrc=$?
    step "gamba $mode-$leg rc=$lrc"
    [ "$lrc" = 0 ] || RC=1
    rm -rf "$OUT/$T-leg$leg-$mode"
    mv "$H109/pair-out-$mode" "$OUT/$T-leg$leg-$mode"
    mv "$H109/pair109-ratios-$mode.out" "$OUT/$T-leg$leg-$mode/ratios.out" 2>/dev/null
  done
done

# ---- giudice meccanico dai .time: header con rc quiescenza, ictx/s mediana PER
# MOTORE, az.rev. #3: riferimento PER CONFIGURAZIONE (ON-ONLY canonico) + firma
# per gamba (ictx% della mediana del motore, rank CPU oracle).
python3 - "$OUT" "$T" > "$VERD" <<'PY'
import re, sys, statistics, os
out, T = sys.argv[1], sys.argv[2]
def t(f):
    d = {}
    for l in open(f, errors="replace"):
        m = re.search(r"([\d.]+)\s+real", l);  d.update(real=float(m.group(1))) if m else None
        m = re.search(r"([\d.]+)\s+user", l);  d.update(user=float(m.group(1))) if m else None
        m = re.search(r"([\d.]+)\s+sys", l);   d.update(sys=float(m.group(1))) if m else None
        m = re.search(r"^\s*(\d+)\s+involuntary context switches", l); d.update(ictx=int(m.group(1))) if m else None
        m = re.search(r"^\s*(\d+)\s+peak memory footprint", l); d.update(pf=int(m.group(1))) if m else None
    return d
legs = ["leg1-off", "leg1-on", "leg2-off", "leg2-on", "leg3-off", "leg3-on"]
print(f"== s136 coppia WP full+media sul pin s135 tentativo={T} (criterio s132-criterio-pair.md RIUSATO + s136-criterio-pair.md) ==")
print(f"grade=VERDICT  # derivazione meccanica dai .time; rc autoritativo = pair-out/pair136-{T}.done")
print("# full: cpu=user+sys (etichettato); media: CANONICA=user-only + companion (az.rev. S-128 #3)")
print("# az.rev. S-131 #3: riferimento PER CONFIGURAZIONE, ON-ONLY = canonico; firma per gamba")
for tag in ["warmup"] + legs:
    p = f"{out}/quiesce-{T}-{tag}.rc"
    v = open(p).read().strip() if os.path.exists(p) else "MANCANTE"
    print(f"quiescenza {tag}: rc={v} (file: pair-out/quiesce-{T}-{tag}.rc)")
O, P, OM, PM = {}, {}, {}, {}
for leg in legs:
    d = f"{out}/{T}-{leg}"
    O[leg] = t(f"{d}/full-oracle.time"); P[leg] = t(f"{d}/full-phpr.time")
    OM[leg] = t(f"{d}/media-oracle.time"); PM[leg] = t(f"{d}/media-phpr.time")
ictx_s = {}
for leg in legs:
    for side, d in (("oracle", O[leg]), ("phpr", P[leg])):
        ictx_s[f"{leg}-{side}"] = d["ictx"] / d["real"] if d.get("real") else -1.0
med = {side: statistics.median([ictx_s[f"{l}-{side}"] for l in legs if ictx_s[f"{l}-{side}"] >= 0])
       for side in ("oracle", "phpr")}
ocpu_rank = sorted(legs, key=lambda l: O[l]["user"] + O[l]["sys"])
flag_legs = set()
for leg in legs:
    fo, fp = ictx_s[f"{leg}-oracle"], ictx_s[f"{leg}-phpr"]
    flagged = (med["oracle"] > 0 and fo > 1.5 * med["oracle"]) or (med["phpr"] > 0 and fp > 1.5 * med["phpr"])
    if flagged: flag_legs.add(leg)
    po = 100.0 * fo / med["oracle"] if med["oracle"] > 0 else -1
    pp = 100.0 * fp / med["phpr"] if med["phpr"] > 0 else -1
    rk = ocpu_rank.index(leg) + 1
    print(f"{leg}: ictx/s oracle={fo:.0f} phpr={fp:.0f}" + ("  SEGNALATA(>1,5x med MOTORE)" if flagged else "  contesa ok"))
    print(f"  firma {leg}: ictx%med oracle={po:.0f}% phpr={pp:.0f}% rank_cpu_oracle={rk}/4 (1=piu' veloce)")
print(f"ictx_s_mediana_oracle={med['oracle']:.0f} ictx_s_mediana_phpr={med['phpr']:.0f}  # PER MOTORE")
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
    print(f"full_intervallo_PULITO_misto={min(pcs.values())/max(ocs.values()):.3f}-{max(pcs.values())/min(ocs.values()):.3f}  # companion (miscela config)")
    props = [pcs[l]/ocs[l] for l in clean]
    print(f"coppie_proprie_PULITE_miste={min(props):.3f}-{max(props):.3f} (N={len(props)})")
    pu = {l: P[l]["user"]/O[l]["user"] for l in clean}
    print(f"coppie_proprie_user_only_miste={min(pu.values()):.3f}-{max(pu.values()):.3f}")
# az.rev. S-131 #3: PER CONFIGURAZIONE — ON-ONLY = riferimento CANONICO
for cfg in ("on", "off"):
    sub = [l for l in clean if l.endswith(f"-{cfg}")]
    if not sub:
        print(f"config {cfg}: NESSUNA gamba pulita"); continue
    so = {l: O[l]["user"] + O[l]["sys"] for l in sub}
    sp = {l: P[l]["user"] + P[l]["sys"] for l in sub}
    props = [sp[l]/so[l] for l in sub]
    canon = " CANONICO" if cfg == "on" else ""
    print(f"full_{cfg}_only_coppie_proprie={min(props):.3f}-{max(props):.3f} (N={len(props)}){canon}")
    print(f"full_{cfg}_only_intervallo={min(sp.values())/max(so.values()):.3f}-{max(sp.values())/min(so.values()):.3f}")
    if cfg == "on":
        REF, BANDA = 1.769, 0.041  # s136-criterio-pair.md: rif S-134 on-only, banda = variabilita' off s134
        dentro = all(abs(r - REF) <= BANDA for r in props)
        print(f"confronto_formale_rif: on-only vs {REF} banda {BANDA} -> " +
              ("COMPATIBILE (tutte le coppie proprie dentro)" if dentro else
               "FUORI BANDA: " + " ".join(f"{r:.3f}(d={r-REF:+.3f})" for r in props if abs(r - REF) > BANDA)))
for leg in legs:
    mu = PM[leg]["user"] / OM[leg]["user"] if OM[leg].get("user") else -1
    mc = (PM[leg]["user"]+PM[leg]["sys"]) / (OM[leg]["user"]+OM[leg]["sys"])
    star = " (SEGNALATA)" if leg in flag_legs else ""
    print(f"media_{leg}: user_only_CANONICA={mu:.3f} user+sys_companion={mc:.3f}{star}")
# parita' per NOME (criterio p.7): media diff vuoto; full diff == SOLO wp_is_stream #2
exp = "> Tests_Functions::test_wp_is_stream with data set #2 ('ftp://example.com', true)"
for leg in legs:
    md = open(f"{out}/{T}-{leg}/media.failnames.diff", errors="replace").read().strip()
    fd = [l for l in open(f"{out}/{T}-{leg}/full.failnames.diff", errors="replace").read().splitlines()
          if l[:1] in "<>"]
    m_ok = "OK(vuoto)" if md == "" else "DIVERSO!"
    f_ok = "OK(solo wp_is_stream #2)" if fd == [exp] else f"DIVERSO! {fd}"
    print(f"parita_{leg}: media={m_ok} full={f_ok}")
PY
echo "rc=$RC" > "$DONE"
step "DONE rc=$RC (fonte rc: $DONE)"
exit "$RC"
