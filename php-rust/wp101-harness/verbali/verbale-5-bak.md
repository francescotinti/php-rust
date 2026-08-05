# Verbale Sedia 5 — Lars Bak (microarchitettura, path caldi, alloc-rate)
## Concilio WP-101 su S-99.0 + bozza §S-100

## VERDETTO

Metodo della pre-misura SOLIDO (target-dir separato, albero ripristinato,
smoke semantico, stessa finestra, tre binari): la DIREZIONE regge — rollout
nelle forme registro morto, percorso pila vivo. Ma la decomposizione 57/43
è pubblicata come quanto fisico quando è uno split dipendente dall'ordine
di rimozione; il criterio pre-registrato 0,7 ns siede SOTTO il pavimento
dichiarato della sonda; e §S-100 punto 3 ripropone il metodo
conteggio×costo senza il profilo a campioni (A-BA-100-3) che avevo chiesto.

## Refutazioni capitali

**R1 — Le componenti di D SI MESCOLANO.** INT1−C2=2,67 non è "traffico Vec"
puro: il pop di lhs in INT1 aggiunge un branch (Option/expect) + memcpy 24B
+ store di len, e il push un capacity-check; C2 sul hit fa last()+last_mut()
(doppio check) + store in place. Su un core OoO questi micro-costi si
SOVRAPPONGONO col carico del payload: C0−INT1 e INT1−C2 sono UN ordine di
rimozione, il termine d'interazione è attribuito in silenzio a una gamba.
Inoltre lo spread INT1 (0,05 s ≈ ±0,33 ns/occ) rende lo split 52–62%, non
"57%": la mia stima 50-70% è "confermata" da una misura la cui banda copre
mezzo intervallo. Si pubblichi come BANDA con l'interazione nominata.

**R2 — Kill-switch sotto il pavimento.** premisura-rollout99.out dichiara
pavimento sonda 1,0 ns (KS-BA-100-2) e criterio di riapertura D_registro ≥
0,7 ns/occ: un criterio SOTTO la risoluzione della sonda non è
aggiudicabile — non può né mordere né assolvere onestamente.

**R3 — Il controfattuale "branch costante = gratis" vale al MICRO.** Con 3
siti caldi il predittore mangia; con 186 corpi e centinaia di copie inlined
di `binary_fast` la pressione BTB/PHT su WordPress è un'altra bestia
(lezione V8). La banda [0, 0,5] vale per il giudice micro; NON esportarla
al macro senza un campione.

**R4 — Provenienza stantia nel file VERDICT.** micro-rebaseline99.out ha in
testa "S-97.0" e pin 4e268c3f (pre-sigillo) mentre il pin corrente è
52330330; e la sessione pubblica DUE arith flag-off (7,52 rebaseline vs
7,44 gamba 4c) senza dire quale è LA baseline: 17,5 diventa 17,3 con
l'altra gamba. R=3 con spread fino a 0,09 s: le cifre decimali dei rapporti
sono al limite della risoluzione.

**R5 — Il census del punto 4 rischia il bersaglio sbagliato due volte.**
(a) Va eseguito sull'emissione POST-FLIP (il punto 2 ritira i siti
register-eligible: frequenze misurate su un'emissione in pensione =
census void). (b) `cmp int-int` ai loop-head vive in CmpJmp/CmpJmpConst
(run.rs:982-1004), che chiamano binary_value col plumbing INTERO
(call+marshalling+push del Bool evitato solo a metà): un census che conta
solo Binary(cmp) manca il consumatore più caldo del meccanismo H-B2.

## Emendamenti

- **A-BA-101-1**: pubblicare la decomposizione come bande
  (call/marshalling 52–62%) col termine d'interazione dichiarato; INT2
  (in-place tenuto, call reintrodotta) SOLO se una decisione di rollout
  dipenderà dallo split — oggi è descrittivo, non decisionale.
- **A-BA-101-2**: la prima misura H-C (§S-100 punto 3) include il profilo
  a campioni sul loop di `prop.php` (A-BA-100-3 eseguito, entrambi i
  motori) come gamba CO-EQUALE alla tavola conteggio×costo — la classe
  "conteggio senza cammino critico" è già stata refutata in S-98.
- **A-BA-101-3**: census del punto 4 = frequenza per-op sul giudice con
  emissione post-flip, forme fuse (CmpJmp*) contate a parte; l'atteso si
  scala per frequenza PRIMA di scegliere l'occorrenza.

## Kill-switch

- **KS-BA-101-1**: ogni sonda futura su D_registro pubblica una BANDA e il
  suo pavimento; se criterio < pavimento, la sonda è VOID e il criterio va
  rialzato al pavimento.
- **KS-BA-101-2**: se il census post-flip mostra che l'occorrenza scelta
  pesa così poco che atteso < risoluzione della coppia di misura,
  l'occorrenza CADE A TAVOLINO prima di ogni riga di codice.
