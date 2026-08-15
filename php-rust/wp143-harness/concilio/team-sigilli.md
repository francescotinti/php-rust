# Team «sigilli» — sintesi fase 2 (Hoare, Matsakis) · Concilio S-143

Fonte VINCOLANTE: i verbali individuali (`verbale-hoare.md`, `verbale-matsakis.md`).

## §Convergenze
- **Entrambe le sedie: ISTRUTTORIA-PRIMA.** La variabile che decide A (quota per-classe dei 471M alloc/free, §7.1) non è nel dossier: è la grandezza deliberanda, non un limite. Votare A oggi = magnitudine ripartita senza A/B proprio.
- **R1 identico**: census CH_* per classe su ORM, giudice monobinario census, 1 sessione — decide quanto compra A.
- **R2 convergente**: profilo per famiglia lato ORACLE (feedback-one-sided-profile) + attribuzione memcpy + audit `size_of::<Zval>`/niche — il bersaglio è il DIFFERENZIALE, non la famiglia; prezza B e depura il tetto 9,8 s.
- **R3 convergente**: prima di scrivere l'arena, modello dei borrow (doppio-oggetto, re-entrancy con payload vivo, foreach-durante-mutazione, sopravvivenza oltre request) + sonda/micro pre-registrata del costo handle-deref (index+gen-check). Il costo SOSTITUTIVO di A non è prezzato: il veto alloc-removal morde A per nome.
- **Tutti e 6 i veti CONFERMATI** da entrambe (NaN-boxing: la niche di B ne compra la parte lecita in safe; contenitori sul call path: il deref d'arena vi sottostà).
- **RetainSet falsifica l'arena per-request pura**: serve promozione fuori arena; §3.22 (__destruct timing) diventa perimetro di soundness, non nota.

## §Conflitti
- **Sequenza attesa post-istruttoria — NON levigato**: **Hoare** pre-registra **B-poi-A** (B safe-banale e composabile, A carica di mine semantiche); **Matsakis** pre-registra il default **A-poi-B** se la quota oggetti+props supera la soglia R1 (B «non è una scommessa di parità», tetto ≥7× anche perfetta).
- **Soglia kill-switch divergente**: Hoare KS-1 = quota oggetti **<15%** ⇒ A cade; Matsakis = oggetti+props **<25%** ⇒ si ri-delibera. Basi diverse (oggetti soli vs oggetti+props), non riconciliate.
- **Budget di parità**: Matsakis mette a verbale il BUCO — anche A+B al massimo teorico restano ~15 s ⇒ ~3×: la scommessa compra la TAPPA, non la parità; esige la dichiarazione di un secondo atto (coda «other» 11,3 s + dispatch). Hoare non lo contesta ma non lo eleva a condizione del deliberato.

## §Delibera di team
**ISTRUTTORIA-PRIMA** (unanime 2/2); sequenza post-istruttoria DIVISA (Hoare B-poi-A · Matsakis A-poi-B condizionato a R1).

## §Priorità per l'ordine S-143/S-144
1. **Census CH_* per classe su ORM** (R1, giudice monobinario, 1 sessione) — con soglie kill-switch pre-registrate ENTRAMBE (15% oggetti / 25% oggetti+props).
2. **Profilo per famiglia lato ORACLE + attribuzione memcpy + size_of/niche Zval** (R2) — stessa sessione; KS-3 Hoare può retrocedere B senza codice.
3. **Solo se R1 passa**: modello borrow su carta + spike/micro handle-deref pre-registrata (R3) — prima di ogni riga d'arena.
