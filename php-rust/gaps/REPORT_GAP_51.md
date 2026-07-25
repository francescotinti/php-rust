# REPORT_GAP_51 — gap perf oracle↔phpr della sessione WP-51

> Convenzione (decisione utente 2026-07-25): questo file contiene SOLO le
> misure della sessione WP-51; il trend cumulativo vive in `GAP_TREND.md`.
> ⚠️ confrontare RAPPORTI, mai tempi assoluti di giornate diverse.

## Metodo di misura
1. **Media group**: oracle 1 run `/usr/bin/time -l` (DB reset + uploads
   azzerati, MIMALLOC_PURGE_DELAY=0) vs phpr → rapporto **user CPU** e
   **peak footprint**.
2. **Full-suite**: CPU del processo master phpr dal tail del `.rss` della
   runN di sessione vs oracle (baseline 5:39) → rapporto; wall indicativo.

## Misure della sessione
- **media CPU (phpr/oracle)**: 59,17/20,82 = **2,84×** ⭐ (miglior media dai
  tempi di WP-35; A/B ab51 old=wp50 vs new=A+B: **+0,01% FLAT**, 2/6 —
  guardia Fase 1 ≤+0,5% RISPETTATA dalla leva boundary-collect)
- **media footprint**: 1,7209/0,3964 = **4,34×** ⭐ peak fisico (nuovo
  minimo; old stesso-giorno 1,7265 = 4,36×, A/B **−0,32%**) — il de-pin del
  canale created (353,7MB full / 114,7MB media, ritenzione di FINE-run)
  monetizza poco sul PICCO mid-run, coerente con la lezione WP-48
- **full-suite master-CPU**: **run39 782,7s ≈13:03 = 2,31×** ⭐⭐ (tre full
  stessa notte: old phpr-wp50 827,9s = 2,44× → leva A classify-fuso run38
  793,7s = 2,34× (−4,1%, **riconciliata alla cifra**: classify_ms census
  142,4→109,4s = −33s) → leve A+B run39 782,7s = 2,31× (−5,5% totale; la
  leva B boundary-collect è GRATIS — il −11s marginale NON è
  mechanism-backed: freed +0,1%, rumore); old di notte coerente con gli
  old di ieri 828,9/832,7 = macchina stabile) — fail-set **BYTE-IDENTICO
  a run33** (88 nomi) su tutte e tre; 30.472 test 0E/2F/86W/73S; RSS max
  di segmento run38 1696 / run39 1945 / old 1700 (rumore, nessun segnale
  per-leva). **Mechanism-check leva B: canale created-registry-only
  INVARIATO a 353,7MB = garbage di TEARDOWN (vivo fino allo smontaggio
  PHPUnit, invisibile a ogni cadenza mid-run per costruzione) — esito
  footprint Fase 1.4 su PHPUnit = NEUTRO, leva tenuta perché gratis +
  ceiling Zend per processi long-running**
- **full-suite wall**: ~17/11,5 min = **1,5×**
