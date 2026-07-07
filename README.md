# php-rust

*Read this in [Italiano](italiano.md).*

> **PHP, reimplemented from scratch in Rust.** A modern, memory-safe, async-ready
> PHP 8.5 runtime — driven by observable behavior, not by the internal architecture
> of the Zend Engine.

```bash
phpr script.php        # a drop-in for `php`, but it's Rust all the way down
```

---

## 💡 The idea

The Zend Engine — the heart of PHP — is ~280,000 lines of C that have piled up since 1999. It
carries manual memory management, a custom garbage collector, a thread-safety layer (TSRM), a
macro-generated VM, and a convoluted JIT. It's battle-tested but brittle: entire classes of
vulnerabilities (*use-after-free*, *buffer overflow*) live there by construction.

The insight behind the project is to flip the problem on its head:

> **The contract worth preserving isn't Zend's *design*, but PHP's *observable output*.**

And that output already has a perfect oracle: the **~21,500 official `.phpt` tests** shipped with
the PHP source. Any runtime that produces the exact same output *is* PHP. This turns the job from
*"translating C"* into *"spec-driven reimplementation"*, where you only read the C to pin down the
semantics in the ambiguous cases.

The result is an engine where Rust does the heavy lifting at zero cost: **ownership** replaces
`zend_alloc`, `Rc`+copy-on-write replaces manual refcounting, `Send`/`Sync` make multi-threading a
property of the type instead of a subsystem (TSRM), and a resident process makes the engine
async-ready by construction.

---

## 🎯 The goal

A PHP runtime that is, in order:

1. **Faithful** — bug-for-bug compatible with PHP 8.5 on the official `.phpt` corpus (including the
   quirks of type juggling, legacy warnings, and byte-identical stack traces).
2. **Safe** — no segfaults at the core level; the C memory-bug classes eliminated by Rust's type
   system.
3. **Modern** — shippable as a **single binary** (the Go/Deno effect), with a built-in native web
   server and a **natively async, multi-threaded** foundation — moving past PHP's historical
   *shared-nothing / single-threaded* limitation.

The real test bench isn't a microbenchmark: it's **running Composer** and then serving a *Hello
World* route on **Laravel/Symfony**. Those milestones stress OOP, autoloading, and Reflection far
more than any synthetic test.

---

## 🗺️ Roadmap

| Phase | Milestone | Status |
|---|---|---|
| **1. Semantic core** | Type juggling faithful to the oracle (`zend_operators.c`), `==`/`===`, coercions | ✅ Done |
| **2. Full language** | Expressions, control flow, functions, arrays, references, closures | ✅ Done |
| **3. OOP** | Classes, inheritance, visibility, `static`/LSB, magic methods, enums, traits, interfaces | ✅ Done |
| **4. Exceptions & errors** | `try/catch/finally`, catchable engine errors, stack traces, line tracking | ✅ Done |
| **5. Bytecode VM** | Generators, `yield from`, Fibers on explicit frames — **no `unsafe`, no stackful coroutines** | ✅ Done |
| **6. Memory** | Cycle collector for circular references (the other big "dragon") | ✅ Done |
| **7. Standard library** | ~500 builtins: array/string/math/json/preg/mbstring/hash/file/stream/date… | ✅ Substantial (long tail in progress) |
| **8. Real Composer** | `composer require monolog/monolog` **end-to-end**: resolution, HTTPS download (rustls), unzip, autoload — and the package **runs** | ✅ Done |
| **8b. Real ecosystem** | **PHPUnit 13.2 green, byte-identical**; Doctrine **DBAL 3769 tests / 0 err / 0 fail** on native PDO+sqlite; **ORM 3484 tests / 12 err**; Monolog, collections, inflector, instantiator… | ✅/🔄 In progress |
| **9. Framework bootstrap** | *Hello World* on Laravel / Symfony | ⏳ Next |
| **10. Async & single-binary** | Tokio event loop + resident Axum web server, standalone distribution | ⏳ Future |
| **11. JIT (Tier 3)** | Clean bytecode → Cranelift/LLVM for on-the-fly machine code | 🔭 Vision |

