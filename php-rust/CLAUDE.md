# php-rust — istruzioni di progetto

## MANDATORY: tooling per ogni sessione
È **obbligatorio** usare in OGNI sessione su questo progetto:

- **Serena** (`mcp__serena__*`) per navigazione ed editing **simbolici** del Rust —
  `find_symbol`, `get_symbols_overview`, `find_referencing_symbols`,
  `replace_symbol_body`, `insert_after_symbol`, `search_for_pattern`. All'avvio:
  `mcp__serena__initial_instructions` (progetto `php-rust`).
- **Vexp** (`mcp__vexp__*`) per il C di php-src e per orientarsi a basso costo di
  token — `get_skeleton`, `expand_vexp_ref`, `run_pipeline`, `index_status`.

grep/cat/Read sui `.rs` via Bash sono **BLOCCATI** dall'hook `serena-vexp-guard.sh`
(e `git add` di `.rs`: usare `git add -u` + `git commit -F`). Non è un fallback:
usare Serena. Vale soprattutto per i file grossi
(`crates/php-runtime/src/vm/mod.rs` ~26k righe, `run.rs` ~7,4k con cap LOC dichiarato).

**REGOLA OBBLIGATORIA — Serena + file `._*`:** quando Serena dà errore
(`UnicodeDecodeError 0xb0`, panic su `._*.rs`) per via dei file `._*` AppleDouble
(il volume esterno li ricrea durante i build), **NON** usare bash/grep come
fallback. DEVI: (1) `scripts/clean-appledouble.sh` (equivale a
`/usr/bin/find . -name '._*' -type f -not -path './.git/*' -delete`; `--git` pulisce
anche `.git/`), e (2) **riprovare ancora con Serena** la stessa operazione. Ripetere
se ricompaiono.

## Ordine di lettura (una fonte sola per ogni fatto)
1. [../AGENTS.md](../AGENTS.md) — mappa delle fonti, layout, invarianti, ambiente.
2. [REGOLE.md](REGOLE.md) — l'UNICA lista del processo (misura, pin, gate, rotazione;
   cap 25 righe).
3. [NEXT_SESSION_WORDPRESS.md](NEXT_SESSION_WORDPRESS.md) — l'UNICO posto dello STATO:
   pin correnti, batteria, corpus, ordine del giorno, veti.
4. [migration/RULEBOOK.md](migration/RULEBOOK.md) — invarianti architetturali.
5. L'ultimo `sessions/WP_SESSION_<N>.md` e `wp<N>-harness/revisione-s<N>.md`.

Apertura/chiusura sessione: skill `apri-sessione` (pre-flight meccanico) e
`chiudi-sessione` (verbale + rotazione handoff).

