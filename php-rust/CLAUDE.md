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

## Stato & copertura
- **[COVERAGE.md](COVERAGE.md)** è la pagina dati (misurata, non stimata): funzioni
  641/2143 (core stdlib 483/654 = 73%), corpus Zend 2324, aree complete, mancanti
  per estensione. **[README.md](README.md)** è la home GitHub. Aggiornarle quando i
  numeri cambiano in modo sostanziale. Divergenze note in
  [PHPR_DIVERGENCES_FROM_PHP.md](PHPR_DIVERGENCES_FROM_PHP.md) (principio
  **correct-or-absent**).

## Build & test (regole)
- **⚠️ FILESYSTEM — il volume esterno "Extreme Pro" NON supporta la compilazione
  incrementale di Rust** (non sa hard-linkare la cache → cargo stampa "hard linking
  files … failed" e può lasciare binari stale/incoerenti). Per questo TUTTI gli
  artefatti di build vivono sul **volume principale** (`/Users/francescotinti/Claude/php-rust-output`).
  Questo è già imposto da `php-rust/.cargo/config.toml` (`target-dir = …`), quindi
  un semplice `cargo build --release` basta. **MAI** ridirigere il `target-dir`
  sul volume esterno né usare il `target/` in-repo. Sorgente e corpus stanno sul
  volume esterno ma sono solo letti (nessun artefatto di compilazione lì).
- Build: `CARGO_TARGET_DIR=$HOME/Claude/php-rust-output cargo build --release`
  (equivalente al config.toml; l'env è ridondante ma innocuo).
- Unit: `CARGO_TARGET_DIR=$HOME/Claude/php-rust-output cargo test --release` → i test
  lib di `php-runtime` devono restare **554 passed** (più gli altri crate verdi); non regredire.
- Corpus: `$HOME/Claude/php-rust-output/release/phpt-runner --list-fails --isolate "/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests"` (foreground, timeout 600000). Delta vs baseline con `comm`; disciplina **zero pass→fail**.
- Oracle: `$HOME/Claude/php-oracle/php-src/sapi/cli/php`; CLI nostro `phpr` = `$HOME/Claude/php-rust-output/release/phpr`. Metodo: `diff <(oracle x.php) <(phpr x.php)` finché IDENTICAL.
- Commit **e** push a ogni step concluso (no chiedere).
