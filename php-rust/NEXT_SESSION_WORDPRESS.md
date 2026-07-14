# Rotta WORDPRESS-FIRST — WP-track (unit-cache per-request da WP-6)

> ⚡ **PERF PER-REQUEST: UNIT-CACHE ATTIVA** (sessione WP-6, 2026-07-15):
> cache process-wide dei moduli include lowerati+compilati+rilocati,
> chiave path+mtime+size + fingerprint dello stato VM (chain degli eventi
> di load + digest seed/globals/class_index), riuso double-checked
> (baseline statics + remap ricomputato). **Home WP 1.85s → 1.2s (-35%),
> dashboard ~2.9 → ~2.0s; parse+lower+compile 39% → 3.4% del tempo.**
> Dettaglio nel changelog di `PHPR_DIVERGENCES_FROM_PHP.md` (WordPress-6).

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap (decisione 2026-07-13,
memoria `php-rust-roadmap-wp-first`)**: obiettivo primario = 100%
compatibilità WordPress. Laravel solo come validazione posteriore.

## Cosa è entrato (sessione WP-6 — sintesi; dettaglio nel changelog)
1. **Unit-cache** in vm/mod.rs: `CachedUnit` {fp, static_off, class_remap,
   new_locals, Rc<Program>, &'static Module} in thread_local, 4 vie per
   file; `Vm::unit_fp()` = unit_chain_fp (hash-chain: main identity, ogni
   include path+mtime+size, ogni eval source) + contatori seed/statics +
   digest IN ORDINE di seed_globals (slot baked per indice!) e seed_traits
   + digest order-independent di class_index e seed_aliases. Solo lowering
   "pure" (nessun retry autoload/defer) è cacheable.
2. **drive_unit splittato**: `unit_class_remap` (dedup per nome / identità
   seed-prefix / append) + `run_linked` (registrazioni + frame + drive) —
   il percorso cached ricomputa il remap e lo confronta col cached (che
   convalida anche lo stub-mask baked); mismatch ⇒ miss, fallback fresco.
3. Il leak per-include (`Box::leak` a ogni load) ora è bounded dal riuso.

## Stato (post sessione WP-6 — baseline gate-o in 4776cd24/scratchpad)
- **WordPress 7.0.1: frontend + wp-admin + pretty permalinks via phpr -S a
  parità oracle, ~1.2s/pagina frontend (era 1.85s), ~2s admin.**
  Workspace: 4776cd24/scratchpad (wp-o/wp-p, admin pass `phpr-wp5-Secret`,
  login-flow.sh, admin-battery.sh + adm-diff.sh, pretty-battery.sh);
  batteria SAPI in 5f883ed2/scratchpad/sapi-probe/battery.sh (48 probe).
- Gate: corpus/sess/date/refl per NOME (baseline gate-o) · hk 1663/3846
  0F · ORM 3E/13F stessi 16 nomi · cargo test · batterie SAPI+WP+admin.

## Prossimo passo: SESSIONE WP-7 = perf infrastrutturale, gradini 1+2
**(decisione Francesco 2026-07-15, dopo verifica su zend_alloc.c: la
velocità di Zend è infrastruttura — ZMM a bin/chunk con reset bulk a fine
richiesta, zend_string interned con hash precomputato, zval inline 16B —
e il profilo post-unit-cache di phpr mostra esattamente quei buchi:
malloc/free ~16% dei campioni, SipHash ~9%, Zval clone/drop, gc.)**

SOLO i due gradini a ROI alto, misurando ciascuno; i gradini grossi
(churn Zval/COW, arena per-request stile ZMM) restano in agenda DOPO la
roadmap funzionale.
1. **Swap del global allocator**: `#[global_allocator]` mimalloc (fallback:
   jemalloc) nei binari phpr/phpt-runner. ~5 righe + Cargo.toml. Misura:
   timing home/dashboard (batteria da 5 curl) + `sample` prima/dopo
   (baseline: perf-home2.sample in 4776cd24, home warm ~1.2s).
2. **FxHash/ahash sulle mappe calde + interning dei nomi**: class_index,
   linked_functions, dyn_vars, constants, preg_cache, ecc. (⚠️ NON
   cambiare l'ordinamento osservabile di PhpArray). Interning simboli
   (nome→u32 con hash cache) se il tempo lo consente — replica
   zend_string; altrimenti solo hasher swap e si rimanda.
   Nota: `Vm::unit_fp` usa DefaultHasher suo — indipendente, non toccare.
   In coda se avanza tempo: dedup O(n²) in run_linked (scan lineare di
   saved.functions per nome).
Gate COMPLETI su ogni gradino (engine-core toccato): corpus/sess/date/
refl per NOME + ORM 3E/13F + hk 0F + cargo + pretty 10 + admin 12 +
SAPI 48. Commit separato per gradino.

## Poi, ordine roadmap (riprende qui)
1. **mysqli** (roadmap tappa 3b): WP con MySQL vero oltre che SQLite.
2. **ext/gd & media** (roadmap tappa 5): chiude anche i residui admin
   documentati (webp/avif upload_error, site-health php_extensions).
3. **Divergenze SAPI residue**: chunked request body; headers_sent()
   oltre output_buffering=4096; `"\u{...}"` escape del lexer; doppio
   confine magico nella stessa catena isset; PHP_CLI_SERVER_WORKERS;
   Warning procedurale timezone_open su tz invalida.
4. Poi: **WP core test suite** (PHPUnit) come gate per nome del filone.

## Lezioni operative (cumulative, aggiornate WP-6)
- ⭐ WP-6: la lowering seminata NON è pura rispetto al solo file: bake di
  id classe (class_index), SLOT GLOBALI PER INDICE (seed_globals, che può
  crescere A RUNTIME via `global $x`/$GLOBALS in ordine request-dependent),
  offset statics, trait per chiave. Una cache di unità compilate deve
  fingerprint-are TUTTO ciò che la lowering osserva e double-checkare
  strutturalmente al riuso (remap + baseline statics), con fallback a miss.
- ⭐ WP-6: il replay deterministico della stessa pagina è ciò che rende la
  cache efficace: la chain fp (main + include mtime + eval src) invalida
  a cascata tutto il downstream quando si edita UN file (dev workflow).
- ⭐ WP-6: se si tocca il binario dopo aver lanciato i gate, i gate vanno
  RILANCIATI sul binario definitivo — mai commitare con gate di un binario
  diverso (kill phpt-runner, rebuild, relaunch: costa meno di un dubbio).
- ⭐ WP-5: probe-FIRST del login (curl cookie-jar sull'oracle) rende anche
  wp-admin un diff meccanico; classificare le divergenze UNA a una.
- ⭐ WP-5: builtin che "perde" elementi = Ref senza deref nel match dei
  valori; stato che "cambia da solo" tra 2 chiamate = write-through
  foreach-by-ref su copia (la DUP Zend spezza i ref refcount-1).
- ⭐ WP-5: `Fatal: undefined function` definita più su nel file = ordine di
  pubblicazione (hoisting) — repro triangolo a→b(define+include)→c(call).
- ⭐ WP-2/4/5/6: pgrep DOPO ogni pkill E lsof sulla porta prima di
  rilanciare: un server morente può tenere la porta e servire il binario
  VECCHIO ("Address already in use" nello stderr del nuovo = probe sul
  vecchio). In WP-6: anche i diff "48/48 falliti" possono essere solo la
  PORTA diversa embedded nei body ($_SERVER) — normalizzare o stesso port.
- ⭐ WP-5: normalizzatori con `<TAG>` in stringhe zsh = redirect → file
  vuoti → falso OK: script bash su file + size check.
- ⭐ WP-4: regex WP muoiono in TRE modi sullo stesso chain; bisezione con
  probe dedicata. `empty()`/`isset()`/`??` = TRE semantiche magic diverse.
- ⭐ WP-3: PROFILARE prima di ottimizzare (`sample <pid>`).
- ⭐ WP-2: preg che non compila = null SILENZIOSO da preg_replace_callback.
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
