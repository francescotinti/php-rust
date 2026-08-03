# Verbale SEDIA 2 — Matsakis (Concilio WP-94)

Perimetro: ownership/aliasing, atomics/ordering, cfg-soundness. Mandato: REFUTARE.

## VERDETTO: PASS-CONDIZIONATO
La meccanica (edge Release/Acquire, monotonia ARRIVALS, cfg-split) regge; la **sufficienza formale** della coppia pulita come dichiarata è REFUTATA (Q3).

## Q1 — L'edge chiude il gap dichiarato? SÌ, ma solo in una direzione.
Dec `fetch_sub(1, Release)` (worker_pool r.607-608) / load `Acquire` (r.305-306): quando `outstanding_post` legge-da la dec, l'intero teardown del worker happens-before il load — il gap dec→load dichiarato (main r.268-269) è chiuso. Inoltre la catena ARRIVALS-inc (r.686) →sb→ send (r.689) →hb canale→ recv (r.515) →sb→ dec(Release) →sw→ post-load(Acquire, main r.282) →sb→ arr_post (r.283) forza arr_post a vedere ogni arrivo la cui dec è stata letta: la richiesta window-contained È catturata in ogni esecuzione fresca. L'inc Relaxed di OUTSTANDING (r.681) e di ARRIVALS (r.686) fra loro non apre buchi nuovi: ogni riordino inc/inc visibile produce pre≠0 o arr_pre≠arr_post ⇒ ADVISORY, fail-closed. **Il buco non è di ordering: è di freshness** — nessun ordering (nemmeno SeqCst) obbliga il testimone a leggere l'ultimo valore; la visibilità in tempo finito è QoI hardware (intro.progress è "should"), non garanzia del modello.

## Q2 — cfg-split coerente? SÌ sulla monotonia, NO su un dente dichiarato.
Ogni path che inc-a OUTSTANDING inc-a ARRIVALS nello stesso blocco `cfg(any(census-instrumentation, mem-census))` (r.673-687); QUEUE_DEPTH simmetrico census-only (inc r.676-680, undo r.706-707). Il failed-send NON undoa ARRIVALS (r.704-708): monotonia da contratto rispettata (doc r.222-224). **Refuto però il dente d'appoggio**: il commento r.700-703 delega la cattura del dispatch-Err al «driver's rows==N» — ma la riga census per-richiesta è census-instrumentation-ONLY (r.180-182): sul canale mem-census rows==N NON ESISTE, e ARRIVALS resta gonfiata di un arrivo mai entrato. VOID dichiarato senza trigger nominato sul canale che conta.

## Q3 — Esiste la sequenza che mente? SÌ (macchina astratta).
Dispatcher: inc OUTSTANDING→1, inc ARRIVALS→N+1, send. Testimone (endpoint inline main r.279-283, NESSUN edge col dispatcher: non passa da dispatch()): `outstanding_pre` legge lo **0 iniziale stale** (l'inc Relaxed non happens-before il load; l'Acquire non compra freshness), `arr_pre`=N+1 (fresca). Il worker esegue e smonta DURANTE il dump, dec→0. `outstanding_post`=0 (iniziale o post-dec: indifferente), `arr_post`=N+1. **pre==post==0 ∧ arr_pre==arr_post con finestra SPORCA**: esecuzione coerente (read-read coherence rispettata) e permessa dal modello C++. Su x86-64/AArch64 reale il RMW lockato è subito coerente e il dump dura ms ⇒ praticamente esclusa; la protezione REALE è la sequenzialità del driver, non la coppia. La coppia è tripwire fail-closed NECESSARIA, non prova SUFFICIENTE.

## Q4 — lsp_check.rs, fragilità pre-wiring.
(a) `link` batch order-dependent con mappa propria (r.1195-1206): al wiring la tabella `Linked` persistente = **seconda verità** parallela alla class table VM; `Linked` clona in profondità ogni membro ereditato (String-heavy, O(depth×membri)) — doppio modello o churn di conversione. (b) **Antenato assente = skip silenzioso** (`if let Some(pl) = map.get(..)` r.1216-1227): fail-open al wiring con classi condizionali/autoload — serve policy nominata. (c) `deps` si PERDONO su `Err` (r.1202-1205): Zend emette la Deprecated al punto di scoperta; il wiring deve emettere subito o l'ordine diverge. (d) `lc = to_lowercase()` Unicode (r.398-400) vs `zend_str_tolower` ASCII-only di Zend: nomi UTF-8 con case-folding non-ASCII (İ) collidono da noi e non in Zend — x5 non lo copre. (e) `unreachable!` su compound in `resolve_atom` (r.692-694): con input dal parser reale = abort di processo; mappare a Err.

## Emendamenti
- **A-MS-59**: riscrivere il commento main r.268-274: la coppia pulita è condizione NECESSARIA fail-closed; verdict-grade SOLO con enforcement sequenziale del driver dichiarato in-band; nessun rafforzamento di ordering compra freshness — dichiararlo.
- **A-MS-60**: trigger VOID per dispatch-Err NOMINATO sul canale mem-census (HTTP 5xx osservato dal driver, o contatore DISPATCH_ERR su `any(...)`).
- **A-MS-61**: lsp_check pre-fase-2: policy antenato-assente + emissione deprecation al punto di scoperta + `lc` ASCII-only + Err al posto di `unreachable!`.

## Kill-switch
- **KS-MS-94-1**: «window clean» consumata come verdict-grade senza riga in-band di enforcement sequenziale del driver ⇒ Δ di fase VOID.
- **KS-MS-94-2**: run mem-census con dispatch-Err senza trigger VOID nominato ⇒ run VOID.

## Refutazioni capitali
- **SÌ (1)**: sufficienza formale di pre==post==0 ∧ arr_pre==arr_post (Q3: esecuzione stale-coerente con finestra sporca e verdetto pulito).
- **NO**: su edge dec→load (reale), monotonia ARRIVALS (regge), cfg-split degli inc (coerente).
