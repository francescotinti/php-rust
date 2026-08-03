#!/bin/bash
# verdict90.sh — giudice della campagna measure90 (S-90.0 p5, Concilio
# WP-91 Sintesi p5). Fail-closed; per-attempt, per-generation (gG mai
# sovrascritte, KS-SK-90-3); judge_sha per generazione (A-AH51) con
# SELF-TETHER (A-BG53: questo giudice rifiuta di giudicare se la sua
# working copy non coincide col blob committato a HEAD — mai piu' un g2
# irrecuperabile, KS-AH-91-2/KG-91-1); righe ledger con reason= e
# supersede_of= (A-BG53/Gregg).
#
# SOGLIE DICHIARATE NEL GIUDICE E NEL HEADER DI CAMPAGNA (KB-91-2):
#   δ (fascia marginali)          = 0,15
#   coverage census (KL-91-3)     = 0,9   (sotto: census ADVISORY)
#   robustezza ratio (KB-91-1)    = 0,8
#   pd discriminante (A-BB63)     = 1000 ms >= finestra
#
# METRICA (A-DL48/KL-91-1): committed_postcollect_win0 e' RIQUALIFICATA
# PEAK-metric (commit==peak_commit su macOS) — b e' la pendenza del PICCO
# di commit. Ogni riga mi_proc di ogni raw viene verificata
# commit==peak_commit: una violazione rende VOID le cifre (KL-91-1).
# ESTRATTORI PER NOME (A-BG54/KG-91-2): ogni selezione win=0/win=9 e'
# keyed a ckpt=, mai posizionale.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
OUT="$REPO/wp78-harness/measure-out"

GIT_REV="$(git -C "$REPO" rev-parse --short HEAD)"
JUDGE_SHA=$(shasum -a 256 "$0" | cut -c1-16)
GITPREFIX="$(git -C "$REPO" rev-parse --show-prefix)"

# --- A-BG53 SELF-TETHER: the judge must BE the committed blob ---------------
JC=$(git -C "$REPO" show "HEAD:${GITPREFIX}wp90-harness/verdict90.sh" 2>/dev/null | shasum -a 256 | cut -c1-16)
if [ "$JUDGE_SHA" != "$JC" ]; then
  echo "REFUSE: verdict90.sh working copy ($JUDGE_SHA) != committed blob at HEAD ($JC) — commit the judge BEFORE judging (A-BG53/KS-AH-91-2)"
  exit 2
fi

# --- .done consumption (KG-88-3) + A-SK54 coherence -------------------------
DONE="$OUT/m90.done"
[ -f "$DONE" ] || { echo "FAIL: m90.done missing — campaign never completed (KG-88-3)"; exit 1; }
ATT=$(sed -n 's/.* attempt=\([0-9]*\) .*/\1/p' "$DONE")
ADONE="$OUT/m90.a$ATT.done"
[ -f "$ADONE" ] || { echo "FAIL: per-attempt done $ADONE missing (A-SK54)"; exit 1; }
cmp -s "$DONE" "$ADONE" || { echo "FAIL: m90.done pointer != m90.a$ATT.done (A-SK54 coherence)"; exit 1; }
D_GIT=$(sed -n 's/.* git=\([0-9a-f]*\) .*/\1/p' "$DONE")
D_MH=$(sed -n 's/.* mem_hash=\([0-9a-f]*\) .*/\1/p' "$DONE")
if [ -z "$D_GIT" ] || [ -z "$ATT" ] || [ -z "$D_MH" ]; then
  echo "FAIL: m90.done lacks git=/attempt=/mem_hash= fields"; exit 1
fi

G=1
while [ -e "$HERE/verdict90.a$ATT.g$G.out" ]; do G=$((G+1)); done
VOUT="$HERE/verdict90.a$ATT.g$G.out"
: > "$VOUT"
say() { echo "$@" | tee -a "$VOUT"; }
FAILS=0
say "== verdict90 — campaign git=$D_GIT attempt=$ATT mem_hash=$D_MH (judge_sha=$JUDGE_SHA COMMITTED-at-HEAD, judged at HEAD=$GIT_REV, generation g$G) =="
say "thresholds (KB-91-2, declared): delta-fascia=0.15 coverage=0.9 robustness=0.8 pd_discriminant=1000"
[ "$G" -gt 1 ] && say "generation g$G SUPERSEDES g1..g$((G-1)) on attempt=$ATT (KS-SK-90-3: docs must cite the MAXIMUM generation)"

# --- A-SK59: matrix tether (judge side) -------------------------------------
MTX_NAME=$(sed -n "s/.* attempt=$ATT phase=identity .* matrix=\([^ ]*\).*/\1/p" "$OUT/m90.campaign.ledger" | tail -1)
if [ -z "$MTX_NAME" ]; then
  say "FAIL: no phase=identity ledger row with matrix= for attempt=$ATT (A-SK59)"; exit 1
