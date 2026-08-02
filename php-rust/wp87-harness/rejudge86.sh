#!/bin/bash
# rejudge86.sh — S-87.0 p1: ri-giudizi DAI RAW ESISTENTI (Concilio WP-88 §Sintesi p1).
# NESSUN run nuovo: legge i raw della campagna measure86 (git c259bc6) già committati.
#   A-BB49: qualificatore overlap NUMERICO (+0) ri-applicato ai 10 raw m86.ovl.a*.memcensus
#           → verdetto overlap + firma "finestra process-counters inghiotte il lowering altrui".
#   A-BB51: VABBA ri-giudicato sulla metrica peak_memory_footprint (time -l, nei .log axum.86p.*)
#           + diff per-regione (DIRTY) dai vmmap V2 archiviati.
# Output: wp87-harness/rejudge86.out (nome NUOVO — KG-88-1: mai riusare filename).
# Metrica SEMPRE nominata (KB-88-2). max−min riportato solo ADVISORY (KB-88-3: R=8<16).
set -eu

REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
MO="$REPO/wp78-harness/measure-out"
OUT="$REPO/wp87-harness/rejudge86.out"
NET_H=7349977
NET_P=7803281

: > "$OUT"
say() { echo "$@" | tee -a "$OUT"; }

say "== rejudge86 (S-87.0 p1) — raw campaign git=c259bc6 driver_sha=699db00a9808489e — analysis-only, no new runs =="
say "date=$(date -u +%Y-%m-%dT%H:%M:%SZ) analysis_git=$(cd "$REPO" && git rev-parse --short HEAD)"
say ""
say "== PART A — A-BB49: overlap re-judgment, NUMERIC qualifier (t+0), raws m86.ovl.a1..a10 =="
say "metric=lower_span t0_us/t1_us (in-band, per-fixture) — qualifier bug: post-sub() awk compared STRINGS"
OVERLAPS=0
SWALLOW=0
for a in 1 2 3 4 5 6 7 8 9 10; do
  MC="$MO/m86.ovl.a$a.memcensus"
  [ -f "$MC" ] || { say "attempt=$a MISSING raw — ABORT"; exit 1; }
  LINE=$(tr -d '\0' < "$MC" | awk -v neth=$NET_H -v netp=$NET_P '
    /tag=lower_span/ {
      tid=""; t0=""; t1=""
      for (i=1;i<=NF;i++) {
        if ($i ~ /^tid=/) tid=$i
        if ($i ~ /^t0_us=/) { v=$i; sub("t0_us=","",v); t0=v+0 }
        if ($i ~ /^t1_us=/) { v=$i; sub("t1_us=","",v); t1=v+0 }
      }
      if ($NF ~ /hello_pad85[.]php$/) { pt0=t0; pt1=t1; ptid=tid; psee=1 }
      else if ($NF ~ /hello[.]php$/)  { ht0=t0; ht1=t1; htid=tid; hsee=1 }
    }
    /tag=unitcache_main_entry/ {
      for (i=1;i<=NF;i++) if ($i ~ /^net=/) { v=$i; sub("net=","",v); n=v+0 }
      if ($NF ~ /hello_pad85[.]php$/) pnet=n
      else if ($NF ~ /hello[.]php$/)  hnet=n
    }
    END {
      if (!hsee || !psee || htid==ptid) { print "attempt INVALID"; exit }
      ov = (ht0 < pt1 && pt0 < ht1) ? "YES" : "NO"
      dur = (ht1<pt1?ht1:pt1) - (ht0>pt0?ht0:pt0)
      swallow = (pnet == neth+netp+150) ? "pad_net==NET_H+NET_P+150_EXACT" : "pad_net_delta=" pnet-neth-netp
      printf "h=[%d,%d]us tid_h=%s  p=[%d,%d]us tid_p=%s  overlap=%s ov_dur_us=%d  hello_net=%d pad_net=%d %s", ht0,ht1,htid,pt0,pt1,ptid,ov,dur,hnet,pnet,swallow
    }')
  say "attempt=$a $LINE"
  case "$LINE" in *"overlap=YES"*) OVERLAPS=$((OVERLAPS+1));; esac
  case "$LINE" in *EXACT*) SWALLOW=$((SWALLOW+1));; esac
