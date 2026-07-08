# Test coverage & PHP compatibility

*This is a living document — it is refreshed as functions and features land in phpr.*
*Read the project intro in [English](README.md) · [Italiano](italiano.md).*

phpr is validated against the **official PHP `.phpt` test corpus** (PHP 8.5.7). Each
test bakes in the reference interpreter's expected output (its `--EXPECT--` /
`--EXPECTF--` section), so a **pass means byte-for-byte parity with the oracle** for
that case — not merely "it ran". An area is considered **done only when every one of
its tests passes** (zero failures).

> **Snapshot:** 2026-07-08. Numbers are produced by running each area in isolation
> with the project test runner (`phpt-runner --isolate <area>`). `pass` / `fail` count
> executed tests; `skip` counts tests the runner declined — almost always because the
> underlying extension is not compiled into phpr, so those tests never execute.

## Status legend

| Badge | Meaning |
|---|---|
| ✅ **DONE** | 0 failures, at least one pass — full parity |
| 🟢 **NEAR** | ≥ 90% of executed tests pass |
| 🟡 **PARTIAL** | 50–90% pass |
| 🟠 **WEAK** | < 50% pass |
| 🔴 **ABSENT** | 0 pass with failures — feature effectively unimplemented |
| ⚪ **NOT REGISTERED** | whole extension not built in — every test skips |

## Headline

| Layer | Pass | Fail | Parity |
|---|---:|---:|---:|
| Engine core (`Zend/tests`) | 2314 | 1473 | ~61% |
| Standard library (`ext/standard`, sampled) | 1191 | 829 | ~59% |

phpr's strength today is **broad coverage of the language core**, not fully-closed
extension areas. The fully-parity set is still small and is being expanded
deliberately, area by area.

## Engine core & standard library

| Area | Pass | Fail | Status |
|---|---:|---:|---|
| `Zend/tests` | 2314 | 1473 | 🟡 PARTIAL |
| `ext/standard` — strings | 374 | 149 | 🟡 PARTIAL |
| `ext/standard` — array | 458 | 211 | 🟡 PARTIAL |
| `ext/standard` — class_object | 28 | 11 | 🟡 PARTIAL |
| `ext/standard` — url | 20 | 10 | 🟡 PARTIAL |
| `ext/standard` — file | 154 | 136 | 🟡 PARTIAL |
| `ext/standard` — general_functions | 65 | 65 | 🟡 PARTIAL |
| `ext/standard` — math | 19 | 27 | 🟠 WEAK |
| `ext/standard` — serialize | 28 | 106 | 🟠 WEAK |
| `ext/standard` — streams | 14 | 39 | 🟠 WEAK |
| `ext/standard` — http | 9 | 24 | 🟠 WEAK |
| `ext/standard` — network | 2 | 9 | 🟠 WEAK |
| `ext/standard` — password | 0 | 7 | 🔴 ABSENT |
| `ext/standard` — filters | 0 | 6 | 🔴 ABSENT |
| `ext/standard` — crypt | 4 | 0 | ✅ DONE |
| `ext/standard` — hrtime | 2 | 0 | ✅ DONE |
| `ext/standard` — versioning | 5 | 0 | ✅ DONE |

## Extensions (registered)

| Area | Pass | Fail | Status |
|---|---:|---:|---|
| `ext/ctype` | 46 | 0 | ✅ DONE |
| `ext/opcache` | 26 | 14 | 🟡 PARTIAL |
| `ext/pcre` | 68 | 42 | 🟡 PARTIAL |
| `ext/json` | 41 | 30 | 🟡 PARTIAL |
| `ext/sqlite3` | 34 | 40 | 🟠 WEAK |
| `ext/pdo_sqlite` | 22 | 44 | 🟠 WEAK |
| `ext/mbstring` | 40 | 70 | 🟠 WEAK |
| `ext/hash` | 16 | 48 | 🟠 WEAK |
| `ext/random` | 8 | 46 | 🟠 WEAK |
| `ext/zip` | 7 | 48 | 🟠 WEAK |
| `ext/date` | 98 | 309 | 🟠 WEAK |
| `ext/reflection` | 154 | 298 | 🟠 WEAK |
| `ext/spl` | 142 | 511 | 🟠 WEAK |
| `ext/uri` | 0 | 117 | 🔴 ABSENT (new in 8.5) |

## Not yet registered

These extensions are not compiled into phpr, so their entire suites skip. Listed by
suite size (largest = biggest latent opportunity), abbreviated:

`soap` (588) · `phar` (565) · `intl` (473) · `mysqli` (448) · `gd` (314) ·
`session` (259) · `openssl` (225) · `curl` (175) · `pdo_mysql` (167) · `bcmath` (166) ·
`zlib` (150) · `ldap` (140) · `filter` (120) · `sockets` (117) · `ffi` (106) ·
`pgsql` (101) · `gmp` (99) · `exif` (98) · `xsl` (81) · `iconv` (76) · `xml` (67) ·
`posix` (61) · `pcntl` (60) · `tokenizer` (53) · `dom` / `simplexml` (large, mostly
skipped) · plus `xmlreader`, `xmlwriter`, `tidy`, `odbc`, `snmp`, `sodium`,
`readline`, `bz2`, `gettext`, `calendar`, `dba`, and the remaining PDO drivers.

## Standard-library function coverage

phpr implements **~560 of the ~2143 internal functions** PHP 8.5 exposes. Most of the
gap is whole unregistered extensions (above); the core-stdlib gap that application
frameworks actually touch is much smaller and is being closed function by function.

## Roadmap (highest leverage first)

1. **Close the near-complete areas** — `opcache`, `pcre`, `json`, `ext/standard/strings`
   and `array`: high pass-rates, each remaining failure is usually a small, independent
   fix (flag/format edge cases).
2. **Scoped features** — `random` (a finite set of engine classes), `hash` (missing
   digests), `mbstring` (encoding tables), `ext/standard/serialize`.
3. **Large subsystems** — `spl`, `reflection` (golden-output export format), `date`
   (temporal arithmetic + tzdata), `uri` (greenfield 8.5 extension).
4. **Whole-extension greenfield** — driven by concrete use cases (e.g. a target
   framework requiring `intl`, `curl`, `bcmath`, …).

## How this is measured

Each row comes from running the reference corpus against a phpr build:

```bash
phpt-runner --isolate <path-to-area>          # summary: pass / fail / skip
phpt-runner --isolate --list-fails <area>     # also lists each failing test
```

Contributions that turn a 🟠/🔴 area into ✅ (or move a headline number) are the most
valuable — pick any failing test, make phpr's output match the oracle, and this table
moves with it.
