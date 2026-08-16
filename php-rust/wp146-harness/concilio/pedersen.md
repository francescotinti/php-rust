# PEDERSEN — bozza indipendente S-146 (lente: confini per-richiesta, lifecycle, ordine __destruct, output-capture, RetainSet, §3.22)

**VERDETTO: CONCORDO CON EMENDAMENTI** sul quesito. Binding output-capture INTATTO e non emendabile: qualunque variante di B3 che differisca una morte oltre il punto refcount-driven è fuori dal tavolo per costruzione.

## Posizioni a–e

**a) CON EMENDAMENTI.** Il flag `take` deciso a compilazione dentro `LoadSlot` è l'unica forma sotto il tetto WP-39..44 — ma la forma è ORTOGONALE alla semantica: non compra un grammo di fedeltà. Nessuna emissione senza R1–R3 sotto; disasm bl-count resta dovuto (lezione H-C2).

**b) CON EMENDAMENTI.** Il nucleo-stringhe **elimina** il rischio identità/lifecycle — le stringhe PHP non hanno `spl_object_id`, WeakReference, `__destruct`, risorse — **solo se** il guard è runtime-esatto: match `Zval::Str` puro, MAI `Ref` (slot condiviso: lo spostamento è osservabile altrove; safe_ref 0,013% rende il fallback economico, non superfluo — la correttezza non si misura in frequenza). Ciò che il perimetro NON elimina: (i) il rischio LIVENESS — uno slot vivo svuotato si rilegge `Undef` via canali dinamici e il fallimento è SILENZIOSO; la lezione S-96 («corretto per fortuna del corpus») vale per l'analisi, non per il valore; (ii) `memory_get_usage`/`debug_zval_refcount` osservano il momento del free — la seconda è già in rinuncia, la prima va a catalogo divergenze PRIMA, non dopo. Quindi: identità eliminata per costruzione del guard; morte-anticipata ridotta a valore senza effetti; residuo = soundness dell'analisi.

**c) CONCORDO: SÌ, obbligatorio.** would_take 47,1% è del media group WP; il giudice della scommessa è ORM. F1-ORM (sola misura, zero rischio) più tetto aritmetico pre-registrato: massimo teorico = mv_str 3,85 ns × 104,1M clone str ≈ 0,40 s; con frazione takeable tipo-WP ~0,2 s su 37,6 s di gap. Le tre bande di design95 §P1 vanno RI-derivate sui numeri ORM prima di decidere.

**d) CON EMENDAMENTI.** Borrow-first/through-borrow ai siti consumatori PRIMA di TakeSlot: non muove, non lascia Undef, non anticipa NESSUNA morte — zero rischio lifecycle per costruzione (precedenti HC1/L-FR1 spediti). «Arena-conteggi»: definire o archiviare; **pre-registro il veto**: qualunque definizione che porti la morte di un valore a un confine (sweep, fine-op, fine-request) invece che al DELREF→0 collide col binding output-capture e allarga la §3.22 — se non è per-valore con morte refcount-driven, è archiviata.

**e) CONCORDO col limite Matsakis, EMENDO col tetto**: B3-stringhe compra al più ~0,4 s del churn modellato 1,52 s; nessun claim su glue 4,4 s né sulla parità. Se il tetto F1-ORM non raggiunge la banda del giudice, KS-B1 è irraggiungibile per aritmetica: si dichiara PRIMA, non si scopre dopo.

## Emendamenti R1–R4
- **R1 (fixture bilaterali per NOME, PRIMA di ogni riga)**: `fx-destructor-order` (ordine di stampa a fine funzione) · `fx-generator-suspend` (locale letto via `get_defined_vars` dopo resume) · `fx-a-append-a` (`$a .= $a`) · `fx-compact-after-last-use` · `fx-weakref-slot` · in più dalla mia lente: `fx-ref-to-str` (`$r=&$s`) e `fx-resource-close-order` (chiusura file osservabile). Byte-id vs oracle; divergenza non sanabile ⇒ a catalogo, mai saltata.
- **R2 (sonda-verità, probe mai pinnabile)**: build che al take lascia SENTINELLA invece di Undef; ogni rilettura aborta col site-id. Corpus 1414×2 + ORM + batteria: conteggio riletture DEVE essere 0 — converte il fallimento silenzioso in rumoroso.
- **R3 (gate STOP allargato)**: fail NUOVO per NOME in `weakrefs/`, `destructor`, **`generators/`, `references/`** ⇒ STOP fetta. Il gate attuale è necessario, NON sufficiente (fortuna del corpus).
- **R4 (ordine d'istruttoria)**: 1· F1-ORM + tetto; 2· famiglia borrow-first (FR1-ext); 3· TakeSlot solo se tetto ≥ banda, con R1–R3 verdi; 4· arena-conteggi definire-o-archiviare sotto il veto in (d).

## Kill-switch pre-registrabili
**KS-P1** = R3 (STOP + revert). **KS-P2**: sentinelle R2 >0 dopo una riparazione ⇒ perimetro falsificato. **KS-P3**: tetto F1-ORM < banda giudice ⇒ TakeSlot NON si scrive. **KS-P4**: qualunque morte differita a confine ⇒ veto immediato, senza misura.

**Mandato inverso**: oggi sappiamo che il collo è il pavimento per-movimento (69,5%) e che il tetto di B3-stringhe è CALCOLABILE (prezzi firmati × conteggi ORM) — in S-96 non lo era.
