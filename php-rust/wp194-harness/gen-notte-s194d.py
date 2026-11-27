#!/usr/bin/env python3
# gen-notte-s194d.py — genera la CODA d della notte b di S-194 (criterio s194-criterio-notte-b.md p.15): due ripetizioni della
# guardia. s194d-lancio-guardia.sh <- s194c-lancio-coda.sh (copia dichiarata, manifest accanto; gate di finestra INVARIATI).
# Generato con assistenza AI (Claude Fable 5.2).
import os, subprocess
os.chdir(os.path.dirname(os.path.abspath(__file__)) + '/..')
def rep(s, old, new, n=1):
    assert s.count(old) == n, (old[:80], s.count(old), n)
    return s.replace(old, new)
SRC = 'wp194-harness/s194c-lancio-coda.sh'
s = open(SRC, encoding='utf-8').read()
head = '''#!/bin/bash
# s194d-lancio-guardia.sh — CODA d della notte b di S-194 (criterio wp194-harness/s194-criterio-notte-b.md p.15): ripete TUTTI i
# gate di finestra, riprende il lock TOKEN s194 e ripete due volte s186-guardia.sh INVARIATO (A = stash s183, Z gemello S-186):
# b7b con B = CANONICO s190, b7c con B = copia dello stash s189; poi s194d-guardia-confronto.py.
# COPIA DICHIARATA di s194c-lancio-coda.sh (manifest s194d-lancio-guardia-copia.diff, generatore gen-notte-s194d.py) coi SOLI
# adattamenti: cartella catena-out-d e file catena-s194d.done, etichette, blocco finale coi due passi di guardia. Testo sotto:
'''
s = head + s[s.index('set -u\n'):]
assert s.count('catena-out-c') >= 6
s = s.replace('catena-out-c', 'catena-out-d')
s = rep(s, 'catena-s194c.done"', 'catena-s194d.done"', 2)
s = rep(s, 'echo "s194 notte-b coda c t32+t33+t30+t31 @ stash s186/s189 $(date', 'echo "s194 notte-b coda d guardia b7b+b7c $(date')
s = rep(s, '== dichiarazione finestra quieta CODA c S-194 (t32, t33, t30, t31)', '== dichiarazione finestra quieta CODA d S-194 (b7b, b7c)')
s = rep(s, '— (1) ORM t32, (2) ORM t33, (3) WP t30, (4) WP t31" >> "$LOG"', '— (1) guardia b7b, (2) guardia b7c" >> "$LOG"')
anchor = 'BIN=/private/tmp/phpr-s194c-bin; mkdir -p "$BIN"'
assert s.count(anchor) == 1
s = s[:s.index(anchor)] + '''BIN=/private/tmp/phpr-s194c-bin
CANON="$HOME/Claude/php-rust-output/release/phpr"
[ "$(shasum -a 256 "$STASH/phpr-s183" | cut -c1-16)" = "9a6ae11bf7e900ef" ] || fine "stash s183 diverso" "rc=9 stash183" 9
[ "$(shasum -a 256 "$CANON" | cut -c1-16)" = "97e000aa00fe802e" ] || fine "canonico != s190" "rc=9 pin" 9
[ "$(shasum -a 256 "$BIN/phpr-s189" 2>/dev/null | cut -c1-16)" = "2d42efd6606e8dd9" ] || fine "copia s189 assente o diversa" "rc=9 copia-s189" 9
ZB="$REPO/wp186-harness/ab-out/s186-leva/phpr-Z"
[ "$(shasum -a 256 "$ZB" 2>/dev/null | cut -c1-16)" = "23c0a94c26db62eb" ] || fine "gemello Z assente o diverso ($ZB)" "rc=9 Z" 9
for t in b7b b7c; do [ -e "$REPO/wp186-harness/s186-guardia-verdetto-$t.out" ] && fine "verdetto $t ESISTE già: spostarlo prima" "rc=7 verdetto-$t" 7; done
echo "$(date '+%F %T') hash ok — (1) GUARDIA b7b (A=stash s183, Z, B=CANONICO s190, R=5)" >> "$LOG"
PIN="$STASH/phpr-s183" Z="$ZB" B="$CANON" R=5 RE3=3 TAG=b7b /bin/bash "$REPO/wp186-harness/s186-guardia.sh"
rc1=$?; G1="$(cat "$REPO/wp186-harness/guardia-out-b7b/guardia.done" 2>/dev/null)"
echo "$(date '+%F %T') guardia b7b fine rc=$rc1 ($G1) — (2) GUARDIA b7c (A=stash s183, Z, B=copia stash s189, R=5)" >> "$LOG"
PIN="$STASH/phpr-s183" Z="$ZB" B="$BIN/phpr-s189" R=5 RE3=3 TAG=b7c /bin/bash "$REPO/wp186-harness/s186-guardia.sh"
rc2=$?; G2="$(cat "$REPO/wp186-harness/guardia-out-b7c/guardia.done" 2>/dev/null)"
python3 "$H/s194d-guardia-confronto.py" >> "$LOG" 2>&1
RC=0; for r in "$rc1" "$rc2"; do [ "$r" -gt "$RC" ] && RC=$r; done
fine "coda d fine: b7b rc=$rc1 · b7c rc=$rc2; confronto in s194d-guardia-confronto.out; lock rimosso" "rc=$RC b7b=[$G1] b7c=[$G2]" "$RC"
'''
b = s[s.index('set -u\n'):]
for bad in ('s194c-orm', 's194c-pair', 'catena-out-c', 'TAG=b7 '):
    assert bad not in b, bad
p = 'wp194-harness/s194d-lancio-guardia.sh'
open(p, 'w', encoding='utf-8').write(s); os.chmod(p, 0o755)
open(p[:-3] + '-copia.diff', 'w', encoding='utf-8').write(subprocess.run(['diff', SRC, p], capture_output=True, text=True).stdout)
print('ok')
