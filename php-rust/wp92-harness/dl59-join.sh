#!/bin/bash
# dl59-join.sh — A-DL-59 (Concilio WP-93, Leijen Q3): does per-theap PAGE SLACK
# explain the INVISIBLE per-worker slope of the m90 census?
#
# ANALYSIS ONLY: reads ONLY the 20 committed m90 slope raws
#   wp78-harness/measure-out/m90.slope.w{4,8,12,16}.r{1..5}.a1.memcensus
# and wp91-harness/repair90-estimators.out (machine truth for the invisible
# slope/intercept). No new instrumentation, no campaigns. Deterministic,
# self-contained, rerunnable: bash + python3 (stdlib only).
#
# Raws contain NUL bytes: read in binary and strip b'\0'.
set -euo pipefail
ROOT="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
RAWDIR="$ROOT/wp78-harness/measure-out"
EST="$ROOT/wp91-harness/repair90-estimators.out"

python3 - "$RAWDIR" "$EST" <<'PYEOF'
import sys, re, statistics

rawdir, estpath = sys.argv[1], sys.argv[2]

# ---------------------------------------------------------------- machine truth
est = open(estpath, 'rb').read().replace(b'\0', b'').decode('utf-8', 'replace')
m_inv = re.search(r'slope_invisible=(\d+) B/worker intercept_invisible=(\d+) B', est)
m_vis = re.search(r'slope_vis=(\d+) B/worker se=\d+ a_vis=(\d+) => marginal_ratio vs b_peak\(median\)=([\d.]+)', est)
assert m_inv and m_vis, "repair90-estimators.out: VCOV lines not found"
slope_inv, icpt_inv = int(m_inv.group(1)), int(m_inv.group(2))
slope_vis, a_vis, mratio = int(m_vis.group(1)), int(m_vis.group(2)), m_vis.group(3)

# ---------------------------------------------------------------------- header
print("A-DL-59 dl59-join: per-theap PAGE SLACK vs invisible per-worker slope (m90 census)")
print("grade=ADVISORY  # analysis on already-consumed m90 raws; lane carries the declared")
print("                # selection-mismatch caveats of repair90 (A-BB65); VERDICT-grade only")
print("                # via the future barrier channel A-DL-57/58")
print("inputs: 20 committed raws wp78-harness/measure-out/m90.slope.w{4,8,12,16}.r{1..5}.a1.memcensus")
print("        + wp91-harness/repair90-estimators.out (authority for invisible slope/intercept)")
print("machine truth (parsed from repair90-estimators.out, not hardcoded):")
print(f"  slope_invisible={slope_inv} B/worker  intercept_invisible={icpt_inv} B")
print(f"  slope_vis={slope_vis} B/worker  a_vis={a_vis}  marginal_ratio_vs_b_peak_median={mratio}")
print()
print("row grammar found (field names per NOME):")
print("  tag=mi_bin thr=<k> src=tls size=<bin> reserved= committed= used_b= used_n= areas=")
print("    (NO ckpt= field on these rows; attribution below)")
print("  tag=peak_self thr=<k> ckpt=peak_inreq alloc_id=memcount-v2-s82   # dump announcement")
print("  tag=mi_bin_thr_sum thr=<k> heap=<ptr> reserved= committed= used_b=  # per-thread SUM of")
print("    its tls rows; SAME heap ptr and SAME committed for every thr => the tls dumps are")
print("    per-thread VIEWS of the ONE shared heap (arena json: heaps=1, theaps=W+extras)")
print("  tag=mi_theap_pages ... ckpt=<c> heap=0 pages= committed= used_blocks= free_pages= free_committed=")
print("    (+ trailer heaps_total=1 heap_overflow=0 declared=heap-by-visit-index,noalloc-visitors-A-MS50)")
print("  tag=mi_theap_bin ... ckpt=<c> heap=0 bin=<b> free_pages= free_committed=")
print()
print("chosen checkpoint (per NOME): ckpt=peak_inreq, per-thread tls dump #1")
print("  There is NO per-thread tls dump at ckpt=peak_inreq_pwork: each thread emits exactly TWO")
print("  tls dumps per run — dump#1 announced by 'tag=peak_self thr=k ckpt=peak_inreq' (between the")
print("  tag=peak_inreq_pwork summary row and the 'mi_proc win=9 ckpt=peak_inreq' marker), and")
print("  dump#2 just before tag=exit (exit-side). dump#1 = the WORK-side in-request-peak snapshot,")
print("  a few hundred ms after pwork with near-identical commit — the closest snapshot to the")
print("  pwork window on which the invisible slope was estimated (VCOV A-BG57, 'pooled OLS of")
print("  pwork-commit minus vis'). dump#2 is reported as a control column.")
print("  ATTRIBUTION rule: last-seen-ckpt is unreliable (peak_self rows poison it; writer")
print("  interleaving differs across raws). Rule used per NOME: per-thread TEMPORAL order — each")
print("  thread's tls rows are split at the first repeated size class into dump#1 / dump#2;")
print("  positional sanity (d1 before / d2 after the 'mi_proc win=9 ckpt=peak_inreq' marker) is")
print("  verified and printed per run.")
print()
print("SLACK formulas (per NOME, declared):")
print("  SLACK_naive(run)  = sum over thr=0..W-1, over size classes, of (committed - used_b)")
print("                      from tag=mi_bin src=tls dump#1 rows  [the task's literal formula]")
print("  SLACK_global(run) = mean over thr of Sum_size (committed - used_b) of that thr's dump#1")
print("                      [deduplicated: the tls dumps all describe the ONE shared heap, so the")
print("                       naive Sum over thr counts the same heap W times — see refutation]")
print("  free_committed of retained pages is NOT a field on src=tls rows; retained fully-free")
print("  pages (page_full_retain=2) stay committed with used_b=0 and land in (committed-used_b)")
print("  (rows with used_n=0 & committed>0 are counted per run below). The explicit free_committed")
print("  fields (mi_theap_pages/mi_theap_bin) cover ONLY the single visited heap (heaps_total=1,")
print("  noalloc-visitor A-MS50/KL-90-3); the visited-heap value at pwork is an advisory column.")
print()

