#!/usr/bin/env python3
# gen-lancio-catena-s186.py — genera s186-lancio-catena.sh come COPIA DICHIARATA di
# ../wp185-harness/s185-lancio-catena.sh (assert di unicità su ogni sostituzione) e il manifest
# s186-lancio-catena-copia.diff. Generato con assistenza AI (Claude Fable 5).
import os, subprocess, sys

H = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(H, "..", "wp185-harness", "s185-lancio-catena.sh")
DST = os.path.join(H, "s186-lancio-catena.sh")
MAN = os.path.join(H, "s186-lancio-catena-copia.diff")
s = open(SRC, encoding="utf-8").read()

def sub(old, new, count=1):
    global s
    assert s.count(old) == count, f"sostituzione non unica ({s.count(old)}): {old[:70]!r}"
    s = s.replace(old, new)

HEAD_OLD = """#!/bin/bash
# s185-lancio-catena.sh — S-185: CATENA NOTTURNA UNICA"""
HEAD_NEW = """#!/bin/bash
# s186-lancio-catena.sh — S-186: CATENA NOTTURNA UNICA (delibera concilio S-186 firmata 2026-10-23; criterio
# s186-criterio-fetta0.md): gate di finestra EREDITATI (quiete CI, anti-flare 6x30 s, loadavg1 <3 x6, E2 <150 % x4,
# processo singolo >50 % x2, dichiarazione uptime+top) -> lock TOKEN s186 -> pin s183 == stash per hash E bracci
# Z/B/B2/M/C presenti -> (1) GUARDIA a buffer pieno DUE volte (s186-guardia.sh: ret-buf-dq K0/4/16, Hoare E3, mcalls/calls
# ABAB tree vs s183, micro R=5, obj-churn; TAG=b con phpr-B = cura 22/10 com'è, TAG=b2 con phpr-B2 = + cura O(1)) -> (2) FETTA 0 (s186-fetta0.sh: xctrace O/P/M + C census) -> lock rimosso;
# esiti SOLO in catena-out/catena-s186.done. COPIA DICHIARATA di ../wp185-harness/s185-lancio-catena.sh (manifest
# s186-lancio-catena-copia.diff, generatore gen-lancio-catena-s186.py) coi SOLI adattamenti: harness wp186/catena-out,
# TOKEN s186, pin atteso = stash phpr-s183 (9a6ae11bf7e900ef) al posto di s182+s181, controllo bracci PRIMA del lock,
# SPD phpr-s186-orm, blocco finale (guardia + fetta 0 al posto di istruttoria + t27; niente E5), watchdog disco su
# catena-s186.done; fine() a 3 argomenti INVARIATA. NESSUNA igiene FS qui. Testo S-185 conservato sotto:
# s185-lancio-catena.sh — S-185: CATENA NOTTURNA UNICA"""
sub(HEAD_OLD, HEAD_NEW)
sub('H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp185-harness"',
    'H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp186-harness"')
# controllo bracci PRIMA della quiete CI e del lock (rc=7 senza aprire la finestra)
sub('''LOG="$H/catena-out/lancio-catena.log"; mkdir -p "$H/catena-out"
echo "$(date '+%F %T') attesa quiete CI" >> "$LOG"''',
    '''LOG="$H/catena-out/lancio-catena.log"; mkdir -p "$H/catena-out"
DONE="$H/catena-out/catena-s186.done"; rm -f "$DONE"
# ADATTAMENTO S-186: i bracci della build serale (s186-leva-build.sh) devono esistere PRIMA di aprire la finestra
for b in Z B B2 M C; do
  [ -s "$H/ab-out/s186-leva/phpr-$b" ] || { echo "$(date '+%F %T') braccio $b ASSENTE (ab-out/s186-leva/phpr-$b): niente finestra" >> "$LOG"; echo "rc=7 braccio $b assente" > "$DONE"; exit 7; }
done
echo "$(date '+%F %T') bracci Z/B/B2/M/C presenti: $(cd "$H/ab-out/s186-leva" && shasum -a 256 phpr-Z phpr-B phpr-B2 phpr-M phpr-C | cut -c1-16 | tr '\\n' ' ')" >> "$LOG"
echo "$(date '+%F %T') attesa quiete CI" >> "$LOG"''')
sub('''echo "s185 catena istruttoria+t27 @ pin s182 $(date '+%F %T') pid=$$" > "$MLOCK"
echo "$(date '+%F %T') lock scritto: $MLOCK (TOKEN s185)" >> "$LOG"''',
    '''echo "s186 catena guardia+fetta0 @ pin s183 $(date '+%F %T') pid=$$" > "$MLOCK"
echo "$(date '+%F %T') lock scritto: $MLOCK (TOKEN s186)" >> "$LOG"''')
sub('''( WD="$H/catena-out/watchdog-catena.txt"; PD="$H/catena-out/catena-s185.done"; t0=$(date +%s); tdone=0''',
    '''( WD="$H/catena-out/watchdog-catena.txt"; PD="$H/catena-out/catena-s186.done"; t0=$(date +%s); tdone=0''')
