# s127-criterio-submicro.md — residuo 427 ns di objchurn (az. rev. S-126 #2) — PRE-registrato

1. REGISTRAZIONE bilaterale (non leva): chiudere l'additività di objchurn (1820 ns/iter phpr) vs objalloc+objmap (1393; residuo 426,7 phpr / 43,4 oracle).
2. Sei categorie nello STESSO run (stesso metodo s126 p.2: user CPU, pavimento per-binario, R=5 alternato, N dal sorgente, parità stdout): objalloc · objchurn · objmap (ri-corse, arm di riferimento same-run) + objdropdef (new+overwrite mappa: drop DIFFERITO, senza data-insert) + objdatains (new+data-insert, drop immediato) + objallocni (ctor senza interpolazione).
3. Letture pre-registrate (per lato, ns/iter netti): Δins = objdatains − objalloc (costo insert su array di proprietà + drop array non vuoto) · Δdrop = objdropdef − (objalloc+objmap) (drop differito vs immediato + interazione) · CHIUSURA additività se |objchurn − (objdropdef + Δins)| ≤ max(4 ns/iter, rumore = spread max della tripla coinvolta su R=5) · objallocni − objalloc registra la contaminazione-interpolazione (solo cifra, nessuna decisione).
4. Se la chiusura fallisce ⇒ residuo NON additivo dichiarato a verbale, il «67%» di L-OL1 si riformula come «objalloc = 9,9×, quota del churn NON ripartita» (la nomina L-OL1 resta: discende dalla regola s127-regola-nomina.md, non dal 67%).
5. Arbitro `s127-submicro.sh` committato in QUESTO commit; cifre citabili solo da `s127-submicro-verdetto.out`; pin s125 002e6cc1 obbligatorio; alarm 900 s ⇒ categoria NULLA dichiarata.
6. Nessuna predizione di magnitudine sulle tre categorie nuove (prima misura).
