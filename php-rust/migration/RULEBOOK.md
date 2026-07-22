# Translation Rulebook — PHP 8.5.7 (C / Zend Engine) → Rust (`phpr`)

> **The meta-rule: if two agents could answer a question differently, the
> answer goes in this file.** Adopted from Anthropic's
> [code-migration-kit](https://github.com/anthropics/code-migration-kit-with-claude-code)
> at session WP-39, retroactively: this port has operated under these rules
> since its start — they lived in session memory and scattered docs; this file
> makes them explicit and citable. It is read-only inside any working loop:
> amendments require the project owner's sign-off and are recorded in the
> session handoff (NEXT_SESSION_WORDPRESS.md).

## 0. Scope and posture

- This is a **behavior-preserving, architecture-adapting** port. The Zend
  engine's C internals are *not* mirrored file-by-file; a safe-Rust VM
  reimplements them. What is preserved is the **observable surface**, at two
  tiers (project-owner policy, 2026-07-13):
  - **Byte-parity** for everything that lands in PHP strings/streams
    (stdout, diagnostics with file:line, `var_dump`/`serialize`/`json`,
    object ids, destructor order, error messages verbatim).
  - **Functional-parity via crates** for what leaves the process (network
    wire protocols, DB clients) — with one override: **if the oracle links a
    system C library (zlib, libgd, libxslt, libtidy), FFI to that same
    library is preferred — it yields byte-parity for free.** Rust
    re-implementations of such libs are banned for byte-compared output
    (measured: zlib crates diverge from system zlib).
- **Correct-or-absent**: a builtin that cannot be implemented faithfully is
  left ABSENT and catalogued in `PHPR_DIVERGENCES_FROM_PHP.md` — never
  approximated silently. `function_exists()` must tell the truth.
- **Unsafe policy**: zero `unsafe` in the value core (Zval / PhpStr /
  PhpArray / Props). Owner decision, re-confirmed twice against measured
  proposals (NaN-boxing WP-32, SSO union WP-38). Do not re-propose without
  an explicit change of course. FFI `unsafe` is confined to ext bindings.
- **The oracle is the executable spec**: brew PHP 8.5.7 CLI with
  `opcache.enable_cli=Off` (JIT off) — interpreter-vs-interpreter symmetry.
  Probe FIRST (`-d log_errors=0`, byte-diff stdout+stderr), read the C
  second (`php-8.5.7/ext/**`), implement third. php.net manual on doubt.
- **Performance is a separate, measured arc**, never mixed into parity
  changes: A/B interleaved same-day pairs on the real workload (WordPress
  suite groups); micro-benchmarks are advisory only — they lie twice
  (branch predictor + cache residency, WP-38). Every perf change ships with
  the full per-name gate green and its own probe battery.

## 1. Ecosystem adoption — what we use and what we ban

| Area | Decision | Why |
|---|---|---|
| Allocator | mimalloc (global) | measured −37% on real workload (WP-7) |
| Refcounting | `Rc` + statement-boundary sweep; **`Arc` banned** in the VM | single-threaded per-request model, like Zend |
| Internal hashing | FxHash for engine maps; SipHash banned in hot paths; PHP-visible string hash = zend_string->h semantics, cached | measured (WP-29) |
| String type | bytes everywhere (`PhpStr` over `Box<[u8]>`); **`String`/UTF-8 banned in the value core**; conversion only at FFI boundaries | PHP strings are byte arrays |
| DB / network | pure-Rust crates where the contract is a wire protocol (`mysql`, `rusqlite`, `ureq`+`rustls`) | functional-parity tier |
| System C libs | FFI to the system dylib when the oracle uses the same one (zlib, libgd+libwebp+libjpeg, libxslt, libtidy, libmagic model) | byte-parity for free |
| Async runtime | NONE | PHP has none; SAPI model mirrored |
| Third-party default | no new crate without a rule here naming it | style museum otherwise |

## 2. Constructs with no equivalent — one canonical mapping each

| Zend construct | Canonical phpr translation | Notes / evidence |
|---|---|---|
| `zval` (16 bytes) | `Zval` enum, **16-byte static-asserted invariant** — no variant may grow it | WP-27; niche layout relied on by packed arrays |
| refcount destruction | `Rc` counts; **PHP destructors NEVER run in Rust `Drop`** — `Op::Sweep` at statement boundaries drives them (eager, Zend-faithful timing) | http-kernel closure; WP-25 `drop_bounded` for shutdown |
| free order | note-buffer FIFO = release order; `gc_birth` for self-seeded entries; parent cascade releases exclusive destructor-less descendants; id-reuse LIFO with parent id on top | WP-28, oracle-pinned probes |
| `&$ref` | `Zval::Ref(Rc<RefCell<Zval>>)`, write-through discipline | D-R3 |
| HashTable | `PhpArray` dual-repr packed/hashed, **one-way escalation, never revive-in-place**; re-insert after unset goes to the tail | WP-27, oracle-pinned |
| object props | slot-based `PropsLayout` per class (`Rc`-shared); unset+re-set returns to the declaration slot | WP-27 |
| exceptions / diags | `PhpError` (throwables) + `Diags` (warnings/notices/deprecations); a warning is raised AT the faulting op and flushed there — an error-handler that throws unwinds from that exact point | WP-33 flush lesson |
| GC cycles | trial-deletion collector over the object graph, Zend-style adaptive threshold | WP-21 |
| request shutdown | explicit sequence (shutdown functions → destructors → session → stream filters → OB flush), then **fast-shutdown leak of the VM — CLI one-shot processes only**, never `-S` or in-process hosts | WP-39 |
| private methods/props | scope-aware resolution; first-wins on trait alias duplicates | bug61998 |

**The BUG rule:** when Zend's behavior is itself odd or buggy, reproduce it
bug-for-bug and let the per-name gates assert it. Improvements are separate,
flagged changes — the port's job is fidelity.

**Markers (adopted WP-39, kit grammar):** new deliberate deviations in code
are marked greppable — `BUG(port):` (bug-for-bug site with repro note),
`PERF(port):` (known slow-but-faithful), `TODO(port):` (conservative
translation pending a rule). Burndown = grep; the catalogue of *behavioral*
declared residues stays `PHPR_DIVERGENCES_FROM_PHP.md`.

## 3. The judge (runs continuously — old code is the spec)

- **Gates are per NAME, never count-only** (`gate-diff-fail-set-not-count`):
  Zend corpus + ext/session + ext/date + ext/reflection phpt fail-sets,
  doctrine/orm + symfony/http-kernel suites, WordPress option/restapi
  groups vs the oracle's own junit, gd/mysqli/media probe batteries
  (byte-id), full WordPress suite runs compared run-over-run. Zero
  pass→fail, ever; removals update the stored baseline.
- A layout/GC/order refactor pins **drop-order sentinels FIRST** (cargo
  tests asserting phpr's current order — WP-32, WP-39): such orders are
  timing-derived, not oracle-diffable, and must not change silently.

## Deviation log

Priced, catalogued divergences live in `PHPR_DIVERGENCES_FROM_PHP.md`
(§3.x) and in the per-name fail-set baselines. Notable engine-level ones:
in-function destructor timing (Zend frees at return, phpr at the enclosing
statement sweep), cycle-collector destructor order, backtraces of
builtin-thrown exceptions (§3.0).
