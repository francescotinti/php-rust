#!/bin/bash
# s154-pair.sh <tentativo> — S-154 p.1: coppia full/media WP @ pin s153 (DOVUTA: pin nuovo)
# criteri s132/s146/s148 EREDITATI INVARIATI + s154-criterio-pair-t5.md
# (GIUDIZIO CANONICO a MEDIANA per finestra vs t1=1,799 t2=1,738 t3=1,789 t4=1,781 —
# az.rev.3 S-149; banda-unione degradata a companion descrittivo).
# COPIA DICHIARATA di wp150-harness/s150-pair.sh (collaudo: copia-gate +
# manifest s154-pair-copia.diff) coi SOLI adattamenti: nomi s154/pair154/
# wp154-harness; pin s153 (phpr 8370c257 + server f030c6fc); header del
# giudice; mediane storiche con t4=1,781 (banda giudizio INVARIATA [1,738;1,799]).
# Le emende S-136 (assestamento a STREAK + retry gate x3) EREDITATE INVARIATE.
# Tutto il resto INVARIATO: warm-up media-only MAI giudicata, quiescenza gate
# SEPARATO per gamba, gate ictx/s a mediana PER MOTORE (pair109 INVARIATA).
# rc autoritativo = SOLO pair-out/pair148-<tentativo>.done scritto QUI.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp154-harness"
H109="$REPO/wp109-harness"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
T="${1:?tentativo (es. t2)}"
OUT="$H/pair-out"; mkdir -p "$OUT"
VERD="$H/s154-pair-verdetto-$T.out"
if [ -e "$VERD" ]; then
  echo "verdetto $VERD ESISTE — tentativo nuovo = file nuovo (az.rev. S-131 #5)" >&2
  exit 7
fi
DONE="$OUT/pair154-$T.done"
WPDEV="/Users/francescotinti/Claude/wpdev"
ORACLE=/opt/homebrew/opt/php/bin/php
PHPR="$HOME/Claude/php-rust-output/release/phpr"
GUARD="/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
step(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress-$T.txt"; }
: > "$OUT/progress-$T.txt"
rm -f "$DONE"

PIN_ATTESO="8370c257ae70cc8e"
SRV_ATTESO="f030c6fcbddfab96"
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
  # del runner CI).
  # EMENDA 2 (t4, dopo rc=8 t3 su leg3-off: il daemon OSCILLA 4→23% in 30 s
  # e una valle breve faceva partire il gate sul picco): (a) l'assestamento
  # richiede una STREAK di 4 campioni consecutivi <5% a passo 5 s (~20 s di
  # quiete continua, la scala dei due campioni del gate); (b) un gate che
  # morde a un confine si RITENTA fino a 3 volte (ri-assestando) prima di
  # abortire — il gate in sé resta invariato; si smette solo di buttare le
  # gambe valide già misurate per UN flare tra le gambe.
  local attempt q
  for attempt in 1 2 3; do
    local w=0 streak=0 c
    while :; do
      c=$(ps -Ao %cpu,comm | awk 'index($0,"mediaanalysisd"){s+=$1} END{printf "%.1f", s+0}')
      if awk -v a="$c" 'BEGIN{exit !(a<5.0)}'; then
        streak=$((streak+1))
        [ "$streak" -ge 4 ] && break
      else
        [ "$streak" -gt 0 ] && step "assestamento $1 (att.$attempt): streak rotta a $streak da $c%"
        streak=0
        w=$((w+1)); step "assestamento $1 (att.$attempt): mediaanalysisd $c% (attesa ~$((w*15))s)"
        [ "$w" -gt 90 ] && break
        /bin/sleep 10
      fi
      /bin/sleep 5
    done
    "$QUIESCE" "$OUT/quiesce-$T-$1.rc" > "$OUT/quiesce-$T-$1.log" 2>&1
    q=$?
    step "quiescenza $1 rc=$q tentativo-gate=$attempt (file: pair-out/quiesce-$T-$1.rc)"
    [ "$q" = 0 ] && return 0
  done
  echo "rc=8 quiescenza-fail su $1 (3 tentativi)" > "$DONE"; exit 8
}

reset_env(){
  mysql -h 127.0.0.1 -u root -e "DROP DATABASE IF EXISTS wptests; CREATE DATABASE wptests; GRANT ALL ON wptests.* TO 'wp'@'%';" || exit 3
  find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
}

