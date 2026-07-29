# Ricognizione tecnica di `php-rust`

**Data dell'audit:** 30 luglio 2026  
**Snapshot analizzato:** commit `c03e29c`, branch `main`, worktree pulito  
**Target:** `/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust`  
**Oracle:** PHP 8.5.7, sorgenti PHPT in `/Volumes/Extreme Pro/Claude/php-8.5.7` e binario CLI in `/Users/francescotinti/Claude/php-oracle/php-src/sapi/cli/php`

> Il nome del file segue quello richiesto (`20270615-chatgpt-sol.md`); la ricognizione fotografa lo stato reale del 30 luglio 2026.

## Executive summary

`php-rust` non è più un toy interpreter. È un **runtime sperimentale avanzato**, costruito da zero in Rust, con un risultato di compatibilità già raro: WordPress core gira a parità effettiva con l'oracle, insieme a Composer, PHPUnit, Doctrine, Symfony e MySQL reale. La pipeline `AST Mago → HIR → bytecode → VM`, il principio `correct-or-absent`, il runner PHPT e la disciplina di misurazione prestazionale sono fondamenta tecniche serie.

Il progetto, tuttavia, non è ancora un sostituto general-purpose e production-grade di PHP. La maturità semantica del profilo WordPress è molto superiore alla maturità di prodotto: build e dipendenze non sono ancora riproducibili, la CI è insufficiente e probabilmente non portabile sul runner dichiarato, i gate `rustfmt` e Clippy sono rossi, il confine FFI contiene problemi di soundness da trattare subito, il server è ancora un SAPI di sviluppo e il codice caldo è concentrato in pochi file molto grandi.

Il giudizio sintetico è:

- **maturità funzionale del profilo WordPress:** alta, circa **8,5/10**;
- **maturità del core linguistico:** medio-alta, circa **7,5/10**;
- **maturità come runtime PHP general-purpose:** media, circa **5,5/10**;
- **maturità production/operativa:** bassa, circa **3,5/10**;
- **maturità complessiva del progetto oggi:** **beta di ricerca / pre-alpha di prodotto**.

La priorità non dovrebbe essere aggiungere indiscriminatamente altre estensioni. Il miglior investimento adesso è consolidare una **baseline riproducibile, sicura e continuamente verificata**, mantenendo in parallelo l'attuale lavoro misurato su CPU e memoria.

## Evidenze principali

| Area | Evidenza osservata | Lettura |
|---|---:|---|
| Compatibilità funzioni | 1.017 / 2.143 funzioni interne (47%) | Superficie generale ancora incompleta |
| Core/stdlib/date | 539 / 654 (82%) | Copertura molto buona per applicazioni reali |
| Corpus Zend | 2.649 pass, 1.418 fail, 1.238 skip su 5.305 | 65,1% dei runnable; 49,9% del totale |
| WordPress | 30.472 test single-site e 31.278 multisite, un solo diff dichiarato per nome | Evidenza end-to-end eccezionalmente forte |
| Test Cargo | 1.651 pass, 1 ignored, 0 fail | Gate funzionale locale verde |
| Rustfmt | `cargo fmt --all -- --check` fallisce con oltre 1 MB di diff | Nessuna baseline formale di stile |
| Clippy | fallisce già in `php-types` con 20 errori | Debito di qualità e almeno un rischio FFI reale |
| Warning build | warning in più crate; un `Result` ignorato in `net.rs` | I warning non sono trattati come regressioni |
| Dimensione codice Rust | circa 135.800 LOC tra sorgenti e test | Progetto ormai grande, serve governance architetturale |
| Hotspot | `vm/mod.rs` 22.958 LOC, `vm/host.rs` 7.594, `vm/run.rs` 5.550 | Concentrazione e costo cognitivo elevati |
| Unsafe | 198 blocchi `unsafe`, 27 `unsafe fn`, 2 `static mut` | Naturale per le FFI, ma richiede audit esplicito |
| Dipendenze risolte | 349 package nel `Cargo.lock` locale | Supply chain non banale |
| Riproducibilità | `Cargo.lock` esiste ma è ignorato e non tracciato | Due build possono risolvere grafi diversi |
| Portabilità | build script con path Homebrew hard-coded e dylib di sistema | Build legata soprattutto al Mac dell'autore |
| CI | un solo job Ubuntu con build/test | Non esegue fmt, Clippy, oracle, PHPT, audit o benchmark |
| Governance | 1.184 commit dal 13 giugno, 218 negli ultimi 7 giorni, un solo autore | Velocità straordinaria, bus factor e review risk alti |
| Distribuzione | nessun tag, release, `LICENSE`, `SECURITY.md` o toolchain pin | Non ancora pronto per utenti esterni |

