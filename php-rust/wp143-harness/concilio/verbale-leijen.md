VERDETTO: CONCORDO CON EMENDAMENTI
DELIBERA: ISTRUTTORIA-PRIMA — census CH_* per classe + sonda prezzi + bilancio bytes (1 sessione), poi A-poi-B condizionata all'esito.

§Analisi (lente allocatore/footprint)
1. Il dossier sopravvaluta il canale ns/coppia. Il fast-path mimalloc (pop thread-local) sta a 4–8 ns; un bump-allocator sta a 2–4. Su 471M coppie il guadagno DIRETTO di un'arena è ~1–3 s, non 26–28. La famiglia alloc leaf è 2,3 s (§2): anche azzerandola tutta, il rapporto resta >7. Quindi l'opzione A NON si giustifica come «arena batte mimalloc»: si giustifica solo se compra i canali adiacenti — dec/inc Rc nel churn (4,4 s), drop-glue in vm_inline (quota dei 7,0 s), nota GC obj (56,5M), e la pressione cache dei 29,4 GB mossi — tutte voci a magnitudine NON ripartita (REGOLE §4). La delibera «A adesso» poggerebbe interamente su grandezze non misurate: buco nominato.
2. Il vero regalo dell'arena non è ns/coppia ma (a) eliminazione del FREE individuale (bulk reset: metà della coppia + niente purge/deferred mimalloc) e (b) località: mimalloc sparge per size-class page, l'arena serializza i payload che l'ORM tocca insieme — il bersaglio è memops 5,4 s e la coda cache-bound di «other» 26,6%. Non prezzabile oggi: il dossier lo ammette (§3 ultima riga). Serve il prototipo-giudice, non la fede.
3. Incoerenza di bilancio: free 33,8 GB > alloc 29,4 GB per run. O il census conta i realloc due volte (9,6 GB mossi) o c'è doppio conteggio nei path di drop. Prima di prezzare qualunque cosa sui bytes, il bilancio deve chiudere.
4. Footprint: un'arena per-request è HIGH-WATER — ciò che muore a metà richiesta resta vivo fino al reset. Su ORM (migliaia di entità/hydration per request) il picco può esplodere e lo shrink −70 MB conquistato è un vincolo storico, non negoziabile. A senza riuso interno o senza gate footprint è inammissibile.
5. Costo sostitutivo di A: l'handle aggiunge un deref (load dalla tabella) su OGNI accesso payload; su propget 29,9M+ letture è un prezzo reale, da modellare per iscritto prima del prototipo (veto pertinente, sotto).

§Emendamenti
R1 — Census CH_* per classe E per taglia: quota oggetti/props dei 471M e dei 29,4 GB. Decide A vs B coi numeri, non a lettura. Giudice: monobinario census, ×2 repliche.
R2 — Sonda monobinaria prezzi alloc/free reali (classe S-138): sostituire l'8–15 ns «plausibile» con una misura firmata.
R3 — Chiudere il bilancio bytes (free>alloc) prima di ogni prezzo sui GB.
R4 — Gate footprint permanente su A: vmmap Physical footprint (mai RSS) su WP e ORM, soglia pre-registrata.
R5 — Modello scritto del costo sostitutivo di A (deref handle × conteggi propget/recv_clone) PRIMA del prototipo.

§Veti (Q3)
NaN-boxing: CONFERMA (B = Option+niche; se scivola verso NaN-boxing il veto morde). Contenitori sul call path: CONFERMA — la tabella handle sia slab/indice diretto, mai HashMap. Alloc-removal senza modello del costo SOSTITUTIVO: CONFERMA, ed è il cuore di A (R5: deref + high-water SONO il costo). SSO inline: CONFERMA, fuori perimetro. Leva GC note-time (WP-21): CONFERMA — il dossier stesso mostra nota 0,5–1,2 s vs sweep dominante; A la riduce solo come sottoprodotto. Notti su PhpStr-full: CONFERMA, stringhe fuori da A/B.

§Kill-switch (Q4)
KS1: census CH_* — oggetti+props <30% delle coppie E <30% dei bytes ⇒ A retrocede, B prima. Giudice: census; 1 sessione.
KS2: prototipo A su micro oggetti — guadagno ABAB <15% su objchurn/objalloc (o segno opposto, R=5, banda) entro 3 sessioni ⇒ A si ferma.
KS3: footprint — WP > baseline+10% o picco ORM > +20% (vmmap) ⇒ stop/redesign riuso.
KS4: suite ORM post-promozione — rapporto net non sotto 8,0 fuori banda ±0,7% entro 2 sessioni dal prototipo promosso ⇒ lettura ciclo-di-vita falsificata per la quota oggetti.
