#!/usr/bin/env python3
# gen-promozione-s190.py — genera s190-promozione.sh e s190-lancio-promo.sh come COPIE DICHIARATE di
# ../wp185-harness/s185-promozione.sh e s185-lancio-promo.sh (ogni sostituzione con assert di unicità;
# manifest = diff dei due script: s190-promozione-copia.diff, s190-lancio-promo-copia.diff). Eseguito una volta in
# S-190 diurna (04/11) dopo: guardia b3 rc=0 (notte 03→04/11), CI OK su cbc3ce5a/a28c376f, dry-run delle fixture col
# braccio B3 (23 gate: tutti i bilaterali == oracle, tutti gli invarianti == stash s183 TRANNE wp172 fx-mc2 §3.28,
# mosso dalla cura 22/10: C4 == oracle, C1 residua ⇒ gate expdiff dichiarato). Generato con assistenza AI (Claude Fable 5).
import os
os.chdir(os.path.dirname(os.path.abspath(__file__)) + '/..')

def rep(s, old, new, n=1):
    assert s.count(old) == n, (old[:70], s.count(old))
    return s.replace(old, new)

s = open('wp185-harness/s185-promozione.sh', encoding='utf-8').read()
head_old = s[:s.index('set -u\n')]
head_new = '''#!/bin/bash
# s190-promozione.sh — gate di PROMOZIONE COMPOSTA del tree «pin s183 + cura §3.34/§3.35 (22/10) + O(1) v2 (S-190) + routing
# cli-server in php-types (S-188) + timeout wrapper http §3.39 (S-189) + main 72f688c3 (S-187: PR #3/#4/#7, portabilità Linux)
# + feature census-time 901515b6 (gated, mai attiva nel pin)» → pin phpr s184 (criterio wp190-harness/s190-criterio-o1v2.md
# p.7, NEXT ordine 3). COPIA DICHIARATA di ../wp185-harness/s185-promozione.sh (manifest s190-promozione-copia.diff,
# generatore gen-promozione-s190.py) coi SOLI adattamenti: (1) tag s184 (harness wp190: verdetto s190-promo-verdetto.out,
# promo-out); (2) CANDIDATO = braccio B3 della guardia (ab-out/s190-leva/phpr-B3 ad57fc29, commit 8bff279e): precondizione
# guardia b3 rc=0 (catena-out/catena-s190.done) e commit in HEAD; (3) REF = stash phpr-s183 (9a6ae11bf7e900ef) per i gate
# invarianti; (4) gate wp172 fx-mc2 (§3.28) da INVARIANTE a DICHIARATO: diff(oracle, pin) == fx-mc2-wp172.expected-diff
# (la cura 22/10 ha curato il caso (ii) C4 — == oracle — e mosso la forma del caso (i) C1: né invariante vs s183 né byte-id;
# funzione expdiff NUOVA); gate invariante NUOVO fx-sw2-gc (conteggi GC §3.33 vs stash s183); (5) corpus: ZERO flip attesi
# (corpus-gate ×2 verde sul tree in dev-release — catena c2b — e in CI su cbc3ce5a/a28c376f; nomi 1407 per modo);
# (6) conferma post-pin = ret-buf-dq-k0 R=5 pin s184 vs stash s183 (bersaglio della guardia b3: attesa A−B ≈ −200 ns/iter
# = 65 Ret × −3,08; tetto 65 × 4 = 260) + mcalls-dq (attesa ≈ −2,33) + calls-dq/prop-dq (|D| < 1); (7) pin-server.sh s190
# (il server riporta la cura O(1) v2: s189 portava la v1); (8) ri-dichiarazione delle salite LOC entrate dopo s183 (REGOLE §8).
# Testo S-185/S-184/S-181/S-180/S-177 conservato sotto:
'''
s = head_new + s[len(head_old):]
s = rep(s, 'H10="$SRC/wp185-harness"', 'H10="$SRC/wp185-harness"\nH11="$SRC/wp190-harness"; H12="$SRC/wp186-harness"')
s = rep(s, 'OUT="$H10/promo-out"; mkdir -p "$OUT"', 'OUT="$H11/promo-out"; mkdir -p "$OUT"')
s = rep(s, 'VERD="$H10/s185-promo-verdetto.out"', 'VERD="$H11/s190-promo-verdetto.out"')
s = rep(s, 'REF="$STASH/phpr-s182"; REF_EXP="159eee40be506171"', 'REF="$STASH/phpr-s183"; REF_EXP="9a6ae11bf7e900ef"')
s = rep(s, 'CAND="$H10/ab-out/s185-leva/phpr-B"; CAND_COMMIT="${CAND_COMMIT:-4176ea3a}"', 'CAND="$H11/ab-out/s190-leva/phpr-B3"; CAND_COMMIT="${CAND_COMMIT:-8bff279e}"')
s = rep(s, '"$H10/fx-mc2.php" "$H10/mcalls-dq.php" "$CAND" "$H10/ab-out/mc2.rc" \\',
        '"$H10/fx-mc2.php" "$H10/mcalls-dq.php" "$H12/ret-buf-dq-k0.php" "$H4/fixtures/fx-sw2-gc.php" "$H11/fx-mc2-wp172.expected-diff" "$CAND" "$H11/catena-out/catena-s190.done" \\')
