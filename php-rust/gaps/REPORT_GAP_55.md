# REPORT_GAP_55 — gap perf oracle↔phpr della sessione WP-55

> SOLO le misure della sessione WP-55; trend cumulativo in `GAP_TREND.md`.
> ⚠️ confrontare RAPPORTI, mai tempi assoluti di giornate diverse.

## Metodo di misura
1. **Media group**: oracle 1 run `/usr/bin/time -l` (DB reset + uploads
   azzerati, MIMALLOC_PURGE_DELAY=0) vs phpr → rapporto **user CPU** e
   **peak footprint**.
2. **Full-suite**: CPU del processo master phpr dal tail del `.rss` della
   runN di sessione vs oracle (baseline 5:39) → rapporto; wall indicativo.

## Misure della sessione
- **media CPU (phpr/oracle)**: mediane ab55 55,06/21,07 = **2,61×
  (invariato)** — A/B old=phpr-wp54 vs new=append-in-place: **FLAT**
  (mediane +0,16%; escludendo il round-1 di warmup −0,04%) — atteso: il
  gruppo media non è append-bound, la leva agisce sul full.
- **media footprint**: mediane 1566,0/377,0MB = **4,15×** peak fisico
  (old stessa-sera 1546,3 = 4,10×; A/B **+1,27%** = +19,7MB: 5,7MB di
  +8B/stringa (713k vive) + ~14MB slack di crescita ammortizzata dei
  buffer append-grown — **guardia ≤+2% RISPETTATA**, delta tenuto e
  verbalizzato, niente revert per direttiva).
- **full-suite master-CPU**: run44 (new) **716,9s ≈ 11:57** vs run44-old
  (phpr-wp54) **735,7s ≈ 12:16 stessa-sera = −2,56% (−18,8s)** →
  **riferimento pubblicato = run44 716,9/339 = 2,11× nuovo minimo**
  (era run43 2,12×; residuo vs WP-40 2,06× ≈ 14s). ⚠️ il pomeriggio lo
  stesso binario old aveva dato 717,3s: deriva d'ambiente ~2,6% — fa
  fede SOLO la coppia stessa-sera. Fail-set **BYTE-ID a run33** (88
  nomi) su run44 E run44-old; 30.472 test 0E/2F/86W/73S ×2.
- **full-suite wall**: ~16/11,5 min = **1,4×**.
- **mechanism-check**: probe 5k×1KB append = old 499ms → new **2ms
  (250×), oracle 2ms** — canale `.=` O(n²) su locale CHIUSO alla cifra;
  sentinelle probe55-sem/dtor BYTE-ID new vs old.
- **Checkpoint Fase 3 (census media, tree wp54)**: arr vive ~421MB
  (39% proxy) vs str ~45MB (4%) ⇒ pilota heap-a-handle = hashed-array.
