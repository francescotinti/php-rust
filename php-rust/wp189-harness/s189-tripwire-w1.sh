#!/bin/bash
# s189-tripwire-w1.sh <tentativo> — S-189 FETTA 1, tripwire di NON-REGRESSIONE a W=1 (criterio
# s189-criterio-guardia-ax.md p.5, Gregg R9): `php-server --axum --workers 1` pin s188 (Aold, stash) vs pin
# s189 (Anew, canonica) interallacciati, URL front+login, W=10 warm-up poi K=30 richieste misurate per URL,
# canonica = Δ user-CPU del processo server (proc_pid_rusage via rusage188.py) / K; spread ≤2% su R=3 boot
# altrimenti cella VOID; |Anew/Aold − 1| ≤ 2% altrimenti KILL (KS-189-6). Mai pubblicato come cifra: il record
# A/P = 1,00 resta il t3. Script NUOVO (non copia: il giudice della coppia esige i bracci P/O); pattern di boot,
# gate richieste e contaminazione pgrep ereditati da s188-wp-server-pair.sh. rc SOLO da pair-out/tw189-<t>.done.
# Generato con assistenza AI (Claude Fable 5).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp189-harness"
T="${1:?tentativo (es. t1, dry1)}"
OUT="$H/pair-out"; mkdir -p "$OUT"
VERD="$H/s189-tripwire-w1-verdetto-$T.out"
DONE="$OUT/tw189-$T.done"
W="${W:-10}"; K="${K:-30}"; R="${R:-3}"
ARMS="${ARMS:-Aold Anew}"
URLS="${URLS:-front:/ login:/wp-login.php}"
PORT=8080; BASE="http://127.0.0.1:$PORT"
WPDEV="$HOME/Claude/wpdev/src"
SRVNEW="$HOME/Claude/php-rust-output/release/php-server"
SRVOLD="${SRVOLD:-/Volumes/Extreme Pro/Claude/phpr-old-target/release/php-server-s188}"
PIN_ATTESO="${PIN_ATTESO:?PIN_ATTESO (hash server pin s189 = Anew) obbligatorio}"
PIN_OLD="${PIN_OLD:-97b58fd225ca7289}"
RU="$REPO/wp188-harness/rusage188.py"
SOCK=/private/tmp/mysql-wp8.sock
MLOCK=/private/tmp/phpr-measure.lock
NURL=$(echo $URLS | wc -w | tr -d ' ')
EXPECTED_REQ=$(( 1 + NURL * (W + K) ))
step(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/tw-progress-$T.txt"; }
fin(){ echo "rc=$1 ${2:-}" > "$DONE"; echo "$1" > "$OUT/tw189-$T.rc"; step "DONE rc=$1 ${2:-}"; exit "$1"; }
: > "$OUT/tw-progress-$T.txt"; rm -f "$DONE" "$OUT/tw189-$T.rc"
SRVPID=""
cleanup(){ [ -n "$SRVPID" ] && kill -TERM "$SRVPID" 2>/dev/null; pkill -TERM -f -- "--port $PORT" 2>/dev/null; }
trap cleanup EXIT

[ -e "$VERD" ] && fin 7 verdetto-esiste
NEWH="$(shasum -a 256 "$SRVNEW" | cut -c1-16)"; OLDH="$(shasum -a 256 "$SRVOLD" | cut -c1-16)"
[ "$NEWH" = "$PIN_ATTESO" ] || fin 9 "pin nuovo $NEWH ≠ $PIN_ATTESO"
[ "$OLDH" = "$PIN_OLD" ] || fin 9 "pin vecchio $OLDH ≠ $PIN_OLD"
[ -r "$MLOCK" ] && grep -q 's189' "$MLOCK" || fin 4 "lock assente o senza TOKEN s189"
if lsof -nP -iTCP:"$PORT" -sTCP:LISTEN >/dev/null 2>&1 || pgrep -f -- "--port $PORT" >/dev/null 2>&1; then fin 3 "porta $PORT occupata"; fi
mysqladmin -S "$SOCK" -uroot ping 2>/dev/null | grep -q alive || fin 5 "MySQL wp8 giù"
[ "$(grep -c WP_BENCH_DISABLE_CRON "$WPDEV/wp-config.php")" = 1 ] || fin 5 "wp-config senza WP_BENCH_DISABLE_CRON"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo no-git)"
DRIVER_SHA="$(shasum -a 256 "$H/s189-tripwire-w1.sh" "$RU" | shasum -a 256 | cut -c1-16)"
if [ "$W" -ge 10 ] && [ "$K" -ge 30 ] && [ "$R" -ge 3 ]; then GRADE=VERDICT; else GRADE=DRY; fi
step "identità: new=$NEWH old=$OLDH git=$GIT_REV driver_sha=$DRIVER_SHA W=$W K=$K R=$R req/boot=$EXPECTED_REQ grade=$GRADE"

