# Verbale sedia 1 — HOARE (design linguaggio/runtime Rust, safe-only) — Concilio WP-105

**Oggetto**: sessione S-103 + bozza d'ordine §S-104. **Mandato**: refutare.

## VERDETTO: CON EMENDAMENTI

## Refutazioni

**R-HO-105-1 (CAPITALE) — Il criterio H-C2 pinna il predicato SBAGLIATO per il fast-out.**
`hc2-criterio.out` fissa «fast-out SOLO via `is_gc_container`» (eredità A-HO-104-5). Ma `is_gc_container` distingue *container collezionabile* da *non-container*, NON *drop banale* da *drop con lavoro*: `Str`, `Resource`, `Generator` rispondono `false` (zval.rs:229-234) eppure hanno Drop non banale (decremento Rc dell'interned string, finalizzazione del descrittore, stato del generatore). Un'implementazione letterale `if !v.is_gc_container() { /* salta drop-machinery */ }` salterebbe drop DOVUTI su specie che il giudice prop (solo Long negli slot) non esercita mai: il micro-A/B e il dump-diff passerebbero, il leak su Str/Resource resterebbe invisibile fino a WordPress. Il criterio protegge i 3 DropC/iter (container), ma il fianco scoperto è l'insieme `!container ∧ !scalare`. Non è un difetto del codice spedito (la leva non è aperta): è un difetto del CRITERIO, e va emendato PRIMA dell'implementazione.

**R-HO-105-2 (non capitale) — La refutazione della premessa A-ST-104-4 REGGE, ma l'assert è a copertura parziale.**
Verificato nel sorgente: `gc_note` instrada i `Ref` nel predicato (`Ref→true`, mod.rs:3926) e lo scartocciamento avviene DENTRO `gc_note_slow`; le catture by-ref sono `Ref` legittimi. La correzione di rotta è quindi fondata. Però il `debug_assert` nested-Ref (mod.rs:3977) vive SOLO nel braccio `strong_count > 1`: nel braccio `== 1` un Ref annidato ricorre in `gc_note(&inner)` e viene digerito in silenzio — l'invariante «i reference non si annidano» è sorvegliata su metà dei sentieri. Inoltre la lezione stessa di S-103 («l'assert si piazza dove il contratto VIVE») indica i siti di COSTRUZIONE del Ref, non solo il consumo: un Ref annidato nasce lì, e un assert nel descend morde solo se il Ref rotto viene notato mentre è condiviso. Nota onesta già a verbale: release senza debug-assertions ⇒ denti solo debug/census — accettabile finché le estensioni MOVE restano gated.

**R-HO-105-3 (minore) — Doc stantia con la premessa refutata.** Il commento di testa di `is_gc_container` (zval.rs:213-214) dice ancora «a `Ref` is unwrapped to its inner value by the caller» — la frase refutata sopravvive tre righe sopra il braccio che la smentisce. Doc contraddittoria = prossimo assert piazzato male.

**Su 19a/19b**: arbitrano davvero base=1 e soglia-esatta (dtor anticipato è osservabile). Ma sono passate «al primo colpo»: nessuna prova che POSSANO fallire. Un arbitro mai visto rosso non è ancora un arbitro (la fixture generator, invece, è nata rossa: quella sì).

## Emendamenti

- **A-HO-105-1 «predicato trivial-drop distinto»**: la leva H-C2 introduce `is_trivial_drop` (match ESAUSTIVO come `is_gc_container`, niente wildcard: true SOLO per Undef/Null/Bool/Long/Double + le specie provate Copy-like, ArgPlace/WeakHandle da classificare esplicitamente) e il fast-out si gate su QUELLO; `debug_assert!(is_trivial_drop(v) ⇒ !v.is_gc_container())` cross-check.
- **A-HO-105-2 «assert nested-Ref issato»**: portare il debug_assert PRIMA del branch strong_count (copre entrambi i sentieri) e valutare il gemello ai siti di creazione del Ref.
- **A-HO-105-3 «doc di is_gc_container emendata»**: riscrivere il commento di testa con la storia corretta (lo scartocciamento è nel descend).
- **A-HO-105-4 «braccio rosso per 19a/19b»**: perturbazione deliberata (off-by-one su una soglia OBS in build census) che le fa fallire, una volta, a registro.

## Kill-switch

- **KS-HO-105-1**: fast-out H-C2 con early-return su `!is_gc_container` (o qualunque predicato che renda `Str`/`Resource`/`Generator` saltabili) = reject senza appello.
- **KS-HO-105-2**: se A-HO-105-1 non è consumato, la leva H-C2 NON si apre — il criterio è VOID sul suo stesso pin «fast-out SOLO via is_gc_container».

## Priorità per l'ordine S-104

Concordo con la sequenza 1→5 della bozza, con UNA inserzione: il punto 2 (leva H-C2) premette A-HO-105-1 come atto zero (mezz'ora, non un prefisso nuovo: è la correzione del criterio già scritto). A-HO-105-2/3 entrano nell'igiene timeboxata; A-HO-105-4 prima di usare 19a/19b come gate di promozione.
