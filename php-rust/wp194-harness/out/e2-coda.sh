#!/bin/bash
# e2-coda.sh — S-194 p.17 (cure of the divergences): full acceptance, then the probe step on the rebuilt arm against C2c.
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")/.." && pwd)"; H="$W/../wp192-harness"; rm -f "$W/out/e2-coda.done"
bash "$H/s192-accettazione.sh" s194E2 > "$W/out/e2-accettazione.log" 2>&1
a=$(cat "$H/out/accettazione-s194E2.rc" 2>/dev/null || echo 99)
echo "accettazione=$a passo=- $(date '+%F %T')" > "$W/out/e2-coda.done"
