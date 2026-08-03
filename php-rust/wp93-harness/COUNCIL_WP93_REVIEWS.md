# COUNCIL_WP93_REVIEWS.md — Concilio a 9 sedie su S-91.0 (7 punti WP-92 eseguiti; gate cifre v2, stimatori riparati, testimone A-PP-63, fase 0 A-DS51)

Protocollo A DUE FASI (decisione utente 2026-08-02): fase 1 = 9 verbali
INDIPENDENTI (nessuna sedia vede le altre); fase 2 = 3 team tematici con
relatore (team-cifre Klabnik+Hejlsberg+Gregg · team-misura
Bak+Pedersen+Matsakis · team-engine Stogov+Leijen+Hoare). I verbali
individuali sono la fonte VINCOLANTE; le note di team alimentano la
sintesi.

Indice: VERBALI INTEGRALI (fase 1) → NOTE DI TEAM (fase 2) → SINTESI DI
CONVERGENZA (ordine S-92.0 + kill-switch).

## VERBALI INTEGRALI (fase 1)

# Verbale sedia 1 — Hoare — Concilio WP-93 (revisione S-91.0)

**Perimetro**: sigilli lessicali v9, pin, fail-fast, doc (gate-lever-pins.sh; vm/mod.rs).
**VERDETTO: CONCORDO CON EMENDAMENTI.**

## Q1 — TH49ML_PROG (gate-lever-pins.sh:1246-1264)

Grafie che ANCORA sfuggono (verificate sulle regole, riga per riga):

1. **Punto-con-commento-in-coda**: `let x = u. // c` + riga `vm_gate(x)`. La tolleranza `(\/\/.*)?$` fu data alla regola NOME (1247) ma NON alla regola punto-nudo (1248: `/[.][[:space:]]*$/`) — asimmetria A-TH-59.
2. **Riga-commento a blocco interposta**: `u.vm_gate` \n `/* c */` \n `(x)` — pend/dot/nmp tollerano solo `//` e blank (1254/1257/1261); la riga `/* c */` cade su `{ pend=0 }`. Il residuo dichiarato ("multi-LINE block comments inside a path", 1237-38) NON copre questa riga-commento intera, lessicalmente raggiungibile.
3. **Nome-spazio-parentesi e turbofish, monoriga**: `retain.production_gate (x)` e `retain.production_gate::<>(x)` eludono TUTTO — `production_gate[(]` (86/118/164), sweep 340, TH49RE_V8 (1245: il ramo `.name…(` esige `/*…*/`), e l'awk (stateless su monoriga). Il pin ==2 su vm/mod.rs resta VERDE con un terzo sito. Buco più grave del perimetro.
4. **`use … CachedUnit as` con `as` a fine riga** + alias alla riga dopo: la regola 1252 esige `[[:space:]]as[[:space:]]` sulla stessa riga.
5. Dot+commento+nome-con-commento (`u.` \n `//c` \n `vm_gate //c2` \n `(x)`): COPERTO (1257→1259 nmp→1262). `use` dentro split: non-caso (illegale in posizione espressione); la regola 1249 resetta pend correttamente.

**awk-slash**: nel file è CHIUSA — ogni `/` nei pattern literal è escapata (`\/\/` a 1247/1254/1257/1261) o vive in regex DINAMICHE (-v re; TH49RE_V8 usa `[/]`; 1156/1167 `/` raw ma dinamiche = sicure). Invariante da dichiarare: TH49RE_V8 non contiene backslash — il processing `-v` li mangerebbe.

## Q2 — A-TH-57 (1416-1432)

Eludono ntot (grep fixed-string 'ProbeWindow::arm', 1426): **`ProbeWindow :: arm` spaziato**; **alias `use …::ProbeWindow as PW`** (TH49RE_V8 copre SOLO CachedUnit|VmGate); **type-alias `type P = ProbeWindow`**; **split multiriga `ProbeWindow::`\n`arm()`**; **siti fuori worker_pool.rs** (census scoped a $WPOOL, nessuno sweep workspace). Il vincolo ntot==2==narm regge strutturalmente (narm⊆ntot per riga) ma SOLO entro la grafia fissa: un terzo sito eluso lascia 2==2 VERDE. La dicitura "census TOTALE" è quindi FALSA alla lettera — è totale su una grafia, in un file.

## Q3 — Guardia numericità (1305)

Copre SOLO v8n/v8ml. Restano FAIL-OPEN su conteggio vuoto (`[ "" -ne N ]` → exit 2 → falso → dente vacuo): GATE_MINTS (140), PROD_N/SELF_N (552), RS_FLUSH (851), r1..r5 (1161), a/ml (1015), d1..d4 (1349), d5 (1447) — tutti output awk o grep-c-su-file-assente. I confronti in forma STRINGA (`[ "$x" = 0 ]`, es. 1357, 1465) sono fail-closed: vuoto≠"0". La guardia è un punto, non un sistema.

## Q4 — Doc A-TH-61 (vm/mod.rs 16548-16564)

L'enum a tre stati (destroyed/alive/never-initialized) è completa rispetto al modello std SOLO se l'accesso durante il PROPRIO destructor è sussunto in already-destroyed (std marca destroyed prima del drop): va DICHIARATO, altrimenti è il quarto caso. Incompletezze certe: (a) lost-write nominata solo per uc_log — **UC_STATS** (stessa lista TLS, 16544) ha lo stesso canale; (b) sul fallback path il never-init produce anche un box LEAKED (scrittura persa + memoria ritenuta). Il pin nth61 (1486-87) è `-ge 1` presence-only: doc svuotabile tenendo il marker.

## Emendamenti

- **A-TH-62**: TH49ML_PROG — tolleranza `//`-in-coda sulla regola punto-nudo + riga `/*…*/` intera nei tre stati; decoy same-commit.
- **A-TH-63**: rete monoriga nome-spazio-paren e turbofish (pin PREFISSO `[.]…(production_gate|vm_gate)` senza àncora `(`, modello A-TH-57) su vm/mod + sweep.
- **A-TH-64**: helper num-assert su OGNI cattura awk/grep-c (o `-ne`→`=` stringa); generalizza la guardia 1305.
- **A-TH-65**: ntot in ERE `ProbeWindow[[:space:]]*::[[:space:]]*arm` + sweep workspace + rami alias/type-alias ProbeWindow in TH49RE_V8; emendare la dicitura "TOTALE".
- **A-TH-66**: doc A-TH-61 — UC_STATS lost-write, leak fallback, collocazione caso durante-proprio-dtor con cite std; pin nth61 ==1 + frase distintiva.
- **A-TH-67**: ml — `as` a fine riga nel ramo use.

## Kill-switch

- **KS-TH-93-1**: nessuna cifra di campagna m91 con probe VERDICT-grade finché A-TH-63/65 non mordono (A-TH-57 è la base della validità probe).
- **KS-TH-93-2**: gate VOID se un conteggio non-numerico sfugge dopo A-TH-64.

## Refutazioni capitali

**NO.** Nessuna evidenza già consumata è falsificata; ma la LETTERA di A-TH-57 ("census TOTALE") e la copertura sistemica della guardia di numericità sono claim eccessivi da emendare prima di appoggiarci verdetti.

---

# Verbale sedia 2 — Matsakis (Concilio WP-93, revisione S-91.0)

Perimetro: ownership/aliasing, cfg/feature soundness, disciplina probe.

## VERDETTO: PASS CON EMENDAMENTI — nessuna refutazione capitale; una SOVRA-DICHIARAZIONE da emendare (il commento «outstanding==0 here is the proof» in main.rs:263-265 promette più di ciò che una lettura singola Relaxed garantisce).

## Q1 — cfg-split A-PP-63: COERENTE

Evidenza (worker_pool.rs): modulo census su `any(census-instrumentation, mem-census)` (r.184); inc OUTSTANDING + note_outstanding nel dispatch sotto `any(...)` (r.646-656) con QUEUE_DEPTH annidato sotto solo census-instrumentation (r.649-653); dec dopo `response_tx.send` sotto `any(...)` (r.578-588); err-undo sotto `any(...)` con QUEUE_DEPTH interno cfg-ristretto e OUTSTANDING incondizionato (r.673-678). Sotto mem-census ogni inc ha esattamente un dec (post-send) o un undo (send fallito), sullo stesso thread del dispatch nel caso undo (sequenced, niente wrap). Il dec del pickup QUEUE_DEPTH (r.498-508) è correttamente census-instrumentation-only: simmetrico al suo inc. Percorsi senza dec: solo panic worker/probe → abort di processo (A-PP9/A-MS31), il contatore muore col canale — non è un leak osservabile. `note_outstanding` (r.246-254) compila sotto mem-census e panica SOLO su wrap (o==fetch_add+1==0 impossibile in aritmetica vera): stesso contratto fail-closed KH81-2 del canale instrumented, nessun abort NUOVO su run sani. Nota minore: l'undo (r.677) non ha tripwire-su-0 — accettabile perché sequenced-after il proprio inc.

## Q2 — happens-before del testimone: FALSIFICATORE VALIDO fail-closed, garanzia formale INCOMPLETA

`census_outstanding_now` = `load(Relaxed)` sul router (r.290-292); dec = `fetch_sub(Relaxed)` sul worker DOPO il send (r.580). Il curl sequenziale dà hb solo fino al send (oneshot recv sincronizza col send, r.572; il dec è sequenced-AFTER): quindi outstanding può leggere 1 stantio su client genuinamente sequenziale — finestra send→dec già dichiarata (A-TH15, r.204-211) e fail-closed (≠0 ⇒ ADVISORY): direzione giusta. Il teardown Vm/RetainSet completa PRIMA del send (execute_request ritorna prima di r.572), e la sua VISIBILITÀ al router viaggia sulla catena oneshot+TCP, non sul contatore. Falso 0: dentro il protocollo (client unico sequenziale, inflight_max≤1) è escluso dalla catena syscall; FUORI protocollo un load Relaxed può leggere 0 pre-inc di una richiesta concorrente — il testimone NON è machine-proof contro traffico estraneo. Con Relaxed manca l'edge formale dec→load: emendare a Release/Acquire (costo zero, chiude il gap di carta).

