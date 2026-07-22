# phpr — Coverage

**PHP 8.5.7 reimplemented in Rust.** This page is the living, data-driven snapshot
of how much of PHP `phpr` actually runs. Numbers are measured, not estimated:
function coverage comes from probing every one of the reference build's internal
functions with `function_exists()` inside `phpr` (grouped by
`ReflectionFunction::getExtensionName()`); the corpus number is the real pass
count of the upstream Zend test suite under `phpt-runner`.

_Last measured: 2026-07-23 (WP-39) · reference: PHP 8.5.7 (`get_defined_functions()`)._

---

## Headline

| Metric | Value |
| --- | --- |
| Internal functions implemented | **1017 / 2143** (47%) |
| — of which **core / language stdlib** (standard + Core + date) | **539 / 654** (82%) |
| Zend test corpus (`Zend/tests/*.phpt`) | **2609 passing** — 64.3% of runnable (2609/4056) |
| **WordPress core test suite** | **full effective parity** — single-site 30,472 tests AND multisite 31,278 tests each at **a single declared name-diff**, stable by name across runs |
| Fully-complete areas | ctype, json, SimpleXML, zlib, bcmath, tokenizer, session, **xml**, **fileinfo**, **tidy**, PDO core |

Corpus breakdown: 5305 total · **2609 pass** · 1447 fail · 1249 skip (skips are
mostly tests that need an extension `phpr` hasn't ported, or SAPI-specific
setup; the runner executes `--INI--` sections as `php -d`-style overrides).

The headline story is now twofold: **82% of the core language stdlib**, and the
**entire WordPress core test suite at a single divergent test name** — a
deliberate, catalogued decision (an honest `stream_get_wrappers`); the second
historical diff closed when **ext/tidy** landed natively on the system libtidy.
The gap to 47%-overall remains **whole database/crypto/network extensions**
that are simply not started yet — not holes in the language.

---

## What already runs end-to-end

Beyond raw function counts, these real-world stacks execute byte-identically to
upstream PHP under `phpr` today:

- **WordPress** (wordpress-develop trunk) — installed and served on **real
  MySQL** (native `mysqli` on the wire), front pages / login / REST / pretty
  permalinks / wp-admin **byte-identical** over HTTP via the `phpr -S`
  server SAPI; the official **core PHPUnit suite (30,472 tests single-site)
  at a single declared name-diff** (multisite, 31,278 tests, also at 1).
  Media pipeline at byte parity via **system libgd FFI** (+ exif);
  `ext/tidy` complete on the **system libtidy**; `ext/xsl` on the **system
  libxslt** (sitemaps' XSLT byte-identical, real
  `registerPHPFunctions`/`php:function` callbacks); fileinfo native (ground
  truth on 849 files); ext/xml SAX; big-5/HTML-ENTITIES mbstring codecs;
  argon2 password hashing; an intl subset (Normalizer).
- **wp-cli** — runs end-to-end from source at oracle parity.
- **Composer** — `require`, `install`, `diagnose`, `about` (real HTTPS via rustls).
- **PHPUnit** — 9.6, 11.5, 13.2 and 13.3-dev, byte-identical output, including
  process isolation (child runs spawn `phpr`).
- **Doctrine** — ORM (3484 tests, 3 err / 13 fail — declared, stable by name),
  DBAL (3769/0/0), Collections, Lexer, Inflector, Event Manager.
- **Symfony http-kernel** — **CLOSED: the full 1665-test suite passes at
  0 errors / 0 failures**, parity with the reference interpreter (DI container
  pipeline end-to-end, PhpDumper byte-identical, Zend-faithful destructor
  timing with the eager per-statement sweep).
- **Symfony http-foundation** — full component suite at 0 errors (the
  historical 12 failures needed the server SAPI that now exists) + String /
  Console / Process.
- **ext/session** — all 23 functions + SessionHandler family; files handler
  byte-identical; Symfony session proxies work.
- **PDO + pdo_sqlite + SQLite3** — on `rusqlite`; WordPress also runs on the
  official SQLite integration plugin.
- **Monolog** — running.
- **Reflection** — framework-grade (types, attributes, enums, union/intersection).
- PHP 8.4 **property hooks**, **lazy objects**, **asymmetric visibility**, a real
  **GC cycle collector** (with Zend-style adaptive collection threshold),
  generators, fibers, enums, `readonly`, **Zend late binding** for class
  declarations.

---

## Coverage by area

Measured `have / total` against the PHP 8.5.7 reference, grouped by the
oracle's own extension names ("standard" + "Core" + "date" together form the
core language stdlib).

