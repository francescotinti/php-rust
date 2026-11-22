#!/usr/bin/env python3
# gen-promozione-s194.py — genera s194-promozione.sh e s194-lancio-promo.sh come COPIE DICHIARATE di
# ../wp193-harness/s193-promozione.sh e s193-lancio-promo.sh (ogni sostituzione con assert di unicità; manifest = diff dei
# due script: s194-promozione-copia.diff, s194-lancio-promo-copia.diff). Criterio: s194-criterio-promozione.md.
# Generato con assistenza AI (Claude Fable 5.2).
import os, subprocess
os.chdir(os.path.dirname(os.path.abspath(__file__)) + '/..')

def rep(s, old, new, n=1):
    assert s.count(old) == n, (old[:70], s.count(old))
    return s.replace(old, new)

def line_from(s, start):
    assert s.count(start) == 1, (start[:70], s.count(start))
    t = s[s.index(start):]
    return t[:t.index('\n')]

s = open('wp193-harness/s193-promozione.sh', encoding='utf-8').read()
head_old = s[:s.index('set -u\n')]
head_new = '''#!/bin/bash
# s194-promozione.sh — gate di PROMOZIONE DI GRUPPO del tree «pin s185 + leve di S-194» → pin phpr s186 (delibera utente 22/11;
# criterio wp194-harness/s194-criterio-promozione.md; s185 resta nello stash).
# COPIA DICHIARATA di ../wp193-harness/s193-promozione.sh (manifest s194-promozione-copia.diff, generatore
# gen-promozione-s194.py) coi SOLI adattamenti: (1) tag s186 (harness wp194: verdetto s194-promo-verdetto.out, promo-out);
# (2) CANDIDATO = braccio con la ricetta del pin su HEAD (target s192-armD sulla bundle, commit d0c153b9 = ultimo commit di
# codice); precondizione = catena di accettazione su HEAD rc=0 (wp192-harness/out/accettazione-s194P.rc); (3) REF = stash
# phpr-s185 (d9d6429301f60f4e) per i gate invarianti; (4) gate di fixture INVARIATI nella forma, sole etichette s184 → s185;
# (5) CINQUE gate nuovi = i denti di comportamento di S-194 (n, f bilaterali; dbis, dedicate, a invarianti); (6) conferma
# post-pin = guardia di costo sul binario della canonica (REGOLE §6), attese piatte o favorevoli; (7) il passo pin-server.sh
# è TOLTO (criterio p.9: il server s191 di record non si tocca; la canonica si ripristina dallo stash con verifica d'hash); (8) dichiarazione LOC: S-194 non alza cap.
# Testo S-193/S-190/S-185/S-184/S-181/S-180/S-177 conservato sotto:
'''
s = head_new + s[len(head_old):]
s = rep(s, 'H13="$SRC/wp193-harness"', 'H13="$SRC/wp193-harness"; H14="$SRC/wp194-harness"')
s = rep(s, 'OUT="$H13/promo-out"; mkdir -p "$OUT"', 'OUT="$H14/promo-out"; mkdir -p "$OUT"')
s = rep(s, 'VERD="$H13/s193-promo-verdetto.out"', 'VERD="$H14/s194-promo-verdetto.out"')
s = rep(s, 'REF="$STASH/phpr-s184"; REF_EXP="60b10d857eb0343a"', 'REF="$STASH/phpr-s185"; REF_EXP="d9d6429301f60f4e"')
s = rep(s, 'CAND="$HOME/Claude/phpr-target/s192-armB/release/phpr"; CAND_COMMIT="${CAND_COMMIT:-debbca2f}"', 'CAND="$HOME/Claude/phpr-target/s192-armD/release/phpr"; CAND_COMMIT="${CAND_COMMIT:-d0c153b9}"')
s = rep(s, '"$CAND" "$H11/s190-c2-catena-s192gc.rc" \\', '"$CAND" "$SRC/wp192-harness/out/accettazione-s194P.rc" "$H14/s194-n-comportamento.php" "$H14/s194-f-comportamento.php" "$H14/s194-dbis-comportamento.php" "$H14/s194-dedicate-comportamento.php" "$H14/s194-a-comportamento.php" \\')
s = rep(s, '|| stop "PRE: stash phpr-s184 hash != $REF_EXP — STOP"', '|| stop "PRE: stash phpr-s185 hash != $REF_EXP — STOP"')
s = rep(s, line_from(s, '[ "$(cat "$H11/s190-c2-catena-s192gc.rc")" = 0 ]'),
        '[ "$(cat "$SRC/wp192-harness/out/accettazione-s194P.rc")" = 0 ] || stop "PRE: catena di accettazione s194P rc=$(cat "$SRC/wp192-harness/out/accettazione-s194P.rc") ≠ 0 — nessuna promozione senza accettazione verde — STOP"')
