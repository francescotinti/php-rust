#!/usr/bin/env python3
# gen-notte-s194.py <hash16 del pin s186> — genera i lanciatori della NOTTE di S-194 (criterio s194-criterio-notte.md) come COPIE
# DICHIARATE di tre script di ../wp193-harness, ogni sostituzione con assert di unicità, manifest = diff vecchio/nuovo:
#   s194-lancio-notte.sh    <- s193-lancio-notte.sh    (catena unica: gate di finestra, lock TOKEN s194, 3 passi + confronto)
#   s194-orm-coppia-t30.sh  <- s193-orm-coppia-t29.sh  (coppia dbal+ORM t30 @ pin s186)
#   s194-pair.sh            <- s193-pair.sh            (coppia WP full+media t28 @ pin s186)
# Nessuna coppia server (il server s191 non è stato toccato). Generato con assistenza AI (Claude Fable 5.2).
import os, re, subprocess, sys
PIN = sys.argv[1]; assert re.fullmatch(r'[0-9a-f]{16}', PIN), PIN
os.chdir(os.path.dirname(os.path.abspath(__file__)) + '/..')

def rep(s, old, new, n=1):
    assert s.count(old) == n, (old[:80], s.count(old), n)
    return s.replace(old, new)

def rep_all(s, old, new, at_least=1):
    c = s.count(old); assert c >= at_least, (old[:80], c)
    return s.replace(old, new)

def out(name, src, text):
    p = 'wp194-harness/' + name
    open(p, 'w', encoding='utf-8').write(text); os.chmod(p, 0o755)
    d = subprocess.run(['diff', src, p], capture_output=True, text=True).stdout
    open(p[:-3] + '-copia.diff', 'w', encoding='utf-8').write(d)

