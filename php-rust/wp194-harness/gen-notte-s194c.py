#!/usr/bin/env python3
# gen-notte-s194c.py — genera la CODA c della notte b di S-194 (criterio s194-criterio-notte-b.md p.10-13) come COPIE DICHIARATE
# degli script della notte b, ogni sostituzione con assert, manifest = diff vecchio/nuovo:
#   s194c-lancio-coda.sh          <- s194b-lancio-notte.sh    (aspetta la catena b, gate INVARIATI, 4 passi + confronto)
#   s194c-orm-coppia-t32/t33.sh   <- s194b-orm-coppia-t31.sh  (coppia dbal+ORM @ stash s186 / s189)
#   s194c-pair-s186/s189.sh       <- s194b-pair.sh            (coppia WP full+media t30 @ s186 / t31 @ s189)
# Generato con assistenza AI (Claude Fable 5.2).
import os, subprocess
os.chdir(os.path.dirname(os.path.abspath(__file__)) + '/..')
PINS = [('s186', '9977cc4b1cf19991', 't32', 't30'), ('s189', '2d42efd6606e8dd9', 't33', 't31')]
BIN = '/private/tmp/phpr-s194c-bin'

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

for (S, HASH, TO, TW) in PINS:
    SRC = 'wp194-harness/s194b-orm-coppia-t31.sh'
    o = open(SRC, encoding='utf-8').read()
    o = rep(o, '#!/bin/bash\n', '#!/bin/bash\n'
            '# s194c-orm-coppia-%s.sh — S-194 notte b, coda c: coppia dbal+ORM %s @ STASH %s (rimisura di controllo nella stessa notte).\n'
            '# COPIA DICHIARATA di s194b-orm-coppia-t31.sh (manifest accanto, generatore gen-notte-s194c.py) coi SOLI adattamenti:\n'
            '# binario = copia dello stash in %s (non la canonica), cartella orm-out-c-%s, verdetto s194c-...-%s.out, PIN_ATTESO,\n'
            '# etichette. Costanti E5 dal lanciatore, INVARIATE. Testo precedente sotto:\n' % (TO, TO, S, BIN, S, TO))
    o = rep(o, 'PHPR="$HOME/Claude/php-rust-output/release/phpr"', 'PHPR="%s/phpr-%s"' % (BIN, S))
    o = rep(o, 'OUT="$H/orm-out-b"; mkdir -p "$OUT"', 'OUT="$H/orm-out-c-%s"; mkdir -p "$OUT"' % S)
    o = rep_all(o, 'orm-out-b/', 'orm-out-c-%s/' % S)
    o = rep(o, 'VERD="$H/s194b-orm-coppia-verdetto-t31.out"', 'VERD="$H/s194c-orm-coppia-verdetto-%s.out"' % TO)
    o = rep(o, 'PIN_ATTESO="${PIN_ATTESO:-97e000aa00fe802e}"', 'PIN_ATTESO="${PIN_ATTESO:-' + HASH + '}"')
    o = rep(o, 'print("coppia-t31 @ s190 (user CPU;', 'print("coppia-%s @ %s (user CPU;' % (TO, S))
    o = rep(o, '{ echo "== s194 coppia t31 dbal+ORM (pin s190 MISURATO $PINM', '{ echo "== s194 coppia %s dbal+ORM (stash %s MISURATO $PINM' % (TO, S))
    assert 'php-rust-output/release/phpr' not in o
    out('s194c-orm-coppia-%s.sh' % TO, SRC, o)

    SRC = 'wp194-harness/s194b-pair.sh'
    p = open(SRC, encoding='utf-8').read()
    p = rep(p, '#!/bin/bash\n', '#!/bin/bash\n'
            '# s194c-pair-%s.sh <tentativo> — S-194 notte b, coda c: coppia full/media WP %s @ STASH %s (rimisura di controllo nella\n'
            '# stessa notte). COPIA DICHIARATA di s194b-pair.sh (manifest accanto, generatore gen-notte-s194c.py) coi SOLI adattamenti:\n'
            '# binario = copia dello stash in %s (non la canonica), etichette %s/%s. Testo precedente sotto:\n' % (S, TW, S, BIN, TW, S))
    p = rep(p, 'PHPR="$HOME/Claude/php-rust-output/release/phpr"', 'PHPR="%s/phpr-%s"' % (BIN, S))
    p = rep(p, '== s194 coppia t29: WP full+media @ pin s190 MISURATO', '== s194 coppia %s: WP full+media @ stash %s MISURATO' % (TW, S))
    p = rep_all(p, 'med_t29', 'med_' + TW)
    p = rep(p, 'mediana_t29=', 'mediana_%s=' % TW)
    p = rep_all(p, 'pin s190 (leve di S-194 dopo s186: attribuzione per leva)', 'stash %s (rimisura di controllo, coda c)' % S)
    p = rep_all(p, 'pin NUOVO s190', 'stash ' + S)
    # the full/media legs run through wp109-harness/pair109.sh, which hardcodes the CANONICAL binary: declared copy with the stash copy
    p = rep(p, '"$H109/pair109.sh" "$mode"', '"$H/s194c-pair109-%s.sh" "$mode"' % S)
    out('s194c-pair-%s.sh' % S, SRC, p)
    SRC = 'wp109-harness/pair109.sh'
    q = open(SRC, encoding='utf-8').read()
    q = rep(q, 'PHPR="$HOME/Claude/php-rust-output/release/phpr"', 'PHPR="%s/phpr-%s"  # s194c: stash copy (declared copy of pair109.sh, generator gen-notte-s194c.py)' % (BIN, S))
    out('s194c-pair109-%s.sh' % S, SRC, q)

