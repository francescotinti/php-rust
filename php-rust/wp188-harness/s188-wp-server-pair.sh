#!/bin/bash
# s188-wp-server-pair.sh <tentativo> — S-188 Tappa 1: coppia server WordPress in MODO NATIVO
# (php-server pin s183, modo cli-server = `phpr -S`, nessun --axum) vs oracle PHP 8.5.7 `php -S`,
# criterio s188-criterio-server-pair.md (protocollo G-77.5 del Council WP-77.5: user-CPU del
# processo server / K, W dichiarato, R≥3 boot, spread ≤2 % ⇒ cella VOID, coppia stessa sera,
# opcache dichiarata in-band, guardia pgrep ⇒ VOID).
# Pattern citati: loop chiuso-sequenziale + /usr/bin/time -l + SIGTERM di wp78-harness/measure78.sh;
# quiesce_gate (streak mediaanalysisd + retry ×3) / ls_sentinel / .done autoritativo di
# wp184-harness/s184-pair.sh; set di URL e regola porta 8080 (= siteurl) di wp94-harness/battery61.sh.
# EMENDE t2 (revisione adversariale t1, 03/11 02:20 — criterio p.11): (a) GATE richieste = il log del
# server deve contare ESATTAMENTE 8 + 6·(W+K) richieste (una scheda browser ha colpito la 8080 in 4/12
# boot di t1); un boot sporco è SCARTATO PER INTERO e ripetuto (max 3), le prove restano in <boot>.void<n>;
# (b) snapshot rusage di mysqld + Δ Questions per blocco (companion «pagina servita»); (c) GHz effettivi
# e IPC per boot da `time -l` (cycles/instructions); (d) braccio di controllo Pphpr = `phpr -S` del pin
# (equivalenza misurata, non asserita); (e) W=20 (drift in-blocco +2,5 % su front a W=10).
# Variabili: W (warm-up, 20) K (misurate, 30) R (cicli, 3) DRAIN (1 = drena il cron dovuto UNA volta
# fuori canale) CONN_GATE (1 = tripwire Connections gate). rc SOLO da pair-out/pair188-<t>.done.
# Generato con assistenza AI (Claude Fable 5).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp188-harness"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
T="${1:?tentativo (es. t2, dry3)}"
OUT="$H/pair-out"; mkdir -p "$OUT"
VERD="$H/s188-wp-server-pair-verdetto-$T.out"
DONE="$OUT/pair188-$T.done"
# CONN_GATE default 1: promosso ADVISORY→GATE dal dry1 (03/11 00:54: Δ=K+1 esatto su ogni URL PHP, entrambi i lati; criterio p.8)
W="${W:-20}"; K="${K:-30}"; R="${R:-3}"; DRAIN="${DRAIN:-1}"; CONN_GATE="${CONN_GATE:-1}"
ARMS="P Pphpr O P0 Ooff"
PORT=8080; BASE="http://127.0.0.1:$PORT"
WPDEV="$HOME/Claude/wpdev/src"
SRVBIN="$HOME/Claude/php-rust-output/release/php-server"
PHPRBIN="$HOME/Claude/php-rust-output/release/phpr"
ORACLE="/opt/homebrew/opt/php/bin/php"
PIN_ATTESO="${PIN_ATTESO:-674f6b9730da41ff}"
PHPR_ATTESO="${PHPR_ATTESO:-9a6ae11bf7e900ef}"
SOCK=/private/tmp/mysql-wp8.sock
MLOCK=/private/tmp/phpr-measure.lock
POST_URL="/2026/07/hello-world/"
URLS="front:/ post:$POST_URL feed:/?feed=rss2 login:/wp-login.php rest:/wp-json/wp/v2/posts asset:/wp-includes/css/buttons.css"
EXPECTED_REQ=$(( 8 + 6 * (W + K) ))   # 1 sonda wait_up + 1 sonda opcache + 6 status + 6·(W+K)
PROBE="$WPDEV/s188-opcache-probe.php"
RU="$H/rusage188.py"
step(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress-$T.txt"; }
fin(){ echo "rc=$1 ${2:-}" > "$DONE"; echo "$1" > "$OUT/pair188-$T.rc"; step "DONE rc=$1 ${2:-}"; exit "$1"; }
: > "$OUT/progress-$T.txt"
rm -f "$DONE" "$OUT/pair188-$T.rc"
SRVPID=""; TIMEPID=""
cleanup(){ rm -f "$PROBE"; [ -n "$SRVPID" ] && kill -TERM "$SRVPID" 2>/dev/null; [ -n "${WDPID:-}" ] && kill "$WDPID" 2>/dev/null; }
trap cleanup EXIT

# ---- Fase 0: rifiuti e identità ------------------------------------------------------------------
[ -e "$VERD" ] && { step "verdetto $VERD ESISTE — tentativo nuovo = file nuovo (az.rev. S-131 #5)"; fin 7 verdetto-esiste; }
[ "$W" -ge 1 ] && [ "$K" -ge 1 ] && [ "$R" -ge 1 ] || fin 2 "W/K/R non validi"
SRV="$(shasum -a 256 "$SRVBIN" | cut -c1-16)"; PHPRH="$(shasum -a 256 "$PHPRBIN" | cut -c1-16)"
[ "$SRV" = "$PIN_ATTESO" ] || { step "ABORT pin: server=$SRV atteso $PIN_ATTESO"; fin 9 pin-mismatch; }
[ "$PHPRH" = "$PHPR_ATTESO" ] || { step "ABORT pin: phpr=$PHPRH atteso $PHPR_ATTESO"; fin 9 pin-mismatch-phpr; }
[ -r "$MLOCK" ] && grep -q 's188' "$MLOCK" || fin 4 "lock $MLOCK assente o senza TOKEN s188"
[ -z "${PHP_CLI_SERVER_WORKERS:-}" ] || fin 2 "PHP_CLI_SERVER_WORKERS deve essere unset"
if lsof -nP -iTCP:"$PORT" -sTCP:LISTEN >/dev/null 2>&1 || pgrep -f "127.0.0.1:$PORT" >/dev/null 2>&1; then fin 3 "porta $PORT occupata"; fi
mysqladmin -S "$SOCK" -uroot ping 2>/dev/null | grep -q alive || fin 5 "MySQL wp8 giù ($SOCK)"
MQ(){ mysql -S "$SOCK" -uroot -N -e "$1" 2>/dev/null; }
MYPID="$(pgrep -x mysqld | head -1)"; [ -n "$MYPID" ] || fin 5 "pid mysqld non trovato"
DD="$(MQ 'SELECT @@datadir')"
[ "$DD" = "/Volumes/Extreme Pro/Claude/mysql-wp8/data/" ] || fin 5 "datadir $DD non è quello esterno"
SU="$(MQ "SELECT option_value FROM wp.wp_options WHERE option_name='siteurl'")"
HM="$(MQ "SELECT option_value FROM wp.wp_options WHERE option_name='home'")"
[ "$SU" = "$BASE" ] && [ "$HM" = "$BASE" ] || fin 5 "siteurl=$SU home=$HM ≠ $BASE"
SLUG="$(MQ "SELECT CONCAT('/',DATE_FORMAT(post_date,'%Y/%m'),'/',post_name,'/') FROM wp.wp_posts WHERE ID=1 AND post_status='publish'")"
[ "$SLUG" = "$POST_URL" ] || fin 5 "permalink post 1 = $SLUG ≠ $POST_URL"
[ "$(grep -c WP_BENCH_DISABLE_CRON "$WPDEV/wp-config.php")" = 1 ] || fin 5 "wp-config.php senza il define WP_BENCH_DISABLE_CRON (criterio p.7)"
DATA_G="$(df -g /System/Volumes/Data | awk 'NR==2{print $4}')"
[ "${DATA_G:-0}" -ge 10 ] || fin 5 "Data ${DATA_G}G < 10G"
ST="$(python3 "$RU" $$ 2>&1)" || fin 2 "self-test rusage188: $ST"
set -- $ST; [ "$1" -gt 0 ] && [ "$3" -gt 0 ] || fin 2 "self-test rusage188: user_ns=$1 instr=$3"
DRIVER_SHA="$(shasum -a 256 "$H/s188-wp-server-pair.sh" "$RU" "$H/s188-lancio.sh" 2>/dev/null | shasum -a 256 | cut -c1-16)"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo no-git)"
PHPV="$("$ORACLE" -r 'echo PHP_VERSION;')"
OPC_INI="$("$ORACLE" -r 'echo "validate_timestamps=", ini_get("opcache.validate_timestamps"), " jit=", ini_get("opcache.jit"), " jit_buffer=", ini_get("opcache.jit_buffer_size");')"
ls_sentinel(){ pgrep -fl 'language[_-]server|Antigravity|rust-analyzer|intelephense|serena|gopls|pylsp|solargraph' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' '; }
LS_START="$(ls_sentinel)"
if [ "$W" -ge 10 ] && [ "$K" -ge 30 ] && [ "$R" -ge 3 ]; then GRADE=VERDICT; else GRADE=DRY; fi
step "identità: server=$SRV phpr=$PHPRH git=$GIT_REV driver_sha=$DRIVER_SHA php=$PHPV ($OPC_INI) W=$W K=$K R=$R req/boot=$EXPECTED_REQ grade=$GRADE Data=${DATA_G}G mysqld=$MYPID LS=${LS_START:-nessuno}"
printf '%s\n' '<?php echo function_exists("opcache_get_status") ? json_encode(opcache_get_status(false)["opcache_enabled"] ?? false) : "nofn";' > "$PROBE"
( while :; do echo "$(date +%H:%M:%S) Data=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}')G" >> "$OUT/data-$T.txt"; sleep 60; done ) > /dev/null 2>&1 &
WDPID=$!

