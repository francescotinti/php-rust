# Rotta WORDPRESS-FIRST — WP-track (dopo WP-27 COMPLETA: PhpArray dual-repr + Props slot-based)

> 🏁 **WP-27 fase 2 (2026-07-20)**: **Props SLOT-BASED** (punto 2 del verdetto
> WP-26). `PropsLayout` per classe (chiavi storage di `prop_defaults` in
> ordine di dichiarazione + FxHash precalcolati, `Rc` in `CompiledClass`
> accanto a `info`); `Props` = `slots: Vec<Option<Zval>>` allineato al layout
> (`Some(Undef)` = typed-uninit presente, `None` = mai-settata/unset) +
> `dyn_entries` (dynamic in ordine di assegnazione) + contatore `live`.
> API per-chiave INVARIATA ⇒ VM intatto salvo ~7 siti di costruzione
> (`Props::with_layout`). ⭐ FIX di divergenza reale: unset+re-set di prop
> DICHIARATA torna allo SLOT DI DICHIARAZIONE (Zend fixed offsets) — prima
> phpr appendeva in coda e serialize/json_encode/var_dump/(array)/foreach
> divergevano (props_probe.php 12 sezioni ora byte-id). + FIX
> `#[AllowDynamicProperties]` su classi ANONIME (lower_anonymous_class
> scartava gli attributi → deprecation spuria). **Memoria: oggetto 25-prop
> 1.852 → 503 B/istanza (oracle 480 — quasi parità); stdClass 1.249 vs
> oracle 1.337 (phpr MEGLIO)**. A/B media: CPU neutra (82,71 vs 82,80 user),
> footprint −3,4%. Gate22 fase 2 tutto verde; **run18 = run17 = run16 per
> nome (minimo teorico)**.

> 🏁 **WP-27 fase 1 (2026-07-20)**: **PhpArray DUAL-REPR packed/hashed Zend-like**
> (php-types/array.rs, punto 1 del verdetto WP-26). `Repr::Packed(Vec<Option<Zval>>)`
> per chiavi dense 0..n (chiavi IMPLICITE: niente Key né HashMap index);
> escalation ONE-WAY a `Repr::Hashed` su string-key, chiave negativa, buco
> oltre len o scrittura su tombstone. ⭐ ORACLE-PINNED: Zend NON fa revive
> in-place dei buchi packed — `unset($a[1]); $a[1]=99` itera 0,2,1 (re-insert
> in CODA) ⇒ il comportamento single-repr di phpr era già giusto e la packed
> deve escalare, mai rivivere in place. `remove` resta sempre packed
> (tombstone; tronca i tombstone in CODA ⇒ array_pop+append riusa la chiave
> senza escalation). Iteratori: Item da `(&Key,&Zval)` a `(Key,&Zval)` (chiavi
> owned — Int copia, Str Rc-bump); solo ~30 call-site toccati. **Memoria**
> (delta peak): 2,5M int **121→39 B/el** (steady ~27; oracle 16,4 — il residuo
> è la realloc-copy di Vec, Zend estende le pagine in place); 100k array
> packed 25 int **81→15,8 B/el = MEGLIO dell'oracle (28,5)**; assoc invariata.
> **CPU**: A/B interleaved media 3 coppie **−2,9%** user (83,49 vs 85,95).

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap**: obiettivo primario = 100%
compatibilità WordPress; la WP core test suite (PHPUnit) è il GATE PER NOME.

## Stato gate per nome (tutte le superfici)
- Gate22 WP-27 verde (wp22-harness/gate-out): corpus 1476 · sess 28 · date 351
  · refl 290 IDENTICI · ORM 3E/13F identico per nome · hk 1665 0E/0F · cargo
  **1567**/0 (+9 test dual-repr) · probe gd/mysqli/media byte-id · http battery
  DIFF-set = 16 (WP-14) · option 413 e restapi 3514 identici per nome.
- **Full-suite single-site run17: IDENTICA a run16 per nome (30.481 test,
  0E/2F/86W/73S) = minimo teorico** (solo `wp_is_stream #2` vs oracle).
  Archiviata in `wp16-harness/full-out/run17/`.