fi
MTX_MH=$(git -C "$REPO" show "HEAD:${GITPREFIX}wp78-harness/matrix-archive/$MTX_NAME" 2>/dev/null | tr -d '\0' | sed -n 's/^bin\[mem-census\] sha256\[0:16\]=\([0-9a-f]*\).*/\1/p')
if [ "$MTX_MH" != "$D_MH" ]; then
  say "FAIL: mem_hash=$D_MH != committed matrix bin[mem-census]=$MTX_MH (matrix=$MTX_NAME at HEAD) — judge-side tether (A-SK59/KS-AH-87-1)"; exit 1
fi
say "matrix tether: mem_hash==bin[mem-census] of COMMITTED $MTX_NAME (A-SK59)"
# A-AH57 companion: the consumption must be LEDGERED
NCONS=$(grep -c "attempt=$ATT phase=consume .*esito=LEGAL" "$OUT/m90.campaign.ledger" || true)
if [ "$NCONS" -lt 1 ]; then
  say "FAIL: no phase=consume row for attempt=$ATT in campaign ledger (A-AH57)"; exit 1
fi
say "battery consumption LEDGERED (A-AH57: phase=consume row present)"

nul_free() { perl -0777 -ne 'exit(index($_,"\0")>=0?1:0)' "$1"; }

WS="4 8 12 16"
SLOPE_RAWS=""
for w in $WS; do for r in 1 2 3 4 5; do SLOPE_RAWS="$SLOPE_RAWS m90.slope.w$w.r$r.a$ATT.memcensus"; done; done
CAL_RAWS="m90.cala.r1.a$ATT.memcensus m90.cala.r2.a$ATT.memcensus m90.calb.r1.a$ATT.memcensus m90.calb.r2.a$ATT.memcensus"
SWEEP_RAWS=""
for dt in 1 5 20; do for o in afirst bfirst; do SWEEP_RAWS="$SWEEP_RAWS m90.sweep.dt$dt.$o.pd0.a$ATT.memcensus"; done; done
for dt in 1 5; do for o in afirst bfirst; do SWEEP_RAWS="$SWEEP_RAWS m90.sweep.dt$dt.$o.pd1000.a$ATT.memcensus"; done; done
ALL_RAWS="$SLOPE_RAWS $CAL_RAWS $SWEEP_RAWS"

