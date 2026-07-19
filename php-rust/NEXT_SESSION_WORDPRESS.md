# Rotta WORDPRESS-FIRST — WP-track (dopo WP-22: perf VM zero-alloc)

> ⚡ **WP-22 (2026-07-19, commit gated `47a7c43`)**: **dispatch e accesso
> proprietà ZERO-ALLOC** — 3 round guidati da profili freschi (`sample` sul
> binario corrente, gruppo media come proxy): media 105,8 → 86,1s user
> (**−18,6%**). (1) Op payload `Box<[…]>`→`Rc<[…]>` (il clone per-dispatch di
> run_loop non alloca più); (2) `PropAccess<'a>` borrowed dalla prop_info di
> classe (spariti gli `storage_key.to_vec()` — il singolo allocatore più caldo,
> 1709 tick attribuiti) + chiavi `Cow` + risoluzione UNICA nei prop-op;
> (3) `Const::Str` = `ZStr` condivisa (PushConst = refcount bump), fast-path
> non-lazy in `lazy_prop_access`, `magic_guard.is_empty()` early-out,
> `has_prop_hooks` per classe. run15: 2 diff per nome = SOLO i dichiarati;
> wall full-suite ~21:50 (da ~23), CPU master 17:17→16:40; hk in ~15s.

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap**: obiettivo primario = 100%
compatibilità WordPress; la WP core test suite (PHPUnit) è il GATE PER NOME.

## Stato gate per nome (tutte le superfici)
- **Full-suite single-site: 2 diff per nome** (`wp16-harness/full-out/run15/`,
  baseline oracle `full-out/full-oracle.names` trunk `81b2b5b`).
- **Full-suite multisite: 2 diff per nome** (`wp19-harness/ms-out/`, baseline
  WP-19; non rilanciata in WP-22 — cambi solo perf, single-site a zero
  regressioni su 30.480).
- I 2 diff sono ENTRAMBI dichiarati e stabili: `wp_is_stream #2`
  (stream_get_wrappers onesto — divergenza DECISA) · `wpIsIniValueChangeable
  #4` (dataset generato solo con ext/Tidy).

## Harness full-suite (WP-16 — invariato, ⚠️ USARE QUESTO per le run intere)
```bash
H="/Volumes/Extreme Pro/Claude/wp16-harness"
nohup perl -e 'use POSIX qw(setsid); fork and exit 0; setsid(); exec { $ARGV[0] } @ARGV' -- \
  "$H/run-full-detached.sh" phpr > /tmp/launch.log 2>&1
# monitorare full-out/full-phpr.rss (APPEND tra run: usare tail); attendere full-phpr.done
# diff per nome: perl "$H/extract-junit.pl" junit | sort > names; diff names
# ⚠️ archiviare full-out/run<N>/ PRIMA di rilanciare.
# multisite: wp19-harness/run-multisite-detached.sh <oracle|phpr> (ms-out/)
# ⚠️ MAI probe su wptests durante una run (WP-20 docet).
# ⚠️⚠️ AZZERARE wpdev/src/wp-content/uploads PRIMA di ogni full run (WP-21:
#   canola.jpg orfano → test_sideload_scaled_unique_filename order-dependent).
```

## Prossimo passo: SESSIONE WP-23
1. **CPU residua** (full-suite CPU master 16:40 vs oracle 8:50): l'allocatore
   è SPARITO dal profilo (mi_malloc 98+92 tick vs 407+287); il residuo è
   STRUTTURALE, in ordine dal profilo opt3-t40 (wp22-harness/prof-out/):
   run_loop leaf ~2566 (dispatch+arm inlined) · memmove 636 (concat 187,
   enter_callee 107) · memcmp 361 (Props::get linear-scan — indice per
   chiave interned?) · Zval drop/clone ~510 · gc note/sweep ~520 ·
   dispatch_instance_call 199 · identical 176 · bind_params 162. Candidati:
   (a) indice/interning nomi proprietà (Props oltre ~8 entry → mini-indice);
   (b) frame setup più magro (enter_callee/bind_params); (c) gc_note
   sampling/filtri. ⚠️ Profilare SEMPRE il binario corrente; attribuire i
   leaf allocator ai frame phpr col parse del call-tree di sample (il leaf
   ranking da solo NON basta — lezione WP-22).