- **Full-suite multisite: 1 diff per nome — minimo teorico** (WP-24,
  `wp19-harness/ms-out/`; solo `wp_is_stream #2`). Non rilanciata in WP-27
  (single-site run17 identica + gate22 pieno); rilanciarla al prossimo cambio
  sostanzioso.
- Suite phpt estensioni (misura): xsl 57/64 · tidy 44/45 ·
  asymmetric_visibility 29/39. ⚠️ Suite phpt SEMPRE con path ASSOLUTO.

## Harness full-suite (WP-16 — invariato)
```bash
H="/Volumes/Extreme Pro/Claude/wp16-harness"
"$H/run-full-detached.sh" phpr   # lanciarlo dentro un task BACKGROUND
                                 # task-managed (lezione WP-25: niente setsid)
# ⚠️ MAI due gate22 insieme; MAI probe su wptests durante una run;
#   azzerare wpdev/src/wp-content/uploads prima di ogni full run;
#   non ricompilare mentre una run/gate usa il binario.
# multisite: wp19-harness/run-multisite-detached.sh <oracle|phpr> (ms-out/)
```

## 🎯 PROSSIMO LAVORO (il verdetto WP-26 è COMPLETO: entrambi i punti chiusi)
1. **Multisite riconferma** (non rilanciata in WP-27: single-site run17/run18
   identiche; al prossimo cambio sostanzioso o subito se si vuole chiudere il
   cerchio: `wp19-harness/run-multisite-detached.sh phpr`, atteso 1 diff
   `wp_is_stream #2`).
2. **Slot-index fast path** (CPU, facoltativo): `resolve_prop_access` conosce
   la classe → può restituire l'INDICE di slot (precalcolato in `PropInfo`)
   e saltare `PropsLayout::slot_of` per accesso O(1) diretto; aggancio ai
   flag per-classe esistenti (all_props_public/plain_set_props). L'A/B WP-27
   fase 2 era CPU-neutro: il guadagno va cercato lì.
3. **Validazione Laravel** ([[php-rust-roadmap-wp-first]]): il layout memoria
   nuovo è a posto (oggetti ~parità oracle, array packed meglio/quasi-parità).

## Candidati successivi
1. **Memoria packed residua** (se mai servisse): la realloc-copy di Vec tocca
   le pagine (39 vs 16,4 B/el di peak sul 2,5M); mimalloc in-place realloc o
   reserve esplicita nei costruttori bulk (array_fill, range, unserialize).
2. **CPU residua strutturale** (profilo wp22-harness/prof-out/): method
   dispatch fast-path (dispatch_instance_call+enter_callee+bind_params);
   interning nomi; memmove da concat. ⚠️ A/B SOLO coppie interleaved.
3. **Residui asymmetric visibility** (10 fail): field-path deny, promotion
   cpp_*, compile-check "must have type", ast_printing, readonly.phpt, gh19044.
4. **Residui strutturali xsl/tidy** (WP-24): stream wrapper nell'I/O libxml
   (xslt008/-mb/009) · registerPHPFunctionNS · bug49634 · tidy 010.
5. **Roadmap post-WP**: validazione Laravel dopo il layout memoria nuovo.
6. Se si toccano date/prelude DateTime: gate ext/date OBBLIGATORIO (351).

## 📊 REPORT GAP PERF ORACLE↔PHPR — ATTIVITÀ RICORRENTE DI FINE SESSIONE (richiesta utente 2026-07-20)
A OGNI chiusura di sessione, prima del commit finale di memoria/handoff,
misurare e riportare all'utente il gap aggiornato, e aggiornare la tabella
qui sotto (storico = trend tra sessioni; ⚠️ confrontare RAPPORTI, mai i
tempi assoluti di giornate diverse):
1. **Media group**: oracle 1 run `/usr/bin/time -l` (DB reset + uploads
   azzerati) vs phpr (riusare la media `new` dell'A/B di sessione se c'è,
   altrimenti 1 run identica) → rapporto **user CPU** e **peak footprint**.
2. **Full-suite**: CPU del processo master phpr dall'ultima riga del
   telemetria `.rss` della runN di sessione vs full-suite oracle (già
   baseline; rimisurarla solo se si sospetta drift ambientale) → rapporto.

