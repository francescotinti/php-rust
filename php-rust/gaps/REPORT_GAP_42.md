# REPORT_GAP_42 — gap perf oracle↔phpr della sessione WP-42

> Convenzione (decisione utente 2026-07-25): questo file contiene SOLO le
> misure della sessione WP-42; il trend cumulativo vive in `GAP_TREND.md`.
> ⚠️ confrontare RAPPORTI, mai tempi assoluti di giornate diverse.

## Metodo di misura
1. **Media group**: oracle 1 run `/usr/bin/time -l` (DB reset + uploads
   azzerati, MIMALLOC_PURGE_DELAY=0) vs phpr → rapporto **user CPU** e
   **peak footprint**.
2. **Full-suite**: CPU del processo master phpr dal tail del `.rss` della
   runN di sessione vs oracle (baseline 5:39) → rapporto; wall indicativo.

## Misure della sessione
- **media CPU (phpr/oracle)**: 57,74/20,93 = **2,76×** ⚠️ giornata rumorosa:
  old stesso-giorno 57,56 = 2,75× → warm-up by-borrow **FLAT su 6 round**
  (keep, leva chiusa); riferimento resta ~2,7×
- **media footprint**: 4,75/0,37GB = **12,7×** raw maxrss (old==new nei 6
  round; oracle di giornata basso 0,36-0,38)
- **full-suite master-CPU**: run32 ~12:50 NOMINALE stessa giornata rumorosa;
  fail-set **byte-id a run31** → riferimento resta **2,06×** (11:39 WP-40)
- **full-suite wall**: invariato **1,4×**
