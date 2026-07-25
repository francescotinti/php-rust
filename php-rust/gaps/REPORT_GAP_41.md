# REPORT_GAP_41 — gap perf oracle↔phpr della sessione WP-41

> Convenzione (decisione utente 2026-07-25): questo file contiene SOLO le
> misure della sessione WP-41; il trend cumulativo vive in `GAP_TREND.md`.
> ⚠️ confrontare RAPPORTI, mai tempi assoluti di giornate diverse.

## Metodo di misura
1. **Media group**: oracle 1 run `/usr/bin/time -l` (DB reset + uploads
   azzerati, MIMALLOC_PURGE_DELAY=0) vs phpr → rapporto **user CPU** e
   **peak footprint**.
2. **Full-suite**: CPU del processo master phpr dal tail del `.rss` della
   runN di sessione vs oracle (baseline 5:39) → rapporto; wall indicativo.

## Misure della sessione
- **media CPU (phpr/oracle)**: 56,08/20,91 = **2,68×** (invariato: shim
  gc_note BOCCIATO su A/B 4 round, +0,62% consistente → revert; A/B odierno
  old 56,08 vs new 56,43)
- **media footprint**: non rimisurato (zero delta codice)
- **full-suite master-CPU**: ~11:39/5:39 = **2,06×** (run31 resta baseline:
  zero delta codice)
- **full-suite wall**: invariato **1,4×**
