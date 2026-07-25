# WP_SESSION_56 — Fase 3 pilota HASHED-ARRAY, tranche 1: indice keyless (tabella singola Zend-style) — footprint −1,79%, full −2,66% GRATIS, parità totale

> ⚡ **WP-56 (2026-07-26, `0943f58`)** — Prima tranche dell'arco Fase 3.
> **Ob.1**: `Repr::Hashed` perde la `FxHashMap<Key,u32>` (Key DUPLICATA,
> ~25B+ctrl/bucket) per **`KeyIndex`: open-addressing `Box<[u32]>` di sole
> posizioni**, chiave letta da `entries[pos]` (= arData + uint32 slots di
> Zend). `entries` INTATTE ⇒ ordine di iterazione, tombstone, posizioni
> slot, cursor, COW e dtor-order invariati PER COSTRUZIONE. Hash:
> `Key::Int` → fmix64(i); `Key::Str` → fmix64(zhash cached WP-29); probing
> lineare, occupancy (live+tomb) ≤ 1/2, rebuild dai VECCHI slot (mai
> rescan di entries: un'entry pushata ma non ancora indicizzata non può
> essere doppio-inserita); `compact()` ricostruisce l'indice a taglia
> live. Arena-compatibile (solo u32, zero puntatori). **Giudici**: ab56
> footprint peak **1564,4 → 1536,3MB = −1,79% (−28,1MB), 6/6 separazione
> pulita → 4,08×**; CPU media **−0,33% (new 5/6) → 2,61× invariato** =
> pilota GRATIS in CPU (checkpoint ≤+2% superato con margine alla prima
> sessione); full run45 **697,8s vs old (phpr-wp55) 716,9s STESSA-SERA =
> −2,66% (−19,1s) → 697,8/339 = 2,06× NUOVO MINIMO** (= riferimento
> WP-40 699s: residuo ≈14s CHIUSO; old replica run44 716,9 alla cifra =
> ambiente stabile). Parità: corpus **1421 IDENTICO** per nome · refl
> **290 IDENTICO** · cargo **1640/0** (nuovo test model-based churn) ·
> ORM 3484 **3E/13F IDENTICO** · hk 1665 **0E/0F** · fail-set full
> **BYTE-ID a run33 (88 nomi) su run45 E run45-old** · probe56
> order/tomb/numkey/dtor BYTE-ID new vs old (pinnate PRIMA del layout;
> order/tomb/numkey anche = oracle). Stash: `phpr-wp56` (sha256
> 65466c64…); old = `phpr-wp55` (ecc04817…).

## Ob.1 — pin rispettati e verdetti

- **Pin (a) size-class PRIMA del layout** (`wp56-harness/design56.md`):
  indice old 28,6–56B/el → new 8–16B/el = −21..−48B/el; entries 32B/el
  invariate. Actual sui MORTI (census, 4,75M dtor identici): **−62,3B/array
  medio alla cifra** (cum 2.943→2.647MB = −296MB churn/run).
- **Pin (b) sentinelle PRIMA del layout**: probe56-{order,tomb,numkey,
  dtor} su oracle+old; dopo il layout new BYTE-ID ×4. Trovata divergenza
  PRE-esistente (vale per old E new, non della leva): i `__destruct` dei
  valori RIMOSSI (`unset($a[k])`, mass-unset) sono DIFFERITI a fine
  script; l'overwrite (`$a[k]=new`) è puntuale — da catalogare in
  PHPR_DIVERGENCES.
- **Pin (c) mechanism-check: banda canale NON verificata — a verbale**:
  ex-ante −25..−45% del canale arr; actual −10,0% sull'estimatore
  (441,3→397,1MB) e −7,4% sul reached-set master a popolazione IDENTICA
  (714.790 array: 77,27→71,56MB = −8,0B/arr). La banda era quotata
  sull'ESTIMATORE live_n×death-avg, che sovrastima il canale arr **5,7×**
  (morti=churn grande 619B/arr; standing reale 108B/arr, quasi tutto
  packed/vuoto). Il per-shape della tabella è confermato dove la
  popolazione è hashed (subprocess reached: −20,2%, −67B/arr).
- **Pin (d) tranche gate-verde**: la tranche 1 chiude in sessione con
  gate pieno verde — nessun atterraggio parziale necessario.
