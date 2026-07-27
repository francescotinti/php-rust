# WP_SESSION_66 — debiti concilio WP-65 consegnati + fronte server ISTRUITO su SAPI -S (KS-P1: parità di risposta byte-id, bordi R3/R4 scattati ⇒ P-2 BLOCCO; SCOPERTA leak 15 unit impure/richiesta) + L-66.1 (65b anomalo; Σcommitted metro di lavoro)

> ⚖️ Verbale EMENDATO alla chiusura dal concilio WP-67 (sintesi in
> `wp67-harness/COUNCIL_WP67_REVIEWS.md`): (i) bug di UNITÀ in L-66.1
> corretto (L-67.1: 65b = 1.807,9 MiB phys / 121,8 fuori-bin; accusa
> al recount RITIRATA — L-67.2); (ii) KK67-1: il probe KS-P1 ebbe
> verdetto macchina FAIL (bordi P66-R3/R4) — la testa non dice più
> "verde"; il letto fp-seq steady-state è POST-HOC (da pre-registrare
> nel probe, K-67.6); (iii) ogni claim KS-P1 porta "SAPI -S
> sequenziale, worker unico" (il fronte axum è ISTRUITO, non aperto);
> (iv) "65b outlier" = ipotesi-forte (G-67.4); (v) claim phys
> standing SOSPESI (KL-66.1 non soddisfatto per-causa).

> ⚡ **WP-66 (2026-07-27 sera, `c3c9e38`→`b1e9466`→`fe33706`→`131f7c0`)**
> — sintesi a 9 recepita INTEGRALE in `wp66-harness/design66.md` PRIMA
> di ogni codice; predizioni P66-R/S/L in `predictions66.md` (+locked +
> copia repo `sessions/WP66_PREDICTIONS.md`, commit 131f7c0) PRIMA dei
> letti — primo snapshot=COPIA della serie (K-66.1).

## Debiti d'apertura consegnati

- **Codice (`b1e9466`)**: H-66.1/S-66.3 contratto `seed_slots ≤
  seed.len()` DENTRO i due helper; M-66.1/P-66.2 `seed_prefix_short` =
  evento nel vocabolario chiuso (10) + contatore UcStats in
  tag=unitcache + **breach FATAL in ogni build** (disciplina K-M5/H2'');
  H-66.2 get_defined_vars a pairing esplicito `(i, name)` (desync
  filter_map/enumerate ELIMINATO); M-66.2 tripwire tail∩seed=∅ alla
  costruzione del Func eliso (oggi vacuo per costruzione — dichiarato).
  cargo 1646/0.
- **Colonne census (`fe33706`, census-only)**: `lower_partial_ns/
  lower_partials` in tag=compilens (drop-guard sui lower che errano
  mid-pass) — B-66.x, il residuo 10,4% ora si CONTA; `tag=lcpath`
  (top-40 dup, `dup_lc=(l+c)·(n−1)/n`) + `tag=lcsum dup_lc_ns` —
  E-66.2, la ri-quota E6 è sbloccabile sul MISURATO (KS66-1
  rispettato: nessuna cifra promossa in sessione).
- **S-66.1**: `wp66-harness/sem-baseline/sem-oracle.diff.expected`
  PINNATO con PROVENANCE (fonte sem-out gate65, binario 778f8ead) —
  il gate lo asserisce IDENTICO. S-66.2 verificata già applicata in
  `c3c9e38`. P-66.4 grep display-only ancorato.
- **GATE66 = primo gate a VERDETTO MACCHINA (K-66.3/K-66.4/KK66-3)**:
  contatore FAIL, done-marker SOLO a zero ✗, baseline mancante=FAIL,
  hk/reverse ASSERITI sui conteggi. **PASS fails=0**: cargo 1646/0 ·
  sentinelle 5 assi BYTE-ID · sentinels65+KS-S6+`seed_prefix_short=0`
  · KE-e · P1 hit_cross=2 · S-65.3 + diff oracle == pinnato · corpus
  1421 · refl 290 · ORM 3E/13F · hk 1665 OK · reverse 2F.

## Fronte axum — KS-P1 replay wpdev (probe66-ksp1.sh, N=10) + S-66.4

- **Parità server VERDE**: body R2/R3/R10 == R1 **BYTE-ID** senza
  normalizzazioni; hit_cross R2=493 (U1=783); steady-state R3..R10:
  fp-seq **IDENTICHE** (512 load), hit-rate 97,1%; `seed_prefix_short=0`
  (il tripwire ha girato sul workload server); miss_dc=0 anche sul
  server (canale dc VUOTO).
- **P66-R3 falsificata PER REFERENTE (classe P65-A)**: R1≢R2 sul
  load-SET (WP carica +300 controller rest-api solo in R2, transitorio
  DB cold→warm) — il referente giusto è steady-state, e lì la fp-seq
  è byte-stabile. K-M66.3/KH66-1 non scattano (attribuito al workload).
