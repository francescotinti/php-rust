# Fase 4 — Divergenze (D-NEW)

> Generato con assistenza AI (Claude Fable 5).

Catalogo delle divergenze semantiche tra la reimplementazione Rust e l'oracle
PHP 8.5.7. Ogni voce: severità, categoria, causa, stato.

## Stato a fine step 2

**Nessuna divergenza residua.** Il differential su 37.835 casi (operatori +
conversioni + formattazione float + diagnostica) è a 0 mismatch.

Le scoperte del differential durante lo step 2 NON sono divergenze residue: sono
state tutte riconciliate verso il comportamento dell'oracle. Le registro qui come
*lezioni* perché erano comportamenti non documentati nei report di analisi iniziali
e sarebbero stati bug latenti:

| # | Comportamento | Lezione |
|---|---|---|
| L1 | Stringa numerica in overflow intero → contesto int **satura** a LONG_MAX/MIN (strtol), silenziosa se round-trip-compatibile | `zendi_try_get_long` usa `zend_dval_to_lval_cap`, non il wrap di `dval_to_lval` |
| L2 | `NAN \| 0` emette **due** diagnostici (Warning "not representable" + Deprecated "loses precision") | `FITS_LONG(NAN)` è true → entra anche nel ramo deprecation |
| L3 | NAN→string: warning solo nel cast `(string)`, NON nella concatenazione | due path distinti in Zend |
| L4 | `pow` int overflow continua il loop square-multiply **in double dal punto di overflow** | `5**100 != pow(5.0,100.0)` per accumulo di rounding |
| L5 | `~true` → "...on true" (value name), non "...on bool" | `zend_zval_value_name` per i bool |
| L6 | Conversione operandi **sequenziale**: se op1 fallisce, op2 non viene convertito (niente warning spurio) | ordine di valutazione di `zendi_try_convert_*` |

## Step 6 — Scoperte dell'import .phpt (Fase 4c)

Il phpt-runner, eseguito sull'intera testsuite PHP (`tests/` + `Zend/tests/`,
6172 file), ha capability-scansionato e fatto girare 72 test in-scope. Ha
trovato **2 bug reali** (classe A, fixati nello stesso step) e **1 divergenza
ereditata dal front-end** (classe D, documentata):

| # | Severità | Categoria | Comportamento | Causa | Stato |
|---|---|---|---|---|---|
| D-NEW-2 | media | offset/string | `??` su offset di stringa: `$s[5] ?? d` restituiva `""` (out-of-range) e `$s["str"] ?? d` restituiva il char a offset 0 invece di `d` | `eval_isset` (path del `??`) usava la lettura normale (out-of-range→`""`, chiave-stringa→`to_long_cast`=0) invece della semantica isset (out-of-range / chiave non-intera → **not set**). Bug #69889. | **FIXATO**: `coalesce_index` + `coerce_key_silent` + `string_offset_silent`; regressione in `eval.rs::coalesce_on_string_offset` |
| D-NEW-3 | media | literali/float | un literale intero di ~320 cifre dava `~1.8e19` invece di `INF` | `lower_int` usava `lit.value` che mago **clampa a `u64::MAX`**; ora si ri-parsa il testo decimale grezzo → `f64::INFINITY`. Bug #74947. | **FIXATO**: `lower_int` ri-parsa il raw; regressione in `eval.rs::huge_integer_literal_overflows_to_inf` |
| D-NEW-4 | bassa | Unicode/front-end | `"\u{61}"` in stringa doppia non viene decodificato (resta `\u{61}` letterale) | **limitazione di mago 1.30**: decodifica `\n`/`\t`/`\x` ma non `\u{...}`. Ereditata via D-G8 (mago come front-end). Non correggibile a valle senza info sul quoting (single vs double). | **noto/aperto** — unico FAIL residuo (`tests/lang/string/unicode_escape.phpt`); le varianti `unicode_escape_*` (escape invalidi) sono già skip perché attendono warning |

