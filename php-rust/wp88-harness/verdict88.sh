#!/bin/bash
# verdict88.sh — S-88.0 p5: verdetto macchina della campagna measure88.
# Fail-closed, COLLECT-THEN-EMIT per blocco (A-SK44); i blocchi a valle
# consumano i FLAG di pulizia a monte (A-SK47/KS-SK-88-2/KS-SK-89-3).
# Novità WP-89 (tutte le sedie):
#   A-SK54/A-BG46: verdetto PER-ATTEMPT e PER-GENERAZIONE
#     (verdict88.aN.gG.out, name-reuse rifiutato); coerenza pointer<->
#     suffixed .done giudicata; esito appeso al LEDGER (A-AH49/KG-89-1).
#   A-SK51: identity line count==1; (VARMS assente in questa campagna —
#     nessun braccio d'ambiente: KL-89-1 non innescabile).
#   A-BG47/KG-89-2: identity v2 giudicata — server_exit==0, srv_pid=
#     riconciliato con TUTTE le righe pid= del raw.
#   A-PP42/KS-PP-89-2: VDISP esige thr-set == {0..W-1} ESATTO
#     (unicità+completezza), mai la sola cardinalità.
#   A-BG48/KG-89-3: companion mancante = FAIL del raw — mai 0/NA dentro
#     min/LSQ; righe peak dai .log con nul_count= IN-BAND (A-SK52: canale
#     DECLARED-DEVIATION quando nul_count>0, mai strip silenzioso).
#   A-BB55/KB-89-1/2, KL-89-2: mode-census per W; LSQ di b sul segmento
#     W in {4,8,12,16} sui valori del MODO DOMINANTE; min-of-R ADVISORY
#     quando i modi sono >=2; banda KL-85-2 confrontata SOLO con b;
#     monotonia dei Delta giudicata (non monotoni => slope ADVISORY).
#   A-BB55: blocco VARENA — righe tag=mi_arena obbligatorie, parse=FAILED
#     = FAIL; granuli 64 KiB nominati.
#   A-BB56: blocco VWARM — discriminazione MECCANICA del surplus padA
#     (base vs warm vs stag) contro le predizioni ex-ante del header
#     campagna; net concorrenti TAGGATI VOID per-thread (KB-88-1).
#   A-DS45/KS-DS-88-1: blocco VUCLOG — putord-pair-guard su log di
#     PRODUZIONE (>=1 coppia, >=2 main_evicted, W==1).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
OUT="$REPO/wp78-harness/measure-out"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD)"

# --- .done consumption (KG-88-3) + A-SK54 coherence -------------------------
DONE="$OUT/m88.done"
[ -f "$DONE" ] || { echo "FAIL: m88.done missing — campaign never completed (KG-88-3)"; exit 1; }
ATT=$(sed -n 's/.* attempt=\([0-9]*\) .*/\1/p' "$DONE")
ADONE="$OUT/m88.a$ATT.done"
[ -f "$ADONE" ] || { echo "FAIL: per-attempt done $ADONE missing (A-SK54)"; exit 1; }
cmp -s "$DONE" "$ADONE" || { echo "FAIL: m88.done pointer != m88.a$ATT.done (A-SK54 coherence)"; exit 1; }
D_GIT=$(sed -n 's/.* git=\([0-9a-f]*\) .*/\1/p' "$DONE")
D_MH=$(sed -n 's/.* mem_hash=\([0-9a-f]*\) .*/\1/p' "$DONE")
if [ -z "$D_GIT" ] || [ -z "$ATT" ] || [ -z "$D_MH" ]; then
  echo "FAIL: m88.done lacks git=/attempt=/mem_hash= fields"; exit 1
fi

# per-attempt, per-generation verdict file (A-BG46: generations NEVER
# overwrite — a re-judgment gets a fresh gG name).
G=1
while [ -e "$HERE/verdict88.a$ATT.g$G.out" ]; do G=$((G+1)); done
VOUT="$HERE/verdict88.a$ATT.g$G.out"
: > "$VOUT"
say() { echo "$@" | tee -a "$VOUT"; }
FAILS=0
say "== verdict88 — campaign git=$D_GIT attempt=$ATT mem_hash=$D_MH (judged at HEAD=$GIT_REV, generation g$G) =="

