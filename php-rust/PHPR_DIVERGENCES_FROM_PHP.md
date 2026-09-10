# phpr — Divergenze note rispetto a PHP standard (8.5.7)

> Catalogo vivo delle **anomalie** di `phpr` rispetto al PHP di riferimento
> (oracle: PHP 8.5.7). Ogni voce è un punto in cui phpr **non** riproduce
> byte-per-byte il comportamento dell'interprete C, oppure lo riproduce solo
> parzialmente. Serve come mappa per rientrarci in modo mirato.
>
> Principio guida del progetto: **correct-or-absent** — uno stub che mente è
> peggio di una funzione assente. Molte voci qui sotto sono "assenze
> consapevoli" o "divergenze circoscritte", non bug silenziosi.
>
> **REGOLA DI MANUTENZIONE (decisione utente 2026-08-03).** Questo è un
> catalogo **VIVO**, non un archivio: una voce che rientra a parità si
> **RIMUOVE**, non si marca «chiusa» e si lascia lì. La rimozione esige la
> **prova** — probe eseguito contro l'oracle, output byte-identico — mai la
> dichiarazione di una sessione. La storia delle chiusure vive in git e in
> `sessions/`; il changelog storico 2026-07 è congelato in
> `doc/archive/DIVERGENCES_CHANGELOG_2026-07.md`.
>
>  **Ultima verifica sul campo: 2026-09-10** — ogni voce con un repro
> eseguibile è stata ri-eseguita contro l'oracle 8.5.7 (pin s172; probe in
> `wp172-harness/` non necessarie: gli script stanno nei verbali della
> revisione). **Rimosse** perché byte-identiche: §3.15 (variadic by-ref
> diretto), §3.19-bis (builtin di processo come callable dinamico; resta il
> solo residuo §3.31), §3.19-ter (`display_errors=stderr`), le parti chiuse
> di §1.1, §3.7 e §3.13, il sotto-punto (c) e l'ordering autoload WP-67 di
> §3.8, i bullet `memory_limit` (§3.5) e `&` in `var_dump($_SESSION)` (§3.6)
> già dichiarati rimossi il 2026-08-03 ma rimasti nel corpo; §1.6 rimossa
> perché NON riproducibile (nessun repro nel testo, sei scenari by-ref
> identici). **Precisate** con la misura: §1.2, §1.4, §2.1 (hang su
> overflow), §2.3, §2.4 (`ftell`/`file_put_contents` su wrapper utente),
> §2.7, §3.1 (UCS-2 SBAGLIATO, UTF-32 assente), §3.3-bis, §3.6, §3.8 (vi),
> §3.28, §3.30. **Aggiunte** §3.31 e §3.32 (S-173). Numerazione STABILE (le voci si citano
> per nome nei verbali): i numeri delle voci rimosse non si riassegnano; il
> §3.4 non è mai stato assegnato.

---

## 1. Gap trasversali dell'architettura builtin (impattano molti phpt "di tipo")

Questi sei gap nascono tutti dalla stessa radice architetturale: le funzioni
del crate **`php-builtins`** sono pure (`fn(args, ctx) -> Result<Zval, PhpError>`)
e **non hanno accesso allo stato della VM**. Non possono quindi rientrare nel
motore per invocare metodi utente, generare backtrace "veri", o consultare lo
stato ZPP dell'engine. Limitano principalmente i **phpt di edge-case/di tipo**,
non l'uso reale delle webapp target (WP/Composer/Doctrine/Laravel/Symfony).

### 1.1 Coercion di oggetti `Stringable` nelle builtin pure: due residui
La coercion `__toString` nelle builtin pure (VM-stateless) passa per la mappa
`stringify` precomputata dalla VM prima del dispatch (gate
`*_builtin_string_coerces`); ~30 value builtin + `natsort`/`natcasesort` +
`implode`/`str_replace` sugli array sono byte-identici (ri-verificato
2026-09-10, nessun `__toString` spurio in is_string/gettype/var_dump).
Restano:
- (a) `sprintf`/`printf` con specifier NON-stringa: `%s` è a parità, ma con
  `%d` l'oracle emette `Warning: Object of class E could not be converted to
  int` e stampa `1`, phpr stampa il valore convertito senza warning (coercion
  PER-specifier non guidabile dalla builtin senza re-entrancy VM). Deferito.
- (b) `str_replace` con search E replace entrambi array di oggetti con
  `__toString` a side-effect: l'ORDINE delle chiamate diverge (phpr: tutti i
  search poi tutti i replace; PHP: interleaved per coppia). Risultato
  byte-identico; diverge solo l'ordine dei side-effect.

### 1.2 Deprecation ZPP `null → parametro non-nullable`
- **Sintomo**: passare `null` a un parametro interno non-nullable (es.
  `strlen(null)`) deve emettere `E_DEPRECATED` da PHP 8.1. phpr **non** la emette
  per la maggior parte delle builtin (`strlen(null)`, `strtoupper(null)`);
  alcune la emettono già byte-identica (`str_contains(null, "")`) — verificato
  2026-09-10.
- **Radice**: la validazione ZPP di phpr non modella la distinzione
  "null implicito coercibile ma deprecato".

### 1.3 `#[SensitiveParameter]` non onorato
- **Sintomo**: i parametri marcati `#[SensitiveParameter]` devono comparire come
  `Object(SensitiveParameterValue)` nei backtrace/messaggi d'errore. phpr mostra
  il valore reale.
- **Impatto**: sicurezza/diagnostica; nessun impatto funzionale.

### 1.4 Validazione ZPP dei callable "upfront"
- **Sintomo**: le funzioni che accettano un `callable` devono validarlo **prima**
  di eseguire il corpo (ZPP). phpr valida SOLO all'invocazione: se il callback
  non viene mai chiamato (`array_map('no_such_fn', [])`, `usort`/`array_filter`
  su array vuoto) phpr non lancia NULLA dove l'oracle lancia il `TypeError`
  upfront; `call_user_func_array('no_such_fn', [])` lancia `Error: Call to
  undefined function` invece del `TypeError` (verificato 2026-09-10).

### 1.5 Location di `ArgumentCountError` da callback invocati internamente
- **Sintomo**: quando una builtin invoca un callback utente con troppi pochi
  argomenti, il file/linea riportati nell'`ArgumentCountError` non coincidono
  con quelli dell'oracle (che punta al sito inline del callback).

---

## 2. Assenze consapevoli (funzioni non implementate per mancanza di infrastruttura)

Non sono bug: sono funzioni **volutamente assenti** perché un'implementazione
fedele richiede stato/infra non ancora presente, e uno stub violerebbe
correct-or-absent.

| Funzione | Perché è assente / diverge | Cosa servirebbe |
|---|---|---|
| `get_defined_constants` | `resolve_constant` è un `match` non enumerabile: non esiste un registro iterabile delle costanti host | Registro costanti iterabile (host + estensioni) |
| `parse_ini_file` / `parse_ini_string` | Il parser INI di PHP è un lexer flex con semantica di coercizione (NORMAL/TYPED) ed edge-case; difficile essere byte-identici | Port fedele del lexer INI + tabella coercizioni |
| `get_include_path` / `set_include_path` | **Le funzioni ESISTONO e fanno roundtrip** (verificato 2026-08-03), ma il resolver di `include` non consulta il path: `set_include_path` non estende la ricerca | Resolver include che consulta `include_path` (vedi §3.5) |
| `preg_last_error` / `preg_last_error_msg` | Presenti ma lo stato d'errore non è popolato: su backtrack-limit l'oracle dà `2`/`Backtrack limit exhausted`, phpr `0`/`No error` (il caso UTF-8 `4` coincide); inoltre in quel caso `preg_match` stessa ritorna `int(0)` dove l'oracle ritorna `false` (verificato 2026-09-10) | Ponte allo stato d'errore dell'engine PCRE |
| `preg_filter` | Assente | Come sopra |
| `getimagesize` (formati rari) | **L'out-param `&$image_info` è rientrato** e GIF/JPEG/PNG/BMP/WebP/ICO/TIFF/PSD sono identici; restano fuori `wbmp` e `xbm` | Parser per i due formati residui |
| `opcache_*` | Nessun opcache | (fuori scope) |

### 2.1 bcmath — 14 funzioni + `BcMath\Number` (metodi + operatori) + `RoundingMode`

Le 14 funzioni procedurali (`bcadd`/`bcsub`/`bcmul`/`bcdiv`/`bcmod`/`bcdivmod`/
`bcpow`/`bcpowmod`/`bcsqrt`/`bccomp`/`bcscale`/`bcfloor`/`bcceil`/`bcround`) sono
implementate byte-identiche (port di `libbcmath`, `crates/php-builtins/src/bcmath.rs`;
~4000 casi fuzz + battery verdi). La classe **`BcMath\Number`** è una classe PHP nel
prelude (`crates/php-runtime/src/lower/prelude_bcmath.php`) che delega ai builtin bc\*,
con le regole di scala di `bcmath_number_*_internal` (add/sub=max, mul=somma,
div/sqrt/pow⁻=+10 e collassa, ecc.). L'enum **`RoundingMode`** (8 casi) è nel prelude.
**Overloading operatori IMPLEMENTATO** (`+ - * / % **`, `<=> == < > <= >=`, `++/--`,
compound-assign): `apply_binop_ovl`/`try_number_binop` (vm/mod.rs) instrada gli operandi
`Number` ai metodi PHP `Number::__op`/`__cmp` via `call_method_sync` (re-entrancy VM già
usata per `__toString`/`offsetGet`); confronti con tipi non-numerici = UNCOMPARABLE; il
path stringa-vs-oggetto salta il `__toString` per i Number. Suite ufficiale
`ext/bcmath`: **100/124** runnable, Zend corpus invariato (0-regr). Residui consapevoli:

- **var_dump object-id** (`#N`): i risultati aritmetici creano un Number intermedio via
  delega (`new Number(...)`), e il free-list degli handle di phpr ricicla gli id in modo
  diverso da PHP → i `#id` in var_dump differiscono (i VALORI sono byte-identici). ~14
  phpt (`operators/*_int|*_string`, `methods/divmod|sqrt`). Intrinseco alla delega.
- **`pow($n, 2)` funzione** (non operatore): la builtin `pow()` non instrada ancora
  gli oggetti Number a do_operation (1 phpt gh20006). L'operatore `**` funziona.
- **Cast engine di `Number`**: `(bool)$n`/`(int)$n` usano `cast_object` in C (zero→false;
  int/float→warning). Una classe PHP non può ridefinire questi cast → `(bool)` di un
  Number è sempre truthy in phpr. ~2 phpt (`cast`, `cast_warning`).
- **Coercizioni ZPP** su `Number`: float→int con deprecation nel costruttore
  `string|int`, e la deprecation "Passing null to parameter" sui metodi non sono
  emesse (cfr. §1.2). Risultato numerico corretto, manca la riga di deprecation.
