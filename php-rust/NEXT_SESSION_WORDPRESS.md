# Rotta WORDPRESS-FIRST — WP-track (dopo WP-40: GC buffer in-object −2,5% media, 2,68× — prossimo = volume gc_note 177M / canale drop-clone Zval)

> ⚡ **WP-40 (2026-07-23, gated `4c8de21`+`2f00d36`, 2 commit)** — **demote
> churn GC chiuso: marks in-object al posto delle strutture hash per-id =
> media −2,5% (2,68×), full −2,4%.** Eseguito il piano del handoff WP-39 in
> due step, entrambi A/B-misurati lo stesso giorno:
> **(1) `GcMark` su Object + buffer unico** (`4c8de21`): la tripla
> `gc_roots: HashMap<u32,Rc>` + `gc_queue: VecDeque` + `gc_birth: HashSet`
> sostituita da `gc_buf: Vec<Option<Rc>>` + cursore `gc_buf_head` sul Vm e
> marks nell'oggetto (`php_types::GcMark`: `pos: Cell<u32>` con MAX=assente
> + bitfield). Dedup = lettura Cell (vacant-entry semantics conservate: una
> release note non tocca il BIRTH di un entry esistente); ⭐ rimozione a
> metà buffer O(1) via slot `None` con indici stabili — il clone droppa
> NELLO STESSO ISTANTE di prima in tutti e 4 i siti (demote, unbuffer del
> candidato, consume del birth-seed nella cascata WP-28, unbuffer nel
> free-loop di collect); ⭐ `gc_buf_head` è un campo Vm, non un locale: il
> break di scheduling-destructor riprende dove era rimasto; fast-path
> empty-sweep WP-39 rimappato sul cursore; verify-mode su flag in-object
> (mismatch ⇒ il candidato si riparcheggia nel suo slot = la vecchia map).
> Esito A/B: −0,76% — il probe hashmap era solo parte del canale.
> **(2) flag-guard** (`2f00d36`): i set per-id RESTANO AUTORITATIVI, GcMark
> porta tre bit specchio esatti per gli oggetti vivi — DESTRUCTED
> sostituisce il probe hashset in gc_note (177M/run; set+flag aggiornati
> nei 4 insert e nel remove di host_reflect, path-throw incluso; unico sito
> di lettura del flag), CYCLE_ROOT/LIGHT_DEMOTED guardano gli insert
> ridondanti del demote (95% dei 47,5M ri-demota membri esistenti). ⭐⭐ La
> regola d'oro dei mirror: il flag va azzerato a OGNI svuotamento del set
> (re-seed `mem::take` per tutti i vivi anche se già bufferizzati, roots
> drain del collect via `created`, `gc_release_child`; id-reuse = flag
> morto con l'oggetto) — un flag true su set vuoto = under-insert =
> destructor perso. gc_note ora a UN borrow (insert inline).
> **Esito complessivo: old stesso-giorno 57,52 → new 56,05 user = −2,5%
> (sys −6%); oracle 20,95 ⇒ 2,68× (old = 2,75×); full run31 master-CPU
> ~11:39 (−2,4% vs 11:56)** — al tetto alto della forchetta 1,5-2,5%
> stimata dal canale (lezione WP-36 applicata). ⭐⭐ gc-census come PROVA DI
> PARITÀ del refactor: inserted/freed/demoted/collect/dtor IDENTICI alla
> baseline WP-39 in entrambi gli step (47.017.314 / 2.085.632 / 47.468.514
> / 1 / 1001; le note variano di ±62 su 177M = nondeterminismo del
> workload) — i conteggi di ingresso in sweep_impl NON sono confrontabili
> col census WP-39 (quello era pre-fast-path). **Gate/run**: sentinelle
> drop-order 9ed457b verdi PRIMA e DOPO ogni step; probe battery
> (dtor39/sent_engine/sent_builtin/probe_wp39) old==new byte-id; gate22
> TUTTO verde per nome (corpus 1447/sess 28/date 351/refl 290 IDENTICI,
> ORM 3E/13F, hk 0E/0F, gd/mysqli/media BYTE-ID, http 16 DIFF attesi,
> option/restapi identici); **run31 = run30 PER NOME** (30.472,
> 0E/2F/86W/73S, 88 righe fail identiche); cargo **1636** invariato.
> **Riprofilo (`wp40-harness/gc-out/new-wp40.sample`, stessa finestra
> t=35s)**: gc_sweep_impl **38→19**, il probe hashmap sparito; gc_note
> residuo **86** = il WALK stesso (borrow + match + discesa
> Ref/Array/Closure a strong_count==1) — ridurre il VOLUME delle note è il
> prossimo bersaglio del canale gc; dominano di nuovo drop/clone Zval
> (132+116) + memmove 108, poi resolve_prop_access 36.
> **→ PROSSIMA SESSIONE**: (a) canale drop/clone Zval + memmove (il collo
> #1 costante da WP-36 — servirà attribuzione per-chiamante prima, metodo
> WP-26/39), oppure (b) arco bytecode-a-registri (unica "leva lunga"
> approvata, cfr. verdetti Gemini post-WP-38 — census WP-33 alla mano per
> il tetto), oppure (c) volume gc_note (177M chiamate: elidere note
> provabilmente ridondanti per costruzione — es. slot già Undef, scalari —
> SEMPRE con census di parità prima/dopo). Il footprint (12,0×) resta il
> fronte non aggredito; Object +8B/istanza (GcMark) = trascurabile
> (2,5M oggetti ⇒ ~20MB teorici sul picco multi-GB).

## 📁 Convenzioni di questo file (adottate WP-40, decisione utente 2026-07-23)

- Questo handoff contiene SOLO: il blocco dell'ULTIMA sessione (il prompt
  operativo), le decisioni in vigore, lo stato gate, il backlog aperto e la
  tabella gap (cumulativa).