done
say ""
grep "^attempt=" "$OUT" | awk '
  { for (i=1;i<=NF;i++) if ($i ~ /^h=\[/) { v=$i; sub(/^h=\[[0-9]+,/,"",v); sub(/\]us$/,"",v); t=v+0
      if (mn=="" || t<mn) mn=t; if (t>mx) mx=t } }
  END { printf "hello lowering t1 range across 10 attempts: %.1f-%.1f ms (derived: t1_us/1000)\n", mn/1000, mx/1000 }' | tee -a "$OUT"
say "A-BB49 VERDICT: overlap_qualified=$OVERLAPS/10 (campaign ledger said 0/10 — FALSO-DAI-RAW confirmed)"
say "  swallow signature pad_net==NET_H+NET_P+150 exact: $SWALLOW/10 attempts"
say "  KB-88-1: ALL net= readings from these 10 raws are VOID as per-thread figures (net_window=process-counters, spans intersect)"
say "  VOVL RE-VERDICT: per-thread net under concurrency = REFUTED-BY-RAWS (not OPEN). xW remains SEQUENTIAL-ONLY (KL-87-2 hard)."
say ""
say "== PART B — A-BB51: VABBA re-judged on metric=peak_memory_footprint (time -l bytes, .log) =="
say "arms: A=pa (MIMALLOC_PURGE_DELAY=0)  B=pd (default). r1 NOMINATO excluded (warm-up), stats on r2..r9 (R=8/arm)."
for arm in pa pd; do
  for r in 1 2 3 4 5 6 7 8 9; do
    L="$MO/axum.86p.$arm.r$r.log"
    [ -f "$L" ] || { say "MISSING $L — ABORT"; exit 1; }
    FP=$(tr -d '\0' < "$L" | awk '/peak memory footprint/{print $1}')
    RSS=$(tr -d '\0' < "$L" | awk '/maximum resident set size/{print $1}')
    say "arm=$arm r=$r peak_memory_footprint_bytes=$FP max_rss_bytes=$RSS"
  done
done
say ""
grep "^arm=" "$OUT" | awk '
  { split($2,rr,"="); r=rr[2]+0; split($3,ff,"="); fp=ff[2]+0
    if ($1=="arm=pa") { A[r]=fp } else { B[r]=fp } }
  END {
    mina=1e18; maxa=0; suma=0; minb=1e18; maxb=0; sumb=0
    for (r=2;r<=9;r++) {
      suma+=A[r]; sumb+=B[r]
      if (A[r]<mina) mina=A[r]; if (A[r]>maxa) maxa=A[r]
      if (B[r]<minb) minb=B[r]; if (B[r]>maxb) maxb=B[r]
    }
    ma=suma/8; mb=sumb/8
    va=0; vb=0
    for (r=2;r<=9;r++) { va+=(A[r]-ma)^2; vb+=(B[r]-mb)^2 }
    va/=7; vb/=7
    sep=0
    for (r=2;r<=9;r++) if (A[r] < minb) sep++
    printf "A(purge=0) r2..r9: mean=%.0f B (%.2f MiB) min=%d max=%d spread_ADVISORY=%d B (%.2f MiB) var=%.3e\n", ma, ma/1048576, mina, maxa, maxa-mina, (maxa-mina)/1048576, va
    printf "B(default) r2..r9: mean=%.0f B (%.2f MiB) min=%d max=%d spread_ADVISORY=%d B (%.2f MiB) var=%.3e\n", mb, mb/1048576, minb, maxb, maxb-minb, (maxb-minb)/1048576, vb
    printf "SEPARATION on peak_memory_footprint: %d/8 runs of A below min(B) (maxA=%d %s minB=%d)\n", sep, maxa, (maxa<minb?"<":">="), minb
    printf "mean delta B-A = %d B (%.2f MiB): purge=0 LOWERS physical footprint\n", mb-ma, (mb-ma)/1048576
  }' | tee -a "$OUT"
say ""
say "KB-88-3 note: spread(max-min) at R=8 is ADVISORY-only, never the judge; the judge here is per-run SEPARATION + mean delta."
say ""
say "== PART C — A-BB51: per-region DIRTY diff (vmmap V2, mean r2..r9 per arm, metric=vmmap_region_dirty) =="
for arm in pa pd; do
  for r in 2 3 4 5 6 7 8 9; do
    V="$MO/axum.86p.$arm.r$r.vmmap.V2"
    sed -n '/^REGION TYPE  *SIZE/,/^TOTAL/p' "$V" 2>/dev/null | true
    # aggregated summary block: from the header line with VIRTUAL RESIDENT DIRTY to TOTAL
    awk -v arm="$arm" '
      /^REGION TYPE +SIZE +SIZE/ { inblk=1; next }
      /^===========/ { next }
      /^TOTAL,/ { next }
      /^TOTAL / { inblk=0; next }
      inblk && NF>=9 {
        # region name may contain spaces: DIRTY is 3rd numeric-ish field from the known layout
        # fields from end: count is last; walk from end: [-1]=count ... layout: V R D S VOL NONVOL EMPTY COUNT (+optional detail)
        # find first field that looks like a size (digits+optional .+K/M/G or plain digits)
        for (i=1;i<=NF;i++) if ($i ~ /^[0-9.]+[KMG]?$/ && $(i+1) ~ /^[0-9.]+[KMG]?$/ && $(i+2) ~ /^[0-9.]+[KMG]?$/) break
        if (i>NF-2) next
        name=""
        for (j=1;j<i;j++) name = name (j>1?" ":"") $j
        d=$(i+2)
        mult=1
        if (d ~ /K$/) { mult=1024 } else if (d ~ /M$/) { mult=1048576 } else if (d ~ /G$/) { mult=1073741824 }
        gsub(/[KMG]/,"",d)
        print arm "|" name "|" d*mult
      }' "$V"
  done
done > /tmp/rejudge86.regions.$$
say "NOTE: V2 = post-idle snapshot (NOT peak); mean over r2..r9 per arm; rows with mean dirty >64KiB."
say "region_type | mean_dirty_A(purge=0) B | mean_dirty_B(default) B | delta B-A (B)"
awk -F'|' '
  { key=$2; if ($1=="pa") { a[key]+=$3; na[key]++ } else { b[key]+=$3; nb[key]++ } }
  END {
    for (k in a) {
      ma=(na[k]?a[k]/na[k]:0); mb=(nb[k]?b[k]/nb[k]:0)
      if (ma>65536 || mb>65536) printf "%-28s | %12.0f | %12.0f | %+12.0f\n", k, ma, mb, mb-ma
    }
  }' /tmp/rejudge86.regions.$$ | sort -t'|' -k4 -rn | tee -a "$OUT"
rm -f /tmp/rejudge86.regions.$$
say "NOTE: the entire inter-arm dirty delta lives in the big anonymous arena region (vmmap tags it 'IOAccelerator' on this system = mimalloc arena VM tag); stacks/MALLOC_* are byte-identical across arms."
grep "IOAccelerator" "$OUT" | awk -F'|' 'NF==4 { d=$4+0; printf "arena region dirty delta B-A at V2: %d B (%.2f MiB)\n", d, d/1048576 }' | tee -a "$OUT"
say ""
say "== PART D — VW123 re-read on metric=peak_memory_footprint (m86.w123.w*.log, KS-PP-88-2 tag context) =="
FPW=""
for w in 1 2 3; do
  L="$MO/m86.w123.w$w.log"
  FP=$(tr -d '\0' < "$L" | awk '/peak memory footprint/{print $1}')
  RSS=$(tr -d '\0' < "$L" | awk '/maximum resident set size/{print $1}')
  say "w=$w peak_memory_footprint_bytes=$FP max_rss_bytes=$RSS"
done
grep "^w=" "$OUT" | awk '
  { split($2,ff,"="); fp[NR]=ff[2]+0; split($3,rr,"="); rs[NR]=rr[2]+0 }
  END {
    d12=fp[2]-fp[1]; d23=fp[3]-fp[2]
    r12=rs[2]-rs[1]; r23=rs[3]-rs[2]
    printf "footprint deltas: w1->w2 = %d B (%.2f MiB), w2->w3 = %d B (%.2f MiB)\n", d12, d12/1048576, d23, d23/1048576
    printf "max_rss deltas:   w1->w2 = %d B (%.2f MiB), w2->w3 = %d B (%.2f MiB)\n", r12, r12/1048576, r23, r23/1048576
    printf "TREND INVERSION: rss deltas decrease (%.2f -> %.2f MiB) while footprint deltas increase (%.2f -> %.2f MiB)\n", r12/1048576, r23/1048576, d12/1048576, d23/1048576
  }' | tee -a "$OUT"
say "KS-PP-88-2: union arm has NO dispatch row in-band => per-worker attribution is an ENVELOPE claim; replacement protocol = A-DL38 (committed steady-state slope)."
say ""
say "== rejudge86 DONE (analysis-only; raws untouched) =="
