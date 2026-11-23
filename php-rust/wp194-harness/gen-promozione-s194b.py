#!/usr/bin/env python3
# gen-promozione-s194b.py — genera s194b-promozione.sh e s194b-lancio-promo.sh come COPIE DICHIARATE di s194-promozione.sh e
# s194-lancio-promo.sh (ogni sostituzione con assert sul numero di occorrenze; manifest = diff: s194b-promozione-copia.diff,
# s194b-lancio-promo-copia.diff). Criterio: s194-criterio-promozione-b.md. Generato con assistenza AI (Claude Fable 5.2).
import os, subprocess, sys
os.chdir(os.path.dirname(os.path.abspath(__file__)))
COMMIT = sys.argv[1]  # ultimo commit di codice del candidato

def rep(s, old, new, n=1):
    assert s.count(old) == n, (old[:70], s.count(old))
    return s.replace(old, new)

def line_from(s, start):
    assert s.count(start) == 1, (start[:70], s.count(start))
    t = s[s.index(start):]
    return t[:t.index('\n')]

s = open('s194-promozione.sh', encoding='utf-8').read()
head_old = s[:s.index('# Testo S-193/')]
head_new = '''#!/bin/bash
# s194b-promozione.sh — gate di PROMOZIONE del tree «pin s186 + C1 + C2 + cure delle divergenze sui tipi dichiarati» → pin phpr
# s187 (delibera utente 23/11; criterio wp194-harness/s194-criterio-promozione-b.md; s186 resta nello stash).
# COPIA DICHIARATA di s194-promozione.sh (manifest s194b-promozione-copia.diff, generatore gen-promozione-s194b.py) coi SOLI
# adattamenti: (1) tag s187, etichette dello stash s185 → s186, verdetto s194b-promo-verdetto.out, uscite in promo-out-b;
# (2) CANDIDATO = braccio con la ricetta del pin su HEAD (commit passato al generatore); precondizione = accettazione s194E4
# rc=0; (3) REF = stash phpr-s186 (9977cc4b1cf19991); (4) il dente C (s194-c-comportamento.php, col suo file strict) entra come
# gate BILATERALE; i gate che le cure rendono diversi dallo stash passano da invarianti a bilaterali, per nome nel criterio;
# (5) corpus congelato 1403 per modo (emenda dichiarata s194-corpus-emenda-2026-11-23.out); (6) dichiarazioni PRE riscritte.
'''
s = head_new + s[len(head_old):]
body = s[len(head_new):]
body = body.replace('s186', 's187').replace('s185', 's186')
s = head_new + body
s = rep(s, 'OUT="$H14/promo-out"; mkdir -p "$OUT"', 'OUT="$H14/promo-out-b"; mkdir -p "$OUT"')
s = rep(s, 'VERD="$H14/s194-promo-verdetto.out"', 'VERD="$H14/s194b-promo-verdetto.out"')
s = rep(s, 'REF_EXP="d9d6429301f60f4e"', 'REF_EXP="9977cc4b1cf19991"')
s = rep(s, 'CAND_COMMIT="${CAND_COMMIT:-d0c153b9}"', 'CAND_COMMIT="${CAND_COMMIT:-%s}"' % COMMIT)
n = s.count('accettazione-s194P.rc'); assert n >= 3, n
s = s.replace('accettazione-s194P.rc', 'accettazione-s194E4.rc')
s = rep(s, '"$H14/s194-a-comportamento.php" \\', '"$H14/s194-a-comportamento.php" "$H14/s194-c-comportamento.php" "$H14/s194-c-comportamento-strict.php" \\')
s = rep(s, 'accettazione s194P rc=', 'accettazione s194E4 rc=')
s = rep(s, line_from(s, 'note "PRE: candidato = braccio su HEAD $CH'), 'note "PRE: candidato = braccio su HEAD $CH (commit $CAND_COMMIT; accettazione s194E4 rc=0, sonda E3 con zero fixture peggiorate, conto in istruzioni su ORM e WordPress); REF = stash s186 $REF_EXP"')
s = rep(s, line_from(s, 'note "PRE: DICHIARAZIONE dei cambiamenti del pin'), 'note "PRE: DICHIARAZIONE dei cambiamenti del pin s187 rispetto a s186 (per nome, wp194-harness/s194-criterio-C.md p.13-17): C1 tipi dichiarati su chiamate e ritorni (64b2e461); C2 parametri tipizzati sul cammino veloce (5ab468f6, 014928f2); cure delle divergenze dal PHP ufficiale: nome della classe nei TypeError di un tipo scalare, iterable = Traversable|array, parametro by-ref tipizzato controllato, tipo di ritorno static applicato, callback di una funzione interna debole in un file strict_types, riga della Deprecated al return, avviso Undefined variable per un argomento di metodo (istruzione PushRefArg). Corpus: fail-set congelato 1407 -> 1403 per modo (quattro test che ora passano), sette digest del golden aggiornati (s194-corpus-emenda-2026-11-23.out)."')
s = rep(s, 'nomi==congelato (1407 per modo)', 'nomi==congelato (1403 per modo, emenda dichiarata 23/11)')
s = rep(s, line_from(s, 'invar s194a "$H14/s194-a-comportamento.php"'), line_from(s, 'invar s194a "$H14/s194-a-comportamento.php"') + '\nbilat s194c "$H14/s194-c-comportamento.php" "" "dente C (S-194): tipi dichiarati su parametri e ritorni, by-ref, static, strict_types da funzioni interne, avvisi" -d log_errors=0 -d display_errors=1')
for a in sys.argv[2:]:  # gate che le cure rendono diversi dallo stash: invar -> bilat, per nome
    l = line_from(s, 'invar %s ' % a)
    s = rep(s, l, 'bilat' + l[len('invar'):] + ' -d log_errors=0 -d display_errors=1')
open('s194b-promozione.sh', 'w', encoding='utf-8').write(s); os.chmod('s194b-promozione.sh', 0o755)
subprocess.run('diff s194-promozione.sh s194b-promozione.sh > s194b-promozione-copia.diff', shell=True)

t = open('s194-lancio-promo.sh', encoding='utf-8').read()
t = rep(t, '# s194-lancio-promo.sh — lanciatore DETACHED della catena di promozione s194 → pin s186 (s194-promozione.sh).', '# s194b-lancio-promo.sh — lanciatore DETACHED della catena di promozione s194b → pin s187 (s194b-promozione.sh).\n# COPIA DICHIARATA di s194-lancio-promo.sh (manifest s194b-lancio-promo-copia.diff, generatore gen-promozione-s194b.py): soli path/tag.')
t = rep(t, 'O="$H/promo-out"', 'O="$H/promo-out-b"')
t = rep(t, 'export PROMO_SP="/private/tmp/promo-s194-sp"', 'export PROMO_SP="/private/tmp/promo-s194b-sp"')
t = rep(t, 'echo "s194 promozione pin s186 ', 'echo "s194b promozione pin s187 ')
t = rep(t, '(TOKEN s194-promo)', '(TOKEN s194b-promo)')
t = rep(t, '"$H/s194-promozione.sh" >> "$LOG" 2>&1', '"$H/s194b-promozione.sh" >> "$LOG" 2>&1')
open('s194b-lancio-promo.sh', 'w', encoding='utf-8').write(t); os.chmod('s194b-lancio-promo.sh', 0o755)
subprocess.run('diff s194-lancio-promo.sh s194b-lancio-promo.sh > s194b-lancio-promo-copia.diff', shell=True)
print('ok', s.count('s187'), s.count('s186'))
