# php-rust-experiment

Reimplementazione moderna di PHP 8.5 in Rust, guidata dal comportamento osservabile
(oracle: i 21.548 test .phpt del sorgente ufficiale), NON porting dell'architettura Zend.

## Riferimenti

- Sorgente C originale (snapshot, NON copiato qui): `/Volumes/Extreme Pro/Claude/php-8.5.7`
- Piano approvato: `/Users/francescotinti/.claude/plans/adesso-un-task-di-happy-tower.md`
- Metodologia: skill `legacy-port` (adattata: reimplementazione spec-driven, non traduzione)

## Convenzioni

- Lingua diario: italiano. Lingua codice e commenti: inglese.
- Branch: `main`. Conventional commits in inglese (`feat:`, `docs:`, `test:`, `chore:`).
- Un commit per step di Fase 3, mai con build red (`cargo test` verde prima di ogni commit).
- Ogni file di diary dichiara "Generato con assistenza AI (Claude Fable 5)".
- Le stringhe PHP sono byte (`[u8]`), MAI `String`/UTF-8.
- Baseline .phpt committata: non deve mai regredire tra step.

## Struttura

- `php-rust/` — workspace Cargo (crates: php-types, php-runtime, php-builtins, php-cli, phpt-runner)
- `diary/` — 00-reconnaissance, 01-semantic-model, 02-mapping-table, 03-translation-log,
  04-divergences, 99-conclusions, metrics

## Comandi

- Test: `cd php-rust && cargo test`
- CLI: `cargo run -p php-cli -- script.php` (binario `phpr`, php drop-in)
- Runner .phpt: `cargo run -p phpt-runner -- <dir o file .phpt>` (`--isolate`, `--list-fails`)
- Trace diagnostico: `PHP_RUST_TRACE=hir|body|exec|all` su stderr (lowering vs evaluation)

> Build: il `target-dir` di cargo è dirottato fuori dal volume "Extreme Pro" via
> `php-rust/.cargo/config.toml` (il filesystem esterno rompe la cache incrementale).
> Evaluator in `php-runtime/src/eval/{mod,expr,stmt,calls,class,builtins}.rs`,
> lowering in `lower/{mod,stmt,class,expr}.rs`.
