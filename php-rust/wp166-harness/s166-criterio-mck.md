# S-166 criterio L-MCk «MC1-k∞: cade il cap argc≤2» — PRE-registrato PRIMA dell'edit
1. LEVA: nell'arm Op::MethodCall cade il SOLO predicato `n <= 2` (il gate vero
   è già DENTRO methodcall_fast: simple_call && n_params==argc, IC-hit, recv
   Object; ArgPlace/decay/RET_DEREF INVARIATI — semantica identica per
   costruzione, stessa catena di s165-criterio-mc1.md p.1). unreachable!×2 NON
   si rimontano (verdetto braccio C S-165: ~5 ns su arrload).
2. GIUDICE: m-mc3.php NUOVO (gemello di m-mc2 a k=3: $o->f($s,1,0) — oggi
   FUORI cap ⇒ funnel sul pin, fast su B); N=20.000.000 dal sorgente;
   ns/iter=(med raw−floor)/N; floor empty.php per-binario.
3. SEGNO +B · SOGLIA max(4, rumore drop-1) col VIETO rumore>4 (rc=8,
   az.rev. S-164 #5) · SMOKE R=3 (AB/BA/AB) · BANDA VINCOLANTE [4;30]
   (stesso bundle salto-funnel di mc1; alloc ≈0 lezione AL3); fuori banda ⇒
   arbitrato census (vecargs Δ=N su m-mc3).
4. GUARDIA NUOVA mc2 (k=2, fast su ENTRAMBI i bracci): attesa D≈0, soglia
   max(4, rumore) — presidia che il cap-removal non tocchi il cammino k≤2.
   Guardie ereditate con banda-layout FONDATA (missload 8,0 · arrfilter 6,0)
   + obj*/sei/host-call, comparatore mc1e. Guardia quantizzata che morde ⇒
   ri-risoluzione a tick≤soglia/4 (REGOLE §3), come s165-ririsolvi.
5. GATE EDIT: disasm bl run_loop A/B Δ DICHIARATO (atteso ≈−1/0: cade un
   test); dente run.rs: variazione dichiarata nello stesso commit; fixture
   fx-mc + fx-mc2 A==B pre-smoke (la semantica k≤2 non deve muoversi, la k≥3
   diventa fast: fx da estendere con un caso k=3 by-ref/hint ESCLUSO).
6. VERDETTO: D≥soglia in banda ⇒ R=5 (DSM dal smoke) ⇒ catena §6 di
   promozione; D<soglia con rumore≤4 ⇒ non pagante, revert al byte.
