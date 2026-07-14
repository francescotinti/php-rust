# Rotta WORDPRESS-FIRST — WP-track in corso (wp-cli GIRA da sessione WP-1)

> 🏁 **Tappa "wp-cli da sorgente" COMPLETATA** (sessione WP-1, 2026-07-14):
> `wp --info` e `wp cli version` girano end-to-end sotto phpr, a parità con
> l'oracle (uniche differenze: campi ambiente-dipendenti corretti — PHP
> binary=phpr, memory_limit=-1 senza php.ini). Dettaglio nel changelog di
> `PHPR_DIVERGENCES_FROM_PHP.md` (2026-07-14, sessione WordPress-1) e in
> memoria `php-rust-wordpress-track`.

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap (decisione 2026-07-13,
memoria `php-rust-roadmap-wp-first`)**: obiettivo primario = 100%
compatibilità WordPress. Laravel solo come validazione posteriore.

## Cosa è entrato (sessione WP-1 — engine fix da recon wp-cli, commit gated)
- **`global $$x` / `global ${expr}`**: StmtKind::Global → Vec<GlobalItem>
  (Static/Dyn), nuovo `Op::BindGlobalDyn` (resolve-or-create per NOME via
  `global_slot_by_name`, cella creata NULL come Zend, alias in slot named o
  `Frame::dyn_vars`; nome-oggetto → Error via vm_stringify). Il pattern
  wp-config di wp-cli (`global ${$key}; ${$key} = $var;`).
- **Compound assign su variable-variable** (`$$n .= r`): desugar
  read-op-write con nome materializzato UNA volta in un temp. `??=` assente.
- ⭐ **SEND_VAR_EX per chiamate non risolte al compile time**:
  CallValue/CallNsFallback e `$f(...)` dinamico passano gli argomenti via
  `push_dyn_args` (PushRef/ArgPlace); materializzazione contro la by-ref
  mask del callee risolto in `invoke_named`, `push_closure_frame` e
  `enter_authorized_method` (⚠️ OGNI funnel di dispatch DEVE materializzare
  gli ArgPlace — un funnel dimenticato = argomenti NULL: i 94 errori hk del
  primo gate venivano dalle FCC di metodo). Fixa il `&$pipes` di
  `Utils\proc_open_compat` (prima riceveva una copia) e +3 test corpus.
- **Coercizione Stringable negli argomenti string dei builtin puri**: ~30
  nomi in value_builtin_string_coerces + swap convert::to_zstr→ctx.to_zstr
  in string/url/crypto/encoding.rs (substr su DirectoryIterator, md5, trim,
  strpos, urlencode…).
