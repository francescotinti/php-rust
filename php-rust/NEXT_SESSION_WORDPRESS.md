# Rotta WORDPRESS-FIRST — SI PARTE QUI (http-kernel chiuso in sessione 8)

> ✅ **Prerequisito SODDISFATTO** (sessione 8, 2026-07-14): symfony/http-kernel
> è CHIUSO — 1663 test, 0 error / 0 failure (parità con l'oracle). Questo è
> ORA il kickoff operativo della prossima sessione. Dettaglio sessione 8 in
> memoria `php-rust-symfony-http-kernel` (SESSIONE-8) e nel changelog di
> `PHPR_DIVERGENCES_FROM_PHP.md` (2026-07-14).

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap riordinata (decisione
2026-07-13, memoria `php-rust-roadmap-wp-first`)**: obiettivo primario =
100% compatibilità WordPress. Ordine: chiusura symfony/http-kernel →
**WP-track** → Laravel (solo come validazione posteriore).

La sessione 7 (2026-07-13) ha chiuso la **tappa 1 del WP-track: TIMEZONE
D-DT3** (era il big rock: 13F DateTimeValueResolverTest, e serve direttamente
a WP: wp_timezone, strtotime, date dei post). Dettaglio in memoria
`php-rust-symfony-http-kernel` (sezione SESSIONE-7).

## Cosa è entrato (sessione 7, commit gated)
- **`php_types::tz`**: lettore TZif v2/v3 su /usr/share/zoneinfo (cache per
  zona; gap/fold DST alla timelib: offset PRE-transizione in entrambi i casi,
  pinnato con l'oracle su America/Toronto 2026; "UTC" sintetizzato).
- **Default timezone reale**: date_default_timezone_set/get, INI
  `date.timezone` (propagazione da `-d` e `ini_set`, reset a UTC per-VM),
  Notice su ID invalido.
- **Builtin locali nel default tz**: date/idate/strftime/mktime/getdate/
  localtime/strtotime (relative wall-clock-preserving attraverso i salti DST;
  ⚠️ gmmktime ora ha la SUA implementazione UTC, non delega più a mktime).
- **DateTime/DateTimeImmutable zone-aware**: ctor via `__strtotime_tz`
  (priorità zona-nella-stringa > argomento > default, label normalizzate
  "+0500"→"+05:00", "Z" verbatim), `format` con O/P/T/e/I/Z/c/r dall'istanza
  via `__tz_offset` + gmdate, setDate/setTime/add/sub/modify con aritmetica
  wall re-ancorata via `__tz_wall_ts`, `diff` sui tempi LOCALI dei due lati,
  `getOffset` + `DateTimeZone::getOffset`.
- Divergenze residue documentate nel changelog di PHPR_DIVERGENCES (≥2037
  footer POSIX non valutato; nomi IANA/abbreviazioni dentro le stringhe
  datetime non parsati; DateTimeZone ctor senza validazione).

## Stato (post sessione 8, commit `84ceed4` — verificato dall'audit 2026-07-14)
- Suite http-kernel (1663 test; oracle 0 fail): **0E/0F — CHIUSA** (hk-run14:
  restano solo warning/deprecation, come l'oracle).
- Zend corpus **2486 pass** (zero nuovi fail per nome) · ext/session 161 ·
  ext/date **215** · ext/reflection 175 · ORM **3E/13F** · cargo **1550/0**.
- **Baselines gate correnti in 85e6296a/scratchpad** (gate-g): corpus-g.norm,
  sess-g.norm, date-g.norm, refl-g.norm, orm-g.names (per data-set),
  hk-run14.log. Probe: p7_tz1.php byte-id (60 assert timezone) + p8_*.
- Workspace suite: 56c2e188 `…/scratchpad/symfony/http-kernel`. ORM:
  77b21d67/scratchpad/orm-work.

## Cosa è entrato (sessione 8 — chiusura http-kernel, engine fix riusabili da WP)
Visibilità del costruttore a `new`; is_callable ZPP completo (static-style,
$syntax_only, &$callable_name); FILTER_VALIDATE_REGEXP; range-check nella weak
coercion a int (overflow → TypeError); enum from/tryFrom = port di
zend_enum_from_base; **DateTime comparabili per istante** (date_object_compare
in ops.rs — WP confronta date dei post!) + `==` di array con valori loose;
**flock(2) reale** sui file stream (WP usa file lock per cache/cron);
**INI error_log onorata da error_log()** (WP debug.log!); attributi sulle
interfacce; ctor Exception/Error condizionale; ⭐ **distruttori eager dopo
ogni statement in ogni body** (semantica refcount Zend).

## Piano: WP-track (dalla memoria php-rust-roadmap-wp-first — 5 tappe)
1. ~~Timezone/date~~ ✅ (sessione 7).
2. **SAPI web server** — superglobali da richiesta reale, header/cookie,
   multipart upload, request lifecycle (php-server/Axum è il bridgehead).
3. **Database in 2 tappe**: (a) WP su SQLite col plugin ufficiale
   `sqlite-database-integration` (via WordPress Playground; gira sul
   PDO/SQLite già verde), poi (b) mysqli reale (crate mysql* + parità dei
   messaggi d'errore che wpdb intercetta).
4. **Media**: gd base (thumbnail), exif, fileinfo, zip, curl (HTTP API).
5. **Coda**: mail()/SMTP, openssl fn-level per i plugin.

**Harness**: wp-cli = harness CLI per far girare WP PRIMA del SAPI (playbook
Doctrine/Symfony: suite → errori → fix gated). Poi WP core test suite
(PHPUnit) = gate per nome del filone; top plugin (WooCommerce, Yoast) come
suite extra. **Primo passo concreto**: recon wp-cli **DA SORGENTE** — ext/phar
è a ZERO (565 test), quindi NIENTE wp-cli.phar: `git clone wp-cli/wp-cli` +
`composer install` (già collaudato sotto phpr) → `phpr vendor/bin/wp --info`
(o il bin del pacchetto wp-cli/wp-cli-bundle), prime rotture = nuova coda di
lavoro. Stesso playbook di PHPUnit/Doctrine.

**Policy fedeltà** (confermata): byte-parity per tutto ciò che rientra in
una stringa PHP; functional-parity (crate Rust) per ciò che esce dal
processo (immagini, rete, mail).

## Lezioni operative (cumulative)
- df PRIMA dei run pesanti (gate corpus ~4GB temp); `cargo clean` se serve.
- Probe timezone SEMPRE con tz fissata (l'oracle gira nella zona di sistema).
- ⚠️ gm* e locali ora DIVERGONO: mai "delegare" una gm-variante alla
  variante locale (il bug gmmktime→mktime della sessione 7).
- Probe con vendor (Data, MockClock): eseguirli NEL workspace della suite.
- pgrep -fl (non ps|perl); MAI cargo test/build durante un gate phpt;
  gate per NOME sempre (`--list-fails`), mai solo conteggio.
- isset($a[k][k2]) e isset($o->p[k][k2]) = OP DIVERSI (IssetPath/FieldIsset).

## Invarianti (identici)
- Gate per OGNI commit: probe byte-id vs oracle · corpus per NOME
  (baseline `corpus-g.norm`) · ext/session+date+reflection per nome ·
  ORM (**3E/13F**, orm-g.names) se ref/arg/reflection · **http-kernel resta
  0E/0F** (hk-run14 è la nuova baseline: mai regredirla) · cargo test.
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED; Serena
  per Rust, Vexp per il C di php-8.5.7; Read tool per i .php; log con
  `LC_ALL=C tr -d '\0'`.
