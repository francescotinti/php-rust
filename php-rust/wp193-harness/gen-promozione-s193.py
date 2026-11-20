#!/usr/bin/env python3
# gen-promozione-s193.py — genera s193-promozione.sh e s193-lancio-promo.sh come COPIE DICHIARATE di
# ../wp190-harness/s190-promozione.sh e s190-lancio-promo.sh (ogni sostituzione con assert di unicità; manifest = diff dei
# due script: s193-promozione-copia.diff, s193-lancio-promo-copia.diff). Eseguito in S-193 (20/11) dopo: catene di
# accettazione S-192 tutte rc=0 (ultima: s190-c2-catena-s192gc.rc), gate server verdi sul sorgente (s192-gate-server-g1.out),
# dry-run delle 25 fixture col braccio B debbca2f (df39c0a92937db0f): tutti i bilaterali == oracle, tutti gli invarianti ==
# stash s184, expdiff fxmc2 == expected-diff ⇒ NESSUN gate ri-dichiarato. Generato con assistenza AI (Claude Fable 5.2).
import os
os.chdir(os.path.dirname(os.path.abspath(__file__)) + '/..')

def rep(s, old, new, n=1):
    assert s.count(old) == n, (old[:70], s.count(old))
    return s.replace(old, new)

def line_from(s, start):
    assert s.count(start) == 1, (start[:70], s.count(start))
    t = s[s.index(start):]
    return t[:t.index('\n')]

s = open('wp190-harness/s190-promozione.sh', encoding='utf-8').read()
head_old = s[:s.index('set -u\n')]
head_new = '''#!/bin/bash
# s193-promozione.sh — gate di PROMOZIONE del tree «pin s184 + SEI leve di S-192» → pin phpr s185 (NEXT §S-193 ordine 2;
# ordine deciso dall'utente il 19/11: promozione di giorno, poi la notte la conferma o la smentisce; s184 resta nello stash).
# COPIA DICHIARATA di ../wp190-harness/s190-promozione.sh (manifest s193-promozione-copia.diff, generatore
# gen-promozione-s193.py) coi SOLI adattamenti: (1) tag s185 (harness wp193: verdetto s193-promo-verdetto.out, promo-out);
# (2) CANDIDATO = braccio B dei gemelli s192gc (target s192-armB sulla bundle, df39c0a92937db0f, commit debbca2f = ultimo
# commit di codice: da lì a HEAD solo documenti); precondizione = ultima catena di accettazione S-192 rc=0
# (wp190-harness/s190-c2-catena-s192gc.rc) e commit in HEAD; (3) REF = stash phpr-s184 (60b10d857eb0343a) per i gate
# invarianti; (4) gate di fixture INVARIATI nella forma (dry-run 20/11: 25/25 verdi col candidato), sole etichette s183 → s184;
# (5) corpus: ZERO flip attesi (corpus-gate ×2 verde a ogni leva di S-192); (6) conferma post-pin = ret-buf-dq-k0 R=5 pin s185
# vs stash s184 col SEGNO di S-192 (gemelli g3: K0 −5,08 ns/Ret a favore della leva ⇒ attesa D = A−B ≈ +330 ns/iter) +
# mcalls-dq/calls-dq/prop-dq (guardie, attese piatte); (7) pin-server.sh s191; (8) dichiarazione LOC: S-192 non alza cap.
# Testo S-190/S-185/S-184/S-181/S-180/S-177 conservato sotto:
'''
s = head_new + s[len(head_old):]
s = rep(s, 'H11="$SRC/wp190-harness"; H12="$SRC/wp186-harness"', 'H11="$SRC/wp190-harness"; H12="$SRC/wp186-harness"; H13="$SRC/wp193-harness"')
s = rep(s, 'OUT="$H11/promo-out"; mkdir -p "$OUT"', 'OUT="$H13/promo-out"; mkdir -p "$OUT"')
s = rep(s, 'VERD="$H11/s190-promo-verdetto.out"', 'VERD="$H13/s193-promo-verdetto.out"')
s = rep(s, 'REF="$STASH/phpr-s183"; REF_EXP="9a6ae11bf7e900ef"', 'REF="$STASH/phpr-s184"; REF_EXP="60b10d857eb0343a"')
s = rep(s, 'CAND="$H11/ab-out/s190-leva/phpr-B3"; CAND_COMMIT="${CAND_COMMIT:-8bff279e}"', 'CAND="$HOME/Claude/phpr-target/s192-armB/release/phpr"; CAND_COMMIT="${CAND_COMMIT:-debbca2f}"')
s = rep(s, '"$CAND" "$H11/catena-out/catena-s190.done" \\', '"$CAND" "$H11/s190-c2-catena-s192gc.rc" \\')
s = rep(s, '|| stop "PRE: stash phpr-s183 hash != $REF_EXP — STOP"', '|| stop "PRE: stash phpr-s184 hash != $REF_EXP — STOP"')
s = rep(s, line_from(s, 'grep -q \'^rc=0 \' "$H11/catena-out/catena-s190.done"'),
        '[ "$(cat "$H11/s190-c2-catena-s192gc.rc")" = 0 ] || stop "PRE: catena di accettazione s192gc rc=$(cat "$H11/s190-c2-catena-s192gc.rc") ≠ 0 — nessuna promozione senza accettazione verde — STOP"')
