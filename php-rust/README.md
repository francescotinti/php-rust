# phpr — PHP 8.5.7, reimplemented in Rust

`phpr` is a from-scratch implementation of the PHP language and runtime in Rust:
a lexer, compiler, bytecode VM, and a growing standard library — no C PHP linked
in. The goal is to run **real PHP applications** byte-identically to the reference
interpreter, not to pass a toy subset.

> **Status:** Composer, PHPUnit 13.2, Doctrine ORM/DBAL, PDO/SQLite, Monolog and
> Symfony components already execute under `phpr` with output identical to
> upstream PHP 8.5.7.

## Coverage at a glance

| | |
| --- | --- |
| Core / language stdlib functions | **483 / 654 (73%)** |
| All internal functions | 641 / 2143 (30%) |
| Zend test corpus passing | **2325** (61% of runnable) |

Full, measured breakdown → **[COVERAGE.md](COVERAGE.md)**.
The 30%→73% spread is the whole story: the *language* is largely done; the
remaining gap is mostly un-started **database / crypto / network extensions**
(pgsql, mysqli, sodium, gmp, ldap, sockets, …), not missing language features.

## What works

- **Language:** PHP 8.4/8.5 features — enums, `readonly`, first-class callable
  syntax, property hooks, asymmetric visibility, lazy objects, fibers,
  generators, attributes, `match`, named args, `never`/DNF types.
- **Runtime:** a real **cycle-collecting GC**, exceptions, `include`/`require`/
  `eval`, autoloading, output buffering, `strict_types` per-unit.
- **Reflection:** framework-grade — types, attributes, enums, union/intersection.
- **Real apps:** Composer (`install`/`require`/`diagnose`, real HTTPS via rustls),
  PHPUnit 13.2, Doctrine ORM + DBAL (3769/0/0), PDO + `pdo_sqlite` + SQLite3 on
  `rusqlite`, Monolog, Symfony String/Console.

## Design principle: correct-or-absent

A builtin that returns *plausible but wrong* results is worse than one that
doesn't exist — it makes bugs silent. So `phpr` only registers a function once it
is verified **byte-identical to PHP 8.5.7** on a battery of cases. Everything
intentionally left out, and every known deviation, is documented in
[PHPR_DIVERGENCES_FROM_PHP.md](PHPR_DIVERGENCES_FROM_PHP.md).

## Repository layout

```
crates/
  php-types/     Zval, type coercion & comparison semantics
  php-builtins/  pure stdlib functions (fn(args, ctx) -> Zval)
  php-runtime/   the VM: compiler, bytecode, host builtins, OOP, PDO, GC
  php-cli/       the `phpr` binary
```

## Build & run

Build artifacts must live off the (external) source volume — `.cargo/config.toml`
already points `target-dir` at a local disk, so a plain build works:

```sh
cargo build --release
./target/release/phpr script.php
```

Run the upstream Zend conformance suite:

```sh
./target/release/phpt-runner --isolate /path/to/php-8.5.7/Zend/tests
```

## Roadmap

Near-term, highest-leverage work (see [COVERAGE.md](COVERAGE.md) for the data):

1. **bcmath** (14 fns) & **gmp** (51 fns) — self-contained, high value for real apps.
2. Remaining **core stdlib** gaps — streams filters, timezone objects, calendar.
3. A discrete extension port: **session**, **zip**, or **finfo**.

---

*Not affiliated with the PHP project. Built to understand PHP by rebuilding it.*
