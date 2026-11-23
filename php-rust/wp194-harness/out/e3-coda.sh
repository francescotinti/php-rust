#!/bin/bash
# e3-coda.sh — S-194 p.17 (cure of the divergences): full acceptance, then the probe step on the rebuilt arm against C2c.
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")/.." && pwd)"; H="$W/../wp192-harness"; rm -f "$W/out/e3-coda.done"
bash "$H/s192-accettazione.sh" s194E3 > "$W/out/e3-accettazione.log" 2>&1
a=$(cat "$H/out/accettazione-s194E3.rc" 2>/dev/null || echo 99)
bash "$W/s194-passo-C.sh" E3 s194-primitive-C2c.out
echo "accettazione=$a passo=$(cat "$W/out/passo-E3.done") $(date '+%F %T')" > "$W/out/e3-coda.done"
