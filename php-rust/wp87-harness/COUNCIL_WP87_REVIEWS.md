# COUNCIL_WP87_REVIEWS.md — Concilio a 9 sedie su S-85.0 (canary VDL28 + VP R=9 + delibera peak) + programma WP-86(sessione)

**Convocato**: 2026-08-01, chiusura S-85.0 (regola utente 2026-07-26: 9 sedie
dopo OGNI sessione, mandato REFUTARE). Atti: NEXT_SESSION_WORDPRESS.md,
sessions/WP_SESSION_85.md, wp85-harness/MEASURE85_RESULTS.md + verdict85.out
+ verdict85.sh + measure85-campaign.sh + dl28-supplement.sh,
wp86-harness/COUNCIL_WP86_REVIEWS.md (gli ordini eseguiti), codice e raw dei
rispettivi perimetri. Verbali VINCOLANTI per il design WP-86(sessione).

## ⚖️ SINTESI DI CONVERGENZA

**Verdetto complessivo: 9× CONCORDO/PASS CON EMENDAMENTI — nessuna
opposizione.** Le CIFRE sono confermate a ricomputo indipendente (Gregg e
Bak al byte sui raw: calibrazioni, dl28s, i 9 peak, l'algebra della
scomposizione; Matsakis ha COMPILATO le sue prove con rustc; Stogov ha
interrogato l'oracle vivo). Gli ordini WP-86 sono giudicati eseguiti nella
lettera da tutte le sedie proprietarie. I colpi della tornata:

### Refutazioni convergenti (≥2 sedie indipendenti)

1. **A-MS29 REFUTATA a metà** (Matsakis, prove compilate): `&()` è soggetto
   a CONSTANT PROMOTION — `vm_gate_probe(&())` produce un token 'static
   legale, e il banking non richiede nemmeno la grafia (`thread::spawn`
   move). Il tipo eterno è ABITABILE senza essere SCRITTO: lo sweep
   letterale non pinna un tipo (convergente con Hoare Q4 e con la lezione
   «sigillo di tipo > allowlist»). Fix gratis: `anchor: &mut ()` (muore in
   E0716) → A-MS32/KS-MS-87-1.
2. **Il leaf module chiude il mint LETTERALE, non il mint per METODO**
   (Hoare): `production_gate` è pub(super) (visibile nei ~20 moduli figli
   di vm), `vm_gate` è pub crate-wide; le sweep sono CIECHE alle grafie
   `.production_gate(`/`.vm_gate(`; senza `#![forbid(unsafe_code)]` il
   transmute in-crate resta esprimibile, e un `impl Copy` duplicherebbe
   token → A-TH38, KH87-1.
3. **A-TH35 NON sound sotto unwind** (Hoare): `_emit_guard` è dichiarata
   DOPO i vec degli sfollati — il commento «declared FIRST» è FALSO nel
   testo; in unwind la guardia si disarma PRIMA che gli sfollati droppino.
   Fix: arm a inizio put → A-TH36 (+A-MS35: `unit_cache_key_present` fuori
   guardia).
4. **KH86-1 ratificata SOLO come PONTE** (Hoare + Hejlsberg convergenti,
   meccanismo SPIEGATO): la feature entra nel `-C metadata` → mangling di
   TUTTI i simboli → l'hash discrimina strutturalmente dove nm non può
   (dead-strip); ma la metà hash sposta l'onere sulla pulizia della matrix
   (di nuovo protocollo). Giudice intrinseco ordinato: `#[used]` static
   marker cfg-gated (A-TH37, KH87-2) + matrix pinnata dal `.done` della
   battery (A-AH40, KS-AH-87-1) + `rustc -V` in archivio (A-AH41).
5. **A-SK38 VIOLATO nel codice del verdict85** (Klabnik): il ramo elsif
   stampa «VDL28 PASS» ANCHE quando il bf same-thr ha già alzato bfail —
   riga PASS greppabile da blocco sporco. Il pattern giusto ha già un NOME:
   A-MS28 raccogli-poi-emetti → A-SK44, KS-SK-87-3. Più: manca il dente
   diretto ord≥2 nel parse dl28s (A-SK45, convergente con Pedersen Q3).
6. **La battery resta forgiabile e il 15/15 è CABLATO** (Klabnik): OUT e
   .done fuori repo non-ledgerati (coerenza OUT↔.done ≠ prova che la
   battery sia girata), «(15/15)» stampato fisso, NESSUN porcelain check in
   testa alla battery → A-SK41/42, KS-SK-87-1.
7. **Classe MODEL-GRADE istituita** (Klabnik + Gregg convergenti): le cifre
   dalla scomposizione additiva vengono da un raw VOID — legali come
   MODELLO con tag di provenienza, MAI load-bearing in una delibera; la
   delibera ×W va riformulata appoggiata SOLO a m85.dl28s → A-BG42,
   KS-SK-87-4/KG-87-2; contro-prova ordine invertito prima di ogni uso
   (A-BB45, KB-87-1: predizione nominata hello-own(ord2)=6.842 B; confound
   trovato da Bak NEI raw: 996.838 B condivisi nel walk a 2 entry).
8. **PER-THREAD = proprietà del PROTOCOLLO sequenziale** (Leijen + Bak
   convergenti): il canary falsifica specchio e finestra congelata, ma il
   sequenziale non prova la concorrenza — trasferire il claim a dispatch
   concorrente/W>2/fixture con distruttori esige re-canary sotto overlap
   (A-BB47, KL-87-2, KB-87-3). La refutazione di sede del canary path-len
   (bracket solo su lower_source) è ACCETTATA da Bak: «avrei dichiarato
   VOID un risultato vero».
9. **phys_peak<phys SPIEGATO alla riga** (Leijen — A-DL30 chiusa):
   `PHYS_PEAK.fetch_max` vive solo nel throttle (ogni 16384 eventi), il
   dump stampa phys fresco con peak stantio — artefatto di campionamento,
   fix da una riga (fetch_max in testa al dump) → A-DL32, KL-87-3; righe
   mi_proc riammesse solo dopo.
10. **«Envelope SALITO» = vizio d'ordine statistico** (Gregg + Bak): max su
    R=9 domina stocasticamente max su R=3 — la salita +12,2MB è DENTRO lo
    spread non attribuito; ogni confronto cross-campagna nomina driver_sha
    E R di TUTTI i lati (A-BG41, KG-87-3). KG-86-2 violata in forma e
    sanata a ricomputo (delta 84→85 = solo wrapper, inerzia favorevole);
    A-BG37 nel supplement è OVERCLAIM documentale (zero head_unmoved) →
    A-BG40 + correzione MEASURE85.
11. **Lo sweep A-PP31 è vacuo al falso-positivo** (Pedersen): `grep -q
    'w=1'` è soddisfatto da `w=10` o da un commento — la presenza di un
    token non è un rifiuto → A-PP35 (digit-guard + parser condiviso con
    bite-test), KS-PP-87-1. Il pre-warm ha CREATO una cecità nuova (la
    vera-req1 del thread esce dalla copertura; insert+evict compensato
    invisibile alla cardinalità) → A-PP33/34; phantom pre-publish
    (connect-ok-abort) shifta il mapping senza righe → dispatch-count
    per-thread in-band (A-PP36, KS-PP-87-3, convergente Klabnik Q4).
12. **Stogov, scoperta GRAVE fuori perimetro di sessione**: la covariance
    LSP non è verificata AFFATTO — `C::m(): int` vs `P::m(): string`
    compila e gira dove l'oracle fatala: correct-or-absent VIOLATO (classe
    sbagliata invece che assente) → A-DS35, PRIMO candidato engine della
    ROADMAP. L'hoisting phpr = semantica OPCACHE (verificato sull'oracle
    con enable_cli: braccio opcache riprodotto esattamente) — divergenza
    dal CLI-oracle da catalogare. `methods_ci` può divergere da solo su
    unità ELISA (fuori dominio a_ds17) → A-DS34/KS-DS-87-1; ordine eventi:
    il prefisso supersede* è infalsificabile (mutua esclusione) e un main
    superseded muore SENZA main_evicted — doc da emendare (A-DS36).

