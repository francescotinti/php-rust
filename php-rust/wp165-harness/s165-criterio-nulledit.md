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

## ESITO C (s165-nulledit-verdetto.out, rc=5) + braccio D PRE-registrato
C: giudice mc2 +3,5 SOTTO soglia (CONTROLLO: il segnale +16..+19 è della
leva) · arrload −5,0 (rumore 2) ANCHE sul nullo ⇒ costo degli unreachable!×2
(call_method_one sul cammino) · missload +1,0 PULITO su C ⇒ i suoi morsi su
B1/B2 = layout INTERNO di run_loop (l'edit dell'arm sposta gli altri arm) ·
host-call NON toccate su C tutte |D|<4 ⇒ banda-layout globale NON fondata ≥4
· objchurn −20 con rumore 16,7 = morso marginale nel suo carattere di rumore
(a verbale). DECISIONE (ramo pre-registrato «C pulito sulle non toccate» +
attribuzione arrload): **braccio D = MC1b PURA senza unreachable!×2**
(calls.rs torna al pin; az.rev. S-163 #4 chiusa con verdetto di misura:
«unreachable!×2 costa ~5 ns su arrload: NON si monta, verbale al posto del
dente»). Attese D (smoke R=3, TAG mc1d): mc2 ≥ +12 in banda; arrload
RIENTRA (|D|<4 o comunque ≥ −4); missload ARBITRO FINALE: se ≥ −4 ⇒ catena
di promozione; se < −4 persiste ⇒ prezzo layout-arm REALE ⇒ NIENTE promo
S-165, leva in coda con banda-layout run_loop da fondare (probe dedicato).

## ESITO D (s165-mc1d-verdetto.out, rc=5) + EMENDA banda-layout (per il rerun, mai ex post)
D: mc2 +14,5 (4ª conferma) · missload −1,0 RIENTRATA (arbitro pre-registrato
SODDISFATTO) · arrload −3,0 RIENTRATA (attribuzione unreachable CONFERMATA) ·
arrfilter −4,0 morde per decimali (rumore ≤2). ARBITRATO arrfilter dai 5
bracci: −4,0/−2,0/−6,0/−3,0(NULLO)/−4,0 — canale NON-semantico (il driver non
ha chiamate a metodo; morde anche sul braccio nullo): banda-layout di
categoria mai fondata. **FONDAZIONE (N=bracci nulli-per-categoria S-165):
banda-layout arrfilter = 6,0 (max|D| su 5 bracci non-semantici) ·
banda-layout missload = 8,0 (max|D| su B1/B2, nulli per il suo cammino)** —
arrload NON fondabile (C toccava call_method_one sul suo cammino): resta
max(4, rumore). **EMENDA criterio (REGOLE §3)**: copione s165-ab-mc1e.sh =
copia dichiarata di s165-ab-mc1.sh con BAND missload=8,0 e arrfilter=6,0
(manifest s165-ab-mc1e-copia.diff); il giudizio si RIESEGUE a R=5 (TAG
mc1dr5, DSM=+14,5) col criterio emendato. Promozione SOLO se R=5 rc=0.
