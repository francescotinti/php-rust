# Revisione adversariale S-188 Tappa 2 — lente SEMANTICA (ponte axum)

Generato con assistenza AI (Claude Fable 5). Revisore: agente fresco, sola lettura, durante la finestra di misura (nessun cargo/curl); mandato REGOLE §7 (≤400 parole, trovare ciò che invalida il claim).

**Claim**: `php-server --axum` serve WordPress con lo stesso comportamento osservabile di `phpr -S` (WebRequest per richiesta sul thread del worker, header/status/cookie restituiti, walk del docroot condiviso, status = response_code o 200, 500 solo per guasto del motore, nessun bleed fra worker). Evidenza: 18 test, G-APERTURA-2 PASS vs `php -S`, battery61-axum 6/6, t3 A/P = 1,00.

## Rilievi del revisore (testuali, per gravità)

1. **Link-fatal: banner PIANO sotto il SAPI web.** La tesi «fatal = 200 + banner html» non vale per i fatal di link (enum che implementa Serializable): `worker_pool.rs:962-966` scrive `\nFatal error: … in … on line …` in `rendered` anche con `vm.web == true`; `finish` dà 200. Stessa forma in `run_module_with_hir` (`vm/mod.rs:1049-1054`), quindi parità axum↔`phpr -S` sì, oracle `php -S` no (html). Non è in §3.9 né nel gate. Riparare: ramo html quando `vm.web` (come `compile_fatal_outcome`) o dichiarare in §3.9.
2. **Dispatch round-robin cieco + loopback WordPress.** `dispatch` (`worker_pool.rs:689-693`) assegna per contatore a code per-worker: una richiesta che WordPress fa verso se stesso (REST loopback di Site Health, `wp_remote_*` bloccanti) ha probabilità 1/W di finire nella coda del worker che la sta aspettando → stallo fino al timeout client. `php -S` con `PHP_CLI_SERVER_WORKERS` usa accept condiviso. La batteria 61 non esercita un loopback bloccante (spawn_cron è a 0,01 s non bloccante), quindi il PASS non copre il caso. Riparare: coda MPMC condivisa, o residuo dichiarato + dente.
3. **Request target: tre scarti non dichiarati.** `route` prende `path_and_query()`: il frammento `/x#y` è troncato dal crate `http` (REQUEST_URI `/x`, la cli-server lo conserva); un target absolute-form perde schema+host; byte non-ASCII nel path → 400 di hyper PRIMA del handler, mentre la cli-server li accetta grezzi. Aggiungere a §3.9.
4. **Log richiesta senza `- detail`.** `log_request` axum non emette il dettaglio del fatal né `- No such file or directory` sul 404; l'oracle lo emette. Mancano anche `Accepted`/`Closing`. Non dichiarato.
5. **Casi d'errore dell'host divergenti.** Script non leggibile: axum testo piano «404: …» vs 500 nudo della cli-server; body oltre il cap → body vuoto silenzioso vs troncamento.
6. **`x-phpr-pid` su ogni risposta**: header sul filo assente nella cli-server, non elencato fra i residui.
7. **Evidenza.** I 18 test non sono vacui (la firma pre-modifica non compila; la coda `web: None` di `two_workers_no_request_bleed` smaschera uno scope non pulito). Ma `battery61-axum.sh` confronta SOLO status-line e body: «byte-identical» va scritto «body byte-identico», gli header non sono confrontati.

Il ciclo `RequestScope` è corretto (installato prima di `main_unit_acquire`, droppato prima di `send`; abort su panic rende irrilevante il drop saltato); HEAD/204/304 li gestisce hyper; `set_sapi_name` prima del runtime.

**Verdetto del revisore: REGGE con riserve** (1 e 2 da dichiarare o chiudere prima del merge a firma; 3–6 a catalogo §3.9).

## Risposta (sessione, 03/11)

- #1 ACCOLTO, da CURARE a fine finestra di misura: forma html del banner di link-fatal quando c'è una richiesta web, condivisa fra worker axum e `run_module_with_hir` (helper in `websapi.rs`, 0 righe nette in `vm/mod.rs` al cap), attesa presa dall'oracle `php -S` con una sonda dedicata; dente nel crate. La cura entra nel tree DOPO il pin s188 misurato (tree = pin + cura di parità, dichiarato).
- #2 ACCOLTO come RESIDUO DICHIARATO + apertura per NOME: il pool a code per-worker con round-robin è un vincolo della macchineria di misura (census per-thread, A-PP36 dispatch map); una coda condivisa cambia quel contratto e va in concilio. Mitigazione oggi: `--workers ≥ 4` e `DISABLE_WP_CRON` nelle misure; la batteria non copre il loopback bloccante (dichiarato nel criterio p.2).
- #3, #4, #5, #6 ACCOLTI a catalogo §3.9 (g–k); #5a (script non leggibile ⇒ 500 nudo come la cli-server) curato insieme a #1.
- #7 ACCOLTO: dicitura «body byte-identico, header non confrontati» in criterio e verbale.