### Ordine vincolante di apertura WP-86(sessione) (non rinegoziare)

1. **Sanatorie immediate di verdetto e documento**: A-SK44 (verdict85 VDL28
   raccogli-poi-emetti per-blocco + RE-RUN del verdict) · A-SK45 (dente
   ord≥2 in dl28s) · riformulazione delibera ×W su SOLO m85.dl28s con tag
   MODEL-GRADE per la scomposizione (A-BG42) · correzione overclaim A-BG37
   in MEASURE85 + A-BG40 (head_unmoved nel supplement) · retro-dichiarazioni
   A-AH42 (A-AH38+dry-run) e A-BB48 (A-BB44 len≥500, predizione 2.002 B) ·
   A-DS34 nota methods_ci · doc A-DS36.
2. **Sigilli v4**: A-MS32 (`&mut ()`) · A-TH36 (arm a inizio put + commento
   corretto) · A-MS35 (guardia in key_present) · A-TH38 (sweep
   `.production_gate(`/`.vm_gate(` + transmute/impl-Copy ==0) · A-TH39
   (functional-update ==0 su CachedUnit) · A-MS33 (!Send/!Sync via
   PhantomData<*mut ()>) · A-TH37 (#[used] marker + controllo positivo che
   DEVE fallire il gate su build contaminato, KH87-2).
3. **Battery v4**: A-SK41 (stamp rev+sha LEDGERATO committed) · A-SK42
   (k/k contato + porcelain fail-closed in testa) · A-AH40 (matrix pinnata
   nel .done, mai tail -1) · A-AH41 (rustc -V in archivio matrix) · A-SK43
   (allowlist bande ± + MiB-only nei MEASURE8[4-9]).
4. **Probe/eredità**: A-PP35 (digit-guard subito + parser reqns condiviso
   con bite-test) · A-PP33/34 (arm no-warm + membership set-based) · A-PP36
   (dispatch-count per-thread in-band, verdict fail-closed) · A-MS34
   (PROBE_ACTIVE o limite dichiarato) · A-DS37 (template VA census con
   vitalità in-band GIÀ scritto) · A-DL33/34 (burst fail-closed in pipeline
   + semantica deflazione/underflow di finish()).
5. **Attribuzione spread VP**: A-DL32 (fix phys_peak, 1 riga) → campagna
   coppia stessa-sera ABBA `MIMALLOC_PURGE_DELAY=0` vs default R≥8-9 con
   env in-band + join peak_rss↔time -l su mono_ms (A-BB46/A-DL35 forma
   unificata; soglie: spread(0)<0,5×spread(default) o contatore ≥80% del
   delta) — SOLO DOPO un eventuale pin (KL-87-1/KB-87-2).
6. **Modello scomposizione**: A-BB45 contro-prova ordine invertito
   (predizione nominata 6.842 B) — fino ad allora KB-87-1 (mai addendo di
   budget).
7. **Metà fisica + concorrenza**: A-DL31/A-DL35 (mi_bin thr= + composizione
   fixture in-band, chiusura 3.605.572±5%) · A-BB47 (canary sotto overlap
   reale) — con A-MS27 restano LE precondizioni della via registry.
8. **ROADMAP**: A-DS35 (covariance LSP, correct-or-absent) come PRIMO item
   engine + catalogazione hoisting-opcache; poi ripresa da
   [[php-rust-todo-master]]. Revert policy KS-DS-80-3 invariata.

### Kill-switch nuovi consolidati (attivi da subito)

| KS | Condizione | Effetto |
|---|---|---|
| KH87-1 | mint per metodo fuori allowlist prima di A-TH38 | sigillo ADVISORY, campagne VOID |
| KH87-2 | positive control A-TH37 (build contaminato) PASSA il gate | KH86-1 riaperta, cifre da binario VOID |
| KH87-3 | panic in put/get osservato con campagna proseguita | Binding Rule 4 violata, run VOID |
| KS-MS-87-1 | firma probe ancora `&()` o anchor promotabile | sigillo di test ADVISORY |
| KS-MS-87-2 | impl Send/Sync/Clone/Copy su VmGate | delibera di Concilio, mai edit in sessione |
| KS-MS-87-3 | riga "census probe panicked" citata in run axum con hook | attribuzione VOID finché A-MS34 |
| KS-SK-87-1 | battery su tree non-porcelain o stamp non ledgerato | battery VOID |
| KS-SK-87-2 | banda ± non in allowlist | figura non coperta |
| KS-SK-87-3 | riga PASS/derivata emessa da blocco con fail | verdetto del blocco VOID |
| KS-SK-87-4 | cifra VOID-sourced senza tag o load-bearing in delibera | delibera VOID |
| KS-AH-87-1 | ENFORCE contro archivio matrix diverso da quello nel .done (post-A-AH40) | campagna VOID |
| KS-AH-87-2 | mismatch hash risolto ri-buildando senza attribuzione per NOME | arm VOID |
| KB-87-1 | scomposizione additiva come addendo di budget pre-A-BB45 | cifra VOID |
| KB-87-2 | pin peak fondato con spread non attribuito | pin VOID (envelope-only, trend dichiarato) |
| KB-87-3 | registry deliberata su canary solo-sequenziale senza A-BB47 | "per-thread sotto concorrenza" CANDIDATO |
| KS-PP-87-1 | sweep soddisfatto da token-presenza | sweep VACUO, eredità non provata |
| KS-PP-87-2 | claim retained oltre {steady, warmed-req1} senza A-PP33 | declassato (sostituisce KS-PP-86-2) |
| KS-PP-87-3 | cifra net da run senza dispatch-count in-band==atteso | riga = envelope, mai attribuzione |
| KL-87-1 | pin peak deliberato prima della coppia purge-arm R=9 | delibera NULLA (estende KL-86-1) |
| KL-87-2 | budget ×W citato per concorrenza/W>2/distruttori senza re-canary | riclassificato IPOTESI |
| KL-87-3 | riga phys_peak citata pre-A-DL32 | esclusa dal corpus (estende KL-86-3) |
| KS-DS-87-1 | "sig esaustiva" citata per unità elisa senza nota A-DS34 | claim in-domain-only |
| KS-DS-87-2 | finestra census da template senza vitalità in-band | cifre VOID |
| KS-DS-87-3 | put-path con supersede/evizione co-occorrenti senza pin d'ordine | ordine ADVISORY |
| KG-87-1 | supplement senza attempt-ledger | raw sostitutivo VOID |
| KG-87-2 | cifra da raw VOID senza MODEL-GRADE(src=…) | citazione VOID; verdict che la consuma NULL |
| KG-87-3 | confronto envelope/min a R diversi senza R dichiarato per lato | confronto VOID |

---

## VERBALI INTEGRALI

# VERBALE — Tony Hoare, sedia 1, Concilio WP-87

**VERDETTO: CON EMENDAMENTI** — i miei ordini WP-86 sono stati eseguiti fedelmente, ma ho trovato quattro superfici residue verificate nel codice. Il mandato è refutare: refuto.

