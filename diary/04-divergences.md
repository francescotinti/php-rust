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

## Step 36 (preg backref/lookaround) — scope-out dichiarati + 1 divergenza nuova

L'auto-fallback `regex`→`fancy-regex` (vedi `metrics.md` § Step 36) copre
backref, lookaround, atomic/possessive group e — scoperta 36-2 — anche `(?R)`,
conditional, `\K`, `\G`. I confini sotto sono decisi dal Decider (D-36.1..4);
non sono bug salvo D-36.4.

| Area | Comportamento nostro | PHP | Motivo |
|---|---|---|---|
| **subroutine** `(?1)`/`(?&name)` (D-36.2) | nessun engine compila → `preg_*` ritorna `false`/`null` | match | né `regex` né `fancy-regex` 0.14 le supportano |
| **control verb** `(*SKIP)`/`(*FAIL)`/`(*PRUNE)` (D-36.2) | idem `false`/`null` | match / controllo backtracking | idem |
| **callout** `(?C1)` (D-36.2) | idem `false`/`null` | callback PCRE | idem |
| **`preg_last_error`** (D-36.3) | sempre `PREG_NO_ERROR`; errore runtime fancy (limite backtracking) → no-match silenzioso | codici d'errore distinti | il funnel preg non propaga i codici |
| **flag PCRE `U`/`D`/`A`/`X`** | ~~ignorati~~ → **implementati allo step 37** | ungreedy / dollar-end-only / anchored / extra | vedi sezione Step 37 |
| **trailing capture trimming** (pre-esistente) | gruppi trailing non-partecipanti inclusi | omessi (`preg_match_non_capture.phpt`) | dettaglio `PREG_*` da step 31 |

**D-36.4 (divergenza di VALORE, ex-hang risolto in 36-3) — backtracking
catastrofico.** `bug41638.phpt`: il pattern `(['"])((.*(\\\1)*)*)\1` (backref +
quantificatori annidati + flag `U` ungreedy) su `fancy-regex` (NFA). Step 36-1
**introdusse un hang**: prima il crate `regex` non compilava il pattern
(no-match, nessun hang); col fallback fancy compila e — siccome non onoriamo il
flag `U`, la `.*` resta greedy — esplode. Diagnosi 36-3: il `backtrack_limit` di
default (1M) *già* limita la singola attempt (~200 ms → `Err`), ma
`captures_iter` (path di `preg_match_all`) non avanza oltre una posizione che
erra → emette lo stesso `Err` all'infinito (loop nel nostro `filter_map`), e
`fancy replace_all` (`try_replacen().unwrap()`) **panica** sull'errore.
**Fix 36-3**: `captures_iter` si ferma al primo `Err`, `replace_all` usa
`try_replacen` con fallback al testo invariato, `backtrack_limit` fissato a 1M
esplicito. Ora il pattern **ritorna** in ~200 ms (`preg_match_all`→`0`,
`preg_replace`→subject invariato) invece di appendere/panicare. **RISOLTO allo step 37-1**: onorando il flag `U`, `.*` diventa lazy → il pattern
non è più catastrofico e matcha PHP byte-per-byte (`bug41638.phpt` passa). Il
guard anti-hang 36-3 resta testato con un pattern catastrofico **senza** `U`.

Corpus `ext/pcre/tests`: **41 pass / 42 fail / 82 skip / 0 timeout** (83
runnable, dopo 36-3; prima 41/41/82 + 1 timeout `bug41638`), +3 pass vs step 31.
I fail sono lo scope-out delle righe sopra + formattazione warning/NUL: nessuna
regressione (i pass salgono).

## Step 37 (flag PCRE U/A/X/D) — implementati; 1 nota prestazionale

`compile()` ora onora i flag che ignorava. **Nessuna divergenza di correttezza
nuova**; tutti oracle-verificati. Una sola nota (non un bug):

| Flag | Implementazione | Note |
|---|---|---|
| **`U`** ungreedy | `regex` `swap_greed(true)` + inline `(?U)` | `?` esplicito re-inverte; risolve D-36.4 |
| **`A`** anchored | wrap body `\A(?:…)` su entrambi gli engine | numerazione gruppi invariata |
| **`X`** extra | no-op esplicito | deprecato in PCRE2; NON è `x` (extended) |
| **`D`** dollar-endonly | `$` resta `\z`-stretto (= nostro default storico) | vedi D-37.1 |

