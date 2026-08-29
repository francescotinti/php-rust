# s162-criterio-am2.md — leva «L-AM2: array_map string-callable UTENTE k=1 senza args-Vec» (NEXT_SESSION p.2; PRE-REGISTRATO prima di ogni edit/misura)

0. **Census-quota (dichiarata, NON censita)**: census s149 array_map = 7,73M
   passaggi ORM pre-AM1; la frazione string-callable NON è censita e NON si
   stima (veto componenti prezzate) — leva MICRO-JUDGED come L-AL2; quota
   suite dichiarata sotto-risoluzione. Il VALORE oltre il micro: quarto sito
   della tabella PER-SITO con predizione falsificabile (p.4).
1. Oggetto: nel ramo 1-array di `ho_array_map` (host.rs), quando il callback è
   STRINGA che risolve a funzione UTENTE `simple_call && n_params==1`
   (risoluzione = specchio di invoke_named: strip `\`, find_fn_ci, poi
   linked_functions — «::» non risolve mai lì per costruzione), risoluzione
   UNA volta per chiamata host e dispatch per-elemento via `call_fn_one`
   NUOVO (specchio di call_closure_one) + `push_fn_frame_one` NUOVO (specchio
   del braccio user-fn di invoke_named MENO il Vec: argc=1,
   slots[0]=decay_arg). Bundle rimosso per-elemento: cb.clone + scan «::» +
   to_vec del nome + find_fn_ci + args-Vec + bind_params-via-Vec. OGNI altra
   forma (builtin, «Class::method», array-callable, closure non ammessa,
   multi-array) resta sul cammino INVARIATO per costruzione. Hoist
   IDEMPOTENTE dichiarato: un nome utente che GIÀ risolve non cambia esito
   (ridefinizione = fatal; linked aggiunge solo nomi nuovi). Edit: host.rs
   +~18 (dente 7708 → salita PRE-dichiarata), mod.rs +~16 (dente 25831 →
   salita PRE-dichiarata), calls.rs +~22 (1438, cap generico 2000, fuori
   allowlist). Bozza agli atti: s162-am2-edit-draft.md.
2. Giudice: `m-strmap.php` (wp162-harness): 200 chiamate × 50.000 elementi =
   N=10.000.000 elementi (N dal sorgente; tick 1,0 ns = soglia/4), marcatore
   `SM-OK 10000200` bilaterale + diff output. Parità estesa: `fx-sm.php`
   oracle==pin BYTE-ID (forme ammesse E non ammesse) — gate di promozione.
3. A/B: braccio A = GEMELLO dal tree s161 (§7-bis), atteso ==pin ec0a636a AL
   BYTE a freddo; byte diverso ⇒ arbitrato a contenuto, si dichiara. B = tree
   + edit p.1. R=5 ABAB, floor3, mediane, rumore drop-1 (matematica s158
   INVARIATA); soglia giudice = max(4, drop-1). Stash SOLO via pin-phpr.sh
   --braccio. Disasm gate run_loop (bl-count Δ=0). Build SOLO a catena p.1
   conclusa (pair t12 + ORM done).
4. **UB dal modello PER-SITO (bivio MECCANICO pre-registrato)**: sito più
   SIMILE = autoload (stessa trasformazione call_callable→call_*_one, k=1,
   senza mode-match/doppio-clone) = 7,0±3,0 (rimisura s162 stash fermi) =
   PAVIMENTO dichiarato (il bundle L-AM2 CONTIENE quel bundle e vi AGGIUNGE
   to_vec+scan+find_fn_ci per-elemento); tetto di riferimento = arrfilter
   17,0+2,0 (bundle più ricco della tabella). Attesa: D ∈ [4,0; 19,0].
   Verdetto: D > 19,0+rumore ⇒ FUORI-UB SOPRA (reperto, sonda dovuta);
   D < 4,0 ⇒ leva NON promossa (sotto soglia); D ∈ [4,0; 7,0−3,0−rumore) ⇒
   impossibile per costruzione (pavimento>soglia solo se rumore<0) — si
   dichiara com'è; altrimenti DENTRO: quarto coeff a TABELLA.
5. **BANDA SMOKE VINCOLANTE + EMENDA rev. S-161 #4**: smoke R=2 con
   early-stop a segno opposto; banda smoke = attesa p.4 [4,0; 19,0] su
   m-strmap; fuori banda ⇒ STOP e arbitrato census PRIMA del R=5 (driver
   m-strmap-census 200×1000, Δ hostcall_n atteso = 200.000 ESATTO
   sull'arbitro array_map, altri nomi zero); **una guardia non-bersaglio che
   morde a R=2 pretende ARBITRATO DICHIARATO come la banda** (rev. #4), non
   un'assoluzione ex post dal R=5.
6. Conferma post-pin (rev. S-161 #5): rumore > attesa/2 ⇒ verdetto «solo
   segno» DICHIARATO, mai cifra. Guardie: micro 6 categorie R=5
   solo-regressione + fixture chain + fx-am/fx-af/fx-sm + corpus 1412×2 per
   NOME + batteria coi denti pre-dichiarati.
7. Esiti a FILE: verdetti `s162-am2-*.out` in wp162-harness; rc SOLO dai
   `.done`/`.rc` degli script; lock della SESSIONE verificato.
