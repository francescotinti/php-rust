# team-engine — Concilio WP-92 (relatore: sedia 8 Stogov, team monocratico)

FONTE VINCOLANTE: `verbali/verbale-8-stogov.md` (VERDETTO: CON EMENDAMENTI,
A-DS53…A-DS57, KS-DS-92-1/2/3). Vincoli di merge integrati dalle sezioni
Emendamenti/Kill-switch di `verbale-3-klabnik.md` e `verbale-9-gregg.md`
limitatamente a dove pesano sull'implementazione A-DS51.

---

## 1. SINTESI DEI VINCOLI — 10 vincoli che l'implementazione LSP deve rispettare

**V1 — contratto by-ref EMENDATO (A-DS48, invariato).** L'esattezza è sulla
REF-NESS, non sul tipo: togliere *o* aggiungere `&` = fatal in entrambe le
direzioni; il TIPO resta contravariante, quindi il widening `int &$x` →
`int|string &$x` è **alive** (v1) e la narrowing è fatal (v2). Il checker non
deve trattare la by-ref come "identità di firma". Le union nei messaggi vanno
nell'ordine canonico Zend (`P::m(string|int &$x)`), non nell'ordine sorgente.

**V2 — esenzioni ctor SOLO ereditarie.** Il costruttore è esente dal check LSP
quando la sede è una **classe plain**; NON è esente quando il genitore è
un'**interfaccia** (v13) o una **classe abstract** (v14). Un'esenzione
`is_constructor ⇒ skip` piatta fa fallire due pin per NOME. Insieme al ctor
vanno nello stesso commit le altre esenzioni: `private`, RTWC (return-type
will change), tentative-Deprecated (A-DS46).

**V3 — timing t1-t4, con t3 a exit 0 come NEGATIVO obbligatorio.**
Dai raw `ds35-verify2.out`: t1 (parent dichiarato DOPO il figlio) fatala
comunque ⇒ il check è sul link hoisted, non sull'ordine testuale; t2
(condizionale eseguita) fatala **dopo** `pre|`; **t3 (condizionale non
eseguita) esce 0 con `pre|post` su ENTRAMBI i bracci** ⇒ una fase che fatala t3
è un falso positivo che rompe codice legale (KS-DS-92-3); t4 discrimina la
forma del lowering (vedi V4).

**V4 — bersaglio t4 = braccio PERSIST, dichiarato per NOME (A-DS55).**
Refutazione capitale: «stdout integrale dell'oracle» **non nomina il braccio**,
e su t4 i due bracci DIVERGONO ai byte —
`plain` emette `pre|\n` prima del fatal, `persist` fatala **pre-output**
(nessun `pre|`). phpr con lowering hoisted fatalerà pre-output ⇒ il bersaglio
byte-fedele è il braccio **persist**, e il pin deve scriverlo per nome.
Un pin che dice solo "stdout dell'oracle" è ambiguo per costruzione su t4.

**V5 — v15 mai skip silenzioso (A-DS56).** Contraddizione ereditata sciolta:
l'appendice WP-89 («skip conservativo su nomi non risolvibili») **cede** a
§3.3-quinquies («mai skip silenzioso»); prevale la delibera WP-91. v15
(`m(): B` vs `m(): A`, classi inesistenti) **fatala** — messaggio di famiglia
propria: `Could not check compatibility between C::m(): B and P::m(): A,
because class B is not available`, exit 255 su entrambi i bracci. La fatal è al
**bind hoisted**; per i bind dinamici il giudice è il gate ORM/hk, non una
decisione a tavolino.

**V6 — pin v3 a byte-count per canale; comparatore a marcatori BANDITO
(A-DS54 / KS-DS-92-1).** Refutazione capitale: `echo "x\n--stderr\ny--end"` è
PHP legale e il suo **stdout contiene i marcatori** ⇒ un comparatore a scansione
di marcatori è forgiabile. Il buco è già visibile nei raw v2: le righe
`alive--stderr` e `pre|post--stderr` (stdout senza newline finale incollato al
marcatore) sono decodificabili solo *conoscendo* i marcatori. Formato v3:
`--stdout bytes=N` / `--stderr bytes=N` per canale, parse = lettura di **N byte
esatti**. Nessuna euristica di riga.

