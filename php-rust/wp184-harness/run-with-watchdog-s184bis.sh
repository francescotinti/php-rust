#!/bin/bash
# run-with-watchdog.sh — esegue un comando con watchdog anti-hang (lezione WP-13:
# il gruppo restapi appeso 3 volte da 40+ min per un loop infinito nel motore).
#
# Due criteri di kill, il PROGRESSO vince sul tempo totale:
#   -s STALL_SECS  : kill dopo N secondi SENZA progresso (richiede -p; default 300)
#   -t TOTAL_SECS  : budget totale wall-clock (default 1800; sempre attivo)
#   -p FILE        : file di progresso da osservare (es. TESTLOG/junit); se la
#                    size cambia il timer di stallo riparte. Senza -p vale solo -t.
#   -o DIR         : dove salvare il post-mortem (default: cwd)
#
# Prima di uccidere fa `sample <pid>` (5s) → DIR/watchdog-sample-<epoch>.txt:
# un hang futuro arriva già col post-mortem, non con un processo morto e zero
# indizi. Exit: 124 su kill da watchdog, altrimenti l'exit code del comando.
#
# Uso: run-with-watchdog.sh [-t s] [-s s] [-p file] [-o dir] -- cmd args...
# EMENDA S-184 bis (2026-10-08): post-mortem e kill estesi ai DISCENDENTI del PID (vedi postmortem_and_kill).
set -u
TOTAL=1800; STALL=300; PROGRESS=""; OUTDIR="."
while [ $# -gt 0 ]; do
  case "$1" in
    -t) TOTAL=$2; shift 2 ;;
    -s) STALL=$2; shift 2 ;;
    -p) PROGRESS=$2; shift 2 ;;
    -o) OUTDIR=$2; shift 2 ;;
    --) shift; break ;;
    *) echo "run-with-watchdog: opzione ignota $1" >&2; exit 2 ;;
  esac
done
[ $# -gt 0 ] || { echo "run-with-watchdog: manca il comando dopo --" >&2; exit 2; }

"$@" &
PID=$!
START=$(date +%s)
prev_size=-1
stall_since=$START

descendants() { # $1 = pid -> tutti i discendenti, il piu' profondo per primo (bash 3.2, ricorsivo)
  local c
  for c in $(pgrep -P "$1" 2>/dev/null); do
    descendants "$c"
    echo "$c"
  done
}

postmortem_and_kill() { # $1 = motivo
  # EMENDA S-184 bis (2026-10-08, dichiarata): la gamba ORM phpr1 della coppia t25 ha
  # completato la suite e NON e' uscita; il sample colpiva /usr/bin/time (il wrapper
  # lanciato dal comando), non phpr: post-mortem cieco. Ora si campionano PRIMA tutti i
  # discendenti (il figlio reale e' il piu' profondo), poi il PID, e si uccidono anche i
  # discendenti (time non inoltra i segnali: mai piu' orfani senza indizi).
  local ts d kids nm; ts=$(date +%s)
  kids=$(descendants "$PID")
  echo "run-with-watchdog: $1 — sample + kill di PID $PID e discendenti [$(echo $kids | tr '\n' ' ')] ($*)" >&2
  for d in $kids; do
    nm=$(basename "$(ps -o comm= -p "$d" 2>/dev/null)" 2>/dev/null); nm=${nm:-sconosciuto}
    sample "$d" 3 -file "$OUTDIR/watchdog-sample-$ts-$d-$nm.txt" >/dev/null 2>&1
  done
  sample "$PID" 3 -file "$OUTDIR/watchdog-sample-$ts.txt" >/dev/null 2>&1
  for d in $kids; do kill "$d" 2>/dev/null; done
  kill "$PID" 2>/dev/null
  sleep 3
  for d in $kids; do kill -9 "$d" 2>/dev/null; done
  kill -9 "$PID" 2>/dev/null
  wait "$PID" 2>/dev/null
  echo "run-with-watchdog: post-mortem in $OUTDIR/watchdog-sample-$ts*.txt" >&2
  exit 124
}

while kill -0 "$PID" 2>/dev/null; do
  now=$(date +%s)
  if [ $((now - START)) -ge "$TOTAL" ]; then
    postmortem_and_kill "budget totale ${TOTAL}s esaurito"
  fi
  if [ -n "$PROGRESS" ]; then
    cur_size=$(stat -f%z "$PROGRESS" 2>/dev/null || echo -1)
    if [ "$cur_size" != "$prev_size" ]; then
      prev_size=$cur_size
      stall_since=$now
    elif [ $((now - stall_since)) -ge "$STALL" ]; then
      postmortem_and_kill "nessun progresso su $PROGRESS da ${STALL}s"
    fi
  fi
  sleep 5
done
wait "$PID"
exit $?
