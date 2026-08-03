#!/bin/bash
# repair90-estimators.sh — S-91.0 p1/p5 (Concilio WP-92, team-misura P2):
# RIPARAZIONE DEGLI STIMATORI SUI 20 RAW SLOPE + 10 SWEEP GIA' COMMITTATI
# della campagna m90 (zero run nuove). Consuma:
#   A-BB64  bandire tiebreak=min con modes==R: punto per-W = MEDIANA
#   A-BB65  additivita' within-run (dove e' esatta) + residuo cross-lane
#           rietichettato "selection mismatch" (label census/self RITIRATA)
#   A-BB66  robustezza non tautologica: ratio min/mediana/media (VOID se
#           lo stimatore di controllo coincide col primario per costruzione)
#   A-BB67  flag run-outlier per NOME (TUTTI i contatori oltre 2*MAD)
#   A-BB68  CI onesti: banda dichiarata +/-2se, MAI "2sigma=>95%" con df=2;
#           fattori t stampati (t(0.975;2)=4.303, t(0.975;18)=2.101)
#   A-BG57  VCOV tabella per-W + coverage MARGINALE (pendenza di vis su W)
#   A-BG59  estrattore commit_at FIXED: reset per riga; riga matching senza
#           commit= = FAIL (mai ereditare dalla riga precedente)
#   A-BG60  conteggio ADVISORY per NOME (modes==R) su TUTTE le lane
#   A-BG61  legge clamp<=>overlap dichiarata a macchina sui 10 sweep
# Discipline: raw letti da HEAD (A-SK55 — mai working tree), NUL strippati
# (tr -d '\0'), estrattori per NOME di campo (mai substring — nota Gregg Q4).
# Soglie ex-ante (header, KB-91-2): fascia marginali +/-15% della slope;
# robustezza threshold 0.8; MAD factor 2; R=5; W in {4,8,12,16}.
# TETHER: prima di riparare, l'estrattore nuovo DEVE riprodurre al byte le
# cifre del g2 committato (b_peak 19723059 / b_boot 2252800 / b_work
# 17276928 / residuo 193331) — se non riproduce, FAIL (nessuna riparazione
# da un estrattore non tetherato).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(git -C "$HERE" rev-parse --show-toplevel)" || { echo "FAIL repair90: not a git repo"; exit 1; }
HEADREV=$(git -C "$ROOT" rev-parse HEAD)
SCRIPT_SHA=$(shasum -a 256 "$0" | cut -c1-16)
MOUT_REL="php-rust/wp78-harness/measure-out"
echo "repair90-estimators identity: head=$HEADREV script_sha=$SCRIPT_SHA"
echo "thresholds ex-ante: marginal band=slope*(1+/-0.15) robustness=0.8 mad_factor=2 t975_df2=4.303 t975_df18=2.101 R=5 W={4,8,12,16}"

fail=0
show_raw() { # <basename> -> content from HEAD, NUL-stripped; FAIL if uncommitted
  local rel="$MOUT_REL/$1"
  if ! git -C "$ROOT" cat-file -e "HEAD:$rel" 2>/dev/null; then
    echo "FAIL repair90: raw NOT committed at HEAD: $rel (A-SK55)"; return 1
  fi
  git -C "$ROOT" show "HEAD:$rel" | tr -d '\0'
}

# A-BG59 FIXED extractor: v reset per row; matching row without commit= = FAIL
commit_at_fixed() { # <basename> <win> <ckpt>
  show_raw "$1" | awk -v w="win=$2" -v c="ckpt=$3" '
    /tag=mi_proc/ { okw=0; okc=0; v=""
      for (i=1;i<=NF;i++) { if ($i==w) okw=1; if ($i==c) okc=1
        if ($i ~ /^commit=/) {v=$i; sub("commit=","",v)} }
      if (okw && okc) { if (v=="") { print "MISSINGCOMMIT"; exit }; last=v } }
    END { if (last!="") print last }'
}

TMP=$(mktemp -d /tmp/repair90.XXXXXX)
trap 'rm -rf "$TMP"' EXIT

