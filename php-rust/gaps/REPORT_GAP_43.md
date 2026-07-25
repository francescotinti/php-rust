# REPORT_GAP_43 — gap perf oracle↔phpr della sessione WP-43

> Convenzione (decisione utente 2026-07-25): questo file contiene SOLO le
> misure della sessione WP-43; il trend cumulativo vive in `GAP_TREND.md`.
> ⚠️ confrontare RAPPORTI, mai tempi assoluti di giornate diverse.

## Metodo di misura
1. **Media group**: oracle 1 run `/usr/bin/time -l` (DB reset + uploads
   azzerati, MIMALLOC_PURGE_DELAY=0) vs phpr → rapporto **user CPU** e
   **peak footprint**.
2. **Full-suite**: CPU del processo master phpr dal tail del `.rss` della
   runN di sessione vs oracle (baseline 5:39) → rapporto; wall indicativo.

## Misure della sessione
- **media CPU (phpr/oracle)**: 55,70/20,77 = **2,68×** (stadio 1 registri =
  infra spenta, A/B 6 round RUMORE ZERO vs old 56,38 = 2,71×; in linea col
  riferimento WP-40/41)
- **media footprint**: 3,73/0,44GB = **8,5×** raw maxrss ⚠️ non comparabile
  alle sessioni vicine: oracle di giornata alto (0,44 vs 0,37) e maxrss
  new/old MADV-rumorosi (old 3,31 con outlier 2,10; old==new entro rumore) —
  il riferimento strutturale resta ~12×
- **full-suite master-CPU**: full-suite NON rilanciata (delta zero provato:
  bytecode byte-id + gate22 verde + A/B flat; run32 resta baseline) →
  riferimento resta **2,06×**
- **full-suite wall**: invariato **1,4×**
