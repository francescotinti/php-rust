# s160-criterio-af1.md — leva L-AF1 «array_filter plumbing 0-alloc per-elemento» — scritto PRIMA di ogni edit

0. **Census-quota (gate pre-registrato in NEXT_SESSION p.4, ESEGUITO)**: dagli
   atti s149 (r1==r2 ESATTO) la tranche-2 nominata (array_filter · array_walk ·
   usort · array_reduce) ha quota ORM SOLO su **array_filter n=1.831.238**
   (b=107,2 MB); array_walk/usort/array_reduce ASSENTI dal census ORM ⇒ la
   tranche COLLASSA a L-AF1 per il suo stesso gate; walk/usort/reduce restano
   in aperture per NOME (quota WP mai censita, si dichiara).
1. Oggetto: `ho_array_filter` caso 1-array + callback closure ANONIMA
   `simple_call && n_params==1` + **mode==0** (default): ammissione UNA volta
   per chiamata, per-elemento chiamata SENZA `vec![v.clone()]` via
   `call_closure_one` RIUSATO (L-AM1, s159: intake = braccio WP-37 di
   bind_params a n=1) con `v.clone()` per il call e `v` per l'insert (stesso
   numero di clone del pieno: si rimuove SOLO l'args-Vec); `to_bool` sul
   risultato INVARIATO (common-mode). OGNI altro caso (no-cb, string/array
   callable, mode 1/2, non-simple, arità≠1, hint) resta sul cammino pieno
   INVARIATO per costruzione. NESSUN codice nuovo in mod.rs/calls.rs (riuso).
2. Giudice: `m-arrfilter.php` (NUOVO: 10.000 chiamate × 1.000 elementi = 10M
   invocazioni-elemento; parità AF-OK 5000000 bilaterale GIÀ verificata
   PRE-leva; tick 1,0 ns = soglia/4, REGOLE §3); N dichiarato dal sorgente.
3. A/B: braccio A = GEMELLO dal tree corrente (§7-bis), identità a CONTENUTO
   con regioni ammesse s158 (LC_UUID 16B · banner __DATE__ mimalloc ≤32B ·
   firma 2×32B; in s159 il gemello fu == pin AL BYTE anche a freddo, N=1;
   byte fuori regione ⇒ STOP); B = tree + edit. R=5 ABAB, floor3, mediane,
   rumore drop-1 (matematica s158 INVARIATA); soglia giudice = max(4, drop-1).
4. UB-alloc FALSIFICABILE = 1 alloc-sito/elemento × coeff TARATO s159 =
   **12,0 ± 2,5 ns/iter**; componenti non prezzate DICHIARATE per nome:
   dispatch call_callable (match variante + named-check + baseline-check),
   `c.clone()` Rc-bump per-elemento, costruzione `call_args` match sul mode.
5. **BANDA SMOKE VINCOLANTE** (ereditata s159, stessi denti NEL giudice):
   smoke R=2 con early-stop a segno opposto; D_smoke ∈ **[8; 22]** ⇒ si
   procede al R=5; FUORI banda ⇒ ARBITRATO DEDICATO PRIMA del R=5 (census
   conteggi sui due bracci, Δ alloc/elemento atteso 1 ESATTO); D_smoke <
   soglia ⇒ STOP.
6. Guardie R=5 a SOLO-REGRESSIONE, comparatore e soglie EREDITATI s159:
   **arrmap (L-AM1 non deve regredire)** + refl + missload + hostargs +
   backtrace24 + obj*6 + sei micro (16 guardie).
7. Semantica: output A==B su m-arrfilter pena rc=2; fixture bilaterale NUOVA
   `fx-af.php` (13 forme: closure-1 fast · no-cb truthy · USE_KEY · USE_BOTH ·
   string-callable · hint→pieno · static closure · eccezione nel callback ·
   array vuoto · capture · Closure::bind · ritorno non-bool · chiavi miste
   preservate) — PRE-leva GIÀ BYTE-ID (nessuna divergenza pre-esistente,
   baseline registrata); oracle==candidato BYTE-ID al gate promo.
8. disasm run_loop bl-count A vs B agli atti; salita denti loc PRE-DICHIARATA:
   host.rs +≈20 (SOLO host.rs: mod.rs/calls.rs INVARIATI per riuso); cap
   aggiornati SOLO in promozione, dichiarati.
9. Promozione SOLO se R=5 SOPRA SOGLIA e guardie 16/16: catena piena via
   pin-phpr.sh (batteria + corpus 1412 per NOME ×2 modi + fixture (fx-am v2
   20 forme + fx-af NEL set) + micro R=5 + ORM 16 nomi + hk 0E/0F); churn
   dichiarato.
10. Esiti a FILE: rc autoritativo `ab-out/af1.rc`; verdetto
    `s160-af1-verdetto.out`; identità gemello `s160-gemelloA-identita.out`.
    FINESTRA: build e A/B SOLO a coppia t10+ORM conclusa (rimisura.done
    presente) — mai edit/build coi run di misura in volo.