# ------------------------------------------------------------------ extraction
Ws = [4, 8, 12, 16]
runs = {}
for w in Ws:
    for r in range(1, 6):
        path = f"{rawdir}/m90.slope.w{w}.r{r}.a1.memcensus"
        text = open(path, 'rb').read().replace(b'\0', b'').decode('utf-8', 'replace')
        per_thr, thrsum = {}, {}
        theap = {}
        peak_self_n = 0
        miproc_peak_line = None   # line index of 'mi_proc win=9 ckpt=peak_inreq '
        pwork_commit = None       # commit= of mi_proc ckpt=peak_inreq_pwork
        for ln, line in enumerate(text.splitlines()):
            if ' tag=mi_proc ' in line:
                if ' ckpt=peak_inreq ' in line and ' win=9 ' in line:
                    miproc_peak_line = ln
                elif ' ckpt=peak_inreq_pwork ' in line:
                    m = re.search(r'(?<!peak_)\bcommit=(\d+)', line)
                    pwork_commit = int(m.group(1))
            elif ' tag=peak_self ' in line and ' ckpt=peak_inreq ' in line:
                peak_self_n += 1
            elif ' tag=mi_bin ' in line and ' src=tls ' in line:
                fld = dict(kv.split('=', 1) for kv in line.split() if '=' in kv and not kv.startswith('/'))
                per_thr.setdefault(int(fld['thr']), []).append(
                    (int(fld['size']), int(fld['committed']), int(fld['used_b']),
                     int(fld['used_n']), ln))
            elif ' tag=mi_bin_thr_sum ' in line:
                fld = dict(kv.split('=', 1) for kv in line.split() if '=' in kv)
                thrsum.setdefault(int(fld['thr']), []).append(
                    (int(fld['committed']), int(fld['used_b'])))
            elif ' tag=mi_theap_pages ' in line and ' heap=0 ' in line:
                fld = dict(kv.split('=', 1) for kv in line.split() if '=' in kv)
                theap[fld['ckpt']] = (int(fld['committed']), int(fld['free_committed']))
        assert set(per_thr) == set(range(w)), f"{path}: thr set {sorted(per_thr)} != 0..{w-1}"
        assert peak_self_n == w, f"{path}: peak_self rows {peak_self_n} != W={w}"
        assert miproc_peak_line is not None and pwork_commit is not None, f"{path}: markers missing"
        naive1 = naive2 = rows1 = freerows = free_c = 0
        d1_after_marker = d2_before_marker = 0
        g1, g2 = [], []            # per-thread global slack, dump1/dump2
        tsum_match = 0
        for t in sorted(per_thr):
            rowlist = per_thr[t]
            seen, split = set(), len(rowlist)
            for i, row in enumerate(rowlist):
                if row[0] in seen:
                    split = i
                    break
                seen.add(row[0])
            d1, d2 = rowlist[:split], rowlist[split:]
            assert d1 and d2, f"{path}: thr={t} did not yield two tls dumps"
            for dd in (d1, d2):
                sz = [x[0] for x in dd]
                assert len(sz) == len(set(sz)), f"{path}: thr={t} duplicate size within a dump"
            s1 = sum(c - u for _, c, u, _, _ in d1)
            s2 = sum(c - u for _, c, u, _, _ in d2)
            naive1 += s1; naive2 += s2
            g1.append(s1); g2.append(s2)
            rows1 += len(d1)
            freerows += sum(1 for _, c, _, un, _ in d1 if un == 0 and c > 0)
            free_c += sum(c for _, c, _, un, _ in d1 if un == 0 and c > 0)
            d1_after_marker += sum(1 for x in d1 if x[4] > miproc_peak_line)
            d2_before_marker += sum(1 for x in d2 if x[4] < miproc_peak_line)
            # cross-check: thr_sum dump#1 committed == Sum committed of tls dump#1
            if t in thrsum and len(thrsum[t]) >= 1:
                if thrsum[t][0][0] == sum(c for _, c, _, _, _ in d1):
                    tsum_match += 1
        runs[(w, r)] = dict(naive=naive1, naive2=naive2, rows=rows1, nthr=len(per_thr),
                            freerows=freerows, free_c=free_c,
                            g1=statistics.mean(g1), g2=statistics.mean(g2),
                            tsum_c=thrsum[0][0][0], tsum_match=tsum_match,
                            pwork_commit=pwork_commit,
                            d1_after=d1_after_marker, d2_before=d2_before_marker,
                            th_free_c=theap.get('peak_inreq_pwork', (0, 0))[1])