quiesce_gate(){ # $1=etichetta — gate SEPARATO, rc nel suo file (s184: streak mediaanalysisd + retry ×3)
  local attempt q
  for attempt in 1 2 3; do
    local w=0 streak=0 c
    while :; do
      c=$(ps -Ao %cpu,comm | awk 'index($0,"mediaanalysisd"){s+=$1} END{printf "%.1f", s+0}')
      if awk -v a="$c" 'BEGIN{exit !(a<5.0)}'; then
        streak=$((streak+1)); [ "$streak" -ge 4 ] && break
      else
        streak=0; w=$((w+1)); step "assestamento $1 (att.$attempt): mediaanalysisd $c% (attesa ~$((w*15))s)"
        [ "$w" -gt 90 ] && break
        /bin/sleep 10
      fi
      /bin/sleep 5
    done
    "$QUIESCE" "$OUT/quiesce-$T-$1.rc" > "$OUT/quiesce-$T-$1.log" 2>&1; q=$?
    step "quiescenza $1 rc=$q tentativo-gate=$attempt"
    [ "$q" = 0 ] && return 0
  done
  fin 8 "quiescenza-fail su $1 (3 tentativi)"
}
dbstat(){ MQ "SHOW GLOBAL STATUS WHERE Variable_name IN ('Connections','Questions')" | awk '{printf "%s ", $2}'; }  # «conn questions»
wait_up(){ local i=0; until curl -s -o /dev/null -m 2 "$1" 2>/dev/null; do i=$((i+1)); [ "$i" -gt 150 ] && return 1; sleep 0.2; done; return 0; }
port_free_wait(){ local i=0; while lsof -nP -iTCP:"$PORT" -sTCP:LISTEN >/dev/null 2>&1; do i=$((i+1)); [ "$i" -gt 100 ] && return 1; sleep 0.1; done; return 0; }
req_count(){ grep -c -E '\]: (GET|POST|HEAD|PUT|DELETE|OPTIONS) ' "$1"; }

