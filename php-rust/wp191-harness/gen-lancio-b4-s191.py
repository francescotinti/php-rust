#!/usr/bin/env python3
# gen-lancio-b4-s191.py — genera s191-lancio-b4.sh come COPIA DICHIARATA di ../wp190-harness/s190-lancio-guardia.sh
# (manifest s191-lancio-b4-copia.diff; ogni sostituzione con assert di unicità). Scritto il 06/11 01:40 dopo il primo tentativo
# della guardia b4 nella catena s191 (rc=8 = s129 quiescenza FAIL: mediaanalysisd 9-14 % su due campioni, analisi Foto attiva).
# Adattamenti: (1) attende il lock ASSENTE prima di tutto (la catena s191 lo tiene fino alla fine) e poi scrive il proprio TOKEN
# s191b4; (2) A = stash phpr-s183, B = stash phpr-s184 (hash asseriti), Z riusato; (3) TAG=b4; (4) tentativi ×3 della guardia
# quando cade SOLO al gate s129 (verdetto «quiescenza FAIL»): il verdetto del tentativo fallito viene CONSERVATO come
# s186-guardia-verdetto-b4-tN.out e si ripete l'anti-flare 6x30 s prima di riprovare; un verdetto vero esistente ⇒ rc=7.
# Generato con assistenza AI (Claude Fable 5).
import os
os.chdir(os.path.dirname(os.path.abspath(__file__)) + '/..')

def rep(s, old, new, n=1):
    assert s.count(old) == n, (old[:80], s.count(old), n)
    return s.replace(old, new)

s = open('wp190-harness/s190-lancio-guardia.sh', encoding='utf-8').read()
head = '''#!/bin/bash
# s191-lancio-b4.sh — S-191: RIPETIZIONE della sola guardia b4 (A = stash phpr-s183 9a6ae11b, Z gemello S-186, B = stash phpr-s184
# 60b10d85, R=5) dopo il tentativo 1 caduto al gate s129 nella catena s191 (06/11 01:30:17, mediaanalysisd 9-14 %). Attende il
# lock ASSENTE (la catena s191 lo tiene fino a t26), poi gate di finestra EREDITATI -> lock TOKEN s191b4 -> hash degli stash ->
# s186-guardia.sh TAG=b4 con ×3 tentativi sul solo gate s129 (verdetti falliti conservati come -b4-tN) -> lock rimosso; esito in
# catena-out/catena-s191b4.done. COPIA DICHIARATA di ../wp190-harness/s190-lancio-guardia.sh (manifest s191-lancio-b4-copia.diff,
# generatore gen-lancio-b4-s191.py) coi SOLI adattamenti elencati nel generatore. Testo S-190/S-186/S-185/S-184 conservato sotto:
'''
s = head + s[s.index('# s190-lancio-guardia.sh — S-190'):]
s = rep(s, 'H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp190-harness"', 'H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp191-harness"')
s = rep(s, 'DONE="$H/catena-out/catena-s190.done"; rm -f "$DONE"', 'DONE="$H/catena-out/catena-s191b4.done"; rm -f "$DONE"')
# file propri (la catena s191 usa lancio-catena.log / watchdog-catena.txt / quiet-decl-catena.txt nella stessa cartella)
s = rep(s, 'LOG="$H/catena-out/lancio-catena.log"; mkdir -p "$H/catena-out"', 'LOG="$H/catena-out/lancio-b4.log"; mkdir -p "$H/catena-out"')
s = rep(s, 'WD="$H/catena-out/watchdog-catena.txt"', 'WD="$H/catena-out/watchdog-b4.txt"')
s = rep(s, 'Q="$H/catena-out/quiet-decl-catena.txt"', 'Q="$H/catena-out/quiet-decl-b4.txt"')
s = rep(s, 'ZB="$H186/ab-out/s186-leva/phpr-Z"; B3B="$H/ab-out/s190-leva/phpr-B3"',
        'STASH0="/Volumes/Extreme Pro/Claude/phpr-old-target/release"; ZB="$H186/ab-out/s186-leva/phpr-Z"; B3B="$STASH0/phpr-s184"\n'
        '# ADATTAMENTO S-191 (1): nessuna finestra finché un altro lock è presente (la catena s191 lo rimuove a fine t26)\n'
        'while [ -e /private/tmp/phpr-measure.lock ]; do sleep 120; done; echo "$(date \'+%F %T\') lock altrui assente: procedo" >> "$LOG"')
