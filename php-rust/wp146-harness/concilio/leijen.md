# Verbale LEIJEN — Concilio S-146 (B3/filone conteggi) — bozza indipendente
Lente: allocatore (mimalloc) / footprint fisico — prezzi alloc/free, bilancio bytes, località.

## VERDETTO: CONCORDO CON EMENDAMENTI

Refutazione centrale (R1): **KS-B4 dice che il collo è il memcpy (69,5%), ma TakeSlot NON compra il memcpy**. Un take converte clone→move: la copia dei 16 byte RESTA; si elidono solo inc/dec (14,1%) e nota (16,4%), cioè la MINORANZA (30,5% = 0,46 s del perimetro modellato 1,52 s), scalata poi da would_take e dal perimetro fedele. L'unica mossa che compra il pavimento «sposta e smista» è NON generare il movimento: borrow-first/through-borrow ai siti consumatori (HC1, L-FR1: zero liveness, zero identità, spedite). Il filone conteggi va quindi ORDINATO con borrow-first come braccio primario e TakeSlot come braccio residuale.

## Posizioni a–e
- **a) CON EMENDAMENTI**: la forma-flag su LoadSlot è l'unica ammissibile (tetto WP-39..44, nm -S predetta, disasm bl-count invariante su run_loop); ma la forma non sana R1 — il flag decide take vs clone, non elimina la copia.
- **b) CONCORDO**: perimetro Stogov (CV non consumati, morte mai anticipata); nucleo-stringhe = unico perimetro senza identità. Dalla mia lente: il take è alloc-neutro anche su str (clone ZStr = inc, non malloc) — nessun acquisto sul canale alloc, vedi R2.
- **c) CONCORDO — serve**: i conteggi 47,1%/90,2% sono del media group WP; il mix ORM è diverso e il moltiplicatore 4,5–6,5% è SCREEN R=1. F1 su ORM è census a rischio zero e oggi ha finalmente un prezzo per-movimento firmato per moltiplicarlo — ma col prezzo GIUSTO (R3).
- **d) CON EMENDAMENTI**: borrow-first PRIMA di TakeSlot (R1). «Arena-conteggi»: **ARCHIVIARE**. Dalla mia lente non è definibile coerentemente: se significa drop-a-blocco è una leva di PREZZO travestita da conteggi (e KS-B4 ha appena mostrato che il prezzo non è il collo), viola il veto alloc-removal-senza-costo-sostitutivo, il binding output-capture, e il mio reperto S-143 (l'arena non batte mimalloc sul ns/coppia, ~1–3 s diretti). Se qualcuno la rivuole, rientra solo come A-pool (pool+refcount+handle-gen) con gli oneri S-143 — fuori dal filone conteggi.
- **e) CON EMENDAMENTI**: B3 compra al più fette del perimetro modellato 1,52 s (tappa, mai parità; nessun claim sui ~4,4 s glue). In più, dalla mia lente: **B3 compra ZERO del canale alloc** — i 471,3M pair (str 27,6%, other 57,9%) restano INTERI. Tranche-3 growth-alloc è quindi **complementare** in istruttoria (census a conteggi, niente codice, non blocca B3) e **concorrente** per la leva successiva: il residuo 57,9% senza nome è più grande di tutto ciò che B3 può comprare.

## Emendamenti
- **R1** (cosa/perché/misura): ordine istruttoria = FR1-ext borrow-first (chiave da SLOT, FieldRead/isset) → F1-ORM census → TakeSlot flag-form solo se (inc-dec+nota)×would_take_ORM supera la soglia REGOLE §3. Misura: criterio per-sito ≤10 righe, micro dedicata, ABAB R=5.
- **R2**: ogni criterio B3 dichiara «galloc/gfree invarianti per costruzione»; census di guardia a conteggi. Un delta galloc sotto una fetta B3 = effetto non capito ⇒ STOP e nominare.
- **R3**: i prezzi pair zcell 8,71–8,80 / arr0 11,57–11,79 sono sotto gate MAI ricollaudato (az.rev. #5): grado **INDIZIO**, e comunque **peso NULLO nel budget B3** — prezzano un canale (nascite/morti) che B3 non tocca; servono solo a un'eventuale A-pool, dove pretenderebbero rerun sotto gate 5%.
- **R4**: gate footprint vmmap resta su ogni fetta (Undef negli slot e borrow non devono muovere il fisico).

## Kill-switch pre-registrabili
- **KS-L1**: fetta B3 con delta galloc_n fuori parità ⇒ STOP fetta.
- **KS-L2**: criterio che usa i prezzi pair come budget di una fetta conteggi ⇒ criterio invalido (giudice sotto-risoluto).

## Mandato inverso (Gregg)
Oggi sappiamo che il pavimento per-movimento è dispatch+move e che nessuna leva della famiglia «take» lo tocca: ieri il filone conteggi sembrava un'alternativa al prezzo, oggi sappiamo che metà della famiglia (take) paga lo stesso pavimento e solo il borrow lo evita.
