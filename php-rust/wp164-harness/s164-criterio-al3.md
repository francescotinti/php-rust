# Criterio S-164 p.4 — leva L-AL3 «FrameExt riciclato via pool sul fast path closure» — scritto PRIMA dell'edit

1. EDIT (studio s164-al3-studio.md, componenti DICHIARATE): (i) freelist
   `exts` (cap 8) in FramePool + take_ext/put_ext; (ii) `recycle_frame`:
   take dell'ext PRIMA di drop(frame), reset `*b = default()` DOPO (ordine
   di drop bit-identico: ext è l'ULTIMO campo, doc 2594-2603); (iii)
   `push_closure_frame_one`: aggancio dal pool prima di `closure_id=Some`;
   (iv) NEL medesimo edit, az.rev. S-163 #4: `unreachable!` sui bracci morti
   gemelli di `call_fn_one` + `call_method_one` (attesa: ZERO effetto perf,
   arbitri = disasm + batteria). Il generico push_closure_frame NON si tocca.
2. BERSAGLIO: m-missload (wp162-harness/m-missload.php, N=10M, closure
   arity-1 no-op = fast path AL2). Attesa D = +7±3 ns/iter (coeff per-sito
   autoload 1 alloc, TABELLA S-162; banda smoke [4;10]).
   SOPRA 10 ⇒ FUORI-UB SOPRA: arbitrato census DOVUTO prima del R=5.
3. CENSUS (unità dello strumento: hostcall_n = TUTTE le alloc sotto il tag):
   driver wp161-harness/m-missload-census.php (N=200000, read-only); attesa
   Δ hostcall_n = 200000 ESATTO su class_exists (1 Box/iter), altri nomi
   zero. **Esito che FERMA (pre-registrato, ANCHE in ns, az.rev. S-163 #3)**:
   (a) Δ ≠ 200000 ESATTO o altri nomi ≠ 0 ⇒ STOP, si torna al sorgente,
   NESSUNA taratura; (b) D_smoke < 4 ns/iter (= soglia) pur con census
   esatto ⇒ leva NON PAGANTE ⇒ STOP dichiarato senza promo (il riciclo non
   ripaga il branch); (c) D_smoke in [4;10] e census esatto ⇒ R=5.
4. MISURA: smoke R=2 early-stop a segno opposto; R=5 ABAB vs GEMELLO A =
   pin s163 fea4a2d0 ricostruito dal tree s163 nel TARGET CANONICO (ricetta
   S-163, identità al byte pena rc=9); user CPU al netto pavimenti
   PER-binario; giudice N emesso dal sorgente del driver.
5. GUARDIE (solo-regressione): 6 micro categorie + m-arrload (non-bersaglio,
   il path [obj,metodo] non usa FrameExt) + disasm bl run_loop prima/dopo
   (Δ atteso: 0 su run_loop; recycle_frame/push_* fuori loop — si dichiara
   il conteggio) + batteria (il branch recycle tocca TUTTI i frame) + corpus
   1412×2 per NOME + fixture bilaterali. Dente loc mod.rs PRE-dichiarato
   all'edit (cap attuale 25909/7726).
6. Esiti a FILE in wp164-harness/ab-out/; copia-gate degli script derivati
   verificato su DIFF INTERO + grep dei path di harness (lezione incidente
   S-164 #1); rc SOLO dai .done degli script; lock S-164 solo VERIFICATO.
   L'edit parte SOLO a coppia CONCLUSA (rust-analyzer = rumore in finestra).
