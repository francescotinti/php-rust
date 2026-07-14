# Rotta WORDPRESS-FIRST — WP-track (WORDPRESS INSTALLATO da sessione WP-2)

> 🏁 **Tappe 2-3 COMPLETATE** (sessione WP-2, 2026-07-14): **WordPress 7.0.1
> è INSTALLATO e interrogabile su SQLite sotto phpr.** Catena verde completa:
> `wp core download` (fresh HTTP + md5 verified + zip estratto **byte-id:
> 3951 file, `diff -rq` pulito con l'oracle**) → `wp config create` →
> `wp core install` col plugin ufficiale `sqlite-database-integration`
> (drop-in db.php) **senza alcun database error** → `wp core is-installed` /
> `option get siteurl` / `post list` (Hello world!) / `user list`
> (admin, roles=**administrator**) a parità con l'oracle. Dettaglio dei 13
> fix engine nel changelog di `PHPR_DIVERGENCES_FROM_PHP.md` (2026-07-14,
> sessione WordPress-2).

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap (decisione 2026-07-13,
memoria `php-rust-roadmap-wp-first`)**: obiettivo primario = 100%
compatibilità WordPress. Laravel solo come validazione posteriore.

## Cosa è entrato (sessione WP-2 — 13 fix engine, tutti probe-pinned)
1. **curl response-sink options** (WRITEFUNCTION/HEADERFUNCTION/FILE/
   WRITEHEADER): stato sul `CurlHandle` prelude, `__curl_exec(id, true)`
   ritorna [headers, body, ret, inc], dispatch nel prelude (header per riga
   CRLF inclusa, body a chunk ≤16384, short-return → errno 23). Sblocca il
   transport curl di rmccue/requests (wp-cli e WP_Http).
2. **`uncaught_throwable` scopato in `run_value_thunk`**: il thunk
   speculativo dei default-param (reflection) non lascia più armato lo
   stash di render_fatal (mascherava i fatal successivi con stack stantii).
3. **Costanti `INI_*`** + **fold namespace-aware** delle costanti engine
   (fold solo dove nessuna costante namespaced può ombreggiare: global ns,
   `\NAME` mono-segmento, o `use const NAME` — il fallback runtime di
   Op::ConstFetch consulta la tabella engine). ns_043/ns_050 + il
   `use const PHP_EOL` di PHPUnit.
4. **`global $x` nelle unità main-style incluse in scope funzione**
   (PushConst+BindGlobalDyn al posto del no-op; `bind_global_dyn` ribinda
   lungo la catena `Frame::bridge_caller` — Zend ha UNA symbol table
   condivisa tra includer e incluso). wp-settings.php/plugin.php.
5. **Shutdown functions coi globali vivi** (`Vm::retired_main`:
   la Ret del main parcheggia il frame; run_shutdown_functions lo
   reinstalla). WP_Fatal_Error_Handler, wp_ob_end_flush_all, _wp_cron.
6. **Niente registrazione eager delle condizionali del seed in drive_unit**
   (remap identità sul prefisso seed — il guard `if (!class_exists())` di
   pomo/translations.php non viene più flippato) + **ri-dichiarazione da
   file re-inclusi** (statement con nome nel prefisso seed → si ri-abbassa,
   bug63741) + **get_declared_classes SOLO registrate** (il residuo
   "conditional compilata = listata" faceva riflettere Composer\
   BinProxyWrapper a doctrine/persistence → 63E in ORM).
7. **Variabili nuove da eval/include pubblicate nel chiamante**
   (fresh-bridge + publish in dyn_vars se definite; get_defined_vars
   include dyn_vars). Il giro `eval(wp-config); get_defined_vars()` di
   wp-cli recupera `$table_prefix` → tabelle `wp_*`.
8. **`Pdo\Sqlite::createFunction` / `PDO::sqliteCreateFunction`** (UDF PHP
   in sqlite via ACTIVE_VM thread-local re-entry, pattern php-src;
   UDF_ERROR ri-propaga l'eccezione originale del callback; deprecation
   8.5 sul metodo BC). Il plugin SQLite ne registra ~45.
9. **Semantica execute/bind pdo_sqlite ri-pinnata (oracle 8.5)**: unbound
   → NULL senza errore; bind ignoto/out-of-range → SQLITE_RANGE 25.
10. **Operatore `namespace\`** in resolve_qualified (utils-wp.php).
11. **PCRE: mix gruppi nominati + backref numerati** via
    `demix_numbered_backrefs` (nomi sintetici `__phprbgN`, nascosti da
    capture_names). FILE_DIR_PATTERN di wp-cli Path.
12. **`str_replace`/`str_ireplace` con `&$count`** (HOST_OUT idx 3, solo a
    4 argomenti; il path registry resta e ora è memmem-accelerato).
    `_deep_replace` non loopa più (esc_url / WP_Sitemaps).
13. **`timezone_identifiers_list()`** (alias di
    DateTimeZone::listIdentifiers; group-filter e nomi BC non modellati —
    2 phpt date "ex-skip" ora girano e falliscono, documentati).

## Stato (post sessione WP-2 — baseline gate-i3 in 37087291/scratchpad)
- **WordPress 7.0.1 installato su SQLite**: workspace di verifica
  37087291/scratchpad/wp-phpr (albero WP + db) e wp-oracle (riferimento).
  wp-cli resta in f302e59d/scratchpad/wp-work/wp-cli.
- Zend corpus **2518 pass** (baseline per NOME: corpus-i3.norm; 26 fixati
  vs h2, 0 nuovi) · ext/session **162** (+1) · ext/date **216** (+1 pass;
  378 fail di cui 2 ex-skip nuovi documentati: timezones-list, bug46111) ·
  ext/reflection **175** (identico) · ORM **3E/13F** (= baseline, stessi
  nomi) · http-kernel **1663 test / 3846 assertion, 0F** (contatori
  byte-id a hk-h2) · cargo test 0 fail.
- ⚠️ Il test hk `KernelTest::testWarmupIsNotRunOnSubsequentBoot` è
  SENSIBILE allo stato di `Tests/Fixtures/var` lasciato da run
  interrotti/binari intermedi: se fallisce da solo, rigenerare lo stato
  (run singolo del file) e rilanciare la suite. Le dir orfane
  `Tests/Fixtures/.!!xxx` sono i tmp-rename di Filesystem::remove di
  symfony MAI completati (bug pre-esistente di phpr da investigare:
  la delete ricorsiva post-rename fallisce silenziosamente).

## Prossimo passo del WP-track
1. **PERFORMANCE del load** (bloccante per l'uso vero): `wp option get` =
   ~20s vs 0.3s oracle. Il costo è lowering+compile PER include con seed
   crescente (~200 file WP): compile_program_stubbed ricompila TUTTE le
   funzioni libere accumulate a ogni include (le classi hanno già lo
   stub-mask, le funzioni no). Piste: fn-stub-mask con dispatch runtime
   per nome, o cache dei moduli compilati per unità.
2. **SAPI web server** (tappa roadmap): php-server/Axum bridgehead,
   superglobali da richiesta reale, header/cookie, multipart.
3. **Divergenze residue WP da chiudere**: attribuzione file/riga dei
   Warning nelle unità incluse (visto prelude:1465 e off-by-file in p34);
   `strtotime("Europe/Rome")` (nome timezone puro) = false (bug46111);
   group-filter/BC-names di timezone_identifiers_list; log_errors CLI su
   stderr (riga "PHP Warning:" duplicata dell'oracle).
4. Poi: **WP core test suite** (PHPUnit) come gate per nome del filone.

## Lezioni operative (cumulative, aggiornate WP-2)
- ⭐ WP-2: un fatal con stack "impossibile" (frame di bootstrap a dispatch
  inoltrato) = stash `uncaught_throwable` stantio di un Err consumato
  host-side: cercare CHI ha valutato thunk speculativi prima.
- ⭐ WP-2: se una classe condizionale "sparisce" solo con un include in
  mezzo, guardare la registrazione eager nel remap di drive_unit (seed
  non-registrato ≠ classe nuova).
- ⭐ WP-2: `esc_url`/`_deep_replace` loopano su QUALSIASI out-param by-ref
  non scritto da una builtin: quando si aggiunge un builtin con &$out,
  censire anche il default quando l'argomento manca.
- ⭐ WP-2: i lexer PHP-level (plugin SQLite, wp-cli Path) muoiono in modi
  remoti quando una preg NON compila: phpr ritorna null SILENZIOSO da
  preg_replace_callback — controllare presto `preg_last_error` mancante.
- ⭐ WP-2: heisenbug nelle suite PHPUnit stateful (cache warmup su disco):
  prima di cercare il bug nel binario, rigenerare lo stato fixture e
  rilanciare — i run interrotti con binari intermedi avvelenano il gate.
- df PRIMA dei run pesanti; probe tz con tz fissata; gm* ≠ locali;
  probe con vendor NEL workspace della suite; pgrep -fl; gate per NOME
  sempre; `^--- (path) ---$` per i fail-path; RTK collassa i body PHP
  (usare Read tool); zsh non espande i glob dentro variabili.

## Invarianti (identici)
- Gate per OGNI commit: probe byte-id vs oracle · corpus per NOME
  (baseline `corpus-i3.norm` in 37087291/scratchpad) · ext/session+date+
  reflection per nome · ORM (**3E/13F**) se ref/arg/reflection ·
  **http-kernel 1663/3846 0F** · cargo test.
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED; Serena
  per Rust, Vexp per il C di php-8.5.7; Read tool per i .php; log con
  `LC_ALL=C tr -d '\0'`.
