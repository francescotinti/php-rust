# Verbale sedia 6 — Pedersen (confini per-richiesta, lifecycle, ambiente) — Concilio WP-100

**Oggetto**: report S-98.0 + programma S-99.0. **Mandato**: refutare.

## VERDETTO: CON EMENDAMENTI — una refutazione capitale sull'ORDINE di S-99

### Refutazione capitale R1 — L'ordine 1→2 di S-99 viola KS-PE-99-1 per costruzione

Il punto 1 (collaudo full+media WordPress col launcher) **usa php-server**: full e media girano dal WP-77 attraverso Axum, e ogni coppia full storica cita il pin server accanto a quello phpr. Ma KS-PE-99-1 dice «VOID **ogni uso/misura** del server prima di restapi+option per NOME sotto env -i» — e il pin 365f4d4069513de3 non è collaudato. Eseguire il punto 1 prima del punto 2 produce un collaudo VOID dal suo stesso kill-switch. **L'ordine va invertito: parità server PRIMA del collaudo WP** (o il punto 1 deve dichiararsi CLI-only, cosa che non è). Il debito è doppio: 832568a7 mai collaudato, 365f4d40 mai collaudato, d45b578 non riproducibile — **due rotazioni di pin senza un solo collaudo**, e il 365f4d40 è nato come *effetto collaterale* di una build, non da un ordine.

### Refutazione R2 — Il canale putenv è APERTO e il dente arriva DOPO il rollout: ordine indifendibile

`putenv()` PHP chiama `std::env::set_var` in-process (`crates/php-builtins/src/file.rs:1277`). `reg_lower::enabled()` è un `OnceLock` **pigro** (`reg_lower.rs:50-53`): il modo del processo è deciso alla PRIMA lettura, che oggi avviene «quando capita» (primo compile: `func.rs:162`, `compile/mod.rs:750`, o costruzione UnitKey `vm/mod.rs:16058`). In CLI il sigillo cade compilando `{main}` prima che lo script possa eseguire putenv — sicuro *per accidente*. Nel server il sigillo dipende dall'ordine di boot (prelude compilato prima dell'accept?): **invariante non pinnata da nessun test**. Peggio: `set_var` con worker concorrenti che leggono env è unsound in Rust. Mettere il dente anti-putenv al punto 4, DOPO le misure flag-on del punto 3 sul percorso di promozione a default, lascia il canale aperto esattamente nella finestra in cui il flag conta di più.

### Refutazione R3 — Il funnel test copre il runner CI solo per DUE variabili, e il pin di parità ora CHURNA

`reg_lower_funnel.rs:18-19` fa `env_remove` di PHPR_REG_LOWER e PHPR_DUMP_OPS prima di `env()`: il caso «flag esportato nell'ambiente del runner» è coperto **per il figlio spawannato**, e M5 (`reg_lower.rs:597-601`) copre la batteria in-process. Bene. Ma (i) è una allowlist di sottrazione a lista chiusa — la lezione WP-96 dice che l'ambiente si COSTRUISCE, non si sottrae: ogni futura PHPR_* che biforca l'emissione riapre il buco in silenzio; (ii) manca il controllo flag-OFF al funnel (nessun dump-assert «zero forme registro» sul binario vero); (iii) il test rilinka il binario di parità a ogni `cargo test` — l'hash del pin cambia per costruzione (4e268c3f ≠ 39e0a359, «stesso sorgente» dichiarato ma non provato da un gate).

### Su (d) — worker con modi diversi nello stesso processo: NO, oggi

`OnceLock` è atomico: un solo valore per processo; `reg_mode` in UnitKey (`vm/mod.rs:15525`) e nel fp (`lower/mod.rs:841`) è cintura-e-bretelle corretta. Ma il commento «fixed per process today» poggia su R2: la fissità è garantita dalla fortuna del primo lettore, non da un sigillo.

## Emendamenti

- **A-PE-100-1**: invertire S-99: (1) parità server restapi+option per NOME sotto env -i sul pin 365f4d40, (2) collaudo WP full+media.
- **A-PE-100-2**: sigillo EAGER di `reg_lower::enabled()` come primo atto dei DUE main (CLI e server) + test che putenv dopo il boot non può flippare il modo — PRIMA del punto 3, non al 4.
- **A-PE-100-3**: registro pin con campo `collaudato: sì/no`; vietata la terza rotazione non collaudata; vietato il pin nato come effetto collaterale.
- **A-PE-100-4**: funnel: aggiungere il braccio flag-OFF con dump-assert «zero forme registro»; il binario di parità si pinna per COMMIT+stash, non per hash churnante.

## Kill-switch

- **KS-PE-100-1**: VOID il collaudo WP di S-99 punto 1 se eseguito prima della parità server sul pin che serve le richieste.
- **KS-PE-100-2**: VOID ogni misura flag-on SERVER e ogni gate di promozione a default finché il sigillo eager + il test anti-putenv non esistono.
- **KS-PE-100-3**: VOID ogni cifra attribuita a un pin server non presente nel registro come `collaudato: sì`.
