#!/bin/bash
# s186-fetta0-mutante.sh — S-186 bis (decisione utente 2026-10-30 08:54, opzione (b)): RILANCIO del SOLO test del mutante
# della fetta 0 col GEMELLO Z profilato — ΔM = M − Z (stesso recipe/target: via il rumore di layout di `run_loop` che il
# confronto M − P del 29/10 portava in dispatch). Soglie INVARIATE (criterio s186-criterio-fetta0.md p.6, EMENDA 30/10):
# finestra [0,85;1,15] × 11,91M × ns_spin (ab-out/s186-leva/mutante-ns.txt) E quota builtin ≥ 75 % di ΔM. Bracci: Z
# (ab-out/s186-leva/phpr-Z, 23c0a94c26db62eb) · M (phpr-M, a3bfe3e373a89489), R=3 interleaved (Z M · M Z · Z M), nessun O/P/C,
# nessuna gamba di riferimento. rc 0 = strumento VIVO (la rotta B RIAPRE: il verdetto dei CANALI del 30/10 resta quello scritto) ·
# 4 = MUTO (B muore definitivamente) · 7 strumento · 8 gambe invalide · 6 lock · 9 pin. Esiti SOLO in fetta0-mut-out/
# fetta0-mut.{rc,done} e nel verdetto s186-fetta0-mutante-verdetto.out; progresso in fetta0-mut-out/progress.txt.
# COPIA DICHIARATA di s186-fetta0.sh (manifest s186-fetta0-mutante-copia.diff) coi SOLI adattamenti: bracci Z/M, ordini,
# uscite in fetta0-mut-out, verdetto ridotto al mutante (ΔM, quota builtin, delta dispatch M − Z a controllo, delta per classe).
# Generato con assistenza AI (Claude Fable 5).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp186-harness"
OUT="$H/fetta0-mut-out"; mkdir -p "$OUT"
VERD="$H/s186-fetta0-mutante-verdetto.out"; RCF="$OUT/fetta0-mut.rc"; DONE="$OUT/fetta0-mut.done"; rm -f "$DONE" "$RCF"
PROG="$OUT/progress.txt"; : > "$PROG"
p(){ echo "$(date +%H:%M:%S) $1" >> "$PROG"; }
fin(){ echo "$1" > "$RCF"; echo "rc=$1 $2" > "$DONE"; p "FINE rc=$1 $2"; exit "$1"; }
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
PHPR="$HOME/Claude/php-rust-output/release/phpr"
ZWIN="$H/ab-out/s186-leva/phpr-Z"
MUT="$H/ab-out/s186-leva/phpr-M"
XC="$H/xc-classi.py"
SP="${MAPPA_SP:?MAPPA_SP (workdir APFS) richiesto}"
PIN_ATTESO="${PIN_ATTESO:-9a6ae11bf7e900ef}"
Z_ATTESO="${Z_ATTESO:-23c0a94c26db62eb}"
M_ATTESO="${M_ATTESO:-a3bfe3e373a89489}"
LOCK=/private/tmp/phpr-measure.lock
KT="$(getconf DARWIN_USER_TEMP_DIR)"
XPATH='//trace-toc/run[@number="1"]/data/table[@schema="time-profile"]'
TESTS_ATTESI=3484

