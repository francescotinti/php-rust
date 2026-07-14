# phpr — PHP 8.5.7, reimplemented in Rust

`phpr` is a from-scratch implementation of the PHP language and runtime in Rust:
a lexer, compiler, bytecode VM, and a growing standard library — no C PHP linked
in. The goal is to run **real PHP applications** byte-identically to the reference
interpreter, not to pass a toy subset.

> **Status:** Composer, PHPUnit 13 (including **process isolation** — child
> runs spawned via `phpr -d … <stdin>`), Doctrine ORM/DBAL, PDO/SQLite, Monolog
> and Symfony components already execute under `phpr` with output identical to
> upstream PHP 8.5.7. **ext/session is complete** (all 23 functions + the
> SessionHandler class family); symfony/http-foundation runs at **zero errors**
> without its Session suite (12 failures need a real HTTP server).
> **symfony/http-kernel is CLOSED**: the full 1663-test suite passes with
> **0 errors / 0 failures** — parity with the reference interpreter. Along
> the way phpr gained **real IANA timezone support** (TZif reader over the
> system zoneinfo database, DST gap/fold semantics pinned to timelib,
> zone-aware `DateTime` arithmetic), PHP 8.4 `{closure:Scope():line}` names,
> `eval()` scope sharing, magic-method trampolines for first-class callables,
> ArrayAccess protocol dispatch in nested `isset`/`??`, real `flock(2)`
> advisory locks, Zend-faithful destructor timing (eager sweep after
> every statement), dynamic `global $$name` binding, and SEND_VAR_EX
> semantics for by-ref parameters of cross-unit / dynamic calls.
> **wp-cli runs end-to-end from source** (`wp --info` at oracle parity) —
> the WordPress track is underway (see Roadmap).

## Coverage at a glance

| | |
| --- | --- |
| Core / language stdlib functions | **522 / 654 (80%)** |
| All internal functions | 785 / 2143 (37%) |
| Zend test corpus passing | **2493** (61.5% of runnable) |

Full, measured breakdown → **[COVERAGE.md](COVERAGE.md)**.
The 37%→80% spread is the whole story: the *language* is largely done; the
remaining gap is mostly un-started **database / crypto / network extensions**
(pgsql, mysqli, sodium, ldap, sockets, …), not missing language features.

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
- **Real apps:** Composer (`install`/`require`/`diagnose`, real HTTPS via rustls),
  PHPUnit 11.5/13.2/13.3, Doctrine ORM + DBAL (3769/0/0), **symfony
  http-foundation (full suite, 0 errors)** + String/Console/Process, PDO +
  `pdo_sqlite` + SQLite3 on `rusqlite`, Monolog.

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

1. **WordPress** (roadmap reordered: WP before Laravel — it dominates the
   web). symfony/http-kernel is **closed at 0/0** and **wp-cli from source
   already runs end-to-end** (`wp --info` / `wp cli version` at oracle
   parity). Next: `wp core download` + the official SQLite integration
   plugin (runs on the already-green PDO/SQLite), then a real server SAPI
   (superglobals, headers, multipart), then `mysqli` and media
   (gd/exif/zip). Plan: NEXT_SESSION_WORDPRESS.md.
2. ext/session tail — trans-sid URL rewriting, the `SID` constant, shared-ref
   (`r:`) unserialize.
3. Remaining **core stdlib** gaps — stream filters (userland), timezone
   objects, calendar.

Longer-term direction (server SAPI, async, single-binary distribution):
[ASYNC_AND_DISTRIBUTION_ROADMAP.md](ASYNC_AND_DISTRIBUTION_ROADMAP.md) ·
extension strategy: [EXTENSIONS_ARCHITECTURE.md](EXTENSIONS_ARCHITECTURE.md).

---

*Not affiliated with the PHP project. Built to understand PHP by rebuilding it.*
