# WP_SESSION_39 — archivio storico della sessione WP-39

> Estratto da NEXT_SESSION_WORDPRESS.md alla rotazione WP-40 (convenzione: un file per sessione; il handoff tiene solo l'ultima).
> Dettagli gemelli in memoria: topic php-rust-wordpress-track.

> ⚡ **WP-39 (2026-07-22/23, gated `ed5d0f4`+`073914d`, 2 commit)** — **warm-up
> §3.7 chiuso + leva E RIDEFINITA DAI DATI: fast-shutdown + sweep empty
> fast-path = media −5,0% (2,85×→2,71×), full −4,4%.**
> **Warm-up (`ed5d0f4`)**: (1) `flag_value_sort` per sort/rsort/asort/arsort
> (NUMERIC via to_double; STRING/LOCALE byte ±FLAG_CASE pre-folded; NATURAL
> strnatcmp ±ci; chiavi 1× per elemento, convenzione natsort) + arm
> SORT_NATURAL mancante in `key_flag_cmp` (ksort/krsort). ⭐ sort NON
> gate-abile §1.1 ($flags runtime) ⇒ Stringable = fallback-warning, residuo
> dichiarato. (2) diagnostica string-offset READ in `read_dim_warn`,
> oracle-pinned: "Uninitialized string offset N" (N pre-aggiustamento),
> "String offset cast occurred" (float/bool/null, ⭐ senza deprecation
> float-precision), "Illegal string offset \"5abc\"" + USO del prefisso
> (prima TypeError ERRATO; "1.5" resta TypeError), Resource-key warning;
> isset/`??`/empty silenti come Zend (residui dichiarati: deprecation float
> su isset, coalesce "5abc"). **8 phpt upstream chiusi: corpus 1455→1447
> (0 nuovi), baseline wp18 aggiornata.** (3) ⭐⭐ **incdec deprecation GIÀ A
> PARITÀ**: il "gap" della probe WP-38 era la copia log "PHP Deprecated:"
> dell'oracle (log_errors=On nel php.ini brew) — **le probe oracle vanno
> SEMPRE con `-d log_errors=0`**. ⭐ visto di passaggio: `$fn($arr,$fl)`
> dinamico su builtin by-ref = "out of slice" (gap noto, probe con switch).
> **Leva E (`073914d`) — attribuzione prima, che RIBALTA la diagnosi**:
> ⭐⭐ il tail-sample WP-36 mescolava shutdown e run: il call-tree mostra 45%
> della finestra in UN drop ricorsivo delle hashmap del Vm a fine processo
> + 18% in `exit → mi_process_done` — tutto POST-semantico (qualificare le
> finestre di sample; attribuire i mi_* per CHIAMANTE). Purge A/B:
> MIMALLOC_PURGE_DELAY=0 costa solo ~0,5% user (+1,9s sys) — non è il collo.
> **gc-census** (feature nuova, pattern op-census, PHPR_GC_CENSUS=1|/path):
> media master = 177M gc_note (47M inserted), 53M sweep per-statement
> (~87% a buffer VUOTO), **47,5M demotion vs 2,1M freed = 95,6% churn**,
> 1 solo cycle-collect (50k root → 1 freed, soglia 50k→100k), 1001 dtor.
> **Le due leve (semantics-identical per costruzione)**: (a) `FAST_SHUTDOWN`
> static — il php-cli one-shot LEAKA il Vm a fine `run_module_with_hir`,
> DOPO l'intera sequenza osservabile (shutdown fn → dtor → session → filtri
> → OB flush) = Zend fast RSHUTDOWN; ⭐ MAI per `phpr -S` né host in-process
> (phpt-runner non-isolate e test cargo continuano a droppare); zero unsafe
> (mem::take dei campi outcome + mem::forget). (b) **sweep empty fast-path**
> nell'arm Op::Sweep: queue vuota (⇒ roots/birth vuoti per invariante) +
> niente light_demoted da ri-seminare (main) + cycle_roots sotto soglia ⇒
> skip totale del corpo. **Esito A/B (2 coppie interleaved, 762 test
> identici): old 59,80/59,77 → new 56,83/56,74 = −5,0% user, sys −20%;
> media pair 56,79/20,93 = 2,71×; full master-CPU 11:56 (−4,4%)**. Probe
> distruttori new≡old byte-id; 📌 scoperta divergenza PREESISTENTE: timing
> dtor in-function (Zend libera al return — "d11" prima dell'output
> dell'echo — phpr allo statement-sweep). ⭐ maxrss media +9% (3,85→4,20GB)
> = accounting MADV_FREE macOS (meno madvise): script controllato = picco
> IDENTICO (267,8MB) e CPU −11% anche lì — caveat WP-20 sulla riga
> footprint. **Gate/run**: gate22 TUTTO verde ×2 (warm-up e leva E; corpus
> 1447 IDENTICO al secondo giro); **run30 = run29 PER NOME** (30.472,
> 0E/2F/86W/73S; 88 righe fail identiche); cargo **1634** (+3).
> **Riprofilo (`wp39-harness/gc-out/new-wp39.sample`, stessa finestra t=35s
> del pre-leve `media-p0a.sample`)**: gc_sweep_impl **141→38** (−73%),
> gc_note 122→111 (canale residuo), drop/clone Zval 134/173, slot_of 60,
> resolve_prop_access 47.
> **→ PROSSIMA SESSIONE = demote churn (47,5M)**: flag in-object
> (`Cell<bool> in_gc_buf` su Object) al posto del probe hashmap di gc_roots
> + buffer Vec<Rc> al posto di map+queue — stessa FIFO, stessa dedup per
> costruzione; ⭐⭐ sentinelle drop-order pinnate PRIMA (metodo WP-32); poi
> ripensare cycle_roots (il collect è quasi-mai-utile: 1/run, 1 freed su
> 50k root). Il canale gc_note (177M chiamate, walk array/closure) è il
> secondo bersaglio.

> **📘 Post-WP-39 (2026-07-23)**: adottato il
> [code-migration-kit](https://github.com/anthropics/code-migration-kit-with-claude-code)
> di Anthropic — **`migration/RULEBOOK.md`** ora è la fonte citabile delle
> regole del porting (posture, ecosistema, mappature canoniche, BUG rule,
> giudice); read-only in sessione, emendamenti via sign-off utente + handoff.
> Marker grammar `BUG(port):`/`PERF(port):`/`TODO(port):` adottata per le
> deviazioni future nel codice. Skill `code-migration` installata user-level
> (kit clonato in `/Volumes/Extreme Pro/Claude/migration-kit`) per migrazioni
> future; phpr resta sul proprio processo (post-Step-6). In coda WP-39 anche:
> **sentinelle drop-order GC committate** (`9ed457b`, cargo 1636) e **GitHub
> sincronizzato** (`0e566ab`: corpus 2609/64,3%, WP single+multi a 1 diff,
> perf 2,71×/2,11× pubblicate).
