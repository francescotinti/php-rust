# DOSSIER — Concilio S-167 «rotta strutturale per il nucleo» (convocato dall'utente, 2026-09-01)

## LA DOMANDA (una sola)
GO/NO-GO a una CAMPAGNA STRUTTURALE su dispatch / rappresentazione Zval /
pila-frame per chiudere arith/prop/calls (oggi 4,8-5,5×) verso la tappa ≤3×
e l'obiettivo 1×. Se GO: la PRIMA FETTA misurabile (giudice, gate,
kill-switch pre-registrati). Se NO-GO: la rotta che tiene l'obiettivo 1×, o
la ridefinizione ONESTA dell'obiettivo.

## I NUMERI (pin s166, R=5, ns/iter phpr vs oracle)
arith 46,8 vs 8,6 (5,4×) · prop 77 (5,5×) · calls-fn 106 (4,8×) ·
method-call: mc2 155, mc3 181 (post L-MC1d/MCk: −8,5%/−10,6%) · str 158
(4,2×) · arr 150 (3,2×) · re 457 (2,5×). Reali: WP 1,746-1,749 (da 1,87 a
giugno) · media 2,43 · ORM 7,0 · dbal 7,3. Corpus 1412×2 verde, batteria 1748.

## DECOMPOSIZIONI FIRMATE (dai modelli S-129/131/136, chiusure 86-96%)
- Statement prop ~300-340 ns QUASI INVARIANTE per forma (oracle 23-37):
  E−E2 166,9 = prop_step interno 130,7 (guardie 49,4 · defer 37,0 · key+op
  34,3 · borrow 1,5 · altro 8,5) + **dispatch 36,3**; dopo le leve IC
  (E1-KO, LO1, IC-NP, AP1, FD1, S-131..136) i residui NOMINATI: dispatch
  36,3 · cammini non cacheabili per costruzione.
- FieldAssign arm 118,2 = walk_driver 37,2 · leaf 18,9 · plumbing 17,6 ·
  prop_step_altro 14,4 · guardia 11,3 · resolve 6,7 · dispatch 7,0 · pop 4,5.
- arith: NESSUNA decomposizione dedicata a registro (apertura storica);
  il driver è `$s += $i*3 − ($i>>2)` — puro loop+op: il costo è
  dispatch+operandi+Zval, non builtin.

## LE CADUTE ISTRUTTIVE (tutte a verdetto, con meccanismo nominato)
- **A3c NO-GO** (S-152, RAFFORZATO S-153): fetta Zval sui canali Object —
  benefici prezzati S2/S3 SOTTO soglia (banda netta 1,1-1,3 s vs soglia
  1,53/4,58); «stile veto NaN-boxing»; restano A3a/A3b micro-judged.
- **L-TD1** (S-153): borrow-unify, 4 borrow/iter rimossi CERTI ⇒ D=−3,3:
  **prezzo borrow in-contesto ≤~1 ns** (mock hot-hot 4,3 falsificato).
- **L-AL3** (S-164): pool di Box sul fast path, census Δ esatto ⇒ D=+0,0:
  **un'alloc mimalloc su fast path ≈0 ns visibili**.
- **H-C2** (S-104): caduta icache-bound (inliner flip bl 1101→0).
- **MC1 inline** (S-165): +45 bl in run_loop ⇒ ±5-8 ns su host-call NON
  toccate (banda-layout); cura = outline #[inline(never)].

## LE VITTORIE ISTRUTTIVE (il pattern che paga)
OGNI leva promossa ha SEMPLIFICATO IL DISPATCH (mai solo spostato alloc):
H-D/HD2 (stack→slot diretti), AL2/AM1/AM2/AU1 (host-call arity-k senza
Vec), MC1d/MCk (salto dell'imbuto method_call), E1-KO/LO1/IC-NP/AP1/FD1
(IC e fast-path su prop/dim). Resa tipica 5-40 ns/sito; il filone
method/prop ha eroso 40-50% dei SUOI micro; arith è rimasto FERMO (46,8;
creep refutato a 0,5 di risoluzione).

## VINCOLI E APPARATO
Safe-only Rust (VmGate ZST) · corpus congelato per NOME ×2 modi ·
byte-parity su ciò che sta in stringhe PHP · REGOLE §3 (criterio prima,
gemello, banda-layout fondata, ri-risoluzione quantizzate) · veti
trasversali: BOLT/NaN-boxing/PGO (il concilio PUÒ levarli o confermarli) ·
esiste PHPR_REG_LOWER (ARCO REGISTRI, flag-on default da S-100).

## ROTTE CANDIDATE NOTE (da refutare/ordinare; NON esaustive)
R1 **dispatch**: costo per-op del run_loop — predecode/fusione di op,
superistruzioni per pattern caldi (cmp+jmp, load+op+store), dispatch a
tabella di funzioni, riduzione della taglia del match; quesito aperto:
quanto del «dispatch 36,3» sopravvive col reg-lowering attivo?
R2 **rappresentazione Zval**: taglia dell'enum, copy/clone semantics sui
cammini caldi, small-string inline, NaN-boxing (oggi a veto).
R3 **frame/pila**: arena per-richiesta, C-lite (piano storico), riduzione
del traffico push/pop operandi (ARCO REGISTRI è già parziale qui).
R4 **accettare il soffitto core**: tappa ridefinita sui workload REALI
(WP≤1,5, ORM≤5) con leve per NOME + builtin; 1× dichiarato non-obiettivo.

## §FONDAMENTALI (obbligo di apertura sintesi)
Sessioni-senza-misura: 0 (S-166 ha misurato 2 coppie + micro + 2 A/B R=5).
Oggetto avanzato: 2 promozioni in 2 giorni (mc2 −8,5%, mc3 −10,6%), WP
1,761→1,746. Rischio d'oggetto più trascurato: arith/prop FERMI da
~15 sessioni — è ESATTAMENTE la domanda di questo concilio.
