# Verbale sedia 7 — Leijen (mimalloc, footprint fisico) — Concilio WP-103

## VERDETTO
S-101 REGGE sulle promozioni H-C1a/b (contatori esatti, non tocco quel
terreno). NON regge la gamba alloc come FORMULATA, e la bozza §S-102
punto 1 ha un confondente di modo non dichiarato. Una refutazione
capitale, tre emendamenti, tre kill-switch.

## REFUTAZIONE CAPITALE — RC-LE-103-1: «alloc/iter≈0» non è dimostrato, è INVISIBILE allo strumento
`MIMALLOC_SHOW_STATS` a granularità di pagina misura il footprint
RITENUTO, non il churn: un canale malloc+free-per-iterazione ricicla la
stessa pagina e produce stats byte-identiche PER COSTRUZIONE — anche a
1 alloc/iter, non solo 0,1. E anche sul ritenuto la sensibilità è
marginale: soglia = 4 pagine ≈ 256 KB su 100K iter ≈ 2,6 B/iter; un
canale da 0,1 alloc/iter × 32 B = 3,2 B/iter siede DENTRO il rumore
dichiarato. Aggravante: la gamba alloc gira sul pin f29883eb, il census
su 725d5a1, il promosso è 48a5d438 — tre binari, nessuno smoke incrociato.
La frase va DECLASSATA a «retained-alloc/iter≈0 su f29883eb»; il claim
di churn resta APERTO. Non travolge H-C1a/b (promosse su contatori
propri), ma vieta di citare «alloc/iter≈0» come premessa in S-102.

## Emendamenti

**A-LE-103-1 (mem-census diretto)**: la gamba alloc di ogni census
futuro si fa col CONTEGGIO DIRETTO — wrapper `#[global_allocator]` che
conta alloc/dealloc/bytes (o build mimalloc MI_STAT≥2), con la stessa
disciplina degli altri contatori: linearità 300:1 verificata, assert
conteggi↔nomi (KS-KL-101-3). SHOW_STATS resta ammesso SOLO per claim di
footprint ritenuto.

**A-LE-103-2 (punto 4: stesso MODO sui due pin)**: il −116 MiB on↔off
intra-sera (1863,8 vs 1979,5; segno e ordine confermati da S-100:
1929,0 vs 1998,5) dice che il MODO muove il peak di ~5-6% — plausibile
oltre il rumore phpr (spread R=2 off in S-100 ~0,5%), ma R=1 per modo in
S-101: è una PISTA, non un fatto. Conseguenza vincolante: l'A/B
S-99↔S-100 del punto 4 si esegue a modo FISSATO (off su entrambi, il
solo modo che esiste sul pin S-99); confrontare default-contro-default
confonderebbe pin e modo e l'effetto-modo (~116 MiB) è più grande
dell'oggetto (+95 MiB). La pista modo→peak si iscrive come voce PROPRIA
(R≥3 per modo) — candidata attribuzione: albero d'emissione lowered più
piccolo.

**A-LE-103-3 (rumore per-motore PRIMA, statistica nominata)**: il ~10%
è rumore dell'ORACLE; usarlo come banda per un A/B tra pin PHPR è
motore-sbagliato. Ordine: (a) misurare il rumore full-peak phpr su UN
pin, R≥5, interleaved; (b) statistica = MEDIANA per arm + spread
max−min (mai la media: il peak è una coda unilaterale); (c) banda
unilaterale = mediana_arm + spread massimo osservato tra i due arm.
A-LE-102-4 esce dal backlog: la cifra 1863,8 resta «riferimento bande
VOID» (così è scritta, e va bene) ma NON può fare da baseline del punto
4 senza smoke sul pin graduato.

## Kill-switch

**KS-LE-103-1**: qualsiasi «alloc/iter≈0» citato in S-102 da stats a
pagine senza conteggio diretto ⇒ gamba VOID.

**KS-LE-103-2**: A/B del punto 4 con modi diversi tra i due pin ⇒
risultato VOID, bisect vietato.

**KS-LE-103-3**: se lo spread intra-arm phpr (R≥5, mediane) risulta
≥ metà dell'effetto cercato (≥~48 MiB), l'A/B è sottopotenziato ⇒
bisect VIETATO finché non si abbassa il rumore o si alza R.

— Leijen, sedia 7
