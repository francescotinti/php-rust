#!/usr/bin/env python3
# gen-promozione.py — genera s184-promozione.sh e s184-lancio-promo.sh come COPIE DICHIARATE di
# ../wp181-harness/s181-promozione.sh e s181-lancio-promo.sh (ogni sostituzione con assert di unicità;
# manifest = diff dei due script). Eseguito una volta in S-184 dopo il verdetto rc=0 dell'A/B composto.
import os
os.chdir(os.path.dirname(os.path.abspath(__file__)) + '/..')

def rep(s, old, new, n=1):
    assert s.count(old) == n, (old[:70], s.count(old))
    return s.replace(old, new)

s = open('wp181-harness/s181-promozione.sh', encoding='utf-8').read()
head_old = s[:s.index('set -u\n')]
head_new = '''#!/bin/bash
# s184-promozione.sh — gate di PROMOZIONE COMPOSTA del tree «pin s181 + L-RT1 + L-RT2» (criterio s184-criterio-rt2.md p.5).
# COPIA DICHIARATA di ../wp181-harness/s181-promozione.sh (manifest s184-promozione-copia.diff) coi SOLI adattamenti:
# (1) tag s182 (harness wp184: verdetto s184-promo-verdetto.out, promo-out); (2) CANDIDATO = braccio B dell'A/B composto
# (ab-out/s184-leva/phpr-B ce6ae73f, commit 454a77c3): precondizione A/B rc=0 (ab-out/rt2.rc) e commit in HEAD;
# (3) REF = stash phpr-s181 (19a2faa83a492745) per i gate invarianti; (4) gate bilaterali NUOVI fx-rt1 (FX-RT1 DONE) e
# fx-rt2 (FX-RT2 DONE) accanto a fx-cr1; (5) corpus: ZERO flip attesi (semantica identica per costruzione, batteria CI
# 1748/0 su 454a77c3); (6) conferma post-pin = calls-dq R=5 pin s182 vs stash s181 (stessa toolchain 1.98.1: attesa
# D ≈ [+12,50;+13,17] dell'A/B) + prop-dq (guardia); (7) pin-server.sh s182. Testo S-181/S-180/S-177 conservato sotto:
'''
s = head_new + s[len(head_old):]
s = rep(s, 'H8="$SRC/wp181-harness"', 'H8="$SRC/wp181-harness"\nH9="$SRC/wp184-harness"; H3="$SRC/wp182-harness"')
s = rep(s, 'OUT="$H8/promo-out"; mkdir -p "$OUT"', 'OUT="$H9/promo-out"; mkdir -p "$OUT"')
s = rep(s, 'VERD="$H8/s181-promo-verdetto.out"', 'VERD="$H9/s184-promo-verdetto.out"')
s = rep(s, 'REF="$STASH/phpr-s180"; REF_EXP="884399fc52277119"', 'REF="$STASH/phpr-s181"; REF_EXP="19a2faa83a492745"')
s = rep(s, 'CAND="$H8/ab-out/s181-leva/phpr-B"; CAND_COMMIT="${CAND_COMMIT:-172814ae}"', 'CAND="$H9/ab-out/s184-leva/phpr-B"; CAND_COMMIT="${CAND_COMMIT:-454a77c3}"')
s = rep(s, '"$H8/fx-cr1.php" "$H8/calls-dq.php" "$CAND" "$H8/ab-out/cr1c.rc" \\', '"$H8/fx-cr1.php" "$H9/calls-dq.php" "$H3/fx-rt1.php" "$H9/fx-rt2.php" "$CAND" "$H9/ab-out/rt2.rc" \\')
s = rep(s, '|| stop "PRE: stash phpr-s180 hash != $REF_EXP — STOP"', '|| stop "PRE: stash phpr-s181 hash != $REF_EXP — STOP"')
s = rep(s, '[ "$(cat "$H8/ab-out/cr1c.rc")" = 0 ] || stop "PRE: A/B cr1c rc=$(cat "$H8/ab-out/cr1c.rc") ≠ 0 — nessuna promozione senza nomina — STOP"',
        '[ "$(cat "$H9/ab-out/rt2.rc")" = 0 ] || stop "PRE: A/B rt2 rc=$(cat "$H9/ab-out/rt2.rc") ≠ 0 — nessuna promozione senza nomina — STOP"')
s = rep(s, 'note "PRE: candidato = braccio B $CH (commit $CAND_COMMIT, A/B cr1c rc=0); REF = stash s180 $REF_EXP"',
        'note "PRE: candidato = braccio B $CH (commit $CAND_COMMIT, A/B rt2 rc=0, cifra composta [+12,50;+13,17]); REF = stash s181 $REF_EXP"')
s = rep(s, '"$SRC/scripts/pin-phpr.sh" s181 > "$OUT/pin.log" 2>&1', '"$SRC/scripts/pin-phpr.sh" s182 > "$OUT/pin.log" 2>&1')
s = rep(s, 'note "promozione corpus-gate: rc=0 — nomi==congelato (1412), CONTENUTO==golden, off-on zero (ZERO flip come atteso: L-CR1 a semantica identica per costruzione)"',
        'note "promozione corpus-gate: rc=0 — nomi==congelato (1412), CONTENUTO==golden, off-on zero (ZERO flip come atteso: L-RT1+L-RT2 a semantica identica per costruzione)"')