s = rep(s, line_from(s, 'note "PRE: candidato = braccio B3 $CH'),
        'note "PRE: candidato = braccio B $CH (commit $CAND_COMMIT, gemelli ORM s192gc; accettazione S-192 rc=0 su ogni leva: batteria 1773/0/2, corpus-gate ×2, sentinelle); REF = stash s184 $REF_EXP"')
s = rep(s, line_from(s, 'note "PRE: DICHIARAZIONE dei cambiamenti del pin s184 rispetto a s183'),
        'note "PRE: DICHIARAZIONE dei cambiamenti del pin s185 rispetto a s184 (per nome, criteri wp192-harness/s192-criterio-*.md): (a) K0 gc_note_this_tail (0ac0b35b); (b) hasher Fx in lower/compile (8b253f18); (c) cache realpath validata (15019848); (d) array_diff a insieme hash (0eca36da, 82244d79); (e) memo __destruct/__construct (1b11e848); (f) registro oggetti vivi IdMap (debbca2f); inoltre contatori census-time (bb0e84dd, cec724cf: gated OFF nel pin). Cifre S-192 = DIREZIONE su una gamba ORM; WordPress mai rimisurato: se t27/t29 differiscono da t26/t28 l\'attribuzione va istruita per leva, mai d\'ufficio."')
s = rep(s, line_from(s, 'note "PRE: salite LOC entrate dopo s183'),
        'note "PRE: LOC (REGOLE §8): S-192 NON alza alcun cap (il dente loc_dente non è toccato dalla promozione s184, a485e626; resta nella batteria di questa catena)."')
s = rep(s, '"$SRC/scripts/pin-phpr.sh" s184 > "$OUT/pin.log" 2>&1', '"$SRC/scripts/pin-phpr.sh" s185 > "$OUT/pin.log" 2>&1')
s = rep(s, '(ZERO flip come atteso: corpus-gate ×2 verde sul tree, catena c2b, e CI OK su cbc3ce5a/a28c376f)"',
        '(ZERO flip come atteso: corpus-gate ×2 verde a ogni leva di S-192, catene s192*)"')
s = rep(s, 'micro-pin-s184.out', 'micro-pin-s185.out', 2)
s = rep(s, 'note "promozione micro pin s184: ', 'note "promozione micro pin s185: ')
s = rep(s, line_from(s, '# ---- conferma POST-PIN: R=5 pin s184 vs stash s183'),
        '# ---- conferma POST-PIN: R=5 pin s185 vs stash s184 (STESSA toolchain 1.98.1) su ret-buf-dq-k0 (bersaglio della leva K0) e mcalls-dq/calls-dq/prop-dq (guardie); D = A(stash) − B(pin): POSITIVO = pin più veloce ----')
s = rep(s, line_from(s, 'note "conferma post-pin ret-buf-dq-k0 (pin s184 vs stash s183'),
        'note "conferma post-pin ret-buf-dq-k0 (pin s185 vs stash s184, stessa toolchain; ns/iter = 65 Ret): $(conferma "$H12/ret-buf-dq-k0.php" k0) (attesa col segno di S-192: D ≈ +330 ns/iter = 65 × 5,08 dei gemelli g3; D ≤ 0 = leva K0 non confermata sul canonico, da dichiarare)"')
s = rep(s, line_from(s, 'note "conferma post-pin mcalls-dq (guardia, pin s184 vs stash s183)'),
        'note "conferma post-pin mcalls-dq (guardia, pin s185 vs stash s184): $(conferma "$H10/mcalls-dq.php" mcalls) (attesa piatta: gemelli g3 +0,33; |D| < 4)"')
s = rep(s, line_from(s, 'note "conferma post-pin calls-dq (guardia, pin s184 vs stash s183)'),
        'note "conferma post-pin calls-dq (guardia, pin s185 vs stash s184): $(conferma "$H9/calls-dq.php" calls) (attesa piatta: gemelli g3 +0,50; |D| < 4)"')
