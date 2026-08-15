# Team «semantica-confini» — Stogov · Pedersen (S-143, fase 2)

I verbali individuali restano la fonte VINCOLANTE.

## §Convergenze
- Entrambi: CONCORDO CON EMENDAMENTI; tutti i 6 veti Q3 confermati (NaN-boxing, contenitori sul call path, alloc-removal senza costo sostitutivo, SSO inline, leva note-time WP-21, notti PhpStr-full).
- **A come è scritta nel dossier è infondata**: per Stogov il claim «azzera il churn Rc» è falso rispetto a Zend (destruct differito ⇒ §3.22 sistemica, weakrefs divergenti); per Pedersen i 29,4 GB/run refutano l'arena-senza-riuso e il `__destruct` allo sweep viola il binding output-capture. A va ricondizionata: refcount conservato stile Zend, destruct refcount-driven nel punto esatto di fine-vita, pool/slab con riuso intra-request, RetainSet fuori pool (handle+generazione nel costo sostitutivo).
- **Il guadagno di A è ignoto** finché il census CH_* per classe (§7.1) non dà la quota oggetti dei 471M; **B non va prezzata** senza profilo oracle per famiglia (§7.2, one-sided-profile).
- **B è semanticamente invisibile** (nessun test a rischio per NOME, non tocca RetainSet né destructor timing) ed è apribile subito con criterio pre-registrato.
- Gate semantico per NOME obbligatorio su A: weakrefs/*, riuso spl_object_id, fixture §3.22, ordine destruct, parità 2ª richiesta; un fail nuovo non catalogabile ⇒ STOP.

## §Conflitti
- **Stogov**: delibera **B-poi-A** — sequenza firmata: B ridefinisce taglia/ABI dello Zval su cui A fissa l'handle; A si riprezza dopo l'istruttoria ma la sequenza è già decisa. Kill A se quota oggetti **<15%**. Chiede di NOMINARE la componente gc-note→possible-root come voce propria del budget (R3), distinta da A e B.
- **Pedersen**: delibera **ISTRUTTORIA-PRIMA** — il census DECIDE se A entra: sotto **30%** di quota oggetti A perde il titolo di headline e resta **B sola**; B-poi-A è solo «plausibile, da confermare». Insiste che A sia rinominata «pool a classi con handle» e che il binding non si emenda: sweep residui DENTRO request_end() DOPO la cattura.
- Soglie kill divergenti (15% vs 30%) e status della sequenza (firmata vs condizionata): non levigato.

## §Delibera di team
B apribile subito + ISTRUTTORIA-PRIMA su A ricondizionata (R1–R4 di entrambi); sequenza B-poi-A indicata da Stogov ma CONDIZIONATA al census per Pedersen — soglie kill 15%/30% non riconciliate.

## §Priorità per l'ordine S-143/S-144
1. Census CH_* per classe (quota oggetti dei 471M, monobinario, r1==r2) + profilo oracle per famiglia — decide A e sconta i canali che anche Zend paga.
2. Aprire B (Zval by-value + niche) con criterio pre-registrato: churn+memops −25% relativo, ORM in banda ±0,7%, ≤3 sessioni.
3. Ricondizionamento scritto di A (pool+refcount+generazione+costo sostitutivo, A/B contro mimalloc) e gate semantico per NOME pre-registrato — PRIMA di ogni prototipo.
