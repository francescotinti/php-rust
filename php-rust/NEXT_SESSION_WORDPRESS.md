# Rotta WORDPRESS-FIRST — WP-track (restapi SBLOCCATO in WP-13)

> 🏁 **WP-13 (2026-07-17)**: **gruppo RESTAPI sbloccato — da "NON TERMINA"
> (>50 min CPU, junit vuoto) a 3514 test in ~6.5 min (2.7x oracle, in linea
> con gli altri gruppi), 19E residui TUTTI = feature mancanti note.**
> Root cause dell'hang: **l'assegnamento composto valutava il target PRIMA
> del RHS** (Zend: RHS prima, poi read-modify-write) — il loop di
> `validate_custom_css` (`$at += strcspn($css,'<',++$at)`) oscillava per
> sempre sul valore stantio. Fix su slot/prop/GLOBALS/superglobali
> (Op::Swap; i path dim erano già corretti). Altri fix: deref dei Ref nei
> Value-builtin (callee dinamico ⇒ prefer-ref rompeva `is_*` in
> `rest_get_best_type_for_value`), `array_unique` onora `__toString`
> (dipendenze PHPUnit `ExecutionOrderDependency` collassate), cast
> string→int SATURANTE (`zend_dval_to_lval_cap`), `gethostbyname`/
> `gethostbynamel` (24 fail: `wp_http_validate_url`),
> `SimpleXMLElement::addChild`/`addAttribute` + `asXML` con dichiarazione
> XML sull'elemento radice. Perf: cache `__reflect_method_info`
> (build suite PHPUnit 60s→21s), compare alloc-free, fast-path
> `compute_stringify`. Dettagli: changelog WordPress-13 in
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
$ORACLE vendor/bin/phpunit --group option  # 413            → phpr IDENTICO (WP-10)
$ORACLE vendor/bin/phpunit --group media   # 762, 52S       → phpr IDENTICO (WP-11/12)
$ORACLE vendor/bin/phpunit --group post    # 906, 1S        → phpr IDENTICO (WP-12)
$ORACLE vendor/bin/phpunit --group user    # 1341, 5W+1S    → phpr IDENTICO (WP-12)
$ORACLE vendor/bin/phpunit --group query   # 1889 OK        → phpr IDENTICO (WP-12)
$ORACLE vendor/bin/phpunit --group restapi # 3514, oracle ~150s 1E/4W/6S
#   → phpr ~6.5min, 19E = SOLO feature mancanti (vedi WP-14 sotto) (WP-13)
# diff per nome: --log-junit + estrazione testcase name/class
```
- ⚠️ **il trunk di wordpress-develop CAMBIA tra un clone e l'altro** (WP-13:
  oracle restapi 149s/1E vs i ~30s/1E/1F annotati in WP-12): rifare SEMPRE
  la baseline oracle sul clone fresco, mai fidarsi dei numeri a memoria.
- **gate13.sh riusabile**: `/Volumes/Extreme Pro/Claude/wp13-harness/gate13.sh`
  (4 suite phpt per nome vs worktree baseline + ORM + hk + cargo + batterie
  gd/mysqli/media/http + WP option/media/post/user/query + restapi + misura
  fileinfo). Baseline worktree: `git worktree add /private/tmp/wpN-base HEAD`
  + copiare crates/php-server (gitignorato!) e Cargo.lock DENTRO php-rust/,
  build con CARGO_TARGET_DIR esterno. Workspace ORM/HK: `/private/tmp/
  wp11-gates/` (tarball sorgente in wp9-harness/gates/).
- mysqld: `/opt/homebrew/opt/mysql/bin/mysqld --datadir="/Volumes/Extreme Pro/Claude/mysql-wp8/data"
  --tmpdir="/Volumes/Extreme Pro/Claude/mysql-wp8/tmp" --port=3306
  --bind-address=127.0.0.1 --socket=/private/tmp/mysql-wp8.sock
  --log-error="/Volumes/Extreme Pro/Claude/mysql-wp8/mysqld.err" &`

## Prossimo passo: SESSIONE WP-14 = i 19E residui di restapi, poi gruppi residui
1. **`DOMDocument::loadHTML`/`loadHTMLFile`** (16 fail: Widgets 14,
   Sidebars 1, get_items_edit_context) — serve un parser HTML work-alike
   libxml2 (wrapping implicito html/body, DOCTYPE, tag-soup). È il pezzo
   grosso. I test lo usano via assertEqualMarkup/normalize dei markup widget.
2. **ext/xml (xml_parser_create & C.)** (1 fail: SimplePie via
   `WP_Widget_RSS`) — parser SAX-style: xml_parser_create,
   xml_set_element_handler, xml_parse, xml_parser_free…
3. **mbstring codec `HTML-ENTITIES`** (2 fail: Schema_Validation
   min/max_length via mb_convert_encoding(…, 'HTML-ENTITIES')).
4. **oEmbed `test_proxy_with_classic_embed_provider`**: fallisce ANCHE
   sull'oracle (1E suo, ragione diversa: da phpr "Attempt to read property
   'queue' on null") — verificare se è flake esterno o divergenza vera.
5. **Perf restapi** (6.5 min vs 150s oracle = 2.7x): gap ~43s tra fine
   addTestFile e primo test (fase pre-run PHPUnit) ancora da profilare;
   il grosso resta il churn generico run_loop/hook (`ho_call_user_func`).
6. **Gruppi successivi**: taxonomy, comment, xmlrpc, multisite? (stessa
   ricetta: baseline oracle → phpr → triage per nome sui junit).
7. **Provider full-suite noti** (ErrorTestCase): `mb_convert_encoding` BIG-5
   (Tests_DB_Charset) · throw in data_wp_validate_site_data via current_time
   (Tests_Multisite_Site) · 1 dataset wpIsIniValueChangeable.
8. La suite INTERA come gate per nome quando i gruppi core sono verdi.

## Lezioni operative (nuove WP-13)
- ⚠️ **phpr bufferizza stdout internamente anche su pty**: `script -q` NON
  dà output live (log a 0 byte fino all'exit). Per il progresso live usare
  MARKER FILE-BASED env-gated iniettati nell'harness PHP:
  `if(getenv("X")) file_put_contents(getenv("X"), ..., FILE_APPEND)` in
  bootstrap (stage), TestSuite::addTestFile (per-file),
  abstract-testcase set_up/set_up_before_class (per-test/classe),
  wp_hash_password (per-hash). Sono la scala di zoom con cui si è trovato
  il test dell'hang.
- ⭐ **Metodo hang→test**: sample (Rust, dice il COSA: call_user_func churn)
  → marker per-test (dice il DOVE: il test che non finisce) → leggere il
  test → estrarre la primitiva → probe oracle-pinned in 20 righe → il probe
  del LOOP divergeva mentre le primitive combaciavano ⇒ semantica del
  costrutto (assign-op), non della funzione.
- ⚠️ I numeri oracle dei memo invecchiano col trunk wpdev: rifare la
  baseline a ogni clone.
- `sample` su `pgrep -f` può agganciare il wrapper `script` invece di phpr:
  usare `pgrep -x phpr`.
- I fail "Call to undefined function/method X" nel junit = la mappa esatta
  delle feature mancanti (correct-or-absent che paga): 49E → 6 cause in
  una passata di triage.

## Invarianti (aggiornati WP-13)
- Gate per OGNI commit: corpus/sess/date/refl per NOME (baseline worktree
  HEAD) · ORM 3E/13F per nome · http-kernel 1665 0E/0F · cargo · probe:
  gd 11/11, mysqli 11/11, media-probe byte-id, run-http · **WP suite:
  option 413 = oracle · media 762 (52S = oracle) · post 906 · user 1341 ·
  query 1889 tutti = oracle · restapi 3514: 19E documentati (loadHTML 16,
  xml_parser 1, HTML-ENTITIES 2 — più 1E condiviso con l'oracle)** ·
  misura ext/fileinfo (29P/25F/8S).
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED; Serena
  per Rust, Vexp/Read per il C; Read/Write tool per i .php; log `tr -d '\0'`.
