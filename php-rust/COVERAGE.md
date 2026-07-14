# phpr — Coverage

**PHP 8.5.7 reimplemented in Rust.** This page is the living, data-driven snapshot
of how much of PHP `phpr` actually runs. Numbers are measured, not estimated:
function coverage comes from probing every one of the reference build's internal
functions with `function_exists()` inside `phpr` (grouped by
`ReflectionFunction::getExtensionName()`); the corpus number is the real pass
count of the upstream Zend test suite under `phpt-runner`.

_Last measured: 2026-07-14 · reference: PHP 8.5.7 (`get_defined_functions()`)._

---

## Headline

| Metric | Value |
| --- | --- |
| Internal functions implemented | **785 / 2143** (37%) |
| — of which **core / language stdlib** (standard + Core + date) | **522 / 654** (80%) |
| Zend test corpus (`Zend/tests/*.phpt`) | **2493 passing** — 61.5% of runnable (2493/4056) |
| Fully-complete areas | ctype, json, SimpleXML, zlib, bcmath, tokenizer, **session**, PDO core |

Corpus breakdown: 5305 total · **2493 pass** · 1563 fail · 1249 skip (skips are
mostly tests that need an extension `phpr` hasn't ported, or SAPI-specific
setup; the runner now executes `--INI--` sections as `php -d`-style overrides,
which moved ~180 formerly-skipped tests into the run).

The single most important number is **80% of the core language stdlib**: the
string / array / math / var / date / output / hashing / regex surface that
ordinary PHP code (and real frameworks) actually touch. The gap to 37%-overall is
almost entirely **whole database/crypto/network extensions** that are simply not
started yet — not holes in the language.

---

## What already runs end-to-end

Beyond raw function counts, these real-world stacks execute byte-identically to
upstream PHP under `phpr` today:

- **Composer** — `require`, `install`, `diagnose`, `about` (real HTTPS via rustls).
- **PHPUnit** — 11.5, 13.2 and 13.3-dev, byte-identical output.
- **Doctrine** — ORM (3484 tests, 3 err / 14 fail), DBAL (3769/0/0), Collections,
  Lexer, Inflector, Event Manager.
- **Symfony http-foundation** — full component test suite **including
  Tests/Session** (1790 tests): 10 errors / 27 failures, with PHPUnit's
  process-isolation tests actually spawning `phpr` child processes. Without the
  Session suite (1419 tests): **0 errors**, the only 12 failures being the
  functional tests that spawn a real `php -S` server (needs a server SAPI).
  Plus String / Console / Process, already validated earlier.
- **Symfony http-kernel** — **CLOSED: the full 1663-test suite passes at
  0 errors / 0 failures**, parity with the reference interpreter. The DI
  container pipeline works end-to-end (ContainerBuilder compiles, PhpDumper
  dumps byte-identically, the Kernel reloads the dumped container); by-ref
  argument binding matches Zend's runtime SEND_VAR_EX; `eval()` shares the
  calling scope; anonymous functions carry PHP 8.4's `{closure:Scope():line}`
  names; first-class callables on magic methods build `__call`/`__callStatic`
  trampolines; nested `isset`/`empty`/`??` over `ArrayAccess` dispatch the
  protocol on intermediate offsets; **real IANA timezones** (TZif reader on
  the system zoneinfo, timelib gap/fold semantics, zone-aware DateTime
  arithmetic); real `flock(2)` advisory locks; and Zend-faithful destructor
  timing (eager per-statement sweep, LIFO object-id reuse).
- **ext/session** — all 23 functions + SessionHandler and the three handler
  interfaces; files handler byte-identical (0600 `sess_<id>` files, php /
  php_binary / php_serialize serializers, lazy_write, mtime GC); official
  phpt suite 161/229 with `--run-skipif` (the tail is trans-sid URL rewriting
  and the deprecated `SID` constant). Symfony's SessionHandlerProxy write
  path (handler calls during `session_write_close`) works.
- **PDO + pdo_sqlite + SQLite3** — on `rusqlite`.
- **Monolog** — running.
- **Reflection** — framework-grade (types, attributes, enums, union/intersection).
- PHP 8.4 **property hooks**, **lazy objects**, **asymmetric visibility**, a real
  **GC cycle collector**, generators, fibers, enums, `readonly`,
  **Zend late binding** for class declarations (a file whose class references an
  unloadable supertype compiles and errors only when the declaration executes).

---

## Coverage by area

Measured `have / total` against the PHP 8.5.7 reference, grouped by the
oracle's own extension names ("standard" + "Core" + "date" together form the
core language stdlib).

