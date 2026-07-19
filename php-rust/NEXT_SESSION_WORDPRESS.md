# Rotta WORDPRESS-FIRST — WP-track (dopo WP-21: GC adattivo + redeclare fatal)

> ⚡ **WP-21 (2026-07-19, commit gated `e3d8583`, +`b95b8bf`)**: **GC adattivo
> Zend-like** (soglia del collettore di cicli che cresce quando una collezione
> libera <100 valori — sul gruppo media 5 collect da 50k root liberavano 0-1
> oggetti, pura CPU sul grafo vivo delle fixture) + dedup HashSet di
> gc_light_demoted + skip della 2ª classify senza distruttori. ⚠️ TENTATA e
> REVERTITA la classificazione a note-time in gc_note (perdeva la rete dello
> sweep di fine statement sui drop UNHOOKED — by-design, vedi commento
> gc_light_demoted). 🔧 FIX divergenza WP-20: "Cannot redeclare function"
> su re-require ora fatala con la NUOVA `PhpError::FatalAt` (banner piano
> Zend, uncatchable, no finally, posizione = nuova dichiarazione);
> gh16509.phpt passa. Perf: media user −6,3%; full-suite CPU −3,6% (17:55 →
> 17:17) — il "28% GC" del profilo t45 era del binario PRE-WP-20, non
> rappresentativo. run14: 2 diff per nome = SOLO i dichiarati.

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap**: obiettivo primario = 100%
compatibilità WordPress; la WP core test suite (PHPUnit) è il GATE PER NOME.

## Stato gate per nome (tutte le superfici)
- **Full-suite single-site: 2 diff per nome** (`wp16-harness/full-out/run14/`,
  baseline oracle `full-out/full-oracle.names` trunk `81b2b5b`).
- **Full-suite multisite: 2 diff per nome** (`wp19-harness/ms-out/`,
  riconfermata su WP-21; baseline WP-19 in `ms-out/wp19-baseline/`).
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
# ⚠️⚠️ AZZERARE wpdev/src/wp-content/uploads PRIMA di ogni full run: i probe
#   di gruppo (es. --group media) lasciano canola.jpg ORFANO e il test REST
#   test_sideload_scaled_unique_filename è ORDER-DEPENDENT upstream: con
#   uploads sporchi fallisce su QUALSIASI motore (oracle incluso, standalone).
#   Costò 2 full-run (run12/13) prima della diagnosi in WP-21.
```

## Prossimo passo: SESSIONE WP-22 (perf CPU, coi profili GIUSTI)
1. **CPU ~2,6× vs oracle** (full-suite ~23 min vs 8:50; CPU phpr 17:17):
   ⚠️ LEZIONE WP-21: profilare il BINARIO CORRENTE su una run
   rappresentativa (`sample` sul PID di phpr — `pgrep -x phpr`, NON `-f`
   che matcha /usr/bin/time!). I sospetti dal t45 (pre-WP-20) al netto del
   GC ora sistemato: run_loop dispatch, Zval clone/drop, memmove/memcmp
   (Props linear-scan con Box<[u8]> per chiave — `resolve_prop_access`),
   invoke_named/enter_callee, hashbrown insert/rehash. Candidati: (a)
   indice/interning dei nomi di proprietà (Props::get è linear-scan con
   memcmp); (b) interning stringhe/chiavi; (c) arena per-request.
   Profilare PRIMA la zona specifica (WP-3), su media/query group (~2 min
   di iterazione) e confermare sulla full-suite.
2. **Memoria residua**: dati PHP vivi (fixture + data provider) × footprint
   per-Zval. Misurare con `vmmap --summary → Physical footprint` (RSS
   macOS MENTE sotto compressor, lezione WP-20).
3. **ext/tidy minimale** SOLO se emergono altri consumatori (chiuderebbe
   wpIsIniValueChangeable #4; oggi non vale la superficie).
4. Valutare misura suite phpt `ext/xsl` (aggiungere "xsl" a
   SUPPORTED_EXTENSIONS di phpt-runner) — misura, non gate.
5. Poi: rotta post-WP (Laravel-validazione) da [[php-rust-roadmap-wp-first]]
   o residui trasversali da [[php-rust-todo-master]].

## Lezioni operative (nuove WP-21)
- ⭐ **Profilare il binario/epoca corrente**: i sample t15/t45 in
  wp20-harness/prof-out erano del binario PRE-fix-include (9GB footprint) —
  il loro "28% in gc_sweep" sovrastimava il GC di run11 di ~7×. Un profilo
  vecchio di una sessione può indicare il bersaglio SBAGLIATO.
- ⭐ **Uploads puliti prima delle full run** (vedi harness sopra): il flake
  sideload è STATO AMBIENTALE, diagnosticato con mtime del file orfano
  (09:26 = orario del bench A/B media) + repro standalone su ORACLE.
- ⭐ **Rete unhooked-drop del GC**: gc_note DEVE alimentare sempre il buffer
  candidati — lo sweep di fine statement ri-verifica il count VIVO e
  cattura i drop non notati avvenuti dopo la nota nello stesso statement
  (temp dell'operand stack). Qualunque "classificazione a note-time" rompe
  quella finestra.
- `sample <pid>`: usare `pgrep -x phpr`; `pgrep -f "phpr vendor"` matcha
  anche `/usr/bin/time` e si campiona il processo sbagliato (2 volte...).
- hk può dare 1F flaky (`testWarmupIsNotRunOnSubsequentBoot`, oltre al noto
  ResponseCacheStrategy) — riconfermare con una seconda run prima di
  indagare (WP-19 docet).
- Il gate va SEMPRE rifatto da capo sul binario definitivo: 2 take di
  gate21 buttati per edit arrivati a gate in corso.

## Invarianti (aggiornati WP-21)
- Gate per OGNI commit: corpus/sess/date/refl per NOME — baseline INVARIATA
  WP-18: **corpus 1489 · sess 28 · date 351 · refl 290** (SOLO rimozioni
  ammesse; fail-set in `wp18-harness/gate-out/*.fails`) · ORM 3484 3E/13F
  per nome · http-kernel 1665 0E/0F · cargo (1556) · probe: gd 11/11,
  mysqli 11/11, media-probe byte-id, run-http (DIFF-set 16 = WP-14) · WP
  suite per-classe = oracle (option 413 · media 762 · post 906 · user 1341 ·
  query 1889 · restapi 3514 · taxonomy 878 · comment 582 · xmlrpc 316 ·
  sitemaps 132 · classi WP-17/18). Script pronto: `wp21-harness/gate21.sh`
  (= gate20 con output in wp21-harness/gate-out).
- Full-suite single-site: solo miglioramenti per nome vs **run14 (2 diff)**.
- Full-suite multisite: solo miglioramenti per nome vs **ms-out (2 diff)**.
- Commit AND push a ogni step; run pesanti SEQUENZIALI e DETACHED sotto
  watchdog/telemetria; Serena per Rust (se in timeout: perl -ne via Bash),
  Vexp/Read per il C; Read/Write tool per i .php; log `tr -d '\0'`; probe
  MAI su wptests durante una run; uploads AZZERATI prima di ogni full run.
