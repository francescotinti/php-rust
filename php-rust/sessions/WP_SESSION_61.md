# WP_SESSION_61 — php-server AUTONOMO sul SAPI maturo: accettazione meccanica VERDE (5/6 BYTE-ID + admin NORM-ID vs phpr -S) su wpdev/src servito end-to-end (admin inclusa)

> ⚡ **WP-61 (2026-07-26, `c0eb97a`+seg.)** — P1 della direttiva utente
> ("voglio verificare PERSONALMENTE che WordPress giri, admin inclusa,
> attraverso il server standalone") CONSEGNATO.

## P1 — php-server = wrapper sottile del SAPI cli-server (WP-4/5/6)

Architettura scelta (rischio minimo per il binario di parità):
- **php-cli esposto anche come lib** (`crates/php-cli/src/lib.rs`:
  `pub mod server; pub mod mime;`) — `main.rs` NON toccato; entrambi i
  target compilano gli stessi sorgenti.
- **php-server riscritto da zero** (`crates/php-server/src/main.rs`):
  via axum/tokio (il prototipo 65 righe GET-only è morto), ora è un
  parser argomenti (`--host`/`--port`/`--docroot|-t`/router posizionale,
  default `127.0.0.1:8080` docroot `.`) che delega a
  `php_cli::server::serve()` in forma `phpr -S`. Stesso global allocator
  mimalloc di phpr, stesso `logging::init`.
- **Crate de-gitignorato** (`crates/php-server/` era in `.gitignore`
  riga 1: il prototipo non era MAI stato in history) — ora è un
  deliverable pubblicato.

## Gate (solo server-layer: engine/vm NON toccati)

- cargo **1645/0** ✓ (criterio pre-registrato per cambi solo-server).
- ⚠️ il hash di `phpr` è cambiato comunque (metadati del crate per il
  nuovo lib target, codegen di main.rs identico): **ri-stash come da
  direttiva → `phpr-old-target/release/phpr-wp61` (sha256 `c7e93597…`)
  = baseline di parità corrente**; phpr-wp60 (46b517fc…) resta accanto.

## Ambiente di verifica: wpdev/src servito per la PRIMA volta

- `~/Claude/wpdev/src/wp-config.php` NUOVO → **DB `wp`** (era vuoto e
  senza grant: `GRANT ALL ON wp.* TO 'wp'@'%'` aggiunto). wp_o/wp_p
  della batteria WP-9 NON toccati. Salts fissi `wp61-…`.
- Install WordPress via **oracle** `php -S` + `install.php?step=2` sulla
  porta definitiva 8080 (siteurl nasce giusto): sito "wpdev", utente
  **admin / wp-secret-Pass1**. Pretty permalinks
  `/%year%/%monthnum%/%postname%/` via SQL + reset `rewrite_rules`
  (rigenerazione lazy verificata).
- 🔵 **wordpress-develop src/ ha un guard anti-unbuilt**: senza
  `npm run build --dev` l'index muore 500 "You are running WordPress
  without JavaScript and CSS files" (wp-login invece passa — il guard
  vive solo in src/index.php e negli entry admin). Risolto col flusso
  standard del repo: `npm install` (18s, cache) + `npx grunt build
  --dev` → `src/wp-includes/{js/dist,build}` popolati. Senza build
  anche l'admin sarebbe stata senza JS: la build era necessaria alla
  verifica personale, non solo al guard.

## Accettazione MECCANICA (battery61.sh, pattern run-http.sh WP-9:
## stessi URL, stessa porta 8080, php-server poi phpr -S in sequenza)

| probe | status | confronto |
|---|---|---|
| front `/` | 200 (57.428B) | **BYTE-ID** |
| asset `/wp-includes/css/buttons.css` | 200 | **BYTE-ID** |
| permalink `/2026/07/hello-world/` | 200 (71.093B) | **BYTE-ID** |
| wp-login GET | 200 | **BYTE-ID** |
| wp-login POST | 302 → /wp-admin/ + cookie `wordpress_logged_in` | **BYTE-ID** |
| dashboard `/wp-admin/` | 200 (143.310B) | **NORM-ID** (solo `_wpnonce` di logout: sessioni di login distinte per costruzione) |

Falsi allarmi rientrati durante la messa a punto: (i) il primo probe
permalink usava `/2026/07/26/hello-world/` col GIORNO — il 301 era il
canonical redirect corretto della struttura senza `%day%`; (ii) il primo
DIFF admin era il nonce fuori-virgolette (`&_wpnonce=…`) non coperto dal
normalizzatore WP-9.

## Consegna

- Comando utente: `~/Claude/php-rust-output/release/php-server --port
  8080 --docroot ~/Claude/wpdev/src` → http://127.0.0.1:8080/ (admin:
  `/wp-admin/`, admin / wp-secret-Pass1). MySQL: datadir
  `mysql-wp8/data`, socket `/private/tmp/mysql-wp8.sock` (MAI start
  naive).
- **Quick-start pubblicato nel README** (sezione "Quick-start: serve
  WordPress with php-server") + layout crates aggiornato; gh-status-sync
  eseguita.
- Server di prova KILLATI a fine batteria (lsof 8080 pulito).

## ⭐ Lezioni

- ⭐⭐ **Lib-target senza toccare main.rs = estrazione a rischio zero**:
  il bin continua a compilare i SUOI moduli, la lib espone gli stessi
  file; nessuna divergenza possibile tra phpr -S e php-server (i 6
  probe BYTE/NORM-ID lo confermano sperimentalmente).
- ⭐⭐ **wpdev/src non è servibile as-is**: il guard npm-build di
  wordpress-develop distingue src/ da un release tree (wp-mo). Chi
  serve src/ DEVE passare da `grunt build --dev` (e la full-suite phpt
  non se ne accorge: i test non caricano gli entry guarded).
- ⭐ Un lib target aggiunto cambia il hash del binario di parità anche a
  codegen identico (metadata cargo): il ri-stash con verbale è la
  risposta giusta, non un allarme.
- ⭐ I nonce WP compaiono anche fuori-virgolette (`&_wpnonce=hex10`): un
  normalizzatore per confronti admin deve coprire entrambe le forme.

## Prossimo (WP-62) — invariato dal programma WP-60/61

1. **Census v2 DEEP dei Module** (se non aperto in coda a questa
   sessione): dedup per indirizzo, payload op per-kind, Const profondi,
   literal cross-unit, slack, G_UNITS — attribuire il ~600MB non-seed.
2. **Fase 0.5 compile-cache** (rilassare la chiave `unit_chain_fp`;
   predizione registrata −240MB+ full, 3 bersagli nominati).
3. **Seed signature-only** (banda misurata ~130-195MB, A/B separato).
