VERDETTO: CONCORDO CON EMENDAMENTI
DELIBERA: ISTRUTTORIA-PRIMA — il census CH_* per classe (§7.1) decide quanto compra A; intanto B è apribile subito perché non tocca il confine per-richiesta; A entra solo ricondizionata da R1–R4.

§Analisi (lente: confine per-richiesta e lifecycle)
1. **Il dossier refuta da sé l'opzione A come è SCRITTA.** §3 dichiara 29,4 GB allocati/run: un'arena per-request che non recupera intra-request richiederebbe ~29 GB residenti sulla suite ORM. Zend NON vive di sweep-a-fine-request: vive di pool/slab con free refcount-driven e riuso immediato. Quindi A va rinominata: «pool a classi con handle», non «arena+sweep». Il dossier non lo dice; è un buco che invalida la formulazione, non la direzione.
2. **A tocca esattamente il BINDING Pedersen/Stogov** (output capture PRIMA di request_end(); parità per-richiesta sul RetainSet). Un `__destruct` differito allo sweep emette output DOPO la cattura → violazione da reject; §3.22 mostra che la classe di divergenza esiste già oggi e si moltiplicherebbe.
3. **Il guadagno di A è NON quantificato**: dei 471M coppie non sappiamo la quota oggetti; gc_note obj è 56,5M su 238,6M (24%), e la famiglia gc è dominata dallo sweep (riconciliazione §3), non dalla nota. A potrebbe comprare 2 s come 12 s: differenza che decide la scommessa.
4. **B è coerente coi numeri firmati** (memops 5,4 s + churn 4,4 s + parte di vm_inline drop-glue) e non tocca RetainSet né destructor timing. Ma B da sola non compra la parità (~10 s su 37,6): la sequenza plausibile resta B-poi-A, da confermare col census.
5. **Manca il profilo lato oracle** (§7.2): senza, i canali che anche Zend paga (map, memops) sono sopravvalutabili — feedback-one-sided-profile.

§Emendamenti
- **R1 (destructor determinism)**: in A il `__destruct` resta refcount-driven nel punto esatto di fine-vita, MAI delegato allo sweep. Misura: fixture `__destruct`+echo con output-capture attiva; gate corpus per NOME sui test d'ordine di drop.
- **R2 (RetainSet fuori arena)**: i payload che sopravvivono alla richiesta non vivono nel pool per-request, o vengono evacuati/pinnati; handle con generazione, e il costo del generation-check ENTRA nel modello del costo sostitutivo. Misura: fixture parità per-richiesta WP (2ª richiesta byte-id).
- **R3 (pool, non arena)**: A ridefinita come slab con riuso intra-request; obbligo di A/B contro mimalloc (già slab, 8–15 ns/coppia) su objalloc/objchurn, criterio pre-registrato REGOLE §3.
- **R4 (ordine del confine)**: ogni sweep residuo per-request (cicli/leak) eseguito DENTRO request_end() DOPO la cattura output — il binding non si emenda, si implementa.

§Veti (Q3)
- NaN-boxing: **CONFERMA** (B = Option+niche by-value, non NaN-box).
- Contenitori sul call path: **CONFERMA**; il deref di handle deve essere indice O(1) in slab, mai hash; disasm prima/dopo sul run_loop obbligatorio.
- Alloc-removal senza modello del costo SOSTITUTIVO: **CONFERMA** — è il cuore di R3; nessuna promozione di A senza il prezzo di handle+generazione firmato.
- SSO inline: **CONFERMA** (str 0,8%: B non deve reintrodurlo).
- Leva GC note-time (WP-21): **CONFERMA**; A che azzera la nota per gli oggetti in pool è rimozione strutturale, NON riapre la leva sul tempo della nota.
- Notti su PhpStr-full: **CONFERMA** (fuori bersaglio).

§Kill-switch (Q4)
- **K1 (istruttoria, 1 sessione)**: se il census CH_* dà quota oggetti <30% delle coppie, A perde il titolo di headline → B sola prima; giudice: census monobinario datato, r1==r2.
- **K2 (istruttoria, 1 sessione)**: profilo oracle per famiglia; se Zend paga quota comparabile su map/memops, quei canali si scontano PRIMA della scelta.
- **K3 (B, ≤3 sessioni dall'apertura)**: A/B pre-registrato su objchurn/objdatains, segno 5/5; se la suite ORM non esce dalla banda ±0,7% a B completa → B ridimensionata, non estesa.
- **K4 (A, ≤4 sessioni dal prototipo)**: se su objalloc il costo sostitutivo (handle+generation-check) mangia ≥50% del rimosso, o un solo fail per NOME su ordine `__destruct`/RetainSet, o la fixture parità 2ª richiesta rompe → STOP A, si tiene B.