2. **Memoria dati vivi**: full-suite ~5min = 5,2G footprint (picco 5,5G),
   media-group 4,6G = WP-20 (le CPU-opt non toccano i dati vivi). Bersaglio:
   footprint per-Zval / interning / arena (backlog WP-7).
3. **ext/tidy minimale** SOLO se emergono altri consumatori (chiuderebbe
   wpIsIniValueChangeable #4; oggi non vale la superficie).
4. Valutare misura suite phpt `ext/xsl` (aggiungere "xsl" a
   SUPPORTED_EXTENSIONS di phpt-runner) — misura, non gate.
5. Poi: rotta post-WP (Laravel-validazione) da [[php-rust-roadmap-wp-first]]
   o residui trasversali da [[php-rust-todo-master]].

## Lezioni operative (nuove WP-22)
- ⭐ **Le due allocazioni più costose erano invisibili nel sorgente**: il
  `.clone()` dell'Op nel dispatch (payload Box) e `storage_key.to_vec()` in
  resolve_prop_access — one-liner che il profilo attribuiva a mi_malloc.
  Metodo: parse del call-tree di `sample` per attribuire i leaf allocator
  ai frame phpr chiamanti (script in sessione; il ranking dei leaf non basta).
- ⭐ **Wall time inquinabile**: la run base è partita con rust-analyzer in
  indicizzazione (wall 231s vs 118s a parità di lavoro). Confrontare SOLO
  lo user CPU del processo (`/usr/bin/time`).
- `cargo fix` NON applica le suggestion E0308: applier custom dei byte-span
  JSON di rustc (66 edit in un colpo, iterare fino a 0 errori).
- PhpStr è immutabile by-design (`new`/`as_bytes`) → condividere le ZStr del
  const pool è safe; le mutazioni stringa ricostruiscono sempre.
- Rc nei payload Op: ogni mutazione in-place di un payload slice va
  RICOSTRUITA (`*types = iter().collect()`), mai `iter_mut` — il buffer può
  essere condiviso tra Func clonate (l'unico sito era relocate CatchMatch).
- Il guard serena-vexp blocca anche gli edit perl sui .rs: Serena
  `replace_content` (occasionali timeout: riprovare — l'edit del primo
  timeout era comunque passato: verificare prima di ripetere).

## Invarianti (aggiornati WP-22)
- Gate per OGNI commit: corpus/sess/date/refl per NOME — baseline INVARIATA
  WP-18: **corpus 1489 · sess 28 · date 351 · refl 290** (SOLO rimozioni
  ammesse; fail-set in `wp18-harness/gate-out/*.fails`) · ORM 3484 3E/13F
  per nome · http-kernel 1665 0E/0F · cargo (1556) · probe: gd 11/11,
  mysqli 11/11, media-probe byte-id, run-http (DIFF-set 16 = WP-14) · WP
  suite per-classe = oracle (option 413 · media 762 · post 906 · user 1341 ·
  query 1889 · restapi 3514 · taxonomy 878 · comment 582 · xmlrpc 316 ·
  sitemaps 132 · classi WP-17/18). Script pronto: `wp22-harness/gate22.sh`
  (output in wp22-harness/gate-out).
- Full-suite single-site: solo miglioramenti per nome vs **run15 (2 diff)**.
- Full-suite multisite: solo miglioramenti per nome vs **ms-out (2 diff)**.
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED sotto
  watchdog/telemetria; Serena per Rust (se in timeout: perl -ne via Bash,
  ma gli EDIT solo via Serena); Vexp/Read per il C; Read/Write tool per i
  .php; log `tr -d '\0'`; probe MAI su wptests durante una run; uploads
  AZZERATI prima di ogni full run.
