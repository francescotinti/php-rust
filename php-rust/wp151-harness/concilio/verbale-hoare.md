# Verbale HOARE — Concilio S-151 (fase 1, indipendente)
Lente: design linguaggio/runtime in Safe Rust — sigilli di TIPO che rendono
gli stati illegali irrappresentabili (precedente: VmGate ZST, vm/mod.rs:542).
Ancoraggio a codice: `Zval::Object(Rc<RefCell<Object>>)` (php-types/src/zval.rs:40),
weak refs `Weak<RefCell<Object>>` + `teardown_weaks` (vm/mod.rs:2872/2876/3274),
`spl_object_id` da object_handle (vm/host.rs:2180).

VERDETTO: **CONCORDO CON EMENDAMENTI** — l'impianto A1→A4 sta in piedi, ma
cade se A3 non è sigillato nella forma LINEARE (R1–R3), se A2 tratta run_loop
come un modulo qualsiasi (R4), e se il census non separa i canali per-TIPO (R5).

## Risposte Q1–Q5 (sintesi; dettaglio negli emendamenti)
- **Q1**: sequenza A1→A2→A3 giusta CON due correzioni. (a) Il census A1 si
  chiava su etichette semantiche STABILI (opcode/helper/canale), MAI file:riga:
  altrimenti A2, spostando i siti, invalida A1 e si ricomincia. (b) Interleaving
  MIRATO: le tranche A2 si ordinano sulla touch-map di A3 derivata dal census
  (prima memory/ e ops-oggetti; run_loop per ULTIMO o mai — R4). Chirurgia
  prima del refactor: REFUTATA (chirurgia dentro 25,7k righe = nessun unit
  test isolabile, rischio edit moltiplicato).
- **Q2**: gate per tranche = batteria + corpus ×2 per NOME + fixture + micro
  R=5 a SOLO-REGRESSIONE + **disasm run_loop (istr/bl) con delta DICHIARATO**;
  coppia WP dovuta SOLO al pin (ogni tranche promossa = pin nuovo ⇒ coppia,
  come da regola utente 2026-08-12 — se questo rende le 4–6 sessioni più care,
  si dichiara, non si dirada di nascosto). Sostituto della byte-identità:
  disasm-invariance + catena gate. Un refactor Rust NON è mai «zero rischio»:
  attraversare confini di modulo flippa l'inliner (precedente S-104, bl 1101→0).
- **Q3**: la forma «store + refcount esplicito senza RefCell + props inline» è
  giusta SOLO nella variante lineare (R1). Cosa si rompe se non presidiato:
  weak refs (oggi poggiano su `Weak<RefCell<Object>>`); timing/ordine
  __destruct RE-ENTRANTE (il dtor esegue PHP che può allocare/liberare nello
  store mentre lo si attraversa); GC cicli (possible-roots oggi legge i conti
  Rc); `spl_object_id` (PHP RIUSA gli id dopo la free: l'id esposto può
  riusare, la generazione resta interna); Resource/Closure/Ref restano Rc ⇒
  due regimi di conteggio coesistono al confine e vanno nominati; RetainSet
  persistente del server (R6). Numeri per la decidibilità: R5.
- **Q4**: dente in BATTERIA (il fatto fresco — CI a backlog ~3 giorni — rende
  un gate solo-CI cieco per costruzione); ratchet per-file su baseline
  committata; controllo positivo obbligatorio nell'atto di armamento (R7).
- **Q5** (Gregg, cosa manca): (i) l'ordine non nomina che A3 = **cambio di
  TIPO whole-program**: togliere/riscrivere `Clone` su Zval tocca ogni sito
  di clone (44% inline in run_loop, census S-140) — il conteggio dei SITI è
  un deliverable di A1, ed è la cifra di costo di A3; (ii) la persistenza
  cross-request (RetainSet, binding output-capture) non compare né in Gemini
  né nell'ordine: senza semantica di sopravvivenza dello store al confine
  richiesta, A3 è indecidibile per il server (R6). Cosa sappiamo oggi che
  ieri no: BT1 ha mostrato che un canale hostcall da 130M chiamate valeva
  6 s di ORM — il census per-canale È lo strumento giusto; il piano deve solo
  non romperlo col refactor.

## Emendamenti
- **R1 — Sigillo del refcount (forma lineare, unica che sigilla)**:
  `ObjSlotId` con campo PRIVATO in `memory/object_store.rs` (nessun
  costruttore pubblico: il mint è solo dello store — schema VmGate);
  `ObjHandle(ObjSlotId)` **NON Copy, NON Clone**; duplicazione SOLO
  `store.incref(&h) -> ObjHandle`; dismissione SOLO `Vm::release(self, h)`
  per VALORE con `&mut Vm` — il dtor PHP gira LÌ, sincrono (timing osservabile
  preservato; il borrow checker impone staticamente la disciplina che oggi
  RefCell fa pagare o panicare a runtime); drop-bomb in debug (`Drop` con
  debug_assert su flag defused) = linearità emulata: l'handle perso morde in
  batteria, non tace. Effetto-sigillo: `derive(Clone)` su Zval NON COMPILA
  più ⇒ ogni sito esistente o nuovo che voglia duplicare un oggetto è
  COSTRETTO dal compilatore a passare dallo store. La variante «refcount in
  Cell + Clone automatico» è REFUTATA: reinventa Rc senza RefCell, conserva
  il churn clone/drop (32%) e non sigilla nulla.
