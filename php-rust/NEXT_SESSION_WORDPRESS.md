# Rotta WORDPRESS-FIRST — WP-track (perf infra chiusa in WP-7)

> ⚡ **PERF: WP-7 CHIUSA (2026-07-15, commit `b574eff` + `de67428`)**:
> mimalloc `#[global_allocator]` + FxHash sulle mappe calde.
> **Home WP 1.20s → 0.76s (-37%), dashboard 1.25s → 0.81s (-35%)**
> (sul già -35% della unit-cache di WP-6: dal pre-WP-6 1.85s → 0.76s,
> -59%). SipHash e malloc spariti dal top-of-stack; ora dominano
> memmove/memcmp e clone/drop di Zval → churn COW/arena per-request
> restano IN AGENDA DOPO la roadmap funzionale (decisione 2026-07-15).
> Dettaglio nel changelog di `PHPR_DIVERGENCES_FROM_PHP.md` (WordPress-7).

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap (decisione 2026-07-13,
memoria `php-rust-roadmap-wp-first`)**: obiettivo primario = 100%
compatibilità WordPress. Laravel solo come validazione posteriore.

## Cosa è entrato (sessione WP-7 — sintesi; dettaglio nel changelog)
1. **Gradino 1 (`b574eff`)**: mimalloc global allocator in php-cli e
   phpt-runner (~5 righe + Cargo.toml). Home -24%, dash -24%.
2. **Gradino 2 (`de67428`)**: rustc-hash su PhpArray::index (ordine
   osservabile in `entries`, hasher invisibile), alias di modulo in
   vm/mod.rs per tutte le mappe del Vm + Frame::dyn_vars,
   Module::class_index + CompiledClass::prop_info in bytecode.rs.
   `Vm::unit_fp` NON toccato; var_dump_debug/stringify_args restano std
   (attraversano l'API php-builtins). Home -17%, dash -15% ulteriori.
3. Gate COMPLETI su ciascun gradino, sul binario definitivo di ciascuno.

## Stato (post WP-7 — baseline gate-o resta il riferimento per nome)
- **WordPress 7.0.1: frontend + wp-admin + pretty permalinks via phpr -S a
  parità oracle, ~0.76s/pagina frontend, ~0.81s dashboard.**
  Workspace: 4776cd24/scratchpad (wp-o/wp-p, admin pass `phpr-wp5-Secret`,
  login-flow.sh, admin-battery.sh + adm-diff.sh, pretty-battery.sh);
  batteria SAPI in 5f883ed2/scratchpad/sapi-probe/battery.sh (48 probe).
  ⚠️ I capture oracle (adm-oracle, flow-oracle) sono del 14/07: i diff
  raw contengono drift di orologio/auto-draft — usare adm-diff.sh e
  classificare; pretty resta byte-id senza normalizzazione.
- Gate: corpus 1528 / sess 67 / date 377 / refl 294 per NOME (baseline
  gate-o) · hk 1663/3846 0F · ORM 3E/13F stessi 16 · cargo 1550/0 ·
  batterie SAPI 48 + pretty 10 + admin 12 + login 5.

## Prossimo passo: SESSIONE WP-8 = mysqli (roadmap tappa 3b)
WordPress con MySQL vero oltre che SQLite (chiude il paragone col
plugin sqlite-database-integration e apre gli hosting reali):
- ext/mysqli work-alike (OOP + procedurale) su un client MySQL nativo
  Rust (candidato: crate `mysql` sync; valutare TLS e auth caching_sha2).
- Parità messaggi d'errore wpdb (`mysqli_real_connect`,
  `mysqli::__construct`, error/errno/sqlstate) — WP li mostra all'utente.
- Harness: MySQL locale (brew mysql o container), `wp core install` su
  DB MySQL con l'oracle → stessa sequenza con phpr; poi batterie HTTP
  esistenti puntate al wp-tree MySQL (pretty/admin/login).
- Metodo collaudato: probe-FIRST sull'oracle (script minimi mysqli),
  poi delega-a-builtin; pattern OOP-stdlib come bcmath/gmp
  (classe PHP prelude + builtin host) se conviene.

