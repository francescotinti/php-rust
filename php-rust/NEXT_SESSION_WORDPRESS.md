# Rotta WORDPRESS-FIRST — WP-track (WP SERVITO VIA HTTP da WP-4)

> 🏁 **SAPI WEB SERVER CHIUSA** (sessione WP-4, 2026-07-14): **`phpr -S
> host:port [-t docroot] [router.php]`** è un work-alike del cli-server di
> PHP, oracle-pinned. **WordPress 7.0.1 è servito via HTTP a parità byte con
> `php -S`**: sullo stesso albero+DB SQLite, 8/8 risposte identiche
> (homepage, ?p=1, ?page_id=2, 404, wp-login.php, /wp-json/,
> /robots.txt→301, ?feed=rss2) + batteria SAPI di 48 probe byte-id + log
> stderr identico riga per riga. Dettaglio completo dei 13 blocchi di lavoro
> (a)–(m) nel changelog di `PHPR_DIVERGENCES_FROM_PHP.md` (sessione
> WordPress-4). Tempo per pagina WP: ~1.5-2s (ricompilazione per request —
> vedi ottimizzazioni sotto).

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap (decisione 2026-07-13,
memoria `php-rust-roadmap-wp-first`)**: obiettivo primario = 100%
compatibilità WordPress. Laravel solo come validazione posteriore.

