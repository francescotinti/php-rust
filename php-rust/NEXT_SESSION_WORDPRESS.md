# Rotta WORDPRESS-FIRST — WP-track (mysqli chiusa in WP-8)

> 🏁 **WP-8 CHIUSA (2026-07-15)**: **ext/mysqli nativa (crate `mysql` v28) —
> WordPress 7.0.1 installato (`wp core install`) e servito su MySQL 9.7.1
> VERO a parità oracle**: 11/11 probe byte-id, schema dbDelta 0 diff,
> 13 rotte front + login flow byte-id, admin = soli volatili noti.
> Cade la dipendenza dal plugin sqlite-database-integration.
> Dettaglio nel changelog di `PHPR_DIVERGENCES_FROM_PHP.md` (WordPress-8, §2.5).

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap (decisione 2026-07-13,
memoria `php-rust-roadmap-wp-first`)**: obiettivo primario = 100%
compatibilità WordPress. Laravel solo come validazione posteriore.

## Cosa è entrato (sessione WP-8 — sintesi; dettaglio nel changelog)
1. `vm/mysqli.rs`: host builtin `__mysqli_*` sul crate `mysql` v28 (sync,
   caching_sha2 + RSA full-auth ok); conn/stmt = handle in Vm; result set
   bufferizzati; multi_query = split client-side; errori client 2002/2019
   sintetizzati, server verbatim; NUM_FLAG + SET NAMES per parità metadata.
2. `lower/prelude_mysqli.php` (5° prelude): 6 classi + ~75 fn procedurali;
   ~100 costanti MYSQLI_* in `resolve_constant`; ext `mysqli`+`mysqlnd`
   annunciate; nuovo `__warning_from_caller` (E_WARNING al call-site).
3. Harness WP-MySQL end-to-end (oracle E phpr) + batteria probe/HTTP.