### Nota sui test differenziali

I tre test Rust denominati “differential” ritornano con successo quando non trovano l'oracle, quindi un normale `cargo test` li mostra come `ok` anche se non hanno confrontato nulla. In questa ricognizione sono stati rieseguiti impostando esplicitamente:

```sh
PHP_ORACLE=/Users/francescotinti/Claude/php-oracle/php-src/sapi/cli/php
```

e sono risultati verdi. Il comportamento predefinito rimane però un **falso verde strutturale** e va corretto.

## Cosa è già best-in-class

### 1. Strategia di compatibilità

La scelta di usare PHP 8.5.7 come oracle osservabile, senza copiare l'architettura Zend, è corretta. I PHPT definiscono contratti su output, warning, errori, timing dei distruttori e casi limite che una specifica narrativa non cattura. `correct-or-absent` evita il rischio più pericoloso in un runtime compatibile: risultati plausibili ma sbagliati.

### 2. Validazione con applicazioni reali

WordPress, Symfony, Composer, PHPUnit e Doctrine coprono interazioni tra parser, autoload, reflection, riferimenti, eccezioni, filesystem, HTTP, database e lifecycle. Sono prove molto più significative di migliaia di micro-test isolati. La parità della suite WordPress single-site e multisite è il principale asset del progetto.

### 3. Pipeline del motore

La separazione concettuale sorgente → AST → HIR → bytecode → VM è sana. Il runtime non dipende direttamente dalle strutture arena-bound del parser e dispone di un unico engine di produzione, evitando la deriva tra tree-walker e VM.

### 4. Disciplina di performance

Le misure riportate non si limitano al wall-clock: attribuzione CPU, memoria live/reached, censimenti, A/B, contatori del GC e conservazione dei risultati indicano un metodo maturo. Il passaggio da circa 3,4× a circa 2,06–2,11× CPU sulla suite WordPress e da 11,9× a circa 4,07× di peak footprint è sostanziale.

### 5. Isolamento del corpus

L'esecuzione isolata dei PHPT limita il blast radius di panic, hang e crash. Per un interprete che esegue input ostile o semplicemente bizzarro è una scelta fondamentale.

## Criticità prioritarie

## P0 — Da affrontare prima di dichiarare il progetto distribuibile

### P0.1 Riproducibilità e CI

Il workspace produce binari, quindi `Cargo.lock` deve essere committato. Oggi è ignorato dalla `.gitignore`; 349 package possono cambiare senza una modifica al repository. Manca inoltre un `rust-toolchain.toml`, mentre Clippy osservato usa Rust 1.96.0: nuovi lint o cambi di compilatore possono rompere il gate da un giorno all'altro.

La `.cargo/config.toml` committata contiene:

```toml
[build]
target-dir = "/Users/francescotinti/Claude/php-rust-output"
```

Questo path personale non deve stare nella configurazione condivisa: su Linux CI è inesistente e probabilmente non scrivibile. Va spostato in una variabile d'ambiente locale, in una configurazione non tracciata o in un wrapper.

Inoltre `php-types/build.rs` collega incondizionatamente:

- `/opt/homebrew/opt/gd/lib`;
- `/opt/homebrew/opt/tidy-html5/lib`;
- `gd`, `tidy`, `xslt`, `exslt`, `xml2`.

