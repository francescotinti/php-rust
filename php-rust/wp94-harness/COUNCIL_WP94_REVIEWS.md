# COUNCIL_WP94_REVIEWS.md — Concilio a 9 sedie su S-92.0 (7 punti WP-93 eseguiti; gate cifre v3, ledger, sigilli v10, testimone a coppia, A-DL-59, checker LSP)

Protocollo A DUE FASI (decisione utente 2026-08-02): fase 1 = 9 verbali
INDIPENDENTI (nessuna sedia vede le altre); fase 2 = 3 team tematici con
relatore (team-cifre Klabnik+Hejlsberg+Gregg · team-misura
Matsakis+Pedersen+Bak · team-engine Stogov+Leijen+Hoare). I verbali
individuali sono la fonte VINCOLANTE; le note di team alimentano la
sintesi.

Indice: VERBALI INTEGRALI (fase 1) → NOTE DI TEAM (fase 2) → SINTESI DI
CONVERGENZA (ordine S-93.0 + kill-switch).

## VERBALI INTEGRALI (fase 1)

# Verbale sedia 1 — Hoare (Concilio WP-94)

**Perimetro**: sigilli lessicali v10 (wp81-harness/gate-lever-pins.sh §10 + reti A-TH-62/63/65/67), pin, fail-fast (num_or_void A-TH-64), doc TLS (A-TH-61/66, vm/mod.rs r.16534-16576). Gate eseguito a HEAD e52a634: PASS rc=0. Controesempi ESEGUITI (scratchpad hoare94-counterexamples.sh) contro le reti di produzione copiate al byte; controllo positivo morde (v8re/th63/th65/th44 = 1).

**VERDETTO: CONCORDO CON EMENDAMENTI.**

## Q1 — grafie che eludono ANCORA le reti v10: SÌ, sette, eseguite (0 su ogni rete)
- **CE1** UFCS a parentesi staccata `RetainSet::production_gate (&r)` e turbofish `MainUnit::vm_gate ::<>(&u)`: A-TH42 esige `(` adiacente, A-TH-63 esige il punto, il token-pin `production_gate[(]` non matcha — un TERZO sito resta verde ANCHE nei due file pinnati.
- **CE2** fn-pointer `let g = RetainSet::production_gate;` poi `g(&r)`: nessuna rete (le alias-net coprono solo TIPI).
- **CE3** split a tre righe con blocco `/*…*/` IN CODA alla riga del nome (`u.` / `vm_gate /*c*/` / `(x)`): le regole ML 1275/1276/1288 tolleran0 solo `//` in coda; asimmetria sistematica //-vs-/*…*/ negli iniziatori pend/dot/nmp. Elude anche nei file pinnati.
- **CE4** `<ProbeWindow>::arm()`: TH65_RX esige `ProbeWindow` seguito da spazi e `::` — il `>` interposto lo buca; ntot==2==narm resta verde con un terzo sito.
- **CE5** `ProbeWindow::` a fine riga, `arm()` alla successiva: grep line-based; il programma ML non conosce ProbeWindow.
- **CE6** alias ProbeWindow multiriga: use-group ML, `as` a fine riga, `type P =\n ProbeWindow;` — il ramo use del ML traccia solo `(CachedUnit|VmGate)` (r.1279).
- **CE7** punto nudo + `/*c*/` in coda, `vm_gate(x)` alla successiva: in un TERZO file elude tutto (nessuno sweep conta `vm_gate(` nudo fuori dai pinnati).
Residuo dichiarato (blocchi multi-RIGA, macro) confermato fuori portata; i sette sopra sono IN portata lessicale.

## Q2 — num_or_void «su OGNI cattura»: NO. La dicitura di S-92.0 p3 è FALSA come universale
Guardate ~12 catture su ~45. **check_pin — l'elica del gate, ~25 denti — è fail-open**: dimostrato eseguito (regex rotta ⇒ awk muore ⇒ n="" ⇒ `[ "" -ne 1 ]` errore ⇒ stampa «OK … sites (pinned)»). Stessa classe: check_class, region_split (PS/PP/PC, SS/SP/SC), body_scoped_some sito reale r.255, count_noncfgtest r.693, publish_if_armed r.476-479, tooth-4 r.525/532, sweep_class (aritmetica su n vuoto), e TUTTI gli sweep in `$( )` salvo TH63/TH65 (`[ "$n" -gt 0 ] && echo` ⇒ file saltato in silenzio); self-test decoys r.81/249/287/417/423/640/666/741/796/817/852/1089-92 (vuoto ⇒ nessun exit 2 ⇒ vacui). I confronti stringa (`= "2"`) sono invece fail-closed per costruzione. Nota: num_or_void dentro `$( )` sarebbe comunque inefficace (exit 2 uccide solo la subshell) — serve il pattern inline-case di TH63/65.

## Q3 — fix bash-3.2 dei case in `$( )`: COMPLETO nel file
Census: case a r.62 (corpo funzione, parse top-level — sicuro), r.1343 (top-level), r.1490/1533 (in `$( )`, ENTRAMBI con `(` aperta). Nessun altro case in subshell; `bash -n` su 3.2.57 pulito. gate-binary-noprobe.sh: unico case (r.87) top-level; un'eventuale rottura lì sarebbe fail-closed (selftest FAIL). Nessun emendamento.

## Q4 — pin content A-TH-66: ELUDIBILE, dimostrato eseguito
I quattro grep -c sono FILE-WIDE: una SOLA riga di commento altrove in vm/mod.rs con marker + le tre frasi passa 1/1/1/1 col doc TLS interamente cancellato (q4demo: «ELUSION CONFIRMED»). Anche l'inversione semantica («NOT SUBSUMED…») matcha il substring.

## Emendamenti
- **A-TH-68**: chiudere CE1/2/3/7 — rete PREFISSO su `::(production_gate|vm_gate)` senza àncora di chiamata (modello A-TH-63); tolleranza `/*…*/` in coda speculare a `//` nei tre stati ML; decoy eseguiti stesso-commit.
- **A-TH-69**: chiudere CE4/5/6 — census del TOKEN `ProbeWindow` per file con allowlist (worker_pool + lib re-export), che assorbe qualified-self, split e alias multiriga in una rete sola.
- **A-TH-70**: guardia numericità UNIVERSALE: catture di check_pin/check_class/region_split/self-test sotto num_or_void; sweep col case inline NON-NUMERIC (modello TH63/65); meta-dente che grep-a il gate stesso per `-ne`/`-gt` su catture non guardate.
- **A-TH-71**: A-TH-66 SCOPATO — finestra awk dall'àncora A-TH54/A-TH-61 a `impl UcEmitGuard`; le tre frasi DENTRO la finestra, count file-wide == count in-window (anti-squat), minimo righe di commento della finestra.
- **A-TH-72**: emendare la riga di WP_SESSION_92 p3 «num_or_void su OGNI cattura» in «12 catture per NOME» (supersessione da ledger, mai riscrittura silenziosa).

## Kill-switch
- **KS-TH-94-1**: finché A-TH-68/69 non mordono con decoy eseguiti, i pin 2==2 (arm) e 1/1 (dot-name) sono ADVISORY — nessuna cifra m9x con probe li cita come VERDICT-grade.
- **KS-TH-94-2**: dente il cui conteggio passa per `-ne`/`-gt` senza guardia ⇒ le sue righe OK sono VOID (vacuità presunta) finché guardato.
- **KS-TH-94-3**: pin content soddisfatto da frasi FUORI dalla finestra ancorata ⇒ doc-pin VOID.

## Refutazioni capitali: NO
Nessuna cifra di campagna cade: i sigilli sono cintura, il giudice dei mint resta rustc (privacy VmGate). Cade però la DICITURA universale di A-TH-64 (A-TH-72) e la pretesa che il pin A-TH-66 sia content-pinned (è marker-pinned distribuito).

---

# Verbale SEDIA 2 — Matsakis (Concilio WP-94)

Perimetro: ownership/aliasing, atomics/ordering, cfg-soundness. Mandato: REFUTARE.

## VERDETTO: PASS-CONDIZIONATO
La meccanica (edge Release/Acquire, monotonia ARRIVALS, cfg-split) regge; la **sufficienza formale** della coppia pulita come dichiarata è REFUTATA (Q3).

## Q1 — L'edge chiude il gap dichiarato? SÌ, ma solo in una direzione.
Dec `fetch_sub(1, Release)` (worker_pool r.607-608) / load `Acquire` (r.305-306): quando `outstanding_post` legge-da la dec, l'intero teardown del worker happens-before il load — il gap dec→load dichiarato (main r.268-269) è chiuso. Inoltre la catena ARRIVALS-inc (r.686) →sb→ send (r.689) →hb canale→ recv (r.515) →sb→ dec(Release) →sw→ post-load(Acquire, main r.282) →sb→ arr_post (r.283) forza arr_post a vedere ogni arrivo la cui dec è stata letta: la richiesta window-contained È catturata in ogni esecuzione fresca. L'inc Relaxed di OUTSTANDING (r.681) e di ARRIVALS (r.686) fra loro non apre buchi nuovi: ogni riordino inc/inc visibile produce pre≠0 o arr_pre≠arr_post ⇒ ADVISORY, fail-closed. **Il buco non è di ordering: è di freshness** — nessun ordering (nemmeno SeqCst) obbliga il testimone a leggere l'ultimo valore; la visibilità in tempo finito è QoI hardware (intro.progress è "should"), non garanzia del modello.

## Q2 — cfg-split coerente? SÌ sulla monotonia, NO su un dente dichiarato.
Ogni path che inc-a OUTSTANDING inc-a ARRIVALS nello stesso blocco `cfg(any(census-instrumentation, mem-census))` (r.673-687); QUEUE_DEPTH simmetrico census-only (inc r.676-680, undo r.706-707). Il failed-send NON undoa ARRIVALS (r.704-708): monotonia da contratto rispettata (doc r.222-224). **Refuto però il dente d'appoggio**: il commento r.700-703 delega la cattura del dispatch-Err al «driver's rows==N» — ma la riga census per-richiesta è census-instrumentation-ONLY (r.180-182): sul canale mem-census rows==N NON ESISTE, e ARRIVALS resta gonfiata di un arrivo mai entrato. VOID dichiarato senza trigger nominato sul canale che conta.

