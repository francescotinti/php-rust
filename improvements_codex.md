# Audit Codex - miglioramenti per php-rust-experiment

Data: 2026-06-26

Repo analizzata: `/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust`

Ambito richiesto: costrutti del linguaggio PHP e Standard Library/core, non estensioni PHP di terze parti. La documentazione di riferimento parte dal manuale ufficiale PHP: <https://www.php.net/manual/en/index.php>.

## Sintesi esecutiva

Il progetto Rust non e' una bozza iniziale: ha gia' una pipeline coerente `PHP source -> mago AST -> HIR -> bytecode -> VM`, con tree-walker rimosso dal percorso produttivo, un nucleo OOP avanzato, autoload SPL, error handler, generatori, Fiber, enums, property hooks e una Standard Library "Composer-driven" piuttosto ampia.

La priorita' non e' "attivare tutto da zero", ma chiudere i punti in cui PHP reale si aspetta metadata e semantiche complete:

- Traits: sono implementati e testati, ma vanno irrobustiti. Non risultano "non attivati"; sono flattenati nel lowering. Mancano pero' controlli PHP completi su compatibilita' di proprieta'/costanti, metadata runtime (`trait_exists`, `class_uses`, `get_declared_traits`) e almeno una divergenza dichiarata su `__CLASS__` dentro i metodi di trait.
- Reflection/attributes: oggi e' presente solo uno stub PHP in prelude (`ReflectionClass`, `ReflectionProperty`) e `getAttributes()` ritorna sempre `[]`. Composer puo' sopravvivere a una parte di questo, ma framework moderni no.
- Named arguments/spread/dynamic dispatch: la base e' buona, ma restano casi non uniformi su builtins, static calls dinamiche, `new static`/`new $cls`, dynamic method calls con named args e funzioni by-ref.
- Type system: parametri e return type coprono hint singoli scalar/array/callable/iterable/object/class e nullable. Restano scoperti union/intersection, `mixed`, `void`, `never`, literal types, `self`/`parent`/`static`, typed properties e typed class constants.
- Standard Library: ci sono circa 268 registrazioni nel registry `php-builtins`, piu' host builtins nel VM. E' utile per Composer, ma non e' ancora guidata da stubs ufficiali, firme complete, metadata Reflection e test generati.
- PHPT runner: e' gia' molto utile, ma salta `--EXPECTREGEX--` e non copre ancora tutte le forme del runner ufficiale. Questo limita la visibilita' dei gap.

Nota operativa: in questa sessione gli MCP Serena/Vexp richiesti da `php-rust/CLAUDE.md` non erano esposti tra gli strumenti disponibili; l'analisi e' stata quindi fatta con `rg`, letture mirate e confronto statico dei file. Non ho modificato il codice Rust.

## Stato osservato

File e moduli principali:

- `crates/php-runtime/src/lib.rs`: entry point e architettura della pipeline.
- `crates/php-runtime/src/hir.rs`: modello intermedio del linguaggio.
- `crates/php-runtime/src/lower/*.rs`: abbassamento da mago AST a HIR.
- `crates/php-runtime/src/compile.rs`: compilazione HIR -> bytecode.
- `crates/php-runtime/src/bytecode.rs`: opcodes del VM.
- `crates/php-runtime/src/vm/mod.rs`: VM, host builtins, error/autoload/OOP/coroutine glue.
- `crates/php-runtime/src/vm/{arrays,calls,coroutines,exceptions,oop}.rs`: scomposizione parziale del VM.
- `crates/php-builtins/src/*.rs`: builtins stateless, registrati in `php-builtins/src/lib.rs`.
- `crates/phpt-runner/src/*.rs`: runner `.phpt`.

Dimensioni indicative:

- `vm/mod.rs`: 13.347 righe.
- `compile.rs`: 3.854 righe.
- `lower/*.rs`: circa 5.374 righe.
- `php-builtins/src/*.rs`: circa 10.463 righe.

La worktree era gia' sporca prima dell'audit:

- modificati: `Cargo.toml`, `FilePutContentsVar1.tmp`;
- cancellato: `fopen_variation6.tmp`;
- non tracciati: `VISION_AND_ROADMAP.md`, `analysis_and_suggestions_v4.md`, `public/`.

## Riferimenti PHP da usare come checklist

Fonti ufficiali consigliate per il backlog:

