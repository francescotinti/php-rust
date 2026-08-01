# COUNCIL_WP86_REVIEWS.md — Concilio a 9 sedie su S-84.0 (sigilli v2 + oracoli + misure decisive peak) + programma WP-85(sessione)

**Convocato**: 2026-08-01, chiusura S-84.0 (regola utente 2026-07-26: 9 sedie
dopo OGNI sessione, mandato REFUTARE). Atti: NEXT_SESSION_WORDPRESS.md,
sessions/WP_SESSION_84.md, wp84-harness/MEASURE84_RESULTS.md + verdict84.out
+ verdict84.sh, wp85-harness/COUNCIL_WP85_REVIEWS.md (gli ordini eseguiti),
codice e raw dei rispettivi perimetri. Verbali VINCOLANTI per il design
WP-85(sessione).

## ⚖️ SINTESI DI CONVERGENZA

**Verdetto complessivo: 9× CONCORDO/CONFERMATO CON EMENDAMENTI — nessuna
opposizione.** Le CIFRE sono confermate a ricomputo indipendente (Gregg:
VP/VDL24/VW2/ledger al byte; Bak: raw integrali, VW2 39/39 righe per cella;
Leijen: algebra probe + primo addendo del gap TROVATO NEL RAW). Gli ordini
WP-85 sono giudicati eseguiti NELLA LETTERA da tutte le sedie proprietarie.
I colpi della tornata:

### Refutazioni convergenti (≥2 sedie indipendenti)

1. **KH85-1 è chiuso a metà** (Hoare): il cfg-gate vale per `cargo build`,
   ma `cargo test --release` unifica la feature del dev-dep sull'unit-graph
   — un binario prelevato dal target DOPO un test-run CONTIENE il probe; e
   dentro vm/mod.rs (~16.400 righe) chiunque minta `VmGate(PhantomData)`
   con un `use` che elude il belt di grafia. Il giudice sul mint in-module
   è ancora l'awk → [A-TH32 micro-modulo `gate` foglia, KH86-1: il dente
   verifica il BINARIO (`nm`/strings), non il sorgente].
2. **Il certificato di partizione è una SPELLING** (Hoare Q2 + Matsakis
   Q3): il pin 1c non vede field-shorthand né `.main_program = Some(…)`;
   la vera chiusura è A-MS27 (campo privato + tipi CachedMain/
   CachedInclude), che diventa **PRECONDIZIONE della via registry**
   [A-TH33, KS-MS-86-2 ≡ KH86-2 convergenti].
3. **BUCO GRAVE di copertura** (Klabnik Q4): battery-84pre invoca
   gate-measure-cifre SENZA argomento ⇒ target = MEASURE81 — **MEASURE84
   non è MAI stato nel perimetro del 15/15**; il dente A-DL26 ha morso
   solo a invocazione manuale. Più: stamp `git=$BREV` non ancorato,
   `.done` forgiabile e scritto ANCHE a FAIL, corner index nel check
   ledger [A-SK36/37/40, KS-SK-86-1..3].
4. **VDL24 PER-THREAD = VALIDA-CONDIZIONATA** (Leijen + Bak, forma
   CONVERGENTE): l'identità al byte (7.349.977 B su thr0 E thr1) è
   compatibile sia col determinismo pieno sia con una finestra-specchio;
   due input IDENTICI non discriminano. Ordinata la **perturbazione
   MONOLATERALE calibrata** (fixture diversa per worker, Δ predetto
   ESATTAMENTE dal piecewise VW2) [A-DL28 ≡ A-BB41; KL-86-2/KB-86-2:
   senza canary il PER-THREAD resta CANDIDATO/IPOTESI e la via registry
   è BLOCCATA].
5. **I tre picchi VP non sono deliberabili così** (Leijen KL-86-1 + Bak
   A-BB40 + Gregg A-BG36): spread non attribuito (r1 217,7 MiB con wall
   4,29s vs 3,32/3,32 — firma di primo-run freddo, bimodalità non
   varianza), R=3 legittimo per dichiarare il pin MOSSO ma NON per
   fondarne uno nuovo (serve R≥9, primo run nominato/escluso), driver_sha
   diverso da 82p (attribuzione alla partizione resta CANDIDATA), e
   `mi_proc phys_peak<phys` incoerente nel raw [A-DL30]. La delibera può
   usare SOLO l'envelope conservativo max = 240.287.744 B = 229,2 MiB.
6. **La sig() resta scoperta STRUTTURALMENTE** (Stogov Q2): m.deferred
   (parent late-bound — identica classe di cecità-di-sede delle
   closures), `Module.strict`, enum_cases, final/abstract, own_prop_vis,
   uses_traits, corpi hook in prop_info — campi che ESISTONO già, quindi
   KS-DS-85-3 (campo "nuovo") non li copre. Forma ordinata: destructuring
   ESAUSTIVO senza `..` — rustc rompe la build a ogni campo nuovo
   [A-DS30 — convergenza col principio ⭐⭐ "sigillo di tipo > allowlist"].
7. **I rifiuti non si ereditano da soli** (Pedersen Q2 + Gregg Q4): il
   rifiuto w≠1 e A-BG33/A-BG35 vivono in verdict84/in una nota — il
   prossimo verdict slope è un file che non esiste e nessuna macchina lo
   obbliga a nascere vincolato [A-PP31, KG-86-1 fail-closed, KS-PP-86-1].
8. **A-AH35 possiede i blocchi, non il FILE** (Hejlsberg): cancellazioni
   fuori da [[package]] sono fail-open; [[patch.unused]]/[metadata] si
   fondono nel buffer dell'ultimo package; e la macchina rafforzata non
   ha MAI avuto un positive-control live [A-AH38, KS-AH-86-1: dry-run
   obbligatorio alla prossima campagna base-arm].
9. **Onestà di forma**: probe `VmGate<'static>` nei test — fix gratis
   (A-MS29); rinomina feature da dichiarare (A-MS30); panic del probe
   DL24 da attribuire allo strumento (A-MS31); cleanup di a_ds26 droppa
   nel borrow — il test che celebra A-MS28 ne viola la lettera (A-DS33);
   l'ordine eventi uc_log è API e va pinnato (A-DS31); controllo hello
   di VW2 senza check steady_n (A-SK38, KS-SK-86-4); "3,44 MB/worker"
   importato senza byte esatti — KL-85-2 morde su cifra di S-82.0
   (sanatoria); commento DL24 "failed connects never dispatch"
   sovra-dichiara (A-PP32); head_unmoved non per-run in Phase W2
   (A-BG37); vitalità census in-band per le finestre contate (A-DS32);
   slope ~55–75× è binario-bound a 7a610457 — smoke-slope a ogni
   trasferimento (A-BB43, KB-86-3); terzo punto VW a len≥500 (A-BB44);
   pin F16 `1 passed` + rustc in-band (A-SK39); manifest trasclusi
   dichiara l'oggetto in-cargo di F16 (A-AH39); a_pp20 pre-warm per la
   finestra once-only-req1 (A-PP30, KS-PP-86-2).

### Ordine vincolante di apertura WP-85(sessione) (non rinegoziare)

1. **Sanatorie immediate**: cifra 3,44 MB/worker con byte esatti dal
   record S-82.0 (KL-85-2) · A-PP32 (commento DL24) · A-MS30 (nota
   rinomina feature) · A-BG37 (head_unmoved per-run) · A-DS33 (cleanup
   a_ds26 fuori borrow) · A-MS31 (panic-attribuzione probe).