SRC = 'wp194-harness/s194b-lancio-notte.sh'
s = open(SRC, encoding='utf-8').read()
head = '''#!/bin/bash
# s194c-lancio-coda.sh — CODA c della notte b di S-194 (criterio wp194-harness/s194-criterio-notte-b.md p.10-13): aspetta la fine
# della catena b (file catena-s194b.done), poi ripete TUTTI i gate di finestra, riprende il lock TOKEN s194 e misura in ORDINE:
# ORM t32 @ stash s186 · ORM t33 @ stash s189 · WP t30 @ stash s186 · WP t31 @ stash s189 · poi s194c-notte-confronto.py.
# COPIA DICHIARATA di s194b-lancio-notte.sh (manifest s194c-lancio-coda-copia.diff, generatore gen-notte-s194c.py) coi SOLI
# adattamenti: attesa iniziale della catena b, cartella catena-out-c e file catena-s194c.done, etichette, blocco finale coi
# quattro passi sui binari copiati dallo stash in @BIN@ (la canonica s190 non si tocca). Testo precedente sotto:
'''.replace('@BIN@', BIN)
s = head + s[s.index('set -u\n'):]
s = rep_all(s, 'catena-out-b', 'catena-out-c', 6)
s = rep(s, 'catena-s194b.done"', 'catena-s194c.done"', 2)
s = rep(s, 'echo "$(date \'+%F %T\') attesa quiete CI" >> "$LOG"\n',
        'echo "$(date \'+%F %T\') coda c: attesa fine catena b" >> "$LOG"\n'
        'while [ ! -e "$H/catena-out-b/catena-s194b.done" ]; do sleep 60; done\n'
        'echo "$(date \'+%F %T\') catena b finita ($(cat "$H/catena-out-b/catena-s194b.done")) — attesa quiete CI" >> "$LOG"\n')
