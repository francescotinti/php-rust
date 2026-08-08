# Verbale Pedersen — Concilio S-116/117 (lente: confine per-richiesta, lifecycle, parità per-request)

## VERDETTO: CONCORDO CON EMENDAMENTI

A subito, B regime, D metodo: sì, ma la raccomandazione com'è scritta contiene tre punti che dalla mia lente non reggono. E la clausola su C («solo se dopo A+B resta >3×») è mal posta: C non è UNA rotta, è due — una compatibile coi vincoli, una no.

## ROTTA DALLA MIA LENTE (3 sessioni)

Ordine: **A' (rescopata) → B (con ammissione per-vagone) → D (con classe-lifecycle dichiarata per tecnica) → C-lite come vagone di D, C-piena VIETATA nella forma arena/NaN-box**.

**Mossa S-117**: spike pipeline — LTO fat + codegen-units=1 + PGO cargo, con profilo che INCLUDE il teardown (request_end, distruttori, sweep RetainSet), non solo i loop micro; poi **rimisurare la banda leve-nulle (×2) sulla pipeline nuova PRIMA di qualunque verdetto di leva**. Gate pieni: batteria, corpus 1415 per NOME ×2, fixture, e tripla census per-request (obj/req deve restare 0,000 — la parità di WP-72 non è negoziabile).

## EMENDAMENTI

**R1 — A: BOLT probabilmente non esiste qui.** Siamo su Darwin ARM64/Mach-O; BOLT è di fatto ELF/Linux. «Layout deterministico» su macOS = order-file di ld64 o niente. MISURA: verifica di piattaforma in apertura S-117; se il tool non c'è, A si dichiara PGO+LTO-only, senza vantarsi del metro.

**R2 — A: «ripara il metro» è un'ipotesi, non un fatto.** PGO rende il layout FUNZIONE del profilo: ogni modifica di codice può rimescolare inlining/impaginazione anche PIÙ di oggi. MISURA: banda leve-nulle rimisurata sulla pipeline nuova; A «ripara il metro» solo se banda_nuova < 10 ns attuali, altrimenti A vale solo per il guadagno assoluto.

**R3 — A: il profilo deve pesare il confine.** Un profilo raccolto solo sui sei micro declassa a freddo request_end/distruttori/output-capture: il full WP (1,867×) e la parità dei giudici held-out possono peggiorare mentre i micro migliorano. MISURA: WP full ON e tripla census nel gate di promozione della pipeline.

**R4 — B: la somma giudica SOLO la performance.** Fedeltà e ammissione restano PER-VAGONE (parità output, dump, batteria, divergenze a catalogo per NOME): un treno che compensa una regressione di un vagone col guadagno di un altro è esattamente il buco che il fail-set congelato esiste per chiudere. Treno bocciato ⇒ revert AL BYTE dell'intero treno.

**R5 — C: spacchettare.** (i) NaN-boxing = bit-play che in Rust vive di transmute: collide col sigillo SAFE-only/VmGate — non riproporre senza dimostrazione safe. (ii) Arena per-richiesta con bulk-free: i distruttori PHP girano in ordine definito, eseguono codice arbitrario e possono risuscitare valori; il RetainSet persistente e ogni valore che sopravvive alla richiesta esigono promozione/copy-out; il bulk-free è lecito solo per valori trivially-drop. E l'ORDINE è mandato permanente: capture output → distruttori (FIFO gc_queue+gc_birth) → reset. (iii) **C-lite** = elisione di refcount sul path caldo con prova di lifetime safe: questa è compatibile e attacca proprio il collo Zval da 9-10 ns/op. MISURA d'ingresso per ogni pezzo di C: tripla census 0,000 + batteria ordering 8/8 byte-id + fail-set per NOME.

**R6 — D: ogni tecnica portata dichiara la sua classe di lifecycle PRIMA del codice.** Interned strings Zend sono per-processo: un intern che cattura stringhe request-local è un leak/una violazione di confine (la lezione RetainSet). Inline cache su class-pointer va invalidata se la tabella classi resetta per-richiesta. Checklist di confine nel criterio PRE di ogni vagone D.

## KILL-SWITCH
- KS-A1: piattaforma senza layout-tool E banda_nuova ≥ banda attuale ⇒ A decade a LTO-only in 1 sessione.
- KS-A2: PGO cambia il fail-set per NOME o la batteria ⇒ abort A, revert pipeline.
- KS-A3: 2 sessioni di A senza Δ spedito ⇒ si torna a B sulla pipeline corrente.
- KS-B1: vagone che fallisce ammissione ⇒ fuori dal treno, treno rigiudicato da capo.
- KS-C1: C-lite inesprimibile safe ⇒ morta, nel registro «NON riproporre».

## NON TOCCARE
request_end e il suo ordine; output-capture prima del reset; pinning per-richiesta del RetainSet; free-order FIFO dei distruttori. Qualunque rotta che li sfiora paga i gate di R5 per intero.
