# AGENTS.md — guida operativa per phpr (mappa delle fonti, non un emendamento)

Redatta da ChatGPT Astra 6 il 20 settembre 2026 (ricognizione a `a1eb40e8`),
ratificata e accorciata in S-180. Regola d'oro del progetto: **una fonte sola
per ogni fatto**. Questo file dice DOVE sta la verità, mai la duplica.

## Che cos'è

Reimplementazione in Rust del comportamento osservabile di **PHP 8.5.7**. Non è
una traduzione di Zend: il contratto lo definiscono l'oracle eseguibile
(`/opt/homebrew/opt/php/bin/php`), i `.phpt` ufficiali
(`/Volumes/Extreme Pro/Claude/php-8.5.7`) e le applicazioni reali (WordPress,
Doctrine ORM/DBAL, Symfony, PHPUnit). Diario e documenti in italiano; codice,
commenti e messaggi di commit in inglese. Non attribuire documenti a modelli che
non li hanno prodotti.

## Ordine di lettura (obbligatorio prima di toccare qualcosa)

1. [php-rust/REGOLE.md](php-rust/REGOLE.md) — l'UNICA lista del processo
   (misura, pin, gate, rotazione). Cap 25 righe.
2. [php-rust/NEXT_SESSION_WORDPRESS.md](php-rust/NEXT_SESSION_WORDPRESS.md) —
   stato corrente, pin, ordine del giorno, veti. È l'unico posto dello STATO.
3. [php-rust/migration/RULEBOOK.md](php-rust/migration/RULEBOOK.md) — invarianti
   architetturali; read-only in sessione, modifiche solo con sign-off utente.
4. L'ultimo `php-rust/sessions/WP_SESSION_<N>.md` e la sua `revisione-s<N>.md`
   nel `wp<N>-harness/`.
5. Storia: `php-rust/gaps/GAP_TREND.md`, `php-rust/PERF_MAP.md`,
   `php-rust/PIN_REGISTRY.md`, `php-rust/PHPR_DIVERGENCES_FROM_PHP.md`.

## Layout

Workspace Cargo in `php-rust/` (eseguire lì cargo e script). Sei crate sotto
`php-rust/crates/`: `php-types` (Zval 16 byte, stringhe byte, array, oggetti),
`php-builtins`, `php-runtime` (lower/HIR/compile/bytecode/vm, GC, OOP),
`php-cli` (`phpr`), `php-server` (Axum dietro feature `axum-server`),
`phpt-runner`. Pipeline unica: mago → AST → HIR → bytecode → VM.
Harness per sessione in `php-rust/wp<N>-harness/` (criteri PRIMA, verdetti
`.out`, copie dichiarate con manifest); output di run MAI nel repo (`.gitignore`
per cartella di output al primo commit dell'harness).

## Invarianti (dettaglio nel Rulebook)

- Correct-or-absent: niente stub che mentano a `function_exists()`.
- Byte-parity per tutto ciò che finisce in stringhe PHP; parità funzionale via
  crate/FFI di sistema (zlib, gd, xslt, tidy) per il resto.
- `Rc` + COW, Vm/Zval `!Send`, nessuna VM migrata fra thread.
- Distruttori PHP mai nel `Drop` Rust; sweep esplicito, ordine osservabile.
- Class table append-only, first-wins, moduli pubblicati immutabili.
- Veti permanenti (⚖️): NaN-boxing, fn-table dispatch, object arena, BOLT/PGO.

## Ambiente e strumenti

- Toolchain in `php-rust/rust-toolchain.toml` (**1.98.1 da S-180**, richiesta
  utente 2026-09-20; i pin precedenti restano attribuiti alla loro ricetta).
  Profilo release: fat LTO, 1 codegen unit. Mai cambiare toolchain o ricetta
  durante un arco di misura.
- Target canonica `~/Claude/php-rust-output` (porta i binari pinnati): nessuna
  build esplorativa lì. Sviluppo ordinario su target dedicata APFS interna
  (`CARGO_TARGET_DIR=$HOME/Claude/php-rust-dev-output`); sempre `--release`.
  Il volume esterno ExFAT non regge la cache incrementale.
- Rust si naviga e si edita con **Serena**; il C di php-src con **Vexp**.
  Hook locali bloccano cat/grep sui `.rs` e `git add` di `.rs` (usare `add -u`
  + `commit -F`).
- Pin/stash SOLO via `scripts/pin-phpr.sh` / `scripts/pin-server.sh`; corpus via
  `scripts/corpus-gate.sh` (fail-set congelato per NOME, oggi 1412 per modo);
  micro via `wp97-harness/micro/run-micro.sh` (pavimento per binario).
- MySQL WordPress solo col datadir esterno `mysql-wp8/data`; uploads solo via
  guardia backup/wipe/restore.

## Git e CI

Repo canonico [francescotinti/php-rust](https://github.com/francescotinti/php-rust),
branch `main`. `origin` ha DUE push URL (GitHub + bare locale
`/Volumes/Extreme Pro/Claude/phpr-ci/repo.git`): ogni push accoda un job della
CI locale (`php-rust/ci/README.md`; feed `phpr-ci/CI_FEED.log`; target
`/private/tmp/phpr-ci-target`). La CI è allarme precoce, non gate di record.
Il runner aspetta il lock `/private/tmp/phpr-measure.lock` (token della
sessione). File AppleDouble `._*` sul volume esterno rompono git e Serena:
rimuoverli, mai indicizzarli.

## Licenza

**PHP License 3.01** (scelta del proprietario, 20 settembre 2026): `LICENSE`
alla radice e in `php-rust/`, SPDX `PHP-3.01` nel workspace Cargo. Il sito
phprust.com va allineato nel suo progetto.