- **var_dump object-id**: i metodi che creano Number intermedi (`divmod`, `sqrt`)
  spostano il contatore `#N` degli handle → i `#id` in var_dump possono differire
  (limite intrinseco della delega a classe PHP, non un errore di valore).
- **`bcmath.scale` INI**: lo scale di default (`bcscale()`) è tenuto in stato
  thread-local, non legato all'INI `bcmath.scale` (phpr non ha un registro INI reale,
  cfr. `get_cfg_var`). I phpt con `--INI-- bcmath.scale=N` sono skippati dal runner
  (sezione INI non supportata), non un difetto dell'implementazione.
- **Overflow di esponente estremo** (`bcpow` con exp che fa traboccare `SIZE_MAX`
  cifre): l'oracle esce con `Fatal error: Possible integer overflow in memory
  allocation` (rc 255, NON un ValueError come scritto in precedenza); phpr **non
  termina** (hang, ucciso a 15 s — verificato 2026-09-10). Da curare con un cap
  sull'esponente PRIMA del calcolo: un hang è peggio di un fatal.

### 2.2 gmp — 49 funzioni + classe `GMP` + operatori (via num-bigint)

Tutte le funzioni gmp non-random (49/51) sono implementate byte-identiche nei VALORI
(port su `num-bigint`, `crates/php-builtins/src/gmp.rs` = primitive `_gmp_*` su stringhe
decimali; classe `GMP` + wrapper `gmp_*` in `crates/php-runtime/src/lower/prelude_gmp.php`).
Verificato con battery + fuzz (aritmetica, divisione+arrotondamenti, teoria dei numeri
gcd/powm/invert/jacobi/kronecker/primi, bitwise two's-complement, operatori
`+ - * / % ** & | ^ ~ << >>` + confronti + `++/--` + compound). Suite ufficiale
`ext/gmp`: **46/90** runnable. Residui consapevoli:

- **Random** (`gmp_random_bits`/`_range`/`_seed`): non-deterministico → non byte-matchabile,
  assente. **`gmp_import`/`gmp_export`**: packing di byte con word-size/endianness, differito.
- **Cast engine** `(int)$g`/`(float)$g`: usano `cast_object` in C (→ intval/float); una classe
  PHP non può ridefinirli → phpr dà il cast oggetto di default. Come §2.1 (cast_object).
- **Suffisso "called in …"**: i TypeError di argomento delle funzioni *userland* del prelude
  aggiungono "called in FILE on line N", che le funzioni interne di PHP non hanno. Mitigato
  usando parametri `mixed` + validazione manuale (`_int`/`_arg`), ma alcuni messaggi residui
  differiscono. Gap uniforme delle funzioni-builtin-in-prelude.
- **var_dump object-id** `#N`: come §2.1 (la delega crea GMP intermedi; free-list handle
  diverso). VALORI byte-identici.
- **Deprecation ZPP** float→int su operandi/argomenti non emessa (cfr. §1.2); valore corretto.

### 2.3 tokenizer — token_get_all/token_name/PhpToken (phase 1) sul lexer di mago

`token_get_all`/`token_name` sono host builtin (`crates/php-runtime/src/vm/tokenizer.rs`)
che girano il **lexer di mago** (già front-end di phpr) e mappano ogni `TokenKind` →
id `T_*` di PHP (o stringa 1-char). 152 costanti `T_*` in `resolve_constant`; classe
`PhpToken` nel prelude (delega a `token_get_all`). Byte-identico su codice reale
(funzioni/classi/array/operatori/commenti/namespace/nullsafe/coalesce/attributi) **e su
interpolazione+heredoc comuni** (`"$a {$b} ${c}"`, `"$a[0]"`, `"$a->b"`, heredoc/nowdoc).
Post-pass: T_OPEN_TAG/T_CLOSE_TAG inglobano 1 newline (con fix del numero di riga),
`&`→409/410 context, `namespace\X`→T_NAME_RELATIVE, e context-machine interno alle stringhe
(`{`→T_CURLY_OPEN, `${name}`→T_STRING_VARNAME, `$a[0]`→T_NUM_STRING, drop di T_ENCAPSED vuoto).
Costanti TOKEN_PARSE/TOKEN_AS_OBJECT. **Error-token recovery + heredoc** (phase-3): su byte
non riconosciuto mago consuma+errora → emetto `T_BAD_CHARACTER` e proseguo; su literale
numerico invalido (`0177...787`) recupero lo span → `T_DNUMBER`; **keyword dopo `->`/`?->`
→ T_STRING** ("looking for property"); **coalescenza dei `T_ENCAPSED_AND_WHITESPACE` adiacenti**
(mago spezza il contenuto stringa/heredoc per riga, PHP no). **Flag `TOKEN_PARSE`** (phase-3 group A):
classe `ParseError`/`CompileError` aggiunte al prelude; sotto `TOKEN_PARSE` (a) i keyword
semi-reserved dopo `::`/`const` diventano T_STRING (feedback del parser: `X::continue`, `X::class`,
`const ARRAY`), (b) gli errori **lexer-level** che phpr rileva lanciano `ParseError` col messaggio
FISSO di PHP ("Invalid numeric literal"; "Invalid UTF-8 codepoint escape sequence[: Codepoint too
large]"), (c) `$o->__halt_compiler()` (metodo, non il costrutto) viene ri-lessato: mago entra in
halt-mode e ingoia il resto come inline-HTML → rilego la coda come PHP e la reinserisco (riga
rebased). Recovery octal-invalido ora sceglie T_LNUMBER/T_DNUMBER per magnitudine (`078`→LNUMBER).
**Deprecation dei cast non-canonici**: sotto TOKEN_PARSE PHP compila, quindi `(double)/(integer)/
(boolean)/(binary)` alzano l'E_DEPRECATED compile-time "Non-canonical cast (x) is deprecated, use
the (y) cast instead" (via `raise_diagnostic` → esegue l'error handler utente, che può lanciare o
ri-entrare — GH-19507; phpr ri-lessa ogni chiamata da zero, quindi niente corruzione) e `(real)`
lancia il ParseError fatale "The (real) cast has been removed, use (float) instead".
Suite ufficiale `ext/tokenizer`: **42/49** runnable. Residui:

- **Messaggi di sintassi bison/yacc** (`TOKEN_PARSE_000` "unexpected identifier", heredoc non
  terminato "unexpected end of file, expecting…"): i messaggi di mago ≠ PHP → byte-identico non
  fattibile senza riprodurre il layer di errori del parser PHP. Hard.
- **`gh19507_throw`**: l'handler invocato da un builtin dev'essere tracciato come `[internal
  function]` con file-arg vuoto (`''`) — concern trasversale trace/handler, non del tokenizer.
- **`__halt_compiler` statement-level** (`bug54089`): la tokenizzazione del contenuto post-halt
  diverge (span PHP-scanner-specifici, es. `" ABC"` come singolo token). Solo il caso `->` è gestito.
- **Keyword-come-identificatore in altri contesti** (trait `use A { namespace as bar; }`):
  PHP → T_STRING, mago → keyword — SOLO sotto `TOKEN_PARSE` (senza il flag entrambi
  danno T_NAMESPACE; verificato 2026-09-10). Gestiti `->`/`?->` e (sotto TOKEN_PARSE) `::`/`const`.
- **`yield from`** = 1 token T_YIELD_FROM in PHP; mago = Yield + ws + From.
- **`PhpToken::is(float)`**: coercizione ZPP float→int (deprecation §1.2) invece del TypeError.

### 2.4 Stream wrappers userland — stream_wrapper_register

`stream_wrapper_register`/`unregister` (registry `scheme→classe`) + `fopen("scheme://…")` istanzia la
classe handler (costruttore + default valutati, come `new`) e chiama `stream_open`; nasce una
`ResKind::UserStream`. Le file-op (`fread`/`fwrite`/`feof`/`fclose`/`fgets`/`rewind`/`fseek`/`ftell`/
`stream_get_contents`/`file_get_contents`) dispatchano ai metodi `stream_*` dell'oggetto (VM-re-entrant),
via un fast-path in `CallBuiltin` che scatta SOLO se l'arg #1 è una UserStream → l'I/O di file normale è
byte-identico e intatto. Fill bufferizzato fedele a PHP (`stream_read($chunk=8192)`+`stream_eof()`;
bounded si ferma su short read, read-to-EOF su read vuota). **Byte-identico** sull'uso reale (wrapper
read-only, file_get_contents, fopen/fgets). **Divergenze consapevoli**:
- il **NUMERO di resource-id** in var_dump (contatore interno, classe §2.1) può differire.
- la **sequenza esatta delle chiamate interne** `stream_eof`/`stream_seek` quando UN SOLO handle
  mescola letture e scritture: PHP emette un `stream_seek(pos)` di sync read→write che phpr non emette →
  osservabile solo da un wrapper di test che fa echo dei propri interni, mai dal codice reale.
- `ftell()` su uno stream utente ritorna `false` (oracle: la posizione, es. `4`) e
  `file_put_contents("scheme://…")` fallisce con «Failed to open stream» mentre
  `fopen(…, 'w')` + `fwrite` funzionano (verificato 2026-09-10).
- differiti: `stream_wrapper_restore`/`stream_get_wrappers`, dir-ops (`dir_opendir`…), `url_stat`
  (file_exists/stat sul wrapper), il flag `STREAM_USE_PATH` (`&$opened_path` accettato ma non propagato).

### 2.5 mysqli — client MySQL nativo Rust (crate `mysql` v28, sessione WordPress-8)

Classi `mysqli`/`mysqli_result`/`mysqli_stmt`/`mysqli_driver`/`mysqli_sql_exception`/`mysqli_warning`
+ ~75 funzioni procedurali nel 5° prelude (`lower/prelude_mysqli.php`), delegate agli host builtin
`__mysqli_*` (`vm/mysqli.rs`); connessioni e prepared statement = handle nativi in `Vm.mysqli_conns`/
`mysqli_stmts` (pattern `__pdo_*`). **11/11 probe byte-id vs oracle** (init/connect, errori di
connessione 1045/2002/1049, query/fetch_* coi TIPI del protocollo testo=stringhe e binario=nativi,
error/errno/sqlstate/error_list + REPORT_STRICT/ERROR, escape, fetch_field completo, multi_query/
next_result, OOP surface, prepared statements bind_param/get_result/bind_result, costanti MYSQLI_*,
caching_sha2 full-auth con password su TCP). NUM_FLAG (32768) ricostruito client-side per i tipi
numerici (il crate lo maschera; DECIMAL escluso, come mysqlnd); handshake riallineato con
`SET NAMES utf8mb4` (charsetnr 255 come mysqlnd, non 45). **Divergenze consapevoli**:
- **`MYSQLI_USE_RESULT` si comporta come STORE** (result set sempre bufferizzato host-side):
  osservabile solo su memoria/latency, mai sui byte.
