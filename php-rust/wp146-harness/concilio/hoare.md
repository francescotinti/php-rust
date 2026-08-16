# Sedia HOARE — Concilio S-146 (B3/filone conteggi)
Lente: design linguaggio/runtime Rust, SAFE-ONLY, sigilli di tipo (VmGate ZST, assert 16B/niche, unsafe solo nel crate types — ZStr S-124).

## VERDETTO: CONCORDO CON EMENDAMENTI

Fatto aritmetico che il fascicolo NON trae dalla sua stessa sonda: **TakeSlot non rimuove il pavimento**. Un take è ancora un movimento — in safe Rust è `mem::replace(slot, Undef)`: copia 16 B e scrive Undef; elimina solo inc-dec (0,21 s) e nota (0,25 s), e solo sulla frazione take-abile. Il 69,5% memcpy resta intatto. Chi rimuove il pavimento è il **borrow-through** (precedenti SPEDITI HC1/L-FR1): il valore non si materializza affatto. Dai prezzi firmati: borrow ≈ −2,9…−3,9 ns per movimento EVITATO; take ≈ −0,5…−1,0 per movimento preso. «Muovere MENO» = borrow prima, take poi.

## Posizioni a–e

**a) forma d'emissione — CONCORDO CON EMENDAMENTI.** Il flag su LoadSlot è esprimibile senza corpo caldo nuovo: campo nel payload dell'op, branch dentro il braccio esistente, per-sito (BTB); sfugge al prerequisito O1 (che vale per corpi NUOVI). MA un campo aggiunto a una variante NON fa scattare il sigillo dei match esaustivi (una variante nuova sì): serve R1+R2. Restano dovuti nm -S predetta (A-LB-97-1) e disasm bl-count (criterio-B p.4).

**b) perimetro — CONCORDO.** Perimetro fedele di Stogov (nucleo-stringhe, niente identità). Dalla mia lente il guard di tipo è indipendente dal perimetro statico: match sulla variante, `Ref` ⇒ fallback clone. safe_ref 0,013% lo rende quasi gratis e MAI superfluo (recount S-96: la correttezza non si misura in frequenza). Il move safe non tocca la repr: le assert 16/8/niche restano intatte per costruzione, zero unsafe nuovo.

**c) censimento F1 su ORM — SÌ ma CONDIZIONATO:** serve solo se TakeSlot supera l'istruttoria borrow. Prima un census dei SITI borrow-abili su ORM (pattern-census classe FR1, zero liveness): più economico, rischio zero.

**d) alternative — CON EMENDAMENTO D'ORDINE (R4).** FR1-ext (chiave da slot, FieldRead/isset) è la leva PRIMA: semanticamente invisibile, zero liveness, e il borrow checker È il sigillo — un borrow tenuto attraverso una chiamata che può mutare il frame NON COMPILA; safe Rust fa da giudice statico gratis. Arena-conteggi: MAI definita; onere di definizione ≤1 pagina col costo sostitutivo, altrimenti ARCHIVIATA (come nominata collide coi veti alloc-removal/contenitori sul call path).

**e) cosa compra — CONCORDO, con cifra più severa.** Take modellato ≤ ~0,46 s × frazione take-abile (solo incdec+nota); borrow aggredisce anche la quota memcpy dei movimenti evitati, ma sempre DENTRO l'1,52 s modellato; ~4,4 s glue fuori modello: nessun claim oltre la risoluzione.

## Emendamenti

- **R1** — `SlotMode` enum {Copy, Take}, non bool: match esaustivi ⇒ un futuro Borrow non compila in silenzio. Misura: compile-fail test per nome.
- **R2** — `Take` costruibile SOLO con token ZST rilasciato dal modulo liveness (pub(crate); precedente VmGate): un'emissione take senza analisi non compila. Misura: compile-fail per nome.
- **R3** — mutation-check del guard: il mutante senza guard DEVE morire su fixture nominata Ref-in-slot-safe (lezione WP-104: il mutante si fa sull'arbitro del rischio).
- **R4** — ordine istruttoria: 1) FR1-ext borrow (census siti ORM + criterio ≤10 righe); 2) F1-liveness SU ORM solo se resta residuo che paga; 3) TakeSlot forma-flag con R1–R3; 4) arena archiviata salvo definizione.
- **R5** — sentinella dinamica read-after-take nel build census (verifica a macchina della liveness statica) PRIMA di ogni emissione.

## Kill-switch pre-registrabili

- **KS-H1**: read-after-take > 0 su corpus 1414×2 o ORM ⇒ STOP emissione take.
- **KS-H2**: mutante-guard sopravvive ⇒ giudice inesistente ⇒ STOP fetta.
- **KS-H3**: bl-count run_loop aumenta o nm -S oltre il predetto ⇒ forma respinta (Δ bracci caldi ≤ 0).
- **KS-H4**: census siti borrow su ORM sotto soglia REGOLE §3 (max(4 ns, rumore, banda)) ⇒ FR1-ext non si apre, si passa al punto 2 di R4.
- Vigente e citato: fail NUOVO per NOME in weakrefs/destructor ⇒ STOP.

## Mandato inverso (Gregg)

Oggi sappiamo che il collo è il pavimento move (2,88 ns × 367,6M) e che take NON lo rimuove — ieri «filone conteggi» era sinonimo di TakeSlot. E i would_take esistono solo su media-WP: su ORM sono IGNOTI.
