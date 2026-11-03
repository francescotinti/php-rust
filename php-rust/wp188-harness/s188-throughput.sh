#!/bin/bash
# s188-throughput.sh <tentativo> — S-188 Tappa 2: throughput WordPress, server residente
# (`php-server --axum --workers 4`, pin s188) vs oracle `php -S` a 4 worker, carico `ab` chiuso
# per livello C ∈ {1,4,8}; criterio s188-criterio-throughput.md. Struttura e igiene ereditate da
# s188-wp-server-pair.sh (quiescenza per boot, gate richieste, rusage dell'albero, GHz da time -l).
# Variabili: C_LEVELS ("1 4 8") N (150) WARM (20) R (3) PIN_ATTESO (hash server pin s188).
# rc SOLO da pair-out/tp188-<t>.done. Generato con assistenza AI (Claude Fable 5).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp188-harness"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
T="${1:?tentativo (es. t1, dry1)}"
OUT="$H/pair-out"; mkdir -p "$OUT"
VERD="$H/s188-throughput-verdetto-$T.out"
DONE="$OUT/tp188-$T.done"
C_LEVELS="${C_LEVELS:-1 4 8}"; N="${N:-150}"; WARM="${WARM:-20}"; R="${R:-3}"; WORKERS=4
ARMS="${ARMS:-AX O4}"
PORT=8080; BASE="http://127.0.0.1:$PORT"
WPDEV="$HOME/Claude/wpdev/src"
SRVBIN="$HOME/Claude/php-rust-output/release/php-server"
ORACLE="/opt/homebrew/opt/php/bin/php"
PIN_ATTESO="${PIN_ATTESO:?PIN_ATTESO (hash server pin s188) obbligatorio}"
SOCK=/private/tmp/mysql-wp8.sock
MLOCK=/private/tmp/phpr-measure.lock
URLS="front:/ post:/2026/07/hello-world/ feed:/?feed=rss2 login:/wp-login.php rest:/wp-json/wp/v2/posts"
RU="$H/rusage188.py"
NURL=$(echo $URLS | wc -w | tr -d ' '); NLEV=$(echo $C_LEVELS | wc -w | tr -d ' ')
EXPECTED_REQ=$(( 1 + NLEV * NURL * (WARM + N) ))
step(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/tp-progress-$T.txt"; }
fin(){ echo "rc=$1 ${2:-}" > "$DONE"; echo "$1" > "$OUT/tp188-$T.rc"; step "DONE rc=$1 ${2:-}"; exit "$1"; }
: > "$OUT/tp-progress-$T.txt"; rm -f "$DONE" "$OUT/tp188-$T.rc"
SRVPID=""; TIMEPID=""
cleanup(){ [ -n "$SRVPID" ] && kill -TERM "$SRVPID" 2>/dev/null; pkill -TERM -f -- "--port $PORT" 2>/dev/null; pkill -TERM -f "127.0.0.1:$PORT" 2>/dev/null; }
trap cleanup EXIT

[ -e "$VERD" ] && fin 7 verdetto-esiste
command -v ab >/dev/null || fin 2 "ab non trovato"
SRV="$(shasum -a 256 "$SRVBIN" | cut -c1-16)"
[ "$SRV" = "$PIN_ATTESO" ] || fin 9 "pin server $SRV ≠ $PIN_ATTESO"
[ -r "$MLOCK" ] && grep -q 's188' "$MLOCK" || fin 4 "lock assente o senza TOKEN s188"
if lsof -nP -iTCP:"$PORT" -sTCP:LISTEN >/dev/null 2>&1 || pgrep -f "127.0.0.1:$PORT" >/dev/null 2>&1 || pgrep -f -- "--port $PORT" >/dev/null 2>&1; then fin 3 "porta $PORT occupata"; fi
mysqladmin -S "$SOCK" -uroot ping 2>/dev/null | grep -q alive || fin 5 "MySQL wp8 giù"
MQ(){ mysql -S "$SOCK" -uroot -N -e "$1" 2>/dev/null; }
[ "$(MQ "SELECT option_value FROM wp.wp_options WHERE option_name='siteurl'")" = "$BASE" ] || fin 5 "siteurl ≠ $BASE"
[ "$(grep -c WP_BENCH_DISABLE_CRON "$WPDEV/wp-config.php")" = 1 ] || fin 5 "wp-config senza WP_BENCH_DISABLE_CRON"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo no-git)"
DRIVER_SHA="$(shasum -a 256 "$H/s188-throughput.sh" "$RU" | shasum -a 256 | cut -c1-16)"
PHPV="$("$ORACLE" -r 'echo PHP_VERSION;')"
if [ "$N" -ge 150 ] && [ "$R" -ge 3 ] && [ "$NLEV" -ge 3 ]; then GRADE=VERDICT; else GRADE=DRY; fi
step "identità: server=$SRV git=$GIT_REV driver_sha=$DRIVER_SHA php=$PHPV C=[$C_LEVELS] N=$N WARM=$WARM R=$R req/boot=$EXPECTED_REQ grade=$GRADE"

