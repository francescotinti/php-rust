# Prompt — Prossima sessione dedicata: ext/session, poi symfony/http-kernel

> Copia il blocco qui sotto come primo messaggio di una NUOVA sessione dedicata.
> Prosecuzione del filone Symfony: http-foundation è COMPLETA (0 errori; restano
> solo i 12 functional test che spawnano `php -S`). Il prossimo passo a leva più
> alta è **ext/session** (riammette i 371 test di Tests/Session ed è il
> prerequisito del session-middleware di HttpKernel), poi **http-kernel**.

---

Riprendiamo phpr (PHP 8.5.7 in Rust, /Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust).
Nuova sessione DEDICATA: **ext/session** (23 funzioni + SessionHandler,
SessionHandlerInterface, SessionUpdateTimestampHandlerInterface,
SessionIdInterface), quindi — se il tempo lo consente — recon di
**symfony/http-kernel**.

## Dove siamo (leggi la memoria PRIMA di agire)
- Indice memoria: MEMORY.md → [[php-rust-symfony-http-foundation]] (stato del
  filone Symfony, 7 commit ba96018→18dde6a) + [[php-rust-todo-master]] + TODO.md.
- Stato: Zend corpus 2352 · fn 754/2143 (core 514/654) · http-foundation 0E/12F
  (i 12 = soli `php -S` server-based) · ORM baseline 3e/15f · ext/date gate
  OBBLIGATORIO quando si tocca date/prelude DateTime.
- Workspace http-foundation e orm-work: ricette nei topic file di memoria
  (effimeri, scratchpad; composer via ORACOLO — phpr non esegue .phar).

## Obiettivo: ext/session
- Modello CLI: session su file (files handler) con save_path, id generation,
  session_start/commit/destroy/regenerate_id, $_SESSION superglobal bind,
  serializzazione formato `php` (key|serialized). Contratti dall'oracolo
  (probe byte-identici) e da ext/session C in /Volumes/Extreme Pro/Claude/php-8.5.7.
- Classi: SessionHandler (delega al save handler interno), le 3 interfacce,
  session_set_save_handler (callable e object form).
- Gate finale: riammettere Tests/Session nella suite http-foundation
  (togliere l'exclude da phpunit-nosession.xml) e fotografare.
- Il playbook zlib (contratti oracle-locked, batch per suite, FFI solo se
  serve) è il template; qui è quasi tutto VM/host + prelude.

## Metodo e gate (INVARIANTI — identici alla sessione precedente)
- RECON-FIRST, probe oracle per ogni contratto, correct-or-absent.
- Gate per OGNI commit: byte-id vs oracle 8.5.7 · Zend corpus diff per NOME
  (`--list-fails`, baseline via stash/rebuild) · ext/session phpt
  (/Volumes/Extreme Pro/Claude/php-8.5.7/ext/session/tests) diff per nome ·
  cargo test · ORM gate se si toccano ref/arg/reflection (baseline 3e/15f).
- Commit AND push a ogni step. Aggiornare a fine sessione: MEMORY.md,
  TODO.md, COVERAGE.md/README (usa la skill `gh-status-sync`),
  PHPR_DIVERGENCES_FROM_PHP.md.

## Tooling
- Serena per nav/edit Rust (hook attivo); log NUL: `LC_ALL=C tr -d '\0'`;
  run pesanti SEQUENZIALI e DETACHED (il wrapper `timeout` produce finti hang);
  phpr bufferizza stdout → per localizzare hang usare `--log-events-text` o
  marker `file_put_contents`.

Inizia con la ricognizione: probe oracle del ciclo session_start→$_SESSION→
commit su file, poi proponimi un piano a cluster prima di implementare.