**V7 — pin a CANALI SEPARATI (A-DS50, confermato).** stdout FULL, stderr FULL,
exit code, **per braccio**; mai `2>&1`, mai troncamento. Divergenza a catalogo
confermata: phpr **non** modella la log-copy stderr dell'oracle
(`PHP Fatal error:  …`, doppio spazio) ⇒ **stderr phpr atteso VUOTO**. Il
comparatore confronta stdout-vs-stdout e exit-vs-exit, e asserisce
`stderr_phpr bytes=0`; non confronta lo stderr dell'oracle con nulla.

**V8 — sette buchi di copertura da colmare PRIMA del merge (A-DS53, fixture
v3).** Tutti oracle-morsi dal vivo su 8.5.7: (1) interfacce multiple in
conflitto; (2) interface-extends-interface incompatibile (fatal **senza alcuna
classe**: il check vive anche nel linking di interfacce); (3) enum implements
(gli enum sono assenti dall'intero set); (4) `self` vs `static` — vincolo di
modellistica su `TypeHint::display_name` (`crates/php-runtime/src/hir.rs:603`):
`self` si stampa come **nome-classe risolto**, non «self»; (5) property hooks —
famiglia di messaggio DIVERSA: `Type of C::$x must be subtype of int (as in
class P)`, «subtype of», non «must be int»; (6) costanti final
(`C::X cannot override final constant P::X`); (7) readonly PROMOTED (stesso
messaggio di v11, path di lowering diverso). Più il **positivo di regressione**
DNF `(A&B)|string → string` (legale, verificato). KS-DS-92-2: merge di A-DS51
senza le fixture A-DS53 **per NOME** nel gate = REJECT.

**V9 — vincoli di evidenza da Klabnik (dove toccano A-DS51).** A-SK-67: il
giudice, il manifest e il budget si leggono da **HEAD**, e un working tree ≠
HEAD ⇒ FAIL (KS-SK-92-1: PASS con blob ≠ HEAD ⇒ VOID) — quindi `ds35-verify3.sh`,
le fixture v3 e il comparatore byte-count vanno **committati prima** della run
che li consuma, non generati durante. A-SK-71: il perimetro del manifest è
**ogni .md committato che pubblica cifre di sessione** — il conteggio fixture
(37→44+) e i numeri di gate che finiranno in `NEXT_SESSION`/`sessions/`/design
richiedono riga di manifest (KS-SK-92-3: cifra fuori perimetro ⇒ non
verdict-grade anche se il MEASURE passa). A-SK-69: se si cita una derivata
(es. «N/M fixture verdi»), operandi dallo **stesso file** e operatore
verificato.

**V10 — vincoli di evidenza da Gregg (dove toccano A-DS51).** A-BG58:
`reason=` autosufficiente — ogni riga verdict/supersede sul canale LSP deve
portare `reason=requalify:<blocco>:<old→new>` e il ledger da solo deve
rigenerare la storia da fase a fase (KS-BG-92-2: reason che non ricostruisce la
riqualifica senza aprire file esterni ⇒ supersede invalido alla campagna
successiva). A-BG59: mai ereditare un campo da una riga precedente — ogni riga
di esito fixture porta i propri campi o è FAIL.

---

## 2. PIANO S-91.0 — TRE FASI ORDINATE (A-DS57, ordine vincolante)

L'ordine è scelto per **rischio ORM/hk minimo**: l'ordine inverso
(registry prima) lascerebbe scoperti 35/37 pin nella finestra intermedia.

### FASE 0 (pre-requisito, nessun runtime) — fixture v3 + pin v3, committati
- Scrivere le **8 fixture A-DS53** (7 buchi + positivo DNF) in
  `wp91-harness/fixtures-ds35-v3/`.
- Riscrivere il pin in formato **byte-count per canale** (V6) e ri-generare
  l'oracle-pin sull'intero set **v1+v2+v3** (18+19+8 = 45 fixture), con il
  bersaglio t4 dichiarato `braccio=persist` per NOME (V4).
- **Committare fixture + script + pin PRIMA** di qualunque run che li consumi
  (V9/A-SK-67).
- *Passa se*: 45/45 pin oracle riprodotti a byte-count; il comparatore
  auto-morde su un caso forgiato (`echo "x\n--stderr\ny--end"`) e lo **rifiuta**
  — se non morde, il comparatore non è ancora un giudice.
- *Gate*: nessuno (nessuna riga di runtime cambia).

### FASE 1 — checker LSP PURO + unit sui messaggi (nessun wiring)
- Modulo checker isolato: dato (firma parent, firma child, contesto sede),
  restituisce `Ok` o l'**esatta stringa di messaggio** Zend.
- Copre: return covariante/contravariante, param contravariante, union in
  ordine canonico Zend, by-ref per V1, `self`/`static` risolti via
  `TypeHint::display_name` (V8/4), nullable, variadic, mixed, void,
  readonly, prop-typed-on-untyped, hook-subtype (famiglia di messaggio
  distinta), final-const, non-risolvibile (V5).
- **Nessuna chiamata dal lowering**: il checker esiste ma non è armato.
- *Fixture che devono passare*: tutte le 45 a livello di **unit sul messaggio**
  (`cargo test --release`), confrontate contro le stringhe estratte dai pin —
  non contro stringhe ribattute a mano.
- *Gate che gira*: `cargo test --release`. Nessun gate di merge pesante: la
  superficie di runtime è invariata (nessun handler nuovo nel run_loop).
- *Controllo positivo obbligatorio*: un contatore tutto-verde al primo giro è un
  controllo positivo FALLITO — almeno una fixture deve rossare prima del fix.

### FASE 2 — wiring nel lowering HOISTED (+ esenzioni nello STESSO commit)
- Sede: `crates/php-runtime/src/lower/class.rs`, dopo il **flatten dei trait**,
  **accanto al final-check** (`Cannot override final method`, ~riga 929,
  vicino a `final_ancestor_method` / `check_readonly_extends`).
- **Nello stesso commit** le esenzioni (V2): ctor-plain-class, private, RTWC,
  tentative-Deprecated (A-DS46) — con la nota che il ctor **non** è esente per
  interfacce/abstract (v13/v14). Una fase 2 che sposta le esenzioni a un commit
  successivo apre una finestra in cui ORM/hk fatalano su codice legale.
- **Le condizionali restano FUORI** da questa fase: t2 non fatala ancora, ed è
  atteso; **t3 = REJECT se fatala** (KS-DS-92-3).
- *Fixture che devono passare*: le 45 meno t2 (che resta al comportamento
  pre-fase-3), con **t1 fatal**, **t3 exit 0 `pre|post`**, **t4 byte-fedele al
  braccio persist** (nessun `pre|`, KS-DS-92-3 esplicita: fase 2 che manca il
  persist-shape di t4 = REJECT), **v15 fatal** con il messaggio
  `Could not check compatibility …`.
- *Gate che gira*: **gate ORM 3E/13F + hk + corpus Zend per NOME (1418) + refl
  290** — per NOME, mai per conteggio.

### FASE 3 — bind-registry per condizionali e dinamiche
- Registry dei bind non-hoisted: quando la dichiarazione condizionale viene
  **eseguita**, il check scatta al bind (t2), mantenendo `pre|` già emesso.
- *Fixture che devono passare*: **tutte e 45**, t2 inclusa — t2 stdout
  `pre|\n` + fatal, exit 255; t3 **ancora** exit 0 `pre|post` (il registry non
  deve armare i rami non presi).
- *Gate che gira*: **di nuovo ORM 3E/13F + hk + corpus per NOME + refl 290**,
  in particolare per i bind **dinamici**, che per V5/A-DS56 sono giudicati dal
  gate e non da una decisione a tavolino.

---

## 3. RISCHI SUI GATE DI MERGE + MOSSA DI MITIGAZIONE

| # | Rischio | Perché è reale | Mitigazione |
|---|---|---|---|
| R1 | **ORM (baseline 3484, 3E/13F) regredisce in fase 2** — Doctrine genera proxy e usa reflection pesante su gerarchie con ctor promossi/interfacce | il check LSP è **nuovo codice che può solo AGGIUNGERE fatal**: ogni falso positivo è un E in più | esenzioni nello STESSO commit (V2); gate ORM **per NOME** subito dopo fase 2, prima di toccare la fase 3; se compare un E nuovo, la mossa è **restringere l'esenzione mancante**, non disarmare il checker (direttiva NIENTE REVERT) |
| R2 | **hk (~1665) regredisce sui bind dinamici** — Symfony http-kernel istanzia via container/factory | fase 3 arma il check su percorsi che fase 2 non tocca | fase 3 **isolata in un commit proprio** con gate hk immediatamente dopo; il gate hk è il giudice dichiarato dei dinamici (V5), quindi il suo esito va **ledgerato** anche se PASS |
| R3 | **corpus Zend 1418 / refl 290 scendono e il conteggio resta uguale** (set diverso, cardinalità identica) | è il forge classico che la regola gate-per-NOME esiste per chiudere | diff del **set di nomi**, mai del conteggio; `--list-fails` con confronto insiemistico contro il baseline `phpr-wp89` (64e9e51c281de6d1) |
| R4 | **Baseline hk incerto (1663 in memoria vs 1665 citato)** | un baseline sbagliato produce sia falsi verdi sia falsi rossi | **ri-pinnare il baseline dal run verde più recente PRIMA di fase 2**, committarlo, e citarlo per sha; mai dal ricordo |
| R5 | **Il pin di fase 0 viene rigenerato durante la run che lo consuma** | KS-SK-92-1: PASS con blob ≠ HEAD ⇒ VOID — l'intera fase 2/3 diventerebbe non-verdict-grade | fixture + script + pin **committati prima**; il comparatore stampa `judge_sha` e verifica working == HEAD, altrimenti FAIL (V9) |
| R6 | **Il comparatore byte-count viene consegnato senza auto-morso** | un giudice che non ha mai rifiutato nulla non è un giudice (lezione ricorrente: un gate che MORDE vale più di dieci che benedicono) | test negativo committato nello stesso commit: la fixture forgiata `echo "x\n--stderr\ny--end"` deve essere **rifiutata** dal comparatore v3 e **accettata** (falso verde) da quello a marcatori, a prova che la sostituzione era necessaria |
| R7 | **Le cifre di copertura LSP finiscono in NEXT_SESSION/sessions senza riga di manifest** | KS-SK-92-3: cifra fuori perimetro ⇒ non verdict-grade anche col MEASURE verde | riga di manifest per ogni .md che pubblica il conteggio fixture o gli esiti gate; derivate con operandi dallo **stesso file** (A-SK-69) |
| R8 | **La storia fase1→fase2→fase3 non è ricostruibile dal solo ledger** | KS-BG-92-2: supersede invalido alla campagna successiva | `reason=requalify:<blocco>:<old→new>` su ogni riga verdict/supersede; nessun campo ereditato da riga precedente (A-BG59) |

**Rischio principale** (uno solo, se se ne deve nominare uno): la fase 2 arma
un check che può **solo aggiungere fatal** su tre gate di merge grandi, e ogni
esenzione mancante si manifesta come regressione ORM/hk — per questo le
esenzioni viaggiano nello stesso commit del wiring e il gate gira **dopo ogni
fase**, mai una volta sola alla fine.

---

## Riferimenti verificati (raw, non ricordi)

- `wp90-harness/ds35-verify2.out` — t4 plain con `pre|`, t4 persist pre-output
  (righe ~867-895); t3 exit 0 `pre|post` entrambi i bracci (~852-866); v15
  fatal 255 entrambi i bracci (~765-790); `alive--stderr` / `pre|post--stderr`
  = prova viva dell'ambiguità del comparatore a marcatori.
- `wp90-harness/fixtures-ds35-v2/` — 19 fixture v2 (t1-t4, v1-v15).
- `crates/php-runtime/src/lower/class.rs` — sede del wiring (final-check
  ~929, `final_ancestor_method` 609, `check_readonly_extends` 627, flatten
  trait 202).
- `crates/php-runtime/src/hir.rs:603` — `TypeHint::display_name`, punto di
  intervento per `self`/`static` risolti (V8.4).
- Assenza confermata: nessun «must be compatible with» per metodi nel codice
  attuale (solo hook `get()`), cioè il checker **non esiste ancora**.
