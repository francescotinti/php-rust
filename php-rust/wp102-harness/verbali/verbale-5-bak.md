# Verbale Sedia 5 — Lars Bak (microarchitettura, path caldi, alloc-rate) — Concilio WP-102

## VERDETTO

S-100 è metodologicamente la sessione migliore dell'arco (flip collaudato nei due modi, decisioni con misura). Ma la prima misura H-C **nomina il meccanismo sbagliato con strumenti ciechi**, e la bozza §S-101 rischia di scrivere H-C1 contro un nemico che sul giudice potrebbe non esistere. Refutazioni capitali: **sì**.

## Refutazioni

**R1 (capitale) — Il meccanismo H-C1 è nominato da visibilità asimmetrica su un carico di SOLI INTERI.** `prop.php` muove int (`$o->x = $o->y + 1`). Il clone di uno Zval intero non dovrebbe allocare né toccare refcount: se `drop_in_place<Zval>` 12,6% + `gc_note` 5,3% compaiono su un carico scalare, il costo NON è "clone vs borrow di dati refcounted" ma drop-glue, controllo di discriminante e write-barrier chiamati incondizionatamente su scalari. La cura "prestito/refcount al posto del clone" presuppone traffico refcount; su int potrebbe AGGIUNGERLO dove oggi non c'è. Intanto l'oracle nasconde il SUO traffico refcount inline negli handler SPEC ("nessun simbolo visibile" = cecità, non assenza). Il meccanismo va ri-nominato dopo un census alloc/refcount, non dedotto dai simboli out-of-line.

**R2 — La decomposizione 2,0×6,2 è un'identità aritmetica, non due leve.** ns/op è una MEDIA su specie eterogenee: i 9 op di puro traffico (Sweep, Pop, Swap, doppioni Load) alzano il conteggio e ABBASSANO meccanicamente il costo/op medio. Eliminare traffico peggiorerebbe la "gamba costo"; le due gambe non sono indipendenti e "domina il costo/op" è un artefatto del framing. E "~9-10 ns quasi invariante di categoria" poggia su n=2 (prop, arith residuo): due punti non fanno un invariante.

**R3 — Il profilo co-equale con run_loop al 50% inlined non giudica.** Il 27% Zval conta solo simboli out-of-line: ogni clone/drop inlinato dentro run_loop è invisibile ⇒ 27% è un PAVIMENTO. Simmetricamente, dentro quel 50% non si distingue dispatch da corpi handler da fast-path clone: attribuire il residuo al dispatch è non supportato. Qualsiasi post-misura di H-C1 giudicata contro questa baseline sfocata è un verdetto sfocato.

**R4 — L=12,9 ns/occ non trasporta, e la sessione stessa lo prova.** Un solo shape, 200M iter, BTB perfettamente addestrata, ~190 ns/iter di contesto: sull'occorrenza residua VERA di add.php la contro-misura dà −5,6 ns/occ — la "banda" si è già dimezzata cambiando vicinato di dispatch. La decisione di estendere resta valida (frequenza ≠ 0, equivalenza provata); il NUMERO no: chiamarlo "coerente" con D=6,27 a fattore 2 di distanza è generoso.

**R5 — H-C1 non chiude H-C nemmeno a successo pieno, e la bozza non lo dice.** Rimuovendo TUTTO il 27%: 5,22×0,73=3,81 s ⇒ ~9,1× — ancora 3× sopra l'obiettivo X≤3. Il tetto atteso va scritto PRIMA, o il verdetto post-misura sarà negoziato dopo.

## Emendamenti

- **A-BA-102-1**: census alloc/iter e refcount-ops/iter su prop.php nei DUE motori (allocatore contatore o stats mimalloc) PRIMA di iscrivere H-C1; se alloc/iter≈0, il meccanismo si ri-nomina (drop-glue/discriminante su scalari).
- **A-BA-102-2**: profilo inline-aware (stack con inlining ricostruito, o build `inline(never)` mirata CON perturbazione misurata) per aprire il 50% di run_loop: quote dispatch/handler/clone-inlinato per NOME.
- **A-BA-102-3**: fixture H-C1 per SPECIE di valore (int, string condivisa, array), criterio per specie — la cura giusta per string può essere quella sbagliata per int.
- **A-BA-102-4**: L=12,9 non entra in alcun registro come coefficiente riusabile; ogni shape futuro porta il SUO micro.

## Kill-switch

- **KS-BA-102-1**: H-C1 NON si iscrive senza tetto atteso pre-registrato in ns/iter (≤ pavimento 27% + quota inline da A-BA-102-2) e la dichiarazione esplicita che il successo pieno lascia prop ~9×.
- **KS-BA-102-2**: la ri-baseline sei categorie si pubblica con ns/op per specie (traffico vs lavoro), mai solo il prodotto conteggio×costo: l'identità non è una decomposizione causale.
