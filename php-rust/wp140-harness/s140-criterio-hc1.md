# S-140 — criterio LEVA HC1 «hint-check senza clone» (pre-registrato PRIMA di ogni A/B)

Meccanismo (dal profilo SUITE, verdetto s140-profilo): `coerce_or_check_hint`
apre con `value.deref_clone()` ma i rami check-only (Array/Object/Class/
Callable/Iterable/Union-exact) usano `v` SOLO per ispezione e ritornano
`Ok(value)`: il clone muore a fine check (Rc++/-- sprecato su Object/Array/
Str). Chiamanti sul cammino caldo: `enter_callee` (per parametro tipizzato),
`run_loop` return-hint, `coerce_typed_prop_write`, `typed_ref_assign`.

**Leva**: ispezione borrow-first quando `value` NON è `Zval::Ref`; il ramo
`Ref` mantiene il `deref_clone` di oggi (byte-identico per costruzione).
**Costo sostitutivo dichiarato**: un match sulla variante + borrow (nessuna
alloc, nessun refcount) — il canale rimosso è clone+drop del valore per check.

1. **Giudice**: `m-hintcall.php` (N=3e6 dal sorgente; 2 check Object/iter:
   parametro + return). Segno atteso ↓ (D>0 = B più veloce).
2. **Soglia** = max(4 ns/iter, rumore drop-1 del run, banda-layout PROPRIA
   del giudice da `s140-banda-verdetto.out` — misurata pin vs build
   `--features null-lever`, R=5 ABAB, PRIMA dell'A/B). A/B ABAB, R=5,
   user CPU `/usr/bin/time -p`, floor med3 per binario; smoke R=2 con
   early-stop a segno opposto; harness = COPIA DICHIARATA di s138-ab-rmw.sh.
3. **Sonda meccanismo (contatore prima dell'orologio)**: probe zval-census —
   pre-leva `hint_checks=6e6 (±1%) hint_checks_rc=6e6 hint_avoided=0` (lo 0 è
   per costruzione); post-leva `hint_avoided≈6e6` e rc invariato come conteggio
   dei check. Conteggi fuori attesa ⇒ modello sbagliato, leva FERMA.
4. **UB falsificabile**: il canale rimosso = 2×(clone+drop di Zval::Object)/iter
   ≈ 2×(Rc++ + Rc-- + gc_note del drop evitato); prezzo NON chiuso da A/B
   precedente ⇒ UB dichiarata dalla coppia di riferimento indiretto
   RECV (S-101, canale ricevitore ~ordine 10–20 ns per coppia clone+drop):
   D atteso 10–40 ns/iter; D > 80 = FUORI MODELLO, sonda dovuta.
5. **Guardie SOLO-REGRESSIONE (R=5, drop-1)**: le sei micro (SL) + objdatains
   (banda 13,3) + m-dimrmw + m-diminc (banda 5,0 dalla conferma post-pin
   S-138). Output giudice/guardie byte-uguali A vs B (diff secco: nessuna
   divergenza a catalogo su questi giudici).
6. **Gamba segnalata (az.rev. S-139 #3)**: quiescenza fallita o rustc vivo ⇒
   tentativo NULLO (si ripete con TAG nuovo); nessuna adiudicazione fuori
   verbale.
7. **Finestra (az.rev. #2)**: lock sessione presente; NIENTE push a finestra
   aperta (commit locali; push a finestra chiusa); pgrep rust-analyzer PRIMA
   di ogni blocco di misura (Serena lo rilancia: kill dichiarato).
8. **Promozione** (tutte e tre: giudice ≥ soglia, riconciliazione smoke↔R5 in
   banda, guardie ok): catena pin scripts/pin-phpr.sh + pin-server.sh, poi
   batteria, corpus 1414 ×2 per NOME, fixture bilaterali, micro R=5, e — la
   leva tocca arg-passing — **gate ORM 3484 3E/13F per NOME + conferma
   post-pin del giudice in banda** (lezione S-138: il gemello di relink non
   eredita il verdetto).
9. **Riferimento categoria** (contesto, non gate): m-hintcall bilaterale
   oracle vs pin R=5 — prima cifra della categoria hint-check.