## Q3 — Esiste la sequenza che mente? SÌ (macchina astratta).
Dispatcher: inc OUTSTANDING→1, inc ARRIVALS→N+1, send. Testimone (endpoint inline main r.279-283, NESSUN edge col dispatcher: non passa da dispatch()): `outstanding_pre` legge lo **0 iniziale stale** (l'inc Relaxed non happens-before il load; l'Acquire non compra freshness), `arr_pre`=N+1 (fresca). Il worker esegue e smonta DURANTE il dump, dec→0. `outstanding_post`=0 (iniziale o post-dec: indifferente), `arr_post`=N+1. **pre==post==0 ∧ arr_pre==arr_post con finestra SPORCA**: esecuzione coerente (read-read coherence rispettata) e permessa dal modello C++. Su x86-64/AArch64 reale il RMW lockato è subito coerente e il dump dura ms ⇒ praticamente esclusa; la protezione REALE è la sequenzialità del driver, non la coppia. La coppia è tripwire fail-closed NECESSARIA, non prova SUFFICIENTE.

## Q4 — lsp_check.rs, fragilità pre-wiring.
(a) `link` batch order-dependent con mappa propria (r.1195-1206): al wiring la tabella `Linked` persistente = **seconda verità** parallela alla class table VM; `Linked` clona in profondità ogni membro ereditato (String-heavy, O(depth×membri)) — doppio modello o churn di conversione. (b) **Antenato assente = skip silenzioso** (`if let Some(pl) = map.get(..)` r.1216-1227): fail-open al wiring con classi condizionali/autoload — serve policy nominata. (c) `deps` si PERDONO su `Err` (r.1202-1205): Zend emette la Deprecated al punto di scoperta; il wiring deve emettere subito o l'ordine diverge. (d) `lc = to_lowercase()` Unicode (r.398-400) vs `zend_str_tolower` ASCII-only di Zend: nomi UTF-8 con case-folding non-ASCII (İ) collidono da noi e non in Zend — x5 non lo copre. (e) `unreachable!` su compound in `resolve_atom` (r.692-694): con input dal parser reale = abort di processo; mappare a Err.

## Emendamenti
- **A-MS-59**: riscrivere il commento main r.268-274: la coppia pulita è condizione NECESSARIA fail-closed; verdict-grade SOLO con enforcement sequenziale del driver dichiarato in-band; nessun rafforzamento di ordering compra freshness — dichiararlo.
- **A-MS-60**: trigger VOID per dispatch-Err NOMINATO sul canale mem-census (HTTP 5xx osservato dal driver, o contatore DISPATCH_ERR su `any(...)`).
- **A-MS-61**: lsp_check pre-fase-2: policy antenato-assente + emissione deprecation al punto di scoperta + `lc` ASCII-only + Err al posto di `unreachable!`.

## Kill-switch
- **KS-MS-94-1**: «window clean» consumata come verdict-grade senza riga in-band di enforcement sequenziale del driver ⇒ Δ di fase VOID.
- **KS-MS-94-2**: run mem-census con dispatch-Err senza trigger VOID nominato ⇒ run VOID.

## Refutazioni capitali
- **SÌ (1)**: sufficienza formale di pre==post==0 ∧ arr_pre==arr_post (Q3: esecuzione stale-coerente con finestra sporca e verdetto pulito).
- **NO**: su edge dec→load (reale), monotonia ARRIVALS (regge), cfg-split degli inc (coerente).

---

# Verbale 3 — Klabnik (Concilio WP-94) — FORGE contro il gate cifre v3

Metodo: bite-test con controllo negativo. Ogni riga sotto è stata ESEGUITA
(giudice a HEAD `judge_sha=8f36d4dd967983c5`, head `e52a634a70fe`; selftest v3
**PASS in 13:30**; `--all` baseline **rc=1** per 8 verbali WP-94 untracked).
Doc forge in scratch fuori repo; nessun commit; albero porcelain
(solo `?? php-rust/wp94-harness/`).

## Forge

| # | Forge (eseguito) | Esito | Controllo negativo |
|---|---|---|---|
| **F-K1** | doc con `b_peak per worker = 6.910.767 B` (prefisso decimale della riga `rev=` WP-80) | **ADVISORY-PASS rc=64** — un'IDENTITÀ nominata legalizza una MISURA | `6.910.768 B` → FAIL «not in committed corpus» |
| F-K1b | `massa residua = 92.952.364 B` (prefisso del commit S-92.0) | **rc=64** (≈88,6 MiB, del tutto pubblicabile) | idem sopra |
| **F-K2** | `picco per il worker W16-20.999.999 B` | **rc=64** — lo stripper di identificatori MANGIA `W16-20` e il resto `.999.999` è invisibile al tokenizer (lookbehind `.`) | `20.999.999 B` nudo → FAIL |
| F-K2b | `b_peak-20.999.999 B per worker` (forma etichetta-trattino, la più pubblicabile) | **rc=64** | idem |
| **F-K3** | `19.922.944 B = 19,00 MiB [derivata: prov 24313856@…m88.slope.w16.r4.a1.memcensus:40 − 4390912@…:47]` — stessa chiave `reserved=`, **righe/istanti diversi** | **rc=64** + «A-SK60 provenance-verified» | senza tag → FAIL 2 token |
| **F-K4** | `bash -c "$(cat patched.sh)" gate-measure-cifre.sh --all` da `wp81-harness` (giudice patchato: dente perimetro disarmato) | **rc=0** + `PASS gate-measure-cifre --all … judge_sha=8f36d4dd967983c5` | STESSO testo come file `zzforge-k5.sh` → `REFUSE … (A-SK-78)` |
| **F-K5** | `.MD` maiuscolo con cifra fabbricata dentro `php-rust/wp94-harness/` + `--all` | **evaso**: nessuna riga FAIL lo NOMINA (8 FAIL prima, 8 dopo) | il gemello `.md` è notoriamente morso (T16) |
| F-K6 | advisory su `ROOT/COVERAGE.md` e `ROOT/sessions/WP_SESSION_75.md` (fuori dal complemento `^php-rust/.*\.md$`) | FAIL **11** e **4** cifre non giudicate, invisibili a `--all` | i gemelli `php-rust/COVERAGE.md` hanno riga manifest |

## Misure a corredo (replica esatta del corpus: cardinalità riprodotta 24.329/24.329)

- Corpus: 916 sorgenti, **cardinalità 24.329 = budget 24.329** (headroom **0**).
- Pool operandi A-SK-81: 616.778 occorrenze etichettate; **3.113.808** coppie
  stessa-chiave/stesso-file → **418.116** differenze distinte, **163.175** nella
  finestra 1–99 MB; chiusura **50/100** dei target MiB-tondi (k·1.048.576,
  k=1..100) e 5/100 dei 1M-tondi. L'etichetta vincola la GRANDEZZA, mai l'ISTANTE.
- Identità: 1.447 antenati, **54** con prefisso decimale ≥7 cifre (33 a 8);
  le 2 righe `rev=` in vigore legalizzano 3 token, spendibili come byte.
- Perimetro: 238 `.md` sotto `php-rust/` (238 righe manifest) contro **22 `.md`
  committati FUORI** (README/COVERAGE/TODO **di root — quelli che GitHub
  pubblica**, `sessions/WP_SESSION_75.md`, `diary/*`), **894** occorrenze di
  token ≥3 cifre mai giudicate.

## VERDETTO

Il v3 **non regge**: quattro rifiutazioni capitali. A-SK-78 firma un OMONIMO A
UN PATH, non il testo che gira (`$0` è scelto dal chiamante); A-SK-76 rifiuta la
colla a destra e la lascia intatta a sinistra; A-SK-81 confonde «stessa chiave»
con «stesso istante»; A-SK-80 chiama «complemento» un sottoalbero. In più T16 è
**vacuo oggi**: asserisce solo `rc==1` da `--all`, e la baseline è già 1 ogni
volta che esiste un `.md` untracked (cioè in ogni sessione di concilio).

## Emendamenti

- **A-SK-82** (tether reale): ancorare a `${BASH_SOURCE[0]}` e REFUSE se vuoto o
  ≠ `$0` — verificato: `bash -c`/`bash -s` danno `BASH_SOURCE` VUOTA, un file
  vero la valorizza. Dente: F-K4 come tooth permanente T23.
- **A-SK-83** (colla a sinistra): un run ≥3 cifre ADIACENTE a un identificatore
  (`[-_.]`) è RIFIUTATO, mai cancellato — simmetrico ad A-SK-76 (T24).
- **A-SK-84** (istante): operandi prov dalla STESSA RIGA, o da righe con
  marcatore d'istante uguale dichiarato; la chiave sola è insufficiente (T25).
- **A-SK-85** (identità ≠ misura): token `rev=` citabile solo senza
  raggruppamento italiano e mai adiacente a un'unità (B/MiB) (T26).
- **A-SK-86** (perimetro vero): ogni `.md` del REPO, regex **case-insensitive**;
  le 22 esclusioni fuori `php-rust/` per NOME nel manifest (T27 = F-K5).
- **A-SK-87** (denti per NOME): ogni dente su `--all` asserisce la RIGA FAIL che
  NOMINA il forge, previa misura della baseline; T16 riscritto.

**Refutazioni capitali: SÌ** — KS-SK-94-1 (tether/F-K4), KS-SK-94-2
(stripper/F-K2), KS-SK-94-3 (istante/F-K3), KS-SK-94-4 (perimetro/F-K5+F-K6);
KS-SK-94-5 = vacuità di T16 e identità-come-misura (F-K1).

---

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

---

# Verbale SEDIA 5 (Bak) — Concilio WP-94 — A-DL-59 (page slack vs pendenza invisibile)