s = rep(s, 'PHPR="$BIN" R=5 "$SRC/wp97-harness/micro/run-micro.sh" > "$OUT/micro-pin-s181.out" 2>&1\nnote "promozione micro pin s181: $(grep -E \'^rapporto_\' "$OUT/micro-pin-s181.out" | tr \'\\n\' \' \')"',
        'PHPR="$BIN" R=5 "$SRC/wp97-harness/micro/run-micro.sh" > "$OUT/micro-pin-s182.out" 2>&1\nnote "promozione micro pin s182: $(grep -E \'^rapporto_\' "$OUT/micro-pin-s182.out" | tr \'\\n\' \' \')"')
s = rep(s, '# ---- conferma POST-PIN: R=5 pin s181 vs stash s180 (STESSA toolchain 1.98.1) su calls-dq (bersaglio) e prop-dq (guardia) ----',
        '# ---- conferma POST-PIN: R=5 pin s182 vs stash s181 (STESSA toolchain 1.98.1) su calls-dq (bersaglio) e prop-dq (guardia) ----')
s = rep(s, 'note "conferma post-pin calls-dq (pin s181 vs stash s180, stessa toolchain): $(conferma "$H8/calls-dq.php" calls) (attesa: D ≈ cifra dell\'A/B cr1)"',
        'note "conferma post-pin calls-dq (pin s182 vs stash s181, stessa toolchain): $(conferma "$H9/calls-dq.php" calls) (attesa: D ≈ cifra composta dell\'A/B rt2 [+12,50;+13,17])"')
s = rep(s, 'note "conferma post-pin prop-dq (guardia, pin s181 vs stash s180): $(conferma "$H/prop-dq.php" prop) (attesa |D| < 1: sola lettura)"',
        'note "conferma post-pin prop-dq (guardia, pin s182 vs stash s181): $(conferma "$H/prop-dq.php" prop) (attesa |D| < 1: sola lettura)"')
s = rep(s, '"$SRC/scripts/pin-server.sh" s181 > "$OUT/pin-server.log" 2>&1', '"$SRC/scripts/pin-server.sh" s182 > "$OUT/pin-server.log" 2>&1')
s = rep(s, 'note "PROMOZIONE COMPLETA rc=0 (da promo-out/rcb): pin s181 = $H2"', 'note "PROMOZIONE COMPLETA rc=0 (da promo-out/rcb): pin s182 = $H2"')
# gate invarianti: la REF ora è lo stash s181 — etichette dei file e dei messaggi
s = rep(s, '"$REF" "$f" > "$OUT/$n-s180.out" 2>&1', '"$REF" "$f" > "$OUT/$n-s181.out" 2>&1')
s = rep(s, 'if diff -q "$OUT/$n-s180.out" "$OUT/$n-pin.out" > /dev/null; then\n    note "promozione gate $n: pin==stash s180 BYTE-ID ($d)"',
        'if diff -q "$OUT/$n-s181.out" "$OUT/$n-pin.out" > /dev/null; then\n    note "promozione gate $n: pin==stash s181 BYTE-ID ($d)"')
s = rep(s, 'diff "$OUT/$n-s180.out" "$OUT/$n-pin.out" > "$OUT/$n.diff" || true\n    stop "gate $n: pin DIVERGE dallo stash s180 (promo-out/$n.diff)"',
        'diff "$OUT/$n-s181.out" "$OUT/$n-pin.out" > "$OUT/$n.diff" || true\n    stop "gate $n: pin DIVERGE dallo stash s181 (promo-out/$n.diff)"')
# gate bilaterali nuovi dopo fx-cr1
anchor = s[s.index('bilat fxcr1 "$H8/fx-cr1.php"'):]
anchor = anchor[:anchor.index('\n') + 1]
s = rep(s, anchor, anchor +
        'bilat fxrt1 "$H3/fx-rt1.php" "FX-RT1 DONE" "presidio DIRETTO L-RT1 (S-183): cicli locali array/oggetto + gc_collect_cycles, ricorsione, foreach, dinamiche, metodo" -d log_errors=0 -d display_errors=1\n'
        'bilat fxrt2 "$H9/fx-rt2.php" "FX-RT2 DONE" "presidio DIRETTO L-RT2 (S-184): ammissione del Ret fuso — hint int/nullable/TypeError, by-ref alias e deref, forma 0 annidata/null/locali, ret_cell thunk statico, INIT_PROPS, __toString, dtor con chiamata" -d log_errors=0 -d display_errors=1\n')
open('wp184-harness/s184-promozione.sh', 'w', encoding='utf-8').write(s)

l = open('wp181-harness/s181-lancio-promo.sh', encoding='utf-8').read()
l = rep(l, '# s181-lancio-promo.sh — lanciatore DETACHED della catena di promozione s181 (s181-promozione.sh).',
        '# s184-lancio-promo.sh — lanciatore DETACHED della catena di promozione COMPOSTA s184 → pin s182 (s184-promozione.sh).\n# COPIA DICHIARATA di ../wp181-harness/s181-lancio-promo.sh (manifest s184-lancio-promo-copia.diff): soli path/tag.')
l = rep(l, 'H="$SRC/wp181-harness"; O="$H/promo-out"; mkdir -p "$O"', 'H="$SRC/wp184-harness"; O="$H/promo-out"; mkdir -p "$O"')
l = rep(l, 'export PROMO_SP="/private/tmp/promo-s181-sp"', 'export PROMO_SP="/private/tmp/promo-s184-sp"')
l = rep(l, '"$H/s181-promozione.sh" >> "$LOG" 2>&1', '"$H/s184-promozione.sh" >> "$LOG" 2>&1')
open('wp184-harness/s184-lancio-promo.sh', 'w', encoding='utf-8').write(l)
print('ok')
