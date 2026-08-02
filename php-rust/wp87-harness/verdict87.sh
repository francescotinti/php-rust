#!/bin/bash
# verdict87.sh — S-87.0 p5: verdetto macchina della campagna measure87.
# Fail-closed, COLLECT-THEN-EMIT per blocco (A-SK44/KS-SK-87-3); i blocchi
# a valle consumano il flag di pulizia dei blocchi sorgente (A-SK47/
# KS-SK-88-2: mai "defined" come prova di "clean"). NUL in un raw parsato =
# REFUSE del raw, MAI strip (A-SK47/KS-SK-88-3). Consuma m87.done
# (KG-88-3). Dispatch giudicato sulla MAPPA per-thread, non sulla sola
# somma (A-PP37). Ogni cifra porta il NOME della metrica (KB-88-2); mai
# max−min come giudice (KB-88-3: min-of-R + LSQ).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
OUT="$REPO/wp78-harness/measure-out"
VOUT="$HERE/verdict87.out"
: > "$VOUT"
say() { echo "$@" | tee -a "$VOUT"; }
FAILS=0

GIT_REV="$(git -C "$REPO" rev-parse --short HEAD)"

# --- .done consumption (KG-88-3) --------------------------------------------
DONE="$OUT/m87.done"
[ -f "$DONE" ] || { say "FAIL: m87.done missing — campaign never completed (KG-88-3)"; say "== VERDICT87 REFUSED =="; exit 1; }
D_GIT=$(sed -n 's/.* git=\([0-9a-f]*\) .*/\1/p' "$DONE")
ATT=$(sed -n 's/.* attempt=\([0-9]*\) .*/\1/p' "$DONE")
D_MH=$(sed -n 's/.* mem_hash=\([0-9a-f]*\) .*/\1/p' "$DONE")
if [ -z "$D_GIT" ] || [ -z "$ATT" ] || [ -z "$D_MH" ]; then
  say "FAIL: m87.done lacks git=/attempt=/mem_hash= fields"; say "== VERDICT87 REFUSED =="; exit 1
fi
say "== verdict87 — campaign git=$D_GIT attempt=$ATT mem_hash=$D_MH (judged at HEAD=$GIT_REV) =="

nul_free() { perl -0777 -ne 'exit(index($_,"\0")>=0?1:0)' "$1"; }

SLOPE_RAWS=""
for w in 1 2 3 4; do for r in 1 2 3 4 5; do SLOPE_RAWS="$SLOPE_RAWS m87.slope.w$w.r$r.a$ATT.memcensus"; done; done
EXTRA_RAWS=""
for r in 1 2 3; do EXTRA_RAWS="$EXTRA_RAWS m87.eager.w2.r$r.a$ATT.memcensus m87.minstack.w2.r$r.a$ATT.memcensus"; done
DISJ_RAWS="m87.cala.r1.a$ATT.memcensus m87.cala.r2.a$ATT.memcensus m87.calb.r1.a$ATT.memcensus m87.calb.r2.a$ATT.memcensus m87.conc.r1.a$ATT.memcensus m87.conc.r2.a$ATT.memcensus"

# --- Block VIDENT: presence, NUL-freedom, identity line, uniformity ---------
IDENT_CLEAN=0
bf=0; buf=""
emit() { buf="$buf$*
"; }
flushb() { if [ "$bf" = 0 ]; then printf '%s' "$buf" | tee -a "$VOUT"; else printf '%s' "$buf" | tee -a "$VOUT"; FAILS=$((FAILS+bf)); fi; buf=""; }
for raw in $SLOPE_RAWS $EXTRA_RAWS $DISJ_RAWS; do
  F="$OUT/$raw"
  if [ ! -f "$F" ]; then emit "FAIL VIDENT: raw missing: $raw"; bf=$((bf+1)); continue; fi
  if ! nul_free "$F"; then emit "FAIL VIDENT: NUL byte in $raw — raw REFUSED, never stripped (A-SK47/KS-SK-88-3)"; bf=$((bf+1)); continue; fi
  ID=$(grep "^mem_hash=" "$F" | tail -1)
  case "$ID" in
    *"mem_hash=$D_MH git=$D_GIT"*" attempt=$ATT "*) : ;;
    *) emit "FAIL VIDENT: identity line mismatch in $raw: $ID"; bf=$((bf+1));;
  esac