- **Storia per-sessione: `sessions/WP_SESSION_<n>.md`** — un file per
  sessione (oggi WP-28…WP-40; le lezioni operative di sessione viaggiano nel
  file della sessione). Per WP-27 e precedenti: memoria topic
  php-rust-wordpress-track + git history di questo file.
- **Rotazione a ogni chiusura**: la sessione N scrive
  `sessions/WP_SESSION_N.md` (blocco completo + lezioni) e sostituisce il
  blocco in testa a questo file; il blocco N−1 esce da qui (vive già nel suo
  file di sessione).

## 🧭 Decisioni in vigore (fonte citabile: migration/RULEBOOK.md)

- **Zero `unsafe` nel value core** (RULEBOOK §0; NaN-boxing bocciato WP-32,
  SSO-union rifiutata WP-38 — da NON riproporre senza cambio di rotta
  esplicito dell'utente).
- **Bytecode a registri = unica "leva lunga" approvata** (verdetti Gemini
  post-WP-38, in WP_SESSION_38.md); JIT fuori orizzonte; arena per-request
  collide con la byte-parity dei distruttori.
- Perf: **micro-bench solo advisory**; verdetti SOLO su A/B interleaved
  stesso-giorno sul workload reale (RULEBOOK §0; lezione WP-38).
- **Gate per NOME a ogni commit**; refactor layout/GC/ordine = sentinelle
  drop-order pinnate PRIMA (RULEBOOK §3); oracle-probe sempre con
  `-d log_errors=0` (WP-39).
- Commit AND push a ogni step; deviazioni deliberate nel codice = marker
  `BUG(port):` / `PERF(port):` / `TODO(port):`.

## Stato gate per nome (aggiornato WP-40)

- Gate22 WP-40 verde (wp22-harness/gate-out): corpus **1447** (baseline
  wp18-harness/gate-out/corpus.fails) · sess 28 · date 351 · refl 290
  IDENTICI · ORM 3484 3E/13F identico per nome · hk 1665 0E/0F · cargo
  **1636**/0 · probe gd/mysqli/media byte-id · http battery DIFF-set = 16
  (WP-14) · option 413 e restapi identici per nome. ⚠️ i work-tree ORM/hk in
  /private/tmp/wp11-gates possono sparire (pulizia /tmp): se "Could not open
  input file: vendor/bin/phpunit", ri-estrarre i tarball da
  wp9-harness/gates/ e ri-runnare.
- **Full-suite single-site run31 (tree ~/Claude/wpdev, trunk@5e3fced):
  30.472 test, 0E/2F/86W/73S — fail-set IDENTICO PER NOME a run30** (stessi
  2F: wpPostsListTable search_hierarchical + wp_is_stream #2 = minimo
  teorico). master-CPU **~11:39**. Confronto per nome =
  `wp16-harness/full-out/run31-fails.txt` (88 righe). Le run future si
  confrontano con run31.
- **Full-suite multisite (WP-28): 1 diff per nome — minimo teorico**
  (31.278 test; solo `wp_is_stream #2`;
  `wp19-harness/ms-out/diff-names-wp28.txt`).
- Suite phpt estensioni (misura): **xsl 63/64** (⚠️ da CWD = root php-8.5.7) ·
  tidy 44/45 · asymmetric_visibility **38/39**. Suite phpt SEMPRE con path
  ASSOLUTO.

## Harness full-suite (WP-16 — invariato)
```bash
H="/Volumes/Extreme Pro/Claude/wp16-harness"
"$H/run-full-detached.sh" phpr   # lanciarlo con un daemonizer perl (double-fork
                                 # + setpgrp) da un task BACKGROUND: il task-kill
                                 # a 10' non deve raggiungere la run
# ⚠️ MAI due gate22 insieme; MAI probe su wptests durante una run;
#   azzerare wpdev/src/wp-content/uploads prima di ogni full run;
#   non ricompilare mentre una run/gate usa il binario.
# multisite: wp19-harness/run-multisite-detached.sh <oracle|phpr> (ms-out/;
#   marker ms-phpr.done)
```

## 🎯 PROSSIMO LAVORO (riprofilo WP-40 ∩ direttive Gemini 23/07)

1. **Warm-up: frontend `gc_note` (Leva C Gemini)** — shim
   `#[inline(always)]` con guardia sul discriminante
   (`Object|Ref|Array|Closure` → slow path out-of-line `gc_note_slow`),
   così i ~60 call-site (loop di gc_note_frame su slots/stack di ogni
   frame che ritorna, overwrite di slot, displaced degli array) pagano un
   confronto inline invece di call+match per i 177M/run. ⭐ il hook
   `gc_census::note()` resta NEL shim (il contatore `notes` deve contare
   TUTTE le chiamate o il confronto census cross-versione si rompe).
   **Tetto dichiarato: ~1-1,5%** (gc_note self = 86 campioni/10s ≈ 2,9%,
   e include walk vero — lezione WP-36). Census di parità prima/dopo.
2. **Canale drop/clone Zval + memmove (Leva A)** — collo phpr-only #1
   costante da WP-36 (132+116+108 campioni/10s): attribuzione
   per-chiamante PRIMA (metodo WP-26/39), poi la leva.
3. **Arco bytecode-a-registri (Leva B)** — leva lunga multi-sessione
   (compiler + run_loop + unit-cache format + riscrittura delle fusioni
   stack-based WP-33/34); census WP-33 alla mano per stimare il tetto
   PRIMA di aprire (lezione WP-36). SOLO quando A e C sono esaurite
   (verdetto condiviso, vedi sotto).
4. **Validazione Laravel** ([[php-rust-roadmap-wp-first]]) quando si decide
   di chiudere l'arco perf.
   Il footprint (12,0×) resta il fronte non aggredito.

## 📨 Direttive Gemini post-WP-40 (`20260723_gemini.md`) — verdetti (verificati sul codice 2026-07-23)

- **✅ §1 Leva C (frontend gc_note) — ACCOLTA, è il warm-up della prossima
  sessione** (punto 1 sopra). Verifica sul codice: gli scalari cadono GIÀ
  in `_ => {}` (e `Str` non è nel match — verdetto WP-30 §4 confermato),
  ma la funzione è grossa (match ricorsivo) e NON viene inlinata: ai
  177M call si paga call+match. Lo shim discriminante-only è esatto per
  costruzione. ⚠️ CORREZIONE DI MIRA sulla parte (b) "elisione a
  compile-time": ridondante col shim — la nota di uno slot
  provabilmente-Undef si riduce già a un confronto inline; un pass del
  compilatore aggiungerebbe complessità per ~nulla. Riconsiderare solo se
  il census post-shim mostra residuo concentrato su siti elidibili.
- **✅ §2 Leva A (churn Zval) — CONCORDANTE con correzione**: la domanda
  CoW è legittima ma il design è già corretto — `Zval::Array(Rc<PhpArray>)`
  + `Rc::make_mut`: il passaggio by-value costa un bump di refcount, MAI
  deep-clone su lettura passiva (= zend refcount++). Il churn misurato È
  il traffico bump/drop del modello a stack; le "reference temporanee per
  argomenti read-only" richiedono plumbing che di fatto coincide con
  l'arco a registri. Resta valido il punto condiviso: attribuzione
  per-chiamante PRIMA di qualunque intervento.
- **✅ §3 Leva B (registri) — CONCORDANTE** col verdetto già in vigore
  (post-WP-38): unica leva lunga approvata, da aprire SOLO ad A+C esaurite;
  l'avvertenza "a cuore aperto" (pass compiler + run_loop + fusioni da
  riscrivere) coincide con la nostra stima multi-sessione.