# --- precondizioni ---
PINM="$(shasum -a 256 "$PHPR" | cut -c1-16)"
[ "$PINM" = "$PIN_ATTESO" ] || fin 9 "pin $PINM != $PIN_ATTESO"
[ -e "$LOCK" ] || fin 6 "measure-lock ASSENTE (finestra non aperta)"
[ -s "$ZWIN" ] || fin 7 "braccio Z assente ($ZWIN)"
[ -s "$MUT" ] || fin 7 "braccio M assente ($MUT)"
[ -s "$XC" ] || fin 7 "xc-classi.py assente"
[ -s "$H/ab-out/s186-leva/mutante-ns.txt" ] || fin 7 "mutante-ns.txt assente (taratura della build)"
xcrun xctrace version > "$OUT/xctrace.version" 2>&1 || fin 7 "xctrace non disponibile"
[ -s "$GATES/orm-work.tgz" ] || fin 7 "orm-work.tgz assente"
mkdir -p "$SP" || fin 7 "MAPPA_SP non creabile"
ZH="$(shasum -a 256 "$ZWIN" | cut -c1-16)"; MH="$(shasum -a 256 "$MUT" | cut -c1-16)"
[ "$ZH" = "$Z_ATTESO" ] || fin 9 "Z $ZH != atteso $Z_ATTESO"
[ "$MH" = "$M_ATTESO" ] || fin 9 "M $MH != atteso $M_ATTESO"
p "pin $PINM == atteso · Z $ZH · M $MH · ns_spin $(cat "$H/ab-out/s186-leva/mutante-ns.txt")"
: > "$OUT/walls.txt"; : > "$OUT/validity.txt"; : > "$OUT/classi.log"
ls_sentinel(){ pgrep -fl 'language[_-]server|Antigravity|rust-analyzer|intelephense|serena|gopls|pylsp|solargraph' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' '; }
p "sentinella LS inizio: $(ls_sentinel)"

now(){ python3 -c 'import time; print("%.3f" % time.time())'; }
ws(){ rm -rf "$SP/orm-work"; tar xzf "$GATES/orm-work.tgz" -C "$SP"; }
valid(){ LC_ALL=C tr -d '\0' < "$1" | LC_ALL=C grep -aqE "^Tests: $TESTS_ATTESI,"; }
engine(){ case "$1" in Z) echo "$ZWIN";; M) echo "$MUT";; esac; }

