# REPORT_GAP_56 — gap perf oracle↔phpr della sessione WP-56

> SOLO le misure della sessione WP-56; trend cumulativo in `GAP_TREND.md`.
> ⚠️ confrontare RAPPORTI, mai tempi assoluti di giornate diverse.

## Metodo di misura
1. **Media group**: oracle 1 run `/usr/bin/time -l` (DB reset + uploads
   azzerati, MIMALLOC_PURGE_DELAY=0) vs phpr → rapporto **user CPU** e
   **peak footprint**.
2. **Full-suite**: CPU del processo master phpr dal tail del `.rss` della
   runN di sessione vs oracle (baseline 5:39) → rapporto; wall indicativo.

## Misure della sessione
- **media CPU (phpr/oracle)**: mediane ab56 54,665/20,945 = **2,61×
  (invariato)** — A/B old=phpr-wp55 vs new=indice keyless: **−0,33%**
  (new sotto in 5/6 round) = il pilota è GRATIS in CPU (checkpoint
  pilota ≤+2% superato con margine alla PRIMA sessione).
- **media footprint**: mediane 1536,3/376,3MB = **4,08×** peak fisico
  (old stessa-notte 1564,4 = 4,16×; A/B **−1,79% = −28,1MB**, 6/6 con
  separazione pulita: max new 1538,3 < min old 1561,6).
- **full-suite master-CPU**: run45 (new) **697,8s ≈ 11:38** vs run45-old
  (phpr-wp55) **716,9s ≈ 11:57 STESSA-SERA = −2,66% (−19,1s)** →
  **riferimento pubblicato = run45 697,8/339 = 2,06× NUOVO MINIMO**
  (l'old replica run44 di ieri alla cifra: ambiente stabile; il residuo
  vs WP-40 2,06×/699s è CHIUSO — run45 697,8 < 699). Fail-set **BYTE-ID
  a run33** (88 nomi) su run45 E run45-old; 30.472 test 0E/2F/86W/73S ×2.
- **full-suite wall**: ~16/11,5 min = **1,4×**.
- **mechanism-check (pin c) — banda NON verificata, a verbale**: per-shape
  CONFERMATO sui morti (census 4,75M dtor identici: **−62,3B/array** alla
  cifra, churn −296MB/run) e sul reached subprocess (−20,2%); ma la banda
  canale ex-ante −25..−45% non si realizza: reached master a popolazione
  IDENTICA (714.790 array) = 77,27→71,56MB = **−7,4%** — l'estimatore
  live×death-avg sovrastima arr **5,7×** (standing reale 108B/arr, quasi
  tutto packed/vuoto). Correzione al checkpoint Fase 3: margine arr/str
  reale ~2,7× (non 10×). Cross-check strumento: str +5,6MB tra i due
  census ≈ quota +5,7MB del growable WP-55.
- **Ob.2 quota probe `.=` non-locali** (5k×1KB vs oracle ~1ms): element
  517ms · prop 541ms · static 592ms · nested 576ms (local fuso: 2,3ms) —
  canale O(n²) VIVO su tutti i siti non coperti dal fuso WP-55;
  frequenza per sito → op-census WP-57.
