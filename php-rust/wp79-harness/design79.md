# design79.md — Leva A-BB6: unit cache TL estesa al MAIN (S-79.0.8, vincoli Council WP-80; EMENDATO S-80.0.7 per Council WP-81)

**Stato**: DESIGN scritto in S-79.0 DOPO il pre-lever hardening ed EMENDATO
in S-80.0.7 NELLO STESSO COMMIT della misura di riferimento R≥3
(`wp80-harness/MEASURE80_RESULTS.md` — il vecchio §1 ADVISORY è sostituito,
A-BG17/KG-81-1) e del DR-1 chiuso a verdetto macchina (§5). Vincoli attivi:
mappa kill-switch in coda + la tabella NUOVA WP-81 (KH81-* · KS-MS-81-* ·
KS-SK-81-* · KS-AH-81-* · KB-81-* · KS-PP-81-* · KL-81-* · KS-DS-81-* ·
KG-81-* — verbale `wp81-harness/COUNCIL_WP81_REVIEWS.md` §Kill-switch).
Nessuna riga di implementazione della leva è stata scritta: questo file è
il contratto che la sessione di implementazione esegue e che il Concilio
giudica.

## 1. Canale (dati decomposti, ASSOLUTI) — S-80.0.6: misura di riferimento

⚠️ La versione originale di questo § citava uno smoke R=1 (binario
37f630d6, 83.834 call/13,36MB) SENZA raw nel repo né riga matrix — marcata
ADVISORY dal Concilio WP-81 (A-BG17, KG-81-1) e SOSTITUITA. **La misura di
riferimento è `wp80-harness/MEASURE80_RESULTS.md`** (R=3, spread 0,0%,
driver ENFORCE con identità git+quintetto, raw committati; git 6910767,
census 5c9c6eec481d5133). Cifre chiave (hello, arm axum, steady, churn
LORDO gross=1):

- fase a totale: **80.476 call / 13.015.960 B** per richiesta
- **a1 (prelude lower+compile): 74.288 call / 10.825.612 B (~83% dei byte)**
  — IDENTICO su ogni fixture e ogni run
- a2 (main proper, derivato): 6.188 call / 2.190.348 B
- a3: 0 steady (unit-cache HIT; >0 su MISS — controllo positivo in-cargo
  `census_split_a1_prelude_and_a3_include_discriminate`)
- b=730/95.627 · c=0 · resid=44,1/43.463 · depth_max=1 · **inflight_max=1**
  (osservabile closed-sequential A-TH9, verdict-grade) · a3_trip=0

La forma fedele della leva (§2) non cacha "il main script" da solo: cacha la
UNIT MAIN, che per costruzione CONTIENE il prelude compilato. Un HIT salta
**a1+a2 insieme** (13,0MB/req su hello; 13,04MB su include_gate; 13,39MB su
include_heavy). Dimensionamento su ASSOLUTI, mai percentuali (KB-80-1);
include_heavy è FIXTURE-B, mai proxy WP — vietata l'estrapolazione di
percentuali a WordPress (KB-81-4).

Cifre churn: CountingAlloc conta `realloc` a taglia piena e non netta i
dealloc — ogni cifra è **churn LORDO, upper bound** (A-DL1/A-AH11), marcato
in-band `gross=1` sulla riga census (A-DL10). Dichiarati (A-BB19):
resid(req 1)=0 per costruzione (init di processo non contato); il
`Vec::with_capacity(functions.len())` sta dentro a1; audit closures-nel-
prelude da eseguire nella sessione leva se il prelude acquisisce closure
literal (oggi ≈0 sulle fixture).

## 2. Forma fedele (Stogov A-DS5 — vincolante)

**NON una seconda cache.** La STESSA unit cache thread-local (WP-63/66:
Rc-owned, fingerprint, ways-eviction, da S-79.0.7 supersede-per-path) estesa
alla unit MAIN:

- Il main oggi NON fa né `canonicalize` né `stat` (`meta.path` arriva dal
  handler): il probe main fa **canonicalize + stat** di `meta.path` e
  costruisce la stessa `UnitKey` (path canonico, mtime ns, size, reg_mode).