2. **Battery/equivalenza onesta v3**: A-SK36 (stamp ancorato + .done
   solo-PASS con sha256(OUT)) · A-SK37 (check index + append committato
   con commit-id) · A-SK40 (cifre-gate su TUTTI i MEASURE8[4-9] in
   battery — sana il buco GRAVE) · A-SK38/39 · A-AH39.
3. **Sigilli v3**: A-TH32 (micro-modulo `gate`) · A-TH33 (grafie eluse)
   · A-MS29 (probe lifetime-bound) · KH86-1 cablato (dente `nm` sul
   binario di campagna) · A-TH34 (split per-grafia) · A-TH35/KH86-3
   (guardia di rientranza).
4. **sig() esaustiva**: A-DS30 (destructuring senza `..`, ≥3 mutanti
   nuovi: deferred-body, enum case, final-flag) · A-DS31 (ordine eventi).
5. **Discriminazione VDL24 + VP deliberabile**: A-DL28≡A-BB41 (canary
   monolaterale, Δ predetto dal piecewise) · A-DL29 (controllo positivo
   pollution) · VP R≥9 min-of-R con primo run nominato (A-BB40) + purge
   in-band + A-DL30 (phys_peak spiegato) · A-BB42 (bande ancorate al
   residuo) · POI la delibera peak (fino ad allora: envelope max
   229,2 MiB, budget ×W = IPOTESI CANDIDATA).
6. **Eredità meccanica**: A-PP31/KS-PP-86-1 + KG-86-1 (il verdict slope
   futuro nasce fail-closed o è illegittimo) · A-PP30 · A-DS32.
7. **A-DL27 eseguibile** (A-DL31): W=1/2/3, net per-thread + mi_bin
   thr= + vmmap purge=0, chiusura additiva ±5% — la domanda "di chi è
   il residuo" diventa "dove dorme il residuo".
8. Poi: **ripresa ROADMAP da [[php-rust-todo-master]]**. Revert policy
   KS-DS-80-3 invariata.

### Kill-switch nuovi consolidati (attivi da subito)

| KS | Condizione | Effetto |
|---|---|---|
| KH86-1 | cifra di campagna da binario con `vm_gate_probe` nei simboli | campagna VOID |
| KH86-2 | "partizione per costruzione" pre-A-MS27/A-TH33 | declassata a disciplina+tripwire |
| KH86-3 | Drop nuovo che tocca UNIT_CACHE senza guardia A-TH35 | KS-MS-85-4 riaperta |
| KS-MS-86-1 | righe unitcache_main_entry da run con exit≠0 | righe VOID |
| KS-MS-86-2 | via registry aperta col routing a tag ancora in sede | delibera VOID finché A-MS27 |
| KS-MS-86-3 | VmGate<'static> fuori cfg-test dopo A-MS29 | sigillo ADVISORY |
| KS-SK-86-1 | equivalenza su OUT senza riga PASS ancorata o .done senza sha | equivalenza VOID |
| KS-SK-86-2 | append ledger non a HEAD a chiusura campagna | equivalenze campagna VOID |
| KS-SK-86-3 | battery citata come copertura MEASURE84+ col cifre-gate sul default 81 | bytes-first ADVISORY, doc non pubblicabile |
| KS-SK-86-4 | NAMED-DEVIATION da controllo senza check validità | declassata a FAIL |
| KS-AH-86-1 | cifra base-arm verdict-grade senza un passaggio live PASS della v2 | arm VOID |
| KS-AH-86-2 | lock-diff con riga cancellata non attribuita per NOME (post-A-AH38) | arm VOID |
| KB-86-1 | pin peak dichiarato con R<9 o primo run non nominato | pin VOID |
| KB-86-2 | registry deliberata senza canary A-BB41 superato | delibera BLOCCATA |
| KB-86-3 | cifra slope citata su binario diverso senza smoke-slope | claim VOID |
| KS-PP-86-1 | verdict che parsa reqns: senza rifiuto w≠1 nel proprio corpo | verdict illegittimo, campioni VOID |
| KS-PP-86-2 | claim "mai retained include-form" pre-A-PP30 | vale solo steady-state |
| KS-PP-86-3 | attribuzione per-worker DL24 dall'ordine richieste anziché thr= | attribuzione VOID |
| KL-86-1 | delibera peak su VP con spread non attribuito | delibera NULLA |
| KL-86-2 | budget ×W promosso senza A-DL28 eseguito | riclassificato IPOTESI |
| KL-86-3 | cifra mi phys_peak citata mentre incoerente | riga esclusa dal corpus |
| KS-DS-86-1 | pin main_evicted==0 dopo modifica al put-path senza F16 nella stessa battery | pin ADVISORY |
| KS-DS-86-2 | claim purezza a_ds17 con campo fuori sig senza esclusione nominata | claim NULLO |
| KS-DS-86-3 | cifra per-fixture census senza vitalità census in-band | cifra ADVISORY |
| KG-86-1 | verdict slope senza per-request base E regime= definito | VERDICT NULL d'ufficio |
| KG-86-2 | confronto con pin cross-campagna senza driver_sha di entrambi i lati | confronto VOID |

---

## VERBALI INTEGRALI

**VERBALE — Tony Hoare, sedia 1, Concilio WP-86 (su S-84.0)**

**VERDETTO: CONCORDO CON EMENDAMENTI.** I miei ordini sono eseguiti nella lettera: A-TH27 (`struct VmGate<'gate>(PhantomData)`, vm/mod.rs:502; `production_gate` privato, :553), A-TH28 (cfg su fn :512 e re-export lib.rs:72-73; belt anti-alias pins.sh:139-151), A-TH29 (split 5/9, pins.sh:354-373), A-TH31/KH85-3 (sezione 1c, pins.sh:166-208). A-TH30: dichiarato eseguito (WP_SESSION_84 p4), non ispezionato in situ — fuori dagli atti consegnatimi; resta da verificare a WP-86. Ma il mandato è refutare, e trovo quattro buchi.

**Q1 — KH85-1: chiuso a metà.** Il resolver="2" (Cargo.toml:2) impedisce l'unificazione dev-dep in `cargo build`. Ma: (a) durante `cargo test --release` — che la nostra regola impone SEMPRE — la feature del dev-dep (php-server/Cargo.toml:25) unifica sull'intero unit-graph dell'invocazione, e i bin ricompilati in quella invocazione CONTENGONO il probe; un binario prelevato dal target dopo un test-run è un binario di campagna col simbolo dentro. (b) `--all-features` lo accende sempre. La garanzia oggi è di PROTOCOLLO (quale verbo ha prodotto il binario), non di rustc. (c) In-module: il campo di VmGate è privato al modulo `vm` — dentro vm/mod.rs (~16.400+ righe) e nei figli di `vm` CHIUNQUE può scrivere `VmGate(PhantomData)`; il belt pinna solo la grafia `VmGate(std::marker::PhantomData)` ==3 (pins.sh:127) — con un semplice `use std::marker::PhantomData` il mint diventa invisibile al belt e legale per rustc. La superficie di mint senza giudice è l'intero modulo.