warmup_media(){ # $1=etichetta dir (warmup|rewarmup) — bilaterale, MAI giudicata
  mkdir -p "$OUT/$1-$T"
  "$GUARD" backup-wipe >> "$OUT/progress-$T.txt" 2>&1 || { echo "rc=4 guard-backup" > "$DONE"; exit 4; }
  cd "$WPDEV" || { echo "rc=2 wpdev" > "$DONE"; exit 2; }
  step "$1 media oracle"
  reset_env
  MIMALLOC_PURGE_DELAY=0 /usr/bin/time -l "$ORACLE" vendor/bin/phpunit --group media \
    > "$OUT/$1-$T/media-oracle.txt" 2> "$OUT/$1-$T/media-oracle.time"
  step "$1 media oracle rc=$?"
  step "$1 media phpr (on)"
  reset_env
  MIMALLOC_PURGE_DELAY=0 /usr/bin/time -l "$PHPR" vendor/bin/phpunit --group media \
    > "$OUT/$1-$T/media-phpr.txt" 2> "$OUT/$1-$T/media-phpr.time"
  step "$1 media phpr rc=$?"
  find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
  "$GUARD" restore >> "$OUT/progress-$T.txt" 2>&1
  step "$1 DONE (mai giudicata)"
}

# ---- WARM-UP dichiarato (criterio p.2): media-only bilaterale, MAI giudicata
quiesce_gate warmup
warmup_media warmup

# ---- 6 gambe TUTTE ON leg1-on..leg6-on (criterio p.1: off PENSIONATA; pair109 INVARIATA)
# S-146 p.5: NESSUNA inserzione tra le gambe (az.rev. S-142 #2: replica
# peak-only SENZA inserzione; i peak per gamba sono osservativi nel giudice).
RC=0
for leg in 1 2 3 4 5 6; do
  for mode in on; do
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
# per gamba (ictx% della mediana del motore, rank CPU oracle) + DERIVA (s148 p.4).
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
legs = ["leg1-on", "leg2-on", "leg3-on", "leg4-on", "leg5-on", "leg6-on"]
print(f"== s154 coppia WP full+media sul pin s153 tentativo={T} (criteri s132/s146/s148 EREDITATI + s154-criterio-pair-t5.md: giudizio a MEDIANA per finestra, az.rev.3 S-149) ==")
print(f"grade=VERDICT  # derivazione meccanica dai .time; rc autoritativo = pair-out/pair154-{T}.done")
print("# full: cpu=user+sys (etichettato); media: CANONICA=user-only + companion (az.rev. S-128 #3)")
print("# az.rev. S-131 #3: riferimento PER CONFIGURAZIONE, ON-ONLY = canonico; firma per gamba")
print("# az.rev. S-136 #2: lettura firma PRE-REGISTRATA: >150% med MOTORE = SPORCA (esclusa); 120-150% = ELEVATA (nel canone, annotata); <=120% pulita")
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
    eleva = "  ELEVATA(120-150% med MOTORE: resta nel canone, annotata — az.rev. S-136 #2)" if (not flagged and (po > 120 or pp > 120)) else ""
    print(f"  firma {leg}: ictx%med oracle={po:.0f}% phpr={pp:.0f}% rank_cpu_oracle={rk}/6 (1=piu' veloce){eleva}")
print(f"ictx_s_mediana_oracle={med['oracle']:.0f} ictx_s_mediana_phpr={med['phpr']:.0f}  # PER MOTORE")
clean = [l for l in legs if l not in flag_legs]
print(f"gambe pulite: {clean}")
for leg in legs:
    oc = O[leg]["user"] + O[leg]["sys"]; pc = P[leg]["user"] + P[leg]["sys"]
    print(f"{leg}: full_oracle_cpu={oc:.2f} full_phpr_cpu={pc:.2f} proprio={pc/oc:.3f} peak_phpr_MiB={P[leg].get('pf',0)/1048576:.1f}")
# S-146 p.5: replica PEAK-ONLY SENZA inserzione (az.rev. S-142 #2; osservativa, NESSUN gate)
pk = {l: P[l].get('pf',0)/1048576 for l in legs}
print("replica_peak_senza_inserzione (p.5): " +
      " ".join(f"{l}={pk[l]:.1f}" for l in legs))
lo = [l for l in legs if pk[l] <= 1825]; hi = [l for l in legs if pk[l] >= 1830]
if lo == ["leg1-on"] and len(hi) == 5:
    print("replica_peak lettura: leg1 BASSA (<=1825) e leg2-6 alte (>=1830) -> pattern S-140 RIPRODOTTO senza inserzione (coerente STATO-post-media; nessuna firma causale)")
elif len(hi) == 6:
    print("replica_peak lettura: 6/6 alte (>=1830) -> livello ALTO unico (il doppio livello NON si riproduce)")
elif len(lo) == 6:
    print("replica_peak lettura: 6/6 basse (<=1825) -> livello BASSO unico (il doppio livello NON si riproduce)")
