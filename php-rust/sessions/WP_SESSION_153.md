# WP_SESSION_153 — L-BT2 SPEDITA (debug_backtrace −36% sul giudice); L-TD1 caduta col prezzo-borrow falsificato; pin NUOVO s153
**In una frase**: abbiamo misurato quanto costa davvero ogni «prestito» degli
oggetti nei punti caldi (quasi nulla: la cura provata lì non paga ed è stata
ritirata) e in compenso abbiamo reso molto più economica la fotografia dello
stack (debug_backtrace), promossa a nuovo binario di riferimento con tutti i
collaudi verdi.

**SCOREBOARD** (pin NUOVO s153 8370c257ae70cc8e + server f030c6fcbddfab96):
arith 5,5 → · prop 5,5 → · calls 4,7 ↓ · str 4,3 → · arr 3,2 ↓ · re 2,6 ↑
(±1 tick, run-to-run) · WP t4 1,781 e ORM 7,104–7,149 = rif s150 (coppia al
pin nuovo DOVUTA → S-154 p.1) · corpus 1412×2 ZERO flip · **leve spedite: 1**
(L-BT2; tentate 2: L-TD1 caduta a verdetto R=5) · incidenti 19 (=; emenda
braccio-gemello DICHIARATA e catturata nell'atto, prima di ogni uso di record).

## Esiti secchi
1·**Ratifiche utente**: A2=T2-only · A3.0 confermato · census server in coda
  (3° slittamento dichiarato). 2·**Sonda conteggi per SITO** (probe s151, k
  ESATTI, `s153-teardown-conteggi.md`): teardown $this k=2 · slot k=1 ·
  free-seq sweep k=5 · PropSetPop k=3+1 (3° borrow = lazy_prop_access).
3·**L-TD1 CADUTA a R=5 GEMELLO** (D=−3,3 vs soglia 4; 4 borrow/iter rimossi
  per costruzione ⇒ prezzo in-contesto ≤~1 ns/borrow vs mock 4,27–4,41
  FALSIFICATO; NO-GO A3c RAFFORZATO; fetta-2 PropSetPop retrocessa; revert
  verificato al byte). **EMENDA §7-bis**: primo smoke col braccio A=pin
  CONTAMINATO dal drift dei 2 commit cfg-gated post-pin (±5–10 ns/iter) ⇒
  regola nuova: braccio A = GEMELLO costruito dal tree corrente.
4·**L-BT2 VINTA e SPEDITA**: chiavi statiche thread-local + BtFrame→ZStr in
  ho_debug_backtrace; A/B R=5 D=+266,7 ns/iter (733→467, −36%), segni 7/7,
  riconciliazione |0,0|, guardie 12/12, fx-backtrace byte-id; FUORI-UB sopra
  (>160+rumore) a verbale ⇒ sonda k post-leva DOVUTA (S-154). Promo rc=0:
  batteria 1748/0/2 (inventario s125+2 denti) · corpus 1412×2 zero flip ·
  fixture 10/10 · micro ±1 tick · conferma post-pin +333,3 (5/5; tick 66,7
  dichiarato) · ORM fail-set==16 nomi · hk 0E/0F · pin phpr+server s153.
5·**Dente A4 ha MORSO la promo t1** (host.rs +35, mod.rs +3 oltre cap) →
  salita DICHIARATA nei cap (protocollo del dente rispettato), promo t2 rc=0.
6·⚠️ Data a 2G (probabile update macOS staged): BLOCCANTE apertura S-154.

## ⭐ Lezioni (max 3)
- ⭐⭐ A pin invariato ma tree avanzato, il braccio A di un A/B si COSTRUISCE
  (gemello dalla ricetta), mai dallo stash: i commit cfg-gated spostano il
  layout e firmano D fantasma.
- ⭐⭐ Il prezzo mock hot-hot è un maggiorante, non un prezzo: una leva si
  prezza col costo IN CONTESTO (qui: 4 borrow certi ⇒ D≈0) — falsificare
  presto costa uno smoke, non una sessione.
- ⭐ Un dente che morde la leva promossa della PROPRIA sessione e si placa
  con la salita dichiarata è la prova che il protocollo anti-ricrescita regge.
