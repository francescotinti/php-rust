# Verbale SEDIA 4 — Hejlsberg (Concilio WP-94) — catene di evidenza, identità toolchain

**VERDETTO: RESPINTO IN PARTE.** A-AH61/A-AH62 reggono sul percorso onesto; **A-AH63 e A-AH64 come attuati NON chiudono KS-AH-93-2/93-3**: sei ledger forgiati passano il checker (rc=0 eseguito), due nutrono direttamente il gate.

## Q1 — hoist A-AH61 (battery-equivalence.sh:295-353)
Variabili: OUT/BREV (r.88), GITPREFIX (r.102) definite prima dell'hoist su ENTRAMBI i percorsi; DSHA (r.180) nasce solo se `.done` esiste — r.320 è guardata (`${DSHA:-__nodsha__}`), r.322 usa `$DSHA` nudo sotto `set -u`: con `.done` assente bash muore "unbound variable" (fail-CLOSED, diagnosi rotta — emendare). Semantica giusta: per 89pre/90pre le righe PASS committate portano sha256 (verificato nel ledger a HEAD: 05726aa/30f5a87/4c99520) ⇒ nessuna equivalenza storica giudicata SBAGLIATA. **MA lo scope dei denti è deciso dal BASENAME di OUT, input del chiamante** (eseguito: `battery-88pre` ⇒ A-AH50/54 EXEMPT + v2 EXEMPT; `b91` ⇒ v2 EXEMPT): il regex PASS è `BATTERY-[0-9]+PRE` generico e il grep dello stamp (r.202) non porta `battery=` — un OUT 91pre rinominato battery-88pre.out consuma saltando TUTTA la disciplina attempts. Latente: A-PP-68 retro-giudica eventuali ABORT 91pre scritti pre-emendamento (oggi zero righe — dichiararlo).

## Q2 — check-campaign-v2.sh: fail-open ESEGUITI (rc=0 su tutti)
- **f1**: riga di tipo IGNOTO (`note=forge esito=PASS generation=g9`) — nessuna R0 di chiusura sui tipi di riga: "una riga fuori grammatica ⇒ VOID" è inapplicabile per costruzione, l'ignoto è invisibile al while/case.
- **f2**: `phase=authorize` senza `gG=<atto>` (A-PP-65 lo esige) — mai controllato.
- **f3**: verdict con `campaign_sha=<start>deadbeef` — grep sottostringa non ancorato: match di PREFISSO.
- **f4**: `judge_sha=<pinned>ffff` mai autorizzato passa R4 — la sed cattura i primi 16 hex. **La stessa classe che A-AH61 ha appena chiuso sul sha256 ({64} non ancorato): il checker ripete il peccato che l'emendamento gemello corregge.**
- **f5**: `reason=requalify:VCKPT:nodelta` senza `<old>-><new>` — R3 non esige `->`, la storia g(n)→g(n+1) NON si rigenera.
- Divergenza dichiaranda: design91-ledger dice reason requalify su «verdict/supersede»; il checker (e la fixture good) lo esige solo su supersede.

## Q3 — matcher A-AH64: FORGIABILE, eseguito
Il matcher è congiunzione di SOTTOSTRINGHE sulla stessa riga, non campi: **f6** `esito=FAIL reason=prev-esito=PASS-retired generation=g2` passa la v2 (rc=0) E il matcher replicato lo ammette (`admitted-as-PASS=1`); f1 idem per g9 via entrambe le alternative (generation= e verdict_file=). Aggravante: il campaign ledger NON ha il dente append-only A-SK57 (copre solo battery-stamps/attempts) — un append post-campagna di una riga ignota con le due sottostringhe legalizza una generazione max-FAIL.

## Q4 — A-AH63 senza dente end-to-end: NON accettabile
Fail-open dimostrato nel wiring: `bash` su chk.sh VUOTO ⇒ rc=0 ⇒ ledger «conforme» da giudice vuoto (eseguito). `qx(git show > chk.sh)` non verifica né rc né non-vuotezza. Serve **bite-test alla prima campagna m91** (canary malformato committato ⇒ gate FAIL atteso) + hardening: sha(chk.sh)==blob HEAD e `--selftest` rc=0 dalla COPIA estratta prima di giudicare.

## Emendamenti
- **A-AH65**: checker v2 — R0 tipi di riga CHIUSI; ancoraggi di campo (campaign_sha/judge_sha `( |$)`, 16hex esatti); `gG=` obbligatorio su authorize; requalify con `->`; esito parsato come CAMPO unico.
- **A-AH66**: matcher A-AH64 su campi parsati di righe `phase=verdict` sole; dente A-SK57 esteso ai campaign ledger.
- **A-AH67**: wiring A-AH63 fail-closed (sha estrazione + selftest della copia) + bite-test end-to-end obbligatorio alla prima campagna.
- **A-AH68**: BATTERY_NAME dal contenuto ancorato (riga PASS terminale / campo battery= dello stamp), mai dal basename; grep stamp r.202 con `battery=`; r.322 guardata.

## Kill-switch
- **KS-AH-94-1**: checker di grammatica senza regola di chiusura sui tipi di riga = fail-open per costruzione.
- **KS-AH-94-2**: lo scope di un dente non dipende MAI da un nome scelto dal chiamante.
- **KS-AH-94-3**: giudice estratto valido solo con estrazione PROVATA (sha==blob, selftest della copia); giudice vuoto che benedice = consumazione VOID.

**Refutazioni capitali: SÌ** (punto 2 WP-93, metà A-AH63/A-AH64: denti dichiarati armati che non mordono su sei forge eseguite). Q1: NO capitale (emendabile A-AH68). Evidenza: forge f1..f6 + matcher replica + empty-checker + tabella scope, eseguiti 2026-08-03 su HEAD.
