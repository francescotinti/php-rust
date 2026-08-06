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
> measured arc of specializing-interpreter and GC work, guided by
> owner-level attribution of both bytes and CPU-seconds (sampled call
> trees reconciled against the master clock), has brought the WordPress
> media benchmark to **~2.61×** the oracle's CPU and cut the
> peak-footprint gap **from 11.9× to ~4.07×**; the latest levers — a
> keyless hashed-array index (single Zend-style table, no duplicated
> key), then an array-storage tranche (header diet to an exact 96B
> allocator bin, index elision for small hashes, and a bounded
> block-recycling arena) — cut peak footprint another −1.8% and −1.05%
> in successive sessions at flat benchmark CPU; the full WordPress-suite
> CPU sits at **~2.06–2.11×** (from 3.4×; the array tranche cost
> +1–2.5% on the full suite only, kept and on the books per the
> no-revert policy). Every value channel (strings, arrays, objects) is
> now measured with exact live byte-accounting — next: attributing the
> physical footprint that lives *outside* those channels, then Laravel
> validation.

## Coverage at a glance

| | |
| --- | --- |
| Core / language stdlib functions | **539 / 654 (82%)** |
| All internal functions | 1017 / 2143 (47%) |
| Zend test corpus passing | **2650** (65.2% of runnable) |
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
  php-cli/       the `phpr` binary (also a lib: exposes the cli-server SAPI)
  php-server/    standalone HTTP server binary over the same SAPI
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

## Quick-start: serve WordPress with `php-server`

`php-server` is a standalone binary over the same oracle-pinned cli-server
SAPI as `phpr -S` (sequential request handling, POST/cookies/uploads,
`$_SERVER`, static files, and the `index.php` PATH_INFO fallback that serves
WordPress pretty permalinks and `/wp-json/` without a router script). It was
validated against `phpr -S` byte-for-byte on a WordPress front page, login
(POST + auth cookie), wp-admin dashboard, a pretty permalink, and a static
asset.

```sh
cargo build --release            # binaries land in the configured target-dir
# prerequisite, one line: a MySQL reachable at the host:port in wp-config.php
# with the WordPress database already installed.
php-server --port 8080 --docroot /path/to/wordpress
# then browse http://127.0.0.1:8080/  (wp-admin included)
```

Defaults are `--host 127.0.0.1 --port 8080 --docroot .`; an optional
positional `router.php` argument is honoured exactly like `phpr -S`.

## Roadmap

Near-term, highest-leverage work (see [COVERAGE.md](COVERAGE.md) for the data,
[TODO.md](TODO.md) for the full list):

1. **Performance** — the WordPress suite is at parity; a data-driven
   specializing-interpreter arc (typed fast paths, bigram-fused opcodes,
   scope-aware inline caches, call-site specialization, Zend-style fast
   shutdown) took the media benchmark from 4.1× to **~2.61×** the
   oracle's CPU, and the **memory-attribution arc** (exact
   reached-vs-live reconciliation over every VM root) cut the
   peak-footprint gap **from 11.9× to ~4.16×**; the full-suite CPU
   residual was attributed — measured, not suspected — first to the
   cycle-collector classify walk (four successive levers: Zend-style
   purge of refcount-dead roots, closure of the statement-sweep
   fast-path band, fusion of the classifier's bookkeeping tables,
   epoch-guarded in-node walk marks — census classify −42.7%, every
   count conserved), then — via the same owner-level method applied to
   CPU-seconds (sampled call trees reconciled against the master
   clock) — to the reflection-descriptor memo (re-keyed on the
   *declaring* class: **−7.4% CPU** alone) and to an O(n²) `.=` append
   gap closed by a growable string representation with in-place
   append at unique refcount (probe 499ms → 2ms = oracle), and — first
   tranche of the heap-to-handle arc — a keyless hashed-array index
   (single Zend-style table, no duplicated key: −62B/array measured on
   4.75M array deaths, −1.8% peak footprint, −2.7% full-suite CPU
   same-evening at zero media-CPU cost): the full suite stands at
   **~2.06×** (from 3.4×), peak footprint at **~4.08×**. Plan:
   NEXT_SESSION_WORDPRESS.md.
2. **Laravel** as the second framework validation target once the perf
   pass lands.
3. Remaining extension surfaces on demand — ext/tidy (one WP test dataset),
   xmlwriter, calendar, sockets.

Longer-term direction (server SAPI, async, single-binary distribution):
[doc/architecture/ASYNC_AND_DISTRIBUTION_ROADMAP.md](doc/architecture/ASYNC_AND_DISTRIBUTION_ROADMAP.md) ·
extension strategy:
[doc/architecture/EXTENSIONS_ARCHITECTURE.md](doc/architecture/EXTENSIONS_ARCHITECTURE.md).
Historical plans, external reviews and one-off analyses live under
[doc/](doc/README.md).

---

*Not affiliated with the PHP project. Built to understand PHP by rebuilding it.*