**Q2 — Pin 1c: regge sui casi chiesti, cade su una grafia.** Il sito spostato in una closure resta nel body (l'awk termina solo su `}` a colonna 0, pins.sh:182-185); l'estrazione in helper fa FALLIRE FORTE (count 0≠1) — bene. Ma entrambi i denti pinnano la grafia `main_program: Some(`: un secondo produttore via field-shorthand (`let main_program = Some(p); CachedUnit { main_program, .. }`) o via assegnazione post-hoc (`cu.main_program = Some(p)` — `=`, non `:`) passa il pin ==1 (pins.sh:192) E lo sweep (:200). Il certificato di partizione è su una SPELLING, non sul tipo.

**Q3 — A-TH29: chiude tra-regione, non dentro-regione.** Il 5/9 uccide la compensazione prod↔selftest di KH84-1. Ma CLASSRE è UN conteggio aggregato per regione: in prod, `lower_source` è coperto a parte dal 2/7 (A-TH23), però seeded(2)+compile(1)=3 sono fusi — sostituire un seeded con un compile_program lascia 5 invariato. Idem in selftest (7ls+2compile=9). Residuo: swap compensativi TRA GRAFIE dentro la stessa regione.

**Q4 — Collect-then-emit: un rischio nuovo, reale.** L'ordine di drop è sano (census bytes calcolati PRIMA del drop, :16374-16376; `superseded` tiene vivi gli slot solo per la finestra di emissione — costo transiente trascurabile). Il rischio è di falsificabilità: prima, un Drop estraneo che rientrasse nella cache produceva PANIC (double-borrow, rumoroso); ora un futuro Drop che chiamasse unit_cache_put/get è LEGALE e silenzioso, interleavando mutazioni tra la fase di mutazione e le statistiche emesse da stato pre-calcolato. KS-MS-85-4 è chiusa scambiando un fallimento rumoroso con uno muto.

**Emendamenti**
- **A-TH32**: sigillo di tipo sul mint (lezione ⭐⭐ WP-83): VmGate+production_gate in un micro-modulo foglia `mod gate` (~30 righe) a campo privato — rustc giudica ANCHE dentro vm/mod.rs; nel frattempo pin VMMOD `VmGate[(]` (classe, ogni grafia) ==3.
- **A-TH33**: pin 1c esteso alle grafie eluse: workspace-sweep `\.main_program = Some(` ==0 e shorthand `main_program,` in costruzione ==0; vera chiusura = A-MS27 (campo privato + costruttore tipato).
- **A-TH34**: split per-GRAFIA dentro le regioni: prod 2/2/1, selftest 7/0/2 — tre contatori, mai un aggregato.
- **A-TH35**: guardia di rientranza in unit_cache_put (thread-local armata in emissione; rientro = panic).

**Kill-switch**
- **KH86-1**: cifra di campagna da binario in cui `nm`/strings trova `vm_gate_probe` ⇒ campagna VOID — il dente verifica il BINARIO, non il sorgente.
- **KH86-2**: "partizione certificata per costruzione" citata prima che A-MS27/A-TH33 atterrino ⇒ declassata a "per disciplina+tripwire".
- **KH86-3**: nuovo Drop che tocca UNIT_CACHE senza A-TH35 armata ⇒ KS-MS-85-4 riaperta d'ufficio.

---

**VERBALE — Matsakis, sedia 2, Concilio WP-86 su S-84.0. Perimetro: ownership/borrow/sigilli di tipo.**

**VERDETTO: CONCORDO CON EMENDAMENTI** — A-MS25/A-MS26/A-MS28 eseguite nella lettera, nessun difetto di soundness; due residui nominati (probe `'static` nei build di test, attribuzione panic del probe A-DL24).

**Q1 — A-MS25 ESEGUITA, il banking muore a compile-time.** `VmGate<'gate>(PhantomData<&'gate ()>)` (vm/mod.rs:502, nessun Clone/Copy); mint 1 `production_gate` PRIVATO `fn(&self) -> VmGate<'_>` (:553); mint 2 `MainUnit::vm_gate(&self) -> VmGate<'_>` (:16008) muore col borrow dell'acquire; le porte prendono `&VmGate<'_>` (park_main :543, vm_new :573). KS-MS-85-1 chiusa da rustc. Residui: (a) `vm_gate_probe` (:512-516) ritorna `VmGate<'static>` — nei build test/feature il banking RESTA possibile. Accettabile perché nei build campagna/parità il simbolo NON ESISTE (cfg = rustc giudice, KH85-1 chiusa) e i probe sono controlli positivi, non evidenza del sigillo; MA il fix è gratis → A-MS29. (b) Feature implementata `vm-gate-probe`; il mio ordine diceva `lifecycle-probe`: sostanza pari, ma il nome vive nei pin — dichiarare la rinomina, mai due nomi nel corpus. (c) Escape teorico `Box::leak(MainUnit)` → token `'static`: classe irriducibile di ogni sigillo a lifetime, nominata, non ordinata.

**Q2 — SÌ, strutturale.** `unit_cache_put`: superseded/victim/replaced dichiarati FUORI (:16319-16321); dentro il `borrow_mut` (:16325-16367) solo move-out (`remove`→push :16339-16341, `mem::replace` :16356/:16358, `remove(0)` :16362); emissione uc_stat/uc_log a borrow CHIUSO (:16368-16412); drop espliciti DOPO l'emissione (:16416-16418) — ledger-before-drop (KL-81-1) preservato. Residuo minore: le `UnitKey` stale droppano DENTRO il borrow (:16338-16342) — dealloc pura senza Drop utente, fuori dalla lettera di KS-MS-85-4; nominato. Probe A-DL24 (:16206-16251): `borrow()` read-only, collect in `Vec<String>`, `census_line` fuori dal `with` (:16249-16251) ✓. Nit: `mains` (cloni `Rc<Program>`) droppa a fine closure PRIMA della `Ref` — decrementi Rc dentro un borrow read aperto; il conteggio non può toccare 0 (la cache tiene i forti), nessun destructor corre: sound, basta una riga di commento.

**Q3 — KS-MS-85-3 MANTENUTA, non alleggerita.** Il morso A-DS26/F16 promuove il tripwire da "dente che non può mordere" a "dente VIVO" — prova la vitalità del contatore, non la costruzione. Il routing resta tag di valore: `is_main_put = cu.main_program.is_some()` (:16324); rustc non vieta un main in corsia include; il produttore unico è un pin (`main_program: Some` solo in `main_publish_ticket` :16286, KH85-3/A-TH31) — spelling+dente, non tipo. "Per costruzione" si scrive SOLO con `CachedMain`/`CachedInclude`. Rafforzamento mirato: la delibera peak WP-86 può aprire la via registry (Send/Sync, cache condivisa) — lì i siti di routing si moltiplicano e la disciplina non scala: A-MS27 diventa PRECONDIZIONE di quella via.

**Q4 — ACCETTABILE con emendamento.** worker_pool.rs:281-291: il dump DEVE girare sul thread worker (UNIT_CACHE thread-local) e gira dopo `worker_loop` dentro il catch_unwind → panic nel dump = abort. Coerente con Binding Rule 4 (panic=FAIL-FAST): un probe che panica deve uccidere la campagna, non emettere righe parziali in silenzio. Due difetti: (a) il messaggio :293-295 attribuisce al WORKER un panic del PROBE — strumento e misurando conflati; (b) panic a metà emissione lascia righe parziali nel log di un processo abortito: il verdict le deve rifiutare per exit-code, non per fiducia.

**Emendamenti:**
- **A-MS29**: `vm_gate_probe(anchor: &()) -> VmGate<'_>` — anche il mint di test lifetime-bound; nessun `VmGate<'static>` nel workspace.
- **A-MS30**: rinomina feature dichiarata (`vm-gate-probe` vs `lifecycle-probe` ordinata); un solo nome in pin/belt/documenti.
- **A-MS31**: dump A-DL24 in catch_unwind PROPRIO con messaggio distinto ("census probe panicked — aborting"), sempre abort; commento drop-order di `mains` nel probe.

**Kill-switch:**
- **KS-MS-86-1**: righe `unitcache_main_entry` citate da un run con exit≠0 ⇒ righe VOID.
- **KS-MS-86-2**: via registry/cache-condivisa aperta col routing a tag (`main_program: Option`) ancora in sede ⇒ delibera VOID finché A-MS27 non atterra.
- **KS-MS-86-3**: `VmGate<'static>` fuori cfg-test dopo A-MS29 ⇒ sigillo declassato ad ADVISORY.

*Niko Matsakis — il lifetime era la metà mancante; ora l'unico token eterno rimasto vive nei test, e anche lì può morire gratis.*

---

**VERBALE KLABNIK — sedia 3, Concilio WP-86 — CON EMENDAMENTI**

**VERDETTO**: le TRE elusioni WP-85 sono chiuse NEL CODICE (A-SK32 r.94-100, A-SK33 r.44-46/141-147, FAIL-refuse r.106). Ne trovo di NUOVE, una grave: il dente A-DL26 in battery non guarda mai MEASURE84.

**Q1 — battery-equivalence v2.** (a) SÌ, elusione: `grep -q "git=$BREV"` (r.94) è substring non ancorato — un OUT vecchio tutto-OK più UNA riga appesa `git=937e79d` (o un log copiato che cita la rev) passa; matcha anche il prefisso (`git=937e79dxx`). Nulla esige la riga terminale `== BATTERY-84PRE PASS (15/15) git=… ==`. (b) `.done` è forgiabile: una riga `rev=$BREV` in una dir propria accanto a un OUT copiato soddisfa r.97-99 — nessun path canonico, nessun hash. Aggravante: battery-84pre:46 scrive `.done` ANCHE a FAIL — il messaggio "never COMPLETED" è sovra-dichiarato. (c) `git diff --quiet HEAD` copre worktree-vs-HEAD (staged+unstaged combinati) MA il corner worktree==HEAD≠index sfugge (modifica→add→ripristino); peggio: l'append (r.158) non committato si cancella ripristinando il worktree — riapre one-per-chain.

**Q2 — A-SK34: implementata, con due residui.** VW2: sn≠30 ⇒ `$valid=0` (r.171) e la NAMED-DEVIATION vive sotto `if ($valid)` (r.189) — corretta. VA2: sn≠30 ⇒ FAIL (r.144) — corretta. Residui: (1) il CONTROLLO hello (r.181-188) NON ha check steady_n: un hello invalido con valori off-floor setta `model_ok=0` ⇒ NAMED-DEVIATION da misura invalida — violazione A-SK34 sul controllo. (2) `$B{$fx}` assegnato PRIMA dei check (r.143): con sn≠30 o a3≠0 la riga "VA2 PASS"+derivata viene comunque STAMPATA (r.148-152) — verdetto globale FAIL, ma riga citabile.

**Q3 — F16.** Freshness SANA: `TMPD` fresco + `rm -f` (r.411) — nessun falso-verde da run precedenti; test rinominato/filtro vuoto (cargo exit 0 a 0 test) è catturato dalla metà `&& grep` su log vuoto. Residui WP-81: (i) manca il pin `test result: ok. 1 passed` in a_ds26.out; (ii) la compilazione del test-bin entra nell'identità del gate senza essere pinnata (né rustc né feature in-band nell'OK di F16) — F16 certifica il TEST-bin, non il binario di campagna: accettabile, ma va dichiarato nell'header.

