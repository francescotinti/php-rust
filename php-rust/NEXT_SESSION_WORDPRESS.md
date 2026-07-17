# Rotta WORDPRESS-FIRST — WP-track (taxonomy/comment/xmlrpc/multisite a parità in WP-15)

> 🏁 **WP-15 (2026-07-17)**: **4 gruppi nuovi a parità oracle sul clone fresco
> (`81b2b5b`): taxonomy 878 (1W = oracle) · comment 582 · xmlrpc 316 ·
> multisite 32**. Sette lavori: **(1) bug static-prelude CHIUSO** (il main
> unit rilowerava gli static id da 0 sopra quelli del prelude → overflow
> run.rs:233; ora ogni unità semina il contatore oltre il range del prelude);
> **(2) gethostbyaddr** (FFI getnameinfo/NI_NAMEREQD su libc di sistema,
> semantica dns.c — chiudeva 22E comment + 2E xmlrpc); **(3) deprecation
> "Passing null to parameter"** sul trim family (helper `null_arg_deprecation`
> riusabile, da estendere per-funzione quando emerge); **(4) DOMNode::C14N/
> C14NFile** (C14N 1.0+exclusive spec-first, 12 casi byte-id) + normalizzazione
> fine-riga XML §2.11 nel parser; **(5) DOMNode::normalize/normalizeDocument**;
> **(6) strtotime "assoluto+relativo"** (`"…14:30:00+10 minutes"` del
> comment-preview WP); **(7) write-chain MagicDescend** (`$o->virtual->x=v` via
> __get; no-autoviv PHP 8.5 con Error/Deprecated/visibilità esatti — chiuso il
> gap storico Bug #34893). Dettagli: changelog WordPress-15 in
> `PHPR_DIVERGENCES_FROM_PHP.md`.

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
$ORACLE vendor/bin/phpunit --group option    # 413          → phpr IDENTICO (WP-10)
$ORACLE vendor/bin/phpunit --group media     # 762, 52S     → phpr IDENTICO (WP-11/12)
$ORACLE vendor/bin/phpunit --group post      # 906          → phpr IDENTICO (WP-12)
$ORACLE vendor/bin/phpunit --group user      # 1341         → phpr IDENTICO (WP-12)
$ORACLE vendor/bin/phpunit --group query     # 1889         → phpr IDENTICO (WP-12)
$ORACLE vendor/bin/phpunit --group restapi   # 3514, 1E     → phpr 1E IDENTICO (WP-14, oEmbed upstream)
$ORACLE vendor/bin/phpunit --group taxonomy  # 878, 1W      → phpr IDENTICO (WP-15)
$ORACLE vendor/bin/phpunit --group comment   # 582          → phpr IDENTICO (WP-15)
$ORACLE vendor/bin/phpunit --group xmlrpc    # 316          → phpr IDENTICO (WP-15)
$ORACLE vendor/bin/phpunit --group multisite # 32 (config single-site) → phpr IDENTICO (WP-15)
# diff per nome: --log-junit + estrazione testcase name/class
```
- ⚠️ **il trunk di wordpress-develop CAMBIA tra un clone e l'altro**: rifare
  SEMPRE la baseline oracle sul clone fresco, mai fidarsi dei numeri a memoria.
- **gate15.sh riusabile**: `/Volumes/Extreme Pro/Claude/wp15-harness/gate15.sh`
  (= gate14 [4 suite phpt per nome vs worktree `/private/tmp/wp14-base` +
  ORM + hk + cargo + batterie gd/mysqli/media/http + WP option/media/post/
  user/query/restapi + misura fileinfo/gd/exif] + i 4 gruppi WP-15 sul clone
  fresco; ⚠️ i gruppi WP-14 girano sul wpdev VECCHIO della sessione WP-11 —
  se sparisce, riclonare e ricreare la baseline).
- ⚠️ **WATCHDOG ANTI-HANG OBBLIGATORIO per ogni run PHPUnit lunga**:
  `/Volumes/Extreme Pro/Claude/wp15-harness/run-with-watchdog.sh
  [-t total_s] [-s stall_s] [-p progress_file] [-o outdir] -- cmd…`
  (sample pre-kill, exit 124; macOS non ha timeout(1)). Già cablato in
  gate15.sh. Baseline worktree: `git worktree add /private/tmp/wpN-base HEAD`
  + copiare crates/php-server (gitignorato!) e Cargo.lock DENTRO php-rust/,
  build con CARGO_TARGET_DIR esterno. Workspace ORM/HK: `/private/tmp/
  wp11-gates/` (tarball sorgente in wp9-harness/gates/).
- mysqld: `/opt/homebrew/opt/mysql/bin/mysqld --datadir="/Volumes/Extreme Pro/Claude/mysql-wp8/data"
  --tmpdir="/Volumes/Extreme Pro/Claude/mysql-wp8/tmp" --port=3306
  --bind-address=127.0.0.1 --socket=/private/tmp/mysql-wp8.sock
  --log-error="/Volumes/Extreme Pro/Claude/mysql-wp8/mysqld.err" &`

## Prossimo passo: SESSIONE WP-16 = verso la SUITE INTERA
1. **Suite intera come gate**: i 10 gruppi gated coprono ~10.7k test; puntare
   alla run COMPLETA di `vendor/bin/phpunit` (default testsuite) oracle vs
   phpr per nome. Provider full-suite noti da chiudere prima (ErrorTestCase):
   `mb_convert_encoding` BIG-5 (Tests_DB_Charset) · throw in
   data_wp_validate_site_data via current_time (Tests_Multisite_Site) · 1
   dataset wpIsIniValueChangeable. Poi triage per nome sui junit.
2. **Multisite VERO**: i 32 test gated girano in config single-site; la suite
   multisite si lancia con `-c tests/phpunit/multisite.xml` (baseline oracle
   da fare — gruppo ms-required).
3. **Perf restapi** (in WP-13 ~6.5 min vs 150s oracle = 2.7x): gap ~43s tra
   fine addTestFile e primo test (fase pre-run PHPUnit) da profilare; il
   grosso resta il churn generico run_loop/hook (`ho_call_user_func`).
4. **Deprecation "Passing null to parameter"**: oggi solo trim family;
   estendere alle altre funzioni interne quando emergono dai test (helper
   `null_arg_deprecation` in php-builtins/src/lib.rs già pronto).
5. Divergenze documentate da tenere d'occhio: colonne errori libxml con
   input high-byte non-UTF-8; mb_strlen &c. batch-1 rifiutano HTML-ENTITIES;
   surrogati HTML-ENTITIES → `?`; C14N senza subsetting $xpath/$nsPrefixes.

## Lezioni operative (nuove WP-15)
- ⭐ **Il metodo spec-first regge anche per C14N**: probe oracle 12 casi →
  implementazione → byte-id al primo build (unico fix: normalizzazione EOL
  §2.11 che mancava nel PARSER, non nel serializzatore. Con input `\r`
  il C14N emetteva `&#xD;` spurio: il bug era a monte).
- ⭐ **Triage "1 gruppo = pochi errori distinti"**: 22E comment erano UNA
  funzione mancante (gethostbyaddr); contare i messaggi distinti PRIMA di
  aprire i singoli test (`grep -A1 "^[0-9]*)" | sort | uniq -c`).
- **Scoprire il messaggio di una deprecation attesa da PHPUnit**: rompere il
  pattern con perl (`ZZZIMPOSSIBLE`) → il failure stampa il messaggio VERO;
  poi `git checkout --` sul file. `--filter` senza file arg (classi con nome
  ≠ filename) + `--group` per non costruire la suite intera.
- **`strtotime("<datetime>+N unit")`**: timelib accetta assoluto+relativo
  concatenati (anche senza spazio); il fallback "prefisso assoluto più lungo
  con resto relativo valido" è monotono (tocca solo input prima false).
- Il write-descend su slot raw assente va SEMPRE deferito al VM (pattern
  AaOp già esistente): il walker free-function non può né chiamare __get né
  leggere allows_dynamic_props. Dopo `call_method_sync` in un drain, i diag
  vanno flushati con `cur_line(top)` esplicito o si attribuiscono all'op dopo.
- mysqld WP-8 sopravvive tra sessioni (pgrep mysqld prima di rilanciarlo).

## Invarianti (aggiornati WP-15)
- Gate per OGNI commit: corpus/sess/date/refl per NOME (baseline worktree
  HEAD) · ORM 3E/13F per nome · http-kernel 1665 0E/0F · cargo · probe:
  gd 11/11, mysqli 11/11, media-probe byte-id, run-http · **WP suite:
  option 413 · media 762 (52S) · post 906 · user 1341 · query 1889 ·
  restapi 3514 (1E oEmbed upstream condiviso) · taxonomy 878 (1W) ·
  comment 582 · xmlrpc 316 · multisite 32 — TUTTI = oracle** · misura
  ext/fileinfo (29P/25F/8S).
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED sotto
  watchdog; Serena per Rust, Vexp/Read per il C; Read/Write tool per i
  .php; log `tr -d '\0'`.