# ---- Fase 1: drenaggio del cron dovuto, UNA volta, fuori canale (criterio p.7) ------------------
if [ "$DRAIN" = 1 ]; then
  step "drain cron: oracle 4 worker, cron NON disattivato"
  ( cd "$WPDEV" && PHP_CLI_SERVER_WORKERS=4 "$ORACLE" -S "127.0.0.1:$PORT" -t "$WPDEV" > /dev/null 2> "$OUT/drain-$T.log" ) &
  DRPID=$!
  if wait_up "$BASE/s188-opcache-probe.php"; then
    curl -s -m 180 -o /dev/null -w 'drain wp-cron http %{http_code} %{time_total}s\n' "$BASE/wp-cron.php?doing_wp_cron" >> "$OUT/progress-$T.txt"
    curl -s -m 60 -o /dev/null "$BASE/"
  else
    step "drain: oracle non risponde (si prosegue: il drain è fuori canale)"
  fi
  pkill -TERM -f "127.0.0.1:$PORT" 2>/dev/null; wait "$DRPID" 2>/dev/null
  port_free_wait || fin 3 "porta $PORT non liberata dopo il drain"
  step "drain DONE"
fi

# ---- un boot completo: $1=arm $2=dir → rc 0 pulito, 1 VOID (motivi in $2/VOID) ------------------
run_boot(){
  local arm="$1" D="$2" entry u path c0 c1 code n
  mkdir -p "$D"; rm -f "$D"/*
  cd "$WPDEV" || fin 2 wpdev
  case "$arm" in
    P)     env WP_BENCH_DISABLE_CRON=1 /usr/bin/time -l "$SRVBIN" --host 127.0.0.1 --port "$PORT" -t "$WPDEV" > /dev/null 2> "$D/server.log" & ;;
    Pphpr) env WP_BENCH_DISABLE_CRON=1 /usr/bin/time -l "$PHPRBIN" -S "127.0.0.1:$PORT" -t "$WPDEV" > /dev/null 2> "$D/server.log" & ;;
    P0)    env WP_BENCH_DISABLE_CRON=1 PHPR_UNIT_CACHE=0 /usr/bin/time -l "$SRVBIN" --host 127.0.0.1 --port "$PORT" -t "$WPDEV" > /dev/null 2> "$D/server.log" & ;;
    O)     env WP_BENCH_DISABLE_CRON=1 /usr/bin/time -l "$ORACLE" -S "127.0.0.1:$PORT" -t "$WPDEV" > /dev/null 2> "$D/server.log" & ;;
    Ooff)  env WP_BENCH_DISABLE_CRON=1 /usr/bin/time -l "$ORACLE" -d opcache.enable=0 -S "127.0.0.1:$PORT" -t "$WPDEV" > /dev/null 2> "$D/server.log" & ;;
  esac
  TIMEPID=$!
  if ! wait_up "$BASE/s188-opcache-probe.php"; then echo "server-non-risponde" > "$D/VOID"; kill "$TIMEPID" 2>/dev/null; SRVPID=""; return 1; fi
  SRVPID="$(lsof -nP -tiTCP:"$PORT" -sTCP:LISTEN | head -1)"
  [ -n "$SRVPID" ] || fin 3 "$arm: nessun pid in LISTEN su $PORT"
  local TREE=$(( $(pgrep -P "$SRVPID" | wc -l | tr -d ' ') + 1 ))
  { echo "arm=$arm pid=$SRVPID tree_pids=$TREE mysqld=$MYPID"; ps -o command= -p "$SRVPID"; } > "$D/identity.txt"
  curl -s -m 5 "$BASE/s188-opcache-probe.php" > "$D/opcache.txt"
  : > "$D/status.txt"
  for entry in $URLS; do u="${entry%%:*}"; path="${entry#*:}"
    code=$(curl -s -o /dev/null -m 120 -w '%{http_code}' "$BASE$path"); echo "$u $code" >> "$D/status.txt"
    [ "$code" = 200 ] || echo "status $u=$code" >> "$D/VOID"
  done
  step "$(basename "$D") boot ok pid=$SRVPID tree=$TREE opcache=$(cat "$D/opcache.txt") status=$(awk '{printf "%s ", $2}' "$D/status.txt")"
  for entry in $URLS; do u="${entry%%:*}"; path="${entry#*:}"
    for _ in $(seq 1 "$W"); do curl -s -m 120 -o /dev/null "$BASE$path"; done
    c0=$(dbstat)
    python3 "$RU" "$SRVPID" $(pgrep -P "$SRVPID") > "$D/$u.ru0" || echo "rusage $u ru0" >> "$D/VOID"
    python3 "$RU" "$MYPID" > "$D/$u.my0" 2>/dev/null
    : > "$D/$u.csv"
    for _ in $(seq 1 "$K"); do curl -s -m 120 -o "$D/$u.last.body" -w '%{http_code} %{time_total}\n' "$BASE$path" >> "$D/$u.csv"; done
    sleep 0.3
    python3 "$RU" "$SRVPID" $(pgrep -P "$SRVPID") > "$D/$u.ru1" || echo "rusage $u ru1" >> "$D/VOID"
    python3 "$RU" "$MYPID" > "$D/$u.my1" 2>/dev/null
    c1=$(dbstat); echo "$c0 $c1" > "$D/$u.conn"
    if pgrep -x cargo >/dev/null 2>&1 || pgrep -x rustc >/dev/null 2>&1 || pgrep -f phpt-runner >/dev/null 2>&1; then echo "contaminazione pgrep durante $u" >> "$D/VOID"; fi
  done
  python3 "$RU" "$SRVPID" > "$D/final.ru" 2>/dev/null
  kill -TERM "$SRVPID" 2>/dev/null
  n=0; while kill -0 "$TIMEPID" 2>/dev/null && [ "$n" -lt 150 ]; do sleep 0.1; n=$((n+1)); done
  if kill -0 "$TIMEPID" 2>/dev/null; then kill -9 "$SRVPID" "$TIMEPID" 2>/dev/null; echo "uscita non pulita" > "$D/PEAK-ADVISORY"; fi
  wait "$TIMEPID" 2>/dev/null; SRVPID=""
  port_free_wait || fin 3 "porta $PORT non liberata dopo $(basename "$D")"
  # EMENDA (a): gate richieste — il log deve contare ESATTAMENTE le richieste dell'harness
  n=$(req_count "$D/server.log"); echo "$n" > "$D/reqcount.txt"
  if [ "$n" != "$EXPECTED_REQ" ]; then
    { echo "GET /s188-opcache-probe.php"; for entry in $URLS; do echo "GET ${entry#*:}"; done; } > "$D/expected-req.txt"
    { echo "richieste nel log $n ≠ attese $EXPECTED_REQ (client estraneo)"; grep -E '\]: (GET|POST|HEAD|PUT|DELETE|OPTIONS) ' "$D/server.log" | sed -E 's/.*\]: //' | grep -v -x -F -f "$D/expected-req.txt" | sort | uniq -c | sort -rn | head -8; } >> "$D/VOID"
  fi
  [ -e "$D/VOID" ] && return 1
  return 0
}

# ---- Fase 2: campagna R cicli × bracci, boot sporchi SCARTATI e ripetuti (max 3) ----------------
ANY_VOID=0; STATUS_BAD=0
for cycle in $(seq 1 "$R"); do
  for arm in $ARMS; do
    BOOT="c$cycle-$arm"; D="$OUT/$T/$BOOT"
    for attempt in 1 2 3; do
      quiesce_gate "$BOOT-a$attempt"
      if run_boot "$arm" "$D"; then step "$BOOT fine (tentativo $attempt, req=$(cat "$D/reqcount.txt"))"; break; fi
      step "$BOOT tentativo $attempt VOID: $(tr '\n' ';' < "$D/VOID" | cut -c1-200)"
      grep -q '^status ' "$D/VOID" && STATUS_BAD=1
      if [ "$attempt" -lt 3 ]; then rm -rf "$D.void$attempt"; mv "$D" "$D.void$attempt"; else ANY_VOID=1; fi
    done
  done
done
LS_END="$(ls_sentinel)"
rm -f "$PROBE"

# ---- Fase 3: giudice meccanico --------------------------------------------------------------------
python3 - "$OUT" "$T" "$SRV" "$PHPRH" "$GIT_REV" "$DRIVER_SHA" "$W" "$K" "$R" "$PHPV" "$GRADE" "$CONN_GATE" "$URLS" "$LS_START" "$LS_END" "$ARMS" "$EXPECTED_REQ" "$OPC_INI" > "$VERD" <<'PY'
import os, statistics, sys, re, filecmp, glob
out, T, SRV, PHPRH, GIT, DSHA, W, K, R, PHPV, GRADE, CONN_GATE, URLS, LS0, LS1, ARMS, EXPREQ, OPCINI = sys.argv[1:19]
W, K, R, EXPREQ = int(W), int(K), int(R), int(EXPREQ); CONN_GATE = CONN_GATE == "1"
urls = [e.split(":", 1)[0] for e in URLS.split()]
paths = dict(e.split(":", 1) for e in URLS.split())
arms = ARMS.split()
label = {"P": "phpr default (php-server modo nativo)", "Pphpr": "controllo `phpr -S` (pin phpr)", "O": "oracle php -S opcache ON", "P0": "phpr PHPR_UNIT_CACHE=0", "Ooff": "oracle -d opcache.enable=0"}
def exp_conn(u):  # asset: via statica, nessuna connessione MySQL — solo la query di stato stessa
    return 1 if u == "asset" else K + 1
def rd(p):
    return open(p, errors="replace").read() if os.path.exists(p) else ""
def ru(p):
    v = rd(p).split()
    return [int(x) for x in v] if len(v) == 5 else None
def timel(log):
    d = {}
    for pat, key in ((r"([\d.]+)\s+user", "user"), (r"([\d.]+)\s+sys", "sys"), (r"^\s*(\d+)\s+cycles elapsed", "cyc"), (r"^\s*(\d+)\s+instructions retired", "ins"), (r"^\s*(\d+)\s+peak memory footprint", "pf")):
        m = re.findall(pat, log, re.M)
        if m: d[key] = float(m[-1])
    return d
boot = {}
for c in range(1, R + 1):
    for a in arms:
        d = f"{out}/{T}/c{c}-{a}"
        tl = timel(rd(f"{d}/server.log"))
        b = {"void": rd(f"{d}/VOID").strip(), "opc": rd(f"{d}/opcache.txt").strip(), "ident": rd(f"{d}/identity.txt"),
             "status": dict(l.split() for l in rd(f"{d}/status.txt").splitlines() if len(l.split()) == 2), "cell": {},
             "req": rd(f"{d}/reqcount.txt").strip() or "n/a", "voids": len(glob.glob(f"{d}.void*")),
             "ghz": (tl["cyc"] / (tl["user"] + tl["sys"]) / 1e9) if tl.get("cyc") and (tl.get("user", 0) + tl.get("sys", 0)) > 0 else None,
             "ipc": (tl["ins"] / tl["cyc"]) if tl.get("cyc") and tl.get("ins") else None, "peak_tl": int(tl.get("pf", -1))}
        m = re.search(r"tree_pids=(\d+)", b["ident"]); b["tree"] = int(m.group(1)) if m else -1
        fr = ru(f"{d}/final.ru"); b["peak_ru"] = fr[4] if fr else -1
        for u in urls:
            r0, r1 = ru(f"{d}/{u}.ru0"), ru(f"{d}/{u}.ru1"); m0, m1 = ru(f"{d}/{u}.my0"), ru(f"{d}/{u}.my1")
            rows = [l.split() for l in rd(f"{d}/{u}.csv").splitlines() if len(l.split()) == 2]
            codes = [x[0] for x in rows]; walls = sorted(float(x[1]) * 1000 for x in rows)
            cv = rd(f"{d}/{u}.conn").split()
            dconn = int(cv[2]) - int(cv[0]) if len(cv) == 4 else -1; dq = int(cv[3]) - int(cv[1]) if len(cv) == 4 else -1
            cell = {"ok": bool(r0 and r1 and len(rows) == K and all(k == "200" for k in codes) and not b["void"]), "n": len(rows),
                    "non200": sum(1 for k in codes if k != "200"), "dconn": dconn, "dq": dq}
            if r0 and r1 and K:
                cell.update(cpu=(r1[0] - r0[0]) / K / 1e6, sys=(r1[1] - r0[1]) / K / 1e6, instr=(r1[2] - r0[2]) / K)
            if m0 and m1 and K:
                cell.update(my=(m1[0] + m1[1] - m0[0] - m0[1]) / K / 1e6)
            if walls:
                cell.update(wmed=statistics.median(walls), wp25=walls[len(walls) // 4], wp75=walls[(3 * len(walls)) // 4])
            if CONN_GATE and dconn != exp_conn(u):
                cell["ok"] = False; cell["why"] = f"Connections Δ={dconn}≠{exp_conn(u)}"
            b["cell"][u] = cell
        boot[(c, a)] = b
print(f"== s188 coppia server WP modo NATIVO: php-server {SRV} (cli-server, no --axum) + controllo phpr {PHPRH} (-S) vs PHP {PHPV} php -S | git={GIT} driver_sha={DSHA} tentativo={T} epoch={int(__import__('time').time())} ==")
print(f"grade={GRADE}  # VERDICT solo con W>=10 K>=30 R>=3; rc autoritativo = pair-out/pair188-{T}.done")
print(f"protocollo: processo SINGOLO per lato · W={W} K={K} R={R} boot/braccio, ordine ciclo {','.join(arms)} · canonica = Δ user-CPU proc_pid_rusage / K (ms) · spread ≤2% su R boot altrimenti cella VOID · wall curl secondario · nessun pavimento sottratto (asset = riga informativa) · DISABLE_WP_CRON via env · post {paths['post']} · tripwire Connections = {'GATE' if CONN_GATE else 'ADVISORY'} · GATE richieste/boot = {EXPREQ} (boot sporco scartato e ripetuto, max 3)")
opc = {a: sorted(set(boot[(c, a)]["opc"] for c in range(1, R + 1))) for a in arms}
print("opcache sonda in-band: " + " ".join(f"{a}={','.join(opc[a]) or 'n/a'}" for a in arms) + f" · oracle ini: {OPCINI} · PHP_CLI_SERVER_WORKERS=unset")
trees = sorted(set(boot[k]["tree"] for k in boot)); voids = [f"c{c}-{a}({boot[(c,a)]['void'][:60]})" for c in range(1, R + 1) for a in arms if boot[(c, a)]["void"]]
retried = [f"c{c}-{a}×{boot[(c,a)]['voids']}" for c in range(1, R + 1) for a in arms if boot[(c, a)]["voids"]]
reqs = sorted(set(boot[k]["req"] for k in boot))
print(f"igiene: tree_pids={trees} · richieste/boot nel log={reqs} (attese {EXPREQ}) · boot scartati e ripetuti: {' '.join(retried) if retried else 'nessuno'} · boot VOID finali: {len(voids)} {' '.join(voids)} · sentinella LS inizio: {LS0 or 'nessuno'} · fine: {LS1 or 'nessuno'}")
ghz = [f"c{c}-{a}={boot[(c,a)]['ghz']:.2f}GHz/IPC {boot[(c,a)]['ipc']:.2f}" for c in range(1, R + 1) for a in arms if boot[(c, a)]["ghz"]]
print("classe di core (companion, da time -l: cycles/(user+sys), instr/cycles per boot): " + " ".join(ghz))
rc = 0
if any(v != "200" for k in boot for v in boot[k]["status"].values()) or any(len(boot[k]["status"]) != len(urls) for k in boot): rc = 6
if "true" not in opc["O"]: print("GATE: opcache oracle NON attiva nel braccio O — run VOID"); rc = rc or 1
med = {}; medmy = {}
for a in arms:
    print(f"\nbraccio {a} = {label[a]} — unità: ms user-CPU/req del processo server; [per ciclo] mediana spread")
    for u in urls:
        cells = [boot[(c, a)]["cell"][u] for c in range(1, R + 1)]
        cpus = [x["cpu"] for x in cells if x.get("cpu") is not None]
        valid = all(x["ok"] for x in cells) and len(cpus) == R
        spread = (max(cpus) - min(cpus)) / statistics.mean(cpus) if len(cpus) >= 2 and statistics.mean(cpus) > 0 else None
        # asset = riga informativa (via statica, ~0,03 ms): mai gate di spread (criterio p.10, dry2)
        void = (not valid) or (u != "asset" and R >= 3 and spread is not None and spread > 0.02)
        if not void:
            med[(a, u)] = statistics.median(cpus)
            mys = [x["my"] for x in cells if x.get("my") is not None]
            if mys: medmy[(a, u)] = statistics.median(mys)
        else: rc = rc or 1
        sp = f"{spread*100:.1f}%" if spread is not None else "n/a"
        why = "" if valid else " INVALIDA(" + ";".join(f"c{i+1}:non200={x['non200']},n={x['n']}" + (f",{x['why']}" if x.get("why") else "") for i, x in enumerate(cells) if not x["ok"]) + ")"
        sysv = statistics.median([x["sys"] for x in cells if x.get("sys") is not None] or [0]); ins = statistics.median([x["instr"] for x in cells if x.get("instr") is not None] or [0])
        myv = statistics.median([x["my"] for x in cells if x.get("my") is not None] or [0]); dqv = statistics.median([x["dq"] for x in cells if x.get("dq", -1) >= 0] or [-1])
        wm = statistics.median([x["wmed"] for x in cells if x.get("wmed")] or [0]); w25 = statistics.median([x["wp25"] for x in cells if x.get("wp25")] or [0]); w75 = statistics.median([x["wp75"] for x in cells if x.get("wp75")] or [0])
        dc = ",".join(str(x["dconn"]) for x in cells)
        print(f"  {u:<6} cpu=[{' '.join(f'{v:.2f}' for v in cpus)}] med={statistics.median(cpus) if cpus else float('nan'):.2f} spread={sp} {'VOID' if void else 'ok'}{why}  sys={sysv:.2f} instr/req={ins/1e6:.2f}M mysqld(u+s)/req={myv:.2f} ΔQuestions/blocco={dqv:.0f} wall med={wm:.1f} p25={w25:.1f} p75={w75:.1f}  Δconn=[{dc}] (atteso {exp_conn(u)})")
    pk = " ".join(f"c{c}={boot[(c,a)]['peak_ru']/1048576:.0f}MiB" + (f"/tl={boot[(c,a)]['peak_tl']/1048576:.0f}" if boot[(c,a)]['peak_tl'] > 0 else "/tl=n/a") for c in range(1, R + 1))
    print(f"  peak lifetime_max_phys_footprint (intero processo) per boot: {pk}")
if all((("P", u) in med) and (("P0", u) in med) for u in urls):
    if all(abs(med[("P0", u)] - med[("P", u)]) / med[("P", u)] < 0.02 for u in urls):
        print("\ncontrollo positivo P0: |P0−P| <2% su OGNI URL ⇒ kill-switch PHPR_UNIT_CACHE=0 inefficace — braccio P0 VOID")
        for u in urls: med.pop(("P0", u), None)
        rc = rc or 1
def ratios(x, y, withmy=False):
    outl = []
    for u in urls:
        per = []
        for c in range(1, R + 1):
            cx, cy = boot[(c, x)]["cell"][u], boot[(c, y)]["cell"][u]
            if cx["ok"] and cy["ok"] and cx.get("cpu") and cy.get("cpu"):
                if withmy and cx.get("my") is not None and cy.get("my") is not None: per.append((cx["cpu"] + cx["my"]) / (cy["cpu"] + cy["my"]))
                elif not withmy: per.append(cx["cpu"] / cy["cpu"])
        tag = "" if ((x, u) in med and (y, u) in med) else " (cella VOID: direzione, non cifra)"
        outl.append(f"{u} [{' '.join(f'{v:.2f}' for v in per)}] med={statistics.median(per) if per else float('nan'):.2f}{tag}")
    return " · ".join(outl)
print("\nrapporti per ciclo (stesso ciclo) P/O: " + ratios("P", "O"))
print("equivalenza misurata Pphpr/P (php-server modo nativo vs phpr -S, stesso pin): " + ratios("Pphpr", "P"))
print("companion «pagina servita» (server+mysqld) P/O: " + ratios("P", "O", True))
print("rapporti P0/Ooff (entrambi senza cache; DIREZIONE: opcache.enable=0 spegne anche l'ottimizzatore Zend): " + ratios("P0", "Ooff"))
print("rapporti P0/P (lavoro evitato dalla cache unità): " + ratios("P0", "P"))
print("rapporti Ooff/O (lavoro evitato dall'opcache): " + ratios("Ooff", "O"))
if all(("P", u) in med and ("O", u) in med for u in urls):
    pa, oa = med[("P", "asset")], med[("O", "asset")]
    print("companion netto-asset P/O: " + " · ".join(f"{u}={(med[('P',u)]-pa)/(med[('O',u)]-oa):.2f}" for u in urls if u != "asset" and med[("O", u)] - oa > 0))
cmpP, cmpO = f"{out}/{T}/c1-P/rest.last.body", f"{out}/{T}/c1-O/rest.last.body"
par = ("BYTE-ID" if filecmp.cmp(cmpP, cmpO, shallow=False) else f"DIFF ({os.path.getsize(cmpP)} vs {os.path.getsize(cmpO)} B)") if os.path.exists(cmpP) and os.path.exists(cmpO) else "n/a"
print(f"parità avvisoria body rest c1 P vs O: {par}")
vc = [f"{a}/{u}" for a in arms for u in urls if (a, u) not in med]
print(f"celle VOID: {' '.join(vc) if vc else 'nessuna'}")
print(f"rc={rc}")
sys.exit(rc)
PY
JRC=$?
[ "$STATUS_BAD" = 1 ] && JRC=6
fin "$JRC" "giudice"
