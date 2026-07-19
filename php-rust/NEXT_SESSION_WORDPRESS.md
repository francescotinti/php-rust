# Rotta WORDPRESS-FIRST — WP-track (dopo WP-25: drop a stack limitato + fast-path proprietà + deny asymmetric visibility)

> 🏁 **WP-25 (2026-07-20, commit gated `b5deac1`)**: **REGRESSIONE WP-24 SCOPERTA E CHIUSA** — il
> take+drop esplicito in `Object::drop` (postorder WP-24) ha ingrossato il
> frame per livello e la media-suite SEGFAULTAVA allo shutdown (rc=139 con
> ~603G instructions = run completa, stdout bufferizzato perso). Root cause:
> drop RICORSIVO di grafi profondi (overflow a ~45k livelli; l'oracle regge
> 1M — su catene OGGETTO; su array puri annidati l'oracle stesso segfaulta).
> Fix: **`drop_bounded`** (php-types/object.rs) — trampolino con guardia
> di profondità (LIMIT=512): oltre, i payload `DeepDrop{Props,Captures,Val}`
> vanno in coda TLS drenata dal livello 0; postorder id-reuse ESATTO fino a
> 512 (test cargo lo pinna), approssimato oltre (inosservabile). Instradati
> Object/Closure/GenState. + **⚡ fast-path proprietà per-classe**: flag
> `all_props_public` (PropGet+PropIsset: hit non-Undef su istanza non-lazy
> di classe no-hook tutta-public → lettura diretta; valido ANCHE con __get,
> la magia scatta solo su miss) e `plain_set_props` (PropSet: overwrite di
> slot presente; Ref-slot con typed_refs vivi → slow). A/B interleaved 3
> coppie (GET-only): −1,6% user coerente. + **deny ASYMMETRIC VISIBILITY
> 8.4** (gap preesistente: phpr non negava NULLA): `asym_write_error` nei 4
> op-site (PropSet/PropUnset/PropOpSet/PropIncDec), `protected(set)` scopato
> sul PROTOTYPE come i metodi protetti (gh19044-6), `__set`/`__unset` su
> slot esplicitamente-unset passano da magic_applies PRIMA della deny.
> Corpus **1485→1476** (9 asym chiusi, 0 nuovi); cargo 1556→**1558**.

Riprendiamo phpr (PHP 8.5.7 in Rust). **Roadmap**: obiettivo primario = 100%
compatibilità WordPress; la WP core test suite (PHPUnit) è il GATE PER NOME.

## Stato gate per nome (tutte le superfici)
- Gate22 WP-25 verde: corpus 1476 (0 nuovi) · sess 28 · date 351 · refl 290
  identici · ORM 3E/13F identico per nome · hk 1665 0E/0F · cargo 1558/0 ·
  probe gd/mysqli/media byte-id · http battery DIFF-set = 16 (WP-14) ·
  option identico · restapi identico per nome (junit).
- **Full-suite single-site: 1 diff per nome — minimo teorico** (run16; solo
  `wp_is_stream #2`). Non rilanciata in WP-25 (cambi = engine con gate22
  pieno + probe byte-id); al prossimo cambio sostanzioso rilanciarla.
- **Full-suite multisite: 1 diff per nome — minimo teorico** (WP-24,
  `wp19-harness/ms-out/`; solo `wp_is_stream #2`).
- Suite phpt estensioni (misura): xsl 57/64 · tidy 44/45 ·
  asymmetric_visibility 29/39 (residui: field-path, cpp_*, ast_printing,
  gh19044, readonly.phpt). ⚠️ Suite phpt SEMPRE con path ASSOLUTO.

## Harness full-suite (WP-16 — invariato)
```bash
H="/Volumes/Extreme Pro/Claude/wp16-harness"
nohup perl -e 'use POSIX qw(setsid); fork and exit 0; setsid(); exec { $ARGV[0] } @ARGV' -- \
  "$H/run-full-detached.sh" phpr > /tmp/launch.log 2>&1
# ⚠️ ATTENZIONE (lezione WP-25): in ambienti Claude-Code sandboxed il
#   detach setsid può essere REAPED dopo ~15-20'; preferire il background
#   task-managed del tool (run_in_background) per gate22/run lunghe, e
#   ricordare che l'output phpt-runner è BUFFERIZZATO (raw a 0 byte ≠ hang;
#   i figli --run-one sono processi momentanei: ps può non coglierli).
# ⚠️ MAI due gate22 insieme; MAI probe su wptests durante una run;
#   azzerare wpdev/src/wp-content/uploads prima di ogni full run;
#   non ricompilare mentre una run/gate usa il binario.
# multisite: wp19-harness/run-multisite-detached.sh <oracle|phpr> (ms-out/)
```

## Prossimo passo: SESSIONE WP-26
1. **CPU residua strutturale** (profilo fresco `wp22-harness/prof-out/
   wp25-base-t40.sample`, leaf: run_loop 2690 · memmove 563 · drop Zval 357
   · gc_sweep 311 · memcmp 300 · resolve_prop_access 263→ridotto dal
   fast-path · gc_note 241 · dispatch_instance_call 201 · Zval clone 176 ·
   identical 158): prossimi candidati (a) fast-path analogo per il METHOD
   dispatch (dispatch_instance_call+enter_callee+bind_params ~460 leaf);
   (b) interning nomi/stringhe; (c) memmove da concat (rope/append-buffer?);
   (d) **QUICK WIN**: flag per-classe `has_asym_set` per saltare il lookup
   prop_info di `asym_write_error` su ogni scrittura slow-path (l'A/B
   round-2 full-treatment ha reso −0,9% vs il −1,6% GET-only: la deny asym
   introdotta in WP-25 costa un hash-lookup per write dichiarata).
   ⚠️ METODO A/B: SOLO coppie interleaved, stesso momento, user CPU.
   Numeri WP-25 (media group, user): GET-only −1,6% (3 coppie); treatment
   completo (GET/SET/isset + deny asym) −0,9% (2 coppie).
2. **Residui asymmetric visibility** (10 fail): deny nel FIELD-PATH
   (`$o->arr[] = v`, nomi dinamici — dim_add/variation*/reference*);
   promozione costruttore `private(set)` (cpp_*, lower/class.rs:1595 "not
   modelled yet"); check compile-time "Property with asymmetric visibility
   C::$p must have type"; ast_printing; readonly.phpt; gh19044.
3. **Residui strutturali xsl/tidy** (v. WP-24): stream wrapper dentro l'I/O
   libxml (xslt008/-mb/009) · registerPHPFunctionNS functionURI · trace
   senza frame prelude (bug49634) · tidy 010 (sweep release-order).
4. **Roadmap post-WP** da [[php-rust-roadmap-wp-first]]: validazione
   Laravel, oppure residui trasversali di [[php-rust-todo-master]].
5. Se si toccano date/prelude DateTime: gate ext/date OBBLIGATORIO (351).
   Se si tocca ref/arg/reflection: gate ORM+hk obbligatorio.

## Lezioni operative (nuove WP-25)
- ⭐⭐ **rc=139 con instructions retired normali = crash allo SHUTDOWN**
  (stdout block-buffered perso): guardare i `.done` delle run passate per
  capire se è una regressione; riprodurre con catene `->next` profonde.
- ⭐⭐ **Drop ricorsivo = bomba a orologeria**: ogni contenitore che possiede
  Zval e può formare catene (Props, captures, proxy, generator slots) deve
  passare da `drop_bounded`. Se si aggiunge un nuovo contenitore Zval-
  bearing con Drop custom, instradarlo lì.
- ⭐ **Fast-path proprietà: la magia (__get/__set) scatta SOLO su miss** —
  un hit su classe tutta-public non consulta mai hook/magic/visibilità;
  questo rende il fast-path valido anche per classi CON __get.
- ⭐ **`protected(set)` si scopa sul PROTOTYPE** (radice della catena di
  dichiarazioni), come i metodi protetti — gh19044-6.
- ⭐ **asym senza tipo è COMPILE-FATAL nell'oracle** ("must have type"):
  ogni prop `private(set)` è typed → nel flag di write `set_visibility` è
  ridondante ma tenuto per difesa.
- ⭐ **Pazienza coi runner**: phpt-runner bufferizza l'output (raw 0 byte a
  metà run è NORMALE) e i figli --run-one sono effimeri; un sample con
  `thread::sleep` nel main è il poll-loop, NON un hang. Prima di dichiarare
  morto un gate: aspettare 2× il tempo storico del passo.
- ⭐ Probe con `E1::A->x = 1` è compile-fatal nell'oracle (temporary in
  write context): nelle probe usare una variabile.