- Il contratto **M-68.5 ("main can never reach this put") è SUPERATO
  DELIBERATAMENTE**: il commento a `vm/mod.rs` (~6924) e la doc del campo
  `Vm::module` (eccezione M-67.2) si riscrivono NELLO STESSO COMMIT che
  introduce il put del main; **l'audit M-67.2 va rifatto** (il main transita
  `park_module` — "unique constructor", niente secondo path — A-MS5c).
- Policy `pure` identica agli include: un main impure NON si pubblica.
  ⚠️ Un main impure (WP index.php può esserlo) dà **hit-rate 0 LEGITTIMO**:
  il controllo positivo (§10) distingue "cache rotta" da "main impure" via
  contatore di put/probe separati.

### Cosa si cacha

`CachedUnit` del main = Module + ciò che serve a `vm_new`: il main ha
bisogno anche del **Program HIR** (`main_hir`, eval-against-image). Campo
nuovo `main_program: Option<Rc<Program>>` nella entry (None per gli
include, che retano la `seed_delta` — WP-20). Il retained del main è quindi
Module+Program: la sua taglia profonda si MISURA PRIMA dell'A/B (§11,
`module_census_bytes` + walk del Program) e si dichiara ×W (cache
thread-local ⇒ retained moltiplicato per worker — KL-80-1/A-TH7).

### Dove ingaggia

- `execute_with_retain` (worker axum) — il bersaglio della roadmap;
- `run_php` del cli-server (SAPI long-lived: stessa semantica, e il braccio
  resta il gemello A/B onesto);
- il CLI one-shot (`phpr script.php`, corpus phpt) NON ingaggia il probe
  (processo single-request: un put è peso morto puro). DICHIARATO qui;
  Klabnik: non è un secondo path di compile — è lo stesso path senza probe.
  **A-TH14 (Council WP-81)**: il corpus per NOME esercita SOLO il path MISS
  — sul path HIT il giudice sono ESCLUSIVAMENTE le fixture F1-F12 + la
  battery rilanciata su HIT (§9); il probe on/off è UN parametro al confine
  SAPI, grep-gated (due call-site con logica propria = secondo path in
  attesa); pin `main_probe==0` sul one-shot (F-probe §9, A-SK12).

## 3. Chiave e invalidazione (KH80-1/KB-80-2 — mai mtime)

- Chiave = `UnitKey` esistente (canonicalize + mtime ns + size + reg_mode)
  **più il fingerprint di CONTENUTO già in uso nella unit cache** (fp match
  dentro lo slot). Per il main il fp della catena è il fp di stato VERGINE
  (tabelle pristine): **`MAIN_CHAIN_FP` COMPUTATO a init dal contenuto
  reale del prelude** (hash della forma lowered/include_str!), MAI un
  letterale hand-maintained (KS-AH-81-2: letterale ⇒ leva respinta) e MAI
  da mtime. **S-81.0 (A-AH22/KS-AH-82-2)**: fp = `main_chain_fp()` in
  lower/mod.rs — hasha il MEDESIMO binding `PRELUDE_SRC` che
  `lower_prelude_uncached` consuma (single-binding, niente copie) + tutte
  le unit prelude hoistate + reg_mode; falsificatore in-cargo
  `main_chain_fp_falsifier_mutated_prelude_changes_fp` (prelude mutato
  attraverso lo stesso seam ⇒ fp DIVERSO). **Input compile-rilevanti
  ENUMERATI (A-DS7) — scelte PINNATE S-81.0 (A-DS14, una sola per
  "oppure")**: (i) vettore ini → **assert-vuoto ESEGUIBILE** in
  `run_source_probed` (probed + ini non vuoto = panic fail-loud, mai una
  unit servita sotto direttive diverse); (ii) autoload-mid-compile → **F10
  falsifica il binding stantio**: per il MAIN l'autoload-mid-compile è
  IMPOSSIBILE per costruzione (il main lowera PRE-Vm: nessun autoload può
  scattare, nessun seed esiste) — niente fold nel fp, la fixture è il
  giudice.