- **DirectoryIterator::__toString = getFilename()** (override Zend) e
  **ordine readdir** (scandir(SCANDIR_SORT_NONE), byte-id con l'oracle) per
  DirectoryIterator/FilesystemIterator/RecursiveDirectoryIterator.
- **`$argv`/`$argc` nel global registry cross-unit** (Zend CLI li registra
  SEMPRE; prima erano visibili solo se l'unità MAIN li menzionava — wp-cli
  li legge da un file required, perdeva gli argomenti e cadeva su `help`,
  il cui pager via proc_open causava il finto "hang").

## Stato (post sessione WP-1 — gate-h2, tutte le baseline in f302e59d/scratchpad)
- **wp-cli 2.13.0-alpha da sorgente: FUNZIONA** (workspace
  f302e59d/scratchpad/wp-work/wp-cli, composer.phar copiato lì).
- Zend corpus **2493 pass / 1563 fail** (zero nuovi fail per nome; +3 fixati:
  backtrace/bug39445, magic_methods/bug43450, unexpected_ref_bug) ·
  ext/session 161 · ext/date 215 · ext/reflection 175 (tutti identici per
  nome) · ORM **3E/13F** (orm-h.names byte-id a orm-g.names) · http-kernel
  **0E/0F** (contatori byte-id a hk-run14) · cargo **1550/0**.
- **Baselines gate correnti: f302e59d/scratchpad** → corpus-h2.norm,
  sess-h2.norm, date-h2.norm, refl-h2.norm, orm-h.names, hk-h2.log.
  (Le gate-g in 85e6296a/scratchpad restano come storico; ⚠️ i probe
  p8_coerce/enum/filter/ifattr NON sono byte-id all'oracle — accettati così
  in sessione 8, confrontare col loro `.phpr` salvato, non con l'oracle.)
- Workspace suite: 56c2e188 `…/scratchpad/symfony/http-kernel` · ORM:
  77b21d67/scratchpad/orm-work · wp-cli: f302e59d/scratchpad/wp-work.

## Prossimo passo del WP-track (tappa 2-3: WordPress vero via wp-cli)
1. **`wp core download`** — scarica e SCOMPATTA WordPress: servono curl/HTTP
   (già ureq) e l'estrazione tar.gz/zip di wp-cli (usa PharData/ZipArchive:
   ext/phar è a ZERO → probabile primo muro; alternativa: scompattare
   WordPress a mano nel workspace e saltare al punto 2).
2. **`wp config create` + WP su SQLite**: plugin ufficiale
   `sqlite-database-integration` (drop-in db.php; gira su PDO/SQLite già
   verde) — niente mysqli per partire.
3. **`wp core install`** → poi `wp post list`, `wp option get siteurl`:
   ogni rottura = coda di lavoro engine, stesso playbook di questa sessione
   (probe minimale → fix gated → gate per nome).
4. Poi: WP core test suite (PHPUnit) come gate per nome del filone.

## Piano: WP-track (dalla memoria php-rust-roadmap-wp-first — 5 tappe)
1. ~~Timezone/date~~ ✅ (sessione 7). 1-bis. ~~wp-cli harness~~ ✅ (WP-1).
2. **SAPI web server** — superglobali da richiesta reale, header/cookie,
   multipart upload, request lifecycle (php-server/Axum è il bridgehead).
3. **Database in 2 tappe**: (a) WP su SQLite col plugin ufficiale (via
   pattern Playground), poi (b) mysqli reale (crate mysql* + parità dei
   messaggi d'errore che wpdb intercetta).
4. **Media**: gd base (thumbnail), exif, fileinfo, zip, curl (HTTP API).
5. **Coda**: mail()/SMTP, openssl fn-level per i plugin.

**Policy fedeltà** (confermata): byte-parity per tutto ciò che rientra in
una stringa PHP; functional-parity (crate Rust) per ciò che esce dal
processo (immagini, rete, mail).

## Lezioni operative (cumulative)
- ⭐ WP-1: un "hang" di wp può essere il PAGER di `help` (proc_open `less`
  che aspetta input) — e wp cade su `help` quando PERDE gli argomenti:
  prima di cercare loop nell'engine, verificare `$this->arguments`.
- ⭐ WP-1: nuovo ArgPlace/PushRef in un percorso di chiamata ⇒ censire TUTTI
  i funnel di dispatch (invoke_named, push_closure_frame,
  enter_authorized_method, dispatch_*): il backstop decay → NULL trasforma
  un funnel mancante in TypeError lontani dalla causa.
- ⭐ WP-1: probe p8 di sessione 8: 4 di essi (coerce/enum/filter/ifattr)
  NON sono byte-id all'oracle — il riferimento è il loro `.phpr` salvato.
- df PRIMA dei run pesanti (gate corpus ~4GB temp); `cargo clean` se serve.
- Probe timezone SEMPRE con tz fissata (l'oracle gira nella zona di sistema).
- ⚠️ gm* e locali DIVERGONO: mai delegare una gm-variante alla locale.
- Probe con vendor (Data, MockClock): eseguirli NEL workspace della suite.
- pgrep -fl (non ps|perl); MAI cargo test/build durante un gate phpt;
  gate per NOME sempre (`--list-fails`), mai solo conteggio; i fail-path si
  estraggono con `^--- (path) ---$` dal log del runner.
- isset($a[k][k2]) e isset($o->p[k][k2]) = OP DIVERSI (IssetPath/FieldIsset).

## Invarianti (identici)
- Gate per OGNI commit: probe byte-id vs oracle · corpus per NOME
  (baseline `corpus-h2.norm`) · ext/session+date+reflection per nome ·
  ORM (**3E/13F**, orm-h.names) se ref/arg/reflection · **http-kernel resta
  0E/0F** (hk-h2/hk-run14) · cargo test.
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED; Serena
  per Rust, Vexp per il C di php-8.5.7; Read tool per i .php; log con
  `LC_ALL=C tr -d '\0'`.
