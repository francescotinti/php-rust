# S-179 — criterio preventivo del costo private non-readonly
1. Oggetto: costo del cammino lento sul workload ORM S-178; PDOStatement bersaglio, UnitOfWork/PersistentCollection controlli; nessuna ottimizzazione o promozione.
2. Pre-flight registrato: lock esclusivo s179-cost-cb7f, Data ≥10 GiB, swap, E2 CPU totale <150% per quattro campioni a 30 s; nessun processo estraneo terminato.
3. E2 fallito ⇒ nessun run temporale, nessuna build esplorativa: consegnare evidenza concreta del blocco, senza interpretare conteggi come tempo.
4. Se E2 passa: archivio sorgente separato a1eb40e8, Rust verificato 1.98.1, release fat LTO/1CGU, target APFS nuova; run pesanti sequenziali detached con watchdog e sentinelle.
5. Prima del costo, censire scope==obj_class, slot effettivo e guardie lazy/hook/magic/readonly/type/ref, senza assumere ammissibile Pdo\Sqlite ereditato.
6. Sonda temporale nuova: controlli positivi/negativi, timer vuoto, sonda spenta e più frequenze di campionamento; rifiutare perturbazione/instabilità >5%; nessun tempo Weak/mappe S-178.
7. Eventuale A/B diagnostico: stessa toolchain, controllo nullo, R=5 ABAB, floor per binario e deriva E1; soglia max(4 ns/iter, rumore, banda-layout), giudice suite reale; nessuno speedup da conteggi×microcosti.
8. Report italiano, raw persistenti e revisore adversariale lente misura; HEAD non verde e pin storici invariati.
