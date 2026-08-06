# Verbale sedia 9 — Gregg (metodologia di misura, attribuzione) — Concilio WP-107 su S-105

## VERDETTO
**NON REFUTATO nei co-primari; PROMOSSO CON EMENDAMENTI.** La promozione della forma 2 regge: criterio 11cc23a committato PRIMA, T (Δ=+23,00, 5/5, R=5 ABAB, rumore ~3,5) ∧ C (census 0,0000 quarta cifra su alloc E free), admission senza flip su ENTRAMBE le forme (KS-BA-106-1 rispettato), G1 probe cap-bump = attribuzione di sito PER MISURA (19,9M eventi spostati esatti). Nessuna refutazione capitale. Due vizi di metodo emendabili.

## Refutazioni
- **R-GR-107-1 (il «~37» è un estimatore post-hoc, VOID come cifra).** Il contrasto (T1=−14)−(T2=+23) è calcolato DOPO, tra due A/B distinti contro lo stesso A; la forma 2 non esisteva all'atto zero, quindi il contrasto non era pre-registrabile — cade sotto l'emenda Leijen S-104. In più le due B differiscono per PIÙ del contenitore (forma 2 elimina anche reverse e transito bind_params): «37 = costo del contenitore» ha confondenti non separati. Legittimo come INDIZIO con confondenti nominati; VIETATO come cifra in premesse future.
- **R-GR-107-2 (early-stop assente, giustificazione retroattiva).** Lo smoke della forma 1 mostrava −15 con segno 2/2 opposto all'attesa; il pieno R=5 è stato speso senza regola. L'esito è stato fecondo (il pieno ha prodotto il controllo e il census a piena potenza), ma la giustificazione «era il controllo» è nata DOPO. Non spreco, vizio di pre-registrazione.

## Emendamenti
- **A-GR-107-1**: nel report e in NEXT_SESSION il «~37 ns/iter» va etichettato «contrasto post-hoc, ipotesi» — mai citato come costo firmato del contenitore. La cifra si firma solo con A/B testa-a-testa forma1↔forma2 pre-registrato (solo se mai servisse: backlog, non aprire).
- **A-GR-107-2**: la stima «calls ≈ f% del tempo WP» si calcola SOLO con la formula pre-registrata in KS-GR-107-3, non con estimatori scelti alla lettura.

## KS (regole)
- **KS-GR-107-1 (early-stop pre-registrato)**: se lo smoke R=2 dà segno 2/2 OPPOSTO all'attesa con |Δ|>max(rumore, tetto banda), è AMMESSO fermarsi con verdetto «caduta indiziata, non firmata»; il pieno R si spende solo dichiarando PRIMA l'uso (controllo/estimatore). Verdetto di PROMOZIONE mai da smoke.
- **KS-GR-107-2**: un contrasto tra due A/B distinti è sempre post-hoc: indizio con confondenti nominati, mai cifra.
- **KS-GR-107-3 (pre-registrazione lettura coppia WP, ORA, prima dei ratios)**: attesa **full CPU on ∈ [1,84, 1,89]** (da 1,89 WP-102; leva = −14% su ns/iter calls; attesa quota-calls f∈[5,20]%) · **media ∈ [2,57, 2,64]**. Stimatore pre-registrato: **f̂ = (1 − r_full/1,89)/0,14**, banda tra-sere ±0,02 propagata. Letture: r<1,83 ⇒ NON attribuire alla leva, sospetto ambientale/di modo; r>1,91 ⇒ regressione da indagare, non narrare; peak oracle = bande VOID (rumore ~10%), peak phpr giudicato con spread 34,64 MiB.

## §OGGETTO (mandato inverso)
Che cosa sappiamo di phpr oggi che ieri non sapevamo: (1) il canale alloc-args ESISTE ed è chiuso — calls 7,6→**6,3**, ns/iter 164→141, census 0,0000: prima leva promossa dopo 3 sessioni, contatore Δ-rapporti 3→0; (2) terza conferma indipendente che la valuta di run_loop è il volume di lavoro per op (mimalloc TL quasi gratis); (3) l'arità reale è bassa (73,1% ≤2 su 985.695 bind): il fast path copre il carico vero, non solo il giudice; (4) due rossi di fedeltà NOMINATI (§3.14 stub, §3.15 variadic by-ref oltre il 1° arg). Che cosa resta FERMO: **prop 11,5 e arith 12,4 — le due categorie peggiori, immobili da S-101/S-102**; a obiettivo ≤3× anche calls 6,3 è oltre il doppio. La rotta S-106/107 deve mordere prop/arith (H-C3), non raffinare calls. Rinvii: BLOCCANTI per S-106 = lettura coppia (.done) e grado PIENO server (senza, nessuna cifra server citabile, KS-PE-106-1); **contatori L1I diventano bloccanti SE si apre H-C3** (la roadmap li dichiara prerequisito di tesi); backlog onesto = terza mutazione OBS-8, mutante leak-parziale, generator get_gc, fixture §3.13 unit.
