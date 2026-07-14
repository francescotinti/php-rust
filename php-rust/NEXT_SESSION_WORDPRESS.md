# Rotta WORDPRESS-FIRST — WP-track (wp-admin + pretty permalinks da WP-5)

> 🏁 **WP-ADMIN VIA HTTP CHIUSO** (sessione WP-5, 2026-07-14): **login flow
> completo (POST wp-login.php → cookie auth → dashboard) + 12 pagine
> wp-admin a parità oracle** (dashboard 125854b e edit.php 109732b byte-id
> modulo nonce/timestamp; residui solo legittimi: capability gd/webp/avif,
> antispambot rand(), auto-draft id). **Pretty permalinks attivi**
> (structure salvata via POST admin) con **10 rotte frontend
> BYTE-IDENTICHE senza normalizzazione** (post, 301 canonico, page,
> category, author, feed, 404, wp-json pretty, mese, home). Dettaglio dei
> 10 fix engine (a)–(j) nel changelog di `PHPR_DIVERGENCES_FROM_PHP.md`
> (sessione WordPress-5).

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap (decisione 2026-07-13,
memoria `php-rust-roadmap-wp-first`)**: obiettivo primario = 100%
compatibilità WordPress. Laravel solo come validazione posteriore.

## Cosa è entrato (sessione WP-5 — sintesi; dettaglio nel changelog)
1. **Hoisting funzioni di unità incluse PRIMA del run** (Zend le hoista a
   compile-time dell'include; l'hook admin_menu le chiama da un include
   annidato) + **symbol table globale unica per catene di include a
   global scope** (nome fresco con catena bridge fino a frame 0 → cella
   globale, non locale staccata; `global $menu` di includes/menu.php).
2. **By-ref attraverso i funnel dinamici**: call_user_func_array passa i
   Ref vivi; build_args_array pusha i Var come PushRef (SEND_VAR_EX);
   split_args_from_array_value/spread_pairs preservano i Ref; spread su
   callee by-ref noti supportato (il Walker di WP usa entrambe le forme).
3. ⭐ **zend_array_dup in PhpArray::clone**: le reference refcount-1
   (residui foreach-by-ref) si SPEZZANO alla duplicazione dell'array,
   come Zend — chiuso il write-through di WP_REST_Server::get_routes
   dentro $this->endpoints (Allow "1", methods [1,1], preload amputato).
4. **Tabella entità HTML 4.01 completa** (152 nomi; D-56.1 chiuso),
   **`?>`-terminatore inghiotte il newline** (check sui byte sorgente),
   **array_flip con ZVAL_DEREF**, **RecursiveArrayIterator**,
   **timezone_open/offset_get/name_get + validazione ctor DateTimeZone**
   (DateInvalidTimeZoneException/DateException, matrice oracle-pinned).

## Stato (post sessione WP-5 — baseline gate-l in 4776cd24/scratchpad)
- **WordPress 7.0.1: frontend + wp-admin + pretty permalinks via phpr -S a
  parità oracle.** Workspace di verifica: 4776cd24/scratchpad (wp-o/wp-p
  alberi gemelli con admin pass `phpr-wp5-Secret`, login-flow.sh,
  admin-battery.sh + adm-diff.sh, pretty-battery.sh, post-probe.sh).
- Gate: corpus/sess/date/refl per NOME (baseline gate-l) · hk 1663/3846
  0F · ORM 3E/13F stessi 16 nomi · cargo 1550/0 · batteria SAPI 48 + WP.

## Prossimo passo del WP-track (ordine roadmap)
1. **Performance per-request**: ogni richiesta rilowera+ricompila l'unità
   (~1.5-2s/pagina WP, admin più pesante). Piste: cache dei Module
   compilati per path+mtime (opcache-like), condividere le Func compilate
   del prelude tra i moduli unità (~12% residuo, nota WP-3), riusare il
   VM warm tra richieste. PROFILARE PRIMA (`sample <pid>`, lezione WP-3).
