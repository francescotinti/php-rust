# WP_SESSION_37 — archivio storico della sessione WP-37

> Estratto da NEXT_SESSION_WORDPRESS.md alla rotazione WP-40 (convenzione: un file per sessione; il handoff tiene solo l'ultima).
> Dettagli gemelli in memoria: topic php-rust-wordpress-track.

> ⚡ **WP-37 (2026-07-22, gated `32a5820`, 1 commit)** — **leva C (sottoinsieme
> sicuro) + groundwork attribuzione per B**. **(C) `Func.simple_call`**
> precomputato nei 6 costruttori (pattern has_hints WP-31): no hints, no
> by-ref, no variadic, non-generator — ⭐ i default NON contano: i fast
> path si attivano solo ad **arity ESATTA**, quindi il prologo default non
> vede mai Undef (identico a oggi). Due fast path: `bind_params` (argc ==
> n_params ⇒ decay diretto negli slot — uguale per costruzione al ramo
> generico non-variadico con param_by_ref tutti falsi) e `enter_callee`
> (solo push del frame — ⭐ call_line e caller_strict alimentano SOLO il
> TypeError degli hint, che un simple_call non può sollevare). Le guardie
> "tipi dell'ultimo hit" di Gemini restano NON fatte (rischiose, ordine
> coercion/TypeError).
> **Esito: call-heavy dedicato −2,0% (4/4 coppie interleaved vs 1b2db38);
> bench36 completo −0,4%; media 60,07/20,94 = 2,87×** (vs 61,4 di
> mattina = coerente col −2% e col rumore di giornata); full ~12:30 ≈
> run27 (rumore); footprint 12,0×. **run28 = run27 per nome** (30.472,
> 0E/2F); gate22 TUTTO verde + ⭐ refold delle 4 suite phpt (editato il
> census nel tree a gate in corso — cfg-out ma la regola è regola);
> cargo **1627** (+1).
> **📊 ATTRIBUZIONE STRINGHE (input della sessione B — misurata, feature
> `php-types/str-census`, run media strumentata `wp36-harness/
> str-census.txt`)**: il processo master crea **51,8M PhpStr** in un run
> media (~40s user ⇒ ~1,3M stringhe/s; 1,72GB di byte cumulativi), OGGI
> = **~104M malloc** (2 alloc/stringa). Istogramma: 0B=0,16M ·
> 1-7B=11,99M · 8-15B=13,69M · 16-23B=3,42M · 24-31B=4,24M ·
> 32-63B=8,77M · 64-255B=9,42M · 256+=0,13M. **Cumulato: ≤15B = 49,9% ·
> ≤23B = 56,5% · ≤31B = 64,7%.** Verdetto per B: **SSO cap 15** (PhpStr
> resta 24B: tag+len+buf[15] al posto del Box) elimina ~25,8M alloc+free
> (≈25% del canale stringhe) senza crescere la struct; cap 23 (struct
> 32B) aggiunge solo +6,6 punti — ⭐ partire da cap 15, valutare cap 23
> solo coi dati footprint. Il 35% in 32-255B resta heap comunque.
> ⚠️ MIMALLOC_SHOW_STATS nei run strumentati produce i 15 errori
> separate-process noti (WP-26/33) e questa build mimalloc non espone i
> conteggi per-bin (denominatore totale non disponibile) — run di sola
> misura, mai per parità.
> **→ PROSSIMA SESSIONE = B (SSO PhpStr in zstr.rs)**: enum
> `{ Inline { len, buf: [u8; 15] }, Heap(Box<[u8]>) }` dentro PhpStr
> (MAI nello Zval — invariante 16B WP-27), hash cell conservata,
> `PhpStr::new` è il funnel UNICO (verificato: from_str/empty vi
> passano); i match sui siti d'uso passano tutti da `as_bytes()`.
> Poi E (gc batching, sentinelle drop-order pinnate prima).