## Backlog aperto (non legato a una sessione)

- Residui strutturali estensioni: `ast_printing.phpt` (serve un vero
  zend_ast_export sull'HIR); xsl `bug69168` (i nodi passati a php:function
  devono ALIASARE il doc live); tidy `010` (ordine free nel caso
  var_dump-di-albero: le over-note del dump inquinano il FIFO).
- Ordine destructor per oggetti CON `__destruct` nel subtree (Ret-hook usa
  ancora gc_cascade, non gc_release_cascade) — nessun test lo copre oggi.
- Verbo "increment/decrement" per `$null->p++` (oggi "assign") — threading
  del verbo nel funnel FieldIncDec.
- Se si toccano date/prelude DateTime: gate ext/date OBBLIGATORIO (351).

## 📊 REPORT GAP PERF ORACLE↔PHPR — ATTIVITÀ RICORRENTE DI FINE SESSIONE
A OGNI chiusura di sessione, prima del commit finale di memoria/handoff,
misurare e riportare all'utente il gap aggiornato e aggiornare la tabella
(⚠️ confrontare RAPPORTI, mai i tempi assoluti di giornate diverse):
1. **Media group**: oracle 1 run `/usr/bin/time -l` (DB reset + uploads
   azzerati, MIMALLOC_PURGE_DELAY=0) vs phpr → rapporto **user CPU** e
   **peak footprint**.
