# COUNCIL_WP92_REVIEWS.md — Concilio a 9 sedie su S-90.0 (attribuzione per fase, gate cifre v91, fixture v2) e sul programma S-91.0

Protocollo A DUE FASI (regola utente 2026-08-02): fase 1 = 9 verbali
INDIPENDENTI (nessuna sedia vede le altre); fase 2 = 4 team tematici con
relatore che legge SOLO i verbali del proprio team. I verbali individuali
sono la fonte VINCOLANTE; le note di team alimentano la sintesi.
Token-lean: nel contesto della sessione sono passate SOLO ricevute ≤80
parole; questo file è stato assemblato per concatenazione.

Composizione team (decisa sui punti di S-90.0):
- **team-cifre** — Klabnik (3) · Hejlsberg (4) · Gregg (9)
- **team-misura** — Bak (5) · Leijen (7) · Pedersen (6)
- **team-sigilli** — Hoare (1) · Matsakis (2)
- **team-engine** — Stogov (8), con integrazione dei vincoli di evidenza

Indice: VERBALI INTEGRALI (fase 1) → NOTE DI TEAM (fase 2) → SINTESI DI
CONVERGENZA (in coda, = ordine vincolante di apertura S-91.0).

---

## VERBALI INTEGRALI (fase 1)

# Verbale sedia 1 — Tony Hoare (Concilio WP-92)

Perimetro: sigilli lessicali v8, pin A-MS47/A-MS48, finestra noprobe A-TH56, doc TLS A-TH54, fail-fast.

## VERDETTO: CONCORDO CON EMENDAMENTI

## Q1 — Grafie che ancora sfuggono a TH49RE_V8/TH49ML_PROG
Verificato contro il testo di produzione (gate-lever-pins.sh 1239-1253):
1. **`use crate::vm::CachedUnit as CU;` MONORIGA senza graffe** sfugge a TUTTO: il ramo regex use richiede `[{]`; l'awk fa `next` esplicito sulle use monoriga; la rete alias A-TH42 richiede `=`. Ironia: la grafia più naturale è l'unica scoperta (la variante multiriga e quella a gruppo sono coperte).
2. **Percorso dot-EOL senza tolleranza commenti**: lo skip commento/blank esiste solo sul ramo pend (`.nome` a EOL); `u.` ⏎ `// c` ⏎ `vm_gate(x)` azzera `dot` sulla riga commento e passa.
3. **Split a TRE righe** `u.` ⏎ `vm_gate` ⏎ `(x)`: nessuna riga contiene una grafia riconosciuta da regex o awk (la regola dot esige nome+paren sulla stessa riga).
4. **Commento `//` in coda al nome**: `u.production_gate // c` ⏎ `(x)` — la regola pend esige il nome a fine riga; nessun `(` sulla riga ⇒ sfugge anche ai pin `.production_gate(`.
5. **`::` spaziato in posizione di TIPO** (`fn f(x: vm :: CachedUnit)`, return type, turbofish): tutti i rami `::`-spaziati sono ancorati a `=`. Severità minore (non è mint diretto), ma il canale d'alias per firma esiste.

## Q2 — Buchi nei pin A-MS47 (==2) e A-MS48
Confermati i 2 siti legali (worker_pool.rs 385, 520). Ma:
1. **A-MS47 non è un census**: `narm` conta solo `let <nome> = ProbeWindow::arm()`. `let mut w =`, `let w: ProbeWindow =`, `let (w,_) =`, `drop(ProbeWindow::arm())`, arm in posizione d'espressione — tutte invisibili a `narm` E a `nsil`. Un TERZO sito così scritto passa con narm==2/nsil==0. Il pin morde sulla conversione dei 2 siti noti (2→1 FAIL), non sull'aggiunta.
2. **A-MS48, quinta grafia**: `CENSUS_PROBE_ACTIVE.with(Cell::set)` / `with(|f| Cell::set(f, true))` (point-free/UFCS-Cell) sfugge alle QUATTRO reti (anybind esige `|b| b.set`; UFCS esige `LocalKey::`). Anche il corpo chiuso a graffe `|f| { f.set(true) }` sfugge ad anybind: una scrittura NUOVA così spellata è invisibile.

## Q3 — Elusioni della finestra noprobe (A-TH56)
1. `npdef` conta solo `^probe_in()` a colonna 0: una **ridefinizione indentata** ` probe_in() { return 1; }` o in forma **`function probe_in {`** dopo il decoy passa npdef==1, l'awk estrae il decoy (form-pin OK), bash usa l'ultima definizione svuotata.
2. Il form-pin è di FORMA, non di raggiungibilità: `false && strings -- "$1" | grep -q '<payload>'` soddisfa nform==1 a rilevazione svuotata. Residuo dichiarabile (oltre la portata lessicale), ma va DETTO.

## Q4 — Doc TLS A-TH54 (vm/mod.rs 16534-16555)
La dicotomia "destructor già eseguito / chiave ancora viva" **omette il terzo stato**: chiave RefCell MAI inizializzata su quel thread ⇒ primo accesso durante il teardown ⇒ std NON garantisce (init pigra con destructor possibilmente mai eseguito, o panic, per piattaforma). KH91-1 sopravvive, ma l'enumerazione è incompleta. Inoltre il percorso silente non è solo "senza fail-fast": l'emissione uc_log finisce in un buffer che muore senza flush — è un canale di **scrittura persa**, non solo di mancato panic.

## Emendamenti
- **A-TH-57** "Census totale arm": aggiungere `ntot = grep -c 'ProbeWindow::arm'` su righe di codice, pin ==2 (chiude Q2.1).
- **A-TH-58** "Use monoriga con as": ramo regex `use [^;{]*(CachedUnit|VmGate) as ` (chiude Q1.1).
- **A-TH-59** "Dot-path tollerante + tre righe": estendere TH49ML_PROG con skip commenti sul ramo dot e stato nome-pendente (Q1.2-4).
- **A-MS-emend** (per il team sigilli): quinta rete point-free/graffe su CENSUS_PROBE_ACTIVE (Q2.2).
- **A-TH-60** "Definizioni probe_in a census": `grep -cE '^[[:space:]]*(function[[:space:]]+)?probe_in'` ==1 (Q3.1).
- **A-TH-61** "Doc A-TH54 terzo stato": aggiungere il caso never-initialized-at-teardown e la scrittura persa (Q4).

## Kill-switch
- **KS-TH-92-1**: se il census totale `ProbeWindow::arm` (righe codice) ≠ narm, ogni figura di campagna con probe è VOID.
- **KS-TH-92-2**: se noprobe contiene una seconda definizione di `probe_in` in QUALSIASI grafia (indentata/function-form), ogni VERDICT che vi si appoggia è VOID.

## Refutazioni capitali
Nessuna: i buchi trovati sono nelle guardie in avanti, non nelle cifre di S-90.0 (i 2 siti reali sono conformi alle grafie contate; nessuna figura emessa attraversa i buchi). Le cifre restano in piedi; i denti vanno affilati PRIMA di S-91.0.

— T. Hoare, sedia 1, WP-92

---

# Verbale sedia 2 — Niko Matsakis (Concilio WP-92)

Perimetro: ownership/aliasing/borrow, soundness visitor census, probe discipline.
Fonti: memcensus.rs (A-MS50 blocco theap in `phys_window_dump` r.1063-1209, `peak_census_dump` r.232, `mi_bin_thread_rows` r.1425), worker_pool.rs (ProbeWindow r.74-114, teardown probe r.386-412, ramo `/__census_self` r.518-543).

## VERDETTO: CONCORDO CON EMENDAMENTI

## Q1 — Il reserve_exact pre-visita basta davvero?
Per l'ALIASING sì: guardia `len==capacity` + reserve pre-visita ⇒ push mai riallocante; il `&mut` in `theap_heap_cb` è scopato e cade prima della visita annidata (r.1152); `acc` viene rifruito solo post-visita (r.1171). Sotto Stacked/Tree Borrows regge. Ma la LETTERA ha due denti scoperti:
1. **La guardia confronta `capacity()`, non `MAX_THEAPS`** (r.1147). `reserve_exact` può concedere PIÙ di 64 slot (arrotondamento dell'allocatore): il "declared cap 64" è in realtà capacity-dipendente — `heaps_total` può superare 64 con `heap_overflow=0`. Il cap dichiarato in-band non è il cap effettivo.
2. **`TheapAgg::zeroed()` è un temporaneo di ~3,9 KB (N_BINS=192) costruito per valore sullo stack di un callback `extern "C"`** sotto i frame C di `mi_subproc_visit_heaps`. Non è UB e non è alloc, ma "no-alloc" tace su "stack headroom sotto iterazione C": rustc non garantisce costruzione in-place. Il read-back `RUST_MIN_STACK` esiste (r.1047) ma nessun pin lo lega a questo blocco.
3. **Identità heap = `last_mut()`** (r.1116): corretta SOLO se visita heap e visita aree sono strettamente sequenziali, mai interlacciate — contratto C assunto, mai dichiarato come pin. E il puntatore heap `_h` viene scartato: le righe `mi_theap_pages` non portano `heap=<ptr>`, quindi il join col dente anti-duplicato di `mi_bin_thr_sum` (r.1469) è IMPOSSIBILE — rilevante per la collision VSELF (tema 4).

## Q2 — Il ramo census_self rispetta la disciplina strumento?
Sì nella forma: ProbeWindow + catch_unwind + drop + abort (r.521-535), speculare al teardown (r.386-412). Due buchi:
1. **I due messaggi di abort sono STRINGHE IDENTICHE** ("A-MS31; instrument, not worker"): un abort non è attribuibile al SITO (in-request vs teardown). Forensics di campagna abortita = cieca sul sito.
2. **`req_t0` avvolge anche il ramo census_self** (r.507-552): con `PHPR_REQ_NS=1` un hit a `/__census_self` inietta un campione fasullo in REQ_NS — inquinamento dei per-request ns non dichiarato.
Nota dichiarata altrove (A-BB55): probe Box + BinTab::boxed() allocano sull'heap misurato PRIMA della visita — perturbazione auto-misura già a verbale, non la riapro.

## Q3 — Il secondo sito arm() indebolisce A-MS41?
La cintura A-MS41 conta le grafie `.set(`, non i call-site: NON scatta, e formalmente regge. Ma la semantica si è indebolita: `probe_active=true` non identifica più UN solo strumento; e `arm()` non è nesting-safe (bool, non contatore): un terzo sito futuro annidato farebbe cadere il flag della finestra esterna al Drop interno — falso negativo silenzioso. Oggi irraggiungibile (census_self dentro worker_loop, teardown dopo), ma è invariante di sequenza non presidiato da alcun dente.

## Q4 — Aliasing residuo nelle callback?
Nessuna violazione trovata: provenance di `arg` valida (raw derivato dal `&mut` al call-site, ri-derivazioni disgiunte nel tempo, `drop(probe)` post-visita in mi_bin_thread_rows). Il residuo è di CONTRATTO, non di borrow: sequenzialità C assunta (Q1.3).

## Emendamenti
- **A-MS-51 "cap effettivo = MAX_THEAPS"**: guardia su `len()==MAX_THEAPS` (non capacity) o `heaps: Box<[TheapAgg]>` a lunghezza fissa; il cap dichiarato deve essere quello che morde.
- **A-MS-52 "sito nell'abort"**: tag di sito distinto nei due eprintln di abort e nella riga del panic-hook (arm porta un `&'static str`).
- **A-MS-53 "heap ptr in-band nel theap census"**: emettere `heap=<ptr>` nelle righe `mi_theap_pages` per il join anti-collisione con `mi_bin_thr_sum` (serve a VSELF iter-3).
- **A-MS-54 "arm nesting-tooth"**: `debug_assert!(!flag)` (o contatore) in `arm()` — l'invariante "mai annidato" deve mordere, non essere raccontato.
- **A-MS-55 "REQ_NS esclude census_self"**: nessun campione ns per hit `/__census_self`.

## Kill-switch
- **KS-MS-92-1**: run con `heaps_total>MAX_THEAPS` e `heap_overflow=0` = census VOID (cap dichiarato non mordente).
- **KS-MS-92-2**: abort di probe senza tag di sito nel ledger = attribuzione sito-livello VOID per quella campagna.
- **KS-MS-92-3**: qualunque nuovo sito arm() senza dente anti-nesting committato nella stessa delibera = attribuzione VOID.

## Refutazioni capitali
Nessuna: le cifre S-90.0 sopravvivono. Ma la lettera "declared cap 64" e l'attribuzione per-sito degli abort sono refutate come dichiarazioni (emendate sopra).

---

# Verbale sedia 3 — Steve Klabnik (Concilio WP-92)

**VERDETTO: CON EMENDAMENTI — refutazioni capitali sul gate cifre.**
Quattro forge REALI costruiti e fatti passare dal gate vivo (repo ripristinato,
`git status` pulito; sandbox `zz-attack-tmp` rimossa).

## Q1 — grandfathering `legacy_frozen` (A-SK60): **FORGE LANDED**
Il codice concede la semantica pre-WP-91 (evaluator X−Y libero) a *qualunque*
doc il cui blob coincida con lo sha di una riga di manifest. "Doc STORICO
CONGELATO" è un **commento**, non un predicato: nessun test di committed-ness,
di data, di appartenenza al set M84-M88. Inoltre il manifest è letto dal
**working tree** (`open "$here/gate-cifre-manifest.tsv"`), mentre il corpus è
letto da HEAD (A-SK55): input non autenticato.
Morso: doc creato 10 secondi prima + **una riga di manifest mai committata** →
`PASS` con `b_base = 19.600.000 B [derivata: 20594560 − 994560]`. Ripetuto sul
manifest REALE (poi ripristinato): stesso PASS. Misura fresca: l'evaluator
resuscitato copre oggi **25,21 %** della finestra [18,6M–20,6M] (era ~11,5 % al
WP-91: il buco è **cresciuto**).

## Q2 — provenance `prov N@path:riga` (A-SK60): **CAPITALE**
L'evaluator libero non è stato abolito: è stato **annotato**. Operandi
ammessi: qualsiasi cifra su qualsiasi riga di qualsiasi blob a HEAD —
**36.573** operandi indirizzabili, contro i 24.042 del corpus. Chiusura delle
differenze nella stessa finestra: **46,25 %** — *più larga* di quella che
A-SK60 doveva chiudere. Tre difetti di specifica, tutti morsi dal vivo:
1. **nessuna commensurabilità**: i due operandi possono venire da campagne
   diverse. Live PASS: `19.600.000 = 23000000@…axum.83c…vmmap.V1:1963 −
   3400000@…axum.82c…vmmap.V1:1964` (il forge esatto che T5 dichiara ucciso);
2. **A-SK61 non si applica al resolver**: le righe citate sono *address range*
   `b22c00000-b23000000` — i frammenti che A-SK61 toglie dal corpus rientrano
   dalla porta della provenance;
3. **l'operatore non è verificato**: `[derivata: prov X@p:1 diviso Y@p:2]` passa
   e stampa "provenance-verified", perché il test `/−|-/` è soddisfatto dal
   trattino dentro `wp89-harness`. Il gate verifica un'operazione diversa da
   quella scritta.

## Q3 — cache, budget, identità del giudice
`--cache/--nonce` (A-SK65) sposta il veleno da env ad **argv**, ugualmente non
autenticato per ogni invocazione diretta: il nonce lo sceglie il chiamante.
Morso a cardinalità invariante (tolgo un token, aggiungo il mio):
`PASS` su doc con `777444111 B` fabbricato, `corpus cardinality=24042
budget=24042`. Budget e manifest letti dal working tree; il gate stesso non
stampa il proprio sha: un giudice patchato produce un PASS indistinguibile.

## Q4 — perimetro e citazioni gG
`--all` cerca a ritroso solo `wp*-harness/MEASURE*_RESULTS.md`. Le cifre che
arrivano davvero all'umano e alla sessione dopo sono fuori: NEXT_SESSION (20
cifre) + `sessions/WP_SESSION_90.md` (23) = **43 non giudicate contro 28
giudicate** in MEASURE90. A-SK66 evaso due volte: riga «verdict89.a1.g1.out
NON e superseded» passa (parola-chiave, non prova) e la citazione **senza
`.out`** non è nemmeno vista.

