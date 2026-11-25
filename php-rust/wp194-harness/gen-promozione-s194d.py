#!/usr/bin/env python3
# gen-promozione-s194d.py — genera s194d-promozione.sh e s194d-lancio-promo.sh come COPIE DICHIARATE di s194c-promozione.sh e
# s194c-lancio-promo.sh (ogni sostituzione con assert sul numero di occorrenze; manifest = diff: s194d-promozione-copia.diff,
# s194d-lancio-promo-copia.diff). Criterio: s194-criterio-promozione-d.md. Stessa forma di gen-promozione-s194c.py.
# Uso: gen-promozione-s194d.py <commit del candidato> [gate invarianti da rendere bilaterali ...]. Generato con assistenza AI (Claude Fable 5.2).
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

s = open('s194c-promozione.sh', encoding='utf-8').read()
head_old = s[:s.index('# Testo S-193/')]
head_new = '''#!/bin/bash
# s194d-promozione.sh — gate di PROMOZIONE del tree «pin s188 + pacchetto P (P1-P4)» → pin phpr s189
# (criterio wp194-harness/s194-criterio-promozione-d.md; s188 resta nello stash; si lancia SOLO con la parola dell'utente).
# COPIA DICHIARATA di s194c-promozione.sh (manifest s194d-promozione-copia.diff, generatore gen-promozione-s194d.py) coi SOLI
# adattamenti: (1) tag s189, etichette dello stash s187 → s188, verdetto s194d-promo-verdetto.out, uscite in promo-out-d;
# (2) CANDIDATO = braccio con la ricetta del pin su HEAD (commit passato al generatore); precondizione = accettazione s194P4b
# rc=0; (3) REF = stash phpr-s188 (98736f5fc9ac2d47); (4) i denti P1-P4 entrano come gate BILATERALI; i gate che le cure
# rendono diversi dallo stash passano da invarianti a bilaterali, per nome nel criterio; (5) corpus congelato 1399 per modo
# (emenda dichiarata s194-corpus-emenda-2026-11-25.out); (6) dichiarazioni PRE riscritte.
'''
body = s[len(head_old):]
assert body.count('s189') == 0
body = body.replace('s188', 's189').replace('s187', 's188')
s = head_new + body
s = rep(s, 'OUT="$H14/promo-out-c"; mkdir -p "$OUT"', 'OUT="$H14/promo-out-d"; mkdir -p "$OUT"')
s = rep(s, 'VERD="$H14/s194c-promo-verdetto.out"', 'VERD="$H14/s194d-promo-verdetto.out"')
s = rep(s, 'REF_EXP="e7025fb37b1a87e0"', 'REF_EXP="98736f5fc9ac2d47"')
s = rep(s, 'CAND_COMMIT="${CAND_COMMIT:-f91a6597}"', 'CAND_COMMIT="${CAND_COMMIT:-%s}"' % COMMIT)
s = rep(s, 'accettazione-s194C3b2.rc', 'accettazione-s194P4b.rc', 3)
s = rep(s, 'accettazione s194C3b2 rc=', 'accettazione s194P4b rc=', 2)
s = rep(s, '"$H14/s194-c3-comportamento.php" "$H14/s194-c3b-comportamento.php" \\', '"$H14/s194-c3-comportamento.php" "$H14/s194-c3b-comportamento.php" "$H14/s194-p1-comportamento.php" "$H14/s194-p2-comportamento.php" "$H14/s194-p3-comportamento.php" "$H14/s194-p4-comportamento.php" \\')
s = rep(s, line_from(s, 'note "PRE: candidato = braccio su HEAD $CH'), 'note "PRE: candidato = braccio su HEAD $CH (commit $CAND_COMMIT; accettazione s194P4b rc=0, sonda P4 con zero fixture peggiorate, conto in istruzioni su ORM e WordPress); REF = stash s188 $REF_EXP"')
s = rep(s, line_from(s, 'note "PRE: DICHIARAZIONE dei cambiamenti del pin'), 'note "PRE: DICHIARAZIONE dei cambiamenti del pin s189 rispetto a s188 (per nome, wp194-harness/s194-criterio-P.md): P1 isset/empty di un percorso misto a una passata (4f6409ec); P2 isset/empty/assegnazione a una chiave su una variabile array senza vettore di chiavi (83b435da; tre righe in run_loop, giudicate sul carico intero); P3 la cache di scrittura ammette le private della classe dello scope (0f3e67d4); P4 la cache di scrittura ammette le readonly inizializzabili dallo scope, e CURA di una divergenza dal PHP ufficiale: una readonly resta non inizializzata se la coercizione tipizzata fallisce (%s). Corpus: fail-set congelato 1400 -> 1399 per modo (un test che ora passa, nessun digest cambiato: s194-corpus-emenda-2026-11-25.out)."' % COMMIT)
s = rep(s, 'nomi==congelato (1400 per modo, emenda dichiarata 24/11)', 'nomi==congelato (1399 per modo, emenda dichiarata 25/11)')
l = line_from(s, 'bilat s194c3b "$H14/s194-c3b-comportamento.php"')
s = rep(s, l, l + '\nbilat s194p1 "$H14/s194-p1-comportamento.php" "" "dente P1 (S-194): isset/empty di percorsi misti proprieta/indice" -d log_errors=0 -d display_errors=1\nbilat s194p2 "$H14/s194-p2-comportamento.php" "" "dente P2 (S-194): isset/empty/assegnazione a una chiave su variabile array" -d log_errors=0 -d display_errors=1\nbilat s194p3 "$H14/s194-p3-comportamento.php" "" "dente P3 (S-194): scritture su proprieta private e protette a sito caldo" -d log_errors=0 -d display_errors=1\nbilat s194p4 "$H14/s194-p4-comportamento.php" "" "dente P4 (S-194): proprieta readonly a sito caldo, coercizione fallita poi scrittura valida" -d log_errors=0 -d display_errors=1')
for a in sys.argv[2:]:  # gate che le cure rendono diversi dallo stash: invar -> bilat, per nome
    l = line_from(s, 'invar %s ' % a)
    s = rep(s, l, 'bilat' + l[len('invar'):] + ' -d log_errors=0 -d display_errors=1')
