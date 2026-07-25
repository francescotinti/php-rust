# REPORT_GAP_40 — gap perf oracle↔phpr della sessione WP-40

> Convenzione (decisione utente 2026-07-25): questo file contiene SOLO le
> misure della sessione WP-40; il trend cumulativo vive in `GAP_TREND.md`.
> ⚠️ confrontare RAPPORTI, mai tempi assoluti di giornate diverse.

## Metodo di misura
1. **Media group**: oracle 1 run `/usr/bin/time -l` (DB reset + uploads
   azzerati, MIMALLOC_PURGE_DELAY=0) vs phpr → rapporto **user CPU** e
   **peak footprint**.
2. **Full-suite**: CPU del processo master phpr dal tail del `.rss` della
   runN di sessione vs oracle (baseline 5:39) → rapporto; wall indicativo.

## Misure della sessione
- **media CPU (phpr/oracle)**: 56,05/20,95 = **2,68×** ⭐ (GC marks
  in-object; old stesso-giorno 57,52 = 2,75×)
- **media footprint**: non rimisurato (maxrss MADV-inquinato; Object
  +8B/istanza ≈ +20MB teorici su picco multi-GB)
- **full-suite master-CPU**: ~11:39/5:39 = **2,06×**
- **full-suite wall**: ~16,6/11,5 min = **1,4×**
