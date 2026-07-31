# design79.md — Leva A-BB6: unit cache TL estesa al MAIN (S-79.0.8, vincoli Council WP-80)

**Stato**: DESIGN scritto in S-79.0 DOPO il pre-lever hardening (punti 1-7,
commit ff22a2e…5966d56) e DOPO lo split a1/a2/a3 (KH80-2/KB-80-1/KS-SK-80-2:
un design fondato sul "99%" aggregato è respinto d'ufficio — qui si cita il
canale decomposto). Nessuna riga di implementazione della leva è stata
scritta: questo file è il contratto che la sessione di implementazione
esegue e che il Concilio giudica.

## 1. Canale (dati decomposti, ASSOLUTI)

Census split (S-79.0.3, build census `census-cli` smoke su include_gate.php,
binario 37f630d6, R=1 — la misura di riferimento R≥3 su entrambe le fixture
è il PRIMO atto della fase misura, prima dell'A/B):

- finestra engine totale: **83.834 call / 13,36MB** per richiesta
- **a1 (prelude lower+compile): 74.288 call / 10,83MB (~81%)**
- a3 (include-compile, COLD): 2.156 call / 198KB → **0 su HIT** (controllo
  positivo in-cargo `census_split_a1_prelude_and_a3_include_discriminate`)
- a2 (main proper, derivato): ~7.400 call / ~2,3MB

Il Concilio aveva ragione: il "99% fase a" era per ~4/5 PRELUDE. MA la forma
fedele della leva (§2) non cacha "il main script" da solo: cacha la UNIT
MAIN, che per costruzione CONTIENE il prelude compilato (lower_prelude +
prefisso prelude di compile_program avvengono DENTRO la compilazione della
unit main). Un HIT salta quindi **a1+a2 insieme** (~13,1MB/req su
include_gate, ~13,0MB su hello). La leva resta dimensionata sugli ASSOLUTI
qui sopra, mai su percentuali (KB-80-1).

Cifre churn: CountingAlloc conta `realloc` a taglia piena e non netta i
dealloc — ogni cifra è **churn LORDO, upper bound** (A-DL1/A-AH11,
dichiarato in design78 §Identità).

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
  Klabnik: non è un secondo path di compile — è lo stesso path senza probe,
  gate: il corpus per NOME resta il giudice.

## 3. Chiave e invalidazione (KH80-1/KB-80-2 — mai mtime)

- Chiave = `UnitKey` esistente (canonicalize + mtime ns + size + reg_mode)
  **più il fingerprint di CONTENUTO già in uso nella unit cache** (fp match
  dentro lo slot). Per il main il fp della catena è il fp di stato VERGINE
  (tabelle pristine): committare un `MAIN_CHAIN_FP` costante derivato da
  reg_mode+prelude, MAI da mtime.
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
  cache unico owner del Module in esecuzione. KS-PP-80-2 arma il reject.
- `retain.len()` atteso per richiesta = unit distinte incluse + **1 (main)**
  sul path HIT e MISS. KS-MS-80-2: `len` oltre = FATAL (riedizione
  KS-DS-78-4). **Il gate A-DS8 (`include_units_pinned_per_request_not_leaked`)
  si aggiorna DICHIARANDOLO PER NOME** in questo design: atteso passa da
  `1` a `2` (lib + main) quando la leva atterra — mai un bump silenzioso.

## 5. Immutabilità del Module condiviso (A-TH4-Hoare) — RISCHIO DR-1

Il riuso cross-request di un `Rc<Module>` del main esige che Module sia
privo di interior mutability osservabile. Gli include lo fanno GIÀ oggi
(unit cache) — ma passano per remap/relocation; il main definisce lo spazio
base e ricompila deterministico, quindi gli id coincidono per costruzione.
Superficie vergine: **le inline cache (PropIc/MethodIc, WP-29+)** se vivono
dentro Func/Module con `Cell`.

**Primo atto dell'implementazione (bloccante)**: audit del grafo raggiungibile
da `Module` per interior mutability (Serena: Cell/RefCell/atomic nei tipi di
`Func`/`CompiledClass`). Esiti ammessi:
  (a) nessuna ⇒ `assert_not_impl_any!`-style static assert o test di
      immutabilità, e si procede;
  (b) IC presenti ⇒ o reset-on-hit, o prova di determinismo degli id
      (fixture F7 stale-IC: accesso polimorfo cross-request con classi
      dichiarate in ordine diverso) — VERDE prima di ogni A/B.
Lezione WP-28 (next_id) in vigore: id riassegnati per richiesta sono la
superficie che morde.

## 6. Esiti mai cachati (A-PP6.4)

Fatal di lower/compile e parse error NON si pubblicano MAI (oggi il
early-return li esclude già per costruzione: il put sta dopo il compile OK —
l'implementazione deve PRESERVARE questa proprietà, gate = fixture F8:
fatal → fix del file → 200 col body nuovo).

## 7. Eviction e osservabilità (KS-AH-80-3, KG-80-2)

- Ways-eviction esistente per slot + supersede-per-path (S-79.0.7) +
  contatori: `entries`/`bytes_counted`/`parked_*`/`stranded_*` già nella
  riga `tag=unitcache`. La leva NON introduce strutture nuove ⇒ nessuna
  cache senza lookup (la lezione KS-DS-78-4 è strutturale: il main cached è
  LETTO a ogni richiesta — il lookup è il suo mestiere).
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
| F6 | stesso file incluso 2 volte non-`_once` nella stessa richiesta | parking duplicato PINNATO (retain.len() esatto dichiarato) |
| F7 | stale-IC (DR-1 §5) | body == oracolo su 3 richieste con ordine di classi variato |
| F8 | fatal → fix del file → richiesta | 500 poi 200 col body nuovo (mai fatal cachato) |
| F9 | eviction/supersede (§7) | contatore visto scattare |

Più: gate_stateful, A-DS9 (static methods), include_gate, G-APERTURA-2 —
tutti RILANCIATI sul path HIT (KS-PP-80-3: ogni verdetto di parità su HIT è
full-body vs ORACOLO, mai HIT==MISS; auto-uguaglianza vietata KS-AH-80-2).

## 10. Predizione ex-ante dei contatori a caldo (KG-80-1 — dichiarata QUI)

Su hello.php, `--workers 1`, richieste ≥11 (steady, HIT):
- `a_calls` (a1+a2): da ~80.400 → **atteso < 4.000** (probe+stat+hash del
  source+bookkeeping; niente lower, niente compile);
- **soglia KS-AH-80-4**: se `a_calls+a1_calls` steady non cala di **≥90%**
  vs baseline build-adiacente, leva VOID — niente credito parziale;
- `a1_calls` = 0 a caldo (nessun lower_prelude);
- `a3` = 0 (invariato); `retain_len` = 1 (main) su hello, 2 su include_gate;
- `main_hit` = 1/req a caldo; `used_n` = 0 invariato; `depth_max` = 1.
Su include_heavy.php: stesso pattern, `retain_len` = 6 (main+5 lib).
La sessione di implementazione ricalcola la baseline R≥3 con la coppia
build-adiacente PRIMA di leggere il braccio leva (KG-78.A/KB-78-4).

## 11. A/B (KL-80-1/KL-80-2/A-BB15 — coppia churn+retained, mai sola-churn)

Coppia build-adiacente stessa-sera (WP-57/65), protocollo design78 emendato:
1. **PRIMA**: misura taglia retained profonda del main cached (Module+
   Program, contatore dedicato con controllo positivo) — dichiarata ×W.
2. Churn: census R≥3 su hello + include_heavy (entrambe le fixture,
   KB-80-1), assoluti; predizioni §10 verificate (KG-80-1).
3. Footprint: gemello NON strumentato — V2 steady `--workers 1` E
   **peak a W=num_cpus** (KL-80-2: il transitorio di primo-compile
   concorrente è ESATTAMENTE la superficie che la leva tocca; picco
   peggiorato >2% non passa in silenzio — lo spread 12% di W=10 è già
   metrica di verdetto per questa leva, A-DL4).
4. CPU: `/usr/bin/time -l` (A-BB15) — la leva è alloc-motivata, l'ipotesi
   CPU si verifica qui, non si presume.
5. Corpus Zend per NOME + workspace suite + battery gate server COMPLETA
   (run-gate, census-twin, concurrent, worker-panic, capture-order,
   stdout-tandem, doc-purge, feature-matrix quaterna).
6. Ogni cifra: quaterna d'identità ENFORCED dal driver (KS-AH-80-1).

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