- Manual index: <https://www.php.net/manual/en/index.php>
- Language reference: <https://www.php.net/manual/en/langref.php>
- Functions and arguments: <https://www.php.net/manual/en/functions.arguments.php>
- Classes and objects: <https://www.php.net/manual/en/language.oop5.php>
- Traits: <https://www.php.net/manual/en/language.oop5.traits.php>
- Enumerations: <https://www.php.net/manual/en/language.enumerations.php>
- Attributes: <https://www.php.net/manual/en/language.attributes.php>
- Fibers: <https://www.php.net/manual/en/language.fibers.php>
- Standard extension: <https://www.php.net/manual/en/book.standard.php>
- Function reference: <https://www.php.net/manual/en/funcref.php>

## Matrice costrutti linguaggio

| Area | Stato osservato | Gap/rischio | Priorita |
| --- | --- | --- | --- |
| Parsing e AST | Usa `mago`, quindi il parser copre molta sintassi PHP moderna. | Bisogna distinguere sintassi parsata da sintassi abbassata/compilata. Mantenere una matrice `AST variant -> lower -> compile -> VM`. | P0 |
| Namespace e `use` | Presenti nel lowering; risoluzione nomi usata da classi/funzioni. | Verificare alias complessi, grouped use, function/const imports e interazione con autoload. | P1 |
| Variabili | Variabili dirette, superglobali, `$GLOBALS`, static locals e global alias sono presenti. | Variable variables (`$$x`) non supportate; `global $$x` non supportato. Questo e' costrutto linguistico PHP, non estensione. | P1 |
| Array | Array PHP, spread in array literal, destructuring, references e molte operazioni sono presenti. | Destructuring by-reference e spread in destructuring sono unsupported. Alcune letture/scritture con path misti oggetto/array restano limitate. | P1 |
| Control flow | `if`, loop, `foreach`, `switch`, `match`, `try/catch/finally`, `goto`, `break/continue`, `return`, `exit` sono ampiamente coperti. | `declare` colon-delimited block non supportato; verificare `ticks`/`encoding`. Alcuni casi di `goto` restano compile-time unsupported. | P1 |
| Funzioni | Hoist, conditional function declarations, closures, arrow functions, first-class callable semplice, variadic e by-ref hanno copertura ampia. | Partial application vera e method/static first-class callable non supportati; dynamic first-class callable non supportato. | P1 |
| Chiamate | Named args su funzioni utente note e spread su diversi percorsi sono presenti. | Named args su builtins, dynamic calls, alcune static calls runtime, `new static`/`new $cls`, reference calls con named/spread e by-ref+spread restano incompleti. | P0 |
| Tipi param/return | HIR conserva `TypeHint` singolo nullable; VM controlla scalar, array, callable, iterable, object e class/interface. | Union/intersection, `mixed`, `void`, `never`, literal types, `self`/`parent`/`static` sono abbassati a `None` o non enforce. | P0/P1 |
| Typed properties | Readonly e property hooks sono modellati. | Il tipo della property non e' in `PropDecl`; typed property initialization/type errors non sono pienamente modellati. | P0 |
| Class constants | Class constants e enum cases sono presenti. | Visibility dei class constants accettata ma non enforce; typed class constants non modellate; dynamic class constant name unsupported. | P1 |
| OOP base | Classi, interfacce, abstract, inheritance, visibility, static props, LSB, magic methods, anonymous classes e readonly class hanno copertura estesa. | `final` class/method non risulta nel modello HIR; verificare enforcement. Interfacce portano solo costanti e abstract method names, con metadata limitato. | P1 |
| Traits | Implementati: lowering, flattening, `insteadof`, `as`, nested traits, abstract requirements, props, static props, consts e test in `php-runtime/tests/eval.rs`. | Mancano controlli completi PHP su compatibilita' di proprieta'/costanti duplicate. Metadata runtime dei traits assente o non esposto. Commento nel codice segnala divergenza su `__CLASS__` dentro trait method. | P0 |
| Enums | Unit/backed enums, casi, `from`, `tryFrom`, readonly case props e interfacce marker sono presenti. | Solo backing `int|string`; trait con properties in enum e' unsupported; verificare metadata Reflection e comportamento con constants/traits. | P1 |
| Attributes | Parser probabilmente li accetta; prelude dice esplicitamente che `phpr` non li conserva. | Mancano storage HIR/VM e ReflectionAttribute; `ReflectionClass::getAttributes()` ritorna sempre `[]`. | P0 |
| Generators | Implementati con semantica lazy e test; supporto coroutine nel VM. | Verificare `yield from`, send/throw edge cases e interazione con finally. | P1 |
| Fibers | Presenti in `vm/coroutines.rs`; documentazione repo segnala `Fiber::throw()` e nesting patologico come gap. | Chiudere `Fiber::throw()` e casi di callable non closure/function se richiesti da framework. | P1 |
| Error handling | `error_reporting`, `trigger_error`, `error_get_last`, `set_error_handler`, `restore_error_handler` implementati e testati. | Mappare tutti i warning/notices dei builtins sullo stesso chokepoint; verificare livelli PHP 8.5 completi e `@` error control. | P0 |
| Exceptions | Gerarchia prelude `Throwable`, `Exception`, `Error`, `TypeError`, `ValueError`, catch/finally e trace sono presenti. | Reflection/trace formatting e file/line in include/eval vanno verificati con corpus piu' ampio. | P1 |
| Include/autoload | Include/require e SPL autoload host builtins presenti; class_exists/interfaccia con autoload gestita. | Serve stress test Composer su `vendor/autoload.php`, include_path, relative path, case sensitivity e error paths. | P0 |
| Reflection | `ReflectionClass` e `ReflectionProperty` minimi in prelude. | Mancano `ReflectionMethod`, `ReflectionFunction`, `ReflectionParameter`, `ReflectionAttribute`, type reflection, modifiers, doc comments, file/line, traits/interfaces metadata completi. | P0 |