## Rulebook di traduzione (LEGGERE prima di toccare semantica)
Le regole del porting — posture byte-parity/functional-parity, correct-or-absent,
policy unsafe, mappature canoniche zval/refcount/destructor/GC, ecosistema
ammesso/bandito, marker `BUG(port):`/`PERF(port):`/`TODO(port):`, disciplina dei
gate per nome — sono in **[migration/RULEBOOK.md](migration/RULEBOOK.md)**
(forma del [code-migration-kit](https://github.com/anthropics/code-migration-kit-with-claude-code)
di Anthropic, adottato in WP-39). Il rulebook è read-only in sessione: gli
emendamenti passano dal sign-off dell'utente e dal handoff.

## Stato & copertura
- **[COVERAGE.md](COVERAGE.md)** è la pagina dati (misurata, non stimata): funzioni
  per estensione, corpus Zend, WordPress, aree complete. I numeri correnti di
  pin/batteria/corpus stanno SOLO in NEXT_SESSION_WORDPRESS.md. Storia:
  [PIN_REGISTRY.md](PIN_REGISTRY.md), [PERF_MAP.md](PERF_MAP.md),
  [gaps/GAP_TREND.md](gaps/GAP_TREND.md). **[README.md](README.md)** è la pagina
  di progetto; la home GitHub del repo è il README della root. Rigenerare con
  `scripts/measure-coverage.sh` / skill `gh-status-sync` a ogni pin nuovo o quando
  corpus/funzioni cambiano di ≥1 %. Divergenze note in
  [PHPR_DIVERGENCES_FROM_PHP.md](PHPR_DIVERGENCES_FROM_PHP.md) (principio
  **correct-or-absent**).

## Build & test (regole)
- **⚠️ FILESYSTEM — il volume esterno "Extreme Pro" NON supporta la compilazione
  incrementale di Rust** (non sa hard-linkare la cache → cargo stampa "hard linking
  files … failed" e può lasciare binari stale/incoerenti). TUTTI gli artefatti di
  build vivono sul **volume principale**. **MAI** ridirigere il `target-dir` sul
  volume esterno né usare il `target/` in-repo. Sorgente e corpus stanno sul volume
  esterno ma sono solo letti.
- **Due target dir**: `~/Claude/phpr-target/dev-output` è il default del `.cargo/config.toml`
  locale (sviluppo ordinario: un semplice `cargo build --release` finisce lì). Vive nella
  sparsebundle APFS `phpr-target.sparsebundle` sul volume esterno: montarla con
  `wp182-harness/phpr-target-bundle.sh mount` (da smontata il mountpoint è bloccato e cargo
  fallisce; `status`/`compact` per spazio).
  `~/Claude/php-rust-output` è la target **CANONICA** e porta i binari pinnati in `release/`
  (il pre-flight ne confronta l'hash col pin dichiarato): ci si costruisce SOLO via
  `scripts/pin-phpr.sh` / `scripts/pin-server.sh` o nelle build di promozione, SEMPRE con
  `CARGO_TARGET_DIR=$HOME/Claude/php-rust-output` esplicito.
- Toolchain pinnata in `rust-toolchain.toml` (1.98.1): mai cambiare toolchain o
  ricetta durante un arco di misura.
- **Build di sviluppo**: `cargo build --profile dev-release` (profilo incrementale senza LTO).
  Binario in `~/Claude/phpr-target/dev-output/dev-release/phpr`: MAI pin, MAI braccio di misura,
  MAI CI. La ricetta del pin
  resta `cargo build --release` (fat LTO, cgu 1). Il profilo debug non si usa (~3,8G).
- **Hash, non mtime**: cargo ricrea `release/<bin>` come COPIA del `deps/` con mtime
  conservato (0 «Compiling»): un gate «binario ricostruito» si fa sull'hash; per forzare
  un relink vero `cargo clean --release -p php-cli -p phpt-runner` nella canonica.
- La ricetta è deterministica solo nella STESSA target (il percorso della target entra nel
  binario): confronti di hash canonica↔canonica. `pin-server.sh` costruisce con
  `--features axum-server`: un `cargo build --release` del workspace sulla canonica cambia
  il feature set del server (non è non-determinismo).
- Unit: `cargo test --release` → rc dal comando, MAI da pipe; il workspace non deve
  MAI regredire la batteria dichiarata in NEXT_SESSION_WORDPRESS.md (il numero cresce
  coi denti nuovi).
- Corpus: `scripts/corpus-gate.sh <phpt-runner> <outdir>` (fail-set CONGELATO per NOME, ×2 modi). Il runner
  grezzo `~/Claude/php-rust-output/release/phpt-runner --list-fails --isolate "/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests"`
  (foreground, timeout 600000) serve solo per indagine; delta con `comm`.
  Disciplina **zero pass→fail**.
- Potatura target a fine sessione: `scripts/target-prune.sh <pin_phpr16> <pin_server16>`
  (tiene i 3 binari pinnati). CI locale per-commit
  (allarme precoce, NON gate di record): `ci/README.md`, feed `phpr-ci/CI_FEED.log`.
- **Processi e finestre**: le sonde/monitor vivono in un FILE (`bash -c` inline coi nomi
  degli script nell'argv fa scattare il `pgrep -f` di coordinatori e runner CI); mai
  lanciare un daemon mentre il tree è in checkout detached; mai build, CI e misura
  concorrenti (S-184: swap 15G, Data 2G, phpt-runner OOM) — Data ≥10G e app pesanti
  chiuse PRIMA della finestra.
- Oracle: `/opt/homebrew/opt/php/bin/php` (PHP 8.5.7, lo stesso degli script);
  CLI nostro `phpr` = `~/Claude/php-rust-output/release/phpr` (pin). Metodo:
  `diff <(oracle x.php) <(phpr x.php)` finché IDENTICAL.
- Commit **e** push a ogni step concluso (no chiedere).
