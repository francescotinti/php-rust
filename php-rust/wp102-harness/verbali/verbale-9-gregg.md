# Verbale Sedia 9 — Brendan Gregg (metodologia di misura, attribuzione) — Concilio WP-102, mandato inverso

## §FONDAMENTALI

**(a) Avanzamento dell'oggetto in S-100 — misure vere, per nome.** Sessione
ricca sull'oggetto: (1) H-B2-sotto-flip decisa CON misura — isolante
dump-verificato L=12,9 ns/occ ≥ pavimento 1,0, estensione eseguita,
contro-misura L'∈[−1,0], giudice add on 3,25 netto (−31% vs off); (2) H-C
prima misura completa — prop on 12,4× decomposto in conteggio 2,0× (census
opcache 9 vs 18 op/iter) × costo/op 6,2× (1,56 vs 9,67 ns/op), profilo
co-equale con simboli per NOME (~27% ciclo vita Zval), candidata H-C1
nominata senza scrivere righe; (3) coppia WP nei due modi stessa-sera con
bande pre-registrate (CPU 1,008/1,004; peak 0,981/0,965); (4) fatto di
strumento NUOVO: oracle full-peak +14,6% intra-sera. Il flip è stato
giudicato da misure, non da fede.

**(b) Contatore sessioni-senza-misura**: **0** (riga ⏱ di NEXT_SESSION:
ultima full/media = WP-100, questa sessione; campagna oggetto = S-100).

**(c) Rischio d'oggetto più trascurato**: la baseline del giudice è oggi
ETEROGENEA — prop e add misurate in modo default, le altre quattro
categorie ferme a S-99 flag-off. Se H-C1 si iscrive prima della
ri-baseline sei-categorie (punto 1 della bozza), il suo criterio nasce da
numeri di due regimi diversi. Secondo rischio: +95 MiB full-peak OFF
cross-albero senza attribuzione — la gamba di rollback invecchia al buio.

## VERDETTO

S-100 è una sessione d'oggetto piena: flip promosso con gate misurati,
H-C decomposta e meccanismo NOMINATO. **Nessuna refutazione capitale.**
Tre refutazioni di metodo e quattro emendamenti.

## Refutazioni (non capitali)

**R1 — Il "costo/op ~9-10 ns quasi invariante" non è un invariante: sono
DUE punti** (prop 9,67; residuo arith 9,9). Usarlo come tariffa predittiva
ripeterebbe l'errore 57/43. Osservazione legittima, tariffa no.

**R2 — Il 27% ciclo-vita-Zval è un PAVIMENTO, non una quota**: il profilo
è top-of-stack e run_loop assorbe il 50% come "dispatch+handler inline" —
massa NON attribuita che può contenere altri clone/drop inlinati.
Simmetricamente, "zero simboli alloc" sull'oracle prova invisibilità alla
granularità dei simboli, non assenza di costo refcount. Il criterio di
H-C1 non può derivare la soglia dal 27%.

**R3 — La sanity ±5% della decomposizione è VACUA**: 12,4 = 2,0 × 6,2 è
esatta per costruzione (entrambi i fattori derivano dagli stessi T e
census: il prodotto ricostruisce il rapporto per identità algebrica).
Manca la sanity INDIPENDENTE: il census è STATICO (op/iter dal dump); la
conferma che 18 op/iter vengano davvero eseguiti esige il contatore
DINAMICO (feature op-census, esiste da WP-33..38) — oppure un micro
variante con corpo diverso che riproduca ~9,7 ns/op.

Sul metodo co-equale in sé: sano. Il campionamento è A TEMPO (4 s per
lato, durate 4,7 vs 5,2 s comparabili): i 300M vs 30M iter non introducono
bias di campionamento — a regime i campioni pesano il tempo, non le
iterazioni. Il difetto è solo l'attribuzione inline (R2).

## Emendamenti

- **A-GR-102-1**: prima del giudizio su H-C1, validare il census statico
  (18 vs 9 op/iter) col contatore dinamico op-census sul lato phpr.
- **A-GR-102-2**: decomporre il 50% di run_loop — riprofilo con
  attribuzione inline-aware (frame pointers/debug info o `#[inline(never)]`
  temporaneo sui candidati) per dare a H-C1 una BANDA di meccanismo.
- **A-GR-102-3**: bande sul full-peak d'ora in poi ASIMMETRICHE e
  calibrate sul rumore misurato (+14,6% intra-sera oracle); i gate
  anti-regressione sono a UN lato.
- **A-GR-102-4**: la ri-baseline sei-categorie in modo default (bozza §S-101
  punto 1) è PREREQUISITO del criterio di H-C1, non passo parallelo.

## KS

- **KS-GR-102-1**: la soglia di caduta di H-C1 nasce dal SUO micro
  isolante ≥ pavimento sonda — mai dal 27% del profilo né dalla "tariffa"
  9-10 ns/op.
- **KS-GR-102-2**: nessuna banda su metrica di peak più stretta del rumore
  run-to-run MISURATO dello strumento nella stessa finestra; violazione ⇒
  gate VOID.

## Sulla bozza §S-101

L'ordine È misura-first (ri-baseline al punto 1, H-C1 iscritta col
controfattuale al 2, apparato timeboxato al 4-5): approvato con gli
emendamenti sopra, in particolare A-GR-102-4 che rende vincolante la
precedenza del punto 1.
