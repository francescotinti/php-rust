# Piano: PHP → Rust come reimplementazione moderna (non porting di Zend)

> **Nota**: questo è il piano *iniziale* (Fase 0), conservato come artefatto storico.
> Per lo **stato corrente** (step 0–61, 934 test, struttura dei crate, tooling) vedi
> `README.md` e il diario in `diary/03-translation-log.md`.

## Context

L'utente vuole migrare PHP (sorgente C in `/Volumes/Extreme Pro/Claude/php-8.5.7`, ~1,94M LOC) verso Rust. Dopo una prima proposta di porting fedele a Zend (opcode VM), la direzione è cambiata su sua indicazione: **non ricreare lo Zend Engine** — è un design del 1999-2004 — ma sfruttare Rust e il suo ecosistema per una reimplementazione moderna e semplificata, prima il core del linguaggio, poi le estensioni.

**Principio guida**: il contratto da preservare non è l'architettura di Zend ma il **comportamento osservabile** di PHP. L'oracle esiste già: **21.548 test .phpt** (95% self-contained) che confrontano lo stdout di uno script. Qualunque runtime che produce lo stesso output è "PHP". Questo trasforma il lavoro da *traduzione del C* a *reimplementazione guidata dalla spec*, dove il C si legge solo per chiarire semantica ambigua.

### Phase 0 — Reconnaissance (completata)
- Nessun runtime PHP completo e maturo esiste in Rust → greenfield per il runtime.
- **mago** (carthage-software/mago, Apache-2.0, attivo): lexer+parser PHP 8.x maturi → si riusa come front-end, eliminando ~25K LOC di scanner re2c + grammatica Bison.
- php-parser-rs archiviato; php.rs hobby-scale; PHPantom solo LSP.

## Cosa Rust elimina di Zend (la semplificazione strutturale)

| Sottosistema Zend | LOC C | Sostituto Rust | LOC Rust stimate |
|---|---|---|---|
| `zend_vm_def.h` + `zend_vm_execute.h` (generato) + `zend_execute.c` | ~146.000 | Evaluator tree-walking su HIR (AST risolto), `match` su enum | 3–5K |
| `zend_compile.c` (AST→opcodes) | 12.400 | Lowering AST→HIR (risoluzione variabili→slot, hoisting funzioni) | 1–2K |
| Lexer re2c + parser Bison + zend_ast | ~25.000 | dipendenza **mago** + bridge | ~500 |
| `zend_alloc.c` (allocator per-request) | 3.600 | allocator di sistema + ownership/Drop | 0 |
| `zend_gc.c` (GC ciclico) | 2.400 | `Rc` + COW; cicli possibili solo con `&$x`/oggetti → differito | 0–300 |
| TSRM (thread safety 2002-style) | 2.000 | `Send`/`Sync` del type system | 0 |
| `Zend/Optimizer` + opcache | ~72.000 | irrilevante: processo residente tiene l'HIR in memoria (l'opcache esiste perché PHP-C ri-parsa a ogni richiesta) | 0 |
| `zend_hash.c` + `zend_string` | 4.500 | `PhpArray` + `PhpStr` propri (semantica osservabile, non layout) | ~800 |
| `zend_operators.c` (type juggling) | 3.900 | **irriducibile**: va portato fedelmente — è l'anima di PHP | ~1.500 |
| win32/ (emulazione POSIX) | 7.500 | `std` cross-platform | 0 |

**~280K LOC del core si riducono a ~8–10K LOC Rust.** L'unico modulo che richiede porting fedele riga-per-riga è `zend_operators.c` (+ formattazione float e messaggi di errore): tutto il resto è sostituito da design moderno o crate.

## Architettura

```
sorgente PHP ──mago──► AST ──lowering──► HIR (AST risolto: slot variabili,
                                          salti risolti, funzioni hoisted, span per line#)
                                              │
                                         Evaluator (tree-walk, match su enum)
                                              │
                              php-types: Zval / PhpStr / PhpArray / operators
                                              │
                              Builtins: trait + registry, implementati su crate Rust
```

