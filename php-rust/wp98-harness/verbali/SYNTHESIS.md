# SINTESI DI CONVERGENZA — Concilio WP-98 (su report S-96.0 + programma §WP-97)

Protocollo a due fasi: 9 bozze INDIPENDENTI (`verbale-*.md`), poi 3 team
tematici composti sui punti di questa sessione (`team-oggetto`, `team-analisi`,
`team-catena`). I verbali individuali restano la fonte VINCOLANTE.

## §FONDAMENTALI (in testa per regola utente 2026-08-03)

**(a) Avanzamento dell'OGGETTO in S-96.0 — Gregg, mandato inverso: NO.**
Due soli fatti nuovi sul MOTORE — che il difetto di soundness non ricorre nel
corpus WordPress (delta F1 esattamente zero, con la fixture che prova che il
difetto è reale) e la divergenza §3.10 sui `TypeError` dei builtin, **trovata
per caso** — contro sei sull'APPARATO che lo misura, e **zero misure di tempo**.
Nessuna sedia difende il contrario. Il fatto più prezioso della sessione (§3.10)
è quello che nessuno stava cercando.

**(b) Contatore sessioni-senza-misura**: ultima full/media cronometrata = WP-94,
**due sessioni fa**; ultima campagna footprint = m90, sei sessioni fa. La rotta
CPU-VM ha consumato tre sessioni (WP-95, WP-96 e la mattina di WP-95) senza mai
far girare un orologio, e si è conclusa decidendo di non costruire la leva.

**(c) Rischio d'oggetto più trascurato**: **una decisione di rotta è stata presa
su un canale SCREEN**. Il moltiplicatore §P1 (il valore del canale) viene da un
profilo R=1 senza spread; tutte le bande di due sessioni ne dipendono. Gregg:
*«SCREEN × VERDICT = SCREEN, e SCREEN non chiude un passo dell'ordine»*. Bak
aggiunge il colpo che rende il conto insalvabile: il costo di un corpo caldo
**non è una costante** — in WP-44 passare da 2 a 9 corpi costò MENO che passare
da 2 a 4 — quindi il tetto è stato usato come una **tariffa**, che non è.

**Regola di ammissione**: l'ordine WP-97 proposto qui sotto è composto di leve
sull'OGGETTO. L'unica voce d'apparato ammessa (P-AMEND-ORFANO) entra perché
rende **irrisolvibile** la provenienza di ogni misura futura, non solo passata.

## Verdetti (9 sedie, NESSUNA benedizione)

Hoare CON EMENDAMENTI · Matsakis esito confermato / motivazione REFUTATA ·
Klabnik **FAIL** (forgia atterrata) · Hejlsberg §WP-97 punto 1 NON istruito ·
Bak PROCEDI CON EMENDAMENTI, chiusura declassata a **sospensione** · Pedersen
**RESPINTO IN PARTE** · Leijen non refutato nel merito, due refutazioni sulle
premesse · Stogov il passo 2 regge ma il perimetro è un TETTO, non un perimetro
fedele · Gregg **APPROVATO CON DECLASSAMENTO**, oggetto NON avanzato.

## Refutazioni capitali

1. **Klabnik — FORGIA ATTERRATA sull'apparato spedito poche ore prima.** Un
   doc-cifra con un accento nel NOME è uscito dal perimetro mentre il gemello
   ASCII veniva nominato: git QUOTA i path non-ASCII e la virgoletta iniziale
   fa fallire l'ancoraggio `^php-rust/`. Il perimetro si fidava della FORMA che
   git sceglie di STAMPARE. **[CHIUSA IN SESSIONE: `-z` + `core.quotePath=false`
   su `ls-files`, la stessa correzione su `check-ignore -v` (che quotava a sua
   volta e faceva sembrare non-ignorati i sidecar accentati), e su `ls-tree`
   — il lato COMMITTATO aveva lo stesso punto cieco, trovato da team-catena.
   Dente permanente T31 col suo morso.]** La CLASSE resta aperta: ~20 siti
   `git status --porcelain` (che quota E collassa le directory untracked:
   servono `-z` e `-uall` insieme) e `diff --name-only`.
2. **Hoare — il raw contiene un delta che il changeset non può produrre.**
   `slot_reads_rc` si conta al sito di lettura e non dipende dall'analisi:
   il −14 è rumore di suite, dello stesso ordine dei −42 già documentati fra
   `before` e F1. Ne segue che le attribuzioni −21/−18/−6 sono state scritte
   **senza controllare il pavimento di rumore**. **[ACCOLTA: annotata nel raw;
   servirebbe una coppia A/A che misuri il pavimento, e non è stata fatta.
   Resta in piedi ciò che non dipende da questo: `would_take`/`would_take_rc`/
   `sites_movable` sono identici al byte fra F1, F2 e riconteggio.]**
