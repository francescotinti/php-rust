# Verbale sedia 2 — Matsakis (ownership/aliasing/borrow) — Concilio WP-104

**Oggetto**: S-102 (audit INV-RECV-1, fixture MOVE 14-18, is_gc_container) +
programma provvisorio S-103. **Mandato**: refutare.

## VERDETTO: CON EMENDAMENTI

## 1. L'audit INV-RECV-1 soddisfa A-MA-103-1? — SÌ nella struttura

L'argomento è quello giusto: non «il valore assoluto non si vede» (falso,
corretto dall'ADDENDUM in hc1b-criterio.out, appeso senza riscrivere la
storia — bene), ma «l'handle mosso è un holder ESTERNO per ogni soglia».
Questo regge *indipendentemente* dalla presenza dello slot (anche ricevitore
temporaneo: soglia k = holders propri dell'osservatore, il braccio aggiunge
+1 in entrambi gli schemi). I punti d'appoggio (i)-(iv) sono verificati sul
codice: il MOVE in run.rs ~3481 (PropGet) e ~3653 (PropSet) esclude `Ref`
per costruzione; i commenti citano audit e KS-MA-103-2. La tavola dei 12 è
onesta sui FUORI PERIMETRO (#3/#10/#12 osservano cella o old, non l'handle).

## 2. Refutazione capitale RC-MA-104-1 — fixture 17-18: finestra giusta, REGIME sbagliato

Le fixture esercitano DAVVERO la re-entrancy sincrona intra-arm (init lazy
e dtor girano PHP dentro il braccio: verificato che `$o->x` e
`$h->slot = …` passano dai bracci col MOVE). MA in entrambe il ricevitore
resta tenuto dal suo slot variabile (`$o`, `$h`) per tutta la finestra:
ogni osservatore exact-count ha slack ≥2 dalla soglia, quindi pre-move e
post-move sono INDISTINGUIBILI per costruzione. **Una fixture che esercita
la finestra a distanza ≥2 dalla soglia non arbitra il −1**: un MOVE bacato
(0 handle) verrebbe MASCHERATO dallo slot. Fino a fixture nuova, la guardia
sul regime minimo è il SOLO audit statico. Il commento in testa alla 17
(«la finestra −1 è QUESTA») sopravvaluta ciò che la fixture prova.

## 3. Emendamenti

- **A-MA-104-1**: fixture 19 «ricevitore-ultima-ref mid-arm»: il dtor/init
  fa `unset` dell'ULTIMA ref esterna del RICEVITORE stesso e poi
  `gc_collect_cycles()` — l'handle mosso resta l'unico holder esterno
  (regime dove #1/#6/#7/#8 siedono a soglia±1). Attese PRIMA, oracle, 2 modi.
- **A-MA-104-2**: estendere l'audit alle API uniqueness-gated:
  `Rc::get_mut` / `try_unwrap` / `make_mut` (+ decisioni su `weak_count`).
  La tavola copre solo LETTORI di strong_count; un sito get_mut al −1
  FLIPPEREBBE da fallimento a successo autorizzando mutazione in-place —
  peggio di una lettura. Se occorrenze = 0, registrare lo ZERO per nome.
- **A-MA-104-3**: KS-MA-103-2/103-3 vanno RIPORTATI in
  NEXT_SESSION (regola fonte-unica): un kill-switch che vive solo in un
  file di harness è invisibile al prompt di sessione.
- **A-MA-104-4** (ring-fence H-C2): il predicato del drop fast-out DEVE
  essere `Zval::is_gc_container` (l'esaustivo a due livelli di S-102), mai
  un `matches!` ad-hoc — una variante nuova deve fallire CHIUSA in
  compilazione, non entrare nel fast-out.

## 4. Kill-switch

- **KS-MA-104-1**: un fast-out H-C2 che salti un drop su valore con
  `is_gc_container == true`, o che bypassi una `gc_note` che lo slow path
  emette = reject senza appello.
- **KS-MA-104-2**: fixture 19 verde ×2 modi PRIMA di qualunque estensione
  della famiglia MOVE (si compone con KS-MA-103-2, che resta ATTIVO e
  correttamente citato nel codice) e prima dell'apertura H-C1c.

## 5. Programma S-103 rispetto ai ring-fence

H-C2 ammessa (canale contato, banda pre-registrata, A/B da sola) CON
A-MA-104-4/KS-MA-104-1. Slot-diretti correttamente gated (leva-nulla +
dump-diff; «borrow nudo dello slot» resta in NON-riproporre). Nessuna
estensione MOVE in programma: KS-MA-103-2 non violato. Il collaudo
php-server come primo atto non tocca il mio perimetro.

**Refutazioni capitali**: 1 (RC-MA-104-1, §2).
