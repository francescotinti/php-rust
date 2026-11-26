#!/usr/bin/env python3
# gen-promozione-s194e.py — genera s194e-promozione.sh e s194e-lancio-promo.sh come COPIE DICHIARATE di s194d-promozione.sh e
# s194d-lancio-promo.sh (ogni sostituzione con assert sul numero di occorrenze; manifest = diff: s194e-promozione-copia.diff,
# s194e-lancio-promo-copia.diff). Criterio: s194-criterio-promozione-e.md. Stessa forma di gen-promozione-s194d.py.
# Uso: gen-promozione-s194e.py <commit del candidato> [gate invarianti da rendere bilaterali ...]. Generato con assistenza AI (Claude Fable 5.2).
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

s = open('s194d-promozione.sh', encoding='utf-8').read()
head_old = s[:s.index('# Testo S-193/')]
head_new = '''#!/bin/bash
# s194e-promozione.sh — gate di PROMOZIONE del tree «pin s189 + C4 + N1 + N2 (con B1/B2) + T1 + N3» → pin phpr s190
# (criterio wp194-harness/s194-criterio-promozione-e.md; s189 resta nello stash; si lancia SOLO con la parola dell'utente).
# COPIA DICHIARATA di s194d-promozione.sh (manifest s194e-promozione-copia.diff, generatore gen-promozione-s194e.py) coi SOLI
# adattamenti: (1) tag s190, etichette dello stash s188 → s189, verdetto s194e-promo-verdetto.out, uscite in promo-out-e;
# (2) CANDIDATO = braccio con la ricetta del pin su HEAD (commit passato al generatore); precondizione = accettazione s194N3
# rc=0; (3) REF = stash phpr-s189 (2d42efd6606e8dd9); (4) i denti C4, N1 e T1 entrano come gate BILATERALI; i gate che le cure
# rendono diversi dallo stash passano da invarianti a bilaterali, per nome nel criterio; (5) corpus congelato 1399 per modo,
# invariato; (6) dichiarazioni PRE riscritte.
'''
body = s[len(head_old):]
assert body.count('s190') == 0
body = body.replace('s189', 's190').replace('s188', 's189')
s = head_new + body
s = rep(s, 'OUT="$H14/promo-out-d"; mkdir -p "$OUT"', 'OUT="$H14/promo-out-e"; mkdir -p "$OUT"')
s = rep(s, 'VERD="$H14/s194d-promo-verdetto.out"', 'VERD="$H14/s194e-promo-verdetto.out"')
s = rep(s, 'REF_EXP="98736f5fc9ac2d47"', 'REF_EXP="2d42efd6606e8dd9"')
s = rep(s, line_from(s, 'CAND_COMMIT="${CAND_COMMIT:-')[:line_from(s, 'CAND_COMMIT="${CAND_COMMIT:-').index('}"') + 2], 'CAND_COMMIT="${CAND_COMMIT:-%s}"' % COMMIT)
s = rep(s, 'accettazione-s194P4b.rc', 'accettazione-s194N3.rc', 3)
s = rep(s, 'accettazione s194P4b rc=', 'accettazione s194N3 rc=', 2)
s = rep(s, '"$H14/s194-p3-comportamento.php" "$H14/s194-p4-comportamento.php" \\', '"$H14/s194-p3-comportamento.php" "$H14/s194-p4-comportamento.php" "$H14/s194-c4-comportamento.php" "$H14/s194-n1-comportamento.php" "$H14/s194-t1-comportamento.php" \\')
s = rep(s, line_from(s, 'note "PRE: candidato = braccio su HEAD $CH'), 'note "PRE: candidato = braccio su HEAD $CH (commit $CAND_COMMIT; accettazione s194N3 rc=0, conto in istruzioni su ORM e WordPress a ogni passo); REF = stash s189 $REF_EXP"')
s = rep(s, line_from(s, 'note "PRE: DICHIARAZIONE dei cambiamenti del pin'), 'note "PRE: DICHIARAZIONE dei cambiamenti del pin s190 rispetto a s189 (per nome, wp194-harness/s194-criterio-C.md p.25, s194-criterio-N.md, s194-criterio-B.md): C4 un argomento-variabile di una chiamata dinamica non promuove piu la variabile a riferimento (29b6d962, 9b9b5f11), con CURA di una divergenza dal PHP ufficiale: una variabile non definita passata a un oggetto invocabile o a un callable-array ora avvisa come PHP; N1 hash veloci su new, include e reflection, ricerca di sottostringa senza cercatore per chiamata (fab50dd2); N2 impronta delle classi registrate incrementale, B1/B2 lettura delle variabili e scarto degli scalari in linea nel ciclo (047bbee5; righe in run_loop, giudicate sul carico intero); T1 memoria dei controlli di tipo di classe riusciti (a0a74e9c); N3 un solo buffer per passata sul seed delle classi (%s). Corpus: fail-set congelato 1399 per modo, invariato."' % COMMIT)
l = line_from(s, 'bilat s194p4 "$H14/s194-p4-comportamento.php"')
s = rep(s, l, l + '\nbilat s194c4 "$H14/s194-c4-comportamento.php" "" "dente C4 (S-194): argomenti-variabile delle chiamate dinamiche" -d log_errors=0 -d display_errors=1\nbilat s194n1 "$H14/s194-n1-comportamento.php" "" "dente N1 (S-194): ricerca di sottostringa, riuso degli id degli oggetti" -d log_errors=0 -d display_errors=1\nbilat s194t1 "$H14/s194-t1-comportamento.php" "" "dente T1 (S-194): controlli di tipo di classe attraverso la memoria dei successi" -d log_errors=0 -d display_errors=1')
for a in sys.argv[2:]:  # gate che le cure rendono diversi dallo stash: invar -> bilat, per nome
    l = line_from(s, 'invar %s ' % a)
    s = rep(s, l, 'bilat' + l[len('invar'):] + ' -d log_errors=0 -d display_errors=1')