sub('''{ echo "== dichiarazione finestra quieta catena S-185 (istruttoria + t27) $(date '+%F %T') =="''',
    '''{ echo "== dichiarazione finestra quieta catena S-186 (guardia + fetta 0) $(date '+%F %T') =="''')
sub('''echo "$(date '+%F %T') quiete raggiunta — dichiarazione in quiet-decl-catena.txt — istruttoria 3 bracci, poi t27" >> "$LOG"''',
    '''echo "$(date '+%F %T') quiete raggiunta — dichiarazione in quiet-decl-catena.txt — guardia a buffer pieno, poi fetta 0" >> "$LOG"''')
TAIL_OLD = s[s.index('DONE="$H/catena-out/catena-s185.done"; rm -f "$DONE"'):]
TAIL_NEW = '''fine(){ echo "$(date '+%F %T') $1" >> "$LOG"; echo "$2" > "$DONE"; rm -f "$MLOCK"; exit "$3"; }
PIN_ATTESO=$(shasum -a 256 "$STASH/phpr-s183" | cut -c1-16); PC=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)
[ "$PC" = "$PIN_ATTESO" ] || fine "pin canonico $PC != stash s183 $PIN_ATTESO" "rc=9 pin" 9
[ "$PIN_ATTESO" = "9a6ae11bf7e900ef" ] || fine "stash s183 $PIN_ATTESO != 9a6ae11bf7e900ef" "rc=9 stash183" 9
SPD=/private/tmp/phpr-s186-orm; mkdir -p "$SPD"
echo "$(date '+%F %T') pin $PC == stash s183 — GUARDIA a buffer pieno ×2 (PIN=pin, Z/B/B2=ab-out/s186-leva, TAG b poi b2)" >> "$LOG"
# (1) guardia della cura 22/10 a buffer PIENO (ordine obbligato team-governo p.2), DUE corse in sequenza:
#     TAG=b  con B = phpr-B  (tree con la cura 22/10 com'è)           -> guardia-out-b/guardia.done, s186-guardia-verdetto-b.out
#     TAG=b2 con B = phpr-B2 (tree + cura O(1), concilio Q3 accolta)   -> guardia-out-b2/guardia.done, s186-guardia-verdetto-b2.out
PIN="$HOME/Claude/php-rust-output/release/phpr" Z="$H/ab-out/s186-leva/phpr-Z" B="$H/ab-out/s186-leva/phpr-B" R=5 RE3=3 TAG=b /bin/bash "$H/s186-guardia.sh"
rc1=$?
GR="$(cat "$H/guardia-out-b/guardia.done" 2>/dev/null)"
echo "$(date '+%F %T') guardia b fine rc=$rc1 ($GR) — verdetto s186-guardia-verdetto-b.out" >> "$LOG"
PIN="$HOME/Claude/php-rust-output/release/phpr" Z="$H/ab-out/s186-leva/phpr-Z" B="$H/ab-out/s186-leva/phpr-B2" R=5 RE3=3 TAG=b2 /bin/bash "$H/s186-guardia.sh"
rc1b=$?
GR2="$(cat "$H/guardia-out-b2/guardia.done" 2>/dev/null)"
echo "$(date '+%F %T') guardia b2 fine rc=$rc1b ($GR2) — verdetto s186-guardia-verdetto-b2.out" >> "$LOG"
# (2) fetta 0: xctrace O/P/M R=3 + C census (criterio s186-criterio-fetta0.md); esito in fetta0-out/fetta0.done
echo "$(date '+%F %T') fetta 0 (MAPPA_SP=$SPD)" >> "$LOG"
PIN_ATTESO="$PIN_ATTESO" MAPPA_SP="$SPD" /bin/bash "$H/s186-fetta0.sh"
rc2=$?
FR="$(cat "$H/fetta0-out/fetta0.done" 2>/dev/null)"
fine "fetta 0 fine rc=$rc2 ($FR) — verdetto s186-fetta0-verdetto.out; lock rimosso" "rc=$rc2 guardia_b=[$GR] guardia_b2=[$GR2] fetta0=[$FR]" "$rc2"
'''
s = s.replace(TAIL_OLD, TAIL_NEW)
assert "$this" not in s and "$nome" not in s
open(DST, "w", encoding="utf-8").write(s)
os.chmod(DST, 0o755)
with open(MAN, "w", encoding="utf-8") as f:
    subprocess.run(["diff", SRC, DST], stdout=f)
print("scritto", DST, "manifest", MAN)