## Emendamenti
- **A-SK-67** manifest, budget e giudice letti da HEAD; PASS stampa
  judge_sha+manifest_sha+budget_sha; working≠HEAD ⇒ FAIL.
- **A-SK-68** `legacy_frozen` abolito: i 5 doc storici o si rigiudicano, o
  `judge=no` con verdetto d'epoca citato. Nessuna riga nuova può concederlo.
- **A-SK-69** prov: operandi dallo **stesso file**, operatore parsato e
  verificato con i path mascherati, strip A-SK61 applicato alla riga citata,
  righe address-range rifiutate.
- **A-SK-70** cache abolita (un solo processo perl per `--all`) o rifiutata se
  non creata dal parent.
- **A-SK-71** perimetro = ogni .md committato che pubblica cifre di sessione
  (NEXT_SESSION, sessions/, design, verbali), riga di manifest obbligatoria.
- **A-SK-72** supersessione provata da riga di ledger committata; regex di
  citazione con `.out` opzionale.
- **A-SK-73** il budget A-SK61 governi anche il pool prov (36.573), altrimenti
  misura la superficie sbagliata.

## Kill-switch
- **KS-SK-92-1** PASS di gate/manifest/budget con blob ≠ HEAD ⇒ VOID.
- **KS-SK-92-2** `prov` con operandi di file/campagne diverse ⇒ non verdict-grade.
- **KS-SK-92-3** cifra pubblicata fuori perimetro manifest ⇒ non verdict-grade,
  anche se il MEASURE passa.
- **KS-SK-92-4** PASS con `--cache/--nonce` forniti dall'invocante ⇒ VOID.

---

# Verbale sedia 4 — Hejlsberg (Concilio WP-92, su S-90.0)
Perimetro: catene di evidenza (attempts/stamps/matrix/judge), identità toolchain.

## VERDETTO: CONCORDO CON EMENDAMENTI

Tutte le verifiche sono state eseguite A MACCHINA su HEAD=dd6cdce.

## Q1 — Le judge_sha g1/g2 risolvono a blob committati? SÌ (verificato)
- g1: `git show 5d3ec7b:php-rust/wp90-harness/verdict90.sh | shasum -256 | cut -c1-16`
  = `1b1e9e96f5019bc0` == judge_sha della riga g1. ✓
- g2: blob a `564e7ac` = `97a8eff0d9783ee4` == judge_sha g2 == blob a HEAD. ✓
- checker_sha della riga phase=consume: blob `battery-equivalence.sh` a bb4b388
  = `0f62beed576f6298`. ✓  Entrambi i commit sono antenati di HEAD; il
  campaign ledger working copy è IDENTICO a HEAD; `verdict90.a1.g2.out` è
  tracked. Il self-tether A-BG53 ha retto: la catena giudici è chiusa.

## Q2 — La riga ABORT è distinguibile da una riga di script? NO (refutazione)
`battery-90pre.sh` (`att_row`, r.33-34) emette SOLO `esito=REFUSE|PASS|FAIL`.
La riga `esito=ABORT reason=head-moved-mid-battery-…-operator-error`
(epoch 1785745856, rev=8da340c) è quindi PER COSTRUZIONE scritta a mano —
ma con grammatica IDENTICA (`attempt_epoch= battery= rev= esito=`). Nessun
campo `writer=`, nessuna firma: la provenienza è una convenzione non
falsificabile. Un operatore potrebbe scrivere `esito=REFUSE` dove il run fece
FAIL e nessun dente morderebbe — il prefix A-AH54 protegge la storia
COMMITTATA, non autentica l'append. Il `reason=` che si auto-dichiara
"operator-error" è onestà volontaria, non un vincolo.

## Q3 — Il triangolo sha256==DSHA copre anche i FAIL? NO (buco confermato)
Il triangolo A-AH54 chiude SOLO il PASS: verificato al byte
sha256(OUT)=`e81ce60d…` == `.done` == stamps ledger == riga attempts PASS. ✓
Ma le righe FAIL (`fails=1/16 first_fail=measure-cifre`) e REFUSE non portano
ALCUNO sha: il `first_fail` è un'asserzione non riverificabile ex-post (l'OUT
del FAIL vive fuori repo in `/Volumes/…/wp90-battery-out/`, mutabile, non
ancorato). Non tocca la consumabilità (si consuma solo il PASS), ma è
esattamente la classe di forgia che A-AH54 ha chiuso sul PASS.

## Q4 — La finestra 4c99520..bb4b388 era pulita? SÌ (verificato)
`git diff --name-only 4c99520..bb4b388` = esattamente 3 path, tutti
allowlistati A-SK50: stamps ledger, attempts ledger, e il SOLO matrix archive
nominato dal `.done` (`feature-matrix.4c99520.20260803-104804.log`,
matrix_sha256 combacia). Delta attempts in finestra = la sola riga PASS
appesa (diff `11a12`, append a granularità di riga). Finestra pulita.

## Osservazione minore (grammatica ledger campagna)
Le righe `phase=verdict` g1/g2 NON portano `campaign_sha=` (presente su
preflight/start/consume): il verdetto è legato all'attempt solo per numero.

## Emendamenti
- **A-AH58 "writer= sulla riga attempts"**: ogni riga porta
  `writer=script:<sha16(script)>` (emessa da att_row) o `writer=operator`;
  `esito=ABORT` legale SOLO con `writer=operator`; lo script guadagna un trap
  che emette ABORT da sé su segnale/HEAD-move. Il checker rifiuta esiti fuori
  {PASS,FAIL,REFUSE,ABORT} e ABORT senza writer=operator.
- **A-AH59 "triangolo sui FAIL/REFUSE"**: att_row FAIL/REFUSE porta
  `sha256=<sha256(OUT-parziale)>` — ogni esito ancorato al proprio OUT, non
  solo il PASS.
- **A-AH60 "campaign_sha sulle righe verdict"**: verdict90.sh append include
  `campaign_sha=` letto dal ledger di campagna (riga phase=start).

## Kill-switch
- **KS-AH-92-1**: dopo A-AH58, una riga attempts senza `writer=` valido ⇒
  la successiva consumazione same-rev è VOID.
- **KS-AH-92-2**: dopo A-AH59, una riga FAIL/REFUSE senza ancora sha ⇒
  battery VOID (mai "assente con motivo").

## Refutazioni capitali: NESSUNA
La catena consumata (PASS→stamp→OUT→matrix→toolchain→judge g2) regge a
macchina su ogni maglia. I due buchi (provenienza ABORT, FAIL non ancorati)
sono laterali alla consumazione ma vanno chiusi in S-91.0.

---

# Verbale sedia 5 — Lars Bak (statistica, stimatori, ricomputo dai raw)

## VERDETTO: CONCORDO CON EMENDAMENTI

Ricomputato TUTTO dai 20 raw committati (`wp78-harness/measure-out/m90.slope.w{4,8,12,16}.r{1..5}.a1.memcensus`, righe `tag=mi_proc` per NOME, `tr -d '\0'` prima di grep). **Ogni cifra del g2 riproduce al byte**: modi dominanti per lane (BOOT 21364736/30474240/39387136/48431104; WORK 132513792/198115328/263782400/340983808; PEAK 151781376/228589568/303431680/389808128), b_boot=2252800 se=6345 a=12386304, b_work=17276928 se=501348 a=61079552, b_peak=19723059,2 se≈447215 a=71172096, residuo 193331, marginali 9/9. Il doc riporta il g2 fedelmente (Q4: sì sulle cifre). Le refutazioni sono sugli STIMATORI, non sui numeri.

## Q1 — b_peak degno di citazione? NO come testata

Tutti e 4 i punti PEAK hanno modes==R=5 (tie=4): il "modo dominante" non esiste e tiebreak=min trasforma la lane in una statistica d'ESTREMO. Ricomputo con stimatori alternativi: b_peak(min)=19.723.059, b_peak(mediana)=20.294.861, b_peak(media)=20.274.872 → il min tira giù ~0,55–0,57 MB/worker (−2,8%). Peggio: a W=12 il min NON rompe un tie, SELEZIONA la run anomala r3 (303,4M contro sorelle 325–334M: −7…−10% su TUTTI i contatori della run). Il doc stesso dichiara i punti ADVISORY (A-BB58) ma titola b_peak "PRIMA attribuzione ESATTA": incoerente. La cifra di testata onesta è la somma delle lane FORTI: **b_boot+b_work = 19.529.728 B** (a 45.875 B dal b m89 19.575.603 — continuità migliore di quella vantata).

## Q2 — tiebreak=min: distorce

Sì (vedi Q1). In più i marginali "9/9 IN fascia" sono estimator-dipendenti: con la mediana, WORK W8→12 = 22.183.936 (OUT alto) e W12→16 = 13.271.040 (OUT basso). Il 9/9 regge solo perché min-W8, min-W12 e min-W16 si allineano tra loro (la run anomala W12 DIVENTA il punto W12). Dispersione W12 ~11% del livello: nessun tiebreak la può nascondere.

## Q3 — additività: né forzata né vera, e il residuo è MAL ATTRIBUITO (refutazione capitale)

Within-run l'identità boot+Δwork+Δpost = exit_collect è aritmetica pura (tautologia). Come computata (cross-lane, punti scelti da RUN DIVERSE per lane) non è tautologica — ma allora il residuo 193.331 misura il MISMATCH DI SELEZIONE, non la fisica: per-W vale −2.097.152 / 0 / +262.144 / +393.216, dominato dal W=4 dove PEAK-min pesca r1 mentre BOOT/WORK pescano r2/r4. La label del giudice "the residue is the post-work segment: self+final census allocs" è **REFUTATA dai raw**: Δcommit post-work (exit_collect−pwork) = 0 su **17/20 raw** (non-zero solo w4.r5 +5.046.272, w16.r2 +1.048.576, w16.r3 +5.177.344). Il segmento self+census a commit costa ZERO nel caso tipico.

## Q4 — min-of-R ratio 1.000: controllo VACUO

Sì: con tie=4 su tutti i punti PEAK, dominante≡min per costruzione ⇒ ratio_minR≡1,000 non può fallire. Robustezza reale misurata: min/mediana = 0,972 (passerebbe comunque la soglia 0,8 — ma va misurata, non tautologizzata).

Nota minore: con 4 punti (df=2) il "2σ" è ~82% di confidenza, non 95% (t(0,975;2)=4,30). VCOV: ordine confermato su spot-check (0,42–0,63 per-raw), composizione non ricomputata al byte.

## Emendamenti

- **A-BB64 "Bandire tiebreak=min con modes==R"**: il punto per-W della lane PEAK diventa la MEDIANA delle R run (o il punto additivo per-run); ripubblicare b_peak (~20,29M atteso).
- **A-BB65 "Residuo VLADDER within-run"**: additività giudicata per-run (dov'è esatta) + residuo cross-lane rietichettato "selection mismatch"; la label census/self va ritirata.
- **A-BB66 "Robustezza non tautologica"**: A-BB62 sostituito da ratio min/mediana/media; se lo stimatore di controllo coincide per costruzione col primario, il check è VOID.
- **A-BB67 "Flag run-outlier"**: run con TUTTI i contatori oltre 2·MAD dalle sorelle (W12 r3) marcata per NOME nel verdict; non può diventare il punto della sua W via min.
- **A-BB68 "CI onesti"**: riportare ±t(df)·se o dichiarare la banda "±2se", mai "2σ⇒95%" con df=2.

## Kill-switch

- **KS-BB-92-1**: se una lane ha modes==R su TUTTI i punti, il suo b NON può essere cifra di testata del doc (testata = lane forti o mediana).
- **KS-BB-92-2**: ogni check di robustezza deve poter fallire sul canale reale; un ratio identicamente 1 per costruzione = check VOID, verdict declassato.

Refutazioni capitali: **sì** — la label del residuo (riga 40 del g2) è contraddetta da 17/20 raw, e b_peak min-based è uno stimatore d'estremo spacciato per modo dominante.

---

# Verbale sedia 6 — Pedersen (confine per-richiesta, lifecycle harness, failpath)

## VERDETTO: CON EMENDAMENTI

## Q1 — Rami di abbandono residui senza server_gone=
Due rami sfuggono alla disciplina A-PP50/51:
1. **`head_unmoved` (measure90-campaign.sh:82-85) esce con `exit 1` SENZA riga di ledger** — né abort, né VOID, né server_gone. Ledger-silenzio puro: la campagna "svanisce". Non orfanogeno (chiamato solo a server spento), ma refuta la lettera "failpath ledgerati su OGNI ramo".
2. **`assert_server_gone` (r.238-245): sul sopravvissuto a TERM chiama `fail` senza escalation KILL e senza server_gone=** — l'unico ramo che lascia un php-server VIVO oltre la fine campagna. Il preflight successivo lo intercetta, ma la riga VOID non testimonia lo stato del processo.
Minore: sul cammino `subshell_failpath` l'ordine righe è VOID→failpath (inverso di `teardown_fail`); e i failpath non fanno `wait "$DPID"` → il .log di `/usr/bin/time` può essere incompleto alla riga di ledger.

## Q2 — Quiescenza al momento di /__census_pwork
**La dichiarazione KS-MS-91-2 come scritta è FALSA nella lettera.** Il curl sequenziale garantisce zero risposte outstanding, non zero *request_end* outstanding: per la Binding Rule (output capture BEFORE request_end) il worker spedisce il body e POI esegue il teardown — curl ritorna mentre il worker può essere ancora in request_end. Keep-alive non c'entra (curl = processo nuovo per richiesta); il rischio è la coda di teardown. Con commit MONOTONO il picco è salvo; la **Δ pboot→pwork può assorbire teardown della richiesta n-esima** → attribuzione b_work formalmente non provata. Il contatore `census::OUTSTANDING` esiste già (worker_pool.rs:630): il testimone costa una riga.

## Q3 — Dove si rompe nreq/W+1 round-robin
Verificato: dispatch = `fetch_add % senders.len()` (worker_pool.rs:610-614), strict RR; `/__reqns` (wait_up + pid-echo) e `/__census_p*` sono ROUTER-side (main.rs:211,245) — non consumano slot; solo hello (100·W, divisibile per W) e le W self. La distinzione delle W self regge a QUALSIASI fase del contatore sotto RR stretto. Si rompe con: (a) **traffico estraneo su 127.0.0.1:8297** — nessuna esclusività client asserita, un client locale qualsiasi ruba uno slot in mezzo al blocco self, non testimoniato; (b) curl che fallisce il connect (slot non consumato) — coperto da ok==nreq solo per gli hello, non per interposizioni. Testimone economico: pin del contatore REQUESTS == nreq prima del blocco self.

## Q4 — Copertura pid-echo + scoping bash
`assert_http_pid` copre UN /__reqns per fase; **nessuna request census è pid-asserita** e i curl census sono senza `-f`: un HTTP 4xx/5xx passa con curl exit 0 — successo vacuo a livello harness, righe mancanti scoperte solo dal giudice. Scoping: `local SRV BOOT_EPOCH` + `identity_row` è CORRETTO (dynamic scoping, chiamata dentro run_*; `set -u` protegge usi esterni). Fragilità residua: `ps -o lstart` / `date -j -f "%a %b"` senza `LC_ALL=C` pinnato. **Ledger**: phase=start pinna judge_sha=1b1e9e96; g1 FAIL con quel giudice; g2 PASS con judge_sha=97a8eff0 e `supersede_of=g1` — **nessuna riga di ledger autorizza la sostituzione del giudice** (catena gG fuori banda).

## Emendamenti
- **A-PP-60** "assert_server_gone con escalation": passa per ensure_gone e ledgera server_gone= prima di fail.
- **A-PP-61** "head_unmoved ledgerato": instradare su fail; vietato l'exit muto.
- **A-PP-62** "census curl -f + pid-echo": `-f` su tutti i census; header x-phpr-pid asserito almeno su pwork e self.
- **A-PP-63** "testimone di quiescenza": la riga pwork porta outstanding=0 da census::OUTSTANDING; ≠0 ⇒ Δ fase ADVISORY.
- **A-PP-64** "LC_ALL=C" nel prologo campagna.
- **A-PP-65** "supersessione giudice in ledger": riga consume/authorize con delibera gG per ogni cambio judge_sha post-start.

## Kill-switch
- **KS-PP-92-1**: Δ di fase (b_boot/b_work) senza testimone outstanding==0 ⇒ mai verdict-grade.
- **KS-PP-92-2**: verdict row con judge_sha ≠ pinned a phase=start senza riga di autorizzazione ⇒ generazione VOID.

## Refutazioni capitali
Una: la garanzia di quiescenza KS-MS-91-2 come dichiarata (riga 291-293 del harness) non è vera al livello allocatore — HTTP-complete ≠ request_end-complete; l'attribuzione b_work resta ADVISORY finché manca il testimone OUTSTANDING==0.

---

# Verbale sedia 7 — Daan Leijen (mimalloc, footprint fisico, semantica contatori)

VERDETTO: **CONCORDO CON EMENDAMENTI**

## Q1 — Collisione mi_heap_of (VSELF REFUSED 20/20)

Meccanica, dai raw: `mi_bin_thr_sum` riporta per thr=0..15 lo **stesso**
`heap=0x105c3ae80` (w16.r1) e `0x10562ee80` (w4.r1) — indirizzo diverso
per RUN, non per thread: è un heap **statico nell'immagine** (ASLR sposta
la base), cioè `_mi_heap_main`. Su mimalloc v3 `mi_heap_get_default()` è
**TLS-only**: chiamata dal thread del census (o via wrapper cross-thread)
restituisce W volte il main-heap del chiamante. Prova interna: gli
`used_b` per-thr jitterano ±20 KB **non monotoni** (246.220.912 …
246.241.984) = un solo heap vivo letto in sequenza mentre i worker
mutano. La stat `heaps.total=1` vs `theaps.total=26` (w16 peak) dice che
i 26 thread-heap ESISTONO ma non sono enumerabili dall'API pubblica.
Non è "pagine condivise": è il wrapper che legge la TLS sbagliata.