## Q3 — sequenza A-MS-51/52+54/53/55: RISPETTATA, UN ordine interno MANCANTE

A-TH-57 è in forma PREFISSO `==2==narm` già in S-91.0 (sigilli v9, WP_SESSION_91 p4) e design91.md r.76-79 dichiara pin narm aggiornato nello STESSO commit del cambio firma: vincolo rispettato. MANCANTE: A-MS-53 (`heap=<ptr>` su mi_theap_pages) è PREREQUISITO del dente REFUSE di A-DL-52 («due thr con lo stesso heap ptr ⇒ REFUSE», design91 r.13-24) — senza ptr in-band il giudice non può rifiutare la collisione; l'ordine non è dichiarato. Inoltre A-MS-55 è confermato NECESSARIO dal codice: oggi `req_t0` campiona ANCHE i hit `/__census_self` (worker_pool.rs r.525 e r.567-571, il branch self non lo salta).

## Q4 — finestra testimone→dump: ESISTE, guardia solo di protocollo

main.rs r.266-273: load outstanding → eprintln mi_quiesce → `peak_census_dump`. Il handler census NON incrementa OUTSTANDING (ritorna prima del dispatch, r.246-279): una richiesta nuova arrivata FRA testimone e dump porta il contatore a 1 senza alzare il watermark oltre il normale — INVISIBILE alla macchina. Oggi garantisce solo il protocollo (curl sequenziale). Serve il bracket: rileggere outstanding DOPO il dump ed emettere `outstanding_post=` sulla stessa riga ckpt; pre==post==0 ⇒ nessun dispatch è entrato nella finestra (con client unico sequenziale un transito completo intra-finestra è escluso).

## Emendamenti

