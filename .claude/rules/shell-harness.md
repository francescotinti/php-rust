---
paths:
  - "php-rust/**/*.sh"
description: Convenzioni per gli script bash di harness, scripts/ e ci/ di phpr (si carica solo quando si legge o modifica uno .sh)
---

# Script bash di phpr (harness, scripts/, ci/)

- **bash 3.2 di macOS**: niente `declare -A`, niente `${var,,}`/`${var^^}`, niente `mapfile`. `set -u`; PATH chiuso con `/sbin` e `/usr/sbin` espliciti (`/sbin/mount`, non `mount`).
- **Esiti solo da file**: ogni script scrive `rc`/`.done`/verdetto `.out`; chi lo lancia legge quei file, mai il log intero né il rc di una pipe (`rc` dal comando, mai da `| tee`).
- **Run lunghe detached**: `perl "/Volumes/Extreme Pro/Claude/wp58-harness/daemonize.pl" <log> <cmd…>`; dopo il lancio leggere il log del daemonizer (gli errori di exec finiscono lì) e `pgrep`. Mai lanciare mentre il tree è in checkout detached; mai `sleep` in foreground nelle verifiche (Monitor o `until`).
- **`pgrep -f` senza auto-match**: il pattern non deve comparire nell'argv di chi lo esegue; le sonde e i monitor vivono in un FILE (`bash file.sh`), mai in `bash -c` inline coi nomi degli script.
- **Copia dichiarata**: uno script derivato da uno precedente nasce da un generatore con assert di unicità su ogni sostituzione, con manifest `diff vecchio nuovo > <nome>-copia.diff` committato accanto; l'intestazione elenca i SOLI adattamenti.
- **Patch sugli archivi**: mutanti e varianti si applicano all'archivio (`git archive`), mai al tree; dopo ogni patch, `rm` o scrittura che condiziona un gate: verifica con `grep`/`ls` (il rc non basta).
- **Niente token `.rs` nel testo**: uno script che cita file Rust si genera col tool Write, non con heredoc Bash (l'hook `serena-vexp-guard` blocca ogni segmento grep/sed/awk/cat/head/tail che contenga `crates/` o `.rs`).
- **Gate di build sull'hash, non sul mtime**: cargo ricrea `release/<bin>` come copia del `deps/` con mtime conservato; relink vero solo con `cargo clean --release -p <crate>`. Tolleranze numeriche dei gate pre-registrate come intervallo motivato: se mordono si rifà la build, non la soglia.
- **Canonica**: `CARGO_TARGET_DIR=$HOME/Claude/php-rust-output` esplicito; solo `-p php-cli` nei collaudi, server solo via `scripts/pin-server.sh` (`--features axum-server`). Target separate dei bracci sulla bundle `~/Claude/phpr-target/`.
- **Lock e finestre**: `/private/tmp/phpr-measure.lock` lo scrive il lanciatore col TOKEN della sessione dopo la quiete CI, mai a mano; gate ambientali con attesa senza limite (updater, calma CPU, processo singolo >50 %, loadavg <3, quiescenza s129, Data ≥10G con watchdog).