**Iter-3 (in ordine):** (a) **census ON-THREAD**: alla barriera di picco
ogni worker chiama `mi_heap_get_default()+mi_heap_visit_blocks` sul
PROPRIO thread e spedisce lo snapshot su canale — zero API nuove;
(b) heap **espliciti** per worker (`mi_heap_new` + allocazioni VM
heap-scoped) → puntatori distinti e stats per-heap; (c) censimento
at-exit per thread (merge stats tld — richiede patch al crate).

## Q2 — P-CLAMP-PD refutata: la lettura "bracket/counter" regge, ma va scopata

pd=1000 **non** rende il run purge-free: dt1.afirst.pd1000 ha
`purge_calls=274`, `purged=85.721.088` già a exit_mi (mono_ms=1064 > pd)
e 127.467.520 dopo collect. Difende solo la FINESTRA: lo span clamped è
[6.059, 20.236] µs — nessuna pagina matura il deadline 1000 ms
in-bracket. Il discriminatore nei raw è **OVERLAP⇔clamp: 6/6 righe
clamped (pd0+pd1000) hanno spans=OVERLAP; 4/4 NO-OVERLAP hanno clamp
zero**; la coppia dt5.pd1000 (afirst NO-OVERLAP clamped=0 vs bfirst
OVERLAP clamped=1, stessi dt e pd) è l'esperimento naturale. E nel
clamped dt1.pd1000: `df=89.955.518 > da=86.862.318` eppure net=0 — il
bracket su contatori **di processo** sottrae il traffico concorrente
dell'altro lato (KB-88-1). Colpevole = finestra process-wide + overlap.
Debolezza: purged≡0 in-window è ARITMETICA, non lettura (A-DL50
unavailable); e 3/4 vs 4/4 è condizionato dallo scheduler.

## Q3 — b_boot 2.252.800 B (550 pagine ×4096, se=6.345 B)

Il contatore è il **commit mimalloc** (mi stats): gli stack pthread
(mmap di sistema) NON ci sono dentro ⇒ l'ipotesi "stack 2 MiB/thread" è
refutata dalla semantica del contatore, non serve misurarla.
Composizione candidata: bootstrap theap per-worker + stato phpr
per-worker (unit-cache TL, scaffolding VM). Intercetta a=12.386.304 =
runtime condiviso (boot W=4: threads=15, theaps=14). Discriminatore:
braccio **bare-thread** (W thread che toccano mimalloc, zero init phpr)
→ b_boot_bare; la differenza è lo stato phpr.

## Q4 — Coverage 0,576: dov'è il 42%

A w16.r1 peak il visit copre 253.420.432 su commit 399.638.528 (0,634).
L'invisibile, per NOME dai raw: (1) **malloc_huge**: 96 chiamate, total
638.582.784 — le huge non stanno nelle pagine-bin di alcun heap, il
visit non le vede (e il census logga il TOTAL cumulativo, serve il
CURRENT); (2) **pages_abandoned=2481** — pagine senza heap visitabile;
(3) free-committed in arena in attesa di purge (purged cumulativo
844.627.968 al picco); (4) pagine degli altri 25 theap. L'arena si
cammina **senza TLS**: estendere `mi_arena_chunk_bins` con classe
per-chunk (huge/abandoned/free-committed/heap-owned) chiude il bilancio.

## Emendamenti

- **A-DL-52 "census on-thread"**: snapshot per-worker eseguito SUL thread
  del worker alla barriera di picco; due thr con lo stesso heap ptr ⇒
  REFUSE (A-DL31 resta il dente).
- **A-DL-53 "purged ai bordi dello span"**: lettura read-only
  purged/purge_calls a t0/t1 dello span sul worker path (chiude A-DL50).
- **A-DL-54 "overlap forzato"**: braccio clamp con doppia richiesta a
  barriera — l'overlap diventa controllato, non emergente.
- **A-DL-55 "huge/abandoned nel census"**: current (non total) di
  malloc_huge + conteggio pagine abandoned + classi per-chunk in
  mi_arena_chunk_bins.
- **A-DL-56 "bare-thread arm"**: scomposizione b_boot = thread-runtime +
  stato phpr.

## Kill-switch

- **KS-DL-92-1**: census per-worker con heap ptr duplicato ⇒ REFUSE, mai
  media.
- **KS-DL-92-2**: qualunque claim sul clamp senza riga overlap-status
  per span ⇒ VOID.

Refutazioni capitali: no. La refutazione P-CLAMP-PD regge (ri-scopata:
vale per la finestra, non per il run); l'ipotesi stack su b_boot cade
per semantica del contatore.

---

# Verbale sedia 8 — Stogov (Zend/opcache, contratto LSP A-DS35) — Concilio WP-92

VERDETTO: **CON EMENDAMENTI** (due refutazioni capitali sul formato pin e sul bersaglio t4).

## Q1 — Copertura RESIDUA del set v2: SETTE buchi, tutti oracle-morsi dal vivo (8.5.7, probe in scratchpad ds35-stogov/)

1. **Interfacce multiple in conflitto** (`implements I1, I2` con firme incompatibili) → fatal `Declaration of C::m(): int must be compatible with I2::m(): string`. Nessuna fixture v1/v2 lo copre (v13 copre solo il ctor di interfaccia).
2. **Interface extends interface** con firma incompatibile → fatal SENZA alcuna classe coinvolta: il check vive anche nel linking di interfacce; la sede duale deve coprirlo o t.b.d. a catalogo.
3. **Enum implements** con firma incompatibile → fatal (`E::m(string $x): int must be compatible with I::m(int $x): int`). Gli enum sono assenti dall'intero set.
4. **self vs static**: `P::m(): static` / `C::m(): self` → fatal col **self RISOLTO nel messaggio**: `Declaration of C::m(): C must be compatible with P::m(): static`. Vincolo di modellistica su `TypeHint::display_name`: `self` si stampa come nome-classe risolto, non «self». n3 non lo copre.
5. **Property hooks**: parent `public int $x { get => … }`, child ridichiarata → messaggio di FAMIGLIA DIVERSA da n8: `Type of C::$x must be subtype of int (as in class P)` («subtype of», non «must be int»). phpr modella gli hook: serve la fixture.
6. **Costanti final**: `C::X cannot override final constant P::X` → fatal, non coperto.
7. **readonly PROMOTED** (ctor promotion) → stesso messaggio di v11 ma path di lowering diverso; fixture a costo zero.

DNF `(A&B)|string → string` è LEGALE (verificato): manca il positivo di regressione. Trait+abstract è coperto (n9).

## Q2 — Formato pin: refutazione CAPITALE, il marcatore è forgiabile

`echo "x\n--stderr\ny--end"` è un programma PHP legale: il suo stdout contiene le righe-marcatore. Un comparatore futuro che parse-a `ds35-verify2.out` a scansione di marcatori è ambiguo per costruzione (oggi non morde solo perché le 37 fixture non emettono quei byte). Inoltre `42--stderr` su una riga (stdout senza newline finale) è decodificabile solo conoscendo i marcatori — stessa fragilità. **Pin v3: `--stdout bytes=N` / `--stderr bytes=N` per canale; parse = lettura di N byte esatti.** Vietato il comparatore a marcatori.

## Q3 — Bersaglio t4: «stdout integrale dell'oracle» non nomina il BRACCIO

Su t4 i due bracci divergono: plain = `pre|` PRIMA del fatal, persist = fatal PRE-output. phpr con lowering hoisted fatalerà pre-output ⇒ il bersaglio byte-fedele di t4 è il braccio **PERSIST**, da dichiarare per NOME nel pin. Contraddizione residua da sciogliere: appendice WP-89 dice «skip conservativo su nomi non risolvibili», §3.3-quinquies dice «mai skip silenzioso» (v15). Prevale la delibera WP-91: v15 fatala fedele al bind hoisted; per i bind dinamici decide il gate ORM/hk.

## Q4 — Ordine A-DS51 (rischio ORM/hk minimo)

(1) checker LSP puro + unit sui messaggi (nessun wiring); (2) wiring nel lowering HOISTED (lower/class.rs, dopo flatten trait, accanto al final-check) **con le esenzioni nello STESSO commit** (ctor-plain-class, private, RTWC, tentative-Deprecated A-DS46 — nota: il ctor NON è esente per interfacce/abstract, v13/v14) → gate ORM 3E/13F + hk 1665 + corpus per NOME; (3) bind-registry per condizionali/dinamiche (t2), gate di nuovo. Vincolo di fase 2: le condizionali restano fuori (t3 = REJECT se fatala). L'ordine inverso lascerebbe scoperti 35/37 pin nella finestra intermedia.

## Emendamenti

- **A-DS53** «fixture v3 — 7 buchi oracle-morsi»: multi-iface, iface-extends, enum-implements, self-vs-static, hook-subtype, final-const, readonly-promoted (+ positivo DNF).
- **A-DS54** «pin v3 a byte-count per canale; comparatore a marcatori bandito».
- **A-DS55** «bersaglio t4 = braccio PERSIST, dichiarato per NOME».
- **A-DS56** «v15: prevale “mai skip silenzioso”; fatal al bind hoisted, dinamici via gate».
- **A-DS57** «ordine A-DS51: checker→hoisted(+esenzioni stesso commit)→registry; gate ORM/hk dopo OGNI fase».

## Kill-switch

- **KS-DS-92-1**: comparatore che parse-a i pin per marcatori senza byte-count = REJECT.
- **KS-DS-92-2**: merge A-DS51 senza fixture A-DS53 per NOME nel gate = REJECT.
- **KS-DS-92-3**: fase-2 (hoisted) che fatala t3 o manca il persist-shape di t4 = REJECT.

---

# Verbale sedia 9 — Brendan Gregg (metodologia di misura, attribuzione, leggibilità dai ledger)

VERDETTO: **CONCORDO CON EMENDAMENTI** (nessuna refutazione capitale: ogni cifra del doc ricomputata a macchina dai raw e dal g2 — additività b_peak=b_boot+b_work+residuo esatta al byte 19.723.059=2.252.800+17.276.928+193.331; medie VCOV 158.462.223/275.198.771 riprodotte al byte dai 20 raw; 34 raw=20+4+10 contati nel ledger; marginali 9/9 IN per NOME; battery 4 tentativi tutti ledgerati).

## Q1 — Il doc riporta esattamente il g2? QUASI: un conteggio ADVISORY è sotto-riportato
Il g2 marca `[A-BB58: modes==R, point ADVISORY]` su **7 punti**: PEAK W4/8/12/16 **e WORK W8/12/16** (tie=4, tiebreak=min). Il doc nomina solo i 4 del braccio PEAK e chiama BOOT/WORK "le lane forti". La scala Δcommit di VSLOPE-WORK resta esatta, ma il censimento per-modo della lane WORK è degenerato su 3/4 punti: il conteggio per NOME nel doc deve dire 7, non 4, e la dicitura "lane forti" va scopata alla sola scala Δcommit.

## Q2 — VCOV media-su-20 nasconde variazione per W? SÌ, misurata
Il giudice fa ratio delle medie pooled (vis/n)/(tot/n). Per-W (ricomputato dai raw): **0,415–0,421 (W4) → 0,545–0,555 (W8) → 0,567–0,626 (W12) → 0,634–0,650 (W16)**. Lo 0,576 non esiste in nessun raw. Peggio: la massa invisibile è dominata dall'INTERCETTA (~74 MB fissi + ~3,9 MB/worker); la **coverage MARGINALE** (pendenza di vis su W) è 15.781.205 B/worker ≈ **0,80 di b_peak** — per l'attribuzione di b il census è molto migliore di quanto lo 0,576 racconti. Un solo numero pooled sbaglia in entrambe le direzioni.

## Q3 — Storia g1→g2 leggibile dal solo ledger? NO
Il campaign ledger registra judge_sha per generazione e supersede_of=g1 (bene), ma `reason=see-verdict-file` (g1) e `reason=all-blocks-clean` (g2) sono puntatori, non ragioni: per sapere COSA è stato riqualificato (VCKPT sweep want 1/3→1/1) bisogna aprire i verdict file e diffare i blob dei giudici. "attempt=1 PULITO" regge SOLO perché entrambi i ledger portano le righe sporche (ABORT operatore a 8da340c nel battery-ledger; esito=FAIL g1 nel campaign ledger): la dicitura è legale ma va sempre citata in coppia con quelle righe, scope = fasi di misura.