done
if [ "$bf" = 0 ]; then emit "VIDENT PASS: $(echo $SLOPE_RAWS $EXTRA_RAWS $DISJ_RAWS | wc -w | tr -d ' ') raws present, NUL-free, identity-uniform (mem_hash=$D_MH git=$D_GIT attempt=$ATT)"; IDENT_CLEAN=1; fi
flushb

# --- Block VDISP (A-PP37): per-thread dispatch MAP == expectation -----------
DISP_CLEAN=0
bf=0
if [ "$IDENT_CLEAN" = 1 ]; then
  for raw in $SLOPE_RAWS $EXTRA_RAWS $DISJ_RAWS; do
    F="$OUT/$raw"
    ID=$(grep "^mem_hash=" "$F" | tail -1)
    W=$(echo "$ID" | sed -n 's/.* w=\([0-9]*\) .*/\1/p')
    NREQ=$(echo "$ID" | sed -n 's/.* nreq=\([0-9]*\) .*/\1/p')
    MAP=$(awk '/tag=worker_dispatch/ { for (i=1;i<=NF;i++) { if ($i ~ /^thr=/) {t=$i; sub("thr=","",t)} if ($i ~ /^count=/) {c=$i; sub("count=","",c)} } print t":"c }' "$F" | sort -t: -k1,1n)
    NTHR=$(echo "$MAP" | grep -c . || true)
    if [ "$NTHR" != "$W" ]; then emit "FAIL VDISP: $raw has $NTHR dispatch rows, expected $W (A-PP37)"; bf=$((bf+1)); continue; fi
    PER=$((NREQ / W))
    BAD=$(echo "$MAP" | awk -F: -v per=$PER '$2+0 != per {print}')
    if [ -n "$BAD" ]; then emit "FAIL VDISP: $raw per-thread map != $PER each: $(echo $BAD | tr '\n' ' ') (A-PP37 — map, not sum)"; bf=$((bf+1)); fi
  done
  if [ "$bf" = 0 ]; then emit "VDISP PASS: per-thread dispatch MAP exact on every raw (each thread == nreq/W; A-PP37)"; DISP_CLEAN=1; fi
else
  emit "VDISP SKIPPED: upstream VIDENT not clean (KS-SK-88-2)"; bf=1
fi
flushb

# --- Block VORD (A-SK45 + KL-88-3 + A-DS39 quiescence) ----------------------
ORD_CLEAN=0
bf=0
if [ "$IDENT_CLEAN" = 1 ]; then
  for raw in $SLOPE_RAWS $EXTRA_RAWS $DISJ_RAWS; do
    F="$OUT/$raw"
    ID=$(grep "^mem_hash=" "$F" | tail -1)
    W=$(echo "$ID" | sed -n 's/.* w=\([0-9]*\) .*/\1/p')
    # unitcache_main_entry rows: exactly one per thread, ord==1, clamped==0
    NME=$(grep -c "tag=unitcache_main_entry" "$F" || true)
    if [ "$NME" != "$W" ]; then emit "FAIL VORD: $raw has $NME main-entry rows, expected $W"; bf=$((bf+1)); continue; fi
    BADORD=$(awk '/tag=unitcache_main_entry/ { ok=0; cl=""; for (i=1;i<=NF;i++) { if ($i=="ord=1") ok=1; if ($i ~ /^clamped=/) cl=$i } if (!ok || cl!="clamped=0") print NR": "$0 }' "$F")
    if [ -n "$BADORD" ]; then emit "FAIL VORD: $raw main-entry row with ord!=1 or clamped!=0 (A-SK45/A-DL36/KL-88-3):"; emit "$BADORD"; bf=$((bf+1)); fi
    # A-DS39 quiescence: every unitcache_thr row entries==1 mains==1
    BADQ=$(awk '/tag=unitcache_thr/ { e=""; m=""; for (i=1;i<=NF;i++) { if ($i ~ /^entries=/) e=$i; if ($i ~ /^mains=/) m=$i } if (e!="entries=1" || m!="mains=1") print NR": "$0 }' "$F")
    if [ -n "$BADQ" ]; then emit "FAIL VORD: $raw thread cache not quiescent-single (A-DS39):"; emit "$BADQ"; bf=$((bf+1)); fi
  done
  if [ "$bf" = 0 ]; then emit "VORD PASS: every main-entry row ord=1 clamped=0, every thread cache entries=1 mains=1 (A-SK45/A-DL36/A-DS39)"; ORD_CLEAN=1; fi