**D-37.1 (nota prestazionale, non divergenza).** Il `$` di default di PCRE
(senza `m`/`D`) è zero-width e matcha a fine subject **o prima di un `\n`
finale**; il `$` del crate `regex` è `\z`-only. Per la correttezza byte-esatta,
quando non c'è `m`/`D` ogni `$` bare è riscritto nel lookahead `(?=\n?\z)`
(`rewrite_dollar_anchor`, salta `\$` e `$` in `[...]`). Il lookaround non ha
equivalente DFA → l'auto-fallback dello step 36 instrada questi pattern a
`fancy-regex`: **i pattern con `$` (e senza `m`/`D`) perdono il fast-path DFA**.
Accettato (Decider D-37.1): correttezza > velocità per questo esperimento; i
pattern senza `$` o con `m`/`D` restano sul motore `regex` veloce. Limite noto
dello scanner: un `$` dentro una classe POSIX `[[:alpha:]$]` non è gestito (caso
rarissimo) — il `]` di `:alpha:]` chiude prematuramente la classe.

Corpus `ext/pcre/tests`: **44 pass / 39 fail / 82 skip / 0 timeout**, +3 vs step
36 (`dollar_endonly`/`ungreedy`/`bug41638`).

## Step 38 (argomenti nominati) — coperto il caso comune; follow-up dichiarati

`nullsafe ?->` era già completo (step 19). I named arguments by-value sono
implementati per funzioni/costruttori/metodi/static e oracle-verificati. I
confini sotto sono scope-out dichiarati (Decider), **non** bug; il corpus
`Zend/tests/named_params` (4/12/17) li espone.

| Area | Comportamento nostro | PHP | Stato |
|---|---|---|---|
| **named → parametro by-ref** `f(&$a)` (D-38.3) | **FATTO step 38-4** (variabile→cella); `id($a)` by-ref-return→by-ref param resta limite pre-esistente | passa la cella per riferimento | parziale (fail `basic.phpt` SEND_REF su by-ref-return) |
| **parametri variadic** `function f(...$rest)` | **FATTO step 38-5** (positional collection) | raccoglie posizionali+nominati extra | named-into-variadic = follow-up |
| **spread di chiamata** `f(...$arr)` (D-38.4) | `Unsupported` al lowering | espande l'array in argomenti (int→posizionali, string→nominati) | follow-up |
| **named ai builtin** (D-38.2) | `Error` "named arguments to builtin … not supported" | supportati (PCRE2/Zend hanno i nomi) | scope-out (registry senza nomi) |
| **named a closure-as-value** `$f(x:1)` | non instradato (CallDynamic) | supportato | follow-up |
| **attributi con named** `#[A(x: 1)]` | attributi non supportati | — | dipende da attributes/Reflection |

I casi coperti (riordino, posizionale+nominato, default saltati, "Unknown named
parameter $x", "Named parameter $x overwrites previous argument", "Cannot use
positional argument after named argument" compile-fatal, costruttore, metodo,
static) sono tutti oracle-verificati nei test unit.

## Step 39 (generatori `yield`) — coperto il core; throw/by-ref/finally scope-out

`yield`, `yield $k=>$v`, `yield from` (array + sub-generatore), `send()`,
`return`/`getReturn()`, metodi Iterator, `foreach` su Generator, instanceof
Generator/Iterator/Traversable, rewind, var_dump/print_r sono implementati e
oracle-verificati. I confini sotto sono **scope-out dichiarati** (D-GEN-4), non
bug; il corpus `Zend/tests/generators` (59/51/74) li espone.

| Area | Comportamento nostro | PHP | Stato |
|---|---|---|---|
| **`Generator::throw()`** + eccezione iniettata in un `yield` | metodo assente → Error | rilancia l'eccezione dal punto di sospensione | scope-out D-GEN-4 |
| **eccezione che attraversa `yield`/`finally`** dentro il generatore | `finally` nel generatore non interagisce con l'unwinding di sospensione | unwinding completo con esecuzione `finally` | scope-out D-GEN-4 |
| **yield by-reference** `function &g(){ yield $x; }` | `&` ignorato (yield by-value) | l'elemento è un alias della cella | scope-out D-GEN-4 |
| **errore resume/rewind** ("Cannot resume…/rewind…") | `PhpError::Error` (classe `Error`, fatale) | lancia un'**`Exception`** catchabile | divergenza classe (Exception vs Error): i test `errors/resume_running_*`, `generator_rewind` falliscono perché fanno `catch(Exception)` |
| **numerazione handle oggetto** in dump con generatori | il generatore consuma `next_object_id` → gli `#N` di stdClass successivi divergono | numerazione PHP | informational (pre-esistente, vale anche per closure) |
| **stack trace** di errori da `->next()`/`->current()` | frame `#0 {main}` (il frame `Generator->next()` interno non è tracciato) | `#0 …: Generator->next()` | fedeltà trace, scope-out |
| **`yield from` su Iterator/Traversable user** | `Error` "Can use \"yield from\" only with arrays and Traversables" | itera l'oggetto | companion del `foreach`-su-oggetti (scope-out) |