## Invarianti (aggiornati WP-25)
- Gate per OGNI commit: corpus/sess/date/refl per NOME — baseline:
  **corpus 1476 (AGGIORNATA WP-25) · sess 28 · date 351 · refl 290**
  (SOLO rimozioni ammesse; fail-set in `wp18-harness/gate-out/*.fails`) ·
  ORM 3484 3E/13F per nome · http-kernel 1665 0E/0F · cargo (1558) ·
  probe: gd 11/11, mysqli 11/11, media-probe byte-id, run-http (DIFF-set
  16 = WP-14) · WP suite per-classe = oracle (option 413 · media 762 ·
  post 906 · user 1341 · query 1889 · restapi 3514 · taxonomy 878 ·
  comment 582 · xmlrpc 316 · sitemaps 132 · classi WP-17/18). Script:
  `wp22-harness/gate22.sh` (gate-out WP-24 in gate-out-wp24-archived).
- Full-suite single-site: solo miglioramenti per nome vs **run16 (1 diff:
  wp_is_stream #2)**. Full-suite multisite: solo miglioramenti vs **ms-out
  WP-24 (1 diff: wp_is_stream #2)**.
- Commit AND push a ogni step; run pesanti SEQUENZIALI, sotto watchdog, e
  in background TASK-MANAGED (non setsid — lezione WP-25); Serena per Rust
  (in timeout: verificare lo stato del file prima di riprovare); Vexp/Read
  per il C; Read/Write tool per i .php; log `tr -d '\0'`; uploads azzerati
  prima di ogni full run.