quiesce_gate(){
  local attempt q
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
    AX) env WP_BENCH_DISABLE_CRON=1 /usr/bin/time -l "$SRVBIN" --axum --workers "$WORKERS" --host 127.0.0.1 --port "$PORT" -t "$WPDEV" > /dev/null 2> "$D/server.log" & ;;
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
      ab -q -c "$lev" -n "$WARM" "$BASE$path" > /dev/null 2>&1
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

python3 - "$OUT" "$T" "$SRV" "$GIT_REV" "$DRIVER_SHA" "$C_LEVELS" "$N" "$R" "$PHPV" "$GRADE" "$URLS" "$ARMS" "$EXPECTED_REQ" > "$VERD" <<'PY'
import os, re, statistics, sys, glob
out, T, SRV, GIT, DSHA, CL, N, R, PHPV, GRADE, URLS, ARMS, EXPREQ = sys.argv[1:14]
levels = CL.split(); N = int(N); R = int(R); urls = [e.split(":", 1)[0] for e in URLS.split()]; arms = ARMS.split()
label = {"AX": "php-server --axum --workers 4 (pin s188)", "O4": "oracle php -S, PHP_CLI_SERVER_WORKERS=4"}
def rd(p): return open(p, errors="replace").read() if os.path.exists(p) else ""
def ru(p):
    v = rd(p).split(); return [int(x) for x in v] if len(v) == 5 else None
def abf(txt, key):
    m = re.search(key + r":\s+([\d.]+)", txt); return float(m.group(1)) if m else None
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
                tpr = abf(t, r"Time per request:\s+[\d.]+ \[ms\] \(mean\)\s*\nTime per request") or abf(t, "Time per request")
                cell = {"rps": rps, "failed": failed, "non2xx": non2 or 0.0, "tpr": tpr,
                        "ok": bool(rps and failed == 0.0 and not non2 and not b["void"] and r0 and r1)}
                if r0 and r1: cell["cpu"] = (r1[0] + r1[1] - r0[0] - r0[1]) / N / 1e6; cell["ucpu"] = (r1[0] - r0[0]) / N / 1e6
                b["cell"][(lev, u)] = cell
        boot[(c, a)] = b
