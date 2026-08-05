# Verbale SEDIA 6 — Pedersen (confine per-richiesta, lifecycle server, ambiente) — Concilio WP-102

## VERDETTO

Il flip S-100 nel merito CLI regge; il **«sì — GRADUATO» del pin server f2ab0636 sovradichiara** e la parità bimodale server ha un buco di controllo positivo. **Una refutazione capitale** (R1).

## Refutazioni

**R1 (CAPITALE) — parità server bimodale senza controllo positivo di modo.** Nei due bracci l'esito ATTESO è identico (fails=0 off e on): `fails=0 ×2` è indistinguibile da «PHPR_REG_LOWER non è mai arrivato al processo server e i due bracci erano entrambi default-on». Il funnel CLI asserisce dump-hash DIVERSI tra i bracci; per il server non esiste l'analogo: nessuna riga di log del modo effettivo, nessuna interrogazione, `srv.log` scritto e mai giudicato. La parità bimodale server è oggi un gate che non può fallire per la causa che dichiara di sorvegliare.

**R2 — gradazione mal nominata.** La voce «server-HTTP option+restapi via phpr CLI ✓» accredita al pin **php-server** una gamba che quel binario non esegue: `run_group` in `s100-parity-server.sh` usa solo `$PHPR`/`$ORACLE`. Solo la gamba A (sentinella) esercita f2ab0636. In più l'hash phpr in gamba B è «registrato, non gate»: la gamba può girare su un phpr non-pin. Il WP VERO dietro HTTP resta non esercitato — la gamba B è CLI phpunit.

**R3 — che cosa la sentinella estesa NON vede.** (a) 16 richieste ÷ 4 per endpoint: i leak LENTI (100+ richieste) restano fuori; (b) la byte-identità dell'output non misura la memoria: un leak di retention stile WP-78 che non altera l'output passa — nessun campione RSS/footprint del processo server; (c) burst concorrente solo su `p1`, nessuna prova che ENTRAMBI i workers abbiano servito (nessun worker-id); (d) `j1.php` è «restapi-shaped», non restapi; (e) status HTTP mai asserito.

**R4 — putenv impotente solo su REG_LOWER (census eseguito in seduta, chiude la parte enumerativa di A-PE-101-2).** `PHPR_STUB_ELISION` e `PHPR_UNIT_CACHE` (`vm/mod.rs:15718/15727`) sono LAZY (`get_or_init`), a grammatica APERTA (`map_or(true, |v| v != "0")` — presence-quirk della stessa classe del vecchio `=0`-accende), NON sigillate: un `putenv` prima del primo uso decide il modo dell'INTERO processo server — azione per-richiesta che muta stato di processo, violazione del confine. Anche `PHPR_REQ_NS` lazy in `worker_pool.rs:155`; `PHPR_MEM_CENSUS` riletto a runtime (apparato, ma osservabile).

**R5 — deployment reale.** Il modo viene dall'env allo spawn, ma solo i launcher lo costruiscono: nessun `--build-info` (backlog A-HO-101-4/A-PE-101-5 ancora aperto) né log di startup dichiara il default del binario deployato; lo spawn fuori-launcher (shell utente, launchd) non è collaudato da nessun atto.

**Su §S-101**: «l'emissione non cambia ⇒ batteria+corpus bastano» esclude il server — ma H-C1 tocca clone/drop/gc_note del ciclo di vita Zval, ESATTAMENTE la classe di retention per-richiesta di WP-78: se H-C1 si scrive, la sentinella (e l'endurance sotto) è DOVUTA.

## Emendamenti

- **A-PE-102-1**: il server dichiari il modo effettivo (log startup + interrogabile); i launcher lo ASSERISCANO per braccio.
- **A-PE-102-2**: sigillo eager + value-parse a lista chiusa per `PHPR_STUB_ELISION` e `PHPR_UNIT_CACHE`; elenco R4 = base per chiudere A-PE-101-2 per NOME.
- **A-PE-102-3**: sentinella di ENDURANCE — N≥100 richieste su endpoint allocante + RSS/footprint del server inizio/fine, banda calibrata sul rumore MISURATO.
- **A-PE-102-4**: una gamba WP VERA servita da php-server via HTTP prima di scrivere «server-HTTP» in una gradazione.
- **A-PE-102-5**: PIN_REGISTRY — ogni gradazione nomini il BINARIO che la esegue; hash phpr in gamba B diventa gate.
- **A-PE-102-6**: se H-C1 si scrive in S-101, sentinella estesa + endurance nel suo ordine di collaudo, non opzionali.

## Kill-switch

- **KS-PE-102-1**: nessuna cifra «server modo X» senza asserzione del modo EFFETTIVO nel processo server.
- **KS-PE-102-2**: nessun nuovo `PHPR_*` dietro `OnceInit` senza grammatica a lista chiusa + braccio anti-putenv.