- **`multi_query` = split client-side** delle statement (quote/backtick/commenti rispettati) eseguite
  in sequenza: il server reale si ferma anch'esso alla prima statement fallita → osservabilmente
  equivalente; diverge solo il timing (esecuzione lazy vs eager) per side-effect di statement dopo
  la prima, mai per i risultati.
- **`max_length` nei field metadata = 0 costante** (fedele a mysqlnd PHP ≥ 8.1).
- Le costanti deprecate (`MYSQLI_REFRESH_*`) esistono col valore giusto ma l'USO non emette
  E_DEPRECATED (folding compile-time in `resolve_constant`).
- `mysqli_options`/`ssl_set`/`attr_set` accettate e ignorate (no-op true); `stat()` assemblato da
  SHOW GLOBAL STATUS (formato fedele, valori reali); `refresh`/`dump_debug_info` → false;
  `mysqli_warning` stub. `host_info` "Localhost via UNIX socket" solo con socket esplicito
  (host `localhost` senza socket va in TCP su 127.0.0.1, come i config WP reali).
- Deprecation di `mysqli_ping()` emessa dal prelude; propr. dinamiche/`var_dump($mysqli)` mostrano
  anche i prop privati `__h`/`__stash` (rappresentazione interna, non surface wpdb).

### 2.6 gd + exif — ext/gd sulla **libgd di SISTEMA** via FFI (sessione WordPress-9)

Classe opaca `GdImage` + ~60 funzioni `image*` nel 6° prelude (`lower/prelude_gd.php`),
delegate agli host builtin `__gd_*` (`vm/gd.rs`) sopra la FFI `php_types::gdio`
(`build.rs` linka `/opt/homebrew/opt/gd/lib/libgd.dylib` — **la stessa dylib che
l'oracle brew linka**, pattern zlibio). Upgrade rispetto alla policy roadmap
(functional-parity via crate): decode/encode passano dagli **stessi codec**
(libjpeg-turbo/libpng/libwebp/libavif) → **i file generati sono BYTE-IDENTICI**
(11/11 probe gd byte-id; pipeline media WP byte-id: subsizes, -scaled 2560,
conversioni webp/avif). ext/exif: `exif_read_data`/`exif_imagetype` +
`iptcparse` + `getimagesize` con `&$image_info` (APPn) e parser AVIF
(php-builtins/exif.rs, port del subset di exif.c: IFD0/EXIF/GPS/IFD1-thumbnail,
COMPUTED, COMMENT, UndefinedTag:0x%04X, rationals "n/d").
`GdImage` è **engine-opaca** (`is_opaque_handle_class` in php-types, consultata
da clone/serialize/var_dump/var_export/print_r/json/Reflection): il prop handle
`$__h` e i metodi helper del prelude restano invisibili; `new`→Error,
`clone`→Error, `serialize`→Exception coi messaggi Zend esatti.
**Divergenze consapevoli**:
- **Byte-parity dei file generati vale finché oracle e phpr linkano la STESSA
  libgd** (upgrade brew di gd/codecs → divergenza *comune* ai due lati, ma
  ricontrollare le probe p04–p06). `GD_VERSION`/`GD_*_VERSION` sono foldate
  compile-time a 2.3.3; `gd_info()['GD Version']` è runtime (`gdVersionString`).
- Assenti (correct-or-absent): `imagettftext`/`imagettfbbox`/`imageftbbox`
  (FreeType), `imagefilter`, `imageconvolution`, `imagegammacorrect`,
  `imagelayereffect`, `imagesetthickness`/`setstyle`/`setbrush`/`settile`,
  `imagearc`/`imagepolygon` family, `imagewbmp`/`imagexbm`/`imagebmp`/`imagegd(2)`
  output, `imagegrabscreen`, affine/interpolation avanzate. `gd_info()` dichiara
  FreeType true (verità della libgd linkata) anche se imagettftext non è esposta.
- `imagecolorat` su truecolor usa `gdImageGetTrueColorPixel`; palette semantics,
  antialias flag e interlace sono write-through sulla struct C (layout 2.3.3).
- exif: MakerNote/INTEROP/FPIX/WINXP non decodificati (i pointer-tag restano
  valori grezzi); `EXIF_USE_MBSTRING` re-encoding fuori scope; il filtro
  `$required_sections` non è enforced (output sempre completo, come osservato
  per i casi WP). AVIF in getimagesize = mini-parser ispe/pixi/auxC (primo
  `ispe`/`pixi` in ipco + auxC "alpha"), non il libavifinfo completo (item
  grid/ipma associations fuori scope).
- `getimagesize`: `&$image_info` popolato solo per JPEG (APP0..APP15,
  first-wins come php_read_APP); segmenti >64KB multi-APP non ricomposti.

### 2.7 fileinfo — detector Rust modellato sulla libmagic BUNDLED 5.46 (sessione WordPress-12)

Classe opaca `finfo` + `finfo_*`/`mime_content_type` nel 7° prelude
(`lower/prelude_fileinfo.php`), delegati all'host builtin `__finfo_detect`
(`php-builtins/src/fileinfo.rs`). Niente FFI (macOS non espone una libmagic
pubblica, e l'oracle brew usa comunque la libmagic **bundled** di PHP con
database patchato): il detector è un work-alike con `encoding.c` (tabelle
`looks_*`), `is_json.c`, `is_csv.c` portati verbatim e una tabella firme
curata sui magic che WordPress e il suo test-corpus esercitano.

**Parità misurata** (ground truth = oracle 8.5.7 sugli 849 file di
`wordpress-develop/tests/phpunit/data`): MIME_TYPE, MIME e MIME_ENCODING
**0 diff su 849**; FILEINFO_NONE (descrizioni) 846/849. I/O PHP-side
(`file_get_contents` cap 7MB = FILE_BYTES_MAX), quindi wrapper userland e
`open_basedir` valgono come nel php_stream path dell'oracle.

Divergenze note (tutte desc-only o fuori dal profilo WP):
- `$magic_database` custom di `finfo_open`/`new finfo` **ignorato** (si usa
  sempre il detector builtin; i phpt che caricano `tests/magic` passano solo
  dove le entry coincidono col database bundled).
- FILEINFO_NONE: PICT senza il sotto-print "QuickTime with decompressor";
  TTF variable-font senza la coda name-strings del Magdir/sfnt 5.46; ELF
  ridotto a "ELF [32/64-bit] [LSB/MSB]" (niente catena completa e niente
  mime dedicato → in MIME mode un ELF cade su octet-stream/text come le
  entry mime-less del mini-db dei phpt).
- FILEINFO_EXTENSION: mappa minima e NON allineata all'oracle (verificato
  2026-09-10: su JSON phpr dà `json`, l'oracle `???`; jpeg identica).
- `ReflectionClass('finfo')` non riporta i metodi (classe opaca engine-level,
  stesso comportamento di GdImage).
- phpt `ext/fileinfo/tests`: 29P/25F/8S (fail residui = magic-db custom,
  formati esotici, dettagli descrizione).

---

## 3. Divergenze di engine circoscritte (documentate nei topic-file di memoria)

| Area | Divergenza | Nota |
|---|---|---|
| Chiamate dinamiche | 5 test Zend "Cannot call X dynamically" non rifiutati | manca il reject per alcune funzioni non chiamabili dinamicamente |
| `extract` | `EXTR_REFS` non supportato | il resto dei flag EXTR_* è fedele |
| PDO/sqlite UDF | Le User-Defined Function SQLite sono deferite | richiedono re-entrancy della VM dentro il callback rusqlite |
| `FETCH_CLASS` protected / `PDORow` / `FETCH_LAZY` | modalità PDO fetch residue | deferite |
| `array_multisort` con **oggetti** negli array | coercizione oggetti in fase di sort segue i gap object/Stringable (§1.1) | 2 `variation` phpt (SORT_NUMERIC/REGULAR su Stringable) |
| `date_parse` artefatti dello SCANNER re2c | input ben formati + date calendar-invalid (`2006-02-30`→"The parsed date was invalid") sono byte-identici (phase 1+2); restano gli artefatti del backtracking dello scanner timelib per input STRUTTURALMENTE malformati (`2006-12--12`→mese 12/giorno 1/zone −43200, `2006-13-01`→mese 1, `25:00:00`→ora 5, `03-03`/`0-0`, `garbage`→timezone-attempt "Double timezone specification") — richiedono il port della macchina a stati char-level di timelib, non replicabile da un parser a token | `date_parse_001`/`date_parse_error` phpt |

### 3.0 Backtrace di eccezioni lanciate da builtin (gap UNIVERSALE)
- **Sintomo**: un'eccezione lanciata da un builtin (value o host) e **non
  catturata** produce un backtrace senza il frame della funzione interna: phpr
  stampa `#0 {main}` mentre l'oracle stampa `#0 file(line): fn(args)` + `#1 {main}`.
- **Verificato** su `mb_internal_encoding`, `filter_input`, ecc. — è trasversale a
  OGNI builtin che lancia, non specifico.
- **Impatto**: solo il backtrace di eccezioni **uncaught** o ispezionate via
  `getTrace()`; il TIPO e il MESSAGGIO dell'eccezione sono corretti. Correlato al
  gap §1.5 (ArgumentCountError location).

### 3.1 Divergenze delle tabelle di encoding (codec mbstring)
Il codec mbstring di phpr usa `encoding_rs` per gli encoding non gestiti a mano
(UTF-8/ASCII/Latin-1/UTF-16 sono diretti). Alcune **tabelle di conversione**
differiscono da quelle di libmbfl, e alcuni encoding non sono mappati. Questo
impatta ogni `mb_*` che decodifica/ricodifica (`mb_convert_encoding`,
`mb_encode_numericentity`, …), **non** la logica delle singole funzioni.

| Encoding | Divergenza | Esempio |
|---|---|---|
| `ISO-2022-JP` | `encoding_rs` decodifica il segno di sterlina (`!r`) in `U+FFE1` (fullwidth) invece di `U+00A3` (regola libmbfl) | `mb_encode_numericentity` test #11 |
| `UCS-4` / `UCS-4LE` / `UCS-2LE` / `UTF-32` | non presenti in `resolve_encoding` → `ValueError "must be a valid encoding"` (verificato 2026-09-10) | `mb_decode_numericentity` test (linea 54) |
| `UCS-2` | **ACCETTATO ma SBAGLIATO**: `"A"` → `41` (1 byte) invece di `0041` — viola correct-or-absent, va tolto da `resolve_encoding` o corretto (scoperto 2026-09-10) | `mb_convert_encoding("A", "UCS-2")` |
| `SJIS`/`EUC-JP` (casi rari) | possibili scostamenti di mapping su codepoint di confine | (potenziale) |

Nota: la **logica** di `mb_encode_numericentity`/`mb_decode_numericentity` è
byte-identica all'oracle (convmap, offset/mask, overflow, `;` opzionale,
pass-through) — verificata su tutte le asserzioni edge-case dei phpt, che
riportano `(Good)`. Gli unici fail residui sono queste tabelle di encoding.