- **R2 — Anti-pendente**: indice GENERAZIONALE (idx u32 + gen u32, check al
  deref, fail-fast). Brand lifetime (GhostCell/branded index) REFUTATO come
  meccanismo primario: previene il mixing TRA store (già garantito dallo
  store unico dentro Vm), NON la stalità TEMPORALE (riuso slot dopo dtor
  re-entrante), ed è infettivo whole-program. Nota di tipo: senza check di
  generazione uno stale id in Safe Rust non è UB ma è **oggetto sbagliato**
  (parity bug silenzioso) ⇒ il check resta ON in release (KS-H3).
- **R3 — Divieto di doppia rappresentazione**: mai una build in cui
  `Zval::Object(Rc<RefCell<_>>)` e `Zval::Object(handle)` coesistono
  (identità, `==`, spl_object_id, weak diventano ambigui). A3 = flip atomico
  della variante con migrazione compile-driven; il phase-in per famiglia di
  opcode è refutato per un cambio di rappresentazione dei DATI.
- **R4 — run_loop non è un modulo qualsiasi**: il monolite del dispatch può
  essere load-bearing per icache/inlining. La tranche `exec/` di Gemini non è
  «rischio zero»: run.rs si spacca per ULTIMO col disasm prima/dopo come gate
  dichiarato, o si lascia intero se il disasm muove.
- **R5 — Census che rende A3 DECIDIBILE**: canali separati PER-TIPO:
  (i) borrow/borrow_mut su Object (conteggio × prezzo unitario bilaterale);
  (ii) incref/decref su OGGETTI, distinti da Str/Array (se il churn 32% è
  dominato da Str/Array, ObjectsStore non lo tocca); (iii) seconda-alloc
  props per costruzione; (iv) MOVIMENTO handle SEPARATO dall'accesso. La
  tensione a registro (tetto movimenti 1,27 s ≈ 3,4% vs «azzerare il
  movimento ripaga» di Gemini) si scioglie così: il movimento è già a TETTO
  e va ESCLUSO dalla somma pro-A3 — A3 si motiva su borrow+refcount
  all'accesso+props, o non si motiva. Soglia X in secondi sul binario census
  ORM pre-registrata PRIMA di leggere i numeri.
- **R6 — RetainSet/server**: store per-VM, NON bump-reset a fine richiesta
  (l'arena «reset del puntatore» di Gemini §2.2-2 collide col RetainSet
  persistente e col binding output-capture-before-reset, che resta sovrano);
  sweep di fine richiesta con promozione ESPLICITA dei sopravvissuti; il
  census A1 conta gli oggetti che attraversano il confine richiesta.
- **R7 — Dente A4**: sede = batteria (cargo test); soglia 2.000 per file
  NUOVI + ratchet DECRESCENTE sui monoliti (baseline per-file committata:
  mai crescere); controllo POSITIVO nell'atto (violazione sintetica che DEVE
  fallire — lezioni auto-morso bea7ea3 e forge-silent-failure); pre-commit
  hook come cintura facoltativa, mai sede unica; CI = specchio, non gate.
- **R8 — Fase 5 Gemini (backend registri in crate parallelo)**: fuori
  registro e fuori ritmo — il veto WP-44 sta (Operand a match runtime,
  +1,28%); un crate «sperimentale parallelo» viola REGOLE §1 (≥70% token
  sull'oggetto). Non entra nell'ordine S-151..S-15x.

## Kill-switch pre-registrabili
- **KS-H1**: census A1: somma canali Object-specifici (R5 i–iii) < soglia X
  dichiarata ⇒ A3 REFUTATA prima di ogni riga di codice; restano A2 e le
  leve per-canale.
- **KS-H2**: tranche A2 con delta disasm run_loop oltre il dichiarato O
  guardia micro rossa ⇒ revert al byte della tranche + ripartizione; due
  tranche consecutive rosse ⇒ stop A2, concilio.
- **KS-H3**: check generazionale al deref oltre soglia su m-prop/objmap in
  A/B ⇒ NON si rimuove alla cieca: resta ON salvo fixture dtor-reentrancy
  dedicate verdi + corpus ×2; la rimozione è una leva a sé con criterio.
- **KS-H4**: qualunque piano di migrazione A3 che richieda la doppia
  rappresentazione in una build ⇒ stop e ridisegno (R3).
- **KS-H5**: dente A4 armato senza controllo positivo passato ⇒ dente NON
  armato, incidente di processo contato.

Firma: sedia HOARE, fase 1 (nessun verbale altrui letto).
