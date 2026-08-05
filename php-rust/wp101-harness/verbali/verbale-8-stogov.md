# Verbale sedia 8 — Dmitry Stogov (Zend engine, semantica PHP) — Concilio WP-101

Oggetto: S-99.0 + bozza §S-100. Mandato: REFUTARE.

## VERDETTO

**A-ST-100-1 (controfattuale registro): ESEGUITO BENE, conclusione VALIDA
nel suo perimetro — ma il perimetro non è dichiarato per intero.
Bozza S-100: DUE refutazioni capitali** (precondizione AssignOp sparita
dai gate della promozione; perimetro compare non dichiarato al punto 4).

## Giudizio sull'esecuzione di A-ST-100-1 (gamba 4a)

Letto `run.rs:1005-1070` a HEAD contro `premisura-rollout99.out`. Il
ragionamento regge: nelle forme SS/SSDst il hit-path è già borrow → guardie
→ `binary_fast` inline → push/store; una `BinaryAddSS` rimuoverebbe solo il
payload BinOp e il `match b`, ramo costante per sito. **Corretto anche il
non-detto esplicito: le guardie `matches!(Undef|Ref)` NON sono rimovibili**
— Undef porta il warning LoadVar, Ref il deref — e nessun risparmio vi è
stato dichiarato. Banda [0, 0,5] pubblicata, criterio D_registro ≥ 0,7
pre-registrato: metodo rispettato. Due incompletezze semantiche (non
capitali, la conclusione per Add int-int sopravvive):

1. **Il rimovibile NON è identico nelle tre forme**: in BinarySC/SCDst il
   hit-path paga anche `func.consts[cidx].to_zval()` (riga 1053) — ~0 per
   Long/Double, ma NON per ZStr (refcount bump). L'enunciato «SOLO carico
   payload + match b» è vero per SS, sovra-ampio come frase generale.
2. **Il controfattuale è hit-path-only**: la banda vale per giudici
   guard-always-hit (arith/add); il `.out` non lo scrive. Su workload
   miss-heavy il costo vive in `reg_load_slot` owned — fuori perimetro.

## Refutazioni capitali sulla bozza S-100

**R1 — Le sette fixture-trappola AssignOp sono SPARITE dai gate della
promozione.** Il backlog di NEXT le nomina (A-ST-100-2/3, «non slot di
sessione se non bloccante») ma il punto 1 di §S-100 elenca solo
A-KL-100-1/2, A-PE-100-4, A-HE-100-4. Col flip del default (punto 2)
l'emissione registro degli AssignOp diventa il percorso di TUTTI —
`$s += $i*3` è esattamente la forma del funnel antiputenv. Una precondizione
nominata che non compare nell'ordine che la consuma è un'equivalenza
dichiarata senza fixture: **bloccante, non backlog**.

**R2 — Il punto 4 tratta «Sub o cmp int-int» come fungibili. Non lo sono.**
Il perimetro di una specializzazione compare è più delicato di Add/Sub: i
cmp_op hanno il ramo __toString (oggetto vs stringa in loose compare, con
side effect e ORDINE osservabili), la loose equality numeric-string vive
SOLO in smart_streq («10»=="1e1", commento run.rs:108), Double porta
NaN/-0.0 nelle forme swapped (Gt = smaller(b,a)), e Spaceship cambia SEGNO
sotto swap degli operandi — rilevante per ogni normalizzazione const-lhs
tipo CmpJmpConst. Se il census sceglie cmp, questo perimetro va DICHIARATO
PRIMA del controfattuale, con fixture per classe; la bozza non lo dice.

## Emendamenti

- **A-ST-101-1**: ri-perimetrare l'enunciato 4a in NEXT: rimovibile
  per-FORMA (SC paga to_zval sul hit-path); banda valida solo
  guard-always-hit; conclusione D≈0 ristretta ad Add int-int.
- **A-ST-101-2**: se il census indica cmp, il controfattuale nasce DOPO la
  dichiarazione scritta del perimetro compare (__toString/ordine,
  smart_streq, NaN/-0.0, segno di Spaceship sotto swap, const_lhs) + una
  fixture per classe.
- **A-ST-101-3**: le sette trappole AssignOp entrano PER NOME nel punto 1
  di §S-100 come gate della promozione.
- **A-ST-101-4**: enumerare le classi di divergenza che vivono SOLO
  flag-on (ordine dei warning Undef con entrambi gli slot Undef in SS;
  fold/normalizzazione const-lhs su op non commutativi e di segno;
  store attraverso Ref-dst in reg_store_slot; fedeltà `lowered()`/dump) e
  mappare ciascuna su un gate esistente o una fixture nuova — il diff
  riga-per-riga (A-KL-100-2) NON fabbrica copertura che il corpus non ha.

## Kill-switch

- **KS-ST-101-1**: VIETATO il flip del default finché le sette fixture
  AssignOp non esistono e non passano byte-identiche nei DUE modi.
- **KS-ST-101-2**: ogni specializzazione compare sul percorso pila scritta
  PRIMA del suo perimetro dichiarato + fixture è VOID.
- **KS-ST-101-3**: il rollout nelle forme registro resta chiuso salvo
  MISURA con D_registro ≥ 0,7 ns/occ — un nuovo argomento statico non lo
  riapre.
