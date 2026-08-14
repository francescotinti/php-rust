# Criterio S-135 p.3c — leva «AP1 fast-path» (AssignPath 1-chiave su Array): commit PRIMA del codice

1. **Forma**: nel braccio `Op::AssignPath`, DOPO la probe ArrayAccess
   (invariata), caso `!append && nkeys==1` e slot base che È GIÀ
   `Zval::Array` (peek senza coercizioni): coerce_key (stesse diag) →
   `Rc::make_mut` → `set_returning_displaced` → `gc_note(displaced)` →
   push del valore. `LeafWrite::Busy` replica il drain Set di path_op
   (try_borrow_mut → replace → gc_note; Err ⇒ drain_fail_note). OGNI altro
   caso (base Ref/Object/Str/Null-vivify, append, nkeys>1, chiave
   illegale) cade nel cammino pieno INVARIATO. Zero contenitori nuovi.
2. **Equivalenza per costruzione**: il fast-path è la specializzazione
   letterale del pieno per quel caso (stessi passi, stesso ordine
   osservabile di diag/errori/gc_note; coerce eseguita SOLO dopo il peek
   Array ⇒ nessuna doppia diag); vivificazione e string-offset restano al
   pieno per costruzione del peek.
   **EMENDA a verbale (az.rev. S-135 #2, S-136)**: «stessi passi, stesso
   ordine» è FALSO alla lettera — il pieno fa `make_mut` PRIMA di
   `coerce_key_diag` (mod.rs:17233-17238), il fast il contrario
   (run.rs:2407-2416). La rivendicazione corretta è l'EQUIVALENZA
   OSSERVABILE, argomentata: `make_mut` è muto (nessuna diag/effetto
   visibile), quindi lo scambio commuta per ogni esito di coerce; l'unica
   differenza è che su chiave illegale il pieno de-condivide l'array prima
   del TypeError e il fast no — differenza di solo stato CoW interno, non
   osservabile da PHP (verificato dal revisore: stato post-errore ==
   oracle su array condiviso). Il codice resta invariato; si emenda la
   lettera del criterio.
3. **Giudice**: `objmap` (baseline s134-submicro 173,3 ns/iter, la leva
   morde il suo arm) — A/B R=5 ABAB vs stash `phpr-s134`, user CPU
   netto-pavimento per-binario, N dal sorgente, quiescenza gate separato,
   CI ferma (lock GIÀ in campo), smoke R=2 early-stop a segno opposto.
4. **Soglia** = max(4 ns, rumore drop-1 del run, spread-batch objmap s134
   = 0,03 s @3e6 → **10,0 ns**). **UB FALSIFICABILE (az.rev. S-134 #2) =
   47,7 ns/iter** = walk-plumbing 38,4 + residuo-arm 9,3 (prezzi MISURATI
   dal modello tempo `s135-tempo-verdetto.out`; il fast-path non può
   rimuovere più del plumbing che bypassa) + banda giudice 10,0 ⇒
   D ∈ (10,0 · 57,7] promuove col modello confermato; D > 57,7 = FUORI
   BANDA dichiarato (si promuove solo col reperto a verbale, sonda dovuta).
5. **Banda submicro↔A/B pre-registrata** (az.rev. S-134 #3): |Δsubmicro −
   D_A/B| ≤ spread-batch giudice (10,0) + spread submicro objmap (0,03 s →
   10,0) = **20,0 ns**; scarto oltre = riconciliazione a verbale PRIMA
   della promozione.
6. **Guardie SOLO-REGRESSIONE** nell'A/B (bande = spread s134-submicro
   phpr): objalloc 3,3 · objdatains 30,0 · objchurn 16,7 → soglia_reg =
   −max(4, SL); le sei micro restano al gate di promozione (catena s134,
   9 gate, pin SOLO via pin-phpr.sh/pin-server.sh emendati).
   **EMENDA (dopo il verdetto ab-ap1, PRIMA del rerun — rev. S-112)**: il
   run R=5 ha morso objalloc a D=−10,0 con banda 3,3 mentre il rumore
   drop-1 del run stesso era A'=6,7/B'=13,3 — banda sotto-fondata rispetto
   al rumore vivo. Guardie emendate alla STESSA formula del giudice:
   soglia_reg = −max(4, banda fondata, rumore drop-1 del run per la
   categoria). Il verdetto ab-ap1 (rc=5) resta agli atti; il rerun è
   `ab-ap1-r2` con TUTTO il criterio riapplicato (giudice compreso);
   una guardia che morde OLTRE il proprio rumore fa cadere la leva.
7. **Sonde di fedeltà PRIMA dell'A/B**: fixture rapide bilaterali su
   forme dim-set (ref-base, vivify null, stringa-offset, append, nested,
   ArrayAccess, chiave float/bool/null, Busy-cycle) — phpr candidato ==
   oracle, e candidato == stash s134 dove il pin è già conforme.
