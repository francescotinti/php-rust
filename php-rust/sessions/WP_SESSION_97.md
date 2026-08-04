# WP_SESSION_97 — S-97.0 + S-97.1: la spina dorsale al lavoro — un'ipotesi spedita, una refutata a tavolino, una eseguita e caduta sul suo criterio

**In una frase**: abbiamo misurato con precisione perché il cuore del nostro
motore è molto più lento dell'originale, provato una cura che elimina quasi
metà dei passi inutili — il programma va davvero più veloce, ma non quanto
la soglia che ci eravamo imposti prima di iniziare — e quindi, come da
regola, l'abbiamo accantonata e sappiamo ora che il problema non è il numero
dei passi ma il costo di ciascuno.

**Data**: 2026-08-04 sera (S-97.0) → 2026-08-05 notte (S-97.1).

## Contesto: le prime due sessioni della SPINA DORSALE

Il cambio di rotta dell'utente (2026-08-04) ha sostituito l'agenda-residuo
con: obiettivo (nucleo ≤ 3× l'oracle sulle categorie pure), giudice (le sei
micro-categorie di `wp97-harness/micro/`), ipotesi in sequenza con criterio
di caduta scritto PRIMA. Queste due sessioni sono le prime eseguite sotto le
nuove regole — e le regole hanno morso, che è il segno che funzionano.

## S-97.0 (verdetti, dai `.out` committati)

- **Decomposizione esatta del 18,5× di `arith`**
  (`wp97-harness/arith-decomposition.out`): opcode/iterazione 20 contro 7
  (2,86×) × costo per opcode 7,95 ns contro 1,23 (6,5×) = 18,6 contro il
  18,5 misurato indipendente — la decomposizione torna.
- **H1 (A-ZV1) REFUTATA SENZA SCRIVERE CODICE**: `Op::Binary` prende gli
  operandi dalla PILA, il clone è già avvenuto in `LoadSlot`; un fast-path
  per riferimento lì non risparmia nulla. Tre sessioni di rinvio del "piano
  B" chiuse da una lettura di 10 minuti del bytecode.
- **H-A2 CONFERMATA E SPEDITA** (`ha2-sweep.out`): il Sweep doppione dei
  blocchi fra graffe eliso all'emissione (solo `StmtKind::Block`; la regola
  generale sarebbe SCORRETTA sul ramo falso degli `if`). 20→19 opcode/iter,
  −0,88% di tempo. ⭐ la scoperta di lato: un Sweep noop costa un quinto
  dell'opcode medio ⇒ il costo per opcode NON è uniforme.
- Pin ruotato d5ce86e3342f3926 → 2f6c1a696b560755 (stash `phpr-s97-ha2`).

## S-97.1 — H-A1: eseguita per intero, caduta sul suo criterio

Il piano diceva «cablare `enum Operand` su Binary/CmpJmp». La ricognizione
in storia git ha mostrato che lo stadio 2 era GIÀ stato scritto in tre forme
in WP-44 e revertato su verdetto d'aggregato — e che la forma migliore (v3
«raw registers», `f4c80cf`) era esattamente ciò che serviva, mai misurata
dal giudice micro. Riarmata quella invece di riscrivere la v1 (che era la
forma PIÙ lenta delle tre): 7 shape monomorfe u16, pass a finestre con remap
totale, tutto dietro `PHPR_REG_LOWER` (unit-cache già `reg_mode`-aware).

Adattamenti alla deriva da luglio: liveness ESAUSTIVA classificata a mano
(i denti A-TH-97-2/A-SK-97-2 hanno morso come progettato: la variante nuova
non compila finché non la classifichi); `reg_load_slot` seed-aware via
`unit_slot_name` (WP-65: il `{main}` linkato cede `slot_names` DOPO il
pass); census a 185 righe.

**Verdetto (`wp97-harness/ha1-registers.out`, coppia stessa-sera R=3 sullo
stesso binario 0dd98ebbb7eb2d96):**

- braccio 1 (opcode/iter < 12): **PASSATO** — 19 → **11** (census esatto:
  1.900.020 → 1.100.019 dispatch su 100k iterazioni);
- braccio 2 (arith −40%): **NON RAGGIUNTO** — 7,83 → 5,43 s netti =
  **−30,7%** (rapporto 18,2 → 12,6);
- ⇒ **il criterio scatta, H-A1 abbandonata senza negoziare** (regola di
  metodo n.3). Il fold ulteriore della coda AssignOp (11→9 possibile) è
  NOMINATO nel `.out` ma deliberatamente non scritto.

