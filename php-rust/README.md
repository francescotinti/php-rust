# phpr — PHP 8.5.7, reimplemented in Rust

`phpr` is a from-scratch implementation of the PHP language and runtime in Rust:
a lexer, compiler, bytecode VM, and a growing standard library — no C PHP linked
in. The goal is to run **real PHP applications** byte-identically to the reference
interpreter, not to pass a toy subset.

> **Status: the entire WordPress core test suite runs at effective oracle
> parity.** Single-site (30,472 tests, wordpress-develop trunk) differs from
> the reference interpreter by **a single test name — one deliberate,
> catalogued divergence** — and multisite (31,278 tests) confirms the same
> **single divergence**, both stable by name across runs.
> WordPress 7.0.1 installs and serves on **real MySQL** (native `mysqli`
> wire protocol) through the built-in `phpr -S` server SAPI — front pages,
> login, REST, pretty permalinks and wp-admin **byte-identical** over HTTP;
> the media pipeline reaches byte parity via the **system
> libgd/libxslt/libtidy through FFI** (ext/tidy is complete, 24/24
> functions; ext/xsl has a real `registerPHPFunctions` trampoline). Also at
> parity: Composer, PHPUnit 9/11/13 (including process isolation), Doctrine
> ORM/DBAL, PDO/SQLite, Monolog, wp-cli, **symfony/http-kernel CLOSED at 0
> errors / 0 failures** (1665 tests) and http-foundation. The runtime has
> real IANA timezones (system TZif, timelib gap/fold semantics), a
> cycle-collecting GC with Zend-style adaptive thresholds and Zend-faithful
> destructor timing, property hooks, lazy objects, fibers, and an
> opcache-like per-request unit cache. Current front: **performance** — a
> measured arc of specializing-interpreter work has brought the WordPress
> media benchmark from 4.1× to **2.71×** the oracle's CPU and the full-suite
> master CPU to **2.11×** — then Laravel validation.

## Coverage at a glance

| | |
| --- | --- |
| Core / language stdlib functions | **539 / 654 (82%)** |
| All internal functions | 1017 / 2143 (47%) |
| Zend test corpus passing | **2609** (64.3% of runnable) |
| WordPress core suite | **effective parity** (single-site AND multisite: **1** declared name-diff each) |

Full, measured breakdown → **[COVERAGE.md](COVERAGE.md)**.
The 47%→82% spread is the whole story: the *language* is largely done; the
remaining gap is mostly un-started **database / crypto / network extensions**
(pgsql, sodium, ldap, sockets, odbc, …), not missing language features.

## What works

- **Language:** PHP 8.4/8.5 features — enums, `readonly`, first-class callable
  syntax, property hooks, asymmetric visibility, lazy objects, fibers,
  generators, attributes, `match`, named args, `never`/DNF types, Zend late
  binding for class declarations, `Closure::bind` scope rebinding, union-type
  weak coercion in Zend's preference order.
- **Runtime:** a real **cycle-collecting GC**, exceptions, `include`/`require`/
  `eval`, autoloading, output buffering **with handler phases** (PHPUnit's
  output capture works; diagnostics flow through the buffer stack like PHP's),
  `strict_types` per-unit, a mutable **INI table** (`ini_set`, `php -d`-style
  CLI overrides, phpt `--INI--` sections), **ext/session** on the files
  handler with user save handlers.
- **Reflection:** framework-grade — types, attributes, enums, union/intersection.
- **Real apps:** **WordPress 7.0.1 on real MySQL** (installed, served,
  full core suite at effective parity — single-site and multisite), wp-cli,
  Composer (`install`/`require`/`diagnose`, real HTTPS via rustls),
  PHPUnit 9.6/11.5/13.2/13.3, Doctrine ORM + DBAL (3769/0/0), **symfony
  http-kernel (1665 tests, 0/0)** and http-foundation + String/Console/
  Process, PDO + `pdo_sqlite` + SQLite3 on `rusqlite`, Monolog.
- **Extensions on system libraries via FFI** (byte parity with the oracle's
  own dylibs): zlib, **gd** (+exif), **libxslt** (XSLTProcessor); native
  Rust: **mysqli** (wire protocol), fileinfo, ext/xml SAX, DOM
  loadHTML/saveHTML, session, bcmath, gmp, tokenizer.

## Design principle: correct-or-absent

A builtin that returns *plausible but wrong* results is worse than one that
doesn't exist — it makes bugs silent. So `phpr` only registers a function once it
is verified **byte-identical to PHP 8.5.7** on a battery of cases. Everything
intentionally left out, and every known deviation, is documented in
[PHPR_DIVERGENCES_FROM_PHP.md](PHPR_DIVERGENCES_FROM_PHP.md).

## Repository layout

```
crates/
  php-types/     Zval, type coercion & comparison semantics, streams, zlib FFI
  php-builtins/  pure stdlib functions (fn(args, ctx) -> Zval)
  php-runtime/   the VM: compiler, bytecode, host builtins, OOP, PDO, GC
  php-cli/       the `phpr` binary
  phpt-runner/   the .phpt conformance harness (differential vs upstream)
```

## Build & run

Build artifacts must live off the (external) source volume — `.cargo/config.toml`
already points `target-dir` at a local disk, so a plain build works:

```sh
cargo build --release
phpr script.php          # binary lands in the configured target-dir
```

Run the upstream Zend conformance suite:

```sh
phpt-runner --isolate /path/to/php-8.5.7/Zend/tests
```

## Roadmap

Near-term, highest-leverage work (see [COVERAGE.md](COVERAGE.md) for the data,
[TODO.md](TODO.md) for the full list):

1. **Performance** — the WordPress suite is at parity; a data-driven
   specializing-interpreter arc (typed fast paths, bigram-fused opcodes,
   scope-aware inline caches, call-site specialization, Zend-style fast
   shutdown) has taken the media benchmark from 4.1× to **2.71×** the
   oracle's CPU and the full-suite master CPU to **2.11×** (12 min vs 5:39).
   Next: the GC note/demote churn (in-object buffer flag) and live-data
   memory footprint. Plan: NEXT_SESSION_WORDPRESS.md.
2. **Laravel** as the second framework validation target once the perf
   pass lands.
3. Remaining extension surfaces on demand — ext/tidy (one WP test dataset),
   xmlwriter, calendar, sockets.

Longer-term direction (server SAPI, async, single-binary distribution):
[ASYNC_AND_DISTRIBUTION_ROADMAP.md](ASYNC_AND_DISTRIBUTION_ROADMAP.md) ·
extension strategy: [EXTENSIONS_ARCHITECTURE.md](EXTENSIONS_ARCHITECTURE.md).

---

*Not affiliated with the PHP project. Built to understand PHP by rebuilding it.*