La CI Ubuntu installa soltanto zlib e `pkg-config`. Ne segue che il workflow non rappresenta una build pulita e portabile del progetto attuale.

**Azioni:**

1. committare `Cargo.lock`;
2. introdurre `rust-toolchain.toml`;
3. rimuovere il path assoluto dalla config tracciata;
4. usare `pkg-config`/`vcpkg` o probe equivalenti nel build script;
5. rendere le estensioni FFI feature-gated (`ext-gd`, `ext-tidy`, `ext-xsl`);
6. creare una matrice almeno macOS + Ubuntu;
7. aggiungere un job “clean clone” che costruisca senza stato locale;
8. pin delle GitHub Actions a SHA, non solo a tag mobili.

### P0.2 Soundness del confine FFI

Clippy segnala funzioni pubbliche safe in `gdio.rs` che ricevono raw pointer e li dereferenziano internamente. Una funzione safe deve poter essere chiamata senza precondizioni non verificabili; altrimenti deve essere `unsafe fn` con una sezione `# Safety`, oppure deve esporre un wrapper sicuro basato su `NonNull`, lifetime e ownership chiari.

I 198 blocchi `unsafe` non sono automaticamente un problema, ma oggi non esiste un gate dedicato che dimostri la qualità del confine. Le FFI verso gd, tidy, libxml/libxslt, zlib e libc sono una parte trusted della sicurezza dell'interprete.

**Azioni:**

- `#![deny(unsafe_op_in_unsafe_fn)]` nei crate FFI;
- wrapper safe minimi e tipi newtype per ogni handle;
- documentare ownership, nullability, thread affinity e regole di free;
- eliminare o giustificare i due `static mut`;
- test ASan/UBSan su Linux/macOS e test Valgrind dove applicabile;
- Miri sui moduli Rust puri;
- audit manuale per callback C, varargs, panic che attraversano FFI e lifetime delle stringhe;
- nessun panic deve attraversare un boundary `extern "C"`.

### P0.3 Definire il perimetro production

`phpr -S` replica volutamente il server CLI di PHP ed è sequenziale: è un ottimo oracle target e un development server, non un application server Internet-facing. Prima di parlare di produzione servono:

- threat model;
- limiti di memoria, istruzioni, recursion depth, body/header, upload e decompressione;
- timeout reali e cancellazione;
- isolamento di una request che panic/crasha;
- verifica di path traversal, symlink escape, request smuggling e header injection;
- lifecycle/reset completo della VM tra richieste;
- policy TLS/reverse proxy;
- soak test di ore o giorni con RSS stabile.

Fino ad allora la documentazione deve etichettare chiaramente `php-server` come **development/compatibility server**.

### P0.4 Hashing e input non fidato

`rustc-hash` è dichiarato non DoS-hardened ed è usato anche nell'indice di `PhpArray`. Le chiavi degli array PHP possono provenire direttamente da query string, JSON, form e applicazioni remote: non sono tutte “engine internal”.

Va definito un modello esplicito:

- hasher resistente per tabelle raggiungibili da input remoto; oppure
- hash Zend-compatible con mitigazione delle collisioni, limite di catena/probe e fallback sicuro;
- benchmark che quantifichi il costo della protezione;
- test avversariali su collisioni e crescita.

Non è accettabile guadagnare qualche punto percentuale medio introducendo un worst-case controllabile dall'attaccante.

## P1 — Alta priorità nel breve periodo

### P1.1 Rendere i gate di qualità verdi

`rustfmt` non è stato applicato sistematicamente. Conviene fare **un unico commit meccanico**, separato da cambi semantici, poi rendere obbligatorio `cargo fmt --check`.

Clippy va introdotto in due passaggi:

1. correggere subito lint di correctness/soundness e warning del compilatore;
2. per lint di stile o complessità, correggere oppure aggiungere `#[expect(...)]` locale con motivazione.