s = rep(s, line_from(s, 'note "PRE: candidato = braccio B $CH'),
        'note "PRE: candidato = braccio su HEAD $CH (commit $CAND_COMMIT; catena s194-catena-P rc=0: denti di comportamento, accettazione s194P, sonda); REF = stash s185 $REF_EXP"')
s = rep(s, line_from(s, 'note "PRE: DICHIARAZIONE dei cambiamenti del pin s185 rispetto a s184'),
        'note "PRE: DICHIARAZIONE dei cambiamenti del pin s186 rispetto a s185 (per nome, criteri wp194-harness/s194-criterio-*.md): leve a/b/c accessori magici (1e9354a5, cd32184f); D builtin per valore risolti una volta (6d2a0e6d); D-bis (dd371530); op dedicati Strlen/Count per deroga utente (182da85d); N builtin in namespace (0588daa4); F funzioni di altri file legate per sito (e074b5f8); A3 confronti senza valore intermedio (a88b1d05); inoltre registro delle istruzioni (d5674650, d0c153b9: dietro op-census, spento nel pin). Cifre di S-194 = ISTRUZIONI sul carico vero (ORM -7,0 %, WordPress -6,2 %), non tempo: il tempo lo dice la notte."')
s = rep(s, line_from(s, 'note "PRE: LOC (REGOLE §8): S-192 NON alza alcun cap'),
        'note "PRE: LOC (REGOLE §8): S-194 NON alza alcun cap (diff del dente loc_dente vuoto da e0a71c40; il dente resta nella batteria di questa catena)."')
s = rep(s, 'note "promozione: build ricetta di HEAD $HEAD0 = braccio B $CH AL BYTE"', 'note "promozione: build ricetta di HEAD $HEAD0 = braccio $CH AL BYTE"')
s = rep(s, '≠ braccio B $CH (target separata', '≠ braccio $CH (target separata')
s = rep(s, '"$SRC/scripts/pin-phpr.sh" s185 > "$OUT/pin.log" 2>&1', '"$SRC/scripts/pin-phpr.sh" s186 > "$OUT/pin.log" 2>&1')
s = rep(s, '(ZERO flip come atteso: corpus-gate ×2 verde a ogni leva di S-192, catene s192*)"',
        '(ZERO flip come atteso: corpus-gate ×2 verde a ogni leva di S-194 e nella catena s194P)"')
s = rep(s, 'micro-pin-s185.out', 'micro-pin-s186.out', 2)
s = rep(s, 'note "promozione micro pin s185: ', 'note "promozione micro pin s186: ')
s = rep(s, line_from(s, '# ---- conferma POST-PIN: R=5 pin s185 vs stash s184'),
        '# ---- conferma POST-PIN = guardia di costo sul binario della canonica (REGOLE §6): R=5 pin s186 vs stash s185 (STESSA toolchain 1.98.1) su ret-buf-dq-k0, mcalls-dq, calls-dq, prop-dq; D = A(stash) − B(pin): POSITIVO = pin più veloce ----')