Collaterali della stessa coppia: prop −12,3%, calls −15,7%, arr −11,5%,
str/re nel rumore. Il codice resta in albero DORMIENTE dietro il flag
(flag-off zero-delta; direttiva no-revert): a differenza di WP-44, il
flag-on VINCE il suo micro — i due verdetti coesistono, giudici diversi.

**Parità (flag off)**: cargo 1214/0 (batteria v3 13 snippet + test negativo
const-first; sentinelle distruttori comprese); corpus Zend per NOME
**identico** (1418 nomi, diff vuoto; log riga-per-riga uguale salvo le sei
righe `random_bytes` note; totali 5305/2649/1418/1238). Gate cifre `--all`
PASS a HEAD cd4fb41 (budget di cardinalità del corpus alzato
deliberatamente per le tre fonti nuove; il valore vive nel file di budget,
mai citato a memoria).

## ⭐ Lezioni

- ⭐⭐ **Il costo per opcode SALE quando togli gli opcode economici**: 8,24 →
  9,87 ns tolto il 42% dei dispatch. Terza conferma della non-uniformità
  (dopo ha2-sweep), e stavolta PINNATA sul residuo: 11 opcode a ~9,9 ns
  contro 7 a 1,23. **Il conteggio è quasi chiuso; il divario vive nel
  COSTO per opcode (~8×)** — l'asse è H-B1/H-B2.
- ⭐⭐ **Un revert d'aggregato può nascondere un vincitore di categoria**: la
  stessa identica leva (v3) è «fallita» su WordPress (+1,0%) e vince il 30,7%
  su `arith`. Nessuno dei due verdetti refuta l'altro: cambia il giudice,
  cambia il claim. Il codice in storia git era riusabile al 90% — riaprire
  un arco chiuso costa poco se il lavoro fu archiviato bene.
- ⭐⭐ **Il fold commutativo della v3 era un buco di soundness dormiente**:
  `3+$x` → `$x+3` inverte l'ordine dei nomi in "Unsupported operand types"
  quando `$x` non è numerico a runtime. Il corpus WP-44 non lo prese mai —
  «corretto per fortuna del corpus» ≠ «corretto» (S-96.0) applicato
  RETROATTIVAMENTE a codice già gate-verde due volte. Rimosso; resta il solo
  mirror dei confronti, che non hanno errori operando-tipizzati.
- ⭐⭐ **Il criterio scritto prima ha fatto il suo lavoro**: a −30,7% la
  tentazione di «un fold ancora e ci siamo» era concreta; la regola n.3
  l'ha spenta in una riga. Il residuo è nominato per il concilio, non
  inseguito in sessione.
- ⭐ **Un `tail` in fondo alla pipe maschera l'exit della build**: la prima
  build census era FALLITA (`--bin phpr` non sta in php-runtime) ma `tail`
  ha restituito 0; il binario stantio ha prodotto un «pass che non riscrive»
  — smascherato dal controllo positivo (`strings | grep BinarySSDst` = 0,
  dump flag-on≡flag-off). Sempre pretendere la prova positiva che il flag
  ha morso PRIMA di leggere una misura.
- ⭐ I match esaustivi di S-96.0 hanno funzionato da dente alla prima
  occasione reale: le 7 varianti nuove NON compilavano finché non
  classificate in liveness. Il costo del refactoring pagato allora si è
  ripagato qui.

## NON fatti (dichiarati)

- Il fold della coda AssignOp (LoadSlot+Swap+BinaryDst → forma fusa, 11→9):
  nominato in `ha1-registers.out`, non scritto (il criterio era già
  scattato).
- Nessuna misura full/media WordPress (per regola: WordPress è collaudo di
  parità, e l'emissione flag-off NON è cambiata).
- H-B1 non iniziata: il suo criterio di caduta esatto va scritto in
  apertura di S-98.0, prima di toccare codice.

## Stato binari e processi

- phpr parità: **0dd98ebbb7eb2d96** (HEAD cd4fb41; stash additivo
  `phpr-s97-ha1`; contiene le varianti registro dormienti). Precedente
  2f6c1a696b560755 in `phpr-s97-ha2`.
- php-server: RICOSTRUITO come effetto collaterale della build workspace →
  **832568a72b925dd1** (stash additivo `php-server-s97`; il pin precedente
  f8f4295a1dcdb627 resta in `php-server-wp94`). Parità server NON
  riverificata in sessione (nessun lavoro server); da riverificare al primo
  uso.
- Census: `phpr-opcensus-target` (build `php-cli --features
  php-runtime/op-census`).
- Nessun processo orfano; uploads non toccati (nessun full run).