**Q1 — A-TH32: il leaf module chiude il mint LETTERALE, non il mint per METODO.** Verificato: le 3 espressioni `VmGate(` vivono in `mod gate` (vm/mod.rs:486-540), campo privato, pin awk anchor-scoped (gate-lever-pins.sh:136-145) corretto. Ma: (a) `production_gate` è `pub(super)` (vm/mod.rs:512) ⇒ visibile in TUTTI i ~20 moduli figli di vm (run.rs, host.rs, calls.rs… >1 MB di codice); il pin `production_gate( ==2` copre solo vm/mod.rs e la sweep GATE_SWEEP (pins:195-207) usa regex `VmGate[(]|vm_gate_probe` — CIECA a `.production_gate()`. Un call-site in vm/run.rs compila e passa la cintura in silenzio. (b) `vm_gate` è `pub` (vm/mod.rs:521): mintabile crate-wide da chiunque abbia `&MainUnit`; pinnato solo in worker_pool.rs ==1, non in vm/mod.rs né in sweep. (c) lib.rs NON ha `#![forbid(unsafe_code)]` (il crate contiene FFI gd/tidy/xslt): `transmute::<(),VmGate>` resta esprimibile in-crate; e un `impl Copy for VmGate<'_>` in-crate duplicherebbe token senza toccare il campo. Il sigillo rustc vale per la costruzione letterale; il perimetro reale è la visibilità dei metodi mint.

**Q2 — KH86-1: RATIFICO nm+hash solo come PONTE.** La refutazione è corretta e onesta (gate-binary-noprobe.sh:11-19). Ma la metà hash sposta l'onere sulla pulizia del build della matrix — che è di nuovo PROTOCOLLO, l'esatto difetto che KH86-1 doveva eliminare: matrix contaminata ⇒ hash combacia, nm muto (dead-strip). Ordino il ritorno a un giudice intrinseco del binario (A-TH37).