s = rep(s, line_from(s, 'note "conferma post-pin ret-buf-dq-k0 (pin s185 vs stash s184'),
        'note "conferma post-pin ret-buf-dq-k0 (guardia, pin s186 vs stash s185, stessa toolchain; ns/iter = 65 Ret): $(conferma "$H12/ret-buf-dq-k0.php" k0) (attesa piatta o favorevole: nessuna leva di S-194 bersaglia il Ret; D ≤ −260 ns/iter = −4 ns/Ret con 5/5 segni = regressione da dichiarare)"')
s = rep(s, line_from(s, 'note "conferma post-pin mcalls-dq (guardia, pin s185 vs stash s184)'),
        'note "conferma post-pin mcalls-dq (guardia, pin s186 vs stash s185): $(conferma "$H10/mcalls-dq.php" mcalls) (attesa piatta o favorevole; D ≤ −4 con 5/5 segni = regressione da dichiarare)"')
s = rep(s, line_from(s, 'note "conferma post-pin calls-dq (guardia, pin s185 vs stash s184)'),
        'note "conferma post-pin calls-dq (guardia, pin s186 vs stash s185): $(conferma "$H9/calls-dq.php" calls) (attesa piatta o favorevole; D ≤ −4 con 5/5 segni = regressione da dichiarare)"')
s = rep(s, line_from(s, 'note "conferma post-pin prop-dq (guardia, pin s185 vs stash s184)'),
        'note "conferma post-pin prop-dq (guardia, pin s186 vs stash s185): $(conferma "$H/prop-dq.php" prop) (attesa piatta o favorevole; D ≤ −4 con 5/5 segni = regressione da dichiarare)"')
i = s.index('"$SRC/scripts/pin-server.sh" s191'); j = s.index('echo 0 > "$OUT/rcb"')
s = s[:i] + '''# criterio s194 p.9: il server s191 di record NON si tocca. La build del workspace nella canonica ha però riscritto
# release/php-server (senza la feature del pin): lo si RIPRISTINA dallo stash e si verifica l'hash del pin di record.
SRV_EXP="c413398a44f9b0a4"
cp "$STASH/php-server-s191" "$HOME/Claude/php-rust-output/release/php-server" || stop "server: ripristino dallo stash FALLITO"
[ "$(shasum -a 256 "$HOME/Claude/php-rust-output/release/php-server" | cut -c1-16)" = "$SRV_EXP" ] || stop "server: hash dopo il ripristino != $SRV_EXP — STOP"
note "promozione server: passo pin-server.sh TOLTO (criterio s194 p.9); canonica ripristinata dallo stash php-server-s191 = $SRV_EXP (pin di record intatto)"

''' + s[j:]
s = rep(s, 'note "PROMOZIONE COMPLETA rc=0 (da promo-out/rcb): pin s185 = $H2"', 'note "PROMOZIONE COMPLETA rc=0 (da promo-out/rcb): pin s186 = $H2"')
s = rep(s, '"$OUT/$n-s184.out"', '"$OUT/$n-s185.out"', 3)
s = rep(s, 'note "promozione gate $n: pin==stash s184 BYTE-ID ($d)"', 'note "promozione gate $n: pin==stash s185 BYTE-ID ($d)"')
s = rep(s, 'stop "gate $n: pin DIVERGE dallo stash s184 (promo-out/$n.diff)"', 'stop "gate $n: pin DIVERGE dallo stash s185 (promo-out/$n.diff)"')
s = rep(s, '# gate INVARIANTE: pin==stash s184 byte-id (+ marcatore quando preteso)', '# gate INVARIANTE: pin==stash s185 byte-id (+ marcatore quando preteso)')
s = rep(s, '"§3.33 fixture di CONTEGGI GC: invariante vs stash s184 (dry-run 20/11 verde anche con IdMap e gc_note_this_tail), non bilaterale"',
        '"§3.33 fixture di CONTEGGI GC: invariante vs stash s185 (prova a secco 22/11 col candidato), non bilaterale"')
