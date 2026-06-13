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

## Divergenze attese (scope-out dichiarato in 02-mapping-table.md)

Non sono bug, sono confini del Tier 1:
- messaggi di parse error (front-end mago ≠ Bison) → skip-list nel phpt-runner
- path `strcoll` locale-dipendente nei confronti di stringhe
- riferimenti, oggetti, resource negli operatori
