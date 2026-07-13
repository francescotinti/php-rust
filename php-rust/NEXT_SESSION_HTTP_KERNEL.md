# Prossima sessione: symfony/http-kernel — gap SEND_VAR_EX su elementi + coda (da 29E/104F)

Riprendiamo phpr (PHP 8.5.7 in Rust). La sessione 2026-07-13 (sessione 3) ha
chiuso il cluster E, il cluster container-dump e il cluster session in 3 commit
gated: `eae74f0` (cluster E: trait autoload + named-args module + SplPriorityQueue
+ crc32/crc32c), `514783d` (trait_exists guard + preg_replace $limit +
FilesystemIterator), batch-3 (SessionState.committing + headers_sent out-param).
Dettaglio in memoria: `php-rust-symfony-http-kernel`.

## Dove siamo
- Suite http-kernel (1663 test; oracle 0 fail): **68E/105F → 29E/104F**
  (−39 errori in una sessione). Run ~2-4 min.
- Zend corpus **2445 pass** / 1607 fail (closure_047/048.phpt fixati in
  sessione-3) · ext/session 78 / ext/date 435 / ext/reflection 294 invariati
  per nome · ORM **3E/15F** identico per nome · cargo **1530/0**.
- Workspace: 56c2e188 `…/scratchpad/symfony/http-kernel` (v8.1.1). Baseline
  gate correnti: `corpus-l.norm` ecc. in 3520338b/scratchpad (fallback: -i in
  77155ebc). ORM workspace: 77b21d67/scratchpad/orm-work.
- Probe sessione-3 in 3520338b/scratchpad/e_probe: p_e1..6 (trait autoload),
  p_spq1, p_crc, p_named1, p_pr1 (preg limit), p_fsi1, p_te2, p_hs1
  (headers_sent), p_sess1..4 + p_sl1/2 (session), **p_ref1/2/3 (repro del gap
  by-ref)**.

## Obiettivo primario: GAP ENGINE — SEND_VAR_EX per elementi di array/prop
**Repro minima (p_ref3)**: `(new P1)->m($j['k'])` con `m(&$x) { $x = 42; }` →
oracle `int(42)`, phpr `int(1)`.
- `push_dyn_args` (compile/expr.rs ~1591: receiver dinamico) pusha per CELLA
  solo `ExprKind::Var` (Op::PushRef); un ELEMENTO (`$a['k']`, `$o->p`) va per
  VALORE → un param by-ref non aliasa MAI. Con receiver STATICAMENTE noto il
  ramo `push_call_args` (1584) ha già il MakeRef per i place — funziona.
- Zend risolve col fetch **FUNC_ARG** deciso a runtime (ZEND_SHOULD_SEND_BY_REF
  su arg_info del callee risolto). phpr deve introdurre l'equivalente: un
  descriptor di place differito che il binder di Op::MethodCall risolve al
  bind (by-ref → MakeRef sul place; by-value → read con warning undefined).
  ⚠️ NON fare MakeRef eager su tutti gli args: creerebbe chiavi mancanti
  (PHP: warning) e promuoverebbe elementi che PHP non promuove.
- Impatto noto: SessionListenerTest::testSessionCookieWrittenNoCookieGiven
  (1F residua — i bag Symfony si legano con `$bag->initialize($session[$key])`
  su receiver dinamico) + silenziosi in giro (correttezza, non solo errori).

## Poi, in ordine di ROI (dalla mappa 29E)
- **getLastErrors (6)**: HARD differito — diagnostica del date parser.
- **Dom\HTML_NO_DEFAULT_NS (5)**: costante ext/dom mancante.
- **"Using $this when not in object context" (3)**: da indagare.
- **Store::requestsMatch arg #3 null (3)**, ControllerEvent callable TypeError
  (2), BadRequestException Closure-not-allowed (2), strtotime `'+ 1 hour'`
  (2+1), Constraint::$groups typed-prop (2), `::class` su null (1),
  ReflectionMethod::isClosure (1), is_uploaded_file (1).
- I **104 FAILURES**: mai analizzati sistematicamente — mappa con il perl su
  `hk-run5.log` (3520338b/scratchpad).
- GAP ENGINE secondario (documentato §changelog): **shadowing di METODI
  privati** parent/child (le prop hanno lo storage-key fix; i metodi no) —
  workaround nel prelude (__dicur/__disync), fix vero in
  resolve_method_runtime.

## Lezioni nuove di sessione-3
- Il messaggio d'errore hardcoded può MENTIRE: "closure from a trait used
  across files" era in realtà il dispatch named-args col modulo sbagliato
  (frame.module = script phpunit). Ora il messaggio include unit/idx/len.
- `class_exists`/`interface_exists`/`trait_exists`/ReflectionClass: la class
  table di Zend è UNICA (classi+interfacce+enum+trait) — ogni lookup che
  fallisce e ri-innesca l'autoload su un nome già dichiarato in un'altra
  "tabella" phpr produce un re-include che collide (PriorityTaggedServiceUtil,
  HttpKernelInterface). Guardie: trait_declared() in resolve_class_autoload /
  try_autoload; class_index in ho_trait_exists.
- preg_replace con $limit era silenziosamente replace-all: il PhpDumper
  corrompeva il PROPRIO output (la riga FrozenParameterBag contiene anch'essa
  "DEPRECATED_PARAMETERS"). Diagnosi: catturare i file di cache generati con
  un copy-loop PRIMA del tearDown.
- phpunit-run da cwd sbagliata = output vuoto silenzioso (il `&&` dopo cd
  cambia directory: lanciare phpunit SEMPRE con cwd = workspace).

## Invarianti (identici)
- Gate per OGNI commit: probe byte-id vs oracle · corpus per NOME
  (`--list-fails`, baseline `corpus-l.norm`) · ext/session+date+reflection per
  nome · ORM (3E/15F) se ref/arg/reflection · cargo test. MAI `cargo build`
  durante un gate phpt. MAI CARGO_TARGET_DIR in /private/tmp.
- Commit AND push a ogni step (⚠️ controllare `git status` per artifact
  spurii PRIMA di `git add -A`); run pesanti SEQUENZIALI e DETACHED; Serena
  per Rust; Read tool per i .php; log con `LC_ALL=C tr -d '\0'`.
