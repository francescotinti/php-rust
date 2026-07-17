# Rotta WORDPRESS-FIRST — WP-track (restapi A PARITÀ ORACLE in WP-14)

> 🏁 **WP-14 (2026-07-17)**: **gruppo RESTAPI a parità oracle — 3514 test,
> 1E phpr = 1E oracle** (oEmbed `test_proxy_with_classic_embed_provider`:
> bug upstream del trunk wpdev, stesso messaggio e stessa riga su entrambi).
> I 19E di WP-13 chiusi con 4 feature: **(1) mbstring HTML-ENTITIES**
> (tabella HTML4 condivisa `php_types::html4`, deprecation con cache
> last_used à la php_mb_get_encoding, mb_list_encodings, mb_check_encoding
> allargato); **(2) DOMDocument::loadHTML/loadHTMLFile/saveHTML/saveHTMLFile**
> (modalità HTML4/libxml2 di parse_html: doctype transitional implicito,
> `<p>` implicito per testo vagante, script→CDATA, attr valueless=nome,
> PI SGML + trick `<?xml encoding>`, charset BOM>xmldecl>meta>Latin-1,
> nodeType 13, errori libxml strutturati con posizioni, serializer fedele
> con entity HTML4 + URI-escape su href/src/action — **probe 514 righe
> byte-identiche**); **(3) ext/xml SAX** (`__xml_tokenize` su quick-xml +
> API xml_parser_* nel prelude, codici errore libxml compat-layer
> oracle-pinned: 5/76/26/9, SKIP_WHITE no-op, DOCTYPE `<!ENTITY>` risolte,
> ns con separator — SimplePie/WP_Widget_RSS renderizza il feed reale);
> **(4) is_callable con AUTOLOAD** (zend_is_callable). Dettagli: changelog
> WordPress-14 in `PHPR_DIVERGENCES_FROM_PHP.md`.

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
mysql -h 127.0.0.1 -u root -e "CREATE DATABASE IF NOT EXISTS wptests; CREATE USER IF NOT EXISTS 'wp'@'%' IDENTIFIED BY 'wp-secret-Pass1'; GRANT ALL ON wptests.* TO 'wp'@'%';"
perl -pe "s/youremptytestdbnamehere/wptests/; s/yourusernamehere/wp/; s/yourpasswordhere/wp-secret-Pass1/; s/define\( 'DB_HOST', 'localhost' \)/define( 'DB_HOST', '127.0.0.1:3306' )/" \
  wp-tests-config-sample.php > wp-tests-config.php