## VERDETTO
**CONFERMATO con emendamenti.** Ricomputo indipendente (python, senza dl59-join.sh) dai 20 raw `wp78-harness/measure-out/m90.slope.w{4,8,12,16}.r{1..5}.a1.memcensus` (NUL rimossi): **match al byte con dl59-join.out su ogni colonna e ogni run**. La refutazione dell'ipotesi page-slack di Leijen regge. La forma FORTE della conclusione («ad arena/chunk») è un overreach: va declassata a «fuori dai bin del heap visitato».

## Ricomputo (per NOME)
- **SLACK_naive, 20/20 al byte** (es. w4.r1=7.970.672; w16.r2=111.306.784; w12.r3=60.351.568).
- **SLACK_global e glob_d2, 20/20** (es. w12.r1 glob=4.956.061,3→4.956.061; d2=6.007.492,7→6.007.493: arrotondamento, non errore).
- **heap_commit, pwork_commit, freePgRows (16 solo su W=16, 4.194.304B=16×262.144), visheap_free_c, thrSumMatch W/W, d1mispl=d2mispl=0: 20/20 al byte.**
- Medie per-W identiche (8.098.364,8 / 28.261.120,0 / 59.613.318,4 / 110.385.798,4; global 2.024.591,2 / 3.532.640,0 / 4.967.776,5 / 6.899.112,4).
- OLS: naive slope=8.455.362,5, int=−32.963.974,4 (ratio **1,887**, int-ratio −0,458); global slope=401.467,5, int=341.355,0 (ratio **0,090**, int-ratio 0,005). 4-means≡20-pt (design bilanciato) ✓.
- Shared-heap: in ogni dump#1 **1 solo heap ptr, 1 solo committed** per tutti i thr; Σ_thr committed/pwork = 1,66× (W4) → 10,4× (W16); w16.r1 Σ=4.049.975.808 ✓ al byte. La Σ naive è genuinamente invalida.
- Autorità verificate su `wp91-harness/repair90-estimators.out`: slope_inv=4.480.174, int_inv=71.934.811, slope_vis=15.777.004, a_vis=692.184 ✓ (righe 101-102).

## Q1 — Le cifre reggono? **SÌ**, zero mismatch su ~260 valori confrontati.

## Q2 — Scelta dump#1: **sana, con una crepa dichiarabile.** La tripla ridondanza morde davvero: split alla prima size-class ripetuta ⊕ uguaglianza committed con mi_bin_thr_sum (20/20, W/W) ⊕ posizione vs marker `mi_proc ckpt=peak_inreq` (0 misplaced su 20). Un bin apparso per la prima volta in dump#2 romperebbe lo split, ma farebbe fallire thrSumMatch: il guard esiste. Fragilità residua: dump#1 NON è esattamente pwork — in **w16.r2 commit(peak_inreq)−commit(pwork)=+1.048.576B, w16.r3 +4.194.304B** (≤1,1%, 2/20 run; immateriale vs invisibile ~137MB, ma il .out dice «near-identical» senza quantificare).

## Q3 — Identità vis_model≈bin-committed: **fatto su TUTTE le 20 run**, non solo sulle medie: deviazione max **+0,172%** (w12.r4/r5), min −0,030%; pattern lieve concavo (W8/12 positivi) = residui del fit, non struttura. NON è coincidenza: il potere discriminante è reale — se il census contasse used_b invece di committed la deviazione sarebbe ~2,7-3,2% (slack/heap); osservato <0,18%. Caveat: è semi-tautologica (vis stimato dalla stessa famiglia census), quindi è un fatto di **coerenza interna** che colloca lo slack DENTRO il visibile — sufficiente per refutare Leijen, non un oracolo indipendente.

## Q4 — «L'invisibile vive ad arena/chunk» è l'UNICA lettura? **NO.** I dati provano solo: invisibile = pwork_commit − bin-committed, cioè **fuori dai bin del heap visitato**. Alternative non escluse, per NOME: (1) **segmenti/pagine abbandonati** di thread morti (di nessun heap; heaps_total=1 non li copre); (2) **MI_BIN_HUGE / allocazioni OS-direct** oltre l'ultimo bin (tls arriva a size=1.048.576 su W16: la soglia di copertura non è dimostrata); (3) **metadata di segment/pagina** mimalloc (scala con W). Esclusa invece la defer-heap: `src=defer visit=NULLHEAP` in tutti i dump delle 20 run ✓. La frazione invisibile cala 58,3%→35,1% (W4→W16), coerente con massa intercept-dominata ✓.

## Emendamenti
- **A-BB73-1**: declassare la chiusa a «l'invisibile vive FUORI dai bin del heap visitato»; arena/chunk-slack, abandoned segments, huge/OS-direct e metadata restano indistinti — discriminarli spetta al canale barrier A-DL-57/58 (dump arena stats + abandoned count).
- **A-BB73-2**: dichiarare il drift dump#1↔pwork con cifre (w16.r2 +1.048.576B, w16.r3 +4.194.304B; 18/20 identici).
- **A-BB73-3**: riformulare l'identità come test committed-vs-used (margine 3% contro <0,18% osservato), non come oracolo esterno.

## KS-BB-94-n
Nessun kill-shot. (Il min..max per-W della sezione identity nasconde le deviazioni per-run, ma verificate tutte ≤0,172%: nessuna si avvicina alla soglia.)

## Refutazioni capitali: **NO.**

---

# Verbale SEDIA 6 — Pedersen (Concilio WP-94)

Perimetro: confine per-richiesta, lifecycle harness, failpath. Mandato: REFUTARE.
Evidenza: `crates/php-server/src/main.rs` (246-299), `crates/php-server/src/worker_pool.rs`
(216-314, 595-615, 677-686), `wp91-harness/battery-91pre.sh` (57-117),
`wp83-harness/battery-equivalence.sh` (333-353), `sessions/WP_SESSION_92.md`;
mini-ledger eseguito: `/private/tmp/pp68-mini.sh` (T1-T5).

## VERDETTO: PASS CON RISERVE — nessuna refutazione capitale; 1 implicazione da declassare, 1 bug failpath, 2 dichiarazioni mancanti.

## Q1 — Finestra della riga unificata
(a) La richiesta che arriva DOPO il post-load ma PRIMA che il driver legga la riga è
**irrilevante PER COSTRUZIONE**: i byte del dump sono già scritti in `$PHPR_MEM_CENSUS`
dentro `peak_census_dump`, e la riga stampa quattro locali già caricati — un arrivo tardivo
può sporcare solo dump FUTURI. Lo dichiaro chiuso.
(b) **Buco residuo formale**: l'implicazione nel commento (main.rs:273) «pre==post==0 ∧
arr_pre==arr_post ⇒ window clean» è FALSA nel modello di memoria puro. Una richiesta
0→1→0 contenuta nella finestra: entrambi i load Acquire di OUTSTANDING possono
legittimamente leggere il PRIMO zero (la coherence non impone freschezza), e allora
nessun synchronizes-with col dec Release esiste; `census_arrivals_now` carica per giunta
**Relaxed** (worker_pool.rs:313) su inc Relaxed — arr_post può mancare l'arrivo. La coppia
può dichiarare pulita una finestra sporca. È schermata SOLO dal protocollo (driver
sequenziale: l'unico client è la richiesta census stessa, che risponde dal router PRIMA di
`dispatch` e non tocca i contatori).
(c) Universo del testimone = sole richieste DISPATCHED: 404, `/__reqns` e il census stesso
non toccano né OUTSTANDING né ARRIVALS — mutazione heap invisibile alla coppia.
(d) Il dec scatta dopo la send sulla ONESHOT, non dopo la write sul socket: il free del
body della richiesta PRECEDENTE (A-PP-69) può cadere dentro una finestra "pulita" anche
in sequenziale (scheduling del task axum). Già dichiarato — resta corretto dichiararlo.

## Q2 — A-PP-68, caso limite costruito
Bash esegue il trap pendente al completamento del figlio, PRIMA del ramo `then`: segnale
durante un gate ⇒ gate_in_flight ancora settato ⇒ mai una riga `none` per un segnale
arrivato IN un gate. Quella direzione regge. Ma: **deferred è DERIVATO, non misurato**
(`defer=[gif≠none]`, battery-91pre.sh:72-73) — porta zero bit oltre gate_in_flight.
Caso limite: segnale durante `say "== $label =="` (riga 109, gate settato ma MAI
lanciato) ⇒ riga `gate_in_flight=label deferred=1`: «il trap è corso a fine gate» è falso,
il gate non è mai partito. Secondo limite: segnale durante i denti di batteria non-gate
(grep judge_sha 146-149, comm/join matrix 153-190) ⇒ `none/0` etichetta come «fra i
gate» lavoro verdict-bearing in corso. Terzo: TERM pid-mirato durante parity-full — il
figlio corre a termine, attempt_epoch = tempo del trap, l'epoca del segnale è PERSA
(anche +20 min). Nessuno è VOID; tutti e tre vanno dichiarati.

## Q3 — reason=signal+head-moved: eseguito
Mini-ledger /tmp, denti v2 ESATTI di battery-equivalence (338-350): **T1 riga reale del
trap PASSA** — '+',':' non rompono la grammatica k=v (nessuno spazio nel valore);
**T3** i due rev si estraggono a macchina (`head-moved:([0-9a-f]+)-to-([0-9a-f]*)`,
hex senza collisioni con '-'/'+'); **T4** il negativo MORDE; **T5** l'adiacenza
`gate_in_flight=X deferred=D` è RIGIDA (ordine invertito ⇒ VOID: fail-closed, da
dichiarare). **BUG trovato (T2)**: se `git rev-parse` fallisce nel trap (volume esterno
staccato — rischio REALE su Extreme Pro) `$now` è vuoto ⇒
`reason=signal+head-moved:REV-to-` passa i denti e traveste un guasto git da head-move
con destinazione VUOTA.

