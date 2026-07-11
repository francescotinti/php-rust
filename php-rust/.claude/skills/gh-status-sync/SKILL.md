---
name: gh-status-sync
description: Use when the phpr project's GitHub-facing docs (README.md, COVERAGE.md, TODO.md header) are stale or a work session just landed commits — measures the real coverage/corpus numbers and syncs every published file, then commits and pushes.
model: sonnet
---

# gh-status-sync — measure phpr's real status and publish it

## Overview

The GitHub docs publish MEASURED numbers, never estimates. This skill gathers
them from the ground truth (probe script + phpt-runner + the framework-suite
logs) and rewrites the published files so they all agree.

## Gather the numbers (ground truth, in this order)

1. **Function coverage** (~1 min):
   ```sh
   ./scripts/measure-coverage.sh
   ```
   First two lines are the headline (`TOTAL h/t p%`, `CORE-STDLIB h/t p%`);
   the rest is the per-extension table. Oracle default:
   `/opt/homebrew/opt/php/bin/php`; phpr default:
   `~/Claude/php-rust-output/release/phpr`. ALWAYS run
   `cargo build --release` first — it is a fast no-op when the binary is
   already fresh, and it removes any doubt about probing a stale build.

2. **Zend corpus** (~12 min, run it — do not reuse stale counts):
   ```sh
   ~/Claude/php-rust-output/release/phpt-runner --isolate \
       "/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests" | LC_ALL=C tr -d '\0' | tail -20
   ```
   Publish the `pass:` count; "% of runnable" = pass / (total − skip).

3. **Framework status**: current baselines live in TODO.md's "Current state"
   paragraph and the session memory; only update them from an actual suite run,
   never by guessing.

## Files to sync (all of them, same numbers everywhere)

| File | What to update |
| --- | --- |
| `README.md` | "Coverage at a glance" table (3 rows), the status blockquote, Roadmap if priorities shifted |
| `COVERAGE.md` | `_Last measured:_` date, Headline table, corpus breakdown line, per-area table (from the script's TSV), "not started" row, framework list |
| `TODO.md` | "Current state (DATE): …" paragraph only |
| any new published file | keep it consistent with the same measured numbers |

Rules:
- The core-stdlib rollup is **standard + Core + date** (654 fns) — keep this
  definition stable across updates.
- Percentages round half-up (the script already does).
- An area that reached 100% moves into the "Fully-complete areas" headline row.
- Extensions at 0 collapse into the single "not started" row with their counts.

## Publish

```sh
git add -A && git commit -m "docs: refresh measured status (corpus NNNN, fns H/T)

Co-Authored-By: Claude <noreply@anthropic.com>" && git push
```

Commit message must cite the fresh corpus and function counts. "Published"
means committed AND pushed to origin — files updated only in the working tree
are still stale as far as GitHub is concerned, so this skill is not done until
the push succeeds.

## Common mistakes

- Publishing counts from memory/TODO instead of re-measuring → the numbers
  drift; always run the script and the corpus.
- Updating README but not COVERAGE.md (or vice versa) → tables disagree.
- Changing the core-stdlib definition → historic comparisons break.
- Forgetting `cargo build --release` first → probing a stale binary.
