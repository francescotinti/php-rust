#!/bin/bash
# s186-fetta0.sh — S-186 FETTA 0 (delibera concilio S-186 Q2, firmata 2026-10-23; criterio s186-criterio-fetta0.md):
# profilo di TEMPO bilaterale (`xctrace` Time Profiler, 1 ms, `--launch`) su UNA gamba ORM intera (orm-work.tgz, phpunit
# --no-coverage), per immagine+simbolo SELF in classi pre-registrate (xc-classi.py). Bracci: O oracle · P pin s183 ·
# M mutante (spin 200 ns in CallBuiltin, ab-out/s186-leva/phpr-M) · C census-time (ab-out/s186-leva/phpr-C, NON profilato:
# contatori Instant 1/64 per classe su stderr: call_fast call_general mcall mcall_general ret_fast ret_slow prop str arr new
# sweep builtin_lookup builtin hostbuiltin; D_obj dal companion della guardia in guardia-out-b/dobj.txt). R=3 round
# interleaved (O P M C · P O C M · M C O P) + UNA gamba di riferimento NON profilata per O e P (`/usr/bin/time -l`:
# overhead del profiler dichiarato). Esiti SOLO in
# fetta0-out/fetta0.{rc,done} e nel verdetto s186-fetta0-verdetto.out; progresso in fetta0-out/progress.txt.
# Script NUOVO (nessun genitore): le regole di finestra stanno nel lanciatore (s186-lancio-catena.sh).
# Generato con assistenza AI (Claude Fable 5).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp186-harness"
OUT="$H/fetta0-out"; mkdir -p "$OUT"
VERD="$H/s186-fetta0-verdetto.out"; RCF="$OUT/fetta0.rc"; DONE="$OUT/fetta0.done"; rm -f "$DONE" "$RCF"
PROG="$OUT/progress.txt"; : > "$PROG"
p(){ echo "$(date +%H:%M:%S) $1" >> "$PROG"; }
fin(){ echo "$1" > "$RCF"; echo "rc=$1 $2" > "$DONE"; p "FINE rc=$1 $2"; exit "$1"; }
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
ORACLE=/opt/homebrew/opt/php/bin/php
PHPR="$HOME/Claude/php-rust-output/release/phpr"
MUT="$H/ab-out/s186-leva/phpr-M"
CEN="$H/ab-out/s186-leva/phpr-C"
XC="$H/xc-classi.py"
SP="${MAPPA_SP:?MAPPA_SP (workdir APFS) richiesto}"
PIN_ATTESO="${PIN_ATTESO:-9a6ae11bf7e900ef}"
LOCK=/private/tmp/phpr-measure.lock
KT="$(getconf DARWIN_USER_TEMP_DIR)"
XPATH='//trace-toc/run[@number="1"]/data/table[@schema="time-profile"]'
TESTS_ATTESI=3484

# --- precondizioni ---
PINM="$(shasum -a 256 "$PHPR" | cut -c1-16)"
[ "$PINM" = "$PIN_ATTESO" ] || fin 9 "pin $PINM != $PIN_ATTESO"
[ -e "$LOCK" ] || fin 6 "measure-lock ASSENTE (finestra non aperta)"
[ -s "$MUT" ] || fin 7 "braccio M assente ($MUT)"
[ -s "$CEN" ] || fin 7 "braccio C assente ($CEN)"
[ -s "$XC" ] || fin 7 "xc-classi.py assente"
xcrun xctrace version > "$OUT/xctrace.version" 2>&1 || fin 7 "xctrace non disponibile"
[ -s "$GATES/orm-work.tgz" ] || fin 7 "orm-work.tgz assente"
mkdir -p "$SP" || fin 7 "MAPPA_SP non creabile"
MH="$(shasum -a 256 "$MUT" | cut -c1-16)"; CH="$(shasum -a 256 "$CEN" | cut -c1-16)"
p "pin $PINM == atteso · M $MH · C $CH · oracle $("$ORACLE" -r 'echo PHP_VERSION;') opcache.enable_cli=$("$ORACLE" -r 'echo ini_get("opcache.enable_cli")===""?"0":ini_get("opcache.enable_cli");') jit=$("$ORACLE" -r 'echo ini_get("opcache.jit") ?: "n/d";')"
: > "$OUT/walls.txt"; : > "$OUT/validity.txt"; : > "$OUT/classi.log"
ls_sentinel(){ pgrep -fl 'language[_-]server|Antigravity|rust-analyzer|intelephense|serena|gopls|pylsp|solargraph' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' '; }
p "sentinella LS inizio: $(ls_sentinel)"

