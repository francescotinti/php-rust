# VERBALE — Niko Matsakis, sedia 2, Concilio WP-91

VERDETTO: CON EMENDAMENTI (una refutazione capitale su Q4).

## Q1 — A-MS43: sigillo di visibilità
Verificato: `crates/php-server/src/worker_pool.rs:74-77` — `thread_local!` PRIVATO, nessun `pub`. Il modulo contenitore NON è il file: è il `mod implementation` interno (riga 46, cfg `axum-server`), che si chiude a riga 1929 e contiene ~1880 righe INCLUSO `mod tests`. Il `pub use` (1931-1935) esporta solo `census_probe_active` (lettura, main.rs:512) — `ProbeWindow` è `pub(crate)` dentro un mod privato ⇒ irraggiungibile fuori dal file. Scrittori esterni: morti a compilazione, confermato (belt 1183-1194 coerente; nota: `pub(crate) static` farebbe npriv=0 ⇒ FAIL, corretto). Scrittori interni residui OGGI: nessuno oltre la coppia arm/Drop (righe 104/111, uniche `.set` nel file). MA il perimetro di privacy è l'intero `implementation`: ogni riga futura del modulo (test compresi) può scrivere il flag senza errore di compilazione — lì giudica solo la belt, che ha buchi (Q3). Il sigillo è completo verso l'ESTERNO, sovradimensionato verso l'interno → A-MS46.

## Q2 — A-MS44: #[must_use] e sito d'uso
Sito reale riga 386: `let probe_window = ProbeWindow::arm();` — binding NOMINATO, vivo fino al `drop(probe_window)` riga 406, A VALLE del `catch_unwind` (387-405). Corretto: il hook scatta sul thread in panico DENTRO la finestra; `abort()` a 411 non ripassa dal hook. Però KS-MS-90-3 NON è oggi macchina-distinguibile: `#[must_use]` morde solo l'espressione nuda `arm();` — `let _ = arm()` SILENZIA il lint E droppa subito (il wildcard non lega). La belt (1196) conta solo la PRESENZA dell'attributo, non la forma del sito. Distinzione a macchina esiste ma va armata: `clippy::let_underscore_must_use` (allow di default) oppure pin testuale sul sito → A-MS47.

## Q3 — A-MS45 belt + .done rc=
Belt (gate-lever-pins.sh 1203-1212), buchi PER NOME, tutti interni a worker_pool.rs (visibilità copre l'esterno):
1. `.with(|g| g.set(true))` — binder ≠ `f`: sfugge al conteggio A-MS41 (1105, letterale `|f| f.set(`) e ad A-MS45 (nrepl copre solo replace|update, NON set). Terzo scrittore ⇒ belt PASS. Buco capitale della belt.
2. Grafie `take()`/`swap()` (Cell E LocalKey::take scrivono false) assenti da entrambe le regex.
3. UFCS `LocalKey::set(&CENSUS_PROBE_ACTIVE, true)` — nessun prefisso puntato ⇒ invisibile.
4. Split multilinea (classe A-TH49) — grep per riga.
.done (gate-parity-83p1.sh 81): rc= in-band è giusto, ma il marker NON viene rimosso all'avvio: run uccisa prima della riga 81 ⇒ .done STANTIO della run precedente con rc/git vecchi; il campo git= permette la verifica ma nessuno la impone → A-MS49.

## Q4 — A-DL46-census: aliasing e visita annidata
Aliasing Rust: `theap_heap_cb` (1074-1082) crea `&mut *(arg)`, fa push, poi passa il RAW `arg` alla visita annidata — sotto Stacked/Tree Borrows è SANO perché `acc` non è più usato dopo il push (il raw padre sopravvive al reborrow); margine fragile: un uso di `acc` dopo la annidata sarebbe UB → riderivare da arg. Le callback sono sequenziali sullo stesso thread: nessun overlap di `&mut`.
REFUTAZIONE CAPITALE: il blocco viola il PROPRIO invariante dichiarato (righe 698-700: «the visitor MUST NOT allocate — it walks the very heap that would serve the allocation»; BinTab usa array fissi per questo). `theap_heap_cb` fa `Vec::push` e `theap_area_cb` fa `BTreeMap::entry().or_insert()`: ALLOCANO via mimalloc mentre `mi_subproc_visit_heaps` enumera i heap e mentre `mi_heap_visit_blocks` cammina le pagine del heap di default del thread visitante — che È tra i heap visitati. Mutazione della struttura sotto iterazione lato C: "non è crashato" in m89 non è soundness. La dichiarazione KL-90-3 copre win=0 post-teardown (quiescenza degli ALTRI thread) ma non copre l'auto-allocazione.

## Emendamenti
- **A-MS46**: restringere il perimetro — flag+ProbeWindow+getter in un `mod probe` annidato (~40 righe), export `pub(super)`; anche i test del modulo muoiono a compilazione.
- **A-MS47**: sito d'uso a macchina — belt: ==1 `let <ident> = ProbeWindow::arm()`, ==0 grafie `let _ =`/espressione nuda; in catena `-D clippy::let_underscore_must_use`.
- **A-MS48**: A-MS45 estesa: closure-set con binder generico, take/swap, UFCS LocalKey, decoy che mordono same-commit.
- **A-MS49**: `rm -f "$W/.done"` in testa a gate-parity-83p1.sh; consumer DEVE confrontare git= con HEAD.
- **A-MS50**: census per-theap senza allocazione nei visitor (capacità pre-riservata prima della visita, bins ad array fisso classe BinTab) e `acc` riderivato da arg dopo la annidata.

## Kill-switch
- **KS-MS-91-1**: terzo scrittore del flag in worker_pool.rs a revisione senza A-MS48 ⇒ attribuzione probe ADVISORY.
- **KS-MS-91-2**: righe mi_theap_* raccolte con worker NON joinati/quiescenti ⇒ righe VOID.
- **KS-MS-91-3**: righe mi_theap_* prodotte da visitor che alloca dal subproc visitato (pre-A-MS50) ⇒ ADVISORY, mai verdict-grade.

Firmato: Niko Matsakis, sedia 2, Concilio WP-91.