- **🔴 LA SCOPERTA (P66-R4, bordo KB66-3 scattato)**: **15 miss-cold
  PER RICHIESTA, PER SEMPRE** (R3=R6=R10; lista stabile:
  script-loader.php (elide 242!), pomo/*, Requests/*, ai-client/*,
  comment.php, block-template.php…). Causa nel codice:
  `lower_unit` marca `*pure=false` a OGNI retry autoload-in-lowering
  (mod.rs:5617) e la publish in cache richiede `pure` (mod.rs:6172) ⇒
  le unit che autoloadano in lowering **non entrano MAI in cache: 15
  compile + 15 `Box::leak(Module)` a OGNI richiesta = leak
  O(richieste)**. KS-P66.3 materializzato: **P-2 è BLOCCO** (nessuna
  banda axum citabile prima); è anche il letto server di M-65.2 in
  forma publish-skip (il ceiling autoload non è dc-miss: è
  never-published).
- **S-66.4 PASS (0 ✗)**: NESSUN bleed cross-request ($var, $GLOBALS
  espliciti, `global`) — KS-S66.1 NON scatta sul SAPI sequenziale;
  template incluso rigiocato byte-id su VM fresca (P66-S3 = il punto
  critico della leva v2 sotto server, verificato); $_GET isolata.
  (iv) session / (vi) cross-worker: N/A oggi, dichiarato.
- **M-66.4**: mini-design P-2 in `wp66-harness/design66-p2.md`: cache
  Rc-owned (superseded/evicted DROPPANO) + RetainSet per-richiesta
  (arena append-only di Rc), hot path INTATTO, eval de-leakate; punto
  duro dichiarato (arena safe ⇒ crate vetted tipo elsa/typed-arena —
  decisione al concilio, KH66-2); sentinelle drop-order e criteri
  d'accettazione pre-registrati.

## L-66.1 fuori-bin (census66-l661.sh su memgc65b INVARIATO, run2)
## ⚠️ EMENDATA DAL CONCILIO WP-67 (L-67.1/L-67.2: bug di unità)

- Metrica riproducibile (phys mi_proc win=0 − Σ mi_bin win=0 dump-1),
  **tutte le cifre in MiB** (la prima stesura mischiava MB decimali):
  **census65 37,4/97,9% · census65b 121,8/93,3% (phys 1.807,9) ·
  l661 33,9/98,0%** ⇒ **census65b resta ANOMALO (ipotesi-forte
  "outlier di run", G-67.4: covariata pressione non controllata —
  ≈290MB swapped a metà l661)**; spread phys a binario invariato
  **89,2 MiB (~5%)**.
- **Σcommitted è il metro di lavoro**: 1.686,1 (65b) vs 1.684,8
  (l661) = 1,3 MiB (0,08%) su due run; la leva slot_names vi replica
  (−58MB da census65); lo standing "stabile alla cifra" si promuove
  alla TERZA run (KL67-1) — L-66.2 confermata (phys del checkpoint
  MAI da solo).
- ⚖️ RITIRATA l'accusa "recount del concilio non riproducibile": il
  concilio contava in MB decimali (Σ65b=1.768,0 = 1.686,1 MiB;
  fuori-bin 127,7 dec = 121,8 MiB) — contabilità coerente; il bug di
  unità era della sessione (L-67.2: ogni cifra dichiara l'unità).
- L'attribuzione vmmap (dirty+swap vs footprint, copertura "199%") è
  VACUA per unità miste ⇒ **KL-66.1 resta NON soddisfatto in forma
  per-causa: i claim phys standing restano SOSPESI** (solo
  counted+committed) finché L-67.3 non consegna la colonna per-causa
  lato mimalloc (segment-committed − bin-committed, ≤100% per
  costruzione).

## Parità e stash

- Binario di sessione **phpr-wp66 (5aa60d56…, tree `fe33706`)**,
  stash ADDITIVO accanto a wp65. Delta vs wp65: tripwire
  seed_prefix_short (ramo mai preso, provato =0 su gate+server) + fix
  H-66.2 + colonne census-only.
- **Full-suite di parità (run new-binary, stessa sera)**: 30.472 test,
  0E/2F/86W/73S, fail-set **88 BYTE-ID = run33** ⇒ phpr-wp66 è
  baseline di parità PIENA (gate66 matrice + full). Run singola,
  nessun claim CPU/peak (niente coppia — B-66.1/G-66.1).

## ⭐ Lezioni

- ⭐⭐ **La cache-publish è condizionata a `pure`** e l'autoload-in-
  lowering la nega: sul server il costo del "ceiling autoload" non è
  un miss ma un NEVER-PUBLISHED (15 unit/richiesta su wpdev) — i
  contatori di miss non lo vedono, si vede SOLO dal confronto
  fp-identico-eppure-cold.
- ⭐⭐ **Il referente di un confronto per-richiesta è lo steady-state**:
  R1/R2 su WP differiscono nel load-SET per stato DB (rest-api +300),
  non per l'engine — pre-registrare fp-seq su R_n/R_{n+1} con n≥3.
- ⭐⭐ **Σcommitted per-bin è stabile alla cifra tra run; il phys
  standing no** (−177MB stesso binario): ogni claim phys standing
  passa dal committed (L-66.2 ora ha la prova sperimentale).
- ⭐ `pgrep -f` matcha anche il wrapper `/usr/bin/time` (la cmdline
  embedda il comando): il supervisor vmmap di run1 ha fotografato il
  wrapper — risolvere il FIGLIO del wrapper.
- ⭐ Redirect di un launcher detached: la out-dir deve esistere PRIMA
  della shell che apre il file (replica della lezione daemonize).

## Prossimo (WP-67) — vedi NEXT_SESSION §WP-67