Il target è:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --release --workspace
```

Il `Result` ignorato in `php-builtins/src/net.rs` deve essere gestito esplicitamente. Anche se l'errore sembra impossibile, l'invariante va codificata e spiegata.

### P1.2 Evitare test “verdi senza test”

Se un test richiede l'oracle, l'assenza dell'oracle deve:

- produrre un vero `ignored` esplicito in sviluppo; oppure
- fallire in un job CI chiamato `oracle-required`.

Mai `return` da un test che Cargo contabilizza come passato. Il runner dovrebbe verificare anche `PHP_VERSION_ID == 80507`, configurazione NTS/ZTS rilevante e hash/versione dell'oracle.

### P1.3 Automatizzare i gate PHPT e applicativi

Il corpus Zend e WordPress sono il vero capitale del progetto ma girano solo localmente. Serve una pipeline a livelli:

- PR: unit/integration, piccoli smoke PHPT deterministici, oracle micro-differential;
- merge su `main`: Zend PHPT completo con baseline per nome;
- nightly/self-hosted: WordPress single-site/multisite, Symfony, Doctrine;
- settimanale: performance, soak e sanitizer.

La baseline deve essere un artefatto versionato, non solo un numero:

```text
test-id | status | categoria | hash output | divergenza autorizzata | issue
```

Il gate fondamentale è `pass→fail = 0`, ma anche `skip` nuovi o cambi di categoria devono richiedere approvazione.

### P1.4 Metriche di copertura non ambigue

“65,1% dei runnable” è corretto, ma da solo può nascondere l'effetto dei 1.238 skip. Pubblicare sempre insieme:

- pass/totale: 2.649/5.305 = 49,9%;
- pass/runnable: 65,1%;
- fail: 1.418;
- skip: 1.238, suddivisi per motivo;
- divergenze deliberate;
- test non eseguibili per limiti del runner.

`COVERAGE.md` misura copertura di API e corpus, non code coverage Rust. Aggiungere separatamente line/branch coverage con `cargo llvm-cov`, senza trasformarla in un obiettivo cieco.

### P1.5 Licensing e sicurezza del progetto

I manifest dichiarano MIT, ma non esiste un file `LICENSE`. Mancano anche `SECURITY.md`, una policy di disclosure, una policy delle dipendenze e una SBOM.

Prima di una release pubblica:

- aggiungere il testo MIT e verificare compatibilità delle librerie collegate;
- inventario delle licenze con `cargo-deny`;
- `cargo audit`/RustSec automatico;
- SBOM CycloneDX o SPDX;
- `SECURITY.md`;
- documentazione chiara: progetto non affiliato a PHP, compatibilità e limitazioni.

## P2 — Modularità, chiarezza e manutenibilità

### P2.1 Ridurre i megafile senza riscrittura

`vm/mod.rs` contiene circa 16.500 righe di produzione prima del grande blocco test, più migliaia di righe di test; `host.rs` supera 7.500 righe. La suddivisione esistente (`arrays`, `calls`, `oop`, `session`, `dom`, ecc.) dimostra già la direzione giusta.

Usare uno **strangler refactor**:

1. spostare i test in `vm/tests/` o moduli `#[cfg(test)]` dedicati;
2. estrarre stato VM, lifecycle request, unit cache, GC, symbol tables e dispatch host;
3. ogni estrazione deve essere un commit solo-move, verificato con `git diff --color-moved`;
4. nessuna ottimizzazione nello stesso commit;
5. gate PHPT per nome dopo ogni tranche.

Non serve creare subito molti nuovi crate. Prima vanno ottenuti confini interni stabili; solo dopo si decide quali meritano un crate.

### P2.2 Registro dichiarativo delle builtin

Oggi ci sono circa 535 registrazioni `add(...)` e centinaia di dispatch per nome. Una tabella dichiarativa o macro potrebbe essere la singola fonte per:

- nome e alias;
- estensione;
- arity;
- parametri by-ref/variadic;
- coercion/ZPP;
- disponibilità per piattaforma/feature;
- metadata Reflection;
- funzione host/value;
- test minimi richiesti.

Da questa fonte si possono generare registry, `get_defined_functions`, reflection, report coverage e controlli duplicati. Questo elimina drift e rende più economica ogni nuova estensione.

### P2.3 Separare il core dai binding di estensione

`php-types`, il crate più basso, collega oggi gd/tidy/xslt. In prospettiva il value model non dovrebbe conoscere tutte le librerie native. Proporrei:

```text
php-types          valori, array, stringhe, object handle, stream traits
php-engine         HIR, bytecode, VM, lifecycle
php-extension-api  registry, native handle, per-request state, error bridge
php-ext-*          gd, tidy, xsl, mysqli, pdo, ...
php-sapi-*         cli, cli-server, futuro server production
```

La migrazione deve essere incrementale. Il primo obiettivo non è la purezza teorica, ma poter disabilitare un'estensione e costruire un core minimale senza le sue dylib.

### P2.4 Gestione degli errori e panic policy

221 `unwrap()`, 399 `expect()` e 115 `panic!()` includono test e invarianti legittime, ma in un interprete gli input utente non devono raggiungere panic host.

Classificare:

- invariant panic: mantenere con messaggio e test;
- errore PHP: trasformare in `PhpError`/diagnostica compatibile;
- errore I/O/FFI: propagare senza perdere contesto;
- OOM/abort: documentare.

Aggiungere fuzz target che considera qualsiasi panic, stack overflow o hang un bug, anche quando l'output PHP non è ancora compatibile.

## Performance

### Cosa mantenere

- benchmark contro lo stesso oracle e la stessa macchina;
- misure CPU e footprint, non solo wall time;
- raw counters e attribuzione per owner;
- nessun cambiamento mantenuto senza parità;
- focus sul workload WordPress reale.

### Cosa migliorare

1. Versionare harness, fixture, commit oracle, configurazione MySQL e comandi.
2. Salvare dati grezzi in JSON/CSV e grafici temporali, non solo sintesi nei documenti.
3. Usare più ripetizioni, warmup, intervalli di confidenza e soglie anti-rumore.
4. Separare startup, compile/cache hit, execution, GC e teardown.
5. Aggiungere microbenchmark Criterion solo per primitive stabili; la decisione finale resta applicativa.
6. Misurare latenza p50/p95/p99 e throughput quando esisterà un server concorrente.
7. Provare, dopo la stabilizzazione, ThinLTO, `codegen-units=1` e PGO; BOLT per build Linux. Ogni opzione va A/B, non assunta.
8. Continuare l'attribuzione del footprint “fuori canale”: allocator metadata, unit/prelude retention, descriptor e indici.

Il gap attuale di circa 2,06–2,11× CPU e 4,07× peak memory è promettente ma ancora grande per un drop-in runtime. Per il primo target serio proporrei:

- CPU WordPress suite ≤ 1,5× oracle;
- peak RSS ≤ 2×;
- nessuna regressione > 3% senza decisione documentata;
- stabilità su almeno 30 run e su due macchine.

Non introdurrei ora `Arc<Mutex>` o una VM async generale: il design `Rc<RefCell>` favorisce un modello thread-per-core. Prima servono limiti, reset request, preload e un server robusto; l'async I/O viene dopo l'esistenza di I/O che lo giustifica.

## Sicurezza e robustezza

Il runtime esegue codice e input potenzialmente ostili. La sicurezza deve diventare un asse autonomo, non un effetto collaterale della compatibilità.

### Programma minimo

- fuzz lexer/lowering/compiler/VM con `cargo-fuzz`;
- differential fuzzing su programmi piccoli e deterministici;
- corpus permanente per ogni crash/hang;
- limiti su regex, XML entity expansion, archive bombs, decompressione, immagini e recursion;
- memory limit e instruction budget enforceable;
- ASan/UBSan per FFI, Miri per core puro;
- audit delle dipendenze e advisory gate;
- test di concorrenza/lifecycle su handle nativi;
- secret-independent timing per crypto confrontabile;
- hardening server e upload;
- zero unsafe pubblico non documentato.