bf=0; buf=""
emit() { buf="$buf$*
"; }
flushb() { printf '%s' "$buf" | tee -a "$VOUT" >/dev/null; FAILS=$((FAILS+bf)); buf=""; }

# named-by-ckpt commit extractor (A-BG54/KG-91-2): LAST row of that ckpt
commit_at() { # <file> <win> <ckpt>
  awk -v w="win=$2" -v c="ckpt=$3" '
    /tag=mi_proc/ { okw=0; okc=0
      for (i=1;i<=NF;i++) { if ($i==w) okw=1; if ($i==c) okc=1; if ($i ~ /^commit=/) {v=$i; sub("commit=","",v)} }
      if (okw && okc) last=v }
    END { if (last!="") print last }' "$1"
}
count_ckpt() { # <file> <win> <ckpt> -> count of mi_proc rows
  awk -v w="win=$2" -v c="ckpt=$3" '
    /tag=mi_proc/ { okw=0; okc=0
      for (i=1;i<=NF;i++) { if ($i==w) okw=1; if ($i==c) okc=1 }
      if (okw && okc) n++ } END { print n+0 }' "$1"
}

# --- Block VIDENT v2 + A-BG55 -----------------------------------------------
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
  [ "$SEXIT" = 0 ] || { emit "FAIL VIDENT: $raw server_exit=$SEXIT != 0 (A-BG47/KG-89-2)"; bf=$((bf+1)); continue; }
  SPID=$(echo "$ID" | sed -n 's/.* srv_pid=\([0-9]*\) .*/\1/p')
  [ -n "$SPID" ] || { emit "FAIL VIDENT: $raw identity lacks srv_pid= (A-BG47)"; bf=$((bf+1)); continue; }
  # A-BG55: boot epoch dal PROCESSO (boot_src=ps-lstart) e coerente
  BSRC=$(echo "$ID" | sed -n 's/.* boot_src=\([a-z-]*\) .*/\1/p')
  [ "$BSRC" = "ps-lstart" ] || { emit "FAIL VIDENT: $raw boot_src='$BSRC' != ps-lstart — harness wall-clock boot is quasi-vacuous (A-BG55)"; bf=$((bf+1)); continue; }
  BEPOCH=$(echo "$ID" | sed -n 's/.* srv_boot_epoch=\([0-9]*\) .*/\1/p')
  REPOCH=$(echo "$ID" | sed -n 's/.* epoch=\([0-9]*\).*/\1/p')
  if [ -z "$BEPOCH" ] || [ -z "$REPOCH" ] || [ "$BEPOCH" -gt "$REPOCH" ]; then
    emit "FAIL VIDENT: $raw srv_boot_epoch=$BEPOCH incoherent vs epoch=$REPOCH (A-BG51/A-BG55)"; bf=$((bf+1)); continue
  fi
  BADPID=$(awk -v want="$SPID" '/^pid=/ { split($1,a,"="); if (a[2]+0 != want+0) { print; exit } }' "$F")
  [ -z "$BADPID" ] || { emit "FAIL VIDENT: $raw census pid rows not reconciled with srv_pid=$SPID (KG-89-2): $BADPID"; bf=$((bf+1)); continue; }
done
if [ "$bf" = 0 ]; then emit "VIDENT PASS: $(echo $ALL_RAWS | wc -w | tr -d ' ') raws present, NUL-free, identity v2 + boot_src=ps-lstart (A-BG55)"; IDENT_CLEAN=1; fi
flushb

# --- Block VPEAK (A-DL48/KL-91-1) -------------------------------------------
PEAK_CLEAN=0
bf=0
if [ "$IDENT_CLEAN" = 1 ]; then
  for raw in $ALL_RAWS; do
    F="$OUT/$raw"
    BADPK=$(awk '/tag=mi_proc/ { c=""; p=""
        for (i=1;i<=NF;i++) { if ($i ~ /^commit=/) {c=$i; sub("commit=","",c)} if ($i ~ /^peak_commit=/) {p=$i; sub("peak_commit=","",p)} }
        if (c!=p) { print; exit } }' "$F")
    if [ -n "$BADPK" ]; then
      emit "FAIL VPEAK: $raw carries a mi_proc row with commit != peak_commit — the PEAK-metric premise broke, figures VOID (A-DL48/KL-91-1): $BADPK"; bf=$((bf+1))
    fi
  done
  if [ "$bf" = 0 ]; then
    emit "VPEAK PASS: commit==peak_commit on EVERY mi_proc row of every raw — committed_postcollect_win0 is PEAK-metric as declared (A-DL48/KL-91-1)"
    PEAK_CLEAN=1
  fi
else
  emit "VPEAK SKIPPED: identity not clean"; bf=$((bf+1))
fi
flushb

# --- Block VCKPT (A-BG54: named checkpoints, counted per NAME) --------------
CKPT_CLEAN=0
bf=0
if [ "$IDENT_CLEAN" = 1 ]; then
  for raw in $SLOPE_RAWS $CAL_RAWS; do
    F="$OUT/$raw"
    NEM=$(count_ckpt "$F" 0 exit_mi); NEC=$(count_ckpt "$F" 0 exit_collect_mi)
    if [ "$NEM" != 1 ] || [ "$NEC" != 1 ]; then
      emit "FAIL VCKPT: $raw win=0 ckpt counts exit_mi=$NEM exit_collect_mi=$NEC, want 1/1 (A-BG54 named pair)"; bf=$((bf+1))
    fi
  done
  # g2 REQUALIFICATION (S-90.0, bitten by attempt=1 raws — the m89 g1
  # class): PHPR_MI_COLLECT_REQ does NOT produce per-request dumps on the
  # axum-worker channel (the request_collect_mi call site lives on the
  # CLI/metro request path, not in execute_request) — every sweep raw
  # carries exit_collect_mi==1, exactly like slope/cal. The g1 letter
  # (==3) was calibrated on the design assumption, not the channel.
  # DECLARED consequence: the A-DL50 per-request purged ladder is
  # UNAVAILABLE on this channel this campaign — the clamp<=>purge
  # implication is judged ONLY through the P-CLAMP-PD discriminant
  # (pd=1000 removes the clamp), which needs no per-request dump.
  # Wiring request_collect_mi into the worker path = named candidate for
  # the next campaign (WP-92).
  for raw in $SWEEP_RAWS; do
    F="$OUT/$raw"
    NEM=$(count_ckpt "$F" 0 exit_mi); NEC=$(count_ckpt "$F" 0 exit_collect_mi)
    if [ "$NEM" != 1 ] || [ "$NEC" != 1 ]; then
      emit "FAIL VCKPT: $raw win=0 ckpt counts exit_mi=$NEM exit_collect_mi=$NEC, want 1/1 (A-BG54; g2 requalified to the REAL channel form)"; bf=$((bf+1))
    fi
  done
  for raw in $SLOPE_RAWS; do
    F="$OUT/$raw"
    NB=$(count_ckpt "$F" 9 peak_inreq_pboot); NW=$(count_ckpt "$F" 9 peak_inreq_pwork); NP=$(count_ckpt "$F" 9 peak_inreq)
    if [ "$NB" != 1 ] || [ "$NW" != 1 ] || [ "$NP" != 1 ]; then
      emit "FAIL VCKPT: $raw win=9 ladder counts pboot=$NB pwork=$NW peak=$NP, want 1/1/1 (A-DL49 v2)"; bf=$((bf+1))
    fi
  done
  if [ "$bf" = 0 ]; then emit "VCKPT PASS: every win=0/win=9 checkpoint selected and counted BY NAME (A-BG54/KG-91-2)"; CKPT_CLEAN=1; fi
else
  emit "VCKPT SKIPPED: identity not clean"; bf=$((bf+1))
fi
flushb

# --- Block VDISP (A-PP42: exact thr-set; self requests counted) -------------
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
      emit "FAIL VDISP: $raw thr-set {$THRSET} != {$WANTSET} (A-PP42/KS-PP-89-2)"; bf=$((bf+1)); continue
    fi
    case "$raw" in
      m90.slope.*) PER=$(( NREQ / W + 1 )) ;;   # +1: the /__census_self dispatch
      *)           PER=$(( NREQ / W )) ;;
    esac
    BAD=$(echo "$MAP" | awk -F: -v per=$PER '$2+0 != per {print}')
    if [ -n "$BAD" ]; then
      emit "FAIL VDISP: $raw per-thr dispatch counts != $PER (self counted on slope): $BAD"; bf=$((bf+1)); continue
    fi
  done
  if [ "$bf" = 0 ]; then emit "VDISP PASS: thr-set exact and per-thr counts (nreq/W, +1 self on slope) on every raw"; DISP_CLEAN=1; fi