- CPU full −2,66%: meccanismo coerente con l'attribuzione WP-54
  (malloc 4,4% + copie): −296MB/run di churn d'indice (alloc 24B/bucket +
  rehash swiss → slot 4B), Clone COW dell'indice a memcpy 6× più piccolo,
  compact-rebuild più economico.

## ⚠️ Correzione al checkpoint Fase 3 (input dell'arco)

Il "**arr ~421MB = 39% proxy**" del checkpoint WP-55 era un artefatto
dell'estimatore (over-count 5,7×): il walk_recon a fine run misura
**arr standing ~77MB (old) → 71,6MB (new)**; il margine arr/str scende da
10× a **~2,7×** (77 vs 29MB reached). La scelta hashed-array resta giusta
(e la tranche ha reso: −28MB di peak REALE sul media, −2,66% full CPU),
ma **le prossime tranche dell'arco vanno quotate su un metro non-biased**
(live-accounting esatto del canale arr o walk al watermark, non
death-avg). Stessa patologia già nota per str (4,9×, WP-55/52).

## Ob.2 — quota `.=` residui (metà probe FATTA)

Probe complessità 5k×1KB (`probe56-concat-sites.php`), phpr new vs
oracle: **local 2,3ms (fuso WP-55, =oracle) · element 517ms · prop 541ms
· static 592ms · nested 576ms (oracle ~1ms su tutti)** ⇒ il canale O(n²)
è VIVO su TUTTI i siti non-locali, ~105-118µs/evento a questa taglia.
La metà FREQUENZA per sito (op-census su AssignOpPath/PropOpSet/
StaticPropOpSet con op=Concat) è rimandata a WP-57: il tree era congelato
per i gate/giudici (lezione WP-33) e l'estensione del fuso si apre solo
coi secondi alla mano.

## ⭐ Lezioni

- ⭐⭐ **L'estimatore live×death-avg mente quando i morti non somigliano
  ai vivi** (churn grande, standing piccolo): arr sovrastimato 5,7×,
  str 4,9× (già noto). Le decisioni di arco (checkpoint Fase 3) vanno
  prese sul reached-set a popolazione confrontabile o su live-accounting
  esatto — MAI sull'estimatore.
- ⭐⭐ **Una banda di canale si quota sulla DISTRIBUZIONE, non solo sul
  per-elemento**: la tabella per-shape era giusta alla cifra (−62,3B/arr
  sui morti), ma la banda −25..−45% presupponeva hashed-dominance che la
  popolazione standing non ha (108B/arr medio = quasi tutto packed/vuoto).
  Predizione-misurata richiede ANCHE l'istogramma per-repr.
- ⭐ **L'indice keyless è stato gratis in CPU alla prima tranche** (media
  −0,33%, full −2,66% GUADAGNATI): il probing lineare α≤1/2 con confronto
  chiave attraverso `entries[pos]` non perde nulla contro la swiss-SIMD
  quando l'hash per-chiave è cached (zhash) e il churn d'allocazione cala
  di 296MB/run. Il modello Zend regge anche in Rust safe.
- ⭐ Il pattern "rebuild dai VECCHI slot, mai rescan di entries" rende
  l'ordine push/indicizza irrilevante e elimina per costruzione la classe
  di bug double-add sotto grow.
- ⭐ Sentinelle pre-layout ripagate subito: la divergenza dtor-differiti
  è emersa DALLA baseline old (non introdotta) — senza pin sarebbe stata
  attribuita alla leva.

## Prossimo (WP-57)

1. **Ob.2 metà frequenza**: op-census esteso (Concat su AssignOpPath/
   PropOpSet/StaticPropOpSet, conteggio + byte) → ns/evento×frequenza; se
   i secondi lo giustificano, estensione del fuso append-in-place ai siti
   prop/element (mirror di ConcatAssignSlot; sentinelle probe55/56 già
   pronte).
2. **Fase 3, tranche 2**: ri-quota dell'arco su metro NON-biased (arr
   live-accounting esatto o walk al watermark) PRIMA di disegnare
   l'arena handle-based; con standing arr ~72MB il ROI dell'arena va
   ri-stimato onestamente (la direttiva "tutte le fasi comunque" regge:
   si esegue, ma le tranche si ordinano coi numeri veri).
3. Divergenze a catalogo: dtor-differiti-su-unset (nuova, WP-56) +
   famiglia `.=` (WP-55) in PHPR_DIVERGENCES_FROM_PHP.md.
