#!/bin/bash
# s166-orm-coppia.sh — coppia dbal+ORM sul pin s165 (DOVUTA: pin nuovo)
# (criterio s166-criterio-orm.md; copia DICHIARATA di s164-orm-coppia.sh,
# manifest s166-orm-copia.diff; adattamenti: pin s165 1fd8757d2f72dc3e,
# path wp166; RIF INVARIATO = registrato s162: ORA_REF=4,885 (REGGE per
# R=5 oracle-only S-165, mediana 4,860), rapporto registrato S-162
# [7,035;7,086]; DUE EMENDE dal verbale S-165 (entrambe PRE-registrate):
# (E1) segnalazione ictx PER MOTORE (istruttoria ictx CHIUSA: firma =
# denominatore su bracci 6-7x diversi); (E2) banda sentinella oracle
# [4,83;4,94] VINCOLANTE (gamba fuori => Delta_norm NON giudicante);
# attesa L-MC1d su ORM: direzione <=0, magnitudine NON pre-registrabile
# (quota chiamate a metodo AMMESSE non censita); attesa-AF1 resta APERTA.
# SENTINELLA CONTAMINAZIONE (lezione S-161): oracle net fuori dal SUO rif
# storico => cifra NULLA anche a flag per-gamba muti; parita' resta valida.
# TRE adattamenti NEL CANONE ereditati: rodaggio non giudicante, quiescenza
# per gamba (retry x3), scaletta a DUE estremi. rc SOLO da orm-out/rimisura.done.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp166-harness"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
WD="/Volumes/Extreme Pro/Claude/wp13-harness/run-with-watchdog.sh"
ORACLE=/opt/homebrew/opt/php/bin/php
PHPR="$HOME/Claude/php-rust-output/release/phpr"
SP="${MAPPA_SP:?MAPPA_SP (workdir APFS) richiesto}"
OUT="$H/orm-out"; mkdir -p "$OUT"
VERD="$H/s166-orm-coppia-verdetto.out"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
: > "$OUT/progress.txt"
rm -f "$OUT/rimisura.done"
PINM="$(shasum -a 256 "$PHPR" | cut -c1-16)"
[ "$PINM" = "1fd8757d2f72dc3e" ] || { echo "rc=9 pin!=s163" > "$OUT/rimisura.done"; exit 9; }
# lock della SESSIONE: si VERIFICA soltanto (niente creazione né trap).
LOCK=/private/tmp/phpr-measure.lock
[ -e "$LOCK" ] || { echo "rc=6 measure-lock ASSENTE (finestra non aperta)" > "$OUT/rimisura.done"; exit 6; }

# sentinella language-server (az.rev. S-157 #4): registrata nel verdetto,
# inizio e fine finestra; presenza != esclusione (misure a user-CPU), si dichiara.
ls_sentinel(){ pgrep -fl 'language[_-]server|Antigravity|rust-analyzer|intelephense|serena|gopls|pylsp|solargraph' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' '; }
LS_START="$(ls_sentinel)"
p "sentinella LS inizio: ${LS_START:-nessuno}"

# ADATTAMENTO (ii) DICHIARATO (istruttoria p.1): quiescenza per gamba giudicata,
# stesso gate del pair (s129-quiescenza.sh), retry x3 con pausa 30 s.
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
quiesce_gate(){ local attempt q
  for attempt in 1 2 3; do
    "$QUIESCE" "$OUT/quiesce-$1.rc" > "$OUT/quiesce-$1.log" 2>&1; q=$?
    p "quiescenza $1 rc=$q tentativo-gate=$attempt (file: orm-out/quiesce-$1.rc)"
    [ "$q" = 0 ] && return 0
    /bin/sleep 30
  done
  echo "rc=8 quiescenza-fail su $1 (3 tentativi)" > "$OUT/rimisura.done"; exit 8
}

# FIX az.rev. S-155 #4 (adattamento DICHIARATO): l'output dbal phpr contiene
# byte non-UTF-8 (latin1 a catalogo) — tr/sed/grep BSD in locale UTF-8 muoiono
# a metà stream e la summary/failnames escono tronche. LC_ALL=C + grep -a.
names(){ LC_ALL=C tr -d '\0' < "$1" | LC_ALL=C sed -n 's/^[0-9][0-9]*) \(.*\)$/\1/p' | LC_ALL=C sort -u; }
summ(){ LC_ALL=C tr -d '\0' < "$1" | LC_ALL=C grep -aE "^(Tests:|OK)" | tail -1; }