## Traits: analisi mirata

Gemini/Claude hanno segnalato "traits non attivati". Nel codice attuale la lettura e' diversa:

- `Program` contiene `traits: Vec<(Vec<u8>, LoweredTrait)>`.
- `LoweredTrait` porta metodi, proprieta', static props, constants e abstract methods.
- `lower_traits` risolve trait top-level e trait usati da altri trait.
- `flatten_into` applica `insteadof` e `as`, copia i membri nella classe consumer e segnala collisioni di metodi non risolte.
- I test in `php-runtime/tests/eval.rs` coprono trait base, props, static props, constants, `static::`, `new static`, alias/visibility, nested traits e abstract requirements.

Quindi i traits non sono assenti; sono parziali. I punti da chiudere sono questi:

1. Compatibilita' proprieta' duplicate.
   - Oggi `flatten_into` usa set di nomi e, se una prop e' gia' vista, la salta.
   - PHP richiede fatal quando due traits o trait+classe dichiarano property incompatibili per visibility/type/readonly/default.
   - Ticket: introdurre `trait_property_compatible(a, b)` e test differenziali contro Zend per visibility/default/type/readonly/static.

2. Compatibilita' costanti duplicate.
   - Oggi le constants duplicate sono filtrate per nome.
   - PHP richiede compatibilita' precisa tra trait constants e class constants; valori/visibility/type non equivalenti devono fallire.
   - Ticket: confrontare valore const-folded, visibility e typed const quando implementate.

3. Metadata runtime.
   - Dopo flattening il runtime perde l'identita' del trait.
   - Mancano funzioni come `trait_exists`, `class_uses`, `get_declared_traits`.
   - Reflection deve poter rispondere a `ReflectionClass::getTraitNames()`, `getTraits()`, `getTraitAliases()`.

4. Magic constants.
   - Il commento in `lower/class.rs` segnala che `__CLASS__` in un metodo di trait dovrebbe essere la classe che usa il trait, ma oggi durante la risoluzione del trait non lo e'.
   - Ticket: differire la risoluzione di `__CLASS__`/`__METHOD__` per metodi trait fino al flattening nella classe consumer, oppure conservare marker HIR risolti in compile.

5. Error messages.
   - PHP ha messaggi molto specifici per collisioni trait.
   - I test devono validare non solo "fatal", ma anche messaggio e line number quando PHPT lo richiede.

## Standard Library e builtins

`php-builtins/src/lib.rs` registra circa 268 builtins stateless. Le aree gia' presenti includono:

