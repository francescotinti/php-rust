# REPORT_GAP_44 — gap perf oracle↔phpr della sessione WP-44

> Convenzione (decisione utente 2026-07-25): questo file contiene SOLO le
> misure della sessione WP-44; il trend cumulativo vive in `GAP_TREND.md`.
> ⚠️ confrontare RAPPORTI, mai tempi assoluti di giornate diverse.

## Metodo di misura
1. **Media group**: oracle 1 run `/usr/bin/time -l` (DB reset + uploads
   azzerati, MIMALLOC_PURGE_DELAY=0) vs phpr → rapporto **user CPU** e
   **peak footprint**.
2. **Full-suite**: CPU del processo master phpr dal tail del `.rss` della
   runN di sessione vs oracle (baseline 5:39) → rapporto; wall indicativo.

## Misure della sessione
- **media CPU (phpr/oracle)**: 55,21/20,785 = **2,66×** (= old e3c8e0b su
  giornata pulitissima, oracle 20,74-20,80 su 6 serie; **stadio 2 registri
  BOCCIATO in TRE forme e REVERTATO**: +1,17% v1 enum / +1,28% v2
  enum-singola-risoluzione / +1,01% v3 raw-monomorfa, 18/18 round new>old →
  main resta il binario WP-43; epitaffio: contano i CORPI HANDLER CALDI, non
  lo stile di estrazione)
- **media footprint**: 4,78/0,476GB = **10,0×** raw maxrss (old di giornata;
  MADV-rumoroso, riferimento strutturale resta ~12×)
- **full-suite master-CPU**: full-suite NON rilanciata (tree revertato
  BYTE-IDENTICO a e3c8e0b già gated; run32 resta baseline) → riferimento
  resta **2,06×**
- **full-suite wall**: invariato **1,4×**