# ------------------------------------------------------------- per-run table
print("per-run SLACK, dump#1 = work-side ckpt=peak_inreq per-thread snapshot (B):")
print(f"{'run':<9}{'SLACK_naive':>13}{'SLACK_global':>13}{'SLACK_glob_d2':>14}{'heap_commit':>13}"
      f"{'pwork_commit':>13}{'freePgRows':>11}{'freePg_c':>10}{'visheap_free_c':>15}"
      f"{'thrSumMatch':>12}{'d1mispl':>8}{'d2mispl':>8}")
for w in Ws:
    for r in range(1, 6):
        d = runs[(w, r)]
        print(f"w{w}.r{r:<6}{d['naive']:>13}{d['g1']:>13.0f}{d['g2']:>14.0f}{d['tsum_c']:>13}"
              f"{d['pwork_commit']:>13}{d['freerows']:>11}{d['free_c']:>10}{d['th_free_c']:>15}"
              f"{str(d['tsum_match'])+'/'+str(d['nthr']):>12}{d['d1_after']:>8}{d['d2_before']:>8}")
print("  (thrSumMatch: threads whose mi_bin_thr_sum dump#1 committed == Sum of its tls dump#1")
print("   committed — validates dump splitting; d1mispl/d2mispl: dump rows on the wrong side of")
print("   the 'mi_proc win=9 ckpt=peak_inreq' marker — 0 = positional attribution confirmed)")
print()

# ---------------------------------------------- shared-heap refutation evidence
print("shared-heap refutation of the naive Sum (per NOME):")
some = runs[(16, 1)]
print(f"  every mi_bin_thr_sum row of a dump carries the SAME heap=<ptr> and SAME committed for")
print(f"  all thr; e.g. w16.r1 dump#1: heap_commit={some['tsum_c']} for each of 16 thr =>")
print(f"  Sum_thr committed = {16*some['tsum_c']:,} >> pwork process commit = {some['pwork_commit']:,}.")
print(f"  The tls dumps are W views of ONE shared heap; Sum over thr counts it W times.")
print(f"  SLACK_global (mean over thr) is the physical page slack of the process heap.")
print()
vm = {w: slope_vis*w + a_vis for w in Ws}
print("identity check — census coverage vs heap bin committed (per NOME, decisive for Leijen Q3):")
for w in Ws:
    cs = [runs[(w, r)]['tsum_c'] for r in range(1, 6)]
    print(f"  W={w:<3} vis_model={vm[w]:>12,}  heap_commit min..max = {min(cs):,} .. {max(cs):,}"
          f"  (vis_model/median = {vm[w]/statistics.median(cs):.4f})")
