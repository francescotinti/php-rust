# REPORT_GAP_49 — gap perf oracle↔phpr della sessione WP-49

> Convenzione (decisione utente 2026-07-25): questo file contiene SOLO le
> misure della sessione WP-49; il trend cumulativo vive in `GAP_TREND.md`.
> ⚠️ confrontare RAPPORTI, mai tempi assoluti di giornate diverse.

## Metodo di misura
1. **Media group**: oracle 1 run `/usr/bin/time -l` (DB reset + uploads
   azzerati, MIMALLOC_PURGE_DELAY=0) vs phpr → rapporto **user CPU** e
   **peak footprint**.
2. **Full-suite**: CPU del processo master phpr dal tail del `.rss` della
   runN di sessione vs oracle (baseline 5:39) → rapporto; wall indicativo.

## Misure della sessione
- **media CPU (phpr/oracle)**: 63,69/21,18 = **3,01×** di giornata ⚠️
  rumorosa (primi round 69s su ENTRAMBI i lati; old d13dd71 stesso-giorno
  64,46 = 3,04×) — **A/B interleaved: CPU −1,19%, 5/6 round new<old — prima
  leva della roadmap che migliora ENTRAMBE le metriche** (recupera parte del
  +7% media WP-46)
- **media footprint**: **1,715/0,394 = 4,35×** peak fisico (old
  stesso-giorno 1,754 = 4,45×, **−2,22%**: i Weak tombstone purgati
  rilasciano gli RcBox pinnati) — **Fase 1.2 created→Weak FALSIFICATA dalla
  predizione misurata: `created-dead-rc1[n=14]=113KB` su 114,73MB del canale
  (0,1%) — il pinnato è garbage CICLICA che un registry Weak non libera;
  obiettivo de-pinning (385MB full) → Fase 1.4**
- **full-suite master-CPU**: run36 ≈13:55/5:39 = **2,46×** ⭐⭐ (**da 19:10 =
  3,39×: −27%, il più grande recupero CPU full della roadmap**; leva = purge
  tombstone al trigger, modello `gc_remove_from_buffer` Zend — i collect
  scattavano su buffer ~97% morti in 530/530 call; mechanism-check a
  CONSERVAZIONE: round 1005→297, classify_ms −62%, freed 14,16M→14,23M
  +0,5%, soglia 1,05M→50k base; residuo vs 2,06× WP-40: classify residuo +
  reflect-thrash + light-sweep 1,01G ingressi) — **fail-set BYTE-IDENTICO a
  run33** (88 nomi; 30.472 test 0E/2F/86W/73S); RSS transiente max 5431MB
  "INVARIATO" (⚠️ poi riclassificato in WP-50 come artefatto di lettura
  cross-segmento del file .rss in append)
- **full-suite wall**: ~18/11,5 min = **1,6×**