## Q4 — Cardinalità A-PP-69
Per le risposte census la dichiarazione regge A FORTIORI: i formati grandi NON viaggiano
nel body HTTP (peak dump → file; self census → stderr via `census_line`/
`mi_bin_thread_rows`, worker_pool.rs:561-580); i body sono stringhe fisse minime, allocate
DOPO il post-load. Ma «un Vec per richiesta» è un **lower bound**: l'involucro risposta
(HeaderMap + insert x-phpr-pid, buffer di scrittura hyper, oneshot) libera anch'esso lato
axum dopo il dec — O(1) alloc per richiesta, non 1; e per le risposte PHP ordinarie
l'unico Vec ha taglia ILLIMITATA (l'intero output): il rischio è la magnitudine, non la
cardinalità.

## Emendamenti
- **A-PP-70**: il claim clean-window è verdict-grade SOLO con driver sequenziale
  dichiarato in-banda (altrimenti ADVISORY), o si passa a SeqCst sui sei op dei contatori;
  emendare il commento main.rs:273 di conseguenza.
- **A-PP-71**: `on_signal` sentinella su `$now` vuoto (`git-unavailable`), mai emettere
  `-to-` con destinazione vuota.
- **A-PP-72**: deferred= dichiarato DERIVATO per NOME; i denti non-gate della batteria
  ricevono etichetta propria (es. `GATE_IN_FLIGHT=tooth:matrix`).
- **A-PP-73**: A-PP-69 riformulato: residuo = involucro risposta, O(1) alloc, taglia ∝
  body; risposte census esenti per costruzione (canali file/stderr).

## KS
- **KS-PP-94-1**: una finestra dichiarata pulita senza driver sequenziale in-banda è ADVISORY.
- **KS-PP-94-2**: un campo derivato non è un secondo testimone.
- **KS-PP-94-3**: un valore composto con separatori resta grammaticale solo finché il
  sotto-valore non può essere vuoto o contenere il separatore.

## Refutazioni capitali: NO.

---

# Verbale SEDIA 7 — Leijen — Concilio WP-94 (mimalloc, contatori, canale iter-3)

**VERDETTO: ACCETTO la refutazione della MIA ipotesi WP-93 (page-slack). E refuto un'assunzione di design92/A-DL-57.**

## Q1 — Refutazione accettata

Sì, senza riserve tecniche. dl59-join.out regge per NOME: thrSumMatch W/W su tutti i 20 raw, d1mispl/d2mispl=0, Σ_thr committed (4.049.975.808 su w16.r1) >> commit di processo, identità decisiva vis_model≡bin-committed <0,5%. La mia formula letterale dava 1,887 con intercetta NEGATIVA: forma e segno sbagliati — moriva anche senza il dedup. Lo slack fisico 0,090 chiude: la pendenza invisibile vive ad arena/chunk.

**Scomposizione CANDIDATA per NOME** (verificata sui raw m90, ckpt=peak_inreq_pwork):
1. **T-HUGE**: `malloc_huge` current = **39.911.424 B × W ESATTO** (159.645.696@w4, 638.582.784@w16), `malloc_huge_count` = **6×W esatto** (24/96). Sestetto huge per-worker. Ma total==peak==current e current@w4 (159MB) > commit (151MB): o il current non decrementa sul free (stat rotta ⇒ i blocchi sono transienti, il residuo committed post-purge è il candidato della pendenza) o vivono nel reserved (1.078.263.808 B fisso) parzialmente decommitted. **È il primo indiziato.**
2. **T-CHUNK**: chunk_bins S current 2→10 (w4→w16), X≈1: granularità chunk per-worker; servono i BYTE per classe dal header vendored (v3, libmimalloc-sys 0.1.49) — non sono in banda.
3. **T-EAGER**: arena_eager_commit (ord 4) — residuo = committed − (bin-committed + huge-committed); nota: pwork_commit ≡ `mi_arena key=committed` AL BYTE (151.781.376 w4.r1) ⇒ l'invisibile è interamente contabilità d'arena.
4. **T-ABAND**: abandoned — atteso ~0 (i worker non muoiono); si misura per REFUTARE.

**Ordine di misura**: (1) controllo positivo decremento huge-current (one-off, binario esistente, zero campagne); (2) promozione malloc_huge* a righe nominate; (3) barriera A-DL-57/58; (4) join INV(W)=T-HUGE+T-CHUNK+T-EAGER+residuo sui W∈{4,8,12,16}, grammar v2.

## Q2 — A-DL-55 va ridisegnato: SÌ

Simboli DISPONIBILI verificati nel tree (memcensus.rs r.631-721): mi_stats_get_json, mi_heap_of, mi_subproc_visit_heaps, mi_heap_visit_blocks/abandoned_blocks, mi_heap_main, mi_option_get, mi_collect. **mi_heap_get_default resta NON esportato** (r.633-637 e r.1425-1429): la lezione vale ancora. Il canale oggi estrae committed/reserved/pages/page_committed (count3) + arena_count/purged/reset/purge_calls/mmap_calls + chunk_bins; **malloc_huge/malloc_huge_count NON sono righe nominate** (solo dentro mi_arena_json) — e sono la leva della pendenza. Vincoli: parse SECTION-AWARE (precedente A-DL46: commit_calls bandito per parse section-blind); `page_committed` legge 0 a pwork ⇒ controllo positivo prima dell'uso (contatore tutto-zero = controllo fallito, lezione WP-72).

## Q3 — Contaminazione dello heap condiviso