- L'handler legge già `source` da disco a ogni richiesta (`fs::read` in
  main.rs): il fingerprint del contenuto è GRATIS sul path server — si
  hasha `meta.source`. Edit same-second, edit same-size, mtime-preserving
  edit: tutti invalidati dal fp. KH80-1: un HIT con contenuto su disco
  cambiato = leva RESPINTA.
- Symlink swap del docroot: canonicalize ⇒ comportamento `revalidate_path=1`
  di opcache — PINNATO dalla fixture F3 (§9), non presunto.

## 4. Ownership e pin (A-PP6.2, KS-PP-80-2)

- La cache OWNS (Rc) come oggi; il worker che fa HIT **parcheggia il clone
  nel RetainSet della richiesta** (park_module — identico agli include,
  WP-67 P-2): una eviction/supersede mid-request non può mai lasciare la
  cache unico owner del Module in esecuzione. **A-MS20 (Concilio WP-83, a
  verbale)**: il park è CINTURA — l'owner PRIMARIO mid-request è il
  CLONE-ON-STACK (`unit` vive nello stack del chiamante fino a fine
  richiesta, borrowck-enforced); sul ramo `probe=false` la safety non
  dipende dal park affatto. **KS-PP-80-2 ESTESA ALLA
  COPPIA (Module, Program) (A-PP13/KS-PP-81-2)**: il RetainSet è
  `FrozenVec<Rc<Module>>` e NON può parcheggiare `main_program` — la forma
  pinnata è il CLONE-ON-STACK dell'`Rc<Program>` per l'intera richiesta
  (mai borrow della entry); leva con Program non pinnato = reject.
- `retain.len()` atteso per richiesta: **riformulato su park-EVENTI
  (A-MS10/KS-MS-80-2 v2)** — `len == numero di park-eventi include + 1
  (main)`: un include doppio non-`_once` parcheggia DUE volte per
  costruzione (F6 dichiara il numero ESATTO atteso, §9), quindi l'invariante
  è sugli eventi, non sulle unit distinte. `len` oltre l'atteso della
  fixture = FATAL (riedizione KS-DS-78-4). **Il gate A-DS8
  (`include_units_pinned_per_request_not_leaked`) si aggiorna DICHIARANDOLO
  PER NOME**: atteso passa da `1` a `2` (lib + main) quando la leva atterra
  — mai un bump silenzioso.
- **Budget ex-ante ×W (A-MS12)**: retained atteso della cache TL per worker
  = main (Module+Program, taglia misurata in §11.1) + unit include del
  working set; con W worker il totale è ×W per costruzione (cache
  thread-local). Budget numerico dichiarato PRIMA dell'A/B nella sessione
  leva, sulla base della misura §11.1 — un retained oltre budget = HALT.

## 5. Immutabilità del Module condiviso (A-TH4-Hoare) — RISCHIO DR-1
### RISCRITTO S-81.0 (A-DS12, Council WP-82): DUE classi di stale, mai una

Il riuso cross-request di un `Rc<Module>` del main ha DUE classi di rischio
distinte — l'epoch chiude la prima, l'enumerazione qui sotto la seconda
(KS-DS-82-1: ogni claim "stessi id per costruzione" senza questa
enumerazione è NULLO):

**Classe 1 — stato IC (interior mutability): CHIUSA dall'epoch.** Le inline
cache (PropIc/MethodIc) sono le UNICHE celle mutabili del grafo Module
(DR-1, 3 occorrenze pinnate via gsub A-TH17); ogni fill embedda l'epoch
thread-local, `bump_ic_epoch()` in `Vm::new` (pin ==1) invalida in O(1) a
ogni richiesta. **Epoch a u64 dal commit leva (A-TH16/KH82-1)**: col main
cached process-lifetime il wrap u32 era diventato classe ATTIVA
(~50 giorni a 1k req/s); a u64 il wrap è macchina-impossibile (~5,8e11
anni) — residuo DR-1 ri-derivato e chiuso.