now(){ python3 -c 'import time; print("%.3f" % time.time())'; }
ws(){ rm -rf "$SP/orm-work"; tar xzf "$GATES/orm-work.tgz" -C "$SP"; }
valid(){ LC_ALL=C tr -d '\0' < "$1" | LC_ALL=C grep -aqE "^Tests: $TESTS_ATTESI,"; }
engine(){ case "$1" in O) echo "$ORACLE";; P) echo "$PHPR";; M) echo "$MUT";; C) echo "$CEN";; esac; }
eopts(){ case "$1" in O) echo "-d memory_limit=-1";; *) echo "";; esac; }

leg_prof(){ # $1=braccio (O|P|M) $2=round → gamba PROFILATA; esiti in $OUT/<tag>.*
  local arm="$1" r="$2" tag="$1$2" E o T t0 t1 xrc
  E="$(engine "$arm")"; o="$(eopts "$arm")"
  ws || { p "$tag untar FALLITO"; return 7; }
  T="$OUT/$tag.trace"; rm -rf "$T"
  p "$tag START (profilato, $E)"
  t0=$(now)
  ( cd "$SP/orm-work" && xcrun xctrace record --template 'Time Profiler' --output "$T" --target-stdout "$OUT/$tag.txt" \
      --launch -- "$E" $o vendor/bin/phpunit --no-coverage > "$OUT/$tag.rec.log" 2>&1 < /dev/null )
  xrc=$?
  t1=$(now)
  echo "$tag wall=$(python3 -c "print('%.3f' % ($t1-$t0))") xrc=$xrc" >> "$OUT/walls.txt"
  [ -d "$T" ] || { p "$tag record FALLITO xrc=$xrc (rec.log)"; rm -rf "$KT"/instruments*.ktrace; return 7; }
  xcrun xctrace export --input "$T" --xpath "$XPATH" > "$OUT/$tag.xml" 2> "$OUT/$tag.exp.err" \
    || { p "$tag export FALLITO"; rm -rf "$T" "$KT"/instruments*.ktrace; return 7; }
  python3 -I "$XC" "$OUT/$tag.xml" "$OUT/$tag.classi.tsv" "$OUT/$tag.top.tsv" >> "$OUT/classi.log" 2>&1 \
    || { p "$tag classificazione FALLITA"; rm -rf "$T" "$KT"/instruments*.ktrace; return 7; }
  rm -rf "$T"; rm -rf "$KT"/instruments*.ktrace
  gzip -f "$OUT/$tag.xml"
  if valid "$OUT/$tag.txt"; then echo "$tag VALID" >> "$OUT/validity.txt"; else echo "$tag INVALID" >> "$OUT/validity.txt"; fi
  p "$tag fine xrc=$xrc $(tail -1 "$OUT/validity.txt") $(tail -1 "$OUT/classi.log" | cut -c1-120)"
  return 0
}

