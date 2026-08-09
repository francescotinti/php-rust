# Revisione S-119 — REVISORE SINGOLO, lente MISURA

## Verifiche fatte (sui raw, non sulla prosa)
- **C-lite**: rifatta l'aritmetica dai raw. phpr: re (34184104−183553)/2M=17,00; str 5,00; arr 245,01 e 4,016/op-int (6,1M); prop net 520/30M=0,00 alloc, zvclone 150000010/30M=5,000 e rc 1,000. Zend: re (10054792−54773)/2M=5,00; str 2,00; arr 121,99→2,00/op-int. Delta re +12, str +3, arr +2,02: **quadrano**. R=2 verificato identico (prop/re due lati; zend-arith ±1 dichiarato). Netting: calloc/realloc costanti tra empty e categorie ⇒ il rumore libc si annulla. Auditabilità: `census-clite.patch` è **contenuto-identico** al diff 00a4824→d0c237a (verificato riga per riga) ⇒ la fonte è nel repo; resta non provato che il binario ffe6a147 fosse esattamente quello stato (run a HEAD 00a4824 con edit non committati).
- **Treno-2**: tutte le 6 righe del verdetto ricalcolate dal tsv (D_med in ns/iter = diff mediana/N; segni contati): **quadrano**. Ordine git: criterio fd2e8d3 PRIMA di patch/smoke/full ⇒ il full R=5 era il giudice pre-registrato; lo smoke rc=1 non è judge-shopping. arr N=6M è override hardcoded dichiarato (riga 61 dello script) ma diverge dai 6,1M op-int di C-lite.
- **Coppia WP**: raw ratios letti; ricalcolo incrociato eseguito.

## IL punto che ridimensiona
**Il rif "full ON 1,716" è per metà rumore del denominatore.** phpr full cpu è identico tra le gambe (826,16 vs 827,07, +0,1%) mentre l'oracle si muove del 7,2% (449,63→481,97). Incrociando: ON con oracle-OFF = 1,840; OFF con oracle-ON = 1,714. Il vero full è **1,71–1,84 in ENTRAMBI i modi**: la distinzione ON/OFF a N=1 non esiste, e i FONDAMENTALI di NEXT_SESSION pinnano il punto 1,716 come nuovo rif (sovrastima ~0,12). La direzione (≤1,913/1,955) sopravvive anche al ricalcolo conservativo — di poco (1,840<1,913). Media ON peggiorata (2,506→2,636; phpr/req +5,4% ON) e peak +94 MiB sono dichiarati N=1 ma restano regressioni sotto etichetta «direzione-solo».

## Verdetti
1. **C-lite: REGGE** — cifre riproducibili dai raw, netting sano, patch auditabile; caveat dichiarato: colonna rc Zend è stima statica (clone-confronto a una gamba).
2. **Treno-2 PROMOSSO: REGGE** — verdetto a macchina fedele ai raw, giudice full pre-registrato, held-out solo-direzione.
3. **Coppia WP: RIDIMENSIONATO** — direzione ok, ma lo split ON/OFF e il punto 1,716 sono artefatti dell'oracle mosso 7%.

## Azioni S-120
1. Coppia WP N≥2 con gambe intercalate stessa sera; pubblicare anche i rapporti cross-oracle.
2. Fino ad allora scrivere il rif full come intervallo 1,71–1,84, non 1,716.
3. Sciogliere media-ON 2,636 e peak +94 MiB con misura dedicata R≥3 prima di nuove leve.
4. Registrare l'hash di una rebuild census da d0c237a; unificare arr op-int (6,0M vs 6,1M).
5. Se un vagone futuro si sceglie sui cloni (re/str), misura dinamica Zend anche parziale, o dichiarare che la classifica ordina solo il lato alloc.