**Classe 2 — id run-scoped COTTI nei payload immutabili (A-DS12):
enumerazione per-campo.** I payload degli op del Module cached portano
questi id, tutti minted da `compile_program` (funzione PURA di
(Program, Registry, reg_mode); Program = funzione pura di (source, catena
prelude) — esattamente gli input del vergine-fp `MAIN_CHAIN_FP` + hash del
source):
- **`ClassId` negli op** (`New`/`InstanceOf`/`StaticProp`/`Const` di
  classe…): deterministico dal vergine-fp — il main compila da stato
  VERGINE, il prelude semina la testa della tabella in ordine fisso e le
  classi user seguono in ordine di hoisting del source. Gli id run-scoped
  degli INCLUDE vivono SOPRA la base (minted al run, richiesta per
  richiesta): il main cached, compilato prima di qualunque include, non può
  cuocerne uno per costruzione — è QUESTO che F13 falsifica (ordine di
  include variato tra richieste, `new`/`instanceof`/`static::` su classi di
  entrambe le lib, body == oracolo su ≥3 richieste, con controllo
  POSITIVO).
- **indici funzione nei call-site**: stessa struttura (prelude prefix +
  user in ordine source); risoluzione per NOME al link — un id cotto è
  Module-relative.
- **indici const-pool / string-pool**: Module-interni per costruzione
  (l'op indica nel pool DELLO STESSO Module riusato).
- **id celle `static $x`**: base = pstatic del prelude (costante della
  catena), user in ordine source; le tabelle runtime sono ricostruite da
  `vm_new` DAL MEDESIMO Module a ogni richiesta.
- **slot id / seed_slots**: idem — il main definisce il proprio name space,
  nessun seed esterno (link=None sul path main).
- **A-DS16 (Concilio WP-83) — enumerazione ESTESA per NOME** (KS-DS-82-1:
  enumerazione o nullità; i campi mancavano nella LETTERA, non nel merito):
  `MakeClosure{fn_idx}` (indice funzione Module-relative),
  `DeclareTrait{idx}` (Module-relative), `DeclareDeferred` /
  `NewAnonDeferred{idx}` (indice dello snippet deferred NELLO STESSO
  Module), `EnumCase{class,case}` (ClassId deterministico dal vergine-fp
  sul path main; remappato sul path include come gli altri op class-carrying
  — vm/mod.rs braccio remap). **Sede del singleton EnumCase DICHIARATA**:
  `Vm::enum_cache` (FxHashMap per-richiesta, VM-side — vm/mod.rs
  `enum_case()`): il Module non porta NESSUNA cella per i case (DR-1 = 0
  celle oltre le IC, verificato S-82.0).
Ogni campo: o "deterministico dal vergine-fp" (tutti i precedenti) o
"Module-relative" (pool indices) — nessun campo richiede remap sul path
main, ed è per questo che la entry main ha relocation vacuamente vuota.

**DR-1 ESEGUITO E CHIUSO in S-80.0.8 (verdetto MACCHINA, KH81-3)** —
`wp80-harness/gate-dr1-module-immut.sh`, PASS a git 6910767:
  - grafo **Program (hir.rs): 0 interior mutability**, pinnato (A-PP13);
  - grafo **Module (bytecode.rs): esattamente 3 siti type-position**, tutti
    l'infrastruttura IC (IC_EPOCH/PropIc/MethodIc), allowlist pinnata — un
    Cell/RefCell/atomic nuovo in uno dei due file = gate FAIL finché DR-1
    non viene ri-audito;
  - **meccanismo: le IC sono EPOCH-GUARDED** — ogni fill embedda l'epoch
    thread-local; `bump_ic_epoch()` in `Vm::new` (call-site di produzione
    pinnato ==1) invalida OGNI entry a ogni richiesta in O(1): la forma
    "reset con contatore" ammessa da KH81-3, più forte del reset-on-hit —
    i due test di invalidazione (`prop_ic/method_ic_equality_is_structural_
    only`, assert "epoch bump invalidates") girano VERDI sotto cargo (in CI
    dal S-80.0.5);
  - residui DICHIARATI: wrap u32 dell'epoch (ADVISORY, ~4,3e9 req/thread con
    cella mai refillata) e confine Zval-nei-const (invariante comportamentale
    coperta da G-APERTURA-2/gate_stateful; F7 resta il tripwire su HIT).