### 3.2 Classe `Directory` — wrapper prelude, non classe interna
`dir($path)` ritorna un oggetto **`Directory`** definito come classe PHP nel
prelude (proprietà `path`+`handle`, metodi `read`/`rewind`/`close` che delegano a
`readdir`/`rewinddir`/`closedir` sull'handle `opendir`). **L'uso reale è
byte-identico** all'oracle: costruzione via `dir()`, iterazione `read()`, `path`,
`var_dump` (`object(Directory)#N (2)` con `handle` = `resource(N) of type
(stream)`) — i 9 call-site reali rilevati dal detector.

Restano divergenti le semantiche **C-level** della classe interna (`ext/standard`
la crea via `create_object` custom con restrizioni non esprimibili in userland):
`new Directory()` NON è bloccato, le proprietà NON sono `readonly`, l'oggetto è
clonabile/serializzabile, e la struttura di reflection differisce. Impatta solo i
phpt `DirectoryClass_cannot_construct/clone/serialize`,
`DirectoryClass_readonly_{path,handle}`, `DirectoryClass_reflection_*` (8 test di
sole-semantiche-interne, già falliti quando la classe era del tutto assente →
nessuna regressione). Nessun framework reale istanzia/clona/serializza
`Directory` direttamente.

### 3.3 Late binding delle dichiarazioni di classe — nessuna deferral nei corpi dei TRAIT
Dal fix "Zend late binding" (una class-like con supertipo irrisolvibile compila
comunque e si binda quando la dichiarazione ESEGUE — `StmtKind::DeclareDeferred`
/ `ExprKind::NewAnonDeferred`, snippet ri-abbassato al punto di esecuzione con
autoload + `Error: Class|Interface|Trait "X" not found` fedele), resta UNA
eccezione consapevole: dentro i **corpi dei trait** la deferral è disattivata
(`resolve_trait` forza `DeferConf::No`). Motivo: i membri dei trait vengono
copiati verbatim nei consumer — anche in ALTRE unit — e l'indice nella tabella
`deferred` per-modulo penzolerebbe (le closure hanno il meccanismo di shift
cross-unit, i deferred no). Impatto: una classe anonima con supertipo
non-caricabile DENTRO un metodo di trait resta un errore di lowering eager
(pre-fix behaviour) invece del binding a runtime. Non osservato in alcun
framework reale; se emerge, la soluzione è dare ai deferred lo stesso shift
cross-unit delle closure. Nota bene: la permissività D-19.10 (forward reference
a classi dichiarate DOPO nello stesso file, che Zend early-binda solo se il
parent è già noto) resta INVARIATA — siamo più permissivi di PHP lì, e il
corpus non lo distingue.

### 3.3-bis `class_uses()` su un NOME di trait → `[]` (uses dei trait non registrati)
`get_parent_class`/`class_implements`/`class_parents`/`class_uses` accettano
nomi di trait (2026-07-13, filone http-kernel: DebugClassLoader::checkClass gira
su ogni simbolo autoloadato, trait inclusi). Oracle-pinned: parent → `false`,
implements/parents → `array(0)` — fedeli sempre, perché un trait non può
estendere né implementare. **`class_uses($trait)` invece riporta `[]` anche
quando il trait usa altri trait**: i `use` dei trait sono appiattiti al lowering
(`LoweredTrait` non conserva la lista). La *shape* (array, non false) è quella
che i chiamanti unionano (DebugClassLoader:488 `+ class_uses($class, false)`);
l'effetto residuo è solo la perdita delle deprecation ereditate via
trait-di-trait nel DebugClassLoader. Da chiudere aggiungendo `uses` a
`LoweredTrait`. Verificato 2026-09-10: `class_implements(enum)` INCLUDE
l'interfaccia esplicita; diverge solo l'ORDINE (phpr `UnitEnum, BackedEnum, I`;
oracle `I, UnitEnum, BackedEnum`).

### 3.3-ter Hoisting delle dichiarazioni = semantica PERSIST di opcache (divergenza dal CLI-oracle; EMENDATA A-DS40, Concilio WP-88)
⚠️ **Ricetta d'innesco CORRETTA (Stogov WP-88, refuta la versione WP-87)**:
l'inversione degli observable NON appare con il solo `opcache.enable_cli=1`
(SHM, prima esecuzione); appare **SOLO col branch PERSIST** —
`opcache.file_cache_only=1` (già al run1) ≡ richiesta calda FPM. Inoltre
`class C {}` semplice e parent-EARLIER sono early-bound in **ENTRAMBI** i
bracci (`class_exists` pre-decl = `true` anche a opcache OFF): la divergenza
esiste solo per le forme che l'early binding non copre (parent-LATER e
affini). Observable divergenti verificati dal vivo (php 8.5.7, ancore
committate in `wp87-harness/fixtures-ds40/` + `ds40-verify.out`, forma
parent-later):
1. `class_exists('C', false)` pre-decl: oracle plain `false` → persist/phpr
   `true` (`t_hoist_parent_later.php`);
2. `return;` prima della decl ⇒ classe mai dichiarata (plain) vs dichiarata
   (persist/phpr) — forma WP-87, stessa classe;
3. timing del fatal LSP (v. §3.3-quater; su phpr osservabile solo post
   A-DS35);
4. **costante di classe pre-decl** (`C::K`): oracle plain Error «Class "C"
   not found» exit 255 → persist/phpr `7` exit 0 (`t_hoist_const.php`);
5. **`get_declared_classes()` pre-decl**: `false` → `true`
   (`t_hoist_declared.php`);
6. **`new C` pre-decl (A-DS43, EMENDATA S-88.0)**: oracle plain Error
   «Class "C" not found» exit 255 → **persist/phpr `C` exit 0**
   (`t_hoist_new_predecl.php`). ⚠️ Il verbale Stogov WP-89 sosteneva che
   il persist fatalasse anche qui («phpr diverge da ENTRAMBI i bracci»,
   hoisting più ampio): **REFUTATO-DALL'ORACLE dal vivo** (2026-08-02,
   php 8.5.7 brew: run1 file_cache fresco, run2 cache calda, SHM-only
   `enable_cli=1` — TUTTI `C` exit 0; `ds40-verify.out`). Testata anche
   l'ipotesi di causa (file_cache rotto ⇒ fallback silenzioso a plain):
   REFUTATA — opcache fatala RUMOROSO su dir inaccessibile; **origine
   TROVATA A MACCHINA (Stogov, Concilio WP-90, ratifica)**: la flag
   `opcache.enable_cli=1` CADUTA con file_cache_only+file_cache validi ⇒
   opcache spento in silenzio ⇒ braccio-persist-MONCO che mima plain
   (fatal identico al verbale WP-89). Corollario: la riqualifica
   «const-folding» dell'observable 4 DECADE — poggiava sulla premessa
   `new`-not-found-sotto-persist, falsa; il meccanismo resta hoist.
**Negativo anti-vacuità (A-DS43)**: classe in blocco condizionale NON
dichiarata in NESSUNO dei tre bracci — `bool(false)`×3
(`t_hoist_conditional.php`): l'hoisting copre solo le decl top-level
incondizionate, in tutti i bracci allo stesso modo.
**Redeclare NON diverge**: `Cannot redeclare class C` con timing identico
(output già emesso) ed exit 255 su plain/persist/phpr (`t_redeclare.php`).
**phpr riproduce ESATTAMENTE il braccio PERSIST** (unit cache = persistent
script) — claim ora ANCORATO anche sulla forma `new` pre-decl
(KS-DS-89-2 soddisfatta). Il gate di parità CLI usa brew php con
opcache_cli OFF ⇒ questi observable sono divergenze OSSERVABILI dal
CLI-oracle, FEDELI al braccio persist. Classe: fedeltà-a-opcache-persist,
non bug; ogni corpus test che distingua i bracci va pinnato sul braccio
persist (KS-DS-88-2: entry senza fixture committata o citata con innesco
`enable_cli` = UNANCHORED).
⚠️ **Ricetta persist VINCOLANTE (A-DS47, Concilio WP-90)**: ogni run
«persist» citato in questo catalogo dichiara SEMPRE le TRE flag per NOME
(`opcache.enable_cli=1` + `opcache.file_cache_only=1` +
`opcache.file_cache=<dir>`) **E** verifica che il run1 abbia scritto ≥1
file `.bin` nel file_cache (il braccio-monco senza `enable_cli` è
SILENZIOSO e mima plain — la recidiva WP-89). Fixture compile-fatal
(nessun `.bin` per costruzione): il monco-check passa a un PROBE
persistabile con le stesse tre flag e mtime retrodatato
(`opcache.file_update_protection` default 2s non cachea file appena
scritti — scoperta S-89.0). Meccanizzata in `wp87-harness/ds40-verify.sh`
(binchk fail-closed). Run persist senza `.bin`-check o senza le tre flag
esplicite ⇒ observable UNANCHORED (**KS-DS-90-2**).

### 3.3-quater 🔴 Covariance/contravariance LSP NON verificata — correct-or-absent VIOLATO (gap engine, PRIMO item ROADMAP)
Scoperta GRAVE (Stogov, Concilio WP-87, fuori perimetro di sessione):
`class C extends P { function m(): int {} }` con `P::m(): string` in phpr
COMPILA e GIRA (exit 0, anche parent-first); l'oracle fatala «Declaration of
C::m(): int must be compatible with P::m(): string». Nessuna verifica di
varianza dei return type (covariance), dei parametri (contravariance) né dei
property type (invariance) esiste nel linker di classe. Violazione del
principio correct-or-absent: la classe è SBAGLIATA invece che assente — un
programma che l'oracle rifiuta gira in silenzio. **Decisione (A-DS35,
S-86.0): CORRECT** — la verifica LSP va implementata (fatal fedele al
messaggio Zend, al timing del braccio opcache per §3.3-ter), NON aggirata:
è il PRIMO item engine della ROADMAP ripresa ([[php-rust-todo-master]]).
Fino ad allora questo è il gap engine più grave a catalogo.
**Ancora committata (A-DS40)**: `wp87-harness/fixtures-ds40/
tC_lsp_covariance.php` — oracle fatala PRE-output («Declaration of C::m():
int must be compatible with P::m(): string», exit 255) su ENTRAMBI i bracci
(plain e persist: parent-earlier ⇒ early binding ⇒ check a compile time);
phpr stampa `out` ed esce 0 (verifica dal vivo in
`wp87-harness/ds40-verify.out`). La spec di chiusura è A-DS35 fase 1
(contratto A-DS41 in todo-master; merge vincolato da KS-DS-88-3).
Gli argomenti del costruttore di una `new class(...)` differita rieseguono nello
scope del chiamante via bridge per-nome dei named slots; `$this` non è un named
slot, quindi `new class($this->x) extends Irrisolvibile {}` dentro un metodo
non vede `$this` alla ri-esecuzione. Caso non osservato (i test Symfony usano
solo locals); da chiudere se emerge.

