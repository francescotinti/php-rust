#!/bin/bash
# s125-p2-chain.sh — catena DETACHED p.2: attende il .done della coppia WP
# (mai build in parallelo a misure di tempo), poi SEQUENZIALE:
# census-build → census-run → census-report → layout-build → layout-band.
# Il report census con rc=2 (grado mancato) NON ferma la banda (esito valido
# registrato); rc=1 di build/run ferma il rispettivo ramo.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp125-harness"
OUT="$H"; mkdir -p "$H/census-out" "$H/banda-out"
p(){ echo "$(date +%H:%M:%S) $1" >> "$H/p2-progress.txt"; }
: > "$H/p2-progress.txt"
rm -f "$H/p2-chain.done"

p "attesa pair-intercal.done (poll 60s)"
while [ ! -f "$H/pair-out/pair-intercal.done" ]; do sleep 60; done
p "coppia WP conclusa: $(cat "$H/pair-out/pair-intercal.done")"

RC=0
p "census-build START"
"$H/s125-census-build.sh" >> "$H/p2-progress.txt" 2>&1; b=$?
p "census-build rc=$b"
if [ "$b" = 0 ]; then
  p "census-run START"
  "$H/s125-census-run.sh" >> "$H/p2-progress.txt" 2>&1; r=$?
  p "census-run rc=$r"
  if [ "$r" = 0 ]; then
    "$H/s125-census-report.sh" >> "$H/p2-progress.txt" 2>&1; rep=$?
    p "census-report rc=$rep (0=confermata 2=grado-mancato)"
    [ "$rep" = 1 ] && RC=1
  else RC=1; fi
else RC=1; fi

p "layout-build START"
"$H/s125-layout-build.sh" >> "$H/p2-progress.txt" 2>&1; lb=$?
p "layout-build rc=$lb"
if [ "$lb" = 0 ]; then
  p "layout-band START"
  "$H/s125-layout-band.sh" >> "$H/p2-progress.txt" 2>&1; bb=$?
  p "layout-band rc=$bb"
  [ "$bb" = 0 ] || RC=1
else RC=1; fi

echo "rc=$RC $(date +%T)" > "$H/p2-chain.done"
p "DONE rc=$RC"
