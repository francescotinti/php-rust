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

## Divergenze attese (scope-out dichiarato in 02-mapping-table.md)

Non sono bug, sono confini del Tier 1:
- messaggi di parse error (front-end mago ≠ Bison) → skip-list nel phpt-runner
- path `strcoll` locale-dipendente nei confronti di stringhe
- riferimenti, oggetti, resource negli operatori
