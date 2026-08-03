# Verbale sedia 5 — Bak (V8/HotSpot: alloc-rate, path caldi, startup/warmup) — Concilio WP-95

**VERDETTO: CON EMENDAMENTI.** B1/B2 (naming + natura transiente al livello malloc) reggono. La refutazione LEVER-2 come scritta NON è consumabile: verdetto NULLO su metrica cieca e senza pavimento di rumore. La dicotomia «riserva vs spike» è falsa al livello OS/provisioning.

## Q1 — Protocollo del probe slope: perché 21837 B/worker non è informativo
1. **Il pavimento di rumore è già nel file e supera l'effetto**: ctrl (build 8e966efd) slope 18814309 vs baseline build b048b697 slope 18852538 → 38229 B/worker di spread tra due run "identiche a meno di build" — MAGGIORE del delta lever 21837. Un delta sotto il pavimento osservato non refuta né conferma: è sotto-risoluzione. La varianza run-to-run non è mai stimata NEL probe (un run per W): chiamarla «refutata con misura» è troppo; è «non rilevata al di sotto della risoluzione».
2. **Metrica strutturalmente cieca**: `peak_footprint` è il massimo; `mi_collect` post-preludio agisce DOPO il picco (lo spike da 39,5MB è già avvenuto). Una leva post-picco non può, per costruzione, muovere il peak: il NULLO era atteso a priori. Serviva anche una metrica di residency post-warmup (checkpoint dopo l'ultima richiesta, purge già a 0).
3. **Due soli W**: slope da 2 punti assume linearità non testabile; N=25·W piccolo fa dominare il warmup nel peak.
**Per renderla consumabile**: R≥5 repliche per arm, INTERLEAVED (A,B,A,B…) sulla stessa binaria; W∈{1,2,4}; mediana±banda 2se pubblicata NEL probe; verdetto NULLO solo come test di EQUIVALENZA (|delta|+banda < soglia dichiarata ex-ante); doppia metrica peak+residency.

## Q2 — Alloc-rate e warmup per-worker (39,5MB churn alla prima richiesta)
Il preludio è `OnceCell` per-thread inizializzato NELLA prima richiesta: ogni worker paga parse+lower di ~406KB e 39,5MB di churn sul request path. Effetti: (a) le prime W richieste dopo ogni deploy/restart/autoscale portano una latenza da cold-start → il p99 di flotta è inquinato a ogni rollout, non «una volta sola»; (b) al restart il thundering herd sincronizza gli spike: picco transiente ~W×39,5MB che il provisioning DEVE coprire — uno spike sincrono su W thread è una riserva ai fini del capacity planning, anche se liberato; (c) freed ≠ decommitted: i raw m90 mostrano arena committed 151MB (W4) persistente anche a exit_collect_mi — al livello OS lo «spike» lascia residuo. Lezione V8: il warmup appartiene allo spawn (o al build), mai alla richiesta — spostare l'init all'avvio del worker toglie il p99-hit a costo ~zero, prima di ogni leva di memoria.

## Q3 — Rank V8-style
1. **Snapshot/precompilazione del preludio** (serializzare l'output del Lowerer a build-time, deserializzazione/mmap a startup): unica leva che elimina SIA i 39,5MB SIA la CPU di parse, per ogni thread E ogni processo CLI (4,42× → verso 1×). È l'endgame (V8: isolate da ~30ms a ~2ms), ma costo/gate massimi (versioning, parità completa).
2. **Condivisione cross-thread** (once-per-process invece di per-thread): spike ×1 invece di ×W e taglia la quota per-worker dello slope; bloccante probabile = `LoweredPrelude` non Send+Sync (Rc/Cell) — serve uno spike di fattibilità PRIMA di deciderne il costo.
3. **Arene per-file**: taglia il picco per-thread (una `Bump` oggi cumula le SETTE unità simultanee — visto nel corpo di `lower_prelude_uncached`), ma NON tocca la CPU di warmup né il ×W. Safe-only, giusta come prima mossa, non come destinazione.

## Q4 — Priorità S-94.0 (FONDAMENTALI-first)
1. battery61 riproducibile (criterio 5, invariato in cima). 2. Probe slope v2 (A-BB-74/75) — prerequisito per giudicare QUALSIASI leva futura. 3. Leva per-file arenas coi gate completi. 4. Cifra warmup: latenza prima-richiesta per worker (A-BB-76). 5. Spike fattibilità Send+Sync (A-BB-77).

## Emendamenti
- **A-BB-74**: protocollo slope v2 — R≥5 interleaved stessa binaria, W∈{1,2,4}, mediana+banda 2se NEL probe; NULLO solo come equivalenza con soglia ex-ante.
- **A-BB-75**: nessuna refutazione di leva post-picco su sola metrica peak — obbligatoria metrica residency post-warmup.
- **A-BB-76**: misurare la latenza della prima richiesta per worker (warmup p99) come cifra di canale prima di scegliere per-file/share/snapshot.
- **A-BB-77**: spike di fattibilità `LoweredPrelude: Send+Sync` (once-per-process) prima di investire nello snapshot.

## KS
- **KS-BB-95-1**: un peak non può refutare una leva che agisce dopo il picco — metrica cieca = refutazione vacua.
- **KS-BB-95-2**: uno spike sincrono su W thread è una riserva ai fini del provisioning; freed ≠ decommitted ≠ non-provisioned.
- **KS-BB-95-3**: il warmup appartiene allo spawn (o al build), mai alla richiesta.

## Refutazioni capitali
**SÌ, una**: la refutazione LEVER-2 «con misura» è inconsumabile come scritta — delta 21837 B/worker sotto il pavimento di rumore 38229 già presente nel file e mai dichiarato, su metrica (peak) strutturalmente insensibile alla leva. Va retrocessa a «non rilevata sotto risoluzione» finché il probe v2 non gira.
