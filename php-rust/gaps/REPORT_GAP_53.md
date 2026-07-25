# REPORT_GAP_53 — gap perf oracle↔phpr della sessione WP-53

> Convenzione (decisione utente 2026-07-25): questo file contiene SOLO le
> misure della sessione WP-53; il trend cumulativo vive in `GAP_TREND.md`.
> ⚠️ confrontare RAPPORTI, mai tempi assoluti di giornate diverse.

## Metodo di misura
1. **Media group**: oracle 1 run `/usr/bin/time -l` (DB reset + uploads
   azzerati, MIMALLOC_PURGE_DELAY=0) vs phpr → rapporto **user CPU** e
   **peak footprint**.
2. **Full-suite**: CPU del processo master phpr dal tail del `.rss` della
   runN di sessione vs oracle (baseline 5:39) → rapporto; wall indicativo.

## Misure della sessione
- **media CPU (phpr/oracle)**: 59,05/20,90 = **2,83×** (pari-minimo; A/B
  ab53 old=wp52 vs new=ret_shape+RET_DEREF+Sweep-elision: **−0,88%,
  new 6/6 stretti** — la leva Fase 2 PAGA anche sul media, poco ma su
  tutti i round)
- **media footprint**: 1627,9/376,1MB = **4,33×** peak fisico (old
  stesso-giorno 1639,2 = 4,36×, A/B **−0,69%**) — guardia ≤+2% della
  Fase 2 rispettata CON MARGINE: meno op ritenuti (DerefTop non emessi +
  sweep dead-code elisi) = bytecode più corto.
- **full-suite master-CPU**: run41 738,6s ≈12:19 vs run41-old (phpr-wp52)
  740,2s ≈12:20 = **−0,2% raw stesso-giorno**; il rapporto di giornata
  (2,18×) è gonfiato dall'ambiente (+2,1%: l'old-binario ieri faceva
  725,1 su run40) ⇒ **il riferimento pubblicato resta run40 = 2,14×**;
  residuo vs WP-40 2,06× ~30s, sostanzialmente intatto. Fail-set
  **BYTE-ID a run33** (88 nomi) su run41 E run41-old; 30.472 test
  0E/2F/86W/73S.
- **full-suite wall**: ~16,5/11,5 min = **1,4×**
- **mechanism-check (op-census, gruppo media, old/new stesso-giorno)**:
  DerefTop 40.233.697 → <1,4M (canale rimosso ≥96,5%); Sweep 53,45M →
  53,31M (−0,27%); Ret 61,74M e ThisMethodCall 26.065.994 INVARIATI alla
  cifra; dispatch totali 713,6M → 673,2M = **−5,66%**. ⭐ Lezione: −5,66%
  di DISPATCH = −0,88% di CPU — i micro-op valgono ~5-15ns l'uno; le
  prossime leve si quotano in ns/evento, non solo in conteggi.