s = rep(s, '|| stop "PRE: stash phpr-s182 hash != $REF_EXP — STOP"', '|| stop "PRE: stash phpr-s183 hash != $REF_EXP — STOP"')
s = rep(s, '[ "$(cat "$H10/ab-out/mc2.rc")" = 0 ] || stop "PRE: A/B mc2 rc=$(cat "$H10/ab-out/mc2.rc") ≠ 0 — nessuna promozione senza nomina — STOP"',
        'grep -q \'^rc=0 \' "$H11/catena-out/catena-s190.done" || stop "PRE: guardia b3 NON verde ($(cat "$H11/catena-out/catena-s190.done")) — nessuna promozione senza guardia rc=0 — STOP"')
s = rep(s, 'note "PRE: candidato = braccio B $CH (commit $CAND_COMMIT, A/B mc2 rc=0, cifra composta [+19,00;+19,17]); REF = stash s182 $REF_EXP"',
        'note "PRE: candidato = braccio B3 $CH (commit $CAND_COMMIT, guardia b3 rc=0 04/11 02:45: K0 +3,08 ns/Ret, K piatto, mcalls +2,33, E3 2,17, micro 6/6); REF = stash s183 $REF_EXP"\n'
        'note "PRE: DICHIARAZIONE dei cambiamenti del pin s184 rispetto a s183 (per nome): (a) cura §3.34/§3.35 (4aa30319, 22/10: dtor dei locali al leave) + cura O(1) v2 (S-190: cursore gc_probe_mark, vm/gc_probe, nota operandi binop lento, leva iv); (b) routing cli-server spostato in php-types (S-188 C1); (c) timeout del wrapper http §3.39 (S-189, php-builtins/http_timeout); inoltre main 72f688c3 (S-187: PR #3/#4/#7 portabilità Linux) e feature census-time (901515b6, gated OFF nel pin). Se t26/t28 differiscono da t25/t27 l\'attribuzione va istruita per braccio, mai d\'ufficio alla cura O(1)."\n'
        'note "PRE: salite LOC entrate dopo s183 (REGOLE §8, ri-dichiarate qui come in S-186 #4): vm/mod.rs cap 25990 → 26082 e vm/run.rs cap 7543 → 7611 (commit 4aa30319 §3.35, 901515b6 census-time, f600e32f O(1) v1); S-190 NON alza alcun cap (mod.rs 26072 ≤ 26082, run.rs 7611 = cap, gc_probe.rs nuovo 78)."')
s = rep(s, '"$SRC/scripts/pin-phpr.sh" s183 > "$OUT/pin.log" 2>&1', '"$SRC/scripts/pin-phpr.sh" s184 > "$OUT/pin.log" 2>&1')
s = rep(s, 'note "promozione corpus-gate: rc=0 — nomi==congelato (1412), CONTENUTO==golden, off-on zero (ZERO flip come atteso: L-MR1+L-MC2 a semantica identica per costruzione, prova per nome criterio-mc2 p.8)"',
        'note "promozione corpus-gate: rc=0 — nomi==congelato (1407 per modo), CONTENUTO==golden, off-on zero (ZERO flip come atteso: corpus-gate ×2 verde sul tree, catena c2b, e CI OK su cbc3ce5a/a28c376f)"')
