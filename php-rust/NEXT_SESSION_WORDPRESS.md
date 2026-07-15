# Rotta WORDPRESS-FIRST — WP-track (ext/gd & media chiusa in WP-9)

> 🏁 **WP-9 CHIUSA (2026-07-15)**: **ext/gd sulla LIBGD DI SISTEMA via FFI +
> ext/exif — media pipeline WordPress a PARITÀ BYTE TOTALE**: 11/11 probe
> gd/exif byte-id, media-probe (sideload+subsizes+srcset+EXIF/IPTC+editor
> ops+conversioni webp/avif) byte-id **inclusi gli md5 dei file generati**,
> batteria HTTP 32/32 risposte byte-identiche SENZA normalizzazione
> (site-health e upload webp/avif chiusi). Dettaglio nel changelog di
> `PHPR_DIVERGENCES_FROM_PHP.md` (WordPress-9, §2.6).

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap (decisione 2026-07-13,
memoria `php-rust-roadmap-wp-first`)**: obiettivo primario = 100%
compatibilità WordPress. Laravel solo come validazione posteriore.

## Cosa è entrato (sessione WP-9 — sintesi; dettaglio nel changelog)
1. `php-types/src/gdio.rs` + `build.rs`: FFI alla libgd brew (la STESSA dylib
   dell'oracle → codec identici → byte-parity dei file), GdImg RAII,
   error-callback va_list→vsnprintf; `vm/gd.rs`: 25 host builtin `__gd_*`,
   handle in `Vm.gd_images`; 6° prelude `prelude_gd.php` (GdImage + ~60 fn);
   ~90 costanti; classe opaca engine-level (`is_opaque_handle_class`).
2. `php-builtins/exif.rs`: exif_read_data (IFD0/EXIF/GPS/THUMBNAIL/COMPUTED/
   COMMENT), exif_imagetype, iptcparse; getimagesize: AVIF + `&$image_info`
   (HOST_OUT + pair-builtin); `__notice_from_caller`; strtotime datenocolon/
   timenocolon (IPTC → created_timestamp).
3. Harness persistente `/Volumes/Extreme Pro/Claude/wp9-harness/`:
   `gd-probe/` (11 probe + assets deterministici — MAI rigenerare gli assets:
   FileDateTime=mtime pinnato nei ref), `run-media.sh` (pipeline media con
   reset DB via dump), `run-http.sh` (batteria HTTP sequenziale con extra
   site-health?tab=debug e media-new), `gate.sh` (gate integrale con
   ricostruzione baseline per NOME da worktree del commit gated).

## Ambiente/harness (per riprodurre — ⚠️ il reboot svuota /private/tmp!)
- MySQL 9.7.1 brew, datadir persistente: avvio identico a WP-8 (vedi sotto);
  DB: wp_o (install oracle, pretty ON) usato da wp9-harness/wp-mo.
  `/opt/homebrew/opt/mysql/bin/mysqld --datadir="/Volumes/Extreme Pro/Claude/mysql-wp8/data"
  --tmpdir="/Volumes/Extreme Pro/Claude/mysql-wp8/tmp" --port=3306
  --bind-address=127.0.0.1 --socket=/private/tmp/mysql-wp8.sock
  --log-error="/Volumes/Extreme Pro/Claude/mysql-wp8/mysqld.err" &`
- Albero WP: `wp9-harness/wp-mo` (wp-config a mano su wp_o; niente wp-cli:
  la media-probe usa media_handle_sideload da CLI). Sorgente WP 7.0.1 anche
  in `wp9-harness/wp-src/` (zip dalla cache wp-cli).
- Batterie WP-8 ancora valide: `wp8-harness/battery.sh` (riusata da
  run-http.sh) e `wp8-harness/mysqli-probe/run-probes.sh`.

## Stato gate (post WP-9) — fail-set per NOME vs baseline da99e8f ricostruita
- corpus / sess / date / refl: vedi `wp9-harness/gate-out/progress.txt`
  (baseline dal worktree del commit gated, target separata su disco esterno).
- ORM 3484 3E/13F nei residui catalogati · hk (tip) 1665 0E/0F · cargo verde ·
  probe mysqli 11/11 · probe gd 11/11 · media-probe byte-id · HTTP 32/32.
- Misura nuova: suite phpt ext/gd ed ext/exif (report in gate-out/*.log —
  molte fail attese: superfici non implementate tipo ttf/filter/arc).

## Prossimo passo: SESSIONE WP-10 (ordine roadmap)
1. **Divergenze SAPI residue** (chiusura tappa server): chunked request body,
   headers_sent() oltre output_buffering=4096, PHP_CLI_SERVER_WORKERS,
   router `return false`, escape `"\u{...}"` lexer (residui WP-4).
2. **WP core test suite (PHPUnit) come gate per nome del filone** — il vero
   moltiplicatore di copertura: harness `wordpress-develop` + `wp-tests-config`,
   richiede probabilmente mysqli TEST_DB + fixture; partire con un sottoinsieme
   (tests/phpunit/tests/media in primis, ora che gd c'è).
3. Poi: perf profonda per-request (churn Zval/COW, arena, interning — piste
   profilate in WP-7), Laravel validazione.

## Lezioni operative (cumulative, aggiornate WP-9)
- ⭐ WP-9: **quando la libreria C dell'oracle è sul sistema, FFI > crate**
  (pattern zlibio→gdio): byte-parity gratis e semantica esatta con wrapper
  sottili; il crate `image` avrebbe dato solo functional-parity a costo alto.
- ⭐ WP-9: probe-FIRST ancora vincente — 5/11 byte-id al primo colpo; i diff
  emersi (speed AVIF -1→6 mappato da PHP non da gd, IMAGETYPE_COUNT 22,
  Notice "Error reading from", forma opaca GdImage) erano tutti invisibili
  senza probe. exif_read_data byte-id al primo colpo dopo il port del subset.
- ⭐ WP-9: le classi-handle opache PHP 8 (GdImage & co.) hanno UNA superficie
  engine trasversale: clone/serialize/dump/export/json/reflection — helper
  condiviso in php-types + name-check nei ~7 siti; var_dump si risolve con un
  debug-info vuoto sintetico nella mappa di compute_debug_info.
- ⭐ WP-9: assets probe con mtime nei riferimenti (exif FileDateTime) = MAI
  rigenerarli; il redirect `2>"$dir/log"` dentro `(cd … && …)` vuole il path
  ASSOLUTO; mysqldump con GTID vuole `--set-gtid-purged=OFF` per il restore.
- ⭐ WP-8: harness/baseline su disco esterno; per il gate per NOME senza
  baseline superstite: worktree del commit gated + CARGO_TARGET_DIR dedicata.
- ⭐ WP-7: estrazione fail-set con path con SPAZI, conteggio>0 obbligatorio.
- ⭐ WP-6: binario ricompilato dopo il lancio dei gate → gate da RILANCIARE.
- ⭐ WP-2/4/5/6: pgrep dopo ogni pkill E lsof sulla porta prima di rilanciare.
- df PRIMA dei run pesanti; gate per NOME sempre; RTK collassa i body PHP
  (Write/Read tool); zsh non espande i glob dentro variabili.

## Invarianti (identici + nuovi WP-9)
- Gate per OGNI commit: probe byte-id vs oracle · corpus per NOME ·
  ext/session+date+reflection per nome · ORM (**3E/13F**) se
  ref/arg/reflection · **http-kernel 0E/0F** · cargo test ·
  batteria SAPI/pagine WP se si tocca server/websapi · batteria admin+pretty
  se si tocca engine-core · probe mysqli 11/11 se si tocca mysqli/prelude ·
  **probe gd 11/11 + media-probe + run-http 32/32 se si tocca
  gd/exif/image/prelude_gd o i codec** (e dopo ogni upgrade brew di gd!).
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED; Serena
  per Rust, Vexp per il C di php-8.5.7; Read tool per i .php; log con
  `LC_ALL=C tr -d '\0'`.