s = rep(s, 'echo "s190 guardia b3 (cura O(1) v2) @ pin s183 $(date \'+%F %T\') pid=$$" > "$MLOCK"', 'echo "s191b4 guardia b4 (pin s184 vs stash s183) $(date \'+%F %T\') pid=$$" > "$MLOCK"')
s = rep(s, 'lock scritto: $MLOCK (TOKEN s190)', 'lock scritto: $MLOCK (TOKEN s191b4)')
s = rep(s, 'PD="$H/catena-out/catena-s190.done"', 'PD="$H/catena-out/catena-s191b4.done"')
s = rep(s, '== dichiarazione finestra quieta S-190 (guardia b3)', '== dichiarazione finestra quieta S-191 (guardia b4, ripetizione)')
s = rep(s, 'quiete raggiunta — dichiarazione in quiet-decl-catena.txt — guardia b3', 'quiete raggiunta — dichiarazione in quiet-decl-b4.txt — guardia b4')
old_tail = s[s.index('PIN_ATTESO=$(shasum -a 256 "$STASH/phpr-s183" | cut -c1-16); PC='):]
assert old_tail.count('TAG=b3') >= 1 and old_tail.count('s186-guardia.sh') == 1  # la coda S-190 (log + chiamata) viene sostituita per intero
s = s[:s.index('PIN_ATTESO=$(shasum -a 256 "$STASH/phpr-s183" | cut -c1-16); PC=')] + '''PIN_ATTESO=$(shasum -a 256 "$STASH/phpr-s183" | cut -c1-16); PB=$(shasum -a 256 "$B3B" | cut -c1-16)
[ "$PIN_ATTESO" = "9a6ae11bf7e900ef" ] || fine "stash s183 $PIN_ATTESO != 9a6ae11bf7e900ef" "rc=9 stash183" 9
[ "$PB" = "60b10d857eb0343a" ] || fine "stash s184 $PB != 60b10d857eb0343a" "rc=9 stash184" 9
[ "$(shasum -a 256 "$ZB" | cut -c1-16)" = "23c0a94c26db62eb" ] || fine "gemello Z diverso" "rc=9 Z" 9
echo "$(date '+%F %T') stash s183 $PIN_ATTESO · stash s184 $PB · Z ok — GUARDIA b4 (A=stash s183, Z, B=stash s184), fino a 3 tentativi sul gate s129" >> "$LOG"
VB="$H186/s186-guardia-verdetto-b4.out"; rc1=7; GR=""
for n in 1 2 3; do
  if [ -e "$VB" ]; then
    if grep -q '^quiescenza FAIL' "$VB"; then k=1; while [ -e "$H186/s186-guardia-verdetto-b4-t$k.out" ]; do k=$((k+1)); done
      mv "$VB" "$H186/s186-guardia-verdetto-b4-t$k.out"; echo "$(date '+%F %T') verdetto b4 «quiescenza FAIL» conservato come -b4-t$k.out" >> "$LOG"
    else fine "verdetto b4 VERO già presente ($VB): non si ripete" "rc=7 verdetto-b4-esiste" 7; fi
  fi
  PIN="$STASH/phpr-s183" Z="$ZB" B="$B3B" R=5 RE3=3 TAG=b4 /bin/bash "$H186/s186-guardia.sh"
  rc1=$?; GR="$(cat "$H186/guardia-out-b4/guardia.done" 2>/dev/null)"
  if [ "$rc1" = 8 ] && grep -q '^quiescenza FAIL' "$VB" 2>/dev/null && [ "$n" -lt 3 ]; then
    echo "$(date '+%F %T') tentativo $n: s129 quiescenza FAIL — anti-flare 6x30 s e riprovo" >> "$LOG"
    calm=0; while [ "$calm" -lt 6 ]; do c=$(top -l 2 -stats pid,cpu,command 2>/dev/null | awk '/mediaanalysisd/ {v=$2} END {print (v==""?0:v)}'); if awk -v c="$c" 'BEGIN{exit !(c<5.0)}'; then calm=$((calm+1)); else calm=0; fi; [ "$calm" -lt 6 ] && sleep 30; done
    continue
  fi
  break
done
fine "guardia b4 fine rc=$rc1 ($GR) — verdetto wp186-harness/s186-guardia-verdetto-b4.out; lock rimosso" "rc=$rc1 guardia_b4=[$GR]" "$rc1"
'''
p = 'wp191-harness/s191-lancio-b4.sh'
open(p, 'w', encoding='utf-8').write(s); os.chmod(p, 0o755)
print('ok')