| Area | have / total | % | Notes |
| --- | ---: | ---: | --- |
| **standard** | 452 / 544 | **83%** | string, array, math, var, filesystem, streams, output, crypt (bcrypt+argon2) |
| **Core** | 52 / 62 | **84%** | class/function introspection, error handling, gc_* |
| **date** | 35 / 48 | 73% | DateTime classes, textual strtotime, **real IANA timezones** (system TZif, DST-correct); official suite 351-fail baseline gated by name |
| session | 23 / 23 | **100%** | files + user save handlers, SessionHandler classes, `$_SESSION` |
| xml | 22 / 22 | **100%** | SAX parser (quick-xml), libxml-compatible error codes, namespace callbacks |
| fileinfo | 6 / 6 | **100%** | native magic detection — byte-identical on an 849-file ground truth |
| tidy | 24 / 24 | **100%** | on the **system libtidy via FFI** (same keg as the oracle) — tidy/tidyNode classes, ob_tidyhandler; 44/45 upstream phpt |
| ctype | 11 / 11 | **100%** | complete |
| json | 5 / 5 | **100%** | complete (HEX_* flags, NUMERIC_CHECK, THROW_ON_ERROR) |
| SimpleXML | 3 / 3 | **100%** | complete (+xpath, casts) |
| tokenizer | 2 / 2 | **100%** | `token_get_all`/`PhpToken` on the mago lexer |
| zlib | 30 / 30 | **100%** | byte-identical via system zlib; gz streams, filters |
| bcmath | 14 / 14 | **100%** | + `BcMath\Number` (methods + operators) |
| PDO | 1 / 1 | **100%** | + pdo_sqlite on rusqlite |
| pcre | 10 / 11 | 91% | byte-mode non-/u, branch reset `(?|…)`, `preg_last_error*` |
| gmp | 46 / 51 | 90% | `GMP` class + operators (num-bigint) |
| random | 8 / 9 | 89% | Mt19937 bit-exact with PHP |
| mysqli | 86 / 106 | **81%** | native wire protocol (`mysql` crate), byte-safe, drives real WordPress on MySQL 9 |
| mbstring | 52 / 65 | 80% | codecs incl. BIG-5 + HTML-ENTITIES, grapheme family, substitute_character |
| SPL | 11 / 15 | 73% | iterators, SplFileObject, SplPriorityQueue, FilesystemIterator |
| hash | 13 / 20 | 65% | common algos incl. crc32 family, byte-identical digests |
| gd | 62 / 105 | **59%** | on the **system libgd via FFI** — byte parity with the oracle (same dylib) |
| exif | 2 / 4 | 50% | exif_read_data/imagetype for the WP media pipeline |
| libxml | 4 / 8 | 50% | + DOMDocument loadHTML/saveHTML (libxml2-mode HTML4 parser) |
| filter | 3 / 7 | 43% | `filter_var(_array)` incl. VALIDATE_IP, FILTER_CALLBACK |
| curl | 13 / 35 | 37% | easy API on `ureq` |
| iconv | 3 / 10 | 30% | + iconv_mime_decode(_headers) |
| pcntl | 5 / 25 | 20% | |
| intl | 15 / 187 | 8% | grapheme + Normalizer + idn_to_* (native punycode); ICU surface huge |
| posix | 3 / 40 | 8% | |
| openssl | 1 / 64 | 2% | TLS handled at stream layer, not fn-level |
| **not started (0%)** | — | 0% | pgsql (123), sodium (110), ldap (55), odbc (48), xmlwriter (42), sockets (37), ftp (36), snmp (24), calendar (18), dba (15), readline (12), bz2/gettext/zip (10 each), opcache (8), sysv* (18), shmop (6), dom (2 fns — the DOM *classes* are implemented), soap (2) |

1126 functions missing overall; the not-started extensions above account for
~550 of them. Class-only surfaces don't show in function counts: **DOM,
XSLTProcessor (system libxslt FFI, incl. `registerPHPFunctions` callbacks),
ZipArchive (write side), XMLReader-level SAX** are implemented as classes.
**The WordPress track is at a single divergent test name on both the full
single-site and multisite suites** — current work is performance: the
specializing-interpreter arc has brought the media benchmark to **2.71×**
the oracle's CPU (from 4.1×) and the full-suite master CPU to **2.11×**;
next is the GC note/demote churn and the live-data memory footprint, then
Laravel validation. See NEXT_SESSION_WORDPRESS.md.

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
#    Automated: ./scripts/measure-coverage.sh

# 3. corpus
phpt-runner --isolate "…/php-8.5.7/Zend/tests"
# → the "pass:" line; runnable = total - skip
```

The core-stdlib rollup = `standard` + `Core` + `date` (654 functions total).
