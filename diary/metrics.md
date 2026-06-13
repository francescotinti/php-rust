# Metriche dell'esperimento

> Generato con assistenza AI (Claude Fable 5). Aggiornato: 2026-06-13 (fine step 2).

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
| Unit/integration (php-types) | 24 |
| Differential vs oracle | 37.835 casi, 0 mismatch |

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
| **Totale a fine step 2** | **~4.5h** |
