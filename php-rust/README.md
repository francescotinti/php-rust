# phpr — PHP 8.5.7, reimplemented in Rust

`phpr` is a from-scratch implementation of the PHP language and runtime in Rust:
a lexer, compiler, bytecode VM, and a growing standard library — no C PHP linked
in. The goal is to run **real PHP applications** byte-identically to the reference
interpreter, not to pass a toy subset.

> **Status:** Composer, PHPUnit 13, Doctrine ORM/DBAL, PDO/SQLite, Monolog and
> Symfony components already execute under `phpr` with output identical to
> upstream PHP 8.5.7. The complete **symfony/http-foundation** test suite runs
> at **zero errors** (the only 12 remaining failures need a real HTTP server).

## Coverage at a glance

| | |
| --- | --- |
| Core / language stdlib functions | **514 / 654 (79%)** |
| All internal functions | 754 / 2143 (35%) |
| Zend test corpus passing | **2352** (61% of runnable) |

Full, measured breakdown → **[COVERAGE.md](COVERAGE.md)**.
The 35%→79% spread is the whole story: the *language* is largely done; the
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
  output capture works), `strict_types` per-unit.
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

1. **ext/session** (23 fns + the SessionHandler class family) — unblocks the
   371 `Tests/Session` cases of symfony http-foundation, prerequisite for
   HttpKernel.
2. **symfony/http-kernel** — the Request→Response cycle, next component in the
   Symfony porting track.
3. Remaining **core stdlib** gaps — stream filters (userland), timezone
   objects, calendar.

Longer-term direction (server SAPI, async, single-binary distribution):
[ASYNC_AND_DISTRIBUTION_ROADMAP.md](ASYNC_AND_DISTRIBUTION_ROADMAP.md) ·
extension strategy: [EXTENSIONS_ARCHITECTURE.md](EXTENSIONS_ARCHITECTURE.md).

---

*Not affiliated with the PHP project. Built to understand PHP by rebuilding it.*
