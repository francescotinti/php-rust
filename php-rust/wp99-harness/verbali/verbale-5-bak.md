# Verbale sedia 5 — Bak (V8/HotSpot: dispatch, code-cache, path caldi, alloc-rate)

**Oggetto**: report S-97.1 (H-A1 caduta) + programma H-B1.
**VERDETTO: CON EMENDAMENTI.**

## 1. La caduta di H-A1 NON è spiegata dai corpi caldi — è aritmetica dei costi (refutazione capitale n.1)

Conto dai `.out`: a costo UNIFORME, 19→11 op/iter darebbe 11/19 × 7,83 = 4,53 s
= **−42,1%**, appena sopra la soglia. Il −40% richiesto presupponeva quindi che
gli 8 opcode rimossi (LoadVar/PushConst/Dup/Pop — i più economici) costassero
quanto l'opcode MEDIO. Ma la non-uniformità era già MISURATA in ha2-sweep
(Sweep noop ≈ 1/5 del medio), stessa sessione che ha scritto il criterio.
**Il criterio era irraggiungibile per la fisica già nota quando fu scritto: la
caduta è un verdetto sul criterio, non sul meccanismo.** Il marginale reale
dei rimossi è (7,83−5,43)/(8×50M) = **6,0 ns per opcode rimosso** — SOPRA la
stima cheap-op (~2-3 ns): H-A1 ha reso più del suo modello, perché i corpi
sostitutivi (fast-path a borrow) sono più economici dei pop/push che
rimpiazzano. La regola n.3 fu applicata bene; la taratura no.

Sul modello WP-44: **non si applica al micro**. Su `arith` il working set
eseguito flag-on è ~10 corpi contro ~11 flag-off — non cresce; tutto sta in
L1I e il target predictor impara la sequenza fissa di 11 opcode. Il +1% di
WP-44 era pressione I-cache/BTB dell'AGGREGATO megamorfo. Quindi una forma
che SOSTITUISCE invece di affiancare **non renderebbe di più sul micro**
(stesso working set eseguito); il suo claim appartiene al giudice aggregato,
e cambierebbe l'emissione di parità — prematuro.

## 2. I 7 arm morti flag-off: WP-33 NON si applica

Il +2,9% di WP-33 era un `if bool` VALUTATO a ogni dispatch. Un arm di match
mai dispatchato è codice freddo dietro una entry di jump-table: zero
istruzioni per tick; costa solo se sposta il layout dei handler caldi.
Evidenza: 7,88 (ha2) → 7,83 (ha1-off) = **−0,6%, segno OPPOSTO a un costo**,
dentro lo spread R=3. Bound onesto: costo ≤ ~1% (binari di HEAD diversi:
è un bound, non una prova di zero). Non rilitigarlo.

## 3. Il tetto di H-B1 è GIÀ MISURATO (refutazione capitale n.2)

ha2-sweep è il controfattuale: un dispatch noop completo (preambolo: guardia
`frames.len`, 3-4 indicizzazioni bound-checked, fetch `ops[ip]`, store
`ip+1`, match, handler-vuoto) costa al margine (7,95−7,88)/50M = **1,4 ns**.
Dunque il PREAMBOLO vale ≤ 1,4 degli 8,24 ns/op: **H-B1 in scope
«preambolo» ha un tetto di −17% sul costo per opcode** — non può «scendere
in modo netto» oltre, e non tocca gli ~8,5 ns che vivono DENTRO i corpi
(catene `self → frames → frame → slots/stack` per operando, push su
`Vec<Zval>`, lavoro Zval — è lì il fattore 8 contro il frame-in-registro di
Ignition/Zend). Un criterio che pretenda più del tetto ripete l'errore del
−40%. Lo scope «frame in registro ANCHE dentro i corpi» non è limitato dal
1,4 ns, ma in Rust safe collide con `&mut self` nei handler: costo
architetturale da dichiarare prima.

## Emendamenti

- **A-BA-99-1**: il criterio di H-B1 si DERIVA dal controfattuale misurato,
  non da una cifra tonda. Scope preambolo: cade se il risparmio < 0,7 ns/op
  (metà del tetto), cioè `arith` flag-off netto > 7,2 s sulla coppia R=3
  stessa-sera (8,24 → ≤ 7,55 ns/op), con spread < metà del delta.
- **A-BA-99-2**: prima di scrivere, dichiarare lo SCOPE (solo-preambolo vs
  frame-esteso) e la predizione-misurata: conteggio statico dei re-borrow
  `self.frames[top]` eliminati per opcode del residuo arith.
- **A-BA-99-3**: ritirare la tariffa «corpi caldi» come obiezione ai micro;
  vale solo per il giudice aggregato (WP-44 resta valido lì).
- **A-BA-99-4**: H-B2 va preparata sulla stessa tavola di decomposizione:
  gli 8,5 ns residui sono corpo-interni, non preambolo.

## Kill-switch

- **KS-BA-99-1**: se il criterio scritto in apertura S-98.0 eccede il tetto
  dello scope dichiarato (1,4 ns/op per il preambolo), il criterio è INVALIDO
  e va riscritto prima di toccare codice.
- **KS-BA-99-2**: ogni misura futura flag-off su `arith` che regredisca >2%
  vs 7,83 a ricetta identica sospende le ipotesi nuove finché il layout
  (arm morti/inlining) non è escluso con una coppia stessa-sera.

## Refutazioni capitali

1. **Il criterio −40% era irraggiungibile con dati già posseduti** (uniforme
   = −42,1%; non-uniformità nota da ha2): H-A1 è caduta su una taratura, non
   sul meccanismo.
2. **H-B1-preambolo ha un tetto misurato di 1,4 ns/op (−17%)**: presentarla
   come l'asse che chiude il fattore ~8 per opcode è refutato dai numeri del
   progetto stesso; il fattore vive nei corpi (pointer-chasing + traffico di
   pila Zval), cioè H-B1-esteso/H-B2.
