#!/bin/bash
# mon-s184.sh — sonda di sola lettura per il Monitor di sessione: emette una riga per evento (coordinatore, bracci,
# lanciatore, verdetto). Vive in un FILE perché l'argv di un `bash -c` inline conteneva i nomi dei copioni e faceva
# scattare il pgrep del coordinatore («build/lanciatore già vivi»: incidente #2 S-184, auto-match, veto S-182).
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp184-harness"
LB="$H/ab-out/s184-leva.done"; LV="$H/s184-leva-build-verdetto.out"; LL="$H/ab-out/lancio-rt2.log"; LD="$H/ab-out/lancio-rt2.done"; AV="$H/ab-out/avvio-s184.log"; VR="$H/s184-rt2-verdetto.out"
seen_lb=0; seen_l=0; la=$(wc -l < "$AV" 2>/dev/null | tr -d ' '); la=${la:-0}; lc=0
while :; do
  n=$(wc -l < "$AV" 2>/dev/null | tr -d ' '); n=${n:-0}
  if [ "$n" -gt "$la" ]; then tail -n $((n-la)) "$AV" | sed 's/^/AVVIO: /'; la=$n; fi
  if [ $seen_lb = 0 ] && [ -e "$LB" ]; then seen_lb=1; echo "BRACCI: $(cat "$LB") $(tail -1 "$LV")"; fi
  m=$(wc -l < "$LL" 2>/dev/null | tr -d ' '); m=${m:-0}
  if [ "$m" -gt "$lc" ]; then tail -n $((m-lc)) "$LL" | grep -vE 'calma CPU|Data .*attesa|loadavg' | sed 's/^/LANCIO: /'; lc=$m; fi
  if [ $seen_l = 0 ] && [ -e "$LD" ]; then seen_l=1; echo "LANCIO DONE: $(cat "$LD")"; echo "VERDETTO: $(grep -E '^ESITO|SOLO DIREZIONE|PROMOZIONE|KILL' "$VR" 2>/dev/null | tail -2 | tr '\n' ' ')"; exit 0; fi
  sleep 30
done