wait_up(){ local i=0; until curl -s -o /dev/null -m 2 "$1" 2>/dev/null; do i=$((i+1)); [ "$i" -gt 150 ] && return 1; sleep 0.2; done; return 0; }
port_free_wait(){ local i=0; while lsof -nP -iTCP:"$PORT" -sTCP:LISTEN >/dev/null 2>&1; do i=$((i+1)); [ "$i" -gt 100 ] && return 1; sleep 0.1; done; return 0; }
req_count(){ grep -c -E '\]: (GET|POST|HEAD) ' "$1"; }
calm_wait(){ local n=0; while pgrep -x cargo >/dev/null 2>&1 || pgrep -x rustc >/dev/null 2>&1 || awk -v l="$(sysctl -n vm.loadavg | awk '{print $2}')" 'BEGIN{exit !(l+0 >= 3.0)}'; do n=$((n+1)); [ "$n" -gt 120 ] && fin 8 "calma non raggiunta"; sleep 10; done; }

run_boot(){ # $1=arm $2=dir → 0 pulito, 1 VOID
  local arm="$1" D="$2" bin entry u path i n code
  mkdir -p "$D"; rm -f "$D"/*
  case "$arm" in Aold) bin="$SRVOLD";; Anew) bin="$SRVNEW";; *) fin 2 "braccio $arm";; esac
  ( cd "$WPDEV" && exec env WP_BENCH_DISABLE_CRON=1 "$bin" --axum --workers 1 --host 127.0.0.1 --port "$PORT" -t "$WPDEV" ) > /dev/null 2> "$D/server.log" &
  SRVPID=$!
  if ! wait_up "$BASE/wp-includes/css/buttons.css"; then echo "server-non-risponde" > "$D/VOID"; kill "$SRVPID" 2>/dev/null; wait "$SRVPID" 2>/dev/null; SRVPID=""; return 1; fi
  echo "arm=$arm pid=$SRVPID bin=$bin" > "$D/identity.txt"
  for entry in $URLS; do u="${entry%%:*}"; path="${entry#*:}"
    for i in $(seq 1 "$W"); do curl -s -o /dev/null -m 60 "$BASE$path"; done
    python3 "$RU" "$SRVPID" > "$D/$u.ru0" || echo "rusage $u" >> "$D/VOID"
    for i in $(seq 1 "$K"); do code=$(curl -s -o /dev/null -m 60 -w '%{http_code}' "$BASE$path"); [ "$code" = 200 ] || echo "status $code su $u" >> "$D/VOID"; done
    sleep 0.3
    python3 "$RU" "$SRVPID" > "$D/$u.ru1" || echo "rusage $u" >> "$D/VOID"
    if pgrep -x cargo >/dev/null 2>&1 || pgrep -x rustc >/dev/null 2>&1 || pgrep -f phpt-runner >/dev/null 2>&1; then echo "contaminazione pgrep $u" >> "$D/VOID"; fi
  done
  kill -TERM "$SRVPID" 2>/dev/null; n=0
  while kill -0 "$SRVPID" 2>/dev/null && [ "$n" -lt 300 ]; do sleep 0.1; n=$((n+1)); done
  kill -0 "$SRVPID" 2>/dev/null && { kill -9 "$SRVPID" 2>/dev/null; echo "uscita non pulita" >> "$D/NOTE"; }
  wait "$SRVPID" 2>/dev/null; SRVPID=""
  port_free_wait || fin 3 "porta non liberata"
  n=$(req_count "$D/server.log"); echo "$n" > "$D/reqcount.txt"
  [ "$n" = "$EXPECTED_REQ" ] || echo "richieste nel log $n ≠ attese $EXPECTED_REQ (client estraneo)" >> "$D/VOID"
  [ -e "$D/VOID" ] && return 1
  return 0
}

for cycle in $(seq 1 "$R"); do
  for arm in $ARMS; do
    BOOT="c$cycle-$arm"; D="$OUT/tw-$T/$BOOT"
    for attempt in 1 2 3; do
      calm_wait
      if run_boot "$arm" "$D"; then step "$BOOT fine (tentativo $attempt, req=$(cat "$D/reqcount.txt"))"; break; fi
      step "$BOOT tentativo $attempt VOID: $(tr '\n' ';' < "$D/VOID" | cut -c1-160)"
      if [ "$attempt" -lt 3 ]; then rm -rf "$D.void$attempt"; mv "$D" "$D.void$attempt"; fi
    done
  done
done

python3 - "$OUT" "$T" "$NEWH" "$OLDH" "$GIT_REV" "$DRIVER_SHA" "$W" "$K" "$R" "$GRADE" "$URLS" "$ARMS" "$EXPECTED_REQ" > "$VERD" <<'PY'
import os, statistics, sys, glob
out, T, NEW, OLD, GIT, DSHA, W, K, R, GRADE, URLS, ARMS, EXPREQ = sys.argv[1:14]
K = int(K); R = int(R); urls = [e.split(":", 1)[0] for e in URLS.split()]; arms = ARMS.split()
def rd(p): return open(p, errors="replace").read() if os.path.exists(p) else ""
def ru(p):
    v = rd(p).split(); return [int(x) for x in v] if len(v) == 5 else None
cpu = {}; void = {}; reqs = set(); retried = []
for c in range(1, R + 1):
    for a in arms:
        d = f"{out}/tw-{T}/c{c}-{a}"; v = rd(f"{d}/VOID").strip(); reqs.add(rd(f"{d}/reqcount.txt").strip() or "n/a")
        if glob.glob(f"{d}.void*"): retried.append(f"c{c}-{a}×{len(glob.glob(f'{d}.void*'))}")
        for u in urls:
            r0, r1 = ru(f"{d}/{u}.ru0"), ru(f"{d}/{u}.ru1")
            if v or not (r0 and r1): void[(c, a, u)] = v or "rusage"; continue
            cpu[(c, a, u)] = (r1[0] - r0[0]) / K / 1e6
print(f"== s189 FETTA 1 tripwire W=1 (NON-REGRESSIONE, mai cifra): Anew {NEW} vs Aold {OLD}, --axum --workers 1 | git={GIT} driver_sha={DSHA} tentativo={T} ==")
print(f"grade={GRADE}  # VERDICT solo con W>=10 K>=30 R>=3; rc autoritativo = pair-out/tw189-{T}.done")
print(f"protocollo: processo singolo, W={W} warm-up poi K={K} misurate per URL, R={R} boot/braccio interallacciati Aold,Anew · canonica = Δ user-CPU proc_pid_rusage / K (ms) · spread ≤2% su R altrimenti cella VOID · |Anew/Aold − 1| ≤ 2% (mediana dei rapporti per ciclo) altrimenti KILL KS-189-6 · gate richieste/boot={EXPREQ}")
print(f"igiene: richieste/boot={sorted(reqs)} (attese {EXPREQ}) · boot scartati: {' '.join(retried) or 'nessuno'} · celle VOID: {len(void)} {' '.join(f'c{c}-{a}/{u}({w[:30]})' for (c,a,u),w in void.items())}")
rc = 0; med = {}
for a in arms:
    print(f"\nbraccio {a} — ms user-CPU/req [per ciclo] mediana spread")
    for u in urls:
        vals = [cpu[(c, a, u)] for c in range(1, R + 1) if (c, a, u) in cpu]
        spread = (max(vals) - min(vals)) / statistics.median(vals) if len(vals) >= 2 else None
        ok = len(vals) == R and spread is not None and spread <= 0.02
        if ok: med[(a, u)] = statistics.median(vals)
        else: rc = rc or 1
        print(f"  {u:<6} cpu=[{' '.join(f'{v:.2f}' for v in vals)}] med={statistics.median(vals) if vals else float('nan'):.2f} spread={(spread*100 if spread is not None else float('nan')):.1f}% {'ok' if ok else 'VOID'}")
print("\nAnew/Aold per ciclo (stesso ciclo) e mediana — tripwire ±2%:")
kill = []
for u in urls:
    rs = [cpu[(c, "Anew", u)] / cpu[(c, "Aold", u)] for c in range(1, R + 1) if (c, "Anew", u) in cpu and (c, "Aold", u) in cpu and cpu[(c, "Aold", u)]]
    if len(rs) < R or ("Anew", u) not in med or ("Aold", u) not in med:
        print(f"  {u:<6} VOID"); continue
    m = statistics.median(rs); bad = abs(m - 1) > 0.02
    if bad: kill.append(f"{u}={m:.3f}")
    print(f"  {u:<6} [{' '.join(f'{r:.3f}' for r in rs)}] med={m:.3f} {'KILL (KS-189-6: |Δ|>2%)' if bad else 'ok'}")
if kill: rc = 2
print(f"rc={rc}  # 0 = tripwire tenuto · 1 = celle VOID · 2 = KS-189-6 HALT ({' '.join(kill)})"); sys.exit(rc)
PY
JRC=$?
fin "$JRC" "giudice"
