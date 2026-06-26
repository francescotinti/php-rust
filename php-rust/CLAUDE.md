# php-rust — istruzioni di progetto

## MANDATORY: tooling per ogni sessione
È **obbligatorio** usare in OGNI sessione su questo progetto:

- **Serena** (`mcp__serena__*`) per navigazione ed editing **simbolici** —
  `find_symbol`, `get_symbols_overview`, `find_referencing_symbols`,
  `replace_symbol_body`, `insert_after_symbol`. All'avvio:
  `mcp__serena__activate_project` (`php-rust`) + `mcp__serena__initial_instructions`.
- **Vexp** (`mcp__vexp__*`) per orientarsi a basso costo di token —
  `get_context_capsule`, `get_skeleton`, `get_impact_graph`, `search_logic_flow`.

Usare grep/Read grezzi **solo come fallback**. Questo vale soprattutto per i file
grossi (`crates/php-runtime/src/vm/mod.rs` ~11k righe).

**REGOLA OBBLIGATORIA — Serena + file `._*`:** quando Serena dà errore
(`UnicodeDecodeError 0xb0`, panic su `._*.rs`) per via dei file `._*` AppleDouble
(il volume esterno li ricrea durante i build), **NON** usare bash/grep come
fallback. DEVI: (1) `/usr/bin/find . -name '._*' -type f -not -path './.git/*' -delete`,
e (2) **riprovare ancora con Serena** la stessa operazione. Ripetere se ricompaiono.
grep/Read diretti NON sono un fallback accettabile per evitare Serena.

## Build & test (regole)
- Build: `CARGO_TARGET_DIR=$HOME/Claude/php-rust-output cargo build --release`
  (il `target/` in-repo è stale; non usarlo).
- Unit: `CARGO_TARGET_DIR=$HOME/Claude/php-rust-output cargo test --release` → deve restare **1499 passed**.
- Corpus: `$HOME/Claude/php-rust-output/release/phpt-runner --list-fails --isolate "/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests"` (foreground, timeout 600000). Delta vs baseline con `comm`; disciplina **zero pass→fail**.
- Oracle: `$HOME/Claude/php-oracle/php-src/sapi/cli/php`; CLI nostro `phpr` = `$HOME/Claude/php-rust-output/release/phpr`. Metodo: `diff <(oracle x.php) <(phpr x.php)` finché IDENTICAL.
- Commit **e** push a ogni step concluso (no chiedere).
