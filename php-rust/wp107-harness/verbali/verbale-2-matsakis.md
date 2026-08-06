# Verbale sedia 2 — MATSAKIS (ownership/aliasing/borrow-discipline) — Concilio WP-107 su S-105

## VERDETTO: APPROVO CON EMENDAMENTI (nessuna refutazione capitale)

Sul piano ownership la forma 2 è pulita per COSTRUZIONE: `frame` è un locale
posseduto, i buffer arrivano da `frame_pool.take()` (unico sito) e vi
rientrano SOLO per move di un frame morente (`recycle_frame`, mod.rs:4631 —
unico `put`); `with_buffers` fa `resize(n_slots+max_temps, Undef)`
(mod.rs:2707), quindi `frame.slots[i]` è in-bounds e il pop dalla pila del
chiamante non può aliasare gli slot del callee: il borrow-checker lo
esclude, niente unsafe nel sentiero. Il fast path non deposita mai `Ref`
negli slot (decay clona il referente): D-R11 preservato. Il canale (a) del
mandato è CHIUSO in favore della leva.

## R-MA-107-n (refutazioni)

- **R-MA-107-1 — «decay_arg è pure-read» è asserito, non provato.** Il
  commento (run.rs:2599-2600) certifica sé stesso. Il braccio `Ref` di
  `decay_arg` (calls.rs:299) clona E POI DROPPA il Ref poppato — un
  decremento Rc. La forma 2 decade in ordine `(0..n).rev()` (run.rs:2605);
  il vecchio braccio fast di bind_params decadeva in ORDINE SORGENTE
  (calls.rs:326). L'ordine dei decrementi Rc è esattamente ciò che
  recycle_frame chiama «the only PHP-observable effect» (riuso handle-id,
  cascate GC). Serve o la prova per NOME (un operando di Op::Call non può
  essere un Ref last-ref) o un dente (due temp Ref/oggetto, ordine di riuso
  handle-id osservato). Finché manca, l'equivalenza d'ordine è ipotesi.
- **R-MA-107-2 — La copertura ArgPlace di G3 è vacua per costruzione, non
  per collaudo.** `PushArgPlace` è emesso SOLO da `push_dyn_args`
  (expr.rs:1872), mai prima di `Op::Call` (che usa push_value_args/
  push_call_args; quest'ultimo route i place by-ref via MakeRef→Ref). Il
  fast path oggi NON può incontrare ArgPlace — ma fx21 non lo testa né può
  testarlo, e il backstop (calls.rs:303, ArgPlace→Null) è FAIL-SILENT: un
  funnel mancato degrada un argomento a Null senza rumore. Il criterio
  11cc23a nominava CallValue/CallNsFallback nello scope della leva: LÌ
  ArgPlace È raggiungibile — estendere il direct-bind senza cambiare classe
  di rischio sarebbe un errore già scritto.

## A-MA-107-n (emendamenti)

- **A-MA-107-1**: dente d'ordine per R-MA-107-1 (fixture con due argomenti
  che droppano last-ref nella finestra di chiamata, arbitro = ordine di
  riuso handle/cascata) OPPURE nota nel commento che declassa «pure-read» a
  «ordine non provato osservabile, nessun controesempio noto».
- **A-MA-107-2**: braccio ArgPlace di decay_arg reso RUMOROSO nei build di
  collaudo (`debug_assert!` o contatore census «funnel mancato»); nel
  commento del fast path: estensione a CallValue/CallNsFallback ⇒ check di
  materializzazione obbligatorio PRIMA del bind.
- **A-MA-107-3 (punto c)**: il RINVIO è LEGITTIMO (la regola niente-build-
  con-coppia-in-volo protegge l'identità del pin misurato; le mutazioni
  richiedono build). MA 19a/19b sono di nuovo citate «PASS» nel gate
  PIN-106 mentre RC-MA-104 è aperto: quel verde vale SOLO byte-parity, non
  arbitrato. Rientro S-106: PRIMO atto dopo la lettura della coppia —
  ordine: (1) terza mutazione OBS-8 (mod.rs:4965, −2→−1: DECIDE), (2)
  mutante leak-parziale su fx20 (la banda a due soglie è nata S-105 ma la
  sua sensibilità al parziale è NON testata), (3) tabella di disposizione
  A-MA-106-2 (intonsa da due sessioni: da eseguire o degradare per NOME).
- **A-MA-107-4 (punto d)**: indiziato §3.15 CONFERMATO nel codice:
  `push_call_args` expr.rs:1615 `by_ref.get(i).copied().unwrap_or(false)`
  con maschera lunga `params.len()` — per `&...$rs` le posizioni oltre
  vslot cadono a value. Il binder dinamico fa già la cosa giusta
  (mod.rs:11582-84 estende la maschera variadica): la cura è compiler-side,
  specchiare quella logica in push_call_args (e nel ramo named
  expr.rs:1417). Fix localizzato, fuori finestra leva: coda fedeltà.

## KS-MA-107-n (vincolanti)

- **KS-MA-107-1**: un claim d'ordine-di-effetti in un commento di fast path
  vale solo con prova o dente citato; senza, è VOID come premessa di leve
  future.
- **KS-MA-107-2**: nessun direct-bind su op dinamiche (CallValue/
  CallNsFallback) senza check ArgPlace nel fast path e backstop rumoroso —
  il Null silenzioso non è un'admission.
