# phpr — Coverage

**PHP 8.5.7 reimplemented in Rust.** This page is the living, data-driven snapshot
of how much of PHP `phpr` actually runs. Numbers are measured, not estimated:
function coverage comes from probing every one of the reference build's internal
functions with `function_exists()` inside `phpr`; the corpus number is the real
pass count of the upstream Zend test suite under `phpt-runner`.

_Last measured: 2026-07-10 · reference: PHP 8.5.7 (`get_defined_functions()`)._

---

## Headline

| Metric | Value |
| --- | --- |
| Internal functions implemented | **701 / 2143** (33%) |
| — of which **core / language stdlib** | **483 / 654** (73%) |
| Zend test corpus (`Zend/tests/*.phpt`) | **2327 passing** — 61.2% of runnable (2327/3804) |
| Fully-complete areas | ctype, json, SimpleXML, bcmath, PCRE core, SPL core |

Corpus breakdown: 5305 total · **2327 pass** · 1477 fail · 1501 skip (skips are
mostly tests that need an extension `phpr` hasn't ported, or SAPI-specific setup).

The single most important number is **73% of the core language stdlib**: the
string / array / math / var / date / output / hashing / regex surface that
ordinary PHP code (and real frameworks) actually touch. The gap to 30%-overall is
almost entirely **whole database/crypto/network extensions** that are simply not
started yet — not holes in the language.

---

## What already runs end-to-end

Beyond raw function counts, these real-world stacks execute byte-identically to
upstream PHP under `phpr` today:

- **Composer** — `require`, `install`, `diagnose`, `about` (real HTTPS via rustls).
- **PHPUnit 13.2** — byte-identical output.
- **Doctrine** — ORM, DBAL (3769/0/0), Collections, Lexer, Inflector, Event Manager.
- **PDO + pdo_sqlite + SQLite3** — on `rusqlite`.
- **Monolog**, **Symfony String / Console** — running.
- **Reflection** — framework-grade (types, attributes, enums, union/intersection).
- PHP 8.4 **property hooks**, **lazy objects**, **asymmetric visibility**, a real
  **GC cycle collector**, generators, fibers, enums, `readonly`.

---

## Coverage by area

Measured `have / total` against the PHP 8.5.7 reference. "core/other" is the PHP
language runtime and general stdlib; everything below it is a discrete extension.

| Area | have / total | % | Notes |
| --- | ---: | ---: | --- |
| **core / language stdlib** | 483 / 654 | **73%** | string, array, math, var, output, date-ish, misc |
| ctype | 11 / 11 | **100%** | complete |
| json | 5 / 5 | **100%** | complete |
| SimpleXML | 3 / 3 | **100%** | complete |
| bcmath | 14 / 14 | **100%** | 14 fns + `BcMath\Number` (methods + operator overloading) + `RoundingMode` |
| gmp | 46 / 51 | 90% | 49 fns + `GMP` class + operator overloading (num-bigint); random + import/export deferred |
| mbstring | 48 / 65 | 73% | codecs + grapheme family |
| SPL | 11 / 15 | 73% | iterators, class_* |
| PCRE | 8 / 11 | 72% | `preg_last_error*` pending |
| hash | 11 / 19 | 57% | common algos |
| filter | 3 / 7 | 42% | `filter_var*` done |
| curl | 13 / 35 | 37% | easy API on `ureq` |
| date | 20 / 56 | 35% | `date_parse` grammar ported |
| pcntl | 5 / 25 | 20% | |
| posix | 3 / 40 | 7% | |
| intl | 11 / 187 | 5% | grapheme done; ICU surface huge |
| gd | 2 / 107 | 1% | |
| openssl | 1 / 64 | 1% | TLS handled at stream layer, not fn-level |
| **not started (0%)** | — | 0% | pgsql, sodium, mysqli, ldap, odbc, sockets, ftp, zlib, sysv*, snmp, tidy, session, dba, bz2, zip, gettext, finfo, exif |

Extension totals of the missing 1502: intl 176, pgsql 123, sodium 110, mysqli
106, gd 105, xml 64, openssl 63, ldap 55, gmp 51, odbc 48, sockets 40, posix 37,
ftp 36, zlib 29 … and ~255 uncategorized core/stdlib functions (streams,
sessions, timezone objects, calendar, readline, opcache introspection, DNS).

---

## Divergences

Known, deliberate deviations from upstream PHP are catalogued in
[`PHPR_DIVERGENCES_FROM_PHP.md`](PHPR_DIVERGENCES_FROM_PHP.md). The governing
principle is **correct-or-absent**: a function that lies is worse than one that is
missing, so partial/incorrect builtins are not registered.

---

## How these numbers are produced

```sh
# function coverage
php -r 'foreach(get_defined_functions()["internal"] as $f) echo "$f\n";' | sort > oracle.txt
# probe each with function_exists() inside phpr → have.txt
comm -23 oracle.txt have.txt        # → missing set

# corpus
phpt-runner --isolate "…/php-8.5.7/Zend/tests"
```
