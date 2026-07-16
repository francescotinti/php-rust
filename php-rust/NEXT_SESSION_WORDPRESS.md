# Rotta WORDPRESS-FIRST — WP-track (gruppo media a parità oracle in WP-11)

> 🏁 **WP-11 (2026-07-16)**: **WP core suite, GRUPPO MEDIA A PARITÀ ORACLE:
> 762/762 test, 0E/0F** (da 680/11F + 82 non caricati) con 10 fix engine
> trasversali — i tre grossi: **docblock stile Zend** (CG(doc_comment)
> sopravvive agli statement: era il bug degli 82 test invisibili a PHPUnit),
> **`@` a semantica BEGIN/END_SILENCE esatta** (error_level mascherato &=4437,
> handler chiamato sotto @, restore condizionale anche sull'unwind — fixa pure
> 7 phpt del corpus error_reporting), **stream wrapper userland oltre fopen**
> (url_stat per la famiglia stat + getimagesize/exif via wrapper). Poi:
> getimagesize TIFF/JP2/PSD/ICO/JPC, exif COMPUTED.UserComment/Aperture/INTEROP,
> return-by-ref di `self::$arr[$k]`, http(s):// nei builtin immagine,
> 4 INI site-health (memory_limit 128M settable, max_input_vars,
> max_execution_time/max_input_time SAPI-swap). Residuo media: 12 skip da
> **ext/fileinfo assente** (l'oracle li passa). Dettaglio nel changelog di
> `PHPR_DIVERGENCES_FROM_PHP.md` (WordPress-11).

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap**: obiettivo primario =
100% compatibilità WordPress; la WP core test suite (PHPUnit) è il GATE PER
NOME del filone. Laravel dopo.

## Harness WP core test suite (ricetta — ⚠️ vive nello scratchpad di sessione!)
```bash
SP=<scratchpad>; ORACLE=/opt/homebrew/opt/php/bin/php
git clone --depth 1 https://github.com/WordPress/wordpress-develop.git "$SP/wpdev"
cd "$SP/wpdev" && cp "/Volumes/Extreme Pro/Claude/wp9-harness/gates/composer.phar" . \
  && $ORACLE composer.phar install --no-interaction --no-scripts
# mysqld WP-8 su (datadir /Volumes/Extreme Pro/Claude/mysql-wp8/data, vedi sotto)
mysql -h 127.0.0.1 -u root -e "CREATE DATABASE IF NOT EXISTS wptests; GRANT ALL ON wptests.* TO 'wp'@'%';"
perl -pe "s/youremptytestdbnamehere/wptests/; s/yourusernamehere/wp/; s/yourpasswordhere/wp-secret-Pass1/; s/define\( 'DB_HOST', 'localhost' \)/define( 'DB_HOST', '127.0.0.1:3306' )/" \
  wp-tests-config-sample.php > wp-tests-config.php
# gruppi a parità (il bootstrap INSTALLA WP in wptests a ogni run, quindi SEQUENZIALI):
$ORACLE vendor/bin/phpunit --group option   # 413, 1W+3S     → phpr IDENTICO (WP-10)
$ORACLE vendor/bin/phpunit --group media    # 762, 52S       → phpr 762 0E/0F, 64S (WP-11;
                                            #   +12 skip = @requires extension fileinfo)
```
- workspace gate ORM/HK: tarball in `wp9-harness/gates/{orm-work,hk-work}.tgz`
  (scompattare su APFS es. /private/tmp/wp11-gates, MAI su exFAT).
- **gate11.sh riusabile**: `/Volumes/Extreme Pro/Claude/wp11-harness/gate11.sh`
  (4 suite phpt per nome vs worktree baseline + ORM + hk + cargo + batterie
  gd/mysqli/media/http + WP option/media). Baseline worktree:
  `git worktree add /private/tmp/wpN-base HEAD` + copiare crates/php-server
  (gitignorato!) e Cargo.lock DENTRO php-rust/, build con CARGO_TARGET_DIR
  esterno. ⚠️ La ROOT git è php-rust-experiment: il worktree ha il progetto in
  `<worktree>/php-rust/`.
- mysqld: `/opt/homebrew/opt/mysql/bin/mysqld --datadir="/Volumes/Extreme Pro/Claude/mysql-wp8/data"
  --tmpdir="/Volumes/Extreme Pro/Claude/mysql-wp8/tmp" --port=3306
  --bind-address=127.0.0.1 --socket=/private/tmp/mysql-wp8.sock
  --log-error="/Volumes/Extreme Pro/Claude/mysql-wp8/mysqld.err" &`

## Prossimo passo: SESSIONE WP-12 = ext/fileinfo, poi gruppi successivi
1. **ext/fileinfo** (12 skip media + `@requires extension fileinfo` sparsi
   nella suite): valutare port del subset libmagic che serve a WP
   (wp_check_filetype_and_ext → finfo_file sui mime immagine/documento) o FFI
   a libmagic — ⚠️ l'oracle brew usa la libmagic BUNDLED di PHP con database
   patchato, non quella di sistema: byte-parity solo con port del database
   PHP (`ext/fileinfo/data_file.c`) o pin sui mime che WP tocca.
2. **Gruppi successivi in ordine di valore**: post, user, query, rest-api,
   xmlrpc (stessa ricetta: baseline oracle → run phpr → triage per nome).
3. **Provider full-suite noti** (ErrorTestCase): `mb_convert_encoding` BIG-5
   (Tests_DB_Charset, 42 dataset persi) · throw in data_wp_validate_site_data
   via current_time (Tests_Multisite_Site) · 1 dataset wpIsIniValueChangeable.
4. La suite INTERA come gate per nome quando i gruppi core sono verdi.
5. Perf suite (media phpr ~102s vs oracle ~28s ≈ 3.6×; option era 16×):
   profilare con `sample` PRIMA di ottimizzare (lezione WP-3).

## Lezioni operative (nuove WP-11)
- ⭐ PHPUnit scopre le classi con `array_slice(get_declared_classes(), ptr)`
  e i GRUPPI dal docblock della classe: un docblock separato dalla classe da
  `require_once` DEVE attaccarsi (CG(doc_comment) Zend). Se un gruppo di test
  "non si carica", controllare getDocComment prima di sospettare il compile.
- ⭐ `--list-tests` di PHPUnit 9 IGNORA `--group`: per contare i test di un
  gruppo servono i junit (`--log-junit`) e il diff per nome si fa lì.
- ⭐ La semantica `@` NON è "drop dei diag": Zend maschera EG(error_reporting)
  a 4437 (BEGIN_SILENCE `&= E_FATAL_ERRORS`) e CHIAMA l'handler; PHPUnit
  scarta su quel valore. Un engine che "salta l'handler sotto @" rompe
  convertWarningsToExceptions (era l'error su test_resize_bad_image).
- ⭐ error_reporting() scritto DENTRO `@` sopravvive alla regione (bug27731);
  il restore END_SILENCE è condizionale (corrente fatal-only ∧ salvato no),
  anche sull'unwind da eccezione (bug33771).
- Il gate su binario ricostruito A METÀ gate è un gate MISTO: rifare sempre
  il pass intero sul binario definitivo (conferma lezione WP-6).
- Riprodurre i test WP fuori da PHPUnit con l'albero wp-mo di wp9-harness
  (wp-load.php + DB wp_o, mysqld su) è il modo più rapido di isolare un
  failure di wp_* (probe10 su wp_crop_image URL).

## Invarianti (aggiornati WP-11)
- Gate per OGNI commit: corpus/sess/date/refl per NOME (baseline worktree
  HEAD) · ORM 3E/13F per nome · http-kernel 1665 0E/0F (workspace CON
  symfony/phpunit-bridge) · cargo · probe: gd 11/11, mysqli 11/11,
  media-probe byte-id, run-http (30/32 byte-id secchi; shdebug/medianew:
  nonce/time volatili + curl_version/opcache assenti = divergenze
  documentate) · **WP suite: option 413 = oracle · media 762 0E/0F
  (64S = 52 oracle + 12 fileinfo)**.
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED; Serena
  per Rust, Vexp/Read per il C; Read/Write tool per i .php; log `tr -d '\0'`.
