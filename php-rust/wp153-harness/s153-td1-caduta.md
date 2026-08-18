# S-153 p.3 — L-TD1 CADUTA — istruttoria (AGGIORNATA al verdetto GEMELLO R=5)

## ⚖️ VERDETTO FINALE (gemello, R=5 di record — `s153-ab-td1g-verdetto.out`, rc=4)
- A=f95a1067 (gemello tree) vs B=297cffc9 (stesso tree + soli edit L-TD1;
  build riproducibile ×2 al byte): **D=−3,3 ns/iter, soglia 4,0** (rumore
  drop-1 3,3/3,3) ⇒ SOTTO SOGLIA; guardie 12/12 ok; companion +3,3/+6,7 al
  rumore; riconciliazione smoke↔R5 FUORI BANDA (il +6,7 dello smoke era
  rumore: a verbale). Revert eseguito e verificato AL BYTE contro il gemello.
- **Reperto pulito**: 4 borrow/iter rimossi PER COSTRUZIONE ⇒ |D| ≤ soglia 4
  ⇒ **prezzo in-contesto ≤ ~1 ns/borrow** nei siti teardown/sweep, contro il
  mock hot-hot 4,27–4,41 (maggiorante non rappresentativo, come già
  qualificato dal §9 del gonogo — qui falsificato sperimentalmente).
- Il primo smoke (D=−5,0, braccio A=pin non-gemello) resta DECLASSATO a
  contaminato (emenda s153-criterio-td1-emenda.md, catturata nell'atto):
  il confronto pin-vs-gemello mostra che il drift di layout dei due commit
  cfg-gated post-pin vale ~±5–10 ns/iter su questo giudice.

## Sotto: prima stesura (smoke contaminato) — SUPERSEDED, conservata agli atti

## Esito secco (verdetto `s153-smoke-td1-verdetto.out`, rc=4 da file)
- Giudice objdropdef R=2: **D=−5,0 ns/iter** (A=1023,3 B=1028,3; soglia 10,0;
  segni **2/2 OPPOSTI** all'atteso) ⇒ early-stop, **niente R=5** (criterio p.4).
- Parità output A==B su 12/12 categorie; guardie 9/9 ok; companion
  objchurn +5,0 / objdatains +3,3 (sotto il loro rumore, non citabili).
- B=297cffc90664f03a (build canonica 3m34s, quiescenza rc=0, lock mio,
  attesi smoke PROMOSSI dal secondo attore PRIMA del run — 4 note non
  bloccanti nel suo verbale, tra cui il floor 4.0 sulle guardie non previsto
  dal criterio: DICHIARATO, non ha morso).

## Meccanismo NOMINATO della caduta
L'edit rimuove **4 borrow/iter CERTI per costruzione** sul cammino del
giudice (sonda k esatti: free-seq 5→2 = −3; ctor-teardown $this 2→1 = −1);
il tempo non si muove (D=−5,0 entro soglia 10). ⇒ il prezzo di un borrow
IN CONTESTO teardown/sweep è **≈0 ns** (indistinguibile da zero alla
risoluzione del giudice; bound: |prezzo| ≤ soglia/4 ≈ 2,5 ns), contro il
mock hot-hot c2_borrow 4,27–4,41. Il mock isola il borrow come cammino
critico; nel codice reale le RefCell-op si sovrappongono al lavoro
circostante (OoO) e non allungano il cammino. È la stessa qualifica già a
verbale nel gonogo (§9 «maggiorante nel modello hot-hot»): qui FALSIFICATA
sperimentalmente sul sito più caldo della famiglia.

## Conseguenze (senza chiusure di fronte — feedback-no-front-closure)
1. **Revert al byte** eseguito (git restore + rebuild → hash atteso
   cbbe7173, esito nel report di sessione).
2. Il verdetto **NO-GO A3c ESCE RAFFORZATO**: la banda_netta [1,109;1,324] s
   era prezzo mock×conteggi; il prezzo in-contesto ≈0 la SGONFIA.
3. **Fetta 2 (PropSetPop 3+1→1+1) ri-prezzata al reperto**: 2 borrow × ≈0 =
   attesa nulla ⇒ retrocede in coda; NON cancellata (contesto diverso dal
   teardown: dentro run_loop l'effetto icache/inline può differire — ogni
   eventuale ripresa pretende attesa fondata su prezzo IN-CONTESTO nuovo).
4. La coda A3a/A3b si riordina: davanti tornano le fette **ALLOC** (BT2-alloc
   m-backtrace, class_exists no-alloc) — il prezzo per-alloc è misurato
   BILATERALE (pair 6,9–11,7 ns) e BT1 ha già dimostrato che tagliare alloc
   paga (ORM −6 s). I conteggi per sito della famiglia borrow restano validi
   come MAPPA (la sonda è riusabile); muore solo l'attesa economica sui
   borrow in questi contesti.
