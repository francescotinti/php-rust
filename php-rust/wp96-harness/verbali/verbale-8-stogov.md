# Verbale sedia 8 — Dmitry Stogov (engine/opcache) — Concilio WP-96

**VERDETTO: CON EMENDAMENTI VINCOLANTI. Una refutazione capitale (Q1) e una
declassazione (criterio 5).**

## Q1 — `stream_get_wrappers`: NON è «correct-or-absent onesto». È split-brain.

Refutato **dal codice**, non da opinione. `crates/php-runtime/src/vm/host.rs:80`
`is_builtin_scheme()` elenca **tutti e dodici** i nomi (`file php http https ftp
ftps data glob phar zip compress.zlib compress.bzip2`) e li usa per **rifiutare**
`stream_wrapper_register` («Protocol ftp:// is already defined.», host.rs:6049).
La lista restituita a userland ne dichiara **cinque**. In PHP c'è **UNA**
tabella (`url_stream_wrappers_hash`): `stream_get_wrappers`, il guard di
`register`, `unregister` e `fopen` la leggono tutti. Qui sono **tre** tabelle
(la terza è il fallback di `stream_is_local`, host.rs:6030) e **si contraddicono**.

Conseguenze per NOME, entrambe osservabili:
1. phpr **occupa** `ftp`/`phar`/`zip` e **nega** di averle: userland non può né
   usarle né fornirle. In PHP l'uscita di sicurezza esiste ed è un idioma
   diffuso — `stream_wrapper_unregister('phar')` (hardening Composer/plugin WP),
   poi eventuale re-register. In phpr quell'`unregister` ritorna **false +
   warning** dove PHP ritorna **true**: divergenza mai catturata dalla suite.
2. §2.4 del catalogo dichiara `stream_get_wrappers` **differito** per i wrapper
   userland ⇒ un `stream_wrapper_register('vfs')` **non compare** nella lista ⇒
   `wp_is_stream('vfs://…')` falso. vfsStream/PHPUnit e Flysystem vivono lì.

Cosa fa PHP davvero: registra i wrapper al MINIT per **build** (senza ext/zip
niente `zip` — quindi «assente per build» è legittimo **in PHP**), e
`allow_url_fopen=0` **non** toglie il nome dalla lista: PHP elenca ciò che
**non aprirà**. Precedente decisivo: **la presenza nella lista significa "questo
schema è dell'engine, non trattarlo come path"**, ed è esattamente ciò che
`wp_is_stream` chiede (`in_array($scheme, stream_get_wrappers(), true)`).
Omettere il nome fa cadere il path in `path_join`/`realpath`/`mkdir` su
`"ftp:/example.com"`: **errore silenzioso**. Dichiararlo dà un **errore
rumoroso** all'open. Su questo asse il "correct-or-absent" va applicato al
**verbo**, non al **nome**.

## Emendamenti

- **A-DS-96-1 (coerenza, non-negoziabile qualunque scelta)**: `is_builtin_scheme`
  **abolita**; una sola registry `nome → {Native | Userland(classe) |
  Declared-Unimplemented}` letta da `stream_get_wrappers`, `register`,
  `unregister`, `restore`, `fopen`, `stream_is_local`. Invariante pinnata:
  `∀n ∈ stream_get_wrappers(): register(n,C)===false` **e** `unregister(n)===true`.
- **A-DS-96-2**: i wrapper userland **entrano** nella lista (oggi differiti), e
  l'**ORDINE** è quello di registrazione — PHP itera hash-order, **non**
  alfabetico. Verificare se phpr ordina: sarebbe una seconda divergenza latente.
- **A-DS-96-3 (graduato)**: `glob://` e `compress.zlib://` **implementati per
  davvero** (zlib è già FFI, glob già esiste come funzione) — due nomi tolti dal
  gap onestamente. `ftp/ftps/zip/phar/compress.bzip2` → `Declared-Unimplemented`:
  elencati, con fallimento **PHP-shaped** all'open (testo del warning esatto,
  `url_stat`→false **senza** warning) e **un phpt di pin per nome**. Uno stub che
  elenca e fallisce a caso è peggio dell'assenza: vietato senza il pin.
