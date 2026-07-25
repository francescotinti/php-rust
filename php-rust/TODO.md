# phpr — Master TODO

The single living list of everything left to do. Each entry says what is
missing, why it is deferred, and where the detail lives. Check items off here
as they complete. Deliberate behavioural deviations are catalogued in
[`PHPR_DIVERGENCES_FROM_PHP.md`](PHPR_DIVERGENCES_FROM_PHP.md); measured
coverage in [`COVERAGE.md`](COVERAGE.md).

Current state (2026-07-25, post session WP-52): Zend corpus **2635** passing
(65.0% of runnable; gate baseline **1421** fails by name) · internal functions
**1017/2143, 47%** (core stdlib **539/654, 82%**). **WORDPRESS: the full
single-site core PHPUnit suite (30,472 tests, wordpress-develop trunk) AND
multisite (31,278 tests) are each at a SINGLE declared name-diff vs the
oracle** (honest `stream_get_wrappers`), stable by name across runs. WP
installs and serves on **real MySQL** via native `mysqli`;
wp-admin/front/REST/permalinks **byte-identical over HTTP** (`phpr -S`
SAPI); media byte-parity on **system libgd FFI** (+exif + fileinfo native);
XSLT on **system libxslt** with the real **registerPHPFunctions/php:function
trampoline** (xsl phpt 63/64). The GC is now a **Zend-model cycle collector
over objects AND containers** (Weak root buffer, Zend-exact
`gc_collect_cycles()` counts, real `gc_enable`/`gc_status`; gc family
36→14 fails, WP-46). Perf: the specializing-interpreter arc (WP-29..44)
plus the GC arc hold the media benchmark at **~2.61×** the oracle's CPU;
the **memory attribution arc (WP-45..48)** — exact reached-vs-live
reconciliation per allocation over every Zval-bearing VM field — found
the dominant holder (an uneviced ReflectionMethod-descriptor memo, 456k
entries/2.48G under PHPUnit mocks) and cut the **peak-footprint gap from 11.9× to ~4.16×**; the full-suite CPU residual (3.4× vs 2.06×) was
attributed entirely to the cycle-collector classify walk (WP-48,
measured), then cut by four successive levers: WP-49's lazy purge of
refcount-dead tombstones at the trigger (**3.4×→~2.5×**, rounds
1005→297, freed conserved), WP-50's closure of the statement-sweep
fast-path band (**~2.41×**; the inline form regressed — hot-arm I-cache
law — cure = bound cached in a field), WP-51's fusion of the
classifier's three bookkeeping tables into one pre-reserved map
(**~2.31×**; census classify −33s reconciles the same-night full A/B
to the digit), and WP-52's **in-node walk marks** (**~2.14×**;
epoch-guarded 8-byte mark on objects/arrays/closures + one contiguous
record table = ZERO hash lookups per edge; census classify **−42.7%**,
109.4s→62.7s, every collector count conserved; media A/B −0.57% new
6/6, peak physical +0.61% — kept and verbalized). WP-52 also
cold-boxed `Object`'s rare feature sets behind one `Option<Box>`
(**−56B/instance measured**, obj churn −2.02GB; `dyn_entries`
deliberately excluded — it is stdClass's storage, a hot path). WP-51's
growth-gated boundary collect proved the pinned `created` channel
(353.7MB at exit) is **teardown garbage**, unreachable by any mid-run
cadence (kept for long-running coverage). WP-53 shipped Fase 2
(ret_shape+RET_DEREF absorbed DerefTop, Sweep emit-time elision:
−5.66% dispatch = only −0.9% CPU — levers are now quoted in ns/event
first). WP-54 applied the attribution method to **CPU-seconds**
(sampled call trees over the full run) and re-keyed the
reflect-descriptor memo on its true owner, the **declaring class**
(96% of entries were inherited duplicates, hit-rate 11.7%→96.7%):
**−7.4% media CPU (2.61×, all-time best), −5.8% peak**. WP-55 shipped
**growable PhpStr (`hash + Vec<u8>`) + a fused `.=` op with in-place
append at unique refcount** (probe 499ms→2ms = oracle; full **−2.6%
same-evening → ~2.11×**, peak **~4.15×** with the +8B layout cost
inside its ≤2% guard) and ran the Fase-3 byte checkpoint: **next pilot
= hashed arrays** (~421MB live at peak vs ~45MB strings)
(NEXT_SESSION_WORDPRESS.md). Other stacks at parity: **symfony/http-kernel
CLOSED 0/0 (1665)**, http-foundation 0 errors, Doctrine ORM 3484 (3E/13F
declared, stable by name) + DBAL 3769/0/0, PHPUnit 9/11/13, Composer,
wp-cli, Monolog. Laravel afterwards as validation.

---

## A. Tokenizer — group-A residuals (42/49, 5 fails)

- [ ] **bison/yacc syntax messages** (`TOKEN_PARSE_000`, unterminated heredoc):
  needs PHP's *parser* error layer (mago's messages differ) — possibly
  infeasible byte-identical without a message adapter.
