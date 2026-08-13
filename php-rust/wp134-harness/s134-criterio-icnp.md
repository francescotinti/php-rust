# Criterio S-134 — leva «IC non-plain» (ultima resolve ctor): commit PRIMA del codice

Scelta S-134 tra le due aperture (dispatch 36,3 · ultima resolve ctor):
**ultima resolve ctor**. Motivo dichiarato: il disasm dell'eccedenza
(`s134-eccedenza-lettura.md`) mostra il sito già pulito (resolve esplicita,
risultato in registri); il dispatch tocca la struttura di run_loop (classe di
rischio H-C2/WP-104; threaded-dispatch VIETATO a catalogo).

## p.1 — FORMA (modello IC non-plain, dal sorgente)

1. **Sede**: `PropIc` (bytecode.rs) resta UNA cella `(epoch, cid+1, scope+1,
   slot)`; lo slot u32 imbarca due bit alti: `NP` (1<<31, entry non-plain) e
   `TY` (1<<30, coercizione typed richiesta). Zero contenitori nuovi, zero
   alloc; la probe `ic.get` esiste GIÀ su ogni entrata → costo nuovo sui
   cammini non cacheabili = un branch di decodifica.
2. **Fill** (nel cammino pieno di `prop_set_entry`, dopo il check readonly e
   prima della coercizione) SOLO se TUTTI i fatti valgono:
   entrata coi check hook eseguiti (`!hook_guarded` percorso) e senza set-hook
   né virtual-hook per (classe, prop) · `PropAccess::Slot{key, slot: Some}` con
   `key == name` (esclude i private mangled per costruzione) ·
   `asym_write_error` None · `prop_readonly_decl` None · `__set`
   strutturalmente ASSENTE (`resolve_method_runtime(ocid, "__set")` None —
   allora il magic-set non può MAI applicarsi, qualunque stato d'istanza) ·
   bit TY = `prop_type_decl` Some. Precedente: le letture GET/ISSET fillano
   già oltre plain (WP-35); la frase «le scritture restano plain-only» nel
   doc di PropIc si EMENDA dichiarando.
3. **Hit** (ramo NP nel blocco IC esistente): guardie per-istanza IDENTICHE al
   ramo plain (cid match · lazy none · non-enum · slot presente; slot Ref solo
   con `typed_refs` vuoto, altrimenti MISS al cammino pieno) → poi, fuori dal
   borrow: se TY, `coerce_typed_prop_write` (stesso ordine del cammino pieno:
   coercizione PRIMA della scrittura, stessa esposizione a __toString; il
   TypeError propaga prima di ogni effetto) → `write_property_at(name,
   Some(slot))` + `gc_note(old)` + push (salvo DISCARD). Slot assente al hit =
   MISS (il caso unset-declared resta al cammino pieno, semantica invariata).
4. **Equivalenza dichiarata per costruzione**: il ramo NP salta SOLO lavoro i
   cui esiti sono fatti DI CLASSE cachati nella chiave (cid, scope): enum-check
   (guardato), hook (assenti), magic (irricevibile senza __set), resolve (slot
   cachato), asym/readonly (verificati al fill), deprecation (slot dichiarato).
   Restano per-scrittura: presenza slot, coercizione typed, typed_refs, write.

## p.2 — MISURA

1. **Giudice objalloc** (A=983,3→986,7 ns/iter su pin s133 = 940,0; ctor `En`:
   2 set tipizzati/iter, entrambi cacheabili) + **co-giudice objdatains**;
   entrambi sopra soglia per promuovere.
2. **Soglia = max(4 ns, rumore drop-1 del run stesso per gamba, spread-batch
   s133 gambe B)** — spread-batch dal verdetto `s133-ab-ctor-verdetto.out`,
   gambe B (=pin s133): **objalloc 2,80–2,88 s @3e6 → 26,7 ns** · **objdatains
   3,52–3,56 → 13,3 ns**.
3. **Parte modellata e componenti dichiarati** (lezione az.rev. S-133 #4): la
   resolve vale 2 × 17,7 = **35,4 ns/iter** (prezzo medio siti ctor, s131). Il
   ramo NP rimuove ANCHE componenti mai prezzati, elencati per NOME: probe
   magic (borrow + `props.contains` hash + is_typed_unset) · lookup set-hook +
   virtual-hook · borrow enum-check · `prop_info` di asym · `prop_info` di
   readonly · check dynamic-deprecation. **D atteso SOPRA 35,4**; la
   riconciliazione dichiara D−35,4 come somma dei componenti nominati (senza
   prezzarli singolarmente — magnitudine non ripartita, REGOLE §4). Resta
   pagato il canale typed: `prop_type_decl` + coercizione + clone.
4. **A/B**: `s134-ab.sh` = COPIA DICHIARATA di `s133-ab.sh` COLLAUDATA con
   `scripts/copia-gate.sh` + manifest committato (dente S-134 p.3) — soli
   cambi: A = stash `phpr-s133` c87439a9, B = candidato, soglie/bande di
   questo criterio, OUT wp134-harness. R=5 ABAB, user CPU netto-pavimento
   per-binario, N dal sorgente, quiescenza gate separato con rc in header,
   argv senza pattern del gate, **CI locale SOSPESA in finestra**. Smoke R=2
   early-stop a segno opposto; riconciliazione smoke↔R5 con banda =
   spread-batch del giudice (26,7 / 13,3).
5. **Guardie SOLO-REGRESSIONE** (bande fondate = spread-batch s133 gambe B):
   objchurn 10,0 · objmap max(4, 0,0)=4 · le sei micro con SL s133
   (arith 0,94 · prop 0,8 · calls 0,73 · str 2,89 · arr 2,49 · re 4,46),
   soglia_reg = −max(4, SL).
6. **Conteggi**: dal modello + sonda s133: resolve objalloc 2→0/iter a regime
   (fill alla prima passata per sito). Sonda a soli conteggi (apparato s133
   riusabile) SOLO se il verdetto A/B esce ambiguo: costo una build.

## p.3 — PROMOZIONE (se A/B sopra soglia)

Catena s133 riusata: `s134-promozione.sh` copia dichiarata di
`s133-promozione.sh` COLLAUDATA con copia-gate + manifest; divergenze ATTESE
nel manifest: tag s134, path wp134, **FX_ATTESI = 9 gate con `stash`**
(emenda S-134 della catena), commenti header. Batteria (rc dal comando,
inventario vs s125) · corpus-gate ×2 · fixture-chain s109 (9 gate) · micro
R=5 · ORM 16 nomi · hk 0E/0F · pin SOLO via `scripts/pin-phpr.sh s134` +
`pin-server.sh s134` (re-hash server post-build). Gate fallito ⇒ niente pin,
revert al byte verificato. Poi coppia WP per-config (ricetta s133-pair, file
nuovo) → REPORT_GAP_134; rimisura dbal/ORM sul pin nuovo se resta finestra.
