# Nota team «tetto-misura» — Concilio WP-99 (relatore)
Fonti: SOLO verbale-5-bak.md + verbale-9-gregg.md. Entrambi: CON EMENDAMENTI.

## 1. Convergenze (indipendenti, stesso numero)

**Tetto numerico condiviso**: dal marginale dello Sweep noop di ha2-sweep
(0,07 s / 50M dispatch) entrambe le sedie derivano **~1,4 ns/op come TETTO del
preambolo di dispatch**. Su `arith` flag-off: Bak lo scrive come −17% sul costo
per opcode (8,24 ns); Gregg come 19×1,4 = 26,6 ns su 156,6 ns/iter ⇒ tetto ~17%,
banda ~8–27% (il Δ 0,07 s è appena 2× lo spread). Corollario comune: **il
fattore ~8 residuo vive nei CORPI degli handler** (catene self→frames→slots,
push su Vec<Zval>, discriminazione di tipo/lavoro Zval), non nel preambolo.

**Conseguenze sull'ordine S-98.0**:
- **H-B1 è DECLASSATA**: non è «l'asse che chiude il fattore 8» (refutazione
  capitale n.2 di Bak; non-sequitur di Gregg). Se eseguita, lo è solo come
  sotto-passo in scope «preambolo», con criterio DERIVATO dal tetto (A-BA-99-1)
  e SOLO dopo la misura M1 con predizione P scritta (KS-GR-99-1). Lo scope
  «frame-esteso dentro i corpi» va dichiarato a parte (costo `&mut self`, Bak).
- **H-B2 è PROMOSSA** all'asse principale: entrambi la indicano come sede degli
  ~8,5 ns residui (A-BA-99-4; Gregg: decomposizione per-opcode degli 11 residui
  = «non misurato e dovuto»).
- Concordano anche sul metodo: il criterio −40% di H-A1 era irraggiungibile con
  dati già posseduti (Bak: uniforme = −42,1%, non-uniformità già misurata) —
  mai più criteri-cifra-tonda; si deriva dal controfattuale (KS-BA-99-1).

## 2. Ricetta di misura unificata (M1, zero codice VM) — che entrambi firmerebbero

1. **Micro noop**: `for($i=0;$i<200000000;$i++){}` con ricetta run-micro.sh
   (coppia stessa-sera, R=3, stesso binario) → **D = ns/dispatch** dei soli op
   economici, su ~4 miliardi di dispatch (statistica stretta, non il Δ 0,07 s).
2. **Census**: ops/iter del noop (attesi ~4-5: IncDecSlot, CmpJmpSC, Jump,
   Sweep[, Pop]) + census flag-on/off su prop e calls per attribuire i
   collaterali (A-GR-99-3); conteggio statico dei re-borrow `self.frames[top]`
   eliminati per opcode del residuo arith (A-BA-99-2 = la predizione-misurata).
3. **ASM del binario CORRENTE**: bounds check/len realmente emessi nel run_loop
   (prof95-media è di un'altra build; LLVM può già eliderne).
4. **Predizione P scritta nel .out PRIMA di ogni riga di codice**:
   P = 19·D/156,6 ns (quota massima di arith attribuibile al preambolo).
   Caveat: D è marginale in pipeline out-of-order ⇒ tetto soffice verso il basso.

**Criterio di caduta numerico congiunto**: H-B1 cade A TAVOLINO se P < 10%
(KS-GR-99-1); se scritta, cade se risparmio < 0,7 ns/op — metà del tetto —
ossia arith flag-off netto > 7,2 s (8,24 → ≤ 7,55 ns/op) sulla coppia R=3
stessa-sera (A-BA-99-1), e comunque se il calo < max(P/2, 3× spread relativo
≈ 1,5%). **Nulli i claim < 3× spread della propria coppia stesso-binario**
(KS-GR-99-2). Ogni confronto cross-binario porta `deriva_inter_build=`
esplicita (A-GR-99-4; deriva nota 0,6%); str/re riscritti come BANDE, vietata
l'etichetta «rumore» (A-GR-99-1: str ~[−11%, +4%]).

## 3. Conflitti

Nessuno sostanziale. Sfumatura: Bak dà il criterio operativo assoluto
(0,7 ns/op / 7,2 s) prima ancora di M1; Gregg subordina TUTTO a P misurata.
Composizione: si esegue M1, e il criterio finale è il PIÙ severo tra i due
(P/2 vs 0,7 ns/op), con la soglia a tavolino P < 10% che precede entrambi.
Nota: Bak ritira la tariffa «corpi caldi» come obiezione ai micro (WP-44 vale
solo per il giudice aggregato) — nessuna sedia la oppone.

## 4. Priorità S-98.0

1. **M1 completa** (noop + census + ASM + P nel .out) — primo atto, zero codice.
2. **Decisione H-B1 dal numero**: P < 10% ⇒ a tavolino ⇒ dritti a H-B2;
   altrimenti sotto-passo preambolo col criterio composto di §2.
3. **H-B2 come asse**: decomposizione per-opcode degli 11 residui di arith
   (quale corpo porta gli ~8,5 ns), poi intervento sui corpi.
4. Igiene permanente: bande su str/re, deriva inter-build dichiarata, prop
   (13,1 flag-on, seconda peggiore) pretende almeno un numero nuovo.
