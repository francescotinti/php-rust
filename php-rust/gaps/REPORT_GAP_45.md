# REPORT_GAP_45 — gap perf oracle↔phpr della sessione WP-45

> Convenzione (decisione utente 2026-07-25): questo file contiene SOLO le
> misure della sessione WP-45; il trend cumulativo vive in `GAP_TREND.md`.
> ⚠️ confrontare RAPPORTI, mai tempi assoluti di giornate diverse.

## Metodo di misura
1. **Media group**: oracle 1 run `/usr/bin/time -l` (DB reset + uploads
   azzerati, MIMALLOC_PURGE_DELAY=0) vs phpr → rapporto **user CPU** e
   **peak footprint**.
2. **Full-suite**: CPU del processo master phpr dal tail del `.rss` della
   runN di sessione vs oracle (baseline 5:39) → rapporto; wall indicativo.

## Misure della sessione
- **media CPU (phpr/oracle)**: non rimisurato (nessuna leva CPU; binario di
  parità = WP-43)
- **media footprint**: **11,9× su PEAK FOOTPRINT FISICO** (4,67-4,78G vs
  0,39-0,40G, 18+3 run; `/usr/bin/time -l` "peak memory footprint" = nuova
  metrica ufficiale, MADV-immune) — **ATTRIBUITO (Fase 0 roadmap)**: ~3,08G
  cicli Rc irraggiungibili (gc_note object-only: cicli Ref/Array/Closure mai
  rooted) + ~1,2G non censito (rounding/tabelle) + 0,30G unit ritenute +
  0,15G stato PHP. Bersaglio WP-46: root-tracking esteso modello Zend
- **full-suite master-CPU**: full non rilanciata (nessun cambio al binario
  di parità) → riferimento **2,06×**
- **full-suite wall**: invariato **1,4×**