F7 (stale-IC cross-request, ordine classi variato) resta OBBLIGATORIA e
VERDE prima di ogni A/B — è il tripwire della classe, non la prova.
Lezione WP-28 (next_id) in vigore: id riassegnati per richiesta sono la
superficie che morde.

## 6. Esiti mai cachati (A-PP6.4)

Fatal di lower/compile e parse error NON si pubblicano MAI (oggi il
early-return li esclude già per costruzione: il put sta dopo il compile OK —
l'implementazione deve PRESERVARE questa proprietà, gate = fixture F8:
fatal → fix del file → 200 col body nuovo). **Emendamenti Pedersen (Council
WP-81)**: F8b — fatal → STESSA richiesta → 500 byte-identico (prova diretta
del "mai pubblicato", non solo del recovery); caso LINK-fatal dichiarato:
il put del main sta DOPO `link_fatal_check` (un main che compila ma
link-fatala non si pubblica) — la posizione del put rispetto al link è
PARTE del contratto, gateata da una fixture link-fatal→fix→200. Nota: i
early-return fatal DRENANO lo split census su ogni uscita dal S-80.0.3
(KS-PP-81-1) — F8 non può più inquinare la riga successiva.

## 7. Eviction e osservabilità (KS-AH-80-3, KG-80-2)

- Ways-eviction esistente per slot + supersede-per-path (S-79.0.7) +
  contatori: `entries`/`bytes_counted`/`parked_*`/`stranded_*` già nella
  riga `tag=unitcache`. La leva NON introduce strutture nuove ⇒ nessuna
  cache senza lookup (la lezione KS-DS-78-4 è strutturale: il main cached è
  LETTO a ogni richiesta — il lookup è il suo mestiere).
- **Supersede in BYTE, non solo entry (KL-81-1/A-DL6)**: contatore
  `stranded_bytes_dropped` (module_census_bytes al sito di drop, build
  census) + probe residente sul GEMELLO non strumentato (workload di edit,
  V-prima/V-dopo, floor dichiarato, R≥3) — "il supersede libera memoria"
  senza entrambi = claim respinto. Dichiarato: se la richiesta corrente ha
  parcheggiato il clone nel RetainSet il free slitta a fine richiesta
  (bounded); un ciclo Rc nel grafo Module renderebbe il drop un no-op
  (lezione WP-71) — il probe twin esiste per vederlo.
- **Filtro `reg_mode` nel supersede (A-MS9)**: la chiave supersede include
  `k.reg_mode == key.reg_mode` — oggi reg_mode è process-constant e il
  filtro è gratis, ma l'invariante non è locale. Costo scan O(entries)/put
  a chiave nuova ⇒ cold-start O(N²) su N path: MISURATO su corpus WP nel
  braccio CPU dell'A/B (A-DS9); oltre soglia dichiarata ⇒ indice per-path.
- Contatore len osservabile = `entries` (riga census); l'A/B lo cita.
- Fixture di eviction che DEVE scattare (KG-80-2): F9 — WAYS+1 varianti
  stesso path (edit) ⇒ `stranded_keys_superseded` si muove; N path distinti
  oltre il working set atteso ⇒ `entries` bounded dal supersede (per-path) e
  dichiarato: la crescita residua è per-PATH-DISTINTO (WP: centinaia — cifra
  da misurare su corpus WP nella sessione leva, budget dichiarato lì).

## 8. Riga census (probe API)

Nuovi contatori (census/parity come UcStats esistenti): `main_probe`,
`main_hit`, `main_put`, `main_impure_skip`. Riga `tag=unitcache` estesa
(bump del gate-census-twin marker count nello stesso commit).

## 9. Fixture obbligatorie PRIMA dell'A/B (A-DS6, KS-DS-80-2 — leva nulla senza queste VERDI)