s = rep(s, line_from(s, 'note "conferma post-pin prop-dq (guardia, pin s184 vs stash s183)'),
        'note "conferma post-pin prop-dq (guardia, pin s185 vs stash s184): $(conferma "$H/prop-dq.php" prop) (attesa |D| < 1: sola lettura)"')
s = rep(s, '"$SRC/scripts/pin-server.sh" s190 > "$OUT/pin-server.log" 2>&1', '"$SRC/scripts/pin-server.sh" s191 > "$OUT/pin-server.log" 2>&1')
s = rep(s, 'note "PROMOZIONE COMPLETA rc=0 (da promo-out/rcb): pin s184 = $H2"', 'note "PROMOZIONE COMPLETA rc=0 (da promo-out/rcb): pin s185 = $H2"')
s = rep(s, '"$OUT/$n-s183.out"', '"$OUT/$n-s184.out"', 3)
s = rep(s, 'note "promozione gate $n: pin==stash s183 BYTE-ID ($d)"', 'note "promozione gate $n: pin==stash s184 BYTE-ID ($d)"')
s = rep(s, 'stop "gate $n: pin DIVERGE dallo stash s183 (promo-out/$n.diff)"', 'stop "gate $n: pin DIVERGE dallo stash s184 (promo-out/$n.diff)"')
s = rep(s, '# gate INVARIANTE: pin==stash s183 byte-id (+ marcatore quando preteso)', '# gate INVARIANTE: pin==stash s184 byte-id (+ marcatore quando preteso)')
s = rep(s, '"§3.33 fixture di CONTEGGI GC: invariante vs stash s183 (come in s186/s190-leva-build), non bilaterale"',
        '"§3.33 fixture di CONTEGGI GC: invariante vs stash s184 (dry-run 20/11 verde anche con IdMap e gc_note_this_tail), non bilaterale"')
body = s[s.index('set -u\n'):]
for bad in ('s183', 'pin s184', 'pin-s184', 's190-promo', 'catena-s190.done'):
    assert bad not in body, bad
open('wp193-harness/s193-promozione.sh', 'w', encoding='utf-8').write(s)

l = open('wp190-harness/s190-lancio-promo.sh', encoding='utf-8').read()
lh_old = l[:l.index('set -u\n')]
lh_new = '''#!/bin/bash
# s193-lancio-promo.sh — lanciatore DETACHED della catena di promozione s193 → pin s185 (s193-promozione.sh).
# COPIA DICHIARATA di ../wp190-harness/s190-lancio-promo.sh (manifest s193-lancio-promo-copia.diff, generatore
# gen-promozione-s193.py): path/tag (wp193, PROMO_SP promo-s193-sp, lock TOKEN s193-promo) e UN gate nuovo: Data ≥ 10G
# PRIMA del lock (veto S-188 «Data <10G al lancio»; la build a freddo sulla canonica ne ricrea ~5). Come in S-190: attesa
# quiete CI, lock scritto dal lanciatore PRIMA della catena e rimosso a fine corsa, sentinelle a inizio e fine nel log.
# Esiti: promo-out/lancio-promo.rc (rc della catena) + lancio-promo.done.
'''
l = lh_new + l[len(lh_old):]
l = rep(l, 'H="$SRC/wp190-harness"; O="$H/promo-out"; mkdir -p "$O"', 'H="$SRC/wp193-harness"; O="$H/promo-out"; mkdir -p "$O"')
l = rep(l, 'export PROMO_SP="/private/tmp/promo-s190-sp"', 'export PROMO_SP="/private/tmp/promo-s193-sp"')
l = rep(l, 'sentinelle inizio\n', 'sentinelle inizio\nDG=$(df -g /System/Volumes/Data | awk \'NR==2{print $4}\')\nif [ "${DG:-0}" -lt 10 ]; then l "STOP: Data ${DG}G < 10G al lancio (veto S-188) — nessuna catena"; echo 7 > "$LRC"; echo "rc=7 rcb=assente (Data ${DG}G < 10G)" > "$LDONE"; exit 7; fi\n')
l = rep(l, 'echo "s190 promozione pin s184 $(date \'+%F %T\') pid=$$" > "$MLOCK"', 'echo "s193 promozione pin s185 $(date \'+%F %T\') pid=$$" > "$MLOCK"')
l = rep(l, 'l "CI quieta — lock scritto: $MLOCK (TOKEN s190-promo)"', 'l "CI quieta — lock scritto: $MLOCK (TOKEN s193-promo)"')
l = rep(l, '"$H/s190-promozione.sh" >> "$LOG" 2>&1', '"$H/s193-promozione.sh" >> "$LOG" 2>&1')
assert 's190' not in l[l.index('set -u\n'):]
open('wp193-harness/s193-lancio-promo.sh', 'w', encoding='utf-8').write(l)
print('ok')
