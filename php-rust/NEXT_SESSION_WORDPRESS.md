# Rotta WORDPRESS-FIRST — WP-track (SAPI residue chiuse + WP core suite avviata in WP-10)

> 🏁 **WP-10 (2026-07-15)**: **residui SAPI chiusi (15/15 probe byte-id:
> chunked body, router return false oracle-esatto, headers_sent alla soglia
> 4096, \u{...}, WORKERS smoke) + WP CORE TEST SUITE AVVIATA: gruppo option
> A PARITÀ ORACLE (413 test 0E/0F)** dopo 2 fix engine chiave (dispatch
> privato NON-virtuale à la Zend + base statica nei write-target) + getopt +
> timezone ALL_WITH_BC. Gruppo media prima passata: 680 test/11F + 82 non
> caricati → il triage è il lavoro di WP-11. Dettaglio nel changelog di
> `PHPR_DIVERGENCES_FROM_PHP.md` (WordPress-10).

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap**: obiettivo primario =
100% compatibilità WordPress; la WP core test suite (PHPUnit) è ora il
GATE PER NOME del filone. Laravel dopo.

## Harness WP core test suite (ricetta — ⚠️ vive nello scratchpad di sessione!)
```bash
SP=<scratchpad>; ORACLE=/opt/homebrew/opt/php/bin/php
git clone --depth 1 https://github.com/WordPress/wordpress-develop.git "$SP/wpdev"
cd "$SP/wpdev" && $ORACLE composer.phar install --no-interaction --no-scripts
# mysqld WP-8 su (datadir /Volumes/Extreme Pro/Claude/mysql-wp8/data, vedi sotto)
mysql -h 127.0.0.1 -u root -e "CREATE DATABASE IF NOT EXISTS wptests; GRANT ALL ON wptests.* TO 'wp'@'%';"
perl -pe "s/youremptytestdbnamehere/wptests/; s/yourusernamehere/wp/; s/yourpasswordhere/wp-secret-Pass1/; s/define\( 'DB_HOST', 'localhost' \)/define( 'DB_HOST', '127.0.0.1:3306' )/" \
  wp-tests-config-sample.php > wp-tests-config.php
# baseline oracle e run phpr (il bootstrap INSTALLA WP in wptests a ogni run):
$ORACLE vendor/bin/phpunit --group option   # 413 test, 1W+3S
phpr    vendor/bin/phpunit --group option   # IDENTICO (WP-10)
```
- composer.phar in `wp9-harness/gates/composer.phar`; workspace gate ORM/HK
  pronti come tarball in `wp9-harness/gates/{orm-work,hk-work}.tgz`
  (scompattare su APFS, MAI su exFAT).
- ⚠️ phpr sul gruppo option: ~88s vs 5.5s oracle (16×) — la perf della
  suite è un tema WP-11+ (profilare con `sample`, lezione WP-3).
- mysqld: `/opt/homebrew/opt/mysql/bin/mysqld --datadir="/Volumes/Extreme Pro/Claude/mysql-wp8/data"
  --tmpdir="/Volumes/Extreme Pro/Claude/mysql-wp8/tmp" --port=3306
  --bind-address=127.0.0.1 --socket=/private/tmp/mysql-wp8.sock
  --log-error="/Volumes/Extreme Pro/Claude/mysql-wp8/mysqld.err" &`

## Prossimo passo: SESSIONE WP-11 = triage WP core suite, gruppo media poi avanti
1. **Gruppo media**: 11 failures + **82 test NON CARICATI** (762 oracle vs
   680 phpr: probabilmente data-provider che falliscono in silenzio o file
   che non compilano — cercare con PHPR_LOG=warn i "lower failed", lezione
   WP-10: il VERO errore sta nel log, non nel messaggio phpunit).
2. Poi gruppi in ordine di valore: post, user, query, rest-api, xmlrpc.
3. La suite INTERA come gate per nome quando i gruppi core sono verdi.
4. Perf suite (16× oracle): profilare prima (sample), niente ottimizzazioni
   alla cieca.

## Lezioni operative (nuove WP-10)
- ⭐ Il messaggio "Failed to compile X" di phpunit NASCONDE l'errore vero:
  `PHPR_LOG=warn PHPR_LOG_FILE=... phpr …` → "lower failed for …:
  Unsupported { what, line }" (logging.rs, target phpr::include).
- ⭐ Dispatch privato: Zend ri-lega `$this->m()` alla private dello scope
  chiamante se è antenato della classe dell'oggetto (ACC_CHANGED gate) —
  `parent_private_rebind` in vm/oop.rs, applicato SOLO al dispatch istanza.
- ⭐ printf di zsh mangla `\u`/`\0` negli heredoc: i .php di probe SEMPRE
  col Write tool (conferma lezione RTK).
- ⭐ oracle `php -S` col router che echoa prima di `return false` produce
  risposte MALFORMATE sugli statici (noise raw pre-status-line): fedeltà =
  riprodurre anche questo; curl la scarta, usare `nc` per pinnarla.
- Probe getopt: stop al PRIMO non-option (tutto ciò che segue è scartato),
  ripetute→array di valori, `x::` solo attached, long `=` o separato.

## Invarianti (aggiornati WP-10)
- Gate per OGNI commit: corpus/sess/date/refl per NOME · ORM 3E/13F se si
  tocca ref/arg/reflection/dispatch · http-kernel 0E/0F (workspace CON
  symfony/phpunit-bridge!) · cargo · probe: gd 11/11, mysqli 11/11,
  sapi WP-10 15/15, media-probe byte-id, run-http 32/32 ·
  **WP core suite gruppo option 413 = oracle** (nuovo invariante).
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED; Serena
  per Rust, Vexp per il C; Read/Write tool per i .php; log `tr -d '\0'`.
