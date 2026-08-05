# Team «misura-server» — Concilio WP-102, fase 2

Relatore: team sedie 7 (Leijen), 9 (Gregg), 6 (Pedersen). Fonte vincolante = verbali individuali
(`verbale-7-leijen.md`, `verbale-9-gregg.md`, `verbale-6-pedersen.md`); questo file compone, non sostituisce.

**Fatto post-fase-1 registrato**: il mode-probe del server (A-PE-102-1 — dump dell'unità nel log del
server, atteso dal contratto) è GIÀ implementato nella sentinella estesa e verrà esercitato nel
ri-collaudo del pin nuovo. La refutazione capitale R1-Pedersen ha quindi già la sua cura in codice:
resta da ESERCITARLA (i launcher devono ASSERIRE il modo per braccio, non solo loggarlo) prima di
riaccreditare qualunque cifra «server modo X» (KS-PE-102-1).

## Convergenze

1. **Bande peak: asimmetriche, calibrate, mai sotto il rumore.** Leijen (R5, A-LE-102-3,
   KS-LE-102-2) e Gregg (A-GR-102-3, KS-GR-102-2) convergono per NOME: gate anti-regressione a UN
   lato; larghezza ≥ rumore run-to-run MISURATO dello strumento nella stessa finestra, PER MOTORE
   (phpr 0,5% vs oracle +14,6% intra-sera: una banda unica è vuota); banda senza calibrazione
   pubblicata = VOID. Si adotta la forma più severa (Leijen): R≥5 sul pin, statistica = mediana
   (lezione WP-91), spread pubblicato — compatibile e inclusiva di KS-GR-102-2.
2. **Nessuna cifra senza il binario che l'ha prodotta.** Leijen R2/A-LE-102-4 (il «full peak 1929,0
   default» è lo stato di un binario MAI misurato: la coppia girò su a2772e62, il pin promosso nasce
   dopo il flip fb861e4; il corpus per NOME non gatea il footprint) e Pedersen R2/A-PE-102-5 (la
   gradazione «server-HTTP ✓» accredita a f2ab0636 una gamba eseguita da phpr CLI; hash gamba B
   registrato-non-gate). Stessa refutazione in due domini: ogni numero pubblicato porta l'HASH del
   binario misurato; ogni gradazione nomina il binario che la esegue.
3. **Gate che non può fallire per la causa che sorveglia = vacuo.** Pedersen R1 (fails=0 ×2
   indistinguibile da env mai arrivato) e Leijen R2 (parità che benedice il footprint) sono la stessa
   lezione: il controllo positivo del canale è parte del gate. Il mode-probe già implementato è la
   risposta al primo; lo smoke peak sul pin promosso (A-LE-102-4, ~1 min) al secondo.
4. **La gamba OFF non deve invecchiare al buio.** Gregg §FONDAMENTALI(c) nomina il +95 MiB come
   secondo rischio d'oggetto trascurato; Leijen ne fa l'oggetto di R1/R3. Voce rinominata
   «**crescita d'albero: OFF +95 / ON +36,5 MiB**» (A-LE-102-2): la crescita tocca ENTRAMBE le
   gambe; «gamba OFF» pre-selezionava la spiegazione.
5. **H-C1 ha prerequisiti nominati, non opzionali.** Gregg A-GR-102-4 (ri-baseline sei-categorie
   in modo default PRIMA del criterio: la baseline oggi è eterogenea, due regimi) + A-GR-102-1
   (census statico 18 vs 9 op/iter validato col contatore DINAMICO op-census) + KS-GR-102-1 (soglia
   dal SUO micro isolante, mai dal 27% né dalla «tariffa» 9-10 ns/op); Pedersen A-PE-102-6 (H-C1
   tocca clone/drop/gc_note = la classe di retention WP-78 ⇒ sentinella estesa + endurance nel suo
   ordine di collaudo).

## Conflitti (posizione per sedia)

- **Verdetto su S-100.** Gregg: sessione d'oggetto piena, NESSUNA refutazione capitale (flip
  giudicato da misure). Leijen: DUE capitali (etichetta «cross-albero» non provata; peak del default
  = numero del candidato). Pedersen: UNA capitale (parità server bimodale senza controllo positivo).
  Non è conflitto di merito — domini disgiunti (oggetto vs contabilità footprint vs confine server) —
  ma i tre verdetti restano distinti e vincolanti ciascuno nel suo dominio.
- **Calibro del rumore peak.** Gregg calibra sulla cifra esistente (+14,6% intra-sera oracle);
  Leijen esige spread R≥5 fresco prima di OGNI banda futura (senza = VOID) e nota che il peak è una
  statistica di MASSIMO (coda destra: R=2 non calibra). Risoluzione di team: forma Leijen (più
  severa) come regola, cifra Gregg come stima provvisoria MAI usabile da sola come banda.
