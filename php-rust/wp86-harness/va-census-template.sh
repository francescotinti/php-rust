#!/bin/bash
# va-census-template.sh — A-DS37 (Council WP-87, Stogov): the MANDATED
# template for every future VA census window (steady MEASURED=30 class).
# KG-86-1 class rule: a gate is born fail-closed or it is illegitimate —
# this template exists BEFORE any window is launched, so no VA window can
# be improvised without the vitality teeth (KS-DS-87-2: a census window
# launched from a template without in-band vitality => cifre VOID, not
# advisory).
#
# CONTRACT (fill the TODOs; never delete a tooth):
#   1. IDENTITY: battery-<N>pre PASS at HEAD (A-SK41 ledgered stamp),
#      matrix resolved FROM the battery .done (A-AH40, never tail -1),
#      binary ENFORCEd against the matrix row + gate-binary-noprobe
#      (KH86-1 nm+strings SUFFICIENT since A-TH37; hash stays belt).
#   2. VITALITY IN-BAND: every measured window row must ride WITH a
#      vitality counter proving the window SAW traffic:
#        - `tag=worker_dispatch thr=<t> count=<n>` (A-PP36) — per-thread
#          dispatch counts; the verdict requires sum == expected and the
#          per-thread split == the protocol's (KS-PP-87-3: a row from a
#          run without matching dispatch counts is ENVELOPE, never
#          attribution).
#        - main_probe delta > 0 across the window (a zero-probe window
#          measured nothing — the F-oneshot lesson: PASS on zero is
#          vacuous).
#   3. WINDOW ROWS: MEASURED=<k> declared ex-ante in-band; rows carry
#      alloc_id (KS-AH-85-3) + arm= label; the verdict is COLLECT-THEN-
#      EMIT per-block (A-SK44/A-MS28): no PASS/derivata line from a block
#      with a fail (KS-SK-87-3).
#   4. ORD TEETH: any unitcache_main_entry row with ord!=1 in a
#      one-dispatch-per-worker protocol => raw VOID (A-SK45).
#   5. DEFLATION: finish() semantics are DECLARED (A-DL34) — frees in
#      window subtract (pre-window blocks included), clamp at zero; a
#      steady window whose net rides the clamp must say so.
#   6. BURST CONTROL (A-DL33): one DEDICATED run with
#      PHPR_MEMCENSUS_TEST_BURST=1 — the row must move >= 1.048.576 B and
#      the verdict must DECLASSIFY that run (fail-closed of the PIPELINE).
#      The burst run is never a measurement run.
#   7. RAWS: never rm (KS-AH-83-2); substitute scripts carry attempt=N +
#      prior raw list (A-BG39/KG-87-1); head_unmoved post-build and
#      pre-launch (A-BG40); reqns lines only via wp86-harness/
#      reqns-guard.pl (A-PP35).
#   8. FIGURES: BYTES-FIRST with verified companion, MiB-only, ± bands
#      from the A-SK43 allowlist; VOID-sourced figures only as
#      MODEL-GRADE(src=<raw>), never load-bearing (A-BG42/KG-87-2).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
echo "TEMPLATE ONLY (A-DS37): copy, fill the TODOs, and wire every tooth" >&2
echo "before launching a VA census window — running the template is refused." >&2
exit 2
