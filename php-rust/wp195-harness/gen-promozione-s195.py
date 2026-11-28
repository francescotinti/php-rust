#!/usr/bin/env python3
# gen-promozione-s195.py — genera s195-promozione.sh e s195-lancio-promo.sh come COPIE DICHIARATE di
# ../wp194-harness/s194e-promozione.sh e ../wp194-harness/s194e-lancio-promo.sh (ogni sostituzione con assert sul numero di
# occorrenze; manifest = diff: s195-promozione-copia.diff, s195-lancio-promo-copia.diff). Criterio: s195-criterio-promozione.md.
# Uso: gen-promozione-s195.py <commit del candidato> [gate invarianti da rendere bilaterali ...]. Generato con assistenza AI (Claude Fable 5.2).
import os, subprocess, sys
os.chdir(os.path.dirname(os.path.abspath(__file__)))
COMMIT = sys.argv[1]  # ultimo commit di codice del candidato
OLD = '../wp194-harness/'

def rep(s, old, new, n=1):
    assert s.count(old) == n, (old[:70], s.count(old))
    return s.replace(old, new)

def line_from(s, start):
    assert s.count(start) == 1, (start[:70], s.count(start))
    t = s[s.index(start):]
    return t[:t.index('\n')]

s = open(OLD + 's194e-promozione.sh', encoding='utf-8').read()
head_old = s[:s.index('# Testo S-193/')]
head_new = '''#!/bin/bash
# s195-promozione.sh — gate di PROMOZIONE del tree «pin s190 + pacchetto S (corpo di Sweep) + valore all'invio (forma J)» → pin
# phpr s191 (criterio wp195-harness/s195-criterio-promozione.md; s190 resta nello stash; delibera dell'utente 28/11).
# COPIA DICHIARATA di wp194-harness/s194e-promozione.sh (manifest s195-promozione-copia.diff, generatore gen-promozione-s195.py)
# coi SOLI adattamenti: (1) tag s191, etichette dello stash s189 → s190 (il pin del SERVER php-server-s191 resta com'era: le sue
# tre righe sono protette nel generatore), verdetto s195-promo-verdetto.out, uscite in wp195-harness/promo-out; (2) CANDIDATO =
# braccio con la ricetta del pin sul commit passato al generatore (out/phpr-cand); precondizione = accettazione s195J rc=0;
# (3) REF = stash phpr-s190 (97e000aa00fe802e); (4) il dente dell'invio entra come gate BILATERALE; i gate che le cure rendono
# diversi dallo stash passano da invarianti a bilaterali, per nome nel criterio; (5) corpus congelato 1399 per modo, invariato;
# (6) dichiarazioni PRE riscritte; (7) una guardia sul disco in piu subito PRIMA di pin-phpr.sh (lancio sotto 10G per delibera
# dell'utente del 28/11: uno stop per spazio deve arrivare prima che il pin nasca); nel lanciatore la soglia 10G scende a 7G.
'''
body = s[len(head_old):]
l = line_from(body, 'CAND="$HOME/Claude/phpr-target/s192-armD/release/phpr"; CAND_COMMIT=')
body = rep(body, l, 'CAND="$H15/out/phpr-cand"; CAND_COMMIT="${CAND_COMMIT:-%s}"' % COMMIT)
body = rep(body, 's191', '@@SERVER@@', 3)  # the server pin keeps its tag
assert body.count('s191') == 0
body = body.replace('s190', 's191').replace('s189', 's190')
body = rep(body, '@@SERVER@@', 's191', 3)
s = head_new + body
s = rep(s, 'H13="$SRC/wp193-harness"; H14="$SRC/wp194-harness"', 'H13="$SRC/wp193-harness"; H14="$SRC/wp194-harness"; H15="$SRC/wp195-harness"')
s = rep(s, 'OUT="$H14/promo-out-e"; mkdir -p "$OUT"', 'OUT="$H15/promo-out"; mkdir -p "$OUT"')
s = rep(s, 'VERD="$H14/s194e-promo-verdetto.out"', 'VERD="$H15/s195-promo-verdetto.out"')
s = rep(s, 'REF_EXP="2d42efd6606e8dd9"', 'REF_EXP="97e000aa00fe802e"')
s = rep(s, 'accettazione-s194N3.rc', 'accettazione-s195J.rc', 3)
s = rep(s, 'accettazione s194N3 rc=', 'accettazione s195J rc=', 2)
s = rep(s, '"$H14/s194-t1-comportamento.php" \\', '"$H14/s194-t1-comportamento.php" "$H15/s195-invio-comportamento.php" \\')
s = rep(s, line_from(s, 'note "PRE: candidato = braccio su HEAD $CH'), 'note "PRE: candidato = braccio da archivio $CH (commit $CAND_COMMIT; accettazione s195J rc=0, conto in istruzioni su ORM e WordPress nei criteri s195); REF = stash s190 $REF_EXP"')
s = rep(s, line_from(s, 'note "PRE: DICHIARAZIONE dei cambiamenti del pin'), 'note "PRE: DICHIARAZIONE dei cambiamenti del pin s191 rispetto a s190 (per nome, wp195-harness/s195-criterio-sweep.md, s195-criterio-invio.md): pacchetto S, corpo di Sweep (b9365351): rilascio a cascata senza hash su due mappe quasi sempre vuote, nota di un oggetto in una funzione propria; valore di un argomento-variabile letto all INVIO (e04ce6e4, %s), con CURA di una divergenza dal PHP ufficiale: in una chiamata a bersaglio dinamico una variabile inviata per valore e modificata da un argomento successivo arriva col valore dell invio. Residui dichiarati nel criterio di promozione p.3. Corpus: fail-set congelato 1399 per modo, invariato."' % COMMIT)
s = rep(s, 'note "PRE: LOC (REGOLE §8): S-194 NON alza alcun cap', 'note "PRE: LOC (REGOLE §8): S-195 NON alza alcun cap')
s = rep(s, 'nessuna leva di S-194 bersaglia il Ret', 'nessuna leva di S-195 bersaglia il Ret')
l = line_from(s, 'bilat s194t1 "$H14/s194-t1-comportamento.php"')
s = rep(s, l, l + '\nbilat s195invio "$H15/s195-invio-comportamento.php" "" "dente dell invio (S-195): valore di un argomento-variabile letto all invio" -d log_errors=0 -d display_errors=1')
for a in sys.argv[2:]:  # gate che le cure rendono diversi dallo stash: invar -> bilat, per nome
    l = line_from(s, 'invar %s ' % a)
    s = rep(s, l, 'bilat' + l[len('invar'):] + ' -d log_errors=0 -d display_errors=1')
