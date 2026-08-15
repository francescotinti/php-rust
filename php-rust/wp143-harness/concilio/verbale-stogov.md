VERDETTO: CONCORDO CON EMENDAMENTI
DELIBERA: B-poi-A — B è semanticamente invisibile e ridefinisce la taglia/ABI dello Zval su cui A deve poi fissare l'handle; A com'è scritta nel dossier è semanticamente infondata e va riprezzata dopo l'istruttoria §7.1.

§Analisi (lente: semantica Zend/engine)
1. Il buco del dossier che nomino: «A azzera il dec/inc Rc nel churn» (§5) è FALSO rispetto a Zend. Zend NON azzera il refcount sugli oggetti: zend_object porta GC_REFCOUNT, ogni copia di zval obj fa GC_ADDREF/GC_DELREF, e il __destruct scatta a DELREF→0 (zend_objects_store_del). Togliere inc/dec sposta il destruct a fine request: la divergenza §3.22 (oggi UN caso catalogato su unset di elemento) diventerebbe SISTEMICA, e WeakReference::get() restituirebbe oggetti che l'oracle dichiara morti (corpus: `Zend/tests/weakrefs/*.phpt`, gh10043-001..010 in testa). L'acquisto onesto di A è: coppia alloc/free (quota oggetti dei 471M, IGNOTA fino al census per classe) + località + parte della nota obj — non il churn Rc.
2. A è comunque Zend-shaped: gli oggetti Zend sono GIÀ handle+store (EG(objects_store).object_buckets indicizzato da handle uint32, free-list di riuso; spl_object_id È l'handle). `===` via handle è sano. Ma il RIUSO degli handle dopo la morte è osservabile (spl_object_id ripetuti): un'arena che non ricicla gli handle diverge in modo lieve ma catalogabile PRIMA.
3. Ciò che Zend davvero non paga e phpr sì: la nota GC PER-MOVIMENTO (238,6M eventi). Zend annota solo possible-root a DELREF che non arriva a zero, bufferizzato. Questa terza componente non è né A né B nel dossier e non coincide col veto WP-21 (che era una leva di timing dentro lo schema esistente): va NOMINATA nella scommessa strutturale.
4. B (Zval by-value+niche) non tocca superficie semantica: nessun test del corpus a rischio per NOME. Ma il suo prezzo netto è sopravvalutabile: anche Zend paga memcpy da 16B e memops di hashtable — senza il profilo lato oracle (§7.2, regola one-sided) i 5,4+4,4 s non sono un tetto di acquisto. La direzione resta firmata dalle tasse §4 (Field* ~10×, costo/op invariante).

§Emendamenti
R1. A conserva il refcount stile Zend (arena = solo alloc/free + località); riprezzarla su questa base DOPO il census CH_* per classe (§7.1). Misura: quota oggetti dei 471M.
R2. Gate semantico pre-registrato per A: weakrefs/*, spl_object_id-riuso, fixture §3.22, destructor-set — fail-set per NOME invariato; ogni divergenza nuova a catalogo PRIMA della promozione, mai dopo.
R3. Nominare la componente gc-note→possible-root-at-decrement come voce propria del budget (0,5–1,2 s nota + parte famiglia gc), con A/B proprio.
R4. Profilo oracle per famiglia (§7.2) PRIMA di prezzare B; B parte subito solo come progettazione+criterio, la promozione aspetta il profilo.

§Veti (Q3)
NaN-boxing: CONFERMO (B via niche non lo richiede). Contenitori sul call path: CONFERMO (l'arena non ne introduce; vietato che A lo faccia). Alloc-removal senza modello del costo SOSTITUTIVO: CONFERMO e lo APPLICO ad A — bump/free-list, sweep di fine request, RSS e RetainSet (binding output-capture) vanno modellati prima del giudizio. SSO inline: CONFERMO (stringhe fuori perimetro). Leva GC note-time WP-21: CONFERMO sul timing; EMENDO il perimetro: la ristrutturazione possible-root (R3) è altra cosa, ammessa solo con criterio pre-registrato. Notti su PhpStr-full: CONFERMO.

§Kill-switch (Q4)
B: falsificata se, a leva spedita, churn_zval+memops (profilo campionario S-140, 2 repliche) non calano ≥25% relativo E la suite ORM resta dentro banda ±0,7% — entro 3 sessioni; giudici: profilo campionario + coppia ORM.
A: falsificata in istruttoria se il census CH_* dà agli oggetti <15% delle 471M coppie, o se il modello del costo sostitutivo mangia >50% del guadagno stimato — entro 2 sessioni; giudice: census monobinario + modello scritto. Gate semantico: un fail nuovo per NOME in weakrefs/destructor non catalogabile ⇒ STOP immediato.