- **CONTAMINATO**: ogni claim m89/m90 formulato per-worker/per-theap dai dump tls (thr=k era heap-by-visit-index replicato); la promessa di A-DL46-census («WHERE the committed lives — per heap») mai realizzata.
- **NON contaminato**: slope_vis/vis_model (l'identità <0,5% lo RIABILITA a livello processo); marginal 0,778 (b_peak e commit sono di processo); b_peak mediana, robustezza 0,972. **VSELF/label census-self**: già ritirata; il finding la spiega meglio (una vista replicata non separa il self), non la resuscita.
- **DA RIVERIFICARE per NOME**: VDL24 per-thread 7.349.977×2 (WP-84) e canary per-thread WP-85 — solo se la sorgente era il piano tls-dump; se era VmGate/contatore on-thread, salvi.

## Q4 — Fedeltà A-DL-57/58

A-DL-57 fedele sui simboli (mi_heap_of esiste). **Ma assume l'esito**: heaps_total=1 in m90 è evidenza prima-facie che in QUESTO tree i thread possano condividere l'heap; se il probe on-thread restituisce ptr IDENTICI, il dente REFUSE scopato rende il canale always-VOID. Serve il pre-test di pluralità PRIMA della barriera. A-DL-58: il jbuf 64KiB di mi_stats_get_json (r.1214) è un'allocazione DENTRO lo snapshot — non nominata fra i buffer pre-allocati.

## Emendamenti

- **A-DL-61**: PT-1 pluralità heap (2 thread spawn, probe, mi_heap_of: ptr distinti?) PRIMA di costruire la barriera; se identici ⇒ split per-worker al piano contatore on-thread, non al piano heap.
- **A-DL-62**: A-DL-55 ridisegno = malloc_huge/malloc_huge_count righe nominate section-aware + mappa byte chunk-bin dal header vendored come riga header + controllo decremento huge.
- **A-DL-63**: controllo positivo page_committed (0 = morto o vero-zero: decidere col morso).
- **A-DL-64**: jbuf pre-allocato fuori fase 2 (o emissione arena-json post-fase-3).

**Kill-switch**: **KS-DL-94-1** lettura di malloc_huge current come massa viva senza controllo decremento ⇒ VOID. **KS-DL-94-2** split per-worker pubblicato senza PT-1 superato in-band ⇒ VOID. **KS-DL-94-3** chiave nuova estratta dal json con parse section-blind ⇒ riga VOID.

**Refutazioni capitali: SÌ (2)** — (a) la mia ipotesi page-slack WP-93 è morta (accettata); (b) l'assunzione implicita di A-DL-57 «il probe produrrà W ptr distinti» è non provata e contraria all'evidenza m90: senza PT-1 il design eredita il difetto che dice di curare.

---

# Verbale SEDIA 8 — Stogov (Zend/opcache, contratto LSP) — Concilio WP-94

Oracle: PHP 8.5.7 `/opt/homebrew/opt/php/bin/php`. Probe: `/tmp/stogov94` (33 file, tutti eseguiti). Oggetto: `crates/php-runtime/src/lsp_check.rs` (fase 1, 56 unit).

## VERDETTO

Il checker è SOLIDO sul perimetro delle 51 fixture: ordine bitmask, DNF, null, static, parent/self, canale proprietà — tutti CONFERMATI al byte. Ma il perimetro NON è il contratto: **2 refutazioni capitali** fuori-fixture (iterable-desugar; ctor-prototype da abstract) + 3 under-fatal e 4 famiglie messaggi mancanti. Fase 2 NON consumabile senza A-DS61..65.

## Q1 — raffinamento bitmask (20 probe, canale `&m()` + canale prop)

CONFERMATI: reversal totale `bool|float|int|string|array|object|callable` → `callable|object|array|string|int|float|bool` (= BUILTIN_ORDER esatto); named e gruppi DNF in ordine DICHIARATO (`(A3&B3)|string|(C3&D3)` → `(A3&B3)|(C3&D3)|string`); null ultimo (`int|null|string` → `string|int|null`); collapse `?T` solo binario (`?int`, `?false`, `?static`; `(A4&B4)|null` NON collassa); `static` in posizione FISSA fra named e builtin anche se dichiarato primo (`static|Foo|int` → `Foo|static|int`); `parent` stampato RISOLTO (`A7`/`P`); `Type of C::$x must be string|int` = stesso formatter.

**REFUTATO (capitale): `iterable` non è un atomo di stampa.** Zend lo desugara: `Traversable` entra nella lista NAMED alla posizione dichiarata, `array` nel bitmask: `iterable|int` → `Traversable|array|int`; `Zoo|iterable` → `Zoo|Traversable|array`; `iterable|Zoo` → `Traversable|Zoo|array`; `?iterable` → `Traversable|array|null` (NIENTE collapse). Il checker stamperebbe `iterable`/`?iterable`. `iterable` solo → `Traversable|array` anche standalone. `array|Traversable` esplicito ≡ identico.

## Q2 — famiglie mancanti provate (per NOME)

1. `Class C contains 2 abstract methods and must therefore be declared abstract or implement the remaining methods (I::m, I::n)` — grammatica singolare/plurale; membro-hook stampato `I::$x::get` (proprietà d'interfaccia non implementata).
2. `Abstract function A::f() cannot be declared private` (in classe; nel trait lecito e l'override private è esente — oracle CLEAN ✓ checker).
3. **final const via INTERFACCIA**: `C::X cannot override final constant I::X` (anche enum `E::X`) — il checker raccoglie i const SOLO dal parent: under-fatal.
4. **set-hook in EREDITARIETÀ è famiglia "Declaration of"**: `Declaration of C::$x::set(string|int $v): void must be compatible with P::$x::set(string|int|float $v): void` — con `: void` stampato; contravariante (widen CLEAN). Il checker controlla solo GET: under-fatal. DS60-5 resta vera SOLO intra-classe.
5. set param esplicito NON tipato su prop tipata = fatal famiglia x2 (`Type of parameter $v of hook C::$x::set…`) — il checker passa silenzioso.
6. `Hooked properties cannot be readonly` (anche promoted).
7. RTWC su hook NON sopprime (fatal normale) ✓ checker già conforme; `static` nei param = `Cannot use the static modifier on a parameter` (parser, fase 2); enum case vs const iface non-final: CLEAN.

## Q3 — A-DS59, probe a 3-4 livelli

x6-replica ✓ (cita I); Mid SILENTE ✓ (cita I); Mid che ALLARGA (`int|string`) → il fatal cita SEMPRE I originale ✓; abstract-Mid-implements-I con ctor concreto → cita I ✓.

**REFUTAZIONE CAPITALE (q3b)**: `abstract class A { abstract __construct(int) }` → `B` ctor concreto → `C(string)`: oracle **FATAL** `…must be compatible with A::__construct(int $x)`; il checker dice CLEAN (il ramo `parent.is_abstract` guarda solo il parent DIRETTO e non registra `ctor_proto`). Propaga a profondità 4 (r_q3e, cita sempre A). Il prototipo-ctor nasce da iface **O da ctor ABSTRACT**, e propaga uguale. Il ctor CONCRETO in classe astratta resta esente (q4a/unit ✓). Viceversa (checker fatala dove Zend no): NON trovato.

## Q4 — rischi fase 2 ORM/hk (per NOME)

- **Stub interni tentative con firme ESATTE**: `ArrayAccess::offset{Exists,Get,Set,Unset}`, `Countable::count`, `IteratorAggregate::getIterator`, `JsonSerializable::jsonSerialize` — q4a: Deprecated con `ArrayAccess::offsetGet(mixed $offset): mixed`; senza registry tentative → falsi FATAL di massa su doctrine/collections legacy e proxy ORM.
- **Grafo builtin**: q4b `getIterator(): ArrayIterator` CLEAN solo se il registry conosce `ArrayIterator`/`Generator`/`Closure`/`UnitEnum`/`BackedEnum`; altrimenti falso `…because class ArrayIterator is not available` su `ArrayCollection::getIterator`, Symfony `ParameterBag/HeaderBag::getIterator`.
- Doctrine: proxy generati (extends entity), guardie `class_exists` condizionali (timing t3), trait `insteadof/as` non modellato (`ClassMetadata`), `iterable` nei tipi (desugar Traversable in ogni messaggio).

## Emendamenti

- **A-DS61**: desugar `Iterable` → `Named("Traversable")` in posizione dichiarata + flag array; vietato il collapse `?iterable`.
- **A-DS62**: `ctor_proto` registrato ANCHE dal ctor ABSTRACT (chiave=prototipo; messaggio cita la classe del prototipo).
- **A-DS63**: check set-hook in ereditarietà (contravariante, `: void` in firma) + fatal x2 su set param esplicito non tipato.
- **A-DS64**: const finali raccolti dalle INTERFACCE + famiglia abstract-methods-count (grammatica sing/plur, membri hook).
- **A-DS65**: registry fase-2 con seed builtin (grafo + tentative) PRIMA del gate ORM/hk.
- **KS-DS-94-1**: ogni emendamento nasce con fixture oracle-morsa e pin length-prefixed PRIMA del codice (legge KS-DS-92-2).
- **KS-DS-94-2**: gate ORM 3E/13F / hk 1665 NON consumabile senza A-DS65.

## Refutazioni capitali: **SÌ — 2** (q3b ctor-proto abstract: il checker benedice dove Zend fatala; iterable-formatter: byte-divergenza su ogni messaggio contenente iterable).

---

# Verbale SEDIA 9 — Gregg (metodologia di misura, doc==macchina per NOME, leggibilità) — Concilio WP-94

**VERDETTO: CON EMENDAMENTI. Due refutazioni capitali** (token di grado mancante su citazioni vive; schegge-virgola nel corpus del gate v3).

## Q1 — doc==macchina AL BYTE

Verificate contro le righe macchina: b_boot 2.252.800/se 6.345,5 (repair90 r37) · b_work 17.276.928/se 501.347,7→501.348 (r52) · somma 19.529.728 (r88) · delta **−45.875 col segno** (r89 `delta=-45875` — A-BG63 attuata) · b_peak(med) 20.289.946/se 1.084.655/banda [18.120.635, 22.459.256] (r87) · robustezza 0,972/0,999 (r86) · marginale 15.777.004=0,778 (r101) · invisibile 4.480.174+71.934.811 (r102) · dl59 ratio 0,090/1,887 (r109/r102 del .out) · 51 fixture "=18 v1+19 v2+8 v3+6 v4" (ds35-verify4.out r830) · budget 24329 (`gate-cifre-corpus.budget`) · rev 6910767 (`gate-cifre-revs.txt` r8) · 2.8/46.25 revocate + 110 espunta (gate r476-478, T19) · ADVISORY-PASS=64 (gate r106/1146) · **+87 esclusioni ESATTO al suo commit**: judge=no 141→228 a 2ab2780 (il mio conteggio a fine sessione dà 89: i 2 extra sono dei commit successivi — la cifra è corretta per NOME al commit A-SK-80) · A-BG65 verificato: `bite-test-historic.log` dichiara head=c71bf9e, atterraggio ce1ee27; `git rev-parse ce1ee27^`==c71bf9e ✓. **Gap dichiarato**: «56 unit / 39 message-asserting» (lsp_check) non ha riga macchina committata — autorità = `cargo test` rieseguibile, ma è l'unica cifra di WP_SESSION_92 (judge=yes) senza .out.

## Q2 — dl59-join.out

Autosufficiente e leggibile SÌ: grade=ADVISORY in testa (r2), input per path, formule SLACK per NOME (r36-46), regola d'attribuzione dichiarata, colonne di validazione (thrSumMatch, d1/d2mispl), refutazione con esempio numerico, identity check, verdetto motivato. **MA formato MISTO**: le righe machine-truth (r7-9) stampano interi nudi, le derivate stampano virgole US (`8,455,362.5`, `-32,963,974.4`, `401,467.5`, `4,049,975,808`). Il tokenizer del corpus (`(\d[\d.]*\d|\d)`) spezza alle virgole: (a) le cifre assolute derivate NON sono citabili nei judge=yes (morso in sessione: il doc cita solo i ratio); (b) **peggio: le schegge ENTRANO nel corpus come token legali** («455», «963», «974.4», «808», «528»…) e possono legalizzare cifre estranee — superficie di fabbricazione viva a HEAD, contata dentro il budget 24329. Serve la regola di formato: SÌ.

## Q3 — token di grado

MEASURE90 ✓ ovunque; NEXT_SESSION:46 ✓ («ADVISORY-RIPUBBLICATA»). **NON scala**: `sessions/WP_SESSION_91.md:45` (judge=yes!) cita «b_peak(mediana) = 20.289.946 B» SENZA token; `gaps/GAP_TREND.md:101` idem (ADVISORY solo sulla somma); riga WP-92 di GAP_TREND e WP_SESSION_92 §6 citano il verdetto dl59 (grade=ADVISORY nel .out) senza token. Il ✓ su «A-BG64 ovunque» (p4) è un ✓ MANUALE mai sweeppato a macchina.

## Q4 — lezioni con evento-prova

L1 ✓ (Scoperta 2: tether vacuo, f9bf110, «PRIMO morso di T17»). L2 ✓ (A-DL-59: naive 1,887 vs fisico 0,090, .out committato). **L3 NO**: «un if bash non distingueva un FAIL da un pass» è un controfattuale argomentato — nessun morso/commit/dente nominato in sessione lo prova.

## Emendamenti

- **A-BG66**: formato degli out macchina: token numerici SOLO cifre ASCII nude (punto decimale ammesso), separatori di migliaia US BANDITI; formato misto nello stesso .out bandito; forma umana in campo companion.
- **A-BG67**: tokenizer corpus fail-closed sulle sequenze `\d,\d` (fusione o REFUSE, estensione A-SK-76 al lato corpus) — mai schegge nel corpus; sanatoria dl59-join.out.
- **A-BG68**: A-BG62 col discriminatore per NOME (half-up ovunque; decimale citato SOLO al tie esatto .5 — oggi 501.347,7→501.348 e 6.345,5 convivono senza regola che li distingua).
- **A-BG69**: sanatoria token di grado su WP_SESSION_91:45 + GAP_TREND:101/102 + WP_SESSION_92 §6; norma estesa alle lane ADVISORY nuove.
- **A-BG70**: L3 → nominare l'evento-prova o declassarla da lezione a razionale.

## Kill-switch

- **KS-BG-94-1**: un .out sorgente-corpus contenente `\d,\d` = gate FAIL finché A-BG67 non morde.
- **KS-BG-94-2**: nessuna condizione di consumabilità spuntata ✓ senza sweep macchina per NOME delle citazioni.

## Refutazioni capitali

1. **SÌ** — «A-BG64 ovunque ✓» (condizione di consumabilità WP-93 n.2) è FALSA per NOME su 2 siti vivi (WP_SESSION_91:45, GAP_TREND:101).
2. **SÌ** — il corpus del gate v3 conia token-scheggia dalle virgole di dl59-join.out: superficie di fabbricazione dentro il perimetro che dovrebbe chiuderla.

---

## NOTE DI TEAM (fase 2)

# TEAM CIFRE — relazione di fase 2 (Concilio WP-94)

Sedie: Klabnik (v-3), Hejlsberg (v-4), Gregg (v-9). I verbali individuali restano
la fonte VINCOLANTE; questa relazione non li riscrive.

## CONVERGENZE

1. **Denti dichiarati armati che non mordono** (Klabnik F-K1..F-K6; Hejlsberg
   f1..f6 + matcher + checker vuoto). Dodici forge ESEGUITE, tutte benedette.
   Il difetto non è nei singoli emendamenti: è fail-open per costruzione.
2. **L'identità dell'artefatto presa da un nome scelto dal CHIAMANTE** — Klabnik:
   A-SK-78 firma `$0`, aggirato con `bash -c` (rc=0, `judge_sha` corretto in
   stampa); Hejlsberg: lo scope dei denti viene dal BASENAME di `OUT`
   (`b91`→`battery-88pre.out` salta l'intera disciplina attempts).
   KS-SK-94-1 e KS-AH-94-2 sono la stessa legge vista da due lati.
3. **Match di PREFISSO/SOTTOSTRINGA al posto del campo** — Hejlsberg f3
   (`campaign_sha=<start>deadbeef`), f4 (`judge_sha` 16hex non ancorati),
   matcher A-AH64 a sottostringhe; Klabnik F-K1 (prefisso decimale di `rev=` e
   di uno sha di commit legalizza una MISURA). Stessa classe che A-AH61 ha
   appena chiuso sul sha256: il checker ripete il peccato del gemello.
4. **Il tokenizer del corpus è una superficie, non una difesa** — Klabnik F-K2
   (colla a sinistra: `W16-20.999.999` invisibile); Gregg A-BG67 (le virgole US
   di `dl59-join.out` entrano come token-scheggia legali, dentro il budget
   24.329 che oggi ha headroom 0).
5. **✓ manuali mai sweeppati a macchina** — Gregg: `WP_SESSION_91:45` e
   `GAP_TREND:101` (judge=yes) citano `20.289.946 B` senza token di grado;
   Klabnik: 894 occorrenze ≥3 cifre fuori perimetro (`.MD` maiuscolo, ROOT
   README/COVERAGE/TODO, `WP_SESSION_75`). KS-BG-94-2 ⟂ A-SK-86.
6. **Asserzioni che non possono fallire** — T16 asserisce solo `rc==1` da
   `--all`, e la baseline è già 1 in ogni sessione di concilio (untracked);
   `chk.sh` VUOTO ⇒ rc=0 ⇒ ledger «conforme» da giudice vuoto.
7. **Denti per NOME** (A-SK-87, A-AH67, KS-BG-94-2): un dente vale solo se
   asserisce la riga FAIL che NOMINA il forge, previa misura della baseline.

## CONFLITTI (registrati, non appianati)

- **CF-1 — gravità della classe «nome dal chiamante».** Hejlsberg: Q1 **NON**
  capitale, emendabile con A-AH68. Klabnik: la stessa classe su `$0` è
  **capitale** (KS-SK-94-1). Compatibili nei fatti, incompatibili nella tassa.
  Posizione del team: la gravità la decide il bite-test, non l'emendabilità —
  ma il dissenso resta di Hejlsberg e va portato in plenaria.
- **CF-2 — cosa fare del token malformato.** A-SK-83 (Klabnik): il run adiacente
  a un identificatore è **RIFIUTATO, mai cancellato**. A-BG67 (Gregg): «fusione
  **o** REFUSE». La fusione è una terza semantica silenziosa: due regole di
  corpus con esiti diversi sullo stesso byte. Da unificare su REFUSE.
- **CF-3 — dove sta il pericolo pubblicato.** Klabnik lo mette FUORI
  `php-rust/` (i file che GitHub pubblica, 11+4 cifre mai giudicate); Gregg lo
  mette DENTRO (schegge vive nel corpus a HEAD). Cumulativi, non alternativi.

## PRIORITÀ PROPOSTE per S-93.0

1. **Tether reale + identità dal contenuto** (A-SK-82 `BASH_SOURCE[0]` con
   REFUSE se vuota o ≠ `$0`; A-AH68 `BATTERY_NAME` dalla riga PASS/campo
   `battery=`). È la falla di autorità più grave: oggi il giudice non sa né
   cosa gira né cosa giudica. Denti T23 + tabella scope.
2. **Ancoraggi di campo ovunque** (A-AH65, A-AH66, A-SK-85): campi parsati con
   `( |$)`, 16/64 hex esatti, `esito` come campo unico, matcher solo su
   `phase=verdict`, A-SK57 esteso ai campaign ledger.
3. **Wiring fail-closed + bite-test end-to-end** (A-AH67): sha(chk.sh)==blob
   HEAD, `--selftest` della COPIA, canary malformato alla prima campagna m91;
   T16 riscritto per NOME (A-SK-87).
4. **Perimetro vero** (A-SK-86): ogni `.md` del REPO, regex case-insensitive,
   le 22 esclusioni per NOME nel manifest. T27 = F-K5.
5. **Corpus**: A-SK-83 + A-BG66/A-BG67 unificati su REFUSE; sanatoria
   `dl59-join.out`; **ricomputo del budget 24.329** (headroom 0 non può
   restare un caso).
6. **A-SK-84** (istante): operandi prov dalla STESSA RIGA — chiude F-K3.
7. Sanatorie doc: A-BG69 (sweep a macchina), A-BG68, A-BG70; `$DSHA` nudo a
   r.322 sotto `set -u`; divergenza `requalify` verdict/supersede dichiarata.

## S-92.0 È CONSUMABILE?

**NO come contenuto verificato — SÌ solo in ADVISORY.** Motivazione a gambe:

- **Would-have-allowed, non forge avvenuto.** Tutte e dodici le forge sono
  girate in scratch fuori repo o su fixture; nessun commit; albero porcelain.
  Nessuna sedia porta evidenza di una cifra fabbricata nel pubblicato.
- **Ma una superficie di fabbricazione è VIVA a HEAD**: le schegge-virgola di
  `dl59-join.out`, contate dentro il budget con headroom 0 (KS-BG-94-1). Non è
  una cifra falsa: è il materiale per coniarla, dentro il perimetro che
  dovrebbe chiuderlo.
- **Verifica al byte parziale.** Gregg verifica al byte ~25 cifre contro le
  righe macchina (incluse `−45.875` col segno e il «+87 esclusioni» esatto al
  suo commit); ma 2 siti judge=yes sono muti sul grado e `56 unit / 39
  message-asserting` non ha `.out` committato. Klabnik: 894 token mai giudicati
  fuori perimetro. Nessuno ha verificato l'INSIEME.

**CONDIZIONI** (tutte falsificabili a macchina):
C1 sweep per NOME dei token di grado su tutto il repo → 0 siti muti.
C2 A-BG67 morde, `dl59-join.out` sanato, budget ricomputato.
C3 le 11+4 cifre di ROOT giudicate o esentate per NOME nel manifest.
C4 T16 riscritto, baseline misurata, ogni dente nomina il proprio forge.
C5 la cifra lsp_check ottiene un `.out` o è declassata ad ADVISORY.
C6 le 6 forge di Hejlsberg falliscono per nome sul checker emendato e il canary
m91 fa FAIL end-to-end.

---

# TEAM-MISURA — relazione di fase 2 (Concilio WP-94)

Relatore: team-misura. Fonti vincolanti: verbale-2-matsakis, verbale-6-pedersen, verbale-5-bak.
Gradi dichiarati dalle sedie: Matsakis **PASS-CONDIZIONATO, 1 refutazione capitale SÌ**;
Pedersen **PASS CON RISERVE, capitali NO**; Bak **CONFERMATO con emendamenti, capitali NO**.

## CONVERGENZE

1. **Stessa proposizione, due sedie**: l'implicazione «pre==post==0 ∧ arr_pre==arr_post ⇒
   window clean» (main.rs:273) è FALSA nel modello di memoria puro. Matsakis Q3 e Pedersen
   Q1(b) la refutano indipendentemente con controesempi distinti.
2. **Stessa causa**: nessun ordering compra *freshness*; la coherence read-read non impone di
   leggere l'ultimo valore. Il buco non è di ordering.
3. **Stessa protezione reale**: ciò che schermerebbe la finestra è il **protocollo** (driver
   sequenziale, unico client = la richiesta census che risponde prima di `dispatch`), non la
   coppia di contatori.
4. **Stesso statuto della coppia**: tripwire fail-closed NECESSARIA, non prova sufficiente
   (Matsakis) = declassata ad ADVISORY senza dichiarazione in-banda (Pedersen).
5. **Emendamenti gemelli**: A-MS-59 ≡ A-PP-70 sul testo del commento; KS-MS-94-1 ≡ KS-PP-94-1.
6. **Universo del testimone ristretto** ai soli DISPATCHED (Pedersen Q1c) — coerente con il
   cfg-split verificato sano da Matsakis Q2 sugli inc.
7. **Bak non tocca la coppia**: il suo perimetro (A-DL-59) è indipendente e non è intaccato
   dalla refutazione.

## CONFLITTI

- **C1 — grado.** Matsakis registra capitale (la *sufficienza formale dichiarata* cade);
  Pedersen no (nessuna riga misurata è falsa: si declassa un'implicazione). Composizione del
  team: **è la stessa refutazione**; la divergenza è su cosa conta come capitale (Matsakis
  giudica la dichiarazione, Pedersen l'esito misurato). Registriamo il grado più severo per il
  claim testuale, il più mite per i dati m9x già raccolti: **nessun dato va ritirato, il
  commento sì**.
- **C2 — SeqCst come via alternativa.** Pedersen offre «o si passa a SeqCst sui sei op»;
  Matsakis nega esplicitamente che qualunque rafforzamento compri freshness. **Composizione:
  SeqCst NON è sostituto della sequenzialità** — chiude solo il buco *extra* di Pedersen (load
  Relaxed di `census_arrivals_now`, worker_pool.rs:313). Resta igiene consigliata, non
  condizione.
- **C3 — asimmetria mecanica/grado.** Il controesempio di Pedersen è strettamente più forte
  (anche arr_post può mancare l'arrivo) ma porta il grado più mite. Da registrare come tale.

## GRADO DEL TESTIMONE ATTUATO

**ADVISORY.** Diventa **verdict-grade** solo con questo elenco CHIUSO, tutto in-banda e
leggibile a macchina dalla riga di fase:

1. Enforcement sequenziale del driver **dichiarato in-band** nella riga (non in un commento).
2. Trigger VOID **nominato** per dispatch-Err sul canale **mem-census** (A-MS-60/KS-MS-94-2):
   `rows==N` non esiste su quel canale, ARRIVALS resta gonfiata.
3. Universo del testimone dichiarato = sole richieste DISPATCHED (A-PP-70/Q1c).
4. Residuo dichiarato: dec dopo la send sulla oneshot, non dopo la write sul socket ⇒ free del
   body precedente dentro una finestra «pulita» (A-PP-69 → A-PP-73: O(1) alloc, taglia ∝ body,
   census esenti per costruzione).
5. Grammatica del ledger sana: sentinella su `$now` vuoto (A-PP-71) e `deferred=` dichiarato
   DERIVATO con etichetta propria per i denti non-gate (A-PP-72).

Fuori elenco (nessuno è condizione): SeqCst, rafforzamenti di ordering, doppio campo derivato.

## GRADO DEL JOIN A-DL-59

**CONSUMABILE come analisi, non come oracolo.** Ricomputo indipendente al byte su 20/20 run e
~260 valori, zero mismatch; identità vis≈bin-committed verificata per-run (dev max +0,172%
contro margine 3% del test committed-vs-used). Consumo ammesso: **refutazione dell'ipotesi
page-slack di Leijen** e collocazione dello slack DENTRO il visibile — coerenza interna,
semi-tautologica. Conclusione da declassare (A-BB73-1): «l'invisibile vive **fuori dai bin del
heap visitato**», non «ad arena/chunk»; abandoned segments, huge/OS-direct e metadata restano
indistinti, discriminazione al canale barrier A-DL-57/58. Drift dump#1↔pwork da dichiarare con
cifre (A-BB73-2: w16.r2 +1.048.576 B, w16.r3 +4.194.304 B; 18/20 identici).

## PRIORITÀ PER S-93.0

P1. Punti 1-2 dell'elenco chiuso (riga in-banda + trigger VOID mem-census): senza questi ogni Δ
di fase nasce ADVISORY.
P2. Punti 3-5 (dichiarazioni + sentinella `$now` + `deferred` derivato).
P3. A-BB73-1/2/3 nel testo di A-DL-59 (riformulazione, cifre del drift, identità come test).
P4. A-MS-61 (lsp_check: policy antenato-assente, deprecation al punto di scoperta, `lc`
ASCII-only, Err al posto di `unreachable!`) — fuori perimetro misura, da instradare al team
competente prima del wiring di fase 2.

---

# team-engine — relazione fase 2, Concilio WP-94

Relatore: sedia-engine. Fonti VINCOLANTI: verbale-8-stogov (checker LSP), verbale-7-leijen (canale), verbale-1-hoare (sigilli). Capitali: 2 (Stogov) + 2 (Leijen) + 0 (Hoare).

## CONVERGENZE

1. **Un solo filone è PRONTO a produrre: i sigilli.** Gate a HEAD e52a634 PASS, controesempi ESEGUITI, nessuna cifra cade. A-TH-68..72 sono lavoro interamente locale allo script, senza dipendenze dagli altri due.
2. **Il canale è BLOCCATO da due cose distinte, non una.** (a) A-DL-57 è refutato *com'è scritto*: heaps_total=1 rende il dente REFUSE potenzialmente scopato ⇒ barriera bloccata da PT-1 (A-DL-61). (b) KS-DL-94-1 blocca la LETTURA di malloc_huge current come massa viva finché non c'è il controllo di decremento. Sono blocchi su oggetti diversi: la barriera e la cifra.
3. **Il checker è BLOCCATO da sé stesso, non da altri.** q3b (benedice dove Zend fatala) e iterable-desugar (byte-divergenza su ogni messaggio con iterable) sono difetti del codice esistente, non lacune del wiring. Un wiring montato sopra installerebbe falsi-negativi di massa (ORM/hk) e messaggi byte-divergenti.
4. **I sigilli sono prerequisito del CANALE, non del checker.** KS-TH-94-1: nessuna cifra m9x con probe può citare i pin arm/dot-name come verdict-grade finché A-TH-68/69 non mordono ⇒ il join INV(W) è a valle dei sigilli. Il giudice del checker è l'ORACLE per fixture (+ gate corpus/ORM per NOME): le reti lessicali non entrano nella sua catena di prova. Il checker corre in PARALLELO.
5. **A-TH-70 precede ogni dente nuovo.** check_pin è fail-open su ~25 denti: ogni riga nominata nuova del canale (A-DL-62) nasce con una cattura numerica. Aggiungere denti prima della guardia moltiplica la vacuità presunta (KS-TH-94-2).
6. **Ordine INTERNO del canale: prima la MISURA sui raw esistenti, dopo gli strumenti** — ma la misura è ammessa solo appaiata al controllo di decremento (one-off, binario esistente, zero campagne). I termini huge sono già nei 30 raw dentro mi_arena_json: servono un parse section-aware e il controllo, non una campagna. Gli strumenti (righe nominate, mappa byte chunk-bin, jbuf) servono al JOIN, non al primo indiziato.
7. **Ordine INTERNO di A-DS51 fase 2: si corregge il checker PRIMA del wiring.** E ogni correzione nasce con fixture oracle-morsa + pin length-prefixed PRIMA del codice (KS-DS-94-1). A-DS65 (registry seed builtin+tentative) è prerequisito del gate ORM/hk (KS-DS-94-2), quindi del wiring, non delle correzioni.

## CONFLITTI / CONFINI DA DICHIARARE

- **C1 — retroattività di KS-TH-94-2.** Se le righe OK dei denti non guardati sono VOID, cadono anche i PASS storici del gate-lever (WP-79..92)? Posizione del team: vacuità presunta **prospettiva** (nessuna cifra NUOVA li cita), verdetti d'epoca ancorati al loro epoch — precedente M84/M85 (WP-91). NON decidibile qui: al plenum / team-cifre via ledger.
- **C2 — proprietà del probe on-thread.** PT-1 (A-DL-61) e A-DL-52 census on-thread (P1 team-misura) insistono sullo stesso territorio. Confine proposto: PT-1 è del CANALE (è un pre-test di pluralità, non un censimento); il suo esito decide il piano di split di team-misura. Da concordare con team-misura prima di S-93.0.
- **C3 — riverifica VDL24/canary.** Leijen la chiede per NOME solo se la sorgente era il piano tls-dump. È atto di LEDGER (supersessione), non di engine: al team-cifre.
- **C4 — il one-off huge non è una cifra.** Il controllo di decremento produce meccanismo, non verdetto: ammesso prima che i sigilli mordano, purché nessun numero esca etichettato verdict-grade.

## ORDINE PROPOSTO PER S-93.0 (prerequisiti per NOME)

- **E1** A-TH-70 (guardia numericità universale + meta-dente su `-ne`/`-gt`) + A-TH-72 (emendare «num_or_void su OGNI cattura» → «12 catture per NOME», supersessione da ledger). Prereq: nessuno. *Sblocca ogni dente successivo.*
- **E2** (parallelo a E1) Controllo positivo decremento `malloc_huge` current, binario esistente. Prereq: nessuno; vincolo C4.
- **E3** A-TH-68 + A-TH-69 con decoy ESEGUITI stesso-commit; A-TH-71 in coda. Prereq: E1. *Estingue KS-TH-94-1.*
- **E4** Termine T-HUGE misurato sui 30 raw m90, parse SECTION-AWARE (KS-DL-94-3). Prereq: E2 (altrimenti VOID).
- **E5** A-DL-61 PT-1 in-band; poi A-DL-62 righe nominate + mappa byte chunk-bin, A-DL-63 controllo page_committed, A-DL-64 jbuf. Prereq: E1 (guardia), E4 (il termine nominato). Barriera A-DL-57/58 solo dopo PT-1 (KS-DL-94-2).
- **E6** Join INV(W)=T-HUGE+T-CHUNK+T-EAGER+residuo su W∈{4,8,12,16}, grammar v2. Prereq: E3, E5.
- **D1** (traccia parallela, giudice = oracle) A-DS61 iterable-desugar + A-DS62 ctor_proto da ctor abstract. Prereq: fixture oracle-morsa + pin length-prefixed PRIMA del codice.
- **D2** A-DS63 (set-hook ereditato, set-param untyped x2) + A-DS64 (final-const da iface, abstract-count sing/plur). Prereq: D1.
- **D3** A-DS65 registry seed (grafo builtin + tentative). Prereq: D2.
- **D4** Wiring fase 2 + gate ORM 3E/13F e hk 1665. Prereq: D3 (KS-DS-94-2) — mai prima.

---


## ⚖️ SINTESI DI CONVERGENZA (compilata dalle ricevute fase 1+2 + estrazioni mirate; verbali = fonte vincolante)

**Verdetto complessivo: 9 sedie CON EMENDAMENTI — nessuna opposizione;
UN dissenso di gravità registrato (CF-1) e due confini cumulativi
(CF-3). UNDICI refutazioni capitali in cinque classi, tutte morse a
macchina o dall'oracle vivo.**

### Refutazioni capitali

1. **🔴 QUATTRO forge NUOVI passati dal gate cifre v3 (Klabnik, 6 forge
   eseguiti col negativo)**: (a) **A-SK-78 AGGIRATO** — `bash -c … 
   gate-measure-cifre.sh --all` con giudice patchato stampa PASS rc=0
   FIRMATO col judge_sha (il tether legge `$0`, che il chiamante
   sceglie: si è chiuso il portone e lasciata la finestra);
   (b) A-SK-76 **speculare** — la colla a SINISTRA (`b_peak-20.999.999
   B`) passa; (c) A-SK-81 aggirato con la **stessa chiave a istanti
   diversi** (50/100 target MiB-tondi mintabili); (d) perimetro: `.MD`
   maiuscolo evade, e 22 .md di root (894 cifre) restano fuori. **T16 è
   VACUO** (non nomina il proprio forge). → A-SK-82..87, KS-SK-94-1..5.
2. **🔴 Il checker campaign-v2 è fail-open (Hejlsberg, 6 forge)**: riga
   di tipo IGNOTO ammessa, authorize senza `gG=`, campaign_sha/judge_sha
   accettati a PREFISSO, requalify senza `->`, `esito=PASS` dentro il
   reason; **checker su file vuoto ⇒ rc=0**; il matcher A-AH64 si
   NUTRE dei forge (admitted=1); lo scope dell'hoist A-AH61 dipende dal
   **basename scelto dal chiamante**; `$DSHA` nudo sotto `set -u` sul
   percorso equivalenza. → A-AH65..68, KS-AH-94-1..3.
3. **🔴 Il testimone a coppia non prova la finestra
   (Matsakis capitale + Pedersen declassamento — CF registrato)**:
   esiste un'esecuzione **stale-coerente** in cui la finestra è SPORCA e
   il verdetto pulito (load stantii ammessi dagli ordering scelti; arr
   Relaxed). La coppia è condizione NECESSARIA, non sufficiente: la
   freschezza non si compra con l'ordering, serve l'enforcement
   sequenziale del driver **in banda**. → A-MS-59..61, A-PP-70..73,
   KS-MS-94-1/2, KS-PP-94-1..3.
4. **🔴 A-DL-57 assume il proprio esito (Leijen, ×2)**: `heaps_total=1`
   nei raw m90 ⇒ il probe on-thread NON crea pluralità di heap per
   costruzione — serve **PT-1** (prova di pluralità: due thread, probe,
   `mi_heap_of` con ptr DISTINTI) PRIMA di ogni barriera. In compenso il
   termine della PENDENZA è NOMINATO nei raw esistenti: **malloc_huge =
   39.911.424 B × W esatto, count 6×W**, con pwork_commit ≡ arena
   committed al byte ⇒ T-HUGE primo indiziato, poi T-CHUNK e T-EAGER.
   → A-DL-61..64, KS-DL-94-1..3.
5. **🔴 Il checker LSP è under-fatal su due classi (Stogov, 33 probe
   oracle)**: `iterable` **desugara** a `Traversable` (named, posizione
   dichiarata) + `array` (bitmask) — mai «?iterable»; e il
   **ctor-prototype nasce ANCHE da un ctor abstract e PROPAGA** (q3b:
   Zend fatala citando A, il checker è CLEAN). Under-fatal anche
   set-hook ereditato, final-const via interfaccia, set-param untyped.
   Bitmask-order, DNF, static/parent: CONFERMATI al byte. → A-DS61..65,
   KS-DS-94-1/2.
6. **🔴 Due claim di S-92.0 sono falsi come scritti (Gregg)**: «token di
   grado ovunque» è FALSO (WP_SESSION_91:45 e GAP_TREND:101 citano
   b_peak(mediana) MUTE); e le **virgole US di dl59-join.out entrano nel
   corpus come SCHEGGE** («455», «963»…) che legalizzano cifre
   estranee — materiale per coniare, dentro il perimetro che dovrebbe
   chiuderlo. Il resto del censimento regge AL BYTE. → A-BG66..70,
   KS-BG-94-1/2.

### Convergenze forti (dai team)

- **team-cifre (7 convergenze, 3 conflitti)**: la classe comune è
  **«l'identità presa da un dato che il chiamante controlla»** — `$0`
  per il giudice, il basename per lo scope del dente: oggi il giudice
  non sa né cosa gira né cosa giudica. Cura: `BASH_SOURCE[0]` +
  identità dal CONTENUTO (A-SK-82/A-AH68). Perimetro e corpus vanno
  chiusi da entrambe le parti (CF-3: le cifre non giudicate FUORI e le
  schegge vive DENTRO sono cumulative). CF-2 da unificare su REFUSE (mai
  fusione silenziosa).
- **team-misura**: Matsakis e Pedersen refutano la STESSA implicazione
  con gradi diversi; SeqCst è NEGATO come via alternativa. Il testimone
  attuato è **ADVISORY** con un elenco CHIUSO di 5 condizioni per il
  verdict-grade (enforcement driver in-band · trigger VOID dispatch-Err
  su mem-census · universo = sole richieste dispatched · residuo body
  dichiarato · grammatica ledger sana). A-DL-59 è **consumabile come
  analisi, non come oracolo**; chiusa declassata a «fuori dai BIN
  dell'heap visitato».
- **team-engine**: PRONTO = sigilli; il canale è bloccato da PT-1 e dal
  controllo-decremento (blocchi DISTINTI); il checker è bloccato da sé
  (si corregge PRIMA del wiring). I sigilli sono prerequisito del
  CANALE, non del checker (il cui giudice è l'oracle). Ordine interno
  del canale: prima il termine T-HUGE sui raw ESISTENTI, gli strumenti
  dopo.

### Delibera di consumabilità (team-cifre)

**S-92.0 è consumabile SOLO in ADVISORY.** Motivazione a tre gambe:
(a) tutte le forge sono would-have-allowed — nessun commit, albero
porcelain, nessuna cifra fabbricata nel pubblicato; (b) MA una
superficie di fabbricazione è **VIVA a HEAD** (schegge-virgola nel
corpus, headroom budget 0); (c) la verifica al byte è **parziale** (~25
cifre censite; 2 siti muti sul grado; 894 token mai giudicati fuori
perimetro; nessuno ha verificato l'INSIEME). **Condizioni C1-C6**:
C1 sweep per NOME dei token di grado → 0 siti muti · C2 A-BG67 morde,
dl59-join.out sanato, budget RICOMPUTATO · C3 le cifre di ROOT
giudicate o esentate per NOME · C4 T16 riscritto con baseline misurata,
ogni dente nomina il proprio forge · C5 la cifra del checker ottiene un
`.out` o è ADVISORY · C6 le 6 forge Hejlsberg falliscono per nome sul
checker emendato + canary m91 FAIL end-to-end.

### Ordine vincolante di apertura S-93.0 (non rinegoziare)

1. **Identità, di nuovo — il tether che il chiamante non sceglie**:
   A-SK-82 (`BASH_SOURCE[0]`, REFUSE se vuoto o ≠ `$0`) + A-AH68
   (BATTERY_NAME dal CONTENUTO ancorato) → dente T23 + tabella di scope.
   È la falla di autorità massima: blocca ogni PASS verdict-grade.
2. **Ancoraggi di campo ovunque**: A-AH65 (tipi di riga CHIUSI, campi
   ancorati `( |$)`, 16/64 hex esatti) + A-AH66 (matcher solo su
   `phase=verdict`, A-SK57 esteso ai campaign) + A-SK-85 (identità ≠
   misura) + A-SK-84 (operandi prov dalla STESSA RIGA).
3. **Wiring fail-closed + bite-test end-to-end**: A-AH67 (sha(chk)==blob
   HEAD + selftest della COPIA + canary m91 malformato) + A-SK-87 (ogni
   dente `--all` asserisce la RIGA FAIL) + T16 riscritto (C4).
4. **Perimetro e corpus**: A-SK-86 (ogni `.md` del REPO,
   case-insensitive, 22 esclusioni per NOME) + A-SK-83 ≡ A-BG66/67
   unificati su **REFUSE** (CF-2) + sanatoria dl59-join.out + **budget
   ricomputato** (headroom 0 non può restare un caso).
5. **Sigilli E1→E3**: A-TH-70 (guardia numericità UNIVERSALE +
   meta-dente su `-ne`/`-gt`) + A-TH-72 (emendare «OGNI cattura» →
   «12 catture per NOME») → A-TH-68/69 con decoy eseguiti stesso-commit
   → A-TH-71. Estingue KS-TH-94-1.
6. **Misura**: sanatorie doc A-BG68/69/70 + A-BB73-1/2/3 (chiusa
   declassata, drift dump#1 con cifre, identità come test
   committed-vs-used) + le 5 condizioni del testimone (A-MS-59/60/61,
   A-PP-70..73) — il Δ di fase resta ADVISORY finché non sono tutte
   in banda.
7. **Canale (E2→E6) e checker (D1→D4), tracce parallele**: E2 controllo
   positivo del decremento malloc_huge → E4 termine T-HUGE sui 30 raw
   (parse SECTION-AWARE) → E5 **PT-1 pluralità heap** poi A-DL-62/63/64
   (barriera SOLO dopo PT-1) → E6 join INV(W). In parallelo D1 A-DS61
   (iterable-desugar) + A-DS62 (ctor_proto da ctor abstract) → D2
   A-DS63/64 → D3 A-DS65 registry seed → **D4 wiring fase 2 + gate
   ORM/hk MAI prima di D3** (KS-DS-94-2). Ogni emendamento del checker
   nasce con fixture oracle-morsa e pin PRIMA del codice (KS-DS-94-1).

### Kill-switch nuovi consolidati (attivi da subito)

KS-TH-94-1/2/3 · KS-MS-94-1/2 · KS-SK-94-1..5 · KS-AH-94-1..3 ·
KS-BB-94-* (A-BB73) · KS-PP-94-1/2/3 · KS-DL-94-1..3 · KS-DS-94-1/2 ·
KS-BG-94-1/2 — 22 nuovi. Ereditati ATTIVI: WP-93 (23) + WP-92 (22) +
WP-91 (27) + WP-88..90. **KS-SK-91-1 resta NON sollevabile.**