# gruppi a parità (bootstrap INSTALLA wptests a ogni run ⇒ SEQUENZIALI,
# ⚠️ e MAI probe che scrivono su wptests durante una run!):
$ORACLE vendor/bin/phpunit --group option  # 413            → phpr IDENTICO (WP-10)
$ORACLE vendor/bin/phpunit --group media   # 762, 52S       → phpr IDENTICO (WP-11/12)
$ORACLE vendor/bin/phpunit --group post    # 906, 1S        → phpr IDENTICO (WP-12)
$ORACLE vendor/bin/phpunit --group user    # 1341, 5W+1S    → phpr IDENTICO (WP-12)
$ORACLE vendor/bin/phpunit --group query   # 1889 OK        → phpr IDENTICO (WP-12)
$ORACLE vendor/bin/phpunit --group restapi # 3514, 1E/4W/6S → phpr 1E IDENTICO (WP-14)
# diff per nome: --log-junit + estrazione testcase name/class
```
- ⚠️ **il trunk di wordpress-develop CAMBIA tra un clone e l'altro**: rifare
  SEMPRE la baseline oracle sul clone fresco, mai fidarsi dei numeri a memoria.
- **gate14.sh riusabile**: `/Volumes/Extreme Pro/Claude/wp14-harness/gate14.sh`
  (4 suite phpt per nome vs worktree baseline `/private/tmp/wp14-base` HEAD
  `9cf4f3b` + ORM + hk + cargo + batterie gd/mysqli/media/http + WP
  option/media/post/user/query + restapi + misura fileinfo/gd/exif).
- ⚠️ **WATCHDOG ANTI-HANG OBBLIGATORIO per ogni run PHPUnit lunga**:
  `/Volumes/Extreme Pro/Claude/wp14-harness/run-with-watchdog.sh
  [-t total_s] [-s stall_s] [-p progress_file] [-o outdir] -- cmd…`
  (sample pre-kill, exit 124; macOS non ha timeout(1)). Già cablato in
  gate14.sh. Baseline worktree: `git worktree add /private/tmp/wpN-base HEAD`
  + copiare crates/php-server (gitignorato!) e Cargo.lock DENTRO php-rust/,
  build con CARGO_TARGET_DIR esterno. Workspace ORM/HK: `/private/tmp/
  wp11-gates/` (tarball sorgente in wp9-harness/gates/).
- mysqld: `/opt/homebrew/opt/mysql/bin/mysqld --datadir="/Volumes/Extreme Pro/Claude/mysql-wp8/data"
  --tmpdir="/Volumes/Extreme Pro/Claude/mysql-wp8/tmp" --port=3306
  --bind-address=127.0.0.1 --socket=/private/tmp/mysql-wp8.sock
  --log-error="/Volumes/Extreme Pro/Claude/mysql-wp8/mysqld.err" &`

## Prossimo passo: SESSIONE WP-15 = gruppi successivi + perf
1. **Gruppi successivi**: taxonomy, comment, xmlrpc, multisite (stessa
   ricetta: baseline oracle sul clone fresco → phpr → triage per nome sui
   junit). Poi verso la SUITE INTERA come gate per nome.
2. **Perf restapi** (in WP-13 ~6.5 min vs 150s oracle = 2.7x): gap ~43s tra
   fine addTestFile e primo test (fase pre-run PHPUnit) da profilare; il
   grosso resta il churn generico run_loop/hook (`ho_call_user_func`).
3. **Provider full-suite noti** (ErrorTestCase): `mb_convert_encoding` BIG-5
   (Tests_DB_Charset) · throw in data_wp_validate_site_data via current_time
   (Tests_Multisite_Site) · 1 dataset wpIsIniValueChangeable.
4. **Bug engine aperto (WP-14)**: `static $x = array(…)` dentro una funzione
   del PRELUDE panica (index out of bounds run.rs:233); worked around in
   xml_error_string con array non-static. Da investigare (userland OK).
5. Divergenze documentate da tenere d'occhio: colonne errori libxml con
   input high-byte non-UTF-8; mb_strlen &c. batch-1 rifiutano HTML-ENTITIES
   (ValueError rumoroso); surrogati HTML-ENTITIES → `?`.

## Lezioni operative (nuove WP-14)
- ⭐ **Metodo spec-first per le feature "work-alike C"**: probe oracle RICCO
  (tree-walk + serializzazione + errori + edge) PRIMA di implementare = la
  spec; iterare fino a diff vuoto. loadHTML chiuso con 2 round di probe
  (514 righe byte-id); ext/xml con 1 round + 4 fix.
- ⚠️ ext/xml di PHP NON è expat: è la compat-layer libxml (ext/xml/compat.c)
  — codici errore xmlParserErrors, stringhe da error_mapping[], SKIP_WHITE
  no-op. Pinnare SEMPRE i codici con un probe oracle, non dai .h di expat.
- ⚠️ `libxml_use_internal_errors(false)` SVUOTA il buffer errori: senza
  questo dettaglio gli errori si accumulano tra load successivi.
- La deprecation mbstring "Handling HTML entities…" warna UNA volta per nome
  (cache last_used_encoding di php_mb_get_encoding), MAI sul lato FROM.
- `is_callable(['C','m'])` deve AUTOLOADARE C (SimplePie Registry asserta
  su classi lazy).

## Invarianti (aggiornati WP-14)
- Gate per OGNI commit: corpus/sess/date/refl per NOME (baseline worktree
  HEAD) · ORM 3E/13F per nome · http-kernel 1665 0E/0F · cargo · probe:
  gd 11/11, mysqli 11/11, media-probe byte-id, run-http · **WP suite:
  option 413 = oracle · media 762 (52S = oracle) · post 906 · user 1341 ·
  query 1889 tutti = oracle · restapi 3514: 1E = 1E oracle (oEmbed
  upstream condiviso)** · misura ext/fileinfo (29P/25F/8S).
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED sotto
  watchdog; Serena per Rust, Vexp/Read per il C; Read/Write tool per i
  .php; log `tr -d '\0'`.