Casi coperti e oracle-verificati: auto-key (anche con chiave int esplicita che
avanza il contatore con la regola array-append), `yield;` nudo (NULL/auto-key),
`yield from` che preserva le chiavi senza avanzare il contatore esterno,
getReturn del delegato, send-forwarding attraverso `yield from`, closure
generator (con cattura `use`), getReturn auto-prime su generatore che `return`
prima di ogni yield, rewind fatale dopo avanzamento.

## Step 40 (spread `f(...$arr)`) — coperto il core; 1 precedence scope-out

Implementato l'argument unpacking per Call/New/MethodCall/StaticCall + la
collezione named-into-variadic (esplicita e da spread). Tutti i comportamenti
nei test sono oracle-verificati (23 test). Una sola divergenza di **precedence**
su input doppiamente-invalido, dichiarata scope-out:

| Caso | Comportamento Rust | PHP 8.5.7 | Stato |
|---|---|---|---|
| `f(...['c'=>3], ...[1])` con `f` **senza** param `$c` né variadic (chiave string sconosciuta *seguita* da chiave int) | adotta il **two-phase** (espandi → piazza): l'int-dopo-named è rilevato durante l'espansione → `Error` "Cannot use positional argument after named argument during unpacking" | piazza i named **incrementalmente** → fallisce prima con `Error` "Unknown named parameter $c" | divergenza solo di **messaggio**, entrambi `Error` catchable, input contrived; scope-out D-40.1 |

Razionale scope-out: PHP valuta/piazza ogni elemento unpacked incrementalmente,
quindi l'unknown-named precede l'ordering-error. La reimplementazione two-phase
(più semplice e uniforme tra tutte le path di chiamata) inverte la precedenza
solo quando *entrambi* gli errori sarebbero presenti. Nei casi a errore singolo
(`f(...['z'=>1])` → "Unknown named parameter $z"; `f(1, ...['k'=>2, 0=>3])` con
variadic → ordering-error) i messaggi coincidono con l'oracle.

Altri scope-out minori (non testati, edge rari): spread-named verso classe
**senza costruttore** non solleva "Unknown named parameter" (gli arg posizionali
extra sono ignorati come da default-ctor PHP); named verso il **nome** del
parametro variadic stesso (`f(args: …)` con `function f(...$args)`) è raccolto
come chiave `'args'` invece che trattato specialmente.

