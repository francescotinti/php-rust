# s153-criterio-td1 — leva L-TD1 «teardown/sweep borrow unico» (A3a fetta 1; PRE-REGISTRATO prima di ogni edit/misura)

1. **Edit** (mod.rs SOLO, fuori run_loop; store INTATTO, A3c chiusa; ordine effetti INVARIATO):
   (a) `gc_note_frame` ramo `$this` Object → id-probe E nota sotto UN borrow (2→1);
   (b) `gc_sweep_impl` sequenza free → unbuffer+class_id+lazy sotto UN borrow, id passato a `gc_release_cascade` (5→2 contati).
2. **Giudice**: m-objdropdef (`wp127-harness/micro-orm/objdropdef.php`), N=3.000.000 EMESSO dal sorgente. Segno atteso D=A−B **POSITIVO**.
3. **Soglia**: max(4 ns/iter; rumore drop-1 del run). Nessuna banda spread-batch fondata per objdropdef: DICHIARATO.
4. **R**: smoke R=2 con early-stop a segno opposto → R=5 coppie ABAB alternate; user CPU `/usr/bin/time -p`, floor med3 per binario su empty.php.
5. **UB falsificabile**: 4 borrow/iter risparmiati (1 ctor-teardown + 3 free-seq) × prezzo mock c2_borrow 4,41 ns (verdetto s152-sonda) = **17,6 ns/iter**; D > 17,6+rumore ⇒ fuori banda a verbale, sonda dovuta.
6. **Companion FIRMATI** (segno atteso +, non gate): objchurn, objdatains. **Guardie SOLO-REGRESSIONE**: objalloc (banda 13,3) · objallocni (10,0) · objmap (3,3) [spread s135-submicro, da s136-ab.sh committato] + arith/prop/calls/str/arr/re (SL storiche s123/s125).
7. **Parità**: output A==B su OGNI categoria pena STOP leva. A = stash `phpr-s150` (cbbe71735effb165, verificato); B = build canonica dalla ricetta, hash dichiarato al run.
8. **Igiene**: lock `/private/tmp/phpr-measure.lock` mio (creato 09:59); quiescenza rc=0 pena STOP; attesi smoke BLIND in `s153-smoke-atteso.md` verificati da SECONDO attore prima del run di record; rc autoritativo da file `.rc`, mai da pipe.
