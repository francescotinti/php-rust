# REPORT_GAP_50 — gap perf oracle↔phpr della sessione WP-50

> Convenzione (decisione utente 2026-07-25): questo file contiene SOLO le
> misure della sessione WP-50; il trend cumulativo vive in `GAP_TREND.md`.
> ⚠️ confrontare RAPPORTI, mai tempi assoluti di giornate diverse.

## Metodo di misura
1. **Media group**: oracle 1 run `/usr/bin/time -l` (DB reset + uploads
   azzerati, MIMALLOC_PURGE_DELAY=0) vs phpr → rapporto **user CPU** e
   **peak footprint**.
2. **Full-suite**: CPU del processo master phpr dal tail del `.rss` della
   runN di sessione vs oracle (baseline 5:39) → rapporto; wall indicativo.

## Misure della sessione
- **media CPU (phpr/oracle)**: 59,84/20,935 = **2,86×** (A/B leva 1:
  **−0,23% flat**, 2/6 — su media la banda vale solo 8,9M ingressi ≈ 0,2s;
  A/B leva 2: **0,00% CPU flat**, 3/6; A/B non ripetuto sul bound-cache
  finale: solo shrink dell'arm)
- **media footprint**: **1,755/0,394 = 4,45×** peak fisico (A/B leva 2
  **+2,64% ATTRIBUITO**: i descriptor reflect ritenuti dal cap 16384
  stazionano nei buffer ctr come possible-root — census `gc-ctr-roots`
  19,14MB cap-8192 → 60,86MB cap-16384 = +41,7MB standing; leva 1 peak
  −0,15% flat)
- **full-suite master-CPU**: **run37c ≈13:37 = 2,41×** ⭐ (6 full
  stesso-sera, tutte BYTE-ID a run33: old wp49 n=2 = 828,9/832,7s spread
  3,8s; la PRIMA forma della leva banda REGREDIVA +17..35s — 848,1
  leva-1-sola / 857,9-870,5 leve-1+2 — nonostante −825,6M ingressi: **legge
  WP-44 su una GUARDIA, +1 load+max nell'arm Op::Sweep = I-cache >
  risparmio, invisibile ai census**; forward-fix `f034c6c` campo
  `gc_sweep_bound` ⇒ **816,8s = old−14s ≈ risparmio banda teorico ~16s,
  bilancio RICONCILIATO**; mechanism-check: light 1.014,7M→209,8M con
  band_skipped 825,6M, classify_ms 142,4≈141,8s, freed −0,04%) — ⭐⭐ RSS
  `.rss` in APPEND: max PER SEGMENTO (il "5431 invariato" tramandato da
  run34 era cross-segmento; veri: run35 2342 / run36 2113 / run37 1757 /
  run37c 2055) — **Ob.2: canale created media = 100% garbage ciclica
  collettabile (114,73MB→0 col probe full-scan `PHPR_GC_EOR_FULL_COLLECT`,
  874ms; created 94.565→22.141; Δroots_total = 99,98% del canale);
  collect_cycles si radica SOLO dai buffer ⇒ serve seed full-scan;
  reflect-cap 16384 FALSIFICATA come leva CPU (miss 890.132→887.798,
  hit-rate 16,3→16,5%)**
- **full-suite wall**: ~18/11,5 min = **1,6×**
