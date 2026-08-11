# s130-e1a-lettura.md — E1a misurata: il residuo E−E2 NON è dominato dalla resolve

**Esito formale**: SONDA-E1A ACQUISITA (rc=0, conteggi deterministici, parità stdout,
tree ripristinato). Quote = MODELLO (build emendata, pin s130).

## Reperti (dai raw, R=3 mediane)
1. **Conteggi k ESATTI**: objalloc k=4,0000 (zero FieldAssign!) · objdatains/p2 k=9
   · p5/p6 k=14. Il controllo objalloc svela che **4 resolve/iter vengono dal
   cammino ctor** (PropSet/init), NON dallo statement: E1a vi vale 72,2 ns con
   E=E2=TOT=0.
2. **La riga «E1a/(E−E2)=67%» del verdetto è da leggersi respinta**: le resolve
   del ctor accadono FUORI da field_set, quindi E1a (tutte le chiamate) non è un
   sottoinsieme di E−E2. Formula pre-registrata male; la correzione non richiede
   rerun perché la lettura per DIFFERENZA era anch'essa nel criterio del metodo
   (stessa build, datains−alloc — s129-criterio-tempo p.4).
3. **Cifra onesta per-statement (per differenza, stessa build)**: E1a_datains −
   E1a_alloc = 111,1−72,2 = **38,9 ns per 5 resolve/statement** (p2−alloc 39,5;
   p5−datains 44,2 e p6−datains 43,1 per il 2° statement, ancora 5 resolve).
   ⇒ dentro E−E2 (165,1 su datains) la resolve pesa **~39 ns ≈ 24%**, non 67%.
4. **UB leve S-131** (modello): resolve-once sul SOLO statement (5→1) ≈ **31–35
   ns/iter** — un ordine sotto l'etichetta residuale «155». Se la leva copre anche
   il cammino ctor (4→~1 con cache per-op) l'UB sale verso ~85–99, ma è un'altra
   forma (tocca New/ctor, non FieldAssign). Il residuo GROSSO di E−E2 (~120 ns) è
   dispatch+prop_step NON-resolve: borrow, 3× prop_key(Box), contains/get_mut/
   replace sulla props-map — candidati da modellare PRIMA di nominare la forma.
5. **Coerenza F4 end-to-end**: TOT sonda 232,9 vs 296,7 di S-129 (pin pre-F4):
   Δ≈64 ≈ preludio 73 − overhead sonde; il modello del tempo trasferisce.
