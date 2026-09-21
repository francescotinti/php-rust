# S-179 ripresa — revisione adversariale indipendente, lente MISURA

Mandato: trovare ciò che invalida quota candidata o attribuzione temporale. Sola lettura delle evidenze e ricalcolo indipendente; nessun carico o build del revisore.

**Verdetto:** nessun rilievo invalidante per il census o per la stima diagnostica aggregata entro il criterio dichiarato. Non è una dimostrazione di speedup né di correttezza del futuro fill.

Ricalcolato direttamente `count.tsv`: 4.979.840 miss, 1.795.090 private non-readonly, 1.222.283 completamenti con tutti i sette bit richiesti (68,0903%). PDOStatement 400.472, UnitOfWork 77.659, PersistentCollection 70.513 candidati; Pdo\Sqlite zero. Le guardie osservate non dimostrano hit futuri o tempo recuperabile. Le fixture verificano diversi positivi/negativi, ma non includono un negativo esplicito lazy/enum.

La serie rada rimane respinta: con due valori già osservati per densità, nessuna terza replica può portare lo scarto delle mediane sotto 5,9813%. Il criterio denso successivo è distinto e preventivo; non riabilita la serie precedente.

Controllati i dodici record densi: guard_rc=0, processo=2 atteso, parità registrata; quattro campioni E2 sotto soglia, sentinelle durante il workload sotto 150% di background e Data almeno 18,67 GiB. Ricalcolo dai TSV conferma mediane inclusive 0,417629504 s e 0,431558400 s; divergenza 3,2276%. Perturbazione CPU (+0,8410% e −0,0561%), confronto nullo (−0,4188%) e deriva (0,8103%) rispettano il criterio.

**Limiti sostanziali:** la somma è inclusiva, potenzialmente sovrapposta durante rientri VM; il rapporto al wall non è quota esclusiva né tetto rigoroso allo speedup. Il timer non distingue il sottoinsieme candidato. Floor e fixture con elapsed positivo non provano accuratezza locale; metadati prima del timer possono riscaldare il ricevente. Il controllo puro proviene dalla serie precedente: il confronto fra off vecchio/nuovo attenua, senza eliminare, il confondimento temporale.

La stabilità aggregata non si trasferisce alle classi: PDOStatement diverge circa 6,3%, UnitOfWork circa 9,7% fra densità. Le rispettive cifre restano diagnostiche con intervalli osservati, non stime certificate dalla soglia globale. Nessuna percentuale di guadagno è sostenuta; un eventuale fill richiede prova semantica e A/B proprio.