I binding alla stessa libreria usata dall'oracle aiutano la parità, ma ereditano anche ABI, CVE e differenze di packaging. Le versioni effettivamente caricate devono essere registrabili (`phpr --diagnostics`), e la compatibilità va definita per una matrice di versioni supportate.

## Documentazione, praticità e governance

La documentazione tecnica è ricca, ma `README.md` è diventato anche un diario prestazionale molto lungo. Per utenti e contributor conviene separare:

- README stabile: cosa è, cosa supporta, quick start, limiti;
- `STATUS.md`: matrice corrente;
- `BENCHMARKS.md`: metodologia e trend;
- changelog/release notes;
- ADR per decisioni architetturali;
- documenti di sessione in archivio.

`PHPR_DIVERGENCES_FROM_PHP.md` ha un'intestazione “ultimo aggiornamento” più vecchia di molte entry successive: aggiungere una validazione automatica della metadata date o rimuovere la data manuale.

La velocità di 1.184 commit in circa sette settimane, con un solo autore, è sia un vantaggio sia un rischio. Per diventare best-in-class servono:

- issue tracker con priorità e acceptance criteria;
- PR anche per il maintainer principale, almeno per cambi FFI/GC/value model;
- review esterna periodica;
- `CODEOWNERS` quando entrano collaboratori;
- changelog e release semantiche;
- almeno una seconda persona in grado di costruire, testare e rilasciare.

## Criticità per orizzonte temporale

### Attuali

- build non riproducibile per `Cargo.lock` ignorato;
- configurazione Cargo personale committata;
- CI non rappresentativa delle dylib collegate;
- rustfmt/Clippy rossi;
- wrapper FFI safe con raw pointer dereferenziati;
- test differential che può essere un falso verde;
- assenza di license/security/release policy;
- server da sviluppo potenzialmente confondibile con server production.

### Breve periodo (0–3 mesi)

- regressioni favorite dall'altissima velocità e dai gate locali manuali;
- drift tra documentazione, coverage e codice;
- break improvviso da toolchain o dipendenze floating;
- bug di lifecycle FFI/GC difficili da diagnosticare;
- aumento ulteriore dei megafile;
- metriche PHPT percepite come più complete di quanto siano per effetto degli skip.

### Medio periodo (3–12 mesi)

- costo crescente di ogni estensione per registry/dispatch duplicati;
- portabilità bloccata dalle librerie Homebrew e da API Unix;
- superficie di attacco del server e delle estensioni I/O;
- collision DoS o exhaustion su workload remoti;
- impossibilità di mantenere contemporaneamente PHP 8.5 e nuove versioni senza una strategia di versioning;
- footprint ancora troppo alto per processi long-running e alta densità.

### Lungo periodo (oltre 12 mesi)

- bus factor uno;
- dipendenza da un singolo oracle/build environment;
- incompatibilità dell'ecosistema che richiede estensioni non portate o ABI Zend;
- frammentazione tra “compatibilità PHP” e nuove feature async/distribuite;
- manutenzione CVE delle librerie native;
- rischio di inseguire il 100% nominale sacrificando profili di compatibilità realmente utili.

## Roadmap consigliata

### Fase 0 — Baseline affidabile (1–2 settimane)

- commit `Cargo.lock`;
- pin toolchain;
- rimuovere target-dir assoluta;
- rendere verde rustfmt;
- eliminare warning e Clippy correctness;
- correggere API unsafe FFI;
- aggiungere `LICENSE` e `SECURITY.md`;
- definire profili `core`, `wordpress`, `full`;
- creare manifest macchina-legibile delle baseline.

**Exit criterion:** clone pulito, build e test verdi su Mac e Ubuntu con comandi documentati.

### Fase 1 — CI e sicurezza (2–6 settimane)

