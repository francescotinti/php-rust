# Rotta WORDPRESS-FIRST — WP-track (post/user/query a parità + ext/fileinfo in WP-12)

> 🏁 **WP-12 (2026-07-17)**: **ext/fileinfo NATIVO** (detector work-alike della
> libmagic bundled 5.46: encoding.c/is_json/is_csv verbatim + firme curate;
> ground truth 849 file di wpdev: MIME_TYPE/MIME/ENCODING **0 diff**, desc
> 846/849; classe `finfo` opaca nel 7° prelude, I/O PHP-side cap 7MB) e
> **WP core suite: gruppi POST (906) / USER (1341) / QUERY (1889) A PARITÀ
> ORACLE** — con option 413 e media 762 (ora 52S = oracle, i 12 skip fileinfo
> spariti). 7 fix engine trasversali: ArrayAccess sulla write-chain
> variable-rooted (sodium BLAKE2b byte-id), compare oggetti property-wise
> (assertEqualSets), ReflectionObject prop dinamiche, **PCRE non-/u
> byte-oriented** (esc_url `[\x80-\xff]`), date_create tz+false
> (get_gmt_from_date era +3h!), grammatica date (textual ordinali, bound
> timelib), parse_url C1→`_` stile BSD. Dettagli nel changelog di
> `PHPR_DIVERGENCES_FROM_PHP.md` (WordPress-12).

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
$ORACLE vendor/bin/phpunit --group option  # 413, 1W+3S    → phpr IDENTICO (WP-10)
$ORACLE vendor/bin/phpunit --group media   # 762, 52S      → phpr IDENTICO (WP-11/12)
$ORACLE vendor/bin/phpunit --group post    # 906, 1S       → phpr IDENTICO (WP-12)
$ORACLE vendor/bin/phpunit --group user    # 1341, 5W+1S   → phpr IDENTICO (WP-12)
$ORACLE vendor/bin/phpunit --group query   # 1889 OK       → phpr IDENTICO (WP-12)
# diff per nome: --log-junit + estrazione testcase name/class
```
- **gate12.sh riusabile**: `/Volumes/Extreme Pro/Claude/wp12-harness/gate12.sh`
  (4 suite phpt per nome vs worktree baseline + ORM + hk + cargo + batterie
  gd/mysqli/media/http + WP option/media/post/user/query + misura fileinfo).
  Baseline worktree: `git worktree add /private/tmp/wpN-base HEAD` + copiare
  crates/php-server (gitignorato!) e Cargo.lock DENTRO php-rust/, build con
  CARGO_TARGET_DIR esterno (root git = php-rust-experiment, progetto in
  `<worktree>/php-rust/`). Workspace ORM/HK: `/private/tmp/wp11-gates/`
  (tarball sorgente in wp9-harness/gates/).
- mysqld: `/opt/homebrew/opt/mysql/bin/mysqld --datadir="/Volumes/Extreme Pro/Claude/mysql-wp8/data"
  --tmpdir="/Volumes/Extreme Pro/Claude/mysql-wp8/tmp" --port=3306
  --bind-address=127.0.0.1 --socket=/private/tmp/mysql-wp8.sock
  --log-error="/Volumes/Extreme Pro/Claude/mysql-wp8/mysqld.err" &`

## Prossimo passo: SESSIONE WP-13 = restapi (hang/perf), poi gruppi residui
1. **Gruppo `restapi`** (nome SENZA trattino; 3514 test, oracle ~30s con
   1E/1F/4W/6S suoi): phpr NON TERMINA — >37min CPU (R-state), junit vuoto.
   Metodo: `sample <pid>` sulla run appesa (lezione WP-3) per capire se è un
   loop o perf catastrofica; poi bisezione per classe di test
   (`--filter`/junit parziali). NON è detto sia solo lentezza.
2. **Gruppi successivi**: taxonomy, comment, xmlrpc, multisite? (stessa
   ricetta: baseline oracle → phpr → triage per nome sui junit).
3. **Provider full-suite noti** (ErrorTestCase): `mb_convert_encoding` BIG-5
   (Tests_DB_Charset) · throw in data_wp_validate_site_data via current_time
   (Tests_Multisite_Site) · 1 dataset wpIsIniValueChangeable.
4. La suite INTERA come gate per nome quando i gruppi core sono verdi.
5. Perf suite (media ~63s vs oracle ~28s; restapi vedi §1): profilare con
   `sample` PRIMA di ottimizzare.
6. ext/fileinfo residui (bassa priorità): 3 desc (PICT QuickTime-decompressor,
   2 TTF variable-font name-strings), FILEINFO_EXTENSION map minima,
   magic-db custom non supportato (documentato §2.7).

## Lezioni operative (nuove WP-12)
- ⚠️ **MAI probe che scrivono su wptests mentre gira una suite** (probe
  update_option/insert concorrenti = 26E/116F fantasma in una run). Per i
  probe WP vivi: wp-mo + DB wp_o, o wpdev bootstrap con WP_TESTS_SKIP_INSTALL
  ma SOLO a run ferme.
- ⭐ La GROUND TRUTH batch è la spec: script che tabula l'oracle su tutto il
  corpus dati (849 file → TSV 4 colonne) + diff colonna-per-colonna = il
  detector fileinfo si è scritto per iterazione (165→6→3→0 diff mime).
- ⭐ Bisezione oracle per i trigger dei magic testuali: bisecare il PREFISSO
  del file finché il mime flippa, poi bisecare la riga (js `(function(`,
  po `\nmsgid`+`\nmsgstr`, desc .mo = prima riga trans[0] + 'trans[1]' cap 127).
- PHPUnit su file con dash nel nome fallisce ("Class nav-menu could not be
  found"): usare `--group X --filter test_...`.
- assertEqualSets canonicalizza via sort() di oggetti → i fail "not of
  expected form" a ordine invertito puntano al compare-oggetti engine.
- RelExpr/split_fused: "21st" arriva come DUE token ("21","st") — i suffissi
  ordinali vanno consumati come token orfani.
- rtk grep: righe con NUL nel TSV oracle silenziate — `tr -d '\0' <` SEMPRE
  prima dei diff sui TSV finfo (conferma lezione RTK).

## Invarianti (aggiornati WP-12)
- Gate per OGNI commit: corpus/sess/date/refl per NOME (baseline worktree
  HEAD) · ORM 3E/13F per nome · http-kernel 1665 0E/0F · cargo · probe:
  gd 11/11, mysqli 11/11, media-probe byte-id, run-http · **WP suite:
  option 413 = oracle · media 762 (52S = oracle) · post 906 · user 1341 ·
  query 1889 tutti = oracle** · misura ext/fileinfo (29P/25F/8S).
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED; Serena
  per Rust, Vexp/Read per il C; Read/Write tool per i .php; log `tr -d '\0'`.
