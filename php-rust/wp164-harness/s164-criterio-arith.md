# Criterio S-164 p.2 — indagine tick arith 5,3→5,5 (DOVUTA) — scritto PRIMA della fase 2

1. DOMANDA: il tick 5,3 (s161) → 5,5 (s162, persistito s163) è numeratore
   (phpr), denominatore (oracle) o quantizzazione del giudice?
2. FASE 1 FORENSE (solo verdetti di RECORD, nessun run): esito in
   `s164-arith-forense.out` — phpr netto 2,35/2,35/2,35 FERMO su
   s161/s162/s163; oracle 0,44/0,43/0,43 con storia 0,43 a s158/s159 ⇒
   l'outlier è il 0,44 di s161 (il 5,3 era l'anomalia, non il 5,5); quanto
   del cronometro 0,01 s su netto 0,43 ⇒ quanto del rapporto ≈0,13 (giudice
   QUANTIZZATO, REGOLE §3/az.rev. S-154). Companion: creep phpr 2,32
   (s158)→2,35 (s161+), +0,6 ns/iter cumulato SOTTO soglia guardie.
3. FASE 2 RIMISURA DE-QUANTIZZATA (dopo la coppia, finestra quieta, lock
   presente): giudice `s164-arith-dq.sh` = derivato DICHIARATO di
   run-micro.sh (arith-only, 4 bracci, 3 decimali); driver `arith-dq.php` =
   copia di micro/arith.php con N=250000000 (×5 DICHIARATO: tick rapporto
   ≈0,026 ≤ 0,1/4); bracci: oracle brew + stash FERMI phpr-s161 (ec0a636a) /
   phpr-s162 (20c63af4) / phpr-s163 (fea4a2d0), byte-verificati pena rc=9;
   R=5 interleaved a rotazione O→161→162→163; pavimenti per-binario R=5;
   N EMESSO dal sorgente del driver.
4. SEGNI PRE-REGISTRATI: (a) phpr s161≈s162≈s163 entro rumore (banda: spread
   storico 0,04 su 2,35 ⇒ ±0,20 su ~11,75) E (b) oracle netto in [2,15;2,20]
   (0,43-0,44 scalato ×5) ⇒ verdetto «tick = quantizzazione + centesimo
   oracle s161, NESSUNA regressione phpr» — indagine CHIUSA. (a) violato ⇒
   creep REALE tra stash: istruttoria per-pin, NESSUNA taratura. (b) violato
   ⇒ finestra sporca: si dichiara e si ripete UNA volta.
5. Esiti a FILE: `s164-arith-dq-verdetto.out`; rc SOLO da
   arith-out/arith-dq.done scritto DALLO script. Nessun numero a memoria:
   attese lette da questo criterio, committato PRIMA del run.