Decisioni chiave (D-decisioni per `02-mapping-table.md`):
1. **Zval = enum** (`Null/Bool/Long(i64)/Double(f64)/Str(Rc<PhpStr>)/Array(Rc<PhpArray>)`), COW esatto via `Rc::make_mut` (= `SEPARATE_ARRAY` di Zend). `Undef` per la semantica "undefined variable".
2. **PhpStr = bytes** (`Box<[u8]>`), mai `String`: le stringhe PHP sono binarie (vincolo per il differential).
3. **PhpArray proprio** (no indexmap): ordered hash con chiavi `Int|Str`, canonicalizzazione numeric-string (`ZEND_HANDLE_NUMERIC`), `next_free` che non decresce. ~600 LOC.
4. **Niente bytecode**: HIR tree-walk. Escape hatch futuro se serve performance: bytecode leggero o cranelift, dietro la stessa semantica già validata dai test. La correttezza prima, registrata dalla baseline .phpt.
5. **Riferimenti `&$x`** differiti (= `Rc<RefCell<Zval>>` quando serviranno); senza riferimenti/oggetti il PHP procedurale non può creare cicli → niente GC.
6. **Esecuzione web moderna**: in prospettiva non si porta fpm/cgi — un server residente (axum/hyper) con isolamento per-request (ogni richiesta = contesto nuovo, share-nothing come PHP) sostituisce fpm + opcache *by design*. Fuori scope Tier 1, ma orienta l'architettura (nessun global mutabile nascosto).

## Migrazione estensioni = mappa su crate (non porting)

| Estensione | LOC C (incl. bundle) | Strategia Rust |
|---|---|---|
| ext/standard | 74K | Reimplementazione incrementale guidata dalla frequenza nei test; molte funzioni = 10-50 LOC Rust su std |
| ext/pcre (PCRE2) | 182K | crate `pcre2` (semantica PCRE esatta) o `regex`+`fancy-regex` con fallback — decidere su differential |
| ext/date (timelib) | 112K | `jiff`/`chrono` per il calendario + **scoped port** del solo parser `strtotime`/format (la parte davvero PHP-specifica) |
| ext/json | 4K | port diretto (~800 LOC, semantica PHP specifica) o `serde_json` + wrapper |
| ext/hash | 11K | RustCrypto digest (glue ~100 LOC/algoritmo) |
| ext/mbstring (libmbfl) | ~80K | `encoding_rs` + `icu4x` |
| ext/openssl, sodium | 18K | `rustls` + RustCrypto + `dryoc` |
| ext/curl | 5.5K | `reqwest` |
| sqlite3/pdo, mysqli/mysqlnd, pgsql | ~36K | `rusqlite`, `sqlx`/`mysql_async`, `tokio-postgres` |
| zlib/bz2/zstd, fileinfo, gd, intl | ~70K | `flate2`/`bzip2`/`zstd`, `infer`/`tree_magic_mini`, `image`, `icu4x` |

Il pattern è sempre: **crate maturo per il lavoro pesante + strato sottile di fedeltà PHP** (firme, type juggling degli argomenti, formato output/errori) validato dai .phpt dell'estensione.

## Roadmap a tier

- **Tier 1 (questo piano)**: core procedurale — tipi, operatori, controllo di flusso, array, funzioni utente, builtin essenziali. Oracle: `Zend/tests` (~600-900 eleggibili, target 400-500 verdi).
- **Tier 2**: OOP (classi, interfacce, ereditarietà, exceptions, closures) — necessario per "gran parte del codice PHP" reale. Sblocca migliaia di .phpt.
- **Tier 3**: estensioni core via crate (pcre, date, json, hash, mbstring, ext/standard completo). Oracle: `ext/*/tests`.
- **Tier 4**: SAPI web moderno (server residente axum, superglobals, sessions).

## Tier 1 — 11 step TDD (stima 30–45h, vs 40–60h del piano opcode)