new_gates = '''# ---- gate NUOVI di S-194 (criterio p.5): i denti di comportamento delle leve promosse ----
bilat s194n "$H14/s194-n-comportamento.php" "" "dente N (S-194): builtin non qualificati in un namespace, ombra di funzione utente, dichiarazione tardiva" -d log_errors=0 -d display_errors=1
bilat s194f "$H14/s194-f-comportamento.php" "" "dente F (S-194): funzioni di altri file legate per sito, builtin per riferimento" -d log_errors=0 -d display_errors=1
invar s194dbis "$H14/s194-dbis-comportamento.php" "" "dente D-bis (S-194): cammino diretto dei builtin per valore INVARIATO"
invar s194ded "$H14/s194-dedicate-comportamento.php" "" "dente Strlen/Count (S-194): coercizioni, avvisi, TypeError INVARIATI"
invar s194a "$H14/s194-a-comportamento.php" "" "dente A3 (S-194): confronti fra interi/float/stringhe/null/riferimenti INVARIATI (non bilaterale: tre avvisi Undefined variable PRE-esistenti)"

'''
s = rep(s, '# E2 (emenda S-177', new_gates + '# E2 (emenda S-177')
body = s[s.index('set -u\n'):]
for bad in ('s184', 'pin s185', 'pin-s185', 's193-promo', 's192gc', 'pin-server.sh" s19'):
    assert bad not in body, bad
open('wp194-harness/s194-promozione.sh', 'w', encoding='utf-8').write(s); os.chmod('wp194-harness/s194-promozione.sh', 0o755)
open('wp194-harness/s194-promozione-copia.diff', 'w', encoding='utf-8').write(subprocess.run(['diff', 'wp193-harness/s193-promozione.sh', 'wp194-harness/s194-promozione.sh'], capture_output=True, text=True).stdout)

l = open('wp193-harness/s193-lancio-promo.sh', encoding='utf-8').read()
lh_old = l[:l.index('set -u\n')]
lh_new = '''#!/bin/bash
# s194-lancio-promo.sh — lanciatore DETACHED della catena di promozione s194 → pin s186 (s194-promozione.sh).
# COPIA DICHIARATA di ../wp193-harness/s193-lancio-promo.sh (manifest s194-lancio-promo-copia.diff, generatore
# gen-promozione-s194.py): soli path/tag (wp194, PROMO_SP promo-s194-sp, lock TOKEN s194-promo). Come in S-193: Data ≥ 10G
# PRIMA del lock, attesa quiete CI, lock scritto dal lanciatore e rimosso a fine corsa, sentinelle a inizio e fine nel log.
# Esiti: promo-out/lancio-promo.rc (rc della catena) + lancio-promo.done.
'''
l = lh_new + l[len(lh_old):]
l = rep(l, 'H="$SRC/wp193-harness"; O="$H/promo-out"; mkdir -p "$O"', 'H="$SRC/wp194-harness"; O="$H/promo-out"; mkdir -p "$O"')
l = rep(l, 'export PROMO_SP="/private/tmp/promo-s193-sp"', 'export PROMO_SP="/private/tmp/promo-s194-sp"')
l = rep(l, 'echo "s193 promozione pin s185 $(date \'+%F %T\') pid=$$" > "$MLOCK"', 'echo "s194 promozione pin s186 $(date \'+%F %T\') pid=$$" > "$MLOCK"')
l = rep(l, 'l "CI quieta — lock scritto: $MLOCK (TOKEN s193-promo)"', 'l "CI quieta — lock scritto: $MLOCK (TOKEN s194-promo)"')
l = rep(l, '"$H/s193-promozione.sh" >> "$LOG" 2>&1', '"$H/s194-promozione.sh" >> "$LOG" 2>&1')
assert 's193' not in l[l.index('set -u\n'):]
open('wp194-harness/s194-lancio-promo.sh', 'w', encoding='utf-8').write(l); os.chmod('wp194-harness/s194-lancio-promo.sh', 0o755)
open('wp194-harness/s194-lancio-promo-copia.diff', 'w', encoding='utf-8').write(subprocess.run(['diff', 'wp193-harness/s193-lancio-promo.sh', 'wp194-harness/s194-lancio-promo.sh'], capture_output=True, text=True).stdout)
print('ok')
