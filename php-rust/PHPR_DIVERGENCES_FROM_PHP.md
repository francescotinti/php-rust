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
> Ultimo aggiornamento: 2026-07-22 (Sessione WordPress-38).

---

## 1. Gap trasversali dell'architettura builtin (impattano molti phpt "di tipo")

Questi sei gap nascono tutti dalla stessa radice architetturale: le funzioni
del crate **`php-builtins`** sono pure (`fn(args, ctx) -> Result<Zval, PhpError>`)
e **non hanno accesso allo stato della VM**. Non possono quindi rientrare nel
motore per invocare metodi utente, generare backtrace "veri", o consultare lo
stato ZPP dell'engine. Limitano principalmente i **phpt di edge-case/di tipo**,
non l'uso reale delle webapp target (WP/Composer/Doctrine/Laravel/Symfony).

### 1.1 Coercion di oggetti `Stringable` nelle builtin pure  ✅ IN GRAN PARTE CHIUSO (`6f7cb31`+`f1fee67`)
- **Era il gap più frequente**: una builtin che coerce un argomento a stringa
  non invocava `__toString()` (il crate `php-builtins` è VM-stateless), emettendo
  un warning spurio "Object of class X could not be converted to string".
- **Meccanismo implementato** (rispecchia il precompute `__debugInfo` di
  `var_dump`): `Ctx` ha una mappa `stringify: &HashMap<u32, ZStr>` (id oggetto →
  risultato `__toString` precomputato) + helper `Ctx::to_zstr(&Zval)`. La VM,
  **prima del dispatch** e SOLO per le builtin che coercono *incondizionatamente*
  (gate `ref_builtin_string_coerces` / `value_builtin_string_coerces`), invoca
  `__toString` via `resolve_method_runtime`+`call_method_sync` (`Vm::compute_stringify`,
  no ricorsione negli array annidati, cycle-guard per id). Le builtin chiamano
  `ctx.to_zstr` invece di `convert::to_zstr`.
- **Coperti**: `natsort`/`natcasesort` (by-ref) + ~28 value builtin via
  `string::str_at` (str_contains/starts_with/ends_with, substr_*, add(c)slashes,
  strtr, wordwrap, levenshtein, htmlspecialchars/htmlentities, strip_tags, …).
  Byte-identici all'oracle; introspection (is_string/gettype/get_class/var_dump)
  **esclusi** → nessuna chiamata `__toString` spuria (verificato). Zend 2322→2323.
- **Esteso agli ARRAY-arg** (`a7c0c63`): `implode`/`join` (host `ho_implode`: glue
  ora via `vm_stringify` come gli elementi) + `str_replace`/`str_ireplace`
  (deep gate: `compute_stringify(recurse_arrays=true)`, walk FIFO in ordine).
- **Residuo minimo rimasto**:
  (a) `sprintf`/`printf` `%s` — coercion PER-specifier (`%d` NON chiama `__toString`),
      quindi un precompute eager sarebbe spurio; servirebbe che la builtin guidi la
      coercion (non fattibile senza re-entrancy VM). **Deferito.**
  (b) `str_replace` con **search E replace entrambi array di oggetti** con
      `__toString` a side-effect: l'ORDINE delle chiamate `__toString` diverge
      (mio: tutti i search poi tutti i replace; PHP: interleaved per-coppia). Il
      RISULTATO è byte-identico; diverge solo l'ordine dei side-effect (raro).
  `to_zstr` VM-side (echo/concat) già OK da prima.

### 1.2 Deprecation ZPP `null → parametro non-nullable`
- **Sintomo**: passare `null` a un parametro interno non-nullable (es.
  `strlen(null)`) deve emettere `E_DEPRECATED` da PHP 8.1. phpr **non** emette la
  deprecation.
- **Radice**: la validazione ZPP di phpr non modella la distinzione
  "null implicito coercibile ma deprecato".

### 1.3 `#[SensitiveParameter]` non onorato
- **Sintomo**: i parametri marcati `#[SensitiveParameter]` devono comparire come
  `Object(SensitiveParameterValue)` nei backtrace/messaggi d'errore. phpr mostra
  il valore reale.
- **Impatto**: sicurezza/diagnostica; nessun impatto funzionale.

### 1.4 Validazione ZPP dei callable "upfront"
- **Sintomo**: le funzioni che accettano un `callable` devono validarlo **prima**
  di eseguire il corpo (ZPP). phpr in alcuni casi valida tardi, quindi l'ordine
  e il testo dei `TypeError` divergono in edge-case.

### 1.5 Location di `ArgumentCountError` da callback invocati internamente
- **Sintomo**: quando una builtin invoca un callback utente con troppi pochi
  argomenti, il file/linea riportati nell'`ArgumentCountError` non coincidono
  con quelli dell'oracle (che punta al sito inline del callback).

### 1.6 Preservazione degli elementi **by-ref** nei risultati array
- **Sintomo**: alcune funzioni array che in PHP restituiscono/mantengono
  riferimenti agli elementi (es. propagazione di reference dentro array
  risultato) in phpr producono copie. Diverge solo negli scenari con reference
  espliciti negli elementi.

---

## 2. Assenze consapevoli (funzioni non implementate per mancanza di infrastruttura)

Non sono bug: sono funzioni **volutamente assenti** perché un'implementazione
fedele richiede stato/infra non ancora presente, e uno stub violerebbe
correct-or-absent.

| Funzione | Perché è assente / diverge | Cosa servirebbe |
|---|---|---|
| `get_defined_constants` | `resolve_constant` è un `match` non enumerabile: non esiste un registro iterabile delle costanti host | Registro costanti iterabile (host + estensioni) |
| `parse_ini_file` / `parse_ini_string` | Il parser INI di PHP è un lexer flex con semantica di coercizione (NORMAL/TYPED) ed edge-case; difficile essere byte-identici | Port fedele del lexer INI + tabella coercizioni |
| `get_include_path` / `set_include_path` | `include_path` come stato scope-aware non è modellato | Stato `include_path` nella VM + interazione con lo stream wrapper |
| `preg_last_error` / `preg_last_error_msg` / `preg_filter` | Nessuno stato d'errore PCRE globale esposto dal motore regex | Ponte allo stato d'errore dell'engine PCRE |
| `get_cfg_var` / `ini_get_all` | Nessun registro INI reale (in CLI la maggior parte darebbe `false`) | Tabella INI runtime |
| `timezone_identifiers_list` | ~348 nomi statici legati alla versione tzdata; rischio di divergere dalla tzdata dell'oracle | Dataset tz allineato alla build oracle |
| `getimagesize` (formati rari + out-param) | Implementati GIF/JPEG/PNG/BMP/WebP; mancano formati rari e il parametro `&$image_info` | Parser per formati residui + supporto out-param |
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
  cifre): phpr calcola invece di lanciare il `ValueError` "exponent is too large";
  irrilevante nella pratica.

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
  PHP → T_STRING, mago → keyword. Gestiti `->`/`?->` e (sotto TOKEN_PARSE) `::`/`const`.
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
- FILEINFO_EXTENSION: mappa minima (jpeg verificata oracle; il resto "???").
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
| `UCS-4` / `UCS-4LE` / `UCS-2` … | non presenti in `resolve_encoding` → `ValueError "must be a valid encoding"` | `mb_decode_numericentity` test (linea 54) |
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
`LoweredTrait`. Osservato anche (da verificare, pre-esistente):
`class_implements(enum)` non include l'interfaccia implementata esplicitamente
(solo UnitEnum/BackedEnum).

### 3.4 `$this` nello scope-bridge delle classi anonime differite
Gli argomenti del costruttore di una `new class(...)` differita rieseguono nello
scope del chiamante via bridge per-nome dei named slots; `$this` non è un named
slot, quindi `new class($this->x) extends Irrisolvibile {}` dentro un metodo
non vede `$this` alla ri-esecuzione. Caso non osservato (i test Symfony usano
solo locals); da chiudere se emerge.

### 3.5 INI table parziale (filone ext/session, 2026-07-12)
La tabella INI (`vm/ini.rs`) registra solo le direttive modellate: 31 `session.*`
(+ `session.trans_sid_tags`/`hosts`, esenti dal freeze headers-sent e dal listing
`ini_get_all('session')`, oddity oracle-verificata), `include_path` e le ~9
chiavi engine-hardwired storiche. Divergenze deliberate:
- `ini_get_all(null)` elenca ~45 direttive, non le ~291 di PHP; un'estensione
  diversa da `session` → warning "cannot be found" anche per estensioni che PHP
  conosce (`Core`, `standard`, …).
- `memory_limit` resta `-1` (PHP brew riporta `128M`): phpr non applica limiti e
  questo evita il re-exec di Composer.
- Le chiavi hardwired (`precision`, `memory_limit`, …) rifiutano `ini_set`
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
  non è tracciato) e non supporta i riferimenti condivisi `r:`/`R:` né il
  C:-format con ref interni (bug79031).
- `var_dump($_SESSION)` non mostra `&` sugli elementi referenziati (006/019/026).
- `open_basedir` non modellata (gh13856); ReflectionFunction sulle funzioni
  interne non costruisce descriptor (bug74541).
- Il flusso `phpr -d`: gli override si applicano SOLO alle direttive registrate
  (identico all'invisibilità di `php -d unknown=x` a `ini_get`).

### 3.7 Gap scovati dalla probe string WP-38 — ✅ CHIUSI in WP-39 (2026-07-22)

Scoperti da `wp38-harness/probe_wp38.php`; chiusi/riclassificati in WP-39
(probe oracle-pinned `wp39-harness/probe_wp39.php` + `probe_offset_edge.php`
+ `probe_edge2/3.php`):

- **`$flags` dei value-sort** ✅ — `flag_value_sort` (gemello di
  `key_flag_cmp`) cablato in sort/rsort/asort/arsort: SORT_NUMERIC (via
  to_double), SORT_STRING/SORT_LOCALE_STRING (byte, ±SORT_FLAG_CASE folded),
  SORT_NATURAL (strnatcmp ±case). Aggiunto anche l'arm SORT_NATURAL mancante
  in `key_flag_cmp` (ksort/krsort). **Residuo dichiarato**: i valori
  Stringable sotto SORT_STRING/NATURAL prendono il fallback-warning
  "could not be converted to string" + placeholder invece di `__toString`
  (sort non può stare nel gate statico §1.1: i `$flags` sono runtime, il
  precompute chiamerebbe `__toString` spurio sotto SORT_REGULAR); la coercion
  è 1× per elemento up-front (convenzione natsort), non per-confronto come
  Zend (conteggio chiamate `__toString` non riprodotto).
- **Diagnostica string-offset READ** ✅ — `read_dim_warn` ora oracle-pinned:
  "Uninitialized string offset N" (N = offset castato PRE-aggiustamento
  negativo), "String offset cast occurred" per chiavi float/bool/null (SENZA
  la deprecation float-precision su questo path), "Illegal string offset
  \"5abc\"" + uso del prefisso intero per stringhe leading-numeric (prima era
  erroneamente TypeError), "Resource ID#n used as offset…". isset/`??`/empty
  restano silenti come Zend. **Residui dichiarati** (funnel separati senza
  diags): `isset($s[1.5])` non emette la deprecation "Implicit conversion
  from float 1.5 to int loses precision"; `$s["5abc"] ?? $d` in Zend emette
  Illegal-warning e LEGGE il prefisso, phpr prende silenziosamente il default.
- **Deprecation increment/decrement** — RICLASSIFICATO: **già a parità**
  (tutti e tre i messaggi 8.3+, incluso `dec('')` → int(-1) con messaggio
  dedicato). Il "mancante" di WP-38 era un artefatto di lettura: mancava solo
  la copia log "PHP Deprecated:" dell'oracle (log_errors=On nel php.ini brew;
  i confronti probe vanno fatti con `-d log_errors=0`).

---

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
- 2026-07-22 (sessione WordPress-39): ✅ CHIUSI i tre gap §3.7 (probe WP-38):
  `$flags` dei value-sort (`flag_value_sort` + arm SORT_NATURAL in
  `key_flag_cmp`), diagnostica string-offset READ oracle-pinned
  (Uninitialized/cast occurred/Illegal + prefisso, prima TypeError errato su
  "5abc"), incdec deprecation RICLASSIFICATA già-a-parità (artefatto
  log_errors nella probe WP-38 — confrontare sempre con `-d log_errors=0`).
  Residui dichiarati in §3.7. Corpus 1455→1447 (−8, 0 nuovi).
- 2026-07-20 (sessione WordPress-25): ✅ CHIUSO il gap (non catalogato)
  dell'**asymmetric visibility 8.4 in scrittura**: phpr non negava affatto
  `$o->p = v` / `unset` / compound / `++` su `private(set)`/`protected(set)`
  da scope escluso. Ora `asym_write_error` (oop.rs) nega nei 4 op-site, con
  `protected(set)` scopato sul PROTOTYPE come i metodi protetti (gh19044-6)
  e `__set`/`__unset` che continuano a passare da `magic_applies` PRIMA
  della deny su slot esplicitamente-unset (ordine Zend). Corpus asym 19→10.
  RESIDUI dichiarati: deny nel **field-path** (`$o->arr[] = v`, nomi
  dinamici — dim_add/variation*/reference*), promozione costruttore
  `private(set)` non modellata (cpp_*), check compile-time "Property with
  asymmetric visibility C::$p must have type" assente (P7 non fatala),
  ast_printing/readonly.phpt. ⚠️ Divergenza NUOVA e deliberata (robustezza
  > parità): il Drop di grafi oggetto è a stack limitato (`drop_bounded`,
  trampolino oltre 512 livelli) — l'ordine LIFO di riuso id resta esatto
  fino a 512 livelli e diventa approssimato oltre (inosservabile nei test;
  l'oracle regge 1M di profondità dove phpr segfaultava a ~45k, e sugli
  ARRAY puri annidati profondi l'oracle stesso segfaulta ~500k mentre phpr
  ora regge se la catena passa da oggetti/closure).
