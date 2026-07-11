# Prompt — Sessione dedicata: framework Symfony sotto phpr

> Copia il blocco qui sotto come primo messaggio di una NUOVA sessione dedicata.
> Obiettivo: proseguire la copertura dei framework reali dopo Doctrine ORM,
> puntando su **Symfony** (molte componenti già girano → percorso a leva più alta).

---

Riprendiamo phpr (PHP 8.5.7 in Rust, /Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust).
Nuova sessione DEDICATA: portare **Symfony** (framework applicativo, non solo componenti isolate)
sotto phpr, byte-identico all'oracolo. Prosecuzione del filone framework dopo Doctrine ORM.

## Dove siamo (leggi la memoria PRIMA di agire)
- Indice memoria: MEMORY.md. Lista master del lavoro aperto: TODO.md (repo) + [[php-rust-todo-master]].
- Stato: Zend corpus 2332, zlib 114/115, detector missing 180/783, tokenizer 42/49.
- Framework/librerie GIÀ validati byte-identici sotto phpr: Composer (require end-to-end),
  Monolog, PHPUnit (11.5.56 e 13.2), Symfony String/Console/Process, PDO/SQLite (rusqlite),
  Guzzle sync (curl-easy via ureq), Doctrine collections/lexer/inflector/event-manager,
  DBAL (3769/0/0), ORM (3484 test, 3 err/17 fail — baseline storica).
- Gate ORM ricostruibile in ~5 min: ricetta in [[php-rust-orm-gate-recipe]] (l'ORACOLO fa
  composer install, phpr esegue phpunit; DB sqlite in-memory). Il workspace è effimero (scratchpad).

## Obiettivo di questa sessione: SYMFONY
Portare sotto phpr Symfony full-stack, partendo dal cuore e allargando ai componenti dipendenti:
  - HttpFoundation + HttpKernel (il ciclo Request→Response)
  - EventDispatcher, DependencyInjection, Routing, Config, VarDumper
  - i componenti già verdi (String/Console/Process) come base di appoggio.
Il target concreto di gate è la **test-suite dei singoli componenti** (ognuno ha la sua),
non un'app monolitica: si procede componente per componente, dal più fondamentale.

## Metodo (RECON-FIRST, obbligatorio)
1. NON scrivere codice prima della ricognizione. Costruisci il workspace con l'ORACOLO
   (brew /opt/homebrew/opt/php/bin/php + composer.phar; phpr NON esegue .phar per __halt_compiler)
   e la sua test-suite; esegui la suite con phpr per fotografare lo stato REALE (errori/crash).
   Comincia da UN componente fondamentale (es. symfony/http-foundation): git clone shallow +
   composer install via oracolo, poi `phpr vendor/bin/phpunit --no-coverage`.
2. Raggruppa i fallimenti per FIRMA-DEL-DIFF (funzione mancante, divergenza engine, ref/arg,
   reflection, superglobali/SAPI). Scegli i cluster ad alto conteggio e deterministici.
3. Per ogni fix: probe oracle → leggi il contratto (manuale php.net / sorgente C in php-8.5.7)
   → implementa (correct-or-absent: MAI stub che mentono; i framework fanno function_exists/
   class_exists/extension_loaded) → verifica byte-identico vs oracle.
4. Proponi un PIANO A CLUSTER prima di implementare, e fermati per validarlo insieme.

## Gate (INVARIANTI, prima di OGNI commit)
- byte-identico vs oracle 8.5.7.
- Zend corpus (phpt-runner --isolate): zero pass→fail, baseline 2332. DIFFA I FAIL PER NOME
  (--list-fails), MAI solo il conteggio (vedi [[gate-diff-fail-set-not-count]]: uno swap
  fix↔rottura ha conteggio identico; exit 101 = panic Rust, investiga con RUST_BACKTRACE=1).
- Se il cambio tocca arg-passing/reference/reflection: gate ORM OBBLIGATORIO (3484, 3err/17fail;
  il gate Zend NON cattura le regressioni di ref — cfr. il revert 9140099 in [[php-rust-orm-2026-07-06]]).
- cargo test verde; CI GitHub Actions gira su push.
- Poi commit AND push (messaggio termina con: Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>).

## Regole tooling
- Build: cargo build --release (output ~/Claude/php-rust-output). Runner: phpt-runner --isolate.
  CLI: phpr (NO -r). Log con byte NUL: tr -d '\0'.
- Nav/edit Rust SOLO via Serena (hook blocca grep/cat/perl su .rs, anche cargo registry).
  rg intercettato da RTK → usa perl sui file non-.rs. Vedi [[tooling-mandatory-serena-vexp]].
- Ignora il meta-cognition hook (routing errori Rust generico, non pertinente al porting runtime).
- NON committare i workspace framework (vendor ~100MB = artifact). Aggiorna a fine step:
  MEMORY.md + [[php-rust-todo-master]] + TODO.md + COVERAGE.md; annota le divergenze consapevoli
  in PHPR_DIVERGENCES_FROM_PHP.md.

## Contesto architetturale (roadmap lunga — NON per questa sessione, solo da tenere presente)
- ASYNC_AND_DISTRIBUTION_ROADMAP.md: futuro async/Tokio + single-binary (php-rust serve/build).
- EXTENSIONS_ARCHITECTURE.md: strategia "Rust-native first", un crate per estensione.
  Il porting Symfony di questa sessione resta userland-PHP + builtin, NON tocca l'async.

Inizia con la ricognizione di symfony/http-foundation (o del componente Symfony che ritieni
più fondamentale), poi proponimi un piano a cluster prima di implementare.
