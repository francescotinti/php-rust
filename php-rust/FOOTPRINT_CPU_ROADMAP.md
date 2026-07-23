# Piano "Concilio" — abbattere il 12× footprint e riaprire il fronte CPU di phpr

## Context

L'utente non accetta la chiusura del fronte perf dopo WP-44. Richiesta: un
modello da "concilio di sviluppatori" (Klabnik/Pedersen/Hoare/Matsakis/
Hejlsberg/Bak) per abbattere il **footprint 12×** vs oracle (4,7-4,8GB vs
0,4GB sul gruppo media WP) e migliorare CPU (2,66×). Fatto chiave: il
fronte footprint **non è mai stato aggredito** (40+ sessioni tutte sul
CPU), e due ricognizioni fresche (2026-07-23) hanno trovato canali di
ritenzione mai contati e due leve CPU compatibili con la fisica WP-44.

**Decisioni utente vincolanti**: footprint-first · safe-only (nessun
emendamento RULEBOOK; crate dipendenti auditati ammessi, unsafe NOSTRO no)
· **TUTTE le fasi si eseguono comunque** (direttiva utente 2026-07-24):
un risultato intermedio negativo NON chiude la roadmap e — rinforzo
esplicito dell'utente — **NIENTE REVERT in caso di insuccesso: si
prosegue comunque**. Interpretazione operativa (trasparente):
- una regressione PERF misurata dall'A/B si TIENE, si verbalizza nel
  gap report (ratchet documenta, non blocca) e si passa oltre;
- una rottura di PARITÀ (fail-set per nome) non si abbandona né si
  reverta: si FIXA in avanti finché il gate torna verde;
- i "gate"/checkpoint decidono il COME (quale tipo pilotare, quale leva
  prima), mai il SE.
Questa direttiva SUPERA, per le sessioni di questa roadmap, la legge
storica revert-su-regressione (WP-38/41/44).

**Legge WP-44 (falsificata 18/18 round, tre forme)**: aggiungere corpi
handler caldi al run_loop costa ~+1% sempre. Le leve CPU ammesse: ridurre
lavoro DENTRO corpi esistenti, elidere op a emit-time, ridurre alloc-rate.

## Fatti verificati dalle ricognizioni (file:line nei report di sessione)

**Footprint** (recon A):
1. **Unit compilate**: ogni include/require/eval compila un Module e lo
   `Box::leak`a (`vm/mod.rs:3331`); accumulo monotono per-Vm (phpunit =
   UN processo). Nessuno `shrink_to_fit` nel codebase; `Func` ritiene
   ops 48B/op + lines 4B/op + ~10 array metadati per-param; **seed HIR
   ritenuti accanto al bytecode**. ⚠️ Da verificare col contatore: se un
   include non-`_once` (template WP!) ricompila+ritiene a OGNI esecuzione
   è un leak, non un canale.
2. **Object**: ~208B fissi su 2 alloc PRIMA delle props; **~96B = header
   di 3 Vec quasi-sempre-vuoti** (`readonly_init`, `readonly_clone_writable`,
   `typed_unset`) + `dyn_entries` (`object.rs:19-72`). Pattern di cura già
   in-codebase: cold-box `Option<Box<FrameExt>>` (WP-32).
3. **Hashed array** (forma dominante in WP): ~48-56B/el — entries 32B/el
   + `index: FxHashMap<Key,u32>` con **Key duplicata** (`array.rs:96-103`).
   Packed = 16B/el.
4. **`created: BTreeMap<u32, Rc<RefCell<Object>>>`** (`vm/mod.rs:1425`):
   ref FORTE a ogni oggetto creato fino a destruct/shutdown — pinna
   transitivamente array e stringhe.
5. **Stringhe**: 2 allocazioni ciascuna (~40B overhead fisso), 51,8M/run
   (`zstr.rs:17-24`). Zend: 1 allocazione. (SSO bocciato ≠ fusione
   header+bytes, mai provata.)
6. mimalloc a default (zero tuning in-code); vmmap Physical footprint =
   unico oracolo di picco (maxrss mente, MADV_FREE).

**CPU** (recon B):
- `DerefTop` emesso INCONDIZIONATAMENTE dopo ogni method/static/dyn call
  a valore (`compile/expr.rs:498/524/548`) — no-op a runtime salvo callee
  by-ref ⇒ **40,5M dispatch sprecati**. Cura: flag `RET_DEREF` GIÀ
  esistente sul frame (`run.rs:2676,2720`), settato a emit-time, deref
  gated su `func.by_ref` dentro il corpo Ret esistente. Zero arm nuovi.
- Corpo `Op::Ret` (62,6M/run) legge 4 flag + guard + hint + by_ref a ogni
  ritorno: un `ret_shape: u8` precalcolato su Func riduce a un branch.
