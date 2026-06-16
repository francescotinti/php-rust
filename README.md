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

**Steps 0–40 completati · 733 test verdi · clippy pulito · differential 37.835 casi a 0 mismatch.**

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
