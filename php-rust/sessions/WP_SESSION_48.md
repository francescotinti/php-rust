# WP_SESSION_48 — Fase 1.1 shrink unit (predizione al byte) + attribuzione CPU full

> ⚡ **WP-48 (2026-07-24, `33e358a`+`ba16547`+…)** — **Fase 1.1 CHIUSA con il
> mechanism-check più pulito della roadmap: predizione MISURATA 70.625.796
> byte di slack cap−len nei moduli leakati → post-shrink il canale unit cala
> ESATTAMENTE di quel numero (100,0% predicted-vs-actual; soglia era ≥70%).
> Guardia CPU ≤+0,5% rispettata: A/B +0,04% = rumore (2/6 round).**

## Obiettivo 1 — Fase 1.1: shrink unit

### L'analisi PRIMA della leva (mandato del prompt: capire cosa è rilasciabile)

- **I seed HIR NON sono rilasciabili**: `Vm::seed_classes`/`seed_traits`/
  `seed_globals` alimentano la lowering seeded di OGNI unit successiva
  (autoload/include arrivano per tutta la vita del processo); WP-20 aveva
  già ridotto la ritenzione della unit-cache a `SeedDelta` (tails, non il
  `Program` intero) e il `Program` HIR di un include muore già a fine
  `run_include`. `Module.deferred`/`conditional_traits` (HIR nel modulo)
  servono a `Op::DeclareDeferred`/`DeclareTrait` a runtime. Verdetto:
  il rilasciabile della Fase 1.1 è lo SLACK di capacity, non gli HIR.
- **Il census conta capacity** (`module_census_bytes`: `ops.capacity()` ecc.)
  ⇒ lo shrink è direttamente leggibile nel canale CH_UNIT.

### Il metodo: predizione misurata, non stimata

1. Contatore `unit_slack` (solo `mem-census`): ai due siti di `Box::leak`
   (`drive_unit` + `run_include`) somma i termini cap−len ESATTAMENTE nelle
   stesse voci che il canale unit conta per capacity (`module_slack_bytes`).
2. Census baseline (binario `phpr-memgc48-pre` = WP-47 + contatore, gruppo
   media): **unit.live 299.925.102 / 2046 moduli · unit_slack 70.625.796**
   = predizione: −70,6MB (23,5% del canale). ⭐ la rotta diceva "shrink unit
   ~0,30G": 0,30G è la TAGLIA DEL CANALE, la leva shrink può togliere solo
   lo slack — il resto è bytecode len reale (leve diverse, non Fase 1.1).

### La leva (`33e358a`)

