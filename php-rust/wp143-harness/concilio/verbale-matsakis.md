VERDETTO: CONCORDO CON EMENDAMENTI
DELIBERA: ISTRUTTORIA-PRIMA — il numero che decide A vs B (quota per-classe dei 471M alloc/free) NON è nel dossier (§7.1, ammesso dal dossier stesso); default pre-registrato A-poi-B se la quota oggetti+props supera la soglia di R1.

§Analisi (lente ownership/aliasing/borrow)
**A (handle+arena).** L'handle Copy elimina inc/dec Rc e la coppia alloc/free per oggetto, ma il costo SOSTITUTIVO è il ri-borrow: ogni tocco del payload = index+generation-check via `&Arena`/`&mut Arena`. Conseguenze non prezzate dal dossier: (1) i siti che oggi tengono un guard RefCell attraverso una chiamata re-entrante (call_method_sync, __get/__set, hooks) non possono tenere `&mut arena` viva — obbligo di take/put o borrow corti, cioè lookup ripetuti sul cammino caldo; (2) le op a due oggetti richiedono split-borrow (`get_disjoint_mut`) — safe, ma codice nuovo su OGNI sito; (3) beneficio: iterazione-durante-mutazione migliora (indici stabili, niente panic da alias), e il mass-drop d'arena uccide i cicli gratis. Però RetainSet e oggetti che sopravvivono la richiesta (binding output-capture) falsificano l'arena per-request PURA: serve promozione fuori arena, e §3.22 (__destruct timing) diventa perimetro, non nota. Precisione: A toglie della nota GC al più obj 56,5M su 238,6M (≈24%); gli scalar 73,7M restano.
**B (Zval by-value+niche).** Riduce clone/drop/memcpy ma i punti che oggi prestano `&mut` dentro la mappa restano identici: l'aliasing non cambia, cambia la taglia mossa. Tetto aritmetico: memops 5,4 + churn 4,4 ≈ 9,8 s; anche B perfetto lascia rapporto ≥7. Composabile, rischio semantico basso, ma non è una scommessa di parità.
**BUCO del dossier, da mettere a verbale:** anche A+B al massimo teorico (26–28 s azzerati) lasciano ~15 s su oracle 4,97 ⇒ ~3×. La scommessa compra la TAPPA ≤3×, NON la parità: va dichiarato un secondo atto (coda «other» 11,3 s + dispatch residuo).

§Emendamenti
- **R1** (decide A): census CH_* per classe su ORM, monobinario census s140, r1==r2 al singolo evento; output = quota oggetti/props vs array vs stringhe vs Vec-args dei 471M. 1 sessione.
- **R2** (decide il budget): profilo per famiglia lato ORACLE (feedback-one-sided-profile): un canale che Zend paga in quota simile esce dal budget di parità.
- **R3** (pre-implementazione A): modello dei borrow su carta — i 4 pattern (doppio-oggetto, re-entrancy con payload vivo, foreach-durante-mutazione, sopravvivenza oltre request) + sonda monobinaria del costo handle-deref (index+gen-check) PRIMA di scrivere l'arena.
- **R4**: il deliberato dichiari A+B ≠ parità; budget residuo ~15 s nominato.

§Veti (Q3)
- **NaN-boxing: CONFERMA.** In safe-only la niche si fa via enum layout (B), non bit-tricks: B ne cattura il grosso senza unsafe.
- **Contenitori sul call path: CONFERMA.** L'handle-deref deve essere slab-index O(1) prezzato da R3; se introduce hash sul cammino caldo, il veto morde.
- **Alloc-removal senza modello del costo sostitutivo: CONFERMA, ESTESO ad A per nome:** il sostitutivo di A è index+gen-check+ri-borrow per tocco; senza R3, A non si vota.
- **SSO inline: CONFERMA** (str 0,8%: fuori bersaglio).
- **Leva GC note-time (WP-21): CONFERMA.** A rimuove strutturalmente le note obj: cosa diversa dal tuning del tempo-nota.
- **Notti su PhpStr-full: CONFERMA**, non pertinente.

§Kill-switch (Q4)
- **Istruttoria:** R1 dà oggetti+props <25% dei 471M ⇒ il canale principale di A è falsificato, si ri-delibera. Giudice: census CH_* ×2, ≤1 sessione.
- **A:** prototipo arena sui micro-oggetti; objchurn/objalloc non migliorano ≥2× al giudice micro R=5 (criterio REGOLE §3) entro 3 sessioni ⇒ A cade. Soundness: corpus 1414 ×2 per NOME invariato; regressione __destruct oltre §3.22 ⇒ reject (binding output-capture).
- **B:** replica profilo suite ×2; famiglie memops+churn non calano della quota pre-registrata entro 2 sessioni ⇒ B cade.
- **Budget:** R2 mostra quote relative oracle comparabili su memops/map ⇒ attribuzione §5 da rifare prima di implementare.
