# s122-istruttoria-arr.md — arr su D2 (+2,02/op-int), NEXT_SESSION §S-122 p.5

Sorgente (arr.php): build 100k chiavi `"k$i"` + 60×100k lookup con chiave
INTERPOLATA per-lookup ⇒ D2 = 6,1M op-int; census s119: phpr 4,02 alloc/op-int
vs zend 2,00 ⇒ **+2,02/op**.

## Ipotesi principale (da verificare, NON promossa)

Ogni lookup ricostruisce la chiave `"k$i"`: int→str + concat. Le ~4 alloc/op
phpr sono indiziate = 2 del **pavimento PhpStr 2-alloc** (Rc + buffer — già
NOMINATO fuori-perimetro nel criterio L-ST1 p.1) + ~2 di temp (format int,
buffer concat); zend paga 2 (zend_string singola con buffer inline, ×2 siti).
Se vera: la colonna arr NON è una leva micro — è il pavimento STRUTTURALE di
PhpStr (layout single-alloc / small-string inline), chirurgia php-types.

## Verifica nominata (quando la macchina è libera dai run S-122)

Census per-SITO (estensione census-clite, target separato) sul percorso di
interpolazione: contatori distinti per (a) int→str temp, (b) buffer concat,
(c) alloc Rc PhpStr, (d) alloc buffer PhpStr; attesa: (c)+(d) ≈ 2,00/op.
Predizione secondaria A VERBALE (az. rev. S-121 #3): se l'ipotesi è vera, la
stessa firma (c)+(d) deve comparire ANCHE nella colonna str (i 2 alloc concat
dell'istruttoria L-ST1) — stessa radice, stessa cura strutturale.