| # | Step | Verifica |
|---|---|---|
| 0 | Scaffolding workspace + diary + CI | `cargo test` verde |
| 1 | `php-types`: ZStr, Zval, PhpArray (chiavi, next_free, ordine, COW) | unit test casi limite |
| 2 | **Operatori/conversioni** (port fedele di `zend_operators.c`: compare :2306, add :1200, concat :2017, identical :2508, smart_streq :3373, increment :2712) | differential ~500 espressioni vs `php -r 'var_dump(...)'` |
| 3 | Bridge mago → HIR (slot variabili, span, hoisting) | smoke test script campione |
| 4 | Evaluator v1: echo, variabili, assegnamenti, if/while/for, ternario, break/continue | differential su corpus script |
| 5 | Builtins registry + nucleo (var_dump formato esatto, strlen, gettype, is_*) + **float formatting** (echo precision=14 vs var_dump shortest-roundtrip) | differential INF/NAN/-0.0/0.1+0.2 |
| 6 | **phpt-runner**: parser sezioni, EXPECTF→regex (regole di run-tests.php), capability-scan automatico (`class\|trait\|yield\|try\|&$` → SKIP motivato), `expectations.toml`, modalità differential vs php di sistema | prima baseline su `Zend/tests/*.phpt` committata |
| 7 | Array end-to-end (accesso/assegnazione dim, isset/empty/unset, nesting COW, var_dump ricorsivo) + foreach + switch/match | `Zend/tests/foreach/`, baseline ↑ |
| 8 | Funzioni utente (hoisting, parametri/default, ricorsione, return) | differential fib + .phpt |
| 9 | **Fedeltà diagnostica**: `Warning: Undefined variable $x in %s on line %d`, `Undefined array key`, DivisionByZeroError — metà degli EXPECTF li contiene | .phpt EXPECTF |
| 10 | Espansione builtin per frequenza nei test (implode, count, substr, sprintf-subset, array_keys/values, in_array, print_r) | baseline ↑ |
| 11 | Chiusura Tier 1: sweep completo, classificazione failure A/B/C/D, D-NEW in `04-divergences.md`, freeze baseline, conclusions | report finale |

Commit per step, entry in `03-translation-log.md`, check-in con l'utente agli step 2, 6, 9, 11.

## Workspace

```
php-rust-experiment/
├── CLAUDE.md, EXPERIMENT_PLAN.md
├── php-rust/crates/
│   ├── php-types      # Zval/PhpStr/PhpArray + operators (zero dep interne)
│   ├── php-runtime    # HIR, lowering da mago, evaluator, errori (unico dep su mago)
│   ├── php-builtins   # trait Builtin + funzioni
│   ├── php-cli        # binario `phpr file.php` / `-r`
│   └── phpt-runner    # harness .phpt + differential
└── diary/             # 00-reconnaissance, 01-semantic-model, 02-mapping-table,
                       # 03-translation-log, 04-divergences, 99-conclusions, metrics
```

`original/` non si copia (1,9M LOC): si referenzia il repo esistente. Fasi metodologiche legacy-port mantenute ma alleggerite: Fase 1 (semantic model, ~3h, solo comportamento osservabile Tier 1, ≥25 citazioni file:line), Fase 2 (mapping table con le D-decisioni sopra), Fase 3 = gli 11 step, Fase 4 integrata (il differential è nativo: i .phpt SONO la testsuite originale, niente converter), Fase 5 sintesi.

## Trade-off dichiarati (onestà del piano)

1. **Performance**: tree-walk < opcode VM Zend in throughput. Accettato: Tier 1 misura correttezza; l'architettura residente recupera il costo di parse/compile che PHP-C paga a ogni richiesta, e l'escape hatch bytecode resta aperto.
2. **PCRE**: `regex` crate non è PCRE; se i differential mostrano divergenze, si usa `pcre2` (binding FFI) — fedeltà > purezza Rust.
3. **strtotime/date parsing**: nessun crate copre la semantica timelib → scoped port dedicato in Tier 3.
4. **Test che osservano internals** (refcount, opcache, gc_collect_cycles) → skip-list motivata, non sono semantica del linguaggio.

## Verifica end-to-end

1. `cargo test` verde a ogni step.
2. `phpt-runner --differential --php $(which php)` su Zend/tests: baseline committata, mai in regressione tra step.
3. Fine Tier 1: sweep + report (verdi/skip/xfail + divergenze D-NEW catalogate).