nul_free() { perl -0777 -ne 'exit(index($_,"\0")>=0?1:0)' "$1"; }
nul_count() { perl -0777 -ne 'my $c=()=/\0/g; print $c' "$1"; }

WS="4 8 12 16"
SLOPE_RAWS=""
for w in $WS; do for r in 1 2 3 4 5; do SLOPE_RAWS="$SLOPE_RAWS m88.slope.w$w.r$r.a$ATT.memcensus"; done; done
CAL_RAWS="m88.cala.r1.a$ATT.memcensus m88.cala.r2.a$ATT.memcensus m88.calb.r1.a$ATT.memcensus m88.calb.r2.a$ATT.memcensus"
CONC_RAWS=""
for m in base warm stag; do for r in 1 2; do CONC_RAWS="$CONC_RAWS m88.conc$m.r$r.a$ATT.memcensus"; done; done
UCLOG_RAW="m88.uclog.a$ATT.memcensus"
ALL_RAWS="$SLOPE_RAWS $CAL_RAWS $CONC_RAWS $UCLOG_RAW"

bf=0; buf=""
emit() { buf="$buf$*
"; }
flushb() { printf '%s' "$buf" | tee -a "$VOUT" >/dev/null; FAILS=$((FAILS+bf)); buf=""; }

# --- Block VIDENT v2 (A-SK51 + A-BG47 + KG-89-2) ----------------------------
IDENT_CLEAN=0
bf=0
for raw in $ALL_RAWS; do
  F="$OUT/$raw"
  if [ ! -f "$F" ]; then emit "FAIL VIDENT: raw missing: $raw"; bf=$((bf+1)); continue; fi
  if ! nul_free "$F"; then emit "FAIL VIDENT: NUL byte in $raw — raw REFUSED, never stripped (KS-SK-88-3)"; bf=$((bf+1)); continue; fi
  NID=$(grep -c "^mem_hash=" "$F")
  if [ "$NID" != 1 ]; then emit "FAIL VIDENT: $raw has $NID identity lines, expected exactly 1 (A-SK51)"; bf=$((bf+1)); continue; fi
  ID=$(grep "^mem_hash=" "$F")
  case "$ID" in
    *"mem_hash=$D_MH git=$D_GIT"*" attempt=$ATT "*) : ;;
    *) emit "FAIL VIDENT: identity mismatch in $raw: $ID"; bf=$((bf+1)); continue;;
  esac
  SEXIT=$(echo "$ID" | sed -n 's/.* server_exit=\([0-9-]*\) .*/\1/p')
  [ "$SEXIT" = 0 ] || { emit "FAIL VIDENT: $raw server_exit=$SEXIT != 0 (A-BG47/KG-89-2 — the a1 orphan signal, now judged)"; bf=$((bf+1)); continue; }
  SPID=$(echo "$ID" | sed -n 's/.* srv_pid=\([0-9]*\) .*/\1/p')
  [ -n "$SPID" ] || { emit "FAIL VIDENT: $raw identity lacks srv_pid= (A-BG47)"; bf=$((bf+1)); continue; }
  BADPID=$(awk -v want="$SPID" '/^pid=/ { split($1,a,"="); if (a[2]+0 != want+0) { print; exit } }' "$F")
  [ -z "$BADPID" ] || { emit "FAIL VIDENT: $raw census pid rows not reconciled with srv_pid=$SPID (KG-89-2): $BADPID"; bf=$((bf+1)); continue; }
done
if [ "$bf" = 0 ]; then emit "VIDENT PASS: $(echo $ALL_RAWS | wc -w | tr -d ' ') raws present, NUL-free, identity v2 (count==1, server_exit==0, srv_pid reconciled; mem_hash=$D_MH git=$D_GIT attempt=$ATT)"; IDENT_CLEAN=1; fi
flushb