s = rep(s, 'echo "s194 notte-b b7+t31+t29 @ pin s190 $(date', 'echo "s194 notte-b coda c t32+t33+t30+t31 @ stash s186/s189 $(date')
s = rep(s, '== dichiarazione finestra quieta NOTTE b S-194 (b7, t31, t29)', '== dichiarazione finestra quieta CODA c S-194 (t32, t33, t30, t31)')
s = rep(s, '— (1) guardia b7, (2) t31, (3) t29" >> "$LOG"', '— (1) ORM t32, (2) ORM t33, (3) WP t30, (4) WP t31" >> "$LOG"')
anchor = 'PIN183=$(shasum -a 256 "$STASH/phpr-s183" | cut -c1-16)'
assert s.count(anchor) == 1
s = s[:s.index(anchor)] + '''BIN=@BIN@; mkdir -p "$BIN"
for pair in s186:9977cc4b1cf19991 s189:2d42efd6606e8dd9; do
  n=${pair%%:*}; h=${pair##*:}
  [ "$(shasum -a 256 "$STASH/phpr-$n" | cut -c1-16)" = "$h" ] || fine "stash $n != $h" "rc=9 stash-$n" 9
  cp "$STASH/phpr-$n" "$BIN/phpr-$n" && chmod 755 "$BIN/phpr-$n"
  [ "$(shasum -a 256 "$BIN/phpr-$n" | cut -c1-16)" = "$h" ] || fine "copia $n != $h" "rc=9 copia-$n" 9
done
SRVC=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/php-server" | cut -c1-16)
[ "$SRVC" = "c413398a44f9b0a4" ] || fine "server canonico $SRVC != s191 c413398a44f9b0a4" "rc=9 server" 9
for v in s194c-orm-coppia-verdetto-t32.out s194c-orm-coppia-verdetto-t33.out s194-pair-verdetto-t30.out s194-pair-verdetto-t31.out; do
  [ -e "$H/$v" ] && fine "verdetto $v ESISTE già: spostarlo prima" "rc=7 $v" 7
done
export E5=1 ORA_REF2=4.870 SENT_LO=4.81 SENT_HI=4.93 REF2_MIN=34.23 REF2_MAX=34.31
echo "$(date '+%F %T') binari copiati e verificati in $BIN — (1) ORM t32 @ s186" >> "$LOG"
SPD=/private/tmp/phpr-s194c-orm-s186; mkdir -p "$SPD"
PIN_ATTESO=9977cc4b1cf19991 MAPPA_SP="$SPD" /bin/bash "$H/s194c-orm-coppia-t32.sh"
rc1=$?; R1="$(cat "$H/orm-out-c-s186/rimisura.done" 2>/dev/null)"
echo "$(date '+%F %T') t32 fine rc=$rc1 ($R1) — (2) ORM t33 @ s189" >> "$LOG"
SPD=/private/tmp/phpr-s194c-orm-s189; mkdir -p "$SPD"
PIN_ATTESO=2d42efd6606e8dd9 MAPPA_SP="$SPD" /bin/bash "$H/s194c-orm-coppia-t33.sh"
rc2=$?; R2="$(cat "$H/orm-out-c-s189/rimisura.done" 2>/dev/null)"
echo "$(date '+%F %T') t33 fine rc=$rc2 ($R2) — (3) WP t30 @ s186" >> "$LOG"
PIN_ATTESO=9977cc4b1cf19991 SRV_ATTESO="$SRVC" /bin/bash "$H/s194c-pair-s186.sh" t30
rc3=$?; R3="$(cat "$H/pair-out/pair194-t30.done" 2>/dev/null)"
echo "$(date '+%F %T') t30 fine rc=$rc3 ($R3) — (4) WP t31 @ s189" >> "$LOG"
PIN_ATTESO=2d42efd6606e8dd9 SRV_ATTESO="$SRVC" /bin/bash "$H/s194c-pair-s189.sh" t31
rc4=$?; R4="$(cat "$H/pair-out/pair194-t31.done" 2>/dev/null)"
python3 "$H/s194c-notte-confronto.py" >> "$LOG" 2>&1
RC=0; for r in "$rc1" "$rc2" "$rc3" "$rc4"; do [ "$r" -gt "$RC" ] && RC=$r; done
fine "coda c fine: t32 rc=$rc1 · t33 rc=$rc2 · t30 rc=$rc3 · t31 rc=$rc4; confronto in s194c-notte-confronto.out; lock rimosso" "rc=$RC t32=[$R1] t33=[$R2] t30=[$R3] t31=[$R4]" "$RC"
'''.replace('@BIN@', BIN)
b = s[s.index('set -u\n'):]
for bad in ('catena-s194c.done" ]; do sleep', 'b7', 'TAG=', 'guardia'):
    assert bad not in b, bad
assert b.count('catena-out-b/catena-s194b.done') == 2
out('s194c-lancio-coda.sh', SRC, s)
print('ok')