### 3.3-quinquies Fatal LSP: canali di emissione e contratto EMENDATO (A-DS48/A-DS50, Concilio WP-91 — S-90.0)
1. **Log-copy stderr NON modellata (divergenza per NOME)**: sul fatal LSP
   l'oracle CLI emette DUE blocchi — display su stdout («\nFatal
   error: …») E log-copy su stderr («PHP Fatal error:  …», doppio
   spazio). phpr emette SOLO il blocco display su stdout (stderr VUOTO;
   n7 byte-identico via od, S-89.0). Il bersaglio byte-fedele
   dell'implementazione A-DS35 è lo STDOUT integrale dell'oracle; la
   log-copy resta divergenza dichiarata. Pin a canali SEPARATI e
   INTEGRALI in `wp90-harness/ds35-verify2.out` (A-DS50 — il vecchio pin
   `2>&1|head -3` era un bersaglio byte-IMPOSSIBILE: teneva solo il
   blocco stderr che phpr non emette; KS-DS-91-3).
2. **Contratto r2 by-ref EMENDATO (A-DS48, refutazione oracle-viva)**:
   esatta è la REF-NESS (aggiungere O togliere `&` = fatal in entrambe
   le direzioni: fixture v3/v4); il TIPO del parametro by-ref resta
   contravariante (widening LEGALE: `int &$x`→`int|string &$x` alive,
   fixture v1). I messaggi portano le union nell'ordine canonico Zend
   («P::m(string|int &$x)»).
3. **Nome irrisolvibile nel check (fixture v15)**: l'oracle NON usa la
   forma «must be compatible» (lettera del verbale WP-91 imprecisa,
   ri-morso S-90.0): emette «Could not check compatibility between
   C::m(): B and P::m(): A, because class B is not available» (fatal,
   rc=255). La sede duale A-DS51 decide per NOME: fatal fedele a questo
   messaggio oppure divergenza a catalogo (mai skip silenzioso).
4. **Timing (fixture t1-t4, KS-DS-91-1)**: hoisted (t1/t4) = fatal al
   bind del set hoisted (persist: PRE-output; plain: post «pre|»);
   condizionale ESEGUITA (t2) = fatal DOPO l'output precedente su
   entrambi i bracci; condizionale NON eseguita (t3) = exit 0 —
   un'implementazione lowering-only che fatala t3 è REJECT.

### 3.5 INI table parziale (filone ext/session, 2026-07-12)
La tabella INI (`vm/ini.rs`) registra solo le direttive modellate: 31 `session.*`
(+ `session.trans_sid_tags`/`hosts`, esenti dal freeze headers-sent e dal listing
`ini_get_all('session')`, oddity oracle-verificata), `include_path` e le ~9
chiavi engine-hardwired storiche. Divergenze deliberate:
- `ini_get_all(null)` elenca ~45 direttive, non le ~291 di PHP; un'estensione
  diversa da `session` → warning "cannot be found" anche per estensioni che PHP
  conosce (`Core`, `standard`, …).
- Le chiavi hardwired (`precision`, …) rifiutano `ini_set`
  (ritorno `false`): meglio un set che fallisce di uno che mente (l'engine non
  le consulterebbe).
- `include_path` è settabile e viene EMBEDDED nei messaggi di include-failure,
  ma il resolver resta cwd-based: `set_include_path('dir1:dir2')` non estende la
  ricerca (Zend/tests `bug39542`, `exceptions/exception_during_include_stat`
  fail onesti).

### 3.6 ext/session: residui dichiarati (filone 2026-07-12)
- **trans-sid / url rewriting assente** (`session.use_trans_sid=1` non riscrive
  l'output; ~15-19 phpt): serve l'infrastruttura url_rewriter.
- **Costante SID assente** (+ deprecation-on-read PHP 8.4): 52 phpt la citano.
- `unserialize()` riporta sempre "Error at offset 0 of N bytes" (l'offset reale
  non è tracciato; oracle `offset 18 of 18`); `R:` è a parità (verificato
  2026-09-10, `&` incluso nel var_dump) ma `r:` su un non-oggetto NON è
  validato (phpr restituisce `[5, NULL]`, l'oracle fallisce a offset 21); il
  C:-format con ref interni (bug79031) non è supportato.
- `open_basedir` non modellata (gh13856); ReflectionFunction sulle funzioni
  interne non costruisce descriptor (bug74541).
- Il flusso `phpr -d`: gli override si applicano SOLO alle direttive registrate
  (identico all'invisibilità di `php -d unknown=x` a `ini_get`).

### 3.7 🟡 Residui della probe string WP-38 (le chiusure WP-39 sono state ri-verificate byte-identiche il 2026-09-10 e rimosse)

- **Stringable sotto `SORT_STRING`/`SORT_NATURAL`**: phpr emette il fallback
  warning "could not be converted to string" + placeholder invece di chiamare
  `__toString` (ordine `b,a,c` vs oracle `a,b,c`): sort non può stare nel gate
  statico §1.1 perché i `$flags` sono runtime; inoltre la coercion è 1× per
  elemento up-front, non per confronto come Zend.
- **`isset($s[1.5])`** non emette la deprecation "Implicit conversion from
  float 1.5 to int loses precision".
- **`$s["1abc"] ?? $d`**: Zend emette l'Illegal-offset warning e LEGGE il
  prefisso (`"b"`), phpr prende silenziosamente il default.

---

### 3.8 Famiglia opcache/unit-cache (depositata in WP-63, design62 §3 (i)-(iv))

La unit-cache di phpr (WP-20, chiave path+mtime+size / fingerprint VM) è
il MECCANISMO analogo a opcache; le divergenze sono di meccanismo, MAI di
output osservabile (il gate lo asserisce):

- **(i)** L'oracolo CLI NON ha opcache attivo; phpr con unit-cache diverge
  in meccanismo (salta lower+compile sul hit) ma mai in output.
- **(ii)** `opcache_reset()` / `opcache_invalidate()` / `opcache_*` API:
  ASSENTI (`function_exists` false); l'oracle CLI brew ha l'estensione
  CARICATA anche con `enable_cli=0`, quindi le funzioni esistono (verificato
  2026-09-10). `TODO(port)`: stub onesti se un workload reale li chiama.
- **(iii)** Retention illimitata su edit ripetuti in php-server: i Module
  sono `Box::leak` `'static`, un supersede NON libera il vecchio
  (`TODO(port)` de-leak su supersede — Leijen R5/KS4; il budget andrà
  per-BYTES, mai per-entries; con axum ×N worker la cache thread_local
  si moltiplica — nota Pedersen).
- **(iv)** `clearstatcache()` vs mtime cacheato nella unit-key: la
  file_update_protection (~2s, mtime giovane ⇒ hash sempre) copre il
  bordo write→include→rewrite; la semantica di clearstatcache sui
  metadata della unit-key resta divergente in meccanismo.
- **(v, WP-63)** Ordine di enumerazione (`get_declared_classes`): l'ordine
  di push nella tabella runtime è INVARIATO dalla stub-elision (sentinella
  KE-a); se un futuro cambio di rappresentazione lo toccasse, la voce va
  promossa a divergenza reale con probe.
- **(vi, WP-64 — probe S-4 Stogov, `wp64-harness/probe64-s4.php`;
  razionale emendato dal concilio S-65.1)** Suffisso user di
  `get_declared_classes` vs oracolo: DIVERGE per OGNI classe
  condizionale legata fuori ordine di sorgente della compile-unit
  (non solo "inclusa tra due incondizionali": anche main script,
  anche classe-in-funzione chiamata tardi). Meccanismo Zend VERO
  (zend_compile.c 8.5.7): le incondizionali si registrano a compile
  time (hoisting, r.9381); le CONDIZIONALI entrano comunque a compile
  time sotto **RTD key** NUL-prefissata (r.9428-9433) = bucket
  segnaposto in POSIZIONE DI SORGENTE; al bind runtime
  `zend_bind_class_in_slot → zend_hash_set_bucket_key` (r.1313)
  ri-chiava il bucket IN PLACE preservando la posizione;
  `get_declared_classes` salta chiavi NUL e alias. (⚠️ NON è "early
  binding ritardato": `DECLARE_CLASS_DELAYED` esiste solo con
  opcache.) phpr registra in ordine di REGISTRAZIONE (incondizionali
  al link prima del body, condizionale al suo `DeclareClass` ⇒
  `Inc1, Inc2, Ghost` vs Zend `Inc1, Ghost, Inc2`; con una classe-in-funzione
  è peggio dell'esempio: phpr `InFn, Inc1, Inc2, Inc3, Ghost` contro oracle
  `Inc1, Ghost, Inc2, InFn, Inc3` — la classe in funzione è issata PRIMA di
  tutte, la condizionale ultima; verificato 2026-09-10). PRE-ESISTENTE
  alla stub-elision (probe identico a `PHPR_STUB_ELISION=0`) e mai
  colta dai gate. Tripwire che RIAPRE la voce (S-65.2): l'idioma
  `end(get_declared_classes())` / diff-after-include (loader di
  plugin, discovery) sceglierebbe la classe SBAGLIATA quando un
  polyfill condizionale scatta. Costo di chiusura: slot segnaposto
  posizionale RTD-like = Zend-fedele MA tocca la semantica di
  registrazione ⇒ KS-S65.3: non si implementa senza riaprire
  RULEBOOK §4 (il name-check dell'identity arm deve distinguere
  pending da misaligned, pena falsi fatal).
- **(vii, WP-65 — probe S-65.3, `wp65-harness/sem-units/`; RIDOTTA il
  2026-09-10 a ciò che si riproduce)** Scope-variabili al toplevel: (a)
  `get_defined_vars()` a toplevel di unit INCLUSA OMETTE i superglobals che
  Zend include a global scope (`_GET`,`_POST`,`_COOKIE`,`_FILES`,`_SERVER`);
  (b) l'ORDINE di enumerazione di `$GLOBALS` diverge (phpr
  `defined,viaglobals,argv,argc,k` vs oracle `argv,argc,defined,k,viaglobals`).
  NON si riproducono più (probe 2026-09-10): slot del main non definiti
  elencati o presenti come NULL, `_SESSION` in `$GLOBALS`,
  `array_key_exists` che mente, e il warning «Undefined variable» a toplevel
  del main (ora emesso). Bordo reale per i temi WP; da chiudere con un fronte
  scope-hygiene dedicato.

### 3.9 php-server (Axum SAPI): corpi d'errore HTTP (S-78.1.4, A-DS10 Council WP-79)
- **Contratto A-TH8** (gate: `wp78-harness/gate-axum/run-gate.sh` fixture
  fatal + test cargo `fatal_maps_to_http_500_compile_and_runtime`): ogni
  fatal (compile E runtime) → **HTTP 500** con body = STDOUT CLI byte-parity
  (il worker rende via `Vm::render_fatal`, la stessa funzione del main;
  script name = path FILESYSTEM risolto, come SCRIPT_FILENAME FPM).
  `exit()/die()` → **200** con l'output catturato (terminazione pulita FPM;
  il codice di uscita non ha canale HTTP). Throwable non catturato con
  `set_exception_handler` attivo → **200** senza banner (epilogo del main).
- **DIVERGENZA 1 — html_errors** (A-DS10): php-fpm/cli-server di default ha
  `html_errors=1` e serve l'error-page con markup (`<br />\n<b>Fatal
  error</b>: …`); phpr in modalità `--axum` serve la forma PIANA del CLI
  (`\nFatal error: … Stack trace…`). Scelta deliberata: la parità
  byte-per-byte è ancorata all'oracle CLI (display_errors=On), il canale
  html_errors del web SAPI non è ancora cablato nel worker Axum.
- **DIVERGENZA 2 — messaggio dei Parse error**: il testo del diagnostico
  del parser phpr NON è quello Zend (`syntax error, unexpected token …`)
  ma il dump del parser interno (`UnexpectedToken(…Span…)`). L'ENVELOPE è
  quello dell'oracolo (`\nParse error: {msg}\n`, HTTP 500), il testo dentro
  DIVERGE. **Nessun claim di parity su questi body** (KS-DS-78-5); vale
  anche per il CLI phpr (stderr `PHP Parse error: {dump}`; l'oracolo con
  display_errors=On stampa il Parse error anche su stdout, phpr no).

### 3.10 🔴 Argomenti `string` dei builtin: coercizione con warning invece di `TypeError` (S-96.0)

Trovata di lato mentre si costruivano le fixture di liveness A-ZV2 (la
fixture voleva un builtin che LANCIASSE prima di scrivere il suo out-param, e
non lanciava). Verificata sul binario di PARITÀ, non su una build strumentata.

Passando un `array` (o un oggetto senza `__toString`) dove il builtin dichiara
`string`, PHP 8 solleva
`TypeError: f(): Argument #N ($x) must be of type string, array given`;
phpr invece **coercizza**, emette `Warning: Array to string conversion` (o
`Warning: Object of class X could not be converted to string`) e **prosegue**.

Sonda (`preg_match`, `preg_split`, `explode`, `substr`, `strtoupper` con un
array): l'oracle lancia su tutte, phpr su nessuna. `strlen` è già CORRETTO
(lancia), quindi il difetto non è nel motore dei tipi ma nel **parsing dei
parametri dei singoli builtin**: chi passa dal percorso stringente lancia, chi
usa la conversione generica no.

- **Perché conta oltre al messaggio**: cambia il FLUSSO, non solo il testo. Un
  `try/catch (\TypeError)` che l'oracle prende, phpr lo salta; e l'out-param
  che l'oracle lascia INTATTO, phpr lo sovrascrive. È la classe di divergenza
  che i test di parità testuale non vedono, perché il programma non stampa
  niente di diverso finché qualcuno non guarda la variabile.
- **Perimetro non misurato**: la sonda è di poche funzioni, scelte a mano. La
  cardinalità reale (quanti builtin sbagliano) NON è stata misurata: chiamarla
  «alcuni builtin» sarebbe una stima travestita da conteggio.
- **Stato**: APERTA, non affrontata in S-96.0 (fuori dall'oggetto della
  sessione). Voce nella lista master.

### 3.11 🔴 AssignOp con lhs indefinito: manca il warning «Undefined variable» (S-100)

Trovata costruendo la trappola (e) di A-ST-99-3 (ordine warning undef-lhs).
Verificata sul binario di parità, identica flag-off e flag-on (NON è del pass).

`$u += expr;` con `$u` mai definito: l'oracle valuta il rhs, emette
`Warning: Undefined variable $u`, poi applica l'op (null + …); phpr valuta il
rhs e applica l'op **senza alcun warning** — in funzione E a toplevel (il
catalogo §3(c) copriva solo l'`echo` a toplevel: questa è la stessa famiglia
ma sul percorso compound-assign, più ampia). Il VALORE risultante è a parità;
manca il diagnostico. Fixture: `wp100-harness/assignop-traps/e-undef-warning-order.php`
(attesa-divergente per NOME in `s100-assignop-oracle.sh`).