# pavimenti per-binario (med3 di `phpunit --version`) PER WORKSPACE — MISURATI
# su QUESTO pin (az.rev.4: mai ereditati).
rm -rf "$SP/orm-work"; tar xzf "$GATES/orm-work.tgz" -C "$SP" || { echo "rc=8 untar" > "$OUT/rimisura.done"; exit 8; }
rm -rf "$SP/dbal-work"; tar xzf "$GATES/dbal-work.tgz" -C "$SP" || { echo "rc=8 untar-dbal" > "$OUT/rimisura.done"; exit 8; }
floor3(){ local E="$1" P="$2" f1 f2 f3
  f1=$( { /usr/bin/time -l "$E" "$P" --version > /dev/null; } 2>&1 | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' )
  f2=$( { /usr/bin/time -l "$E" "$P" --version > /dev/null; } 2>&1 | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' )
  f3=$( { /usr/bin/time -l "$E" "$P" --version > /dev/null; } 2>&1 | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' )
  printf '%s\n%s\n%s\n' "$f1" "$f2" "$f3" | sort -n | awk 'NR==2'
}
FO=$(floor3 "$ORACLE" "$SP/orm-work/vendor/bin/phpunit"); FP=$(floor3 "$PHPR" "$SP/orm-work/vendor/bin/phpunit")
DFO=$(floor3 "$ORACLE" "$SP/dbal-work/vendor/bin/phpunit"); DFP=$(floor3 "$PHPR" "$SP/dbal-work/vendor/bin/phpunit")
echo "floor_user_med3 orm: oracle=$FO phpr=$FP · dbal: oracle=$DFO phpr=$DFP (phpunit --version, pavimento PARZIALE dichiarato)" > "$OUT/floors.txt"
p "floors: orm o=$FO p=$FP · dbal o=$DFO p=$DFP"

run_leg(){ # W TGZ DIR TIMEOUT ENGINE LABEL
  local W="$1" TGZ="$2" DIR="$3" T="$4" E="$5" L="$6"
  rm -rf "$SP/$DIR"; tar xzf "$GATES/$TGZ" -C "$SP" || return 7
  p "$W $L START"
  # EMENDA ereditata (s147 p.7): oracle con memory_limit=-1 (§3.14, parità
  # di condizioni).
  local ML=""
  [ "$E" = "$ORACLE" ] && ML="-d memory_limit=-1"
  ( cd "$SP/$DIR" && "$WD" -t "$T" -s 600 -p "$OUT/$W-$L.txt" -o "$OUT" -- \
      /usr/bin/time -l "$E" $ML vendor/bin/phpunit --no-coverage > "$OUT/$W-$L.txt" 2> "$OUT/$W-$L.time" )
  local rc=$?
  p "$W $L rc=$rc"
  return 0
}

WLS="${WORKLOADS:-orm dbal}"

# ADATTAMENTO (i) DICHIARATO (istruttoria p.1): RODAGGIO non giudicante — una
# gamba per motore e workload, scarica il transitorio primo-run (leg1 598/s ->
# leg2-3 ~120/s in isolamento, daemon scagionati). FUORI da ogni statistica;
# ictx/s a verbale come companion.
case " $WLS " in *" orm "*)
  run_leg orm orm-work.tgz orm-work 3600 "$ORACLE" "rodaggio-oracle"
  run_leg orm orm-work.tgz orm-work 3600 "$PHPR"   "rodaggio-phpr" ;;
esac
case " $WLS " in *" dbal "*)
  run_leg dbal dbal-work.tgz dbal-work 3600 "$ORACLE" "rodaggio-oracle"
  run_leg dbal dbal-work.tgz dbal-work 3600 "$PHPR"   "rodaggio-phpr" ;;
esac

for leg in 1 2; do
  case " $WLS " in *" orm "*)
    quiesce_gate "orm-leg$leg"
    run_leg orm orm-work.tgz orm-work 3600 "$ORACLE" "oracle$leg"
    run_leg orm orm-work.tgz orm-work 3600 "$PHPR"   "phpr$leg" ;;
  esac
  case " $WLS " in *" dbal "*)
    quiesce_gate "dbal-leg$leg"
    run_leg dbal dbal-work.tgz dbal-work 3600 "$ORACLE" "oracle$leg"
    run_leg dbal dbal-work.tgz dbal-work 3600 "$PHPR"   "phpr$leg" ;;
  esac