- 2026-07-19 (sessione WordPress-24): ✅ CHIUSE le divergenze xsl/dom residue
  attaccabili (phpt xsl 44→57/64): (a) transcodifica iso-8859-1/windows-1252/
  iso-8859-15→UTF-8 nel load XML (la dichiarazione è onorata) + RI-CODIFICA
  di saveXML whole-doc verso l'encoding dichiarato (char non rappresentabili
  → `&#N;` come libxml) — probe saveXML byte-id con l'oracle (xslt004/007);
  (b) `file://` strippato in DOMDocument::load/loadHTMLFile con semantica
  libxml (host vuoto/localhost; `file://./…` FALLISCE come l'oracle);
  (c) `documentURI` CANONICA (xmlPathToURI via FFI: file:// locale ridotto
  al path, byte URI-invalidi %-escaped) → bug53965 byte-id anche con SPAZI
  nel path (la base canonica fa risolvere xsl:include relativi); (d) un
  DOMDocument mai caricato entra nel transform come xmlNewDoc (bug71571) e
  il report di ricorsione libxslt (una chiamata, 2 righe) è ricucito in UN
  warning; (e) unset di una hooked prop dichiarata nel PRELUDE → shape
  nativa "Cannot unset C::$p" senza "hooked property" (le hook userland
  restano "hooked property", verificato oracle anche su sottoclassi);
  (f) trampolino php:function: zero argomenti → Error "Function name must
  be passed as the first argument"; ritorno oggetto non-DOM → TypeError
  parcheggiata (era Warning); autoloader che LANCIA in risoluzione → Error
  "Invalid callback …, class … not found" con l'eccezione chained come
  previous (shim prelude `__xsl_call`: is_callable innesca l'autoload, la
  chiamata resta diretta così le eccezioni del body passano intatte);
  registerPHPFunctions con le validazioni Zend 8.4 complete (callable-check
  con reason, nomi vuoti/NUL → ValueError, `get_debug_type` nel messaggio
  ZPP); hasExsltSupport → ArgumentCountError su argomenti extra.
  ✅ ENGINE trasversali: `set_error_handler(null, …)` = handler di default
  con lo stack preservato (prima lanciava "Value of type null is not
  callable"); il cast ESPLICITO `(string)`/interpolazione (Op::Stringify) di
  Closure/Generator ora LANCIA Error come Zend (il funnel infallibile di
  `echo` resta D-19.18, perimetro RIDOTTO); Drop di `Object` POSTORDER —
  la free-list riceve i FIGLI prima del padre, riuso LIFO = id del padre,
  come zend_objects_store_del — e `next_id` ora ripulisce TUTTE le tabelle
  per-id (fibers/generators/marks GC/cache transienti: un Fiber morto non
  "è già stato avviato" sul riuso dell'id, fibers/destructors_002). Corpus
  1487→1485 (bug60738 e closures/closure_015 chiusi). tidy 010 AVANZATO
  (la prima sezione passa) ma NON chiuso: per i grafi trattenuti in
  gc_roots l'ordine di rilascio dello sweep resta di nota, non di cascata.
  Residui xsl (7, tutti catalogati): bug49634 (frame prelude nei trace),
  bug69168 (identity/liveness dei nodi — divergenza DICHIARATA WP-23),
  registerPHPFunctionNS (serve functionURI dal contesto XPath via FFI),
  xsl-phpinfo, xslt008/-mb/009 (stream wrapper PHP dentro l'I/O di libxml).
- 2026-07-19 (sessione WordPress-23): 🟣 **ext/tidy NATIVA su libtidy 5.8.0
  di SISTEMA via FFI** (php_types::tidyio + vm/tidy.rs + prelude/tidy.php,
  pattern gd/xslt: stessa keg Homebrew dell'oracle → output e diagnostica
  byte-identici; phpt 44/45 runnable). Divergenze ext/tidy: (1) l'INI
  `tidy.clean_output=1` a runtime NON avvia l'output-handler automatico
  (l'entry è settable e riflette il valore; `ob_start("ob_tidyhandler")`
  esplicito È implementato); (2) default `tidy.clean_output`="0" (default di
  sorgente; il php.ini brew lo normalizza a "" via scanner Off→empty — phpr
  non legge php.ini); (3) 010.phpt: object-id REUSE order su teardown
  ricorsivo (Zend libera i figli PRIMA del padre → free-list LIFO riusa l'id
  del padre; phpr rilascia in pre-ordine → riusa l'id dell'ultimo figlio) —
  gap engine cosmetico trasversale, non tidy-specifico. tidy/tidyNode
  numeric-cast → 0 (cast handler) cablato in Op::Cast come SimpleXML.
  🟣 **ext/xsl: registerPHPFunctions/php:function TRAMPOLINO reale**
  (xsltRegisterExtFunction + valuePop/Push via FFI; re-entry VM col pattern
  ACTIVE_VM di pdo.rs; suite phpt 21→44/64 runnable). Divergenze xsl:
  (a) nodeset passati al callback e nodi RESTITUITI sono COPIE
  content-preserving (serializza→re-parse in doc temporanei con vita =
  transform), niente identità/liveness col documento d'origine — i passi
  XPath sul risultato (`php:function(...)/i`) funzionano sulla copia;
  (b) residui NON implementati: transcodifica iso-8859-1 nel DOM load
  (xslt004/007/008), DOMDocument::load con path relativi in sottodir
  (bug53965), document() su stream wrapper (xslt009), shape di
  warning/errore minori (bug69168, callables_errors, edge cases),
  byref/unset su hooked prop (maxTemplate*_bypass), xsl-phpinfo.
  🔧 **FIX ENGINE hook-guard**: un property-hook (o magic) che LANCIA non
  rilasciava il guard di ricorsione (solo Ret lo faceva) → ogni accesso
  successivo alla proprietà bypassava l'hook in silenzio. L'unwinder ora
  rilascia i guard_release dei frame morti. 🔧 DOMAttr::$value ora VIRTUALE
  (__get/__set live sul documento): prima una scrittura su un attr da
  DOMXPath finiva solo nel wrapper (xslt002: mutare xsl:output/@method non
  aveva effetto). ⚡ Props hash-first (indice u64 FxHash oltre 8 entry) e
  PhpArray::holds_containers (gc_note/gc_classify saltano gli array
  scalar-only) — pure ottimizzazioni, nessuna divergenza osservabile.
- 2026-07-19 (sessione WordPress-21): ⚡ **perf GC** (nessuna divergenza nuova;
  un fix). (1) Il collettore di cicli automatico ora ADATTA la soglia come
  Zend `gc_adjust_threshold`: una collezione che libera <100 valori (i root
  bufferizzati erano dati vivi, es. fixture di una test-suite) alza il trigger
  di uno step (50k, cap 1e9); una efficace lo riabbassa verso la base. Il
  timing della collezione AUTOMATICA non è osservabile a parità (Zend stesso
  la adatta dinamicamente); `gc_collect_cycles()` esplicito resta immediato.
  (2) ⚠️ TENTATA e REVERTITA la classificazione a note-time in `gc_note`
  (count>2 → dritto nei cycle-root): perdeva la rete di sicurezza dello
  sweep di fine statement sui drop UNHOOKED avvenuti dopo la nota nello
  stesso statement — un cleanup in `__destruct` (unlink di file) slittava
  oltre i test dipendenti (flake REST sideload unique-filename in run12).
  Il buffer candidati resta sempre-alimentato come in WP-20.
  (3) 🔧 **FIX della divergenza catalogata in
  WP-20**: il re-`require` di un file con funzione non-condizionale ora
  fatala "Cannot redeclare function f() (previously declared in file:line)"
  — check in `run_linked` contro `linked_functions` e il modulo corrente
  (prelude esente via marker `file == "prelude"`). Reso con la NUOVA variante
  `PhpError::FatalAt` (E_ERROR non-throwable): banner piano "Fatal error:
  {msg} in {file} on line {line}" SENZA "Uncaught", né trace, UNCATCHABLE e
  senza `finally` (come Exit), posizionato alla NUOVA dichiarazione come
  `zend_bind_function_error` — il phpt `line_numbers/gh16509.phpt` passa
  (prima SKIPPAVA come compile-error: il secondo include produceva un bogus
  "Failed to compile"). Divergenze RESIDUE: (a) niente backtrace del fatal
  (PHP 8.5 `fatal_error_backtraces=On` in CLI aggiunge "Stack trace:\n#0
  {main}"; sotto run-tests è spento, quindi i phpt combaciano); (b) per eval
  il file della dichiarazione precedente è "eval()'d code:N" senza il path
  del file ospite; (c) il path CONDIZIONALE (`Op::DeclareFn`) resta in forma
  "Uncaught Error" senza "previously declared" (pre-esistente, invariato).
- 2026-07-18 (sessione WordPress-20): ⚡ **perf memoria del macchinario
  include** (nessuna divergenza nuova; due note). (1) Le funzioni del PRELUDE
  compilate ora sono UNA copia condivisa (`Rc`) dal modulo main in ogni unit
  include/eval — prima ogni include ricompilava e leakkava ~1070 funzioni
  (~1,5MB/file: bootstrap WP test-suite 5,3GB → ~2,9GB di picco; 300 include
  sintetici 484MB → 56MB). Effetto semantico: le `static $x` delle funzioni
  prelude sono ora CONDIVISE tra main e unit (prima ogni unit aveva celle
  proprie) — è la semantica Zend (una funzione = un set di static), quindi
  un AVVICINAMENTO all'oracle, non una divergenza. Stessa condivisione per
  gli stub di classe seed (internati per nome) e la unit-cache ritiene un
  `SeedDelta` (classi/slot/trait nuovi) al posto dell'INTERO `Rc<Program>`
  HIR. Gate20 integrale per nome su baseline WP-18.
  🐛 **Divergenza PRE-esistente scoperta (non introdotta, non fixata)**:
  ri-`require` dello stesso file che dichiara una funzione non-condizionale
  NON produce il fatal Zend "Cannot redeclare function f() (previously
  declared …)" — `run_linked` registra le funzioni con `or_insert` senza
  check di ridichiarazione (e il probe della unit-cache non c'entra: accade
  anche a cache fredda). Repro: `one.php` con `function f0(){}`, main con
  `require` in loop ×2 — oracle fatala, phpr prosegue. Innocuo per WP
  (require_once ovunque), da fixare con un check in `run_linked` quando la
  fn è già presente in `linked_functions`/modulo corrente e non-condizionale.
  FFI** (`php_types::xsltio` → `/usr/lib/libxslt.1.dylib` + libexslt + libxml2,
  le STESSE dylib che linka l'oracle brew ⇒ output byte-identico; probe
  10 sezioni a diff zero). Classe `XSLTProcessor` nel prelude dom
  (importStylesheet/transformToXml/transformToDoc/transformToUri/set-get-
  removeParameter clark-notation/hasExsltSupport/set-getSecurityPrefs,
  proprietà tipizzate doXInclude/cloneDocument/maxTemplateDepth/maxTemplateVars
  con default 3000/15000 dai globals libxslt), costanti XSL_*/LIBXSLT_*/
  LIBEXSLT_*, "xsl" in extension_loaded; EXSLT registrato (exsltRegisterAll,
  hasExsltSupport=true come oracle). L'interscambio è per DOCUMENTO
  SERIALIZZATO (saveXML → xmlReadMemory): il transform dipende dall'infoset,
  non dalla forma serializzata. Errori libxslt catturati via open_memstream
  come contesto del generic-error handler di default (le riscritture
  "$maxTemplateDepth"/"$maxTemplateVars" di php_xsl applicate alle righe) →
  Warning al call-site come l'oracle. Sblocca i 3 test sitemaps
  (assertXMLEquals/normalizeXML di WP_Test_XML_TestCase); gruppo sitemaps 132
  test IDENTICO per nome. **Divergenze dichiarate ext/xsl**: (a) nei messaggi
  di errore di compilazione su stylesheet MALFORMATI, l'URL è %-escaped e il
  numero di riga può differire di +1 (la ri-serializzazione aggiunge la
  dichiarazione XML) — solo diagnostica; (b) registerPHPFunctions/
  registerPHPFunctionNS/setProfiling ASSENTI (richiederebbero callback
  XPath→VM dentro libxslt; undefined-method onesto); (c) `clone $proc` non
  vietato (oracle: uncloneable) e le prop private `__h`/`__params` visibili a
  var_dump; (d) doXInclude ignorato (XInclude fuori slice).
- 2026-07-18 (sessione WordPress-18): 🏁 **chiusura dei 56 diff per nome di
  run8** (tutti i cluster + singleton attaccabili). Fix engine: ⭐ `(object)$x`
  IDENTITARIO su Closure/Generator (WP `_wp_filter_build_unique_id` usa
  `spl_object_id((object)$callback)` — il wrapper nuovo rompeva remove_filter:
  Tests_Theme 6F) · ⭐ **branch reset group `(?|…)`** emulato con riscrittura
  `(?:…)` + rinumerazione e `Engine::Remap` fisico→logico (Duotone 6F) ·
  ⭐ **goto in blocchi trasparenti** (if/try/catch/`{}`) ora salta davvero sul
  bytecode piatto; loop/switch restano il fatal di lowering, finally il suo
  (html-api 5E) · ⭐ string-offset: chiave stringa non-integrale → isset false/
  empty true (WP style-engine) e read = TypeError "Cannot access offset of
  type string on string" (themeJson 2F) · `static::${$n}` come class-value
  runtime via Op::ClassNameStatic (themeJson 2E) · ⭐ **flush dei diag di
  FetchDim al punto di raise** (un handler che LANCIA deve svolgere dallo
  statement colpevole — PHPUnit expectWarning, Tests_Locale) · __unset magic
  su path multi-step (`unset($this->a->b)`, WpListTable/TextDiffRenderer) ·
  strtotime: **orario nelle espressioni relative** ("next Sunday 1pm",
  "yesterday 08:15", meridian spaziato) + **"next|last|this week" = lunedì
  della settimana** (tempo conservato) (wpCommunityEvents 5, DateI18n) ·
  DateTime("−06:00") = campi "now" + zona offset · 'B' (Swatch) zone-independent
  in DateTime::format · mb_strlen su input invalido = **maximal subpart**
  WHATWG per-range (wpUtf8CodePointCount 4F) · json_decode: `$associative=null`
  → decide JSON_OBJECT_AS_ARRAY; INVALID_UTF8_SUBSTITUTE/IGNORE scrub per-byte
  (BlockProcessor 5F) · regex: **escape ottali nelle classi** `[\200-\377]` →
  `\x{HH}` (PHPMailer 8-bit, wp_mail) · htmlspecialchars: doctype XML1/XHTML/
  HTML5 → `&apos;` (esc_xml) · copy(stesso file) = false · zend_version() ·
  ZipArchive::addEmptyDir · iconv_mime_decode(_headers) RFC 2047 ·
  ext/xml: eventi **start_ns** (prefix default = false, mai end_ns) e default
  handler per COMMENTI interi; testo del prologo mai consegnato · **argon2i/
  argon2id** in password_* (crate argon2, PHC) + costanti PASSWORD_ARGON2* ·
  **ext/intl subset**: normalizer_normalize/is_normalized (unicode-
  normalization) + classe Normalizer + "intl" in extension_loaded (RemoveAccents
  NFD). Divergenze residue DICHIARATE: sitemaps 3 (serve XSLTProcessor —
  candidato libxslt FFI in WP-19), wpIsIniValueChangeable #4 (serve ext/Tidy),
  ftp wrapper (deciso), stderr "PHP Xxx:" CLI mai emesso (nota storica).
- 2026-07-18 (sessione WordPress-17): 🏁 **chiusura di massa dei cluster della
  full-suite**: tutti i cluster maggiori del triage WP-16 sono a parità per
  nome (Template 86 OK · php-ai-client 255 OK · Privacy export 31 OK · Kses
  358 OK · wpTexturize 357 OK · BackgroundSupport 27 OK · GetBookmark 46 OK ·
  DB_Charset 100 OK · Translation 36 OK · ExportWp 11 OK · WpEmailAddress 79
  OK · hooks order e includesPost a parità W-per-W · sitemaps a parità).
  Fix engine/builtin: **INI** default_mimetype/default_charset/
  error_prepend_string/error_append_string + `error_reporting` come direttiva
  (sync bidirezionale con EG(error_reporting) — ini_set/error_reporting()
  vedono lo stesso mask); **diagnostica**: i diagnostici vanno nel FILE
  `error_log` (log_errors, stamp php_log_err) sotto ogni SAPI; il render
  display avvolge con error_prepend/append_string; file/line passati
  all'error handler utente = nearest-non-prelude frame (prima: il main
  script — sotto PHPUnit arrivava `vendor/bin/phpunit`) con nome composito
  `file(N) : eval()'d code` per le unit eval; **ob_start flags**
  (CLEANABLE|FLUSHABLE|REMOVABLE): gating per-op con le notice esatte di
  main/output.c ("Failed to flush/delete/send/discard buffer of %s (%d)"),
  ob_get_clean/get_flush ritornano comunque il contenuto su non-removable,
  status['flags'] = richiesti|USER; **`$a[]` come argomento** (ArgPlace
  Append: by-ref appende+alias, by-value = Error runtime "Cannot use [] for
  reading") — class-pclzip.php ora compila; **unset($GLOBALS['k'])** rispettato
  dallo snapshot $GLOBALS (il check Undef leggeva il valore già mappato a
  null); **LSB nelle closure** (campo `lsb` catturato alla creazione,
  `static::` dentro `static function` creato via `Child::m()` risolve Child;
  bindTo/bind lo spostano); **preg**: backref `\g1`/`\g{n}`/`\g{-n}`/`\g{name}`
  normalizzati, condizionali con lookbehind `(?(?<=…)sì|no)` riscritti come i
  lookahead, classi `\d\s\w` (e negazioni) in forma ASCII per i pattern
  non-`/u` (PCRE byte-mode; `\b` resta Unicode — divergenza pre-esistente
  documentata), `\xNN`≥80 → `\x{NN}` nel path oniguruma (dominio char della
  vista Latin1, non byte UTF-8); **mysqli byte-safe**: query/prepare passano
  i byte grezzi sul wire (il round-trip lossy UTF-8 corrompeva i payload
  charset-nativi di wpdb::strip_invalid_text con U+FFFD); **idn_to_ascii/
  idn_to_utf8 NATIVE** (punycode RFC 3492 + mapping UTS46-lite lowercase;
  `$idna_info` out-param non popolato — divergenza doc.) + costanti IDNA_*;
  **json**: json_encode onora `$depth` (JSON_ERROR_DEPTH), il fallimento con
  resource nel tree è JSON_ERROR_UNSUPPORTED_TYPE ("Type is not supported"),
  json_decode(null) depreca; **array_unique flag-aware** (SORT_REGULAR =
  confronto loose SENZA stringificazione — gli oggetti confrontano
  property-wise: ModalityEnum/WP_User; SORT_NUMERIC sul double) e
  **ksort/krsort flag-aware** (SORT_NUMERIC: 'test'→0 prima di 10, tie
  stabili — WP_Hook); **strlen/strcmp-family** onorano il precompute
  Stringable; **deprecation float-key** "Implicit conversion from float %s to
  int loses precision" su write/read/path-write/array-literal (isset/unset
  e il line-number nel punto esatto restano divergenze minori); **null-arg
  deprecations** estese (tabella nomi-parametro in str_at + strpos/substr/
  str_replace/json_decode); **tempnam(""/false)** → temp dir di sistema
  (WP temp_filename passa realpath('TMPDIR')=false); **ZipArchive in
  SCRITTURA** (zip::ZipWriter: open CREATE/OVERWRITE su file nuovo, addFile/
  addFromString deflate, close finalizza — privacy export); **SimpleXML**:
  `xpath()` (delegato all'engine __dom_xpath) e cast `(int)`/`(float)` via
  testo del nodo (handler cast_object di SimpleXML; l'oggetto generico resta
  1); **XPath**: node test `processing-instruction("target")` e funzione
  `namespace-uri()`; **lexer**: escape esadecimale `\X41` MAIUSCOLO decodifica
  come `\x41` (Zend accetta entrambi; il provider kses usa "\X1C");
  **date**: `d-m-Y`/`m/d/Y` col 4-digit finale, anno a 2 cifre rimappato
  00-69→2000/70-99→1900, time-only `H:i[:s]` = oggi a quell'ora
  (mysql_to_rfc3339); **mb_detect_order**, **get_defined_functions**
  (internal = registry+host tables ordinati; prelude→internal). Residui
  rimandati a WP-18: theme 6F (cache/filtri get_stylesheet_directory),
  duotone 6F (branch reset `(?|…)` non supportato dai backend regex),
  wp_is_stream('ftp://') 1F (stream_get_wrappers resta onesto: niente ftp
  senza supporto reale — correct-or-absent), goto-into-block html-api 5E
  (D-45.1), display-in-ob-handler ("Producing output from user output
  handler" con discard) e stderr "PHP Xxx:" del CLI SAPI (mai emesso).
- 2026-07-17 (sessione WordPress-16): 🏁 **la WP core suite INTERA (default
  testsuite, 30.480 test / 4,55M assertion) arriva IN FONDO su phpr per la
  prima volta**: da "muore silenziosamente" a 123E/199F/15S-diff (336 test
  differenti per nome vs oracle 30.321P/86W/73S/1F; -1 test = dataset tidy,
  ambientale). Cinque blocker seriali abbattuti: **(1) mysqli senza
  `__destruct`** — connessioni leakate (una per classe di test via
  `wpdb::db_connect()`) tenevano transazioni aperte e metadata lock che
  DEADLOCKAVANO l'install del bootstrap nei figli `@runInSeparateProcess`
  (catena a 3 processi: padre in stream_get_contents ← figlio isolato ←
  install.php bloccato su MySQL); ora il prelude chiude alla caduta
  dell'ultimo riferimento come Zend. **(2) RSS fino al jetsam-kill**: mimalloc
  (WP-7) trattiene le pagine liberate — `MIMALLOC_PURGE_DELAY=0` nell'harness
  full-suite le restituisce all'OS (2.3GB→0.5-1GB steady). **(3) panic Rust
  "not a total order"**: il confronto loose di PHP non è un ordine totale sui
  tipi misti e la std panica; `sort/rsort/asort family/array_multisort` ora
  usano un merge sort STABILE tollerante (`php_types::ops::stable_sort_by`).
  ⚠️ divergenza: su array misti con comparatore INCOERENTE la permutazione
  può differire da zend_sort (i tie e i comparatori coerenti sono identici).
  **(4) foreach by-ref su proprietà statiche** (`foreach (self::$ids as &$id)`
  e stepped `self::$m['p']`) iterava una COPIA: ora alias dello storage vivo
  via StaticPropRef(+MakeRef); nome dinamico `C::${$n}` resta by-value
  (documentato). **(5) cluster full-suite**: `mb_substitute_character` (stato
  onorato in decode/encode: none/long/entity/codepoint, probe byte-id — long/
  entity valgono solo lato encode, decode-bad → `?`; granularità malformed
  UTF-8 = WHATWG maximal-subpart) + `mb_scrub` · BIG-5 NATIVO (port verbatim
  di unicode_table_big5.h + semantica mb_big5_to_wchar in `php_types::big5`,
  24k celle decode + BMP encode BYTE-ID; encoding_rs WHATWG divergeva su 260
  celle simbolo, lead HKSCS e sostituto) · `JSON_UNESCAPED_LINE_TERMINATORS`
  (+ U+2028/29 escapati sotto UNESCAPED_UNICODE senza il flag, json.c 7.1+) ·
  `preg_last_error`/`_msg` + costanti PREG_* (0/1/4; phpr non ha backtrack
  limit → 2/3/6 mai prodotti, divergenza documentata) · `chown`/`chgrp`/
  `lchown`/`lchgrp` su libc (nome→uid/gid via getpwnam/getgrnam, warning
  oracle-pinned) · ini: access PERDIR|SYSTEM su 6 direttive, `upload_tmp_dir`
  e `open_basedir` con semantica NULL (e ini_set di open_basedir aggiorna
  anche global_value); open_basedir NON restringe le operazioni file di phpr
  (documentato). Residuo full-suite per WP-17: Template 38 · php-ai-client 28
  · Privacy export 27 (PclZip) · Kses 21 · BackgroundSupport 20 · GetBookmark
  19 · DB_Charset 16 · Translation 12 · goto-into-block html-api (D-45.1) ·
  WPDieException 26 · Hooks order 8+8.
- 2026-07-17 (sessione WordPress-15): 🏁 **gruppi WP taxonomy/comment/xmlrpc/
  multisite a parità oracle** (878/582/316/32 test; multisite lo era già al
  primo colpo). Sette lavori: **(1) bug static-prelude CHIUSO** — il prelude
  lowera con un contatore `static` proprio (da 0) e il main unit ripartiva
  anch'esso da 0: collisione di id / overflow di `Vm::statics` (il panic
  index-out-of-bounds run.rs:233 di WP-14); ora `LoweredPrelude` esporta il suo
  `static_count` e ogni unità (main e seeded, con `max`) semina il contatore
  oltre il range del prelude; `xml_error_string` è tornato `static`.
  **(2) `gethostbyaddr`** — FFI `getnameinfo(NI_NAMEREQD)` sulla libc di
  sistema (`php_types::netio`), ordine inet_pton v6→v4 come dns.c: malformed →
  warning+false, senza PTR → ip invariato, altrimenti hostname (stesso
  resolver dell'oracle ⇒ stessi risultati sulla stessa macchina).
  **(3) deprecation PHP 8.1 "Passing null to parameter #N ($p) of type T is
  deprecated"** sul trim family (`trim`/`ltrim`/`rtrim`, parametri #1/#2) via
  helper riusabile `null_arg_deprecation` — era il fail di
  Tests_WP_Generate_Tag_Cloud (ltrim(null) nei filtri). Le ALTRE funzioni
  interne NON emettono ancora la deprecation: aggiungerla per-funzione quando
  emerge dai test. **(4) `DOMNode::C14N`/`C14NFile`** (C14N 1.0 + exclusive,
  probe oracle 12 casi byte-id: attr sortati per (ns-uri, localname), ns decls
  prima ordinate per prefisso, rendering inclusive = tutto l'in-scope non già
  reso / exclusive = solo visibly-utilized, escaping C14N, PI/commenti
  doc-level con `\n`, nodi testo/attr, empty element espansi) + fix parser XML:
  normalizzazione fine-riga §2.11 (`\r\n`/`\r`→`\n`) sull'INPUT come libxml.
  **(5) `DOMNode::normalize` / `DOMDocument::normalizeDocument`** (merge Text
  adiacenti, drop Text vuoti, CDATA barriera — PHPUnit DOMNodeComparator li
  chiama in assertXmlStringEqualsXmlString). **(6) strtotime "assoluto poi
  relativo"** (`"2026-07-17 14:30:00+10 minutes"`, anche senza spazio — il
  comment-preview di WP): fallback che prova il prefisso assoluto più lungo il
  cui resto è un'espressione relativa valida; cambiamento monotono (solo input
  prima `false`). **(7) write-chain su proprietà overloaded/assenti/negate
  (`AaOp::MagicDescend`)** — chiude il gap documentato in `prop_key_read`
  (Bug #34893): uno step INTERMEDIO di scrittura con slot raw assente o
  inaccessibile ora passa dal VM: `__get` guarded e prosecuzione sull'oggetto
  ritornato; risultato non-oggetto → Notice "Indirect modification of
  overloaded property C::$p has no effect" + Error "Attempt to assign property
  on <tipo>" se lo step successivo è una proprietà (silente se è un indice);
  senza `__get`: Denied → "Cannot access {vis} property", assente + step
  proprietà → niente autoviv (PHP 8.5) con "Attempt to assign property on
  null" (e Deprecated dynamic-property sulle classi non-dynamic), assente +
  step indice → autoviv array e prosegue. Oggetti lazy: percorso legacy
  invariato. Probe a 3 file tutti byte-id.
- 2026-07-17 (sessione WordPress-14): 🏁 **RESTAPI A PARITÀ ORACLE: i 19E
  residui → 1E, identico all'oracle** (oEmbed
  `test_proxy_with_classic_embed_provider`, bug upstream del trunk wpdev:
  stesso messaggio "Attempt to read property \"queue\" on null" e stessa riga
  su ENTRAMBI). Quattro feature nuove: **(1) mbstring `HTML-ENTITIES`**
  (porta fedele di `mbfilter_htmlent.c`: tabella HTML4 252 entity condivisa in
  `php_types::html4`, decode byte-wise Latin-1 con wrapping u32 e bad-entity
  pass-through, encode <0x80 literal + named/`&#N;`; **deprecation con cache
  `last_used_encoding`** à la `php_mb_get_encoding` — warn solo al miss, il
  lato FROM di mb_convert_encoding non warna mai; `mb_check_encoding` allargato
  a tutti gli encoding risolvibili via `validates`; `mb_list_encodings` =
  lista oracle filtrata sui nomi che phpr risolve; iconv RIFIUTA
  HTML-ENTITIES come l'oracle. Divergenze note: decode di `&#xD800;` → `?`
  invece dei byte WTF-8 ED A0 80 — la pipeline String non può rappresentarli;
  funzioni batch-1 (mb_strlen &c.) restano UTF-8-only e rifiutano
  HTML-ENTITIES con ValueError rumoroso). **(2) `DOMDocument::loadHTML` /
  `loadHTMLFile` / `saveHTML` / `saveHTMLFile`** — modalità HTML4/libxml2 del
  parser HTML (`parse_html_impl(html4)`): doctype implicito "-//W3C//DTD HTML
  4.0 Transitional//EN", testo vagante a livello html/head avvolto in `<p>`
  implicito, NIENTE skeleton forzato (niente head/body impliciti se non
  richiesti dal contenuto), script/style come CDATA (nodeType 4), attributi
  senza valore → valore=nome, PI SGML (`<?php …?>`, trick `<?xml
  encoding=…?>` con charset onorato), doctype con PUBLIC/SYSTEM id
  case-preserved, charset BOM > xml-decl > meta (label verbatim in
  `$doc->encoding`) > default ISO-8859-1 VERO, `xmlVersion` null, nodeType
  documento 13 (XML_HTML_DOCUMENT_NODE); **errori libxml strutturati**
  (level/code/line/col: Tag invalid 801, Attribute redefined 42,
  htmlParseEntityRef no name 68, Document empty 4; colonna = posizione
  cursore in BYTE dell'input raw, char-aware dopo transcodifica latin1) e
  `libxml_use_internal_errors(false)` che SVUOTA il buffer come ext/libxml;
  serializer `save_html` fedele (entity named HTML4/`&#N;` per i non-ASCII,
  void elements HTML4, boolean attrs minimizzati, URI-escape
  `xmlURIEscapeStr` su href/src/action e name-su-`<a>`, `<!DOCTYPE …>\n` +
  newline finale). Probe spec-first 2 round: **514 righe byte-identiche
  all'oracle**. Divergenza nota: colonne errori con input high-byte non
  UTF-8 esatte solo per input ASCII/UTF-8. **(3) ext/xml SAX expat-style**
  (SimplePie/WP_Widget_RSS): builtin `__xml_tokenize` su quick-xml (eventi
  granulari NON coalescati come libxml: un evento per run di testo, per
  reference risolta, per sezione CDATA; DOCTYPE internal subset con
  `<!ENTITY>` risolte — SimplePie dichiara le entity HTML così; namespace
  `xml_parser_create_ns` con separator arbitrario, xmlns strippati, default
  ns solo sugli elementi) + API `xml_parser_*` nel prelude (create/create_ns/
  set_option/get_option/set_object/handlers/parse/parse_into_struct/
  get_error_code/error_string/posizioni; **codici errore = xmlParserErrors
  libxml della compat-layer** oracle-pinned: empty/truncato 5, mismatch 76,
  entity non dichiarata 26, charref invalido 9; `xml_error_string` =
  trascrizione verbatim di `error_mapping[]` di ext/xml/compat.c;
  XML_OPTION_SKIP_WHITE è un NO-OP sulla compat-layer, oracle-probed).
  Probe byte-identico. **(4) `is_callable(['C','m'])` AUTOLOADA la classe**
  come zend_is_callable (l'assert di SimplePie Registry su classi non ancora
  caricate). ⚠️ bug engine scoperto (aperto): `static $x = array(…)` dentro
  una funzione del PRELUDE panica (index out of bounds run.rs:233) — worked
  around con array non-static; userland OK.
- 2026-07-17 (sessione WordPress-13): 🏁 **WP CORE SUITE: gruppo RESTAPI
  sbloccato** (3514 test; prima phpr NON TERMINAVA — >50 min CPU, junit vuoto).
  Root cause del "hang": **l'assegnamento composto valutava il target PRIMA del
  RHS** — Zend (ASSIGN_OP/ASSIGN_OBJ_OP) valuta il RHS e POI fa il
  read-modify-write, quindi un RHS che muta il target contribuisce col valore
  post-mutazione. `validate_custom_css` di WP itera con
  `$at += strcspn($css,'<',++$at)`: col valore stantio `$at` oscillava per
  sempre (`WP_REST_Global_Styles_Controller_Test::
  test_update_allows_valid_css_with_more_syntax`, il 1173° test: loop VERO,
  non lentezza). Fix su TUTTE le forme non ancora RHS-first: slot
  (compile/expr.rs, via Op::Swap), prop `$o->p op= rhs` (Dup/rhs/Swap/PropGet/
  Swap — __get/__set invariati), `$GLOBALS['x'] op=` e superglobali
  (compile/assign.rs); i path dim (FieldAssignOp/AssignOpPath) erano già
  corretti. Probe oracle-pinned: `$x+=++$x`→4, `$y-=$y=3`→0, `$z*=$z+=1`→9,
  statics→4, `$g+=$g+=3`→16, `$s.=$s.="b"`→"abab". Altri 3 fix engine emersi
  dai 11F di Tests_REST_API:
  **(a) i Value-builtin dereferenziano i `Zval::Ref` al choke point**
  (run_value_builtin; un call-site dinamico — callee ignoto ⇒ prefer-ref —
  consegnava Ref ai predicati `is_*` che matchano le varianti direttamente:
  `rest_get_best_type_for_value` con `$checks[$type]($value)` tornava ''
  o il tipo sbagliato);
  **(b) `array_unique` onora `__toString`** (gate nel deep-stringify
  precompute condizionato ai flag SORT_STRING-like + `ctx.to_zstr` nel
  builtin; PHPUnit deduplica oggetti `ExecutionOrderDependency` con
  array_unique — prima warning spurio E dipendenze @depends collassate);
  **(c) cast string→int SATURANTE per overflow** (`zend_dval_to_lval_cap`:
  `(int)'9223372036854775807000'`→i64::MAX, negativo→i64::MIN, inf/nan→0 —
  una stringa numerica satura, un float zval vero continua a wrappare;
  convert.rs, `rest_sanitize_value_from_schema` sui bigint).
  Perf: **cache VM dei descriptor `__reflect_method_info`** (chiave
  (ClassId, nome_lc), immutabile per costruzione — la build della suite
  PHPUnit ricostruisce migliaia di ReflectionMethod sulle stesse coppie:
  fase build 60s→21s) + compare alloc-free in find_method_reflect + fast-path
  alloc-free in compute_stringify (pre-scan senza oggetto → mappa vuota).
  ⚠️ Lezioni: l'oracle restapi oggi = 149s/1E/4W/6S (trunk wpdev cambiato vs
  memo "~30s"); phpr bufferizza stdout anche su pty (⇒ `script` NON dà
  progresso live: usare marker file-based in set_up/bootstrap — TESTLOG/
  TRACELOG/HASHLOG con `getenv`); il triage dell'hang è stato: sample →
  hashlog → stage-marker → per-test-marker → bisezione del singolo test.
- 2026-07-17 (sessione WordPress-12): 🏁 **ext/fileinfo NATIVO (§2.7) + WP CORE
  SUITE: gruppi POST/USER/QUERY A PARITÀ ORACLE** (post 906, user 1341, query
  1889 — user era 24E, post 2E/13F, query 2E/7F). Fix engine trasversali:
  **(a) ArrayAccess sulla catena di scrittura variable-rooted** (`$a[0][$i]=v`
  con oggetto intermedio → offsetGet+drill ripreso; leaf compound/incdec via
  offsetGet→op→offsetSet — PathAa in vm/mod.rs+arrays.rs; sbloccava
  sodium_compat BLAKE2b, hash **byte-identici**);
  **(b) confronto oggetti stessa classe property-wise** (zend_std_compare:
  count poi per-chiave; cross-class resta uncomparable — ops::compare;
  `assertEqualSets` di PHPUnit canonicalizza con sort di stdClass);
  **(c) ReflectionObject vede le prop dinamiche dell'istanza**
  (hasProperty/getProperty via `__reflect_object_instance`; è ciò che usa
  assertObjectHasProperty);
  **(d) PCRE non-/u BYTE-ORIENTED**: subject e pattern con byte alti passano
  alla vista Latin1 1-byte-per-char (preg.rs subject_text/compile) —
  `[\x80-\xff]` matcha i singoli byte UTF-8 (esc_url), `.` un byte, i
  letterali multibyte per byte;
  **(e) grammatica date**: `date_create*`/`*_from_format` propagano
  `$timezone` e tornano FALSE su parse-error (get_gmt_from_date!);
  "May 4th, 2008"/"June 12, 2008" (textual month-first, ordinali, virgole);
  "June 21st +1 year" (mese+giorno ordinale nel parser relativo);
  "May 2028" → day=1; bound timelib (mese≤12, giorno≤31, ore≤24 —
  '2020-12-41' è errore, non wrap);
  **(f) parse_url stile macOS**: iscntrl BSD include C1 0x80–0x9F → `_`
  (php_replace_controlchars come l'oracle brew);
  **(g) SplFixedArray**: `$a[] =` → Error "[] operator not supported".
  Residuo WP-12: gruppo `restapi` (3514 test) — phpr non termina (>37min
  CPU vs ~30s oracle): triage perf/hang a WP-13.
- 2026-07-16 (sessione WordPress-11): 🏁 **WP CORE SUITE, GRUPPO MEDIA A PARITÀ
  ORACLE: 762/762 test, 0E/0F** (da 680 test/11F — 82 test non si caricavano
  nemmeno). Fix engine trasversali:
  **(a) docblock stile Zend**: `CG(doc_comment)` sopravvive agli statement
  interposti e viene consumato dalla dichiarazione successiva
  (`backup_doc_comment` nella grammatica) — `doc_for`/`doc_span_for` in
  lower/mod.rs attaccano attraverso il gap con una bail-list conservativa
  (keyword consumatrici, modificatori, graffe, heredoc: mai un docblock
  sbagliato, al peggio assente) e `push_deferred` re-emette il docblock
  staccato nello snippet deferred. Era il bug degli 82 test media invisibili:
  `/** @group media */ require_once …; class X` perdeva i gruppi PHPUnit.
  **(b) `@` a semantica BEGIN/END_SILENCE esatta**: `error_level` salvato e
  mascherato con `&= 4437` (E_FATAL_ERRORS) all'ingresso, ripristino
  condizionale all'uscita e sull'unwind da eccezione (solo se il corrente è
  fatal-only e il salvato no); l'handler utente VIENE chiamato sotto `@`
  (flush a SuppressBegin/End) e legge `error_reporting()`==4437 — il
  protocollo con cui PHPUnit scarta le warning soppresse; le scritture di
  `error_reporting($x)` dentro `@` sopravvivono alla regione. Fixa anche
  bug27731/bug33771/error_reporting02/03/08/09/10 e restore_error_reporting
  del corpus. Stato `@` parcheggiato per-fiber (incluso il mask).
  **(c) getimagesize: TIFF II/MM, JP2/JPC, PSD, ICO** (port fedele dei
  php_handle_* di image.c, con `bits`/`channels` e mime oracle-esatti).
  **(d) exif**: `COMPUTED.UserComment`/`UserCommentEncoding` (header 8 byte
  ASCII/UNICODE→ISO-8859-15/JIS-raw/UNDEFINED + strip spazi Olympus),
  `ApertureFNumber` con overwrite-FNumber/fallback-APEX in f32 (exif.c),
  IFD INTEROP (tag 0xA005 → sezione INTEROP, tabella IOP).
  **(e) stream wrapper userland oltre fopen**: famiglia stat via `url_stat`
  (file_exists/is_file/is_dir/is_readable/is_writable/filesize/filemtime/
  stat, flags QUIET=2 come php_stat) e getimagesize/exif_imagetype/
  exif_read_data via lettura-a-EOF del wrapper (`__exif_read_data_bytes`/
  `__exif_imagetype_bytes` interni; `Vm::user_wrapper_path_op`).
  **(f) return-by-ref di elemento di proprietà statica**
  (`function &f(){ return self::$a[$k]; }`) via `static_prop_rmw` — il ref
  cell coniato nella copia e la write-back la installa (separazione alla
  Zend); era l'errore `get_directory_ref` di WP_Test_Stream.
  **(g) http(s):// nei builtin path-taking**: getimagesize/exif_imagetype/
  exif_read_data leggono via il trasporto HTTP (come lo stream layer Zend) —
  `read_for_builtin` in file.rs; chiude `test_wp_crop_image_with_url` (crop
  byte-identico all'oracle, 3477 byte).
  **(h) INI site-health**: memory_limit 128M (ora SETTABLE: wp_raise_memory_limit
  scrive 256M in admin e site-health riporta memory_limit+admin_memory_limit —
  phpr non applica comunque il limite), max_input_vars 1000, max_input_time e
  max_execution_time SAPI-dipendenti (CLI 0/-1 hardwired alla Zend, cli-server
  30/60 nello swap web di vm/mod.rs).
  Residui documentati: **ext/fileinfo assente** (12 skip media che l'oracle
  passa: `@requires extension fileinfo`; da valutare per WP-12),
  `curl_version` e blocco opcache di site-health (phpr non ha curl di sistema
  né opcache: righe assenti), nonce/timestamp volatili nell'http battery.
  Provider full-suite noti: `mb_convert_encoding` BIG-5 (Tests_DB_Charset) e
  un throw in `data_wp_validate_site_data` via current_time (Tests_Multisite_Site)
  → ErrorTestCase; backlog full-suite.
- 2026-07-15 (sessione WordPress-10): 🏁 **RESIDUI SAPI CHIUSI + WP CORE TEST
  SUITE AVVIATA (gruppo option A PARITÀ ORACLE: 413 test 0E/0F)**.
  SAPI (15/15 probe byte-id, wp10-harness/sapi-probe): body
  `Transfer-Encoding: chunked` decodificato come PHP (php://input
  de-chunked, CONTENT_LENGTH assente, $_POST dal body decodificato);
  router `return false` NON scarta l'output del router (oracle-pinned:
  per target .php si accoda IN TESTA al body dello script; per statici/404
  è flushato RAW sul socket PRIMA della status line — sì, risposta
  malformata anche nell'oracle); `headers_sent()` sotto il web SAPI flippa
  quando il sink ATTRAVERSA il write-buffer 4096 del cli-server
  (`WEB_SEND_THRESHOLD`), con "output started at FILE:LINE" = lo statement
  che attraversa (non il primo echo) e header()/setcookie() successivi →
  Warning "Cannot modify header information" + drop;
  `PHP_CLI_SERVER_WORKERS` accettato (esecuzione resta sequenziale =
  workers 1, divergenza dichiarata). Lexer: escape `\u{...}` decodificato
  nei literal double-quoted (ri-unescape dal RAW, così `\\u{..}` resta
  letterale; heredoc/interpolate già coperti); le sequenze INVALIDE
  restano letterali (PHP: Parse error — cavalca il rendering parse-error
  non fedele, divergenza esistente). Nuovi: `getopt()` fedele
  (stop al primo non-option, rest_index by-ref via HOST_OUT, cluster
  corti, ripetute→array, `--`, long =/separato — il launcher di PHPUnit 9
  lo usa); base statica nei write-target (`self::$wpdb->last_error = …`,
  lower_place arm StaticProperty); **DISPATCH PRIVATO NON-VIRTUALE**
  (`parent_private_rebind`, port di zend_get_parent_private_method):
  `$this->m()` dallo scope che dichiara una private `m` lega a QUELLA
  anche se la sottoclasse la ridefinisce — PHPUnit 9 runBare→
  checkRequirements() era il consumer (408 errori→0 nel gruppo option);
  applicato al dispatch istanza (posizionale+named+arg-ref-target), NON
  al dispatch statico; `timezone_identifiers_list(ALL_WITH_BC)` = +179
  zone BC con sort case-insensitive (sanitize_option timezone_string di
  WP; il bit 2048 da solo resta vuoto come nell'oracle).
  **Harness WP core test suite** (wordpress-develop + wp-tests-config su
  MySQL wptests, ricetta in NEXT_SESSION): gruppo option 413 test
  **IDENTICO all'oracle** (1 phpunit-warning + 3 skip); gruppo media
  prima passata 680 test/11F + 82 non caricati (triage → WP-11).
  Gate finale: corpus 1526 (2 test FIXATI dal dispatch privato:
  dynamic_call/bug46246, inheritance/bug38772; 0 nuovi), altre 5 suite
  identiche, ORM 3E/13F per nome, hk 1665 0E/0F, batterie tutte verdi.
  Residuo documentato: il debug tab di site-health legge anche
  max_input_vars/max_execution_time/memory_limit (sotto -S l oracle
  mostra 128M, phpr -1) — 4 chiavi INI da allineare (WP-11). Lezione
  infrastruttura: ENOSPC a metà gate tronca i binari allo strip e un
  rebuild sul target exFAT produce binari INCONSISTENTI (falsi fail):
  recovery = cargo clean dei crate del workspace + rebuild sul target
  interno; il phpt-runner NON viene ricompilato da `build -p php-cli`
  (binari separati: buildare SEMPRE entrambi prima di un gate).
- 2026-07-15 (sessione WordPress-9): 🏁 **ext/gd sulla LIBGD DI SISTEMA via FFI
  + ext/exif — MEDIA PIPELINE WORDPRESS A PARITÀ BYTE TOTALE** (§2.6).
  Decisione di design: invece del crate `image` (functional-parity prevista
  dalla roadmap), FFI alla stessa `libgd.3.dylib` che l'oracle brew linka
  (pattern zlibio): stessi codec ⇒ **file generati byte-identici**, wrapper
  sottili ⇒ semantica GD esatta gratis (resampling, palette, alpha, colori).
  Superficie: `php_types::gdio` (FFI + GdImg RAII + error-callback va_list
  formattato con vsnprintf, mappato in Warnings dal prelude via
  `__warning_from_caller`); `vm/gd.rs` (25 host builtin `__gd_*`, handle in
  `Vm.gd_images`, liberati eagerly dal `__destruct` del prelude); 6° prelude
  `prelude_gd.php` (GdImage opaca + ~60 fn: create/from{jpeg,png,gif,webp,
  avif,bmp,wbmp,tga,string}, jpeg/png/gif/webp/avif output su file/OB/stream
  resource via I/O PHP-side, copyresampled/resized/copy, rotate, flip, crop,
  scale, palette↔truecolor, colori/alpha/at/sforindex, draw primitives,
  bitmap font string/char, interlace/antialias/blending/savealpha,
  setinterpolation, imagetypes 495, gd_info oracle-shape, imagedestroy
  deprecato 8.5); ~90 costanti IMG_*/GD_*/PNG_* + EXIF_USE_MBSTRING in
  `resolve_constant` (IMAGETYPE_COUNT corretta 21→22); `gd`+`exif` in
  extension_loaded. **Classe opaca engine-level**: `is_opaque_handle_class`
  (php-types) consultata da Op::Clone (Error), serialize (Exception,
  graph-walk), var_dump (debug-info vuoto sintetico), var_export/print_r
  (props nascoste), json_encode ({}), Reflection (0 metodi/0 props).
  ext/exif nuovo `php-builtins/exif.rs`: `exif_imagetype` (detection
  php_getimagetype completa incl. AVIF ftyp-brands), `exif_read_data`
  (JPEG/TIFF: IFD0+EXIF+GPS+IFD1 thumbnail in ordine file, COMPUTED con
  html/ByteOrderMotorola/ApertureFNumber/Copyright/Thumbnail.*, COMMENT gd,
  SectionsFound, formati rational "n/d", UndefinedTag:0x%04X, FileDateTime/
  FileSize/MimeType, modalità $as_arrays), `iptcparse` (dataset 2#NNN,
  extended length). `getimagesize`: parser AVIF (ispe/pixi/auxC) con
  bits/channels oracle-shape, out-param `&$image_info` (APPn first-wins) via
  CallHostBuiltinOut (nuove entry HOST_OUT + pair-builtin registry
  `__getimagesize_info`), Notice "Error reading from %s!" per input <12 byte.
  Fix `strtotime`: formati timelib datenocolon `YYYYMMDD` + timenocolon
  `HHMMSS[+ZZZZ]` (IPTC 2#055/2#060 → created_timestamp di
  wp_read_image_metadata). Nuovo builtin `__notice_from_caller` (E_NOTICE al
  call-site, per imagecolorat out-of-bounds). **Verifiche**: 11/11 probe
  gd/exif BYTE-ID (costanti/classe/load+warnings libjpeg formattati/save
  md5 su tutti i formati e qualità/pipeline resize WP md5/rotate+flip/
  palette/OB+stream/getimagesize+iptc/exif Canon completo/misc);
  media-probe WP (media_handle_sideload di foto EXIF 5472px + png alpha +
  gif palette su WP 7.0.1/MySQL): metadata+subsizes+srcset+image_meta
  EXIF/IPTC+editor rotate/flip/crop+conversioni webp/avif **BYTE-ID,
  inclusi gli md5 di TUTTI i file generati**; batteria HTTP sequenziale
  `php -S` vs `phpr -S`: **32/32 risposte BYTE-IDENTICHE senza
  normalizzazione** (13 front + login 5 + admin 12 con site-health e
  upload.php + site-health?tab=debug + media-new.php) — i residui webp/avif
  e php_extensions gd di WP-5/8 sono CHIUSI.
- 2026-07-15 (sessione WordPress-8): 🏁 **ext/mysqli NATIVA (crate `mysql` v28)
  — WORDPRESS 7.0.1 INSTALLATO E SERVITO SU MYSQL VERO A PARITÀ ORACLE**
  (chiude roadmap tappa 3b: cade la dipendenza dal plugin
  sqlite-database-integration, si aprono gli hosting reali). Architettura:
  5° prelude `prelude_mysqli.php` (6 classi + ~75 funzioni procedurali, §2.5)
  → host builtin `__mysqli_*` in `vm/mysqli.rs` (connessioni/stmt =
  handle in `Vm.mysqli_conns`/`mysqli_stmts`, pattern `__pdo_*`); result set
  bufferizzati host-side e cursore lato PHP; ~100 costanti MYSQLI_* in
  `resolve_constant`; `mysqli`+`mysqlnd` annunciate da extension_loaded;
  nuovo builtin `__warning_from_caller` (E_WARNING attribuito al call-site,
  gemello di `__deprecated_from_caller`) per il path REPORT_ERROR.
  Fedeltà chiave (probe-FIRST, **11/11 probe byte-id** vs oracle su MySQL
  9.7.1 locale): protocollo testo → TUTTE stringhe, protocollo binario
  (prepared) → tipi nativi; errori server verbatim (1045/1049/1064/1146/
  1062 + sqlstate); errori client sintetizzati (2002 "Connection refused",
  2019 charset, sqlstate connect sempre HY000); NUM_FLAG client-side come
  mysqlnd; charsetnr 255 via SET NAMES post-handshake; caching_sha2
  full-auth (RSA su TCP) col crate, byte-id anche il fail path.
  **Harness WP-MySQL**: `wp core install` (wp-cli da sorgente) su MySQL
  9.7.1 con l'oracle E con phpr → **schema dbDelta 0 diff, wp_options
  identiche salvo 5 volatili** (cron/timestamp/hash-di-path); poi STESSO
  albero+DB (install oracle, pretty permalinks) servito da `php -S` e
  `phpr -S` in sequenza: **13 rotte front+feed+rest+robots+404 e login
  flow 5 step BYTE-IDENTICI**, admin 12 pagine con soli volatili noti
  (nonce/orologio, auto-draft id, antispambot rand, webp/avif gd,
  site-health moduli+loopback, sessione doppia da cattura sequenziale;
  widgets.php 500 = parità block-theme, body 2796b identico). Gate: corpus
  1528 / sess 67 / date 377 / refl 294 **fail-set IDENTICI per nome** ·
  ORM 3484 test **3E/13F tutti nei residui catalogati** · http-kernel
  (tip aggiornato) 1665 test **0E/0F** · cargo 1550/0 · probe battery
  11/11 anche post-reboot. ⚠️ un crash una-tantum di `phpr -S` al primo
  passaggio su widgets.php NON riprodotto in 3 batterie complete
  successive (watch). Divergenze consapevoli in §2.5.
- 2026-07-15 (sessione WordPress-7): ⚡ **PERF INFRASTRUTTURALE, gradini 1+2
  (allocatore + hasher)** — la controparte phpr dell'infrastruttura di Zend
  (ZMM a bin/chunk; zend_string con hash precomputato). Gradino 1
  (`b574eff`): **mimalloc `#[global_allocator]`** nei binari phpr e
  phpt-runner (il profilo post-unit-cache dava malloc/free di sistema a
  ~16% dei campioni): home WP 1.20s → **0.91s** (-24%), dashboard 1.25s →
  **0.95s** (-24%); mi_malloc+mi_free ~5% nel sample post-swap. Gradino 2
  (`de67428`): **rustc-hash (FxHash) sulle mappe calde** — PhpArray::index
  (l'ordinamento osservabile vive in `entries`, l'hasher non è osservabile),
  tutte le mappe del Vm via alias di modulo (class_index, linked_functions,
  constants, preg_cache, enum_cache, included_files, static_props,
  closure_statics, gc_roots/destructed/gc_cycle_roots, generators, fibers,
  ecc.), Frame::dyn_vars, Module::class_index e CompiledClass::prop_info
  (letto a ogni accesso a proprietà). `Vm::unit_fp` NON toccato (suo
  DefaultHasher, indipendente); var_dump_debug/stringify_args restano std
  (attraversano l'API di php-builtins, path freddi). Home 0.91s → **0.76s**
  (-17%), dashboard 0.95s → **0.81s** (-15%). **Cumulato WP-7: home 1.20 →
  0.76s (-37%), dashboard 1.25 → 0.81s (-35%)**; SipHash sparito dal
  top-of-stack del sample (era ~10%): restano memmove/memcmp e clone/drop
  di Zval (churn COW/arena per-request: in agenda DOPO la roadmap
  funzionale). Nessuna divergenza utente: gate per nome su corpus/sess/
  date/refl IDENTICI, ORM 3E/13F stessi 16, hk 1663/3846 0F, cargo 1550/0,
  SAPI 48, pretty 10/10 byte-id, admin 12/12, login 5/5 — su ENTRAMBI i
  gradini, ciascuno sul proprio binario definitivo.
- 2026-07-15 (sessione WordPress-6): ⚡ **UNIT-CACHE per-request (opcache-like):
  cache process-wide dei moduli include già lowerati+compilati+RILOCATI**,
  chiave = identità file (path canonico + mtime ns + size) + fingerprint dello
  stato VM osservabile dalla lowering seminata (`Vm::unit_fp`): hash-chain degli
  eventi di load (identità del main; path+mtime+size di ogni include; sorgente
  di ogni eval) + contatori seed/statics + digest ordinato di seed_globals
  (gli SLOT globali sono baked per indice) e seed_traits + digest
  order-independent di class_index (name→id: gli id classe sono baked negli
  op) e seed_aliases. Un web server che riserve la stessa pagina rigioca la
  stessa catena di include su una VM fresca → ogni step riproduce lo stesso
  fingerprint → lower+compile+relocate saltati e il modulo (immutabile, già
  leaked) ri-guidato as-is. Riuso double-checked strutturalmente per hit
  (baseline statics + remap ricomputato == remap cached, che convalida anche
  lo stub-mask); mismatch = miss con fallback al percorso fresco. Cacheable
  solo lowering "pure" (nessun retry autoload/defer: quelle hanno side effect
  non rigiocabili). `drive_unit` splittato in `unit_class_remap` +
  `run_linked` (metà condivisa col percorso cached). Effetto misurato su
  WordPress 7.0.1 (wp-p): home 1.85s → **1.2s** (-35%), dashboard ~2.9s →
  ~2.0s; quota parse+lower+compile del tempo per-request 39% → 3.4%
  (profilo `sample`). La cache limita anche la crescita di memoria del
  server (i moduli include venivano già Box::leak-ati A OGNI load; ora il
  leak è bounded dal riuso). Nessuna divergenza utente: batterie SAPI 48
  probe, WP pretty 10/10, login-flow 5/5 e admin 12/12 a parità oracle
  (residui = i legittimi WP-5: auto-draft id/orologio, capability gd).
- 2026-07-14 (sessione WordPress-5): 🏁 **wp-admin VIA HTTP: login flow
  completo + dashboard + 12 pagine admin a parità oracle, e PRETTY
  PERMALINKS attivi con 10 rotte frontend BYTE-IDENTICHE senza alcuna
  normalizzazione** (post /2026/07/hello-world/, redirect canonico 301
  senza slash, page, category, author, feed, 404 pretty, /wp-json/wp/v2/
  pretty, archivio mese, home). Login flow pinnato probe-FIRST sull'oracle
  (curl cookie-jar): GET wp-login 4593b → POST credenziali 302 →
  /wp-admin/ (cookie auth wordpress_* con path /wp-content/plugins +
  /wp-admin + logged_in; struttura Set-Cookie identica, diverge solo il
  token di sessione random) → dashboard 125854b → edit.php 109732b →
  bad-login 4957b: tutti byte-id modulo nonce/timestamp. Write-path
  verificato: POST options.php con nonce → 302 settings-updated + opzione
  persistita nel DB; POST options-permalink.php attiva la structure.
  Fix engine della sessione (ognuno ridotto a repro minimale):
  **(a) hoisting delle funzioni di unità inclusa PRIMA del run del body**:
  drive_unit registrava le `linked_functions` solo a unità completata —
  Zend le hoista a compile-time dell'include, quindi un include annidato
  (o un hook che scatta da lì) deve già vederle (wp-admin/menu.php
  registra `_add_themes_utility_last` su admin_menu e il do_action parte
  da wp-admin/includes/menu.php, incluso prima che menu.php finisca).
  **(b) symbol table globale UNICA per le catene di include a global
  scope**: un nome fresco introdotto da un'unità il cui includer non è il
  frame 0 ma la cui catena bridge_caller arriva a frame 0 ora aliasa la
  cella GLOBALE (global_slot_by_name) invece di una cella locale staccata
  — menu.php costruiva `$menu`, includes/menu.php faceva `global $menu` e
  leggeva la cella globale NULL: uksort(null) fatal.
  **(c) call_user_func_array passa gli elementi-reference BY-REFERENCE**
  (prima li decadeva pur sapendo quali erano ref): il Walker di WP
  accumula l'output con `call_user_func_array([$this,'start_el'],
  array_merge(array(&$output,…), $args))`.
  **(d) spread by-ref (SEND_VAR_EX sui componenti)**: `build_args_array`
  pusha i componenti plain-Var come PushRef (Walker 6.x: `$this->
  start_el( $output, $el, $depth, ...array_values($args) )` con
  `start_el(&$output,…)`); split_args_from_array_value e spread_pairs
  preservano i Ref (il binder li decade sui param by-value). NB: per i
  callee NOTI al compile-time con parametri by-ref lo spread resta
  Unsupported→skip (il gate l'ha imposto: instradarli su CallSpread
  perdeva il write-back degli elementi di `test1(...$array)` con
  variadic by-ref e il warning "by unpacking a Traversable" —
  arg_unpack/by_ref*.phpt); il caso WordPress è il dispatch dinamico
  di metodo, che è coperto.
  **(e) ⭐ `zend_array_dup`: PhpArray::clone SPEZZA le reference con
  refcount 1** (residuo di `foreach (… as &$v)` dopo la morte dell'alias)
  come Zend/zend_hash.c — senza lo split, il by-ref foreach di
  WP_REST_Server::get_routes sulla COPIA di `$this->endpoints` scriveva
  attraverso le celle sopravvissute dentro la property: Allow header "1"
  (strtoupper(true)), methods [1,1], e il preload REST del block editor
  amputato di ~190KB su post-new.php. Con l'ECCEZIONE Zend-esatta del
  self-cycle: il ref il cui referente è l'array sorgente stesso
  (`$a[] =& $a`) resta condiviso (`Z_ARRVAL_P(Z_REFVAL_P(data)) !=
  source` in zend_array_dup_element; bug69376/bug69376_2 — il gate l'ha
  beccata). Lo split ha anche FIXATO 5 test corpus di vecchia data
  (bug72543, dynamic_call/bug52940, gc/bug60138, switch/bug71756,
  switch/bug72508).
  **(f) tabella entità HTML 4.01 COMPLETA** (152 nomi symbols+special:
  hellip, mdash/ndash, quote tipografiche, greche, frecce, matematici,
  euro, carte…) in htmlentities/html_entity_decode — chiude il vecchio
  scope-out D-56.1; WP_Scripts::localize() html_entity_decode-a OGNI
  stringa localizzata (`Crunching&hellip;` deve tornare `…`).
  **(g) `?>` da TERMINATORE di statement inghiotte il newline**: quando
  `?>` chiude uno statement senza `;` (`echo $key\n?>`) il parser lo
  assorbe come terminator e il nodo ClosingTag non esiste — ora il check
  guarda i 2 byte di sorgente prima del chunk Inline (l'attributo
  `preload=` dei template media di wp-includes/media-template.php).
  **(h) array_flip fa ZVAL_DEREF sugli elementi** (la map di
  WP_Theme::get_page_templates arrivava col titolo Ref-wrapped → il
  <select> Template della Quick Edit di edit.php?post_type=page spariva).
  **(i) RecursiveArrayIterator** nel prelude (WpOrg\Requests
  Curl::get_expect_header cammina gli header data con
  RecursiveIteratorIterator).
  **(j) timezone_open/timezone_offset_get/timezone_name_get + VALIDAZIONE
  del costruttore DateTimeZone** (matrice oracle-pinned: `GMT+2`→`+02:00`,
  `z`→`Z`, offset ±H[H][:MM] normalizzati, identificatori/abbreviazioni
  validati con probe __tz_offset, altrimenti DateInvalidTimeZoneException
  — gerarchia DateException aggiunta; wp_timezone_override_offset moriva
  su timezone_string=UTC). Più DateTimeZone::__debugInfo (var_dump con
  timezone_type 1/2/3 + timezone, timezone_open_basic1 verde), ZPP
  TypeError su timezone_offset_get/name_get, e
  **DateTimeZone::getTransitions + timezone_transitions_get** (nuovo host
  `__tz_transitions` sul TZif: riga 0 = stato a $timestampBegin poi ogni
  transizione nel range, byte-id all'oracle su Europe/Rome/UTC; false per
  zone offset/abbreviazione come PHP — options-general.php col
  timezone_string settato mostra il prossimo cambio DST da lì). Residui EX-SKIP documentati
  (test che prima venivano saltati per builtin mancante e ora girano):
  timezone_open_warning (il Warning procedurale su tz invalida non è
  emesso — ritorna solo false), timezone_offset_get_error /
  DateTime_construct_error / date_create-1 (il parser datetime di phpr
  non accetta un NOME di timezone nudo come stringa datetime: `new
  DateTime("GMT")` → Failed to parse; e niente ArgumentCountError
  sull'arità del ctor), bug78139 (lettere militari singole come
  abbreviazione: `x`→`X` type 2), bug79580 (messaggio del parser "A
  'day of year' can only come after a year has been found" vs
  "Unexpected data found.").
  Divergenze admin RESIDUE legittime e documentate: `webp_upload_error`/
  `avif_upload_error` in plupload e il site-health test php_extensions
  "required modules missing" (ext/gd e affini non implementate — tappa
  media della roadmap WP); antispambot() usa rand() per carattere (non
  riproducibile per costruzione); post_ID auto-draft e pagegen_timestamp
  dipendono dalla storia del DB/istante.
  Gate-n finale (4776cd24/scratchpad): corpus **2527 pass** (+5 fixati
  dal ref-split: bug72543, dynamic_call/bug52940, gc/bug60138,
  switch/bug71756, switch/bug72508; 0 nuovi fail) · sess 162 e refl 175
  IDENTICI per nome · date **225 pass** (+9 vs baseline: fixati
  DateTimeZone_clone_basic1, DateTimeZone_construct_basic, bug68406,
  DateTimeZone_getTransitions_basic1, DateTimeZone_getTransitions_bug1,
  bug80963, bug81504; +6 ex-skip sopra documentati) · ORM 3E/13F stessi
  16 nomi · hk 1663/3846 0F · cargo 1550/0 · batteria SAPI 47/48
  byte-id (1 = Max-Age dipendente dall'orologio) + 8 pagine WP byte-id ·
  login 5/5, admin 12/12 (modulo clock), pretty 10/10 byte-id ·
  wp-cli smoke identico.
- 2026-07-14 (sessione WordPress-4): 🏁 **SAPI WEB SERVER: `phpr -S host:port
  [-t docroot] [router.php]`, work-alike del cli-server di PHP** — e
  **WordPress 7.0.1 SERVITO VIA HTTP a parità byte con l'oracle `php -S`**:
  sullo stesso albero+DB SQLite, 8/8 risposte identiche (homepage, ?p=1,
  ?page_id=2, 404, wp-login.php, /wp-json/, /robots.txt→301, ?feed=rss2),
  più la batteria SAPI di 48 probe byte-id (headers normalizzati solo su
  Date/porta/tmp-path) e il log stderr identico riga per riga (banner,
  Accepted/[code]/Closing, diagnostiche "PHP Warning:" timestampate).
  Architettura: server sequenziale hand-rolled su `TcpListener` in
  php-cli/server.rs (la VM è piena di `Rc`, e serve controllo byte-level su
  status-line e ordine header); per-request `WebRequest` thread-local in
  `php_types::sapi` (php://input, getallheaders, upload registry) + VM in
  modalità web (`vm.web`). Semantica oracle-pinned:
  **(a) superglobali web** (ordine esatto $_SERVER del cli-server, HTTP_* in
  wire-order con CONTENT_TYPE/LENGTH doppi, $_GET/$_POST/$_COOKIE con le
  asimmetrie vere: nome cookie NON urldecodato, first-wins; multipart
  rfc1867 → $_FILES con nesting per-attributo, tmp file registrati per
  is_uploaded_file/move_uploaded_file e spazzati a fine request);
  **(b) header-family stateful** (header/headers_list/header_remove/
  setcookie/setrawcookie/http_response_code; replace = REMOVE+APPEND in
  coda, non in-place — pinned col Content-Type tardivo del feed RSS;
  Location→302 implicito; il cli-server bufferizza la risposta INTERA →
  headers_sent() sempre false e header() funziona anche dopo output);
  **(c) display html_errors=1** (`<br />\n<b>Warning</b>:  … in <b>F</b> on
  line <b>N</b><br />`, fatal col tail "thrown in" boldato) + error_log
  lines per lo stderr del server; ini registrate display_errors/log_errors/
  html_errors/output_buffering/implicit_flush (default CLI + override web);
  **(d) risoluzione path del cli-server** (walk longest-prefix con
  PATH_INFO, index.php/index.html nelle dir, **fallback a index.php del
  docroot con PATH_INFO=path per gli URL virtuali** — è ciò che serve
  /robots.txt e /wp-json/ di WP senza router; router script con
  SCRIPT_NAME=path e fall-through su `return false`; 404 template
  byte-identico; mime map generata da sapi/cli/mime_type_map.h, charset
  solo su text/*; statiche con Content-Length, PHP senza);
  **(e) session web** (id dal cookie di richiesta; Set-Cookie PHPSESSID
  solo per id nuovi; cache-limiter nocache: Expires/Cache-Control/Pragma).
  Fix engine emersi servendo WP, tutti probe-pinned:
  **(f) condizionali PCRE con condizione lookahead** `(?(?=A)B|C)` →
  riscrittura equivalente `(?:(?=A)B|(?!A)C)` (wp_html_split/wptexturize);
  **(g) `[` NUDO dentro le character class** escapato (`[([{"\-]` di
  wptexturize uccideva regex/fancy e il fallback onig rifiuta i lookbehind
  variabili) e **(h) `(?<!A|B|C)` decomposto in `(?<!A)(?<!B)(?<!C)`**
  (De Morgan, esatto) — la regola apostrofi di wptexturize compilava NULL
  e svuotava i chunk (Sample Page coi paragrafi vuoti);
  **(i) `array_replace_recursive` con elementi `Ref`-wrappati**: il match
  diretto su Zval::Array saltava i ref (residui di foreach by-ref) e
  SOSTITUIVA invece di ricorrere — WP_Theme_JSON::merge perdeva TUTTI i
  preset default di theme.json (palette/gradients/shadows dal CSS globale);
  **(j) gate BP_VAR_IS per `??`/`??=`** (`Op::PropIssetFetchGate`): una
  classe con `__get` senza `__isset` serve il valore (WP_Block->attributes
  è lazy così; `$b->attributes['k'] ?? d` dava sempre il default e i
  blocchi perdevano i default degli attributi), con il null da `__get` che
  prende comunque il default/assegna (oracle-pinned);
  **(k) `field_magic_probe`: isset()/empty() su CATENE con proprietà magica
  a QUALSIASI step** (prima il protocollo __isset/__get valeva solo al
  primo step: `empty($this->block_type->uses_context)` — privata dietro
  __get in WP_Block_Type — rispondeva true e il context dei blocchi non
  fluiva: classi wp-block-navigation-item perse); pinned anche i terminali:
  isset() con solo __get = false SENZA chiamarlo, empty() idem;
  **(l) `htmlspecialchars($s, $f, $cs, double_encode: false)`** onorato
  (entity esistenti non ri-encodate; nomi ≥2 alfanumerici — approssimazione
  della tabella per-doctype, WP normalizza prima con kses quindi combacia);
  **(m) `RecursiveRegexIterator`** nel prelude (scan template block theme),
  **`hash_hmac_algos()`**, **`move_uploaded_file`/`getallheaders`/
  `apache_request_headers`** (le ultime due SOLO sotto SAPI web, come
  l'oracle), **ENT_XML1/ENT_XHTML**, **PHP_SAPI/php_sapi_name()**
  configurabili dal host (fold compile-time sicuro: set prima di ogni
  lowering). Residui documentati: niente chunked request body né
  PHP_CLI_SERVER_WORKERS; headers_sent() resta false anche oltre i 4096
  byte (l'oracle flusha lì); l'escape `"\u{...}"` manca nel lexer; il
  doppio confine magico nella STESSA catena isset non ridispatcha (il rest
  cammina plain). Gate: batteria 48 probe + 8 pagine WP byte-id, corpus/
  sess/date/refl per NOME, hk 1663/3846 0F, ORM 3E/13F, cargo 0F.
- 2026-07-14 (sessione WordPress-3): ⚡ **PERFORMANCE del load WP: il seeding
  HIR per include è ora condiviso via `Rc` invece che deep-clonato** —
  `wp option get` su WordPress 7.0.1/SQLite passa da **22.8s a 3.0s cold /
  1.7s warm** (oracle 0.3s). Il profilo (`sample`) attribuiva ~88% del tempo
  a clone+drop dell'immagine seed per ogni include (~200 file WP):
  `low.classes = sclasses.to_vec()` deep-clonava TUTTE le ClassDecl
  accumulate (metodi HIR compresi) e `prelude_functions()` deep-clonava le
  FnDecl del prelude, poi il `Program` dell'unità li droppava tutti a fine
  include — quadratico sull'autoload storm. Ora `Program.classes:
  Vec<Rc<ClassDecl>>` e `Program.functions: Vec<Rc<FnDecl>>` (idem Lowerer,
  cache prelude e `Vm::seed_classes`): il seeding è un bump di refcount e
  il borrow checker dimostra che nessun sito muta i decl condivisi (zero
  `DerefMut`). NESSUNA divergenza semantica: corpus/session/date/reflection
  fail-set identici per NOME, http-kernel 1663/3846 contatori byte-id,
  ORM 3E/13F stessi nomi, cargo test 0 fail. Residuo per-include (~12%
  compile delle fn prelude per unità, ~20% lowering del file): pista futura
  = condividere le `Func` COMPILATE del prelude tra i moduli unità.
- 2026-07-14 (sessione WordPress-2): 🏁 **WordPress 7.0.1 INSTALLATO e
  interrogabile su SQLite sotto phpr**: `wp core download` (curl callbacks →
  Requests transport, zip estratto byte-id: 3951 file `diff -rq` puliti con
  l'oracle), `wp config create`, `wp core install` su
  sqlite-database-integration (drop-in db.php ufficiale) **senza alcun
  database error**, poi `wp core is-installed`/`option get`/`post list`/
  `user list` (roles=administrator) a parità con l'oracle. Fix engine,
  tutti oracle-pinned:
  **(a) curl response-sink options** (CURLOPT_WRITEFUNCTION/HEADERFUNCTION/
  FILE/WRITEHEADER vivono sul CurlHandle prelude; `__curl_exec(id, true)`
  restituisce [header_block, body, return_transfer, include_header] e il
  curl_exec del prelude smista: header callback riga-per-riga CRLF inclusa,
  body a chunk ≤16384, short-return → errno 23 via __curl_set_cb_error;
  probe p30 byte-id, incluso abort e array-callable);
  **(b) `uncaught_throwable` stash scopato in `run_value_thunk`** (il
  default-param thunk speculativo della reflection lasciava armato lo stash
  di render_fatal: un fatal successivo mostrava lo stack STANTIO del thunk —
  i comandi wp after_wp_load morivano su "Undefined constant ABSPATH" di
  Core_Command::get_wp_details riflesso al bootstrap);
  **(c) costanti `INI_USER/INI_PERDIR/INI_SYSTEM/INI_ALL`** (wp_initial_constants
  → wp_is_ini_value_changeable) **+ fold namespace-aware delle costanti
  engine** (dentro un namespace `const INI_ALL = 0` DEVE vincere: il fold
  compile-time ora avviene solo a namespace vuoto/nome fully-qualified
  mono-segmento, e Op::ConstFetch consulta la tabella engine sul fallback
  globale — ns_043/ns_050);
  **(d) `global $x` nelle unità main-style eseguite in scope funzione**
  (compile-time no-op era sbagliato per wp-settings.php/plugin.php require'd
  da Runner::load_wordpress: ora PushConst(nome)+BindGlobalDyn, e
  bind_global_dyn RIBINDA il simbolo lungo la catena dei bridge di scope —
  Frame::bridge_caller — perché in Zend includer e incluso condividono UNA
  symbol table);
  **(e) shutdown functions coi globali VIVI** (`Ret` del main parcheggia il
  frame in `Vm::retired_main` invece di droppare gli slot — che SONO le
  variabili globali — e run_shutdown_functions lo reinstalla; prima
  register_shutdown_function leggeva NULL da ogni global: p36);
  **(f) classi condizionali di unità esterne non più registrate eagerly**
  (drive_unit: una classe del SEED non ancora dichiarata veniva ri-appesa e
  registrata da QUALSIASI include annidato, flippando il guard
  `if (!class_exists(...))` esterno — pomo/translations.php via mo.php
  perdeva Gettext_Translations; ora remap identità sul prefisso seed) **+
  ri-dichiarazione da file re-inclusi** (nome nel prefisso seed a livello
  statement → si ri-abbassa la dichiarazione, non si sopprime: bug63741);
  **(g) variabili NUOVE definite da eval/include pubblicate nello scope del
  chiamante** (bridge con cella fresca + publish in dyn_vars solo se
  DEFINITE, e get_defined_vars include dyn_vars: il
  `eval(get_wp_config_code()); foreach (get_defined_vars() ...)` di wp-cli
  perdeva `$table_prefix` → tabelle senza prefisso `wp_` → il lexer del
  plugin SQLite trattava `options` come keyword MySQL);
  **(h) `Pdo\Sqlite::createFunction` / `PDO::sqliteCreateFunction`**
  (UDF PHP dentro sqlite via puntatore di re-entry ACTIVE_VM thread-local,
  pattern php-src; connection estratta da Vm.pdo_conns durante la query;
  eccezione del callback ri-propagata originale via slot UDF_ERROR; il
  plugin SQLite di WordPress ne registra ~45 — deprecation 8.5 sul metodo
  BC compresa);
  **(i) semantica execute/bind pdo_sqlite ri-pinnata all'oracle 8.5**
  (placeholder NON bindati = NULL senza errore — execute(array()) su
  pragma_table_info(:table_name) è legale; bind di nome/posizione IGNOTI =
  SQLITE_RANGE 25; PRIMA execute(array) con size≠pc errava sempre);
  **(j) operatore `namespace\` nei nomi qualificati** (resolve_qualified:
  primo segmento `namespace` → namespace corrente; il
  `namespace\strip_tags()` di utils-wp.php componeva
  "WP_CLI\Utils\namespace\strip_tags");
  **(k) pattern PCRE che MISCHIANO gruppi nominati e backreference numerati**
  (fancy-regex e oniguruma li rifiutano: demix_numbered_backrefs assegna
  nomi sintetici `__phprbgN` ai gruppi target e riscrive `\N`→`\k<...>`,
  capture_names() li nasconde; il FILE_DIR_PATTERN di wp-cli Path
  restituiva stringa vuota → wp-config MAI eseguito);
  **(l) `str_replace`/`str_ireplace` col 4° parametro by-ref `&$count`**
  (in HOST_OUT a indice 3, solo quando l'argomento è presente — il
  percorso registry resta per le chiamate a 3 argomenti, ora
  memmem-accelerato; `_deep_replace` di WordPress fa
  `while ($count) { str_replace(..., $count) }` → loop infinito in
  esc_url a WP_Sitemaps init);
  **(m) `timezone_identifiers_list()`** (alias prelude di
  DateTimeZone::listIdentifiers; sblocca populate_options — il group-filter
  e i nomi BC restano non modellati: timezones-list.phpt/bug46111.phpt sono
  fail "ex-skip" documentati, prima il runner li saltava per builtin
  assente).
  Divergenze residue note: `user list` ecc. ~20s vs 0.3s oracle (costo
  lowering/compile per-include, quadratico sul seed — prossimo lavoro perf);
  attribuzione file/riga dei Warning dentro unità incluse a volte spostata
  (visto su a.php:5 vs b.php:4 in p34 pre-fix e su prelude:1465);
  log_errors CLI su stderr non modellato (niente riga "PHP Warning:"
  duplicata, solo display_errors).
- 2026-07-14 (sessione WordPress-1): 🏁 **wp-cli da sorgente gira end-to-end**
  (`wp --info` / `wp cli version` a parità con l'oracle, modulo campi
  ambiente-dipendenti: PHP binary=phpr, memory_limit=-1 senza php.ini).
  Fix engine, tutti oracle-pinned:
  **(a) `global $$x` / `global ${expr}`** (StmtKind::Global → Vec<GlobalItem>
  Static/Dyn; nuovo Op::BindGlobalDyn: resolve-or-create della cella globale
  per NOME runtime via global_slot_by_name — creata NULL come lo
  zend_hash_add del global fetch, appare in $GLOBALS anche senza assign — e
  alias nello slot named o in Frame::dyn_vars; wp-config import di wp-cli);
  **(b) compound assign su variable-variable** (`$$n .= r`: desugar
  read-op-write col NOME materializzato UNA volta in un temp; `??=` resta
  assente, correct-or-absent);
  **(c) SEND_VAR_EX per chiamate a funzione non risolte al compile time**
  (CallValue/CallNsFallback e `$f(...)` dinamico ora passano gli argomenti
  via push_dyn_args — PushRef/ArgPlace — e invoke_named/push_closure_frame
  materializzano contro la by-ref mask del callee risolto; PRIMA un
  `&$param` di una funzione cross-unit — `Utils\proc_open_compat(&$pipes)`
  di wp-cli — riceveva una COPIA. Residuo documentato: argomento non-place
  a param by-ref resta by-value silenzioso dove Zend darebbe Error);
  **(d) coercizione Stringable negli argomenti string dei builtin puri**
  (~30 nomi aggiunti alla value_builtin_string_coerces + swap
  convert::to_zstr→ctx.to_zstr in string/url/crypto/encoding.rs: substr su
  DirectoryIterator, md5, trim, strpos, urlencode, …);
  **(e) DirectoryIterator::__toString = getFilename()** (override Zend; via
  SplFileInfo ereditato dava il PATHNAME) e **ordine readdir** per
  DirectoryIterator/FilesystemIterator/RecursiveDirectoryIterator
  (scandir(SCANDIR_SORT_NONE), byte-id con l'oracle su APFS);
  **(f) `$argv`/`$argc` registrati nel global registry cross-unit**
  (Zend CLI li mette nel global symbol table SEMPRE con
  register_argc_argv=On; prima erano seminati solo se l'unità MAIN li
  menzionava — wp-cli li legge da un file required e perdeva TUTTI gli
  argomenti, cadendo su `help`, il cui pager via proc_open causava il hang).
- 2026-07-14 (sessione 8): ✅ **CHIUSA la suite symfony/http-kernel: 1663
  test, 0E/0F** (da 0E/25F). Fix engine, tutti oracle-pinned:
  **(a) visibilità del costruttore a `new`** (`check_new_ctor_access` nei 3
  Op::Alloc*: "Call to private C::__construct() from {scope}" senza la parola
  "method", classe DICHIARANTE nel messaggio, abstract/interface/enum vincono;
  le allocazioni interne — unserialize/reflection/host — NON checkano, come
  object_init; `ReflectionClass::newInstance{,Args}` → ReflectionException
  "Access to non-public constructor of class X");
  **(b) is_callable ZPP completo**: static-style (`'C::m'`/`['C','m']`) su
  metodo d'istanza VISIBILE → false senza fallback a `__callStatic` (metodo
  inaccessibile/mancante → callable sse `__callStatic` esiste); param
  `$syntax_only` (shape-only: ogni stringa, closure, array `[0=>str|obj,
  1=>str]`); 3° param by-ref `&$callable_name` via out-param table
  (zend_get_callable_name: "C::m", "Class::__invoke", `{closure:file:line}`,
  cast scalare, "Array" per array malformato);
  **(c) FILTER_VALIDATE_REGEXP** (php_filter_validate_regexp: miss →
  false/NULL_ON_FAILURE, "regexp" mancante → ValueError);
  **(d) weak coercion int con range check** (zend_parse_arg_long_weak:
  numeric-string/float FUORI dal range long → TypeError, non troncamento —
  prima `'9223372036854775808'` → int param troncava con Deprecated);
  **(e) enum from/tryFrom = port dello ZPP di zend_enum_from_base**
  (int-backed: Z_PARAM_LONG con coercion weak completa; string-backed:
  STR_OR_LONG weak / STR strict del CHIAMANTE; null → deprecation
  "Passing null … of type string|int" + valore zero; messaggi "int" per
  int-backed, "string|int" per string-backed);
  **(f) date_object_compare in ops.rs**: DateTime/DateTimeImmutable
  confrontano per ISTANTE assoluto (epoch+µs, cross-class e cross-timezone)
  in compare()/loose_eq; identità `Rc::ptr_eq` → 0 nel ramo ordering; NUOVO
  arm loose_eq (Array,Array) con uguaglianza LOOSE dei valori (prima due
  array contenenti la STESSA istanza oggetto non erano mai `==`);
  **(g) flock(2) REALE sui file stream** (due handle sullo stesso file
  contendono anche in-process — Store di HttpCache; LOCK_NB miss → false +
  `$would_block=1`; backend non-file → false come PHP);
  **(h) INI `error_log` onorata da error_log()** (append
  "[d-M-Y H:i:s TZ] msg" nel default tz; ⚠️ le DIAGNOSTICHE engine
  (Warning/Deprecated) vanno ancora a stderr anche con error_log settata);
  **(i) attributi sulle INTERFACCE riflessi** (lower_interface li scartava);
  **(j) ctor di Exception/Error scrive message/code/previous SOLO se
  forniti/non-zero** (zend_exceptions.c: una sottoclasse che ridichiara i
  default li mantiene — `protected $code = 'non-integer-code'`);
  **(k) ⭐ SWEEP DISTRUTTORI EAGER OVUNQUE**: Op::Sweep emesso dopo OGNI
  statement in OGNI body (prima: solo top-level) — Zend distrugge a
  refcount-zero; i configurator Symfony DI registrano le definizioni in
  `__destruct` e la riga dopo le legge (granularità statement, non
  sub-espressione: resta teoricamente osservabile in `f(new Temp(), g())`).
  Divergenze RESIDUE nuove documentate: ordine di `class_implements`
  (phpr: ordine di dichiarazione; Zend: ordine interno diverso);
  attribuzione di LINEA delle deprecation di coercion implicita (Zend: riga
  del RECV nel callee; phpr: riga della chiamata) e della deprecation
  null-arg flushata durante l'unwinding (+1 riga);
  `ReflectionProperty::getValue` su typed prop non inizializzata non lancia
  (oracle: Error "must not be accessed before initialization"); i frame
  builtin non compaiono nei backtrace dei fatal (`#0 {main}` vs
  `#0 file(line): fn()`); stderr non riceve la copia "PHP Fatal error:"/
  "PHP Deprecated:" dei diagnostici (solo stdout).
- 2026-07-13 (sessione 7): ✅ **CHIUSO il gap timezone D-DT3** — phpr non è
  più UTC-only: lettore TZif v2/v3 sopra `/usr/share/zoneinfo` di sistema
  (`php_types::tz`, cache per zona, gap/fold DST risolti alla timelib:
  offset PRE-transizione in entrambi i casi, pinnato con l'oracle su
  America/Toronto 2026), default timezone di processo reale
  (`date_default_timezone_set/get`, INI `date.timezone` con propagazione da
  `-d`/`ini_set`, Notice "Timezone ID '%s' is invalid" su ID sconosciuto),
  `date`/`idate`/`strftime`/`mktime`/`getdate`/`localtime`/`strtotime` nel
  default tz (relative wall-clock preserving attraverso i salti DST),
  DateTime/DateTimeImmutable zone-aware (ctor `__strtotime_tz` con priorità
  zona-nella-stringa > argomento > default; `format` con O/P/T/e/I/Z/c/r
  dall'istanza; `setDate`/`setTime`/`add`/`sub`/`modify` con aritmetica wall
  re-ancorata; `diff` sui tempi LOCALI dei due lati; `getOffset` +
  `DateTimeZone::getOffset` + normalizzazione label offset "+0500"→"+05:00").
  **Residui divergenti**: (a) epoch oltre l'ultima transizione memorizzata
  (≥2037 per le zone DST) usano l'ultimo tipo della tabella — il footer
  POSIX TZ non è valutato; (b) le stringhe datetime accettano solo
  UTC/GMT/Z/±offset come zona inline — i nomi IANA e le abbreviazioni
  ("EST", "America/Toronto") dentro le stringhe restano parse failure;
  (c) `new DateTimeZone('Bogus')` non lancia DateInvalidTimeZoneException
  (nessuna validazione nel ctor); (d) `timezone_identifiers_list`/
  `DateTimeZone::listIdentifiers` resta la lista statica; (e) il default di
  `__date_from_format` per i campi mancanti usa il "now" UTC, non locale.
- 2026-07-13 (sessione 6): **eval() condivide lo scope del chiamante** come
  include (prima: unit isolata → il `new class($initializer)` di
  ContainerBuilder::createService riceveva null); **nomi sintetici dei closure
  in formato PHP 8.4** `{closure:Scope():line}` (scope = `Class::method()` /
  `func()` / nome del closure racchiudente verbatim / file a top-level;
  `__FUNCTION__`/`__METHOD__` nel corpo = quel nome — residuo: le unit eval
  si chiamano ancora `eval()'d code`, Zend usa `file(line) : eval()'d code`);
  **Closure::fromCallable/first-class-callable su metodi magici** crea il
  trampolino `__call`/`__callStatic` (ReflectionFunction: nome bare, 0 param,
  no file — residuo: il messaggio d'errore per callable invalidi resta
  "is not callable", Zend usa "Failed to create closure from callable: …");
  **unset() su readonly** segue il write-path Zend (permesso su prop NON
  inizializzata dallo scope set-visibility — pattern lazy-ghost LazyClosure —
  messaggi "Cannot unset …" derivati da readonly_write_error); **isset/empty
  con Index annidati su ArrayAccess** dispatchano offsetExists/offsetGet
  sugli intermedi (BP_VAR_IS quiet fetch, dim-path e field-path) e **`?? `
  su ArrayAccess** dispatcha il protocollo (VarDumper Data sbloccato:
  LoggerDataCollector/RequestDataCollector interi file verdi); out-param
  `flock(&$wouldBlock)` (sempre 0) e `preg_replace_callback(&$count)` +
  `$limit` implementato; `ReflectionFunction::getAttributes()` sui closure
  method-backed (closure_func_mod risolve `Class::method`).
- 2026-07-13 (sessione 5, batch 2): **le call non qualificate in namespace il
  cui nome è un builtin ora compilano a `Op::CallNsFallback`** (prima:
  `Op::CallBuiltin` diretto, che rendeva INERTE lo shadowing runtime — le
  `sleep()`/`time()` eval-dichiarate da ClockMock del phpunit-bridge non
  intercettavano mai: la suite http-kernel dormiva ~400s REALI nei test
  stale-if-error). Direct-bind resta solo nel namespace globale (lì la
  ridichiarazione è fatal). RESIDUO: i builtin **RefFirst** (sort…) e i
  **host builtin** (fopen, is_callable…) chiamati non qualificati in ns
  restano direct-bind (shadowing userland non visto).
  **`unset()` su typed prop dichiarata = slot Undef + flag `typed_unset`**
  (modello Zend: slot resta UNDEF ma il flag IS_PROP_UNINIT viene azzerato):
  var_dump/reflection continuano a mostrare `uninitialized` (il primo
  tentativo a slot-RIMOSSO regrediva lazy_objects/unset_* e
  readonly_clone_success2 — beccato dal gate corpus per NOME), ma
  `magic_applies` tratta Undef+flag come ASSENTE → lettura → `__get` se
  esiste (l'idioma lazy di symfony Constraint::$groups), altrimenti lo stesso
  Error before-init; never-initialized (Undef senza flag) = Error ANCHE con
  `__get`. Matrice oracle-pinnata in p_b2.php (guard ricorsivo,
  isset/isInitialized→false). RESIDUO: `__set` dopo unset su typed prop non
  scatta (la write torna diretta); su prop UNTYPED il giro __set funziona già
  (entry rimossa).
  **`ReflectionFunction::isAnonymous()`** ora true solo per `{closure*}` (una
  FCC method-backed riporta false → ArgumentMetadataFactory::getPrettyName e
  ControllerResolver::checkController risolvono la classe).
  **`is_callable`/hint `callable`**: fallback `__call`/`__callStatic` per
  metodo mancante + check di VISIBILITÀ dallo scope chiamante
  (method_visible_from; prima un metodo private/protected risultava callable
  da fuori). **`array_pop`** decrementa nNextFreeElement quando poppa l'ultimo
  auto-index (array.c:3579; pop+append riusa la chiave — RequestStack
  push/pop/push di HttpKernel). **`intval($s, $base)`** onora base ≠ 10
  (strtol + ramo "0b" di type.c). **include/eval ereditano `$this` e lo scope
  di classe** (drive_unit propaga this/class/static_class — i template
  .html.php di HtmlErrorRenderer usano `$this->` e chiamano metodi private).
  **`DateTime(Immutable)::getLastErrors()`**: contratto minimo — stato
  thread-local aggiornato SOLO da createFromFormat (il ctor testuale non lo
  tocca — divergenza dichiarata), false quando pulito (PHP 8.2+), warning
  "The parsed date/time was invalid" su overflow normalizzati, messaggi di
  errore = sottoinsieme generico (niente wording per-specifier timelib);
  `file_get_contents`/file.rs ora castano via `ctx.to_zstr` (stringify).
- 2026-07-13 (sessione 5, batch 1): **`Dom\` API nuova (PHP 8.4) — subset
  crawler-oriented**: `Dom\HTMLDocument::createFromString` + gerarchia
  Node/CharacterData/Text/CDATASection/Comment/ProcessingInstruction/Element/
  Attr/Document/DocumentType/NodeList/NamedNodeMap (prelude_ns.php) sopra un
  **parser HTML5-lite host-side** (`DomDoc::parse_html`, vm/dom.rs): struttura
  html/head/body implicita, void elements, rawtext (script/style) + RCDATA
  (title/textarea), commenti/bogus comments/doctype, auto-close `<p>`/li/td…,
  entità numeriche + core named (amp/lt/gt/quot/apos/nbsp), sniff `<meta
  charset>` (label WHATWG famiglia Latin-1 → windows-1252/ISO-8859-15,
  transcodifica a UTF-8). SCOPE-OUT dichiarati: adoption agency (formatting
  reconstruction), table fostering, template contents, tabella completa named
  entities (~2200), niente warning "tree error …" del parser lexbor (sotto la
  Crawler sono comunque soppressi da libxml_use_internal_errors), `tagName`
  sempre lowercase (= comportamento HTML_NO_DEFAULT_NS, l'unico flusso
  esercitato). Costante `Dom\HTML_NO_DEFAULT_NS` seminata host-side (il
  top-level del prelude non esegue). Probe p_dom1 byte-id vs oracle (tranne i
  warning lexbor).
  Inoltre: `is_uploaded_file` (sempre false su CLI, onesto);
  `ReflectionMethod::isClosure()` → false; strtotime/DateTime **relnumber
  timelib** `[+-]*[ \t]*[0-9]+` (segni staccati dalle cifre: "+ 1 hour",
  "--2 hours"; ext/date +3: bug35456/40861/73858); upload_max_filesize "2M" +
  post_max_size "8M" in tabella INI (default `php -n`); **arg path dei builtin
  file (arg_os_path/os_path_at) via `ctx.to_zstr`** + famiglia path in
  `value_builtin_string_coerces`: `rename($src, $splFileInfo)` e
  `file_get_contents($splFileInfo)` ora guidano `__toString` precomputato
  (prima: warning + path fantasma dal nome classe — rename creava file spurii
  nel cwd, probabile fonte degli artifact 32-hex nel root del repo).
- 2026-07-13 (sessione 4): ✅ CHIUSO il gap SEND_VAR_EX (repro p_ref3):
  gli argomenti *place* (`$a['k']`, `$_SESSION[$k]`, `$this->p`, `$o->p['k']`)
  verso callee risolti solo a runtime (receiver dinamico, `$cls::m()`,
  `$obj->$m()`, `new` dinamico) ora viaggiano come **`Zval::ArgPlace`
  differito** (`Op::PushArgPlace`: base + steps Index/Prop + chiavi valutate):
  ogni funnel di dispatch (`method_call`, `dispatch_static_call`,
  `Op::InvokeCtor`, ramo Fiber::suspend) li materializza contro la maschera
  by-ref del callee risolto — by-ref → W-fetch via `make_ref_cell` (estratto
  dal handler `Op::MakeRef`: alias, chiave mancante creata silente); by-value
  → R-fetch fedele (`arg_place_read`: warning "Undefined variable"/"Undefined
  array key" flushati con la LINEA della call, `offsetGet` sync per
  ArrayAccess, `prop_read_sync` con hook/`__get` guidati inline via
  `drive_to_return`). Inoltre i **costruttori** onorano i param by-ref:
  ctor noto a compile time → `push_call_args` con maschera (prima SEMPRE
  `push_value_args`); ctor dinamico → `push_dyn_args` + materializzazione in
  `InvokeCtor`. Sblocca la catena Symfony Session (SessionBagProxy ctor
  `array &$data` + `$bag->initialize($session[$key])`): p_sl1/p_sl2 BYTE-ID.
  ⚠️ Residui: (a) place con step **PropDyn** (`->$n`) o base call-result non
  differiti (restano by-value); (b) manca l'Error runtime "Argument #N ($p)
  could not be passed by reference" per un NON-place (es. `$x ?? []`) passato
  a un param by-ref di callee dinamico — phpr passa il valore silenziosamente;
  (c) i warning R-fetch fuoriescono al BIND (linea della call, corretta) ma
  DOPO la valutazione degli argomenti successivi — l'ordine relativo a side
  effect di altri argomenti può divergere in casi patologici.
- 2026-07-13 (sessione 3, batch 3): SessionState.committing — il prelude
  \SessionHandler opera DURANTE sess_commit (la sessione conta già chiusa per
  bug60634, ma la guardia PHP è "nessun handler aperto", non status==active:
  SessionHandlerProxy di Symfony chiamava write/close → "Session is not
  active"). headers_sent(&$file, &$line): out-param CABLATI (tabella
  host_builtin_out_param, secondo out come exec) — prima gli argomenti
  venivano LETTI come valori (warning "Undefined variable" che, stampando,
  rendevano headers_sent=true da sé). ⚠️ GAP ENGINE documentato (repro
  p_ref3): **SEND_VAR_EX solo per variabili semplici** — un ELEMENTO di
  array/prop passato a un metodo con receiver dinamico è pushato per VALORE,
  quindi un param by-ref non aliasa (`$bag->initialize($session[$key])` dei
  bag Symfony: unica failure residua di SessionListenerTest). Zend risolve
  col fetch FUNC_ARG deciso a runtime; phpr richiede un descriptor di place
  differito nel binder. I casi con receiver STATICAMENTE noto e le funzioni
  libere già funzionano (push_call_args ha il ramo MakeRef).
- 2026-07-13 (sessione 3, batch 2): trait_exists() su un nome già dichiarato
  come classe/interfaccia → false SENZA ri-innescare l'autoloader (speculare
  al fix trait: PhpDumper sonda trait_exists(HttpKernelInterface::class) e il
  re-include collideva). preg_replace(): il 4° argomento $limit era IGNORATO
  (sempre replace-all) — il PhpDumper pota il template del container con
  limit:1 e la seconda rimozione produceva PHP corrotto ("expected Class");
  ora Engine::replacen per-pattern-per-subject, &$count coerente, 0=nessuna,
  -1=tutte. FilesystemIterator: era uno stub di sole costanti → implementata
  (extends DirectoryIterator; SKIP_DOTS onorato come FLAG: il default 4096 li
  salta, flags espliciti senza SKIP_DOTS mostrano `.`/`..` — oracle-pinned;
  CURRENT_AS_PATHNAME/SELF/FILEINFO, KEY_AS_FILENAME, get/setFlags, seek).
  ⚠️ GAP ENGINE emerso: **shadowing di METODI privati** — da scope X,
  `$this->m()` con `m` privato in X deve chiamare X::m anche se il receiver
  è una sottoclasse che ridefinisce `m` privato (le PROP hanno già lo
  storage-key fix; i metodi no: phpr risolve sul receiver e lancia
  visibility error). Workaround nel prelude: helper privati con nomi
  per-classe (__dicur/__disync in DirectoryIterator). Da fixare in
  resolve_method_runtime.
- 2026-07-13 (sessione 3): autoload dei nomi di trait allineato alla class
  table unica di Zend (un trait dichiarato non ri-innesca MAI l'autoloader da
  class_exists/interface_exists/ReflectionClass — prima il re-include
  collideva con le altre dichiarazioni del file, es. PriorityTaggedServiceUtil);
  trait_exists($n, true) ora passa il nome case-preserved a PSR-4;
  ReflectionClass su trait: getFileName reale, getName canonico —
  getStartLine/getEndLine però = span dei METODI (approssimazione; l'oracle
  riporta la riga di `trait`/`}`), e un trait senza metodi resta
  getFileName=false. SplPriorityQueue nel prelude (sift di spl_heap.c
  replicato: ordine pareggi byte-id; var_dump mostra le prop interne phpr,
  non la shape di ext/spl). FIX ENGINE: il dispatch dei metodi con NAMED
  ARGS usava il modulo corrente invece di class_mod → MakeClosure/Op::Call
  risolvevano nell'unit sbagliata (i data provider PHPUnit chiamano i test
  method con named args). hash(): aggiunti crc32 (BZIP2, digest LSB-first
  come ext/hash) e crc32c (Castagnoli), oracle-pinned.
- 2026-07-13: §3.3-bis — class_* su nomi di trait (class_uses(trait) → `[]`,
  residuo trait-di-trait); nota class_implements(enum) da verificare.
- 2026-07-09: creazione. Catalogati i 6 gap trasversali builtin, le assenze
  consapevoli Tier-A, le divergenze di engine circoscritte, gli invarianti
  byte-identici.