# --- Block VDISP (A-PP37 + A-PP42/KS-PP-89-2: exact thr-set) ----------------
DISP_CLEAN=0
bf=0
if [ "$IDENT_CLEAN" = 1 ]; then
  for raw in $ALL_RAWS; do
    F="$OUT/$raw"
    ID=$(grep "^mem_hash=" "$F")
    W=$(echo "$ID" | sed -n 's/.* w=\([0-9]*\) .*/\1/p')
    NREQ=$(echo "$ID" | sed -n 's/.* nreq=\([0-9]*\) .*/\1/p')
    MAP=$(awk '/tag=worker_dispatch/ { t=""; c=""; for (i=1;i<=NF;i++) { if ($i ~ /^thr=/) {t=$i; sub("thr=","",t)} if ($i ~ /^count=/) {c=$i; sub("count=","",c)} } print t":"c }' "$F" | sort -t: -k1,1n)
    THRSET=$(echo "$MAP" | cut -d: -f1 | tr '\n' ',' | sed 's/,$//')
    WANTSET=$(seq 0 $((W-1)) | tr '\n' ',' | sed 's/,$//')
    if [ "$THRSET" != "$WANTSET" ]; then
      emit "FAIL VDISP: $raw thr-set {$THRSET} != {$WANTSET} — uniqueness+completeness, not cardinality (A-PP42/KS-PP-89-2)"; bf=$((bf+1)); continue
    fi
    PER=$((NREQ / W))
    BAD=$(echo "$MAP" | awk -F: -v per=$PER '$2+0 != per {print}')
    if [ -n "$BAD" ]; then emit "FAIL VDISP: $raw per-thread map != $PER each: $(echo $BAD | tr '\n' ' ')"; bf=$((bf+1)); fi
  done
  if [ "$bf" = 0 ]; then emit "VDISP PASS: thr-set == {0..W-1} exact and per-thread count == nreq/W on every raw (A-PP37+A-PP42)"; DISP_CLEAN=1; fi
else
  emit "VDISP SKIPPED: upstream VIDENT not clean (KS-SK-88-2)"; bf=1
fi
flushb

# --- Block VORD (A-SK45 + A-DL36 + A-DS39; mode-aware for warm) -------------
ORD_CLEAN=0
bf=0
if [ "$IDENT_CLEAN" = 1 ]; then
  for raw in $SLOPE_RAWS $CAL_RAWS $CONC_RAWS; do
    F="$OUT/$raw"
    ID=$(grep "^mem_hash=" "$F")
    W=$(echo "$ID" | sed -n 's/.* w=\([0-9]*\) .*/\1/p')
    case "$raw" in
      *concwarm*) EXP_PER_THR=2 ;;   # hello(ord=1) + pad(ord=2) per worker
      *)          EXP_PER_THR=1 ;;
    esac
    NME=$(grep -c "tag=unitcache_main_entry" "$F" || true)
    WANTME=$((W * EXP_PER_THR))
    if [ "$NME" != "$WANTME" ]; then emit "FAIL VORD: $raw has $NME main-entry rows, expected $WANTME"; bf=$((bf+1)); continue; fi
    BADORD=$(awk -v maxo=$EXP_PER_THR '/tag=unitcache_main_entry/ { ok=0; cl=""; for (i=1;i<=NF;i++) { if ($i ~ /^ord=/) { o=$i; sub("ord=","",o); if (o+0>=1 && o+0<=maxo) ok=1 } if ($i ~ /^clamped=/) cl=$i } if (!ok || cl!="clamped=0") print NR": "$0 }' "$F")
    if [ -n "$BADORD" ]; then emit "FAIL VORD: $raw main-entry row outside ord profile (1..$EXP_PER_THR) or clamped!=0:"; emit "$BADORD"; bf=$((bf+1)); fi
    BADQ=$(awk -v e="entries=$EXP_PER_THR" -v m="mains=$EXP_PER_THR" '/tag=unitcache_thr/ { ee=""; mm=""; for (i=1;i<=NF;i++) { if ($i ~ /^entries=/) ee=$i; if ($i ~ /^mains=/) mm=$i } if (ee!=e || mm!=m) print NR": "$0 }' "$F")
    if [ -n "$BADQ" ]; then emit "FAIL VORD: $raw thread cache not at expected profile ($EXP_PER_THR/$EXP_PER_THR):"; emit "$BADQ"; bf=$((bf+1)); fi
  done
  if [ "$bf" = 0 ]; then emit "VORD PASS: main-entry ord profile + clamped=0 + thread-cache profile hold on every judged raw (warm profile 2/2 declared)"; ORD_CLEAN=1; fi