done

# parità: ORM phpr per NOME vs baseline 16; dbal fail-set phpr STABILE tra gambe
RC=0
for leg in 1 2; do
  names "$OUT/orm-phpr$leg.txt" > "$OUT/orm-phpr$leg.failnames"
  if ! diff -q "$REPO/wp125-harness/orm-baseline-failnames.txt" "$OUT/orm-phpr$leg.failnames" > /dev/null; then
    p "orm phpr$leg: fail-set DIVERGE — cifra NULLA"; RC=1
  fi
  names "$OUT/dbal-phpr$leg.txt"   > "$OUT/dbal-phpr$leg.failnames"
  names "$OUT/dbal-oracle$leg.txt" > "$OUT/dbal-oracle$leg.failnames"
done
if ! diff -q "$OUT/dbal-phpr1.failnames" "$OUT/dbal-phpr2.failnames" > /dev/null; then
  p "dbal: fail-set phpr NON stabile tra gambe — cifra NULLA"; RC=1
fi

python3 - "$OUT" "$FO" "$FP" "$DFO" "$DFP" "$RC" > "$OUT/ratios.txt" <<'PY'
import re, sys, statistics, os
out = sys.argv[1]
fo, fp, dfo, dfp = map(float, sys.argv[2:6])
parita_rc = int(sys.argv[6])
def t(f):
    d = {}
    for l in open(f):
        m = re.search(r'([\d.]+)\s+real', l)
        if m: d['real'] = float(m.group(1))
        m = re.search(r'([\d.]+)\s+user', l)
        if m: d['user'] = float(m.group(1))
        m = re.search(r'([\d.]+)\s+sys', l)
        if m: d['sys'] = float(m.group(1))
        m = re.search(r'^\s*(\d+)\s+involuntary context switches', l)
        if m: d['ictx'] = int(m.group(1))
    return d
def failset(f):
    return set(x.strip() for x in open(f) if x.strip()) if os.path.exists(f) else set()
print("coppia-s166 (user CPU; pavimenti PARZIALI phpunit --version: orm o=%.3f p=%.3f · dbal o=%.3f p=%.3f)" % (fo, fp, dfo, dfp))
print("grade=VERDICT  # derivazione meccanica dai .time")
new_net = {}; ora_net = {}; ratio_net = {}
orm_flags = []
for w, flo, flp in (("orm", fo, fp), ("dbal", dfo, dfp)):
    rate = {}; absi = {}
    for leg in (1, 2):
        o = t(f"{out}/{w}-oracle{leg}.time"); p_ = t(f"{out}/{w}-phpr{leg}.time")
        raw = p_['user']/o['user']
        net = (p_['user']-flp)/(o['user']-flo)
        new_net.setdefault(w, []).append(p_['user']-flp)
        ora_net.setdefault(w, []).append(o['user']-flo)
        ratio_net.setdefault(w, []).append(net)
        for k, d in ((f"oracle{leg}", o), (f"phpr{leg}", p_)):
            absi[k] = d.get('ictx', -1)
            rate[k] = (d['ictx']/d['real']) if ('ictx' in d and d.get('real', 0) > 0) else -1
        print(f"{w}_leg{leg}: oracle_user={o['user']:.2f} (sys={o.get('sys',0):.2f}) phpr_user={p_['user']:.2f} (sys={p_.get('sys',0):.2f}) ratio_raw={raw:.3f} ratio_net={net:.3f}")
    # EMENDA S-166 (istruttoria s165 CHIUSA: firma ictx = DENOMINATORE, bracci
    # 6-7x diversi in durata): la segnalazione si giudica PER MOTORE.
    flag = []
    for eng in ("oracle", "phpr"):
        ev = {k: v for k, v in rate.items() if k.startswith(eng) and v >= 0}
        emed = statistics.median(list(ev.values())) if ev else 0
        flag += [k for k, v in ev.items() if emed > 0 and v > 1.5*emed]
    if w == "orm": orm_flags = flag
    print(f"{w}_ictx_abs: " + " ".join(f"{k}={absi[k]}" for k in absi))
    print(f"{w}_ictx/s (gate emenda S-127): " + " ".join(f"{k}={rate[k]:.1f}" for k in rate) + (f"  SEGNALATE(>1,5x med): {flag}" if flag else "  contesa ok"))
