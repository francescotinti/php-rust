# phpr — Master TODO

The single living list of everything left to do. Each entry says what is
missing, why it is deferred, and where the detail lives. Check items off here
as they complete. Deliberate behavioural deviations are catalogued in
[`PHPR_DIVERGENCES_FROM_PHP.md`](PHPR_DIVERGENCES_FROM_PHP.md); measured
coverage in [`COVERAGE.md`](COVERAGE.md).

Current state (2026-07-14): Zend corpus **2486** passing · internal functions
**785/2143, 37%** (core stdlib **522/654, 80%**) · ext/tokenizer **42/49** ·
ext/zlib **complete** (30/30, suite 114/115) · **ext/session: COMPLETE**
(23/23 functions, official suite 161/229) · ext/date official suite **215**
pass. **symfony/http-foundation**: no-session config at **0 errors /
12 failures** (the 12 spawn a real `php -S` server). **symfony/http-kernel:
CLOSED** — the full 1663-test suite passes at **0 errors / 0 failures**
(parity with the oracle). Sessions 5–8 closed 286 errors / 84 failures in
total; the last mile brought **real IANA timezones** (TZif reader over the
system zoneinfo, timelib gap/fold DST semantics, zone-aware DateTime
arithmetic, `date_default_timezone_set` + INI `date.timezone`), constructor
visibility at `new`, ZPP-faithful `is_callable`/enum `from()`/int-range
coercion, `DateTime` comparison semantics in the operator table, real
`flock(2)` advisory locks, `error_log()` with the INI target, and
**Zend-faithful destructor timing** (eager per-statement sweep with LIFO
object-id reuse — Symfony DI configurators rely on `__destruct` running
between statements). Corpus gained +41 across sessions 5–8 with zero
regressions by name. **Next front (NEXT_SESSION_WORDPRESS.md, roadmap in
memory `php-rust-roadmap-wp-first`): WORDPRESS** — wp-cli from source on
the official SQLite integration plugin, then a real server SAPI, then
`mysqli` and media (gd/exif/zip); Laravel afterwards as validation.

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