2. **Full-suite**: CPU del processo master phpr dal tail del `.rss` della
   runN di sessione vs oracle (baseline 5:39) → rapporto; wall indicativo.

| sessione | media CPU (phpr/oracle) | media footprint | full-suite master-CPU | full-suite wall |
|---|---|---|---|---|
| WP-26 (baseline) | 85,8/21,0 = **4,1×** | 5,0/0,4GB = **12,7×** | (wall, non comparabile) | ~1,9× |
| WP-27 | 82,7/21,1 = **3,9×** | 4,78/0,40GB = **12,0×** | 16:11/5:39 = **2,9×** | ~22/11,5 min = **1,9×** |
| WP-28 | 87,6/23,0 = **3,8×** | 4,83/0,40GB = **12,2×** | 16:43/5:39 = **3,0×** | ~22/11,5 min = **1,9×** |
| WP-29 | 82,4/23,0 = **3,6×** | 4,84/0,40GB = **12,1×** | 15:27/5:39 = **2,7×** | ~22/11,5 min = **1,9×** |
| WP-30 | 80,7/21,0 = **3,8×** ⚠️ | 4,80/0,40GB = **12,1×** | 15:12/5:39 = **2,7×** | ~20/11,5 min = **1,7×** |
| WP-31 | 72,4/20,95 = **3,5×** | 4,82/0,40GB = **12,1×** | 13:02/5:39 = **2,3×** | ~17,5/11,5 min = **1,5×** |
| WP-32 | 69,0/20,91 = **3,3×** | 4,75/0,39GB = **12,0×** | 12:54/5:39 = **2,3×** | ~19,5/11,5 min = **1,7×** |
| WP-33 | 66,9/20,97 = **3,19×** | 4,75/0,39GB = **12,0×** | 12:20/5:39 = **2,18×** | ~16,5/11,5 min = **1,4×** |
| WP-34 | 65,1/20,92 = **3,11×** | 4,73/0,39GB = **12,0×** | ~12:35/5:39 = **2,2×** (rumore) | ~17,5/11,5 min = **1,5×** |
| WP-35 | 59,6/20,99 = **2,84×** ⭐ | 4,73/0,39GB = **12,0×** | ~12:05/5:39 = **2,14×** | ~17/11,5 min = **1,5×** |
| WP-36 | 61,4/21,06 = **2,92×** ⚠️ | 4,78/0,40GB = **12,1×** | ~12:05/5:39 = **2,14×** | ~17/11,5 min = **1,5×** |
| WP-37 | 60,07/20,94 = **2,87×** | 4,72/0,39GB = **12,0×** | ~12:30/5:39 = **2,2×** (rumore) | ~17/11,5 min = **1,5×** |
| WP-38 | 59,75/20,955 = **2,85×** (SSO revertato: neutro) | 4,72/0,39GB = **12,0×** (invariato) | ~12:29/5:39 = **2,2×** | ~17/11,5 min = **1,5×** |
| WP-39 | 56,79/20,93 = **2,71×** ⭐ (fast-shutdown + sweep fast-path) | 4,20/0,435GB = **9,7×** ⚠️ maxrss stesso-giorno (old 8,9×; il +9% new = accounting MADV_FREE, picco reale identico — caveat WP-20) | 11:56/5:39 = **2,11×** | ~17,4/11,5 min = **1,5×** |
| WP-40 | 56,05/20,95 = **2,68×** ⭐ (GC marks in-object; old stesso-giorno 57,52 = 2,75×) | non rimisurato (maxrss MADV-inquinato; Object +8B/istanza ≈ +20MB teorici su picco multi-GB) | ~11:39/5:39 = **2,06×** | ~16,6/11,5 min = **1,4×** |