Metrica full-suite (definita WP-27, stesso giorno per entrambi): CPU del
processo MASTER dal tail del `.rss` (⚠️ esclude i figli isolati, undercount
per entrambi) + wall; RSS di telemetria solo indicativo (mente sotto
compressor — vmmap per i footprint veri).

| sessione | media CPU (phpr/oracle) | media footprint | full-suite master-CPU | full-suite wall |
|---|---|---|---|---|
| WP-26 (baseline) | 85,8/21,0 = **4,1×** | 5,0/0,4GB = **12,7×** | (metrica non comparabile: "~1,9×" era il wall) | ~1,9× |
| WP-27 | 82,7/21,1 = **3,9×** | 4,78/0,40GB = **12,0×** | 16:11/5:39 = **2,9×** | ~22/11,5 min = **1,9×** |

## Lezioni operative (nuove WP-27)
- ⭐⭐ **Semantica packed di Zend, oracle-pinned**: NIENTE revive in-place dei
  tombstone (re-insert dopo unset va in CODA all'ordine di iterazione, sia
  packed che hashed — `packed_probe.php` 16 sezioni byte-id); array_pop
  (pop_adjust_next_free) + append riusa la chiave e in packed deve restare
  packed (troncare i tombstone di coda al remove dell'ultimo vivo).
- ⭐ **Item iteratori owned**: cambiare `(&Key,&Zval)` in `(Key,&Zval)` costa
  pochissimo grazie alle match ergonomics (~30 errori E0308/E0614 su ~150
  call-site; il compilatore li trova tutti — nessun silent break possibile).
- ⭐ **`Option<Zval>` = 16B** (niche del discriminante) — const-assert pinnato
  in array.rs; la packed è a costo Zend per slot.
- ⭐ **peak footprint ≠ resident**: la realloc di Vec (memcpy) TOCCA le pagine
  ⇒ il peak conta old+new buffer; Zend erealloc estende in place e le pagine
  mai toccate della table non contano. Confrontare anche lo steady.
- ⭐ I campi di PhpArray sono privati al modulo array.rs: il refactor di repr
  è stato possibile senza audit manuale dei consumer — il borrow checker
  enumera lui la superficie. Da preservare per Props (fase 2).
- ⚠️ Nelle probe PHP niente `"$GLOBALS..."` in doppi apici (Array-to-string;
  e l'oracle duplica il warning su stderr via log_errors → falso diff).

## Invarianti (aggiornati WP-27)
- Gate per OGNI commit: corpus/sess/date/refl per NOME — baseline:
  **corpus 1476 · sess 28 · date 351 · refl 290** (SOLO rimozioni ammesse;
  fail-set in `wp18-harness/gate-out/*.fails`) · ORM 3484 3E/13F per nome ·
  http-kernel 1665 0E/0F · cargo (**1567**) · probe: gd 11/11, mysqli 11/11,
  media-probe byte-id, run-http (DIFF-set 16 = WP-14) · WP suite per-classe =
  oracle (option 413 · media 762 · post 906 · user 1341 · query 1889 ·
  restapi 3514 · taxonomy 878 · comment 582 · xmlrpc 316 · sitemaps 132 ·
  classi WP-17/18). Script: `wp22-harness/gate22.sh`
  (gate-out WP-26 in gate-out-wp26-archived).
- Full-suite single-site: solo miglioramenti per nome vs **run17 (= run16;
  1 diff: wp_is_stream #2)**. Multisite: vs **ms-out WP-24 (1 diff idem)**.
- Commit AND push a ogni step; run pesanti SEQUENZIALI, sotto watchdog, e
  in background TASK-MANAGED (non setsid); Serena per Rust (in timeout:
  verificare lo stato del file prima di riprovare); Vexp/Read per il C;
  Read/Write tool per i .php; log `tr -d '\0'`; uploads azzerati prima di
  ogni full run. A/B: binario old in worktree con `crates/php-server` e
  `Cargo.lock` copiati dal working tree (gitignored), target separato
  (`wp22-harness/build-old-wp27.sh` come modello).
