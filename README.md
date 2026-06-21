# php-rust — reimplementazione moderna di PHP in Rust

Esperimento di traduzione **PHP 8.5.7 (C) → Rust** guidato dal comportamento
osservabile, non dall'architettura interna dello Zend Engine.

> **Principio guida**: il contratto da preservare non è il design di Zend (1999–2004)
> ma l'**output osservabile** di PHP. L'oracle esiste già: i ~21.500 test `.phpt` del
> sorgente ufficiale. Qualunque runtime che produce lo stesso output *è* PHP.
> Questo trasforma il lavoro da *traduzione del C* a *reimplementazione guidata dalla
> spec*, dove il C si legge solo per inchiodare la semantica.

Metodologia: skill `legacy-port`, adattata (Strategia A adapter per il front-end +
full port semantico del solo `zend_operators.c`).

## Stato attuale

**Steps 0–57 completati · 918 test verdi · clippy pulito · differential 37.835 casi a 0 mismatch.**

Step 57 ha aggiunto il secondo batch di funzioni stringa pure (`strrpos`/`stripos`/`strripos`,
`strspn`/`strcspn`, `strtr` byte-map + array, `chunk_split`, `strip_tags`, `quotemeta`,
`levenshtein`): sulla copia pulita di `ext/standard/tests/strings` il pass-rate sul runnable
sale a **58.0% (228/393)** con `--isolate` (il run in-process aborta su un crash *pre-esistente*
di `sprintf` con la sintassi `*`, vedi `diary/04-divergences.md`). Il corpus ha fatto trovare e
fixare 1 bug di fedeltà (`strtr("", $map)` non deve emettere il Warning chiave-vuota).

Step 56 ha aggiunto un batch di funzioni stringa pure (`bin2hex`/`hex2bin`, `addslashes`/
`stripslashes`, `substr_replace`, `nl2br`, `wordwrap`, `htmlspecialchars`/`htmlentities`
+ decode, `vsprintf`/`vprintf`): sulla copia pulita di `ext/standard/tests/strings` il
pass-rate sul runnable è **51% (143/280)** alla prima sweep.

Step 55 ha aggiunto un batch di builtin stream/file read (`file`, `readfile`, `fpassthru`,
`stream_get_contents`, `stream_copy_to_stream`, `ftruncate`) + `getenv`/`putenv` +
`disk_free_space`/`disk_total_space`: sulla copia pulita di `ext/standard/tests/file` i
pass salgono **71 → 86** (skip −26).

Step 54 ha aggiunto due engine di parsing: **scanf** (`sscanf`/`fscanf`, con modo
return-array e modo by-reference) e **CSV** (`str_getcsv`/`fgetcsv`/`fputcsv`), eliminando
il bucket di skip "missing builtin: fscanf/fgetcsv/fputcsv" su `ext/standard/tests/file`.

Step 52 ha aggiunto il sottosistema filesystem (predicati `file_exists`/`is_*`/`filetype`,
famiglia `stat`/`lstat`/`fstat` + accessor, mutatori `unlink`/`mkdir`/`rename`/`copy`/`touch`/
`symlink`/`chmod`/…, `scandir`/`glob`/`tempnam`/`tmpfile`): sul corpus `ext/standard/tests/file`
i pass salgono **2 → 63**. Step 53 ha aggiunto `strstr`/`strrchr`/`stristr`,
`get_resource_type`, la famiglia `opendir`/`readdir`/`closedir`/`rewinddir` e
`fprintf`/`vfprintf`, e ha corretto un panic latente (dir handle in un builtin di stream).

> Hardening tooling (non-funzionale): depth-guard nell'evaluator (`MAX_CALL_DEPTH`,
> converte la ricorsione runaway in un `Error` catchable invece di un SIGABRT del
> processo) + modalità `phpt-runner --isolate` (ogni test in un sotto-processo: un
> crash è contenuto come un FAIL, non aborta il batch). Oracle ricompilato con
> `--enable-mbstring` → sblocca la validazione di `mb_*`.