else:
    print("replica_peak lettura: esito MISTO -> nessuna firma, si dichiara")
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
# az.rev. S-131 #3: PER CONFIGURAZIONE — ON-ONLY = riferimento CANONICO (off PENSIONATA)
for cfg in ("on",):
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
        # s154-criterio-pair-t5.md p.2 (az.rev.3 S-149): GIUDIZIO CANONICO a
        # MEDIANA per finestra vs mediane storiche t1=1,799 t2=1,738 t3=1,789 t4=1,781;
        # la banda-unione resta SOLO companion descrittivo.
        MED_LO, MED_HI = 1.738, 1.799
        med_t5 = statistics.median(props)
        print(f"mediana_t5={med_t5:.3f} (N={len(props)} coppie proprie ON PULITE; mediane storiche t1=1.799 t2=1.738 t3=1.789 t4=1.781)")
        if MED_LO <= med_t5 <= MED_HI:
            print(f"giudizio_MEDIANA (canonico, criterio t5 p.2): COMPATIBILE (mediana in [{MED_LO}; {MED_HI}]) — nessun claim")
        elif med_t5 < MED_LO:
            print(f"giudizio_MEDIANA (canonico, criterio t5 p.2): direzione GIU' SEGNALATA (mediana {med_t5:.3f} < {MED_LO}) — attesa BT2 su WP dichiarata piccola/nulla: NON attribuibile senza census proprio")
        else:
            print(f"giudizio_MEDIANA (canonico, criterio t5 p.2): REGRESSIONE SEGNALATA (mediana {med_t5:.3f} > {MED_HI}) — indagine PRIMA di ogni altra leva")
        print(f"companion banda-unione (descrittivo): coppie in [1.722-1.823]: {sum(1 for r in props if 1.722 <= r <= 1.823)}/{len(props)}")
        # s140-criterio-pair.md p.4: FONDAZIONE banda ON-config (N>=5 gambe pulite)
        if len(props) >= 5:
            print(f"banda_ON_fondata={max(props)-min(props):.3f} (max-min di N={len(props)} coppie proprie ON PULITE; unione finestre nel verdetto di sessione)")
        else:
            print(f"banda_ON: N={len(props)} INSUFFICIENTE (<5 gambe pulite) — tentativo da integrare (criterio p.4)")
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
# S-148 p.4 (az.rev. S-146 #3): test ordinato DERIVA peak<->rapporto —
# soglie PRE-REGISTRATE in s148-criterio-t2.md PRIMA dei dati t2;
# OSSERVATIVO, nessun gate. Spearman, N=6, critico 0,829 (alpha~0,05 unilat.).
def spearman(xs, ys):
    def rank(v):
        s = sorted(range(len(v)), key=lambda i: v[i])
        r = [0.0]*len(v)
        i = 0
        while i < len(s):
            j = i
            while j+1 < len(s) and v[s[j+1]] == v[s[i]]:
                j += 1
            for k in range(i, j+1):
                r[s[k]] = (i+j)/2 + 1
            i = j+1
        return r
    rx, ry = rank(xs), rank(ys)
    mx, my = sum(rx)/len(rx), sum(ry)/len(ry)
    num = sum((a-mx)*(b-my) for a, b in zip(rx, ry))
    dx = sum((a-mx)**2 for a in rx) ** 0.5
    dy = sum((b-my)**2 for b in ry) ** 0.5
    return num/(dx*dy) if dx and dy else 0.0
if len(clean) == 6:
    idx = [legs.index(l)+1 for l in clean]
    pks = [pk[l] for l in clean]
    rats = [pcs[l]/ocs[l] for l in clean]
    ra = spearman(idx, pks)
    rb = spearman(pks, rats)
    ca = ("DERIVA DISCENDENTE confermata" if ra <= -0.829 else
          ("DERIVA ASCENDENTE (opposta)" if ra >= 0.829 else "nessuna firma di deriva"))
    cb = ("ACCOPPIAMENTO peak<->rapporto confermato" if rb >= 0.829 else "accoppiamento NON confermato")
    print(f"deriva_test (s148 p.4, N=6 critico 0,829): rho_A(leg,peak)={ra:.3f} -> {ca}; rho_B(peak,rapporto)={rb:.3f} -> {cb}")
    if ra <= -0.829 and rb >= 0.829:
        print("deriva_firma: stato discendente correlato al rapporto (OSSERVATIVA, nessun gate)")
else:
    print(f"deriva_test: gambe pulite N={len(clean)} < 6 -> SOTTO-CAMPIONATO, nessuna soglia applicata (s148 p.4)")
PY
echo "rc=$RC" > "$DONE"
step "DONE rc=$RC (fonte rc: $DONE)"
exit "$RC"
