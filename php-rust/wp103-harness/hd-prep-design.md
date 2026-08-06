# H-D — design della CIFRA NETTA (S-103 punto 4, KS-LE-104-1: nessuna leva
# senza chiusura per-sito)

«2 alloc/chiamata, ~35 B» è ESISTENZA, non cifra (RC-LE-104-7): realloc
conta doppio anche in-place, linearità mai misurata, siti mai attribuiti.
Estensioni MINIME a `php-types/src/memcensus.rs` (feature `mem-census`,
nessuna build di parità le vede):

## 1. Realloc DISAGGREGATO (A-LE-104-1)

Oggi `CountingMi::realloc` fa `galloc_note(new)+gfree_note(old)` ⇒ un
realloc in-place appare come 1 alloc + 1 free pieni. Aggiungere:
- `GA_REALLOC_N` (eventi realloc), `GA_REALLOC_OLD/NEW_BYTES` (somme).
- `galloc_note`/`gfree_note` NON più chiamate dal ramo realloc: il dump
  pubblica le tre famiglie SEPARATE (alloc puri, free puri, realloc) e
  il netto si calcola senza doppi conteggi. La riga storica del dump
  resta byte-identica; le nuove cifre su righe NUOVE (convenzione S-102).

## 2. Istogramma SIZE-CLASS (A-LE-104-2 implicita nel pacchetto)

Bucket fissi su `galloc_note`: ≤16, ≤32, ≤48, ≤64, ≤96, ≤128, ≤256,
≤512, ≤1k, ≤4k, >4k (AtomicU64 ×11). Il «~35 B medio» diventa una
distribuzione: 2 alloc/chiamata da ~16+~19 B è un'altra storia che
1 da 70 B — l'istogramma decide.

## 3. Tag PER-SITO thread-local RAII (A-LE-104-3 ≡ A-BA-104-4, residuo≡0)

- `memcensus::SiteTag` RAII: thread-local `Cell<u8>` col sito corrente;
  `galloc_note` attribuisce l'evento al sito attivo (default `Other`).
- Siti INDIZIATI da taggare nel call-path (run.rs/mod.rs, con Serena):
  `RetCell` (Rc del ret_cell — indiziato principale), `ArgsVec`
  (materializzazione argomenti), `FramePush` (push_frame/frame pool),
  `Diag` (accodamento diagnostici), `Other` (tutto il resto).
- **Criterio di chiusura: residuo `Other` ≡ 0 per-iter** (al netto
  dell'avvio, delta calls_small↔calls): se Other > 0, l'enumerazione è
  incompleta e si itera PRIMA di nominare la cifra. (A-BA-104-5 backlog
  copre il sito Other + grow della pila come voce propria.)

## 4. calls_small (linearità, A-LE-104-1 coda)

`wp97-harness/micro/calls.php` in variante 100K iter (100:1): stessa
disciplina del 300:1 di prop — i rapporti per-sito devono reggere il
cambio di scala, le code costanti si cancellano nel delta.

## Ordine di esecuzione (finestra build S-103, dopo ab.done)

build census (zval-census+mem-census) → calls + calls_small → tavola
per-sito con residuo≡0 → **cifra netta alloc/chiamata** in
`hd-cifra-netta.out` → SOLO POI si può nominare una leva (non in questa
sessione se il tempo non c'è: la cifra è il deliverable).
