# Verbale sedia 1 — HOARE (WP-108, revisione S-106 / programma S-107)

**VERDETTO: CONCORDO CON EMENDAMENTI.** Nessuna refutazione capitale.

## Verifiche eseguite (Read mirato, non da verbale)

- **Braccio VM** (run.rs:1207-1233): `BinarySTDst` usa `read_slot` — la
  STESSA funzione dell'arm `Op::LoadSlot` (run.rs:715), silenziosa e
  Ref-seguente — più `binary_value_ab` e `reg_store_slot` condivisi al
  simbolo col braccio `BinaryDst`. Profondità di pila identica al tris in
  OGNI punto fallibile (tris: +1+1−2; fusa: +1−1) ⇒ su errore di
  `binary_value_ab` lo slot dst resta INTATTO in entrambe le forme (lo
  store è dopo il `?`) e l'unwind vede la stessa pila. Confermato.
- **Finestra** (reg_lower.rs:483-491, 536-537): guardie ereditate reali
  (blocked su jump/exc-boundary, stessa source line, u16); `ST` senza
  coda restituisce `ops[i].clone()` a larghezza 1 — Swap e Binary
  rientrano nelle finestre successive, nessuna perdita. La scelta
  «nessuna forma value» è minimalità sana, veto già a registro: CONCORDO.
- **D-9** (run.rs:2627-2636): declassamento ONESTO — nomina il
  meccanismo (drop del wrapper Ref in `decay_arg` può liberare l'ultima
  ref), la condizione di equivalenza, e separa osservato da provato.
  Terza opzione di D-9 legittimamente esercitata; sufficiente FINCHÉ il
  dente resta in S-107 punto 1 e il direct-bind non si allarga.

## Refutazioni

- **R-HO-108-1 (D-11, non capitale)**: il sigillo riscritto blocca il
  SUO controesempio (`Bool→Box<bool>`: E0277, provato dal mutante) e,
  composto con size/align/niche (array.rs:88-99), prova davvero
  no-Drop/bit-copy (T:Copy ⇒ niente Drop). Ma un payload Copy-con-
  indirezione (`&'static T`, `*const T`, fn-pointer, 8B align 8)
  ATTRAVERSA tutti e quattro i sigilli: innocuo per il drop in safe-only,
  però il commento rivendica «trivial arms» — più di ciò che è provato.
  E la lista (3 costruttori, in array.rs, lontana da Zval e dal fast
  path consumatore) è ENUMERABILE: una quarta variante triviale futura
  non è coperta per costruzione — lezione WP-96.
- **R-HO-108-2 (famiglia Dst, ereditata — non introdotta da H-A1)**:
  «zero biforcazione» è provato verso `BinaryDst`, non verso lo
  spelling NON fuso: se `reg_store_slot` LANCIA (coercizione typed-ref,
  run.rs:301-306), il tris+`Dup;StoreSlot;Pop` lascia un temporaneo in
  pila al punto di throw, la forma fusa no — divergenza osservabile solo
  via timing di drop dei temporanei in catch (`__destruct`). Il rischio
  (ii) del criterio copriva la exc-region della FINESTRA, non lo store
  che lancia. Corpus ×2 identico = indizio, non copertura provata.
- **R-HO-108-3 (minore)**: l'equivalenza col tris poggia su un
  invariante IMPLICITO — il pop di rhs non tocca gli slot (la lettura
  dello slot è migrata DOPO il pop). Vero oggi; non scritto da nessuna
  parte. E la finestra fonde solo lo spelling `LoadSlot`: il compound
  assign in forma `LoadVar` (warning) resta fuori — sano, ma va detto
  come limite di copertura, non sottinteso.

## Emendamenti

- **A-HO-108-1**: il commento del sigillo dichiari SOLO ciò che prova
  («no-Drop/bit-copy», non «scalari»); lista dei costruttori sigillati
  co-locata o cross-linkata col fast path che consuma la trivialità.
- **A-HO-108-2**: dente throwing-store per l'intera famiglia Dst:
  compound assign via typed-ref che lancia nello store, in try/catch,
  operandi con `__destruct`, flag-on/off/oracle — in S-107 punto 1.
- **A-HO-108-3**: una riga nel commento BinarySTDst: «equivalenza al
  tris ⇐ il pop non muta gli slot»; nel doc del pass, il limite «solo
  spelling LoadSlot» esplicito.

## Kill-switch

- **KS-HO-108-1**: ogni estensione di BinarySTDst (forma value, rhs
  const, spelling LoadVar) senza criterio proprio pre-registrato E
  dente A-HO-108-2 verde = VOID.
- **KS-HO-108-2**: ogni allargamento del direct-bind mentre il commento
  drop-order resta declassato = VOID (continuità KS-MA-107-1).

## Ordine S-107

CONCORDO con la sequenza 1-5: denti prima, §3.15 testa di fedeltà, leva
dopo — fedeltà-prima-di-velocità è l'ordine giusto. Unico emendamento
d'ordine: il dente A-HO-108-2 entra nel punto 1 accanto a D-9/D-10; la
candidata leva (arith/prop) non tocca il call path, quindi nessun
conflitto con KS-HO-108-2.