- **A-MS-56**: dec OUTSTANDING → `Release`, load del testimone → `Acquire` (edge formale dec→load; commento main.rs riformulato: «proof» solo con l'edge).
- **A-MS-57**: bracket del testimone — seconda lettura POST-dump, campo `outstanding_post=` in-band sulla riga mi_quiesce.
- **A-MS-58**: dichiarare in design l'ordine A-MS-53 ≺ A-DL-52 (il REFUSE anti-collisione richiede heap ptr in-band).

## Kill-switch

- **KS-MS-93-1**: claim «teardown-complete» da testimone a lettura SINGOLA (senza bracket pre/post) ⇒ il Δ resta ADVISORY.
- **KS-MS-93-2**: dente REFUSE di A-DL-52 esercitato senza `heap=<ptr>` in-band (A-MS-53) ⇒ census VOID.

## Refutazioni capitali: NO.

---

# Verbale sedia 3 — Steve Klabnik (Concilio WP-93, revisione S-91.0)

## VERDETTO: **GATE v2 NON VERDICT-GRADE** — 6 forge nuovi COSTRUITI, 6 LANDED (6/6), ognuno con controllo negativo che FALLISCE.

Il gate v2 ha chiuso i quattro forge WP-92 (T13-T16 mordono davvero). Ne ho
costruiti sei nuovi contro v2: **tutti passano**. Tre sono refutazioni capitali.

## Forge tentati (comando + esito)

Baseline: `bash wp81-harness/gate-measure-cifre.sh <doc>` (advisory, ~25 s,
corpus card=24232/budget 24232). Docs in scratchpad, repo mai toccato.

| # | Forge | Riga piantata | Esito | Controllo negativo |
|---|---|---|---|---|
| F1 | **Identità rev DECIMALI** (Q1) | `b_peak rivisto = 23.330.397 B per worker` | **PASS** (rc=0) | `23.330.398` → FAIL |
| F2 | **%ALLOW come lavanderia** (Q3) | `regressione CPU full: 2,8 volte la media` + `46,25 percento` | **PASS** | `2,9` → FAIL |
| F3 | **Unità incollata = troncamento** | `b_work rivisto = 1.024.999B per worker` | **PASS** | `1.024.999 B` (spazio) → FAIL |
| F4 | **Storia del budget** (Q2) | `outstanding al checkpoint: 23.999 allocazioni` | **PASS** | `23.998` → FAIL |
| F5 | **prov v2 su valore rotondo** (Q5) | `18.000.000 B [derivata: prov 18287875@…axum.83cr.lever.n2000.r1.log:3 − 287875@…:3]` | **PASS** (stampa `provenance-verified`) | — |
| F6 | **Giudice-COPIA** | copia patchata `wp81-harness/zzforge-judge93.sh --all` con doc perimetrale NON committato presente | **PASS verdict-grade** con `judge_sha=3b035cc737887991` = blob HEAD del giudice **pristine** | giudice pristine `--all` → FAIL |

**Misure a corredo.**
- Q1: 1431 antenati, **53 con prefisso decimale a 7 cifre, 32 a 8** → ≥53 valori
  fabbricabili in 1-9,9 M e 32 in 10-99 M **esattamente la finestra dei byte**.
  La normalizzazione italiana (`23.330.397`→`23330397`) porta la cifra dentro
  la porta: il residuo «raro» dichiarato in commento è una popolazione nota.
- Q2: 8 blob storici del budget, 7 valori (20097…24232); il budget è **autorità
  E sorgente di corpus**, non è output macchina, ed è già escluso dal pool prov
  (A-SK-73, T14d): contraddizione interna.
- Q5: chiusura same-file sulla finestra 15-22 M scesa da 46,25% a **1,270%** —
  ma sul sottospazio **pubblicabile**: multipli di 1 MiB **6/6 = 100%**, di
  0,5 MiB **13/13 = 100%**, di 100.000 22,5%. La chiusura uniforme è la metrica
  sbagliata: il forger pubblica MiB tondi, non interi a caso.
- Q4: `run_gate` (battery-91pre r.83) decide **solo sul codice di uscita**;
  ADVISORY-PASS esce **0** ⇒ indistinguibile da PASS (batteries 82/83/84-pre
  invocano senza `--all`). In advisory un **giudice sporco** produce solo NOTE.
- Q4 perimetro: `class_rx` non ammette sottodirectory ⇒ **questo stesso verbale**
  (`wp93-harness/verbali/*.md`) è fuori; fuori anche 75 .md committati, tra cui
  **FOOTPRINT_CPU_ROADMAP 53 cifre, COVERAGE 50, TODO 34, README 16** — i file
  PUBBLICATI su GitHub. E i verbali del concilio sono `judge=no`: le voci ALLOW
  2.8/46.25 sono cifre **non giudicate** promosse a costanti legali ovunque.

## Emendamenti (da A-SK-74)

- **A-SK-74** — porta rev decimali ABOLITA: niente `rev-parse` vivo; le identità
  decimali citate diventano righe committate `rev=<sha40>` pinnate per NOME.
- **A-SK-75** — `%ALLOW` è un'AUTORITÀ: ogni voce porta provenienza
  `path@riga` di un blob committato **e** la dichiarazione «non è una misura».
  **2.8 e 46.25 REVOCATE** (o resi `judge=yes` i verbali che le producono).
- **A-SK-76** — tokenizzazione fail-closed: un run di cifre seguito da
  `[A-Za-z]` non si TRONCA, si **rifiuta** (mai giudicare `1.024` per `1.024.999B`).
- **A-SK-77** — il budget esce dal corpus (fonte non-macchina), coerenza con A-SK-73.
- **A-SK-78** — **self-tether del giudice**: all'avvio `hash-object($0)` ==
  blob HEAD di `JUDGE_REL`, altrimenti REFUSE. Il `judge_sha` firma il codice
  che gira, non un omonimo.
- **A-SK-79** — il GRADO nel codice d'uscita: ADVISORY-PASS esce 64; `run_gate`
  pretende la riga `PASS … judge_sha=`.
- **A-SK-80** — perimetro per COMPLEMENTO: ogni `.md` committato sotto
  `php-rust/` (sottodirectory incluse) esige una riga di manifest, default
  `judge=yes`.
- **A-SK-81** — prov: operandi ETICHETTATI e della stessa grandezza (stessa
  chiave), non digit-run qualunque della stessa riga.

## Kill-switch

- **KS-SK-93-1** — nessuna PASS verdict-grade senza A-SK-78 dimostrato dal
  bite-test con copia patchata.
- **KS-SK-93-2** — i sei forge WP-93 diventano denti permanenti T17-T22; se uno
  non morde, il gate non è verdict-grade.
- **KS-SK-93-3** — nessuna voce ALLOW nuova senza A-SK-75; 2.8/46.25 fuori entro S-92.0.
- **KS-SK-93-4** — ADVISORY-PASS non chiude una riga di battery.

## Refutazioni capitali: **SÌ (3)**

1. «Un giudice patchato non può produrre un PASS anonimo» (A-SK-67) — **REFUTATA**:
   il PASS non è anonimo, è firmato con l'identità di un ALTRO file (F6).
2. «Il perimetro è ogni doc che pubblica cifre» (A-SK-71) — **REFUTATA**: 153
   cifre nei quattro doc più letti restano fuori.
3. «A-SK-69/A-SK-73 chiudono il forge prov» — **REFUTATA sul sottospazio che conta**:
   100% dei multipli di MiB nella finestra plausibile.

*Repo lasciato porcelain; nessun commit; forge rimossi.*

---

# Verbale sedia 4 — Hejlsberg (Concilio WP-93, revisione S-91.0)

Perimetro: catene di evidenza (ledger, attempts, supersessioni), identità toolchain. Mandato: REFUTARE.

## VERDETTO
La delibera unica di formato (design91-ledger.md, d783a74) è ben scritta ma **applicata a metà**: la grammar v2 delle battery morde su UN solo percorso di consumazione, la grammar v2 campaign non ha alcun dente, e il corpus non distingue «max perché ultima» da «max perché valida». Due refutazioni capitali.

## Q1 — battery-91pre v7 (trap/att_row/head-move)
- **Doppio writer=: NO.** `att_row` (r.47-52) appende `writer=script:` SOLO nel ramo `*)`; la riga del trap (r.58) contiene `writer=operator` ⇒ ramo `*writer=*)` la scrive verbatim, UN solo writer. Residuo minore: il case è substring-match, non field-anchored — un valore k=v contenente «writer=» sopprimerebbe la firma script (oggi non raggiungibile: FIRSTFAIL è sed-vincolato `[a-z0-9-]*`).
- **assert_head_unmoved NON copre tutti i terminali.** È chiamato solo a r.161 (prima di PASS/FAIL). Il REFUSE porcelain (r.78) e l'ABORT del trap (r.58) emettono la riga terminale SENZA il check — la lettera della delibera («prima di OGNI riga terminale», design91-ledger r.18-19) è violata. Il trap è il caso serio: può scattare DOPO un HEAD-move mid-battery e la riga stampa `rev=$GIT_REV` con `reason=signal` che maschera il head-moved.
- **Trap armato prima del primo att_row: SÌ** (trap r.61, primo att_row possibile r.78). Finestra residua non dichiarata: un segnale nel preambolo r.24-60 lascia il ledger muto.

## Q2 — battery-equivalence grammar v2
- Riga vuota in V2ROWS: impossibile per costruzione (output di grep, guardia `[ -n ]`); se iniettata, `grep -cvE` la conta come violazione (verificato: count=1) — fail-closed.
- `writer=script:SHA` a fine riga: il ramo `$` in `( |$)` funziona su grep BSD/macOS (verificato: match=1). Le righe PASS, che chiudono con writer=, passano.
- Esiti fuori set: riga senza `esito=` o con esito ignoto ⇒ conteggiata (grep -cv, fail-closed). Sfuggono: (a) **TUTTO il blocco v2 (r.389-404), come A-AH50/A-AH54, vive DENTRO `if [ "$SAME_REV" = 1 ]`** — una consumazione in modalità EQUIVALENZA di una battery 9x salta writer=, àncore e triangolo attempts: la delibera dice «alla consumazione», il codice dice «alla consumazione same-rev». **REFUTAZIONE CAPITALE.** (b) `battery-9[1-9]*` non copre battery a 3 cifre (100pre) — buco futuro, da dichiarare. (c) `sha256=[0-9a-f]{64}` non ancorato: 65-hex passa (minore).

## Q3 — ledger_supersession_proved / cite_max
Il dente (r.578-591) è consultato SOLO per g<max (r.740). Per g non-max con prova FAIL: citarla come «verità» verbale passa (il wording è morto per design A-SK-72), ma le sue CIFRE sono fuori corpus (r.451-453) ⇒ residuo solo narrativo, accettabile. **Il buco reale è il max-FAIL**: il filtro corpus (r.444-453) tiene la generazione MASSIMA per solo numero G, senza check di esito — una campagna terminata con FAIL finale mai superseduto lascerebbe il suo .out NEL corpus e le sue cifre legalizzerebbero token come verità. Oggi non-live (m89 max=g3 PASS, m90 max=g2 PASS, verificato dai ledger), ma il meccanismo è aperto. **REFUTAZIONE CAPITALE** (mechanism-level).

## Q4 — campaign v2 solo deliberata
**Nessun dente, solo delibera.** measure91-campaign.sh/verdict91.sh NON esistono (verificato); measure90-campaign.sh è riusabile a mano per una m91 con grammatica v1; `ledger_supersession_proved` parsa felicemente un ledger v1 (regex supersede_of/esito=FAIL, zero check su campaign_sha/reason/authorize); nessun checker chiavato su `m9[1-9]`. La lezione di A-AH58 («la provenienza era una convenzione, non un campo») si applica alla delibera stessa.

## Emendamenti
- **A-AH61**: il blocco grammar v2 + denti attempts (A-AH50/54/58/59) esce dal ramo same-rev: morde su ENTRAMBI i percorsi di consumazione.
- **A-AH62**: assert_head_unmoved chiamato anche nel REFUSE porcelain e nel trap (HEAD mosso ⇒ reason=signal+head-moved sulla riga).
- **A-AH63**: dente pre-nascita campaign v2, committato ORA: qualunque riga di `m9[1-9]*.campaign.ledger` senza disciplina v2 ⇒ rifiuto alla prima consumazione.
- **A-AH64**: ammissione al corpus della generazione max richiede riga `esito=PASS` committata; max-FAIL = storia (classe judge=no), mai verità.

## Kill-switch
- **KS-AH-93-1**: consumazione (qualunque percorso) di battery 9x senza verifica grammar v2 ⇒ consumazione VOID.
- **KS-AH-93-2**: riga m9[1-9] fuori grammatica v2 ⇒ campagna VOID.
- **KS-AH-93-3**: cifra legalizzata da un .out max senza riga PASS committata ⇒ doc FAIL.

## Refutazioni capitali
**SÌ, due**: (1) enforcement v2 battery solo same-rev + campaign v2 senza dente (Q2a+Q4 — la revisione di formato appena deliberata è per metà convenzione); (2) corpus ammette la generazione max senza esito (Q3).

---

# Verbale sedia 5 — Lars Bak — Concilio WP-93 (revisione S-91.0)

Perimetro: statistica, stimatori. Metodo: ricomputo INDIPENDENTE (python, non l'awk del repair) dai 20 raw `m90.slope.*.memcensus` (tr -d '\0', campi per NOME: win=9 pboot/pwork, win=0 exit_collect_mi). Estrazione mia vs ladder del `.out`: **20/20 byte-identica** — nessun mismatch.

## VERDETTO: PASS CON EMENDAMENTI
Le cifre di testata riprodotte al byte; due difetti statistici reali: la banda pooled df=18 NON è citabile come CI onesto (indipendenza refutata dai raw) e il MAD flag inclusivo MASCHERA un terzo outlier (w16.r1).

## Q1 — b_peak(mediana) riprodotta; outlier dentro è giusto
Mie cifre: b=20.289.945,6 · se=1.084.655,4 — **al byte col .out**. Esclusione PRE-mediana:
- solo w4.r1: W4med(4)=153.944.064 → b=20.285.030 (Δ=−4.915, −0,02%).
- politica coerente (TUTTI i flaggati: w4.r1+w12.r3): W12med(4)=330.825.728 → b=20.357.120 (Δ=+67.174, +0,33%), se SALE a 1.272.980 e i marginali peggiorano (W8→12 25.100.288 OUT alto, W12→16 15.269.888 OUT basso).
Verdetto: **mediana-su-5 con l'outlier DENTRO è lo stimatore giusto** (breakdown 40%: tollera 2/5); l'esclusione compra ≤0,33% al prezzo di un grado di libertà del ricercatore e di se più larghi. Q1 sostiene il .out.

## Q2 — leave-one-out: SÌ, esiste un outlier mascherato
LOO (mediana+MAD delle 4 SORELLE, run giudicata fuori) col criterio A-BB67 «TUTTI e 3 i contatori»: flagga **w4.r1, w12.r3 E w16.r1**. w16.r1 (pboot 48.758.784 max, pwork 399.638.528 = +9,4 MB sulle sorelle, exit idem) è mascherata dal criterio inclusivo: sul pboot la sua stessa presenza porta dev==MAD==131.072, esattamente dentro 2·MAD; con LOO dev=229.376 > 2·98.304. Effetto sulle testate: **ZERO** (outlier alto: né min né mediana lo pescano; mediane invariate). Difetti collaterali: (a) MAD==0 (w8 pboot) rende il criterio DEGENERE e il codice lo tratta silenziosamente come non-outlier; (b) su singoli contatori LOO flagga 13 run — il congiuntivo ALL-3 è la sola cosa che tiene il flag utilizzabile.

## Q3 — pooled df=18: se riprodotto, banda NON citabile come 95%
Mie cifre: b=20.274.872,3 · se=427.628 · banda t18 [19.376.426, 21.173.319] — riprodotte. MA: con disegno bilanciato la pendenza pooled ≡ pendenza sulle 4 MEDIE per-W (verificato: identiche); l'informazione extra dei 20 punti sta solo nell'se, che presume 20 errori indipendenti. Dai raw: MS_between(df2)=2,86e14 vs MS_within(df16)=4,65e13, **F(2,16)=6,16 > F0,95≈3,63** ⇒ le run sorelle NON sono rumore scambiabile attorno alla retta (ICC≈0,51, design effect ≈3,0 ⇒ se corretto ~744.541, vicino all'se per-medie 846.130). CR0 con G=4 cluster (se=397.448) è downward-biased, inutilizzabile. **La banda [19.376.426, 21.173.319] NON è citabile come 95%**; citabile: la banda df=2 sulle medie [16.633.973, 23.915.772] (t=4,303) o il pooled etichettato «condizionato a indipendenza NON provata».

## Q4 — coerenza gradi/cifre di MEASURE90 emendato
Verificate al byte dai miei conti: b_boot 2.252.800/se 6.345,5 · b_work 17.276.928/se 501.347,7 · somma 19.529.728 · b_peak(med) 20.289.946/se 1.084.655/banda 2se · robustezza 0,972/0,999 · coverage marginale 0,778 · residui per-W · conversioni MiB (2,15/16,48/18,63/19,35) · delta 45.875. **COERENTI**. Due punti che il .out non sostiene: (1) la dicitura «pooled … CI onesto» (§b_peak): onesto nei df (A-BB68), NON nell'indipendenza (Q3); (2) «nessuna delle due può diventare il punto della sua W via min» — come ASSERZIONE è refutata dallo stesso doc (PEAK-min HA pescato w4.r1): va riformulata come norma. Inoltre «additività within-run 20/20» è un'identità aritmetica (il .out lo dichiara): mai contarla come verifica.

## Emendamenti
- **A-BB69**: MAD flag in forma LEAVE-ONE-OUT (sorelle = R−1, run giudicata esclusa); w16.r1 flaggata per NOME; MAD==0 ⇒ stato DEGENERE dichiarato in output, mai silenzio.
- **A-BB70**: politica outlier di testata: outlier DENTRO la mediana; esclusione pre-stimatore BANDITA salvo delibera, e allora TUTTI i flaggati + Δb dichiarato.
- **A-BB71**: pooled df=18 declassato a ADVISORY-conditional; F-test intra-W (ex-ante) obbligatorio prima di citare qualunque banda n>W-points come 95%; emendare la riga «CI onesto» in MEASURE90.
- **A-BB72**: riformulare la frase «can never become its W point via min» in forma NORMATIVA; l'additività within-run etichettata sempre «tautologia».

## Kill-switch
- **KS-BB-93-1**: banda pooled citata come 95% senza F-test di scambiabilità intra-W superato ⇒ FAIL del giudice.
- **KS-BB-93-2**: flag outlier calcolato con la run dentro le proprie sorelle ⇒ FAIL (solo LOO).

## Refutazioni capitali
- **SÌ (1)**: «pooled df=18 = CI onesto» — refutata dai raw (F=6,16): la banda [19.376.426, 21.173.319] non è un 95%.
- **NO** sulle cifre di testata: b_peak(mediana), se, b_boot, b_work, somma, robustezza, coverage marginale — tutte riprodotte al byte dai raw; l'outlier mascherato w16.r1 non sposta alcuna testata (effetto zero su min e mediane).

---

# Verbale sedia 6 — Pedersen (Concilio WP-93, revisione S-91.0)

Perimetro: confine per-richiesta, lifecycle harness, failpath. Mandato: REFUTARE.

## VERDETTO

A-PP-63 è vivo e la sua catena è più solida di quanto il verbale S-91.0
lasciasse temere (Q2 REGGE). Ma il testimone è un **istante, non una
finestra**: la pretesa del commento in main.rs («outstanding==0 here is
the proof that no request teardown can leak into the phase Δ», righe
262-265) è REFUTATA nella sua estensione al dump. Una refutazione
capitale.

## Q1 — Testimone-istante ≠ testimone-finestra: SÌ, serve la coppia

`census_outstanding_now()` è letto a main.rs:266, la riga emessa a
267-272, il dump a 273 — tutto sul task axum del probe, che NON passa
dal pool. Fra load e fine-dump un task axum concorrente può fare
`dispatch` (inc a worker_pool.rs:653) ed eseguire DURANTE il dump.
Niente nel server impone la sequenzialità del driver. Peggio: un
campione post-dump da solo NON basta — una richiesta interamente
contenuta nella finestra riporta outstanding a 0, e sotto mem-census il
contatore monotono `REQUESTS` NON incrementa (fetch_add a riga 944 è
`cfg(census-instrumentation)`; solo la coppia OUTSTANDING è wired,
righe 179-182). `OUTSTANDING_MAX` non discrimina (watermark già 1).
Serve: coppia pre/post + monotono di arrivo compilato sul canale
mem-census.

## Q2 — La catena REGGE: il dec copre il teardown intero

Verificato: `execute_request` (worker_pool.rs:694-706) crea il
RetainSet sul proprio stack; il Vm nasce a 854 dentro
`execute_with_retain` e muore al return; il RetainSet muore al return
di execute_request — ENTRAMBI morti prima di `response_tx.send`
(572), che precede il dec (579-580). Quindi outstanding==0 certifica
request_end + teardown Vm/RetainSet COMPLETI, non solo il send. Il
send è sul oneshot; il body lo scrive hyper via il task axum dopo
`rx.await`: curl-return ⇒ send fatto ma dec forse pendente (finestra
send→dec, ammessa a 1804-1813) — direzione FAIL-CLOSED (spurio ≠0 ⇒
ADVISORY), mai fail-open. UN buco dichiarabile: il `Vec<u8>` del body
attraversa il canale e viene liberato sul task axum DOPO la scrittura
socket — quel free sfugge al testimone.

## Q3 — Trap: nessun orfano diretto, ma l'ABORT è DIFFERITO

`run_gate` esegue i gate in FOREGROUND (battery-91pre.sh:87); bash
DIFFERISCE i trap finché il figlio foreground non termina. Con `kill
-TERM/HUP` al PID dello script: il gate corre fino in fondo, POI
on_signal ledgera — `attempt_epoch` è l'istante del trap, non del
segnale, e il gate «abortito» è in realtà completato. Con INT da
tastiera l'intero process group muore: nessun orfano dello script, ma
eventuali server DETACHED avviati dai gate (setsid/nohup interni)
sopravvivono — la battery in sé non detacha nulla. L'ancora out_sha è
sana (solo lo script scrive OUTF).

## Q4 — Sede measure91-campaign: SUFFICIENTE

`date +%s` è locale-immune; sort/comm/join della battery sono
auto-consistenti (stessa env intra-run, nomi ASCII space-free per
costruzione A-AH52). A-PP-60/61/62/64 restano al campaign alla
nascita; nessuno va armato prima. Igiene gratuita, non bloccante:
`export LC_ALL=C` nel prologo battery.

## Emendamenti

- **A-PP-66**: testimone a COPPIA — seconda riga mi_quiesce POST-dump,
  stesso ckpt, campi outstanding= pre e post.
- **A-PP-67**: contatore monotono di arrivo (inc in dispatch) compilato
  `cfg(any(census-instrumentation, mem-census))`, campo `arr=` su
  entrambe le righe della coppia; uguaglianza pre/post = finestra
  pulita.
- **A-PP-68**: riga ABORT del trap porta `gate_in_flight=<label>` e
  `deferred=1` quando il trap scatta a gate concluso (la semantica
  temporale diventa un campo, non una convenzione).
- **A-PP-69**: free del body fuori-testimone DICHIARATO nel commento
  A-PP-63 (cardinalità: un Vec per richiesta, lato axum).

## Kill-switch

- **KS-PP-93-1**: Δ di fase verdict-grade SOLO con coppia A-PP-66/67
  (pre==post==0 e arr uguali); testimone a riga singola ⇒ ADVISORY.
- **KS-PP-93-2**: riga esito=ABORT senza gate_in_flight/deferred dopo
  l'attuazione di A-PP-68 ⇒ battery VOID.

## Refutazioni capitali

**SÌ, una**: il claim «outstanding==0 alla riga ⇒ nessun teardown può
inquinare il Δ di fase» è falso come garanzia di FINESTRA — vale solo
all'istante del load; il canale mem-census oggi non possiede nemmeno il
monotono per chiudere la finestra. Q2/Q3/Q4: nessuna refutazione
(catena verificata, denti minori).

---

# Verbale sedia 7 — Daan Leijen (Concilio WP-93, revisione S-91.0)

Perimetro: mimalloc, semantica contatori, canale misura iterazione 3. Mandato: REFUTARE.

## VERDETTO

La riparazione a macchina (repair90) è onesta: additività 20/20, tether al byte, gradi corretti. Ma **il piano A-DL-52 come scritto in design91.md è già refutato dal canale**: cita `mi_heap_get_default()`, simbolo che il mimalloc v3 di questo tree NON esporta (memcensus.rs r.634-637 e r.1426-1428 lo dichiarano due volte: «not exported by the tree's mimalloc v3»). Stessa classe della lezione WP-90: la lettera del design va calibrata sul canale reale.

## Q1 — heap lazy-init e collisione legittima

Il rischio del worker idle è reale su mimalloc (default heap TL lazy: pre-init tutti i thread puntano al medesimo heap statico vuoto), ma qui è MOOT per costruzione: non avendo `mi_heap_get_default`, l'unica via è quella già collaudata in `mi_bin_thread_rows` — **probe `Box::new` ON-thread + `mi_heap_of(probe)`**. Il probe stesso È il disambiguatore: forza la lazy-init, quindi un thread che ha emesso il probe non è mai "mai-allocante". Il dente A-DL31 va SCOPATO: REFUSE solo per collisione ptr fra thread DISTINTI entrambi post-probe; `heaps_total<W+1` da `mi_subproc_visit_heaps` è DECLARED, non REFUSE.

## Q2 — barriera di picco

Con la sola architettura HTTP round-robin il piano NON è eseguibile: W richieste in volo non garantiscono W worker distinti (accodamento su worker occupato ⇒ deadlock o snapshot parziale). Serve un **Barrier(W) condiviso a due fasi**: (1) rendezvous ARRIVO (tutti i W dentro, nessuno idle, heap congelati dal punto di vista del proprio churn); (2) snapshot on-thread (probe+visit del PROPRIO heap; BinTab e buffer pre-allocati PRIMA della fase 2, righe formattate DOPO — A-MS50 vale anche qui); (3) rendezvous PARTENZA. Timeout fail-closed obbligatorio (riga in-band, snapshot VOID — mai campagna appesa). Visite concorrenti su heap propri: lecite (pagine per-heap), ma il metadato subproc è condiviso — dichiararlo nel header.

## Q3 — massa invisibile

Coerente con la mia analisi WP-92: l'**intercetta** (71,9 MB) è la sede naturale di arena eager-commit/granularità 64KiB, malloc_huge (total vs current) e stato boot main-heap — tutti termini di PROCESSO, non di worker. La **pendenza** 4,48 MB/worker la spiega per prima il **page slack per-theap**: committed−used_b delle pagine parzialmente piene + free_committed delle pagine retained (page_full_retain=2 per size-class PER heap ⇒ scala esattamente con W). È già misurabile dalle righe `mi_theap_pages`/`mi_theap_bin` esistenti: joinare committed−used per heap contro la pendenza invisibile PRIMA di nuova strumentazione. A-DL-55 (current huge, abandoned, chunk_bins) chiude l'intercetta, non la pendenza.

## Q4 — armare A-DL-54

**Doppia richiesta a barriera, pad INVARIATI.** Allungare i pad distrugge l'àncora cal 7.801.102 B (quinta campagna al byte) e confonde durata con massa. Rischi dell'overlap forzato: (a) l'attesa di barriera dentro il bracket gonfia lo span per timing, non per allocazione ⇒ rendezvous FUORI dal bracket del net, span provato da mono_us in-band; (b) serve il gemello serializzato (stessa barriera, rilascio sequenziale) come NO-OVERLAP controllato di pari macchineria. Predizione ex-ante: pd=1000+overlap ⇒ clamped_rows>0 conferma bracket/counter; clamp assente ⇒ legge A-BG61 refutata.

## Emendamenti

- **A-DL-57**: riscrivere A-DL-52 senza `mi_heap_get_default`: probe on-thread + `mi_heap_of`, dente A-DL31 scopato post-probe.
- **A-DL-58**: barrier condiviso a due fasi con timeout fail-closed e thr-set/heap-set cardinalità W asserita in-band.
- **A-DL-59**: join pendenza-invisibile ⇔ Σ(committed−used_b)+free_committed per-theap dai raw esistenti, prima di ogni strumento nuovo.
- **A-DL-60**: A-DL-54 = braccio barriera + gemello serializzato, pad cal-anchored, rendezvous fuori bracket.

## Kill-switch

- **KS-DL-93-1**: riga snapshot per-worker senza probe on-thread in-band ⇒ VOID.
- **KS-DL-93-2**: barriera in timeout o heap-set < W distinti ⇒ split per-worker VOID (mai pubblicazione parziale).
- **KS-DL-93-3**: run A-DL-54 con pad diversi dall'àncora cal ⇒ braccio VOID.

## Refutazioni capitali

**SÌ, due**: (1) A-DL-52 come scritto invoca un simbolo inesistente sul canale; (2) l'architettura a canali attuale non porta W worker alla barriera — serve il Barrier condiviso, non è opzionale.

— Leijen, sedia 7

---

# Verbale sedia 8 — Dmitry Stogov (Concilio WP-93, revisione S-91.0)

Perimetro: Zend/opcache, contratto LSP A-DS35/A-DS51. Oracle vivo: PHP 8.5.7 (probe in /tmp/stogov93, MAI nel repo).

## VERDETTO

Fase 0 SOLIDA (pin length-prefixed regge anche il multibyte, Q2), ma le 8 fixture v3 NON chiudono il perimetro: QUATTRO buchi residui morsi dall'oracle (Q1), regole di formattazione mancanti in fase 1 (Q3), e UNA refutazione capitale sull'esenzione ctor (Q4) che, non emendata, produce falsi fatal su ORM/hk in fase 2.

## Q1 — Buchi residui: SÌ, quattro (+1 di famiglia)

1. **Enum abstract method** — famiglia NUOVA, non "Declaration of": `Fatal error: Enum method E::m() must not be abstract` (line della decl). Buco → v4.
2. **Hook SET** — famiglia NUOVA, senza "Declaration of" e senza tipi nel messaggio: `Fatal error: Type of parameter $v of hook C::$x::set must be compatible with property type`. Il set si giudica contro il TIPO DELLA PROPERTY, non contro il set del parent. Buco → v4.
3. **Intersection pura**: param `A&B`→`A` è LEGALE (contravarianza, exit 0); il negativo è il ritorno `A&B`→`A`: `Declaration of C::m(): A must be compatible with P::m(): A&B`. Negativo non pinnato (w8 copre solo il positivo DNF) → v4.
4. **By-ref hook, property invariance**: `Declaration of & C::$x::get(): string must be compatible with & P::$x::get(): int` — prefisso `& ` CON SPAZIO, non modellato dal vincolo w5. Buco → v4.
5. Interface `final const`: `C::X cannot override final constant I::X` — stessa famiglia di w6 (route implements ≠ extends); fixture v4 opzionale.

## Q2 — Length-prefixed su multibyte: REGGE

Probe classe UTF-8 (`class Città` / `Provincia extends Città`): fatal `Declaration of Provincia::m(): string must be compatible with Città::m(): int`; `stdout_bytes=163` (wc -c) vs 162 caratteri (`à`=2 byte); rilettura per OFFSET di 163 byte = `cmp` byte-exact. Il formato A-DS54 è corretto per costruzione sui byte. Però NESSUNA delle 45 fixture ha payload multibyte: il caso non è pinnato → fixture UTF-8 in v4 (regressione anti-`wc -m`).

## Q3 — Vincoli fase 1: INSUFFICIENTI, cinque regole mancanti

Oracle: `(B&A)|string` stampato COME DICHIARATO (nessun riordino); `string|int` idem; `int|null` E `null|int` collassano a `?int`; MA `string|int|null` resta ESTESO — il collasso `?T` vale SOLO per l'unione binaria con null. Quindi: (a) "ordine canonico Zend" va emendato in "ordine DICHIARATO, zero sorting"; (b) collasso `?T` solo binario; (c) by-ref param grafia `int &$x` (spazio tipo/&, & attaccata a `$x`: `Declaration of C::m(string &$x): void must be compatible with P::m(int &$x): void`); (d) prefisso `& ` sugli hook by-ref (Q1.4); (e) la famiglia hook-set NON usa la forma "Declaration of" (Q1.2).

## Q4 — Esenzioni: INCOMPLETE, con trappola semantica

- **q4a**: ctor CONCRETO in classe ABSTRACT è ESENTE (oracle: vive, `concrete-ctor-in-abstract-EXEMPT`). L'esenzione è keyed sul PROTOTIPO (ctor da interfaccia o dichiarato `abstract`), NON sull'abstract-ness della classe. La dicitura "ctor NON esente per iface/abstract" nella lettura per-classe è REFUTATA — e Doctrine è piena di abstract con ctor concreto: falsi fatal garantiti.
- **q4b**: il vincolo iface-ctor si PROPAGA ai nipoti: `Declaration of C::__construct(string $y) must be compatible with I::__construct(int $x)` due livelli sotto. Nessun pin (v13 copre solo l'implements diretto) → v4.
- **w7 = anti-esenzione**: il check readonly-promoted morde DENTRO il ctor esente — l'esenzione copre il SIGNATURE check, non il lowering delle promoted. Va detto per NOME nel commit fase 2.

## Emendamenti

- **A-DS58**: sei fixture v4 per NOME coi messaggi esatti sopra: x1 enum-abstract, x2 hook-set, x3 intersection-return-widen, x4 byref-hook-get, x5 utf8-classname, x6 iface-ctor-grandchild; stesso pin length-prefixed.
- **A-DS59**: esenzione ctor riformulata: checked SOLO contro prototipo da interfaccia o ctor `abstract`; classe abstract con ctor concreto = ESENTE.
- **A-DS60**: le cinque regole di formattazione Q3 entrano nei vincoli di modellistica fase 1 (unit dedicate).

## Kill-switch

- **KS-DS-93-1**: merge fase 2 senza le fixture A-DS58 per NOME nel gate = REJECT (estende KS-DS-92-2).
- **KS-DS-93-2**: fatal del checker su ctor concreto di classe abstract (pattern q4a) = REJECT.
- **KS-DS-93-3**: comparatore in CARATTERI (wc -m) anziché BYTE = REJECT.

## Refutazioni capitali

**SÌ, una**: lettura per-classe dell'esenzione ctor (Q4/q4a) — refutata dall'oracle vivo. Emendative (non capitali): "ordine canonico" → ordine dichiarato; collasso `?T` solo binario.

---

# Verbale sedia 9 — Brendan Gregg — Concilio WP-93 (revisione S-91.0)

Perimetro: metodologia di misura, doc==macchina per NOME, leggibilità ledger. Mandato: REFUTARE.

## VERDETTO: PASS con emendamenti minori. Refutazioni capitali: NO.

## Q1 — Censimento MEASURE90 emendato ↔ repair90-estimators.out (righe .out citate)

Ogni cifra del perimetro ha la sua riga; censimento per NOME:
- b_peak(mediana) 20.289.946 → r87 `REPAIRED b_peak ... 20289946` AL BYTE; se 1.084.655 → r87; banda [18.120.635, 22.459.256] → r87. Min-based 19.723.059 → r67+r82 (tether g2 PASS).
- b_boot 2.252.800 → r37; b_work 17.276.928 → r52; somma 19.529.728 → r88; continuità 19.575.603/45.875 → r89; residuo 193.331 → r82 (193331,2).
- Robustezze 0,972 / 0,999 → r86 (tautology=no).
- VCOV per-W: tutti e 12 i ratio → r96-r99, AL BYTE. Pooled 0,576 «in nessun raw» → r100.
- Marginale 15.777.004 = 0,778 → r101; invisibile 4.480.174 + intercetta 71.934.811 → r102.
- Residui per-W mode_min (−2.097.152 / 0 / 262.144 / 393.216) → r91; dpost nonzero (w4.r5 +5.046.272, w16.r2 +1.048.576, w16.r3 +5.177.344; 17/20 zero) → r93; outlier w4.r1/w12.r3 → r94-r95; ADVISORY=7 per NOME → r36; marginali OUT a mediana (22.183.936, 13.271.040) → r59-r61; clamp⇔overlap 10/10 → r113; 3/4 pd1000 dt∈{1,5} ricostruibile da r109-r112; t 4,303/2,101 → r2/r83-r85.
- cal 7.801.102: NON nel .out — fonte dichiarata verdict90.a1.g2.out r46, verificata. Legittimo (non è cifra riparata).

DUE nit di trascrizione, non refutazioni: (1) se b_boot doc «6.345» vs macchina 6345,5 — TRONCAMENTO, mentre ovunque il doc arrotonda half-up (501347,7→501.348; 1084655,4→1.084.655; 20289945,6→20.289.946): regola incoerente su UNA cifra, propagata a NEXT_SESSION:40. (2) delta continuità senza segno (macchina −45875).

## Q2 — Gradi in GAP_TREND (r101) e NEXT_SESSION

GAP_TREND: mediana «RIPUBBLICATA», min-statistic «DECLASSATA», somma «(ADVISORY)» — gradi giusti. NEXT_SESSION §Misure: b_boot «VERDICT-GRADE come MAGNITUDINE» (consentito), b_work e somma «ADVISORY», continuità «suggestiva» — conformi. NESSUNA cifra citata sopra il grado deliberato. Un buco: b_peak(mediana) in NEXT_SESSION sta sotto l'intestazione «gradi» SENZA token di grado, unica riga muta in mezzo a righe etichettate — leggibile per omissione come verdict-grade, mentre i 4 punti PEAK sono ADVISORY per NOME e la testata è «DA RIFARE» (KS-BB-92-1).

## Q3 — bite-test-historic.log

AUTOSUFFICIENTE: header con mandato (A-SK-68/WP-92 P2), head=c71bf9e, budget from HEAD per doc; ogni FAIL con file, NUMERO di riga, token e ragione (M84: 2; M85: 4 — coerenti con WP_SESSION_91 p2); i PASS M86/87/88 col criterio stampato e la label «NEVER verdict-grade». Collocazione: citato nel manifest a wp81-harness/gate-cifre-manifest.tsv:15; M84/85 ancorati verdict= (r28-r29). Nit: «committato in questo stesso commit» — il log atterra in ce1ee27 (FIGLIO di c71bf9e, parent verificato); il log non può nominare il proprio sha, la frase dica «commit figlio di head».

## Q4 — Lezioni ⭐⭐ WP_SESSION_91

Tutte e tre con evento-prova nominato in-sessione: (1) numericità ← Scoperta 4 (awk macOS bracket-class, `[ "" -ne 8 ]` vacuo); (2) bite-test decide ← Scoperta 2 + log (attesa sbagliata 3/5); (3) autorità da HEAD ← forge working-tree, «porta di TRE forge su quattro», denti T13-T16. Nessuna da riformulare.

## Emendamenti

- **A-BG62**: regola di arrotondamento DICHIARATA (half-up) per ogni trascrizione decimale→intero nei doc; correggere se b_boot 6.345→6.346 (o citare 6.345,5) in MEASURE90:27 e NEXT_SESSION:40.
- **A-BG63**: delta di continuità col SEGNO (−45.875 B) in MEASURE90 e GAP_TREND ove citato.
- **A-BG64**: token di grado ESPLICITO su b_peak(mediana) ovunque citata (RIPUBBLICATA, testata DA RIFARE — mai muta sotto intestazione «gradi»).
- **A-BG65**: i log di bite-test dichiarino «commit figlio di head=<sha>»; riga di ledger che lega log→sha di atterraggio.

## Kill-switch

- **KS-BG-93-1**: trascrizione doc di una cifra macchina con arrotondamento non conforme alla regola A-BG62 = FAIL del gate cifre (dente sulla coppia companion decimale→intero).
- **KS-BG-93-2**: riga di cifra sotto intestazione di gradi SENZA token di grado = FAIL (il grado si legge sulla riga, non si eredita dal silenzio).

## Refutazioni capitali: NO — doc==macchina regge sul perimetro; i quattro rilievi sono di trascrizione/etichetta, non di sostanza.

---

## NOTE DI TEAM (fase 2)

# Team CIFRE — Concilio WP-93 (fase 2)

Relatore: team-cifre. Sedie: 3 Klabnik, 4 Hejlsberg, 9 Gregg. I verbali individuali
(`verbale-3-klabnik.md`, `verbale-4-hejlsberg.md`, `verbale-9-gregg.md`) restano la
fonte VINCOLANTE; questo file li colloca.

## CONVERGENZE

1. **La delibera applicata a metà è il pattern comune** (3, 4). Klabnik: il PASS
   verdict-grade vive in un giudice spoofabile (F6) e ADVISORY-PASS esce 0; Hejlsberg:
   grammar v2 morde solo nel ramo `--same-rev`, head-check assente su REFUSE/trap,
   campaign v2 senza dente. Stessa malattia: enforcement su UN percorso, convenzione
   sugli altri. Cura comune: il dente morde su TUTTI i terminali/percorsi (A-SK-78/79,
   A-AH61/62/63).
2. **Le cifre non-macchina sono AUTORITÀ, non misure, e vanno marchiate** (3, 4, 9).
   Klabnik A-SK-75 (%ALLOW 2.8/46.25 revocate o provenienzate) e A-SK-77 (budget fuori
   corpus); Hejlsberg A-AH64 (max-FAIL = storia, mai verità); Gregg A-BG64/KS-BG-93-2
   (il grado si legge sulla riga, mai ereditato dal silenzio — b_peak mediana muta).
3. **L'identità di ciò che firma va dimostrata, non dichiarata** (3, 9). A-SK-78
   (self-tether `hash-object($0)` == blob HEAD) e A-BG65 (il log bite-test dichiara
   «commit figlio di head», non il proprio sha impossibile). Corollario condiviso di
   A-BG53/WP-90.
4. **Perimetro per COMPLEMENTO, non per allowlist** (3, 4). A-SK-80 (ogni .md
   committato esige riga manifest, verbali e doc GitHub inclusi: 153 cifre oggi fuori)
   e il buco `battery-9[1-9]*` che non copre le 3 cifre (4, Q2b).
5. **Fail-closed sulla tokenizzazione/parsing** (3, 4): A-SK-76 (run+lettera si
   rifiuta, non si tronca) converge con i residui minori di Hejlsberg (case
   substring-match su writer=, sha256 non ancorato).
6. **I bite-test con controllo negativo sono il criterio** (3, 9): i 6 forge di
   Klabnik hanno tutti il negativo che FALLISCE; Gregg conferma la lezione ⭐⭐
   «il bite-test decide» con evento-prova. KS-SK-93-2: i sei forge diventano T17-T22.

## CONFLITTI

- **PASS di Gregg vs «gate NON verdict-grade» di Klabnik: COMPATIBILI, perimetri
  disgiunti.** Gregg giudica il CONTENUTO: le cifre S-91.0 pubblicate coincidono con
  la macchina al byte, censimento manuale per NOME riga-per-riga. Klabnik giudica il
  CHECKER: la sua capacità di discriminare un forger futuro. Un doc onesto passa un
  gate bucato — le due cose coesistono. Il punto di frizione reale: il sigillo
  automatico che dovrebbe garantire *in futuro* ciò che Gregg ha verificato *a mano*
  è spoofabile (F6, judge_sha di un omonimo). Quindi il PASS di Gregg vale come fatto
  storico verificato dalla sedia 9, NON come garanzia riproducibile dal gate. Nessuna
  sedia dissente da questa collocazione.
- **Nessun conflitto 4↔9**: il max-FAIL di Hejlsberg è mechanism-level e verificato
  NON-live (m89 g3 PASS, m90 g2 PASS dai ledger) — coerente col censimento di Gregg.
- Dissensi da registrare: NESSUNO sostanziale; solo differenza di grado (3 e 4:
  refutazioni capitali; 9: nit di trascrizione).

## PRIORITÀ PROPOSTE per S-92.0

1. **A-SK-78 + bite-test F6 (giudice-copia)** — SÌ, è la falla di autorità massima:
   forgia lo strato di IDENTITÀ su cui poggiano tutte le altre firme (judge_sha nei
   ledger di Hejlsberg, i sigilli che renderebbero macchinale il censimento di Gregg).
   Blocca ogni PASS verdict-grade futuro (KS-SK-93-1). Primo atto.
2. **A-SK-79 + A-AH61 (grado nel codice d'uscita; grammar v2 su ENTRAMBI i percorsi
   di consumazione)** — blocca ogni consumazione di battery 9x (KS-AH-93-1).
3. **A-AH63 (dente campaign v2 pre-nascita) + A-AH64/A-SK-77 (igiene corpus:
   max-PASS-only, budget fuori)** — da committare PRIMA di qualunque m91; blocca la
   campagna.
4. **A-AH62 (head-check su REFUSE/trap) + A-SK-74/75/76/80/81 + T17-T22 permanenti**
   — hardening gate; 2.8/46.25 fuori entro S-92.0 (KS-SK-93-3).
5. **A-BG62/63/64/65 (arrotondamento dichiarato, segno, token di grado, log figlio)**
   — piccoli, non bloccanti, chiudibili in coda.

## S-91.0: CONSUMABILE?

**SÌ, con condizioni — nessuna ri-giudicatura.** Motivazione a tre gambe: (a) i buchi
sono *would-have-allowed* (mechanism-level), nessuna evidenza di forge avvenuto;
(b) il censimento indipendente della sedia 9 copre al byte il perimetro cifre
(seconda gamba oltre il judge_sha spoofabile); (c) il max-FAIL è verificato non-live.
Condizioni: (1) 2.8 e 46.25 non citabili finché non ri-provenienzate (A-SK-75);
(2) b_peak(mediana) riceve token di grado esplicito ovunque (A-BG64), testata resta
DA RIFARE; (3) ogni consumazione FUTURA di battery/campagne 9x avviene sotto i denti
nuovi — un PASS S-91.0 già consumato resta valido, nessun nuovo PASS verdict-grade
prima degli emendamenti di priorità 1-2.

---

# Team-misura — Concilio WP-93 (fase 2)

Relatore: team-misura. Sedie: 5 Bak, 6 Pedersen, 2 Matsakis. Fonti vincolanti: verbale-5-bak.md, verbale-6-pedersen.md, verbale-2-matsakis.md. Questo verbale riconcilia; NON supersede i verbali individuali.

## CONVERGENZE

**1. La coppia di Matsakis e la finestra di Pedersen sono lo STESSO emendamento, con un pezzo in più di Pedersen.** A-MS-57 (bracket: seconda lettura POST-dump, campo `outstanding_post=` in-band sulla riga mi_quiesce) e A-PP-66 (testimone a COPPIA: seconda riga POST-dump, stesso ckpt, campi pre/post) coincidono nella sostanza: chiudere la finestra load→dump con una rilettura. Divergono solo nella forma (campo sulla stessa riga vs seconda riga) e nella sufficienza: Matsakis ritiene pre==post==0 sufficiente sotto protocollo (client unico sequenziale esclude il transito completo intra-finestra); Pedersen dimostra che a livello MACCHINA non basta — una richiesta interamente contenuta nella finestra riporta outstanding a 0, e il monotono `REQUESTS` NON compila sotto mem-census (fetch_add r.944 è census-instrumentation-only). Quindi serve anche A-PP-67 (monotono di arrivo su `any(census-instrumentation, mem-census)`, campo `arr=`).

**Forma unificata deliberata (componibile in UNA riga testimone):** riga mi_quiesce unica, stesso ckpt, campi `outstanding_pre= outstanding_post= arr_pre= arr_post=`; dec OUTSTANDING → `Release`, load testimone → `Acquire` (A-MS-56, edge formale dec→load); pre==post==0 ∧ arr_pre==arr_post ⇒ finestra pulita. Un solo commit, tre emendamenti (A-MS-56/57 + A-PP-66/67) fusi. Il commento main.rs:262-265 va riformulato: «proof» solo con edge + bracket + arr.

**2. Stessa diagnosi sul claim.** Entrambe le sedie refutano l'estensione del testimone da ISTANTE a FINESTRA; entrambe confermano che la catena sottostante REGGE (teardown Vm/RetainSet completi prima del send; direzione fail-closed). A-PP-69 (free del `Vec<u8>` body lato axum fuori-testimone, dichiarato nel commento) accolto senza obiezioni.

## CONFLITTI

**1. Grado della refutazione (registrato, non riconciliato).** Pedersen la classifica CAPITALE («garanzia di finestra» falsa); Matsakis la classifica SOVRA-DICHIARAZIONE senza refutazione capitale. Il reperto è identico; differisce il grado. Il team registra: capitale per il claim-come-finestra (Pedersen vincola), PASS per la soundness del cfg-split e della catena (Matsakis vincola). Nessuna contraddizione di merito.

**2. Sufficienza del bracket.** KS-MS-93-1 (bracket pre/post) è SUSSUNTO da KS-PP-93-1 (coppia + arr uguali): il dente più severo vince — Δ di fase verdict-grade SOLO con coppia completa incluso `arr=`; testimone a riga singola O bracket senza monotono ⇒ ADVISORY. Matsakis non obietta: il suo Q2 già ammette che il testimone non è machine-proof contro traffico estraneo.

**3. Bak ↔ .out (non intra-team):** la dicitura «pooled … CI onesto» in MEASURE90 va emendata (A-BB71); nessun dissenso interno.

## GRADO DEL TESTIMONE A-PP-63 ATTUATO

**Attuazione PARZIALE, viva.** Vale come istante: outstanding==0 al load certifica request_end + teardown Vm/RetainSet completi (Q2 Pedersen verificato; Q1 Matsakis: cfg-split coerente, ogni inc ha dec/undo). NON vale come finestra: il Δ di fase resta **ADVISORY** finché non attuati, nello stesso commit: A-MS-56 (Release/Acquire), la riga unificata pre/post (A-MS-57≡A-PP-66), A-PP-67 (arr monotono compilato sotto mem-census), A-PP-69 (buco body dichiarato). Con questi quattro, la promozione di b_work a verdict-grade resta a portata di S-92.0: la decomposizione b_boot+b_work di WP-90 non è toccata nelle cifre, solo nel grado del sigillo di fase.

## GRADO DELLE BANDE (b_peak mediana)

Cifre di testata riprodotte 20/20 al byte (b_peak(med)=20.289.946, se=1.084.655); outlier DENTRO la mediana è lo stimatore giusto (A-BB70); w16.r1 mascherato flaggato per NOME ma effetto ZERO sulle testate. **La banda pooled df=18 [19.376.426, 21.173.319] NON è citabile come 95%** (F(2,16)=6,16, ICC≈0,51): declassata ADVISORY-conditional (A-BB71). **Unica banda citabile come 95%: la banda df=2 sulle medie per-W [16.633.973, 23.915.772]** (t=4,303). Qualunque banda n>W-points richiede F-test di scambiabilità ex-ante superato (KS-BB-93-1).

## PRIORITÀ PER S-92.0

1. **Riga testimone unificata** (A-MS-56 + A-MS-57≡A-PP-66 + A-PP-67, un commit) + commento main.rs riformulato + A-PP-69 — prerequisito della promozione b_work.
2. **Emendare MEASURE90**: riga «CI onesto» (A-BB71), frase «never via min» in forma normativa + additività etichettata tautologia (A-BB72).
3. **MAD flag in forma LOO** con stato DEGENERE dichiarato (A-BB69) prima della prossima campagna.
4. **A-MS-58** (ordine A-MS-53 ≺ A-DL-52) e **A-MS-55** (req_t0 salta `__census_self`) in design.
5. **A-PP-68** (trap `gate_in_flight=`/`deferred=1`) nella battery.
6. Kill-switch armati: KS-PP-93-1 (sussume KS-MS-93-1), KS-PP-93-2, KS-MS-93-2, KS-BB-93-1, KS-BB-93-2.

---

# Team ENGINE — Concilio WP-93 (fase 2)

Relatore: team-engine. Sedie: 8 Stogov (LSP/A-DS51), 7 Leijen (canale mimalloc iter-3), 1 Hoare (sigilli). I tre verbali individuali restano VINCOLANTI; questo file riconcilia e ordina.

## CONVERGENZE

1. **Lezione unica in tre grafie**: la lettera (di un design, di un gate, di una dicitura) va calibrata sul canale reale. Stogov morde con l'oracle vivo (q4a, hook-set, `& ` con spazio); Leijen morde col simbolo non esportato (`mi_heap_get_default` assente dal mimalloc v3 del tree, memcensus.rs r.634-637/1426-1428); Hoare morde con le grafie che eludono (nome-spazio-paren, turbofish, `ProbeWindow :: arm`). Nessuno dei tre contraddice gli altri: è la stessa legge WP-90 applicata a tre superfici.

2. **Le due refutazioni Leijen NON cambiano l'ordine della fase engine — lo CONFERMANO.** L'ordine roadmap (A-DS51 primo item engine) resta: le refutazioni bloccano il filone canale iter-3 (A-DL-52 va riscritto come A-DL-57, e senza Barrier(W) condiviso a due fasi il piano non è nemmeno ESEGUIBILE — A-DL-58), ma non toccano nulla di A-DS51. Il team registra che cambiano l'ordine INTERNO del filone misura: prima A-DL-57/58 (design emendato), e A-DL-59 (join pendenza-invisibile dai raw ESISTENTI `mi_theap_pages`/`mi_theap_bin`) è anticipabile subito perché è analisi, non campagna nuova.

3. **Il vincolo di sequenza A-TH-57-prefisso REGGE col nuovo A-TH-65, ma solo dopo l'emendamento.** La struttura narm⊆ntot per riga è sana; la dicitura «census TOTALE» è falsa alla lettera (grafia fissa, un solo file). A-TH-65 (ERE `ProbeWindow[[:space:]]*::[[:space:]]*arm` + sweep workspace + rami alias in TH49RE_V8) ri-fonda il vincolo sulla rete larga: il prefisso resta il modello giusto (Hoare lo riusa in A-TH-63 per nome-spazio-paren). Conseguenza d'ordine: **KS-TH-93-1 lega i sigilli al canale iter-3** — nessuna cifra m91 con probe VERDICT-grade finché A-TH-63/65 non mordono. I sigilli v10 sono quindi prerequisito del filone misura, NON di A-DS51.

## CONFLITTI

Nessun conflitto frontale. Due confini da dichiarare (non dissensi):

- **A-DL-59 vs KS-TH-93-1**: il join di Leijen usa raw già consumati; KS-TH-93-1 vieta CIFRE DI CAMPAGNA nuove, non l'analisi dei raw. Delibera del team: A-DL-59 eseguibile subito, ma con grado ADVISORY finché i sigilli A-TH-63/65 non mordono; VERDICT-grade solo dopo.
- **Scoping A-DL31**: Leijen restringe il dente (REFUSE solo per collisione ptr fra thread distinti entrambi post-probe; `heaps_total<W+1` = DECLARED). Non collide con Hoare, ma il dente scopato deve mordere sul proprio harness prima dell'uso (legge WP-88), e la modifica va per NOME nel commit.

## ORDINE PROPOSTO per S-92.0

1. **Sigilli v10 (Hoare, A-TH-62..67)** — PRIMO: corto, tutto pre-misurato riga-per-riga nel verbale, e sblocca KS-TH-93-1 che tiene in ostaggio il filone misura. Ogni dente nuovo morde sul proprio harness con decoy same-commit.
2. **A-DS51 fase 1 (Stogov, emendata A-DS58/59/60)** — CORPO della sessione: è il filone PRONTO (refutazione q4a emendabile pre-codice, nessuna dipendenza dai sigilli né dal canale) ed è il primo item engine di roadmap (gate KS-DS-88-3/89-3/91-1..3).
3. **Canale iter-3 (Leijen)** — ULTIMO, perché BLOCCATO dalle due refutazioni capitali: prima riscrittura A-DL-57 (probe on-thread + `mi_heap_of`, mai `mi_heap_get_default`) e A-DL-58 (Barrier(W) a due fasi, timeout fail-closed), poi campagna. Eccezione anticipabile: A-DL-59 come analisi ADVISORY dai raw esistenti, in coda alla fase 1 se resta budget.

Filone bloccato = canale iter-3. Filone pronto = A-DS51. Sigilli = abilitante trasversale.

## PREREQUISITI per NOME

- **Sigilli v10**: A-TH-62 (punto-nudo `//`-in-coda + riga `/*…*/`), A-TH-63 (rete prefisso nome-spazio-paren/turbofish + sweep), A-TH-64 (num-assert su OGNI cattura awk/grep-c), A-TH-65 (ntot ERE + sweep workspace + alias ProbeWindow), A-TH-66 (doc: UC_STATS lost-write, leak fallback, caso durante-proprio-dtor; nth61 ==1), A-TH-67 (`as` fine riga); decoy same-commit per ogni dente; KS-TH-93-2 armato.
- **A-DS51 fase 1**: A-DS58 (sei fixture v4 per NOME: x1 enum-abstract, x2 hook-set, x3 intersection-return-widen, x4 byref-hook-get, x5 utf8-classname, x6 iface-ctor-grandchild, pin length-prefixed in BYTE), A-DS59 (esenzione ctor keyed sul PROTOTIPO; w7 anti-esenzione per NOME nel commit fase 2), A-DS60 (cinque regole formattazione Q3 in unit dedicate); KS-DS-93-1..3 nel gate.
- **Canale iter-3**: A-DL-57 (riscrittura senza simbolo inesistente, A-DL31 scopato post-probe), A-DL-58 (Barrier a due fasi, cardinalità W asserita in-band), A-DL-60 (A-DL-54 = braccio barriera + gemello serializzato, pad cal-anchored 7.801.102 B, rendezvous FUORI bracket), A-DL-59 (join advisory); KS-DL-93-1..3 armati; A-TH-63/65 mordenti (KS-TH-93-1).

---


## ⚖️ SINTESI DI CONVERGENZA (compilata dalle ricevute fase 1+2 + estrazioni mirate; verbali = fonte vincolante)

**Verdetto complessivo: 9× CON EMENDAMENTI — nessuna opposizione; nei
team NESSUN conflitto frontale (un dissenso di GRADO registrato nel
team-misura). DIECI refutazioni capitali in sei classi, tutte morse a
macchina o dall'oracle:**

### Refutazioni capitali

1. **🔴 SEI forge NUOVI passati dal gate cifre v2, 6/6 (Klabnik)** — il
   v2 ha chiuso i quattro forge WP-92 ma ha APERTO porte nuove:
   (a) **giudice-copia che stampa PASS col judge_sha del giudice
   pristine** (la falla di autorità MASSIMA: il sigillo A-SK-67 è
   spoofabile da chi non esegue il giudice committato); (b) porta rev
   DECIMALI (decine di prefissi-commit antenati tutti-cifre
   indirizzabili); (c) %ALLOW senza autorità (2.8/46.25 il precedente);
   (d) tokenizzazione: `1.024.999B` (unità incollata) giudica solo
   `1.024`; (e) storia del budget = fonte non-macchina nel corpus;
   (f) prov su MiB tondi (chiusura totale sul sottospazio pubblicabile).
   → A-SK-74..81, KS-SK-93-1..4; i sei forge diventano denti permanenti
   T17-T22 (KS-SK-93-2). **KS-SK-91-1 resta NON sollevabile.**
2. **🔴 La delibera ledger v2 è applicata a metà (Hejlsberg)**: il
   blocco grammar v2 vive SOLO nel ramo --same-rev della consumazione;
   assert_head_unmoved manca su REFUSE porcelain e nel trap; la
   grammatica campaign v2 non ha alcun dente pre-nascita; un .out di
   generazione max-FAIL resta ammesso al corpus (oggi non-live,
   verificato). → A-AH61..64, KS-AH-93-1..3.
3. **🔴 Il testimone A-PP-63 è un ISTANTE, non una finestra
   (Pedersen+Matsakis, fusi dal team-misura)**: outstanding==0 alla
   riga non copre il DUMP che segue (arrivi durante il census
   invisibili; sotto mem-census manca anche il contatore monotono di
   arrivo); il load Relaxed non ha l'edge formale dec→load. → riga
   testimone UNIFICATA in UN commit: coppia pre/post (A-PP-66≡A-MS-57)
   + contatore arrivi arr= (A-PP-67) + Release/Acquire (A-MS-56);
   KS-PP-93-1 (sussume KS-MS-93-1). L'attuazione S-91.0 vale come
   PARZIALE: Δ di fase ADVISORY finché la coppia non è viva; la
   promozione b_work resta a portata di S-92.0.
4. **🔴 Il design A-DL-52 è irrealizzabile come scritto (Leijen, ×2)**:
   `mi_heap_get_default()` NON è esportata da questo mimalloc v3 (la
   via reale: probe on-thread + `mi_heap_of`, dente A-DL31 scopato
   post-probe); la barriera di picco NON si costruisce coi soli canali
   HTTP (serve Barrier(W) condiviso a due fasi con timeout
   fail-closed). → A-DL-57..60, KS-DL-93-1..3; **il canale iter-3 è
   BLOCCATO finché A-DL-57/58 non atterrano**; A-DL-59 (join
   pendenza-invisibile sui raw ESISTENTI) viene PRIMA di ogni
   strumento nuovo.
5. **🔴 La banda pooled df=18 NON è un 95% (Bak)**: le 5 run per W sono
   correlate (F-test intra-W significativo, ICC alto) — citabile solo
   la banda df=2; il MAD flag va in forma LEAVE-ONE-OUT (trova anche
   w16.r1, mascherato; effetto zero sulle testate). Ricomputo
   indipendente 20/20 al byte: le CIFRE reggono, gli STIMATORI di
   contorno si emendano. → A-BB69..72, KS-BB-93-1/2.
6. **🔴 L'elenco esenzioni della fase 2 A-DS51 è sbagliato su un punto
   (Stogov)**: il ctor CONCRETO in classe abstract è ESENTE (la chiave
   è il PROTOTIPO — interfaccia o ctor abstract — non la classe); un
   checker che vi fatala è REJECT (KS-DS-93-2). + quattro buchi fixture
   v4 oracle-morsi, regole di formattazione messaggi mancanti,
   comparatore in caratteri bandito (byte only). → A-DS58/59/60,
   KS-DS-93-1..3.

### Convergenze forti (dai team)

- **team-cifre**: la «delibera-a-metà» è un PATTERN trasversale (gate e
  ledger); identità = cosa DIMOSTRATA (hash del giudice in esecuzione),
  mai stampata; perimetro per COMPLEMENTO (A-SK-80 ≡ estensione
  A-SK-71); il bite-test resta il criterio di collocazione. PASS di
  Gregg (contenuto==macchina al byte) e «non verdict-grade» di Klabnik
  (checker forgiabile) sono compatibili: perimetri disgiunti.
- **team-misura**: A-MS-57≡A-PP-66 sono lo STESSO emendamento; la banda
  citabile per b_peak(mediana) è la df=2; pooled = ADVISORY-conditional
  con F-test ex-ante (A-BB71).
- **team-engine**: le refutazioni Leijen NON cambiano l'ordine engine —
  bloccano solo il canale; i sigilli (A-TH-63/65) sono PREREQUISITO del
  canale (KS-TH-93-1: nessuna cifra probe di m91 verdict-grade prima),
  non di A-DS51; il vincolo di sequenza A-TH-57-prefisso regge SOLO via
  A-TH-65 (ERE con spazi).
- **Gregg (igiene doc)**: doc==macchina al byte 20/20; regola di
  arrotondamento dichiarata (A-BG62), delta col segno (A-BG63), token
  di grado esplicito su ogni citazione di b_peak-mediana (A-BG64),
  bite-test log ancorato allo sha di atterraggio (A-BG65).

### Delibera di consumabilità (team-cifre)

**S-91.0 resta CONSUMABILE** — i suoi PASS valgono per ciò che
giudicavano (contenuto riprodotto al byte da Gregg; catene Hejlsberg
verificate) — A CONDIZIONE che: (1) ALLOW 2.8/46.25 siano REVOCATE o
munite d'autorità (A-SK-75) entro S-92.0 (KS-SK-93-3); (2) ogni
citazione di b_peak(mediana) porti il token di grado (A-BG64);
(3) i consumi FUTURI passino dal gate v3 coi denti T17-T22.

### Ordine vincolante di apertura S-92.0 (non rinegoziare)

1. **Gate cifre v3 — l'autorità, di nuovo (blocca ogni consumo
   futuro)**: A-SK-78 self-tether del giudice in esecuzione + A-SK-79
   grado nell'exit-code (ADVISORY-PASS=64; run_gate distingue) →
   A-SK-74 porta rev abolita → A-SK-75 ALLOW con autorità (revoca o
   motivazione di 2.8/46.25) → A-SK-76 tokenizzazione fail-closed →
   A-SK-77 budget fuori corpus → A-SK-80 perimetro per complemento →
   A-SK-81 prov etichettata → **T17-T22: i sei forge come denti che
   DEVONO fallire** (KS-SK-93-1..4).
2. **Ledger — la metà mancante (pre-m91)**: A-AH61 (grammar v2 su
   ENTRAMBI i percorsi di consumazione) + A-AH62 (head-check su
   REFUSE/trap) + A-AH63 (dente pre-nascita campaign v2, committato
   PRIMA di measure91) + A-AH64 (corpus solo max-PASS) —
   KS-AH-93-1..3.
3. **Sigilli v10**: A-TH-63/65 (prerequisito cifre probe m91,
   KS-TH-93-1) + A-TH-62/64/66/67 + KS-TH-93-2; quinta rete e num-assert
   generalizzati.
4. **Riga testimone UNIFICATA + emende misura (UN commit strumento + UN
   commit doc)**: A-PP-66/67 + A-MS-56/57 + A-PP-68/69 + A-MS-58 (ordine
   A-MS-53 ≺ A-DL-52 dichiarato) · emende MEASURE90/GAP_TREND: A-BB69
   LOO (w16.r1 per NOME) + A-BB70 politica outlier + A-BB71 pooled
   ADVISORY-conditional + A-BB72 + A-BG62/63/64/65 — KS-PP-93-1/2,
   KS-BB-93-1/2, KS-BG-93-1/2, KS-MS-93-1/2.
5. **A-DS51 fase 1 (checker LSP PURO)**: con A-DS59 esenzione
   riformulata (prototipo, non classe — KS-DS-93-2) + A-DS58 sei
   fixture v4 per NOME + A-DS60 regole di formattazione nei vincoli
   fase 1 — KS-DS-93-1..3; poi fase 2 come da A-DS57.
6. **Canale iter-3 (SBLOCCO)**: A-DL-59 join sui raw ESISTENTI (prima
   di ogni strumento nuovo) → design A-DL-57 (probe on-thread +
   mi_heap_of) + A-DL-58 (Barrier(W) 2 fasi) → A-DL-60 (A-DL-54
   braccio barriera + gemello serializzato) — KS-DL-93-1..3; solo
   dopo: measure91 (grammar v2, A-AH63 già armato).
7. **Design/deferred**: A-MS46 · A-TH55 · A-BB60/A-PP49 · backlog
   invariati.

### Kill-switch nuovi consolidati (attivi da subito)

KS-TH-93-1/2 · KS-MS-93-1/2 · KS-SK-93-1..4 · KS-AH-93-1..3 ·
KS-BB-93-1/2 · KS-PP-93-1/2 · KS-DL-93-1..3 · KS-DS-93-1..3 ·
KS-BG-93-1/2 — 23 nuovi. Ereditati: WP-92 (22) + WP-91 (27) +
WP-88/89/90 attivi. **KS-SK-91-1 NON sollevabile (refutazione 1:
la classe forge vive nel v2).**