| F | Fixture | Atteso |
|---|---|---|
| F1 | main modificato tra richieste (contenuto+size) | body NUOVO alla richiesta successiva, `main_put` +1 |
| F2 | edit SAME-SIZE (touch contenuto, size uguale; mtime può o meno muoversi) | MISS via fingerprint contenuto — mai stale |
| F3 | symlink swap del docroot (v1→v2) | comportamento canonicalize PINNATO (= revalidate_path 1: nuovo path canonico ⇒ MISS) |
| F4 | main con include condizionale/impure | main con `pure=false` ⇒ `main_impure_skip` +1, MAI pubblicato, body sempre corretto |
| F5 | hit-counter positivo | `main_hit` VISTO muoversi (≥1 alla richiesta 2; KG-79.A) |
| F6 | stesso file incluso 2 volte non-`_once` nella stessa richiesta | park-EVENTI pinnati (A-MS10): `retain.len()` == 2 park include + 1 main = 3 ESATTO |
| F7 | stale-IC (DR-1 §5) | body == oracolo su 3 richieste con ordine di classi variato |
| F8 | fatal → fix del file → richiesta | 500 poi 200 col body nuovo (mai fatal cachato) |
| F8b | fatal → STESSA richiesta ripetuta | 500 byte-identico alla prima (il fatal non è MAI stato pubblicato) |
| F8c | link-fatal (compile OK, link FAIL) → fix → richiesta | 500 poi 200; il put sta DOPO link_fatal_check (§6) |
| F9 | eviction/supersede (§7) | contatore visto scattare |
| F10 | early-binding (KS-DS-81-1): main `extends P` con P da lib; edit della lib | MAI il binding vecchio su HIT — stale semantico ⇒ leva RESPINTA |
| F11 | `__FILE__`/`__DIR__` (KS-DS-81-2): due spelling che canonicalizzano allo stesso file | byte-parity con oracolo su HIT — divergenza ⇒ REVERT immediato |
| F12 | `declare(strict_types=1)` nel main cached | la coercion che deve fatal-are resta identica su HIT |
| F-probe | probe-fail (A-SK12): canonicalize/stat che fallisce | esito PINNATO = MISS senza put, contatore `main_probe_fail` (§8) |
| F-oneshot | CLI one-shot (§2, A-TH14) — TRE DENTI (A-SK16/KS-SK-82-2) | t1 canale uc_log VIVO sul one-shot (reqmark presente: assente≠zero); t2 `main_probe == 0`; t3 positivo gemello: `main_probe == nreq` sull'arm server nella stessa campagna |
| F13 | ordine di include VARIATO tra richieste su HIT (A-DS13, classe 2 del §5): include condizionale A-poi-B vs B-poi-A, `new`/`instanceof`/`static::` su classi di entrambe le lib | body == ORACOLO su ≥3 richieste; controllo POSITIVO obbligatorio (variante che rompe il determinismo deve far mordere — auto-uguaglianza vietata KS-AH-80-2); body ≠ oracolo ⇒ leva RESPINTA (KS-DS-82-1) |

**Nota F4 (deviazione DICHIARATA, S-81.0)**: sul tree attuale un main
IMPURO non esiste per costruzione (il main lowera pre-Vm: niente
autoload-mid-lower, niente seed condizionale) — `main_impure_skip` è
difensivo (guard `!program.used_conditional_seed` al publish). F4 esercita
la NON-interferenza dell'include impuro (body corretto, include non
pubblicata) col pin `main_impure_skip == 0`; il wiring del contatore è
provato a livello di guard, non da una fixture positiva end-to-end.
Deviazione da giudicare al Concilio WP-83 (lezione goto-vs-Unsupported).