## Ambiente/harness (per riprodurre — ⚠️ il reboot svuota /private/tmp!)
- **MySQL 9.7.1 brew, datadir PERSISTENTE su disco esterno**. Avvio:
  `/opt/homebrew/opt/mysql/bin/mysqld --datadir="/Volumes/Extreme Pro/Claude/mysql-wp8/data"
  --tmpdir="/Volumes/Extreme Pro/Claude/mysql-wp8/tmp" --port=3306
  --bind-address=127.0.0.1 --socket=/private/tmp/mysql-wp8.sock
  --log-error="/Volumes/Extreme Pro/Claude/mysql-wp8/mysqld.err" &`
  (socket NON sul volume esterno: "Operation not supported"). Utenti:
  root/'' e wp/'wp-secret-Pass1' (caching_sha2); DB: probe, wp_o (WP
  installato dall'oracle, pretty permalinks ON), wp_p (installato da phpr).
- **Batterie PERSISTENTI in `/Volumes/Extreme Pro/Claude/wp8-harness/`**:
  `mysqli-probe/run-probes.sh` (11 probe oracle-vs-phpr, exit≠0 su diff) e
  `battery.sh` (front+login+admin; ricetta ricostruzione albero WP in testa).
  Gli ALBERI wp (wp-mo/wp-mp) e i workspace wp-cli/ORM/HK vivevano in
  /private/tmp e vanno ricostruiti (ricette: testa di battery.sh per WP;
  memoria `php-rust-orm-gate-recipe` per ORM; per http-kernel: clone shallow
  symfony/http-kernel + `composer update` + `composer require --dev
  phpunit/phpunit:^11.5 -W` con l'oracolo — il tip si muove: baseline
  2026-07-15 = 1665 test 0E/0F).
- ⚠️ watch: un crash una-tantum di `phpr -S` (widgets.php, primo passaggio
  della prima batteria) NON riprodotto in 3 run completi dopo; se ricompare,
  RUST_BACKTRACE=1 e log server.

## Stato gate (post WP-8)
- corpus 1528 / sess 67 / date 377 / refl 294 per NOME (invariati) ·
  ORM 3484 3E/13F nei residui catalogati · hk (tip) 1665 0E/0F ·
  cargo 1550/0 · probe mysqli 11/11 byte-id · WP-MySQL: front 13 rotte +
  login 5 step byte-id, admin 12 = volatili noti, widgets 500 parità.

## Prossimo passo: SESSIONE WP-9 = ext/gd & media (roadmap tappa 5)
Chiude i residui admin documentati (upload webp/avif `upload_error`,
site-health `php_extensions` gd) e il media pipeline WP:
- ext/gd work-alike su crate Rust (candidati: `image` + `imageproc`;
  attenzione a byte-parity dei formati: WP fa resize/crop/thumbnail via
  GD — la parità sui BYTE dei file generati è probabilmente impossibile
  (algoritmi di resampling diversi), quindi **functional-parity** (policy
  roadmap: ciò che esce dal processo = functional) + parità dei METADATI
  (dimensioni, mime, srcset generati nelle pagine).
- Superficie: imagecreatefrom{jpeg,png,gif,webp,avif}, imagesx/y,
  imagecopyresampled, imagejpeg/png/webp/avif, imagerotate, exif_read_data?
  (site-health), getimagesize (già?), image_type_to_mime_type.
- Harness: `wp media import` + pagina upload.php + srcset nelle pagine
  frontend; site-health "gd" verde.
- Poi (ordine roadmap): divergenze SAPI residue (chunked body,
  headers_sent oltre 4096, PHP_CLI_SERVER_WORKERS, …) → WP core test
  suite (PHPUnit) come gate per nome → perf profonda (churn Zval/COW,
  arena per-request, interning).

## Lezioni operative (cumulative, aggiornate WP-8)
- ⭐ WP-8: harness e baseline SEMPRE su disco esterno (`wp8-harness/`), MAI
  solo in /private/tmp: il reboot li azzera (persi i workspace WP-5/ORM/HK
  storici; ricostruiti da ricetta in ~15 min).
- ⭐ WP-8: probe-FIRST paga — 6/10 probe byte-id al primo colpo; i 4 fix
  emersi (DriverError→2002, sqlstate connect HY000, NUM_FLAG client-side,
  SET NAMES per charsetnr) erano tutti INVISIBILI senza probe.
- ⭐ WP-8: per il diff HTTP oracle-vs-phpr, servire lo STESSO albero+DB in
  SEQUENZA sulla stessa porta = byte-parity senza normalizzazioni (le due
  install separate divergono solo su cron/timestamp/hash-di-path e la
  SECONDA login crea la sessione doppia → "log out everywhere" in profile).
- ⭐ WP-8: `mysql -e "…"` di brew ok; mysqld si inizializza con
  `--initialize-insecure` e NON supporta socket su exFAT.
- ⭐ WP-7: estrazione fail-set con `^--- (.+\.phpt) ---$` (path con SPAZI),
  conteggio>0 obbligatorio prima del verdetto.
- ⭐ WP-6: binario ricompilato dopo il lancio dei gate → gate da RILANCIARE.
- ⭐ WP-2/4/5/6: pgrep dopo ogni pkill E lsof sulla porta prima di rilanciare.
- ⭐ WP-3: PROFILARE prima di ottimizzare (`sample <pid>`).
- df PRIMA dei run pesanti; gate per NOME sempre; RTK collassa i body PHP
  (Write/Read tool); zsh non espande i glob dentro variabili.

## Invarianti (identici)
- Gate per OGNI commit: probe byte-id vs oracle · corpus per NOME ·
  ext/session+date+reflection per nome · ORM (**3E/13F**) se
  ref/arg/reflection · **http-kernel 0E/0F** · cargo test ·
  batteria SAPI 48 probe + 8 pagine WP se si tocca server/websapi ·
  batteria admin 12 pagine + pretty 10 rotte se si tocca engine-core ·
  probe mysqli 11/11 se si tocca mysqli/prelude.
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED; Serena
  per Rust, Vexp per il C di php-8.5.7; Read tool per i .php; log con
  `LC_ALL=C tr -d '\0'`.