## Q4 — commit_at ultima-riga: dove può mentire?
Il conteggio ==1 di VCKPT copre i nomi giudicati (ultima=unica). Ma nell'awk `v` NON è resettato per riga: una riga mi_proc che matcha win/ckpt ma è PRIVA di commit= eredita silenziosamente il commit della riga precedente. Verificato: 0/20 raw m90 colpiti (misscommit=0) — latente, non attuale. Inoltre VIS in VCOV usa regex-substring (/win=9/) non match di campo esatto. Sul clamp: nei 10 sweep **clamped ⇔ spans=OVERLAP 10/10** — legge osservata che il doc non dichiara; il "3/4 con pd=1000" è confuso: l'unico run senza clamp è anche l'unico NO-OVERLAP (negativo non controllato). La refutazione di P-CLAMP-PD regge, e il 10/10 la RAFFORZA verso bracket/counter.

## Emendamenti
- **A-BG57 "VCOV per-W + coverage marginale"**: il giudice riporti la tabella per-W e la pendenza di vis (coverage del margine), mai il solo pooled.
- **A-BG58 "reason= autosufficiente"**: righe verdict/supersede con reason=requalify:<blocco>:<old→new>; il ledger da solo deve rigenerare la storia g(n)→g(n+1).
- **A-BG59 "commit_at reset per riga"**: azzerare v a ogni riga mi_proc; riga matching senza commit= = FAIL, mai ereditare.
- **A-BG60 "ADVISORY per NOME=7"**: doc emendato: 4 PEAK + 3 WORK; "lane forti" scopata alla scala Δcommit.
- **A-BG61 "legge clamp⇔overlap"**: dichiararla 10/10 e progettare il negativo controllato (pd=1000 CON overlap forzato).

## Kill-switch
- **KS-BG-92-1**: VCOV citata senza tabella per-W ⇒ la metà census non è citabile nemmeno come ADVISORY.
- **KS-BG-92-2**: riga verdict il cui reason non ricostruisce la riqualifica senza aprire file esterni ⇒ supersede invalido alla campagna successiva.

---

## NOTE DI TEAM (fase 2)

# Team CIFRE — relazione di riconciliazione (Concilio WP-92)

Sedie: 3 Klabnik (gate cifre/forge) · 4 Hejlsberg (catena evidenza) · 9 Gregg
(metodologia, attribuzione doc-vs-verdict). Relatore: sedia 3 delega la
relazione; **i tre verbali individuali restano la fonte VINCOLANTE** — questa
relazione non li riassume né li supera, li colloca.

Verdetti: Klabnik CON EMENDAMENTI (4 forge reali passati dal gate vivo) ·
Hejlsberg CONCORDO CON EMENDAMENTI (nessuna refutazione capitale) · Gregg
CONCORDO CON EMENDAMENTI (nessuna refutazione capitale).

---

## CONVERGENZE

1. **Un'asserzione vale quanto il blob a cui si risolve.** Hejlsberg lo
   verifica in positivo (Q1: judge_sha g1=`1b1e9e96f5019bc0`, g2=
   `97a8eff0d9783ee4`, checker_sha=`0f62beed576f6298`, tutti risolti a blob di
   commit antenati di HEAD); Klabnik lo verifica in negativo (Q1: manifest e
   budget letti dal **working tree** mentre il corpus è letto da HEAD ⇒ input
   non autenticato, forge landed). **A-SK-67** è la forma generale della stessa
   legge che **A-BG53/self-tether** ha già fatto reggere sulla catena giudici:
   nessuna tensione, un'unica regola da estendere a manifest/budget/giudice.

2. **Ciò che non è emesso da uno strumento non è autenticato.** Hejlsberg
   Q2: la riga `esito=ABORT reason=…operator-error` ha grammatica identica a
   quelle di `att_row` ma è per costruzione scritta a mano (**A-AH58**
   `writer=`). Klabnik Q3: `--cache/--nonce` sposta il veleno in argv, scelto
   dal chiamante, e il gate non stampa il proprio sha (**A-SK-70**, **A-SK-67**
   parte judge_sha). Stessa classe: *provenienza dell'append/dell'invocazione*.

3. **Le coperture parziali sono buchi, non gradazioni.** Hejlsberg Q3: il
   triangolo `sha256==DSHA` chiude solo il PASS, FAIL/REFUSE non portano alcuno
   sha e il loro OUT vive fuori repo, mutabile (**A-AH59**). Klabnik Q4: `--all`
   guarda solo `wp*-harness/MEASURE*_RESULTS.md`, mentre **43 cifre** in
   NEXT_SESSION (20) e `sessions/WP_SESSION_90.md` (23) restano non giudicate
   contro 28 giudicate (**A-SK-71**). Stessa forma: il perimetro è definito
   dallo strumento, non dal flusso reale dell'evidenza.

4. **Un solo numero pubblicato al posto di una distribuzione mente.**
   Gregg Q2: VCOV 0,576 non esiste in nessun raw (per-W 0,415→0,650; coverage
   marginale 15.781.205 B/worker ≈ 0,80·b_peak) — **A-BG57**, **KS-BG-92-1**.
   Convergenza con Klabnik Q2: la `prov` ammette operandi da campagne diverse
   (live PASS `19.600.000 = 23000000@…axum.83c… − 3400000@…axum.82c…`), cioè
   *pool sbagliato* invece di *aggregato sbagliato*. In entrambi i casi la
   difesa è la stessa: **la cifra pubblicata deve nominare il proprio dominio**.

5. **doc == verdict si controlla per NOME, non per conteggio.** Gregg Q1: il g2
   marca ADVISORY su **7** punti (4 PEAK + 3 WORK), il doc ne nomina 4 e chiama
   BOOT/WORK "lane forti" (**A-BG60**). È lo stesso difetto che Klabnik trova
   in **A-SK66** (la riga «NON e superseded» passa per parola-chiave, non per
   prova) e che **A-SK-72** chiude con ledger committato. La coppia
   A-BG60+A-SK-71 è la stessa disciplina applicata a due perimetri.

6. **Il resolver deve verificare l'operazione che è scritta.** Klabnik Q2.3:
   `[derivata: prov X@p:1 diviso Y@p:2]` passa e stampa "provenance-verified"
   perché il test `/−|-/` è soddisfatto dal trattino di `wp89-harness`
   (**A-SK-69**). Gregg Q4 trova il gemello nel giudice: nell'awk `v` non è
   resettato per riga, una riga senza `commit=` eredita la precedente
   (**A-BG59**, 0/20 latente), e VIS usa regex-substring `/win=9/` non match di
   campo. Stessa patologia: **matching lasco al posto di parsing**.

7. **Il ledger deve rigenerare la propria storia da solo.** Gregg Q3:
   `reason=see-verdict-file` / `all-blocks-clean` sono puntatori, non ragioni
   (**A-BG58**, **KS-BG-92-2**); Hejlsberg (osservazione minore): le righe
   `phase=verdict` non portano `campaign_sha=`, il verdetto è legato
   all'attempt solo per numero (**A-AH60**). Due maglie dello stesso anello.

8. **"Attempt=1 PULITO" è legale solo in coppia con le righe sporche.**
   Gregg Q3 lo dichiara esplicitamente (ABORT operatore a `8da340c` nel
   battery-ledger, `esito=FAIL` g1 nel campaign ledger, scope = fasi di misura);
   Hejlsberg Q4 fornisce la prova che rende la coppia leggibile (finestra
   `4c99520..bb4b388` pulita, 3 path allowlistati A-SK50, delta attempts =
   la sola riga PASS appesa). Convergenza piena: la dicitura resta, con obbligo
   di citazione appaiata.

---

## CONFLITTI

**Nessuna contraddizione di fatto tra le tre sedie.** Nessuna cifra affermata
da una sedia è negata da un'altra; nessun emendamento di una sedia rende
illegale un emendamento di un'altra. Restano **cinque tensioni di
collocazione**, che il plenum deve risolvere come ordine, non come merito.

**T1 — "quattro forge capitali" (Klabnik) vs "nessuna refutazione capitale"
(Hejlsberg, Gregg): perimetri disgiunti, non giudizi opposti.**
- *Klabnik*: il gate che **ammette** cifre è forgiabile dal vivo; l'evaluator
  X−Y non è stato abolito ma annotato, e oggi copre il **46,25 %** delle
  differenze nella finestra (contro l'~11,5 % del WP-91: il buco è **cresciuto**,
  non chiuso), su **36.573** operandi indirizzabili contro i 24.042 del corpus.
- *Hejlsberg*: la catena **consumata** in S-90.0 regge a macchina su ogni
  maglia (PASS→stamp→OUT→matrix→toolchain→judge g2); i suoi controlli sono
  stati eseguiti risolvendo blob a HEAD **a mano**, quindi non passano dal
  percorso working-tree che Klabnik ha morso: le due conclusioni sono
  indipendenti e entrambe vere.
- *Gregg*: ogni cifra del doc è stata **ricomputata dai raw** (additività
  19.723.059 = 2.252.800 + 17.276.928 + 193.331 esatta al byte; medie VCOV
  riprodotte al byte dai 20 raw; 34 raw = 20+4+10; 9/9 marginali IN per NOME),
  quindi il *contenuto* di S-90.0 non è in discussione.
- *Collocazione proposta*: S-90.0 **resta consumabile**; i forge di Klabnik
  colpiscono la **ammissibilità futura** e le 43 cifre già pubblicate fuori
  perimetro. Non si riapre il verdetto 90; si chiude il gate prima che il
  prossimo verdetto lo attraversi.

**T2 — A-SK-67 (`working≠HEAD ⇒ FAIL`) vs il flusso reale di scrittura di un
doc.** Un MEASURE/NEXT_SESSION in scrittura non è a HEAD per definizione.
- *Klabnik* chiede HEAD per **manifest, budget, giudice** (gli input di
  autorità), non necessariamente per il doc giudicato.
- *Hejlsberg* (implicito in Q1/Q4) tratta come verdict-grade solo ciò che è
  risolvibile a blob committato.
- *Risoluzione proposta*: due modi espliciti nel gate — `advisory`
  (pre-commit, doc dal working tree, autorità comunque da HEAD, **mai**
  verdict-grade) e `verdict` (tutto da HEAD, PASS che stampa
  judge_sha+manifest_sha+budget_sha). Senza questa distinzione A-SK-67
  rende il gate ineseguibile durante la sessione.

**T3 — A-SK-70 (cache abolita) vs A-SK-71 (perimetro da 28 a ~71 cifre).**
Abolire la cache mentre il perimetro quasi triplica è un costo che nessuna
sedia ha misurato. Klabnik offre già la via che non paga il conflitto (**un
solo processo perl per `--all`**, invece della cache creata dal parent):
adottare quel ramo, non il ramo "cache rifiutata se non creata dal parent",
che lascia in piedi un'autorità da argv.

**T4 — A-SK-68 (abolizione `legacy_frozen`) vs i doc storici congelati.**
Nessuna sedia difende `legacy_frozen` (Klabnik: "doc STORICO CONGELATO" è un
**commento**, non un predicato — nessun test di committed-ness, di data, di
appartenenza a M84-M88; il forge è passato con una **riga di manifest mai
committata**). Ma l'abolizione secca, applicata prima di aver collocato i 5
doc, li spinge in un limbo: rigiudicati con le regole nuove **fallirebbero**
(usano l'evaluator libero pre-WP-91), e con essi diventerebbero non
verdict-grade cifre già citate a valle (NET_H/NET_P di WP-85, slope WP-87,
b_base WP-88/89). Vedi P2 nelle priorità: l'ordine risolve, il merito no.

**T5 — chi scrive per primo sulla stessa riga.** A-AH58 (`writer=` su
attempts), A-AH59 (sha su FAIL/REFUSE), A-AH60 (`campaign_sha` su verdict),
A-BG58 (`reason=` autosufficiente) e A-SK-72 (supersessione da ledger
committato) toccano **le stesse due grammatiche di ledger**. Applicati in
delibere separate produrrebbero tre revisioni di formato in una sessione, e
ogni cambio di grammatica invalida i checker che la leggono. Vanno emessi
come **una sola delibera di formato**, con i checker aggiornati nello stesso
commit.

---

## COMPATIBILITÀ DELLE ABOLIZIONI (richiesta esplicita del mandato)

**A-SK-68 (abolire `legacy_frozen`) è compatibile con Hejlsberg** — anzi ne è
il corollario: sostituire un predicato "sha in una riga di manifest del
working tree" con "verdetto d'epoca citato per blob committato" *aumenta* la
risolvibilità a blob, che è esattamente il criterio con cui Hejlsberg ha
chiuso Q1. **È compatibile con Gregg** a una condizione: `judge=no` non
significa "cifra libera". Gregg pretende doc==verdict **per NOME**; per un doc
storico il verdetto è quello d'epoca, quindi la riga deve nominare
`verdict_blob=<sha16>` e le cifre del doc devono corrispondere a quel blob per
nome — altrimenti `judge=no` diventa la nuova `legacy_frozen`.

**A-SK-70 (abolire la cache) è compatibile con entrambi senza condizioni**:
nessun emendamento di Hejlsberg o Gregg dipende dalla cache; l'unico effetto è
il costo di runtime di T3.

**Ordine che non rompe i doc storici congelati** (questo è il punto delicato):
non abolire prima e collocare poi. Prima si esegue un **bite-test dei 5 doc**
con il resolver `prov` emendato (A-SK-69) e si assegna a ciascuno l'esito:
`judge=yes` per quelli che reggono con operandi dello stesso file, `judge=no +
verdict_blob` per gli altri, **con la riga di manifest committata nello stesso
commit dell'abolizione**. Solo dopo si rimuove il ramo `legacy_frozen` dal
codice. Così nessun doc attraversa uno stato in cui non è né giudicato né
collocato, e nessuna cifra a valle perde il proprio ancoraggio.

---

## PRIORITÀ PROPOSTE per l'ordine S-91.0
*(ordinate per potere di blocco: cosa deve atterrare PRIMA perché il resto sia
consumabile)*

**P1 — A-SK-67 + KS-SK-92-1 + KS-SK-92-4: autenticare gli input di autorità
del gate (manifest, budget, giudice) da HEAD; PASS stampa
judge_sha+manifest_sha+budget_sha; modi `advisory`/`verdict` (T2).**
Potere di blocco massimo: finché manifest e budget vengono dal working tree e
il giudice non si nomina, **ogni** altro emendamento al gate è aggirabile
dalla stessa porta, e i risultati di P3/P5 non sarebbero credibili. Include
l'abolizione della cache (A-SK-70) nel ramo "un solo processo per `--all`",
perché è la stessa superficie di autorità-da-argv. Controllo positivo
obbligatorio: **i quattro forge di Klabnik devono essere ripetuti e devono
FALLIRE**, con l'output del fallimento ledgerato (il dente nuovo morde sul
proprio harness — lezione WP-88/89).

**P2 — A-SK-69 + A-SK-73, poi bite-test dei 5 doc storici, poi A-SK-68.**
Secondo per potere di blocco: la `prov` è oggi *più larga* del buco che doveva
chiudere (46,25 % di chiusura, 36.573 operandi contro 24.042), quindi ogni
cifra ammessa dopo P1 ma prima di P2 resta forgiabile per composizione.
L'ordine interno è vincolante e non negoziabile: **resolver emendato →
bite-test → collocazione committata dei 5 doc (`judge=yes` | `judge=no +
verdict_blob`) → rimozione del ramo `legacy_frozen`**, tutto con la riga di
manifest nello stesso commit. Invertire questi passi mette i doc storici in
limbo (T4).