else
  emit "VORD SKIPPED: upstream VIDENT not clean (KS-SK-88-2)"; bf=1
fi
flushb

# --- Block VSLOPE (A-DL38≡A-BB52) -------------------------------------------
bf=0
if [ "$IDENT_CLEAN" = 1 ] && [ "$DISP_CLEAN" = 1 ] && [ "$ORD_CLEAN" = 1 ]; then
  # committed (post-collect win=0) per raw; min-of-R per W; LSQ slope.
  : > /tmp/vslope87.$$
  for w in 1 2 3 4; do
    for r in 1 2 3 4 5; do
      F="$OUT/m87.slope.w$w.r$r.a$ATT.memcensus"
      grep -q "exit_collect_mi" "$F" || { emit "FAIL VSLOPE: $F lacks exit_collect_mi marker — collect not armed"; bf=$((bf+1)); continue; }
      C=$(awk '/tag=mi_proc win=0/ { for (i=1;i<=NF;i++) if ($i ~ /^commit=/) {v=$i; sub("commit=","",v)} } END { print v+0 }' "$F")
      SL=$(awk 'BEGIN{inpost=0} /tag=mi_proc win=0/ { inpost=1; s=0 } inpost && /tag=mi_bin win=0/ { c=0; u=0; for (i=1;i<=NF;i++) { if ($i ~ /^committed=/) {c=$i; sub("committed=","",c)} if ($i ~ /^used_b=/) {u=$i; sub("used_b=","",u)} } s += c-u } END { print s+0 }' "$F")
      FP=$(tr -d '\0' < "$OUT/m87.slope.w$w.r$r.a$ATT.log" | awk '/peak memory footprint/{print $1}')
      echo "$w $r $C $SL ${FP:-0}" >> /tmp/vslope87.$$
      emit "slope w=$w r=$r committed_postcollect_win0_bytes=$C slack_committed_minus_used_bytes=$SL peak_memory_footprint_bytes=${FP:-NA}"
    done
  done
  if [ "$bf" = 0 ]; then
    SLOPE_REPORT=$(awk '
      { if (minc[$1]=="" || $3<minc[$1]) minc[$1]=$3
        if (minsl[$1]=="" || $4<minsl[$1]) minsl[$1]=$4
        if (minfp[$1]=="" || $5<minfp[$1]) minfp[$1]=$5 }
      END {
        n=0; sx=0; sy=0; sxx=0; sxy=0
        for (w=1;w<=4;w++) {
          printf "W=%d min-of-R: committed=%d B (%.2f MiB) slack=%d B peak_footprint=%d B\n", w, minc[w], minc[w]/1048576, minsl[w], minfp[w]
          n++; sx+=w; sy+=minc[w]; sxx+=w*w; sxy+=w*minc[w]
        }
        slope=(n*sxy-sx*sy)/(n*sxx-sx*sx)
        for (w=1;w<4;w++) printf "delta committed W%d->W%d = %d B (%.2f MiB)\n", w, w+1, minc[w+1]-minc[w], (minc[w+1]-minc[w])/1048576
        printf "VSLOPE LSQ slope (metric=committed_postcollect_win0, min-of-R, W=1..4): %.0f B/worker (%.2f MiB)\n", slope, slope/1048576
        band=3605572
        lo=band*0.95; hi=band*1.05
        if (slope>=lo && slope<=hi) printf "VSLOPE verdict: WITHIN the KL-85-2 derivation band 3605572 B +/-5%% [%.0f, %.0f]\n", lo, hi
        else printf "VSLOPE verdict: NAMED-DEVIATION — slope %.0f B outside 3605572 B +/-5%% [%.0f, %.0f] (protocol clean; the physics answer is the answer)\n", slope, lo, hi
        for (w=1;w<4;w++) printf "slack delta W%d->W%d = %d B (named term of the A-DL38 closure)\n", w, w+1, minsl[w+1]-minsl[w]
      }' /tmp/vslope87.$$)
    emit "$SLOPE_REPORT"
    emit "VSLOPE PASS: protocol clean (collect armed, min-of-R, LSQ; KB-88-3 respected — no max-min judged)"
  fi
  rm -f /tmp/vslope87.$$
else
  emit "VSLOPE SKIPPED: upstream blocks not clean (KS-SK-88-2)"; bf=$((bf+1))
fi
flushb

# --- Block VARMS (candidate subtraction, ADVISORY R=3) ----------------------
bf=0
if [ "$IDENT_CLEAN" = 1 ] && [ "$DISP_CLEAN" = 1 ]; then
  base=$(for r in 1 2 3 4 5; do awk '/tag=mi_proc win=0/ { for (i=1;i<=NF;i++) if ($i ~ /^commit=/) {v=$i; sub("commit=","",v)} } END { print v+0 }' "$OUT/m87.slope.w2.r$r.a$ATT.memcensus"; done | sort -n | head -1)
  for arm in eager minstack; do
    a=$(for r in 1 2 3; do awk '/tag=mi_proc win=0/ { for (i=1;i<=NF;i++) if ($i ~ /^commit=/) {v=$i; sub("commit=","",v)} } END { print v+0 }' "$OUT/m87.$arm.w2.r$r.a$ATT.memcensus"; done | sort -n | head -1)
    d=$((a - base))
    emit "VARMS $arm: min-of-R committed W=2 = $a B vs base $base B — delta $d B (metric=committed_postcollect_win0; ADVISORY: R=3 arm vs R=5 base, candidate named, never a pin)"
  done
  emit "VARMS PASS (advisory candidate arms reported by name)"
else
  emit "VARMS SKIPPED: upstream not clean (KS-SK-88-2)"; bf=$((bf+1))
fi
flushb

# --- Block VDISJ (A-BB53) ---------------------------------------------------
bf=0
if [ "$IDENT_CLEAN" = 1 ] && [ "$DISP_CLEAN" = 1 ] && [ "$ORD_CLEAN" = 1 ]; then
  # calibrations: net+floor_inc per side, r1 must equal r2 at byte
  getnet() { awk -v pat="$2" '/tag=unitcache_main_entry/ && $0 ~ pat { for (i=1;i<=NF;i++) { if ($i ~ /^net=/) {n=$i; sub("net=","",n)} if ($i ~ /^floor_inc=/) {f=$i; sub("floor_inc=","",f)} } print n" "f }' "$1"; }
  CA1=$(getnet "$OUT/m87.cala.r1.a$ATT.memcensus" "pad87a[.]php")
  CA2=$(getnet "$OUT/m87.cala.r2.a$ATT.memcensus" "pad87a[.]php")
  CB1=$(getnet "$OUT/m87.calb.r1.a$ATT.memcensus" "pad87b[.]php")
  CB2=$(getnet "$OUT/m87.calb.r2.a$ATT.memcensus" "pad87b[.]php")
  if [ -z "$CA1" ] || [ -z "$CB1" ]; then emit "FAIL VDISJ: calibration rows missing"; bf=$((bf+1)); fi
  if [ "$CA1" != "$CA2" ]; then emit "FAIL VDISJ: cala r1 ($CA1) != r2 ($CA2) — calibration not byte-reproduced"; bf=$((bf+1)); fi
  if [ "$CB1" != "$CB2" ]; then emit "FAIL VDISJ: calb r1 ($CB1) != r2 ($CB2) — calibration not byte-reproduced"; bf=$((bf+1)); fi
  if [ "$bf" = 0 ]; then
    CALA_NET=${CA1%% *}; CALA_FI=${CA1##* }
    CALB_NET=${CB1%% *}; CALB_FI=${CB1##* }
    emit "VDISJ cal: pad87a net=$CALA_NET B floor_inc=$CALA_FI B | pad87b net=$CALB_NET B floor_inc=$CALB_FI B (metric=net-at-lower, net_window=process-counters, W=1 sequential — legal per KL-87-2)"
    for a in 1 2; do
      F="$OUT/m87.conc.r$a.a$ATT.memcensus"
      OV=$(awk '
        /tag=lower_span/ {
          tid=""; t0=""; t1=""
          for (i=1;i<=NF;i++) {
            if ($i ~ /^tid=/) tid=$i
            if ($i ~ /^t0_us=/) { v=$i; sub("t0_us=","",v); t0=v+0 }
            if ($i ~ /^t1_us=/) { v=$i; sub("t1_us=","",v); t1=v+0 }
          }
          if ($NF ~ /pad87a[.]php$/) { a0=t0; a1=t1; atid=tid }
          else if ($NF ~ /pad87b[.]php$/) { b0=t0; b1=t1; btid=tid }
        }
        END {
          if (atid=="" || btid=="" || atid==btid) { print "INVALID"; exit }
          print (a0<b1 && b0<a1) ? "OVERLAP" : "NO-OVERLAP"
        }' "$F")
      NA=$(getnet "$F" "pad87a[.]php"); NB=$(getnet "$F" "pad87b[.]php")
      NA_NET=${NA%% *}; NA_FI=${NA##* }
      NB_NET=${NB%% *}; NB_FI=${NB##* }
      SUM=$((CALA_NET + CALB_NET))
      emit "VDISJ conc r$a: spans=$OV padA net=$NA_NET floor_inc=$NA_FI | padB net=$NB_NET floor_inc=$NB_FI | calA+calB=$SUM B"
      emit "VDISJ conc r$a: nets under concurrency are VOID as per-thread figures (KB-88-1) — reported only to scope the decomposition"
      if [ "$OV" = "OVERLAP" ]; then
        DA=$((NA_NET - SUM)); DB=$((NB_NET - SUM))
        emit "VDISJ conc r$a prediction check (ex-ante): |netX-(calA+calB)| -> padA delta=$DA B, padB delta=$DB B (swallow signature on a NEST-FREE pair)"
        FDA=$((NA_FI - CALA_FI)); FDB=$((NB_FI - CALB_FI))
        emit "VDISJ conc r$a floor prediction (ex-ante): floor_inc==cal floor_inc per side -> padA delta=$FDA B, padB delta=$FDB B (nest-free => no shared-node discount)"
      fi
    done
    emit "VDISJ PASS: calibrations byte-reproduced; concurrency observations reported with KB-88-1 tags"
  fi
else
  emit "VDISJ SKIPPED: upstream not clean (KS-SK-88-2)"; bf=$((bf+1))
fi
flushb

if [ "$FAILS" = 0 ]; then
  say "== VERDICT87 PASS (git=$D_GIT attempt=$ATT) =="
  exit 0
else
  say "== VERDICT87 FAIL($FAILS) =="
  exit 1
fi