- **Ordine della bozza §S-101.** Gregg APPROVA l'ordine (misura-first, ri-baseline al punto 1) con
  emendamenti; Leijen chiede di RIFORMULARE il punto 4 (attribuzione) prima dell'esecuzione: A/B
  pin sigillati stessa-sera PRIMA del bisect, census allocatore prima del bisect (il census dà il
  CANALE, il bisect solo il commit a 5× il costo). Composizione: l'ordine regge, il punto 4 si
  riscrive nella forma A-LE-102-1 — nessuna incompatibilità residua.
- **Sigillo eager dei due env lazy (Pedersen R4).** Nessuna sedia si oppone, ma è apparato che non
  blocca la ri-baseline: il team lo colloca al prossimo tocco del pin server (vedi priorità), non
  nell'ordine bloccante — Pedersen lo accetta per costruzione (KS-PE-102-2 vincola i NUOVI `PHPR_*`,
  non impone la data del sigillo sugli esistenti).

## Priorità per l'ordine S-101

**BLOCCANTI (per NOME, in quest'ordine):**

1. **Ri-baseline sei-categorie in modo default** (bozza punto 1) — prerequisito del criterio H-C1
   (A-GR-102-4 vincolante).
2. **Attribuzione «crescita d'albero» riformulata** (ex punto 4): prima mossa = **A/B stessa-sera
   dei due pin sigillati S-99 (`phpr-s99-sigillo`) vs S-100, stesso ambiente** (A-LE-102-1); se il
   delta si riproduce ⇒ census allocatore (vmmap per fase + stats mimalloc,
   `MIMALLOC_PURGE_DELAY=0`) PRIMA del bisect. **KS-LE-102-1: delta < 2× spread intra-sera ⇒ voce
   riclassificata AMBIENTE, bisect VIETATO.** Entrambe le gambe si misurano (A-LE-102-2).
3. **Calibrazione rumore full-peak** R≥5 per motore, mediana + spread pubblicato, bande d'ora in
   poi unilaterali (A-LE-102-3 + A-GR-102-3). KS: banda senza spread pubblicato = VOID
   (KS-LE-102-2, KS-GR-102-2).
4. **Smoke peak media sul pin promosso 725a2ffa SUBITO** (~1 min; full alla prossima coppia); se
   fuori dallo spread calibrato del candidato ⇒ il «1929,0» si RITIRA dalla rotazione e il flip si
   ricollauda sul footprint (A-LE-102-4, KS-LE-102-3). Ogni peak in rotazione porta l'hash del
   binario.
5. **Ri-collaudo del pin nuovo col mode-probe** (A-PE-102-1, GIÀ implementato): i launcher
   ASSERISCONO il modo effettivo per braccio (non solo log); hash phpr in gamba B promosso a GATE
   (A-PE-102-5). KS-PE-102-1: nessuna cifra «server modo X» senza asserzione del modo effettivo.
6. **Se H-C1 si scrive in S-101** (condizionale ma non opzionale una volta attivato): (a) census
   dinamico op-census valida il 18 vs 9 (A-GR-102-1); (b) soglia di caduta dal micro isolante ≥
   pavimento sonda (KS-GR-102-1); (c) sentinella estesa + **endurance N≥100 richieste su endpoint
   allocante + RSS/footprint server inizio/fine con banda calibrata** nel suo ordine di collaudo
   (A-PE-102-3 + A-PE-102-6).

**BACKLOG per NOME (apparato-solo-se-blocca):**

- **A-PE-102-2** — sigillo eager + value-parse a lista chiusa per `PHPR_STUB_ELISION` e
  `PHPR_UNIT_CACHE` (vm/mod.rs:15718/15727) + `PHPR_REQ_NS` (worker_pool.rs:155); chiude
  A-PE-101-2 per NOME. Si attiva al prossimo tocco/ri-pin del server (cambia il binario ⇒ nuovo
  pin comunque).
- **A-PE-102-4** — una gamba WP VERA servita da php-server via HTTP; fino ad allora la gradazione
  NON scrive «server-HTTP» per la gamba B (che resta CLI phpunit, correttamente nominata).
- **A-GR-102-2** — riprofilo inline-aware del 50% run_loop (frame pointers o `#[inline(never)]`
  temporaneo) per dare a H-C1 una BANDA di meccanismo; diventa bloccante solo al momento del
  GIUDIZIO di H-C1.
- **R5-Pedersen / `--build-info`** — resta nel backlog aperto A-HO-101-4/A-PE-101-5 (spawn
  fuori-launcher non collaudato); nessun nuovo apparato finché non blocca una cifra pubblicata.
- **Residui sentinella (R3-Pedersen)** — worker-id nel burst, status HTTP asserito, restapi vera al
  posto di «restapi-shaped»: si compongono nell'endurance quando A-PE-102-3 si attiva, non come
  atti separati.

**Regola apparato-solo-se-blocca applicata**: endurance, riprofilo inline-aware e sigillo eager
sono NOMINATI con la loro condizione di attivazione; nessuno entra nell'ordine bloccante finché la
condizione non scatta.
