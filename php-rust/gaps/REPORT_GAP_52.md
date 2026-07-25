# REPORT_GAP_52 — gap perf oracle↔phpr della sessione WP-52

> Convenzione (decisione utente 2026-07-25): questo file contiene SOLO le
> misure della sessione WP-52; il trend cumulativo vive in `GAP_TREND.md`.
> ⚠️ confrontare RAPPORTI, mai tempi assoluti di giornate diverse.

## Metodo di misura
1. **Media group**: oracle 1 run `/usr/bin/time -l` (DB reset + uploads
   azzerati, MIMALLOC_PURGE_DELAY=0) vs phpr → rapporto **user CPU** e
   **peak footprint**.
2. **Full-suite**: CPU del processo master phpr dal tail del `.rss` della
   runN di sessione vs oracle (baseline 5:39) → rapporto; wall indicativo.

## Misure della sessione
- **media CPU (phpr/oracle)**: 58,84/20,82 = **2,83×** ⭐ (nuovo minimo;
  A/B ab52 old=wp51b vs new=in-node marks+cold-box: **−0,57%, new 6/6** —
  entrambe le guardie rispettate, la leva CPU PAGA anche sul media)
- **media footprint**: 1,7202/0,3947 = **4,36×** peak fisico (old
  stesso-giorno 1,7098 = 4,33×, A/B **+0,61% = +10,4MB**) — costo del
  +8B/nodo Ob.1 al picco sopra la stima count-based (probabile size-class
  mimalloc 80→96B sugli array); REGRESSIONE PICCOLA TENUTA e verbalizzata
  (direttiva no-revert; guardia ≤+2% comunque rispettata). ⚠️ l'RSS
  telemetrico dei full (new 4024 vs old 2897 MB) è accounting MADV/pagine
  tenute residenti dai mark — NON footprint (peak fisico ufficiale sopra).
- **full-suite master-CPU**: **run40 725,1s ≈12:05 = 2,14×** ⭐⭐ (coppia
  stesso-giorno: old phpr-wp51b run40-old 787,4s = 2,32× → **−7,9% raw /
  ~−6,7% corretto per distanza-dal-sample**; mechanism-backed: classify_ms
  census 109.383→62.678 = −46,7s ≈ −5,9% + churn obj −2,02GB del cold-box;
  old diurno 787,4 coerente con old notturno 782,7 = macchina stabile) —
  fail-set **BYTE-IDENTICO a run33** (88 nomi) su census52, run40 e
  run40-old; 30.472 test 0E/2F/86W/73S. Residuo vs riferimento WP-40
  2,06×: ~30s (era ~85s a inizio sessione).
- **full-suite wall**: ~16,3/11,5 min = **1,4×**