Risultato finale del run completo: **71 pass / 1 fail / 6100 skip** (98.6% dei
runnable). Le skip sono motivate per categoria (vedi `metrics.md`): la grande
maggioranza (`unsupported`, 5215) è il confine atteso del Tier 1 (OOP, funzioni
utente, namespace, ecc.), non difetti.

## Step 8 — Scoperte dell'import .phpt (funzioni utente)

L'aggiunta delle funzioni utente ha reso *runnable* +44 test. L'import ha
trovato **1 bug reale** (classe A, fixato) e ha materializzato **1 divergenza
di design** dichiarata nello scope di step (classe D):

| # | Severità | Categoria | Comportamento | Causa | Stato |
|---|---|---|---|---|---|
| D-NEW-5 | media | eval-order | `$a[f()][g()] = $b[h()] = …`: gli offset dell'lvalue venivano valutati **dopo** la RHS, dando l'ordine invertito `i5 i6 i3 i4 i1 i2` invece di `i1..i6` (`engine_assignExecutionOrder_005/006`) | l'arm `AssignPlace` faceva `eval(rhs)` prima di `resolve_steps(place)`; PHP valuta gli offset del target da sinistra a destra **prima** della RHS. `AssignOpPlace` era già corretto | **FIXATO**: resolve-steps-first in `AssignPlace`; regressione `eval.rs::assignment_evaluates_lvalue_offsets_before_rhs` |
| D-NEW-6 | bassa | type-hint | `function f(float $n)` con default intero `0` → l'oracle stampa `float(0)`, noi `int(0)` (`scalar_float_with_integer_default_weak.phpt`) | **scelta di scope step 8 (D 8.3)**: type hint accettate ma **non enforced** — niente coercizione weak-mode né TypeError. Richiede il motore di coercizione dei tipi (step futuro) | **CHIUSO in step 14**: enforcement scalare weak-mode (int/float/string/bool + nullable) per param, default e return, con coercizione + TypeError + deprecation float→int. Il caso specifico (`float $n = 0` → `float(0)`) ora passa: i default sono coercizzati come gli argomenti |

Risultato del run completo dopo step 8: **114 pass / 2 fail / 6056 skip**
(98.3% dei runnable). I 2 FAIL residui sono D-NEW-4 (unicode `\u{}`, mago) e
D-NEW-6 (type-hint non enforced) — entrambi noti e documentati.

## Step 9 — Triage dei fail esposti dal rendering dei diagnostici

Rendere i diagnostici su stdout (vedi `03`) ha reso *runnable* i ~176 file prima
skippati come `diag-or-fatal`. Esito sul corpus completo: **126 pass / 62 fail**
(da 114/2). I 62 fail sono il segnale: divergenze che prima erano nascoste. La
triage li classifica — quasi tutti **scope gap di feature non implementate**, non
difetti del rendering (che è verificato dai 7 test `rendered_*`).

Prima della triage ho aggiunto **104 skip `compile-error`**: EXPECT che iniziano
con `Parse error:` o `Fatal error:` non-`Uncaught` sono diagnostiche compile-time
del motore (validazione attributi/tipi, strictness parser) che non modelliamo —
mago fa da front-end e accetta codice che il compilatore PHP rifiuta. Skip onesto
(capability scan), non fail.

Breakdown dei 62 fail residui (Classe B/scope salvo dove indicato):

