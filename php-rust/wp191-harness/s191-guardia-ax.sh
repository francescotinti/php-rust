#!/bin/bash
# s191-guardia-ax.sh <tentativo> — S-191 coppia server: guardia di NON-REGRESSIONE del throughput AXnew = pin s190 (canonica,
# 721b7802e2d12058) vs AXold = pin s189 (stash, 80eea401b71c2be7), stesso giudice di S-189 (bande [0,95;1,05] a C=1,
# [0,97;1,05] a C=4/8). COPIA DICHIARATA di ../wp189-harness/s189-guardia-ax.sh (manifest s191-guardia-ax-copia.diff,
# generatore gen-notte-s191.py) coi SOLI adattamenti: harness wp191, done ga191, lock TOKEN s191, SRVOLD/PIN_OLD default s189;
# le etichette «pin s188/s189» nel testo S-189 sotto si leggono «s189 (AXold) / s190 (AXnew)». Testo S-189 conservato:
# s189-guardia-ax.sh <tentativo> — S-189 FETTA 1: guardia di NON-REGRESSIONE del throughput WordPress
# fra `php-server --axum --workers 4` pin s188 (AXold, stash) e pin s189 (AXnew, canonica), interallacciati
# nella STESSA finestra, carico `ab` chiuso per livello C ∈ {1,4,8}; criterio s189-criterio-guardia-ax.md.
# COPIA DICHIARATA di ../wp188-harness/s188-throughput.sh (manifest s189-guardia-ax-copia.diff, generatore
# gen-fetta1.py) coi SOLI adattamenti: nomi s189/ga189, bracci AXold/AXnew (due binari, due hash), warm-up
# a -c W (unit cache di ogni worker calda), lock TOKEN s189, giudice SOSTITUITO (AXnew/AXold per cella e
# per ciclo, bande [0,97;1,05] a C=4/8 e [0,95;1,05] a C=1, p99 e cpu/req companion). Il resto INVARIATO.
# Variabili: C_LEVELS N WARM R PIN_ATTESO (hash s189) PIN_OLD (hash s188) SRVOLD (stash). rc SOLO da
# pair-out/ga191-<t>.done. Generato con assistenza AI (Claude Fable 5).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp191-harness"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
T="${1:?tentativo (es. t1, dry1)}"
OUT="$H/pair-out"; mkdir -p "$OUT"
VERD="$H/s191-guardia-ax-verdetto-$T.out"
DONE="$OUT/ga191-$T.done"
C_LEVELS="${C_LEVELS:-1 4 8}"; N="${N:-150}"; WARM="${WARM:-20}"; R="${R:-3}"; WORKERS=4
ARMS="${ARMS:-AXold AXnew}"
PORT=8080; BASE="http://127.0.0.1:$PORT"
WPDEV="$HOME/Claude/wpdev/src"
SRVNEW="$HOME/Claude/php-rust-output/release/php-server"
SRVOLD="${SRVOLD:-/Volumes/Extreme Pro/Claude/phpr-old-target/release/php-server-s189}"
ORACLE="/opt/homebrew/opt/php/bin/php"
PIN_ATTESO="${PIN_ATTESO:?PIN_ATTESO (hash server pin s189 = AXnew) obbligatorio}"
PIN_OLD="${PIN_OLD:-80eea401b71c2be7}"
SOCK=/private/tmp/mysql-wp8.sock
MLOCK=/private/tmp/phpr-measure.lock
URLS="front:/ post:/2026/07/hello-world/ feed:/?feed=rss2 login:/wp-login.php rest:/wp-json/wp/v2/posts"
RU="$REPO/wp188-harness/rusage188.py"
NURL=$(echo $URLS | wc -w | tr -d ' '); NLEV=$(echo $C_LEVELS | wc -w | tr -d ' ')
EXPECTED_REQ=$(( 1 + NLEV * NURL * (WARM + N) ))
step(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/tp-progress-$T.txt"; }
fin(){ echo "rc=$1 ${2:-}" > "$DONE"; echo "$1" > "$OUT/ga191-$T.rc"; step "DONE rc=$1 ${2:-}"; exit "$1"; }
: > "$OUT/tp-progress-$T.txt"; rm -f "$DONE" "$OUT/ga191-$T.rc"
SRVPID=""; TIMEPID=""
cleanup(){ [ -n "$SRVPID" ] && kill -TERM "$SRVPID" 2>/dev/null; pkill -TERM -f -- "--port $PORT" 2>/dev/null; pkill -TERM -f "127.0.0.1:$PORT" 2>/dev/null; }
trap cleanup EXIT

[ -e "$VERD" ] && fin 7 verdetto-esiste
command -v ab >/dev/null || fin 2 "ab non trovato"
SRV="$(shasum -a 256 "$SRVNEW" | cut -c1-16)"; OLDH="$(shasum -a 256 "$SRVOLD" | cut -c1-16)"
[ "$SRV" = "$PIN_ATTESO" ] || fin 9 "pin server nuovo $SRV ≠ $PIN_ATTESO"
[ "$OLDH" = "$PIN_OLD" ] || fin 9 "pin server vecchio $OLDH ≠ $PIN_OLD"
[ -r "$MLOCK" ] && grep -q 's191' "$MLOCK" || fin 4 "lock assente o senza TOKEN s188"
if lsof -nP -iTCP:"$PORT" -sTCP:LISTEN >/dev/null 2>&1 || pgrep -f "127.0.0.1:$PORT" >/dev/null 2>&1 || pgrep -f -- "--port $PORT" >/dev/null 2>&1; then fin 3 "porta $PORT occupata"; fi
mysqladmin -S "$SOCK" -uroot ping 2>/dev/null | grep -q alive || fin 5 "MySQL wp8 giù"
MQ(){ mysql -S "$SOCK" -uroot -N -e "$1" 2>/dev/null; }
[ "$(MQ "SELECT option_value FROM wp.wp_options WHERE option_name='siteurl'")" = "$BASE" ] || fin 5 "siteurl ≠ $BASE"
[ "$(grep -c WP_BENCH_DISABLE_CRON "$WPDEV/wp-config.php")" = 1 ] || fin 5 "wp-config senza WP_BENCH_DISABLE_CRON"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo no-git)"
DRIVER_SHA="$(shasum -a 256 "$H/s189-guardia-ax.sh" "$RU" | shasum -a 256 | cut -c1-16)"
PHPV="$("$ORACLE" -r 'echo PHP_VERSION;')"
if [ "$N" -ge 150 ] && [ "$R" -ge 3 ] && [ "$NLEV" -ge 3 ]; then GRADE=VERDICT; else GRADE=DRY; fi
step "identità: server_new=$SRV server_old=$OLDH git=$GIT_REV driver_sha=$DRIVER_SHA php=$PHPV C=[$C_LEVELS] N=$N WARM=$WARM R=$R req/boot=$EXPECTED_REQ grade=$GRADE"