**Q3 — A-TH35: sound sul cammino normale, NON sound sotto unwind.** Verificato: pre-check in put (vm/mod.rs:16379-16383) e get (:15976-15980), arm a :16447, drop espliciti :16495-16497 — sul cammino normale i drop degli sfollati sono guardati. Ma il commento «declared FIRST so it drops LAST» (:16444-16446) è FALSO nel testo: `_emit_guard` è dichiarata DOPO `superseded/victim/replaced` (:16394-16396). In unwind i locali droppano in ordine inverso di dichiarazione ⇒ un panic nella finestra di emissione disarma la guardia PRIMA che gli sfollati droppino; e un panic nella fase di mutazione (guardia mai armata, borrow RefCell rilasciato dall'unwind della closure) fa droppare gli sfollati non guardati. Cargo.toml non fissa `panic="abort"` ⇒ le finestre esistono. Severità mitigata dal fail-fast, ma la guardia certifica meno di quanto dichiara.

**Q4 — grafie che ancora sfuggono (trovate, per NOME):** (1) functional update `CachedUnit { …, ..other }` copia `main_program: Some` senza nessuna delle tre grafie pinnate; (2) `main_program: mp` (variabile che porta il Some) elude sia il pin `main_program: Some(` (pins:235) sia TH33RE (:266 — lo shorthand richiede la virgola adiacente); (3) le varianti di spaziatura `main_program:Some(`; (4) le grafie-mint `.vm_gate(`/`.production_gate(` di Q1. A-TH34 (pins:429-476) è verificato corretto con decoy che discrimina.

**Emendamenti:**
- **A-TH36**: spostare `UcEmitGuard::arm()` a inizio `unit_cache_put` (subito dopo il check enabled, PRIMA di kpath/superseded) — chiude entrambe le finestre di unwind, rende ridondante il pre-check, e correggere il commento falso.
- **A-TH37**: `#[used] static` marker byte-string cfg-gated sotto `vm-gate-probe` ⇒ la metà strings torna SUFFICIENTE (llvm.used sopravvive a -dead_strip); positive control: build contaminato DEVE fallire il gate; hash resta cintura.
- **A-TH38**: sweep workspace `.production_gate(`/`.vm_gate(` ==0 fuori dai siti nominati + pin `transmute.*VmGate` ==0 e `impl .*Copy.* for VmGate` ==0.
- **A-TH39**: pin `[.][.]` ==0 dentro i literal `CachedUnit {` + estendere TH33RE a `main_program[[:space:]]*:` non-Some (ponte fino ad A-MS27, che resta la VERA chiusura tipata).

**Kill-switch:**
- **KH87-1**: se compare un mint per metodo fuori allowlist prima di A-TH38 ⇒ sigillo VmGate declassato ADVISORY, campagne con quel binario VOID.
- **KH87-2**: se il positive control di A-TH37 (build contaminato) PASSA il gate ⇒ KH86-1 torna aperta, ogni cifra da binario è VOID.
- **KH87-3**: se un panic in put/get viene osservato con campagna proseguita ⇒ Binding Rule 4 violata, run VOID.

Firmato: T. Hoare, sedia 1.

---

# VERBALE — Matsakis, sedia 2, Concilio WP-87 su S-85.0. Perimetro: ownership/borrow/sigilli. (Verifiche eseguite sul codice e con rustc — repliche compilate.)

**VERDETTO: CON EMENDAMENTI VINCOLANTI.** A-MS30/A-MS31/A-TH35 eseguite a regola; **A-MS29 è REFUTATA a metà**: il lifetime lega, ma l'anchor è promotabile — il token eterno resta mintabile gratis nei build test/probe.

**Q1 — REFUTATA.** `vm_gate_probe(anchor: &()) -> VmGate<'_>` (vm/mod.rs:534-539): l'elisione lega davvero il token all'anchor. MA `&()` è soggetto a *constant promotion*: ho compilato `let g = vm_gate_probe(&()); want_static(g)` — **PASSA** (rustc, exit 0), come passano `&UNIT` (static item) e `Box::leak`. Peggio: **il banking non richiede la grafia** — `std::thread::spawn(move || { let _banked = g; })` compila senza mai scrivere `VmGate<'static>` (promo3, exit 0). Il claim del doc «no VmGate<'static> exists anywhere in the workspace» (:530) è falso come garanzia: il tipo eterno è ABITABILE senza essere SCRITTO. Il fix è gratis: **`anchor: &mut ()`** — verificato: la stessa prova muore in **E0716** (`&mut` di temporary non è mai promosso; `static mut` è unsafe/negato). Residuo irriducibile solo `Box::leak(Box::new(()))` → `&'static mut ()`, classe già nominata in WP-86. I due call-site reali (worker_pool.rs:924-925, :1308+) usano anchor locale e migrano con un `mut`.

**Q2 — TIENE.** worker_pool.rs:282 (AssertUnwindSafe esterno) e :295-306: il probe interno cattura SOLO `worker_idx: usize` (Copy, UnwindSafe) — il `catch_unwind` nudo a :297 compila perché rustc ha VERIFICATO l'unwind-safety, più onesto dell'esterno. Sound. **PERÒ**: per il commento stesso a :270-280, in modalità axum il global abort hook (main.rs) scatta PRIMA dell'unwind — nel build mem-census dell'arm axum (esattamente dove A-DL24 vive) **il messaggio distinto :301-303 non stamperà MAI**: l'attribuzione A-MS31 vale solo nei contesti test senza hook. Limite non dichiarato nel commento :289-294.

**Q3 — TIENE con un buco advisory.** Guardia verificata: check in `unit_cache_put` :16379, in `unit_cache_get` :15976, arm :16447 dopo chiusura borrow :16443; drop-order di `mains` documentato e sound (:16262-16265, Rc non arriva a 0 dentro il `Ref`). Fuori guardia (censimento completo dei `UNIT_CACHE.with`): `unit_cache_key_present` :15940 (unico caller prod :7033, ramo else-if DOPO un get — coperta solo transitivamente), enumeratori :16211/:16226, `memcensus_unitcache_main_rows` :16242 (collect-then-emit rispettato, census_line a :16289-16291 fuori borrow), dump :1484, test :18031/:19159/:19212 (A-DS33 corretto: bind poi drop). Nessuno oggi raggiungibile da un Drop (i CachedUnit displaced contengono solo Rc di dati piatti), ma `key_present` è sulla corsia acquire: se un futuro caller la chiama senza get a monte, l'interleave torna muto.

**Q4 — REFUTATA.** Sweep verificato: gate-lever-pins.sh:146-158, `count_nontest "VmGate<'static>"` LETTERALE. Evasioni: (a) `VmGate < 'static >` — Rust legale, regex cieca; (b) `type G<'a> = VmGate<'a>;` poi `G<'static>`; (c) la decisiva: **occupancy senza spelling** (Q1/promo3). Una grafia non pinna un tipo; solo A-MS32 chiude per rustc.

**EMENDAMENTI**
- **A-MS32**: firma `vm_gate_probe(anchor: &mut ())`; aggiorna i 2 call-site e il doc :526-533 (prova E0716 citata). Lo sweep resta come belt.
- **A-MS33**: `VmGate` !Send/!Sync (secondo `PhantomData<*mut ()>`): anche un token evaso non attraversa thread — coerente con :477. Attenzione: l'assert negativo richiederebbe la grafia `'static` → usare un fn-check esente dallo sweep.
- **A-MS34**: flag PROBE_ACTIVE (atomic) letto dal global hook, o dichiarazione esplicita del limite nel commento — oggi A-MS31 è morta nell'arm per cui è nata.
- **A-MS35**: guard-check anche in `unit_cache_key_present` (3 righe, stessa classe di :15976).

**KILL-SWITCH**
- **KS-MS-87-1**: firma probe ancora `&()` o call-site con anchor promotabile ⇒ sigillo di test ADVISORY; i positive-control non valgono come evidenza del sigillo.
- **KS-MS-87-2**: qualunque impl che renda VmGate Send/Sync/Clone/Copy ⇒ delibera di Concilio, mai edit in sessione.
- **KS-MS-87-3**: riga "census probe panicked" citata come attribuzione in un run axum con hook installato ⇒ attribuzione VOID finché A-MS34 non atterra.

*Niko Matsakis — un lifetime elising su un anchor promotabile è una catena legata a un gancio che il compilatore stesso solleva a 'static: si cambia il gancio, non la catena.*

---

# VERBALE KLABNIK — sedia 3, Concilio WP-87

**VERDETTO: CON EMENDAMENTI — 4 evasioni nuove trovate, di cui una violazione LETTERALE di A-SK38 nel codice del verdict e una forgeabilità residua di A-SK36.**

**Q1 — A-SK36 v3: la forge resta possibile, e lo stamp 15/15 è cablato.**
Verificato: `battery-85pre.sh:20-22` (`: > OUTF`, `rm -f .done`), `:57` scrive `.done` solo su PASS; `battery-equivalence.sh:104-121` ricomputa sha256(OUT) e pretende la riga ancorata unica e finale. Ma OUT e `.done` vivono FUORI repo, non firmati, non committati: chiunque può scrivere a mano un OUT con la riga `== BATTERY-85PRE PASS (15/15) git=X ==` come ultima riga e ricalcolare sha256 — il checker prova solo la coerenza OUT↔.done, mai che la battery sia GIRATA. Due aggravanti: (a) `battery-85pre.sh:55` stampa "(15/15)" **cablato**, non contato — con 14 gate nella lista stamperebbe comunque 15/15; (b) **nessun porcelain check**: `:24` prende `GIT_REV` da HEAD ma la battery può girare su tree sporco (crates modificate uncommitted) e stampare OK a rev X per codice che non è rev X. Il grep due-forme (`:113`) non è ambiguo: la forma nuda senza sha cade su `:118`.
→ **A-SK41**: lo stamp rev+sha va appeso a un file evidence COMMITTATO al momento della battery (stessa legge del ledger A-SK33/A-SK37); il checker confronta col committed stamp. **A-SK42**: k/k contato + `git status --porcelain` fail-closed in testa alla battery. **KS-SK-87-1**: battery su tree non-porcelain o stamp non ledgerato = battery VOID.

**Q2 — A-SK40: la banda ± è un'eccezione senza padrone e "MB==MiB" mente al lettore.**
Verificato `gate-measure-cifre.sh:165-180`. Il companion 0,5·10^-dec è arrotondamento legittimo bounded (240,83→"241 MiB" passa con dec=0: bias di mezza unità scegliendo il verso — accettabile ma da vincolare a dec≥1 nelle delibere). L'evasione vera: `:180` blanka **qualunque** `N±M unit` — e con M a 1-2 cifre (fuori scope `:12`) "232±16 MiB" passa companion, banda e corpus: un doc può allargare il pin a piacere. Terzo: `:156-157` dichiara MB==MiB — il gate forza la matematica binaria sotto etichetta SI: "19,69 MB" dove SI darebbe 20,65. Coerente dentro, menzognero fuori.
→ **A-SK43**: allowlist esplicita delle bande legali (oggi: 232±1 MiB, e basta); vietare `MB` nudo nei MEASURE8[4-9] (imporre la `i`). **KS-SK-87-2**: banda ± non in allowlist = figura non coperta.

**Q3 — A-SK38 VIOLATO nel codice: PASS emesso da blocco sporco.**
`verdict85.sh:103-108`: se scatta il bf same-thr (`:103`, bfail++), il ramo `elsif` (`:107`) stampa comunque "VDL28 PASS" — riga PASS da blocco con fail, contro la lettera di A-SK38 (e la riga derivata `:95` esce prima dei check a valle). Il verdetto globale resta FAIL, ma la riga è greppabile come PASS. Il pattern giusto esiste già per NOME: **A-MS28 raccogli-poi-emetti** — non adottato qui.
Parser latenti (grep eseguito su wp7*/wp8*): `wp84-harness/verdict84.sh:129` `fixture=(\S+) rev=` si spezza su path con spazio (fail-closed spurio, non falso PASS); `verdict84.sh:85` ignora il path del tutto; `verdict83.sh:120-121` sono token sicuri. Nessun altro `(\S+)$` su path terminale.
→ **A-SK44**: verdict85 VDL28 a buffer per-blocco, flush solo con bfail==0. **KS-SK-87-3**: riga PASS/derivata emessa da blocco con fail = verdetto del blocco VOID.

**Q4 — dl28: un dente manca (ord≥2) e le cifre da raw VOID servono una classe nuova.**
Il phantom connect-ok-timeout (`dl28-supplement.sh:46-51`, `curl -m 1`) È possibile: il retry ridispatcherebbe hello. Le topologie pericolose sono quasi tutte chiuse (`verdict85.sh:59` duplicate ord=1; `:103` same-thr fail-closed), e phantom+success sullo stesso thread è un HIT senza riga nuova (net intatto). Ma manca il dente diretto: **zero righe ord≥2 in m85.dl28s** — proprio il modo di rottura che ha voidato m85.dl28 (pad a ord=2) è rilevato solo indirettamente. → **A-SK45**: tooth `ord!=1 count==0` nel parse dl28s.
Le cifre 🔵 (`MEASURE85_RESULTS.md:40-46`) da raw VOID: il corpus cifre (`gate-measure-cifre.sh:114-117`, glob `m85*`) include i raw VOID **indiscriminatamente** — il gate non distingue validità. E il residuo 7.343.135 B (VOID-derived) è citato nella delibera ×W (`:63-69`). Per la mia spec NON è legale senza classe: serve **MODEL-GRADE**: cifra da raw VOID citabile solo con tag di provenienza `[void-raw: <label>]`, mai load-bearing in un verdetto (qui il canary regge da solo — la delibera va riformulata per appoggiarsi SOLO a m85.dl28s). **KS-SK-87-4**: cifra VOID-sourced senza tag in un MEASURE, o load-bearing in una delibera = delibera VOID.

*— Klabnik, sedia 3.*

---

# VERBALE — Hejlsberg, sedia 4, Concilio WP-87

**VERDETTO: CON EMENDAMENTI.** La campagna S-85.0 è sana nel mio perimetro; la scoperta KH86-1 è corretta e correttamente cablata; ma due debiti di forma (Q2, Q4) vanno chiusi per NOME.

**Q1 — Il meccanismo è la METADATA, non il codice del probe.** La feature `php-runtime/vm-gate-probe` entra nel fingerprint di Cargo e quindi nell'`-C metadata` dell'unit php-runtime: il set di feature è input dell'hash di metadata, che è suffisso di mangling di **TUTTI** i simboli del crate. Accendere la feature ricompila php-runtime con simboli globalmente rinominati → il binario linkato differisce ovunque, mentre `vm_gate_probe` stesso, senza caller nell'unit-graph del bin, non viene mai tirato dal linker (Cargo.toml php-runtime:26-30 conferma: mint solo dietro feature, dev-dep in php-server:24-25). Perciò l'hash discrimina **strutturalmente** dove nm non può: una build feature-diversa non può collidere con la riga matrix. Giudizio: hash==matrix è discriminatore SUFFICIENTE per l'identità di feature, e **fail-closed** verso ogni non-determinismo residuo — path assoluti, RUSTFLAGS, bump toolchain producono mismatch → VOID, mai falso PASS (release ha incremental off; stessa macchina/path/lock → deterministico in pratica). Il verbo matrix è pulito per costruzione: gate-feature-matrix.sh:62-71 rifiuta su porcelain whole-repo, e A-AH12 (:85-97) logga l'hash della **plain** `cargo build`, lo stesso verbo di `build_arm`; il gate script è nel perimetro dirty-check della campagna (:71-72). Buco residuo: il log matrix **non registra `rustc -V`** — due archivi same-rev con toolchain diversi sono indistinguibili (→ A-AH41, e si salda a Q4).

**Q2 — Apertura NON dichiarata: violazione di forma, non di sostanza.** A-AH38/KS-AH-86-1 legittimamente non eseguiti (nessuna fase slope/base-arm, A-BB43), ma né WP_SESSION_85 §Residui né NEXT_SESSION li nominano: la disciplina «mai chiusure in silenzio» vale anche per le NON-esecuzioni ereditate. **Ordino la forma**: (a) l'apertura va appesa per NOME nei residui della prossima sessione; (b) il dry-run resta **obbligatorio PRIMA** del prossimo uso base-arm, e A-AH38 (scope-file su split_lock) deve atterrare **prima** del dry-run — un dry-run sulla v2 vecchia certifica l'oggetto sbagliato e andrebbe rifatto. KS-AH-86-1/2 restano armate invariate.

**Q3 — Coerente, e a costo marginale ZERO.** La riga `lever-fixtures2:…,crates/php-runtime/` (battery-equivalence.sh:69) non produce alcun falso rifiuto ADDIZIONALE: ogni cambio a php-runtime trippa già il dente (i) (:75-78, delta `crates/` vuoto) e rifiuta TUTTA l'equivalenza. Il manifest è la dichiarazione, (i) resta belt — esattamente il mio ordine WP-86. Il suo valore si attiva solo se (i) venisse ristretto: allora la larghezza smette di essere ridondante. Mantenere.

**Q4 — Ambiguità REALE, serve il pin.** `MTX=$(ls …feature-matrix.$GIT_REV.*.log | tail -1)` prende l'archivio lessicograficamente ultimo: una battery rilanciata (o un run standalone del matrix-gate, magari dopo bump toolchain) alla stessa rev produce un secondo archivio e la campagna ENFORCEerebbe contro una matrix che **non è quella dentro il 15/15** — la catena battery⊃matrix si spezza in silenzio. Oggi 368c91d ha un solo archivio (verificato), ma la classe esiste.

**Emendamenti**:
- **A-AH40**: la battery scrive nel `.done` `matrix=<filename> sha256=<…>`; la campagna risolve $MTX **dal .done**, mai da `tail -1`; archivio assente/divergente ⇒ FAIL.
- **A-AH41**: gate-feature-matrix logga `rustc -V`/`cargo -V` in testa a ogni archivio (attribuzione dei moti d'hash classe b55e2f78/15fb6b46).
- **A-AH42** (da Q2): l'apertura A-AH38+dry-run entra per NOME in NEXT_SESSION §aperture.

**Kill-switch**:
- **KS-AH-87-1**: cifra di campagna ENFORCEd contro un archivio matrix diverso da quello pinnato nel `.done` della battery (post-A-AH40) ⇒ campagna VOID.
- **KS-AH-87-2**: mismatch hash-vs-matrix risolto ri-buildando fino al match senza attribuire la causa per NOME (toolchain/env/lock) ⇒ arm VOID.

*Anders Hejlsberg — sedia 4. L'hash morde perché la feature vive nel nome di ogni simbolo; ora la matrix deve dire quale toolchain l'ha firmata, e la battery quale matrix ha benedetto.*

---

# VERBALE — Lars Bak, sedia 5, Concilio WP-87

**VERDETTO: PASS CON EMENDAMENTI** — tutte le cifre ricomputate dai raw combaciano; accetto la refutazione di sede su A-BB41; la scomposizione additiva è MODELLO, non verdetto; il pin peak resta RITIRATO e ordino il protocollo di attribuzione.

**Q1 — VDL28 ricomputato.** cal-h: `thr=0 ord=1 net=7.349.977` (hello). cal-p: `thr=0 ord=1 net=7.803.281` (pad, floor_inc 1.161.438). dl28s: `thr=1 net=7.803.281` (pad) · `thr=0 net=7.349.977` (hello) — **byte-exact confermato dai raw**. Accetto la refutazione di sede: il bracket avvolge solo `lower_source` (vm/mod.rs:16121), il mio canary path-len≥384 avrebbe prodotto Δ nullo BILATERALE — e per la mia stessa regola A-BB41 avrei dichiarato VDL24 VOID su un risultato vero: la forma contenuto di Leijen non è solo operativa, ha evitato un falso VOID. Il byte-exact stavolta è credibile: il contatore conta eventi, non indirizzi, e soprattutto **le due righe portano numeri DIVERSI (Δ=453.304 B) ciascuno uguale alla propria calibrazione** — lo specchio (stessa coppia di snapshot `process-counters`) avrebbe dato numeri UGUALI. Il canary esclude il mirror; NON esclude l'artefatto residuo: il protocollo è sequenziale (una-richiesta-per-worker), quindi le finestre non si sovrappongono mai nel tempo — "per-thread" è provato senza concorrenza. → A-BB47.

**Q2 — Algebra verificata dai raw**: dl28 VOID ha `thr0 ord=2 net=460.146` (pad); 7.803.281−460.146=7.343.135; 7.349.977−7.343.135=6.842; 7.343.135+460.146=7.803.281 ✓. **Status: MODELLO 🔵, punto singolo, mai addendo di budget.** Il confound è NEI raw: retained_walk thr0 a 2 entry = 7.919.802 contro 4.283.308+4.633.332=8.916.640 standalone ⇒ **996.838 B condivisi**: pad-own misurato a ord=2 può sottostimare il pad-own standalone. Contro-prova che ordino: ordine invertito, predizione nominata hello-own(ord2)=6.842 B.

**Q3 — VP ricomputato**: i 9 peak dai log combaciano; r1=232.079.360 B minimo, wall 4,94s; spread r2..r9 = 252.526.592−235.634.688 = **16.891.904 B** ✓; envelope 252.526.592 B = 240,8 MiB ✓ (>229,2 di campagna-84: envelope SALITO, seconda campagna di fila senza attribuzione); media r2..r9 243.431.424, mediana 242.884.608. WP-86 prevedeva r1 **BASSO** (in 84: r1=217,7 MiB minimo, wall 4,29s — "firma di primo-run freddo, bimodalità non varianza"): confermato. Ma l'ipotesi wall→purge spiega SOLO r1: dentro r2..r9 la correlazione muore (r3 wall 3,31s→252,5M max; r4 wall 3,31s→238,2M; r8 wall 3,28s→235,6M min). Protocollo che ordino (A-BB46): due bracci **interleaved ABBA** — A con `MIMALLOC_PURGE_DELAY=0` (purge in-band forzato), B default — R≥8 per braccio, vmmap Physical footprint per-run + somma `mi_bin committed` in-band per-run. Pin rifondabile SOLO se il braccio A collassa lo spread ≥50% oppure un contatore nominato spiega ≥80% del delta per-run.

**Q4** — A-BB43: N/A **legittimo** — nessuna cifra slope citata sul binario nuovo, MEASURE85 dichiara binario-bound 7a610457 + KG-86-1; KB-86-3 non scatta. A-BB44 (terzo punto VW len≥500): **NON eseguito NÉ dichiarato** — chiusura silente, viola "aperture per NOME". Forma ordinata: retro-dichiarazione come apertura nominata in MEASURE85/NEXT_SESSION, esecuzione in S-86.0 con predizione nominata: len=500 ⇒ a_calls=4, a_bytes=2·500+2·501=2.002.

**Emendamenti**
- **A-BB45**: contro-prova ordine invertito della scomposizione (pad ord1 + hello ord2, stesso thread; predizione 6.842 B); fino ad allora la scomposizione resta modello.
- **A-BB46**: protocollo attribuzione spread VP come sopra (ABBA purge, soglie 50%/80%).
- **A-BB47**: canary sotto overlap wall-clock reale dei due lowering prima della delibera registry — il sequenziale non prova la concorrenza.
- **A-BB48**: A-BB44 retro-dichiarata + eseguita con la predizione 2.002 B.

**Kill-switch**
- **KB-87-1**: scomposizione additiva usata come addendo di budget prima di A-BB45 ⇒ cifra VOID.
- **KB-87-2**: pin peak fondato con spread non attribuito (nessun braccio discriminante superato) ⇒ pin VOID; unica cifra legale l'envelope, con trend cross-campagna dichiarato (229,2→240,8 MiB).
- **KB-87-3**: registry deliberata sul solo canary sequenziale senza A-BB47 ⇒ il claim "per-thread sotto concorrenza" resta CANDIDATO.

— Bak, sedia 5

---

# VERBALE — Concilio WP-87, sedia 6 (Pedersen). Mandato: REFUTARE.

**VERDETTO: CON EMENDAMENTI** — A-PP30/31/32 eseguite nella lettera; Q1 chiusa ma il pre-warm ha CREATO una cecità nuova; Q2 forma debole, ordino la forte; Q3 un caso valido-ma-shiftato esiste; Q4 declassamento sollevato solo in parte.

**Q1 — chiusa la finestra, aperta un'altra.** Verificato: pre-warm con include REALE (`worker_pool.rs:1071-1087`), pin `t_warm>e0` (`:1098-1102`), delta==0 su ENTRAMBE le fatali contro lo snapshot warm (`:1121-1127`), ledger fp-replace (`vm/mod.rs:16197-16202`). Due cecità OLTRE la nominata: **(a)** il pre-warm CONSUMA lo status di prima-richiesta del thread — un inserter once-per-thread che pubblica il main della prima richiesta ora spara sul warm, e la configurazione «fatale = vera req1» è USCITA dalla copertura. La scoperta 4 (fixture banale non semina) rende l'arm no-warm banale: baseline cache-vuota, delta==0 anche su req1 (il fatale non include nulla ⇒ seeding legittimo = 0). **(b)** insert+evict COMPENSATO: `uc_entry_count` somma `slot.len()` — un inserter che pubblica il fatale evictando l'entry warm lascia il conteggio invariato (+1−1); i conti vedono CARDINALITÀ, non appartenenza, e `main_evicted` esiste come evento ma a_pp20 non lo pinna.

**Q2 — NON basta; forma forte ordinata.** `gate-lever-pins.sh:742` usa `grep -q 'w=1'`: un futuro script con `w=10` nel testo (o un commento `# TODO w=1`) SODDISFA lo sweep senza alcun rifiuto. Il decoy `:752-756` è MONOLATERALE: prova solo l'assenza del token, mai il falso-positivo. (Il guard runtime `verdict85.sh:164` con `/ w=1 /` spazi-delimitato è invece sano.) Ordino: subito digit-guard (`w=1[^0-9]`) + decoy `w=10` nel self-test; poi la via A del mio A-PP31 originale — parser `reqns:` CONDIVISO tracked con bite-test proprio (raw sintetico w=2 ⇒ VOID), sweep che esige il sourcing per NOME. La presenza di un token non è un rifiuto.

**Q3 — il caso non rilevato: phantom pre-publish.** Il verdict è robusto sui casi esecutivi: phantom che ESEGUE hello su thr0 ⇒ duplicate ord=1 (`verdict85.sh:59`) o pad ord=2 mancante (`:98-99`) ⇒ FAIL. Ma un connect accettato-poi-timeout (`dl28-supplement.sh:46-51`) ABORTITO PRIMA del publish (client chiude in fase header) consuma il round-robin senza emettere righe: hello→thr1, pad→thr0, righe VALIDE, byte-exact, thread distinti ⇒ **PASS**. VDL28 sopravvive (attribuzione per PATH), ma il claim di protocollo «il primo SUCCESS È request 1» (`:7-8,47-48`) è FALSO in quel run: request 1 era il phantom. Ogni lettura futura degli ordinali di dispatch da quel raw è VOID (spirito KS-PP-86-3). Fix: contatore dispatch PER-THREAD in-band a teardown; il verdict esige somma==2 e ==1 per thr — l'advisory A-PP32 diventa fatto contato, e morde phantom eseguiti E abortiti.

**Q4 — KS-PP-86-2: sollevato PARZIALMENTE.** Il claim copre ora steady + req1-con-warm (pin su entrambe le fatali, `:1119-1127`). NON copre la vera prima richiesta del thread (Q1a) né il replace fp-in-place (dichiarato) né l'evict compensato (Q1b). Status: declassamento ristretto, non rimosso.

**Emendamenti:**
- **A-PP33**: arm no-warm in a_pp20 — fatale come VERA req1 del thread, baseline vuota, delta==0 su req1 (abilitato dalla scoperta 4).
- **A-PP34**: pin evict==0 (contatori/eventi `main_evicted`) attraverso le fatali, o snapshot del SET (key,fp) — membership, non cardinalità.
- **A-PP35**: A-PP31 forma forte — digit-guard + decoy w=10 subito; parser condiviso tracked con bite-test come chiusura.
- **A-PP36**: dispatch-count per-thread in-band; verdict dl28-class fail-closed su somma≠attesa.

**Kill-switch:**
- **KS-PP-87-1**: sweep soddisfatto da token-presenza (`w=1` come sottostringa) ⇒ sweep VACUO, eredità non provata.
- **KS-PP-87-2**: claim «mai retained include-form» esteso oltre {steady, warmed-req1} senza A-PP33 ⇒ declassato (sostituisce KS-PP-86-2).
- **KS-PP-87-3**: cifra net certificata da run senza dispatch-count in-band==atteso ⇒ riga = envelope, mai attribuzione (phantom non escludibile).

— Pedersen, sedia 6.

---

# VERBALE WP-87 — Sedia 7, Daan Leijen (mimalloc/footprint/contabilità)

**VERDETTO: CONCORDO CON EMENDAMENTI. A-DL28 eseguito nella MIA forma e il canary DISCRIMINA; ma il PER-THREAD resta una proprietà del PROTOCOLLO (sequenziale, una-richiesta-per-worker), non della finestra. E il phys_peak<phys l'ho trovato: è un artefatto di throttle, con la riga di codice colpevole.**

**Q1 — Sì, è il mio discriminatore, con perimetro nominato.** Ricomputo: ΔNET = 7.803.281−7.349.977 = 453.304 B = Δown (460.146−6.842) ESATTO. Il canary falsifica lo specchio (righe diverse ⇒ non stessa coppia di snapshot) e la finestra congelata. MA `net_window=process-counters` (GA_ALLOC/GA_FREE atomics di processo, memcensus.rs:957-958): con richieste SEQUENZIALI l'attribuzione corretta è attesa ANCHE da un contatore di processo — il disegno non esclude (a) drop differiti cross-richiesta che DEFLAZIONEREBBERO il bracket successivo, (b) init lazy dentro il bracket. Il byte-exact li esclude EMPIRICAMENTE (pollution netta ==0 in QUESTI run, su QUESTO workload senza teardown differiti), non STRUTTURALMENTE. Verdict-grade per questo protocollo: sì. Trasferibile a dispatch concorrente o fixture con distruttori: no → KL-87-2.

**Q2 — NON basta.** A-DL29 prova che il CONTATORE conta i thread estranei (held attraverso finish(), vm/mod.rs:17681-17693 ✓). Ma: (i) manca il controllo di DEFLAZIONE (free-in-window di un blocco pre-window: il net deve scendere o dichiararsi clampato — oggi comportamento non specificato); (ii) nessun dente di VERDICT rifiuta una riga polluta in produzione — il burst nel selftest prova il meccanismo, non il fail-closed della pipeline → A-DL33/A-DL34.

**Q3 — Modello PLAUSIBILE, costanti fixture-shaped.** hello-own 6.842/221 = 31,0× è dominato dall'overhead fisso (Program+Module+Rc+header stringhe 16 B). Il marginale pad: 453.304/9.055 = 50,06 B/byte-sorgente — plausibile SOLO perché hello_pad85 è 60 funzioni (statement-dense, verificato nel fixture); un literal-pad darebbe ~2-4×. I 31×/50× NON sono costanti generali: A-DL31 registri la composizione (n. funzioni/stmt) accanto all'own. Residuo 7.343.135: walk-visibile 4.283.308 + walk-invisibile 3.066.669 (pad: 3.169.949 — interner/side-table, il mio addendo WP-86). Contro il fisico 3.605.572 B/worker: gap contato−fisico = 3.737.563 B ⇒ slack-condiviso (segmenti già committed: bin 32 mostra reserved 327.520/used 672) + contato-mai-toccato. **A-DL31 aggiornata**: W=1/2/3 hello, chiusura `7.343.135+own = Δcommitted(mi_bin thr=) + slack + untouched` con predizione fisica 3.605.572±5% o FAIL.

**Q4 — TROVATO il meccanismo di A-DL30, ricomputato dal codice.** `PHYS_PEAK.fetch_max` vive SOLO in `phys_check()` (memcensus.rs:812, throttle ogni 16384 eventi); `phys_window_dump` (:843-848) stampa `phys` FRESCO ma `PHYS_PEAK` STANTIO ⇒ phys_peak<phys è artefatto di campionamento (dl28s: 38.732.304<74.711.544; cal-h: 38.486.544<46.203.384). Fix da una riga: fetch_max(phys) in testa al dump. Per lo spread 16.891.904 B: i 9 picchi ordinati NON quantizzano su passo d'arena (diff 49.152…6.651.904 B) ⇒ il candidato forte è il purge timing, non la quantizzazione. Protocollo: (1) A-DL32 (fix peak, poi riammissione righe mi_proc); (2) coppia stessa-sera R=9 con `MIMALLOC_PURGE_DELAY=0` vs default, env IN-BAND nel raw: spread(0)<0,5×spread(default) ⇒ attribuito al purge; (3) join peak_rss mi per-finestra ↔ time -l su mono_ms per datare DOVE cade il picco; (4) solo dopo, mi_bin thr= per first-touch.

**Emendamenti**
- **A-DL32**: `phys_window_dump` esegue `PHYS_PEAK.fetch_max(phys)` prima di stampare; righe mi_proc riammesse solo dopo.
- **A-DL33**: burst 1 MiB DENTRO un bracket di produzione in una run di campagna dedicata: la riga deve muoversi ≥1 MiB e il verdict deve DECLASSARLA — prova del fail-closed, non del contatore.
- **A-DL34**: controllo di deflazione (free-in-window); semantica underflow di finish() dichiarata.
- **A-DL35**: A-DL31 con composizione fixture in-band e chiusura ±5% come sopra.

**Kill-switch**
- **KL-87-1**: pin peak deliberato prima della coppia purge-arm R=9 ⇒ delibera NULLA (estende KL-86-1).
- **KL-87-2**: budget ×W citato per dispatch concorrente, W>2, o fixture con distruttori cross-richiesta senza re-canary ⇒ riclassificato IPOTESI.
- **KL-87-3**: riga phys_peak citata pre-A-DL32 ⇒ esclusa dal corpus (estende KL-86-3).

*Daan Leijen — l'algebra chiude al byte; il peak mentiva per throttle, non per malizia: una riga di fetch_max e torna onesto.*

---

# VERBALE STOGOV — sedia 8, Concilio WP-87 (Zend/opcache, cache di unit)

**VERDETTO: PASS CON EMENDAMENTI** — la lettera dei miei ordini WP-86 è onorata (A-DS30/31/33 verificati nel codice); refuto due claim collaterali e ho una scoperta empirica nuova, verificata sull'oracle vivo.

**Q1 — Esaustività: REGGE nel dominio del test, il commento SBORDA su un campo.** Destructuring verificato campo-per-campo contro bytecode.rs:1765-1890 (CompiledClass) e :1957-2030 (Module) vs vm/mod.rs:19274-19341: nessun campo manca, niente `..`, rustc giudica (KS-DS-86-2 rispettata: esclusioni NOMINATE). Derived genuini: `fn_ci`/`class_index` da nomi coperti; `class_name` da `name`; i 4 booleani da `prop_info`; `info` (ObjectInfo, object.rs:569-586: entries+types) da visibilità/type-display coperti; `props_layout` ha un TESTIMONE coperto — `prop_info.slot` — quindi una permutazione non può tacere. **Ma `methods_ci` (bytecode.rs:1803-1810) può divergere da solo su unità ELISA** (`elided: Some(n)`, :2025 sgg.): la parent-chain baked leaf-wins pesca i metodi del seed dall'immagine accumulata del VM, che NON sta in alcun campo coperto — due Module a sig identica possono portare methods_ci diversi. Nel dominio di a_ds17 (compile_program fresco, Registry fresca, no elisione) il claim regge; come invariante generale no. Non cade l'esaustività: cade la portata del commento → A-DS34.

**Q2 — Hoisting: fedele a Zend+OPCACHE, divergente dall'oracle CLI — catalogare.** Provato empiricamente su php 8.5.7: senza opcache `class_exists('C')` prima dello statement = **false**, `return;` prima della decl = figlia MAI dichiarata, fatal LSP DOPO l'output; con `opcache.enable_cli=1` = **true/dichiarata/fatal prima dell'output**. phpr riproduce esattamente il braccio opcache (true/true su t1/t3). Coerente col modello dichiarato (unit cache = persistent script), ma il gate di parità CLI usa brew php con opcache_cli OFF: divergenza osservabile da catalogare (il catalogo ha §3.3/r.516 ma non questo caso). **Scoperta collaterale GRAVE**: la variance LSP non è verificata AFFATTO — `C::m(): int` vs `P::m(): string` in phpr compila e gira, exit 0, ANCHE parent-first (t5); l'oracle fatala "must be compatible". Zero hit per "covarian" nel catalogo. Correct-or-absent violato: classe SBAGLIATA invece che assente → A-DS35.

**Q3 — L'ordine documentato ha una metà infalsificabile.** Il pin (vm/mod.rs:19190-19204) è sano nel SUO buffer controllato. Ma il doc (:15727-15733) dichiara `supersede* → main_evicted → evict fp`, e: (a) supersede ed evizione sono mutuamente esclusivi PER COSTRUZIONE (:16407-16418: chiave nuova ⇒ slot fresco ⇒ mai overflow) — il prefisso `supersede*→` non è pinnabile da nessun test, oggi; (b) un main dentro uno slot superseded muore SENZA `main_evicted` (:16448-16466 conta solo stranded) — il doc lascia credere il contrario; (c) le righe non portano put-ordinal: fuori da F16 (che CONTA, non ordina) un riordino cross-put da un futuro flush batched passerebbe tutto — a_ds26 vede solo la prima coppia del proprio buffer. → A-DS36.

**Q4 — Rinvio accettato SOLO nella lettera.** MEASURE85_RESULTS.md:24-25: nessuna finestra census ⇒ KS-DS-86-3 non innescata, N/A legittima. Ma la classe KG-86-1 dice: un gate nasce fail-closed o è illegittimo. **Ordino il template GIÀ scritto ora** → A-DS37.

**Emendamenti**: **A-DS34** — nota di dominio su methods_ci (elided ⇒ deriva dall'immagine, fuori sig) + mutante recompute-compare su unità elisa. **A-DS35** — catalogare hoisting-observables (class_exists pre-decl, return-before-decl, timing fatal) come semantica-opcache, e la covariance LSP mancante come gap engine con decisione correct-or-absent. **A-DS36** — put-ordinal in-band nelle righe evict/main_evicted + pin di ADIACENZA per-put; doc emendato: supersede non co-occorre e un main superseded non emette main_evicted. **A-DS37** — template VA census con vitalità in-band scritto prima di qualunque finestra.

**Kill-switch**: **KS-DS-87-1** — claim "sig esaustiva" citato per unità elisa senza la nota A-DS34 ⇒ claim declassato in-domain-only. **KS-DS-87-2** — finestra census lanciata da template privo di vitalità in-band ⇒ cifre VOID, non advisory. **KS-DS-87-3** — modifica al put-path che renda supersede/evizione co-occorrenti senza nuovo pin d'ordine nella stessa battery ⇒ ordine ADVISORY, campagna non verdict-grade.

*Dmitry Stogov — l'engine che tace su una firma incompatibile non è fedele: è gentile. Zend non è gentile.*

---

# VERBALE — B. Gregg, sedia 9, Concilio WP-87 (raw riconciliati a ricomputo indipendente)

**VERDETTO: PASS CON EMENDAMENTI.** Le cifre REGGONO tutte al byte; trovo DUE violazioni di forma (una è di un mio kill-switch) e un overclaim documentale. Nessuna refutazione di sostanza.

**Riconciliazione**: 9 peaks dai summary = esattamente i 9 del verdict; min=232.079.360 (r1), max=252.526.592 (r3), spread r2..r9 = 252.526.592−235.634.688 = 16.891.904 ✓. Wall r1=4,94s (real, time -l; elapsed in-band 4,389s), r2..r9 3,28–3,40 ✓. cal-h net=7.349.977 · cal-p net=7.803.281 · dl28s: thr0 hello=7.349.977 E thr1 pad=7.803.281, al byte ✓. dl28 (VOID): entrambi i thread hello ord1, pad come ord2 thr0 net=460.146; algebra: 7.803.281−460.146=7.343.135; 7.349.977−7.343.135=6.842; somma=NET_P esatto ✓.

**Q1 — Pulita CON condizioni, una manca.** Il supplement rispetta: stesso rev (git=368c91d in-band), stesso binario (mem_hash==CAL enforce, a3c901df…), delta di protocollo NOMINATO nell'header (doppio dispatch hello → up-probe body = request 1), raw VOID tenuto in place, e il nuovo script POTEVA fallire (oracle + fail-close both-on-one-thread). Il filesystem mostra UN solo tentativo (un solo m85.dl28s, un solo m85s.build log) — ma niente lo VINCOLA. La porta "si riscrive finché passa" si chiude solo col ledger: **A-BG39** — ogni script sostitutivo dichiara `attempt=N` in-band ed elenca i raw dei tentativi precedenti (tutti tenuti); il raw VOID resta SEMPRE (confermo KS-AH-83-2).

**Q2 — Legale come MODELLO, e l'etichetta c'è** ("🔵… dal raw VOID, ord=2" — onesta). Ma la disciplina va formalizzata: **A-BG42** — classe in-band per ogni cifra del measure-doc: `VERDICT-GRADE` / `MODEL-GRADE(src=<raw>)`; **KG-87-2**: cifra da raw VOID senza tag MODEL-GRADE + raw nominato ⇒ citazione VOID; verdict script che consuma righe non-VERDICT-GRADE ⇒ VERDICT NULL.

**Q3 — A-BG38: COMPLETO** (riga VP: arm=union, W=10, R=9, fixture=hello.php, driver_sha ✓). **A-BG37: campaign SÌ, supplement NO.** Nel campaign `head_unmoved` è prima di OGNI invocazione (riga 105 dentro memrun → cal-h/cal-p/dl28; riga 151 → 9×measure78). Nel supplement: **zero occorrenze** — GIT_REV letto una volta (riga 23), poi una BUILD intera prima del launch senza ricontrollo. MEASURE85 riga 10-11 dichiara "A-BG37 anche in supplement": **overclaim, da correggere**. → **A-BG40**: head_unmoved reale nel supplement (post-build, pre-launch). **KG-86-2: VIOLATA in forma** — "pin 232±1 MOSSO" e "envelope SALITO (vs 240.287.744)" nominano solo 54717a9a; il lato-84 (fe6983d8fee7d0c2) NON è in chiaro da nessuna parte. La SANO qui a ricomputo: driver_sha = sha(measure78.sh + gate-feature-matrix.sh + wrapper); measure78.sh intoccato da 7b90614 (pre-84) e gate-feature-matrix da 7fed57d ⇒ **il delta 84→85 è SOLO il wrapper** (measure84- vs measure85-campaign.sh), che ordina i run e non tocca la finestra misurata: inerzia FAVOREVOLE, "measure78 invariato" vero ma incompleto. Envelope-84 riconciliato dai raw: 228.278.272/239.878.144/240.287.744 ✓.

**Q4 — Delibera LEGALE nella parte eseguita.** ×W: il Concilio WP-86 aveva pre-autorizzato con precondizione nominata (canary prima della delibera); la precondizione è soddisfatta byte-exact ⇒ accettazione verdict-grade legittima in-sessione. Pin: NON deliberato ma RITIRATO — conservativo, KL-86-1 rispettata. "Envelope SALITO" invece: (a) VOID-in-forma per KG-86-2 (sanata sopra); (b) vizio d'ordine statistico — max su R=9 domina stocasticamente max su R=3: la "salita" +12,2MB è DENTRO lo spread non attribuito (16,89MB) e non è un regress dimostrato (anche il min sale solo +3,8MB). → **A-BG41**: ogni confronto cross-campagna nomina driver_sha di TUTTI i lati citati (incluso il lato-origine del pin S-82.0) E l'R di entrambi; **KG-87-3**: confronto envelope/min a R diversi senza R dichiarato per lato ⇒ VOID. **KG-87-1**: supplement senza attempt-ledger ⇒ raw sostitutivo VOID.

*— B. Gregg, sedia 9*

---