- **A-DS-96-4 (battery61, normalizzatore)**: `s/[0-9a-f]{10}\b/` è **troppo
  largo su tre assi**: (a) niente `\b` a sinistra ⇒ normalizza la **coda** di
  ogni md5/sha (32 hex → ultimi 10 cancellati); (b) `[0-9a-f]` include le
  **cifre** ⇒ **ogni intero decimale a 10 cifre**, cioè **ogni timestamp Unix**
  (`1785801803` è nell'`.out` stesso), sparisce — proprio la classe dove
  `time()`/`date()`/`uniqid()` divergerebbero; (c) è per **forma**, non per
  **nome**, contro la regola già scritta nel commento del file. Sostituire con
  cattura **contestuale** (`name="_wpnonce" value="(…)"`, `_wpnonce=(…)`,
  `"nonce":"(…)"`, `_ajax_nonce`) + **conteggio sostituzioni per lato stampato
  e preteso uguale**.
- **A-DS-96-5 (battery61, copertura)**: si confronta solo `head -1` dell'`.hdr`:
  `Content-Type`, `Set-Cookie` (flag HttpOnly/SameSite/path), `Location`,
  `Cache-Control` **non sono mai confrontati**. Probe `5-loginpost` è
  `BYTE-ID … bytes=0`: **due corpi vuoti** — dente **vacuo** per il criterio che
  questa stessa sessione ha appena imparato (lezione 4). Il suo contenuto
  probatorio sta negli header: confrontarli, o marcarlo `VACUOUS-BODY`.
- **A-DS-96-6 (leva, tabella dei simboli)**: il preludio va reso **come la
  function-table interna di PHP** — nomi/arità/segnatura **eager**, **corpi**
  lazy. Pin a tre istanti (start / dopo aver forzato il parse di UN file / fine):
  `get_defined_functions`, `get_declared_classes`, `get_declared_interfaces`,
  `get_defined_constants` identici **e nello stesso ORDINE**. L'ordine di
  `get_declared_classes` è osservabile ed è ordine di dichiarazione.
- **A-DS-96-7 (leva, binding)**: early binding. In PHP `class B extends A`
  top-level si lega a compile-time solo se `A` è già legata, altrimenti
  `DECLARE_CLASS_DELAYED`. Il parse lazy per-file **cambia chi è già legato** ⇒
  cambia l'ordine, `class_exists('B', false)`, e i messaggi «Cannot redeclare
  X()» / «Cannot declare class X, because the name is already in use» —
  che devono restare **identici e nello stesso punto** con e senza pigrizia.
- **A-DS-96-8 (leva, visibilità degli errori di parse)**: un errore di sintassi
  in un file di preludio **mai chiamato** oggi è fatale; col parse lazy diventa
  **invisibile**. Bite-test: iniettare l'errore, pretendere lo stesso fatale.
- **A-DS-96-9 (leva, lifetime)**: nessun artefatto compilato può sopravvivere
  alla propria arena — default-arg AST, attributi ritenuti per reflection,
  stringhe internate condivise. Falsificatore = il corpus **refl 290** eseguito
  **dopo** il drop delle arene, non prima.

## Kill-switch

- **KS-DS-96-1**: se A-DS-96-1/2/3 muovono anche **un solo nome** in corpus 1418,
  refl 290, ORM 3E/13F o hk 1665 → si tiene **solo** il pin di catalogo, il
  cambio di lista si annulla.
- **KS-DS-96-2**: se la leva per-file altera l'**ordine** di
  `get_declared_classes`/`get_defined_functions` di **un** elemento → la leva è
  **morta a quello step**. Non si adatta il test.
- **KS-DS-96-3**: se il bite-test di A-DS-96-4 (divergenza sintetica a 10 hex /
  10 cifre fuori dai nonce) **non** porta rc=1, il NORM-ID di S-94.0 è
  **ADVISORY** e il **criterio 5 torna PARZIALE**.

## Refutazioni

1. **CAPITALE** — «reticenza onesta» è falso: la registry di `register` già
   rivendica i dodici nomi. Non è assenza, è **incoerenza fra due tabelle** che
   in PHP sono una sola; e la metà nascosta (`unregister('phar')` che ritorna
   false) è peggiore di quella misurata.
2. Il normalizzatore NORM-ID è **per forma** e ingoia timestamp a 10 cifre e code
   di hash: la dashboard non è provata identica, è provata **non-distinguibile
   dal filtro che l'ha giudicata**. Senza controllo negativo non è un giudice.
3. `5-loginpost BYTE-ID bytes=0` è un PASS vacuo, dalla stessa famiglia della
   lezione 4 di S-94.0: il dente non annuncia di aver smesso di mordere.