s1, s2 = failset(f"{out}/dbal-phpr1.failnames"), failset(f"{out}/dbal-phpr2.failnames")
o1 = failset(f"{out}/dbal-oracle1.failnames")
diff = sorted(s1 - o1)
print(f"dbal_parita': phpr_failset_stabile={s1 == s2} |phpr_fail|={len(s1)} |diff_vs_oracle|={len(diff)}")
if diff[:20]: print("dbal_diff_nomi: " + " | ".join(diff[:20]))
# --- ATTESA L-MC1d (criterio s166-criterio-orm.md p.3, aritmetica meccanica) ---
# Giudizio CANONICO su net ORACLE-NORMALIZZATO EREDITATO; scaletta a DUE
# ESTREMI NEL CANONE (S-160); replica-AL1 RIMOSSA (attesa CHIUSA S-160).
REF2_MIN, REF2_MAX = 34.47, 34.51  # RIF s162-finestra gambe PULITE (2/2, sentinella negativa)
ORA_REF2 = 4.885                   # media DICHIARATA gambe oracle finestra s162 (4,87/4,90)
SENT_LO, SENT_HI = 4.83, 4.94      # EMENDA S-166: banda sentinella oracle PRE-registrata (s165-istruttoria-ictx-orm.md B1, VINCOLANTE; ORA_REF REGGE per R=5 s165 mediana 4.860)
RES = 0.293                        # risoluzione KS-146-1
RREF_LO, RREF_HI = 7.035, 7.086    # rapporto net registrato S-162 (companion)
ATT_LO, ATT_HI = 0.0, 0.0          # attesa L-MC1d DICHIARATA: direzione <=0 (mai peggiorare); magnitudine NON pre-registrabile (quota di chiamate a metodo AMMESSE — simple_call arita' esatta IC-hit — nel workload doctrine NON censita); GIU' = coerente con la leva, da dichiarare SENZA ripartizione senza census proprio (Composer/Doctrine a REGIME = classmap hit, i miss autoload [obj,metodo] RARI; quota miss NON censita => sotto-risoluzione; attesa-AF1 resta APERTA, non risolta da questa coppia)
nn = sorted(new_net.get("orm", []))
on = ora_net.get("orm", [])
rr = sorted(ratio_net.get("orm", []))
if len(nn) == 2 and len(on) == 2:
    d_min = REF2_MIN - nn[-1]; d_max = REF2_MAX - nn[0]
    print(f"attesa-MC1 assoluto (companion): phpr_orm_net_new=[{nn[0]:.2f}; {nn[1]:.2f}] ref=[{REF2_MIN}; {REF2_MAX}] Delta=[{d_min:+.2f}; {d_max:+.2f}] s")
    def _norm(ora_ref, ref_min, ref_max):
        normed = sorted(pn * ora_ref / o for pn, o in zip(new_net["orm"], on))
        return normed, ref_min - normed[-1], ref_max - normed[0]
    def _classe(dn_min, dn_max):
        # scaletta a DUE ESTREMI NEL CANONE (az.rev. S-159 #1): COMP SOLO a
        # intervallo INTERO dentro (-RES; RES); a cavallo su QUALUNQUE lato -> NR.
        if dn_min >= RES: return "GIU"
        if dn_max < -RES: return "REGR"
        if dn_min > -RES and dn_max < RES: return "COMP"
        return "NR"
    sent_fuori = [round(x, 2) for x in on if not (SENT_LO <= x <= SENT_HI)]
    n2, dn2_min, dn2_max = _norm(ORA_REF2, REF2_MIN, REF2_MAX)
    print(f"attesa-MC1 ORACLE-NORMALIZZATO (CANONICO coppia): net_norm=[{n2[0]:.2f}; {n2[1]:.2f}] (ORA_REF={ORA_REF2}, oracle_net_gambe={[round(x,2) for x in on]}) Delta_norm=[{dn2_min:+.2f}; {dn2_max:+.2f}] s")
    print(f"rapporto net (companion): [{rr[0]:.3f}; {rr[1]:.3f}] vs registrato S-162 [{RREF_LO}; {RREF_HI}]")
    c2 = _classe(dn2_min, dn2_max)
    if sent_fuori:
        print(f"SENTINELLA ORACLE FUORI BANDA [{SENT_LO};{SENT_HI}]: gambe {sent_fuori} — Delta_norm NON GIUDICANTE (emenda S-166), istruttoria drift dedicata")
    if parita_rc != 0:
        print("attesa-MC1: CIFRA NULLA (parita' rotta) — nessun giudizio")
    elif sent_fuori:
        pass  # giudizio sospeso dalla sentinella (riga sopra)
    elif c2 == "GIU":
        print(f"attesa-MC1: DIREZIONE GIU' FUORI RUMORE (Delta_norm_min {dn2_min:.2f} >= {RES}) — OLTRE l'attesa a tetto {ATT_LO}-{ATT_HI}: dichiarare, nessuna attribuzione senza census ORM proprio")
    elif c2 == "REGR":
        print(f"attesa-MC1: REGRESSIONE SEGNALATA (Delta_norm_max {dn2_max:.2f} < -{RES}) — indagine PRIMA di ogni altra leva")
    elif c2 == "COMP":
        print(f"attesa-MC1: COMPATIBILE (Delta_norm dentro il rumore ±{RES}; attesa a tetto {ATT_LO}-{ATT_HI} SOTTO risoluzione — esito dichiarato, nessuna cifra attribuita a L-AU1)")
    else:
        print(f"attesa-MC1: NON RISOLTA (intervallo Delta_norm a cavallo di {RES}) — si dichiara")
    # replica-AL1 RIMOSSA (attesa CHIUSA S-160); gambe segnalate restano a verbale
    if orm_flags:
        print(f"gambe ictx SEGNALATE {orm_flags} — a verbale (rodaggio nel canone; se ricorre, istruttoria dedicata)")