open('s194d-promozione.sh', 'w', encoding='utf-8').write(s); os.chmod('s194d-promozione.sh', 0o755)
subprocess.run('diff s194c-promozione.sh s194d-promozione.sh > s194d-promozione-copia.diff', shell=True)

t = open('s194c-lancio-promo.sh', encoding='utf-8').read()
t = rep(t, '# s194c-lancio-promo.sh — lanciatore DETACHED della catena di promozione s194c → pin s188 (s194c-promozione.sh).', '# s194d-lancio-promo.sh — lanciatore DETACHED della catena di promozione s194d → pin s189 (s194d-promozione.sh).\n# COPIA DICHIARATA di s194c-lancio-promo.sh (manifest s194d-lancio-promo-copia.diff, generatore gen-promozione-s194d.py): soli path/tag.')
t = rep(t, 'O="$H/promo-out-c"', 'O="$H/promo-out-d"')
t = rep(t, 'export PROMO_SP="/private/tmp/promo-s194c-sp"', 'export PROMO_SP="/private/tmp/promo-s194d-sp"')
t = rep(t, 'echo "s194c promozione pin s188 ', 'echo "s194d promozione pin s189 ')
t = rep(t, '(TOKEN s194c-promo)', '(TOKEN s194d-promo)')
t = rep(t, '"$H/s194c-promozione.sh" >> "$LOG" 2>&1', '"$H/s194d-promozione.sh" >> "$LOG" 2>&1')
open('s194d-lancio-promo.sh', 'w', encoding='utf-8').write(t); os.chmod('s194d-lancio-promo.sh', 0o755)
subprocess.run('diff s194c-lancio-promo.sh s194d-lancio-promo.sh > s194d-lancio-promo-copia.diff', shell=True)
print('ok', s.count('s189'), s.count('s188'), s.count('s187'))