- array: `count`, `array_keys`, `array_values`, `array_merge`, diff/intersect, sort/ref-first, splice, push/pop/shift/unshift, ecc.
- string/format/html/csv: `strlen`, `substr`, `strpos`, `str_replace`, casing, trim, sprintf/printf/vsprintf/vprintf, htmlspecialchars, CSV.
- file/stream/filesystem: `fopen` host-side, `fread`, `fwrite`, `file_get_contents`, `file_put_contents`, stat/path functions, directory functions, permissions, tempnam, glob.
- date/time: `date`, `gmdate`, `strtotime`, `DateTime`/`DateTimeImmutable` prelude minimale.
- json/serialize/pack/hash/crypto/mbstring: nucleo utile per Composer.
- env/runtime: `ini_get`, `ini_set`, memory/gc stubs, `php_sapi_name`, `extension_loaded`.

Il VM aggiunge host builtins dove serve accesso allo stato runtime:

- SPL autoload, `call_user_func`, `call_user_func_array`.
- `define`, `defined`, `constant`.
- higher-order array functions (`array_map`, `array_filter`, `array_reduce`) e by-ref host builtins.
- class/object introspection (`get_class`, `class_exists`, `interface_exists`, `method_exists`, `property_exists`, `class_parents`, `class_implements`, `get_object_vars`, ecc.).
- error/exception handlers.
- `preg_*`, `json_decode`, `unserialize`, `debug_backtrace`.

Raccomandazione principale: passare da registry manuale a "stub-driven builtins".

1. Importare le firme dagli stubs PHP ufficiali (`*.stub.php` in php-src) per Standard Library/core.
2. Generare metadata: nome canonico, alias, arita', default, by-ref, variadic, out params, return type, deprecation.
3. Usare un macro/proc-macro o codegen per registrare i builtins e creare adapter Rust.
4. Collegare lo stesso metadata a ReflectionFunction/ReflectionParameter.
5. Generare test minimi firma/errore per arity/type/default.

Questo riduce il rischio maggiore della Standard Library attuale: implementazione utile ma "a mano", con firme e Reflection non necessariamente allineate.

## Composer: backlog P0

Per Composer e pacchetti moderni la priorita' non e' coprire tutto il manuale in ordine alfabetico. Serve chiudere i costrutti che rompono autoload, dependency resolution e bootstrap.

1. Reflection e attributes.
   - Implementare metadata HIR per attributes, doc comments, file/line, modifiers, traits, interfaces, parent, methods, properties, constants, params e return types.
   - Aggiungere `ReflectionMethod`, `ReflectionFunction`, `ReflectionParameter`, `ReflectionAttribute`, `ReflectionNamedType`, `ReflectionUnionType`.
   - Non lasciare `getAttributes()` sempre vuoto.

2. Trait metadata e compatibilita'.
   - Vedi sezione Traits.
   - Aggiungere `trait_exists`, `class_uses`, `get_declared_traits`.

3. Named/spread ABI unica.
   - Unificare binder runtime per funzioni, metodi, static calls, constructors e builtins.
   - Obiettivo: nessun `Unsupported("argument unpacking (spread)")` per chiamate valide PHP.

4. Typed properties e class constants.
   - Conservare type hint in `PropDecl` e `ClassConstDecl`.
   - Enforce su assignment, initialization, readonly, hooks, unset/read uninitialized.
   - Implementare typed class constants e visibility enforcement.

5. Error model uniforme.
   - Tutti i builtins devono emettere warning/notice/deprecated attraverso il chokepoint del VM, cosi `set_error_handler` vede tutto.
   - Audit specifico su `@`, `error_reporting(0)`, shutdown e fatal.

6. PHPT runner piu' fedele.
   - Supportare `--EXPECTREGEX--`.
   - Valutare le sezioni `SKIPIF`, `INI`, `ENV`, `ARGS`, `CLEAN`, `FILEEOF` dove utili per Zend/tests.
   - Non classificare come skip cio' che e' una regressione reale.

7. Composer harness.
   - Creare un target ripetibile: bootstrap Composer, `composer validate`, `composer install --no-scripts` su fixtures piccole, poi pacchetti reali.
   - Loggare missing builtin, unsupported lowering, unsupported compile e fatal mismatch in categorie separate.

## Backlog P1 per completezza PHP 8.5

1. Type declarations complete.
   - Union/intersection/DNF types.
   - `mixed`, `void`, `never`, `true`, `false`, `null`.
   - `self`, `parent`, `static` nei type hints.
   - Return `void`/`never` con semantica corretta.

2. OOP modifiers.
   - Enforce `final` su classi/metodi.
   - Verificare abstract/final/private combinations e messaggi.
   - Completare asymmetric property visibility PHP 8.4 se non gia' gestita in modo completo.