- **Stato**: APERTA (S-100, trovata di lato — fuori dall'oggetto promozione).
  L'ordine rhs→warning→errore-op non è collaudabile finché il warning non esiste.
- **Perimetro MISURATO (S-103, A-ST-104-2)**: la famiglia è TUTTO il canale
  read-modify-write su lhs indefinito — compound-assign (`+=`,`-=`,`.=`,…),
  incr/decr (`$u++`, `++$u`) e le array-key nei RMW (due specie: var
  indefinita = 2 warning mancanti; key indefinita su var definita = 1).
  Valori sempre a parità; `??=` (controllo) a parità piena. Probe:
  `wp103-harness/censimento-311-312/` (verdetto in `censimento-verdetto.out`).

### 3.12 🔴 Typed-LVALUE: AssignOp fallito — Zend AZZERA, phpr conserva (S-100; rititolata S-103)

Trovata costruendo la trappola (b) di A-ST-99-3. Verificata sul binario di
parità, identica nei due modi (NON è del pass).

```php
class T { public int $i = 1; }
$t = new T; $r = &$t->i;
try { $r += "abc"; } catch (\TypeError $e) {}   // Unsupported operand types
echo $t->i;   // oracle 8.5.7: 0 (!) — phpr: 1
```

Dopo il `TypeError` dell'op, l'oracle lascia il typed-ref **azzerato** (int 0:
plausibilmente UNDEF ri-coercizzato dal type-check del ref); phpr conserva il
valore precedente. Il messaggio del TypeError è a parità; diverge lo STATO
post-errore. phpr qui è «più ragionevole», ma la policy è byte-parity con
l'oracle: la voce resta aperta finché non si replica il comportamento Zend (o
non lo si dichiara assenza consapevole). Fixture:
`wp100-harness/assignop-traps/b-typed-ref.php` (attesa-divergente per NOME).

- **Stato**: APERTA (S-100, trovata di lato — fuori dall'oggetto promozione).
- **Perimetro MISURATO (S-103, A-ST-104-2)**: 🔵 il titolo originario
  «typed-REF» era troppo stretto — l'oracle azzera il typed-LVALUE dopo
  QUALUNQUE AssignOp fallito, anche la proprietà diretta SENZA ref
  (`$t->i += "abc"` ⇒ oracle `i==0`, phpr `i==1`). 4/4 specie divergenti:
  ref a prop typed, ref a static typed, prop diretta, param by-ref.
  TypeError e messaggio a parità; diverge SOLO lo stato post-errore.
  Probe: `wp103-harness/censimento-311-312/`.
- **⚠️ EMENDA WP-105 (Stogov, A-ST-105-1 — TRE regimi, non uno)**: il
  censimento 4/4 era TUTTO nel regime (i) «op fallisce + weak-mode» (⇒
  Zend ri-coercizza UNDEF allo zero del tipo). Regime (ii)
  `strict_types=1`: Zend **CONSERVA** (phpr già a parità); regime (iii)
  op RIESCE ma il verify di tipo fallisce (es. `.=` su typed int): Zend
  **CONSERVA**. Un eventuale fix deve replicare la catena
  UNDEF→verify-weak PER TIPO e portare bracci strict e `.=` nel gate
  (KS-ST-105-1) — un azzeramento indiscriminato sarebbe una divergenza
  NUOVA. Dettaglio: `wp105-harness/verbali/verbale-8-stogov.md`.

### 3.13 🟡 Warning «Undefined property»: la marca di riga non porta l'UNITÀ (include/eval) (WP-105, Stogov A-ST-105-3)

La famiglia PropGet è a parità sulla RIGA dal S-102 (`diag_line_marks` +
`mark_pending_diag_lines`; ri-verificata byte-identica il 2026-09-10 su
lettura di proprietà rimossa a due righe diverse). Resta il canale UNIT: la
marca porta la riga ma non l'unità — se la lettura avviene in un file incluso
e il flush cade nell'includente, il warning esce con riga giusta ma **file
del flush** (oracle `inc.inc on line 2`, phpr `main.php on line 2`); su
`eval` manca lo pseudo-file (`main.php(7) : eval()'d code on line 1`). La
marca deve diventare (unit, line). Claim ridimensionato (S-103, A-ST-104-2):
la disciplina della marca copre 5 siti su ~435 punti di accodamento
diagnostico; le altre famiglie di warning accodati restano timbrate al flush
(es. §3.11 quando il warning nascerà).

### 3.14 🔴 `memory_get_usage`/`memory_get_peak_usage` = STUB costante (scoperta S-104, mutation-check fx20)

`memory_get_usage()` di phpr restituisce una **costante** (2.000.000),
qualunque sia lo stato reale dell'heap; scoperto NON dalla review ma dal
mutation-check della fixture fx20 (Str→forget: il leak indotto non muoveva
la cifra ⇒ verdetto in-script VACUO). Conseguenze già vincolate:

- **KS-MA-106-1 (Concilio WP-106): nessun verdetto di fixture o gate può
  poggiare su `memory_get_usage` finché è stub** — il verdetto di leak
  vive nel braccio RSS del gate (fx20: cap 150 MiB, clean ~50 vs mutante
  ~301).
- Cura pre-approvata a **due gradini** (Stogov A-ST-106-1, KS-ST-106-1):
  contatore per-thread TLS oppure interrogazione mi_* on-demand;
  functional-parity DICHIARATA (mai byte-parity con Zend, i due heap
  contano cose diverse); REFUTATI gli atomics process-global in release
  (conflazionano i worker del server e tassano calls). Da eseguire SOLO
  fuori dalla finestra di una leva.
- Violazione del principio **correct-or-absent**: uno stub che mente è
  peggio di un'assenza — la voce resta 🔴 finché il contatore non è vero.

### 3.16 🔴 Riga sbagliata nel warning «Undefined variable» del RICEVITORE di un prop-assign (scoperta S-109, fixture w9a caso B)

`$u->p = $u->p + 1;` con `$u` mai assegnata, dentro un `try { } catch`:
l'ordine e i messaggi sono ORACLE-IDENTICI (Warning undef-var → Warning
read-prop-on-null → Error assign-prop-on-null), ma phpr attribuisce il
PRIMO warning alla riga dell'`echo` dentro il blocco `catch` (riga 17)
invece che alla riga dello statement (riga 15). BILATERALE (on ≡ off:
NON è un effetto delle finestre fuse — diverge identicamente a pila
pura). Indiziato (non provato): la `lines[]` dell'op che legge il
ricevitore del write (`AssignPath`/base-fetch) o l'emissione pigra del
warning che legge la riga all'ip corrente. Fixture repro parcheggiata:
`wp109-harness/w9-fixtures/parked-w9a-caso-b-receiver-undef.php` (fuori
dal gate finché la voce non è curata; il gate w9 copre il caso A
__get-che-lancia, byte-identico).