**Nota S-81.0 su F8c/F5/F8b/F-oneshot**: ARMATE nello stesso commit della
leva in `wp81-harness/gate-lever-fixtures.sh` (F8c in FORMA CONTATORI,
A-PP17: doppio link-fatal ⇒ put==0/hit==0 e 500/banner byte-identico; fix ⇒
put==1) — PASS al primo run. I pin strutturali (A-MS13 allowlist vm_new/
park_main, A-PP16 put-dopo-link, KS-PP-82-3 SplitDrain-primo-return,
A-TH14 probe un-parametro) sono macchina in
`wp81-harness/gate-lever-pins.sh`.

Più: gate_stateful, A-DS9 (static methods), include_gate, G-APERTURA-2 —
tutti RILANCIATI sul path HIT (KS-PP-80-3: ogni verdetto di parità su HIT è
full-body vs ORACOLO, mai HIT==MISS; auto-uguaglianza vietata KS-AH-80-2).

## 10. Predizione ex-ante dei contatori a caldo (KG-80-1; emendata S-80.0.7 su MEASURE80)

Baseline R=3 (spread 0,0%): `wp80-harness/MEASURE80_RESULTS.md`. Su
hello.php, `--workers 1`, richieste ≥11 (steady, HIT):
- `a_calls` (a1+a2): da **80.476** → **atteso < 4.000** (probe+stat+hash del
  source+bookkeeping; niente lower, niente compile). **Floor non-compile
  ex-ante dichiarato (A-BB16/KB-81-2): ≤200 call/req** (probe unit-HIT
  misurato 18,5 call/unit; bookkeeping ~41-44) — la soglia sta 20× sopra
  il floor: predizione falsificabile, non-vacua;
- **soglia KS-AH-80-4 v2 (KS-SK-81-4, UNA quantità)**: se `a_calls` steady
  non cala di **≥90%** vs baseline build-adiacente (hello: <8.048), leva
  VOID — niente credito parziale; a1 mai più sommata ad a;
- `a1_calls` = 0 a caldo (nessun lower_prelude); `a2` → assorbita nel salto;
- `a3` = 0 (invariato); `retain_len` = 1 (main) su hello, 3 su include_gate
  (2 park + main), 6 su include_heavy (5 park + main) — forma park-eventi;
- `main_hit` = 1/req a caldo; `used_n` = 0 invariato; `depth_max` = 1;
  `inflight_max` = 1; `a3_trip` = 0;
- **resid (KG-81-3)**: atteso INVARIATO ≈ 44±15 call / 43,5±5KB su hello —
  il fs::read del main resta nel resid; un leak da leva si nasconderebbe
  qui: il verdetto A-BB6 DEVE citare resid a caldo vs questa predizione;
- **b (A-BG21)**: invariato ±5% (hello 730/95.627; include_heavy
  27.982/1.867.510 — corretto da 2.798 in chiusura S-80.0, A-BG22/KG-82-1:
  il ricomputo va SCRIPTATO dai raw, mai trascritto a mano) — la leva non
  tocca il run. **A-PP21 (Concilio WP-83)**: la definizione di b è EMENDATA
  — `publish_if_armed` (put del main) cade nella finestra b della richiesta
  MISS: b non è «solo run», contiene anche il publish sulla MISS. Su HIT
  publish=None e il termine è 0; la riga MISS req=1 è il pin (A-BB30,
  sezione REQ1 di verdict81.out).
La sessione di implementazione ricalcola la baseline R≥3 con la coppia
build-adiacente PRIMA di leggere il braccio leva (KG-78.A/KB-78-4).
Confronti censuscli↔axum SOLO con la dichiarazione d'asimmetria
superglobali (A-DS10/KS-DS-81-3, vedi MEASURE80).

## 11. A/B (KL-80-1/KL-80-2/A-BB15 — coppia churn+retained, mai sola-churn)

