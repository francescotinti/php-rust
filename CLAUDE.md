# php-rust-experiment

Reimplementazione moderna di PHP 8.5 in Rust, guidata dal comportamento osservabile
(oracle: i 21.548 test .phpt del sorgente ufficiale), NON porting dell'architettura Zend.

## Riferimenti

- Sorgente C originale (snapshot, NON copiato qui): `/Volumes/Extreme Pro/Claude/php-8.5.7`
- Mappa delle fonti e ordine di lettura obbligatorio: [AGENTS.md](AGENTS.md)
  (→ `php-rust/REGOLE.md` processo → `php-rust/NEXT_SESSION_WORDPRESS.md` stato)
- Metodologia: skill `legacy-port` (adattata: reimplementazione spec-driven, non traduzione)

## Convenzioni

- Lingua diario: italiano. Lingua codice e commenti: inglese.
- Branch: `main`. Conventional commits in inglese (`feat:`, `docs:`, `test:`, `chore:`).
- Processo (misura, pin, gate, rotazione): SOLO `php-rust/REGOLE.md`. Commit+push a ogni
  passo, mai con build red. File `.rs` mai nei comandi git: `git add -u` + `git commit -F`.
- Ogni file di diary dichiara "Generato con assistenza AI (Claude Fable 5)".
- Le stringhe PHP sono byte (`[u8]`), MAI `String`/UTF-8.
- Baseline .phpt committata: non deve mai regredire tra step.

## Struttura

- `php-rust/` — workspace Cargo (sei crate in `crates/`, mappa in AGENTS.md); harness per
  sessione in `php-rust/wp<N>-harness/`, verbali in `php-rust/sessions/`
- `diary/` — 00-reconnaissance … 04-divergences, metrics, NEXT-*.md (backlog per area)

## Comandi

- Test: `cd php-rust && cargo test --release` (SEMPRE `--release`: il profilo
  debug rigenera ~3,8G di artefatti)
- Build di sviluppo: `cargo build --profile dev-release` (incrementale; binario in
  `~/Claude/phpr-target/dev-output/dev-release/phpr`, mai pin né misura). Pin: SOLO
  `scripts/pin-phpr.sh` / `scripts/pin-server.sh` (ricetta `cargo build --release` con target
  canonica esplicita); la CI usa la stessa ricetta.
- CLI: `cargo run -p php-cli -- script.php` (binario `phpr`, php drop-in)
- Runner .phpt: `cargo run -p phpt-runner -- <dir o file .phpt>` (`--isolate`, `--list-fails`)
- Logging: `PHPR_LOG=debug|trace` (stderr), `PHPR_LOG_FILE=<path>`,
  `PHPR_LOG_CONFIG=<log4rs.yaml>` (vedi `php-runtime/src/logging.rs`)
- Run lunghe (build, A/B, catene): SEMPRE detached con
  `perl "/Volumes/Extreme Pro/Claude/wp58-harness/daemonize.pl" <log> <cmd…>` (vive FUORI
  dal repo); esiti solo da file `.done`/`.rc`; dopo il lancio leggere il log del daemonizer
  (gli errori di exec finiscono lì) e `pgrep`.

> **Build / filesystem:** MAI build sul volume esterno "Extreme Pro" (niente hard link,
> cache cargo rotta): target canonica `~/Claude/php-rust-output` (pin) e di sviluppo
> `~/Claude/phpr-target/dev-output` (sparsebundle da montare) sul volume principale; regole
> complete in `php-rust/CLAUDE.md`.
> Engine: VM a bytecode unico (pipeline mago AST→HIR→bytecode→VM); il vecchio
> tree-walker `eval/` è stato eliminato. Lowering in `php-runtime/src/lower/`,
> VM in `php-runtime/src/vm/` (mod.rs ~26k righe, loop caldo in `run.rs` ~7,5k con
> cap LOC dichiarato in `tests/loc_dente.rs` — usare Serena). L'hook blocca OGNI segmento
> Bash con grep/sed/awk/cat/head/tail che citi `crates/` o un token `.rs`, anche dentro
> stringhe o heredoc: gli script che nominano file `.rs` si scrivono col tool Write.