# ---------------------------------------------------------------- 1. catena unica
SRC = 'wp193-harness/s193-lancio-notte.sh'
s = open(SRC, encoding='utf-8').read()
head = '''#!/bin/bash
# s194-lancio-notte.sh — NOTTE di S-194 (criterio wp194-harness/s194-criterio-notte.md): CATENA UNICA, un solo lock TOKEN s194,
# gate di finestra EREDITATI, poi in ORDINE: (1) guardia b6 = s186-guardia.sh INVARIATO con A = stash phpr-s183, Z gemello S-186,
# B = binario CANONICO (pin s186) · (2) coppia dbal+ORM t30 @ pin s186 (costanti E5 INVARIATE) · (3) coppia WP full+media t28
# @ pin s186 per ULTIMA · poi s194-notte-confronto.py (regola esiti → conferma contro la notte del 20/11).
# COPIA DICHIARATA di ../wp193-harness/s193-lancio-notte.sh (manifest s194-lancio-notte-copia.diff, generatore gen-notte-s194.py)
# coi SOLI adattamenti: harness wp194, TOKEN s194, predicato anti-flare su mediaanalysisd E photolibraryd (delibera 20/11),
# controlli di hash (stash s183/s185/s186, canoniche phpr s186 e server s191, gemello Z), blocco finale coi tre passi di S-194
# (tag b6/t30/t28; nessuna coppia server) e il confronto. Testo S-193/S-191/S-185/S-184 sotto:
'''
s = head + s[s.index('set -u\n'):]
s = rep(s, 'php-rust/wp193-harness"', 'php-rust/wp194-harness"')
s = rep(s, 'echo "s193 notte b5+t29+server-t4+t27 @ pin s185 $(date', 'echo "s194 notte b6+t30+t28 @ pin s186 $(date')
s = rep(s, 'lock scritto: $MLOCK (TOKEN s193)', 'lock scritto: $MLOCK (TOKEN s194)')
s = rep(s, '''awk '/mediaanalysisd/ {v=$2} END {print (v==""?0:v)}')''', '''awk '/mediaanalysisd/ {a=$2} /photolibraryd/ {b=$2} END {a+=0; b+=0; print (a>b?a:b)}')''')
s = rep(s, 'mediaanalysisd=$c calm=$calm', 'mediaanalysisd/photolibraryd(max)=$c calm=$calm')
s = rep(s, 'catena-out/catena-s193.done"', 'catena-out/catena-s194.done"', 2)
s = rep(s, '== dichiarazione finestra quieta NOTTE S-193 (b5, t29, server t4, t27)', '== dichiarazione finestra quieta NOTTE S-194 (b6, t30, t28)')
s = rep(s, '— (1) guardia b5, (2) t29, (3) server t4, (4) t27" >> "$LOG"', '— (1) guardia b6, (2) t30, (3) t28" >> "$LOG"')
anchor = 'PIN183=$(shasum -a 256 "$STASH/phpr-s183" | cut -c1-16)'
assert s.count(anchor) == 1
s = s[:s.index(anchor)] + '''PIN183=$(shasum -a 256 "$STASH/phpr-s183" | cut -c1-16); PIN185=$(shasum -a 256 "$STASH/phpr-s185" | cut -c1-16); PIN186=$(shasum -a 256 "$STASH/phpr-s186" | cut -c1-16)
CANON="$HOME/Claude/php-rust-output/release/phpr"
PC=$(shasum -a 256 "$CANON" | cut -c1-16)
SRVC=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/php-server" | cut -c1-16)
[ "$PIN183" = "9a6ae11bf7e900ef" ] || fine "stash s183 $PIN183 != 9a6ae11bf7e900ef" "rc=9 stash183" 9
[ "$PIN185" = "d9d6429301f60f4e" ] || fine "stash s185 $PIN185 != d9d6429301f60f4e" "rc=9 stash185" 9
[ "$PIN186" = "@PIN@" ] || fine "stash s186 $PIN186 != @PIN@" "rc=9 stash186" 9
[ "$PC" = "$PIN186" ] || fine "pin canonico $PC != stash s186 $PIN186" "rc=9 pin" 9
[ "$SRVC" = "c413398a44f9b0a4" ] || fine "server canonico $SRVC != s191 c413398a44f9b0a4" "rc=9 server" 9
ZB="$REPO/wp186-harness/ab-out/s186-leva/phpr-Z"
[ "$(shasum -a 256 "$ZB" 2>/dev/null | cut -c1-16)" = "23c0a94c26db62eb" ] || fine "gemello Z assente o diverso ($ZB)" "rc=9 Z" 9
[ -e "$REPO/wp186-harness/s186-guardia-verdetto-b6.out" ] && fine "verdetto b6 ESISTE già: spostarlo prima" "rc=7 verdetto-b6" 7
echo "$(date '+%F %T') pin canonico $PC == stash s186 · stash s183 $PIN183 · server $SRVC (s191, invariato) · Z ok — (1) GUARDIA b6 (A=stash s183, Z, B=CANONICO s186, R=5)" >> "$LOG"
PIN="$STASH/phpr-s183" Z="$ZB" B="$CANON" R=5 RE3=3 TAG=b6 /bin/bash "$REPO/wp186-harness/s186-guardia.sh"
rc1=$?; GR="$(cat "$REPO/wp186-harness/guardia-out-b6/guardia.done" 2>/dev/null)"
echo "$(date '+%F %T') guardia b6 fine rc=$rc1 ($GR) — verdetto wp186-harness/s186-guardia-verdetto-b6.out — (2) ORM t30 (E5 INVARIATE: ORA_REF2 4,870 SENT [4,81;4,93]; REF2 [34,23;34,31])" >> "$LOG"
SPD=/private/tmp/phpr-s194-orm; mkdir -p "$SPD"
export E5=1 ORA_REF2=4.870 SENT_LO=4.81 SENT_HI=4.93 REF2_MIN=34.23 REF2_MAX=34.31
PIN_ATTESO="$PIN186" MAPPA_SP="$SPD" /bin/bash "$H/s194-orm-coppia-t30.sh"
rc2=$?; TR="$(cat "$H/orm-out/rimisura.done" 2>/dev/null)"
echo "$(date '+%F %T') t30 fine rc=$rc2 ($TR) — verdetto s194-orm-coppia-verdetto-t30.out — (3) WP t28 full+media @ pin s186 (ULTIMA: le gambe media risvegliano mediaanalysisd)" >> "$LOG"
PIN_ATTESO="$PIN186" SRV_ATTESO="$SRVC" /bin/bash "$H/s194-pair.sh" t28
rc3=$?; PR="$(cat "$H/pair-out/pair194-t28.done" 2>/dev/null)"
python3 "$H/s194-notte-confronto.py" >> "$LOG" 2>&1
RC=0; for r in "$rc1" "$rc2" "$rc3"; do [ "$r" -gt "$RC" ] && RC=$r; done
fine "notte fine: b6 rc=$rc1 · t30 rc=$rc2 · t28 rc=$rc3; confronto in s194-notte-confronto.out; lock rimosso" "rc=$RC guardia_b6=[$GR] t30=[$TR] t28=[$PR]" "$RC"
'''.replace('@PIN@', PIN)
b = s[s.index('set -u\n'):]
for bad in ('TOKEN s193', 'catena-s193', 'wp193', 't29', 't27', 'b5 ', 'TAG=b5', 'server t4', 's190'):
    assert bad not in b, bad