Casi coperti e oracle-verificati: int→posizionale (valore chiave ignorato,
ordine d'iterazione), string→named, default su named parziali, leading
positional + spread, spread multipli, spread→variadic (re-keyed 0,1,2),
named-into-variadic (esplicito e da spread, chiave string preservata),
Traversable/generator (chiavi int e string), array vuoto, TypeError su
non-iterabile, overwrite, compile-fatals (positional-after-unpacking,
unpacking-after-named).

## Step 41 (mbstring batch 1) — UTF-8 core; divergenze su edge dichiarate

23 funzioni `mb_*` stringa UTF-8 (vedi `03-translation-log.md` + `NEXT-mbstring.md`).
18 test oracle-verificati. Comportamenti core esatti; divergenze solo su edge
dichiarati (scope-out):

| # | Caso | Comportamento Rust | PHP 8.5.7 | Stato |
|---|---|---|---|---|
| D-MB1 | `$encoding` non-UTF-8 *valido* (Shift-JIS, EUC, …) | `ValueError` "must be a valid encoding" | processa nell'encoding | scope-out (serve `encoding_rs`); per ENCODING SCONOSCIUTO il messaggio combacia |
| D-MB3a | `mb_convert_case(.., TITLE)` con punteggiatura Case_Ignorable | l'apostrofo è boundary: `o'brien`→`O'Brien` | `o'brien` (apostrofo Case_Ignorable, non boundary) | scope-out (no tabelle Case_Ignorable); boundary su spazio/trattino/cifra è esatto |
| D-MB3b | `MB_CASE_FOLD` su char con fold ≠ lowercase (es. ß) | `to_lowercase` (ß→ß) | full case-fold (ß→ss) | scope-out (approssimazione) |
| D-MB3c | `*_SIMPLE` (mode 4-7) su char con espansione (ß) | trattati come le versioni full (ß→SS) | mapping 1:1 senza espansione (ß→ß) | scope-out (raro) |
| D-MB-rpos | offset ≠ 0 su `mb_strrpos`/`mb_strripos` | ignorato (cerca tutta la stringa) | offset onorato | scope-out batch 1 (il default è il caso comune) |
| D-MB-ci | ricerca case-insensitive con fold length-changing (İ, final-sigma) | fold semplice (primo char del lowercase) | fold pieno | scope-out (raro) |
| D-MB-inv | rendering di byte UTF-8 invalidi in substr/case | byte copiati verbatim / U+FFFD per unità | sostituzione mbstring | scope-out (il CONTEGGIO/offset è corretto: `mb_strlen("a\xFF\xFEb")==4`) |

Casi coperti e oracle-verificati: conteggio code-point (incl. combining +
byte invalidi), substr/str_split (negativi, len omessa, chunk), case full-Unicode
(`ß→SS`/`ı→I`/`İ→i̇`/final-sigma/greco), TITLE (boundary spazio/trattino/cifra),
ucfirst/lcfirst, strpos family (offset code-point, neg, empty needle, miss→false,
case-insensitive), strstr/strrchr family (before_needle, needle intero, last
occurrence), substr_count non-overlapping, ord/chr (surrogate/range→false),
str_pad (LEFT/RIGHT/BOTH, pad ciclato), trim/ltrim/rtrim (default + charlist),
check_encoding (UTF-8 valido/invalido), `ValueError` su encoding sconosciuto /
mode invalido / needle vuoto / stringa vuota in ord.

**Corpus** `ext/mbstring/tests`: 417/420 SKIP categoria "section" (il phpt-runner
scarta `--EXTENSIONS--`/`--SKIPIF--`/`--INI--`). NON è regressione: validazione
via unit test. Rilassare `--EXTENSIONS--` = item tooling cross-cutting separato.

## Step 42 (mbstring batch 2A) — encoding + width; divergenze dichiarate

5 funzioni (`mb_convert_encoding`/`mb_detect_encoding` via `encoding_rs`;
`mb_strwidth`/`mb_strimwidth`/`mb_strcut` via tabella EAW portata). +8 test
oracle-verificati. Comportamenti core esatti; divergenze solo su scelte di
scope dichiarate:

| # | Caso | Comportamento Rust | PHP 8.5.7 | Stato |
|---|---|---|---|---|
| D-MB-enc-latin1 | `ISO-8859-1`/`latin1` | true Latin-1 hand-rolled (byte ↔ code point ≤0xFF; `\x80`→U+0080) | identico | parità ✓ — `encoding_rs` mapperebbe `iso-8859-1`→windows-1252 (WHATWG), deliberatamente NON usato |
| D-MB-enc-subst | char non rappresentabile nel target (`€`→ISO-8859-1) | sostituito con `?` (0x3F) | `?` (substitute char mbfl) | parità ✓ — NON entità HTML (che `encoding_rs::encode` emetterebbe) |
| D-MB-enc-utf16 | `UTF-16`/`UTF-16BE`/`UTF-16LE` come target | hand-rolled (`encoding_rs` non *codifica* UTF-16); `UTF-16` nudo = BE | identico | parità ✓ |
| D-MB-enc-list | nomi encoding supportati | sottoinsieme risolvibile (UTF-8/16, ASCII, Latin1, + label `encoding_rs`); `mb_list_encodings`/`mb_encoding_aliases` non implementate | mbfl elenca ~79 nomi | scope-out (nessun driver dal corpus) |
| D-MB-enc-htmlent | `HTML-ENTITIES`/`BASE64`/`UUENCODE`/7bit/8bit/JIS/UTF-7 | `ValueError`/`contains invalid encoding` | encoding speciali mbfl | scope-out (deprecate in 8.5) |
| D-MB-enc-detect | `mb_detect_encoding` non-strict | primo candidato valido, else primo candidato (mai false) | euristica mbfl (best-fit per # errori) | approssimazione; tutti i casi sondati combaciano |
| D-MB-width-eaw | `mb_strwidth` | tabella EAW portata verbatim; width=2 solo Wide/Fullwidth, 1 altrimenti (combining/ZW/control inclusi) | identico | parità ✓ — `unicode-width` (che dà 0 a combining/ZW) deliberatamente NON usato |
| D-MB-width-enc | `$encoding` ≠ UTF-8 su `mb_strwidth`/`strimwidth`/`strcut` | `ValueError` (coerente con D-MB1) | transcodifica nell'encoding | divergenza dichiarata (le funzioni width restano UTF-8-only in questo batch) |
| D-MB-strimwidth-neg | `width` negativo (deprecato) | nessun char preso (ritorna marker/empty) | E_DEPRECATED + calcolo | scope-out (non nel corpus) |

Casi coperti e oracle-verificati: `mb_strwidth` (ASCII=1, CJK/emoji/fullwidth/
Hangul=2, halfwidth/combining/ZWSP/ambiguous=1, byte invalidi=1 cad.);
`mb_strimwidth` (start in code-point, marker conta verso il limite, tail che ci
sta → no marker, marker più largo del limite → solo marker, start negativo,
`start==len`→empty, out-of-range→`ValueError`); `mb_strcut` (offset byte, mai
spezza un char, start arrotonda giù, length dal rounded start, oltre fine→empty);
`mb_convert_encoding` (UTF-8↔ISO-8859-1/Windows-1252/SJIS/UTF-16BE/LE, from null,
from-detect-list, substitute `?`, errori to/from); `mb_detect_encoding` (default
[ASCII,UTF-8], comma-list/array, nomi canonici, strict→false, fallback non-strict,
liste vuote/invalide→`ValueError`).

D-NEW: nessuna (i fail sono scelte di scope dichiarate, non bug). `bin2hex`/
`var_export`/`mb_list_encodings` NON sono builtin implementati → i test usano
output byte-esatti / `var_dump` / echo diretto.

## Step 43 (mbstring batch 2B) — famiglia regex `mb_ereg*` via oniguruma

~16 funzioni `mb_ereg*`/`mb_split`/`mb_regex_*` come adapter su oniguruma reale
(crate `onig`). +9 test oracle-verificati. Primo step mbstring con modifiche
all'evaluator (stato persistente + out-param arg #3). Divergenze dichiarate:

| # | Caso | Comportamento Rust | PHP 8.5.7 | Stato |
|---|---|---|---|---|
| D-MB-ereg-enc | `mb_regex_encoding` ≠ UTF-8 | i pattern/subject sono trattati come UTF-8; il setter memorizza ma non transcodifica | mbregex nell'encoding scelto | scope-out (coerente con D-MB1) |
| D-MB-ereg-syntax | opzioni avanzate / selettori syntax non `r`/default | `parse_options` mappa i/x/m/s/p/l/n + r/z/d/b/j/u/g/c; combinazioni esotiche non validate a fondo | tutte le opzioni mbregex | parità sui casi comuni; resto non verificato |
| D-MB-ereg-build | dipendenza C oniguruma (vendored via `onig_sys`+`bindgen`) | compila in ambiente (clang); fallback dichiarato fancy-regex NON attivato | usa liboniguruma di sistema | parità funzionale ✓ (build C, non più pure-Rust) |

Casi coperti e oracle-verificati: `mb_ereg`/`mb_eregi` (return bool, `$regs` arg
#3 by-ref con gruppi numerati + `false` per non-partecipante + named per chiave
stringa, no-match→false+`[]`), backref nel pattern `\1`, named group `(?<n>)`,
`mb_ereg_replace`/`mb_eregi_replace` (backref `\0`-`\9` nel replacement),
`mb_ereg_replace_callback` (callable, multi-match, aritmetica), `mb_split`
(campi vuoti, limite, no-match→stringa intera), `mb_ereg_match` (ancorato
all'inizio, classi POSIX `[[:digit:]]`, `.` matcha newline per opzione `p`),
`mb_regex_encoding`/`mb_regex_set_options` (default "UTF-8"/"pr"), pattern
invalido→false+Warning, e l'intera famiglia stateful `mb_ereg_search_*` (cursore
byte, walk multi-match, getregs/getpos/setpos).

D-NEW: nessuna. `bin2hex`/`var_export` non implementati → i test usano echo
diretto / `var_dump` / `implode`. Nessun CLI standalone (`php-cli` è stub) →
validazione differenziale via unit test oracle-derivati + probe oracle manuali.