3. **Bak — il tetto A-LB-97-1 usato come tariffa.** Il costo per corpo caldo non
   è una costante: dipende da quanto quel corpo è caldo e da che cosa sposta
   fuori dalla i-cache. La chiusura del passo 2 va declassata a SOSPENSIONE.
4. **Matsakis — canale cross-frame.** `current_frame_args` legge gli slot vivi
   di OGNI frame: un canale che il test di TIPO a runtime non vede. Trovato
   indipendentemente anche da Stogov (team-analisi: B6/B7).
5. **Stogov — `namespace X; extract($a)` sfugge alla rinuncia**, provato a
   macchina: `observes_scope` è indicizzata sul nome SCRITTO, e il fallback di
   namespace lo aggira. `would_take_safe_str` è un TETTO, non un perimetro
   fedele.
6. **Leijen — `would_take_safe_str` conta `Rc::clone` elisi, ZERO allocazioni.**
   Non c'è nessuna lettura in chiave footprint da fare su quel numero: chi
   volesse convertirlo in byte risparmiati sbaglierebbe canale.
7. **Pedersen — P-AMEND-ORFANO.** L'identity registra `head=`, poi un `--amend`
   sostituisce l'oggetto: la citazione sopravvive, il referente no, e al primo
   `gc` la provenienza diventa **irrisolvibile**, non solo indimostrata. E
   l'amend che ha orfanizzato quel commit è proprio quello che portava nel
   giudice la cura di Klabnik: *la cura dell'uno è la causa dell'altro*.
8. **Hejlsberg — `LoadSlot{take}` è la peggiore delle due forme.** Il layout è
   neutro (`LoadSlot` usa 4 byte sullo slack di `Op`), quindi il flag non compra
   nulla, e tassa 60,6 M letture per servirne 9,99 M.

## Ordine WP-97 proposto (convergenza dei tre team)

1. **Ri-profilo R≥3, stesso workload, ZERO cambi di codice** — con
   mispredict-indiretti/op e L1I-miss/op normalizzati su `op-census`, peak
   registrato, e regola di lettura scritta PRIMA. È l'unica voce che serve i tre
   mandati insieme: rimette in moto il cronometro, ripara il denominatore da cui
   dipendono tutte le bande di due sessioni, e dice se O1 ha un canale prima di
   scriverla.
2. **O1 (outlining dei bracci freddi)** con controllo positivo DOPPIO (taglia
   predetta + outlineati ∩ `op-census`; L1I-miss/op giù, `op-census`
   invariante) e coppia stessa-sera.
3. **Braccio NULL cronometrato, nelle DUE forme** (dentro `run_loop` — Gregg; Δ
   di taglia equivalente FUORI — Bak): è ciò che rende decidibile il conto del
   passo 2.
4. **La forma dell'emissione** solo DOPO che l'entropia del bit è misurata: la
   sua motivazione scritta è oggi refutata (Bak: il bit è preso il 42,33% delle
   volte, quindi «per-sito quindi ben predetto» è falso).
5. **P-AMEND-ORFANO** (voce d'apparato ammessa): `refs/measure/<run>` piantata
   prima del run, oppure `head=` scritto DOPO l'ultimo amend.
6. **Footprint**: nessuna leva nominata; il falsificatore T_max di Leijen in
   timebox, e se non è misurato entro la sessione la leva arene si dichiara
   CHIUSA.

## Conflitti REGISTRATI (non appianati)

- **Conoscenza o rinuncia?** Gregg: rinuncia · Bak: né l'uno né l'altro,
  derivazione refutata ⇒ sospensione · Leijen: disciplina.
- **Segno del passo 2**: Klabnik NON OMOGENEO · Pedersen NULLO (F3 sospesa, non
  archiviata) · Hejlsberg REGGE nel suo perimetro · Stogov regge ma su un tetto.
- **`LoadSlot{take}`**: Klabnik «mai valutata, e §5.1 ammette che cambierebbe il
  verdetto» · Hejlsberg «valutata e refutata». Il team catena registra:
  *Klabnik ha ragione sul GRADO (argomento, non misura), Hejlsberg sulla
  DIREZIONE*.
- **Prima voce**: Gregg O1 · Bak il denominatore prima di O1 · Leijen T_max.
- **`would_take_safe_ref`**: Matsakis lo dichiara refutato da una sola firma
  (una sola sede di conteggio non copre il canale cross-frame).
