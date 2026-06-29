# Robustness improvements - audit Codex

Data: 2026-06-27

Repo analizzata: `/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust`

Obiettivo: suggerire interventi per rendere piu' robusto il runtime PHP-in-Rust, dopo l'introduzione del logging con `log4rs`.

## Sintesi

L'aggiunta di `log4rs` e' una buona scelta: il progetto usa il facade `log`, il logging e' spento di default, e l'output va su stderr/file senza contaminare stdout, che deve restare byte-perfect per confronti con PHP.

Le aree dove ora conviene investire sono:

1. Logging piu' affidabile e osservabilita' per fasi VM/corpus.
2. Igiene repo contro file AppleDouble `._*`, che gia' stanno disturbando `git`.
3. Riduzione delle panics raggiungibili e gestione esplicita degli invarianti VM.
4. Limiti di risorse configurabili: stack, heap, output, regex cache, include/autoload.
5. Hardening di `php-server`, che esiste ma non e' nel workspace Cargo.
6. Tooling di supply-chain, fuzzing e CI locale ripetibile.

Nota tooling: `php-rust/CLAUDE.md` richiede Serena/Vexp, ma in questa sessione tali MCP non erano disponibili tra gli strumenti esposti. La scansione e' stata fatta con `rg`, `cargo metadata` e letture mirate.

## Evidenze rapide

- `php-runtime/src/logging.rs` inizializza `log4rs` con `PHPR_LOG`, `PHPR_LOG_FILE`, `PHPR_LOG_CONFIG`.
- `php-cli` e `phpt-runner` chiamano `php_runtime::logging::init()`.
- Sono presenti circa 10 call site `log::...!` nel runtime: run/compile, include, gc, calls, exceptions.
- Una ricerca su sorgenti runtime/builtins/types/cli/runner segnala circa 255 occorrenze `unwrap()`/`expect()` nei sorgenti; molte sono invarianti interne o test inline, ma vanno classificate.
- Sono presenti circa 11 occorrenze `unsafe`, concentrate soprattutto in wrapper libc/filesystem.
- `git status` ha emesso errori su `.git/objects/pack/._pack-...idx`.
- Esistono file `._*` dentro sorgenti e crate, per esempio `crates/php-runtime/src/._mod.rs`, `crates/php-builtins/src/._array.rs`, `php-rust/._Cargo.toml`.
- `crates/php-server/Cargo.toml` esiste, ma `cargo metadata --no-deps` non lo lista nei `workspace_members`.
- Non risultano config locali per `cargo-deny`, `cargo-audit`, fuzzing, `.github/workflows`, `clippy.toml` o `rust-toolchain`.

## P0 - Interventi ad alto impatto

### 1. Rendere il logging fallibile in modo visibile

Problema:

- `logging.rs` ignora il risultato di `log4rs::init_file` e `log4rs::init_config`.
- Se `PHPR_LOG_CONFIG` punta a un file rotto, il logger puo' restare disattivato senza segnale.
- `Once` impedisce un secondo tentativo nello stesso processo anche dopo un errore iniziale.

Suggerimento:

- Aggiungere `pub fn try_init() -> Result<(), LoggingError>` e lasciare `init()` come wrapper silenzioso solo per compatibilita'.
- In `phpr` e `phpt-runner`, se l'utente ha impostato `PHPR_LOG*` e l'inizializzazione fallisce, stampare un warning su stderr.
- Accettare livelli case-insensitive (`WARN`, `Warn`, `warn`) e rendere `PHPR_LOG=off` esplicito.
- Aggiungere una micro-suite di test per:
  - logger off di default;
  - config path inesistente;
  - file appender non scrivibile;
  - `PHPR_LOG_FILE` che non deve mai scrivere su stdout.

### 2. Aumentare la copertura dei log nei punti di diagnosi

Il logging oggi e' corretto ma ancora scarno. Aggiungerei target stabili:

- `phpr::lower`: parse/lower start, failure class, line, construct.
- `phpr::compile`: unsupported compile con op/expr/stmt e file.
- `phpr::vm`: start/end run, fatal class, exit code.
- `phpr::autoload`: registrazione/rimozione loader, nome classe, recursion guard.
- `phpr::builtin`: missing builtin, host builtin error, warning routed.
- `phpr::limits`: superamento limiti runtime.
- `phpr::phpt`: test path, status, categoria skip/fail, durata.

Regola importante: non loggare valori PHP completi a livello `info` o `debug` se possono essere enormi o sensibili. A `trace`, troncare e indicare lunghezza.

### 3. Pulizia e prevenzione AppleDouble `._*`

Problema osservato:

- Ci sono file `._*` nei sorgenti.
- `git status` produce errori su pack index AppleDouble dentro `.git/objects/pack`.
- Le istruzioni locali citano gia' problemi Serena con `._*.rs`.

Suggerimento:

- Aggiungere script `scripts/clean-appledouble.sh`:
  - cancella `._*` fuori da `.git`;
  - opzionalmente segnala quelli dentro `.git` invece di cancellarli automaticamente.
- Aggiungere script `scripts/preflight.sh` che fallisce se trova `._*` fuori da `.git`.
- Aggiungere `.gitignore` robusto:
  - `._*`
  - `.DS_Store`
  - `**/._*`
- Documentare per macOS/volumi esterni:
  - `export COPYFILE_DISABLE=1`
  - usare `dot_clean` solo consapevolmente su directory di lavoro, non alla cieca su `.git`.

### 4. Classificare `unwrap()`/`expect()` in runtime production

Problema:

- Il VM usa molti `expect()` su stack operand, frame, iteratori, class id e path.
- Molti rappresentano invarianti reali del compilatore bytecode, ma se uno e' raggiungibile da PHP input diventa crash dell'host, non errore PHP.

Suggerimento:

- Introdurre una categoria `VmInvariantError` con:
  - opcode corrente;
  - instruction pointer;
  - frame/function;
  - stack depth;
  - file/line PHP se disponibile.
- Creare helper locali:
  - `pop_stack(top, "OpName operand") -> Result<Zval, PhpError>`;
  - `last_frame() -> Result<usize, PhpError>`;
  - `expect_object_class(...) -> Result<ClassId, PhpError>`.
- Priorita' di conversione:
  1. Stack pops nel main dispatch loop.
  2. `unwrap()` in coroutine/generator/fiber state.
  3. `expect()` su sync method result e object class id.
  4. `expect()` in array/object path helpers.
- Lasciare `debug_assert!` per invarianti provate dal compiler, ma evitare panic in build release.

### 5. Aggiungere panic boundary per binari e server

`phpt-runner --isolate` gia' contiene i crash dei singoli test tramite processo figlio. `phpr` e `php-server` invece non hanno una boundary.

Suggerimento:

- In `phpr`, valutare `std::panic::catch_unwind` intorno a `run_source_with_argv`, restituendo exit code 255 e messaggio interno su stderr quando il runtime panica.
- In `php-server`, evitare che una panic in `spawn_blocking` diventi `.await.unwrap()` e risponda con crash/task abort.
- Loggare panic payload e backtrace quando `PHPR_LOG` e' attivo.

## P1 - Limiti e isolamento

### 6. Introdurre `RuntimeLimits`

Esiste gia' `MAX_CALL_DEPTH = 25_000` e `PHPT_TIMEOUT_SECS` per isolate runner. Mancano pero' limiti omogenei.

Proposta:

```rust
pub struct RuntimeLimits {
    pub max_call_depth: usize,
    pub max_instructions: Option<u64>,
    pub max_output_bytes: Option<usize>,
    pub max_output_buffer_depth: usize,
    pub max_include_depth: usize,
    pub max_autoload_depth: usize,
    pub max_preg_cache_entries: usize,
    pub max_tracked_objects: Option<usize>,
}
```

Usi:

- CLI: default molto permissivo, compatibile con PHP.
- phpt-runner: limiti diagnostici per catturare loop e memory blow-up.
- server: limiti stretti per richiesta.
- test differenziali: limiti fissi e loggati per riproducibilita'.

### 7. Limitare cache e collezioni per richiesta

Punti da guardare:

- `preg_cache: HashMap<Vec<u8>, Option<Rc<Engine>>>` e' per-run ma non sembra avere limite.
- `included_files`, `autoloaders`, `autoloading`, `shutdown_fns`, `created`, `generators`, `fibers`, `enum_cache`, `constants` crescono con il programma.
- Output buffering e stdout/rendered possono crescere senza soglia.

Suggerimento:

- Per CLI puro si puo' restare quasi illimitati.
- Per runner/server, applicare limiti configurabili e fatal PHP-like quando possibile.
- Aggiungere metriche finali a `phpr::run`/`phpr::limits`: max frames, output bytes, objects tracked, includes, preg cache size.

### 8. Regex hardening

Punti positivi:

- `preg.rs` fissa `fancy_regex` backtrack limit a 1_000_000.

Da completare:

- Rendere il limite configurabile per test/server.
- Loggare quando una regex cade sul motore `fancy-regex` invece del motore `regex`.
- Mettere un limite alla cache PCRE per richiesta.
- Per `onig`/mbregex, valutare timeout/limiti dove disponibili o almeno test di pattern patologici.

### 9. Filesystem policy opzionale

I builtins file usano direttamente il filesystem host. Questo e' corretto per CLI compatibility, ma non per server o harness non fidato.

Suggerimento:

- Introdurre una `FsPolicy` opzionale:
  - `open_basedir`;
  - allow/deny write;
  - allow symlink follow;
  - temp dir controllata;
  - path canonicalization centralizzata.
- Il CLI default resta permissivo.
- Il server usa policy restrittiva.