## Cosa è entrato (sessione WP-4 — sintesi; dettaglio nel changelog)
1. **php-cli/server.rs**: server HTTP sequenziale su TcpListener (niente
   axum: la VM è Rc-piena e serve controllo byte su status/ordine header).
   Risoluzione path cli-server (longest-prefix + PATH_INFO, index files,
   **fallback a index.php del docroot** = permalink WP senza router),
   router script con fall-through su `return false`, 404 template byte-id,
   mime map generata da sapi/cli/mime_type_map.h (charset solo text/*),
   log asctime (Accepted/[code]/Closing + "PHP Warning:" multiline).
2. **php_types::sapi**: `WebRequest` thread-local per request (php://input,
   getallheaders, upload tmp registry), `set_sapi_name` processo-globale
   (PHP_SAPI foldato DOPO il set → set PRIMA di ogni lowering).
3. **VM web mode** (vm/websapi.rs): superglobali web esatte, multipart
   rfc1867 → $_FILES, header-family stateful (replace = remove+append!),
   html_errors display + error_log per stderr host, session cookie/cache
   limiter, ini display_* con default CLI e override web.
4. **Fix engine da WP**: condizionali PCRE lookahead riscritti, `[` nudo
   nelle class escapato, `(?<!A|B|C)` decomposto (wptexturize!),
   array_replace_recursive coi Ref (theme.json defaults!), gate BP_VAR_IS
   per ??/??= (`WP_Block->attributes`), `field_magic_probe` per
   isset/empty su catene con magic a qualsiasi step
   (`WP_Block_Type->uses_context`), double_encode=false, ENT_XML1/XHTML,
   RecursiveRegexIterator, hash_hmac_algos, move_uploaded_file.

## Stato (post sessione WP-4 — baseline gate-k in 5f883ed2/scratchpad)
- **WordPress 7.0.1 servito via phpr -S a parità oracle** (workspace di
  verifica: 5f883ed2/scratchpad/wp-same, batteria `wp-battery.sh`; alberi
  originali wp-phpr/wp-oracle in 37087291/scratchpad).
- Batteria SAPI: 5f883ed2/scratchpad/sapi-probe (docroot probe + battery.sh
  riusabile identica su oracle e phpr; 48 casi + log).
- Gate: corpus/sess/date/refl per NOME (baseline precedente corpus-i3 in
  37087291/scratchpad) · hk 1663/3846 0F · ORM 3E/13F (16 nomi) · cargo 0F.

## Prossimo passo del WP-track
1. **wp-admin via HTTP**: login flow completo (POST wp-login.php → cookie
   auth → dashboard). Le session/cookie sono wired; servono probabilmente
   fix su redirect chain, nonces, wp_salt/auth cookie parsing.
2. **Pretty permalinks**: attivare permalink_structure e verificare il
   fallback index.php (già implementato) su rotte /2026/07/hello-world/.
3. **Performance per-request**: ogni richiesta rilowera+ricompila l'unità
   (~1.5-2s/pagina WP). Piste: cache dei Module compilati per path+mtime
   (opcache-like), condividere le Func compilate del prelude tra i moduli
   unità (~12% del residuo, nota WP-3), riusare il VM warm.
4. **Divergenze SAPI residue**: chunked request body; headers_sent() oltre
   output_buffering=4096 (l'oracle flusha lì); `"\u{...}"` escape del
   lexer; doppio confine magico nella stessa catena isset (rest plain);
   PHP_CLI_SERVER_WORKERS.
5. Poi: **WP core test suite** (PHPUnit) come gate per nome del filone.

## Lezioni operative (cumulative, aggiornate WP-4)
- ⭐ WP-4: probe-FIRST anche per un SAPI intero: pinnare i byte dell'oracle
  (batteria curl + log stderr) PRIMA di scrivere il server ha reso la
  parità un diff meccanico; la stessa batteria gira identica su entrambi.
- ⭐ WP-4: le regex WP muoiono in TRE modi diversi sullo stesso engine-chain
  (condizionali lookahead, `[` nudo in class, lookbehind negativo ad
  alternanza variabile): quando un chunk di testo "sparisce", cercare
  SUBITO il preg_replace(array) che ritorna null — bisezione dei pattern
  uno a uno con probe dedicata.
- ⭐ WP-4: replace di header() = REMOVE+APPEND in coda (il feed RSS lo
  smaschera col Content-Type tardivo); NON in-place come sembrava da probe
  ravvicinate — pinnare con chiamate DISTANZIATE.
- ⭐ WP-4: `empty()`/`isset()`/`??` hanno TRE semantiche magic diverse
  (oracle-pinned): ?? e ??= chiamano __get senza __isset (BP_VAR_IS);
  isset() ed empty() NO sui terminali; sulle catene il protocollo vale a
  OGNI confine magico, non solo al primo step.
- ⭐ WP-4: un valore che "sparisce" da un merge ricorsivo = elemento
  Ref-wrappato (foreach by-ref residuo) non dereferenziato nel match Rust:
  controllare `deref_clone` su ENTRAMBI i lati di ogni match su Zval::Array.
- ⭐ WP-2/4: heisenbug da server stantio: pkill può lasciare il vecchio
  binario sulla porta → il nuovo logga "Address already in use" e i probe
  colpiscono il VECCHIO codice. SEMPRE pgrep dopo pkill prima di rilanciare.
- ⭐ WP-3: PROFILARE prima di ottimizzare (`sample <pid>`).
- ⭐ WP-2: preg che non compila = null SILENZIOSO da preg_replace_callback.
- df PRIMA dei run pesanti; probe tz con tz fissata; probe con vendor NEL
  workspace della suite; pgrep -fl; gate per NOME sempre; RTK collassa i
  body PHP (usare Write/Read tool); zsh non espande i glob dentro variabili;
  le funzioni zsh inline perdono il PATH sotto sandbox → script bash su file.

## Invarianti (identici)
- Gate per OGNI commit: probe byte-id vs oracle · corpus per NOME ·
  ext/session+date+reflection per nome · ORM (**3E/13F**) se
  ref/arg/reflection · **http-kernel 1663/3846 0F** · cargo test ·
  batteria SAPI 48 probe + 8 pagine WP se si tocca server/websapi.
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED; Serena
  per Rust, Vexp per il C di php-8.5.7; Read tool per i .php; log con
  `LC_ALL=C tr -d '\0'`.