---

## 🏗️ Architecture

A single production engine: a **bytecode VM**. Source flows through
`parser (mago) → AST → HIR → bytecode → VM dispatch loop`. (The project started with a
tree-walker, later removed once the VM reached full parity: see
[HISTORY.md](HISTORY.md).)

```
php-rust/crates/
  php-types      Zval / PhpStr / PhpArray / Object + operators (the soul of PHP:
                 type juggling, full-port from zend_operators.c). Zero internal dependencies.
  php-runtime    HIR + lowering from `mago`, and the bytecode VM:
                 compile.rs (HIR→bytecode) + vm/{mod,exceptions,coroutines,arrays,oop,calls}.rs
  php-builtins   registry of ~380 pure builtins (var_dump, array_*, sprintf, json_*, preg_*,
                 mb_*, hash/encoding, file/stream, …) + ~120 host builtins VM-side
                 (reflection, callable, PDO/sqlite, dom/xml, curl, proc_open, …)
  php-cli        the `phpr` binary — drop-in for `php`, CLI-faithful streams + faithful exit code
  php-server     native web server (Axum + Tokio) — the bridgehead toward async
  phpt-runner    runs the official `.phpt` tests with capability scan and unified diff vs oracle
diary/           methodological journal: 00-reconnaissance … 99-conclusions + metrics
```

**Why Rust collapses Zend** — the structural payoff, in numbers:

| Zend subsystem | C LOC | Rust replacement | Rust LOC |
|---|---:|---|---:|
| Generated VM + `zend_execute.c` | ~146,000 | bytecode VM (single engine) | inside `php-runtime` |
| `zend_compile.c` (AST→opcodes) | ~12,400 | AST→HIR lowering + compile.rs | inside `php-runtime` |
| re2c lexer + Bison parser + AST | ~25,000 | `mago` dependency + bridge | ~500 |
| `zend_alloc` / `zend_gc` / TSRM / opcache / win32 | ~88,000 | ownership, `Rc`+COW, `Send`/`Sync` + cycle collector | ~1,000 |
| `zend_operators.c` (type juggling) | ~3,900 | faithful full-port | ~1,500 |

**~280K LOC of core C (extensions not counted) → ~68K LOC of total Rust today** — engine, stdlib,
PDO/sqlite, dom/xml, TLS, and tooling included. The ~4:1 ratio holds even as functionality has
grown by an order of magnitude over the first estimates.

---

## 📍 Where we are

The **core language is complete and faithful**: all of control flow, functions, arrays, the
reference system, closures, **full OOP** (classes, inheritance, visibility, `static` + late static
binding, magic methods, enums, traits, framework-grade Reflection), **exceptions** (including
byte-identical stack traces and catchable engine errors), **generators** and **Fibers** — the
latter implemented by parking frames on an explicit VM stack, **with no `unsafe` and no stackful
coroutines**. The hard parts of modern PHP are here too: PHP 8.4 **property hooks** and **lazy
objects** (ghost/proxy), first-class callables, `strict_types` resolved per-unit from the call site.

But the real leap is that **the real ecosystem runs**:

- **Composer** installs packages end-to-end: resolution, **native HTTPS** download
  (ureq + rustls), native unzip, autoloader dump — and the installed package **executes**.
- **PHPUnit 13.2** boots and produces output **byte-identical** to the oracle.
- **Doctrine DBAL: 3769 tests, 0 errors, 0 failures** — on a **native Rust implementation of
  PDO / pdo_sqlite / ext-sqlite3** (bundled rusqlite, with SQLSTATE/errmode/metadata semantics
  verified one by one against the oracle).
- **Doctrine ORM: 3484 tests, 12 errors / 22 failures** (and falling) — hydration, UnitOfWork,
  XML mapping included. Collections, inflector, lexer, event-manager, instantiator: **green**.