## P1 - `php-server`

`crates/php-server` e' il punto piu' acerbo.

Problemi osservati:

- Non e' membro del workspace root.
- Non chiama `php_runtime::logging::init()`.
- Path traversal difeso solo con `path.contains("..")`.
- Usa `spawn_blocking(...).await.unwrap()`.
- Usa `TcpListener::bind(...).await.unwrap()` e `axum::serve(...).await.unwrap()`.
- Risponde sempre `Html<String>` senza status code HTTP corretto.
- Espone errori runtime come HTML/plain text.

Suggerimenti:

- Decidere: includerlo nel workspace o spostarlo fuori finche' non e' supportato.
- Se resta:
  - aggiungere a `[workspace].members`;
  - usare `PathBuf`, percent-decoding e canonicalizzazione;
  - verificare che il path canonicale resti dentro `public/`;
  - restituire `StatusCode`;
  - configurare address/docroot da env/CLI;
  - gestire `JoinError` senza panic;
  - inizializzare logging;
  - non mostrare dettagli interni in risposta HTTP se non in debug.

## P1 - Supply chain e CI locale

Mancano config visibili per audit dipendenze e CI.

Suggerimento:

- Aggiungere `rust-toolchain.toml` per fissare toolchain.
- Aggiungere `cargo fmt --check`.
- Aggiungere `cargo clippy --workspace --all-targets -- -D warnings` almeno in CI/preflight.
- Aggiungere `cargo deny` con:
  - advisory DB;
  - licenze consentite;
  - duplicate deps;
  - ban/allow per crate native (`onig`, `libc`, crypto).
- Aggiungere `cargo audit` se non si usa `cargo deny advisories`.
- Aggiungere script `scripts/preflight.sh` che esegue:
  - clean/check AppleDouble;
  - cargo metadata;
  - fmt;
  - clippy;
  - test;
  - phpt smoke selezionato.

## P2 - Fuzzing e property tests

Per un runtime di linguaggio, fuzzing e' molto redditizio.

Target consigliati:

1. Parser/lower/compiler:
   - input: bytes PHP casuali o corpus mutato da Zend/tests;
   - oracle: non deve panicare; se parse/lower fallisce deve tornare errore classificato.

2. VM bytecode invariants:
   - input: programmi PHP piccoli generati;
   - oracle: `phpr` non deve panicare; output confrontato con PHP quando il programma e' nel subset supportato.

3. Builtins puri:
   - array/string/number conversion/json/serialize/pack.
   - usare `proptest` con confronto a oracle per casi piccoli.

4. Regex:
   - pattern e subject generati con limiti;
   - oracle: nessun hang, nessuna panic, errore classificato.

Setup:

- `cargo fuzz` per fuzzing byte-level.
- `proptest` per proprieta' deterministiche.
- corpus seed da `.phpt` ridotti.

## P2 - Error taxonomy e osservabilita' test

Il runner gia' distingue pass/fail/skip e categorie unsupported. Si puo' rendere ancora piu' utile:

- Ogni `Unsupported` dovrebbe avere codice stabile, non solo stringa.
- Esempio: `LOWER_VARIABLE_VARIABLE`, `COMPILE_DYNAMIC_NAMED_CALL`, `VM_INVARIANT_STACK_UNDERFLOW`.
- Il runner aggrega per codice e stampa top-N con esempi.
- I log `phpr::phpt` includono durata, categoria, fatal class e file.
- Salvare report JSON opzionale: `--json-report out.json`.

## P2 - Unsafe e wrapper libc

Le occorrenze `unsafe` sembrano concentrate su `libc` filesystem (`statvfs`, `access`, `utimes`). Suggerimento:

- Isolare ogni unsafe in funzioni piccole in un modulo `os`.
- Documentare invarianti sopra ogni unsafe block.
- Aggiungere test per path con byte non UTF-8, symlink, permessi, file mancanti.
- Valutare `cargo geiger` periodico per avere inventario.

## Sequenza consigliata

1. Aggiungere preflight AppleDouble e correggere la repo sporca.
2. Rendere `logging::try_init()` osservabile e aggiungere test logging.
3. Decidere destino di `php-server`: nel workspace e hardened, oppure fuori scope.
4. Introdurre `RuntimeLimits` minimo: call depth, output bytes, preg cache entries.
5. Convertire i primi 20 `expect()` del VM dispatch in errori interni contestualizzati.
6. Aggiungere `cargo-deny`/`cargo-audit` e `rust-toolchain.toml`.
7. Avviare fuzzing su lower/compile "no panic".

## Nota finale

Il progetto ha gia' una robustezza non banale: runner isolato con timeout, call-depth guard, backtrack limit regex, logging spento di default e stdout protetto. Il prossimo salto di qualita' e' trasformare queste buone difese locali in una politica coerente: preflight, limiti runtime, panic boundaries, errori invarianti e report riproducibili.
