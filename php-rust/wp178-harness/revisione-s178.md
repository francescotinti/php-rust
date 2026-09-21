# Revisione S-178 — lente PROCESSO

Revisore adversariale separato, sola lettura. **Nessun rilievo bloccante per consegnare il censimento; nessun verdetto prestazionale.**

Ricalcolati direttamente dal TSV: 8.998.101 chiamate, 9.219 init, 4.979.840 miss; private non-readonly 1.795.090, 36,047% dei miss e 19,970% delle chiamate non-init. Confermati 535.617 tentativi ripetuti sullo stesso oggetto/slot e 1.777.221 ripetizioni condizionate di sito. Readonly: 2.950.438. Le classi PHPUnit raccolgono 1.803.237 miss, 36,211%: non sono maggioranza.

Il rapporto distingue ricevente, file esecutore e costo temporale; dichiara gli otto miss non classificati, il perimetro dopo lazy forwarding e l'esclusione degli altri handler. La fixture verifica categorie positive/negative, slot privati omonimi e ripetizioni tra istanze distinte. Weak preserva la vita semantica degli oggetti, ma altera memoria e allocazioni: il divieto di usare i tempi del census è necessario e rispettato.

Il gate ORM dimostra soltanto invarianza del riepilogo e dei 16 nomi rispetto al baseline, con sonda attiva/spenta. Non dimostra equivalenza completa degli output o correttezza generale di HEAD; il rapporto evita queste estensioni.

Private non-readonly è ammissibile come **prossima ipotesi da misurare**. Non autorizza un fill per tutti gli 1,795 milioni di miss: quota con `obj_class == scope`, slot utilizzabile e guardie compatibili resta ignota. La distinzione per Pdo\Sqlite ereditato è corretta. La maggiore frequenza readonly non stabilisce una priorità temporale opposta.

Costo effettivo e beneficio quantitativo restano aperti, esplicitamente e per scelta dell'utente. La prossima decisione richiede profilazione rappresentativa in quiete e A/B con stessa toolchain, non conteggi o microcosti moltiplicati. Nessuna ottimizzazione promossa.

Correzioni preliminari incorporate PRIMA del census: chiave storage anziché nome semplice per i rewrite; categorie non classificate=-1; ricevente dopo forwarding; injection monouso con cardinalità degli anchor; nessuna modalità temporale nella sonda finale.
