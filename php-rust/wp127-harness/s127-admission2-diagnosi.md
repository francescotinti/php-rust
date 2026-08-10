# s127-admission2-diagnosi.md — diagnosi del FUORI PREDIZIONE (criterio ab p.1: diagnosi PRIMA dell'A/B)

Verdetto `s127-admission2-verdetto.out`: D = −2,00 su objalloc/objallocni/objdropdef,
−1,00 su objdatains/objchurn, 0,00 su objmap (alloc≡free, R=2 identici, parità
output leva==pin 7/7).

**Modello unico che spiega 6/6**: Δ = −2 + w, con w=1 se il micro SCRIVE nel
default-array condiviso, 0 altrimenti.
- La predizione −1,00 contava solo l'array `[]` per-oggetto del thunk. Saltare il
  thunk risparmia in realtà **2** alloc/new: l'array `[]` + una allocazione della
  CHIAMATA-thunk (indiziato: `Op::InitProps` costruiva il frame con `Frame::new`
  diretto, NON dal pool `recycle_frame` — sentiero ora morto dopo il primo new).
- Le categorie che scrivono (`$e->data['k']=$i`) ripagano **+1**: clone COW
  dell'array condiviso alla prima scrittura (array vuoto: clone a costo minimo).
  Lettura-sola non copia MAI (guadagno netto sui carichi read-heavy).

Errore di predizione = sottoconto del costo del thunk, non un effetto ignoto:
la forma rimuove PIÙ del previsto e nessuna categoria peggiora. Nessun meccanismo
di regressione visibile (il caso peggiore, default non vuoto scritto subito,
sostituisce la costruzione per-oggetto con una copia per-oggetto + thunk saltato).
Si procede all'A/B (smoke R=2 poi R=5) come da criterio.

Disasm "dopo" (protocollo S-104, riferimento in s127-admission-lettura.md):
`alloc_object` 383→266 istruzioni, 36→**31 bl**; seeding in `fresh_props`
(208 righe/16 bl, ramo caldo = solo template.clone()).