`Module::shrink()` a fine `compile_program_impl` (mai sull'execution path):
`shrink_to_fit` su ops/lines/consts/exc_table/static_vars/attributes di ogni
`Func` + ricorsione su TUTTI i Func annidati (metodi, prop_init, thunk di
costanti di classe, `CompiledAttribute` new/args, hook get/set, default dei
parametri, `StaticInit::Thunk`) + le Vec di metadati di `CompiledClass` e
`Module`. Rc condivisi (prelude fns, stub interned) skippati via
`Rc::get_mut`: già shrunk quando il modulo owner ha compilato. Nessun cambio
a UnitKey/fingerprint/linking (il contenuto delle Vec è identico; solo la
capacity cambia, non osservabile).

### Mechanism-check (census post, stesso giorno, stesso workload)

- unit.live **299.925.102 → 229.299.306 = −70.625.796 byte: il 100,0%
  ESATTO della predizione** (2046 moduli identici; str/arr/obj alla cifra).
- `unit_slack` post-shrink = **0** (lo slack non esiste più).
- proxy_peak census: 957,2 → 890,1MB = −67,2MB (≈95% dello slack anche al
  picco del proxy).

### Gate e A/B

- cargo test --release **1639/0**; corpus **1421 IDENTICO per nome** (0 nuovi).
- A/B 6 round interleaved old=`ce4be6e` (oracle 20,79/20,93 stabilissimo):
  - **user CPU: old 61,02s → new 61,05s = +0,04%, round 2/6 = RUMORE**
    (guardia ≤+0,5% rispettata — prima leva memoria della roadmap a costo
    CPU zero).
  - **peak footprint fisico: 1,756 → 1,747G = −0,52%** (new più stabile:
    max 1,760 vs 1,792 old). ⭐ il canale è calato di 70,6MB ma il peak
    fisico solo di ~9MB: il PICCO è mid-run, quando i 2046 moduli non sono
    ancora tutti caricati — lo slack pieno esiste solo verso fine run. La
    leva vale il suo prezzo (zero) ma il peak fisico del media group non è
    il punto dove si vede.
- Rapporti di giornata: media CPU 61,05/20,86 = **2,93×** · footprint
  1,747/0,395 = **4,42×** (oracle di giornata più basso di WP-47:
  0,395 vs 0,41 — confrontare i RAPPORTI con cautela, lezione REPORT_GAP).

## Obiettivo 2 — Attribuzione CPU full-suite (3,42× vs 2,06× WP-40)

Contatori nuovi (`ba16547`, solo feature `gc-census`, parità intoccata):
- `classify_ns`: tempo cumulativo dentro `gc_classify` → quota del residuo
  attribuibile al collect-walk.
- `reflect_hits/misses/evictions` su `reflect_method_info_cache` → il thrash
  del cap 8192 di WP-47 (rebuild dei descrittori = il +1,3% media).

Run strumentata full-suite (binario `phpr-memgc48`, mem+gc census,
`wp48-harness/run-full-census.sh`, 30.472 test / 2F / 86W / 73S = conteggi
run34 anche sotto census):

- **`classify_ms` 494.921 ≈ 8:15 di CPU dentro `gc_classify`**, su
  **1005 collect** (9.928.400 root processati, 14.157.791 freed —
  sulla full il collector LIBERA davvero, a differenza del media group
  WP-46; soglia adattiva arrivata a 1.050.000). Il residuo da attribuire
  era ≈7:41 (19:20 run34 − 11:39 WP-40): **il collect-walk da solo copre
  l'intero residuo (~107% — il timer è dentro classify, non affetto
  dall'overhead census dei note-hook)**. ⭐ ATTRIBUZIONE CHIUSA: il 3,42×
  è il classify sul grafo vivo della full, non il churn dell'eviction.
- **Reflect-cache (cap 8192) in thrash conclamato ma SECONDARIO**:
  890.132 miss / 173.842 hit (hit-rate 16%) / 108 eviction — quasi tutti
  i miss sono rebuild post-eviction (108×8192 ≈ 884k). Costo stimato in
  decine di secondi (ordine 10× sotto il classify). Un cap più alto è da
  misurare col suo costo memoria (~5,4KB/entry: cap 16384 ≈ +45MB).
- Contorno: canale unit full = 501,3MB / 4726 moduli con `unit_slack=0`
  (lo shrink attivo vale ~155MB stimati sulla full, slack 23,5%);
  `notes` 4,11 MILIARDI (inserted 282M); sweeps light 209,8M;
  RSS max telemetria 1.768MB (binario census).

**Leve WP-49 giustificate dai numeri** (SOLO ora che l'attribuzione c'è):
1. Ridurre il costo del classify (child-list solo sul sottografo white è
   già WP-47: il resto è il live-BFS by-borrow sul grafo vivo — mark
   intrusivi sui container restano impossibili in safe Rust; candidate:
   cadenza/escalation della soglia sulla full, rooting più selettivo per
   non processare 9,9M root).
2. Cap reflect-cache più alto SE il suo costo memoria misurato regge
   (thrash provato: hit-rate 16%).

## Gate22 completo

**VERDE col conteggio** (binario `33e358a`+`ba16547`, 14:05→14:29, archivio
`gate-out-wp48-archived/`): corpus **1421** (0 nuovi-fail) · sess 28 /
date 351 / refl 290 IDENTICI · ORM 3484 **3E/13F per nome** · hk 1665
**0E/0F** · cargo **1639/0** · gd/mysqli/media probe BYTE-ID · http
DIFF-set atteso (WP-14) · **option 413/1061 IDENTICO col conteggio** ·
**restapi 3508/15947 (1E = oracle) IDENTICO col conteggio**.

## Full run35 (parità + gap)

- **fail-set BYTE-IDENTICO a run33** (88 nomi = run34; 30.472 test,
  0E/2F/86W/73S). Baseline per nome resta `run33-fails.txt`.
- Master-CPU **≈19:10 = 3,39×** (run34 era ≈19:20 = 3,42×: rumore, oggi
  nessuna leva CPU è stata applicata al binario di parità). Wall ~24 min
  = 2,1×.
- RSS telemetry max **5431MB mid-run, IDENTICO a run34** (a fine run
  ~2,1G): il transiente mid-run non è cambiato — il binario census
  (più lento) ha mostrato max 1.768MB, quindi il picco 5,4G è
  timing/pacing-dipendente, non una ritenzione secca. Resta materia
  d'attribuzione footprint per la coda della Fase 1 (item `created`→Weak,
  cold-box Object).

## ⭐ Lezioni

- ⭐⭐ **La predizione di una leva memoria si MISURA con un contatore
  dedicato prima di scrivere la leva** (unit_slack): il mechanism-check
  diventa un'identità contabile (100,0%), non una speranza — stesso metodo
  reached-vs-live di WP-47 applicato a una leva invece che a un'attribuzione.
- ⭐ **"Canale da X MB" ≠ "leva da X MB"**: il canale unit era 0,30G ma la
  parte aggredibile con shrink era il 23,5% (slack); il resto è len.
  Scrivere sempre QUALE quota del canale la leva bersaglia.
- ⭐ Il peak footprint fisico è cieco alle ritenzioni che maturano a fine
  run (i moduli si accumulano): per le leve da coda-di-run il metro giusto
  è il canale census / live-end, non il picco.
- ⭐ perl `glob()` splitta i pattern sugli SPAZI: su path con spazi
  (`/Volumes/Extreme Pro/…`) usare `opendir`/`readdir`.

## Prossimo (WP-49)

1. **Recupero CPU full-suite, ora con bersaglio attribuito**: il residuo
   3,4× vs 2,06× è il classify su 1005 collect / 9,9M root. Leve in
   ordine: cadenza/escalation soglia sulla full (la soglia arriva a 1,05M
   ma i collect restano 1005 — capire il pattern per-test), rooting più
   selettivo (9,9M root processati per 14,2M freed: buon rapporto, ma
   quanti root sono ricorrenti vivi?), cap reflect-cache più alto col suo
   costo memoria misurato (thrash provato: 16% hit).
2. Fase 1 residua: `created` registry → Weak (item 1.2, il più grosso),
   cold-box Object (1.3), disciplina di confine per-test (1.4).
3. Attribuzione del transiente 5,4G mid-run della full (invariato da
   run34; il census binario non lo mostra — serve uno snapshot al picco
   sulla full).
4. ⚠️ pre-flight disco WP-49: Data chiusa a ~10Gi (php-rust-output 2,1G
   con artifact di test; le cache utente grosse sono già state pulite in
   WP-48 — candidati: snapshot APFS, `cargo clean` dei soli test artifact).