leg_census(){ # $1=round → braccio C NON profilato (census-time su stderr)
  local r="$1" tag="C$1" t0 t1 rc
  ws || { p "$tag untar FALLITO"; return 7; }
  p "$tag START (census, non profilato)"
  t0=$(now)
  ( cd "$SP/orm-work" && /usr/bin/time -l "$CEN" vendor/bin/phpunit --no-coverage > "$OUT/$tag.txt" 2> "$OUT/$tag.err" )
  rc=$?
  t1=$(now)
  echo "$tag wall=$(python3 -c "print('%.3f' % ($t1-$t0))") xrc=$rc" >> "$OUT/walls.txt"
  LC_ALL=C grep -a '^census-time' "$OUT/$tag.err" > "$OUT/$tag.census.tsv" || true
  if valid "$OUT/$tag.txt"; then echo "$tag VALID" >> "$OUT/validity.txt"; else echo "$tag INVALID" >> "$OUT/validity.txt"; fi
  p "$tag fine rc=$rc righe census $(wc -l < "$OUT/$tag.census.tsv" | tr -d ' ') $(tail -1 "$OUT/validity.txt")"
  return 0
}

leg_ref(){ # $1=braccio (O|P) → gamba di RIFERIMENTO non profilata (/usr/bin/time -l): overhead del profiler
  local arm="$1" tag="ref-$1" E o t0 t1 rc
  E="$(engine "$arm")"; o="$(eopts "$arm")"
  ws || { p "$tag untar FALLITO"; return 7; }
  p "$tag START (riferimento non profilato)"
  t0=$(now)
  ( cd "$SP/orm-work" && /usr/bin/time -l "$E" $o vendor/bin/phpunit --no-coverage > "$OUT/$tag.txt" 2> "$OUT/$tag.time" )
  rc=$?
  t1=$(now)
  echo "$tag wall=$(python3 -c "print('%.3f' % ($t1-$t0))") xrc=$rc" >> "$OUT/walls.txt"
  if valid "$OUT/$tag.txt"; then echo "$tag VALID" >> "$OUT/validity.txt"; else echo "$tag INVALID" >> "$OUT/validity.txt"; fi
  p "$tag fine rc=$rc user=$(awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' "$OUT/$tag.time")"
  return 0
}

# --- R=3 round interleaved (ordine pre-registrato, criterio p.2) ---
STRUM=0
ORDINI="O_P_M_C P_O_C_M M_C_O_P"
r=0
for ord in $ORDINI; do
  r=$((r+1))
  for arm in $(echo "$ord" | tr '_' ' '); do
    case "$arm" in
      C) leg_census "$r" || STRUM=1;;
      *) leg_prof "$arm" "$r" || STRUM=1;;
    esac
  done
done
# --- riferimento non profilato (uno per motore) ---
leg_ref O || STRUM=1
leg_ref P || STRUM=1
p "sentinella LS fine: $(ls_sentinel)"
rm -rf "$SP/orm-work"
[ "$STRUM" = 0 ] || { p "STRUMENTO: almeno una gamba senza record/export/classi (vedi progress)"; }

# --- verdetto (criterio p.4-p.9) ---
DOBJ="$H/guardia-out-b/dobj.txt"
MUTNS="$H/ab-out/s186-leva/mutante-ns.txt"
python3 -I - "$OUT" "$VERD" "$PINM" "$MH" "$CH" "$STRUM" "$DOBJ" "$MUTNS" <<'PY'
import sys, os, re, statistics
OUT, VERD, PINM, MH, CH, STRUM, DOBJ, MUTNS = sys.argv[1:9]
CLASSES = ["dispatch","call_ret","prop_ic","str","arr_hash","obj_life","alloc","builtin","compile","libc","kernel","pavimento","other"]
PAV = ("pavimento","kernel")          # pavimento comune = PDO/sqlite3/avvio + syscalls (dichiarato)
NAMED = [c for c in CLASSES if c != "other"]
R = 3
lines = []
def say(s): lines.append(s); print(s)
validity = {}
for l in open(os.path.join(OUT,"validity.txt")):
    t = l.split()
    if len(t) == 2: validity[t[0]] = (t[1] == "VALID")