print(f"== s188 throughput WP: php-server {SRV} --axum --workers 4 vs PHP {PHPV} php -S ×4 worker | git={GIT} driver_sha={DSHA} tentativo={T} epoch={int(__import__('time').time())} ==")
print(f"grade={GRADE}  # VERDICT solo con N>=150 R>=3 e i tre livelli; rc autoritativo = pair-out/tp188-{T}.done")
print(f"protocollo: ab chiuso per livello C={levels}, warm-up escluso, N={N} per cella, R={R} boot/braccio ordine AX,O4 · canonica = req/s (ab) · companion = CPU (user+sys) dell'albero/req, user/req, time/request · spread ≤5% su R altrimenti cella VOID · gate richieste/boot={EXPREQ} · DISABLE_WP_CRON via env")
voids = [f"c{c}-{a}({boot[(c,a)]['void'][:50]})" for c in range(1, R + 1) for a in arms if boot[(c, a)]["void"]]
retried = [f"c{c}-{a}×{boot[(c,a)]['voids']}" for c in range(1, R + 1) for a in arms if boot[(c, a)]["voids"]]
print(f"igiene: tree_pids per braccio {sorted(set((a, boot[(c,a)]['tree']) for c in range(1,R+1) for a in arms))} · richieste/boot={sorted(set(boot[k]['req'] for k in boot))} (attese {EXPREQ}) · boot scartati: {' '.join(retried) or 'nessuno'} · boot VOID finali: {len(voids)} {' '.join(voids)}")
print("classe di core (time -l): " + " ".join(f"c{c}-{a}={boot[(c,a)]['ghz']:.2f}GHz/IPC {boot[(c,a)]['ipc']:.2f}" for c in range(1, R + 1) for a in arms if boot[(c, a)]["ghz"]))
rc = 0; med = {}
for a in arms:
    print(f"\nbraccio {a} = {label.get(a, a)} — req/s [per ciclo] mediana spread · CPU(u+s)/req ms · user/req · time/req ms")
    for lev in levels:
        for u in urls:
            cells = [boot[(c, a)]["cell"][(lev, u)] for c in range(1, R + 1)]
            rps = [x["rps"] for x in cells if x.get("rps")]
            valid = all(x["ok"] for x in cells) and len(rps) == R
            spread = (max(rps) - min(rps)) / statistics.median(rps) if len(rps) >= 2 else None
            void = (not valid) or (R >= 3 and spread is not None and spread > 0.05)
            if not void: med[(a, lev, u)] = statistics.median(rps)
            else: rc = rc or 1
            cpu = statistics.median([x["cpu"] for x in cells if x.get("cpu") is not None] or [0]); ucpu = statistics.median([x["ucpu"] for x in cells if x.get("ucpu") is not None] or [0])
            tpr = statistics.median([x["tpr"] for x in cells if x.get("tpr")] or [0]); fl = sum(int(x["failed"] or 0) for x in cells); n2 = sum(int(x["non2xx"]) for x in cells)
            sp = f"{spread*100:.1f}%" if spread is not None else "n/a"
            print(f"  C={lev:<2} {u:<6} rps=[{' '.join(f'{v:.1f}' for v in rps)}] med={statistics.median(rps) if rps else float('nan'):.1f} spread={sp} {'VOID' if void else 'ok'}  cpu/req={cpu:.1f} user/req={ucpu:.1f} t/req={tpr:.1f} failed={fl} non2xx={n2}")
def ratio(a, b, lev, u):
    return med[(a, lev, u)] / med[(b, lev, u)] if (a, lev, u) in med and (b, lev, u) in med and med[(b, lev, u)] else None
if "AX" in arms and "O4" in arms:
    print("\nO4/AX req/s per cella: " + " · ".join(f"C={lev} {u}={r:.2f}" if (r := ratio('O4', 'AX', lev, u)) else f"C={lev} {u}=VOID" for lev in levels for u in urls))
for a in arms:
    if len(levels) >= 2:
        print(f"scaling {a} req/s C{levels[1]}/C{levels[0]}: " + " · ".join(f"{u}={med[(a,levels[1],u)]/med[(a,levels[0],u)]:.2f}" if (a,levels[1],u) in med and (a,levels[0],u) in med else f"{u}=VOID" for u in urls))
    if len(levels) >= 3:
        print(f"scaling {a} req/s C{levels[2]}/C{levels[1]}: " + " · ".join(f"{u}={med[(a,levels[2],u)]/med[(a,levels[1],u)]:.2f}" if (a,levels[2],u) in med and (a,levels[1],u) in med else f"{u}=VOID" for u in urls))
print(f"celle VOID: {' '.join(f'{a}/C{lev}/{u}' for a in arms for lev in levels for u in urls if (a, lev, u) not in med) or 'nessuna'}")
print(f"rc={rc}"); sys.exit(rc)
PY
JRC=$?
fin "$JRC" "giudice"