| # fail | Gruppo | Causa | Classe | Stato |
|---|---|---|---|---|
| 13 | output divergence varia | mix (offset stringa, ordini, builtin parziali) | B | aperto/scope |
| 12 | deprecation da attributi / funzioni interne | `#[\Deprecated]`, `Function X() is deprecated`, nullable implicito — richiede attributi + segnatura builtin | B | scope (step OOP/builtin) |
| 10 | enforcement tipi (return/param) | `Return value must be of type …` — famiglia **D-NEW-6** (type hint accettate, non enforced) | D (dichiarata) | noto/aperto |
| 9 | diagnostica mancante (altre feature) | output vuoto vs warning atteso da feature non presenti | B | scope |
| 6 | altre deprecation | es. `case` seguito da `;`, parametri opzionali prima dei richiesti | B | scope |
| 6 | warning su offset stringa (write/illegal) | `Illegal string offset`, `Cannot use a scalar value as an array` su write | B | scope |
| 3 | superglobale `$GLOBALS` | trattata come variabile non definita → cascata di fatal | B | scope (superglobali non modellate) |
| 2 | `ArgumentCountError` + frame annidati | usiamo `PhpError::Error` con messaggio diverso; lo stack trace di un fatal lanciato *dentro* una call mostra i frame (`#0 file(line): f(...)`), noi rendiamo solo `#0 {main}` | A (modeling) | noto — vedi D-NEW-7 |
| 1 | precisione `float→int` nel warning | `serialize_precision=-1` (17 cifre) vs nostro shortest: `-9.223372036860776E+18` vs `-9.2233720368608E+18` | A (minore) | noto — vedi D-NEW-8 |

Nuove divergenze catalogate:

| # | Severità | Categoria | Comportamento | Causa | Stato |
|---|---|---|---|---|---|
| D-NEW-7 | bassa | fatal/stack-trace | un fatal lanciato dentro una funzione utente rende `#0 {main}` invece dei frame (`#0 %s(%d): f(...)`); inoltre `Too few arguments` usa classe `Error` invece di `ArgumentCountError` e wording diverso | step 9 modella solo il fatal top-level (`#0 {main}`); i frame richiedono uno stack di call esplicito nell'evaluator | **noto/aperto** — scope di un futuro step su eccezioni/stack |
| D-NEW-8 | molto bassa | float/precisione | il messaggio "The float … is not representable as an int" usa la rappresentazione *shortest* invece delle 17 cifre di `serialize_precision=-1` | il warning riusa `dtoa::double_to_shortest`; PHP qui formatta con precisione piena | **noto/aperto** — 1 solo test (`bug27354`) |

Fix Classe A applicato in step 9 (era nei missing-deprecated): **null come array
offset** → `Deprecated: Using null as an array offset is deprecated …` aggiunto a
`coerce_key`. +1 pass; regressione `rendered_null_array_offset_deprecation`.

## Step 22 — Scoperte dall'import .phpt (magic methods)

Corpus `Zend/tests/magic_methods`: 19 pass / 21 fail / 117 skip (47.5% runnable).
Due bug reali (Classe A) trovati e fixati durante la curation:

| ID | Severità | Categoria | Divergenza | Causa | Stato |
|---|---|---|---|---|---|
| D-NEW-9 | media | magic/empty | `empty($o->p)` con `__isset`→true ma **senza** `__get` emetteva un warning "Undefined property" (bug #44899); l'oracle è silenzioso e il warning compare solo sulla lettura normale successiva | `place_empty` leggeva il valore via `read_property` (che avvisa) invece che in contesto silent | **FIXATO**: helper `prop_value_silent` (chiama `__get` se c'è, altrimenti valore presente/NULL silenzioso), esteso a `empty`/`??`/`??=`; regressione `magic_empty_silent_when_isset_true_no_get` |
| D-NEW-10 | media | magic/dispatch | `parent::priv()` (o metodo ignoto) dentro un metodo instradava a `__callStatic` invece di `__call` (bug #53826) | `call_static` sceglieva sempre `__callStatic` sul fallback, ignorando la presenza di `$this` | **FIXATO**: in object-context (cur_this compatibile) instrada a `__call` d'istanza, `__callStatic` solo senza `$this`; regressione `magic_call_via_parent_in_object_context` |

Fail residui = feature adiacenti (scope-out): `__destruct`, `Stringable`
auto-interface, validazione firma/return dei magic method, `&__get` by-ref,
reference dentro prop overloaded, differenze `var_dump`/`print_r`. Dettaglio in
`02-mapping-table.md` (Step 22 IMPLEMENTATO).

## Step 23 — Scoperte (enum)

Corpus `Zend/tests/enum`: 43 pass / 18 fail / 91 skip (70.5% runnable). Tre
gap pre-esistenti — **generali, non enum-specifici** — emersi e fixati durante
l'implementazione degli enum:

| ID | Severità | Categoria | Divergenza | Causa | Stato |
|---|---|---|---|---|---|
| D-NEW-11 | alta | oggetti/identità | `$a === $b` fra due oggetti ritornava **sempre** `false` (anche `$a === $a`); mai testato finora | `ops::identical` non aveva arm `Object` → cadeva su `_ => false` | **FIXATO**: arm `(Object,Object) => Rc::ptr_eq` (semantica handle PHP); prerequisito per `===`/`match` sugli enum singleton; regressione `object_identity_handle_semantics` |
| D-NEW-12 | media | oggetti/uguaglianza | `$a == $b` fra oggetti cadeva su `compare()` (conversione scalare errata) invece della semantica PHP "stessa classe + props loosely-equal" | `ops::loose_eq` non aveva arm `Object` | **FIXATO**: arm `Object` (stessa istanza, o stessa classe + ogni prop `==`); per i case enum si riduce all'identità; regressioni `object_loose_equals_same_class_and_props`, `enum_loose_equals_is_identity` |
| D-NEW-13 | media | costanti/interfacce | `C::CONST` ereditata da `implements I` non si risolveva ("Undefined constant"); gh7821 | `eval_class_const` camminava solo la catena `parent`, non le interfacce | **FIXATO**: `find_class_const` cerca own→parent→interfacce (transitivo); regressioni `enum_inherits_interface_constants`, `class_inherits_interface_constants` |

Fail residui (scope-out dichiarato, vedi Step 23 in `02-mapping-table.md`):
- **by-ref readonly** (3): `no-pass/return/through-references` → "Cannot
  indirectly modify readonly property" (path by-reference non intercettato).
- **comparison.phpt**: operatori d'ordine `</>/<=/>=` fra oggetti (feature
  generale di confronto oggetti, non implementata).
- **validazione compile-time** (4): duplicate backing value, case type vs
  backing mismatch, `from()` argument TypeError, class type-hint enforcement.
- **dipendenti da subsystem assenti** (8): Reflection*, SplObjectStorage,
  WeakMap.
- **stack-trace frames** (1): `enum_in_stack_trace` (vedi D-NEW-7).

## Step 29 — Scoperte (espansione builtin)

Corpus mirato `ext/standard/tests/{strings,array}` sulle nuove funzioni: ogni
builtin dello step è byte-corretto. L'esecuzione ha però fatto **diventare
runnable** test prima skippati (le funzioni non esistevano), esponendo due
divergenze pre-esistenti del subsystem stringhe — **generali, non
builtin-specifiche**:

| ID | Severità | Categoria | Divergenza | Causa | Stato |
|---|---|---|---|---|---|
| D-NEW-14 | media | stringhe/interpolazione | gli escape (`\n` `\t` `\$` `\\` `\x..` `\0..` `\u{..}`) nei segmenti **literal** di una stringa interpolata venivano emessi grezzi (`echo "x $v\n"` stampava un backslash-n) | step 25 lowerava `CompositeString` ma usava `StringPart::Literal.value` (raw da mago) senza unescaping | **FIXATO** (29-4): `unescape_double_quoted()` in `lower.rs` processa il set double-quoted su ogni literal; i literal non interpolati arrivano già unescaped da mago. Regressione `interp_processes_escapes_in_literals` |
| D-NEW-15 | media | stringhe/heredoc | il corpo di un heredoc conserva la **newline finale** prima del marker di chiusura (`"Hello\n"` invece di `"Hello"`); causa residua dei diff `strrev_basic`/`array_fill_basic` | lowering heredoc (era 25) non strippa la newline terminale | **FIXATO** (step 30): `lower_document()` replica il lexer (dedent indentazione marker + strip newline finale; heredoc interpola+escape con `\"` letterale, nowdoc verbatim). Corregge anche la regressione 29-4 sul nowdoc. `strrev_basic`/`array_fill_basic` passano; `Zend/tests/heredoc_nowdoc` 7/7 runnable |

Non-bug catalogato: `array_search.phpt` diverge perché la EXPECTF codifica il
byte NUL nella chiave come placeholder (`a%0b`) mentre l'output reale ha il NUL
— artefatto del rendering dell'harness, non una divergenza del porting.

## Divergenze attese (scope-out dichiarato in 02-mapping-table.md)

Non sono bug, sono confini del Tier 1:
- messaggi di parse error (front-end mago ≠ Bison) → skip-list nel phpt-runner
- path `strcoll` locale-dipendente nei confronti di stringhe
- riferimenti, oggetti, resource negli operatori

## Macro-step 34 (DateTime/date) — scope-out dichiarati

Confini decisi dal Decider (D-DT1..5, vedi `NEXT-datetime-macro-step.md` e la
sezione Macro-step 34 di `metrics.md`). Non sono bug:

| Area | Comportamento nostro | PHP | Motivo |
|---|---|---|---|
| **Timezone** (D-DT3) | solo UTC; `e`/`T`→"UTC", `O`/`P`→"+0000"/"+00:00", `Z`→0, `I`→0; `date_default_timezone_set` no-op | tz-database completo + DST + abbreviazioni ("GMT","CET",…) | il tz-db è enorme; i test usano quasi sempre UTC |
| **`now`/`time()`** (D-DT5) | leggono l'orologio reale (`SystemTime`) → non-deterministici | idem | non riproducibili nel differential → testati solo i path con input esplicito |
| **`strtotime`** (D-DT4) | subset: `@N`, `now`, ISO/`Y/m/d`[+time], relativi `[+-]N unit` | parser vastissimo (relativi testuali, formati esotici, locale) | il parser completo è un sotto-progetto a sé |
| **`createFromFormat`** | subset di format char espliciti | tutti i char + opzioni avanzate | sufficiente per l'uso comune |
| **API procedurale** | solo OOP (`DateTime`/`DateTimeImmutable`/`DateInterval`) | anche `date_create`/`date_diff`/`date_format`/`getdate`/`localtime`/`strftime`/… | l'OOP è il cuore; il procedurale è zucchero |
| **`DateTimeZone`** | non implementato (`getTimezone`/`getOffset`) | classe completa | dipende dal tz-db |
| **var_dump/print_r/serialize** degli oggetti Date* | rappresentazione interna diversa (`$__ts` privato; DateInterval senza `from_string`) | `date`/`timezone_type`/`timezone`; DateInterval con `from_string` | si testano metodi e `format()`, non il dump |

Corpus `ext/date/tests`: 37 pass / 155 fail / 497 skip (192 runnable). I 155
fail ricadono **tutti** nelle righe sopra (campionati e verificati: nessun bug
di logica nelle funzioni implementate).

## Macro-step 35 (API procedurale date) — zero D-NEW

Lo step 35 implementa l'API procedurale (riga "API procedurale" sopra: ora
**implementata** come wrapper-prelude + `getdate`/`localtime` builtin puri).
Corpus `ext/date/tests` risalito a **46 pass / 178 fail / 465 skip** (224
runnable, +32): le funzioni ora definite rendono *raggiungibili* test prima
skippati, +9 passano, gli altri ricadono nelle stesse righe scope-out sopra —
**nessuna divergenza nuova**. Due casi specifici verificati:

| Test | Causa | Riga scope-out |
|---|---|---|
| `getdate_basic` (e variazioni) | setta `Asia/Calcutta` (+5:30), si aspetta `hours/minutes` locali; noi UTC | Timezone (D-DT3) |
| `date_interval_create_from_date_string` su `'1 year + 1 day'` | token connettore `+` fuori dal subset `strtotime` (i 4 casi senza `+` passano) | `strtotime`/relativi (D-DT4) |
| `date_create_basic`, `date_modify-*` | `var_dump` della rappresentazione interna degli oggetti Date* + parsing esotico nel costruttore | var_dump oggetti Date* |
| `strftime`/`gmstrftime` | deprecate in PHP 8.1, fuori scope | (scope-out esplicito) |