**P3 — delibera unica di formato ledger: A-AH58 + A-AH59 + A-AH60 + A-BG58 +
A-SK-72, checker aggiornati nello stesso commit (T5).**
Blocca la *leggibilità* di tutto ciò che S-91.0 produrrà: senza `writer=` un
ABORT resta indistinguibile da un esito di script; senza sha su FAIL/REFUSE
la classe di forgia chiusa sul PASS resta aperta su ogni altro esito; senza
`reason=` autosufficiente e `campaign_sha=` la storia g(n)→g(n+1) non è
ricostruibile dal solo ledger. Emettere queste cinque regole insieme costa
una migrazione di grammatica; emetterle separate ne costa tre.

**P4 — A-SK-71 + A-BG60 + A-SK-72 (lato regex `.out` opzionale): perimetro
reale delle cifre pubblicate.**
Deve venire **dopo** P1-P2, altrimenti estende un gate forgiabile a 43 cifre
in più invece di proteggerle. Include l'emendamento del doc S-90.0 sul
conteggio ADVISORY (**7 = 4 PEAK + 3 WORK**, "lane forti" scopata alla sola
scala Δcommit): è il caso di prova che dimostra che la disciplina doc==verdict
per NOME morde anche dentro il perimetro già coperto.

**P5 — A-BG57 + A-BG59 + A-BG61 + KS-BG-92-1: igiene del giudice e della
pubblicazione delle metriche.**
Non blocca il consumo di S-90.0 (Gregg ha ricomputato tutto dai raw), ma
blocca l'**iterazione 3** dell'attribuzione: VCOV va pubblicata per-W con la
pendenza marginale (15.781.205 B/worker) e mai come solo pooled; `v` va
resettato per riga (0/20 latente oggi, ma è un ereditamento silenzioso); la
legge **clamped ⇔ spans=OVERLAP 10/10** va dichiarata e dotata del suo
negativo controllato (pd=1000 **con** overlap forzato), perché oggi l'unico run
senza clamp è anche l'unico NO-OVERLAP.

**Nota di sequenza.** P1 e P2 sono l'unico blocco veramente seriale: toccano
lo stesso file di gate e la stessa autorità. P3 è indipendente da P1-P2 e può
procedere in parallelo se lo tocca un'altra mano. P4 dipende da P1+P2+P3
(serve il manifest autenticato *e* la grammatica nuova). P5 è indipendente da
tutti e può chiudere la sessione.

---

# Verbale TEAM-MISURA — Concilio WP-92 (fase 2)

**Composizione**: sedia 5 Bak (statistica/stimatori) · sedia 6 Pedersen
(lifecycle/quiescenza) · sedia 7 Leijen (mimalloc/contatori). Relatore: sedia 5.
**Fonti**: `verbale-5-bak.md`, `verbale-6-pedersen.md`, `verbale-7-leijen.md`
(integrali). I verbali individuali restano VINCOLANTI: questo documento
riconcilia, non sostituisce né emenda.

**Verdetti individuali**: tutte e tre le sedie CON EMENDAMENTI. Refutazioni
capitali dichiarate: Bak 2 (label del residuo; b_peak min-based), Pedersen 1
(quiescenza al census), Leijen 0.

---

## CONVERGENZE

**CV-1 — Le CIFRE del g2 non sono in discussione; gli STIMATORI e le ETICHETTE
sì.** Bak ha ricomputato tutte e 20 le raw committate
(`m90.slope.w{4,8,12,16}.r{1..5}.a1.memcensus`) e ogni numero del g2 riproduce
al byte (b_boot=2.252.800 se=6.345; b_work=17.276.928 se=501.348;
b_peak=19.723.059,2 se≈447.215; residuo 193.331; marginali 9/9). Nessuna sedia
contesta la fedeltà doc↔g2 (Q4 Bak: sì). L'attacco è a monte (selezione del
punto) e a valle (che cosa il numero significhi), mai sull'aritmetica.

**CV-2 — Il canale è PROCESS-WIDE mentre l'oggetto misurato è PER-WORKER.**
Tre attacchi indipendenti nominano la stessa malattia: Bak Q3 (i punti per-lane
vengono da RUN DIVERSE ⇒ il residuo cross-lane misura mismatch di selezione),
Leijen Q2 (il bracket su contatori di processo sottrae il traffico concorrente
dell'altro lato: `df=89.955.518 > da=86.862.318` eppure net=0, KB-88-1),
Pedersen Q3 (nessuna esclusività client asserita su 127.0.0.1:8297: un client
locale ruba uno slot senza testimone). Corollario condiviso: finché la misura
non è scopata all'ATTORE (thread/worker) e alla RUN, ogni Δ è contaminabile.

**CV-3 — Il residuo 193.331 non è il segmento post-work.** Coincide
esattamente con b_peak − (b_boot+b_work), cioè è per costruzione un artefatto
cross-lane. Bak lo refuta dai raw: Δcommit post-work = 0 su **17/20**. Leijen
spiega perché il canale non potrebbe comunque vederlo: le allocazioni di
self+census atterrano su spazio già committato in attesa di purge (purged
cumulativo 844.627.968 al picco) o su `malloc_huge` (96 chiamate, total
638.582.784) — invisibili al commit e al visit. La label del giudice va
ritirata come **non falsificabile su questo canale**, non solo come falsa.

**CV-4 — Principio epistemico comune: un check che non può fallire è VOID.**
KS-BB-92-2 (ratio min-of-R ≡ 1,000 per costruzione con tie=4 ⇒ controllo
vacuo) e KS-DL-92-1 (heap ptr duplicato ⇒ REFUSE, mai media) sono lo stesso
dente applicato a due oggetti. Pedersen lo applica al lifecycle: un ramo che
esce senza riga di ledger (`head_unmoved`, campaign.sh:82-85) è un
non-testimone.

**CV-5 — b_boot è la lane più solida delle tre.** se relativo 0,28%; modi
dominanti reali (non tie); quiescenza triviale (pboot precede ogni richiesta,
quindi KS-PP-92-1 non morde); semantica del contatore pulita — Leijen refuta
l'ipotesi "stack 2 MiB/thread" per SEMANTICA (il commit mimalloc non contiene
gli mmap di sistema degli stack pthread), senza bisogno di misura.

**CV-6 — La composizione di b resta ignota anche dove la magnitudine è
solida.** b_boot è un numero senza scomposizione (Leijen: serve il braccio
bare-thread, A-DL-56); la copertura del visit è 0,576–0,634 e il 42% mancante
è nominato ma non contabilizzato (huge current, `pages_abandoned=2481`,
free-committed, i 25 theap non enumerabili). Nessuna sedia considera
l'attribuzione chiusa.

**CV-7 — Gli strumenti per il prossimo passo esistono già.**
`census::OUTSTANDING` (worker_pool.rs:630), pid-echo, `mi_heap_get_default` +
`mi_heap_visit_blocks` on-thread (zero API nuove), i 20 raw committati. Nessuna
priorità dell'iterazione 3 richiede infrastruttura nuova.

---

## CONFLITTI

### CF-1 — Grado della cifra di testata (l'unico conflitto vero)

- **Bak**: la testata onesta è **b_boot+b_work = 19.529.728 B** (lane FORTI),
  a 45.875 B da b m89 = 19.575.603; b_peak min-based non può essere testata
  (KS-BB-92-1). Bak non assegna un grado alla somma: la propone come forma
  migliore.
- **Pedersen**: **b_work è ADVISORY** finché la riga pwork non porta
  `outstanding=0` (KS-PP-92-1). HTTP-complete ≠ request_end-complete: per la
  Binding Rule il body parte PRIMA del teardown, quindi la Δ pboot→pwork può
  assorbire il teardown della richiesta n-esima. Aggravante che il team
  registra: il residuo di teardown è **per-worker**, quindi aliasa nella
  PENDENZA, non nell'intercetta — è esattamente il parametro citato.
- **Leijen**: non si pronuncia sul grado; convalida la semantica del contatore
  per b_boot e non porta obiezioni alla lane WORK come magnitudine.

**Composizione del team**: le due posizioni sono su assi diversi (Bak = quale
stimatore; Pedersen = quale grado) e si compongono a strati. Vedi NODO.

### CF-2 — Tensione con l'ORDINE WP-91 ("b = pendenza del PICCO")

- **Bak**: la lane PEAK ha `modes==R=5` (tie=4) su TUTTI e 4 i punti; il
  tiebreak=min la rende una statistica d'ESTREMO (−2,8% vs mediana) e a W=12
  SELEZIONA la run anomala r3 (303,4M contro sorelle 325–334M, −7…−10% su tutti
  i contatori). Quindi: non citabile come testata.
- **Leijen**: usa senza riserve i raw di picco (w16.r1 peak: commit 399.638.528,
  `heaps.total=1` vs `theaps.total=26`) per coverage e diagnosi.
- **Pedersen**: con commit monotono il PICCO è salvo; è la Δ di FASE a essere
  esposta.

**Non è un conflitto tra sedie**: il PUNTO di picco resta uno stato osservato
valido (Leijen, Pedersen); è la PENDENZA della lane PEAK a non essere
stimabile **con l'attuale disegno**. L'ordine WP-91 non va revocato, va reso
eseguibile: la lane PEAK diventa stimabile solo se il punto di picco è preso
per-worker on-thread a una barriera (P1) e stimato con mediana/punto additivo
per-run (P2). Il team registra questo come vincolo di progetto, non come
dissenso dal Concilio WP-91.

### CF-3 — Nessun conflitto su clamp/purge

La refutazione P-CLAMP-PD regge ma va SCOPATA (Leijen): pd=1000 non rende
purge-free il RUN (`purge_calls=274`, purged 85.721.088 già a exit_mi con
mono_ms=1064) — difende solo la FINESTRA (span clamped [6.059, 20.236] µs). Il
discriminatore reale è **OVERLAP⇔clamp: 6/6 clamped hanno spans=OVERLAP, 4/4
NO-OVERLAP hanno clamped=0**, con la coppia dt5.pd1000 come esperimento
naturale. Debolezza dichiarata: `purged≡0` in-window è ARITMETICA, non lettura
(A-DL50 unavailable) — chiude A-DL-53. Le altre due sedie non obiettano.

---

## NODO CENTRALE — la scomposizione b_boot/b_work è citabile? A che grado?

**POSIZIONE UNICA DEL TEAM (motivata, senza dissenso registrato):**

1. **b_boot = 2.252.800 B/worker (se=6.345) — VERDICT-GRADE come
   MAGNITUDINE.** Regge a tutti e tre gli attacchi: modi dominanti reali (non
   min-statistic ⇒ KS-BB-92-1 non morde), quiescenza triviale al pboot
   (KS-PP-92-1 non morde), semantica del contatore convalidata (Leijen Q3).
   **Non** è verdict-grade come COMPOSIZIONE: "bootstrap theap + stato phpr"
   resta ipotesi finché non gira il braccio bare-thread (A-DL-56). Da citare
   come *quanto*, mai come *di che cosa*.

2. **b_work = 17.276.928 B/worker (se=501.348) — ADVISORY.** Due ragioni
   indipendenti, entrambe sufficienti: (a) KS-PP-92-1, manca il testimone
   `outstanding==0` e il residuo di teardown è per-worker ⇒ aliasa nella
   pendenza; (b) Bak, i marginali "9/9 IN fascia" sono estimator-dipendenti —
   con la mediana W8→12 = 22.183.936 (OUT alto) e W12→16 = 13.271.040 (OUT
   basso), e la dispersione a W12 è ~11% del livello. Citabile con l'etichetta
   ADVISORY esplicita e la banda; mai come cifra di verdetto.

3. **b_peak = 19.723.059 B/worker — DA RIFARE (declassata da testata).**
   Stimatore d'estremo spacciato per modo dominante; il controllo di robustezza
   min-of-R è VOID per costruzione (KS-BB-92-2). Ricomputabile subito sulle raw
   esistenti come mediana (~20,29M attesi) o punto additivo per-run, con la run
   W12 r3 flaggata per NOME (A-BB67).

4. **b_boot+b_work = 19.529.728 B — miglior FORMA di testata (Bak), ma eredita
   il grado della lane più debole: ADVISORY.** La continuità con b m89
   (19.575.603, Δ 45.875 B) è **suggestiva, non verdict-grade**: due numeri
   ADVISORY che si somigliano non fanno una conferma. Va citata come coerenza
   di ordine, mai come conferma di P-RET0 o di alcunché.

5. **Il residuo 193.331 non è citabile con la label attuale** (CV-3): va
   rietichettato "selection mismatch cross-lane" (A-BB65) e l'additività va
   giudicata **within-run**, dove è esatta, con il residuo cross-lane riportato
   a parte.

6. **Correzione formale trasversale**: con 4 punti (df=2) "2σ⇒95%" è falso
   (t(0,975;2)=4,30 ⇒ ~82%). Tutte le bande vanno ripubblicate come ±2se
   dichiarato o ±t(df)·se (A-BB68). Questo tocca ogni figura del doc, incluse
   quelle che restano.

**Condizione di promozione a verdict-grade per b_work**: riga pwork con
`outstanding=0` da `census::OUTSTANDING` (A-PP-63) **e** punto di lane preso
con stimatore non-tautologico (A-BB64/66) **e** censimento per-worker
on-thread senza collisione di heap ptr (A-DL-52/KS-DL-92-1). Le tre condizioni
sono congiunte: due su tre lasciano b_work ADVISORY.

---

## PRIORITÀ ITERAZIONE 3 (attribuzione di b) — ordinate per POTERE DISCRIMINANTE

Criterio: quanto l'intervento cambia la CLASSE di conclusioni ottenibili, non
il suo costo. Il costo è annotato a parte perché l'ordine di ESECUZIONE
temporale differisce (vedi P0).

**P0 (da eseguire per primo, costo ≈ 3 righe, potere alto per unità di costo)**
— testimone di quiescenza `outstanding=0` sulla riga pwork (A-PP-63) + `-f` e
pid-echo su tutti i census (A-PP-62) + `LC_ALL=C` (A-PP-64). Decide da solo se
b_work è ADVISORY o promuovibile. Contatore e header esistono già.

**P1 — CENSUS ON-THREAD PER-WORKER ALLA BARRIERA DI PICCO (A-DL-52, dente
KS-DL-92-1).** Potere massimo: è l'unico intervento che cambia la NATURA della
stima. Oggi b è una regressione cross-lane su 4 punti scelti da run diverse;
con lo snapshot preso dal thread del worker su `mi_heap_get_default()` +
`mi_heap_visit_blocks`, b diventa una misura **per-worker, per-run**. In un
colpo: (a) chiude la collisione `mi_heap_of` — l'indirizzo identico per tutti i
thr (0x105c3ae80 a w16.r1, 0x10562ee80 a w4.r1, diverso per RUN non per thread)
è `_mi_heap_main` letto dalla TLS del chiamante, non pagine condivise, provato
dal jitter non monotono ±20 KB degli `used_b`; (b) elimina il mismatch di
selezione (CV-2, CV-3) perché l'additività torna within-run; (c) rende il
worker quiescente rispetto alla propria richiesta *per costruzione* alla
barriera, sussumendo in gran parte P0; (d) rende finalmente eseguibile
l'ordine WP-91 sulla pendenza del PICCO. Fallback in ordine: heap espliciti per
worker (`mi_heap_new` + allocazioni VM heap-scoped), poi merge stats tld
at-exit (richiede patch al crate).