- `Op::Sweep` emesso dopo OGNI statement (`compile/mod.rs:677`): 53M
  dispatch, ~47M noop — elidibile a emit-time per statement che non
  allocano container.
- Per-call: un `Vec` args per method/static call (`pop_keys` split_off).

## Il modello del concilio (sintesi delle lenti)

- **Bak** (V8/HotSpot): l'alloc-RATE è una metrica CPU di prima classe
  (51,8M stringhe × 2 malloc = esecuzione malloc-bound); purge/arena
  discipline prima dei redesign; shapes già presenti (PropsLayout/IC).
- **Hejlsberg**: interning delle stringhe duplicate (nomi hook/option WP)
  e lavoro spostato nel COMPILATORE (emit-time elision) — mai nel loop.
- **Hoare/Matsakis/Klabnik**: misura riconciliata prima di ogni redesign;
  arene a INDICI in safe Rust solo su verticale pilota con kill-criterion;
  capacity≠len; Rc header come voce esplicita di bilancio.
- **Pedersen**: disciplina di confine richiesta (per-test: cycle-collect
  + drain pool + `mi_collect`) — è il modello request-bound che dà a Zend
  i suoi 0,4GB, simulabile senza toccare le strutture.

## Roadmap (fasi, ognuna = 1 sessione WP salvo nota; parità per nome a ogni commit; A/B interleaved come giudice)

### Fase 0 — Attribuzione byte-per-struttura (WP-45)
- **Giorno zero, zero-codice**: esperimento `MIMALLOC_PURGE_DELAY=0` (+
  eventuale `mi_collect` a fine test) sul gruppo media → calibra il
  protocollo vmmap e quota la ritenzione allocatore.
- **Byte-census** (feature `mem-census`, template dei 3 census esistenti,
  `vm/census.rs` come modello): per canale {stringhe, array packed/hashed,
  oggetti, closure, unit compilate, tabelle VM, output buffer} tre numeri:
  `live_bytes`, `peak_bytes` (watermark), `cumulative_bytes`; **capacity
  E len** per i canali Vec; campionamento 1-su-N di
  `mi_usable_size/richiesto` (rounding di size-class); **Rc/RefCell
  header come voce esplicita**.
- **Snapshot AL PICCO** (callback watermark: footprint > max+64MB → dump
  contatori + `task_info` phys_footprint + nome del test corrente), non a
  fine run.
- **Censimenti aggiuntivi**: COW-effectiveness (eventi di copia array ×
  byte — il backup/restore globals di PHPUnit è il sospetto per picchi
  multi-GB); duplicazione contenuti stringhe (quota internabile);
  **conteggio unit ritenute + risposta alla domanda template-include**.
- **Lato oracle**: stessa run sotto C PHP con `memory_get_peak_usage`
  per-test → target per-canale, non solo il 12× aggregato.
- **Gate di accettazione Fase 0**: identità di riconciliazione
  Σ(canali) + rounding campionato + ritenzione allocatore ≈ vmmap al
  picco entro ±10-15%; tabella copre ≥90% del gap; **tabella decisionale
  pre-registrata** (canale % → azione) scritta PRIMA di leggere i numeri.

### Fase 0.5 — Se i template-include leakano: fix con budget proprio
Cache di compilazione keyed sul path risolto (o riuso del Module già
linkato). È un LEAK, non un canale: si chiude prima di attribuire il resto.

### Fase 1 — Quick win footprint (ordine per rischio; ogni item: parità
per nome + A/B con **guardia CPU ≤ +0,5%** + predicted-vs-actual sul
canale ≥70%; land sequenziale con ri-misura, gli item NON sono indipendenti)
1. `shrink_to_fit`/`Box<[T]>` sulle Vec ritenute di Func/Module + drop
   dei seed HIR post-link (stesso sottosistema, un solo A/B).
2. **`created` registry → `Weak` (o eviction a rc==1)** — il buco più
   grosso trovato dal review: de-pinna oggetti+array+stringhe transitivi.
3. Object cold-box: 3 Vec rari + dyn_entries dietro `Option<Box<RareObj>>`
   (~96B × N istanze; pattern WP-32 provato).
4. Disciplina di confine per-test (Pedersen): cycle-collect + drain pool
   + `mi_collect(true)` al boundary del test case — simula il modello
   request-bound di Zend senza toccare strutture.
5. Interning stringhe (letterali + chiavi array) SE il censimento
   duplicati lo quota ≥ centinaia di MB.
**Demansionati da quick-win** (classe Fase-3, si decidono coi dati):
redesign hashed-array a tabella singola (parità enorme: ordine di
iterazione, tombstone, chiavi numeriche-stringa, dtor-order — e va
disegnato arena-compatibile per non rifarlo in Fase 3) e fusione
single-alloc delle stringhe (probabile leva CPU più che footprint; serve
crate DST auditato pre-approvato; prima il contatore size-class).