s = rep(s, 'PHPR="$BIN" R=5 "$SRC/wp97-harness/micro/run-micro.sh" > "$OUT/micro-pin-s183.out" 2>&1\nnote "promozione micro pin s183: $(grep -E \'^rapporto_\' "$OUT/micro-pin-s183.out" | tr \'\\n\' \' \')"',
        'PHPR="$BIN" R=5 "$SRC/wp97-harness/micro/run-micro.sh" > "$OUT/micro-pin-s184.out" 2>&1\nnote "promozione micro pin s184: $(grep -E \'^rapporto_\' "$OUT/micro-pin-s184.out" | tr \'\\n\' \' \')"')
s = rep(s, '# ---- conferma POST-PIN: R=5 pin s183 vs stash s182 (STESSA toolchain 1.98.1) su mcalls-dq (bersaglio) e calls-dq/prop-dq (guardie) ----',
        '# ---- conferma POST-PIN: R=5 pin s184 vs stash s183 (STESSA toolchain 1.98.1) su ret-buf-dq-k0 (bersaglio della guardia b3) e mcalls-dq/calls-dq/prop-dq (guardie); D = A(stash) − B(pin): negativo = pin più lento ----')
s = rep(s, 'note "conferma post-pin mcalls-dq (pin s183 vs stash s182, stessa toolchain): $(conferma "$H10/mcalls-dq.php" mcalls) (attesa: D ≈ cifra composta dell\'A/B mc2 [+19,00;+19,17])"',
        'note "conferma post-pin ret-buf-dq-k0 (pin s184 vs stash s183, stessa toolchain; ns/iter = 65 Ret): $(conferma "$H12/ret-buf-dq-k0.php" k0) (attesa: D ≈ −200 ns/iter = 65 × −3,08 della guardia b3; tetto |D| < 260 = 65 × 4)"\n'
        'note "conferma post-pin mcalls-dq (guardia, pin s184 vs stash s183): $(conferma "$H10/mcalls-dq.php" mcalls) (attesa: D ≈ −2,33 della guardia b3, |D| < 4)"')
s = rep(s, 'note "conferma post-pin calls-dq (guardia, pin s183 vs stash s182): $(conferma "$H9/calls-dq.php" calls) (attesa |D| < 1: Ret fuso delle funzioni già ammesso)"',
        'note "conferma post-pin calls-dq (guardia, pin s184 vs stash s183): $(conferma "$H9/calls-dq.php" calls) (attesa |D| < 1: guardia b3 +0,33)"')
s = rep(s, 'note "conferma post-pin prop-dq (guardia, pin s183 vs stash s182): $(conferma "$H/prop-dq.php" prop) (attesa |D| < 1: sola lettura)"',
        'note "conferma post-pin prop-dq (guardia, pin s184 vs stash s183): $(conferma "$H/prop-dq.php" prop) (attesa |D| < 1: sola lettura)"')
s = rep(s, '"$SRC/scripts/pin-server.sh" s183 > "$OUT/pin-server.log" 2>&1', '"$SRC/scripts/pin-server.sh" s190 > "$OUT/pin-server.log" 2>&1')
s = rep(s, 'note "PROMOZIONE COMPLETA rc=0 (da promo-out/rcb): pin s183 = $H2"', 'note "PROMOZIONE COMPLETA rc=0 (da promo-out/rcb): pin s184 = $H2"')
# gate invarianti: la REF ora è lo stash s183 — etichette dei file e dei messaggi
s = rep(s, '"$OUT/$n-s182.out"', '"$OUT/$n-s183.out"', 3)
s = rep(s, 'note "promozione gate $n: pin==stash s182 BYTE-ID ($d)"', 'note "promozione gate $n: pin==stash s183 BYTE-ID ($d)"')
s = rep(s, 'stop "gate $n: pin DIVERGE dallo stash s182 (promo-out/$n.diff)"', 'stop "gate $n: pin DIVERGE dallo stash s183 (promo-out/$n.diff)"')
s = rep(s, '# gate INVARIANTE: pin==stash s182 byte-id (+ marcatore quando preteso)', '# gate INVARIANTE: pin==stash s183 byte-id (+ marcatore quando preteso)')
# gate DICHIARATO nuovo (expdiff) definito prima del primo gate
first_gate = 'bilat fxce "$H/fx-ce.php" "" "12 forme class/interface/trait_exists"\n'
s = rep(s, first_gate,
        '# gate DICHIARATO (S-190): diff(oracle, pin) == expected-diff committato — per un caso che la cura 22/10 ha MOSSO\n'
        '# (né invariante vs stash né byte-id con l\'oracle): la forma residua è dichiarata per nome nel catalogo.\n'
        'expdiff(){ # $1=nome $2=file $3=expected-diff $4=descrizione\n'
        '  local n="$1" f="$2" e="$3" d="$4"\n'
        '  "$ORACLE" "$f" > "$OUT/$n-oracle.out" 2>&1\n'
        '  "$BIN" "$f" > "$OUT/$n-pin.out" 2>&1\n'
        '  [ -s "$OUT/$n-pin.out" ] || stop "gate $n: output pin VUOTO"\n'
        '  diff "$OUT/$n-oracle.out" "$OUT/$n-pin.out" > "$OUT/$n.diff" || true\n'
        '  if cmp -s "$OUT/$n.diff" "$e"; then note "promozione gate $n: diff(oracle,pin) == expected-diff ($d)"; else stop "gate $n: diff(oracle,pin) ≠ expected-diff dichiarato (promo-out/$n.diff vs $e)"; fi\n'
        '}\n' + first_gate)
