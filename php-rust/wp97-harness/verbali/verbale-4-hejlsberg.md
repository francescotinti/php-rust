# Verbale — Sedia 4 (Hejlsberg) — WP-97
Perimetro: compilatori incrementali, interning/dedup, collocazione delle analisi.

## VERDETTO
S-95.0 (F1+F2 in sola misura) regge nel mio perimetro: la direzione d'errore è dichiarata, i conteggi sono deterministici, il binario di parità è invariato. Il programma §WP-96 (F3) invece **NON è eseguibile come scritto**: due assunti sono refutati sotto. F3 ammissibile SOLO sotto gli emendamenti.

## Emendamenti

**A-AH-97-1 (collocazione).** L'analisi va nel COMPILATORE, all'emissione della funzione, PRIMA dell'inserimento nella unit cache (A-BB6): la riscrittura `LoadSlot→TakeSlot` deve stare nella unit cache stessa, così include ripetuti la pagano UNA volta per unit, non per inclusione. Nessuna struttura di analisi sopravvive a runtime: il risultato si consuma nell'emissione e si butta. `eval` paga per compilazione (corpi piccoli; coperto da A-AH-97-2).

**A-AH-97-2 (budget di compile-time).** `analyze` è un punto fisso full-sweep con `Bits::new`+`clone` per op per iterazione e `exc_edges` O(regione×op): accettabile in misura, non in compilazione su ~migliaia di funzioni WordPress. Obbligo: (a) bailout per funzioni sopra soglia `ops × slot_words` — il bail = rinuncia totale (si tiene il clone), correttezza intatta; la soglia DERIVATA dalla distribuzione `sites_total` per funzione del census, non scelta (NON-riproporre «soglia non derivata»); (b) buffer preallocati/worklist al posto del full-sweep; (c) giudice = oracle compile-side `--list-tests` (WP-59), coppia prima/dopo nello stesso commit.

**A-AH-97-3 (identità ≠ puntatore).** La chiave `(addr Func, addr ops, len)` di `zvalcensus.rs` è tollerabile solo perché il conteggio è advisory. Se una qualunque cache d'analisi sopravvive in F3, la chiave deve essere identità STRUTTURALE (unit-id + indice funzione, o hash del contenuto ops), mai indirizzi: con unit cache TL, eval e riuso dell'allocatore, l'ABA su indirizzo attribuirebbe `movable` di una funzione a un'altra — in F3 non è un sovraconteggio, è un take sbagliato.

**A-AH-97-4 (layout di Op).** `size_of::<Op>() == 48` come static assert/test NELLO STESSO commit dell'opcode: una variante nuova è layout-free solo finché il payload sta nel massimo esistente E il conteggio varianti non attraversa la soglia del discriminante. In alternativa flag: MAI un campo nuovo dentro `LoadSlot`; se flag, un bit rubato nell'operando slot (slot < 2³¹) o bitmap laterale per-funzione (1 bit/op, fuori dall'array caldo).

**A-AH-97-5 (banda al netto).** La banda F4 (1,91–2,75% safe) è LORDA: non sottrae il costo dispatch del corpo nuovo. Serve una build A/A con handler compilato ma mai emesso per quotare la penalità alla WP-38 prima di leggere il Δ.

## Kill-switch

**KS-AH-97-1**: `size_of::<Op>() != 48` dopo F3 → commit respinto (footprint per worker).
**KS-AH-97-2**: oracle `--list-tests` compile-side peggiora oltre il budget deliberato → niente ship, prima bailout/worklist.
**KS-AH-97-3**: qualunque cache keyed-by-pointer nel percorso F3 di release → reject senza discussione.

## Refutazioni capitali

**RC-1.** design95 chiude con «questa NON aggiunge opcode al percorso caldo, ne cambia uno esistente»; §WP-96 dice «l'opcode TakeSlot». Le due frasi non stanno insieme: o corpo handler nuovo (lezione WP-43: il costo è il NUMERO di corpi caldi) o branch in un arm esistente (WP-38: +2,9% da un branch mai preso). La banda predetta ignora entrambi i costi: com'è scritta, la predizione P3 per F4 è irricevibile finché non è al netto (A-AH-97-5).

**RC-2.** Riusare il meccanismo F1 tal quale (analisi lazy a runtime, cache a puntatori) in F3 è refutato in capite: puntatore non è identità di funzione, e ciò che in misura era una collisione innocua in emissione diventa corruzione semantica silenziosa. F3 esiste solo come analisi di compilazione (A-AH-97-1/3).
