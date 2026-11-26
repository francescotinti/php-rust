#!/usr/bin/env python3
# gen-notte-s194b.py <hash16 del pin s190> — genera i lanciatori della NOTTE b di S-194 (criterio s194-criterio-notte-b.md) come
# COPIE DICHIARATE dei tre script della notte del 22/11 (stesso harness), ogni sostituzione con assert, manifest = diff vecchio/nuovo:
#   s194b-lancio-notte.sh    <- s194-lancio-notte.sh    (catena unica: gate di finestra INVARIATI, 3 passi + confronto)
#   s194b-orm-coppia-t31.sh  <- s194-orm-coppia-t30.sh  (coppia dbal+ORM t31 @ pin s190)
#   s194b-pair.sh            <- s194-pair.sh            (coppia WP full+media t29 @ pin s190)
# Generato con assistenza AI (Claude Fable 5.2).
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
SRC = 'wp194-harness/s194-lancio-notte.sh'
s = open(SRC, encoding='utf-8').read()
head = '''#!/bin/bash
# s194b-lancio-notte.sh — NOTTE b di S-194 (criterio wp194-harness/s194-criterio-notte-b.md): CATENA UNICA, un solo lock TOKEN s194,
# gate di finestra INVARIATI, poi in ORDINE: (1) guardia b7 = s186-guardia.sh INVARIATO con A = stash phpr-s183, Z gemello S-186,
# B = binario CANONICO (pin s190) · (2) coppia dbal+ORM t31 @ pin s190 (costanti E5 INVARIATE) · (3) coppia WP full+media t29
# @ pin s190 per ULTIMA · poi s194b-notte-confronto.py (regola esiti -> conferma contro la notte del 22/11).
# COPIA DICHIARATA di s194-lancio-notte.sh (manifest s194b-lancio-notte-copia.diff, generatore gen-notte-s194b.py) coi SOLI
# adattamenti: cartella catena-out e file catena-s194b.done, etichette b7/t31/t29 e pin s190, controlli di hash (stash
# s183/s186/s190, canoniche phpr s190 e server s191, gemello Z), blocco finale coi tre passi della notte b. Testo precedente sotto:
'''
s = head + s[s.index('set -u\n'):]
s = rep_all(s, 'catena-out', 'catena-out-b', 6)
s = rep(s, 'catena-s194.done"', 'catena-s194b.done"', 2)
s = rep(s, 'echo "s194 notte b6+t30+t28 @ pin s186 $(date', 'echo "s194 notte-b b7+t31+t29 @ pin s190 $(date')
s = rep(s, '== dichiarazione finestra quieta NOTTE S-194 (b6, t30, t28)', '== dichiarazione finestra quieta NOTTE b S-194 (b7, t31, t29)')
s = rep(s, '— (1) guardia b6, (2) t30, (3) t28" >> "$LOG"', '— (1) guardia b7, (2) t31, (3) t29" >> "$LOG"')
anchor = 'PIN183=$(shasum -a 256 "$STASH/phpr-s183" | cut -c1-16)'
assert s.count(anchor) == 1
s = s[:s.index(anchor)] + '''PIN183=$(shasum -a 256 "$STASH/phpr-s183" | cut -c1-16); PIN186=$(shasum -a 256 "$STASH/phpr-s186" | cut -c1-16); PIN190=$(shasum -a 256 "$STASH/phpr-s190" | cut -c1-16)
CANON="$HOME/Claude/php-rust-output/release/phpr"
PC=$(shasum -a 256 "$CANON" | cut -c1-16)
SRVC=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/php-server" | cut -c1-16)
[ "$PIN183" = "9a6ae11bf7e900ef" ] || fine "stash s183 $PIN183 != 9a6ae11bf7e900ef" "rc=9 stash183" 9
[ "$PIN186" = "9977cc4b1cf19991" ] || fine "stash s186 $PIN186 != 9977cc4b1cf19991" "rc=9 stash186" 9
[ "$PIN190" = "@PIN@" ] || fine "stash s190 $PIN190 != @PIN@" "rc=9 stash190" 9
[ "$PC" = "$PIN190" ] || fine "pin canonico $PC != stash s190 $PIN190" "rc=9 pin" 9
[ "$SRVC" = "c413398a44f9b0a4" ] || fine "server canonico $SRVC != s191 c413398a44f9b0a4" "rc=9 server" 9
ZB="$REPO/wp186-harness/ab-out/s186-leva/phpr-Z"
[ "$(shasum -a 256 "$ZB" 2>/dev/null | cut -c1-16)" = "23c0a94c26db62eb" ] || fine "gemello Z assente o diverso ($ZB)" "rc=9 Z" 9
[ -e "$REPO/wp186-harness/s186-guardia-verdetto-b7.out" ] && fine "verdetto b7 ESISTE già: spostarlo prima" "rc=7 verdetto-b7" 7
echo "$(date '+%F %T') pin canonico $PC == stash s190 · stash s183 $PIN183 · server $SRVC (s191, invariato) · Z ok — (1) GUARDIA b7 (A=stash s183, Z, B=CANONICO s190, R=5)" >> "$LOG"
PIN="$STASH/phpr-s183" Z="$ZB" B="$CANON" R=5 RE3=3 TAG=b7 /bin/bash "$REPO/wp186-harness/s186-guardia.sh"
rc1=$?; GR="$(cat "$REPO/wp186-harness/guardia-out-b7/guardia.done" 2>/dev/null)"
echo "$(date '+%F %T') guardia b7 fine rc=$rc1 ($GR) — verdetto wp186-harness/s186-guardia-verdetto-b7.out — (2) ORM t31 (E5 INVARIATE: ORA_REF2 4,870 SENT [4,81;4,93]; REF2 [34,23;34,31])" >> "$LOG"
SPD=/private/tmp/phpr-s194b-orm; mkdir -p "$SPD"
export E5=1 ORA_REF2=4.870 SENT_LO=4.81 SENT_HI=4.93 REF2_MIN=34.23 REF2_MAX=34.31
PIN_ATTESO="$PIN190" MAPPA_SP="$SPD" /bin/bash "$H/s194b-orm-coppia-t31.sh"
rc2=$?; TR="$(cat "$H/orm-out-b/rimisura.done" 2>/dev/null)"
echo "$(date '+%F %T') t31 fine rc=$rc2 ($TR) — verdetto s194b-orm-coppia-verdetto-t31.out — (3) WP t29 full+media @ pin s190 (ULTIMA: le gambe media risvegliano mediaanalysisd)" >> "$LOG"
PIN_ATTESO="$PIN190" SRV_ATTESO="$SRVC" /bin/bash "$H/s194b-pair.sh" t29
rc3=$?; PR="$(cat "$H/pair-out/pair194-t29.done" 2>/dev/null)"
python3 "$H/s194b-notte-confronto.py" >> "$LOG" 2>&1
RC=0; for r in "$rc1" "$rc2" "$rc3"; do [ "$r" -gt "$RC" ] && RC=$r; done
fine "notte b fine: b7 rc=$rc1 · t31 rc=$rc2 · t29 rc=$rc3; confronto in s194b-notte-confronto.out; lock rimosso" "rc=$RC guardia_b7=[$GR] t31=[$TR] t29=[$PR]" "$RC"
'''.replace('@PIN@', PIN)
b = s[s.index('set -u\n'):]
for bad in ('catena-s194.done', 'catena-out/', 'catena-out"', 't30', 't28', 'b6+', 'guardia b6', 'verdetto-b6', 'out-b6', 'TAG=b6', 's186 $(date', 'orm-out/'):
    assert bad not in b, bad
