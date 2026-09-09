# php-rust

*Read this in [Italiano](italiano.md).*

*🌐 Official website: **[phprust.com](https://phprust.com/)***

*📊 Live [measured coverage](php-rust/COVERAGE.md) — function coverage per extension, the Zend `.phpt` corpus, and the real-world stacks that run byte-identically.*

> **PHP, reimplemented from scratch in Rust.** A modern, memory-safe, async-ready
> PHP 8.5 runtime — driven by observable behavior, not by the internal architecture
> of the Zend Engine.

```bash
phpr script.php        # a drop-in for `php`, but it's Rust all the way down
```

> **Status (2026-09-10, session S-172 pin).** The **entire WordPress core test
> suite** (30,472 tests single-site, 31,278 multisite, wordpress-develop trunk)
> runs at **effective oracle parity** — one deliberate, catalogued name-diff — on
> **real MySQL** through a native `mysqli` wire protocol and the built-in server
> SAPI. Composer, PHPUnit 9/11/13, Doctrine ORM/DBAL, symfony/http-kernel (0/0),
> http-foundation, wp-cli and Monolog run at parity too. **The current front is
> performance**, measured on a six-category micro benchmark against the
> reference interpreter's CPU: **arith 2.7× · regex 2.5× · array 3.0× ·
> property 3.8× · string 4.1× · calls 4.7×** (from 9.3 / 3.5 / 3.9 / 7.9 / 5.3
> / 5.1 in August), the full WordPress suite at **~1.77×** the oracle's CPU.
> Target: **parity (1×)**; the ≤3× stage is reached on arith and regex, with array at the threshold.

---

## 💡 The idea

The Zend Engine — the heart of PHP — is ~270,000 lines of C that have piled up since 1999. It
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

The real test bench isn't a microbenchmark: it's **running Composer**, then a full framework
test suite, then a real application end-to-end. Those milestones stress OOP, autoloading, and
Reflection far more than any synthetic test — and WordPress on real MySQL was the first
application to fall.

---

## 🗺️ Roadmap

| Phase | Milestone | Status |
|---|---|---|
| **1. Semantic core** | Type juggling faithful to the oracle (`zend_operators.c`), `==`/`===`, coercions | ✅ Done |
| **2. Full language** | Expressions, control flow, functions, arrays, references, closures | ✅ Done |
| **3. OOP** | Classes, inheritance, visibility, `static`/LSB, magic methods, enums, traits, interfaces, PHP 8.4 property hooks, asymmetric visibility, lazy objects | ✅ Done |
| **4. Exceptions & errors** | `try/catch/finally`, catchable engine errors, stack traces, line tracking | ✅ Done |
| **5. Bytecode VM** | Generators, `yield from`, Fibers on explicit frames — **no `unsafe`, no stackful coroutines** | ✅ Done |
| **6. Memory** | Zend-model cycle collector over objects and containers, adaptive thresholds, Zend-faithful destructor timing | ✅ Done |
| **7. Standard library** | 1017 registered internal functions (measured; core language stdlib 539/654 = 82%): array/string/math/json/preg/mbstring/hash/file/stream/date/session… | ✅ Substantial (long tail deferred) |
| **8. Real Composer** | `composer require monolog/monolog` **end-to-end**: resolution, HTTPS download (rustls), unzip, autoload — and the package **runs** | ✅ Done |
| **8b. Real ecosystem** | **PHPUnit 9/11/13 byte-identical** (incl. process isolation); Doctrine **DBAL 3769 / 0 / 0**, **ORM 3484 tests at 3 err / 13 fail, stable by name**; **symfony/http-kernel CLOSED 1665 tests 0/0**, http-foundation 0 errors; Monolog, wp-cli, collections, inflector, instantiator… | ✅ Done |
| **9. Real application** | **WordPress 7.0.1 on real MySQL** (native `mysqli`), served by the built-in SAPI byte-identically; **full core PHPUnit suite at effective parity, single-site AND multisite**; media pipeline at byte parity on system libgd/libxslt/libtidy via FFI | ✅ Done |
| **10. Performance** | Parity with the oracle's CPU. Stage ≤3× per micro-category: reached on arith and regex, array at the threshold; property, string and calls in progress. WordPress full suite ~1.77× | 🔄 **Current front** |
| **11. Second framework** | Laravel as the next validation target (same gate recipe as ORM/http-kernel) | ⏳ Queued |
| **12. Async & single-binary** | Tokio event loop + resident Axum web server, standalone distribution | ⏳ Future |
| **13. JIT (Tier 3)** | Clean bytecode → Cranelift/LLVM for on-the-fly machine code | 🔭 Vision |

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
                 compile/ (HIR→bytecode, superinstruction fusion) + vm/ (dispatch loop,
                 inline caches, exceptions, coroutines, arrays, OOP, calls, GC)
  php-builtins   registry of pure value builtins (var_dump, array_*, sprintf, json_*, preg_*,
                 mb_*, hash/encoding, file/stream, …); together with the VM-side host
                 builtins (reflection, callable, PDO/sqlite, dom/xml, curl, proc_open,
                 session, mysqli, …) phpr registers 1017 internal functions (measured by probe)
  php-cli        the `phpr` binary — drop-in for `php`, CLI-faithful streams + faithful exit code
  php-server     native web server over the same cli-server SAPI (`phpr -S` semantics)
  phpt-runner    runs the official `.phpt` tests with capability scan and unified diff vs oracle
diary/           methodological journal: 00-reconnaissance … 99-conclusions + metrics
```

**Why Rust collapses Zend** — the structural payoff, in numbers (all LOC
measured with `wc -l` on PHP 8.5.7 and on this repo, 2026-09-10):

| Zend subsystem | C LOC | Rust replacement | Rust LOC |
|---|---:|---|---:|
| Generated VM + `zend_execute.c` | ~136,000 | bytecode VM (single engine): `php-runtime/src/vm/` | ~62,300 |
| `zend_compile.c` (AST→opcodes) | ~12,400 | HIR + lowering + compiler (`hir.rs`, `lower/`, `compile/`) | ~19,200 |
| re2c lexer + Bison parser + AST | ~23,400 | `mago` dependency; the AST→HIR bridge is counted in the lowering row above | — |
| `zend_alloc` / `zend_gc` / TSRM / opcache (incl. JIT) / win32 | ~108,000 | ownership, `Rc`+COW, `Send`/`Sync` + a cycle collector | ~1,000 |
| `zend_operators.c` + numeric strings (type juggling) | ~3,900 | faithful full-port (`ops.rs`, `convert.rs`, `numstr.rs`) | ~1,900 |

**~270K LOC of core Zend C (extensions not counted) → ~112K LOC of engine Rust**
(`php-runtime` + `php-types`). The whole project today is **~153K LOC of Rust**
plus ~9.3K of PHP prelude — and that total also includes the standard library
(~34K), mysqli, PDO/sqlite, dom/xml, TLS and the phpt tooling, functionality
that on the C side lives in `ext/`/`sapi/` and is *not* part of the 270K. The
engine grew by ~45K lines since July: that is the price of the specializing
interpreter (typed fast paths, fused superinstructions, inline caches) that
carried the WordPress suite from 4.1× to ~1.77× the oracle's CPU.

---

## 📍 Where we are

The **core language is complete and faithful**: all of control flow, functions, arrays, the
reference system, closures, **full OOP** (classes, inheritance, visibility, `static` + late static
binding, magic methods, enums, traits, framework-grade Reflection), **exceptions** (including
byte-identical stack traces and catchable engine errors), **generators** and **Fibers** — the
latter implemented by parking frames on an explicit VM stack, **with no `unsafe` and no stackful
coroutines**. The hard parts of modern PHP are here too: PHP 8.4 **property hooks** (by-ref
included), **lazy objects** (ghost/proxy), asymmetric visibility, first-class callables,
`strict_types` resolved per-unit from the call site, real IANA timezones with timelib gap/fold
semantics, and an opcache-like per-request unit cache.

The real leap is that **the real ecosystem runs**:

- **WordPress 7.0.1 runs on real MySQL** (native `mysqli` wire protocol) through the
  built-in server SAPI — wp-admin, front pages, login, REST and pretty permalinks
  **byte-identical over HTTP**. The **full core PHPUnit suite** (30,472 tests
  single-site, 31,278 multisite) is at **effective oracle parity**: a single
  deliberate, catalogued name-diff, stable by name across every run. The media
  pipeline reaches byte parity via **system libgd / libxslt / libtidy through FFI**
  (+ native exif, fileinfo). **wp-cli** runs from source at parity.
- **Composer** installs packages end-to-end: resolution, **native HTTPS** download
  (ureq + rustls), native unzip, autoloader dump — and the installed package **executes**.
- **PHPUnit 9.6 / 11.5 / 13** boot and produce output **byte-identical** to the oracle,
  process isolation included (child runs spawn `phpr`).
- **Doctrine DBAL: 3769 tests, 0 errors, 0 failures** — on a **native Rust implementation of
  PDO / pdo_sqlite / ext-sqlite3** (bundled rusqlite, with SQLSTATE/errmode/metadata semantics
  verified one by one against the oracle). **Doctrine ORM: 3484 tests, 3 errors / 13 failures**,
  declared and stable by name (the remainder is triaged: XSD `schemaValidate`, lazy-proxy edges).
  Collections, inflector, lexer, event-manager, instantiator: **green**.
- **Symfony**: **http-kernel CLOSED — the full 1665-test suite at 0 errors / 0 failures**
  (DI container compiles, dumps and reloads; Zend-faithful destructor timing);
  http-foundation full suite at 0 errors; String / Console / Process green.
- Extensions modeled without C: `pdo`, `pdo_sqlite`, `sqlite3`, `mysqli`, `dom`, `libxml`,
  `simplexml`, `xml` (SAX), `curl` (easy-API on ureq), `openssl`/TLS (rustls), `zip`,
  `mbstring`, `pcre`, `hash`, `json`, `session`, `pcntl`, `posix`, `ctype`, `bcmath`, `gmp`,
  `tokenizer`, `fileinfo`, an `intl` subset; on the **system libraries via FFI** (byte parity
  with the oracle's own dylibs): `zlib`, `gd` (+exif), `xsl`, `tidy`.

All three historical "dragons" of a PHP port have been confronted:

- 🐉 **Circular references** → a Zend-model **cycle collector** over objects *and* containers
  (possible-roots buffer, Zend-exact `gc_collect_cycles()` counts, adaptive thresholds), with
  O(candidates) sweep: a pathological test of 87,380 cyclic objects went from ~11s to ~0.25s.
- 🐉 **Bug-for-bug compatibility** → the entire strategy is anchored to the `.phpt` corpus and the
  real framework suites; every promoted build passes a frozen fail-set gate **by name**, in two
  execution modes, plus bilateral fixtures run on both engines.
- 🐉 **The C extension ecosystem (PECL)** → targeted native rewrites where the semantics live in
  PHP strings (PDO/sqlite, mysqli, dom/simplexml, curl), and **FFI to the very same system
  dylibs the oracle uses** where byte parity is a property of the library (gd, xslt, tidy, zlib).

**Fidelity** (at 2026-09-10, pin of session S-172): differential type-juggling vs real PHP at
**0 mismatches** (37,835 cases — the *operator* differential, a metric distinct from the `.phpt`
corpus); **1,748** green Rust unit/integration tests; on the official `Zend/tests` corpus
**2655 phpt pass** (65.3% of the runnable ones, with a frozen "zero pass→fail
by name" gate on every promoted build); WordPress full core suite at parity with **full-suite CPU
at ~1.77×** the oracle (median of the last measured pair, band [1.74; 1.80]).
Live measured coverage: **[php-rust/COVERAGE.md](php-rust/COVERAGE.md)**;
multi-workload perf map: **[php-rust/PERF_MAP.md](php-rust/PERF_MAP.md)**;
current route: **[php-rust/NEXT_SESSION_WORDPRESS.md](php-rust/NEXT_SESSION_WORDPRESS.md)**.

> The detailed history of the ~70 build steps lives in **[HISTORY.md](HISTORY.md)**; the
> replicable methodological journal is in **[diary/](diary/)**; from the WordPress arc onward
> every work session has its own file in **[php-rust/sessions/](php-rust/sessions/)** and its
> perf-gap snapshot in **[php-rust/gaps/](php-rust/gaps/)**.

---

## ⚡ Performance: where the gap is, and how it is being closed

Since August the project runs under a written measurement protocol
([php-rust/REGOLE.md](php-rust/REGOLE.md)): every lever is pre-registered with its
criterion, measured A/B interleaved against the pinned binary with per-binary startup
floors subtracted, guarded on the non-target categories, and promoted only through a
scripted gate (build → hash → test battery → frozen corpus fail-set by name ×2 modes →
bilateral fixtures → micro R=5). Every session ends with an adversarial review; the
whole trail is in `php-rust/sessions/` and `php-rust/gaps/GAP_TREND.md`.

Micro benchmark, same PHP source on both engines, ratio of user CPU (phpr / PHP 8.5.7):

| category | Aug 2026 (S-110) | **Sep 2026 (S-172)** | stage ≤3× |
|---|---:|---:|:---:|
| arith | 9.3× | **2.7×** | ✅ |
| regex | 3.5× | **2.5×** | ✅ |
| array | 3.9× | **3.0×** | ✅ |
| property | 7.9× | **3.8×** | 🔄 |
| string | 5.3× | **4.1×** | 🔄 |
| calls | 5.1× | **4.7×** | 🔄 |

Real applications: **WordPress full suite ~1.77×** the oracle's CPU (from 4.1× at the start
of the arc; the peak-footprint gap went from 11.9× to ~2.3×, last ratio measured in August), **Doctrine ORM suite ~7.1×**
(from 8.4×; object-heavy, the hardest workload on the map).

What the measurements established, in order: the threaded-dispatch hypothesis was refuted
(pure dispatch costs 1.75 ns/op — the same as the whole oracle instruction); the gap lives
in the **body of the handlers**, i.e. in the lifecycle of temporary `Zval`s around every
operation. The lever that followed — a *sealed Long form* that keeps hot integer arithmetic
and property read-modify-write on bare `i64` with no temporaries — halved the arith judge
(46.8 → 23.4 ns/iter) and took property access from 5.2× to 3.8×, with zero `unsafe`. The
same form is now being applied to calls and strings. Vetoed by measurement, and not coming
back: NaN-boxing, function-table dispatch, an object arena, BOLT/PGO.

---

## 🚀 Next steps

1. **Performance to parity** — the route is fixed by measurement: sealed forms on the
   remaining hot handler bodies (property fetch + arithmetic peephole, then the call frame,
   then string concatenation and `substr`), each under its own pre-registered criterion and
   category-level judge. Route and open questions: `php-rust/NEXT_SESSION_WORDPRESS.md`.
2. **Laravel** — the second framework validation target, queued behind the perf front
   (user decision); method = the proven ORM/http-kernel gate recipe.
3. **Doctrine ORM to zero** — 3 errors / 13 failures, stable by name and triaged.
4. **Remaining extension surfaces on demand** — xmlwriter, calendar, sockets; the database /
   crypto / network extensions not started yet (pgsql, sodium, ldap, odbc) are the bulk of
   the 47%→100% function gap, not missing language features.
5. **Robustness** — convert user-input-reachable `unwrap`/`expect` into typed VM errors + fuzz the
   `lower/compile` pipeline, for a *no-panic* guarantee.
6. **The async leap** — integrate a **Tokio** event loop and consolidate `php-server` into a
   resident runtime, toward a natively concurrent PHP and a shippable **single binary**.

---

## 🛠️ Quickstart

```bash
cd php-rust
cargo build --release                    # binaries land in the configured target-dir
phpr script.php                          # run a script
cargo test --release                     # unit + integration tests (1,748)

# Differential vs oracle (requires a php binary; auto-skips if absent):
PHP_ORACLE=/path/to/php cargo test -p php-types --test differential

# Run the official .phpt corpus through the VM:
phpt-runner --isolate /path/to/php-src/Zend/tests
phpt-runner --isolate --list-fails <path>   # one test = one sub-process, with diff

# Serve a PHP application (WordPress included) on the built-in SAPI:
php-server --port 8080 --docroot /path/to/wordpress
```

Diagnostics: `PHP_RUST_TRACE=hir|body|exec|all phpr script.php` prints the lowered HIR and/or the
execution trace to **stderr**, without polluting the stdout compared against the oracle.

---

## 🤝 Contributing

The idea of *"rewriting PHP in Rust to make it async and safe"* is a magnet for the Rust community.
The best way to contribute once you've found your footing: pick a missing builtin or a group of
failing `.phpt` tests (`phpt-runner --list-fails`), reproduce them against the oracle, and close
the gap while staying byte-identical. Deliberate deviations are catalogued by name in
[php-rust/PHPR_DIVERGENCES_FROM_PHP.md](php-rust/PHPR_DIVERGENCES_FROM_PHP.md); a builtin that
returns *plausible but wrong* results is never registered (**correct-or-absent**). The
project's golden rule: **the oracle is always right.**

## 📄 License

MIT.
