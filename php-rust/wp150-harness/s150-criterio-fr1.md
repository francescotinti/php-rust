# Criterio S-150 p.5 — ISTRUTTORIA FR1 (regressione m-dimrmw10 +3,00 s142→s145) — scritto PRIMA di ogni run/build

1. Obbligo NEXT: mutante a PARITÀ DI LAYOUT + disasm bl-count PRIMA di ogni
   revert. Nessun revert in questa sessione: l'istruttoria NOMINA il
   meccanismo. Grado ISTRUTTORIA: nessuna cifra in PERF_MAP.
2. FATTO STATICO 1 (dump): `PHPR_DUMP_OPS=1` sul pin s150 con
   `wp146-harness/m-dimrmw10.php` e con `wp145-harness/m-dimread.php`:
   il fused `PropDimGetConst` è ATTESO ASSENTE dal giudice RMW e PRESENTE
   nel giudice dimread (il peephole è sul cammino READ). Se invece è
   PRESENTE nel RMW ⇒ ipotesi runtime VIVA, l'A/B p.4 la giudica.
3. FATTO STATICO 2 (disasm, metodo S-109): run_loop size+bl su stash s142,
   stash s145, pin s150 — il delta strutturale della variante (S-145:
   71694/5988 → 72489/6014) si RIVERIFICA e resta agli atti.
4. Mutante M1 monobinario (classe S-138 kill-switch): env `PHPR_FR1_OFF=1`
   letto SOLO nel matcher del peephole alla LOWERING (mai nel run_loop):
   spegne l'EMISSIONE del fused; run_loop/dispatch IDENTICI per costruzione
   (disasm M1 vs pin: size+bl ATTESI uguali, registrati). A/B monobinario
   ON vs OFF, R=5 ABAB, giudici m-dimrmw10 (ns/iter=(user−pav)/3e7) e
   m-dimread (ns/iter=(user−pav)/3e6); soglia = max(1,0; drop-1; 0,67)
   [dimrmw10] e max(4; drop-1) [dimread]; pavimenti med3 per-binario;
   parità stdout su ogni run; ancoraggio M1-ON vs pin s150 smoke R=2
   (scarto kill-switch DICHIARATO).
5. DENTE (forgia mai silenziosa): su m-dimread l'OFF DEVE perdere il
   guadagno FR1 (attesa ≈ +16,7 ns/iter vs ON, segni ≥4/5) — pena: mutante
   NON provato, istruttoria VOID (il «0» sul RMW non direbbe nulla).
6. Esiti PRE-REGISTRATI su m-dimrmw10 (D = OFF − ON, col dente p.5 verde):
   (a) D ≤ −soglia (OFF recupera ~3): meccanismo = RUNTIME del fused sul
       cammino RMW (coerente solo se p.2 lo trova nel RMW) ⇒ cura mirata a
       catalogo S-151, nessun revert cieco;
   (b) |D| < soglia: il +3 NON vive nell'emissione ⇒ prezzo STRUTTURALE
       della variante (layout/dispatch, delta p.3) ⇒ il revert non curerebbe
       il meccanismo: voce CHIUSA come prezzo dichiarato (keep-partial-wins:
       dimread −16,7 ≫ +3);
   (c) D ≥ +soglia: segno opposto ⇒ da re-istruire (nessuna conclusione).
7. Ordine: DOPO coppia WP e coppia ORM (sequenziale, lock di sessione
   verificato); build M1 = edit dichiarato SOLO nel matcher (branch di
   sessione? NO: env-check committato, feature-neutro, rimane nel codice
   come interruttore d'istruttoria documentato o si rimuove a fine
   istruttoria con revert al byte dichiarato).