quiesce_gate(){
  local attempt q
  # SKIP_QUIESCE=1 salta il gate SOLO in un dry run (grade=DRY): collaudo dell'harness mentre la CI costruisce; mai per un record
  if [ "${SKIP_QUIESCE:-0}" = 1 ] && [ "$GRADE" = DRY ]; then step "quiescenza $1 SALTATA (dry run, SKIP_QUIESCE=1)"; return 0; fi
  for attempt in 1 2 3; do
    local w=0 streak=0 c
    while :; do
      c=$(ps -Ao %cpu,comm | awk 'index($0,"mediaanalysisd"){s+=$1} END{printf "%.1f", s+0}')
      if awk -v a="$c" 'BEGIN{exit !(a<5.0)}'; then streak=$((streak+1)); [ "$streak" -ge 4 ] && break
      else streak=0; w=$((w+1)); step "assestamento $1 (att.$attempt): mediaanalysisd $c%"; [ "$w" -gt 90 ] && break; /bin/sleep 10; fi
      /bin/sleep 5
    done
    "$QUIESCE" "$OUT/tp-quiesce-$T-$1.rc" > "$OUT/tp-quiesce-$T-$1.log" 2>&1; q=$?
    step "quiescenza $1 rc=$q tentativo-gate=$attempt"
    [ "$q" = 0 ] && return 0
  done
  fin 8 "quiescenza-fail su $1"
}
wait_up(){ local i=0; until curl -s -o /dev/null -m 2 "$1" 2>/dev/null; do i=$((i+1)); [ "$i" -gt 150 ] && return 1; sleep 0.2; done; return 0; }
port_free_wait(){ local i=0; while lsof -nP -iTCP:"$PORT" -sTCP:LISTEN >/dev/null 2>&1; do i=$((i+1)); [ "$i" -gt 100 ] && return 1; sleep 0.1; done; return 0; }
req_count(){ grep -c -E '\]: (GET|POST|HEAD|PUT|DELETE|OPTIONS) ' "$1"; }
tree(){ echo "$SRVPID" $(pgrep -P "$SRVPID"); }