else
  emit "VDISP SKIPPED"; bf=$((bf+1))
fi
flushb

# --- Block VARMS90 (env read-back per raw: purge_delay) ----------------------
ARMS_CLEAN=0
bf=0
if [ "$IDENT_CLEAN" = 1 ]; then
  for raw in $ALL_RAWS; do
    F="$OUT/$raw"
    case "$raw" in
      *pd1000*) WANTPD=1000 ;;
      *)        WANTPD=0 ;;
    esac
    NRB=$(awk -v want="val=$WANTPD" '/tag=env_readback/ && /name=MIMALLOC_PURGE_DELAY/ { for (i=1;i<=NF;i++) if ($i==want) n++ } END { print n+0 }' "$F")
    NOPT=$(awk -v want="val=$WANTPD" '/tag=mi_option/ && /name=purge_delay/ && /ord=15/ { for (i=1;i<=NF;i++) if ($i==want) n++ } END { print n+0 }' "$F")
    if [ "$NRB" -lt 1 ] || [ "$NOPT" -lt 1 ]; then
      emit "FAIL VARMS90: $raw purge_delay read-back env=$NRB opt=$NOPT rows with val=$WANTPD (want >=1 each — A-DL41 class)"; bf=$((bf+1))
    fi
  done
  if [ "$bf" = 0 ]; then emit "VARMS90 PASS: MIMALLOC_PURGE_DELAY read back (env + ord15) with the arm's value on every raw"; ARMS_CLEAN=1; fi
else
  emit "VARMS90 SKIPPED"; bf=$((bf+1))
fi
flushb

# --- Block VORD90 (clamped only in sweep; declared scope) --------------------
ORD_CLEAN=0
bf=0
if [ "$IDENT_CLEAN" = 1 ]; then
  for raw in $SLOPE_RAWS $CAL_RAWS; do
    F="$OUT/$raw"
    NCL=$(grep -c "tag=unitcache_main_entry.*clamped=1" "$F" || true)
    if [ "$NCL" -gt 0 ]; then
      emit "FAIL VORD90: $raw carries $NCL clamped=1 main-entry rows outside the sweep regime (g3 lesson: clamped is a sweep-only physics)"; bf=$((bf+1))
    fi
  done
  # DECLARED narrower scope than verdict89 VORD: the putord invariants are
  # judged by the shared putord machinery in battery (a_ds26/a_ds38); this
  # block pins only the clamped physics placement.
  if [ "$bf" = 0 ]; then emit "VORD90 PASS: clamped=0 on every non-sweep raw (scope DECLARED: putord invariants live in battery a_ds26/a_ds38)"; ORD_CLEAN=1; fi
else
  emit "VORD90 SKIPPED"; bf=$((bf+1))
fi
flushb

