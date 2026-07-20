# Rotta WORDPRESS-FIRST — WP-track (dopo WP-27 fase 1: PhpArray dual-repr packed/hashed)

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

## 🎯 PROSSIMO LAVORO: WP-27 fase 2 — Props SLOT-BASED (punto 2 del verdetto WP-26)
Layout per-classe condiviso + `Vec<Zval>` slots + overflow map per le dynamic:
da 1.852 → ~650 B/oggetto stimati (oggi `Props` = `Vec<(Box<[u8]>, Zval)>`:
~800B/istanza di sole chiavi duplicate + 400B di fat-pointer su 25 prop).
Sussume l'interning dei nomi e dà accesso per indice di slot alla Zend.
- Punti di attacco: `php-types/object.rs` (Props), `bytecode.rs` (per-class
  flags già esistenti: all_props_public/plain_set_props/has_asym_set — il
  layout slot si aggancia lì), `resolve_prop_access`/`PropAccess<'a>` di WP-22.
- ⚠️ Il fast-path WP-25 (PropGet/PropSet) e la lazy-hash WP-23 di Props
  vengono ASSORBITI dal layout nuovo: rifare l'A/B dopo.
- ⚠️ Se si tocca ref/arg/reflection: gate ORM+hk obbligatorio (già nel gate22).
- Probe memoria: riusare `memcost_arr.php` pattern (scratchpad WP-27) con
  oggetto 25-prop; target ≤650B/istanza, stdClass invariata (già meglio).
- Dopo la fase 2: A/B media + full-suite run18 + multisite.

## Candidati successivi (in coda dopo Props slot-based)
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
