# Verbale sedia 4 — Hejlsberg (Concilio WP-95)

Perimetro: compilatori incrementali, interning/dedup, catene di evidenza e identità. Mandato: REFUTARE.

## VERDETTO
CON EMENDAMENTI. La leva per-file è la giusta prima mossa ma va emendata (pre-size, contatore per-unità PRIMA, vincolo d'ordine extends provato staticamente). L'identità dal contenuto (A-AH68) chiude il lato basename ma la catena stamp→attempts→OUT ha ancora TRE lati aperti nel `.done` e nell'autenticazione degli append — uno è capitale.

## Q1 — Leva per-file (nominata S-93.0 B3)
Fattibile e già semi-provata dai lifetimes: l'arena muore a fine `lower_prelude_uncached` (lower/mod.rs:904-1027) e i prodotti sopravvivono ⇒ nessun borrow evade, il Drop lo prova. I 6 unit separati (NS/BC/GMP/mysqli/gd/fileinfo) sono GIÀ hoist in chiamate distinte: splittare il concat di 9 file (mod.rs:747-757) è la stessa forma. DEVE PROVARE: (a) nessuna `extends`/`implements` in avanti TRA i 9 file — `parent` è risolto a `ClassId` AL lowering (hir.rs:207-210), quindi file k può riferire solo file ≤k: verificabile staticamente prima del codice; (b) contatore per-unità (A-AH-72) con predizione-misurata WP-48; (c) fp `main_chain_fp` INVARIATO (hasha i sorgenti, non l'arena — mod.rs:851-861): guardia d'identità gratis. Emenda tecnica: `Bump::with_capacity` per unit — la coda mai usata 13.738.592 B (huge-sites.out:80-83) è il costo della catena x2+16, evitabile senza cambiare altro.

## Q2 — Preludio PRE-COMPILATO (rkyv/bincode)
L'ostacolo `Rc<ClassDecl>` è più piccolo del temuto: dentro `ClassDecl` (hir.rs:198-278) e nei corpi NON ci sono `Rc` né `Cell` — gli `Rc` stanno solo alle tabelle (hir.rs:38, hir.rs:82; LoweredPrelude mod.rs:797-805), senza cicli né aliasing profondo: la dedup si ricostruisce al load (deserializza `Vec<ClassDecl>`, ri-avvolgi in `Rc`). Il vero costo: derive su TUTTA la foresta enum HIR + versioning. Legale SOLO come build-input embedded (`include_bytes!` da build-tool) dentro l'identità del binario — una cache su disco a runtime è una superficie di fabbricazione (KS-AH-95-2). DEVE PROVARE: dente che TIENE l'HIR plain-data (A-AH-73) e byte-determinismo dell'artefatto.

## Q3 — Preludio condiviso una volta per processo
Bloccato oggi da `Rc` !Send/!Sync a livello tabella: `Program.functions/classes` (hir.rs:38/82) e ogni consumatore. Serve Rc→Arc o `&'static ClassDecl` — ripple su tutta la VM, costo atomics sui path clone-caldi (unit cache WP-81). Aiuta SOLO il server (W−1 thread), zero sul CLI che è l'oggetto 4.42×. DEVE PROVARE PRIMA il numeratore: quota preludio-retained dei ~18,8 MB/worker (canale m91, A-MS-53) e l'immutabilità post-seed dei decl.

## Q4 — A-AH68: la catena regge?
Il lato basename è chiuso bene (bnc_judge unico predicato, battery-equivalence.sh:74-81, consumato da :214 e :83). Ma: (a) `DSHA` e `DMTX` sono estratti con `sed|head -1` da QUALUNQUE riga del `.done` (:229, :246) mentre `rev=$BREV` è cercato a parte (:224) — i campi possono nascere da DUE righe diverse: stessa classe del bug A-AH40; (b) il grep del triangolo A-AH54 (:374) è NON ancorato e la grammar ancora sha256 solo su FAIL/REFUSE/ABORT (:399) — le righe PASS sono esenti; (c) **capitale**: gli append in-window ai due ledger sono allowlisted (:430) e append-only (:450-470) ma MAI autenticati — `writer=script:<16hex>` è verificato solo in FORMA (:392), mai contro lo sha del battery script a HEAD: un consumo interamente fabbricato in-window (OUT+`.done`+stamp+riga PASS ben formate) non ha dente che lo morda.

## Q5 — Priorità S-94.0 (FONDAMENTALI-first: la leva È l'oggetto)
1) Leva per-file+pre-size col contatore per-unità stesso-commit, gate parità COMPLETI + battery-91pre alla prima ricompila; 2) misura CLI hello/refl post-leva vs oracle (il 4.42 deve muoversi); 3) battery61 nativo (criterio 5). Gli emendamenti al checker restano A VERBALE: apparato congelato (condizione 4), si attuano nella prossima finestra apparato.

## Emendamenti
- **A-AH-69**: `.done` parsato per-RIGA — i 4 campi dalla STESSA riga che porta `rev=$BREV`; più righe `rev=` ⇒ REFUSE (:229/:246).
- **A-AH-70**: ancora `sha256=[0-9a-f]{64}( |$)` anche sul grep triangolo (:374) e grammar-anchor esteso alle righe PASS (:399).
- **A-AH-71**: `writer=script:<h16>` deve eguagliare i primi 16 hex di sha256 dello script battery a HEAD — autenticazione, non forma (:392).
- **A-AH-72**: PHPR_PRELUDE_STATS v2 per-unità (`unit=<file> allocated=…`) PRIMA della leva.
- **A-AH-73**: test che `ClassDecl`/`FnDecl` restino plain-data (niente `Cell`/`Rc` interni) — precondizione della via precompilata.

## Kill-switch
- **KS-AH-95-1**: leva preludio attuata senza contatore per-unità pre-misurato ⇒ misura non consumabile.
- **KS-AH-95-2**: cache del preludio su DISCO a runtime ⇒ NEGATA; il precompilato è legale solo embedded nell'identità del binario.
- **KS-AH-95-3**: consumo battery con `.done` multi-riga a campi non accoppiati ⇒ VOID.

## Refutazioni capitali
**SÌ, una**: la catena stamp→attempts è autenticata in FORMA ma non in ORIGINE — append in-window allowlisted e mai legato allo script che li scrive (Q4c). I lati (a)/(b) sono aperti ma non capitali da soli.

## RANK leve
1. **Per-file + pre-size** (bassa superficie, il Drop già prova i lifetimes, serve CLI E worker).
2. **Pre-compilato embedded** (HIR è già serializzabile; solo se il residuo post-leva-1 resta ≥2× oracle — paga anche il parse CPU).
3. **Condiviso 'static/Arc** (solo server; prima il numeratore m91, poi la migrazione Rc→Arc misurata su full CPU).
4. **Lazy per-unità** (guadagno duplicato dalla 1; rompe l'invariante id-contigui mod.rs:793-796 e la parità reflection — superficie massima).
