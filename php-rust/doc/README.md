# doc/ — documentation archive

Historical and reference documentation moved out of the repo root
(2026-07-24, post WP-49; index refreshed 2026-09-10). The **live** documents
stay in the root: `README.md`, `COVERAGE.md`, `TODO.md`, `REGOLE.md` (the
measurement/promotion protocol, single list), `NEXT_SESSION_WORDPRESS.md`
(current route), `PERF_MAP.md` (multi-workload perf map), `PIN_REGISTRY.md`
(promoted binaries by hash), `PHPR_DIVERGENCES_FROM_PHP.md` (living divergence
catalog), `CLAUDE.md`. Session history lives in `../sessions/`, perf-gap
snapshots in `../gaps/` (trend in `GAP_TREND.md`), per-session measurement
harnesses in `../wpNNN-harness/`, migration rules in `../migration/RULEBOOK.md`.

## architecture/ — long-term direction (still current, consulted on demand)
- `VISION_AND_ROADMAP.md` — overall project vision.
- `ASYNC_AND_DISTRIBUTION_ROADMAP.md` — server SAPI, async,
  single-binary distribution direction.
- `EXTENSIONS_ARCHITECTURE.md` — extension implementation strategy
  (native vs FFI vs class surfaces).

## plans-archive/ — executed or closed plans (kept as record)
- `FOOTPRINT_CPU_ROADMAP.md` — the 2026-07 "council" plan that opened the
  footprint/CPU arc (12× footprint, 2.66× CPU at the time); **superseded** by
  `REGOLE.md` + `NEXT_SESSION_WORDPRESS.md` (moved here 2026-09-10).
- `REGISTER_BYTECODE_PLAN.md` — register-bytecode arc; **closed at stage 1**
  (WP-44 verdict, three forms falsified). Code comments still cite its §4/§5.
- `refactor_plan_for_claude_2026.md`, `refactor_plan_claude_review_2026.md`,
  `refactor_plan_prelude_claude.md`, `vm_refactor_plan.md` — the executed
  architectural refactor (vm/mod split).
- `HK-RECON-PLAN.md`, `NEXT_SESSION_HTTP_KERNEL.md` — symfony/http-kernel
  track, **closed 0E/0F** (session 8, 2026-07-14).
- `NEXT_SESSION.md` — superseded generic kickoff (pointer to the WP route).

## archive/ — frozen changelogs of the divergence catalogue
- `DIVERGENCES_CHANGELOG_2026-07.md` (2026-07-09 → 07-27, WP-1..27) and
  `DIVERGENCES_CHANGELOG_2026-08.md` (2026-08-03 → 08-13, S-96..134): the
  history of additions/closures of `PHPR_DIVERGENCES_FROM_PHP.md`, which by
  rule holds ONLY open entries; later closures are traced in `../sessions/`.

## gemini/ — external model reviews (Gemini), chronological
Second-opinion analyses requested during the perf arc (WP-38..WP-44
rebuttals, Rust-constraints notes, early feedback). Inputs to session
verdicts; the binding conclusions live in `../sessions/` and memory.

## analysis/ — one-off analyses
`analysis_and_suggestions*.md` (v1-v5) — early external improvement
suggestions, largely absorbed into TODO/roadmaps.