# ---- ladder extraction: 20 slope raws --------------------------------------
: > "$TMP/ladder"
for w in 4 8 12 16; do for r in 1 2 3 4 5; do
  raw="m90.slope.w$w.r$r.a1.memcensus"
  CB=$(commit_at_fixed "$raw" 9 peak_inreq_pboot) || { fail=1; continue; }
  CW=$(commit_at_fixed "$raw" 9 peak_inreq_pwork)
  CE=$(commit_at_fixed "$raw" 0 exit_collect_mi)
  for v in "$CB" "$CW" "$CE"; do
    case "$v" in ''|MISSINGCOMMIT)
      echo "FAIL repair90: $raw ladder commit missing/uncommitted-field (A-BG59)"; fail=1;;
    esac
  done
  [ "$fail" = 1 ] && continue
  # VCOV vis: exact-field match (never substring — Gregg Q4), heap= required
  VIS=$(show_raw "$raw" | awk '
    /tag=mi_theap_pages/ { okw=0; okc=0; okh=0; s2=0
      for (i=1;i<=NF;i++) { if ($i=="win=9") okw=1; if ($i=="ckpt=peak_inreq_pwork") okc=1
        if ($i ~ /^heap=/) okh=1
        if ($i ~ /^committed=/) {v=$i; sub("committed=","",v); s2=v} }
      if (okw && okc && okh) s+=s2 }
    END { print s+0 }')
  echo "$w $r $CB $CW $CE $VIS" >> "$TMP/ladder"
done; done
[ "$fail" = 1 ] && { echo "REPAIR90 FAIL: ladder extraction dirty"; exit 1; }
NL=$(wc -l < "$TMP/ladder" | tr -d ' ')
echo "ladder raws extracted: $NL/20 (from HEAD)"
[ "$NL" = 20 ] || { echo "REPAIR90 FAIL: expected 20 ladder raws"; exit 1; }
while read -r w r cb cw ce vis; do
  echo "tag=ladder w=$w r=$r pboot=$cb pwork=$cw exit=$ce dwork=$((cw-cb)) dpost=$((ce-cw)) vis=$vis"
done < "$TMP/ladder"

# ---- estimator repair (single awk pass over the 20-point table) ------------
awk '
  function med(v, n,   i,j,t,a) { for(i=1;i<=n;i++)a[i]=v[i]
    for(i=1;i<n;i++)for(j=i+1;j<=n;j++)if(a[j]<a[i]){t=a[i];a[i]=a[j];a[j]=t}
    return (n%2)? a[(n+1)/2] : (a[n/2]+a[n/2+1])/2 }
  function lsq(x, y, n,   i,sx,sy,sxx,sxy,b,a,sse,rr,wbar,sww,seb) {
    sx=0;sy=0;sxx=0;sxy=0
    for(i=1;i<=n;i++){sx+=x[i];sy+=y[i];sxx+=x[i]*x[i];sxy+=x[i]*y[i]}
    b=(n*sxy-sx*sy)/(n*sxx-sx*sx); a=(sy-b*sx)/n
    sse=0; for(i=1;i<=n;i++){rr=y[i]-(a+b*x[i]); sse+=rr*rr}
    wbar=sx/n; sww=0; for(i=1;i<=n;i++) sww+=(x[i]-wbar)*(x[i]-wbar)
    seb=(n>2)? sqrt((sse/(n-2))/sww) : 0
    LB=b; LA=a; LSE=seb }
  function abs(z){return z<0?-z:z}
  { w=$1; r=$2; nb[w]++
    boot[w"_"nb[w]]=$3; work[w"_"nb[w]]=$4-$3; peak[w"_"nb[w]]=$5
    dpost[w"_"nb[w]]=$5-$4; runname[w"_"nb[w]]="w"w".r"r
    pb[w"_"nb[w]]=$3; pw[w"_"nb[w]]=$4; pe[w"_"nb[w]]=$5; vis[w"_"nb[w]]=$6
    N++; PX[N]=w; PY[N]=$5; VX[N]=w; VY[N]=$6; VR[N]=$6/$4; VW[N]=w
    RN[N]="w"w".r"r; DP[N]=$5-$4; TW[N]=$4 }
  END {
    nws=split("4 8 12 16", ws, " "); R=5
    # --- per-lane per-W census: mode(min-tiebreak) / median / mean / min ---
    adv_total=0; adv_names=""
    for (li=1; li<=3; li++) {
      lane=(li==1?"BOOT":(li==2?"WORK":"PEAK"))
      for (i=1;i<=nws;i++) {
        w=ws[i]
        for (j=1;j<=R;j++) v[j]=(li==1?boot[w"_"j]:(li==2?work[w"_"j]:peak[w"_"j]))
        delete cnt; for (j=1;j<=R;j++) cnt[v[j]]++
        best=""; bestn=0; nm=0
        for (u in cnt) { nm++
          if (cnt[u]>bestn) {best=u; bestn=cnt[u]}
          else if (cnt[u]==bestn && u+0<best+0) best=u }
        mn=v[1]; for(j=2;j<=R;j++) if(v[j]<mn) mn=v[j]
        s=0; for(j=1;j<=R;j++) s+=v[j]
        m=med(v,R)
        printf "census %s W=%d: modes=%d mode_min=%d median=%.0f mean=%.0f min=%d%s\n", lane, w, nm, best, m, s/R, mn, (nm==R? " [modes==R: point ADVISORY per NOME — A-BG60]":"")
        if (nm==R) { adv_total++; adv_names=adv_names" "lane"-W"w }
        MODE[lane"_"w]=best; MED[lane"_"w]=m; MEAN[lane"_"w]=s/R; MIN[lane"_"w]=mn
      }
    }
    printf "ADVISORY points per NOME (A-BG60): count=%d names=[%s ]\n", adv_total, adv_names
    # --- OLS per lane x estimator (4 per-W points, df=2) -------------------
    for (li=1; li<=3; li++) {
      lane=(li==1?"BOOT":(li==2?"WORK":"PEAK"))
      for (ei=1; ei<=3; ei++) {
        est=(ei==1?"mode_min":(ei==2?"median":"mean"))
        for (i=1;i<=nws;i++) { X[i]=ws[i]
          Y[i]=(ei==1?MODE[lane"_"ws[i]]:(ei==2?MED[lane"_"ws[i]]:MEAN[lane"_"ws[i]])) }
        lsq(X, Y, nws)
        printf "slope %s est=%s: b=%.1f B/worker se=%.1f a=%.1f band_2se=[%.1f, %.1f] band_t2=[%.1f, %.1f] [df=2: 2se~82%%, t=4.303 for 95%% — A-BB68]\n", lane, est, LB, LSE, LA, LB-2*LSE, LB+2*LSE, LB-4.303*LSE, LB+4.303*LSE
        SL[lane"_"est]=LB; SE[lane"_"est]=LSE
        nout=0; names=""
        for (i=2;i<=nws;i++) {
          dd=Y[i]-Y[i-1]; mg=dd/(ws[i]-ws[i-1])
          inb=(mg>=LB*0.85 && mg<=LB*1.15)?"IN":"OUT"
          if (inb=="OUT"){nout++; names=names sprintf(" %.0f(W%d->W%d)",mg,ws[i-1],ws[i])}
          printf "marginal %s est=%s W%d->W%d = %.0f B/worker [%s]\n", lane, est, ws[i-1], ws[i], mg, inb
        }
        printf "marginals-OUT %s est=%s: count=%d names=[%s ]\n", lane, est, nout, names
      }
    }
    # --- tether: mode_min slopes must reproduce the committed g2 -----------
    tfail=0
    if (abs(SL["PEAK_mode_min"]-19723059.2)>1) tfail=1
    if (abs(SL["BOOT_mode_min"]-2252800)>1) tfail=1
    if (abs(SL["WORK_mode_min"]-17276928)>1) tfail=1
    res_repro=SL["PEAK_mode_min"]-SL["BOOT_mode_min"]-SL["WORK_mode_min"]
    if (abs(res_repro-193331.2)>1) tfail=1
    printf "tether g2 (byte-reproduction of the committed verdict): b_peak=%.1f b_boot=%.1f b_work=%.1f residuo=%.1f => %s\n", SL["PEAK_mode_min"], SL["BOOT_mode_min"], SL["WORK_mode_min"], res_repro, (tfail? "FAIL":"PASS")
    if (tfail) { print "REPAIR90 FAIL: new extractor does NOT reproduce the committed g2 — no repair from an untethered extractor"; exit 1 }
    # --- pooled per-run OLS (n=20, df=18) for each lane --------------------
    for (li=1; li<=3; li++) {
      lane=(li==1?"BOOT":(li==2?"WORK":"PEAK")); k=0
      for (i=1;i<=nws;i++) for (j=1;j<=R;j++) { k++
        X[k]=ws[i]; Y[k]=(li==1?boot[ws[i]"_"j]:(li==2?work[ws[i]"_"j]:peak[ws[i]"_"j])) }
      lsq(X, Y, k)
      printf "slope %s est=pooled_per_run(n=20,df=18): b=%.1f B/worker se=%.1f a=%.1f band_2se=[%.1f, %.1f] band_t=[%.1f, %.1f] [t(0.975;18)=2.101 — A-BB68]\n", lane, LB, LSE, LA, LB-2*LSE, LB+2*LSE, LB-2.101*LSE, LB+2.101*LSE
      SL[lane"_pooled"]=LB
    }
    # --- A-BB66 robustness (non-tautological) ------------------------------
    taut=1
    for (i=1;i<=nws;i++) if (MIN["PEAK_"ws[i]]!=MED["PEAK_"ws[i]]) taut=0
    printf "robustness PEAK (A-BB66): min/median=%.3f mean/median=%.3f mode_min/median=%.3f [threshold 0.8; tautology=%s]\n", SL["PEAK_mode_min"]/SL["PEAK_median"], SL["PEAK_mean"]/SL["PEAK_median"], SL["PEAK_mode_min"]/SL["PEAK_median"], (taut? "VOID — control==primary by construction (KS-BB-92-2)":"no — control can fail")
    # --- headline figures (A-BB64): PEAK republished as MEDIAN -------------
    printf "REPAIRED b_peak (A-BB64, est=median): %.0f B/worker se=%.0f band_2se=[%.0f, %.0f]\n", SL["PEAK_median"], SE["PEAK_median"], SL["PEAK_median"]-2*SE["PEAK_median"], SL["PEAK_median"]+2*SE["PEAK_median"]
    printf "headline sum b_boot+b_work (est=mode_min, grade=ADVISORY inherited): %.0f B/worker\n", SL["BOOT_mode_min"]+SL["WORK_mode_min"]
    printf "continuity vs b_m89: %.0f vs 19575603 delta=%.0f B [suggestive, NOT verdict-grade — two ADVISORY figures]\n", SL["BOOT_mode_min"]+SL["WORK_mode_min"], SL["BOOT_mode_min"]+SL["WORK_mode_min"]-19575603
    # --- A-BB65: within-run additivity + cross-lane residual ---------------
    exact=1
    for (i=1;i<=nws;i++) for (j=1;j<=R;j++) {
      w=ws[i]; idn=w"_"j
      if (pb[idn]+work[idn]+dpost[idn] != pe[idn]) { exact=0
        printf "within-run additivity BROKEN at %s\n", runname[idn] } }
    printf "within-run additivity (A-BB65): pboot+dwork+dpost==exit on %s [arithmetic identity, exact by construction]\n", (exact? "20/20":"NOT ALL")
    xl=""; for (i=1;i<=nws;i++) { w=ws[i]
      resid=MODE["PEAK_"w]-MODE["BOOT_"w]-MODE["WORK_"w]
      xl=xl sprintf(" W%d=%d", w, resid) }
    printf "cross-lane residual per-W (est=mode_min) [SELECTION MISMATCH, not physics — A-BB65; census/self label RITIRATA]:%s\n", xl
    xm=""; for (i=1;i<=nws;i++) { w=ws[i]
      residm=MED["PEAK_"w]-MED["BOOT_"w]-MED["WORK_"w]
      xm=xm sprintf(" W%d=%.0f", w, residm) }
    printf "cross-lane residual per-W (est=median):%s\n", xm
    # --- dpost census: the census/self label refuted -----------------------
    nz=0; nzn=""
    for (k=1;k<=N;k++) if (DP[k]!=0) { nz++; nzn=nzn sprintf(" %s=+%d", RN[k], DP[k]) }
    printf "dpost (exit_collect - pwork) census: zero on %d/20 raws, nonzero:[%s ] [self+census commit cost is ZERO in the typical case — label refuted]\n", 20-nz, nzn
    # --- A-BB67 run-outlier MAD flag ---------------------------------------
    for (i=1;i<=nws;i++) { w=ws[i]
      for (j=1;j<=R;j++) {
        allout=1; ctrs=0
        for (ci=1; ci<=3; ci++) {
          for (jj=1;jj<=R;jj++) s5[jj]=(ci==1?pb[w"_"jj]:(ci==2?pw[w"_"jj]:pe[w"_"jj]))
          mm=med(s5,R)
          for (jj=1;jj<=R;jj++) ad[jj]=abs(s5[jj]-mm)
          madv=med(ad,R)
          x=(ci==1?pb[w"_"j]:(ci==2?pw[w"_"j]:pe[w"_"j]))
          if (madv==0 || abs(x-mm) <= 2*madv) allout=0
          else ctrs++
        }
        if (allout) printf "run-outlier (A-BB67): %s — ALL %d counters beyond 2*MAD from sisters [flagged per NOME; can never become its W point via min]\n", runname[w"_"j], ctrs
      }
    }
    # --- A-BG57 VCOV per-W table + marginal coverage -----------------------
    for (i=1;i<=nws;i++) { w=ws[i]
      rmin=2; rmax=-1; rs=0
      for (j=1;j<=R;j++) { rt=vis[w"_"j]/pw[w"_"j]; rs+=rt
        if (rt<rmin) rmin=rt; if (rt>rmax) rmax=rt }
      printf "VCOV per-W (A-BG57) W=%d: ratio_min=%.3f ratio_max=%.3f ratio_mean=%.3f\n", w, rmin, rmax, rs/R
    }
    sv=0; st=0; for (k=1;k<=N;k++) { sv+=VY[k]; st+=pw[VW[k]"_"((k-1)%R+1)] }
    # pooled ratio reproduced the way the g2 did it: mean(vis)/mean(tot)
    tots=0; viss=0
    for (i=1;i<=nws;i++) for (j=1;j<=R;j++) { tots+=pw[ws[i]"_"j]; viss+=vis[ws[i]"_"j] }
    printf "VCOV pooled (as g2): mean_vis=%.0f mean_tot=%.0f ratio=%.3f [exists in NO raw — per-W table is the citable form, KS-BG-92-1]\n", viss/20, tots/20, (viss/20)/(tots/20)
    lsq(VX, VY, N)
    bv=LB; av=LA; sev=LSE
    lsq(VX, TW, N)
    printf "VCOV marginal coverage (A-BG57): slope_vis=%.0f B/worker se=%.0f a_vis=%.0f => marginal_ratio vs b_peak(median)=%.3f\n", bv, sev, av, bv/SL["PEAK_median"]
    printf "VCOV invisible mass (A-BG57): slope_invisible=%.0f B/worker intercept_invisible=%.0f B [pooled OLS of pwork-commit minus vis: the invisible mass is intercept-dominated]\n", LB-bv, LA-av
  }
' "$TMP/ladder" || { echo "REPAIR90 FAIL (estimator pass)"; exit 1; }

# ---- A-BG61: clamp<=>overlap law on the 10 committed sweep raws ------------
law_ok=1; nlaw=0
for spec in dt1.afirst.pd0 dt1.bfirst.pd0 dt5.afirst.pd0 dt5.bfirst.pd0 \
            dt20.afirst.pd0 dt20.bfirst.pd0 dt1.afirst.pd1000 dt1.bfirst.pd1000 \
            dt5.afirst.pd1000 dt5.bfirst.pd1000; do
  raw="m90.sweep.$spec.a1.memcensus"
  read -r OV NCL <<EOF
$(show_raw "$raw" | awk '
    /tag=lower_span/ { n++; for(i=1;i<=NF;i++){ if($i ~ /^t0_us=/){v=$i;sub("t0_us=","",v);t0[n]=v+0}
      if($i ~ /^t1_us=/){v=$i;sub("t1_us=","",v);t1[n]=v+0} } }
    /tag=unitcache_main_entry/ { for(i=1;i<=NF;i++) if($i=="clamped=1") ncl++ }
    END { ov=(n==2 && t0[1]<t1[2] && t0[2]<t1[1]) ? "OVERLAP":"NO-OVERLAP"
      if (n!=2) ov="INVALID"
      print ov, ncl+0 }')
EOF
  [ "$OV" = "INVALID" ] && { echo "FAIL repair90: $raw spans INVALID"; law_ok=0; continue; }
  agree="no"
  { [ "$OV" = "OVERLAP" ] && [ "$NCL" -gt 0 ]; } && agree="yes"
  { [ "$OV" = "NO-OVERLAP" ] && [ "$NCL" = 0 ]; } && agree="yes"
  [ "$agree" = "yes" ] && nlaw=$((nlaw+1)) || law_ok=0
  echo "sweep $spec: spans=$OV clamped_rows=$NCL law_clamp_iff_overlap=$agree"
done
echo "clamp<=>overlap law (A-BG61): $nlaw/10 agree [negative NOT controlled: no pd1000+forced-overlap run exists — A-DL-54 is the controlled negative]"
[ "$law_ok" = 1 ] || { echo "REPAIR90 FAIL: clamp<=>overlap law violated on some raw"; exit 1; }

echo "REPAIR90 PASS: estimators repaired on the 30 committed raws (20 slope + 10 sweep), zero new runs (Concilio WP-92 team-misura P2)"
