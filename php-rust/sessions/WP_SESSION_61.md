# WP_SESSION_61 — php-server AUTONOMO sul SAPI maturo (accettazione VERDE: 5/6 BYTE-ID + admin NORM-ID vs phpr -S su wpdev/src, admin inclusa) + census v2 DEEP: il ~600MB non-seed è ATTRIBUITO (net unit 648,8MB: classi 58%, payload-Rc solo 14%)

> ⚡ **WP-61 (2026-07-26, `c0eb97a`+seg.)** — P1 della direttiva utente
> ("voglio verificare PERSONALMENTE che WordPress giri, admin inclusa,
> attraverso il server standalone") CONSEGNATO.

## P1 — php-server = wrapper sottile del SAPI cli-server (WP-4/5/6)

Architettura scelta (rischio minimo per il binario di parità):
- **php-cli esposto anche come lib** (`crates/php-cli/src/lib.rs`:
  `pub mod server; pub mod mime;`) — `main.rs` NON toccato; entrambi i
  target compilano gli stessi sorgenti.
- **php-server riscritto da zero** (`crates/php-server/src/main.rs`):
  via axum/tokio (il prototipo 65 righe GET-only è morto), ora è un
  parser argomenti (`--host`/`--port`/`--docroot|-t`/router posizionale,
  default `127.0.0.1:8080` docroot `.`) che delega a
  `php_cli::server::serve()` in forma `phpr -S`. Stesso global allocator
  mimalloc di phpr, stesso `logging::init`.
- **Crate de-gitignorato** (`crates/php-server/` era in `.gitignore`
  riga 1: il prototipo non era MAI stato in history) — ora è un
  deliverable pubblicato.

## Gate (solo server-layer: engine/vm NON toccati)

- cargo **1645/0** ✓ (criterio pre-registrato per cambi solo-server).
- ⚠️ il hash di `phpr` è cambiato comunque (metadati del crate per il
  nuovo lib target, codegen di main.rs identico): **ri-stash come da
  direttiva → `phpr-old-target/release/phpr-wp61` (sha256 `c7e93597…`)
  = baseline di parità corrente**; phpr-wp60 (46b517fc…) resta accanto.

## Ambiente di verifica: wpdev/src servito per la PRIMA volta

- `~/Claude/wpdev/src/wp-config.php` NUOVO → **DB `wp`** (era vuoto e
  senza grant: `GRANT ALL ON wp.* TO 'wp'@'%'` aggiunto). wp_o/wp_p
  della batteria WP-9 NON toccati. Salts fissi `wp61-…`.
- Install WordPress via **oracle** `php -S` + `install.php?step=2` sulla
  porta definitiva 8080 (siteurl nasce giusto): sito "wpdev", utente
  **admin / wp-secret-Pass1**. Pretty permalinks
  `/%year%/%monthnum%/%postname%/` via SQL + reset `rewrite_rules`
  (rigenerazione lazy verificata).
- 🔵 **wordpress-develop src/ ha un guard anti-unbuilt**: senza
  `npm run build --dev` l'index muore 500 "You are running WordPress
  without JavaScript and CSS files" (wp-login invece passa — il guard
  vive solo in src/index.php e negli entry admin). Risolto col flusso
  standard del repo: `npm install` (18s, cache) + `npx grunt build
  --dev` → `src/wp-includes/{js/dist,build}` popolati. Senza build
  anche l'admin sarebbe stata senza JS: la build era necessaria alla
  verifica personale, non solo al guard.

## Accettazione MECCANICA (battery61.sh, pattern run-http.sh WP-9:
## stessi URL, stessa porta 8080, php-server poi phpr -S in sequenza)

| probe | status | confronto |
|---|---|---|
| front `/` | 200 (57.428B) | **BYTE-ID** |
| asset `/wp-includes/css/buttons.css` | 200 | **BYTE-ID** |
| permalink `/2026/07/hello-world/` | 200 (71.093B) | **BYTE-ID** |
| wp-login GET | 200 | **BYTE-ID** |
| wp-login POST | 302 → /wp-admin/ + cookie `wordpress_logged_in` | **BYTE-ID** |
| dashboard `/wp-admin/` | 200 (143.310B) | **NORM-ID** (solo `_wpnonce` di logout: sessioni di login distinte per costruzione) |

Falsi allarmi rientrati durante la messa a punto: (i) il primo probe
permalink usava `/2026/07/26/hello-world/` col GIORNO — il 301 era il
canonical redirect corretto della struttura senza `%day%`; (ii) il primo
DIFF admin era il nonce fuori-virgolette (`&_wpnonce=…`) non coperto dal
normalizzatore WP-9.

## Consegna

- Comando utente: `~/Claude/php-rust-output/release/php-server --port
  8080 --docroot ~/Claude/wpdev/src` → http://127.0.0.1:8080/ (admin:
  `/wp-admin/`, admin / wp-secret-Pass1). MySQL: datadir
  `mysql-wp8/data`, socket `/private/tmp/mysql-wp8.sock` (MAI start
  naive).