**Q4 — A-DL26.** "20,65MB" attaccato È preso (`\s*` ammette zero). Evasioni residue: (1) `Mi?B` solo — GiB/KiB/GB/TB fuori perimetro; (2) case-sensitive: "mb/mib" evade; (3) le eccezioni sono LINE-WIDE: `x\s*W` matcha qualunque "xW"/"x W" in prosa ("linux Worker", "matrix W") ⇒ riga intera esente; (4) companion non verificato: un "1 B =" qualunque sblocca qualunque MiB sulla riga; (5) **la GRAVE: battery-84pre:42 invoca il gate SENZA argomento ⇒ TARGET=MEASURE81 (r.20) — MEASURE84 non è MAI nel perimetro battery; il dente ha morso solo a invocazione manuale, il 15/15 non lo copre.**

**Emendamenti:**
- **A-SK36**: stamp ancorato — obbligatoria e UNICA la riga terminale `^== BATTERY-84PRE PASS (15/15) git=$BREV ==`; `.done` scritto SOLO a PASS, porta `rev=$BREV sha256=<sha256(OUT)>`, checker ricomputa.
- **A-SK37**: ledger — aggiungere `git diff --quiet --cached HEAD` (index); l'append committato in campagna col commit-id citato nel claim.
- **A-SK38**: verdict — steady_n==30 anche sui controlli; PASS/derivata emessi solo a blocco senza fail (flag per-blocco).
- **A-SK39**: F16 — pin `1 passed` in a_ds26.out + rustc -V in-band; header dichiara "certifica il test-bin".
- **A-SK40**: gate-cifre — loop su TUTTI i `MEASURE8[4-9]*_RESULTS.md` in battery; unità `[KMGT]i?B` case-insensitive; eccezioni per-FIGURA; companion verificato (bytes/1.048.576 ≈ MiB).

**Kill-switch:**
- **KS-SK-86-1**: equivalenza su OUT senza riga PASS ancorata o `.done` senza sha ⇒ equivalenza VOID.
- **KS-SK-86-2**: append di ledger non a HEAD a chiusura campagna ⇒ equivalenze della campagna VOID.
- **KS-SK-86-3**: battery citata come copertura MEASURE84+ mentre measure-cifre gira sul default 81 ⇒ bytes-first ADVISORY, documento non pubblicabile.
- **KS-SK-86-4**: NAMED-DEVIATION raggiunta da un controllo privo di check di validità ⇒ declassata a FAIL (estende KS-SK-85-3).

*— Klabnik, sedia 3.*

---

# VERBALE — Hejlsberg, sedia 4, Concilio WP-86

**VERDETTO: CONCORDO CON EMENDAMENTI. La v2 chiude il provenance-strip nella regione `[[package]]`, NON a scopo-file; la macchina rafforzata non ha un positive-control live — lo ordino.**

**Q1 — A-AH35 in base-arm-build.sh v2.** (a) **BUCO NOMINATO, fail-open**: le cancellazioni FUORI dai blocchi `[[package]]` non le giudica NESSUNO. `split_lock` colleziona solo dopo il primo `[[package]]`: una riga `version = 4` di header (o commento @generated) cancellata entra in `pruned_lines` (:66) ma non in (c) né nel pruned-set (d) — passa legale. Un downgrade di formato lock è un vettore reale (cambia la semantica dei checksum). Secondo difetto, fail-closed ma sporco: `[[patch.unused]]`/`[metadata]` in coda non resettano `collecting` — le loro righe si fondono nel buffer dell'ULTIMO package e le coppie name/version RI-CHIAVANO il blocco; cancellazioni lì → VOID con chiave fuorviante, e un arm LEGITTIMO con `[[patch.unused]]` rischia VOID spurio. (b) L'edge disambiguato `"foo 1.2.3 (registry+…)"` passa come edge legale: **giusto** — è sintassi lecita delle liste `dependencies`; e la sed (:126) estrae correttamente `foo` (lo spazio esce dalla classe). Nessun buco. (c) Registrare il TARGET dell'edge è coerente col mio intento: la campagna nomina il PACKAGE potato (il caso reale S-83.0 nomina `mimalloc`, non il proprietario superstite); l'edge cancellato è l'ombra dello stesso prune nella lista del survivor, e il proprietario resta auditabile nel diff INTEGRALE archiviato (:150). Coerente; miglioria opzionale in A-AH38.