⚠️ riga WP-36: NON è una regressione — l'old-binary (WP-35) rimisurato lo
STESSO giorno dà 61,1s (2,90×): la giornata di WP-35 era favorevole; il
confronto interleaved new/old dà phpr −0,5/−1% (rumore/flat).

⚠️ riga WP-30: phpr media in calo ASSOLUTO (82,4→80,7) ma l'oracle del giorno
gira −9% (23,0→21,0) → il rapporto sale per rumore dell'oracle, non per una
regressione phpr (2 coppie consistenti: 80,42/21,03 e 80,97/21,02).

## Invarianti (aggiornati WP-40)

- Gate per OGNI commit: corpus/sess/date/refl per NOME — baseline:
  **corpus 1447** · sess 28 · date 351 · refl 290 (SOLO rimozioni ammesse;
  fail-set in `wp18-harness/gate-out/*.fails`) · ORM 3484 3E/13F per nome ·
  http-kernel 1665 0E/0F · cargo **1636** · probe: gd 11/11, mysqli 11/11,
  media-probe byte-id, run-http (DIFF-set 16 = WP-14) · WP suite per-classe =
  oracle (option 413 · restapi · classi WP-17/18). Script:
  `wp22-harness/gate22.sh` (col daemonizer perl; ~19 min).
- Full-suite single-site: solo miglioramenti per nome vs **run31** (88
  righe, 2F = minimo teorico). Multisite: vs **ms-out WP-28** (1 diff).
- Run pesanti SEQUENZIALI, sotto watchdog o daemonizer, marker .done su
  disco; MAI due gate22 insieme; uploads azzerati prima di ogni full run;
  non ricompilare mentre una run/gate usa il binario; Serena per Rust,
  Vexp/Read per il C, Read/Write per i .php; log `tr -d '\0'`.