print("  => vis(W) = slope_vis*W + a_vis coincides with the heap BIN-COMMITTED bytes to <0.5%.")
print("  The census coverage already counts committed bin pages (used + slack). Page slack is")
print("  therefore INSIDE the visible mass, NOT inside the invisible mass = pwork_commit - vis")
print("  (which lives at arena/chunk level and outside mimalloc bins).")
print()

# --------------------------------------------------------------- per-W means
def wmean(key):
    return {w: statistics.mean(runs[(w, r)][key] for r in range(1, 6)) for w in Ws}
mn, mg = wmean('naive'), wmean('g1')
print("per-W mean SLACK (over 5 runs each):")
print(f"{'W':<4}{'mean_naive':>15}{'mean_global':>14}{'global/W':>12}")
for w in Ws:
    print(f"{w:<4}{mn[w]:>15.1f}{mg[w]:>14.1f}{mg[w]/w:>12.0f}")
print()

def ols(pts):
    n = len(pts)
    sx = sum(p[0] for p in pts); sy = sum(p[1] for p in pts)
    sxx = sum(p[0]*p[0] for p in pts); sxy = sum(p[0]*p[1] for p in pts)
    b = (n*sxy - sx*sy) / (n*sxx - sx*sx)
    a = (sy - b*sx) / n
    return b, a

for label, key, means in (("SLACK_naive", 'naive', mn), ("SLACK_global", 'g1', mg)):
    b4, a4 = ols([(w, means[w]) for w in Ws])
    b20, a20 = ols([(w, runs[(w, r)][key]) for w in Ws for r in range(1, 6)])
    print(f"least-squares {label} vs W:")
    print(f"  over 4 W-means : slope={b4:,.1f} B/worker  intercept={a4:,.1f} B")
    print(f"  over 20 points : slope={b20:,.1f} B/worker  intercept={a20:,.1f} B")
    print(f"  (balanced design: the two fits coincide by construction on slope/intercept)")
    print(f"  ratio slope_{label}/slope_invisible({slope_inv}) : 4-means={b4/slope_inv:.3f}  20-pt={b20/slope_inv:.3f}")
    print(f"  intercept vs invisible intercept {icpt_inv:,} B : {a4:,.1f} B (ratio {a4/icpt_inv:.3f})")
    print()

print("anomalies per NOME (flagged runs of repair90 A-BB67 seen through SLACK):")
def sisters(w, r, key):
    return [runs[(w, rr)][key] for rr in range(1, 6) if rr != r]
for (w, r) in ((4, 1), (12, 3), (16, 1)):
    d = runs[(w, r)]
    sg = sisters(w, r, 'g1'); sc = sisters(w, r, 'pwork_commit')
    print(f"  w{w}.r{r}: SLACK_global={d['g1']:,.0f} (sisters {min(sg):,.0f}..{max(sg):,.0f})"
          f"  pwork_commit={d['pwork_commit']:,} (sisters {min(sc):,}..{max(sc):,})")
print("  - w4.r1: low pair with w4.r5 in SLACK_global (-2.5% vs the r2..r4 cluster) and lowest")
print("    pwork_commit (-1.4%); mild in SLACK — its A-BB67 outlier flag is not slack-driven.")
print("  - w12.r3: HIGHEST SLACK_global of its W (+1.6%) while its pwork_commit is ~25-30MB BELOW")
print("    the sisters: the run is an outlier in the invisible (arena-level) mass, in the opposite")
print("    direction of its slack — further evidence slack and invisible mass are different bodies.")
print("  - w16.r1: unremarkable in SLACK (mid-pack). All W=16 runs (only they) show the retained")
print("    free pages: 16 view-rows x 262,144 B = 4,194,304 B naive (262,144 B deduplicated).")
print()
print("VERDICT (ADVISORY): the page-slack hypothesis does NOT hold.")
print("  - The task-literal Sum-over-theaps SLACK_naive gives slope ~1.9x the invisible slope with")
print("    a NEGATIVE intercept (wrong shape AND wrong sign), but it is invalid per NOME: the tls")
print("    dumps are W per-thread views of one shared heap (identical heap ptr + identical")
print("    committed across thr; Sum_thr committed >> process commit).")
print("  - The physical (deduplicated) SLACK_global slope is ~9% of the invisible slope: page")
print("    slack cannot be 'first explanation' of the invisible per-worker slope.")
print("  - Decisive identity: vis_model(W) == heap bin-committed (<0.5%): bin page slack is")
print("    already counted INSIDE the census coverage; the invisible mass = pwork_commit - vis")
print("    sits at arena/chunk level, outside the bins the slack lives in.")
PYEOF
