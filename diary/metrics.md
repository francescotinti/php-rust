# Metriche dell'esperimento

> Generato con assistenza AI (Claude Fable 5 / Opus 4.8). Aggiornato: 2026-06-13 (fine step 8).

## LOC (target Rust, escluso codice di test)

| Crate / modulo | LOC (≈) | Note |
|---|---|---|
| php-types/zstr.rs | 100 | PhpStr binary-safe + hash lazy |
| php-types/zval.rs | 80 | enum Zval |
| php-types/array.rs | 230 | PhpArray ordered hash |
| php-types/numstr.rs | 220 | is_numeric_string |
| php-types/dtoa.rs | 150 | zend_gcvt |
| php-types/convert.rs | 230 | conversioni |
| php-types/ops.rs | 620 | operatori (full port semantico) |
| php-types/diag.rs | 45 | Diag / PhpError |
| **Totale step 1–2** | **~1.675** | core types + operatori |

## Test

| Tipo | Conteggio |
|---|---|
| Unit/integration (workspace, fine step 9) | 131 (17 suite) |
| Differential vs oracle (php-types) | 37.835 casi, 0 mismatch |
| phpt-runner su testsuite PHP completa | 6172 file: 126 pass / 62 fail / 5984 skip (67.0% dei runnable) |

## phpt-runner — skip per categoria (run completo `tests/` + `Zend/tests/`, fine step 9)

| Categoria | Conteggio | Significato |
|---|---|---|
| unsupported | 5028 | confine Tier 1 (OOP, namespace, by-ref/variadic, …) — atteso |
| section | 660 | sezioni I/O/INI/SKIPIF/EXTENSIONS non modellate |
| builtin | 114 | builtin non ancora implementato (step 10) |
| compile-error | 104 | diagnostica compile-time del motore (validazione attributi/tipi, strictness parser) non modellata — **nuova** in step 9 |
| parse | 67 | sintassi che mago non parsa nel nostro path |
| malformed | 6 | `.phpt` senza FILE/EXPECT |
| expectregex | 4 | `--EXPECTREGEX--` non supportato |
| expectf-%r | 1 | placeholder `%r` non supportato |

**Step 9** ha eliminato la categoria `diag-or-fatal` (176): quei file sono ora
*runnable* (+72 netti dopo lo skip `compile-error`). Il pass-rate scende a 67.0%
perché il corpus ora **confronta** i diag-test invece di skipparli: **+12 pass**
(11 diag + 1 null-offset Classe A) e **62 fail** esposti, tutti triagiati in
`04-divergences.md` (quasi tutti scope gap di feature, non difetti di rendering).
Tra i 62 ci sono ancora i 2 fail storici D-NEW-4 (`\u{}`) e la famiglia D-NEW-6
(type-hint non enforced).

## Differential — convergenza (step 2)

| Iterazione | Mismatch |
|---|---|
| Prima implementazione | 2.711 / 37.835 |
| Dopo fix conversione int (saturazione, doppi diagnostici) | 8 / 37.835 |
| Dopo fix pow-overflow + bitnot value-name | **0 / 37.835** |

## Compressione C → Rust (parziale, solo moduli portati)

| Modulo C | LOC C | LOC Rust | Compressione |
|---|---|---|---|
| zend_operators.c (subset osservabile) | ~3.900 | ~620 (ops) + ~450 (numstr+convert) | ~73% |
| zend_strtod.c → zend_gcvt | ~120 (la sola gcvt) | ~150 | n/a (più esplicito su Ryū) |

## Divergenze catalogate (D-NEW)

Vedi `04-divergences.md`. Allo step 2: 0 divergenze residue (tutte le scoperte
del differential sono state riconciliate verso il comportamento dell'oracle).

## Tempo

| Fase | Tempo (≈) |
|---|---|
| Phase 0 + Fase 1 + Fase 2 (diary) | ~1.5h |
| Step 1 (php-types core) | ~0.5h |
| Step 2 (operatori + oracle build + differential) | ~2.5h |
| Step 3 (bridge mago→HIR) | ~0.5h |
| Step 4 (evaluator tree-walking) | ~1h |
| Step 5 (builtins registry + nucleo) | ~1h |
| Step 7 (array + foreach/switch/match) | ~2h |
| Step 6 (phpt-runner + Fase 4c import + 2 bugfix) | ~2.5h |
| Step 8 (funzioni utente + Fase 4c re-import + 1 bugfix eval-order) | ~1.5h |
| Step 9 (rendering diagnostici/fatal + skip compile-error + triage corpus + 1 fix null-offset) | ~2h |
| **Totale a fine step 9** | **~13.5h** |
