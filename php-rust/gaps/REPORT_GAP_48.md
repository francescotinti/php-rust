# REPORT_GAP_48 — gap perf oracle↔phpr della sessione WP-48

> Convenzione (decisione utente 2026-07-25): questo file contiene SOLO le
> misure della sessione WP-48; il trend cumulativo vive in `GAP_TREND.md`.
> ⚠️ confrontare RAPPORTI, mai tempi assoluti di giornate diverse.

## Metodo di misura
1. **Media group**: oracle 1 run `/usr/bin/time -l` (DB reset + uploads
   azzerati, MIMALLOC_PURGE_DELAY=0) vs phpr → rapporto **user CPU** e
   **peak footprint**.
2. **Full-suite**: CPU del processo master phpr dal tail del `.rss` della
   runN di sessione vs oracle (baseline 5:39) → rapporto; wall indicativo.

## Misure della sessione
- **media CPU (phpr/oracle)**: 61,05/20,86 = **2,93×** di giornata (+0,04%
  vs old ce4be6e stesso-giorno 61,02 = RUMORE, round 2/6; **guardia CPU
  ≤+0,5% della roadmap Fase 1 rispettata — prima leva memoria a costo CPU
  zero**; oracle 20,79-20,93)
- **media footprint**: **1,747/0,395 = 4,42×** peak fisico (old di giornata
  1,756 = 4,45×, −0,52%; oracle di giornata basso 0,394-0,396 vs 0,41 WP-47
  — confrontare i rapporti con cautela) — **Fase 1.1 shrink unit LANDED col
  mechanism-check più pulito della roadmap: canale unit −70.625.796 byte =
  il 100,0% ESATTO dello slack cap−len predetto dal contatore `unit_slack`**
  (299,9→229,3MB, 2046 moduli; slack = 23,5% del canale, il resto è bytecode
  len reale); il peak fisico ne vede ~9MB perché il picco è mid-run coi
  moduli non ancora tutti caricati (proxy-peak census −67,2MB conferma il
  meccanismo pieno)
- **full-suite master-CPU**: run35 ≈19:10/5:39 = **3,39×** (≈ run34 3,42×:
  nessuna leva CPU applicata, rumore) — **ATTRIBUZIONE CPU CHIUSA (Ob.2):
  `classify_ms` 494,9s ≈ 8:15 su 1005 collect / 9,93M root / 14,16M freed =
  l'INTERO residuo vs 2,06× WP-40 (≈7:41); reflect-cache cap 8192 in thrash
  (890k miss / 16% hit / 108 eviction) ma SECONDARIO (ordine 10× sotto)** —
  fail-set **BYTE-IDENTICO a run33** (88 nomi; 30.472 test 0E/2F/86W/73S);
  RSS transiente mid-run 5431MB "invariato da run34" (⚠️ poi riclassificato
  in WP-50 come artefatto di lettura cross-segmento del file .rss in append)
- **full-suite wall**: ~24/11,5 min = **2,1×**