- Extensions modeled without C: `pdo`, `pdo_sqlite`, `sqlite3`, `dom`, `libxml`,
  **`simplexml`** (on the Rust DOM), `curl` (easy-API on ureq), `openssl`/TLS (rustls),
  `zip`, `mbstring`, `pcre` (3 engines), `hash`, `json`, `pcntl`, `posix`, `ctype`.

Two of the three historical "dragons" of a PHP port have already been tamed:

- 🐉 **Circular references** → a Zend-style **cycle collector** (*possible-roots* algorithm),
  with O(candidates) sweep: a pathological test of 87,380 cyclic objects went from ~11s to ~0.25s.
- 🐉 **Bug-for-bug compatibility** → the entire strategy is anchored to the `.phpt` corpus and the
  real framework suites, the only real shield against regressions.

The third dragon — the **C extension ecosystem (PECL)** — is being tackled by targeted native
rewrites (PDO/sqlite, dom/simplexml, and curl have already fallen that way); a compatibility FFI
layer remains the long-term option for the tail.

**Fidelity** (at HEAD `e0b5080`, 2026-07-07): differential type-juggling vs real PHP at
**0 mismatches** (37,835 cases — this is the *operator* differential, a metric distinct from the
`.phpt` corpus); 20 green Rust crate suites; on the official `Zend/tests` corpus **2,138 phpt pass**
(58% of the runnable ones, growing every session, with a "zero pass→fail" gate on every commit);
~650 commits of history tracked session by session.

> The detailed history of the ~70 build steps lives in **[HISTORY.md](HISTORY.md)**; the
> replicable methodological journal is in **[diary/](diary/)**.

---

## 🚀 Next steps

1. **Close the last language constructs** — the bulk of the "compile-unsupported" bucket is already
   back in (`--run-skipif`, dynamic named/spread args, variable variables, `C::{$expr}`, faithful
   compile-time fatals: corpus 2,071→2,138); the next block is **by-ref property hooks**
   (`&get`, PHP 8.4) and the remaining tail.
2. **Doctrine ORM to zero** — the last 12 errors are triaged (XSD `schemaValidate`, typed props on
   lazy proxies, singletons); the bulk of the work is done.
3. **Framework bootstrap** — *Hello World* on Laravel/Symfony: the ultimate stress test for
   autoloading and Reflection, now within reach given that PHPUnit and Doctrine already run.
4. **Robustness** — convert user-input-reachable `unwrap`/`expect` into typed VM errors + fuzz the
   `lower/compile` pipeline, for a *no-panic* guarantee.
5. **The async leap** — integrate a **Tokio** event loop and consolidate `php-server` (Axum) into a
   resident runtime, toward a natively concurrent PHP and a shippable **single binary**.

---

## 🛠️ Quickstart

```bash
cd php-rust
cargo run -p php-cli -- script.php       # run a script with `phpr`
cargo test                               # unit + integration tests

# Differential vs oracle (requires a php binary; auto-skips if absent):
PHP_ORACLE=/path/to/php cargo test -p php-types --test differential

# Run the official .phpt corpus through the VM:
cargo run -p phpt-runner -- /path/to/php-src/tests /path/to/php-src/Zend/tests
cargo run -p phpt-runner -- --isolate --list-fails <path>   # one test = one sub-process, with diff
```

Diagnostics: `PHP_RUST_TRACE=hir|body|exec|all phpr script.php` prints the lowered HIR and/or the
execution trace to **stderr**, without polluting the stdout compared against the oracle.

---

## 🤝 Contributing

The idea of *"rewriting PHP in Rust to make it async and safe"* is a magnet for the Rust community.
The best way to contribute once you've found your footing: pick a missing builtin or a group of
failing `.phpt` tests (`phpt-runner --list-fails`), reproduce them against the oracle, and close
the gap while staying byte-identical. The project's golden rule: **the oracle is always right.**

## 📄 License

MIT.