Coppia build-adiacente stessa-sera (WP-57/65), protocollo design78 emendato:
1. **PRIMA**: misura taglia retained profonda del main cached (Module+
   Program) — dichiarata ×W. **Strumento DICHIARATO (KL-81-2/A-DL7)**:
   `module_census_bytes` è feature `mem-census`, NON census-instrumentation
   — il binario che produce la cifra DEVE avere la sua riga nel
   feature-matrix.log (config aggiunta al gate) o la cifra è NULLA. Il
   walker OGGI: (i) salta le entry Rc-shared (`strong_count>1 ⇒ 0`) — col
   main cached il prelude È shared e il put in cache ALZA strong_count:
   serve la REGOLA di ownership esplicita (cache-as-owner conta la entry
   per intero UNA volta; il walk cita la regola accanto alla cifra);
   (ii) esclude i Const string bytes per costruzione (dichiarato);
   (iii) il walk del PROGRAM non esiste — va scritto. OBBLIGO: controllo
   positivo su modulo sintetico di taglia nota prima di citare qualunque
   retained; cifra senza controllo positivo/regola Rc = NULLA, A/B VOID.
2. Churn: census R≥3 su hello + include_heavy (entrambe le fixture,
   KB-80-1), assoluti; predizioni §10 verificate (KG-80-1, incluse resid
   e b — KG-81-3).
3. Footprint: gemello NON strumentato — V2 steady `--workers 1` E
   **peak a W=num_cpus** (KL-80-2: il transitorio di primo-compile
   concorrente è ESATTAMENTE la superficie che la leva tocca; picco
   peggiorato >2% non passa in silenzio — lo spread 12% di W=10 è già
   metrica di verdetto per questa leva, A-DL4). **Floor NUMERICO accanto a
   OGNI claim (A-DL9/KL-80-3)**: no-leak sul path HIT = V2(N)==V2(2N) con
   floor vmmap 0,1MB ⇒ su 100 req floor ~1KB/req — N si scala finché
   floor ≤ 1/10 dell'effetto cercato; vale anche per le probe V2 a
   W=num_cpus (floor per-worker).
4. CPU: **slope a DUE N (100/200 req, KB-81-5/A-BB21)** — `time -l` su
   totale singolo ingloba lo startup: CPU da totale singolo = ADVISORY;
   il verdetto è la slope per-request. Nel braccio CPU si misura anche il
   costo scan del supersede (§7, A-DS9).
5. Corpus Zend per NOME + workspace suite + battery gate server COMPLETA
   (run-gate, census-twin, concurrent, worker-panic, capture-order,
   stdout-tandem, doc-purge, feature-matrix QUINTETTO + DR-1 gate).
6. Ogni cifra: identità quintetto+git ENFORCED dal driver (KS-AH-80-1,
   KG-81-2) + raw committati nel repo (KG-81-1).
7. **M-68.5/A-DS11**: il commento `vm/mod.rs:~6939` ("main can never reach
   this put") si riscrive NELLO STESSO COMMIT del put del main + pattern
   doc-purge nuovo che vieti la frase vecchia.
8. **Bound autoload-run in a3 (KB-81-3/A-BB18)**: fixture positiva
   autoload-che-alloca-al-run per quantificare la quota RUN dentro a3 —
   senza bound, "a3 = ciò che il HIT salta" resta ADVISORY.

## 12. Revert policy (KS-DS-80-3)

Un solo body ≠ oracolo su gate_stateful / include_gate / fixture F1-F9 dopo
la cache del main ⇒ **REVERT della leva, mai fix-forward**. Il revert è un
`git revert` del commit leva; la diagnosi riparte da questo design.

## Kill-switch attivi su questa leva (mappa)

KH80-1/KB-80-2 (stale hit ⇒ respinta) · KH80-2/KB-80-1/KS-SK-80-2 (split/
assoluti) · KH80-3/KL-80-1 (coppia churn+retained) · KS-AH-80-3 (eviction+len)
· KS-AH-80-4 (soglia ex-ante §10) · KS-MS-80-2 (retain len) · KS-PP-80-2
(owner unico) · KS-PP-80-3/KS-AH-80-2 (HIT vs oracolo) · KL-80-2 (peak W=n)
· KL-80-3 (floor di risoluzione nei claim no-leak) · KS-DS-80-1 (supersede)
· KS-DS-80-2 (fixture F1-F9) · KS-DS-80-3 (revert) · KG-80-1 (predizione)
· KG-80-2 (eviction che scatta) · KG-80-3 (peak senza uscita pulita).
