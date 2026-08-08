# S-113 istruttoria (PRIMA del criterio, vincolo NEXT_SESSION §S-113)

**Candidata (g) census BinOp residui — ESAURITA sui giudici.** Census a
livello SORGENTE dei call-site di `binary_value_ab` in run.rs: tutte le forme
slot/pila calde (BinarySS, BinarySSDst, BinarySC, BinarySCDst, BinaryTC,
BinarySCSC, BinarySCSCDst ×4 dopo A2b, BinarySTDst dopo A2c, CmpJmpSS,
CmpJmpSC, BinaryTCPropSetPop) hanno GIÀ la guardia `binary_fast` inline.
Senza guardia restano solo **BinaryDst** (run.rs:1708) e **CmpJmpConst**
(run.rs:1609): NESSUNO dei due appare nei corpi caldi dei sei giudici né
nel corpo di `f` di calls (dump S-112, admission-out/). (g) può mordere solo
su WP reale (frequenza non censita): non è la leva di stasera.

**Candidata (f) prop — ISTRUITA fino alla forma.** Corpo caldo dal dump
(cand-on-prop.main): 8 op/iter = CmpJmpSC · **PropGetSlotRecv** ·
**BinaryTCPropSetPop** · Sweep · **PropGetSlot** · BinarySTDst · Sweep ·
IncDecSlotJmp. Dal pin-verdetto S-112: prop 107,67/30M ≈ 13,5 ns/op contro
~11,8 ns/op di arith post-A2 (47,20/4): TUTTE le op di prop hanno già corsia
veloce/IC — il residuo sopra il muro per-op è ciclo di vita, non corsia
assente. Falla nominata leggendo run.rs:
- `PropGetSlot` (run.rs:4162) e `PropGetSlotRecv` (run.rs:4169) chiamano
  `reg_load_slot` = **clone posseduto del ricevitore (bump+drop Rc) che
  sull'IC-hit muore a fine arm senza servire a nulla** (dcn: «l'handle mosso
  muore a fine arm»). L'hit legge solo via borrow.
- Prior art ESATTO: `ThisPropGet` (WP-34, run.rs:4181) fa l'hit «in
  prestito» senza clonare il ricevitore («the IC hit borrows the receiver
  in place — no This clone»), fallback condiviso per tutto il resto.
- Il lato set (prop_set_entry, BinaryTCPropSetPop) NON ha clone sprecato:
  l'handle arriva posseduto dalla pila e viene mosso. Non si tocca.

**Scelta: leva H-P1 «probe IC col ricevitore in prestito»** — nei 2 bracci
PropGetSlot/PropGetSlotRecv, probe inline nella forma di ThisPropGet
(hit: Option<Zval> calcolato sotto borrow dello slot, condizioni REPLICATE
VERBATIM da prop_get_entry: slot direttamente `Zval::Object` (Ref/Undef al
funnel), `ic.get(sk)`, `class_id+1==cid1`, `lazy.is_none()`, `get_slot`
non-Undef, `deref_clone` del valore, census P1 sotto feature); ogni miss →
sentiero attuale INVARIATO (`reg_load_slot` + `prop_get_entry`, warning
Undef compreso). Per PropGetSlotRecv il push del recv resta PRIMA del probe
(ordine di pila identico). Nessuna modifica al pass/emissione: admission a
dump INVARIATI AL BYTE. Beneficiario: prop (2 siti/iter, 60M hit/run).
Non-bersagli: arith/calls/str/arr/re non usano PropGetSlot* nei corpi caldi
(dump S-112) → guardie a SOLO-REGRESSIONE. Nessuna stima esterna nel
criterio (vincolo S-112): attesa qualitativa = 2 (clone+drop Rc + transito
owned) evitati per iterazione sul giudice peggiore.
