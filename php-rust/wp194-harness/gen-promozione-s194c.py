#!/usr/bin/env python3
# gen-promozione-s194c.py — genera s194c-promozione.sh e s194c-lancio-promo.sh come COPIE DICHIARATE di s194b-promozione.sh e
# s194b-lancio-promo.sh (ogni sostituzione con assert sul numero di occorrenze; manifest = diff: s194c-promozione-copia.diff,
# s194c-lancio-promo-copia.diff). Criterio: s194-criterio-promozione-c.md. Generato con assistenza AI (Claude Fable 5.2).
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

s = open('s194b-promozione.sh', encoding='utf-8').read()
head_old = s[:s.index('# Testo S-193/')]
head_new = '''#!/bin/bash
# s194c-promozione.sh — gate di PROMOZIONE del tree «pin s187 + C3a + C3b + tre cure di divergenze» → pin phpr s188
# (delibera utente 24/11; criterio wp194-harness/s194-criterio-promozione-c.md; s187 resta nello stash).
# COPIA DICHIARATA di s194b-promozione.sh (manifest s194c-promozione-copia.diff, generatore gen-promozione-s194c.py) coi SOLI
# adattamenti: (1) tag s188, etichette dello stash s186 → s187, verdetto s194c-promo-verdetto.out, uscite in promo-out-c;
# (2) CANDIDATO = braccio con la ricetta del pin su HEAD (commit passato al generatore); precondizione = accettazione s194C3b2
# rc=0; (3) REF = stash phpr-s187 (e7025fb37b1a87e0); (4) i denti C3a e C3b entrano come gate BILATERALI; i gate che le cure
# rendono diversi dallo stash passano da invarianti a bilaterali, per nome nel criterio; (5) corpus congelato 1400 per modo
# (emenda dichiarata s194-corpus-emenda-2026-11-24.out); (6) dichiarazioni PRE riscritte.
'''
body = s[len(head_old):]
body = body.replace('s187', 's188').replace('s186', 's187')
s = head_new + body
s = rep(s, 'OUT="$H14/promo-out-b"; mkdir -p "$OUT"', 'OUT="$H14/promo-out-c"; mkdir -p "$OUT"')
s = rep(s, 'VERD="$H14/s194b-promo-verdetto.out"', 'VERD="$H14/s194c-promo-verdetto.out"')
s = rep(s, 'REF_EXP="9977cc4b1cf19991"', 'REF_EXP="e7025fb37b1a87e0"')
s = rep(s, 'CAND_COMMIT="${CAND_COMMIT:-ebc96f6d}"', 'CAND_COMMIT="${CAND_COMMIT:-%s}"' % COMMIT)
n = s.count('accettazione-s194E4.rc'); assert n >= 3, n
s = s.replace('accettazione-s194E4.rc', 'accettazione-s194C3b2.rc')
s = rep(s, '"$H14/s194-c-comportamento.php" "$H14/s194-c-comportamento-strict.php" \\', '"$H14/s194-c-comportamento.php" "$H14/s194-c-comportamento-strict.php" "$H14/s194-c3-comportamento.php" "$H14/s194-c3b-comportamento.php" \\')
s = rep(s, 'accettazione s194E4 rc=', 'accettazione s194C3b2 rc=', 2)
s = rep(s, line_from(s, 'note "PRE: candidato = braccio su HEAD $CH'), 'note "PRE: candidato = braccio su HEAD $CH (commit $CAND_COMMIT; accettazione s194C3b2 rc=0, sonda C3b2 con zero fixture peggiorate, conto in istruzioni su ORM e WordPress); REF = stash s187 $REF_EXP"')
s = rep(s, line_from(s, 'note "PRE: DICHIARAZIONE dei cambiamenti del pin'), 'note "PRE: DICHIARAZIONE dei cambiamenti del pin s188 rispetto a s187 (per nome, wp194-harness/s194-criterio-C.md p.19-24): C3a chiamate con meno argomenti dei parametri sui cammini veloci (48bd41c1); C3b cache del sito con lo scope nella chiave per i metodi non pubblici (f91a6597, nessuna lettura dentro run_loop); cure delle divergenze dal PHP ufficiale: riga di ArgumentCountError = riga della dichiarazione, codice di -r chiamato Command line code, __DIR__ di -r e stdin = directory corrente (48bd41c1, 56a24cc4); contatori census-time fuori dal binario (faaa24f4). Corpus: fail-set congelato 1403 -> 1400 per modo (tre test che ora passano, nessun digest cambiato: s194-corpus-emenda-2026-11-24.out)."')
s = rep(s, 'nomi==congelato (1403 per modo, emenda dichiarata 23/11)', 'nomi==congelato (1400 per modo, emenda dichiarata 24/11)')
l = line_from(s, 'bilat s194c "$H14/s194-c-comportamento.php"')
s = rep(s, l, l + '\nbilat s194c3 "$H14/s194-c3-comportamento.php" "" "dente C3a (S-194): chiamate con meno argomenti dei parametri, default, ArgumentCountError" -d log_errors=0 -d display_errors=1\nbilat s194c3b "$H14/s194-c3b-comportamento.php" "" "dente C3b (S-194): metodi non pubblici, scope alternati via Closure::bind, privati omonimi, __call" -d log_errors=0 -d display_errors=1')
for a in sys.argv[2:]:  # gate che le cure rendono diversi dallo stash: invar -> bilat, per nome
    l = line_from(s, 'invar %s ' % a)
    s = rep(s, l, 'bilat' + l[len('invar'):] + ' -d log_errors=0 -d display_errors=1')
open('s194c-promozione.sh', 'w', encoding='utf-8').write(s); os.chmod('s194c-promozione.sh', 0o755)
subprocess.run('diff s194b-promozione.sh s194c-promozione.sh > s194c-promozione-copia.diff', shell=True)

t = open('s194b-lancio-promo.sh', encoding='utf-8').read()
t = rep(t, '# s194b-lancio-promo.sh — lanciatore DETACHED della catena di promozione s194b → pin s187 (s194b-promozione.sh).', '# s194c-lancio-promo.sh — lanciatore DETACHED della catena di promozione s194c → pin s188 (s194c-promozione.sh).\n# COPIA DICHIARATA di s194b-lancio-promo.sh (manifest s194c-lancio-promo-copia.diff, generatore gen-promozione-s194c.py): soli path/tag.')
t = rep(t, 'O="$H/promo-out-b"', 'O="$H/promo-out-c"')
t = rep(t, 'export PROMO_SP="/private/tmp/promo-s194b-sp"', 'export PROMO_SP="/private/tmp/promo-s194c-sp"')
t = rep(t, 'echo "s194b promozione pin s187 ', 'echo "s194c promozione pin s188 ')
t = rep(t, '(TOKEN s194b-promo)', '(TOKEN s194c-promo)')
t = rep(t, '"$H/s194b-promozione.sh" >> "$LOG" 2>&1', '"$H/s194c-promozione.sh" >> "$LOG" 2>&1')
open('s194c-lancio-promo.sh', 'w', encoding='utf-8').write(t); os.chmod('s194c-lancio-promo.sh', 0o755)
subprocess.run('diff s194b-lancio-promo.sh s194c-lancio-promo.sh > s194c-lancio-promo-copia.diff', shell=True)
print('ok', s.count('s188'), s.count('s187'), s.count('s186'))
