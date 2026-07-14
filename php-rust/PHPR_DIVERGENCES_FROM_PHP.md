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
> Ultimo aggiornamento: 2026-07-09 (Sessione VII stdlib Tier-A).

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