s = rep(s, 'invar fxmc2 "$H/fx-mc2.php" "" "§3.28 ordine SEND_VAR_EX/dtor temp PRE-esistente INVARIATO"',
        'expdiff fxmc2 "$H/fx-mc2.php" "$H11/fx-mc2-wp172.expected-diff" "§3.28: (ii) C4 CURATA dalla cura 22/10 (== oracle), (i) C1 residua nella forma post-22/10 — expected-diff dichiarato S-190"')
sw1 = s[s.index('bilat fxsw1 "$H4/fixtures/fx-sw1.php"'):]
sw1 = sw1[:sw1.index('\n') + 1]
s = rep(s, sw1, sw1 + 'invar fxsw2gc "$H4/fixtures/fx-sw2-gc.php" "FX-SW2 DONE" "§3.33 fixture di CONTEGGI GC: invariante vs stash s183 (come in s186/s190-leva-build), non bilaterale"\n')
assert '$this' not in s.split('expdiff fxmc2')[1].split('\n')[0]
open('wp190-harness/s190-promozione.sh', 'w', encoding='utf-8').write(s)

l = open('wp185-harness/s185-lancio-promo.sh', encoding='utf-8').read()
lh_old = l[:l.index('set -u\n')]
lh_new = '''#!/bin/bash
# s190-lancio-promo.sh — lanciatore DETACHED della catena di promozione COMPOSTA s190 → pin s184 (s190-promozione.sh).
# COPIA DICHIARATA di ../wp185-harness/s185-lancio-promo.sh (manifest s190-lancio-promo-copia.diff, generatore
# gen-promozione-s190.py): SOLI path/tag (wp190, PROMO_SP promo-s190-sp, lock TOKEN s190-promo). Come in S-185: attesa
# quiete CI (nessun cargo/rustc/phpt-runner, coda vuota; doppia verifica a 60 s), lock scritto dal lanciatore PRIMA della
# catena e rimosso a fine corsa. Via daemonize.pl; nessun token «phpr»/«php-server» nell'argv (falso positivo di
# s129-quiescenza): PROMO_SP è esportata QUI e il suo valore non contiene il token. Sentinelle (Data, swap, CPU totale,
# top-5) a inizio e fine nel log. Esiti: promo-out/lancio-promo.rc (rc della catena, letto da promo-out/rcb) + lancio-promo.done.
'''
l = lh_new + l[len(lh_old):]
l = rep(l, 'H="$SRC/wp185-harness"; O="$H/promo-out"; mkdir -p "$O"', 'H="$SRC/wp190-harness"; O="$H/promo-out"; mkdir -p "$O"')
l = rep(l, 'export PROMO_SP="/private/tmp/promo-s185-sp"', 'export PROMO_SP="/private/tmp/promo-s190-sp"')
l = rep(l, 'echo "s185 promozione pin s183 $(date \'+%F %T\') pid=$$" > "$MLOCK"', 'echo "s190 promozione pin s184 $(date \'+%F %T\') pid=$$" > "$MLOCK"')
l = rep(l, 'l "CI quieta — lock scritto: $MLOCK (TOKEN s185-promo)"', 'l "CI quieta — lock scritto: $MLOCK (TOKEN s190-promo)"')
l = rep(l, '"$H/s185-promozione.sh" >> "$LOG" 2>&1', '"$H/s190-promozione.sh" >> "$LOG" 2>&1')
open('wp190-harness/s190-lancio-promo.sh', 'w', encoding='utf-8').write(l)
print('ok')
