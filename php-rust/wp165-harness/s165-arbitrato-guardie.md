# S-165 — ARBITRATO guardie morse allo smoke mc1sm (emenda rev. S-161 #4), PRIMA del R=5

## Reperto (s165-mc1sm-verdetto.out, R=3)
Giudice mc2 D=+19,0 (soglia 4, rumore ≤2, DENTRO banda [4;30]). Guardie morse:
missload −8,0 · arrload −7,0 (rumore ≤3). Nello stesso run: refl +15,0 e
objdropdef +13,3 MIGLIORATE oltre il loro rumore ⇒ oscillazione BILATERALE
su categorie non-bersaglio.

## Arbitrato dal SORGENTE (canale meccanico escluso per NOME)
1. m-missload.php: loader CLOSURE — per-miss via `call_closure_one`
   (INTATTO al byte in questo edit); il driver non contiene alcuna chiamata
   a metodo ⇒ l'arm nuovo di `Op::MethodCall` NON esegue MAI su questo
   cammino. Zero alloc/dispatch cambiati.
2. m-arrload.php: loader `[$l,'loadClass']` — per-miss via `call_method_one`,
   toccato SOLO dall'`unreachable!` su un braccio MORTO (mai eseguito:
   sostituisce un return equivalente, dead code da S-163); anche qui nessuna
   chiamata a metodo nel driver ⇒ arm nuovo mai eseguito.
3. Entrambi i driver macinano `run_loop` (corpo del loop + corpo loader):
   l'unico canale rimasto è la TAGLIA/LAYOUT di run_loop (+79 righe, bl
   6082→6127, Δ=+45 dichiarato nel disasm gate) — stessa classe del reperto
   S-103/S-104 (banda-layout; H-C2 icache). Un census qui misurerebbe
   Δ=0 alloc per costruzione (nessun sito d'alloc toccato su quei cammini):
   arbitro non discriminante, si dichiara invece il canale dal sorgente.

## Regola di decisione R=5 (PRE-registrata, TAG mc1r5, DSM=+19.0)
- Giudice mc2: soglia max(4, rumore drop-1); riconciliazione con D_smoke
  +19,0 a banda max(4, rumore).
- Guardie missload/arrload: se a R=5 RIENTRANO (D ≥ −soglia) ⇒ il morso R=3
  era rumore/layout transitorio, promozione con catena piena (batteria,
  corpus 2 modi, fixture bilaterali, micro R=5, denti).
- Se PERSISTONO sotto −soglia ⇒ prezzo di layout REALE: NIENTE promo di
  questo braccio; variante di cura L-MC1b «outline» (fast path in metodo
  `#[inline(never)]`, run_loop torna ~piccolo, si paga 1 call sul giudice)
  = BRACCIO NUOVO con stesso criterio e TAG nuovo, stesso protocollo.
- In ogni esito: nessuna assoluzione ex post — questo file È l'arbitrato,
  scritto PRIMA del R=5.