out('s194-lancio-notte.sh', SRC, s)

# ---------------------------------------------------------------- 2. ORM t30
SRC = 'wp193-harness/s193-orm-coppia-t29.sh'
o = open(SRC, encoding='utf-8').read()
o = rep(o, '#!/bin/bash\n', '#!/bin/bash\n'
        '# s194-orm-coppia-t30.sh — S-194: coppia dbal+ORM t30 @ pin s186 (DOVUTA: pin nuovo con le leve di S-194; t29 era @ s185).\n'
        '# COPIA DICHIARATA di ../wp193-harness/s193-orm-coppia-t29.sh (manifest s194-orm-coppia-t30-copia.diff, generatore\n'
        '# gen-notte-s194.py) coi SOLI adattamenti: harness wp194 (orm-out, verdetto s194-orm-coppia-verdetto-t30.out), PIN_ATTESO\n'
        '# default s186, etichette t30/s186. Costanti E5 dal lanciatore, INVARIATE. Testo S-193/S-191/S-185 sotto:\n')
o = rep(o, 'H="$REPO/wp193-harness"', 'H="$REPO/wp194-harness"')
o = rep(o, 'VERD="$H/s193-orm-coppia-verdetto-t29.out"', 'VERD="$H/s194-orm-coppia-verdetto-t30.out"')
o = rep(o, 'PIN_ATTESO="${PIN_ATTESO:-d9d6429301f60f4e}"', 'PIN_ATTESO="${PIN_ATTESO:-' + PIN + '}"')
o = rep(o, 'print("coppia-t29 @ s185 (user CPU;', 'print("coppia-t30 @ s186 (user CPU;')
o = rep(o, '{ echo "== s193 coppia t29 dbal+ORM (pin s185 MISURATO $PINM', '{ echo "== s194 coppia t30 dbal+ORM (pin s186 MISURATO $PINM')
out('s194-orm-coppia-t30.sh', SRC, o)

# ---------------------------------------------------------------- 3. WP t28
SRC = 'wp193-harness/s193-pair.sh'
p = open(SRC, encoding='utf-8').read()
p = rep(p, '#!/bin/bash\n', '#!/bin/bash\n'
        '# s194-pair.sh <tentativo> — S-194: coppia full/media WP t28 @ pin s186 (DOVUTA: pin nuovo con le leve di S-194; t27 era\n'
        '# @ s185). COPIA DICHIARATA di ../wp193-harness/s193-pair.sh (manifest s194-pair-copia.diff, generatore gen-notte-s194.py)\n'
        '# coi SOLI adattamenti: nomi s194/pair194/t28; harness wp194; mediane storiche + t27=1,705 (banda di giudizio INVARIATA\n'
        '# [1,738;1,799]: il confronto con t27 lo fa s194-notte-confronto.py); messaggi riferiti al pin s186. Testo S-193 sotto:\n')
p = rep(p, 'H="$REPO/wp193-harness"', 'H="$REPO/wp194-harness"')
p = rep(p, 'VERD="$H/s193-pair-verdetto-$T.out"', 'VERD="$H/s194-pair-verdetto-$T.out"')
p = rep_all(p, 'pair193-', 'pair194-')
p = rep(p, '== s193 coppia t27: WP full+media @ pin s185 MISURATO', '== s194 coppia t28: WP full+media @ pin s186 MISURATO')
p = rep(p, "vs t26=1.751: {'≤' if med_t27 <= 1.751 else '>'}", "vs t27=1.705: {'≤' if med_t27 <= 1.705 else '>'}")
p = rep(p, 't25=1.747 t26=1.751', 't25=1.747 t26=1.751 t27=1.705', 2)
p = rep_all(p, 'med_t27', 'med_t28')
p = rep(p, 'mediana_t27=', 'mediana_t28=')
p = rep_all(p, 'pin s185 (sei leve di S-192: attribuzione per leva)', 'pin s186 (leve di S-194: attribuzione per leva)')
p = rep_all(p, 'pin NUOVO s185', 'pin NUOVO s186')
out('s194-pair.sh', SRC, p)
print('ok')
