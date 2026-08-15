# Verbale sedia Bak (lente: VM di produzione — V8/HotSpot) · S-143

VERDETTO: CONCORDO CON EMENDAMENTI
DELIBERA: ISTRUTTORIA-PRIMA — una sessione per §7.1+§7.2 con regola di decisione PRE-REGISTRATA che converte i numeri in B-poi-A (lean attuale) o A-poi-B; nessuna scommessa pluri-sessione su quote non censite.

## §Analisi
Da una VM di produzione: nessun team V8/HotSpot spedisce un cambio di rappresentazione senza allocation-site profiling per classe. Il dossier è onesto (§7.1): la quota OGGETTI dei 471M pair è IGNOTA. L'acquisto di A oggi non è un numero — in workload ORM-hydration l'esperienza dice che stringhe/array/Vec-args spesso dominano gli alloc, non gli oggetti. B invece attacca il PREZZO UNITARIO di ogni movimento in ogni sito (memops 5,4 + churn 4,4 + parte del drop-glue in vm_inline): è l'unica opzione che risponde per costruzione alla lezione delle 4 falsificazioni («il singolo sito non muove la suite»). A attacca UNA famiglia con quota ignota e AGGIUNGE un load per accesso (handle-deref): V8 lo paga su Local<> e lo ammortizza con HandleScope on-stack + pointer compression + IC; phpr ha PropIc ma propget 29,9M + recv_clone 14,8M sono il moltiplicatore della nuova tassa — va prezzato PRIMA (veto alloc-removal). Nota: l'acquisto GC di A sulla nota è piccolo (obj 56,5M × 2–5 ns = 0,1–0,3 s); il grosso gc è sweep (59,2M siti), che A ridisegna ma non azzera. Dubbi sui prezzi §3: il pair 8–15 ns è plausibile ma la riconciliazione regge solo includendo i memcpy; realloc 19,5M (9,6 GB mossi) è Vec-growth che A NON tocca e B tocca solo via taglia. Ordine: se A mette un handle u32 nello Zval, RISCRIVE il layout — fare B poi A significa pagare due volte la migrazione; anche per questo la sequenza si decide sui numeri del census, non a gusto.

## §Emendamenti
- **R1**: census CH_* per classe (oggetti/array/str/Vec-args sui 471M) + tabella accessi (propget/recv_clone) = moltiplicatore handle-tax. Regola pre-registrata: oggetti ≥40% dei pair ⇒ A-poi-B; <25% ⇒ B-poi-A; in mezzo ⇒ riconvoca.
- **R2**: profilo per famiglia lato ORACLE (feedback-one-sided-profile): i canali che anche Zend paga (memops, map) vanno scontati o si sopravvaluta l'acquisto di entrambe.
- **R3**: per A, modello del costo SOSTITUTIVO obbligatorio e pre-registrato: handle-deref × accessi, sweep arena per-request, crescita tabella, RetainSet/output-capture.
- **R4** (§7.4): sonda monobinaria classe S-138 sul prezzo pair alloc/free prima di usare 3,8–7,1 s come budget.
- **R5**: «other» 26,6% riquantificata (S-141) prima di attribuire alla struttura guadagni residui — non è riserva di caccia.

## §Veti (Q3)
- NaN-boxing: CONFERMO. B = Option/niche by-value, non NaN-boxing; riaprirlo esige dossier proprio.
- Contenitori sul call path: CONFERMO. La tabella handle di A = slab indicizzata per-request, mai HashMap sul cammino caldo.
- Alloc-removal senza modello del costo sostitutivo: CONFERMO ed ELEVO — è esattamente il rischio di A (R3 vincolante).
- SSO inline: CONFERMO (stringhe fuori perimetro di A e B).
- Leva GC note-time (WP-21): CONFERMO — A non si vende come cura della nota (0,1–0,3 s); il suo acquisto gc è sulla forma dello sweep, da modellare.
- Notti su PhpStr-full: CONFERMO.

## §Kill-switch (Q4)
1. Istruttoria (1 sessione): census CH_* r1==r2 <1% + profilo oracle; se oggetti <25% E l'oracle paga quota memops comparabile ⇒ la scommessa torna in concilio.
2. Via deliberata (B o A): prototipo entro ≤3 sessioni; giudici = micro churn/objdatains (A/B ABAB, soglia REGOLE §3) E coppia suite ORM 2/lato net: Δ suite <3% ⇒ FALSIFICATA (la struttura promette la suite: il giudice che ha ucciso 4 micro-leve la giudica).
3. Solo A: corpus 1414 per NOME ×2 + fixture ===/weakref/__destruct (§3.22); violazione binding output-capture = reject immediato; handle-tax misurata oltre l'UB di R3 su micro prop ⇒ kill.