### Fase 2 — CPU, binario parallelo (compatibile legge WP-44; UN solo A/B
cumulativo con **mechanism-check**: l'op-census deve mostrare il calo
previsto di ~40,5M+47M dispatch; guardia footprint ≤ +2%)
1. **RET_DEREF + ret_shape COME UNA SOLA MODIFICA** (il review ha
   scoperto che farle separate aggiunge un flag-read a 62,6M Ret):
   bitmask `ret_shape` su Func a compile-time, deref assorbito nel corpo
   Ret esistente, `DerefTop` non più emesso dopo le call a valore.
2. Sweep emit-time elision per statement senza op che allocano container
   (⚠️ elidere PRIMA della risoluzione jump, mai peephole; gate con i
   nomi gc/dtor-order: lo sweep è il safepoint dei distruttori — e
   guardia footprint perché meno sweep = picco più alto).
3. Args-Vec pool bounded (come FramePool 64×512) se l'alloc-rate census
   lo quota.

### Fase 3 — Arco heap-a-handle (multi-sessione, SI ESEGUE COMUNQUE)
- **Checkpoint d'ingresso (informativo, non bloccante)**: ri-run
  dell'attribuzione DOPO la Fase 1 — serve a scegliere QUALE tipo
  pilotare per primo (quello col peso maggiore tra {Rc header, rounding,
  frammentazione, doppia-alloc}), non a decidere se partire.
- **Verticale, non orizzontale** (review): pilota su UN solo tipo
  (stringhe O array, scelto dal checkpoint) end-to-end con arena
  tipizzata a handle u32 in safe Rust — niente dual-mode su tutto il
  value graph (raddoppia la superficie di ogni builtin e diventa
  permanente).
- **Checkpoint pilota (direttiva utente: non è un kill)**: se il pilota
  costa >+2% CPU dopo due sessioni di ottimizzazione, si REPORTA il
  verbale all'utente e si prosegue col secondo tipo / con le sessioni di
  ottimizzazione successive — la singola versione regressiva si reverta,
  l'arco continua.
- Qui dentro ricade il redesign hashed-array (layout Zend-style a
  tabella singola, arena-compatibile) e la fusione single-alloc delle
  stringhe (demansionate dalla Fase 1).

### Trasversale — Ratchet di regressione
Registrare le mediane A/B per merge in `gaps/` e rifiutare regressioni
>0,5% non spiegate (il verbale 18/18 di WP-44 è stato possibile solo
perché si misura: istituzionalizzarlo).

## File critici

- `crates/php-runtime/src/vm/census.rs` — template per il byte-census.
- `crates/php-runtime/src/vm/mod.rs` — `Box::leak` (`:3331`), `created`
  (`:1425`), gc_note/gc_buf, FramePool.
- `crates/php-runtime/src/vm/run.rs` — corpo Ret (`:2642-2737`), Sweep
  (`:5090`), DerefTop (`:1543`).
- `crates/php-runtime/src/compile/expr.rs` — emissione DerefTop
  (`:498/524/548`); `compile/mod.rs:677` — emissione Sweep.
- `crates/php-types/src/{zstr.rs, array.rs, object.rs}` — layout valore.
- Riuso: harness gate22 (`wp22-harness/`), A/B (`wp44-harness/ab44.sh`
  come modello), daemonizer perl, watchdog.

## Verifica (per ogni sessione della roadmap)

1. Parità per nome: gate22 completo (option 413 / restapi 3508 COL
   conteggio; per modifiche gated da env: prova positiva del flag nel log,
   lezione WP-44).
2. A/B interleaved 6 round same-day sul gruppo media, con la metrica di
   guardia incrociata (CPU-guard sulle modifiche memoria, footprint-guard
   vmmap sulle modifiche CPU) e mechanism-check (census: il delta previsto
   di dispatch/byte deve esserci — un "win" senza il meccanismo è rumore).
3. Footprint: SOLO vmmap Physical footprint al picco, mai maxrss.
4. Fase 0 ha il suo gate di riconciliazione ±10-15% — se non riconcilia,
   la tabella va corretta PRIMA di usarla per le scelte (ma la roadmap
   prosegue comunque, direttiva utente: al peggio le fasi successive
   partono con priorità stimate anziché misurate).
5. Chiusura sessione: rotazione handoff standard (WP_SESSION_N,
   REPORT_GAP_N con la nuova colonna footprint per-canale).

## Prima sessione eseguibile (WP-45)

Fase 0 completa + esperimento purge day-zero + risposta alla domanda
template-include. Deliverable: tabella "dove vivono i 4,7GB" riconciliata
±15%, tabella decisionale pre-registrata compilata, verdetto Fase 0.5
sì/no, e il gap report esteso con la baseline footprint per-canale.