else
  emit "VORD SKIPPED: upstream VIDENT not clean (KS-SK-88-2)"; bf=1
fi
flushb

# --- Block VARENA (A-BB55: in-band arena/commit term) -----------------------
ARENA_CLEAN=0
bf=0
if [ "$IDENT_CLEAN" = 1 ]; then
  for raw in $SLOPE_RAWS; do
    F="$OUT/$raw"
    grep -q "tag=mi_arena_json win=0" "$F" || { emit "FAIL VARENA: $raw lacks mi_arena_json win=0 row (A-BB55)"; bf=$((bf+1)); continue; }
    grep -q "tag=mi_arena win=0 .*parse=FAILED" "$F" && { emit "FAIL VARENA: $raw has parse=FAILED mi_arena rows"; bf=$((bf+1)); continue; }
    grep -q "tag=mi_arena win=0 key=committed " "$F" || { emit "FAIL VARENA: $raw lacks extracted committed row"; bf=$((bf+1)); continue; }
  done
  if [ "$bf" = 0 ]; then emit "VARENA PASS: mi_arena_json + extracted rows present, no parse failures, on every slope raw"; ARENA_CLEAN=1; fi
else
  emit "VARENA SKIPPED: upstream VIDENT not clean (KS-SK-88-2)"; bf=1
fi
flushb

