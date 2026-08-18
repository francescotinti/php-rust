#!/bin/bash
# s154-lancio-abce1b.sh — attende l'assestamento di mediaanalysisd (STREAK di
# 4 campioni consecutivi <5% a passo 5 s, emenda pair t3/t4) poi lancia il
# record R=5 con tag ab-ce1b. Argv NEUTRO (niente "phpr": lezione smoke-ce1).
set -u
export PATH=/usr/bin:/bin:/usr/sbin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp154-harness"
LOG="$H/ab-out/lancio-abce1b.log"
echo "$(date '+%F %T') attesa assestamento mediaanalysisd" >> "$LOG"
streak=0
while [ "$streak" -lt 4 ]; do
  c=$(ps -axo pcpu,comm | awk '/mediaanalysisd$/{s+=$1} END{print s+0}')
  if awk -v c="$c" 'BEGIN{exit !(c<5)}'; then streak=$((streak+1)); else streak=0; fi
  sleep 5
done
echo "$(date '+%F %T') assestato — lancio R=5 ab-ce1b" >> "$LOG"
exec /bin/bash "$H/s154-ab-ce1.sh" /private/tmp/s154-armB/candB e634d95c ab-ce1b 5 24.0
