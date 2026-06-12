# Phase 0 — Reconnaissance

> Generato con assistenza AI (Claude Fable 5). Data: 2026-06-13.

## Domanda

Esiste già un porting Rust di PHP (o un ecosistema di crate) che renda inutile partire da zero?

## Cosa è stato cercato

- GitHub / crates.io: "PHP interpreter Rust", "php-rs", "php parser rust", "mago"
- Stato di manutenzione e licenze dei progetti trovati

## Cosa è stato trovato

| Progetto | Cosa è | Stato | Verdetto |
|---|---|---|---|
| [mago](https://github.com/carthage-software/mago) | Toolchain PHP in Rust: lexer, parser, linter, formatter, static analyzer | Attivo (v1.30, 2026), Apache-2.0 | **Riusabile come front-end** (lexer+parser). Non ha runtime/VM. |
| [php.rs](https://github.com/alexisbouchez/php.rs) | Interprete PHP in Rust con replica dell'architettura Zend | Hobby-scale, non maturo | Non adatto come base (Strategia G esclusa) |
| [php-parser-rs](https://github.com/php-rust-tools/parser) | Parser PHP recursive-descent | **Archiviato** | Escluso |
| PHPantom | Language server PHP in Rust | Attivo | Solo LSP, non pertinente |

## Decisione

**Nessun runtime PHP completo e maturo esiste in Rust** → si procede greenfield per il runtime
(non si applica la Strategia G fork+contribute all'engine).

**mago si adotta come dipendenza per il front-end** (Strategia A — adapter): elimina ~25K LOC
di scanner re2c + grammatica Bison + AST. Il bridge verso il nostro HIR è isolato in un solo
modulo per mantenere la sostituibilità.

## Cambio di rotta architetturale (decisione utente, 2026-06-13)

La prima proposta era un porting fedele a Zend (opcode VM, subset di ~60 opcode). L'utente ha
richiesto un approccio più moderno: **non ricreare lo Zend Engine** (design 1999-2004 con
allocator custom, VM generata da template, TSRM, opcache) ma reimplementare il **comportamento
osservabile** sfruttando Rust e il suo ecosistema. Il contratto è l'output dei .phpt, non
l'architettura interna. Conseguenze:

- Evaluator tree-walking su HIR (AST risolto) invece di opcode VM → ~146K LOC di VM generata
  sostituite da 3-5K LOC di `match`
- zend_alloc/zend_gc/TSRM/Optimizer/opcache: eliminati by design (ownership, Rc+COW,
  Send/Sync, processo residente)
- Estensioni: mappa su crate maturi + strato sottile di fedeltà PHP, non porting
- Unico porting fedele riga-per-riga: `zend_operators.c` (type juggling) + formattazione
  float + formato messaggi di errore

## Vincoli ambiente rilevati

- Nessun binario `php` sul sistema dev (macOS, no Homebrew php): l'oracle differenziale
  primario sono le sezioni `--EXPECT--`/`--EXPECTF--` dei .phpt. Per il differential su
  espressioni (step 2) si compilerà il php 8.5.7 di riferimento dal sorgente locale
  (fallback: `brew install php`).
- cargo 1.90.0 disponibile.