**Q2 — NO, il bite sintetico non basta.** Ha provato che il dente morde (strip→VOID), non che un arm LEGITTIMO passa: il positive-control live della v2 non è MAI girato (measure84 senza base arm), e il difetto `[[patch.unused]]` sopra è esattamente la classe di VOID spurio che un dry-run scoprirebbe. Lezione WP-83: «un dente che non può mai passare non certifica nulla» — vale simmetricamente per una macchina mai vista passare. **Ordino il dry-run** (KS-AH-86-1).

**Q3 — Nessun buco IN PRATICA, ma copertura per COINCIDENZA.** F16 trasclude `cargo test -p php-runtime --lib a_ds26_…` (gate-lever-fixtures2.sh:412): l'oggetto include il codice test in crates/php-runtime. Il dente (i) (battery-equivalence.sh:70) esige delta VUOTO su `crates Cargo.toml Cargo.lock` per QUALSIASI equivalenza — un cambio al test trippa (i) e rifiuta tutto. Quindi coperto — ma dalla LARGHEZZA di (i), non dal manifest: la riga `lever-fixtures2:` (:64) non nomina l'oggetto in-cargo. Se (i) venisse mai ristretto, il buco si aprirebbe in silenzio. Sigillo di tipo > allowlist testuale vale anche qui: la copertura va DICHIARATA (A-AH39).

**Q4 — Pulito, verificato.** `resolver = "2"` pinnato alla radice (Cargo.toml:2): i feature dei dev-dep NON unificano su `cargo build` — `vm-gate-probe` accende php-runtime SOLO nei target test; le due flavor convivono nel target dir con fingerprint distinti (niente thrash), e la prova è in campagna: U2==UNION_HASH e FINAL==UNION (measure84:147,186) PASS. I binari di lane DIVERSI sono il sigillo, non un difetto (KH85-1: il simbolo non esiste nel build campagna). **Lock invariato: CONFERMATO dalla storia** — i feature non si registrano nel lock, php-runtime è path-dep; ultimo tocco a Cargo.lock = 7fed57d (S-82.0), nulla in 746bc24…9c2f946.

**Emendamenti:**
- **A-AH38**: legalità A-AH35 estesa a SCOPO-FILE — ogni riga cancellata deve essere ATTRIBUITA (edge in survivor, riga di blocco interamente potato, o niente); cancellazioni fuori dai blocchi `[[package]]` ⇒ arm VOID. `split_lock` flusha su OGNI header `[[`/`[`, non solo `[[package]]`; opzionale: header registra coppie owner→target.
- **A-AH39**: la riga manifest `lever-fixtures2` nomina l'oggetto in-cargo (`crates/php-runtime/` come dir-prefix o il modulo test esatto); idem ogni futuro gate che trasclude `cargo test`. La copertura via (i) resta belt, non dichiarazione.