leg_prof(){ # $1=braccio (Z|M) $2=round → gamba PROFILATA; esiti in $OUT/<tag>.*
  local arm="$1" r="$2" tag="$1$2" E T t0 t1 xrc
  E="$(engine "$arm")"
  ws || { p "$tag untar FALLITO"; return 7; }
  T="$OUT/$tag.trace"; rm -rf "$T"
  p "$tag START (profilato, $E)"
  t0=$(now)
  ( cd "$SP/orm-work" && xcrun xctrace record --template 'Time Profiler' --output "$T" --target-stdout "$OUT/$tag.txt" \
      --launch -- "$E" vendor/bin/phpunit --no-coverage > "$OUT/$tag.rec.log" 2>&1 < /dev/null )
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

# --- R=3 round interleaved (ordine pre-registrato: Z M · M Z · Z M) ---
STRUM=0
ORDINI="Z_M M_Z Z_M"
r=0
for ord in $ORDINI; do
  r=$((r+1))
  for arm in $(echo "$ord" | tr '_' ' '); do
    leg_prof "$arm" "$r" || STRUM=1
  done
done
p "sentinella LS fine: $(ls_sentinel)"
rm -rf "$SP/orm-work"
[ "$STRUM" = 0 ] || { p "STRUMENTO: almeno una gamba senza record/export/classi (vedi progress)"; }

# --- verdetto (criterio p.6, EMENDA 30/10: comparatore Z; finestra e quota INVARIATE) ---
MUTNS="$H/ab-out/s186-leva/mutante-ns.txt"
python3 -I - "$OUT" "$VERD" "$PINM" "$ZH" "$MH" "$STRUM" "$MUTNS" <<'PY'
import sys, os, re
OUT, VERD, PINM, ZH, MH, STRUM, MUTNS = sys.argv[1:8]
CLASSES = ["dispatch","call_ret","prop_ic","str","arr_hash","obj_life","alloc","builtin","compile","libc","kernel","pavimento","other"]
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
say(f"== S-186 bis FETTA 0 — SOLO MUTANTE col gemello Z (xctrace Time Profiler 1 ms, --launch, self per immagine+simbolo) — pin {PINM} (NON profilato) · Z {ZH} · M {MH} · R={R} interleaved Z M · M Z · Z M — EMENDA 30/10 (firma utente 08:54): ΔM = M − Z, finestra e quota INVARIATE ==")
say(f"strumento: {'TUTTE le gambe registrate/esportate/classificate' if STRUM == '0' else 'ALMENO UNA gamba senza record/export/classi (rc=8 se un braccio resta sotto 2 round validi)'}")
arms = {}
for arm in "ZM":
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
for arm in "ZM":
    if len(arms[arm]) < 2:
        say(f"BRACCIO {arm}: {len(arms[arm])} round validi < 2 ⇒ nessun verdetto (rc=8)"); rc = 8
if rc == 0 and STRUM != "0":
    say("strumento: gambe mancanti ma ogni braccio ha ≥2 round validi: verdetto emesso sui round validi (dichiarato)")
if rc == 8:
    open(VERD, "w").write("\n".join(lines) + "\n"); sys.exit(8)
M = {arm: {c: med([d[c] for d in arms[arm]]) for c in CLASSES + ["TOTAL"]} for arm in "ZM"}
for arm in "ZM":
    say(f"MEDIANE {arm}: " + " ".join(f"{c}={M[arm][c]:.2f}" for c in CLASSES) + f" TOTAL={M[arm]['TOTAL']:.2f}")
say("DELTA per classe M − Z (controllo: il rumore di layout del 29/10 stava in dispatch, +0,59 s con M − P):")
for c in CLASSES:
    say(f"  {c}: Z={M['Z'][c]:.2f} M={M['M'][c]:.2f} Δ={M['M'][c]-M['Z'][c]:+.2f} s")
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
dM = M["M"]["TOTAL"] - M["Z"]["TOTAL"]
dMb = M["M"]["builtin"] - M["Z"]["builtin"]
dMd = M["M"]["dispatch"] - M["Z"]["dispatch"]
quota = (dMb / dM) if dM > 0 else 0.0
in_win = lo <= dM <= hi
vivo = in_win and quota >= 0.75
say(f"MUTANTE (spin fuori linea `builtin_spin_mutant_s186` in CallBuiltin, self nel proprio simbolo → classe builtin; {fin_txt}): ΔM = M_total − Z_total = {M['M']['TOTAL']:.2f} − {M['Z']['TOTAL']:.2f} = {dM:+.2f} s ⇒ finestra {'RISPETTATA' if in_win else 'VIOLATA'}; Δ in classe builtin = {dMb:+.2f} s = {quota*100:.0f}% di ΔM (atteso ≥75 %) ⇒ quota {'RISPETTATA' if quota >= 0.75 else 'VIOLATA'}; Δ dispatch M − Z = {dMd:+.2f} s (controllo del rumore di layout) ⇒ strumento {'VIVO' if vivo else 'MUTO'}")
wM = [walls[f'M{r}'][0] for r in range(1, R+1) if f'M{r}' in walls]; wZ = [walls[f'Z{r}'][0] for r in range(1, R+1) if f'Z{r}' in walls]
if wM and wZ: say(f"  wall M−Z (mediane) = {med(wM)-med(wZ):+.2f} s (controllo, non giudice)")
say("LETTURA del 29/10 a registro: M − P = +2,36 s (finestra [1,66;2,25] violata), builtin 62 % ⇒ rc=4 per lettera (comparatore pin ≠ gemello): resta scritta in s186-fetta0-verdetto.out")
if vivo:
    say("ESITO: strumento VIVO col comparatore Z ⇒ il kill del 29/10 era dello strumento, non della realtà: la rotta B RIAPRE e il verdetto dei CANALI del 30/10 (dispatch Δ+12,0 s GO · obj_life Δ+4,32 s GO) resta quello scritto (rc=0)"); rc = 0
else:
    say("ESITO: mutante MUTO anche col comparatore Z ⇒ strumento non provato ⇒ B MUORE definitivamente (rc=4); riga 1 del NEXT resta a D"); rc = 4
say(f"ESITO rc={rc}")
open(VERD, "w").write("\n".join(lines) + "\n")
sys.exit(rc)
PY
prc=$?
fin "$prc" "verdetto s186-fetta0-mutante-verdetto.out"