s = rep(s, '"$SRC/scripts/pin-phpr.sh" s191 > "$OUT/pin.log" 2>&1', 'disk_guard pin  # S-195: a stop for disk space must come BEFORE the pin is born (launch below 10G by user decision, 28/11)\n"$SRC/scripts/pin-phpr.sh" s191 > "$OUT/pin.log" 2>&1')
open('s195-promozione.sh', 'w', encoding='utf-8').write(s); os.chmod('s195-promozione.sh', 0o755)
subprocess.run('diff ' + OLD + 's194e-promozione.sh s195-promozione.sh > s195-promozione-copia.diff', shell=True)

t = open(OLD + 's194e-lancio-promo.sh', encoding='utf-8').read()
t = rep(t, '# s194e-lancio-promo.sh — lanciatore DETACHED della catena di promozione s194e → pin s190 (s194e-promozione.sh).', '# s195-lancio-promo.sh — lanciatore DETACHED della catena di promozione s195 → pin s191 (s195-promozione.sh).\n# COPIA DICHIARATA di wp194-harness/s194e-lancio-promo.sh (manifest s195-lancio-promo-copia.diff, generatore gen-promozione-s195.py): soli path/tag.')
t = rep(t, 'H="$SRC/wp194-harness"; O="$H/promo-out-e"; mkdir -p "$O"', 'H="$SRC/wp195-harness"; O="$H/promo-out"; mkdir -p "$O"')
t = rep(t, 'export PROMO_SP="/private/tmp/promo-s194e-sp"', 'export PROMO_SP="/private/tmp/promo-s195-sp"')
t = rep(t, 'echo "s194e promozione pin s190 ', 'echo "s195 promozione pin s191 ')
t = rep(t, '(TOKEN s194e-promo)', '(TOKEN s195-promo)')
t = rep(t, '"$H/s194e-promozione.sh" >> "$LOG" 2>&1', '"$H/s195-promozione.sh" >> "$LOG" 2>&1')
t = rep(t, '-lt 10 ]', '-lt 7 ]')  # user decision 28/11: launch below 10G (uv cache in use, nothing else to free)
t = rep(t, '< 10G', '< 7G (soglia 10G abbassata per delibera 28/11)', 2)
open('s195-lancio-promo.sh', 'w', encoding='utf-8').write(t); os.chmod('s195-lancio-promo.sh', 0o755)
subprocess.run('diff ' + OLD + 's194e-lancio-promo.sh s195-lancio-promo.sh > s195-lancio-promo-copia.diff', shell=True)
print('ok', s.count('s191'), s.count('s190'), s.count('s189'))