| Area | have / total | % | Notes |
| --- | ---: | ---: | --- |
| **standard** | 442 / 544 | **81%** | string, array, math, var, filesystem, streams, output, include_path |
| **Core** | 50 / 62 | **81%** | class/function introspection, error handling |
| **date** | 30 / 48 | 63% | DateTime classes, textual strtotime, HTTP-date formats, **real IANA timezones** (system TZif, DST-correct); official suite 215 pass |
| session | 23 / 23 | **100%** | files + user save handlers, SessionHandler classes, `$_SESSION`; suite 161/229 |
| ctype | 11 / 11 | **100%** | complete |
| json | 5 / 5 | **100%** | complete (HEX_* flags, NUMERIC_CHECK, THROW_ON_ERROR) |
| SimpleXML | 3 / 3 | **100%** | complete |
| tokenizer | 2 / 2 | **100%** | `token_get_all`/`token_name` + `PhpToken` on the mago lexer; official suite 42/49 |
| zlib | 30 / 30 | **100%** | byte-identical via system zlib; gz streams, compress.zlib://, stream filters — suite 114/115 |
| bcmath | 14 / 14 | **100%** | + `BcMath\Number` (methods + operators) + `RoundingMode` |
| PDO | 1 / 1 | **100%** | + pdo_sqlite on rusqlite |
| gmp | 46 / 51 | 90% | `GMP` class + operators (num-bigint); random + import/export deferred |
| random | 8 / 9 | 89% | Mt19937 bit-exact with PHP |
| mbstring | 48 / 65 | 74% | codecs + grapheme family + 8bit/BINARY |
| SPL | 11 / 15 | 73% | iterators, class_*, SplFileObject/SplTempFileObject, SplPriorityQueue, FilesystemIterator |
| pcre | 8 / 11 | 73% | `preg_last_error*` pending |
| hash | 12 / 20 | 60% | common algos incl. crc32/crc32b/crc32c, byte-identical digests |
| libxml | 4 / 8 | 50% | |
| filter | 3 / 7 | 43% | `filter_var(_array)` incl. VALIDATE_IP (RFC 6890 flags), FILTER_CALLBACK, REQUIRE/FORCE_ARRAY, min/max ranges |
| curl | 13 / 35 | 37% | easy API on `ureq` |
| pcntl | 5 / 25 | 20% | |
| iconv | 1 / 10 | 10% | |
| posix | 3 / 40 | 8% | |
| intl | 11 / 187 | 6% | grapheme done; ICU surface huge |
| openssl | 1 / 64 | 2% | TLS handled at stream layer, not fn-level |
| **not started (0%)** | — | 0% | pgsql (123), sodium (110), mysqli (106), gd (105), ldap (55), odbc (48), xmlwriter (42), sockets (37), ftp (36), snmp (24), tidy (24), xml (22), calendar (18), dba (15), readline (12), bz2/gettext/zip (10 each), opcache (8), sysv* (18), fileinfo (6), shmop (6), exif (4), dom (2), soap (2) |

1358 functions missing overall; the not-started extensions above account for
~780 of them. **symfony/http-kernel is closed (0/0 on 1663 tests)** and
**wp-cli runs end-to-end from source** (`wp --info` at oracle parity) — the
current front is **WordPress**: `wp core download` + the official SQLite
integration plugin first, then a real server SAPI, then `mysqli` and media
(gd/exif/zip). See NEXT_SESSION_WORDPRESS.md.

---

## Divergences

Known, deliberate deviations from upstream PHP are catalogued in
[`PHPR_DIVERGENCES_FROM_PHP.md`](PHPR_DIVERGENCES_FROM_PHP.md). The governing
principle is **correct-or-absent**: a function that lies is worse than one that is
missing, so partial/incorrect builtins are not registered.

---

## How these numbers are produced

```sh
# 1. oracle list with extension names (reference PHP 8.5.7)
php -r 'foreach (get_defined_functions()["internal"] as $f) {
    $r = new ReflectionFunction($f);
    echo ($r->getExtensionName() ?: "core"), "\t", $f, "\n"; }' | sort > oracle-fns.tsv

# 2. generate a probe script: function_exists() per function, run it under phpr
#    (emit "ext\tfn\t0|1" per line), tally have/total per extension.

# 3. corpus
phpt-runner --isolate "…/php-8.5.7/Zend/tests"
# → the "pass:" line; runnable = total - skip
```

The core-stdlib rollup = `standard` + `Core` + `date` (654 functions total).