out('s194b-lancio-notte.sh', SRC, s)

# ---------------------------------------------------------------- 2. ORM t31
SRC = 'wp194-harness/s194-orm-coppia-t30.sh'
o = open(SRC, encoding='utf-8').read()
o = rep(o, '#!/bin/bash\n', '#!/bin/bash\n'
        '# s194b-orm-coppia-t31.sh — S-194 notte b: coppia dbal+ORM t31 @ pin s190 (DOVUTA: conferma di s187..s190; t30 era @ s186).\n'
        '# COPIA DICHIARATA di s194-orm-coppia-t30.sh (manifest s194b-orm-coppia-t31-copia.diff, generatore gen-notte-s194b.py) coi\n'
        '# SOLI adattamenti: cartella orm-out-b, verdetto s194b-orm-coppia-verdetto-t31.out, PIN_ATTESO default s190, etichette\n'
        '# t31/s190. Costanti E5 dal lanciatore, INVARIATE. Testo precedente sotto:\n')
o = rep(o, 'OUT="$H/orm-out"; mkdir -p "$OUT"', 'OUT="$H/orm-out-b"; mkdir -p "$OUT"')
o = rep_all(o, 'orm-out/', 'orm-out-b/')
o = rep(o, 'VERD="$H/s194-orm-coppia-verdetto-t30.out"', 'VERD="$H/s194b-orm-coppia-verdetto-t31.out"')
o = rep(o, 'PIN_ATTESO="${PIN_ATTESO:-9977cc4b1cf19991}"', 'PIN_ATTESO="${PIN_ATTESO:-' + PIN + '}"')
o = rep(o, 'print("coppia-t30 @ s186 (user CPU;', 'print("coppia-t31 @ s190 (user CPU;')
o = rep(o, '{ echo "== s194 coppia t30 dbal+ORM (pin s186 MISURATO $PINM', '{ echo "== s194 coppia t31 dbal+ORM (pin s190 MISURATO $PINM')
out('s194b-orm-coppia-t31.sh', SRC, o)