# --- Block VSLOPE-HI (A-BB55≡A-DL42; KB-89-1/2; KL-89-2; A-BG48) ------------
bf=0
if [ "$IDENT_CLEAN" = 1 ] && [ "$DISP_CLEAN" = 1 ] && [ "$ORD_CLEAN" = 1 ] && [ "$ARENA_CLEAN" = 1 ]; then
  TMP=$(mktemp)
  for w in $WS; do
    for r in 1 2 3 4 5; do
      F="$OUT/m88.slope.w$w.r$r.a$ATT.memcensus"
      grep -q "exit_collect_mi" "$F" || { emit "FAIL VSLOPE: $F lacks exit_collect_mi marker"; bf=$((bf+1)); continue; }
      C=$(awk '/tag=mi_proc win=0/ { for (i=1;i<=NF;i++) if ($i ~ /^commit=/) {v=$i; sub("commit=","",v)} } END { print v+0 }' "$F")
      SL=$(awk 'BEGIN{inpost=0} /tag=mi_proc win=0/ { inpost=1; s=0 } inpost && /tag=mi_bin win=0/ { c=0; u=0; for (i=1;i<=NF;i++) { if ($i ~ /^committed=/) {c=$i; sub("committed=","",c)} if ($i ~ /^used_b=/) {u=$i; sub("used_b=","",u)} } s += c-u } END { print s+0 }' "$F")
      AC=$(awk '/tag=mi_arena win=0 key=committed /{ for (i=1;i<=NF;i++) if ($i ~ /^current=/) {v=$i; sub("current=","",v)} } END { print v+0 }' "$F")
      AN=$(awk '/tag=mi_arena win=0 key=arena_count /{ for (i=1;i<=NF;i++) if ($i ~ /^val=/) {v=$i; sub("val=","",v)} } END { print v+0 }' "$F")
      L="$OUT/m88.slope.w$w.r$r.a$ATT.log"
      [ -f "$L" ] || { emit "FAIL VSLOPE: companion log missing for w=$w r=$r — raw FAILS, never defaulted (A-BG48)"; bf=$((bf+1)); continue; }
      NC=$(nul_count "$L")
      if [ "$NC" = 0 ]; then
        FP=$(awk '/peak memory footprint/{print $1}' "$L")
        DEVTAG=""
      else
        FP=$(tr -d '\0' < "$L" | awk '/peak memory footprint/{print $1}')
        DEVTAG=" DECLARED-DEVIATION nul_count=$NC (A-SK52/KS-SK-89-2: parsed via strip, deviation in-band)"
      fi
      [ -n "$FP" ] || { emit "FAIL VSLOPE: peak companion MISSING for w=$w r=$r — FAIL, never 0/NA in an aggregate (A-BG48/KG-89-3)"; bf=$((bf+1)); continue; }
      echo "$w $r $C $SL $FP $AC $AN" >> "$TMP"
      emit "slope w=$w r=$r committed_postcollect_win0_bytes=$C slack_committed_minus_used_bytes=$SL peak_memory_footprint_bytes=$FP arena_committed_current_bytes=$AC arena_count=$AN$DEVTAG"
    done
  done
  if [ "$bf" = 0 ]; then
    SLOPE_REPORT=$(awk '
      { vals[$1] = vals[$1] " " $3
        if (minc[$1]=="" || $3<minc[$1]) minc[$1]=$3
        if (minsl[$1]=="" || $4<minsl[$1]) minsl[$1]=$4
        if (minfp[$1]=="" || $5<minfp[$1]) minfp[$1]=$5
        if (minac[$1]=="" || $6<minac[$1]) minac[$1]=$6
        an[$1]=$7 }
      END {
        nws=split("4 8 12 16", wsarr, " ")
        # mode census per W (KB-89-2): distinct byte values + multiplicity
        multi_mode=0
        for (i=1;i<=nws;i++) {
          w=wsarr[i]
          n=split(vals[w], vv, " ")
          delete cnt
          for (j=1;j<=n;j++) cnt[vv[j]]++
          best=""; bestn=0; nm=0
          for (v in cnt) {
            nm++
            printf "mode-census W=%d: committed=%d B x%d\n", w, v, cnt[v]
            if (cnt[v]>bestn || (cnt[v]==bestn && (best=="" || v+0<best+0))) { best=v; bestn=cnt[v] }
          }
          if (nm>=2) multi_mode=1
          dom[w]=best
          printf "mode-census W=%d: modes=%d dominant=%d B (x%d) min-of-R=%d B%s\n", w, nm, best, bestn, minc[w], (nm>=2 ? " [min-of-R ADVISORY: >=2 byte-distinct modes, KB-89-2]" : "")
          printf "W=%d granules64k: dominant=%.2f min=%.2f residue_dominant=%d B | arena committed min=%d B arena_count=%d\n", w, dom[w]/65536, minc[w]/65536, dom[w]%65536, minac[w], an[w]
        }
        # LSQ of b over W in {4,8,12,16} on DOMINANT-mode values (KL-89-2)
        n=0; sx=0; sy=0; sxx=0; sxy=0
        for (i=1;i<=nws;i++) { w=wsarr[i]; n++; sx+=w; sy+=dom[w]; sxx+=w*w; sxy+=w*dom[w] }
        b=(n*sxy-sx*sy)/(n*sxx-sx*sx)
        a=(sy-b*sx)/n
        mono=1
        for (i=2;i<=nws;i++) {
          d=dom[wsarr[i]]-dom[wsarr[i-1]]
          printf "delta dominant W%d->W%d = %d B (%.2f MiB) over %d workers = %.0f B/worker\n", wsarr[i-1], wsarr[i], d, d/1048576, wsarr[i]-wsarr[i-1], d/(wsarr[i]-wsarr[i-1])
          if (d<=0) mono=0
        }
        printf "VSLOPE-HI b (LSQ dominant-mode, metric=committed_postcollect_win0, W in {4,8,12,16}): %.0f B/worker (%.2f MiB) a=%.0f B\n", b, b/1048576, a
        # advisory min-based for continuity
        n=0; sx=0; sy=0; sxx=0; sxy=0
        for (i=1;i<=nws;i++) { w=wsarr[i]; n++; sx+=w; sy+=minc[w]; sxx+=w*w; sxy+=w*minc[w] }
        bm=(n*sxy-sx*sy)/(n*sxx-sx*sx)
        printf "VSLOPE-HI b_min (LSQ min-of-R, ADVISORY under multi-mode): %.0f B/worker (%.2f MiB)\n", bm, bm/1048576
        band=3605572; lo=band*0.95; hi=band*1.05
        grade=(mono ? "monotone deltas" : "NON-MONOTONE deltas => slope ADVISORY, never verdict-grade (KB-89-1)")
        if (b>=lo && b<=hi) printf "VSLOPE-HI verdict: b WITHIN the KL-85-2 band 3605572 B +/-5%% [%.0f, %.0f] — %s\n", lo, hi, grade
        else printf "VSLOPE-HI verdict: b NAMED-DEVIATION — %.0f B outside 3605572 B +/-5%% [%.0f, %.0f] (band compared with b ONLY, KL-89-2) — %s\n", b, lo, hi, grade
      }' "$TMP")
    emit "$SLOPE_REPORT"
    emit "VSLOPE-HI PASS: protocol clean (collect armed, mode-census in-band, b on dominant modes, companions never defaulted)"
  fi
  rm -f "$TMP"
else
  emit "VSLOPE-HI SKIPPED: upstream blocks not clean (KS-SK-88-2)"; bf=$((bf+1))
fi
flushb

# --- Block VWARM (A-BB56: mechanical discrimination) ------------------------
bf=0
if [ "$IDENT_CLEAN" = 1 ] && [ "$DISP_CLEAN" = 1 ] && [ "$ORD_CLEAN" = 1 ]; then
  getnet() { awk -v pat="$2" '/tag=unitcache_main_entry/ && $0 ~ pat { for (i=1;i<=NF;i++) { if ($i ~ /^net=/) {n=$i; sub("net=","",n)} if ($i ~ /^floor_inc=/) {f=$i; sub("floor_inc=","",f)} } print n" "f }' "$1"; }
  CA1=$(getnet "$OUT/m88.cala.r1.a$ATT.memcensus" "pad87a[.]php")
  CA2=$(getnet "$OUT/m88.cala.r2.a$ATT.memcensus" "pad87a[.]php")
  CB1=$(getnet "$OUT/m88.calb.r1.a$ATT.memcensus" "pad87b[.]php")
  CB2=$(getnet "$OUT/m88.calb.r2.a$ATT.memcensus" "pad87b[.]php")
  if [ -z "$CA1" ] || [ -z "$CB1" ]; then emit "FAIL VWARM: calibration rows missing"; bf=$((bf+1)); fi
  [ "$CA1" = "$CA2" ] || { emit "FAIL VWARM: cala r1 ($CA1) != r2 ($CA2) — not byte-reproduced"; bf=$((bf+1)); }
  [ "$CB1" = "$CB2" ] || { emit "FAIL VWARM: calb r1 ($CB1) != r2 ($CB2) — not byte-reproduced"; bf=$((bf+1)); }
  if [ "$bf" = 0 ]; then
    CALA_NET=${CA1%% *}; CALA_FI=${CA1##* }
    CALB_NET=${CB1%% *}; CALB_FI=${CB1##* }
    SUM=$((CALA_NET + CALB_NET))
    emit "VWARM cal: pad87a net=$CALA_NET B floor_inc=$CALA_FI B | pad87b net=$CALB_NET B floor_inc=$CALB_FI B [metric=net-at-lower, net_window=process-counters, W=1 sequential]"
    for m in base warm stag; do
      for r in 1 2; do
        F="$OUT/m88.conc$m.r$r.a$ATT.memcensus"
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
        [ -n "$NA" ] && [ -n "$NB" ] || { emit "FAIL VWARM: conc$m.r$r pad entries missing"; bf=$((bf+1)); continue; }
        NA_NET=${NA%% *}; NA_FI=${NA##* }
        NB_NET=${NB%% *}; NB_FI=${NB##* }
        DA=$((NA_NET - SUM)); DB=$((NB_NET - SUM))
        eval "DA_${m}_${r}=$DA"
        emit "VWARM conc$m r$r: spans=$OV padA net=$NA_NET B floor_inc=$NA_FI B | padB net=$NB_NET B floor_inc=$NB_FI B | dA=$DA B dB=$DB B [nets under concurrency VOID as per-thread figures, KB-88-1 — reported to discriminate the driver only]"
      done
    done
    if [ "$bf" = 0 ]; then
      DAB=$(( (DA_base_1 + DA_base_2) / 2 ))
      DAW=$(( (DA_warm_1 + DA_warm_2) / 2 ))
      DAS=$(( (DA_stag_1 + DA_stag_2) / 2 ))
      VER="UNRESOLVED"
      ABSW=${DAW#-}; ABSB=${DAB#-}
      if [ "$ABSB" -gt 0 ] && [ $((ABSW * 5)) -lt "$ABSB" ]; then VER="FIRST-TOUCH (dA collapses when both workers are warmed — P-WARM confirmed)"
      elif [ "$ABSB" -gt 0 ] && [ $((ABSW * 10)) -gt $((ABSB * 8)) ]; then VER="PER-REQUEST (dA survives warm — P-WARM refuted)"
      fi
      emit "VWARM discrimination: mean dA base=$DAB B | warm=$DAW B | stag=$DAS B -> $VER"
      emit "VWARM PASS: calibrations byte-reproduced; discrimination reported against the ex-ante predictions (P-BASE/P-WARM/P-STAG in campaign header)"
    fi
  fi
else
  emit "VWARM SKIPPED: upstream not clean (KS-SK-88-2)"; bf=$((bf+1))
fi
flushb

# --- Block VUCLOG (A-DS45/KS-DS-88-1) ---------------------------------------
bf=0
if [ "$IDENT_CLEAN" = 1 ]; then
  F="$OUT/$UCLOG_RAW"
  UCL="$OUT/m88.uclog.a$ATT.uclog"
  ID=$(grep "^mem_hash=" "$F")
  W=$(echo "$ID" | sed -n 's/.* w=\([0-9]*\) .*/\1/p')
  [ "$W" = 1 ] || { emit "FAIL VUCLOG: armed-log phase ran at W=$W != 1 (KH89-1: putord not opposable at W>1 without thr=)"; bf=$((bf+1)); }
  [ -f "$UCL" ] || { emit "FAIL VUCLOG: production uclog missing: $UCL"; bf=$((bf+1)); }
  if [ "$bf" = 0 ]; then
    NEV=$(grep -c "^unitcache main_evicted " "$UCL" || true)
    [ "$NEV" -ge 2 ] || { emit "FAIL VUCLOG: only $NEV main_evicted rows in production log, expected >=2 (A-DS45)"; bf=$((bf+1)); }
    GOUT=$("$REPO/wp87-harness/putord-pair-guard.pl" "$UCL" 2>&1) || { emit "FAIL VUCLOG: putord-pair-guard VOID on production log: $GOUT"; bf=$((bf+1)); }
    NP=$(echo "$GOUT" | sed -n 's/^putord-pair-guard: \([0-9]*\) pair.*/\1/p')
    if [ "$bf" = 0 ]; then
      [ -n "$NP" ] && [ "$NP" -ge 1 ] || { emit "FAIL VUCLOG: zero pairs verified — positive is vacuous (A-DS45)"; bf=$((bf+1)); }
    fi
    if [ "$bf" = 0 ]; then emit "VUCLOG PASS: production log W=1, main_evicted=$NEV, putord pairs verified=$NP (first NON-selftest bite of the pair-guard; KS-DS-88-1 satisfied on production)"; fi
  fi
else
  emit "VUCLOG SKIPPED: upstream VIDENT not clean (KS-SK-88-2)"; bf=$((bf+1))
fi
flushb

LEDGER="$OUT/m88.campaign.ledger"
if [ "$FAILS" = 0 ]; then
  say "== VERDICT88 PASS (git=$D_GIT attempt=$ATT, generation g$G) =="
  echo "attempt_epoch=$(date +%s) git=$GIT_REV campaign_sha=judge attempt=$ATT phase=verdict esito=PASS verdict_file=wp88-harness/verdict88.a$ATT.g$G.out" >> "$LEDGER"
  exit 0
else
  say "== VERDICT88 FAIL($FAILS) =="
  echo "attempt_epoch=$(date +%s) git=$GIT_REV campaign_sha=judge attempt=$ATT phase=verdict esito=FAIL fails=$FAILS verdict_file=wp88-harness/verdict88.a$ATT.g$G.out" >> "$LEDGER"
  exit 1
fi