- [ ] **`gh19507_throw`**: an error handler invoked from a builtin must appear
  in traces as `[internal function]` with an empty file arg (cross-cutting, §D).
- [ ] **`bug54089`** statement-level `__halt_compiler` post-halt content
  (PHP-scanner-specific spans). Only the `$o->__halt_compiler()` method case works.
- [ ] **`bug77966`** keyword-as-identifier in trait adaptation
  (`use A { namespace as bar; }` → T_STRING).
- [ ] **`PhpToken::is(float)`**: ZPP float→int deprecation instead of TypeError (§D).

## B. Streams — cluster residuals

- [ ] **stream_socket_server / _sendto / _recvfrom**: implementable on the
  existing Tcp/Udp backends; byte-identical testing needs a local network
  harness (non-deterministic).
- [ ] **stream_bucket_*** (userland filter protocol) — the built-in filter layer
  (zlib.* + convert.base64-*) is done; userland `php_user_filter` classes are not.
- [ ] **stream_get_wrappers**: include user-registered wrappers (today it lists
  only phpr's 5 built-ins — honest but incomplete).
- [ ] **Userland wrapper completions**: `url_stat` (file_exists/stat through the
  wrapper), dir ops (`dir_opendir`…), `STREAM_USE_PATH` propagation,
  `stream_wrapper_restore`.
- [ ] Known divergences (§2.4): resource-id numbers in var_dump; the internal
  `stream_eof`/`stream_seek` call sequence on a mixed read/write handle.

## C. Missing-builtin detector — 180 remaining (real-app-usage ranked)

### C.1 Near-term (pure/deterministic, high real usage) — next candidates
- [x] ~~**strftime / gmstrftime**~~ ✅ a990d28 — full C-locale UTC formatter,
  Deprecated notice, 28/28 runnable phpt. (Deferred: setlocale non-C locales.)
- [x] ~~**dir** — OOP `Directory`~~ ✅ a990d28 — dir()->read()/rewind()/close()
  byte-identical; internal-class semantics (readonly/no-construct) diverge, see
  PHPR_DIVERGENCES §3.2.
- [ ] **DNS family**: gethostbyname/gethostbyaddr/gethostbynamel/dns_get_record/
  checkdnsrr — std::net, but network-dependent; needs an honest local-resolution story.
- [ ] **get_defined_constants** (7): needs an enumerable constant table
  (`resolve_constant` is a non-enumerable match — hard).
- [ ] **get_include_path / set_include_path**: include-path state is a scope-out.

### C.2 ⛔ Would-be-lying stubs (do NOT add until the real support exists)
- **hash_hmac_algos** (44 algos) / **mb_list_encodings** (79 encodings): phpr
  does not support them all — listing them would lie to `function_exists` probes.
- **timezone_identifiers_list** (419 names): embeddable, but phpr's timezones
  are name-only (no offset/DST database) — the area itself is incomplete.

### C.3 SAPI / state / infra (deferred by nature)
- [ ] **mail** (22) — side-effecting SMTP/sendmail.
- [ ] **is_uploaded_file / move_uploaded_file** — upload SAPI (CLI: always false).
- [ ] **parse_ini_file / parse_ini_string** — a full flex lexer to match byte-for-byte.
- [ ] **get_cfg_var / ini_get_all** — ini state.
- [ ] **preg_last_error / _msg** — phpr does not track PCRE error state.
- [ ] **opcache_invalidate / opcache_is_script_cached** — the oracle has opcache, phpr doesn't.

### C.4 Whole extensions at 0% (bounded, one dedicated session each)
- [ ] **zip** (crate `zip`), **bz2**, **sodium** (~110 fns, `dryoc`),
  **finfo/fileinfo** (magic db), **session** (stateful + SAPI), **gettext**,
  **exif**, **ftp**, **sockets**, DB drivers **pgsql/mysqli** (network protocol).
  The zlib playbook (system-library FFI in `php-types`, exact PHP parameters,
  oracle-locked contracts, suite-driven batches) is the template.

### C.5 zlib — remaining edge (extension complete: 30/30, 114/115)
- [x] ~~`__DIR__` absolute under relative invocation~~ — was never actually
  broken; compress_zlib_wrapper passes. (The prior "fail" was a mislabelled
  bug61820 panic, fixed in fb6ecbe.)
- [ ] `bug71417` streaming partial-decode (decode-up-front cannot recover partial
  output from corrupt input — documented divergence; the runner times it out).
- [ ] gzopen `$use_include_path`; gzseek on write streams: forward-only + zero-fill.

## D. Cross-cutting engine gaps (limit type-error phpts, not real usage)

- [ ] **Stringable coercion in value builtins**: `convert::to_zstr` cannot run
  `__toString` (no VM). Partially closed via `Ctx.stringify` (~28 builtins).
- [ ] **ZPP null-to-non-nullable deprecation** ("Passing null to parameter…").
- [ ] **#[SensitiveParameter]** redaction in traces.
- [ ] **Upfront callback validation** ("must be a valid callback" ZPP error).
- [ ] **ArgumentCountError location** raised from inside internal-function callbacks.
- [ ] **By-ref array element preservation** through some array functions.
- [ ] **Handler-from-builtin trace frame** as `[internal function]` + empty file arg.

## E. Pre-existing backlog

- [x] ~~**Restore the doctrine/orm suite**~~ ✅ e011438 — workspace rebuilt
  (oracle-driven composer install, phpr runs phpunit 11.5.56); full run =
  **3484 tests, 3 err / 17 fail**, identical to the last baseline (no drift).
  Needed advertising the `filter` extension. The workspace is ephemeral
  (session scratchpad); recreation recipe + baseline in memory. This gate is
  MANDATORY for arg-passing / reference / reflection changes.
- [ ] **Bounded ext areas toward 100%**: json, pcre, opcache, standard/strings,
  array — categorize fails by diff signature.
- [ ] **lazy_objects hard queue** (8 deferred: by-ref proxy stack overflow,
  WeakRef-during-init, initializer stack traces, …).
- [ ] **Reflection non-matchable residue** (needs a C-extension registry). Low ROI.
- [ ] **Refactor residue** (low ROI): split the remaining vm/host.rs clusters,
  lower/decl.rs.

## Working rules (invariants)

- **Correct-or-absent**: a lying stub is worse than a missing function
  (frameworks probe with `function_exists`).
- **ORM gate is mandatory** for changes touching arg-passing, references, or
  the reflection surface (the Zend gate alone is not enough).
- Every function ships **byte-identical vs the PHP 8.5.7 oracle**, plus a full
  Zend-corpus gate (zero pass→fail) before committing.