open('s194e-promozione.sh', 'w', encoding='utf-8').write(s); os.chmod('s194e-promozione.sh', 0o755)
subprocess.run('diff s194d-promozione.sh s194e-promozione.sh > s194e-promozione-copia.diff', shell=True)

t = open('s194d-lancio-promo.sh', encoding='utf-8').read()
t = rep(t, '# s194d-lancio-promo.sh — lanciatore DETACHED della catena di promozione s194d → pin s189 (s194d-promozione.sh).', '# s194e-lancio-promo.sh — lanciatore DETACHED della catena di promozione s194e → pin s190 (s194e-promozione.sh).\n# COPIA DICHIARATA di s194d-lancio-promo.sh (manifest s194e-lancio-promo-copia.diff, generatore gen-promozione-s194e.py): soli path/tag.')
t = rep(t, 'O="$H/promo-out-d"', 'O="$H/promo-out-e"')
t = rep(t, 'export PROMO_SP="/private/tmp/promo-s194d-sp"', 'export PROMO_SP="/private/tmp/promo-s194e-sp"')
t = rep(t, 'echo "s194d promozione pin s189 ', 'echo "s194e promozione pin s190 ')
t = rep(t, '(TOKEN s194d-promo)', '(TOKEN s194e-promo)')
t = rep(t, '"$H/s194d-promozione.sh" >> "$LOG" 2>&1', '"$H/s194e-promozione.sh" >> "$LOG" 2>&1')
t = rep(t, 'H="$SRC/wp194-harness"; O="$H/promo-out-e"; mkdir -p "$O"', 'H="$SRC/wp194-harness"; O="$H/promo-out-e"; mkdir -p "$O"')
open('s194e-lancio-promo.sh', 'w', encoding='utf-8').write(t); os.chmod('s194e-lancio-promo.sh', 0o755)
subprocess.run('diff s194d-lancio-promo.sh s194e-lancio-promo.sh > s194e-lancio-promo-copia.diff', shell=True)
print('ok', s.count('s190'), s.count('s189'), s.count('s188'))