## Poi, ordine roadmap (riprende qui)
1. **ext/gd & media** (roadmap tappa 5): chiude anche i residui admin
   documentati (webp/avif upload_error, site-health php_extensions).
2. **Divergenze SAPI residue**: chunked request body; headers_sent()
   oltre output_buffering=4096; `"\u{...}"` escape del lexer; doppio
   confine magico nella stessa catena isset; PHP_CLI_SERVER_WORKERS;
   Warning procedurale timezone_open su tz invalida.
3. Poi: **WP core test suite** (PHPUnit) come gate per nome del filone.
4. (Dopo la roadmap funzionale) perf profonda: churn Zval/COW, arena
   per-request stile ZMM, interning simboli (nome→u32, replica
   zend_string), dedup O(n²) in run_linked.

## Lezioni operative (cumulative, aggiornate WP-7)
- ⭐ WP-7: swap infrastrutturali (allocator/hasher) = engine-core a tutti
  gli effetti: gate COMPLETI per nome su ogni gradino, commit separati.
  L'iterazione delle HashMap std era GIÀ random per istanza → nessun
  output può dipendere dall'ordine: lo swap a FxHash (deterministico) è
  sicuro per costruzione; l'ordine osservabile di PhpArray vive in
  `entries` (Vec), non nell'indice hash.
- ⭐ WP-7: i tipi che attraversano il confine di crate (Ctx di
  php-builtins) NON vanno swappati alla cieca: lasciare std dove il path
  è freddo costa zero e tiene il diff contenuto.
- ⭐ WP-7: estrazione dei fail-set: marker `--- <path> ---` coi PATH CON
  SPAZI → regex `^--- (.+\.phpt) ---$`, MAI `\S+`; conteggio>0
  obbligatorio prima del verdetto (0 estratti = regex rotta, non 0 fail).
- ⭐ WP-7: capture oracle invecchiano: diff raw vs capture del giorno
  prima = drift orologio (Max-Age, "overdue by N hours", orari nelle
  option) + contatori auto-draft. Classificare col char-level diff prima
  di gridare alla regressione; widgets.php 500 è PARITÀ (block theme
  senza sidebar, anche l'oracle dà 500).
- ⭐ WP-6: se si tocca il binario dopo aver lanciato i gate, i gate vanno
  RILANCIATI sul binario definitivo.
- ⭐ WP-5: probe-FIRST del login (curl cookie-jar sull'oracle) rende anche
  wp-admin un diff meccanico; classificare le divergenze UNA a una.
- ⭐ WP-5: builtin che "perde" elementi = Ref senza deref; stato che
  "cambia da solo" tra 2 chiamate = write-through foreach-by-ref su copia.
- ⭐ WP-2/4/5/6: pgrep DOPO ogni pkill E lsof sulla porta prima di
  rilanciare (server morente = binario VECCHIO in servizio; diff "tutti
  falliti" può essere solo la PORTA embedded nei body).
- ⭐ WP-4/5: normalizzatori bash su file + size check; regex WP muoiono in
  TRE modi sullo stesso chain; `empty()`/`isset()`/`??` = TRE semantiche.
- ⭐ WP-3: PROFILARE prima di ottimizzare (`sample <pid>`).
- df PRIMA dei run pesanti; gate per NOME sempre; RTK collassa i body PHP
  (Write/Read tool); zsh non espande i glob dentro variabili.

## Invarianti (identici)
- Gate per OGNI commit: probe byte-id vs oracle · corpus per NOME ·
  ext/session+date+reflection per nome · ORM (**3E/13F**) se
  ref/arg/reflection · **http-kernel 1663/3846 0F** · cargo test ·
  batteria SAPI 48 probe + 8 pagine WP se si tocca server/websapi ·
  batteria admin 12 pagine + pretty 10 rotte se si tocca engine-core.
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED; Serena
  per Rust, Vexp per il C di php-8.5.7; Read tool per i .php; log con
  `LC_ALL=C tr -d '\0'`.
