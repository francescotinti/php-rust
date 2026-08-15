# S-140 — criterio LEVA HC1 v2 (pre-registrato PRIMA del tentativo t2)

Esito t1 (verdetto s140-r5): D=+3,3 SOTTO il pavimento 4,0 su giudice a
2 check/iter — riconciliazione in banda (5,0≤5,0), guardie 9/9 ok, segno
consistente 9/10 coppie. Lettura: lo STRUMENTO non risolve il prezzo
per-check (~1,65 ns); nessun claim di assenza oltre la risoluzione (veto
simmetrico). La leva NON è promossa da t1; t2 = tentativo NUOVO.

1. **Giudice v2**: `m-hintcall6.php` (N=3e6; 5 param tipizzati Object +
   return = **6 check/iter**) — scala la RISOLUZIONE, non la soglia: se il
   prezzo per-check è ~1,65 ns, D atteso ≈ 6×1,65 ≈ 10; se il canale non è
   reale, D resta sotto il pavimento. Il guadagno REALE nel workload resta
   prezzo×conteggi: census ORM in `s140-census-orm.out` (quota dichiarata in
   PERF_MAP qualunque esito).
2. **Soglia** = max(4 ns/iter, rumore drop-1 del run, banda-layout PROPRIA di
   m-hintcall6 misurata pin vs null 275e5836 R=5 PRIMA del t2). Stesso
   harness (arg giudice), ABAB R=5, smoke R=2 early-stop a segno opposto.
3. **Modello**: D atteso 6-20 (6 × prezzo per-check 1,0-3,3 dal t1);
   D > 40 = FUORI MODELLO, sonda dovuta. Riconciliazione smoke↔R5 banda
   max(banda-layout, 5,0).
4. **Guardie, gamba segnalata, finestra**: come criterio v1 p.5-p.7
   (invariati; m-hintcall v1 NON è guardia: stessa leva, giudice sostituito).
5. **Promozione**: come v1 p.8 (pin + batteria + corpus ×2 + fixture +
   micro + gate ORM per NOME + conferma post-pin del giudice v2).
6. **Se anche t2 resta sotto soglia**: REVERT della leva verificato
   (ricostruzione byte vs pin) e reperto a PERF_MAP: canale hint-check
   prezzato sotto risoluzione, quota suite dal census.