**P2 — RIPARAZIONE DEGLI STIMATORI SUI 20 RAW GIÀ COMMITTATI (A-BB64, A-BB65,
A-BB66, A-BB67, A-BB68).** Potere alto e retroattivo, zero run nuove:
ripubblica b_peak con mediana/punto additivo, flagga W12 r3 per NOME,
sostituisce il controllo di robustezza tautologico con min/mediana/media
(misurato: 0,972), rietichetta il residuo, corregge le bande. Discrimina se le
conclusioni di WP-90 **sopravvivono al cambio di stimatore** — e i marginali
già dicono di no in due punti su nove. Da fare PRIMA di qualunque run nuova:
se le conclusioni non reggono sui dati esistenti, la campagna nuova
misurerebbe la cosa sbagliata meglio.

**P3 — CHIUSURA DEL BILANCIO DI COPERTURA E SCOMPOSIZIONE DI b_boot (A-DL-55,
A-DL-56).** Potere sulla COMPOSIZIONE, non sulla magnitudine: `malloc_huge`
CURRENT (non total), `pages_abandoned`, free-committed in arena, classi
per-chunk in `mi_arena_chunk_bins` — l'arena si cammina senza TLS, quindi
questo braccio è indipendente da P1 e può girare in parallelo. Il braccio
bare-thread (W thread che toccano mimalloc, zero init phpr) separa
thread-runtime da stato phpr dentro b_boot. Senza P3 l'attribuzione di b
resta un numero senza composizione (CV-6).

**P4 (sotto la linea, ma da non perdere)** — overlap forzato a barriera
(A-DL-54) per rendere l'overlap controllato invece che emergente; lettura
purged/purge_calls ai bordi dello span sul worker path (A-DL-53, chiude A-DL50
e toglie l'aritmetica dal posto della lettura); disciplina failpath (A-PP-60/61)
e supersessione del giudice in ledger (A-PP-65, KS-PP-92-2 — g1→g2 è avvenuto
con judge_sha 1b1e9e96→97a8eff0 e `supersede_of=g1` **senza riga che autorizzi
la sostituzione**: catena gG fuori banda).

---

## KILL-SWITCH ADOTTATI DAL TEAM (unione, nessuno ritirato)

KS-BB-92-1 (lane con modes==R su tutti i punti ⇒ mai cifra di testata) ·
KS-BB-92-2 (check di robustezza che non può fallire ⇒ VOID, verdict declassato)
· KS-PP-92-1 (Δ di fase senza testimone outstanding==0 ⇒ mai verdict-grade) ·
KS-PP-92-2 (judge_sha ≠ pinned a phase=start senza autorizzazione ⇒
generazione VOID) · KS-DL-92-1 (heap ptr duplicato tra worker ⇒ REFUSE, mai
media) · KS-DL-92-2 (claim sul clamp senza riga overlap-status per span ⇒ VOID).

---

# TEAM-SIGILLI — Concilio WP-92 (relatore su verbali 1-Hoare, 2-Matsakis)

Fonti VINCOLANTI: `verbale-1-hoare.md`, `verbale-2-matsakis.md`. Questo
documento riconcilia e ordina; non emenda né attenua i verbali.

Verdetti individuali: entrambi **CONCORDO CON EMENDAMENTI**; entrambi
dichiarano **nessuna refutazione capitale** — le cifre di S-90.0 restano
in piedi, i buchi sono nelle GUARDIE IN AVANTI. Il team conferma: nulla
qui declassa una figura già emessa; tutto qui deve mordere PRIMA di
S-91.0.

## CONVERGENZE

1. **Il pin `==2` sui siti arm() non è un census.** Stesso vizio da due
   lati. Hoare Q2.1: `narm` (gate-lever-pins 1346) riconosce solo
   `let <nome> = ProbeWindow::arm()` — `let mut w =`, `let w: ProbeWindow =`,
   `let (w,_) =`, `drop(ProbeWindow::arm())`, arm in posizione
   d'espressione sono invisibili sia a `narm` sia a `nsil`: un TERZO sito
   passa con narm==2/nsil==0. Il pin morde la CONVERSIONE dei 2 siti noti
   (2→1 FAIL), non l'AGGIUNTA. Matsakis Q3: la cintura A-MS41 conta le
   grafie `.set(`, non i call-site — formalmente regge, ma la semantica
   si è indebolita (`probe_active=true` non identifica più UN solo
   strumento). Rimedi **complementari, non alternativi**: A-TH-57 (census
   totale lessicale, `grep -c 'ProbeWindow::arm'`, ==2) chiude
   l'aggiunta a VISTA; A-MS-54 (nesting-tooth: `debug_assert!(!flag)` o
   contatore in `arm()`) chiude l'annidamento a ESECUZIONE. Nessuno dei
   due copre il buco dell'altro: un terzo sito scritto in grafia esotica
   sfugge al dente runtime finché non è annidato; un annidamento tra i
   due siti già contati sfugge al census. Kill-switch coerenti e
   cumulativi: KS-TH-92-1 (ntot≠narm ⇒ figure con probe VOID) e
   KS-MS-92-3 (nuovo sito arm senza dente anti-nesting nella STESSA
   delibera ⇒ attribuzione VOID).

2. **Ciò che il compilatore può uccidere non va inseguito con la regex —
   ma la belt resta finché il recinto non c'è.** Hoare Q2.2 trova la
   QUINTA grafia su `CENSUS_PROBE_ACTIVE` (`with(Cell::set)`,
   `with(|f| Cell::set(f,true))`, corpo a graffe `|f| { f.set(true) }`)
   invisibile alle quattro reti A-MS48 (A-MS-emend). Matsakis conferma
   dal lato tipo/visibilità (linea A-MS46, WP-91): il perimetro di
   privacy è ancora l'intero `mod implementation` (~1900 righe, test
   inclusi). Convergenza: la quinta rete è una toppa a vita corta, ma
   dovuta finché A-MS46 non è in albero.

3. **Un abort/una guardia devono essere ATTRIBUIBILI al sito.** Matsakis
   Q2.1: i due `eprintln` di abort (teardown r.386-412 e `/__census_self`
   r.521-535) hanno stringhe IDENTICHE ("A-MS31; instrument, not
   worker") ⇒ forensics di campagna abortita cieca sul sito (A-MS-52,
   KS-MS-92-2). Stessa famiglia di Hoare Q3.1: `npdef` conta solo
   `^probe_in()` a colonna 0 ⇒ una ridefinizione indentata o in forma
   `function probe_in {` passa (A-TH-60, KS-TH-92-2): in entrambi i casi
   **due entità distinte condividono la stessa impronta osservabile** e
   il ledger non può separarle. Regola comune: ogni sito/definizione
   duplicabile porta un tag proprio (stringa distinta, `&'static str`),
   e il conteggio è di CENSUS (tutte le grafie), non di forma canonica.

4. **Il cap/la forma DICHIARATA deve essere quella che morde.** Matsakis
   Q1.1: la guardia theap confronta `capacity()`, non `MAX_THEAPS`
   (memcensus r.1147) — `reserve_exact` può concedere >64 slot, quindi
   `heaps_total` può superare 64 con `heap_overflow=0`: il "declared cap
   64" non è il cap effettivo (A-MS-51, KS-MS-92-1). Hoare Q3.2 dice la
   stessa cosa sul form-pin noprobe: `nform==1` è di FORMA, non di
   RAGGIUNGIBILITÀ (`false && strings … | grep -q` lo soddisfa a
   rilevazione svuotata) — residuo fuori portata lessicale, ma **va
   DETTO** nel testo dei sigilli, non lasciato implicito.

5. **I contratti assunti vanno scritti come pin, non raccontati.**
   Matsakis Q1.3: identità heap = `last_mut()` è corretta solo se visita
   heap e visita aree sono strettamente sequenziali — contratto C
   assunto, mai dichiarato; e lo scarto del puntatore `_h` rende
   IMPOSSIBILE il join con `mi_bin_thr_sum` (A-MS-53, rilevante per la
   collision VSELF / iter-3). Hoare Q4: la doc TLS A-TH54 omette il
   terzo stato (chiave RefCell MAI inizializzata sul thread, primo
   accesso in teardown — std non garantisce) e il percorso silente è un
   canale di **scrittura persa** (uc_log in buffer che muore senza
   flush), non solo di mancato panic (A-TH-61). Convergenza: enumerazioni
   incomplete e contratti impliciti sono lo stesso difetto.

6. **Il pin A-MS47 non è l'unica lettera esposta al refactor** (nodo
   emerso dalla riconciliazione, vedi CONFLITTI/§ordine): il `narm`
   attuale esige `ProbeWindow::arm\(\)` a parentesi VUOTE.

## CONFLITTI E TENSIONI DI METODO

**Nessun conflitto frontale.** Entrambe le sedie assolvono le cifre di
S-90.0 e attaccano solo le guardie. Restano tre tensioni, tutte
risolvibili per composizione:

- **T1 — Belt vs recinto (ricorrente da WP-91).** *Hoare*: estendere la
  belt lessicale (A-TH-58 use monoriga `as`, A-TH-59 dot-path tollerante
  ai commenti + stato nome-pendente su tre righe, A-TH-57 census,
  A-TH-60 census probe_in): il rilevatore di produzione deve riconoscere
  la grafia PIÙ NATURALE, che oggi è l'unica scoperta (`use crate::vm::CachedUnit as CU;`).
  *Matsakis*: spostare il giudizio dove muore a compilazione o a
  esecuzione (A-MS46 mod probe annidato; A-MS-51 `Box<[TheapAgg]>` a
  lunghezza fissa; A-MS-54 debug_assert), tenendo la belt come cintura.
  Composizione: la belt copre il residuo, il recinto riduce il residuo.
  Non è una scelta, è un ordine.

- **T2 — A-MS46 riduce la superficie dei sigilli? SÌ, ma solo su UNA
  delle tre famiglie.** Valutazione del team:
  * **Scrittura del flag** (A-MS41 `.set(` ==2, A-MS48 quattro reti +
    A-MS-emend quinta): superficie da ~1900 righe a ~40 ⇒ riduzione
    REALE; dopo A-MS46 le grafie point-free/UFCS/graffe restano
    tecnicamente eludibili ma dentro un modulo che un umano legge
    interamente, e i test del modulo muoiono a compilazione. La belt
    scende da giudice a cintura.
  * **Siti `arm()`**: nessuna riduzione. `arm()` resta esportato
    `pub(super)` e i call-site (2 oggi: worker_pool 385, 520) vivono
    nell'intero `implementation`. A-TH-57 e A-MS-54 NON sono assorbiti
    da A-MS46 e vanno fatti comunque.
  * **`probe_in` / noprobe** (A-TH-60, KS-TH-92-2, form-pin): superficie
    bash dell'harness, ortogonale al refactor Rust. Zero riduzione.
  Conclusione: A-MS46 è un moltiplicatore per la famiglia 1, neutro per
  le altre due ⇒ **non è un motivo per rinviare i sigilli-subito**.

- **T3 — Vincolo di SEQUENZA scoperto in riconciliazione (nuovo, non in
  nessuno dei due verbali).** A-MS-52 e A-MS-54 convergono su una sola
  modifica di firma: `arm()` che porta un `&'static str` di sito (lo dice
  Matsakis stesso in A-MS-52) più il dente anti-nesting. Ma quella firma
  **rompe la lettera dei pin attuali**: `narm` (r.1346) esige
  `ProbeWindow::arm\(\)` a parentesi vuote ⇒ con l'argomento diventa 0 e
  il gate FAIL-CLOSED (comportamento corretto, ma il gate va aggiornato
  nella STESSA delibera, disciplina A-PP48). Corollario vincolante per
  A-TH-57: il census va scritto in forma **prefisso** (`ProbeWindow::arm`
  senza parentesi), altrimenti nasce già cieco alla propria evoluzione.
  Stesso vincolo su A-MS46: spostando flag/ProbeWindow in `mod probe`,
  le finestre awk e i pin `npriv` di A-MS43/A-MS48 sono ancorati alla
  forma odierna e vanno riancorati nello stesso commit.

