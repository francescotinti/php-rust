#!/usr/bin/env python3
# gen-lancio-guardia-s190.py — genera s190-lancio-guardia.sh come COPIA DICHIARATA di
# ../wp186-harness/s186-lancio-catena.sh (assert di unicità su ogni sostituzione) e il manifest
# s190-lancio-guardia-copia.diff. Adattamenti SOLI: harness wp190/catena-out, TOKEN s190, bracci richiesti = Z (riuso
# wp186) + B3 (wp190), blocco finale = UNA guardia (s186-guardia.sh TAG=b3) al posto di guardia×2 + fetta 0, nessun
# SPD/MAPPA_SP, watchdog su catena-s190.done. Generato con assistenza AI (Claude Fable 5).
import os, subprocess, stat

H = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(H, "..", "wp186-harness", "s186-lancio-catena.sh")
DST = os.path.join(H, "s190-lancio-guardia.sh")
MAN = os.path.join(H, "s190-lancio-guardia-copia.diff")
s = open(SRC, encoding="utf-8").read()


def sub(old, new, count=1):
    global s
    assert s.count(old) == count, f"sostituzione non unica ({s.count(old)}): {old[:70]!r}"
    s = s.replace(old, new)


HEAD_OLD = "#!/bin/bash\n# s186-lancio-catena.sh — S-186: CATENA NOTTURNA UNICA"
HEAD_NEW = """#!/bin/bash
# s190-lancio-guardia.sh — S-190 cura O(1) v2: FINESTRA NOTTURNA della sola GUARDIA a buffer pieno (criterio
# s190-criterio-o1v2.md p.7): gate di finestra EREDITATI (quiete CI, anti-flare 6x30 s, loadavg1 <3 x6, E2 <150 % x4,
# processo singolo >50 % x2, dichiarazione uptime+top) -> lock TOKEN s190 -> pin s183 == stash per hash E bracci Z (riuso
# wp186) + B3 (wp190) presenti -> s186-guardia.sh TAG=b3 (B = phpr-B3) -> lock rimosso; esito in catena-out/catena-s190.done.
# COPIA DICHIARATA di ../wp186-harness/s186-lancio-catena.sh (manifest s190-lancio-guardia-copia.diff, generatore
# gen-lancio-guardia-s190.py) coi SOLI adattamenti elencati nel generatore. Testo S-186 conservato sotto:
# s186-lancio-catena.sh — S-186: CATENA NOTTURNA UNICA"""
sub(HEAD_OLD, HEAD_NEW)
sub('H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp186-harness"',
    'H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp190-harness"\nH186="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp186-harness"')
sub('DONE="$H/catena-out/catena-s186.done"', 'DONE="$H/catena-out/catena-s190.done"')
ARMS_OLD = """for b in Z B B2 M C; do
  [ -s "$H/ab-out/s186-leva/phpr-$b" ] || { echo "$(date '+%F %T') braccio $b ASSENTE (ab-out/s186-leva/phpr-$b): niente finestra" >> "$LOG"; echo "rc=7 braccio $b assente" > "$DONE"; exit 7; }
done
echo "$(date '+%F %T') bracci Z/B/B2/M/C presenti: $(cd "$H/ab-out/s186-leva" && shasum -a 256 phpr-Z phpr-B phpr-B2 phpr-M phpr-C | cut -c1-16 | tr '\\n' ' ')" >> "$LOG"
"""
ARMS_NEW = """ZB="$H186/ab-out/s186-leva/phpr-Z"; B3B="$H/ab-out/s190-leva/phpr-B3"
[ -s "$ZB" ] || { echo "$(date '+%F %T') braccio Z ASSENTE ($ZB): niente finestra" >> "$LOG"; echo "rc=7 braccio Z assente" > "$DONE"; exit 7; }
[ -s "$B3B" ] || { echo "$(date '+%F %T') braccio B3 ASSENTE ($B3B): niente finestra" >> "$LOG"; echo "rc=7 braccio B3 assente" > "$DONE"; exit 7; }
echo "$(date '+%F %T') bracci Z/B3 presenti: Z $(shasum -a 256 "$ZB" | cut -c1-16) B3 $(shasum -a 256 "$B3B" | cut -c1-16)" >> "$LOG"
"""
sub(ARMS_OLD, ARMS_NEW)
sub('echo "s186 catena guardia+fetta0 @ pin s183 $(date \'+%F %T\') pid=$$" > "$MLOCK"',
    'echo "s190 guardia b3 (cura O(1) v2) @ pin s183 $(date \'+%F %T\') pid=$$" > "$MLOCK"')
sub("(TOKEN s186)", "(TOKEN s190)")
sub('WD="$H/catena-out/watchdog-catena.txt"; PD="$H/catena-out/catena-s186.done"',
    'WD="$H/catena-out/watchdog-catena.txt"; PD="$H/catena-out/catena-s190.done"')
sub("== dichiarazione finestra quieta catena S-186 (guardia + fetta 0)", "== dichiarazione finestra quieta S-190 (guardia b3)")
sub("quiete raggiunta — dichiarazione in quiet-decl-catena.txt — guardia a buffer pieno, poi fetta 0",
    "quiete raggiunta — dichiarazione in quiet-decl-catena.txt — guardia b3")
i = s.index("SPD=/private/tmp/phpr-s186-orm; mkdir -p \"$SPD\"")
TAIL_NEW = """echo "$(date '+%F %T') pin $PC == stash s183 — GUARDIA a buffer pieno TAG=b3 (PIN=pin, Z=$ZB, B=$B3B)" >> "$LOG"
# guardia della cura O(1) v2 (criterio s190 p.7): stesso giudice di S-186, braccio B = phpr-B3
PIN="$HOME/Claude/php-rust-output/release/phpr" Z="$ZB" B="$B3B" R=5 RE3=3 TAG=b3 /bin/bash "$H186/s186-guardia.sh"
rc1=$?
GR="$(cat "$H186/guardia-out-b3/guardia.done" 2>/dev/null)"
fine "guardia b3 fine rc=$rc1 ($GR) — verdetto wp186-harness/s186-guardia-verdetto-b3.out; lock rimosso" "rc=$rc1 guardia_b3=[$GR]" "$rc1"
"""
s = s[:i] + TAIL_NEW
with open(DST, "w", encoding="utf-8") as f:
    f.write(s)
os.chmod(DST, os.stat(DST).st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
with open(MAN, "w", encoding="utf-8") as f:
    subprocess.run(["diff", "-u", SRC, DST], stdout=f, check=False)
print("scritto", os.path.basename(DST), "+ manifest", os.path.basename(MAN))