# ---------------------------------------------------------------- 3. WP t29
SRC = 'wp194-harness/s194-pair.sh'
p = open(SRC, encoding='utf-8').read()
p = rep(p, '#!/bin/bash\n', '#!/bin/bash\n'
        '# s194b-pair.sh <tentativo> — S-194 notte b: coppia full/media WP t29 @ pin s190 (DOVUTA: conferma di s187..s190; t28 era\n'
        '# @ s186). COPIA DICHIARATA di s194-pair.sh (manifest s194b-pair-copia.diff, generatore gen-notte-s194b.py) coi SOLI\n'
        '# adattamenti: etichette t29/s190; mediane storiche + t28=1,646 (banda di giudizio INVARIATA [1,738;1,799]: il confronto con\n'
        '# t28 lo fa s194b-notte-confronto.py); messaggi riferiti al pin s190. Stesse cartelle (i nomi portano il tag). Testo sotto:\n')
p = rep(p, '== s194 coppia t28: WP full+media @ pin s186 MISURATO', '== s194 coppia t29: WP full+media @ pin s190 MISURATO')
p = rep(p, "vs t27=1.705: {'≤' if med_t28 <= 1.705 else '>'}", "vs t28=1.646: {'≤' if med_t28 <= 1.646 else '>'}")
p = rep(p, 't25=1.747 t26=1.751 t27=1.705', 't25=1.747 t26=1.751 t27=1.705 t28=1.646', 2)
p = rep_all(p, 'med_t28', 'med_t29')
p = rep(p, 'mediana_t28=', 'mediana_t29=')
p = rep_all(p, 'pin s186 (leve di S-194: attribuzione per leva)', 'pin s190 (leve di S-194 dopo s186: attribuzione per leva)')
p = rep_all(p, 'pin NUOVO s186', 'pin NUOVO s190')
out('s194b-pair.sh', SRC, p)
print('ok')
