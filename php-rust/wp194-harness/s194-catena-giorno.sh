#!/bin/bash
# s194-catena-giorno.sh <Z> <L> — S-194: day chain for levers a/b/c (criterion s194-criterio-leve-abc.md), one heavy step at a time:
# hygiene line, twins Z-Z (noise band of the instrument), twins Z-L, ORM profile, WordPress profile. Tools of S-192 unchanged.
# Outcomes: s194-catena-giorno.out (one line per step), out/catena-giorno.done. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$P/wp192-harness"; W="$P/wp194-harness"
Z="${1:?Z}"; L="${2:?L}"; V="$W/s194-catena-giorno.out"; mkdir -p "$W/out"; : > "$V"; rm -f "$W/out/catena-giorno.done"
cd "$P" || exit 2
igiene() { echo "igiene $1: $(date '+%F %T') Data=$(df -g /System/Volumes/Data | tail -1 | awk '{print $4}')G load=$(sysctl -n vm.loadavg) cargo=$(pgrep -x cargo | wc -l | tr -d ' ')" >> "$V"; }
passo() { n="$1"; shift; igiene "prima di $n"; "$@" > "$W/out/catena-$n.log" 2>&1; echo "$n: rc=$?" >> "$V"; }
passo gemelli-zz python3 "$H/s192-gemelli-orm.py" s194zz "$Z" "$Z"
passo gemelli-zl python3 "$H/s192-gemelli-orm.py" s194abc "$Z" "$L"
passo anatomia-orm python3 "$H/s192-anatomia.py" s194abc
passo anatomia-wp python3 "$H/s192-anatomia.py" s194w wp
igiene fine
echo "fine $(date '+%F %T')" >> "$V"; echo 0 > "$W/out/catena-giorno.done"