2. **mysqli** (roadmap tappa 4): WP con MySQL vero oltre che SQLite.
3. **ext/gd & media** (roadmap tappa 5): chiude anche i residui admin
   documentati (webp/avif upload_error, site-health php_extensions).
4. **Divergenze SAPI residue**: chunked request body; headers_sent()
   oltre output_buffering=4096; `"\u{...}"` escape del lexer; doppio
   confine magico nella stessa catena isset; PHP_CLI_SERVER_WORKERS;
   Warning procedurale timezone_open su tz invalida.
5. Poi: **WP core test suite** (PHPUnit) come gate per nome del filone.

## Lezioni operative (cumulative, aggiornate WP-5)
- ⭐ WP-5: il probe-FIRST del login (curl cookie-jar sull'oracle, 5 step
  pinnati) ha reso anche wp-admin un diff meccanico; le divergenze
  restanti si classificano UNA a una come engine-bug o legit (rand,
  capability, storia DB) — mai fermarsi al conteggio delle righe di diff.
- ⭐ WP-5: un array che "perde" elementi dentro UNA SOLA funzione builtin
  (array_flip vuoto ma json_encode/count/keys ok) = elemento Ref-wrapped
  e match Rust senza deref: controllare il deref su OGNI builtin che
  matcha i VALORI (il gemello della lezione WP-4 sui merge ricorsivi).
- ⭐ WP-5: stato che "cambia da solo" tra due chiamate della stessa
  funzione (get_routes 1a vs 2a chiamata) = write-through di un foreach
  by-ref su una COPIA che condivide celle Ref con l'originale — la regola
  Zend è che la DUP spezza i ref refcount-1; phpr ora la implementa in
  PhpArray::clone.
- ⭐ WP-5: `Fatal: Call to undefined function X()` in un file che la
  DEFINISCE più su = ordine di pubblicazione (hoisting) — repro col
  triangolo a.php→b.php(define+include)→c.php(call), 3 righe.
- ⭐ WP-2/4/5: pgrep DOPO ogni pkill E lsof sulla porta prima di
  rilanciare: un server morente può tenere la porta e servire il binario
  VECCHIO ("Failed to listen ... Address already in use" nello stderr del
  nuovo = i probe stanno colpendo il vecchio).
- ⭐ WP-5: normalizzatori di diff con `<TAG>` DENTRO stringhe zsh = zsh li
  tratta come redirect → file vuoti → **falso OK del diff**: sempre
  script bash su file E check size>0 prima del verdetto.
- ⭐ WP-4: le regex WP muoiono in TRE modi sullo stesso engine-chain;
  quando un chunk "sparisce", bisezione dei pattern con probe dedicata.
- ⭐ WP-4: `empty()`/`isset()`/`??` = TRE semantiche magic diverse.
- ⭐ WP-3: PROFILARE prima di ottimizzare (`sample <pid>`).
- ⭐ WP-2: preg che non compila = null SILENZIOSO da preg_replace_callback.
- df PRIMA dei run pesanti; probe con vendor NEL workspace della suite;
  gate per NOME sempre; RTK collassa i body PHP (usare Write/Read tool);
  zsh non espande i glob dentro variabili.

## Invarianti (identici)
- Gate per OGNI commit: probe byte-id vs oracle · corpus per NOME ·
  ext/session+date+reflection per nome · ORM (**3E/13F**) se
  ref/arg/reflection · **http-kernel 1663/3846 0F** · cargo test ·
  batteria SAPI 48 probe + 8 pagine WP se si tocca server/websapi ·
  batteria admin 12 pagine + pretty 10 rotte se si tocca engine-core.
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED; Serena
  per Rust, Vexp per il C di php-8.5.7; Read tool per i .php; log con
  `LC_ALL=C tr -d '\0'`.
