# Criterio S-136 p.3b — leva «FD1 fast-path dim-write su proprietà» (IC su FieldAssign): commit PRIMA del codice

1. **Forma**: cella `PropIc` aggiunta a `Op::FieldAssign` (2 siti emit).
   Fast path SOLO per `steps == [Prop(name), Index]`: base che risolve a
   oggetto PLAIN (per-oggetto: no lazy, no proxy, no enum-case, borrow_mut
   libero — il busy cade al pieno che ha la sua rotta contata), IC hit per
   (classe, scope), e slot child che È GIÀ `Zval::Array` (peek senza
   coercizioni). Al hit: `field_write_walk(child, steps, 1, …)` RIUSATO
   LETTERALMENTE (il passo Index — ensure/make_mut/coerce/set/append-error —
   è LO STESSO CODICE del pieno) + `gc_note(dropped)` e `drain_aa_pending`
   come `field_set_mode`. OGNI altro caso (miss IC, child Ref/Str/assente,
   nkeys≠1, base non-oggetto, denied) cade nel cammino pieno INVARIATO.
   Zero contenitori nuovi.
2. **Fill** (fatti di classe provati, modello S-134 NP): dal cammino pieno
   del ramo F4, SOLO a esito Ok e SOLO se: resolve → `Slot` con
   `key == name` e slot-index stampato · NON readonly (`prop_readonly_decl`
   none: il container-guard su readonly DEVE restare) · asym-write OK per lo
   scope cachato (idem) · la classe è già senza prop-hooks (garanzia F4).
   `__set` irrilevante al hit: slot PRESENTE ⇒ il pieno non lo consulta.
3. **Bypass al hit** (dal modello s136-tempo, prezzi MISURATI):
   walk_driver 37,2 · resolve 6,7 · guardia 11,3 · prop_step_altro 14,4.
   **UB FALSIFICABILE = 69,6 ns/statement** — tetto per costruzione: dentro
   guardia/altro stanno quote RESTITUITE dal fast path (child-fetch, borrow)
   non prezzate separatamente, quindi il guadagno vero è < della somma.
4. **Giudice**: `objdatains` (1 statement dim-write/iter, N=3e6 dal
   sorgente; baseline s135-submicro 1060,0 ns/iter) — A/B R=5 ABAB vs stash
   `phpr-s135`, user CPU netto-pavimento per-binario, quiescenza gate
   separato, CI ferma (lock in campo), smoke R=2 early-stop a segno opposto.
5. **Soglia** = max(4, rumore drop-1 del run, spread-batch objdatains s135 =
   0,04 s @3e6 → **13,3 ns**). D ∈ (13,3 · 82,9] (= UB 69,6 + banda 13,3)
   promuove col modello confermato; D > 82,9 = FUORI BANDA dichiarato
   (promozione solo col reperto a verbale + sonda di ripartizione dovuta).
6. **Banda submicro↔A/B**: |Δsubmicro objdatains − D_A/B| ≤ 13,3 + 13,3 =
   **26,7 ns**; scarto oltre = riconciliazione a verbale PRIMA della
   promozione. m-dimwrite = OSSERVAZIONE (non giudice).
7. **Guardie SOLO-REGRESSIONE** (formula del giudice, rev. S-112):
   soglia_reg = −max(4, spread s135, rumore drop-1 del run per categoria);
   spread s135: objalloc 13,3 · objmap 3,3 · objchurn 6,7 · objallocni 10,0.
   Una guardia che morde OLTRE il proprio rumore fa cadere la leva. Le sei
   micro restano al gate di promozione (catena 9 gate, pin SOLO via
   pin-phpr.sh/pin-server.sh).
8. **Sonde di fedeltà PRIMA dell'A/B**: fixture dedicata dim-write su prop
   (typed/untyped · unset-prop → __set/dynamic · readonly drill = errore ·
   asym set-privata da fuori = errore · prop con Ref · stringa-prop offset ·
   nested 2 chiavi · append su prop · classe con __get/__set · scope
   privato) bilaterale oracle==candidato==stash dove il pin è già conforme,
   + fixtures-ap1-v2 rieseguita ==. Il candidato porta ANCHE az.rev. #3
   (contatore AP1_BUSY, cargo check rc=0) — dichiarato nel churn.