# --- Block VLADDER (A-DL49 v2 + braccio 2: Δcommitted per fase) --------------
LADDER_CLEAN=0
B_MAIN=""; B_BOOT=""; B_WORK=""
bf=0
if [ "$PEAK_CLEAN" = 1 ] && [ "$CKPT_CLEAN" = 1 ] && [ "$DISP_CLEAN" = 1 ] && [ "$ARMS_CLEAN" = 1 ] && [ "$ORD_CLEAN" = 1 ]; then
  TMP=$(mktemp)
  for raw in $SLOPE_RAWS; do
    F="$OUT/$raw"
    W=$(grep "^mem_hash=" "$F" | sed -n 's/.* w=\([0-9]*\) .*/\1/p')
    CB=$(commit_at "$F" 9 peak_inreq_pboot)
    CW=$(commit_at "$F" 9 peak_inreq_pwork)
    CP=$(commit_at "$F" 9 peak_inreq)
    CE=$(commit_at "$F" 0 exit_collect_mi)
    if [ -z "$CB" ] || [ -z "$CW" ] || [ -z "$CP" ] || [ -z "$CE" ]; then
      emit "FAIL VLADDER: $raw ladder commit missing (pboot=$CB pwork=$CW peak=$CP exit=$CE)"; bf=$((bf+1)); continue
    fi
    if [ "$CB" -gt "$CW" ] || [ "$CW" -gt "$CP" ] || [ "$CP" -gt "$CE" ]; then
      emit "FAIL VLADDER: $raw ladder NOT monotone (pboot=$CB pwork=$CW peak=$CP exit=$CE) — A-DL48 monotonia violated"; bf=$((bf+1)); continue
    fi
    echo "$W $CB $CW $CE" >> "$TMP"
  done
  if [ "$bf" = 0 ]; then
    REPORT=$(awk '
      function lsq(dom,   n,sx,sy,sxx,sxy,i,w,b,a,sse,r,wbar,sww,seb,out) {
        n=0; sx=0; sy=0; sxx=0; sxy=0
        for (i=1;i<=nws;i++) { w=wsarr[i]; n++; sx+=w; sy+=dom[w]; sxx+=w*w; sxy+=w*dom[w] }
        b=(n*sxy-sx*sy)/(n*sxx-sx*sx); a=(sy-b*sx)/n
        sse=0; for (i=1;i<=nws;i++) { w=wsarr[i]; r=dom[w]-(a+b*w); sse+=r*r }
        wbar=sx/n; sww=0
        for (i=1;i<=nws;i++) { w=wsarr[i]; sww+=(w-wbar)*(w-wbar) }
        seb=sqrt((sse/(n-2))/sww)
        LSQ_B=b; LSQ_A=a; LSQ_SE=seb
      }
      { wv=$1; boot[wv]=boot[wv]" "$2; work[wv]=work[wv]" "($3-$2); fin[wv]=fin[wv]" "$4
        if (minf[wv]==""||$4<minf[wv]) minf[wv]=$4 }
      END {
        nws=split("4 8 12 16", wsarr, " "); R=5
        # mode-census per metric with A-BB61 tie in-band
        for (mi=1; mi<=3; mi++) {
          name = (mi==1?"BOOT":(mi==2?"WORK":"PEAK"))
          for (i=1;i<=nws;i++) {
            w=wsarr[i]
            src = (mi==1?boot[w]:(mi==2?work[w]:fin[w]))
            n=split(src, vv, " "); delete cnt
            for (j=1;j<=n;j++) cnt[vv[j]]++
            best=""; bestn=0; nm=0; tie=0
            for (v in cnt) { nm++
              if (cnt[v]>bestn) { best=v; bestn=cnt[v]; tie=0 }
              else if (cnt[v]==bestn) { tie++; if (v+0<best+0) best=v }
            }
            tierow = (tie>0 ? sprintf(" tie=%d tiebreak=min (A-BB61, declared in-band)", tie) : "")
            advrow = (nm==R ? " [A-BB58: modes==R, point ADVISORY]" : "")
            printf "mode-census %s W=%d: modes=%d dominant=%d B (x%d)%s%s\n", name, w, nm, best, bestn, tierow, advrow
            dom[name"_"w]=best
          }
        }
        # LSQ per metric
        for (mi=1; mi<=3; mi++) {
          name = (mi==1?"BOOT":(mi==2?"WORK":"PEAK"))
          for (i=1;i<=nws;i++) d[wsarr[i]]=dom[name"_"wsarr[i]]
          lsq(d)
          bb[name]=LSQ_B; aa[name]=LSQ_A; se[name]=LSQ_SE
          grade="verdict-grade-candidate"; nout=0; names=""
          for (i=2;i<=nws;i++) {
            dd=d[wsarr[i]]-d[wsarr[i-1]]; m=dd/(wsarr[i]-wsarr[i-1])
            inband = (m >= LSQ_B*0.85 && m <= LSQ_B*1.15) ? "IN" : "OUT"
            if (inband=="OUT") { grade="ADVISORY"; nout++; names=names sprintf(" %.0f(W%d->W%d)", m, wsarr[i-1], wsarr[i]) }
            printf "delta %s W%d->W%d = %d B, marginal %.0f B/worker [%s ex-ante band b*(1+/-0.15)]\n", name, wsarr[i-1], wsarr[i], dd, m, inband
          }
          # A-BG56 judge half: marginals OUT counted AND named
          printf "marginals-OUT %s: count=%d names=[%s ]\n", name, nout, names
          metric = (mi==3 ? "committed_postcollect_win0 (PEAK-metric, A-DL48)" : (mi==1 ? "commit_pboot_win9 (PEAK-metric)" : "delta_commit_pwork_minus_pboot_win9"))
          printf "VSLOPE-%s b (LSQ dominant-mode, metric=%s, W in {4, 8, 12, 16}): %.0f B/worker se=%.0f B twosigma=[%.0f, %.0f] a=%.0f B grade=%s\n", name, metric, LSQ_B, LSQ_SE, LSQ_B-2*LSQ_SE, LSQ_B+2*LSQ_SE, LSQ_A, grade
        }
        # A-BB62 robustness in-band for the MAIN estimator: min-of-R alt
        for (i=1;i<=nws;i++) d[wsarr[i]]=minf[wsarr[i]]
        lsq(d)
        ratio = (bb["PEAK"]!=0 ? LSQ_B/bb["PEAK"] : 0)
        printf "robustness PEAK (A-BB62): b_minR=%.0f B/worker ratio_minR=%.3f [insensitivity vs dominant-mode; threshold 0.8 declared]\n", LSQ_B, ratio
        # additivity of the ladder decomposition
        res = bb["PEAK"]-bb["BOOT"]-bb["WORK"]
        addb = (res >= -0.15*bb["PEAK"] && res <= 0.15*bb["PEAK"]) ? "IN" : "OUT"
        printf "VLADDER additivity: b_peak=%.0f = b_boot=%.0f + b_work=%.0f + residuo_post=%.0f B/worker [residuo %s ex-ante band 0.15*b; the residue is the post-work segment: self+final census allocs]\n", bb["PEAK"], bb["BOOT"], bb["WORK"], res, addb
      }' "$TMP")
    emit "$REPORT"
    B_MAIN=$(echo "$REPORT" | sed -n "s/^VSLOPE-PEAK b (LSQ[^:]*): \([0-9]*\) B\/worker.*/\1/p")
    B_BOOT=$(echo "$REPORT" | sed -n "s/^VSLOPE-BOOT b (LSQ[^:]*): \([0-9]*\) B\/worker.*/\1/p")
    B_WORK=$(echo "$REPORT" | sed -n "s/^VSLOPE-WORK b (LSQ[^:]*): \([0-9]*\) B\/worker.*/\1/p")
    emit "VLADDER PASS: ladder monotone on 20/20 raws; b decomposed by PHASE on the exact monotone counter (braccio 2, Leijen)"
    LADDER_CLEAN=1
  fi
  rm -f "$TMP"
else
  emit "VLADDER SKIPPED: upstream blocks not clean (KS-SK-88-2)"; bf=$((bf+1))
fi
flushb

# --- Block VSELF (A-DL49 v2 + A-DL31 collision honesty) ----------------------
bf=0
if [ "$IDENT_CLEAN" = 1 ] && [ "$CKPT_CLEAN" = 1 ]; then
  COLLIDED=0; SPLIT_OK=0; NSLOPE=0
  for raw in $SLOPE_RAWS; do
    F="$OUT/$raw"
    NSLOPE=$((NSLOPE+1))
    W=$(grep "^mem_hash=" "$F" | sed -n 's/.* w=\([0-9]*\) .*/\1/p')
    THRS=$(awk '/tag=peak_self/ { for (i=1;i<=NF;i++) if ($i ~ /^thr=/) { t=$i; sub("thr=","",t); print t } }' "$F" | sort -n | tr '\n' ',' | sed 's/,$//')
    WANTT=$(seq 0 $((W-1)) | tr '\n' ',' | sed 's/,$//')
    if [ "$THRS" != "$WANTT" ]; then
      emit "FAIL VSELF: $raw self-census thr set {$THRS} != {$WANTT} — round-robin coverage broken"; bf=$((bf+1)); continue
    fi
    # the self rows follow the pwork ladder rows: take the LAST W thr_sum
    # rows (the teardown probe emits its own thr_sum later at exit — those
    # carry the SAME ckpt-less form; select the ones between peak_self tags)
    NH=$(awk '/tag=peak_self/ {inwin=1} inwin && /tag=mi_bin_thr_sum/ { for (i=1;i<=NF;i++) if ($i ~ /^heap=/) print $i }' "$F" | sort -u | wc -l | tr -d ' ')
    if [ "$NH" -lt "$W" ]; then
      COLLIDED=$((COLLIDED+1))
    else
      SPLIT_OK=$((SPLIT_OK+1))
    fi
  done
  if [ "$bf" = 0 ]; then
    if [ "$COLLIDED" -gt 0 ]; then
      emit "VSELF: per-worker split REFUSED on $COLLIDED/$NSLOPE slope raws — mi_heap_of returns a SHARED heap on mimalloc v3 (A-DL31 honesty tooth: same heap pointer across workers => the per-thread reading reads ONE heap W times). DECLARED ex-ante in the campaign header; the per-worker decomposition of b stays OPEN for the council (channel structural limit, not a campaign FAIL)"
    fi
    [ "$SPLIT_OK" -gt 0 ] && emit "VSELF: per-worker split VALID on $SPLIT_OK/$NSLOPE raws (distinct heap pointers)"
    emit "VSELF PASS: thr coverage exact on every slope raw; heap-collision judged per A-DL31 (refused, never blessed)"
  fi
else
  emit "VSELF SKIPPED"; bf=$((bf+1))
fi
flushb

# --- Block VCOV (KL-91-3: census coverage at the peak) -----------------------
bf=0
if [ "$LADDER_CLEAN" = 1 ]; then
  COVREP=$(for raw in $SLOPE_RAWS; do
    F="$OUT/$raw"
    CW=$(commit_at "$F" 9 peak_inreq_pwork)
    VIS=$(awk '/tag=mi_theap_pages/ && /win=9/ && /ckpt=peak_inreq_pwork/ && /heap=/ { for (i=1;i<=NF;i++) if ($i ~ /^committed=/) {v=$i; sub("committed=","",v); s+=v} } END { print s+0 }' "$F")
    echo "$raw $VIS $CW"
  done | awk '{ vis+=$2; tot+=$3; n++ } END { printf "%d %d %.3f", vis/n, tot/n, (tot>0? vis/tot : 0) }')
  MVIS=$(echo "$COVREP" | cut -d' ' -f1); MTOT=$(echo "$COVREP" | cut -d' ' -f2); MRAT=$(echo "$COVREP" | cut -d' ' -f3)
  BELOW=$(awk -v r="$MRAT" 'BEGIN { print (r < 0.9) ? 1 : 0 }')
  if [ "$BELOW" = 1 ]; then
    emit "VCOV: census at peak covers mean $MVIS of $MTOT B commit (ratio=$MRAT < 0.9) => the census half of the attribution is ADVISORY (KL-91-3, DECLARED — never a mute census blessed; the exact half is the VLADDER commit deltas)"
  else
    emit "VCOV: census at peak covers ratio=$MRAT >= 0.9 — census half verdict-grade (KL-91-3)"
  fi
  emit "VCOV PASS: coverage computed and declared against the 0.9 threshold (KL-91-3)"
else
  emit "VCOV SKIPPED: ladder not clean"; bf=$((bf+1))
fi
flushb

# --- Block VSWEEP90 (cal + spans + clamped + A-BB63/A-DL50) ------------------
bf=0
if [ "$IDENT_CLEAN" = 1 ] && [ "$DISP_CLEAN" = 1 ] && [ "$ARMS_CLEAN" = 1 ]; then
  getnet() { awk -v pat="$2" '/tag=unitcache_main_entry/ && $0 ~ pat { for (i=1;i<=NF;i++) { if ($i ~ /^net=/) {n=$i; sub("net=","",n)} if ($i ~ /^floor_inc=/) {f=$i; sub("floor_inc=","",f)} } print n" "f }' "$1"; }
  CA1=$(getnet "$OUT/m90.cala.r1.a$ATT.memcensus" "pad87a[.]php")
  CA2=$(getnet "$OUT/m90.cala.r2.a$ATT.memcensus" "pad87a[.]php")
  CB1=$(getnet "$OUT/m90.calb.r1.a$ATT.memcensus" "pad87b[.]php")
  CB2=$(getnet "$OUT/m90.calb.r2.a$ATT.memcensus" "pad87b[.]php")
  if [ -z "$CA1" ] || [ -z "$CB1" ]; then emit "FAIL VSWEEP90: calibration rows missing"; bf=$((bf+1)); fi
  [ "$CA1" = "$CA2" ] || { emit "FAIL VSWEEP90: cala r1 ($CA1) != r2 ($CA2) — not byte-reproduced"; bf=$((bf+1)); }
  [ "$CB1" = "$CB2" ] || { emit "FAIL VSWEEP90: calb r1 ($CB1) != r2 ($CB2) — not byte-reproduced"; bf=$((bf+1)); }
  if [ "$bf" = 0 ]; then
    CALA_NET=${CA1%% *}; CALB_NET=${CB1%% *}
    SUM=$((CALA_NET + CALB_NET))
    emit "VSWEEP90 cal: pad87a net=$CALA_NET B | pad87b net=$CALB_NET B [byte-reproduced r1==r2 per side; the m89 anchor 7801102 is a NAMED-ADVISORY comparison only — the census binary CHANGED (A-MS50/ckpt/self)]"
    [ "$CALA_NET" = 7801102 ] && emit "VSWEEP90 cal note: pad87a net EQUALS the m89 anchor (5th campaign byte-repro)" || emit "VSWEEP90 cal note: pad87a net=$CALA_NET differs from the m89 anchor 7801102 — instrument-change NAMED-DEVIATION (declared ex-ante)"
    DT20OK=0; DT20N=0; PDBIG_CL=0; PDBIG_N=0; PD0_CL_BADPURGE=0; PD0_CL=0
    for raw in $SWEEP_RAWS; do
      F="$OUT/$raw"
      dt=$(echo "$raw" | sed -n 's/.*sweep\.dt\([0-9]*\)\..*/\1/p')
      o=$(echo "$raw" | sed -n 's/.*\.\(afirst\|bfirst\)\..*/\1/p')
      pd=$(echo "$raw" | sed -n 's/.*\.pd\([0-9]*\)\.a.*/\1/p')
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
      if [ "$OV" = "INVALID" ]; then
        emit "FAIL VSWEEP90: dt$dt.$o.pd$pd spans=INVALID (A-SK58)"; bf=$((bf+1)); continue
      fi
      if [ "$dt" = 20 ] && [ "$OV" != "NO-OVERLAP" ]; then
        emit "FAIL VSWEEP90: dt20.$o spans=$OV, requires NO-OVERLAP"; bf=$((bf+1)); continue
      fi
      NA=$(getnet "$F" "pad87a[.]php"); NB=$(getnet "$F" "pad87b[.]php")
      [ -n "$NA" ] && [ -n "$NB" ] || { emit "FAIL VSWEEP90: dt$dt.$o.pd$pd pad entries missing"; bf=$((bf+1)); continue; }
      NA_NET=${NA%% *}; NB_NET=${NB%% *}
      NCL=$(grep -c "tag=unitcache_main_entry.*clamped=1" "$F" || true)
      # g2: the A-DL50 per-request purged ladder is UNAVAILABLE on this
      # channel (no per-request dumps on the axum-worker path — see the
      # VCKPT requalification above). The exit dump still carries the
      # CUMULATIVE purged of the whole run, reported for the record; the
      # clamp<=>purge implication is judged through P-CLAMP-PD only.
      PURGED_EXIT=$(awk '/tag=mi_arena/ && /win=0/ && /ckpt=exit_collect_mi/ && /key=purged/ { for (i=1;i<=NF;i++) if ($i ~ /^val=/) {v=$i; sub("val=","",v); last=v} } END { print last }' "$F")
      if [ "$pd" = 1000 ]; then
        PDBIG_N=$((PDBIG_N+1))
        [ "$NCL" -gt 0 ] && PDBIG_CL=$((PDBIG_CL+1))
        emit "VSWEEP90 dt$dt $o pd1000: spans=$OV padA net=$NA_NET B | padB net=$NB_NET B | clamped_rows=$NCL purged_cumulative_exit=$PURGED_EXIT [A-BB63 arm; A-DL50 per-request ladder DECLARED-unavailable on this channel; nets VOID per-thread KB-88-1]"
      else
        if [ "$NCL" -gt 0 ]; then
          PD0_CL=$((PD0_CL+1))
          emit "VSWEEP90 dt$dt $o pd0: spans=$OV CLAMPED($NCL) purged_cumulative_exit=$PURGED_EXIT [declared, excluded from surplus tally; A-DL50 per-request ladder DECLARED-unavailable — clamp<=>purge judged via P-CLAMP-PD]"
        else
          DA=$((NA_NET - SUM)); DB=$((NB_NET - SUM))
          emit "VSWEEP90 dt$dt $o pd0: spans=$OV padA net=$NA_NET B dA=$DA B | padB net=$NB_NET B dB=$DB B [nets VOID per-thread, KB-88-1]"
          if [ "$dt" = 20 ]; then
            DT20N=$((DT20N+2))
            [ "$NA_NET" = "$CALA_NET" ] && DT20OK=$((DT20OK+1))
            [ "$NB_NET" = "$CALB_NET" ] && DT20OK=$((DT20OK+1))
          fi
        fi
      fi
    done
    if [ "$bf" = 0 ]; then
      if [ "$DT20N" -gt 0 ] && [ "$DT20OK" = "$DT20N" ]; then
        emit "VSWEEP90 P-DT20 CONFIRMED: net==cal AL BYTE on $DT20OK/$DT20N dt=20 sides"
      elif [ "$DT20N" -gt 0 ]; then
        emit "VSWEEP90 P-DT20 REFUTED: net==cal only $DT20OK/$DT20N at dt=20 (label, not gate)"
      fi
      if [ "$PDBIG_CL" = 0 ] && [ "$PDBIG_N" -gt 0 ]; then
        emit "VSWEEP90 P-CLAMP-PD CONFIRMED: purge_delay=1000 >= finestra elimina il clamp su $PDBIG_N/$PDBIG_N run dt{1,5} — la deflazione dei dt intermedi E' il decommit da purge-al-free dentro la finestra dell'altro lato (A-BB63+A-DL50, Bak Q3+Leijen Q4)"
      elif [ "$PDBIG_N" -gt 0 ]; then
        emit "VSWEEP90 P-CLAMP-PD REFUTED: clamp persiste su $PDBIG_CL/$PDBIG_N run con pd=1000 — la colpa e' del bracket/counter, non della purge (esito ex-ante alternativo dichiarato)"
      fi
      [ "$PD0_CL" -gt 0 ] && emit "VSWEEP90 A-DL50: $PD0_CL raw clamped su pd0 — la scala Δpurged per-request e' DECLARED-unavailable su questo canale (wiring worker-path = candidato WP-92); l'implicazione clamp<=>purge e' giudicata dal solo P-CLAMP-PD"
      emit "VSWEEP90 PASS: cal byte-reproduced, spans judged, clamped physics judged against the ex-ante predictions"
    fi
  fi
else
  emit "VSWEEP90 SKIPPED: upstream not clean"; bf=$((bf+1))
fi
flushb

# --- Verdict + ledger row (A-BG53: reason= and supersede_of=) ----------------
SUP="none"; [ "$G" -gt 1 ] && SUP="g$((G-1))"
if [ "$FAILS" = 0 ]; then
  say "== VERDICT90 PASS (attempt=$ATT, generation g$G, judge committed at HEAD) =="
  echo "attempt_epoch=$(date +%s) git=$GIT_REV attempt=$ATT phase=verdict generation=g$G judge_sha=$JUDGE_SHA esito=PASS fails=0 verdict_file=verdict90.a$ATT.g$G.out reason=all-blocks-clean supersede_of=$SUP" >> "$OUT/m90.campaign.ledger"
  exit 0
else
  say "== VERDICT90 FAIL($FAILS) (attempt=$ATT, generation g$G) =="
  echo "attempt_epoch=$(date +%s) git=$GIT_REV attempt=$ATT phase=verdict generation=g$G judge_sha=$JUDGE_SHA esito=FAIL fails=$FAILS verdict_file=verdict90.a$ATT.g$G.out reason=see-verdict-file supersede_of=$SUP" >> "$OUT/m90.campaign.ledger"
  exit 1
fi