- matrice CI;
- oracle-required micro differential;
- PHPT completo nightly;
- RustSec/cargo-deny/SBOM;
- sanitizer e primi fuzz target;
- resource limits minimi;
- feature-gating delle estensioni native.

**Exit criterion:** nessun gate critico dipende solo dal computer dell'autore.

### Fase 2 — Modularità e operabilità (1–3 mesi)

- split meccanico dei megafile;
- registry dichiarativo;
- extension API/handle lifecycle;
- telemetry strutturata;
- benchmark harness versionato;
- pacchetto diagnostico `phpr --version --diagnostics`;
- release candidate del profilo WordPress.

**Exit criterion:** un contributor esterno può aggiungere una builtin/estensione senza modificare più dispatch manuali e senza conoscere tutto `vm/mod.rs`.

### Fase 3 — Performance e beta controllata (3–6 mesi)

- continuare heap-to-handle e attribuzione memoria;
- PGO/LTO misurati;
- soak test server;
- suite framework automatizzate;
- artefatti firmati macOS/Linux e checksum;
- documentazione installazione/disinstallazione.

**Exit criterion:** beta dichiarata con SLO, matrice supportata e limiti noti.

### Fase 4 — Server production e futuro (6–12+ mesi)

- server multi-worker thread-per-core o integrazione reverse proxy/FastCGI;
- reset/pool VM dimostrato;
- cancellazione e timeout;
- eventuale async I/O tramite sospensione esterna dei fiber;
- strategia PHP 8.6/9 e corpus versionato.

## Definition of “best in class”

Il progetto potrà ragionevolmente definirsi best-in-class quando soddisferà contemporaneamente:

1. **Correctness:** baseline per nome, zero regressioni, divergenze esplicite.
2. **Reproducibility:** lockfile, toolchain, clean build e artifact provenance.
3. **Safety:** FFI audited, sanitizer/fuzzing, nessun panic da input.
4. **Security:** threat model, limiti, advisory gate e disclosure policy.
5. **Performance:** metodologia ripetibile e target quantitativi raggiunti.
6. **Modularity:** extension API e ownership chiara dei sottosistemi.
7. **Portability:** almeno macOS e Linux realmente supportati.
8. **Operability:** logging, diagnostics, crash report, metriche e soak test.
9. **Usability:** installazione semplice, profili supportati e messaggi chiari.
10. **Governance:** release, review e bus factor superiore a uno.

## Le dieci azioni con il miglior rapporto valore/costo

1. Committare `Cargo.lock`.
2. Togliere il path assoluto da `.cargo/config.toml`.
3. Far fallire davvero i test differential senza oracle.
4. Un commit solo-format e gate rustfmt permanente.
5. Correggere i wrapper raw-pointer segnalati da Clippy.
6. Rendere verde una policy Clippy concordata.
7. Sistemare CI macOS/Ubuntu con librerie native rilevate correttamente.
8. Versionare l'elenco dei risultati PHPT per nome e categoria.
9. Aggiungere license, security policy, cargo-audit/deny e SBOM.
10. Estrarre i test da `vm/mod.rs` e iniziare lo split meccanico del runtime.

## Conclusione

Il progetto ha superato la fase in cui la domanda principale è “può funzionare?”. WordPress a parità effettiva dimostra che **può**. La domanda ora è “può essere costruito, verificato, mantenuto e usato in sicurezza da persone diverse dall'autore, su macchine diverse, per mesi?”.

La risposta oggi è: non ancora, ma la distanza è colmabile. Le fondamenta semantiche e la disciplina sperimentale sono più forti di quelle di molti progetti molto più maturi. Per fare il salto di classe occorre dedicare un ciclo esplicito al prodotto: riproducibilità, CI, soundness FFI, sicurezza, modularità e release engineering. Se queste priorità vengono trattate come gate e non come backlog cosmetico, `php-rust` può diventare non soltanto un notevole esperimento, ma un runtime alternativo credibile e tecnicamente distintivo.