- **Quick-start pubblicato nel README** (sezione "Quick-start: serve
  WordPress with php-server") + layout crates aggiornato; gh-status-sync
  eseguita.
- Server di prova KILLATI a fine batteria (lsof 8080 pulito).

## ⭐ Lezioni

- ⭐⭐ **Lib-target senza toccare main.rs = estrazione a rischio zero**:
  il bin continua a compilare i SUOI moduli, la lib espone gli stessi
  file; nessuna divergenza possibile tra phpr -S e php-server (i 6
  probe BYTE/NORM-ID lo confermano sperimentalmente).
- ⭐⭐ **wpdev/src non è servibile as-is**: il guard npm-build di
  wordpress-develop distingue src/ da un release tree (wp-mo). Chi
  serve src/ DEVE passare da `grunt build --dev` (e la full-suite phpt
  non se ne accorge: i test non caricano gli entry guarded).
- ⭐ Un lib target aggiunto cambia il hash del binario di parità anche a
  codegen identico (metadata cargo): il ri-stash con verbale è la
  risposta giusta, non un allarme.
- ⭐ I nonce WP compaiono anche fuori-virgolette (`&_wpnonce=hex10`): un
  normalizzatore per confronti admin deve coprire entrambe le forme.

## P2 — census v2 DEEP dei Module ESEGUITO (design61, `9ada23d`)

Metro nuovo: **net-compile delta per unit** (alloc_counters attorno a
`compile_program_seeded` + relocation, ai 3 siti: include inline, eval,
deferred) = deep VERO, payload op Rc INCLUSI — il complemento esatto del
clone-delta. Registro CENSUS_UNITS esteso (counted, net, ptr leaked);
dump v2: walk address-dedup dei Module ritenuti (`unitpath2/unitsum2`
quota dup su net · `opkind` n per-kind · `modlit` literal dedup
indirizzo+contenuto · `modrecon` split priv/shared per strong_count).
Binario **phpr-memgc61** (9600a662…), harness `wp61-harness/`.

**Risultati `--list-tests`** (memcensus61-listtests.txt, 11,9s user /
1,44GB phys — overhead walk piccolo):
- **net_tot = 648,8MB su 1950 unit** (counted-v1: 211,1MB) ⇒ **il ~600MB
  non-attribuito di WP-60 è CHIUSO**: è il retained vero delle unit.
- Split: **cls_priv 325,6MB (58%)** + mod_owned 202,9MB + fns_priv
  30,7MB = owned_priv 559,2MB; **rc_residue (payload op Rc + overhead) =
  89,6MB (14%)** ⇒ l'ipotesi "è tutto payload op" è RIDIMENSIONATA dal
  contatore: il grosso è clone-visibile (CompiledClass + Module tables).
- `opkind`: owned=0 su TUTTI i kind (2,71M op, 58.575 func) — i payload
  sono interamente Rc/inline; PushConst 611,9k = 22,6% degli op.
- `modlit`: uniq 352.880 = 16,7MB; **dup contenuto cross-unit 273.255 =
  11,7MB logici** (banda MASSIMA interning literal compilati: modesta).
- Leak bootstrap: dup_net 10,4MB (counted 6,3MB) — piccolo, come WP-60;
  il bordo vero resta il full.
- Smoke probe: **net/re-include ~127KB vs 3,1KB counted (~40×)** — il
  costo vero del template-include include la compilazione del prefisso
  stub del seed accumulato (cresce col seed: super-lineare sul full).
- **Mappa FULL v2**: lanciata detached a fine sessione →
  `wp61-harness/census-out/memcensus61-full.txt` (+ flag `.done`);
  lettura = PRIMO ATTO di WP-62 (la predizione Fase 0.5 si ri-quota in
  NET su quella mappa).

## ⭐ Lezioni P2

- ⭐⭐ **net-compile delta = il deep-size che il clone-delta non può dare**
  (payload Rc): i due metri sono complementari e la differenza è essa
  stessa un contatore (rc_residue). Zero walker, zero match per-variante.
- ⭐⭐ **Il recon deve separare gli universi prima di sottrarre**: il primo
  smoke dava owned>net perché il prelude Rc-condiviso (0,88MB) entrava
  in owned ma non nel net delle unit — split priv/shared per
  strong_count al dump, MAI mescolare pool del main con pool delle unit.
- ⭐ Un probe di re-include con classe non guardata muore di redeclare
  (semantica corretta, P4-iii): i template di probe si scrivono come i
  template veri (solo funzioni guardate + literal).

## Prossimo (WP-62)

1. **Leggere la mappa FULL v2** (memcensus61-full) → ri-quotare in NET la
   predizione Fase 0.5 (era −240MB+ counted; in net sarà maggiore).
2. **Fase 0.5 compile-cache** (rilassare la chiave `unit_chain_fp`;
   vincoli P4-i/iii pinnati; gate PIENO).
3. **Seed signature-only** (banda misurata ~130-195MB, A/B separato).
4. 🆕 **php-server: front-end axum** (richiesta utente 2026-07-26): oggi
   il binario è il SAPI sequenziale byte-parity; l'evoluzione richiesta
   è axum davanti (HTTP production-grade, keep-alive, concorrenza) + pool
   di worker engine dietro (un VM per thread, il motore è !Send). Da
   progettare in sessione dedicata; la parità byte del SAPI resta il
   riferimento dei probe.
5. ⚠️ **uploads di wpdev/src**: dal momento in cui Francesco carica media
   nella verifica personale, i futuri harness full NON devono più
   azzerare `src/wp-content/uploads` senza backup (stasera la dir era
   vuota: il wipe del census61-full è stato l'ultimo naive).