| Step | Contenuto | Stato |
|---|---|---|
| 0 | Scaffolding workspace + diary + Phase 0 reconnaissance | ✅ |
| 1 | `php-types`: `PhpStr`, `Zval`, `PhpArray` | ✅ |
| 2 | Operatori + conversioni (`zend_operators.c`) + **differential 37.835 casi, 0 mismatch** | ✅ |
| 3 | Bridge mago → HIR | ✅ |
| 4 | Evaluator (echo, variabili, controllo di flusso) | ✅ |
| 5 | Builtins nucleo + `var_dump` | ✅ |
| 7 | Array end-to-end + `foreach` / `switch` / `match` | ✅ |
| 6 | `phpt-runner` (capability scan + import testsuite) — **6172 file, 98.6% dei runnable** | ✅ |
| 8 | Funzioni utente | ✅ |
| 9 | Rendering diagnostici (warning/fatal su stdout) | ✅ |
| 10 | Espansione builtin (count, array_*, implode/explode, substr/strpos/str_replace, sprintf/printf, abs/max/min, print_r) — **baseline 126 → 135 pass** | ✅ |
| 11 | Reference semantics — `$b = &$a` (11a), parametri `f(&$x)` (11b), builtin by-ref `array_push`/`sort`/`array_pop`/`array_shift` (11c), element-ref + `foreach as &$v` via `Zval::Ref` (11d) | ✅ |
| 12 | `global $x` + `$GLOBALS['literal']` (frame overlay globale/locale) | ✅ |
| 13 | Return-by-reference `function &f()` | ✅ |
| 14 | Type-hint enforcement scalare *weak* (+ `TypeError` byte-esatto) | ✅ |
| 15 | Variabili `static` (persistenza cross-call) | ✅ |
| 16 | `declare(strict_types=1)` | ✅ |
| 17 | Espansione builtin per frequenza (case/build/trim/math/array — ~24 funzioni) | ✅ |
| 18 | Closures & callables — `use`, arrow `fn`, first-class `f(...)`, `array_map`/`filter`/`usort`, costanti engine | ✅ |
| 19 | OOP/classi — `new`, `$this`, ereditarietà, visibility, static + LSB, interfacce, abstract, `instanceof`, `__toString`, closure bind, `var_dump`/`print_r` oggetti | ✅ |
| 20 | Eccezioni — `try`/`catch`/`finally`/`throw` + gerarchia `Throwable` (prelude PHP) | ✅ |
| 21 | Traits (flatten-at-lowering, `insteadof`/`as`, collisioni → Fatal) | ✅ |
| 22 | Magic methods — `__get`/`__set`/`__isset`/`__unset`/`__call`/`__callStatic`/`__invoke` | ✅ |
| 23 | Enum (pure + backed) — case singleton, `from`/`tryFrom`/`cases`, `UnitEnum`/`BackedEnum` | ✅ |
| 24 | `Stringable` auto-interface + `__destruct` (shutdown LIFO + sweep refcount-zero immediato) | ✅ |
| 25 | Interpolazione stringhe — `"$x"`, `"$a[k]"`, `"$o->p"`, `"{$expr}"` | ✅ |
| 26 | `json_encode` / `json_decode` (assoc array + `stdClass`, flag PRETTY/UNESCAPED_*) | ✅ |
| 27 | Regex `preg_*` — `match`/`match_all`/`replace`/`replace_callback`/`split`/`quote` (crate `regex`) | ✅ |
| 28 | Stack-trace frame reali — `getTrace`/`getTraceAsString` + render uncaught con frame | ✅ |
| 29 | Builtin puri string+array + cast `(object)` | ✅ |
| 30 | Heredoc / nowdoc (`lower_document`) | ✅ |
| 31 | `preg_*` named groups + flag `PREG_*` (OFFSET_CAPTURE, SET_ORDER, UNMATCHED_AS_NULL, SPLIT_*) | ✅ |
| 32 | Array by-ref family — `array_splice` + `array_walk` | ✅ |
| 33 | Array key/assoc set-ops (`array_diff_key`/`assoc`, `array_intersect_key`/`assoc`) + `array_column` | ✅ |
| 34 | **DateTime/date()** — `date`/`gmdate`/`mktime`/`checkdate`/`strtotime` (subset) + OOP `DateTime`/`DateTimeImmutable`/`DateInterval` (`format`/`modify`/`add`/`sub`/`diff`/`createFromFormat`), scope UTC (crate `time`) | ✅ |
| 35 | **API procedurale date** — `date_create`/`date_format`/`date_diff`/`date_add`/`date_sub`/`date_modify`/`date_*_set`/`date_create_from_format`/`date_interval_format`/`date_interval_create_from_date_string` (funzioni globali del prelude) + `getdate`/`localtime` (builtin puri). Infra: il prelude ora trasporta anche le funzioni globali | ✅ |
| 36 | **preg backref/lookaround** — auto-fallback `regex`→`fancy-regex` (`enum Engine`): backref, lookaround, atomic/possessive, `(?R)`/conditional/`\K`/`\G`. Scope-out: subroutine `(?1)`, control verb `(*SKIP)`, callout. Hardening 36-3: `backtrack_limit` + stop-on-error (niente hang/panic su pattern patologici). Corpus `ext/pcre` 38→41 pass | ✅ |
| 37 | **flag modificatori PCRE** `U` (ungreedy, `swap_greed`+`(?U)`), `A` (anchored, wrap `\A(?:…)`), `X` (no-op PCRE2), `D`/`$` (default `$` zero-width prima di `\n` finale via lookahead `(?=\n?\z)`→fancy; `D`=`\z` stretto). Corpus `ext/pcre` 41→44 pass | ✅ |
| 38 | **argomenti nominati** `f(c: 3, a: 1)` per funzioni/costruttori/metodi/static (riordino, default saltati, named→by-ref, errori catchable, posizionale-dopo-nominato compile-fatal) + **parametri variadic** `f(...$rest)`. `nullsafe ?->` già dallo step 19. Follow-up: spread `...$arr`, named→variadic | ✅ |
| 39 | **generatori `yield`** — esecuzione sospendibile via coroutine stackful `corosensei` (D-GEN-1). `yield`/`yield $k=>$v`/`yield;`/`yield from` (array+sub-generatore), `send()`, `return`+`getReturn()`, Iterator (current/key/next/valid/rewind), `foreach` su Generator, instanceof Generator/Iterator/Traversable, var_dump. Closure-generator. Corpus `Zend/tests/generators` 59/110. Scope-out (D-GEN-4): `throw()`, eccezioni/finally attraverso yield, yield by-ref | ✅ |
| 40 | **argument unpacking / spread** `f(...$arr)` per Call/New/MethodCall/StaticCall: chiavi int→posizionali (valore chiave ignorato), chiavi string→nominati, spread→variadic (re-keyed), Traversable/generator, `TypeError` su non-iterabile, compile-fatal posizionale-dopo-spread / spread-dopo-nominato. **named→variadic** (`...$rest` raccoglie i nominati senza match con chiave string, esplicita e da spread). Scope-out D-40.1: precedenza messaggio su input doppiamente-invalido | ✅ |
| 41 | **mbstring batch 1** (UTF-8 code-point) — `mb_strlen`/`mb_substr`/`mb_str_split`, case (`mb_strtoupper`/`mb_strtolower`/`mb_convert_case`/`mb_ucfirst`/`mb_lcfirst`, full Unicode via std), ricerca (`mb_strpos`/`stripos`/`strrpos`/`strripos`/`mb_strstr`/`stristr`/`strrchr`/`strrichr`/`mb_substr_count`), `mb_ord`/`mb_chr`/`mb_str_pad`/`mb_trim`/`ltrim`/`rtrim`/`mb_check_encoding`. Builtin puri. Scope-out: encoding non-UTF-8 (serve `encoding_rs`), `mb_ereg*`, `mb_convert_encoding`/`detect`/`strwidth` | ✅ |
| 42 | **mbstring batch 2A** (encoding + width) — `mb_convert_encoding`/`mb_detect_encoding` via `encoding_rs` (UTF-8↔ISO-8859-1/Windows-1252/SJIS/EUC-JP/UTF-16; true Latin-1 e UTF-16 hand-rolled, substitute `?`); `mb_strwidth`/`mb_strimwidth`/`mb_strcut` via tabella EAW portata da libmbfl. Builtin puri. Scope-out: `mb_ereg*`/`mb_split` (oniguruma → step 43), `mb_list_encodings`, width su encoding ≠ UTF-8 | ✅ |
| 43 | **mbstring batch 2B** (regex `mb_ereg*`) — adapter su **oniguruma reale** (crate `onig`): `mb_ereg`/`mb_eregi` (`$regs` by-ref), `mb_ereg_replace`/`mb_eregi_replace`/`mb_ereg_replace_callback`, `mb_split`, `mb_ereg_match`, `mb_regex_encoding`/`set_options`, e famiglia stateful `mb_ereg_search_*`. Default Ruby syntax + opzioni `pr` (classi POSIX, named group, backref). Primo step con stato persistente sull'`Evaluator` + higher-order builtins. Scope-out: encoding ≠ UTF-8 | ✅ |
| 44 | **phpt-runner `--EXTENSIONS--` relax + import corpus mbstring** (Phase 4c) — gating selettivo (allowlist `core/standard/mbstring/pcre/json/date`) sblocca 163 test mbstring-only; run `ext/mbstring/tests` = 30 pass / 37 fail / 350 skip. **3 bug classe A fixati** (offset out-of-range su `mb_str(r)(i)pos`, lista encoding vuota su `mb_detect_encoding`/`mb_convert_encoding`). 37 fail residui = scope-out dichiarati; **2 D-NEW** (array input in `mb_convert_encoding`; titlecase digrammi in `MB_CASE_TITLE`) | ✅ |
| 45 | **`goto` + label** — ultima feature di control-flow. `Flow::Goto` + `exec_stmts` con indice (salto same-block / propagazione out-of-block, incl. uscita da loop/`try`+`finally`); validazione compile-time (undefined / dup label, into-loop/switch, **into-finally**) via stack di barriere. Corpus `Zend/tests/*goto*` = 5 pass / 5 skip (non-goto) / 0 fail. Scope-out **D-45.1**: salto *dentro* un blocco trasparente (raro, mai nel corpus). +2 fix di fedeltà al phpt-runner (strip backtrace con `fatal_error_backtraces=Off`; nome script = path `.php` reale) | ✅ |
| 46 | **`print` + `exit`/`die`** — costrutti di linguaggio. `print` = espressione (emette, ritorna `1`); `exit`/`die` si propagano via `Err(PhpError::Exit(u8))` (uncatchable, **NON** girano i `finally`), nuovo `Outcome.exit_code`. Coercion `string|int $status`: int/bool/float/null → exit code, string/`__toString` → messaggio, array/oggetto non-stringabile → `TypeError`. Sblocca `finally_goto_005` + test `Zend/tests/exit`. Scope-out **D-46.1**: Deprecated notice di coercion non emessi | ✅ |
| 47 | **`var_export` + reflection** — `var_export` (port di `php_var_export_ex`: indent esatto, float con `.0`, stringhe single-quote + NUL via `. "\0" .`, `(object) array`/`__set_state`, modalità return, warning su ref circolari); `get_class_methods`/`get_object_vars` scope-aware (visibilità da `cur_class`, ereditarietà child→parent, metodi d'interfaccia via nuovo `ClassDecl.abstract_methods`). +14 test. Scope-out **D-47.1/2**: visibilità `abstract protected`, aliasing reference di `get_object_vars` | ✅ |
| 48 | **micro-step runner breakdown + dynamic class refs + `@`** — (a) il phpt-runner riporta il costrutto specifico non supportato (`expr:*`/`stmt:*`) e i builtin mancanti (top 20), in-process e `--isolate`; (b) `ClassRef::Dynamic` per `new $cls`/`$cls::m()`/`$cls::CONST`/`$obj::m()`/`$x instanceof $cls` (stringa o oggetto → class id; non-forwarding); (c) operatore `@` via `suppress_depth` (no-op di `flush_diags` + truncate dei diag; throwable NON soppressi). +9 test. Scope-out **D-48.1** | ✅ |

> Lo step 6 è stato eseguito **dopo** lo step 7 (deciso con l'utente: gli array
> rendono il phpt-runner molto più utile, quintuplicando i test in-scope).

**Risultato chiave (step 2)**: il porting di `zend_operators.c` — type juggling,
confronti PHP 8, formattazione float, increment Perl-style, bitwise su stringhe — è
verificato **byte-per-byte** contro un binario PHP 8.5.7 compilato dal sorgente, su
37.835 casi (47 valori × 47 × 17 operatori binari + 6 unari), diagnostica inclusa.

**Risultato chiave (step 6)**: il `phpt-runner` esegue un **capability scan** della
testsuite ufficiale (`tests/` + `Zend/tests/`, 6172 file): fa girare i test in-scope
e categorizza i fuori-scope come SKIP motivati (l'unico FAIL è una divergenza di
output reale). Baseline attuale: **71 pass / 1 fail / 6100 skip = 98.6% dei runnable**.
L'import ha scoperto e fatto fixare 2 bug reali (`??` su offset di stringa #69889,
literale intero gigante → `INF` #74947) e 1 divergenza ereditata da mago (`\u{}`).

## Perché Rust semplifica Zend

| Sottosistema Zend | LOC C | Sostituto Rust | LOC Rust |
|---|---|---|---|
| VM generata (`zend_vm_execute.h`) + `zend_execute.c` | ~146.000 | evaluator tree-walk su HIR | 3–5K |
| `zend_compile.c` (AST→opcodes) | 12.400 | lowering AST→HIR | 1–2K |
| lexer re2c + parser Bison + AST | ~25.000 | dipendenza `mago` + bridge | ~500 |
| `zend_alloc` / `zend_gc` / TSRM / Optimizer / opcache / win32 | ~88.000 | ownership, `Rc`+COW, `Send`/`Sync`, processo residente | ~0 |
| `zend_operators.c` (type juggling) | 3.900 | **full port fedele** (l'anima di PHP) | ~1.500 |

~280K LOC del core → ~8–10K LOC Rust stimati.

## Struttura

```
php-rust/crates/
  php-types      Zval / PhpStr / PhpArray / Object + operatori (zero dep interne)
  php-runtime    HIR, lowering da mago, evaluator tree-walk (OOP, eccezioni,
                 enum, closure, __destruct, interpolazione; json_decode +
                 preg_* intercettati; stack-trace)
  php-builtins   registry ~65 builtin (var_dump/print_r, array_*, string,
                 sprintf, math, json_encode, …)
  php-cli        binario `phpr`                           (scheletro)
  phpt-runner    runner .phpt + capability scan (bin + lib)
diary/           00-reconnaissance … 99-conclusions + metrics
```

## Build & test

```bash
cd php-rust
cargo test                       # unit + integration
# differential vs oracle (richiede un binario php):
#   build dal sorgente:  ./configure --disable-all --enable-cli && make
PHP_ORACLE=/path/to/php cargo test -p php-types --test differential
```

Il differential si auto-salta con un messaggio se l'oracle non è disponibile.

### phpt-runner

Esegue i `.phpt` ufficiali attraverso l'evaluator, con capability scan e
classificazione PASS/FAIL/SKIP (l'unico FAIL è una divergenza di output reale):

```bash
cargo run -p phpt-runner -- /path/to/php-src/tests /path/to/php-src/Zend/tests
cargo run -p phpt-runner -- --list-fails <path>   # mostra i diff dei fail
```

## Diario

Il deliverable principale dell'esperimento è il **diario metodologico** in `diary/`,
non solo il codice: decisioni (`02-mapping-table.md`), log per step
(`03-translation-log.md`), divergenze trovate, conclusioni.

---

*Generato con assistenza AI (Claude Fable 5 / Opus 4.8). Codice e commenti tecnici
in inglese, diario in italiano.*