### 3.17 🟡 Riga sbagliata nel warning «A non-numeric value encountered» (scoperta S-111, revisore semantica sul giudice held-out err)

`$sum += @("12x" + 1);` — senza `@`, l'oracle attribuisce il warning alla
riga dell'espressione aritmetica; phpr lo emette PER-iterazione (nessun
folding: il conteggio è giusto) ma lo attribuisce alla RIGA D'USO
SUCCESSIVA (repro del revisore: riga 6 vs riga 4 oracle). Stessa famiglia
di §3.13/§3.16 (emissione pigra che legge la riga all'ip corrente invece
che all'op che genera). Nel giudice `wp111-harness/heldout/err.php` la
divergenza è NASCOSTA dal `@`: la parità d'output del giudice NON
certifica la diagnostica soppressa (nota dichiarata nel README held-out).

### 3.18 🔴 `preg_match`: due piste fredde divergenti (scoperte S-121, fixture az. rev. S-120 #1)

Fixture congelata `wp121-harness/fixtures/fx-preg-re1.php` (12 casi per NOME),
BILATERALE on ≡ off sul pin s120:
- **(a) nomi duplicati con `(?J)`**: `/(?J)(?<x>a)|(?<x>b)/` su `'zb'` —
  l'oracle matcha (`{"0":"b","x":"b","1":"","2":"b"}`), phpr ritorna
  `false` con `$m = null` (compile della pattern rifiutato). Riga 2 del gate.
- **(b) nome utente col prefisso sintetico `__phprbg`**: `/(?<__phprbg1>z)/`
  — l'oracle espone la chiave nominata, phpr la NASCONDE (il filtro di
  `Engine::capture_names` non distingue i sintetici di `demix` dai nomi
  utente; già presente PRIMA di L-RE1 — revisione S-120). Riga 11 del gate.

Le altre 10 piste (nomi base, `(?|)`, NULL/unmatched, PREG_UNMATCHED_AS_NULL,
PREG_OFFSET_CAPTURE, offset arg, subject latin1, backref demix `\1`, mix
nome+backref, no-match) sono BYTE-IDENTICHE nei 2 modi. Gate fail-closed:
`wp121-harness/s121-fx-preg-gate.sh` (golden phpr pinnati per riga 2/11 —
alla cura il gate diventa ROSSO e i golden si aggiornano nello stesso commit).

### 3.19 🔴 `__halt_compiler` statement-level + phar stub: `composer.phar` NON eseguibile (S-126, mappa2 p.7)

`phpr composer.phar …` muore a t=0 con `Parse error: unsupported construct
(stmt:HaltCompiler)` (riga 30 dello stub). Capability phar onestamente assente
(cfr. `stream_get_wrappers`); già noto come residuo tokenizer (bug54089), qui
la conseguenza pratica: ogni tool distribuito come phar non parte. Workaround
canonico storico: sorgente estratto (`bin/composer`), verde a luglio. ⚠️
AGGRAVANTE (S-126, stessa sera): anche il composer 2.10 ESTRATTO muore su
phpr con **rc=255 SILENTE** (zero output su stdout+stderr; shebang escluso
con probe; smoke rc=0 esatto dell'arbitro ha rifiutato la misura). Da
bisecare in S-127: regressione phpr vs costrutti nuovi di composer 2.10
(a luglio girava il composer di allora). Fixture di fatto: gambe compoff
run1 + abort compoff2 in `wp126-harness/mappa2-out/`.
**BISEZIONE CHIUSA (S-127)** → verdetto `wp127-harness/s127-compoff-bisez-verdetto.out`:
il rc=255 silente era la COMPOSIZIONE di §3.19-bis + §3.19-ter (entrambe curate
in S-127, ri-verificate al byte e rimosse dal catalogo il 2026-09-10; residuo
in §3.31), innescata
dalla guardia sudo di Composer (`Application.php:246`,
`Silencer::call('exec', "sudo -K …")`). Nessun costrutto nuovo del 2.10 in causa.

### 3.19-quater 🟡 canale log CLI (`log_errors=1`) → stderr ASSENTE (S-127)

L'oracle CLI (log_errors=1, error_log vuoto) scrive ogni diagnostica ANCHE in
forma-log «PHP Warning:  …» su stderr; phpr non emette il canale log su CLI.
Pre-esistente, scoperta con le sonde cure319 (che lo spengono con log_errors=0
per confrontare il canale display). Cura da valutare con cautela: tocca lo
stderr di OGNI run CLI (harness che uniscono 2>&1).

### 3.19-quinquies 🟡 composer install: post-install phpcs config-set fallisce (S-127)

Con le cure ondata-2 `composer install` offline COMPLETA (rc=0, vendor_ok,
gambe bilaterali): l'UNICA riga stdout divergente è il plugin
phpcodesniffer-composer-installer — oracle «PHP CodeSniffer Config
installed_paths set to …» vs phpr «Failed to set …». Pista: residuo famiglia
processo nel plugin. stderr diverge solo per granularità progress-bar (timing).
Nella stessa catena: FILTER_FLAG_EMAIL_UNICODE aggiunta (json-schema email) ·
drift SimpleXML sanato nella lista cased di get_loaded_extensions ·
**iconv DICHIARATA col patto test-driven** (iconv() core c'è;
strlen/substr/mime_* mancanti = onesto undefined function) + costanti ICONV_*
dall'oracle. La voce compoff della mappa è RIAPERTA (rimisura S-128).

### 3.20 🟡 doctrine/dbal 4.4-dev: 10 fail per NOME phpr-only (S-126, mappa2)

Fail-set stabile 2/2 gambe (0,25% di 3929; nomi integrali nel verdetto
`wp126-harness/s126-mappa2-verdetto.out`): famiglia
`Functional\PortabilityTest` (9: testCaseConversion*, testFetch*,
testFullFetchMode, testGetDatabaseName — middleware Portability su sqlite,
da bisecare) + `Schema\Name\Parser\GenericNameParserTest::testValidInput #11`
(identificatori unicode `schéma."übermäßigkeit"…` — `ExpectedDot at position
9`: sospetto offset byte-vs-char nel lexer dei nomi, pista preg/mb da
verificare col manuale). Oracle 0 fail equivalenti.

### 3.21 🟡 Diagnostica dim-set (array offset in SCRITTURA): tre divergenze pre-esistenti (revisore S-135, catalogate S-136)

Trovate dal revisore semantico S-135 sui probe della leva AP1 (identiche su pin
s135 E stash s134 ⇒ PRE-esistenti, non-leva; probe `rev135-p1/p2.php`,
fixture `wp135-harness/fixtures-ap1.php` s8/s9/s12):

- **(a) messaggio TypeError chiave illegale**: `$a[[]] = 1` → phpr
  «Illegal offset type» vs oracle 8.5 «Cannot access offset of type array
  on array» (il testo Zend è per-tipo e per-contesto; phpr usa il messaggio
  legacy). Stessa classe, messaggi diversi ⇒ i phpt con expect sul testo
  divergono.
- **(b) deprecation 8.5 mancanti sul cammino dim-set**: `$x[null]` in
  scrittura non emette la Deprecated «null array offset» 8.5; `$f = false;
  $f[0] = 1` non emette la Deprecated «Automatic conversion of false to
  array» (il valore/verdetto finale è corretto in entrambi i casi).
- **(c) riga del Deprecated float-key attribuita a uno statement SUCCESSIVO
  (punto di FLUSH del canale diag)**: fixture s8 su una riga sola lo
  mascherava; spezzata (S-136) espone +1 nel probe del revisore (28 vs 27)
  e +5 nella fixture v2 (52 vs 47: il flush avviene alla READ successiva
  dell'array, `var_dump`). Famiglia §3.13/§3.16/§3.17 (canale diag che
  legge la riga del pc al momento del flush, non del sito).

### 3.22 🔴 `unset($a[k])` su ELEMENTO d'array: `__destruct` DIFFERITO al drop dell'array (S-142, pre-esistente)

Trovata dalla micro di parità az.rev. S-141 #4 (`wp142-harness/parita-hashed.php`
+ sonda a 3 casi `probe-unset.php`): **identica su stash s140 E candidato L-RD1
⇒ PRE-esistente, non-leva** (A==B byte-identico nei 2 modi).

- Zend: `unset($a[k])` distrugge il valore ALL'ISTANTE (il `__destruct`
  dell'oggetto contenuto scatta sull'unset, prima dello statement successivo).
- phpr: il tombstone (packed E hashed) TIENE VIVO il valore fino al drop
  dell'ARRAY contenitore (fine scope o ultimo Rc): il `__destruct` slitta lì.
  `unset($var)` su variabile è invece a PARITÀ (eager).
- Conseguenze: (1) ordine/timing dei `__destruct` osservabilmente diverso nei
  programmi che usano `unset` di elemento come rilascio-risorsa (unlink, close);
  (2) footprint: subtree tombstonati restano vivi finché vive l'array.
- Cura NON tentata in S-142 (catena L-RD1 in corso; toccare il drop-path
  avrebbe invalidato l'A/B): apertura per NOME. Una cura deve citare i fail
  del corpus congelato che flippa (famiglia destructors/gc del fail-set).

### 3.23 🟡 `debug_backtrace`: residui OLTRE il perimetro BT1 (S-150, az.rev.1 lente SEMANTICA)
- La leva BT1 (pin s150) onora `options` int e `limit` sulla forma della
  fixture (7 combinazioni, un call-site metodo): fx-backtrace byte-id ×2 modi.
- RESIDUO per NOME: `Zend/tests/backtrace/debug_backtrace_options.phpt` resta
  FAIL (contenuto MUTATO al flip S-150: diff ridotto, non chiuso). Superfici
  non coperte dalla fixture: `options` non-int (true/false/coercizioni),
  `PROVIDE_OBJECT|IGNORE_ARGS` combinati, `limit` > profondità reale,
  call-site closure/include, VALORI di args (la fixture stampa solo chiavi).
- Cura futura: estendere fx-backtrace a quelle superfici e citare il flip di
  `debug_backtrace_options.phpt`.

### 3.24 🟡 `debug_print_backtrace`: `options`/`limit` IGNORATI (asimmetria con la gemella, S-150)
- Perimetro BT1 dichiarato (s149-criterio-bt1.md p.1): `ho_debug_print_backtrace`
  INVARIATO (`collect_backtrace()` = delega a `_opt(0,false)`) ⇒ i suoi
  parametri restano ignorati mentre `debug_backtrace` ora li onora.
- Fail per NOME nel congelato: `Zend/tests/backtrace/debug_print_backtrace_limit.phpt`
  (in famiglia, NON flippato da BT1 — atteso).
- Cura futura (leva di fedeltà candidata S-151): passare options/limit anche
  qui; bersaglio dichiarato = flip di `debug_print_backtrace_limit.phpt`.

### 3.25 🟡 `debug_backtrace` DENTRO l'autoloader: manca il frame del builtin innescante (S-157, az.rev. S-156 #3, sonda bilaterale)
- Sonda `wp157-harness/sonda-bt-autoload.php` (oracle vs pin s156 vs braccio
  L-AL1: i due phpr IDENTICI — contratto storico, NON introdotto da HD2/AL1):
  in Zend il backtrace dentro l'autoloader innescato da
  `class_exists`/`interface_exists` mostra `#0 {closure}` E
  `#1 class_exists nargs=1 a0=<nome COME SCRITTO, backslash incluso>`;
  in phpr appare SOLO `#0 {closure}` (il frame del builtin chiamante manca:
  gli args escono dal frame prima del dispatch — contratto HD2 §5 rev. S-156).
- Il nome passato ALL'AUTOLOADER è invece identico (strip del `\` iniziale,
  case preservato) — `AL:` byte-uguale sui tre motori.