run_boot(){ # $1=arm $2=dir → 0 pulito, 1 VOID
  local arm="$1" D="$2" entry u path lev n
  mkdir -p "$D"; rm -f "$D"/*
  cd "$WPDEV" || fin 2 wpdev
  case "$arm" in
    AXold) env WP_BENCH_DISABLE_CRON=1 /usr/bin/time -l "$SRVOLD" --axum --workers "$WORKERS" --host 127.0.0.1 --port "$PORT" -t "$WPDEV" > /dev/null 2> "$D/server.log" & ;;
    AXnew) env WP_BENCH_DISABLE_CRON=1 /usr/bin/time -l "$SRVNEW" --axum --workers "$WORKERS" --host 127.0.0.1 --port "$PORT" -t "$WPDEV" > /dev/null 2> "$D/server.log" & ;;
    O4) env WP_BENCH_DISABLE_CRON=1 PHP_CLI_SERVER_WORKERS=$WORKERS /usr/bin/time -l "$ORACLE" -S "127.0.0.1:$PORT" -t "$WPDEV" > /dev/null 2> "$D/server.log" & ;;
  esac
  TIMEPID=$!
  if ! wait_up "$BASE/wp-includes/css/buttons.css"; then echo "server-non-risponde" > "$D/VOID"; kill "$TIMEPID" 2>/dev/null; SRVPID=""; return 1; fi
  SRVPID="$(lsof -nP -tiTCP:"$PORT" -sTCP:LISTEN | head -1)"; [ -n "$SRVPID" ] || fin 3 "nessun pid in LISTEN"
  sleep 0.5
  { echo "arm=$arm pid=$SRVPID tree_pids=$(tree | wc -w | tr -d ' ')"; ps -o command= -p "$SRVPID"; } > "$D/identity.txt"
  step "$(basename "$D") boot ok pid=$SRVPID tree=$(tree | wc -w | tr -d ' ')"
  for lev in $C_LEVELS; do
    for entry in $URLS; do u="${entry%%:*}"; path="${entry#*:}"
      ab -q -c "$WORKERS" -n "$WARM" "$BASE$path" > /dev/null 2>&1  # S-189: warm-up a -c W (ogni unit cache calda)
      python3 "$RU" $(tree) > "$D/c$lev-$u.ru0" || echo "rusage $u" >> "$D/VOID"
      ab -q -c "$lev" -n "$N" "$BASE$path" > "$D/c$lev-$u.ab" 2>&1 || echo "ab rc≠0 c$lev $u" >> "$D/VOID"
      sleep 0.3
      python3 "$RU" $(tree) > "$D/c$lev-$u.ru1" || echo "rusage $u" >> "$D/VOID"
    done
    if pgrep -x cargo >/dev/null 2>&1 || pgrep -x rustc >/dev/null 2>&1 || pgrep -f phpt-runner >/dev/null 2>&1; then echo "contaminazione pgrep c$lev" >> "$D/VOID"; fi
  done
  kill -TERM "$SRVPID" 2>/dev/null
  n=0; while kill -0 "$TIMEPID" 2>/dev/null && [ "$n" -lt 300 ]; do sleep 0.1; n=$((n+1)); done
  if kill -0 "$TIMEPID" 2>/dev/null; then pkill -9 -P "$SRVPID" 2>/dev/null; kill -9 "$SRVPID" "$TIMEPID" 2>/dev/null; echo "uscita non pulita" > "$D/PEAK-ADVISORY"; fi
  wait "$TIMEPID" 2>/dev/null; SRVPID=""
  pkill -TERM -f "127.0.0.1:$PORT" 2>/dev/null; port_free_wait || fin 3 "porta non liberata"
  n=$(req_count "$D/server.log"); echo "$n" > "$D/reqcount.txt"
  [ "$n" = "$EXPECTED_REQ" ] || echo "richieste nel log $n ≠ attese $EXPECTED_REQ (client estraneo)" >> "$D/VOID"
  [ -e "$D/VOID" ] && return 1
  return 0
}

for cycle in $(seq 1 "$R"); do
  for arm in $ARMS; do
    BOOT="c$cycle-$arm"; D="$OUT/tp-$T/$BOOT"
    for attempt in 1 2 3; do
      quiesce_gate "$BOOT-a$attempt"
      if run_boot "$arm" "$D"; then step "$BOOT fine (tentativo $attempt, req=$(cat "$D/reqcount.txt"))"; break; fi
      step "$BOOT tentativo $attempt VOID: $(tr '\n' ';' < "$D/VOID" | cut -c1-160)"
      if [ "$attempt" -lt 3 ]; then rm -rf "$D.void$attempt"; mv "$D" "$D.void$attempt"; fi
    done
  done
done

python3 - "$OUT" "$T" "$SRV" "$OLDH" "$GIT_REV" "$DRIVER_SHA" "$C_LEVELS" "$N" "$R" "$PHPV" "$GRADE" "$URLS" "$ARMS" "$EXPECTED_REQ" > "$VERD" <<'PY'
import os, re, statistics, sys, glob
out, T, NEW, OLD, GIT, DSHA, CL, N, R, PHPV, GRADE, URLS, ARMS, EXPREQ = sys.argv[1:15]
levels = CL.split(); N = int(N); R = int(R); urls = [e.split(":", 1)[0] for e in URLS.split()]; arms = ARMS.split()
label = {"AXold": f"php-server --axum --workers 4 pin s189 {OLD} (stash)", "AXnew": f"php-server --axum --workers 4 pin s190 {NEW} (canonica)"}
BAND = {lev: ((0.95, 1.05) if lev == "1" else (0.97, 1.05)) for lev in levels}
def rd(p): return open(p, errors="replace").read() if os.path.exists(p) else ""
def ru(p):
    v = rd(p).split(); return [int(x) for x in v] if len(v) == 5 else None
def abf(txt, key):
    m = re.search(key + r":\s+([\d.]+)", txt); return float(m.group(1)) if m else None
def p99(txt):
    m = re.search(r"^\s*99%\s+(\d+)", txt, re.M); return float(m.group(1)) if m else None
boot = {}
for c in range(1, R + 1):
    for a in arms:
        d = f"{out}/tp-{T}/c{c}-{a}"; log = rd(f"{d}/server.log")
        tl = {}
        for pat, key in ((r"([\d.]+)\s+user", "user"), (r"([\d.]+)\s+sys", "sys"), (r"^\s*(\d+)\s+cycles elapsed", "cyc"), (r"^\s*(\d+)\s+instructions retired", "ins")):
            m = re.findall(pat, log, re.M)
            if m: tl[key] = float(m[-1])
        b = {"void": rd(f"{d}/VOID").strip(), "req": rd(f"{d}/reqcount.txt").strip() or "n/a", "voids": len(glob.glob(f"{d}.void*")),
             "tree": (re.search(r"tree_pids=(\d+)", rd(f"{d}/identity.txt")) or [None, "-1"])[1],
             "ghz": (tl["cyc"] / (tl["user"] + tl["sys"]) / 1e9) if tl.get("cyc") and (tl.get("user", 0) + tl.get("sys", 0)) > 0 else None,
             "ipc": (tl["ins"] / tl["cyc"]) if tl.get("cyc") and tl.get("ins") else None, "cell": {}}
        for lev in levels:
            for u in urls:
                t = rd(f"{d}/c{lev}-{u}.ab"); r0, r1 = ru(f"{d}/c{lev}-{u}.ru0"), ru(f"{d}/c{lev}-{u}.ru1")
                rps = abf(t, "Requests per second"); failed = abf(t, "Failed requests"); non2 = abf(t, "Non-2xx responses")
                cell = {"rps": rps, "failed": failed, "non2xx": non2 or 0.0, "p99": p99(t),
                        "ok": bool(rps and failed == 0.0 and not non2 and not b["void"] and r0 and r1)}
                if r0 and r1: cell["cpu"] = (r1[0] + r1[1] - r0[0] - r0[1]) / N / 1e6
                b["cell"][(lev, u)] = cell
        boot[(c, a)] = b
print(f"== s191 coppia server guardia AX (NON-REGRESSIONE; in banda ⇒ s190 server di record): AXnew s190 {NEW} vs AXold s189 {OLD}, --axum --workers 4, WordPress, PHP {PHPV} assente per disegno | git={GIT} driver_sha={DSHA} tentativo={T} epoch={int(__import__('time').time())} ==")
print(f"grade={GRADE}  # VERDICT solo con N>=150 R>=3 e i tre livelli; rc autoritativo = pair-out/ga191-{T}.done")
print(f"protocollo: ab chiuso per livello C={levels}, warm-up -n 20 a -c 4 escluso, N={N} per cella, R={R} cicli ordine AXold,AXnew nella STESSA finestra · canonica = AXnew/AXold req/s per cella nello stesso ciclo, mediana dei {R} · bande pre-registrate C=4/8 [0,97;1,05], C=1 [0,95;1,05] · spread ≤5% per braccio altrimenti cella VOID · mediana fuori banda con un ciclo dentro ⇒ VOID (rimisura), {R} cicli concordi fuori ⇒ KILL (KS-189-5) · companion p99 (C=8 nuovo ≤ vecchio; >1,10× REPERTO) e cpu(u+s)/req (±2%) · gate richieste/boot={EXPREQ}")
voids = [f"c{c}-{a}({boot[(c,a)]['void'][:50]})" for c in range(1, R + 1) for a in arms if boot[(c, a)]["void"]]
retried = [f"c{c}-{a}×{boot[(c,a)]['voids']}" for c in range(1, R + 1) for a in arms if boot[(c, a)]["voids"]]
print(f"igiene: tree_pids per braccio {sorted(set((a, boot[(c,a)]['tree']) for c in range(1,R+1) for a in arms))} · richieste/boot={sorted(set(boot[k]['req'] for k in boot))} (attese {EXPREQ}) · boot scartati: {' '.join(retried) or 'nessuno'} · boot VOID finali: {len(voids)} {' '.join(voids)}")
print("classe di core (time -l): " + " ".join(f"c{c}-{a}={boot[(c,a)]['ghz']:.2f}GHz/IPC {boot[(c,a)]['ipc']:.2f}" for c in range(1, R + 1) for a in arms if boot[(c, a)]["ghz"]))
rc = 0; med = {}; per = {}
for a in arms:
    print(f"\nbraccio {a} = {label.get(a, a)} — req/s [per ciclo] mediana spread · p99 ms · CPU(u+s)/req ms")
    for lev in levels:
        for u in urls:
            cells = [boot[(c, a)]["cell"][(lev, u)] for c in range(1, R + 1)]
            rps = [x["rps"] for x in cells if x.get("rps")]
            valid = all(x["ok"] for x in cells) and len(rps) == R
            spread = (max(rps) - min(rps)) / statistics.median(rps) if len(rps) >= 2 else None
            void = (not valid) or (R >= 3 and spread is not None and spread > 0.05)
            if not void:
                med[(a, lev, u)] = statistics.median(rps); per[(a, lev, u)] = [x["rps"] for x in cells]
            else: rc = rc or 1
            p9 = statistics.median([x["p99"] for x in cells if x.get("p99") is not None] or [0]); cpu = statistics.median([x["cpu"] for x in cells if x.get("cpu") is not None] or [0])
            sp = f"{spread*100:.1f}%" if spread is not None else "n/a"
            print(f"  C={lev:<2} {u:<6} rps=[{' '.join(f'{v:.1f}' for v in rps)}] med={statistics.median(rps) if rps else float('nan'):.1f} spread={sp} {'VOID' if void else 'ok'}  p99={p9:.0f} cpu/req={cpu:.1f} failed={sum(int(x['failed'] or 0) for x in cells)} non2xx={sum(int(x['non2xx']) for x in cells)}")
kills = []; voidc = []
print("\nAXnew/AXold req/s per cella [per ciclo] mediana banda esito:")
for lev in levels:
    lo, hi = BAND[lev]
    for u in urls:
        if ("AXnew", lev, u) not in med or ("AXold", lev, u) not in med:
            print(f"  C={lev:<2} {u:<6} VOID (cella senza mediana)"); voidc.append(f"C{lev}/{u}"); continue
        rs = [n / o for n, o in zip(per[("AXnew", lev, u)], per[("AXold", lev, u)]) if o]
        m = statistics.median(rs); inside = [lo <= r <= hi for r in rs]
        if lo <= m <= hi: esito = "ok"
        elif any(inside): esito = "VOID (rimisura: un ciclo dentro)"; voidc.append(f"C{lev}/{u}")
        else: esito = "KILL (3 cicli concordi fuori banda)" if m < lo else "REPERTO >1,05 (attribuire prima di promuovere)"; kills.append(f"C{lev}/{u}={m:.3f}")
        print(f"  C={lev:<2} {u:<6} [{' '.join(f'{r:.3f}' for r in rs)}] med={m:.3f} banda=[{lo:.2f};{hi:.2f}] {esito}")
def comp(key, name, thr, only_lev=None):
    # criterio p.4: il REPERTO p99 è definito SOLO a C=8 (revisione S-189 rilievo 4: a C=4 era un over-flag);
    # cpu/req ±2% su ogni cella. Le altre celle p99 restano stampate come advisory.
    rows = []
    for lev in levels:
        for u in urls:
            n = statistics.median([boot[(c, "AXnew")]["cell"][(lev, u)].get(key) or 0 for c in range(1, R + 1)])
            o = statistics.median([boot[(c, "AXold")]["cell"][(lev, u)].get(key) or 0 for c in range(1, R + 1)])
            flag = " REPERTO" if o and n / o > thr and (only_lev is None or lev == only_lev) else ""
            rows.append(f"C={lev} {u}={n/o:.3f}{flag}" if o else f"C={lev} {u}=n/a")
    scope = f" a C={only_lev}, advisory altrove" if only_lev else ""
    print(f"companion {name} nuovo/vecchio (REPERTO se > {thr}{scope}): " + " · ".join(rows))
comp("p99", "p99", 1.10, only_lev="8"); comp("cpu", "cpu(u+s)/req", 1.02)
print(f"celle VOID: {' '.join(voidc) or 'nessuna'} · KILL/REPERTO: {' '.join(kills) or 'nessuno'}")
if kills: rc = 2
print(f"rc={rc}  # 0 = tutte le celle in banda · 1 = celle VOID (rimisura) · 2 = KS-189-5 (cura NON promossa)"); sys.exit(rc)
PY
JRC=$?
fin "$JRC" "giudice"
