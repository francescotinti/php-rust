# s138-criterio-rmw — leva FD1-ext RMW su FieldAssignOp/FieldIncDec (PRE-REGISTRATO, prima dei numeri)

1. Leva: fast path a IC-hit per `$o->prop[k] op= v` e `$o->prop[k]++/--`
   (lowering verificato col dump: `FieldAssignOp{[Prop,Index]}`,
   `FieldIncDec{[Prop,Index]}`). Il pieno paga DUE walk (read `field_value` +
   write `field_set_op`) + preludio byref/lazy; il fast: admission FD1
   (IC NP-hit, base plain, child Array) + peek entry + op + `field_write_walk`
   RIUSATO sul child (leaf identico per costruzione, disciplina S-136).
   Campo `ic: PropIc` AGGIUNTO alle due varianti (siti di costruzione: 2 in
   compile/assign.rs; altri match usano `..`). Fill: riusa `field_assign_fill`
   sul ramo piano a esito Ok, gated da `field_prelude_skip` (stessi fatti F4).
2. PERIMETRO fast (tutto il resto MISS → pieno INVARIATO): steps==[Prop,Index]
   · 1 chiave ∈ {Long, Str} (coercizione silente ≡ diag per questi tipi) ·
   entry PRESENTE (assente → pieno, che emette il suo warning) · old (deref)
   ∈ {Long, Double} · AssignOp: op ∈ {Add, Sub, Mul} e rhs ∈ {Long, Double}
   (op silenti e puri: niente codice utente, ri-eseguibili) · IncDec: old ∈
   {Long, Double} (string-increment deprecato resta al pieno).
3. Giudici bersaglio: m-dimrmw (D_rmw) e m-diminc (D_inc), A/B interleaved
   ABAB candidato-vs-pin-s136, R=5, user ns/iter (denominatore 3e6 dal
   sorgente); soglia = max(4, drop-1 simmetrico). SMOKE R=2 early-stop:
   giudica SOLO la direzione dei bersagli — le guardie NON si giudicano allo
   smoke ma SOLO a R=5 col drop-1 vero (az.rev. S-136 #1, lezione `re` S-136).
4. UB parte-modellata = **63,3** (prezzo CHIUSO oggi dall'A/B s135↔s136 sul
   giudice del modello: sostituzione del SOLO write-walk). Componenti NON
   prezzate DICHIARATE per NOME (S-134): read-walk `field_value` RIMOSSO (+) ·
   preludio RMW byref/indirect/lazy RIMOSSO (+) · peek+op fuori-walk AGGIUNTO
   (−). Attesa: D ≥ 63,3 − 13,3; eccedenza sopra 63,3+13,3 ATTRIBUITA per
   NOME alle componenti dichiarate; se D > 63,3+100 → fuori modello, sonda
   dovuta prima di promuovere.
5. Guardie SOLO-REGRESSIONE a R=5 (banda drop-1 propria per giudice): arith ·
   prop · calls · str · arr · re · objdatains (bersaglio FD1 esistente: non
   deve regredire). Una guardia morsa allo smoke NON ferma (p.3); morsa a R=5
   oltre banda → leva FERMA, dichiarare.
6. Fedeltà PRIMA del tempo: fixtures-rmw + fixtures-fd1 con candidato==pin
   BYTE-IDENTICI (le divergenze pieno-vs-oracle trovate oggi sono
   PRE-ESISTENTI, a catalogo per NOME, fuori leva) · parità stdout dei due
   bench vs oracle su ogni run · batteria `cargo test --release` rc=0 dal
   comando prima dell'A/B.
7. Promozione SOLO via `scripts/pin-phpr.sh` (collaudo-nell'atto): batteria ·
   corpus 1414 per NOME ×2 · fixture-chain · micro R=5. Working tree
   DICHIARATO pulito al commit della leva (az.rev. S-136 #5). Disasm run_loop
   prima/dopo (bl-count) a verbale (protocollo S-104).
8. Verdetti: `s138-ab-rmw-verdetto.out` (A/B) · promozione nel registro pin.
   rc autoritativi da file. Lock misura + quiescenza + pgrep rust-analyzer
   per ogni finestra.
