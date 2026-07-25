# REPORT_GAP_47 — gap perf oracle↔phpr della sessione WP-47

> Convenzione (decisione utente 2026-07-25): questo file contiene SOLO le
> misure della sessione WP-47; il trend cumulativo vive in `GAP_TREND.md`.
> ⚠️ confrontare RAPPORTI, mai tempi assoluti di giornate diverse.

## Metodo di misura
1. **Media group**: oracle 1 run `/usr/bin/time -l` (DB reset + uploads
   azzerati, MIMALLOC_PURGE_DELAY=0) vs phpr → rapporto **user CPU** e
   **peak footprint**.
2. **Full-suite**: CPU del processo master phpr dal tail del `.rss` della
   runN di sessione vs oracle (baseline 5:39) → rapporto; wall indicativo.

## Misure della sessione
- **media CPU (phpr/oracle)**: 63,60/21,21 = **3,00×** di giornata ⚠️ (+1,3%
  vs old e6af390 stesso-giorno 62,77 = 2,96×, 6/6 round new>old, TENUTO per
  direttiva: la ricostruzione dei descrittori evitti costa più di quanto il
  classify 2-passate recuperi sul media; giornata lenta, oracle 21,12-21,30
  — il riferimento comparabile resta ~2,85× WP-46)
- **media footprint**: **1,77/0,41GB = 4,3×** ⭐⭐⭐ peak fisico (old di
  giornata 4,88-4,92G = 11,9×: **−63,8%, il più grande salto footprint della
  storia del progetto**) — attribuzione owner-level RIUSCITA: root-walk
  esteso a tutti i campi Vm riconcilia arr 100,0%/str 96,7%, owner =
  `reflect_method_info_cache` 456k entry/2,48G (mock PHPUnit = ClassId
  freschi, memo mai evitta) → epoch eviction cap 8192 (`fa100ad`)
- **full-suite master-CPU**: run34 ≈19:20/5:39 = **3,42×** (da 21:00 = 3,71×
  WP-46, −8% da isteresi soglia + classify 2-passate `d684cd7`; riferimento
  WP-40 2,06× non ancora recuperato) — **fail-set BYTE-IDENTICO a run33**
  (88 nomi; 30.472 test 0E/2F/86W/73S)
- **full-suite wall**: ~23/11,5 min = **2,0×**