3. Dynamic language constructs.
   - Variable variables.
   - Dynamic static property names.
   - Dynamic class constant names.
   - Named args su dynamic calls.

4. First-class callables e partial application.
   - Supportare `Class::method(...)`, `$obj->method(...)`, `$obj->$m(...)`, static/dynamic variants.
   - Verificare `Closure::fromCallable`, binding scope e visibility.

5. Properties/hooks.
   - Abstract property hooks.
   - By-reference hooks.
   - Hook contracts in interfaces/abstract classes.
   - Interaction hooks + readonly + typed properties + Reflection.

6. Serialization.
   - `__sleep`, `__wakeup`, `__serialize`, `__unserialize`.
   - `__PHP_Incomplete_Class` invece del fallback a `stdClass` per classi sconosciute.
   - Shared-reference markers `r:`/`R:`.

7. Standard classes core.
   - SPL iterators/classes piu' complete.
   - `DateTimeZone`, `DatePeriod`, eccezioni SPL, array/object iterators con flags.
   - `stdClass`/dynamic properties deprecation behavior.

## Backlog P2 per manutenibilita'

1. Scomporre `vm/mod.rs`.
   - Continua il lavoro gia' iniziato in `vm/{arrays,calls,coroutines,exceptions,oop}.rs`.
   - Target: `dispatch`, `frame`, `builtins_host`, `autoload`, `properties`, `reflection`, `errors`, `include_eval`.
   - Evitare refactor massivi senza test: spostare per aree e usare snapshot PHPT.

2. Ridurre `expect()`/`unwrap()` nel VM.
   - Molti `expect()` sono invarianti interne, ma in un runtime PHP maturo devono diventare `VmInvariant`/`PhpError` dove input utente puo' raggiungerli.
   - Priorita' alle operazioni su stack/path/object/property.

3. Matrice automatica degli unsupported.
   - Generare report da `rg "Unsupported"` e dal `phpt-runner --list-fails --isolate`.
   - Salvare top-N per categoria: lower unsupported, compile unsupported, missing builtin, runtime fatal mismatch.

4. Test differenziali piccoli e frequenti.
   - Per ogni costrutto aggiungere micro-test Rust e, se possibile, `.phpt` corrispondente contro oracle Zend.
   - Validare stdout, stderr/rendered diagnostics, fatal type, message e line.

5. Metadata unico per funzioni/classi.
   - Oggi runtime, prelude PHP e builtins Rust duplicano conoscenza.
   - Creare una sorgente unica di metadata per Reflection, call binder e error messages.

## Checklist manuale PHP -> implementazione

Da usare come tabella di avanzamento:

- Basics: tags, comments, echo/print, literals, strings, heredoc/nowdoc, interpolation.
- Types: scalar, array, object, callable, iterable, resource placeholder, null.
- Variables: normal, references, globals, statics, superglobals, variable variables.
- Expressions: operators, precedence, casts, `isset`, `empty`, `eval`, `include`, `clone`, `new`, `match`.
- Control structures: all loops, switch/match, declare, goto, exceptions, finally.
- Functions: defaults, named args, unpacking, by-ref, variadic, closures, arrow, first-class callable.
- OOP: classes, anonymous classes, inheritance, interfaces, traits, enums, visibility, abstract/final, readonly, hooks, magic methods.
- Type declarations: params, returns, properties, constants, union/intersection/DNF/literal/nullable.
- Attributes: storage, target validation, repeatability, Reflection.
- Generators/Fibers: send/throw/return/finally/nesting.
- Error model: warnings/notices/deprecations/fatals, handlers, shutdown, `@`.
- Standard Library core: array/string/math/date/file/json/serialize/hash/regex/SPL/Reflection/Error.

## Raccomandazione finale

La strada piu' efficace e':

1. Trattare Composer come harness di prodotto, non come unico oracle.
2. Usare Zend/tests come oracle semantico.
3. Generare una compatibility matrix automatica dal codice e dai test.
4. Chiudere prima metadata, Reflection, traits e call ABI.
5. Solo dopo ampliare la Standard Library in orizzontale.

Il progetto ha gia' un nucleo VM molto piu' maturo di quanto suggeriscano alcune note storiche. Il rischio principale ora e' la compatibilita' "di bordo": metadata, introspezione, firme, messaggi e casi dinamici. Sono proprio queste le zone che Composer e i framework PHP moderni usano per capire il mondo.
