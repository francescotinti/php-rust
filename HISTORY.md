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

**Steps 0–68 + Sessione F completati · 1.496 test verdi · clippy pulito · differential 37.835 casi a 0 mismatch.**

> **✅ Migrazione VM completata (Sessione F).** La **VM a bytecode** (`compile.rs` HIR→bytecode +
> `vm/` dispatch loop) è ora **l'unico motore di produzione**. Il tree-walker (`eval/`, ~7.000 righe),
> la dipendenza `corosensei` e l'unico `unsafe` non-FFI del runtime sono stati **eliminati**: il payoff
> progettato è realizzato — generatori e Fiber girano su uno **stack di frame esplicito** (park dell'`ip`
> in una side-table del `Vm`), senza coroutine stackful né reborrow `*mut Evaluator`. L'harness di parità
> dual-engine VM↔eval ha esaurito il suo scopo ed è stato rimosso; la fedeltà resta ancorata al PHP 8.5.7
> reale via `differential.rs`. Le sezioni *«in corso»* qui sotto sono il **registro storico** di come la
> VM è stata portata a parità prima dello switch.

**Come la VM è stata costruita (storico).** Coperto e validato da test unitari VM-side: espressioni e control-flow,
chiamate, array, **OOP completo** (classi, `$this`, proprietà/visibility, metodi, `static` + LSB,
costanti di classe, magic `__get`/`__set`/`__isset`/`__unset`/`__toString`/`__destruct`, nullsafe
`?->`), e e il **blocco reference completo** — **REF-1** (`$a = &$b` fra variabili bare, `global`), **REF-2**
(parametri by-ref su funzioni utente, con propagazione attraverso chiamate annidate), **REF-3**
(`foreach $a as &$v`, incluso il *lingering-reference gotcha* fedele all'oracle) e **REF-4** (ref su
elementi array `$x = &$a[0]` / `$a[0] = &$x`, e ritorno by-ref `function &f()` con `$y = &f()`).
Inoltre le **closure** (CLO): funzioni anonime e arrow `fn`, cattura `use($x)`/`use(&$x)` e
auto-cattura, `$this` legato nei metodi, invocazione dinamica `$f(...)` (incluso IIFE), chiamata
via stringa-nome e first-class callable `f(...)`. E le **eccezioni** (EXC): `throw` e
`try`/`catch`/`finally` per oggetti user — unwinding dei frame via tabella di regioni protette per
funzione (robusta a uscite anticipate), catch per tipo/interfaccia (`catch (Throwable)`), multi-catch,
propagazione attraverso le chiamate, `finally` su tutti i path (normale/catturato/propagante/annidato).
Inoltre (**EXC-3a**) gli **engine error sono catchabili**: `DivisionByZeroError` (`1 % 0`, `1 / 0`),
`TypeError` (`[] + 1`), `Error` (es. istanza di classe astratta) vengono risolti alla classe-prelude
per nome e *sintetizzati* in un Throwable (con `getMessage()` corretto), instradato al `catch` come un
throw user — incluso il match per supertipo (`catch (ArithmeticError)` su un `DivisionByZeroError`) e
`catch (Throwable)`. E (**EXC-3b**) il **line-tracking**: ogni op porta la sua linea sorgente (tabella
`Func.lines` parallela a `ops`), così ogni Throwable — sia `new Exception` (linea fissata al `new`,
come PHP, non nel costruttore) sia engine error sintetizzato (linea dell'op che fallisce) — riporta
`getLine()`/`getFile()` corretti. E (**EXC-3c**) lo **stack trace**: `getTrace()` (array di frame con
`file`/`line`/`function`/`class`/`type`) e `getTraceAsString()` (`#i file(line): fn()` … `#N {main}`),
ricostruiti dallo stack di frame della VM — byte-identici al tree-walker per i throw user, con un
miglioramento di fedeltà a PHP per gli engine error (la VM ne cattura il trace al punto di errore,
mentre il tree-walker lo sintetizza dopo l'unwind e lo lascia vuoto). Il **blocco eccezioni (EXC) è
così completo** nella VM. E sono partiti i **generatori** (**GEN-1**): nella VM un generatore è un
`Frame` *sospeso parcheggiato* in una side-table del `Vm` — `yield`/`yield $k=>$v`, `foreach`,
`current`/`key`/`next`/`valid`/`rewind`, chiavi auto/esplicite con bump del contatore,
closure-generator. Il dispatch è ora un *bounded runner* (`run_loop(baseline)`): `yield` sospende
ritornando su per lo stack Rust, `resume` ripristina — **niente `corosensei`, niente `unsafe`** (il
payoff della migrazione, poi realizzato eliminando `eval/` alla Sessione F). **GEN-2** aggiunge
`send()` (ping-pong: il valore arriva come risultato del `yield` sospeso, con priming automatico),
`return`/`getReturn()` e la fedeltà a PHP sulle eccezioni di misuso (rewind-dopo-run e
getReturn-troppo-presto sono `Exception`, dove il tree-walker usa `Error`). **GEN-3** aggiunge
`yield from` su array e sub-generatori: l'opcode `YieldFrom` si **ri-entra da solo** a ogni resume
(un passo di delega per volta, chiavi verbatim col contatore esterno intatto, `send()` inoltrato nel
sub-generatore, valore dell'espressione = `getReturn()` del delegato), con delega annidata. E
(**GEN-4**) le **Fiber** — *funzionalità nuova, assente nel tree-walker*, validata solo contro PHP
reale: `start`/`resume`/`Fiber::suspend`/`getReturn`/`getCurrent`/`is*`, con passaggio argomenti e
propagazione delle eccezioni fuori dalla fiber. A differenza di un generatore (un frame), `Fiber::
suspend` può essere chiamata da *qualsiasi profondità* dello stack della fiber, quindi la sospensione
parcheggia l'**intero segmento di frame** in una side-table del `Vm` e lo ripristina al resume — di
nuovo senza `corosensei`/`unsafe`. Scope-out dichiarati: `Fiber::throw()` e il nesting patologico
fiber-dentro-generator. **GEN-5/CLEANUP — fatto (Sessione F):** switch del motore in `lib.rs`,
eliminazione di `eval/` e drop di `corosensei` + del trait `GenDriver`/dell'`unsafe`.

**Parità VM (storico).** Con i generatori/Fiber completi, è iniziato il lavoro per portare la VM alla
parità di copertura con l'eval (pre-requisito per spegnere `eval/`). Primo blocco: **parametri di
default + arità** — un *prologo* per-funzione (`Op::FillDefault`) riempie gli argomenti opzionali
omessi nel frame del chiamato (così un default può riferirsi a parametri precedenti), e il binding
degli argomenti è ora limitato a `n_params` (gli argomenti in eccesso vengono scartati come in PHP).
Vale per funzioni, metodi, costruttori e closure. Secondo blocco: **funzioni variadiche**
(`function f($a, ...$rest)`) — il binder raccoglie gli argomenti in eccesso in un array (chiavi intere
sequenziali, vuoto se nessuno) nello slot variadico, combinandosi coi default. Terzo blocco: **riferimenti dinamici a classe** —
`new $cls` (`AllocDynamic`: il nome-classe è risolto a runtime via `class_index`, `\` iniziale tolto,
oppure un oggetto riusa la sua classe; classe ignota → `Error` catchabile) e `$x instanceof $cls`
(`InstanceOfDynamic`: classe ignota → `false`, come PHP) e `$cls::metodo()` (`StaticCallDynamic`:
classe risolta a runtime e dispatch non-forwarding via l'helper condiviso `dispatch_static_call`,
con default/variadici/`__callStatic`/ereditarietà) e `$cls::CONST` / `$cls::class`
(`ClassConstFromValue`: costante risolta a runtime con ereditarietà; `$obj::class` → nome classe,
`$str::class` → `TypeError` come PHP 8). La **famiglia classi-dinamiche è completa**. Inoltre **tutti i cast** sono ora supportati — `(float)`/`(array)` (`apply_cast`) e `(object)`
(`object_cast`: array→stdClass con una property per elemento, scalare→`{scalar:v}`, null→stdClass
vuoto, oggetto passthrough; mirror di `eval::object_cast`). E l'**`ArgumentCountError`** per arità
insufficiente: un guard `CheckArity` nel prologo della funzione confronta gli argomenti passati
(`Frame.argc`) coi parametri richiesti e lancia il messaggio fedele a PHP (`Too few arguments to
function f(), N passed in <file> on line L and exactly/at least M expected`, con `Class::method` per i
metodi e la linea del *call-site*). E gli **argomenti con nome** per le chiamate a funzione note
(`f(b: 2, a: 1)`): risolti a *compile-time* in layout posizionale (i nomi mappati agli slot via
`FnDecl.slots`), con `Op::PushUndef` per gli opzionali saltati che il prologo riempie di default;
fallback al tree-walker per i casi non esprimibili (variadici/by-ref, nome ignoto, required mancante).
E le **proprietà statiche su classe dinamica** `$cls::$p` (lettura/scrittura/`op=`): la classe è
risolta a runtime e instradata nel percorso `ensure_static` esistente (la classe è *peekata* dallo
stack così l'eventuale init-thunk può ri-eseguire l'op). Restano da portare alla VM: argomenti con
nome anche su **`new ClassName(...)`** e **`Class::metodo(...)`** a classe nota (risoluzione del
costruttore/metodo a compile-time + `emit_named_layout` condiviso). E l'**unpacking di argomenti**
`f(...$arr)` per le funzioni: si costruisce a runtime un array di argomenti (`ArrayInit` + `ArrayPush`
+ `ArrayAppendSpread`, che fonde un array o pilota un generatore) e `Op::CallArgs` lega i valori ai
parametri via lo stesso binder (default/variadici compongono). Restano da portare alla VM: argomenti
con nome su **metodi d'istanza** `$obj->m()`, spread su metodi/`new`/static, `$cls::$p++`/`??=`, e altri
edge. **Non sono problemi della VM ma del *lowering* (condivisi con l'eval)**: spread in array literal
`[...$a]` e nomi di membro dinamici `$o->{expr}`/`$o->$n` (rifiutati in `member_name`/lowering).

**Builtin host (storico).** Il grosso del divario di copertura residuo non è semantica di linguaggio
ma *funzioni che finora solo l'evaluator implementava*: la VM le sta assorbendo come **builtin host**
(`Op::CallHostBuiltin`, più la variante by-ref-first `Op::CallHostBuiltinRef` per chi muta il primo
argomento), molti dei quali richiamano codice utente in modo sincrono tramite un *runner annidato*
(`drive_to_return`) — così un higher-order builtin può eseguire una callback restando dentro il
dispatch della VM. Portati finora: gli **higher-order su array** (`array_map`/`array_filter`/
`array_reduce`/`usort`/`array_walk`), i **varargs** (`func_num_args`/`func_get_args`/`func_get_arg`),
`sprintf`/`printf` con risoluzione di `__toString` (`vm_stringify`, anch'esso un run annidato),
l'**introspezione classi** (`get_class`/`get_parent_class`/`get_object_vars`/`get_class_methods`) e i
predicati di esistenza (`function_exists`/`class_exists`/`interface_exists`/`method_exists`/
`property_exists`/`get_called_class`), il **sistema diagnostico/errori** (`error_reporting`/
`trigger_error`/`error_get_last`, con `error_level` per-VM e fix di `E_ALL`=`30719` a PHP 8.5),
`preg_replace_callback`, `set_exception_handler`/`restore_exception_handler` (stack di handler +
routing degli uncaught), `define`/`defined`/`constant` con tabella delle costanti utente, e un set di
**stub d'ambiente** (`gc_*`, `memory_get_usage`/`peak`, `php_sapi_name`, `ini_get`/`set`). Con questi,
sul corpus `Zend/tests` la VM **supera** il pass-rate dell'evaluator. Prossimo lever singolo più
grande: `set_error_handler` (instradare ogni diagnostico a una callback utente) — pianificato a parte.

**Modularizzazione `vm.rs`.** Cresciuto a ~9.500 righe, `vm.rs` è stato spezzato in
`vm/{mod,exceptions,coroutines,arrays,oop,calls}.rs` (ognuno un blocco `impl Vm`), lasciando in
`mod.rs` le struct, il `run_loop` di dispatch e la registry dei builtin host. Refactor puramente
meccanico verificato dal compilatore, **zero cambi di comportamento**, 1.402 test verdi a ogni
sotto-step — stesso trattamento già dato a `eval.rs` (step 60) e `lower.rs` (step 61).

Step 61 ha completato i suggerimenti della code-review esterna: (E) **diff unificato** nel
`phpt-runner` (`--list-fails` mostra un line-diff EXPECTF-aware invece di due blob troncati); (B)
flag **`PHP_RUST_TRACE`** che su stderr dumpa l'HIR (`=hir`/`body`) e/o traccia ogni statement
eseguito indentato per profondità di chiamata (`=exec`/`all`), per il triage lowering-vs-eval; e la
**modularizzazione di `lower.rs`** (3.783 → `lower/{mod,stmt,class,expr}.rs`, `mod.rs` 1.412
righe, −63%, zero cambi di comportamento); e (C) **7 test unitari oracle-independent** su
`php-types::ops` (l'anima type-juggling, prima senza test inline). Scartata solo la macro di
binding builtin (rischiosa).

Step 60 ha **modularizzato `eval.rs`** (era un monolite da 6.965 righe, segnalato da una
code-review esterna): spezzato in `eval/{mod,expr,stmt,calls,class,builtins}.rs`, ognuno un
blocco `impl Evaluator`. Refactor puramente meccanico, **zero cambi di comportamento**, 927 test
verdi a ogni sotto-step; `mod.rs` ora 1.913 righe (−72%). (`lower.rs`, 3.783 righe, candidato per
lo stesso trattamento in futuro.)

Step 59 ha (a) implementato il CLI **`phpr`** (era uno stub `fn main(){}`): ora è un `php`
drop-in che esegue uno script e scrive lo stream CLI-faithful con exit code fedele, utile anche
come differential vs l'oracle; (b) chiuso 13 dei 29 fail sprintf/printf residui con un batch di
fedeltà (modificatore `l`, specifier sconosciuti→ValueError, errori catchable col tipo giusto,
conteggio "N arguments are required", threading dei warning di coercion, pad char in
left-justify). Sweep `strings` **229→242/393 (61.6%)**. I residui non sono bug del motore
(output byte-identico all'oracle) ma di runner-EXPECTF su binario e `fopen(__FILE__)` non
materializzato dall'harness (l'interleaving dei warning dell'evaluator è stato **risolto** allo
step 66). Infra: `target-dir` di cargo spostato su disco interno (il volume esterno rompeva le
build incrementali).

Step 58 ha **chiuso il motore sprintf**: (a) fix del crash `capacity overflow` su width/precision
oltre `INT_MAX` (un `%9999…f` abortiva l'intero run in-process) → ora `ValueError`; (b) sintassi
`*` (width/precision da argomento, PHP 8.4) con binding posizionale e validazione fedele; (c)
conversioni `%g`/`%G`/`%h`/`%H` (port di `php_gcvt`: fixed/scientific shortest-form, byte-exact
vs oracle su 24×9 casi). `sprintf_star.phpt` ora passa; lo sweep `strings` completa in-process
(229/393). Il valore è trasversale: `%g` e `*` sono comuni in tutto il corpus, e ogni run
in-process è ora robusto.

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
| 49 | **constant expressions** — `define()`/`const`, costanti magiche (`__LINE__`/`__FILE__`/`__DIR__`/…), costanti named risolte in lowering; + hardening del phpt-runner | ✅ |
| 50 | **`serialize()` / `unserialize()`** — formato byte-exact (scalari, array, oggetti `O:`, ref), round-trip vs oracle | ✅ |
| 51 | **`fopen` + sottosistema filesystem-stream** — tipo `Zval::Resource`, `php://memory`/`temp`/`std*`, file reali r/w/a/x/c+; `fread`/`fwrite`/`fgets`/`fseek`/`ftell`/`feof`/`fclose` + `file_get_contents`/`file_put_contents`, byte-exact | ✅ |
| 52 | **predicati/operazioni filesystem** — `file_exists`/`is_*`/`filetype`, famiglia `stat`/`lstat`/`fstat` + accessor, mutatori `unlink`/`mkdir`/`rename`/`copy`/`touch`/`symlink`/`chmod`, `scandir`/`glob`/`tempnam`/`tmpfile`. `ext/standard/tests/file` **2 → 63 pass** | ✅ |
| 53 | lever cheap dir `file` — `strstr`/`strrchr`/`stristr`, `get_resource_type`, famiglia `opendir`/`readdir`/`closedir`/`rewinddir`, `fprintf`/`vfprintf` + fix panic dir-handle | ✅ |
| 54 | **engine di parsing** — `scanf` (`sscanf`/`fscanf`, return-array + by-ref) e **CSV** (`str_getcsv`/`fgetcsv`/`fputcsv`) | ✅ |
| 55 | batch stream/file read (`file`/`readfile`/`fpassthru`/`stream_get_contents`/`stream_copy_to_stream`/`ftruncate`) + `getenv`/`putenv` + `disk_*_space`. `file` **71 → 86 pass** | ✅ |
| 56 | batch funzioni stringa (`bin2hex`/`hex2bin`, `addslashes`/`stripslashes`, `substr_replace`, `nl2br`, `wordwrap`, `htmlspecialchars`/`htmlentities`+decode, `vsprintf`/`vprintf`). `strings` 51% runnable | ✅ |
| 57 | batch stringhe #2 (`strrpos`/`stripos`/`strripos`, `strspn`/`strcspn`, `strtr` byte-map+array, `chunk_split`, `strip_tags`, `quotemeta`, `levenshtein`) + fix `strtr("",$map)`. `strings` 58% | ✅ |
| 58 | **chiusura motore sprintf** — fix crash `capacity overflow` (width/precision > INT_MAX → `ValueError`); sintassi `*` (PHP 8.4, da argomento, posizionale); `%g`/`%G`/`%h`/`%H` (port di `php_gcvt`, byte-exact su 24×9 casi). `sprintf_star.phpt` passa | ✅ |
| 59 | **CLI `phpr`** (era uno stub: ora esegue uno script, stream CLI-faithful, exit code fedele) + batch fedeltà sprintf/printf (modificatore `l`, specifier ignota/mancante → `ValueError`, errori catchable col tipo giusto, conteggio "N arguments required", warning Array-to-string, pad char left-justify). `strings` **242/393 (61.6%)**. Infra: `target-dir` cargo fuori dal volume esterno | ✅ |
| 60 | **modularizzazione `eval.rs`** (6.965 righe) → `eval/{mod,expr,stmt,calls,class,builtins}.rs`, ognuno un `impl Evaluator`. Refactor meccanico, zero cambi di comportamento; `mod.rs` −72% | ✅ |
| 61 | **DevEx tooling** (code-review esterna): diff unificato EXPECTF-aware nel `phpt-runner` (E), flag `PHP_RUST_TRACE` = dump HIR + trace d'esecuzione su stderr (B), 7 test unitari oracle-independent su `php-types::ops` (C); + split `lower.rs` (3.783) → `lower/{mod,stmt,class,expr}.rs` (−63%) | ✅ |
| 62 | **famiglia hash/encoding** (`base64_encode`/`base64_decode`, `md5`, `sha1`, `crc32`, `hash`) in `encoding.rs` — base64 port byte-exact di `php_base64_decode_impl` (strict/lenient, padding), digest via RustCrypto (`md-5`/`sha1`/`sha2`), CRC-32 zlib via `crc32fast`. base64/md5/sha1/crc32 `.phpt` tutti verdi | ✅ |
| 63 | **`pack`/`unpack`** in `pack.rs` — port fedele di `ext/standard/pack.c` (host little-endian): tutti i codici `aAZ hH cC sSnv iI lLNV qQJP fgG deE xX@`, ValueError/Warning fedeli, chiavi nominali in `unpack`. `pack_*`/`unpack_*` `.phpt` verdi, byte-identico all'oracle su sweep ampio (inclusi codici 64-bit) | ✅ |
| 64 | **`crypt`** in `crypto.rs` su `pwhash` (DES/BSDi/MD5/SHA-256/512/bcrypt) + dispatch e convenzione `*0`/`*1` di `php_crypt`, costanti `CRYPT_*`, pre-check anti-hang su `rounds=N`. Dir `crypt/` 4/4 + crypt/sha256/sha512/des verdi, byte-identico all'oracle; 3 D-NEW (`$2x$`, bcrypt 8-bit, salt md5 non-standard — limiti pwhash, casi deprecati) | ✅ |
| 65 | **`strtok`** (stateful → evaluator-dispatched): campo `strtok_state` sull'`Evaluator`, port fedele di `PHP_FUNCTION(strtok)`. Forma a 2 arg (re)inizializza il cursore, 1 arg lo riprende, cleared a fine stringa. Tutti gli `strtok_*` `.phpt` verdi, byte-identico all'oracle | ✅ |
| 66 | **fix interleaving diag/output dei builtin**: in `dispatch_value_builtin` i diagnostici sollevati da un builtin che scrive su stdout (`printf`/`vprintf`) sono ora resi su `rendered` **prima** del suo output — il warning nasce durante la formattazione, prima della scrittura (es. `Array to string conversion` precede il risultato di printf, come PHP). Byte-identico all'oracle su casi multipli | ✅ |
| 67 | **harness: materializza lo script su disco** prima di eseguirlo (come run-tests.php) → `__FILE__`/`fopen(__FILE__)`/`include __FILE__` risolvono contro un file reale. Guardato: crea solo se assente, rimuove solo ciò che ha creato (mai sovrascrive file companion). Toglie i warning `fopen` spuri da decine di test | ✅ |
| 68 | **`%s` di sprintf/printf onora `__toString`**: la famiglia `sprintf`/`printf`/`vsprintf`/`vprintf`/`fprintf`/`vfprintf` è ora evaluator-dispatched — gli argomenti oggetto sono risolti via `__toString` (o lanciano il fatale corretto se assente) prima del motore di formato puro, ricorsivamente negli array dei `v*`. `strings` 290→292 pass; 1 D-NEW (`%d`/`%f` su oggetto-con-`__toString`) | ✅ |
| **F** | **Switch del motore al bytecode VM + eliminazione del tree-walker.** F1: `run_source`/`Outcome` → VM. Long-tail (chiusura gap VM): `@`/exit/goto, `json_decode`, `mb_split`/`mb_regex`, `sscanf`/`fscanf` (ABI out-param variadici by-ref), famiglia `mb_ereg*` (13 fn), introspezione `[parameter]` delle Closure, funzione indefinita = fatale a runtime, `goto`-attraverso-`finally` + scope-out `goto`-dentro-blocco (D-45.1). F2: eliminato `eval/` (~7.000 righe) + il dual-engine del runner. F3: drop `corosensei` + trait `GenDriver`/`unsafe`. **1.496 test verdi, zero `unsafe` non-FFI, VM motore unico** | ✅ |

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
| VM generata (`zend_vm_execute.h`) + `zend_execute.c` | ~146.000 | **VM a bytecode** (`compile.rs` HIR→bytecode + `vm/` dispatch loop, motore unico) | ~9.5K |
| `zend_compile.c` (AST→opcodes) | 12.400 | lowering AST→HIR | 1–2K |
| lexer re2c + parser Bison + AST | ~25.000 | dipendenza `mago` + bridge | ~500 |
| `zend_alloc` / `zend_gc` / TSRM / Optimizer / opcache / win32 | ~88.000 | ownership, `Rc`+COW, `Send`/`Sync`, processo residente | ~0 |
| `zend_operators.c` (type juggling) | 3.900 | **full port fedele** (l'anima di PHP) | ~1.500 |

~280K LOC del core → ~8–10K LOC Rust stimati.

> **Da due motori a uno.** Il progetto è partito con un *tree-walker* su HIR — sufficiente a riprodurre
> l'output osservabile di PHP. Ma generatori, `yield from` e `Fiber` richiederebbero, su un tree-walker,
> coroutine stackful (`corosensei`) e `unsafe`. Per evitarlo è stata costruita una **VM a bytecode**:
> avanzando un instruction pointer esplicito su uno stream di `Op`, sospendere un generatore è
> *parcheggiare un `Frame`* e il salto non-strutturato è un'istruzione ordinaria — niente coroutine,
> niente `unsafe`. I due motori sono coesistiti e si sono validati a vicenda finché la VM non ha
> raggiunto la piena parità di copertura; **alla Sessione F lo switch è stato completato**: `eval/` e
> `corosensei` sono stati rimossi e la VM è ora l'unico motore.

## Struttura

```
php-rust/crates/
  php-types      Zval / PhpStr / PhpArray / Object + operatori (zero dep interne)
  php-runtime    HIR, lowering da mago, e la VM a bytecode (motore unico):
                 compile.rs (HIR→bytecode) + vm/{mod,exceptions,coroutines,
                 arrays,oop,calls}.rs. Copre OOP, eccezioni, enum, closure,
                 generatori/Fiber (su frame espliciti, senza corosensei/unsafe),
                 __destruct, interpolazione, json_decode/preg_*/mb_*, stack-trace.
                 lowering in lower/{mod,stmt,class,expr}.rs
  php-builtins   registry ~243 builtin (var_dump/print_r, array_*, string,
                 sprintf/printf, math, json_encode, file/stream, mbstring,
                 hash/encoding: base64/md5/sha1/crc32/hash, pack/unpack, crypt,
                 strtok, …)
  php-cli        binario `phpr` (CLI: esegue uno script, stream CLI-faithful, exit code)
  phpt-runner    runner .phpt + capability scan + diff unificato (bin + lib)
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

Il differential si auto-salta con un messaggio se l'oracle non è disponibile; i
**test unitari** in `php-types::ops` coprono lo stesso nucleo type-juggling
**senza** bisogno dell'oracle.

### CLI `phpr`

`phpr` è un `php` drop-in: esegue uno script e scrive lo stream CLI-faithful
(output + diagnostics + fatal inline) con exit code fedele (`exit`/`die`, `255`
su fatal, altrimenti `0`). Utile anche come differential contro l'oracle.

```bash
cargo run -p php-cli -- script.php
```

### Tracing diagnostico (`PHP_RUST_TRACE`)

Per capire se un `.phpt` fallisce in *lowering* o in *evaluation*, su **stderr**
(non inquina lo stdout confrontato), valido per `phpr` e per il runner:

```bash
PHP_RUST_TRACE=hir   phpr s.php   # dump dell'HIR abbassato (Program intero)
PHP_RUST_TRACE=body  phpr s.php   # solo la lista di statement top-level
PHP_RUST_TRACE=exec  phpr s.php   # traccia ogni statement eseguito, indentato per call-depth
PHP_RUST_TRACE=all   phpr s.php   # HIR + trace d'esecuzione
```

### phpt-runner

Esegue i `.phpt` ufficiali attraverso la VM a bytecode, con capability scan e
classificazione PASS/FAIL/SKIP (i fuori-scope sono SKIP motivati):

```bash
cargo run -p phpt-runner -- /path/to/php-src/tests /path/to/php-src/Zend/tests
cargo run -p phpt-runner -- --isolate <path>      # ogni test in un sotto-processo (un crash = un FAIL)
cargo run -p phpt-runner -- --list-fails <path>   # diff unificato (riga + contesto) per ogni fail
```

## Diario

Il deliverable principale dell'esperimento è il **diario metodologico** in `diary/`,
non solo il codice: decisioni (`02-mapping-table.md`), log per step
(`03-translation-log.md`), divergenze trovate, conclusioni.

---

*Generato con assistenza AI (Claude Fable 5 / Opus 4.8). Codice e commenti tecnici
in inglese, diario in italiano.*
