#!/usr/bin/env python3
# gen-orm-t26.py — genera s184-lancio-orm-t26.sh come COPIA DICHIARATA di s184-lancio-pair-t25.sh (S-184 bis: t26 ORM-only
# @ pin s182 dopo ORM t25 rc=8 NON GIUDICABILE). Ogni sostituzione e' asserita UNICA; manifest s184-lancio-orm-t26-copia.diff.
import os, subprocess
H = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(H, "s184-lancio-pair-t25.sh"); DST = os.path.join(H, "s184-lancio-orm-t26.sh")
s = open(SRC).read()
def sub(old, new):
    global s
    assert s.count(old) == 1, f"NON unico ({s.count(old)}): {old[:70]!r}"
    s = s.replace(old, new)
sub("#!/bin/bash\n# s184-lancio-pair-t25.sh — COPPIA t25 @ pin s182",
    "#!/bin/bash\n# s184-lancio-orm-t26.sh — S-184 bis: t26 ORM-ONLY (dbal+ORM, s176-orm-coppia.sh emendato) @ pin s182, finestra PROPRIA dopo\n"
    "# l'ORM t25 rc=8 NON GIUDICABILE (sentinella oracle fuori banda + ictx oracle1). COPIA DICHIARATA di s184-lancio-pair-t25.sh\n"
    "# (manifest s184-lancio-orm-t26-copia.diff) coi SOLI adattamenti: log/esiti in orm-out (*-t26.*), lock TOKEN s184bis RIMOSSO\n"
    "# dal lanciatore a ORM finito (finestra = il solo ORM), watchdog disco riferito a rimisura-t26.done (10 min dopo, max 3 h),\n"
    "# e al posto del pair: archivio dei raw t25 di wp176-harness/orm-out in orm-out/t25-raw (verifica ls), pin == stash s182,\n"
    "# s176-orm-coppia.sh con PIN_ATTESO/MAPPA_SP, copia verdetto+done in orm-out. Gate di finestra INVARIATI (quiete CI,\n"
    "# anti-flare 6x30 s, loadavg1 <3 x6, E2 <150 % x4, processo singolo >50 % x2, dichiarazione uptime+top). NESSUNA igiene FS qui.\n"
    "# s184-lancio-pair-t25.sh — COPPIA t25 @ pin s182")
sub('LOG="$H/pair-out/lancio-t25.log"; mkdir -p "$H/pair-out"', 'LOG="$H/orm-out/lancio-t26.log"; mkdir -p "$H/orm-out"')
sub('echo "s184 coppia t25 @ pin s182 $(date \'+%F %T\') pid=$$" > "$MLOCK"\necho "$(date \'+%F %T\') lock scritto: $MLOCK (TOKEN s184)" >> "$LOG"',
    'echo "s184bis ORM t26 @ pin s182 $(date \'+%F %T\') pid=$$" > "$MLOCK"\necho "$(date \'+%F %T\') lock scritto: $MLOCK (TOKEN s184bis)" >> "$LOG"')
sub('( WD="$H/pair-out/watchdog-t25.txt"; PD="$H/pair-out/pair184-t25.done"; t0=$(date +%s); tdone=0',
    '( WD="$H/orm-out/watchdog-t26.txt"; PD="$H/orm-out/rimisura-t26.done"; t0=$(date +%s); tdone=0')
sub('    { [ "$tdone" != 0 ] && [ $((now-tdone)) -ge 14400 ]; } && break\n    [ $((now-t0)) -ge 50400 ] && break',
    '    { [ "$tdone" != 0 ] && [ $((now-tdone)) -ge 600 ]; } && break\n    [ $((now-t0)) -ge 10800 ] && break')
sub('Q="$H/pair-out/quiet-decl-t25.txt"\n{ echo "== dichiarazione finestra quieta t25 $(date \'+%F %T\') =="',
    'Q="$H/orm-out/quiet-decl-t26.txt"\n{ echo "== dichiarazione finestra quieta t26 ORM $(date \'+%F %T\') =="')
sub('echo "$(date \'+%F %T\') quiete raggiunta — dichiarazione in quiet-decl-t25.txt — lancio pair t25" >> "$LOG"\nexec /bin/bash "$H/s184-pair.sh" t25\n',
    'echo "$(date \'+%F %T\') quiete raggiunta — dichiarazione in quiet-decl-t26.txt — archivio raw t25 + lancio ORM t26" >> "$LOG"\n'
    'REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"\n'
    'DONE="$H/orm-out/rimisura-t26.done"; rm -f "$DONE"\n'
    'fine(){ echo "$(date \'+%F %T\') $1" >> "$LOG"; echo "$2" > "$DONE"; rm -f "$MLOCK"; exit "${2#rc=}"; }\n'
    '# archivio dei raw t25 (gamba 2 notturna + gamba 1 rimisurata + dbal): s176 sovrascrive orm-out\n'
    'rm -rf "$H/orm-out/t25-raw"; cp -R "$REPO/wp176-harness/orm-out" "$H/orm-out/t25-raw" || fine "archivio t25-raw FALLITO" "rc=8"\n'
    'ls "$H/orm-out/t25-raw/orm-phpr2.time" "$H/orm-out/t25-raw/orm-phpr1.time.t25-hang" > /dev/null || fine "archivio t25-raw incompleto" "rc=8"\n'
    'PIN_ATTESO=$(shasum -a 256 "$STASH/phpr-s182" | cut -c1-16); PC=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)\n'
    '[ "$PC" = "$PIN_ATTESO" ] || fine "pin canonico $PC != stash s182 $PIN_ATTESO" "rc=9"\n'
    'SPD=/private/tmp/phpr-s184-orm; mkdir -p "$SPD"\n'
    'echo "$(date \'+%F %T\') pin $PC == stash s182 — ORM t26 (MAPPA_SP=$SPD)" >> "$LOG"\n'
    'PIN_ATTESO="$PIN_ATTESO" MAPPA_SP="$SPD" /bin/bash "$REPO/wp176-harness/s176-orm-coppia.sh"\n'
    'rc=$?\n'
    'cp "$REPO/wp176-harness/s176-orm-coppia-verdetto.out" "$H/orm-out/s184-orm-coppia-verdetto-t26.out" 2>/dev/null\n'
    'fine "ORM t26 fine rc=$rc ($(cat "$REPO/wp176-harness/orm-out/rimisura.done" 2>/dev/null)) — verdetto in orm-out/s184-orm-coppia-verdetto-t26.out; lock rimosso" "$(cat "$REPO/wp176-harness/orm-out/rimisura.done" 2>/dev/null | awk \'{print $1}\')"\n')
open(DST, "w").write(s); os.chmod(DST, 0o755)
with open(os.path.join(H, "s184-lancio-orm-t26-copia.diff"), "w") as m:
    subprocess.run(["diff", SRC, DST], stdout=m)
print("generato", DST)