- Cura futura candidata: frame sintetico del builtin nel collettore backtrace
  quando il run_loop è annidato in un hostcall (bersaglio: la riga `#1` della
  sonda byte-uguale all'oracle).

### 3.26 🟡 string-callable dinamici: tre divergenze scoperte al collaudo fixture L-AM2 (S-162 + estensione S-163, PRE-esistenti alla leva)

Scoperte con `wp162-harness/fx-sm.php` sul pin s161 (PRIMA dell'edit L-AM2;
la leva non le tocca per costruzione — gate d'invarianza `fx-sm-div.php`,
pin==stash BYTE-ID in promozione):
1. `array_map('f', …)` con `function f(&$x)` (parametro by-ref): l'oracle
   emette `Warning: f(): Argument #1 ($x) must be passed by reference,
   value given` per elemento e procede; phpr chiama in silenzio (nessun
   warning). Stessa famiglia del trampolino §3.19-bis, ma per funzioni
   UTENTE via callable dinamico.
2. callback INESISTENTE: l'oracle lancia `TypeError: array_map(): Argument
   #1 ($callback) must be a valid callback or null, function "…" not found
   or invalid function name`; phpr lancia `Error: Call to undefined
   function …()` (classe E messaggio diversi — la validazione del callable
   in Zend avviene PRIMA della chiamata, in phpr al dispatch).
3. **(S-163, rev. S-162 rilievo 3)** callback INESISTENTE con array
   VUOTO: l'oracle valida il callback PRIMA di iterare e lancia lo stesso
   `TypeError` del punto 2 anche su `array_map('undef', [])`; phpr, col
   loop a entries vuote, non chiama mai e restituisce `[]` MUTO (nessun
   errore). Conseguenza diretta del punto 2 (validazione al dispatch e
   non all'ingresso). Fixture: `fx-sm-div.php` (esteso S-163).

### 3.27 🟡 autoload lista LIVE: self-unregister CON successore non ferma la camminata (S-163, PRE-esistente)

Scoperta con `wp163-harness/fx-au.php` sul pin s162 (PRIMA della leva
L-AU1; la leva non tocca il cursore per costruzione — gate d'invarianza
`fx-au-div.php`, pin==gemello BYTE-ID in promozione): un loader che si
DE-registra DA SOLO durante il lookup e ha un SUCCESSORE registrato —
l'oracle TERMINA la camminata (il successore NON scatta); phpr, col
cursore element-stable di S-71.2, prosegue e lo chiama. Il perimetro
S-71.2 copriva il self-unregister SENZA successore (fine camminata,
byte-id) e l'unregister di un ALTRO loader (fx-au t8, byte-id): il caso
misto è nuovo. Corredo: in Zend la rimozione del bucket corrente mette
l'iteratore sulla sentinella di fine, qualunque cosa segua.

### 3.28 🟡 argomenti di chiamata: ordine di VALUTAZIONE (SEND_VAR_EX) e timing/ordine dei dtor dei temp (S-166, fixture fx-mc2, PRE-esistenti alla leva L-MC1d)

Scoperte con `wp165-harness/fx-mc2.php` (probe del revisore S-165); **pin
s165 == stash s163 BYTE-IDENTICI** su ogni caso ⇒ divergenze del FUNNEL,
non della leva. Due membri della stessa famiglia:
(i) **ordine di valutazione**: Zend valuta gli argomenti LEFT-TO-RIGHT
(`$o->add($a[], new R)`: l'Error «Cannot use [] for reading» scatta PRIMA
di costruire `R`, che non nasce mai); phpr valuta i value-arg al push e
DIFFERISCE i place (SEND_VAR_EX) alla materializzazione ⇒ `R` nasce e
muore, il suo dtor appare, il riuso degli handle-id diverge. Vale per ogni
place con side-effect (`__get`, warning) che PRECEDE un value-arg.
(ii) **timing/ordine dei dtor dei temp-argomento al ritorno**: Zend
distrugge i temp PRIMA che il valore di ritorno sia consumato
(`dtor:Da 9`); phpr DOPO il consumo (`9 dtor:Da`) — stessa radice
timing-only delle entry WP-46/WP-56. Conteggi e contenuti identici.
L'«ordine slot inverso» (`Dy, Dx`) NON si riproduce su una funzione utente
a due argomenti (entrambi `Dx, Dy`, verificato 2026-09-10): resta la sola
componente di timing.

### 3.29 🟡 `Fiber` non è `final`: phpr accetta `class X extends Fiber` (S-166, fixture fx-mc2-fib)

L'oracle 8.5 muore a compile («Class MyFib cannot extend final class
Fiber»); phpr compila ed esegue (i metodi Fiber su un'istanza subclass
passano comunque da `fiber_method`). Nota di sistema: il linguaggio stesso
rende IRREALIZZABILE il caso «Fiber-subclass nell'IC» temuto dalla
revisione S-165 — la soundness IC è garantita due volte. Perimetro
probabile: enforcement di `final` sulle classi NATIVE (da censire).

### 3.30 🟡 default di proprietà `float` scritto come literal int NON coerce a float (S-172, fixture fx-sl2-div)

`class T2 { public float $f = 1; }` → oracle `var_dump($t->f)` = `float(1)`
(coercizione int→float del default a compile della classe); phpr = `int(1)`.
La scrittura a runtime coerce correttamente (`$t->f = $t->y + 1` → `float(4)`
== oracle, presidiata in fx-sl2 bilaterale). PRE-esistente alla leva L-SL2
(pin s171 == candidato). Perimetro misurato 2026-09-10: anche `?float $g = 2`
e `static float $s = 3` danno `int`; la promozione nel costruttore
(`float $p = 4`) è a parità.

### 3.31 🟡 `call_user_func('proc_open')` senza argomenti: manca l'`ArgumentCountError` (residuo di §3.19-bis, 2026-09-10)

I builtin di processo come callable dinamico (`$f='exec'`, `call_user_func`,
`is_callable`) sono a parità con l'oracle (cura S-127, ri-verificata al byte
il 2026-09-10 e rimossa dal catalogo). Unico residuo: `call_user_func('proc_open')`
con zero argomenti — l'oracle lancia `ArgumentCountError: proc_open() expects
at least 3 arguments`, phpr non lancia nulla. `popen` resta ASSENTE
(correct-or-absent: manca la risorsa pipe).

### 3.32 🟡 `Deprecated: Creation of dynamic property` riporta un NUMERO DI RIGA sbagliato (S-173, fixture fx-sl3-div)

Il messaggio è a parità, la riga no: l'oracle cita la riga dell'assegnazione
(`$g->z = 1;` → «on line 5»), phpr cita la riga 1 (o, dentro un file con
funzioni, la riga di una definizione precedente: fx-sl3 → «line 11» al posto
di 92). Sonda minima 2026-09-10 (`class G {public $y=3;} $g=new G; $g->z=1;`
→ oracle line 5, phpr line 1; stessa cosa in un loop `for` e con rhs da
chiamata). Meccanismo indiziato: il diag nasce nella scrittura di proprietà
(sentiero `write_property`/`prop_set_entry`) con la riga corrente del frame
NON aggiornata dal sito dell'assegnazione. PRE-esistente alla leva fetta 3
(pin s172 == stash s171 sulla forma). Presidio: fx-sl3-div (pin==stash).


## 4. Punti di forza da NON toccare (invarianti verificati byte-identici)

Per evitare regressioni, questi comportamenti sono **già** byte-identici con
l'oracle e vanno preservati:

- **`mt_rand` / `mt_srand`**: bit-esatti con il Mt19937 di PHP. Di conseguenza
  ogni builtin RNG-based (`array_rand`, `str_shuffle`, `shuffle`, …) è
  byte-identica **dopo il seed** via `crate::math::mt_range`. Non reimplementare
  l'RNG.
- **`strnatcmp` / `strnatcmp_ex`**: comparazione naturale fedele (riusata da
  `natsort`/`natcasesort`).
- **hashing** (`md5`, `sha1`, `md5_file`, `sha1_file`, `password_hash` bcrypt):
  digest byte-identici.

---

## 5. Come si verifica una divergenza (procedura)

1. **Probe oracle**: eseguire lo stesso snippet con
   `~/Claude/php-oracle/php-src/sapi/cli/php` (o brew `/opt/homebrew/opt/php/bin/php`).
2. **Leggere il C**: fonte esatta in `php-8.5.7/ext/**` (via Read/Vexp, non grep).
3. **Diff byte-per-byte** stdout+stderr phpr vs oracle.
4. **Gate corpus Zend** (`phpt-runner --isolate`) confronto per NOME dei blocchi
   `^---`: zero regressioni obbligatorio.
5. Se non si può essere fedeli → **lasciare assente** e annotare qui.

---

### Changelog di questo documento

Storico congelato: `doc/archive/DIVERGENCES_CHANGELOG_2026-07.md` (07-09 →
07-27) e `doc/archive/DIVERGENCES_CHANGELOG_2026-08.md` (08-03 → 08-13). Da
qui in poi le chiusure si tracciano nei verbali `sessions/`; questo elenco
tiene solo le REVISIONI del catalogo.

- 2026-09-10 (revisione integrale, decisione utente): ogni voce con repro
  ri-eseguita contro l'oracle 8.5.7 al pin s172 (audit in due metà, script
  conservati fuori repo). Rimosse le voci provate byte-identiche (§1.6 non
  riproducibile, §3.15, §3.19-bis, §3.19-ter, parti chiuse di §1.1/§3.7/§3.13/
  §3.8), precisate 13 voci con la misura, aggiunta §3.31, changelog interno
  (1354 righe) archiviato. Numerazione stabile. Vedi il paragrafo «Ultima
  verifica sul campo» in testa.