PY

rod_ictx(){ # companion istruttoria: ictx/s delle gambe di rodaggio (FUORI dal gate)
  local f="$OUT/$1.time" r i
  [ -f "$f" ] || { echo "n/d"; return; }
  r=$(tr -d '\0' < "$f" | awk '/[0-9.]+ *real/{for(j=1;j<=NF;j++) if($j=="real"){print $(j-1)}}')
  i=$(tr -d '\0' < "$f" | awk '/involuntary/{print $1}')
  python3 -c "print(f'{$i/$r:.1f}')" 2>/dev/null || echo "n/d"
}

{ echo "== s166 coppia dbal+ORM (pin s165 MISURATO $PINM vs oracle 8.5.7; criterio s166-criterio-orm.md) =="
  echo "# estrazione summ/names con LC_ALL=C + grep -a (fix az.rev. S-155 #4); reperto conteggi dbal phpr vs oracle A VERBALE (companion, non arbitra)"
  echo "# ADATTAMENTI (i)-(iii) dall'istruttoria p.1 + az.rev. S-159 #1: rodaggio non giudicante, quiescenza per gamba, scaletta a due estremi"
  echo "sentinella language-server inizio finestra (az.rev. S-157 #4): ${LS_START:-nessuno}"
  LS_END="$(ls_sentinel)"
  echo "sentinella language-server fine finestra: ${LS_END:-nessuno}"
  echo "rodaggio (NON giudicato, companion istruttoria) ictx/s: orm_oracle=$(rod_ictx orm-rodaggio-oracle) orm_phpr=$(rod_ictx orm-rodaggio-phpr) dbal_oracle=$(rod_ictx dbal-rodaggio-oracle) dbal_phpr=$(rod_ictx dbal-rodaggio-phpr)"
  for g in orm-leg1 dbal-leg1 orm-leg2 dbal-leg2; do
    echo "quiescenza $g: rc=$(cat "$OUT/quiesce-$g.rc" 2>/dev/null || echo MANCANTE) (file: orm-out/quiesce-$g.rc)"
  done
  cat "$OUT/floors.txt"
  cat "$OUT/ratios.txt"
  echo "parita': orm phpr per NOME vs baseline 16 · dbal fail-set stabile tra gambe — rc=$RC (0=tutte valide)"
  for leg in 1 2; do
    echo "orm  oracle$leg: $(summ "$OUT/orm-oracle$leg.txt")"
    echo "orm  phpr$leg:   $(summ "$OUT/orm-phpr$leg.txt")"
    echo "dbal oracle$leg: $(summ "$OUT/dbal-oracle$leg.txt")"
    echo "dbal phpr$leg:   $(summ "$OUT/dbal-phpr$leg.txt")"
  done
} > "$VERD"
echo "rc=$RC $(date +%T)" > "$OUT/rimisura.done"
p "DONE rc=$RC"