**Kill-switch:**
- **KS-AH-86-1**: cifra base-arm usata verdict-grade senza UN passaggio live PASS della v2 a verbale (dry-run alla prossima campagna slope, PRIMA dell'uso) ⇒ arm VOID.
- **KS-AH-86-2**: lock-diff con riga cancellata non attribuita per NOME (post-A-AH38) ⇒ arm VOID.

*Anders Hejlsberg — sedia 4. Il dente distingue prune da provenienza dentro i blocchi; ora deve possedere anche il file — e passare vivo almeno una volta.*

---

**CONCILIO WP-86 — SEDIA 5 (Lars Bak) — perimetro: misure, alloc-rate, statistica**

**VERDETTO: CON EMENDAMENTI.** Ho ricomputato dai raw: tutte le cifre del verdict84 tornano al byte. Ma R=3 non fonda un pin nuovo, il byte-identico VDL24 non è ancora provato non-artefatto, e la slope non si trasferisce gratis a bit mossi.

**Q1 — VP: pin MOSSO sì, pin NUOVO no.** Raw ricomputati: r1 228.278.272 B = 217,71 MiB · r2 239.878.144 B = 228,77 MiB · r3 240.287.744 B = 229,16 MiB. Pin S-82.0 = 232±1 MiB = [242.221.056; 244.318.208] B: r3, il più alto, manca la banda per 1.933.312 B = 1,84 MiB. **Tutti e tre fuori, stesso lato: dichiarare il pin MOSSO con R=3 è legittimo** — la lezione min-of-R serve a stimare un valore, non a falsificare una banda che tutti i punti violano. Ma lo spread NON è rumore gaussiano: r2↔r3 distano 409.600 B (0,17%), r1 è sotto di 11,45 MiB (5,26%) **e ha wall 4,29s contro 3,32/3,32s dei gemelli** — firma di primo-run freddo (overlap worker diverso), cioè bimodalità run1-vs-resto, non varianza. Nota anche il commit: 251,8 vs 262,6/260,3 MiB, stessa struttura. La delibera peak PUÒ poggiare su questi punti solo come **envelope conservativo (max = 229,2 MiB)**; qualunque pin identità nuovo esige R≥9 con il primo run nominato o escluso. Il vecchio 232/232/232 spread-0% era con ogni probabilità quantizzazione d'arena, non identità fisica: la partizione ha spostato il quanto.

**Q2 — VW2: nessuna cella fuori modello.** Ricomputo integrale: len383 → 39/39 righe steady a_calls=2, a_bytes=766=2·383; len384 → 39/39 righe a_calls=4, a_bytes=1538=2·384+2·385; hello len98 → 39/39 righe a_calls=2, a_bytes=196=2·98. a3_calls=0 ovunque; req=1 (warm-up ~61MB) correttamente fuori dallo steady. **KB-85-3 VERIFICATO: a_calls ∈ {2,4} su entrambi i lati, bisezione esatta.** Residuo: il piecewise è provato solo AL confine — manca un terzo punto len≥500 che verifichi che il coefficiente (len+1) scala (A-BB44).

**Q3 — VDL24: bande difendibili ma nude; il byte-identico va discriminato.** Le bande 500KB/3MB non sono arbitrarie rispetto al segnale atteso (residuo one-time campagna-83 = 7,00 MB: alto=43%, basso=7% del segnale, zona AMBIGUA fail-closed in mezzo — buona forma), ma sono pin numerici liberi che invecchiano: vanno ancorate strutturalmente al residuo di record (A-BB42). Il byte-identico 7.349.977 B su thr0/thr1 (e floor_inc=997.878, retained_walk 4.283.308 identici) È compatibile con determinismo del conteggio (si contano call/byte, non indirizzi; fixture identica) — MA la riga dichiara `net_window=process-counters`: se entrambe le righe derivano dalla stessa coppia di snapshot di processo, l'identità è artefatto. **Controllo negativo che discrimina: perturbazione MONOLATERALE calibrata** — fixture di thr1 servita via path con len≥384 (Δ predetto ESATTAMENTE dal piecewise VW2: +2 call, +2·(len+1) byte) o canary alloc K nel prelude del solo thr1. Se Δ appare solo sulla riga thr1 → finestre attribuibili, PER-THREAD regge; se appare su entrambe o su nessuna → stessa cella, VDL24 VOID (A-BB41).

**Q4 — Slope: chiusa sul binario 83, non sul binario 84.** Corretto non rifare la campagna slope, MA la cifra ~55–75× è una proprietà misurata di 7a610457; VmGate v2 + partizione hanno mosso i bit del path che la leva attraversa. Trasferirla al binario nuovo come attesa è lecito; citarla come cifra di verdetto sul binario 84 no. Serve uno smoke-slope min-of-R R=3 a ogni cambio binario, con sola banda d'allarme (lever >10× la sua cifra ⇒ slope riaperta) — costo minuti, non una campagna (A-BB43).

**Emendamenti**
- **A-BB40**: pin peak nuovo solo con R≥9, primo run nominato/escluso; delibera WP-86 usa envelope max 229,2 MiB, mai la media di 3.
- **A-BB41**: canary monolaterale VDL24 (Δ predetto dal piecewise) prima di fidarsi del byte-identico.
- **A-BB42**: bande VDL24 riscritte come 0,5×/0,1× del residuo-83 (3,50/0,70 MB), ancorate al record.
- **A-BB43**: cifre slope binario-bound (tag 7a610457); smoke-slope obbligatorio a ogni trasferimento.
- **A-BB44**: terzo punto VW2 a len≥500.

**Kill-switch**
- **KB-86-1**: pin peak dichiarato con R<9 o senza nominare il primo run ⇒ pin VOID.
- **KB-86-2**: registry condivisa deliberata senza canary A-BB41 superato ⇒ delibera BLOCCATA, PER-THREAD resta CANDIDATA.
- **KB-86-3**: cifra slope citata su binario diverso da quello di misura senza smoke-slope ⇒ claim VOID.

— Bak, sedia 5

---

**VERBALE — Concilio WP-86, sedia 6 (Pedersen, confine per-richiesta/lifecycle/test). Mandato: REFUTARE.**

**VERDETTO: CON EMENDAMENTI — A-PP26/28/29 eseguite nella lettera; Q1 chiusa SOLO in steady-state (cecità req1 non nominata), Q2 rifiuto NON ereditabile per macchina, Q3 assunzione mista ma verdetto order-agnostic, Q4 fail-closed verificato.**

**Q1 — A-PP26: chiusura PARZIALE, la cecità che nominai è ancora silente.** Il pin totale c'è ed è giusto (`worker_pool.rs:1070` t_mid dopo req1; `:1073-1078` totale dopo req2 == t_mid; gemello positivo `:1120-1124`; `uc_entry_count` a `vm/mod.rs:16190-16192` somma `slot.len()` = entrambe le corsie, precondizione same-thread documentata `:16186-16189` come ordinato). MA il commento `:1047-1051` dichiara che «request 1 legitimately seeds the prelude include entries» e piega il pin a «request 2 adds ZERO»: un inserter **once-only** (insert-if-absent) che pubblicasse il main fatale in forma-include DENTRO req1 viene assorbito in t_mid e non è MAI osservabile — né il codice né il doc del probe nominano questa finestra. È cecità silente: il seeding è dichiarato, la cecità no. In più il totale non è mai visto contare la corsia include (il positivo `:1120` è un MAIN pulito; nessun pin `t_mid > e0+…`).

**Q2 — A-PP28: il binario è a posto, l'eredità NO.** `worker_count()` = W EFFETTIVO post-default (`worker_pool.rs:229-231`, senders.len()); stamp in-band `main.rs:223-224`; W è fisso alla creazione del pool, quindi drain-time == sample-time. Ma il rifiuto vive SOLO nella guardia di verdict84 (`verdict84.sh:199-208`), scoped al glob `axum.84*.log` di QUESTA campagna. Il prossimo verdict slope è un file che non esiste: nessuna macchina lo obbliga a nascere col rifiuto — NEXT_SESSION §4 lo affida a una nota («da cablare»), cioè disciplina. **NON erediterà. Il pin va ordinato.**

**Q3 — Assunzione MISTA; il verdetto regge comunque.** Il mapping è di MACCHINA: `next_worker` parte da 0 (`worker_pool.rs:306`) e il dispatch è `fetch_add(1, Relaxed) % senders.len()` (`:439-442`) — richiesta k→worker k mod 2, deterministico a client sequenziale. Ma «failed connects never dispatch» (`measure84-campaign.sh:126`) copre solo il connect RIFIUTATO: un probe che connette e poi scade su `-m 1` HA dispatchato (fetch_add consumato) — lì il conteggio è disciplina, e il commento sovra-dichiara. Però VDL24 NON dipende dal mapping: legge thr= in-band, esige ESATTAMENTE 2 thread con ord=1 (`verdict84.sh:99`), rifiuta thr duplicato (`:88`) e prende first=max/second=min (`:101-102`) — order-agnostic. Due richieste sullo stesso worker ⇒ il secondo worker ha cache vuota, zero righe ord=1 ⇒ @thr==1 ⇒ FAIL (più `NROWS>=2`/`NTHR>=2` in campagna `:139-140`). **Verdetto valido: fallirebbe forte, mai classificherebbe.**

**Q4 — La finestra ESISTE, il fail-closed pure.** Panic di QUALSIASI worker ⇒ `process::abort()` (`worker_pool.rs:292-297`); in axum il hook globale abortisce PRIMA dell'unwind (`:273-276`): nessun dump di nessun thread, e abort non flusha i buffer — righe perse anche per chi aveva già dumpato. SIGTERM non-graceful o SIGKILL: idem. In TUTTI i casi la lettura è FAIL, mai cifra: campagna `:139-140` + verdict `:99`; riga tronca cade sui pin alloc_id/arm (`:90-91`). Fail-closed su <2 thread rows: **verificato**.

**Emendamenti:**
- **A-PP30**: a_pp20 con PRE-WARM — richiesta pulita PRIMA del primo fatale, snapshot totale, delta==0 su ENTRAMBE le richieste fatali (chiude la finestra once-only-req1); cecità residua NOMINATA nel doc del probe; pin che il totale ha contato ALMENO una entry include.
- **A-PP31**: rifiuto w≠1 spostato nel PARSER condiviso delle righe `reqns:` (punto unico, tracked) O grep-gate in gate-lever-pins: ogni script che parsa `reqns:` deve contenere il rifiuto fail-closed — il verdict slope non può nascere senza.
- **A-PP32**: retro-correzione del commento DL24 (`:126`): connect-ok-timeout DISPATCHA; il claim di mapping è ADVISORY, l'attribuzione è solo thr= in-band.

**Kill-switch:**
- **KS-PP-86-1**: verdict che parsa `reqns:` senza rifiuto w≠1 nel proprio corpo ⇒ verdict illegittimo, campioni VOID.
- **KS-PP-86-2**: claim «mai retained include-form» pre-A-PP30 ⇒ vale solo steady-state, declassato.
- **KS-PP-86-3**: attribuzione per-worker DL24 basata sull'ordine delle richieste anziché su thr= in-band ⇒ attribuzione VOID.

— Pedersen, sedia 6.

---

# VERBALE WP-86 — Sedia 7, Daan Leijen (mimalloc/footprint/contabilità)

**VERDETTO: ESECUZIONE DI A-DL24 ACCETTATA CON EMENDAMENTI. La classificazione PER-THREAD è VALIDA-CONDIZIONATA: manca il discriminatore che un'identità al byte non può dare. La delibera peak NON può usare i tre picchi VP così come sono.**

**Q1 — PER-THREAD: plausibile, non ancora falsificato.** Il probe è costruito bene: `net` è depositato AL PUBLISH per-thread (`cu.main_program_net`, riga 16220) e solo EMESSO al teardown — le due righe non sono due letture dello stesso contatore vivo. Ma il bracket resta `net_window=process-counters`, e il codice stesso lo dichiara pollutabile ("other threads can pollute it — A-DL20", riga ~1511). La sequenzialità dello script (probe→req2→req3) mitiga; non elimina i drop differiti del request precedente (che DEFLAZIONEREBBERO thr1) né l'init lazy dell'altro worker dentro il bracket di thr0. L'identità 7.349.977 B = 7,01 MiB su tre osservazioni è coerente con determinismo pieno (stessa sorgente, FxHash deterministico, conteggio bytes-requested) — ed è essa stessa evidenza che la pollution fu zero. Però l'identità è compatibile ANCHE con una simmetria patologica: due input IDENTICI non possono distinguere "il secondo thread ripaga" da "la finestra fattura due volte lo stesso lavoro". Il mio Q2 di WP-85 prevedeva ≈7,35MB ⇒ per-thread e la cifra è arrivata esatta: troppo esatta per fidarsi senza perturbazione (lezione di memoria: un dente nuovo va fatto mordere). Ordino A-DL28/29 prima che il ×W diventi delibera.

**Q2 — Il gap ha già un primo addendo NEL RAW.** net(ord1) 7.349.977 B contro `retained_walk_bytes` 4.283.308 B per thread: 3.066.669 B vivi-al-lower ma FUORI dal walk (prelude/interner thread-local, side-table) — parente stretto dei 3.057.613 B opachi di WP-83, denominatori diversi. Contro il fisico 3,44 MB/worker i candidati sono: pagine mimalloc ricommittate senza nuovo fisico (il raw mostra slack enorme: bin size=32 reserved=327.520 committed=81.888 used_b=672) e bytes contati mai toccati. **A-DL27 eseguibile**: run unico W=1/2/3 hello sequenziale, per finestra (a) net per-thread (righe esistenti), (b) `mi_bin` esteso con `thr=`, somme committed/used per heap, (c) vmmap Physical footprint con MIMALLOC_PURGE_DELAY=0 in-band. Verdetto: marginale-contato − marginale-fisico scomposto in slack-condiviso + contato-non-toccato, chiusura ±5% o FAIL.

**Q3 — VP: tre numeri, nessuna attribuzione — REFUTO l'uso deliberativo.** 228.278.272/239.878.144/240.287.744 B con r1 sotto di 12.009.472 B = 11,5 MiB: lo spread è tornato dopo un pin 0-spread. Candidati allocatore: purge timing (PURGE_DELAY non pinnato in-band nel raw VP), ordine di spawn dei thread-heap sotto la partizione UnitSlot, first-touch del primo run. Nel raw noto anche `mi_proc phys_peak=38.650.360 < phys=69.976.568`: un peak sotto il corrente è incoerente — il contatore mi va spiegato PRIMA di citarlo ancora. Ordine: R≥9 min-of-R (la forma che ha chiuso VC), env purge in-band, `peak_rss` mi per-finestra accanto a time -l. Senza, i tre picchi sono aneddoto.

**Q4 — A-DL26: rispettato sulle cifre nuove, VIOLATO su una importata.** Campione: «7.349.977 B = 7,01 MiB» ✓; «228.278.272 B = 217,7 MiB» ✓; «12.009.472 B = 11,5 MiB [derivata]» ✓. Ma «3,44 MB/worker (vmmap)» compare DUE volte (righe 33-34 e 78) senza byte esatti: KL-85-2 morde sulla cifra importata da S-82.0. Sanare portando i byte dal record originale.

**Emendamenti**
- **A-DL28**: perturbazione discriminante — due fixture DIVERSE (hello vs hello+padding), una per worker; per-thread ⇒ ogni net traccia la PROPRIA fixture; nets uguali ⇒ finestra congelata, classificazione RITIRATA.
- **A-DL29**: controllo positivo pollution — burst allocativo concorrente durante un bracket; il net DEVE muoversi, altrimenti il contatore non conta.
- **A-DL30**: `mi_bin`/`mi_proc` con `thr=` e spiegazione del phys_peak<phys prima di ogni nuovo uso.
- **A-DL31**: A-DL27 nella forma eseguibile del Q2, chiusura additiva ±5%.

**Kill-switch**
- **KL-86-1**: delibera peak su VP con spread non attribuito → delibera NULLA.
- **KL-86-2**: budget ×W promosso senza A-DL28 eseguito → riclassificato IPOTESI.
- **KL-86-3**: cifra mi (`phys_peak`) citata mentre incoerente → riga esclusa dal corpus.

*Daan Leijen — l'algebra regge; ora tocca all'identità dimostrare di non essere uno specchio.*

---

**VERBALE STOGOV — sedia 8, Concilio WP-86 (Zend/opcache, cache di unit)**

**VERDETTO: CONCORDO CON EMENDAMENTI.** I miei quattro ordini WP-85 sono eseguiti nella lettera; due buchi nuovi nominati, uno strutturale (Q2).

**Q1 — A-DS26: SÌ, il dente morde nel posto giusto.** Il test (vm/mod.rs:19027-19098) inietta il main-tagged in testa alla corsia include (:19061-19067, bypass dichiarato, coerente con KH85-3) e l'evizione passa dal **put PUBBLICO** (:19074-19076): il ramo `victim.main_program.is_some()` (:16402-16407) è esattamente quello esercitato. Contatore +1 E ways_evictions==1 pinnati (:19081-19082). L'evento è pinnato solo se armato (:19085-19093): in `cargo test` nudo la metà-evento NON è mai asserita — vive SOLO in F16 (fixtures2.sh:411-420, `PHPR_UNIT_CACHE_LOG` + grep ancorato `^unitcache main_evicted `). Accettabile perché F16 è in battery, ma va vincolato (KS-DS-86-1). Cleanup: la chiave è rimossa (:19095-19097), però `c.borrow_mut().remove(&key);` droppa lo UnitSlot **con la RefMut ancora viva** (ordine dei temporanei) — il test che celebra A-MS28 viola la sua lettera; benigno oggi, da sanare (A-DS33). Residuo: UC_STATS.main_evicted resta 1 sul thread di test — futuri pin ASSOLUTI ==0 in-cargo sarebbero avvelenati; solo delta.

**Q2 — A-DS28: la sig resta SCOPERTA, e la classe di cecità delle closures si RIPETE.** Verificato su bytecode.rs: `methods_ci`/`fn_ci`/`class_index`/`all_props_public` sono DERIVATI — non divergono da soli, esclusione dichiarabile. Ma divergono per mutazione REALE e confrontano UGUALE oggi: **`m.deferred`** (classe con parent late-bound compila lì, NON in m.classes — identica sede-separata delle closures), `conditional_traits`, `Module.strict` (declare(strict_types) fuori sig!), `enum_cases`, `attributes`/`prop_attributes`/`const_attributes`, `is_final`/`is_abstract`/`instantiable`, `own_prop_vis` (private→public), `uses_traits`, `abstract_sigs`, e i corpi hook get/set (in `prop_info`, non in `c.methods` — relocate_module_class_ids li elenca separati). KS-DS-85-3 ("campo NUOVO same-commit") non basta: questi campi ESISTONO già. La forma giusta è compile-time: sig via destructuring esaustivo senza `..` — rustc rompe la build a ogni campo nuovo (A-DS30).

**Q3 — VA2: il VOID è SANATO nella lettera.** Ledger (fixture-oracle.ledger:7-9): full-body PASS per fixture, `rev=937e79d`, `arm=union`, PRIMA della finestra — KS-DS-85-1 legittimamente sollevata; verdict84.out:7 lo lega ad a3==0 sui tre arm. Il mio ordine diceva «sull'arm union»: la lettera è quella giusta e rispettata. **Ma il buco residuo esiste ed è REALE in-tree**: la finestra gira su census (d70b86d0 ≠ union d440c341) e `reloc_unexpected_shared` panica SOLO sotto mem-census (vm/mod.rs:16448-16455) — un fatal solo-census resta non osservabile dall'oracle; a3==0 è vitalità, non parità. → A-DS32.

**Q4 — Put collect-then-emit: semantica INVARIATA nella lettera.** Routing per tag (:16324/16345), supersede-per-path solo su chiave nuova con drop dello slot intero (:16332-16343), main revalidate/replace MAI evizione (:16351-16356), include fp-replace in place / FIFO remove(0) (:16357-16365): la mia semantica opcache regge. Contatori: fp_replaced :16390, inserts :16393, ways_evictions :16396, main_evicted :16403 — condizioni di scatto identiche; byte superseded ledgerati PRIMA del drop (:16374-16376). Drops fuori borrow (:16416-16418). **Non certifico l'ordine al bit**: i drop ora avvengono DOPO tutte le emissioni (in v1 erano intercalati) e l'ordine intra-victim (main_evicted→evict-fp) non è pinnato da nessun gate — i detector contano, non ordinano. Innocuo sotto l'audit KS-MS-85-4, ma l'ordine eventi è API di probe (WP-64): va pinnato (A-DS31).

**Emendamenti:**
- **A-DS30**: sig() riscritta a destructuring ESAUSTIVO di Module/CompiledClass (rustc giudice); superfici mancanti in sig + ≥3 mutanti che mordono: deferred-body, enum case value, final-flag; derived dichiarati esclusi.
- **A-DS31**: pin dell'ORDINE eventi uc_log per singolo put (supersede*→main_evicted→evict-fp) in test in-cargo + doc UC_LOG_EVENTS.
- **A-DS32**: vitalità per-fixture dell'arm CENSUS in-band (status 200 + body-len>0) nella finestra, o body-sha census fuori finestra.
- **A-DS33**: cleanup di a_ds26 droppa lo slot FUORI dal borrow; residuo main_evicted=1 sul thread documentato.

**Kill-switch:**
- **KS-DS-86-1**: pin main_evicted==0 citato dopo QUALUNQUE modifica al put-path senza F16 verde nella stessa battery ⇒ pin ADVISORY.
- **KS-DS-86-2**: claim di purezza a_ds17 con campo Module/CompiledClass fuori sig senza esclusione NOMINATA ⇒ claim NULLO.
- **KS-DS-86-3**: cifra per-fixture di finestra census senza vitalità census in-band ⇒ cifra declassata ad ADVISORY.

*Dmitry Stogov — la lettera dei miei ordini è onorata; la sig impari da rustc a contare i campi.*

---

**VERBALE — B. Gregg, sedia 9, Concilio WP-86 (perimetro: metodologia, attribuzione, identità dei raw)**

**VERDETTO: PASS CON EMENDAMENTI.** Ho riconciliato i raw a ricomputo indipendente: VP 228.278.272/239.878.144/240.287.744 B = 217,7/228,8/229,2 MiB (raw r1 riga `maximum resident set size` e campaign.out concordano; 240.287.744−228.278.272 = 12.009.472 B ✓); VDL24 net(ord1) 7.349.977 B su thr0 E thr1, identici al byte, con `alloc_id=memcount-v2-s82`, `arm=axum-worker`, meta `w=2 phase=dl24 git=937e79d mem_hash=85fd009f` ✓; VW2 766=2·383, 1538=2·384+2·385, hello 196=2·98 ✓; ledger 5 righe PASS a 937e79d, union_hash d440c341 ✓. Nessuna cifra del MEASURE84 diverge dai raw.

**Q1 — Catena: REGGE, con un buco nominato.** Battery VERA a HEAD: «battery-84pre PASS at rev 937e79d == HEAD (per NAME, 15/15)» — ramo diretto, NON equivalenza. Ogni run porta la tripla in-band nel raw stesso (r1 riga 57: `driver_sha=fe6983d8 git=937e79d bin=d440c341`) e l'ENFORCE per-run `bin == matrix bin[arm] (git == matrix git)` su tutti gli 8 run. MA `head_unmoved` NON è per-run in Phase W2: un solo refusal (riga 179) copre DUE invocazioni (180–181); il run len384 vive sull'ENFORCE del driver, non sulla doppia catena. → A-BG37.

**Q2 — VP: parametri IDENTICI, driver NO.** Dai summary: 82p e 84p condividono `warmup=10 measured=100 idle_secs=10`, mode=axum, hello.php, W=10, R=3. Però `driver_sha` 76e74a77 (82p) → fe6983d8 (84p): il driver è cambiato al commit 7b90614 («measure78 reqns drain hooks») — e l'harness è parte dell'IDENTITÀ del protocollo (lezione WP-81). Il confronto col pin 232±1 resta LEGITTIMO come esito nominato («pin MOSSO», mai PASS silente — la forma del verdict è corretta), perché l'unica superficie nuova (drain reqns) risulta mai ingaggiata: guard KS-PP-85-1 = 0 righe reqns. Ma «mai ingaggiata» è plausibilità, non certificazione: l'ATTRIBUZIONE alla partizione resta lettura candidata — come MEASURE84 onestamente dichiara — finché l'inerzia del driver non è certificata. → A-BG36/KG-86-2.

**Q3 — Dicitura: PULITA.** Zero occorrenze di «interleaved» in MEASURE84 e verdict84.out (KG-85-2 rispettata); fasi P→DL24→A2→W2 sequenziali dichiarate («Sequential, ENFORCE, one rev») e così eseguite nel campaign.out. Scope nominati: VDL24 (arm=axum-worker, W=2, per-thread), VA2 (fixture+arm+rev), VW2 (sito+len+arm census). UNICA smagliatura: la riga VP nomina W, R, arm ma NON la fixture né il driver_sha. → A-BG38.

**Q4 — La dichiarazione NON basta.** A-BG33/A-BG35 come «vincoli della prossima slope» sono promesse; A-PP28 invece ha già il dente (guard w= cablata nel verdict). L'asimmetria è esattamente il difetto che il Concilio ha già pagato tre volte (zeri ciechi). Ordino il pin fail-closed: KG-86-1.

**Emendamenti**
- **A-BG36**: ogni confronto cross-campagna con un pin NOMINA il driver_sha di ENTRAMBI i lati; se differiscono, allegare audit del diff driver con giudizio di inerzia per il modo misurato. Retro: il «pin MOSSO» resta valido; l'attribuzione alla partizione resta candidata.
- **A-BG37**: `head_unmoved` prima di OGNI invocazione del driver (sanare Phase W2, righe 179–181 di measure84-campaign.sh).
- **A-BG38**: la riga VP del prossimo verdict nomina anche fixture e driver_sha (lo scope completo è arm, W, R, fixture, driver).

**Kill-switch**
- **KG-86-1**: prossimo verdict slope senza (a) campioni base per-request `curl -w %{time_total}` nei raw E (b) stringa `regime=` definita nel verdict ⇒ VERDICT NULL d'ufficio, fail-closed.
- **KG-86-2**: confronto con pin cross-campagna senza driver_sha di entrambi i lati in chiaro ⇒ confronto VOID.

*— B. Gregg, sedia 9*

---