- **T4 — Classificazione di A-TH53 (disaccordo con la bozza d'ordine).**
  Il brief lo mette tra i DESIGN; design90.md §A-TH53 assegna però
  l'attuazione del pin Cargo.toml alla «prossima revisione dei sigilli».
  Posizione del team: la **ratifica di policy** è design (già fatta in
  WP-91), il **pin** è meccanico (due `grep -c` su
  php-runtime/php-server Cargo.toml, ==0) ⇒ va nei sigilli-subito. Il
  residuo dichiarato (grafia 7, macro a incollaggio) resta fuori portata
  lessicale fino ad A-MS27.

## PRIORITÀ PER S-91.0

### SIGILLI-SUBITO (meccanici, solo script/doc, chiudono kill-switch attivi)
Nessuno tocca runtime; tutti eseguibili senza il ciclo di build.

1. **A-TH-57 — census totale arm**, `ntot = grep -c 'ProbeWindow::arm'`
   su righe di codice, pin **==2**, in forma PREFISSO (no `\(\)`), con
   FAIL se `ntot != narm`. Chiude Q2.1 e arma KS-TH-92-1. *Primo perché
   è il buco che lascia entrare un TERZO strumento senza accorgersene.*
2. **A-TH-60 — census delle definizioni `probe_in`**,
   `grep -cE '^[[:space:]]*(function[[:space:]]+)?probe_in'` ==1.
   Chiude Q3.1 e arma KS-TH-92-2 (decoy-front/gutting-behind).
3. **A-MS-emend — quinta rete su `CENSUS_PROBE_ACTIVE`**: point-free
   `with(Cell::set)`, UFCS-Cell `with(|f| Cell::set(f,…))`, corpo a
   graffe `|f| { f.set(true) }`. Toppa a vita corta (cade con A-MS46) ma
   dovuta nell'interim.
4. **A-TH-58 — use monoriga con `as`**: ramo `use [^;{]*(CachedUnit|VmGate) as `.
   È la grafia più naturale ed è l'unica scoperta.
5. **A-TH-59 — dot-path tollerante + tre righe**: skip commenti/blank
   sul ramo dot e stato nome-pendente in TH49ML_PROG (Q1.2-1.4).
6. **A-TH53 (pin) — bando `paste`/`concat_idents`** nei Cargo.toml di
   php-runtime/php-server, ==0 (vedi T4).
7. **A-TH-61 — doc A-TH54, terzo stato**: chiave TLS mai inizializzata
   al teardown + canale di scrittura PERSA (uc_log senza flush). Costo
   zero, KH91-1 già attivo sui claim.
8. **Residui DICHIARATI** (non chiudibili lessicalmente, da scrivere nel
   commento dei sigilli accanto alla lane A-TH53/A-MS27): form-pin
   noprobe non è raggiungibilità (Q3.2); `::` spaziato in posizione di
   TIPO (firma/return/turbofish) non è mint diretto ma è canale d'alias
   (Q1.5).

**Regola di ordine interna**: 1-2 prima di ogni altra cosa (sono i due
che governano kill-switch che rendono VOID figure di campagna); 3-5 in
qualunque ordine; 6-8 in coda, costo nullo.

### DESIGN (vanno COL CODICE, stesso commit del dente — disciplina A-PP48)

Ordine proposto, dal recinto che riduce superficie al dente che misura:

- **D1 — A-MS46 `mod probe` annidato** (~40 righe, `pub(super)`).
  PRIMO tra i design: è l'unico che RIDUCE la superficie dei sigilli
  (famiglia 1, vedi T2), e fa decadere KS-MS-91-1 a compilazione.
  Vincolo: riancorare finestre awk / pin `npriv` di A-MS43/A-MS48 nello
  stesso commit; A-MS41/45/47/48 restano come belt.
- **D2 — A-MS-52 + A-MS-54 insieme** (unica modifica di firma: `arm(site:
  &'static str)` + `debug_assert!(!flag)`/contatore; tag di sito nei due
  eprintln e nella riga del panic-hook). Vincolo T3: aggiornare `narm`
  (r.1346) e A-TH-57 nello stesso commit; senza dente anti-nesting il
  terzo sito è VOID per KS-MS-92-3.
- **D3 — A-MS-51 cap effettivo**: guardia su `len()==MAX_THEAPS` o
  `heaps: Box<[TheapAgg]>` a lunghezza fissa; arma KS-MS-92-1. Piccolo,
  ma è una LETTERA in-band oggi falsa ("declared cap 64").
- **D4 — A-MS-55 REQ_NS esclude `/__census_self`**: `req_t0` non deve
  avvolgere il ramo census_self (r.507-552). Tocca il canale delle
  figure per-request ⇒ da coordinare col team-misura.
- **D5 — A-MS-53 `heap=<ptr>` in-band nelle righe `mi_theap_pages`**:
  prerequisito del join anti-collisione con `mi_bin_thr_sum` ⇒ serve a
  VSELF iter-3. Rinviato al team-misura per la priorità relativa (è un
  ingrediente di misura, non un sigillo).
- **D6 — A-TH55 ordine canonico KIND intra-putord** in a_ds26/a_ds38:
  invariato, va con la prossima sessione che tocca quei test.
- **Aperti fuori perimetro sigilli**: contratto di sequenzialità della
  visita C (Q1.3) e headroom di stack per `TheapAgg::zeroed()` ~3,9 KB
  su frame `extern "C"` con `RUST_MIN_STACK` non pinnato al blocco
  (Q1.2) — da instradare al team-misura come pin di canale.

## NOTA AL TEAM-MISURA
A-MS-53 (heap ptr in-band, VSELF iter-3), A-MS-55 (inquinamento REQ_NS
da census_self), A-MS-51 (cap dichiarato ≠ cap effettivo ⇒ KS-MS-92-1
sul canale theap) e i due pin di canale sopra toccano figure, non
sigilli: la priorità relativa spetta a loro.

— relatore team-sigilli, Concilio WP-92

---

# team-engine — Concilio WP-92 (relatore: sedia 8 Stogov, team monocratico)

FONTE VINCOLANTE: `verbali/verbale-8-stogov.md` (VERDETTO: CON EMENDAMENTI,
A-DS53…A-DS57, KS-DS-92-1/2/3). Vincoli di merge integrati dalle sezioni
Emendamenti/Kill-switch di `verbale-3-klabnik.md` e `verbale-9-gregg.md`
limitatamente a dove pesano sull'implementazione A-DS51.

---

## 1. SINTESI DEI VINCOLI — 10 vincoli che l'implementazione LSP deve rispettare

**V1 — contratto by-ref EMENDATO (A-DS48, invariato).** L'esattezza è sulla
REF-NESS, non sul tipo: togliere *o* aggiungere `&` = fatal in entrambe le
direzioni; il TIPO resta contravariante, quindi il widening `int &$x` →
`int|string &$x` è **alive** (v1) e la narrowing è fatal (v2). Il checker non
deve trattare la by-ref come "identità di firma". Le union nei messaggi vanno
nell'ordine canonico Zend (`P::m(string|int &$x)`), non nell'ordine sorgente.

**V2 — esenzioni ctor SOLO ereditarie.** Il costruttore è esente dal check LSP
quando la sede è una **classe plain**; NON è esente quando il genitore è
un'**interfaccia** (v13) o una **classe abstract** (v14). Un'esenzione
`is_constructor ⇒ skip` piatta fa fallire due pin per NOME. Insieme al ctor
vanno nello stesso commit le altre esenzioni: `private`, RTWC (return-type
will change), tentative-Deprecated (A-DS46).

**V3 — timing t1-t4, con t3 a exit 0 come NEGATIVO obbligatorio.**
Dai raw `ds35-verify2.out`: t1 (parent dichiarato DOPO il figlio) fatala
comunque ⇒ il check è sul link hoisted, non sull'ordine testuale; t2
(condizionale eseguita) fatala **dopo** `pre|`; **t3 (condizionale non
eseguita) esce 0 con `pre|post` su ENTRAMBI i bracci** ⇒ una fase che fatala t3
è un falso positivo che rompe codice legale (KS-DS-92-3); t4 discrimina la
forma del lowering (vedi V4).

**V4 — bersaglio t4 = braccio PERSIST, dichiarato per NOME (A-DS55).**
Refutazione capitale: «stdout integrale dell'oracle» **non nomina il braccio**,
e su t4 i due bracci DIVERGONO ai byte —
`plain` emette `pre|\n` prima del fatal, `persist` fatala **pre-output**
(nessun `pre|`). phpr con lowering hoisted fatalerà pre-output ⇒ il bersaglio
byte-fedele è il braccio **persist**, e il pin deve scriverlo per nome.
Un pin che dice solo "stdout dell'oracle" è ambiguo per costruzione su t4.

**V5 — v15 mai skip silenzioso (A-DS56).** Contraddizione ereditata sciolta:
l'appendice WP-89 («skip conservativo su nomi non risolvibili») **cede** a
§3.3-quinquies («mai skip silenzioso»); prevale la delibera WP-91. v15
(`m(): B` vs `m(): A`, classi inesistenti) **fatala** — messaggio di famiglia
propria: `Could not check compatibility between C::m(): B and P::m(): A,
because class B is not available`, exit 255 su entrambi i bracci. La fatal è al
**bind hoisted**; per i bind dinamici il giudice è il gate ORM/hk, non una
decisione a tavolino.

**V6 — pin v3 a byte-count per canale; comparatore a marcatori BANDITO
(A-DS54 / KS-DS-92-1).** Refutazione capitale: `echo "x\n--stderr\ny--end"` è
PHP legale e il suo **stdout contiene i marcatori** ⇒ un comparatore a scansione
di marcatori è forgiabile. Il buco è già visibile nei raw v2: le righe
`alive--stderr` e `pre|post--stderr` (stdout senza newline finale incollato al
marcatore) sono decodificabili solo *conoscendo* i marcatori. Formato v3:
`--stdout bytes=N` / `--stderr bytes=N` per canale, parse = lettura di **N byte
esatti**. Nessuna euristica di riga.

**V7 — pin a CANALI SEPARATI (A-DS50, confermato).** stdout FULL, stderr FULL,
exit code, **per braccio**; mai `2>&1`, mai troncamento. Divergenza a catalogo
confermata: phpr **non** modella la log-copy stderr dell'oracle
(`PHP Fatal error:  …`, doppio spazio) ⇒ **stderr phpr atteso VUOTO**. Il
comparatore confronta stdout-vs-stdout e exit-vs-exit, e asserisce
`stderr_phpr bytes=0`; non confronta lo stderr dell'oracle con nulla.

**V8 — sette buchi di copertura da colmare PRIMA del merge (A-DS53, fixture
v3).** Tutti oracle-morsi dal vivo su 8.5.7: (1) interfacce multiple in
conflitto; (2) interface-extends-interface incompatibile (fatal **senza alcuna
classe**: il check vive anche nel linking di interfacce); (3) enum implements
(gli enum sono assenti dall'intero set); (4) `self` vs `static` — vincolo di
modellistica su `TypeHint::display_name` (`crates/php-runtime/src/hir.rs:603`):
`self` si stampa come **nome-classe risolto**, non «self»; (5) property hooks —
famiglia di messaggio DIVERSA: `Type of C::$x must be subtype of int (as in
class P)`, «subtype of», non «must be int»; (6) costanti final
(`C::X cannot override final constant P::X`); (7) readonly PROMOTED (stesso
messaggio di v11, path di lowering diverso). Più il **positivo di regressione**
DNF `(A&B)|string → string` (legale, verificato). KS-DS-92-2: merge di A-DS51
senza le fixture A-DS53 **per NOME** nel gate = REJECT.

**V9 — vincoli di evidenza da Klabnik (dove toccano A-DS51).** A-SK-67: il
giudice, il manifest e il budget si leggono da **HEAD**, e un working tree ≠
HEAD ⇒ FAIL (KS-SK-92-1: PASS con blob ≠ HEAD ⇒ VOID) — quindi `ds35-verify3.sh`,
le fixture v3 e il comparatore byte-count vanno **committati prima** della run
che li consuma, non generati durante. A-SK-71: il perimetro del manifest è
**ogni .md committato che pubblica cifre di sessione** — il conteggio fixture
(37→44+) e i numeri di gate che finiranno in `NEXT_SESSION`/`sessions/`/design
richiedono riga di manifest (KS-SK-92-3: cifra fuori perimetro ⇒ non
verdict-grade anche se il MEASURE passa). A-SK-69: se si cita una derivata
(es. «N/M fixture verdi»), operandi dallo **stesso file** e operatore
verificato.

**V10 — vincoli di evidenza da Gregg (dove toccano A-DS51).** A-BG58:
`reason=` autosufficiente — ogni riga verdict/supersede sul canale LSP deve
portare `reason=requalify:<blocco>:<old→new>` e il ledger da solo deve
rigenerare la storia da fase a fase (KS-BG-92-2: reason che non ricostruisce la
riqualifica senza aprire file esterni ⇒ supersede invalido alla campagna
successiva). A-BG59: mai ereditare un campo da una riga precedente — ogni riga
di esito fixture porta i propri campi o è FAIL.

---

## 2. PIANO S-91.0 — TRE FASI ORDINATE (A-DS57, ordine vincolante)

L'ordine è scelto per **rischio ORM/hk minimo**: l'ordine inverso
(registry prima) lascerebbe scoperti 35/37 pin nella finestra intermedia.

### FASE 0 (pre-requisito, nessun runtime) — fixture v3 + pin v3, committati
- Scrivere le **8 fixture A-DS53** (7 buchi + positivo DNF) in
  `wp91-harness/fixtures-ds35-v3/`.
- Riscrivere il pin in formato **byte-count per canale** (V6) e ri-generare
  l'oracle-pin sull'intero set **v1+v2+v3** (18+19+8 = 45 fixture), con il
  bersaglio t4 dichiarato `braccio=persist` per NOME (V4).
- **Committare fixture + script + pin PRIMA** di qualunque run che li consumi
  (V9/A-SK-67).
- *Passa se*: 45/45 pin oracle riprodotti a byte-count; il comparatore
  auto-morde su un caso forgiato (`echo "x\n--stderr\ny--end"`) e lo **rifiuta**
  — se non morde, il comparatore non è ancora un giudice.
- *Gate*: nessuno (nessuna riga di runtime cambia).

### FASE 1 — checker LSP PURO + unit sui messaggi (nessun wiring)
- Modulo checker isolato: dato (firma parent, firma child, contesto sede),
  restituisce `Ok` o l'**esatta stringa di messaggio** Zend.
- Copre: return covariante/contravariante, param contravariante, union in
  ordine canonico Zend, by-ref per V1, `self`/`static` risolti via
  `TypeHint::display_name` (V8/4), nullable, variadic, mixed, void,
  readonly, prop-typed-on-untyped, hook-subtype (famiglia di messaggio
  distinta), final-const, non-risolvibile (V5).
- **Nessuna chiamata dal lowering**: il checker esiste ma non è armato.
- *Fixture che devono passare*: tutte le 45 a livello di **unit sul messaggio**
  (`cargo test --release`), confrontate contro le stringhe estratte dai pin —
  non contro stringhe ribattute a mano.
- *Gate che gira*: `cargo test --release`. Nessun gate di merge pesante: la
  superficie di runtime è invariata (nessun handler nuovo nel run_loop).
- *Controllo positivo obbligatorio*: un contatore tutto-verde al primo giro è un
  controllo positivo FALLITO — almeno una fixture deve rossare prima del fix.

### FASE 2 — wiring nel lowering HOISTED (+ esenzioni nello STESSO commit)
- Sede: `crates/php-runtime/src/lower/class.rs`, dopo il **flatten dei trait**,
  **accanto al final-check** (`Cannot override final method`, ~riga 929,
  vicino a `final_ancestor_method` / `check_readonly_extends`).
- **Nello stesso commit** le esenzioni (V2): ctor-plain-class, private, RTWC,
  tentative-Deprecated (A-DS46) — con la nota che il ctor **non** è esente per
  interfacce/abstract (v13/v14). Una fase 2 che sposta le esenzioni a un commit
  successivo apre una finestra in cui ORM/hk fatalano su codice legale.
- **Le condizionali restano FUORI** da questa fase: t2 non fatala ancora, ed è
  atteso; **t3 = REJECT se fatala** (KS-DS-92-3).
- *Fixture che devono passare*: le 45 meno t2 (che resta al comportamento
  pre-fase-3), con **t1 fatal**, **t3 exit 0 `pre|post`**, **t4 byte-fedele al
  braccio persist** (nessun `pre|`, KS-DS-92-3 esplicita: fase 2 che manca il
  persist-shape di t4 = REJECT), **v15 fatal** con il messaggio
  `Could not check compatibility …`.
- *Gate che gira*: **gate ORM 3E/13F + hk + corpus Zend per NOME (1418) + refl
  290** — per NOME, mai per conteggio.

### FASE 3 — bind-registry per condizionali e dinamiche
- Registry dei bind non-hoisted: quando la dichiarazione condizionale viene
  **eseguita**, il check scatta al bind (t2), mantenendo `pre|` già emesso.
- *Fixture che devono passare*: **tutte e 45**, t2 inclusa — t2 stdout
  `pre|\n` + fatal, exit 255; t3 **ancora** exit 0 `pre|post` (il registry non
  deve armare i rami non presi).
- *Gate che gira*: **di nuovo ORM 3E/13F + hk + corpus per NOME + refl 290**,
  in particolare per i bind **dinamici**, che per V5/A-DS56 sono giudicati dal
  gate e non da una decisione a tavolino.

---

## 3. RISCHI SUI GATE DI MERGE + MOSSA DI MITIGAZIONE

| # | Rischio | Perché è reale | Mitigazione |
|---|---|---|---|
| R1 | **ORM (baseline 3484, 3E/13F) regredisce in fase 2** — Doctrine genera proxy e usa reflection pesante su gerarchie con ctor promossi/interfacce | il check LSP è **nuovo codice che può solo AGGIUNGERE fatal**: ogni falso positivo è un E in più | esenzioni nello STESSO commit (V2); gate ORM **per NOME** subito dopo fase 2, prima di toccare la fase 3; se compare un E nuovo, la mossa è **restringere l'esenzione mancante**, non disarmare il checker (direttiva NIENTE REVERT) |
| R2 | **hk (~1665) regredisce sui bind dinamici** — Symfony http-kernel istanzia via container/factory | fase 3 arma il check su percorsi che fase 2 non tocca | fase 3 **isolata in un commit proprio** con gate hk immediatamente dopo; il gate hk è il giudice dichiarato dei dinamici (V5), quindi il suo esito va **ledgerato** anche se PASS |
| R3 | **corpus Zend 1418 / refl 290 scendono e il conteggio resta uguale** (set diverso, cardinalità identica) | è il forge classico che la regola gate-per-NOME esiste per chiudere | diff del **set di nomi**, mai del conteggio; `--list-fails` con confronto insiemistico contro il baseline `phpr-wp89` (64e9e51c281de6d1) |
| R4 | **Baseline hk incerto (1663 in memoria vs 1665 citato)** | un baseline sbagliato produce sia falsi verdi sia falsi rossi | **ri-pinnare il baseline dal run verde più recente PRIMA di fase 2**, committarlo, e citarlo per sha; mai dal ricordo |
| R5 | **Il pin di fase 0 viene rigenerato durante la run che lo consuma** | KS-SK-92-1: PASS con blob ≠ HEAD ⇒ VOID — l'intera fase 2/3 diventerebbe non-verdict-grade | fixture + script + pin **committati prima**; il comparatore stampa `judge_sha` e verifica working == HEAD, altrimenti FAIL (V9) |
| R6 | **Il comparatore byte-count viene consegnato senza auto-morso** | un giudice che non ha mai rifiutato nulla non è un giudice (lezione ricorrente: un gate che MORDE vale più di dieci che benedicono) | test negativo committato nello stesso commit: la fixture forgiata `echo "x\n--stderr\ny--end"` deve essere **rifiutata** dal comparatore v3 e **accettata** (falso verde) da quello a marcatori, a prova che la sostituzione era necessaria |
| R7 | **Le cifre di copertura LSP finiscono in NEXT_SESSION/sessions senza riga di manifest** | KS-SK-92-3: cifra fuori perimetro ⇒ non verdict-grade anche col MEASURE verde | riga di manifest per ogni .md che pubblica il conteggio fixture o gli esiti gate; derivate con operandi dallo **stesso file** (A-SK-69) |
| R8 | **La storia fase1→fase2→fase3 non è ricostruibile dal solo ledger** | KS-BG-92-2: supersede invalido alla campagna successiva | `reason=requalify:<blocco>:<old→new>` su ogni riga verdict/supersede; nessun campo ereditato da riga precedente (A-BG59) |

**Rischio principale** (uno solo, se se ne deve nominare uno): la fase 2 arma
un check che può **solo aggiungere fatal** su tre gate di merge grandi, e ogni
esenzione mancante si manifesta come regressione ORM/hk — per questo le
esenzioni viaggiano nello stesso commit del wiring e il gate gira **dopo ogni
fase**, mai una volta sola alla fine.

---

## Riferimenti verificati (raw, non ricordi)

- `wp90-harness/ds35-verify2.out` — t4 plain con `pre|`, t4 persist pre-output
  (righe ~867-895); t3 exit 0 `pre|post` entrambi i bracci (~852-866); v15
  fatal 255 entrambi i bracci (~765-790); `alive--stderr` / `pre|post--stderr`
  = prova viva dell'ambiguità del comparatore a marcatori.
- `wp90-harness/fixtures-ds35-v2/` — 19 fixture v2 (t1-t4, v1-v15).
- `crates/php-runtime/src/lower/class.rs` — sede del wiring (final-check
  ~929, `final_ancestor_method` 609, `check_readonly_extends` 627, flatten
  trait 202).
- `crates/php-runtime/src/hir.rs:603` — `TypeHint::display_name`, punto di
  intervento per `self`/`static` risolti (V8.4).
- Assenza confermata: nessun «must be compatible with» per metodi nel codice
  attuale (solo hook `get()`), cioè il checker **non esiste ancora**.

---


---

## ⚖️ SINTESI DI CONVERGENZA (compilata dalle ricevute fase 1+2 + estrazioni mirate; verbali = fonte vincolante)

**Verdetto complessivo: 9× CON EMENDAMENTI — nessuna opposizione. Nei
team: UN conflitto vero (team-misura, sul GRADO della testata),
composto a strati senza dissenso; nessun conflitto frontale altrove,
ma nove tensioni di collocazione registrate.** QUATTRO refutazioni
capitali, tutte morse A MACCHINA:

### Refutazioni capitali

1. **🔴 QUATTRO forge PASSATI dal gate cifre vivo (Klabnik)** — il
   fix WP-91 non ha chiuso la classe: (a) `[derivata: prov …]`
   RE-LEGALIZZA il forge WP-91 (la chiusura delle differenze fra
   operandi risolti resta al 46,25% del vicinato plausibile);
   (b) `legacy_frozen` è concesso da un manifest letto dal WORKING TREE
   (non da HEAD): un manifest non committato regala la semantica
   legacy; (c) la cache `--cache/--nonce` è avvelenabile
   dall'INVOCANTE (il nonce autentica il padre, non il contenuto);
   (d) 43 cifre pubblicate vivono FUORI dal perimetro manifest. →
   A-SK-67 (manifest/budget/giudice letti da HEAD, judge_sha stampato),
   A-SK-68 (legacy_frozen ABOLITO), A-SK-69 (prov commensurabile:
   operandi dallo stesso file/campagna, operatore parsato),
   A-SK-70 (cache abolita o rifiutata se fornita), A-SK-71 (perimetro
   esteso), A-SK-72 (supersessione provata da riga committata),
   A-SK-73 (budget anche sul pool prov) — KS-SK-92-1..4.
2. **🔴 La quiescenza al census NON è provata (Pedersen)**:
   HTTP-complete ≠ request_end-complete — il curl sequenziale garantisce
   che la RISPOSTA è partita, non che il worker ha finito il teardown
   della richiesta; il checkpoint `pwork` può cadere dentro la coda di
   una richiesta ancora viva ⇒ **b_work non è verdict-grade** finché la
   riga non porta un testimone `outstanding=0` letto dal census stesso.
   → A-PP-63, KS-PP-92-1 (+ A-PP-60/61/62/64/65).
3. **🔴 Il residuo VLADDER 193.331 B è un MISMATCH DI SELEZIONE, non
   «census+self» (Bak)**: Δpost-work è ESATTAMENTE 0 su 17/20 raw — il
   residuo nasce dal confronto fra stime aggregate su serie diverse,
   non da allocazioni reali del segmento; inoltre **b_peak è una
   min-statistic** (tiebreak=min su punti con modes==R: −2,8% vs la
   mediana) e la robustezza min-of-R è TAUTOLOGICA (min==dominante per
   costruzione). → A-BB64 (bandire tiebreak=min con modes==R: mediana),
   A-BB65 (additività per-run), A-BB66 (robustezza non tautologica),
   A-BB67 (flag run-outlier: W12 r3), A-BB68 (CI onesti df=2).
4. **🔴 Il marcatore `--stderr` del pin A-DS50 è FORGIABILE (Stogov)**:
   un programma PHP legale può stampare la stringa marcatore e spezzare
   il parsing del comparatore futuro; inoltre il bersaglio t4 non nomina
   il BRACCIO (plain ≠ persist: il fatal cade in punti diversi). →
   A-DS54 (pin v3 a BYTE-COUNT per canale, comparatore a marcatori
   BANDITO), A-DS55 (t4 = braccio PERSIST per nome), A-DS53 (fixture v3:
   7 buchi nuovi oracle-morsi), A-DS56, A-DS57 — KS-DS-92-1..3.

### Convergenze forti (non capitali ma vincolanti)

- **Leijen spiega la collisione a MECCANISMO**: `mi_heap_of(probe)`
  restituisce l'heap del CHIAMANTE via TLS su heap statico
  (heaps=1 vs theaps=26 nei raw) ⇒ la via è il **census ON-THREAD**
  (A-DL-52), con purged ai bordi dello span (A-DL-53), overlap forzato
  (A-DL-54), huge/abandoned nel census (A-DL-55), braccio bare-thread
  per scomporre b_boot (A-DL-56) — KS-DL-92-1/2.
- **La legge clamp⇔overlap è convergente e a due mani**: Gregg 10/10,
  Leijen 6/6 sul proprio sottoinsieme (A-BG61) — il clamp segue
  l'OVERLAP delle finestre, non la purge (coerente con P-CLAMP-PD
  refutata in S-90.0).
- **Doc vs verdict (Gregg)**: ogni cifra del doc riprodotta AL BYTE dai
  raw, ma gli ADVISORY per NOME sono **7 (4 PEAK + 3 WORK)**, non 4, e
  VCOV va per-W (0,415→0,650; coverage MARGINALE ≈0,80 di b_peak) →
  A-BG57/58/59/60/61, KS-BG-92-1/2.
- **Catena evidenza (Hejlsberg)**: judge_sha g1/g2 e la finestra
  4c99520..bb4b388 verificate a macchina e TENGONO; restano la riga
  ABORT operatore indistinguibile da una di script (A-AH58 `writer=`) e
  i FAIL/REFUSE senza ancora sha (A-AH59), + campaign_sha sulle righe
  verdict (A-AH60) — KS-AH-92-1/2.
- **Sigilli (Hoare+Matsakis)**: buchi solo nelle guardie IN AVANTI
  (census totale arm ==2 A-TH-57, use-monoriga A-TH-58, dot-path
  tollerante A-TH-59, definizioni probe_in a census A-TH-60, doc terzo
  stato never-initialized A-TH-61; cap effettivo A-MS-51, sito
  nell'abort A-MS-52, heap ptr in-band A-MS-53, nesting-tooth A-MS-54,
  REQ_NS esclude census_self A-MS-55) — le cifre di S-90.0 reggono.
  ⚠️ Vincolo di sequenza scoperto dal team: se `arm()` prendesse un
  argomento (A-MS-52), il pin `narm` a parentesi vuote si romperebbe ⇒
  **A-TH-57 va scritto in forma PREFISSO** prima di toccare la firma.

### GRADO DELLE CIFRE S-90.0 (delibera team-misura, composta a strati)

- **b_boot = 2.252.800 B/worker: VERDICT-GRADE come MAGNITUDINE**
  (non come composizione: cosa lo compone resta aperto, A-DL-56).
- **b_work: ADVISORY** (quiescenza non provata, KS-PP-92-1).
- **somma b_boot+b_work: ADVISORY** (eredita il min).
- **b_peak: DA RIFARE** (min-statistic su punti modes==R —
  KS-BB-92-1: una lane con modes==R su TUTTI i punti non è citabile).
  Il MEASURE90 va emendato di conseguenza in apertura di S-91.0.

### Ordine vincolante di apertura S-91.0 (non rinegoziare)

1. **Sanatorie di grado e forma (PRIMA di ogni nuova misura)**:
   MEASURE90 riscritto coi gradi deliberati (b_boot verdict-grade come
   magnitudine · b_work e somma ADVISORY · b_peak DA RIFARE · ADVISORY
   per NOME = 7 · VCOV tabella per-W + coverage marginale · residuo
   ri-attribuito a mismatch di selezione, label census/self RITIRATA);
   A-BG60/A-BB65/A-BG57.
2. **Gate cifre — l'autorità (blocca tutto il resto)**: A-SK-67 +
   A-SK-70 (tutto da HEAD; cache abolita/rifiutata) con i QUATTRO forge
   di Klabnik ripetuti come denti che DEVONO fallire; poi A-SK-69 +
   A-SK-73 (prov commensurabile + budget sul pool), poi bite-test dei 5
   doc storici, poi A-SK-68 (abolizione legacy_frozen — MAI abolire
   prima di aver collocato i doc), poi A-SK-71 (perimetro 43 cifre),
   poi A-SK-72.
3. **Delibera UNICA sul formato del ledger** (tre grammatiche toccate
   insieme, tensione registrata dal team-cifre): A-AH58 `writer=` +
   A-AH59 ancora sha su FAIL/REFUSE + A-AH60 campaign_sha + A-BG58
   reason= autosufficiente + A-SK-72 supersessione provata + A-PP-65
   autorizzazione del giudice — UNA sola revisione di formato, non sei.
4. **Sigilli v9 (meccanici)**: A-TH-57 in forma PREFISSO (vincolo di
   sequenza) · A-TH-60 · quinta rete A-MS48 · A-TH-58/59 · pin A-TH53
   Cargo.toml · doc A-TH-61 + residui dichiarati.
5. **Canale di misura iterazione 3**: A-DL-52 census ON-THREAD alla
   BARRIERA (con A-PP-63 testimone outstanding=0 — i due si compongono:
   la barriera È il testimone) · A-DL-53 purged ai bordi · A-DL-54
   overlap forzato · A-DL-55 huge/abandoned · A-DL-56 bare-thread ·
   A-PP-60/61/62/64 igiene campagna · A-BB64/66/67/68 stimatori
   riparati SUI 20 RAW GIÀ COMMITTATI (non serve rimisurare per
   riparare le stime).
6. **ROADMAP A-DS51 — implementazione LSP in QUATTRO fasi (team-engine,
   ordine vincolante)**: fase 0 = fixture v3 (A-DS53) + pin v3
   byte-count per canale (A-DS54) committati PRIMA; fase 1 = checker LSP
   PURO + unit sui messaggi (45 pin, solo `cargo test --release`);
   fase 2 = wiring lowering HOISTED con le esenzioni nello STESSO
   commit (t3 exit 0, t4 sul braccio PERSIST) + gate ORM/hk/corpus per
   NOME; fase 3 = bind-registry per condizionali/dinamiche + gate di
   nuovo. **Rischio principale dichiarato: la fase 2 arma un check che
   può solo AGGIUNGERE fatal su ORM/hk — ogni esenzione mancante è una
   regressione.**
7. **Design/deferred**: A-MS46 (mod probe annidato — riduce solo la
   famiglia flag, non i siti arm né probe_in) · A-MS-51/52+54/53/55 ·
   A-TH55 · A-BB60/A-PP49 · A-MS27/A-PP18/A-PP27/A-AH38 invariati.

### Kill-switch nuovi consolidati (attivi da subito)

KS-TH-92-1 (census arm ≠ narm ⇒ figura VOID) · KS-TH-92-2 (seconda
definizione probe_in in qualsiasi grafia ⇒ VERDICT VOID) · KS-MS-92-1
(heaps_total>MAX_THEAPS con heap_overflow=0 ⇒ census VOID) · KS-MS-92-2
(abort probe senza tag di sito ⇒ attribuzione sito-live VOID) ·
KS-MS-92-3 (nuovo sito arm() senza dente anti-nesting nello stesso
commit) · KS-SK-92-1 (PASS con blob ≠ HEAD ⇒ VOID) · KS-SK-92-2 (prov
con operandi di file/campagne diverse ⇒ non verdict-grade) · KS-SK-92-3
(cifra fuori perimetro manifest ⇒ non verdict-grade) · KS-SK-92-4 (PASS
con cache/nonce forniti dall'invocante ⇒ VOID) · KS-AH-92-1 (attempts
senza writer= valido ⇒ consumazione VOID) · KS-AH-92-2 (FAIL/REFUSE
senza ancora sha ⇒ battery VOID) · KS-BB-92-1 (lane con modes==R su
TUTTI i punti ⇒ b non citabile) · KS-BB-92-2 (check di robustezza che
non può fallire ⇒ VOID) · KS-PP-92-1 (Δ di fase senza testimone
outstanding==0 ⇒ mai verdict-grade) · KS-PP-92-2 (judge_sha non
autorizzato in-band ⇒ VOID) · KS-DL-92-1 (census per-worker con heap
ptr duplicato ⇒ REFUSE) · KS-DL-92-2 (claim sul clamp senza riga
overlap-status ⇒ VOID) · KS-DS-92-1 (comparatore a marcatori senza
byte-count ⇒ REJECT) · KS-DS-92-2 (merge A-DS51 senza fixture A-DS53
per NOME nel gate ⇒ REJECT) · KS-DS-92-3 (fase-2 che fatala t3 o manca
il persist-shape di t4 ⇒ REJECT) · KS-BG-92-1 (VCOV senza tabella
per-W ⇒ metà census non citabile) · KS-BG-92-2 (reason non
ricostruibile senza aprire il verdict ⇒ riga non autosufficiente).
[22 KS]
