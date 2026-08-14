# S-139 — istruttoria leva dim-read (famiglia FD1 sbloccata) — SOLO piano, nessuna esecuzione in finestra di misura

Prerequisito (criterio rmw-collaudo p.5): collaudo CALDO RMW chiuso PRIMA di
spedire qualunque leva nuova della stessa famiglia IC.

## Domande aperte (ordine di risposta, tutte post-finestra)
1. **Lowering**: `PHPR_DUMP_OPS` su `m-dimread.php` — la read leaf
   `$x = $e->data['k']` scende come op unica (FieldRead [Prop,Index]) o come
   coppia (PropGet; FetchDim)? Il perimetro della leva dipende da questo.
2. **Riferimento vivo**: m-dimread R=5 su pin s138 E oracle (banda dal drop-1,
   metodo REGOLE §3) — PRIMA cifra della categoria, oggi non esiste.
3. **Modello del tempo** (metodo S-136, chiusura ≥90% dichiarata): probe build
   con timer nominati (MAI nel pin), canali per NOME attesi: dispatch ·
   resolve/IC · walk read · clone del leaf · plumbing. NB il read NON muta:
   candidato borrow/copy-scalare senza make_mut — da provare, non assumere.
4. **Peso nel reale**: la rimisura ORM/dbal di stanotte (verdetto s139) è il
   primo indizio se il canale read pesa nelle suite; il profilo SUITE decide
   la priorità dim-read vs alternativa (REGOLE §1: leva dai numeri).
5. **Perimetri adiacenti dichiarati FUORI** (misurare prima la forma
   dominante): isset/empty (DimIsLeaf), coalesce (CoalesceFetchDim), read
   nested multi-chiave, base non-$this/global.

## Criterio (da PRE-REGISTRARE in file proprio prima dell'A/B)
Segno atteso ↓ su m-dimread; soglia = max(4 ns/iter, rumore drop-1,
banda-layout); R=5 ABAB; guardie SOLO-REGRESSIONE sui giudici delle leve
precedenti (m-dimrmw, m-diminc, objdatains, micro sei categorie); fixtures
byte-id bilaterali; conferma post-pin (lezione S-138: il gemello di relink
non eredita il verdetto).
