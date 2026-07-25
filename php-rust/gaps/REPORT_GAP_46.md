# REPORT_GAP_46 — gap perf oracle↔phpr della sessione WP-46

> Convenzione (decisione utente 2026-07-25): questo file contiene SOLO le
> misure della sessione WP-46; il trend cumulativo vive in `GAP_TREND.md`.
> ⚠️ confrontare RAPPORTI, mai tempi assoluti di giornate diverse.

## Metodo di misura
1. **Media group**: oracle 1 run `/usr/bin/time -l` (DB reset + uploads
   azzerati, MIMALLOC_PURGE_DELAY=0) vs phpr → rapporto **user CPU** e
   **peak footprint**.
2. **Full-suite**: CPU del processo master phpr dal tail del `.rss` della
   runN di sessione vs oracle (baseline 5:39) → rapporto; wall indicativo.

## Misure della sessione
- **media CPU (phpr/oracle)**: 59,39/20,82 = **2,85×** ⚠️ **REGRESSIONE
  +7,0% TENUTA per direttiva no-revert** (old dd148a0 stesso-giorno 55,50 =
  2,67×; 6/6 round new>old, oracle 20,79-20,85; costo = 11 collect ×
  classify su ~726k root VIVI + rooting container nel note-path, ripagati da
  freed 543 = nulla) — in cambio: **parità funzionale migliore di sempre**
  (corpus 1447→1421, famiglia gc 36→14, tutte le 18 probe gc oracle byte-id)
- **media footprint**: 4,55/0,37GB = **12,3×** peak fisico (old di giornata
  4,43-4,46 = 12,0×; +2,5% = buffer ctr + walks) — **il bersaglio ~3G NON è
  caduto: collector operativo ma i root notati sono VIVI (mem-census: arr
  3,113M/1,9G e str 23,8M/1,29G INVARIATI alla cifra). La diagnosi WP-45
  "cicli irraggiungibili" va ri-attribuita: WP-47 = owner-tracer + root-walk
  esteso (FramePool, tabelle VM)**
- **full-suite master-CPU**: run33: ~21:00/5:39 = **3,71×** ⚠️⚠️ (+80% vs
  11:39 run31/32: il collect-walk — HashMap handles/in_edges/children
  dell'INTERO grafo raggiungibile a ogni collect — scala col grafo vivo
  della full; i collect "efficaci" riabbassano la soglia a 50k ⇒ ri-walk
  continui; recupero = classify a 2 passate / mark intrusivi / collect al
  confine test, DOPO l'attribuzione WP-47) — **fail-set BYTE-IDENTICO a
  run32** (88 nomi; 30.472 test 0E/2F/86W/73S)
- **full-suite wall**: ~26/11,5 min = **2,3×**