walls = {}
for l in open(os.path.join(OUT,"walls.txt")):
    m = re.match(r"(\S+) wall=([\d.]+) xrc=(\d+)", l)
    if m: walls[m.group(1)] = (float(m.group(2)), int(m.group(3)))
def classi(tag):
    f = os.path.join(OUT, f"{tag}.classi.tsv")
    if not os.path.exists(f): return None
    d = {}
    for l in open(f):
        k, v = l.rstrip("\n").split("\t")
        if k.startswith("#"): continue
        d[k] = float(v)
    return d
def med(v):
    v = sorted(v); n = len(v)
    return v[n//2] if n % 2 else (v[n//2-1] + v[n//2]) / 2
say(f"== S-186 FETTA 0 — profilo di tempo bilaterale ORM (xctrace Time Profiler 1 ms, --launch, self per immagine+simbolo) — pin P {PINM} · M {MH} · C {CH} · R={R} interleaved ==")
say(f"strumento: {'TUTTE le gambe registrate/esportate/classificate' if STRUM == '0' else 'ALMENO UNA gamba senza record/export/classi (rc=7 se un braccio resta sotto 2 round validi)'}")
arms = {}
for arm in "OPM":
    rows = []
    for r in range(1, R+1):
        tag = f"{arm}{r}"
        d = classi(tag)
        ok = validity.get(tag, False) and d is not None
        w = walls.get(tag, (float('nan'), -1))
        say(f"  {tag}: {'VALID' if validity.get(tag) else 'INVALID'} wall={w[0]:.1f}s xrc={w[1]} " + (" ".join(f"{c}={d[c]:.2f}" for c in CLASSES) + f" TOTAL={d['TOTAL']:.2f}" if d else "classi ASSENTI"))
        if ok: rows.append(d)
    arms[arm] = rows
rc = 0
for arm in "OPM":
    if len(arms[arm]) < 2:
        say(f"BRACCIO {arm}: {len(arms[arm])} round validi < 2 ⇒ nessun verdetto (rc=8)"); rc = 8
if rc == 0 and STRUM != "0":
    say("strumento: gambe mancanti ma ogni braccio ha ≥2 round validi: verdetto emesso sui round validi (dichiarato)")
# census C
cens = {}
ccount = 0
for r in range(1, R+1):
    f = os.path.join(OUT, f"C{r}.census.tsv")
    if not os.path.exists(f) or not validity.get(f"C{r}", False): continue
    ccount += 1
    for l in open(f):
        t = l.rstrip("\n").split("\t")
        if len(t) >= 5 and t[0] == "census-time":
            cls, cnt, smp, est = t[1], int(t[2]), int(t[3]), float(t[4])
            cens.setdefault(cls, []).append((cnt, smp, est))
say(f"  C: round validi {ccount}; classi census: " + (", ".join(f"{k}: n={med([x[0] for x in v]):.0f} campioni={med([x[1] for x in v]):.0f} est={med([x[2] for x in v])/1e9:.2f}s" for k, v in sorted(cens.items())) if cens else "NESSUNA riga census-time (braccio C muto: calibrazione NON eseguita, dichiarato)"))
n_obj = med([x[0] for x in cens["new"]]) if "new" in cens else None
if rc == 8:
    open(VERD, "w").write("\n".join(lines) + "\n"); sys.exit(8)
M = {arm: {c: med([d[c] for d in arms[arm]]) for c in CLASSES + ["TOTAL"]} for arm in "OPM"}
def net(arm): return M[arm]["TOTAL"] - sum(M[arm][c] for c in PAV)
def chiusura(arm):
    n = net(arm)
    return (sum(M[arm][c] for c in NAMED if c not in PAV) / n) if n > 0 else 0.0
for arm in "OPM":
    say(f"MEDIANE {arm}: " + " ".join(f"{c}={M[arm][c]:.2f}" for c in CLASSES) + f" TOTAL={M[arm]['TOTAL']:.2f} pavimento(pav+kernel)={sum(M[arm][c] for c in PAV):.2f} NETTO={net(arm):.2f} chiusura={chiusura(arm)*100:.1f}% other={M[arm]['other']:.2f}")
# riferimento non profilato
for arm in "OP":
    tf = os.path.join(OUT, f"ref-{arm}.time")
    u = None
    if os.path.exists(tf):
        for l in open(tf, errors="replace"):
            m = re.search(r"([\d.]+)\s+user", l)
            if m: u = float(m.group(1))
    w = walls.get(f"ref-{arm}", (float('nan'), -1))
    say(f"RIFERIMENTO non profilato {arm}: user={u if u is not None else 'n/d'} s wall={w[0]:.1f}s {'VALID' if validity.get(f'ref-{arm}') else 'INVALID'} · profilato TOTAL(Running)={M[arm]['TOTAL']:.2f} s ⇒ overhead profiler = {(M[arm]['TOTAL']-u) if u is not None else float('nan'):+.2f} s (dichiarato, non sottratto)")
surplus = net("P") - net("O")
say(f"SURPLUS netto P−O = {net('P'):.2f} − {net('O'):.2f} = {surplus:.2f} s (riferimento t27: REF2 [34,23;34,31] vs ORA_REF 4,870 ⇒ ≈29,4 s)")
chi_ok = chiusura("P") >= 0.85 and chiusura("O") >= 0.85
say(f"CHIUSURA ≥85 %: P {chiusura('P')*100:.1f}% O {chiusura('O')*100:.1f}% ⇒ {'OK' if chi_ok else 'FALLITA (classi non chiudono: other troppo grande)'}")
go, nom = [], []
say("CANALI (classe: P_c − O_c, P_c/O_c):")
for c in NAMED:
    if c in PAV: continue
    dc = M["P"][c] - M["O"][c]
    ratio = (M["P"][c] / M["O"][c]) if M["O"][c] > 0 else float('inf')
    flag = ""
    if dc >= 3.0 and ratio >= 3: go.append(c); flag = " ⇒ GO-LEVA (≥3,0 s E ratio ≥3)"
    elif 1.5 <= dc < 3.0: nom.append(c); flag = " ⇒ NOMINATO a registro (in [1,5;3,0) s, non leva)"
    elif dc >= 3.0: flag = " (≥3,0 s ma ratio <3: non leva, a registro)"
    say(f"  {c}: P={M['P'][c]:.2f} O={M['O'][c]:.2f} Δ={dc:+.2f} s ratio={ratio:.2f}{flag}")
# tetti pre-registrati
say("TETTI pre-registrati (classe >2× il tetto ⇒ modello falso, dichiarato):")
tetti = {"call_ret": 2.1, "prop_ic": 1.2}
dobj = None
if os.path.exists(DOBJ):
    try: dobj = float(open(DOBJ).read().split()[0])
    except Exception: dobj = None
if n_obj is not None and dobj is not None:
    tetti["obj_life"] = n_obj * dobj / 1e9
    say(f"  tetto oggetti = N_obj {n_obj:.0f} × D_obj {dobj:.1f} ns = {tetti['obj_life']:.2f} s")
else:
    say(f"  tetto oggetti n/d (N_obj={'n/d' if n_obj is None else int(n_obj)}, D_obj={'n/d' if dobj is None else dobj}) — dichiarato")
falso = []
for c, t in tetti.items():
    dc = M["P"][c] - M["O"][c]
    stato = "MODELLO FALSO (>2× il tetto)" if dc > 2 * t else ("sopra il tetto, entro 2×" if dc > t else "entro il tetto")
    if dc > 2 * t: falso.append(c)
    say(f"  {c}: Δ={dc:+.2f} s vs tetto {t:.2f} s ⇒ {stato}")
# mutante — finestra CALIBRATA dalla build (ab-out/s186-leva/mutante-ns.txt: ns per CallBuiltin, pre-registrati [120;320]):
# ΔM atteso = [0,85;1,15] × 11,91e6 × ns_spin; file assente ⇒ ripiego [2,1;2,7] s DICHIARATO.
N_CB = 11.91e6
ns_spin = None
if os.path.exists(MUTNS):
    try: ns_spin = float(open(MUTNS).read().split()[0])
    except Exception: ns_spin = None
if ns_spin is not None:
    lo, hi = 0.85 * N_CB * ns_spin / 1e9, 1.15 * N_CB * ns_spin / 1e9
    fin_txt = f"spin calibrato {ns_spin:.0f} ns/CallBuiltin ⇒ atteso [0,85;1,15]×11,91M×{ns_spin:.0f} ns = [{lo:.2f};{hi:.2f}] s"
else:
    lo, hi = 2.1, 2.7
    fin_txt = "mutante-ns.txt ASSENTE ⇒ ripiego alla finestra fissa [2,1;2,7] s (200 ns nominali), DICHIARATO"
dM = M["M"]["TOTAL"] - M["P"]["TOTAL"]
dMb = M["M"]["builtin"] - M["P"]["builtin"]
quota = (dMb / dM) if dM > 0 else 0.0
vivo = (lo <= dM <= hi) and quota >= 0.75
say(f"MUTANTE (spin fuori linea `builtin_spin_mutant_s186` in CallBuiltin, self nel proprio simbolo → classe builtin; {fin_txt}): ΔM = M_total − P_total = {M['M']['TOTAL']:.2f} − {M['P']['TOTAL']:.2f} = {dM:+.2f} s; Δ in classe builtin = {dMb:+.2f} s = {quota*100:.0f}% di ΔM (atteso ≥75 %) ⇒ strumento {'VIVO' if vivo else 'MUTO'}")
wM = [walls[f'M{r}'][0] for r in range(1, R+1) if f'M{r}' in walls]; wP = [walls[f'P{r}'][0] for r in range(1, R+1) if f'P{r}' in walls]
if wM and wP: say(f"  wall M−P (mediane) = {med(wM)-med(wP):+.2f} s (controllo, non giudice)")
# calibrazione C — classi di C: call_fast, call_general, mcall, mcall_general, ret_fast, ret_slow, prop, str, arr, new,
# sweep, builtin_lookup, builtin, hostbuiltin. Nella calibrazione si SOMMANO solo le classi a tempo: call_fast+mcall+ret_fast
# → call_ret · prop → prop_ic · str → str · arr → arr_hash (SOLO FetchDim*/ArrayPush/CoalesceFetchDim: gli STORE dim non sono
# strumentati, dichiarato) · new+sweep → obj_life · builtin → builtin. call_general/mcall_general/ret_slow sono SOLI CONTEGGI
# di ammissione (est_ns=0: mai sommati); builtin_lookup è SOTTO-scope di builtin (quota lookup, non sommata); hostbuiltin può
# contenere codice utente annidato (riportato a parte, non sommato).
MAP = {"call_ret": ["call_fast","mcall","ret_fast"], "prop_ic": ["prop"], "str": ["str"], "arr_hash": ["arr"], "obj_life": ["new","sweep"], "builtin": ["builtin"]}
NOTE = {"arr_hash": " (C: solo FetchDim*/ArrayPush/CoalesceFetchDim, store dim NON strumentati — dichiarato)"}
# Gli scope di C sono INCLUSIVI (il Ret contiene lo sweep che esegue; builtin contiene corpo e allocazioni; call_ret contiene
# push del frame e decay degli argomenti), le classi xctrace sono SELF per simbolo: il ±15 % vale per le classi a foglia
# (prop, str, arr, new, sweep); per call_ret e builtin C è un TETTO della classe xctrace (C ≥ xctrace = «TETTO (inclusivo ≥
# self)», contraddizione SOLO se C < xctrace × 0,85).
INCLUSIVE = {"call_ret", "builtin"}
if cens:
    say("CALIBRAZIONE C (Instant 1/64 per classe, estensione ×64; confronto con xctrace su P: ±15 % sulle classi a foglia, TETTO inclusivo su call_ret/builtin):")
    for xc_c, cls in MAP.items():
        est = sum(med([x[2] for x in cens[k]]) for k in cls if k in cens) / 1e9
        if est < 0.5:
            say(f"  {xc_c}: C={est:.2f} s < 0,5 s: fuori confronto{NOTE.get(xc_c, '')}"); continue
        px = M["P"][xc_c]
        dev = (px - est) / est if est > 0 else float('inf')
        if xc_c in INCLUSIVE:
            if est >= px: esito = "TETTO (inclusivo ≥ self): coerente"
            elif est >= 0.85 * px: esito = "C sotto il self entro il 15 %: COMPATIBILE (inclusivo)"
            else: esito = "NON calibra (C < xctrace × 0,85: l'unica direzione che contraddice): conta solo xctrace (dichiarato)"
        else:
            esito = "CALIBRA" if abs(dev) <= 0.15 else "NON calibra: conta solo xctrace (dichiarato)"
        say(f"  {xc_c}: C={est:.2f} s · xctrace P={px:.2f} s · scarto {dev*100:+.0f}% ⇒ {esito}{NOTE.get(xc_c, '')}")
    def cnt(k): return med([x[0] for x in cens[k]]) if k in cens else None
    def est_s(k): return (med([x[2] for x in cens[k]]) / 1e9) if k in cens else None
    fmt = lambda v: "n/d" if v is None else f"{v:.0f}"
    say(f"  AMMISSIONE (soli conteggi, mai sommati): call fast {fmt(cnt('call_fast'))} / generale {fmt(cnt('call_general'))} · mcall fast {fmt(cnt('mcall'))} / generale {fmt(cnt('mcall_general'))} · ret fuso {fmt(cnt('ret_fast'))} / ret_slow {fmt(cnt('ret_slow'))}")
    bl, bb = est_s("builtin_lookup"), est_s("builtin")
    hb = est_s("hostbuiltin")
    quota_l = (bl / bb * 100) if (bl is not None and bb) else float('nan')
    say(f"  BUILTIN: lookup per nome = {('n/d' if bl is None else f'{bl:.2f} s')} = quota {quota_l:.0f}% del corpo builtin {('n/d' if bb is None else f'{bb:.2f} s')} (sotto-scope, non sommato) · hostbuiltin {('n/d' if hb is None else f'{hb:.2f} s')} (può annidare codice utente: a parte, non sommato)")
else:
    say("CALIBRAZIONE C: non eseguita (nessuna riga census): conta solo xctrace, dichiarato")
# esito
if not vivo:
    say(f"ESITO: mutante MUTO ⇒ strumento non provato ⇒ B MUORE in questa sessione (rc=4); riga 1 del NEXT scrive D"); rc = 4
elif not chi_ok:
    say(f"ESITO: chiusura <85 % ⇒ il modello a classi non copre il netto ⇒ B MUORE in questa sessione (rc=4); riga 1 del NEXT scrive D"); rc = 4
elif not go:
    say(f"ESITO: nessun canale con Δ ≥3,0 s E ratio ≥3 (nominati: {', '.join(nom) if nom else 'nessuno'}) ⇒ B MUORE in questa sessione (rc=4); riga 1 del NEXT scrive D"); rc = 4
else:
    say(f"ESITO: GO-LEVA sui canali {', '.join(go)} (nominati a registro: {', '.join(nom) if nom else 'nessuno'}; modello falso su: {', '.join(falso) if falso else 'nessuna classe'}) ⇒ rotta B GO, fetta successiva per NOME sul canale maggiore (rc=0)"); rc = 0
say(f"ESITO rc={rc}")
open(VERD, "w").write("\n".join(lines) + "\n")
sys.exit(rc)
PY
prc=$?
fin "$prc" "verdetto s186-fetta0-verdetto.out"
