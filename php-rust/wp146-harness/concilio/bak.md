# Sedia BAK — Concilio S-146 — quesito B3/filone conteggi

Lente: VM hot-path (alloc-rate, icache/BTB, code-cache); miei il tetto WP-39..44, O1, A-LB-97-1. Bozza indipendente; mandato: refutare.

## VERDETTO: CON EMENDAMENTI

L'aritmetica della sonda REFUTA TakeSlot come prima leva del filone: KS-B4 dice che il collo è il pavimento (memcpy 1,06 s), e TakeSlot NON muove meno — muove più a buon mercato. Un take è ancora un movimento (dispatch + copia 16 B + scrittura Undef): elide inc/dec e al più la nota, cioè pesca SOLO nelle fette incdec 0,21 s + nota 0,25 s = 0,46 s massimi teorici (al 100% di eleggibilità, irrealistico: would_take_rc 47,1% su media-WP). Realisticamente ~0,2 s, cioè SOTTO la banda del giudice della scommessa (±0,7% coppia ORM ≈ ~0,3 s, derivata). Una leva il cui massimo modellato sta sotto la risoluzione del proprio giudice non è istruibile come scommessa suite.

## Posizione a–e

**a) CON EMENDAMENTI.** Due sub-forme, pedaggi diversi, il criterio deve nominarne UNA: (i) bit `take` dentro il braccio LoadSlot esistente = Δ corpi caldi 0 ⇒ A-LB-97-1 soddisfatto per costruzione e **O1 NON è prerequisito** (rispondo al mandato: O1 era prerequisito per «ogni corpo caldo IN PIÙ»; qui non ce n'è); il vincolo però NON evapora — si trasforma in vincolo di TAGLIA: il flag allarga il braccio (branch+Undef-write+skip-inc, stimo +16–48 B) e può spingerlo oltre la soglia d'inlining, che è il modo esatto in cui è caduta H-C2 (inliner flippato, bl 1101→0, WP-104). (ii) opcode-variante `LoadSlotTake` = braccio nuovo, target BTB nuovo ⇒ tetto pieno + O1 prerequisito. Nessuna delle due si istruisce senza nm -S PREDETTA prima.

**b) CONCORDO** (Stogov S-96): perimetro fedele = solo nucleo senza identità; guard di tipo su Ref obbligatorio — ramo quasi mai preso (safe_ref 0,013%), ben predetto dal BTB, quindi economico, MAI superfluo.

**c) CONCORDO, anzi lo esigo:** i conteggi S-95/96 sono media-WP; il giudice è ORM. Senza censimento F1-ORM con classi ALLINEATE alla partizione sonda (per derivare il guadagno in SECONDI = conteggio × prezzo firmato, non in %CPU screen) nessuna forma si apre.

**d) CON EMENDAMENTI — è il cuore:** la famiglia borrow-first/through-borrow ai siti consumatori è l'UNICA che comprime il CONTEGGIO e quindi l'unica coerente con KS-B4: attacca il pavimento 1,06 s più incdec+nota dei movimenti eliminati; due precedenti SPEDITI (HC1, L-FR1 −28%) con zero liveness. Va istruita PRIMA; TakeSlot retrocede (R2). Arena-conteggi: mai definita, non riduce movimenti per alcuna definizione nota, e i veti alloc (quota_obj 2,4%, costo sostitutivo) mordono ⇒ ARCHIVIARE salvo definizione scritta che riduca movimenti.

**e) CONCORDO** (Matsakis R4): perimetro modellato 1,52 s su 37,6 s ≈ 4% del gap — B3 compra una frazione della tappa, mai la parità; nessun claim oltre la risoluzione (glue ~4,4 s fuori modello).

## Posizione secca sul pavimento
**2,88 ns/movimento è comprimibile SOLO per-conteggio.** Il dispatch ~9–10 ns è invariante dichiarato (S-103, non-bersaglio); il residuo è copia 16 B + smistamento al pavimento macchina; taglia e niche già incassate, rappresentazioni alternative vietate. Chi promette di comprimerlo per-movimento deve prima nominare il meccanismo.

## Emendamenti
- **R1**: prima mossa = censimento consumatori su ORM (ripartizione dei 367,6M per sito/opcode, monobinario ×2, r1==r2); guadagno atteso in secondi.
- **R2**: TakeSlot ammesso solo se il censimento mostra movimenti take-eligible NON borrowable con (incdec+nota)×conteggio ≥ banda giudice; altrimenti chiuso per aritmetica.
- **R3**: forma-flag: nm -S di run_loop E del braccio LoadSlot predette prima/misurate dopo; sub-forma nominata nel criterio.
- **R4**: bl-count (s144-criterio-B p.4) NECESSARIO ma NON sufficiente per H-C2: aggiungere delta-taglia run_loop PREDETTO; scostamento ⇒ STOP prima dell'A/B.

## Kill-switch pre-registrabili
- **KS-BAK-146-1**: bl-count o taglia run_loop fuori predizione ⇒ STOP fetta, niente A/B.
- **KS-BAK-146-2**: censimento ORM: (movimenti eliminabili × prezzo) < banda coppia ⇒ famiglia ridimensionata a micro-only, niente scommessa suite.
- **KS-BAK-146-3**: micro in segno ma pair zcell/arr0 fuori gate 5% ⇒ leva in istruttoria (guardia layout).

Mandato inverso (Gregg): oggi sappiamo che le leve «muovere più a buon mercato» hanno un tetto aritmetico di 0,46 s — ieri TakeSlot sembrava il filone; e che la famiglia borrow spedisce senza liveness (HC1, L-FR1 lo provano).
