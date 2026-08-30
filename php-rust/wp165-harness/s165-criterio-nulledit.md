# S-165 — criterio braccio C «null-edit» (PRE-registrato): fondare la banda-layout host-call

## Reperto che lo impone (arbitrato 2 dell'emenda S-161 #4, prima del R=5 mc1b)
mc1bsm: giudice mc2 +16,0 (terza conferma del segno: +19,0/+17,5/+16,0) ma
guardie host-call mordono con run_loop a Δ=+4 bl (disasm-mc1b.out):
arrload −6,0 (rumore 1) · missload −5,0 (rumore 5) · arrfilter −6,0 (rumore
≤3). La cura outline SMENTISCE il canale «taglia di run_loop»; anche il
canale meccanico è escluso dal sorgente (arbitrato 1). Resta il canale:
**layout GLOBALE del binario** (LTO fat + cgu=1: ogni edit sposta gli
indirizzi di tutto; missload −5/−6/−8 su TRE bracci diversi senza che il suo
cammino sia mai toccato).

## Braccio C (esperimento discriminante)
Tree = pin s163 + SOLO `unreachable!`×2 in calls.rs (sostituzione di bracci
MORTI mai eseguiti: semanticamente NULLO — fixture e parità lo provano).
Build ricetta A′, stash via pin-phpr.sh --braccio s165-nulledit-C.

## Attese PRE-registrate (smoke R=3 vs A=pin, stesso copione TAG=nulledit)
1. Giudice mc2: |D| < 4 (C non contiene il fast path) — rc atteso 4/8, NON
   è un fallimento: qui il «giudice» è solo un'altra categoria di controllo.
2. Guardie host-call (missload/arrload/arrfilter/strmap/arrmap/hostargs/
   refl/backtrace): i loro D misurano la BANDA-LAYOUT per categoria.
   - Se |D| ≥ 4-6 ricorre su categorie NON toccate ⇒ banda-layout host-call
     FONDATA = max|D| osservato per categoria (N=1, come banda-layout S-103);
     il criterio mc1 si EMENDA (guardie host-call a soglia
     max(4, rumore, banda-layout_cat)) e il giudizio si RIESEGUE (R=5 col
     criterio emendato, REGOLE §3) — mai assoluzione ex post: l'emenda vale
     solo per il rerun.
   - Se C esce PULITO (tutte |D| < 4) ⇒ il layout NON spiega i morsi di
     mc1bsm: il prezzo è specifico dei bracci MC1 (canale ignoto) ⇒ la leva
     NON si promuove in S-165; reperto a verbale e coda.
3. Ogni esito va a verbale con: bl run_loop di C (atteso ≈6082, l'edit non
   tocca run.rs), fixture fx-mc A==C, parità 2 modi dello stash script.
