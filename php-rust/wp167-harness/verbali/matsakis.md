# Verbale — sedia MATSAKIS (ownership/aliasing/borrow) — S-167, Fase 1

## VERDETTO: GO-CONDIZIONATO
**Rotta in una riga:** R1 fuso con la parte traffico-operandi di R3 — BinOp a 3 indirizzi slot-diretti sopra il reg-lowering, refutando R2-per-arith e l'arena.

## Refutazione dal mio angolo
1. **L'ipotesi «è il move/clone dell'enum grande» è FALSA a metà.** Zval è
   16 B, pinnata a compile-time (`php-types/src/array.rs:93`) — pari a
   Zend. Il move di un `Long` sono 2 registri; il clone caldo su arith non
   tocca mai Rc. Il costo NON è la taglia né la copia in sé.
2. **Il costo è il ROUND-TRIP di memoria imposto dall'architettura a
   pila:** ogni op fa `self.frames[top].stack` (doppio Vec-index + bounds
   check) per pop/last/push (run.rs ~1707-1714): il risultato scende in
   memoria e l'op successiva lo ripesca — store+load+capacity-check per
   valore, più il match sull'Op. Coi borrow ≤1 ns (L-TD1) e alloc ≈0
   (L-AL3), è l'unico canale residuo compatibile coi 36,3 di dispatch
   firmato. È R1 + traffico (ARCO REGISTRI completato), non R2, non arena.
3. **R3-arena: NO.** (a) non compra nulla — attacca il canale alloc già
   prezzato ≈0; (b) rompe l'ordine dei Drop (gc_queue FIFO, distruttori
   eager) su mass-drop di regione; (c) i valori ESCONO dalla richiesta
   (RetainSet, binding output-capture): servono handle generazionali, il
   cui check dinamico equivale al borrow già prezzato ≤1 ns ⇒ guadagno
   nullo per costruzione (già istruito in S-143 R3).

## Emendamenti
- **E1**: R2 su arith rinviata; veto NaN-boxing CONFERMATO per questo
  canale (parità 16 B con Zend: non c'è taglia da comprare).
- **E2**: chiudere PRIMA il quesito «quanto del dispatch 36,3 sopravvive
  col reg-lowering», con superop GENERICA (3 indirizzi), mai fusione
  cucita sul driver (benchmark-gaming).
- **E3**: disasm bl-count di run_loop prima/dopo obbligatorio (lezioni
  H-C2, MC1 inline); outline `#[inline(never)]` se cresce.
- **E4**: nessuna leva può cambiare taglia/layout Zval: pin 16 B resta
  kill-switch di compilazione (KS-HE-104-1).

## Kill-switch
Δ arith < 4 ns/iter sulla fetta ⇒ canale dispatch-per-op REFUTATO a
questa grana ⇒ stop campagna, si delibera R4. Pin Zval violato o
bl-esplosione non outlinata ⇒ fetta VOID.

## PRIMA FETTA
Sonda **R1-3A**: `BinOp` 3 indirizzi Long-guarded (+,−,*,>>) slot→slot,
fallback al funnel intero. Giudice: micro arith R=5 ABAB netto pavimenti,
con str/calls-fn/mc2 NON peggiorati oltre banda-layout fondata. Soglia:
promozione ≥8 ns/iter su arith; pavimento 4. Gate pieno: corpus 1412×2
per NOME + batteria + coppia WP al pin nuovo.
