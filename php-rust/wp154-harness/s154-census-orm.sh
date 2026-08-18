#!/bin/bash
# s154-census-orm.sh — testa hostcall RIFONDATA @ s153 (criterio
# s154-criterio-sonda.md p.4; COPIA DICHIARATA di s151-census-orm.sh, manifest
# s154-census-copia.diff; adattamenti: probe s154 b28899092ea1e2cc da
# census-prep-s154, lock della SESSIONE solo VERIFICATO, §5-bis sostituito
# dalle attese s154).
# CONTEGGI, mai cifre di tempo. Identità §3 pena census NULLO.
# rc: 0=VALIDO · 5=identità violate (NULLO) · 6=guardia pre-run · 7=setup ·
#     8=probe muto/raw vuoto · 9=lock di misura altrui.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp154-harness"; PREP="$H/census-prep-s154"; OUT="$H/census-out"; mkdir -p "$OUT"
V="$H/s154-census-verdetto.out"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
PROBE="$PREP/phpr-census-s154"
LOCK=/private/tmp/phpr-measure.lock
: > "$V"; rm -f "$OUT/census.done"
fin(){ echo "$1" > "$OUT/census.rc"; touch "$OUT/census.done"; exit "$1"; }

echo "== s154 testa hostcall rifondata @ s153 (probe b28899092ea1e2cc; criterio s154-criterio-sonda.md p.4; CONTEGGI mai tempo) ==" >> "$V"
HP=$(shasum -a 256 "$PROBE" | cut -c1-16)
[ "$HP" = "b28899092ea1e2cc" ] || { echo "rc=6 probe $HP != b28899092ea1e2cc (criterio p.4)" >> "$V"; fin 6; }
[ -e "$LOCK" ] || { echo "rc=9 lock della SESSIONE ASSENTE (adattamento dichiarato: solo verifica)" >> "$V"; echo 9 > "$OUT/census.rc"; touch "$OUT/census.done"; exit 9; }
echo "lock: presente (della sessione), solo verificato" >> "$V"

PAD=$(printf 'x%.0s' $(seq 1 70))
SP="/private/tmp/s154-census-sp-$PAD"
WORK="$SP/orm-work"
[ "${#WORK}" -ge 100 ] || { echo "rc=6 workdir ${#WORK} char < 100 (criterio §4)" >> "$V"; fin 6; }
echo "workdir=${#WORK} char (>=100, criterio §4)" >> "$V"

SMK="$OUT/smoke-rerun"; mkdir -p "$SMK"
CENSUS_PHPR="$PROBE" "$PREP/smoke151-check.sh" "$SMK" > "$SMK/check.log" 2>&1
SRC_RC=$?
[ "$SRC_RC" -eq 0 ] || { echo "rc=8 smoke pre-run FALLITO (rc=$SRC_RC, dettaglio $SMK/check.log)" >> "$V"; fin 8; }
echo "smoke pre-run: rc=0 (checker a esiti esatti della prep, rieseguito)" >> "$V"

busy(){ { pgrep -x rustc || pgrep -x cargo || pgrep -f rust-analyzer; } > /dev/null 2>&1 && echo BUSY || echo CLEAN; }

rm -rf "$SP"; mkdir -p "$SP"
tar xzf "$GATES/orm-work.tgz" -C "$SP" || { echo "rc=7 untar" >> "$V"; fin 7; }
cd "$WORK" || { echo "rc=7 cd" >> "$V"; fin 7; }

for R in 1 2; do
  RAW="$OUT/census-mem-r$R.txt"; rm -f "$RAW"
  PRE=$(busy)
  PHPR_MEM_CENSUS="$RAW" "$PROBE" vendor/bin/phpunit --no-coverage > "$OUT/run-r$R.txt" 2>&1 &
  PID=$!
  ( sleep 1800; kill -9 "$PID" 2>/dev/null ) & WD=$!
  wait "$PID"; RC=$?
  kill "$WD" 2>/dev/null; wait "$WD" 2>/dev/null
  POST=$(busy)
  echo "replica r$R: run rc=$RC busy_pre=$PRE busy_post=$POST (sentinella contesa: conteggi non tempo, non-gate)" >> "$V"
  [ -s "$RAW" ] || { echo "rc=8 raw r$R VUOTO (probe muto)" >> "$V"; fin 8; }
  tr -d '\0' < "$OUT/run-r$R.txt" | sed -n 's/^[0-9][0-9]*) \(.*\)$/\1/p' | sort -u > "$OUT/r$R.failnames"
  tr -d '\0' < "$RAW" > "$OUT/clean-r$R.txt"
done
diff -q "$REPO/wp125-harness/orm-baseline-failnames.txt" "$OUT/r1.failnames" > /dev/null 2>&1 && P1=OK || P1=DIFF
diff -q "$OUT/r1.failnames" "$OUT/r2.failnames" > /dev/null 2>&1 && P12=OK || P12=DIFF
echo "sentinella fail-set: r1-vs-baseline16=$P1 · r1-vs-r2=$P12 (non-gate, si dichiara)" >> "$V"

NULLO=0
for R in 1 2; do
  awk -v R="$R" '
    /^s151site /{ c=""; n=0; for(i=2;i<=NF;i++){split($i,kv,"="); if(kv[1]=="channel")c=kv[2]; else if(kv[1]=="n")n=kv[2]+0} sum[c]+=n; next }
    /^s151tot /{ c=""; t=0; for(i=2;i<=NF;i++){split($i,kv,"="); if(kv[1]=="channel")c=kv[2]; else if(kv[1]=="n")t=kv[2]+0} tot[c]+=t; seen[c]=1; next }
    /^s151overlap /{ split($2,kv,"="); OV+=kv[2]+0; next }
    /^s151clsovf /{ split($2,kv,"="); CO+=kv[2]+0; next }
    /^s151snap /{ split($2,kv,"="); SNAP+=kv[2]+0; next }
    /^s151cons /{ b=cc=d=l=0; cl=""; for(i=2;i<=NF;i++){split($i,kv,"="); if(kv[1]=="class")cl=kv[2]; else if(kv[1]=="births")b=kv[2]+0; else if(kv[1]=="clones")cc=kv[2]+0; else if(kv[1]=="drops")d=kv[2]+0; else if(kv[1]=="live_end")l=kv[2]+0}
        CB+=b; CC+=cc; CD+=d; CL+=l; NC++; if(b+cc!=d+l){NV++; if(NV<=5)viol=viol" "cl"("b"+"cc"!="d"+"l")"} next }
    /^s149sum /{ hn=sn=un=ovf=0; for(i=2;i<=NF;i++){split($i,kv,"="); if(kv[1]=="hostcall_n")hn=kv[2]+0; else if(kv[1]=="sum_name_n")sn=kv[2]+0; else if(kv[1]=="unnamed_n")un=kv[2]+0; else if(kv[1]=="overflow")ovf=kv[2]+0} HN+=hn; SN+=sn; UN+=un; OVF+=ovf; next }
    END{
      bad=0
      for(ch in seen){ if(sum[ch]!=tot[ch]){bad++; printf "IDVIOL r%s %s: Sigma_siti=%d != tot=%d\n",R,ch,sum[ch],tot[ch]} else printf "ID r%s %s: Sigma_siti=%d == tot OK\n",R,ch,tot[ch] }
      printf "OV r%s overlap=%d clsovf=%d snap=%d\n",R,OV,CO,SNAP
      if(OV!=0||CO!=0){bad++}
      if(NV>0){bad++; printf "CONSVIOL r%s classi_violate=%d es.%s\n",R,NV,viol} else printf "CONS r%s classi=%d b=%d c=%d d=%d l=%d (b+c==d+l per OGNI riga-classe)\n",R,NC,CB,CC,CD,CL
      ok=(HN==SN+UN && UN==0 && OVF==0)
      printf "S149 r%s hostcall_n=%d sum_name=%d unnamed=%d overflow=%d %s\n",R,HN,SN,UN,OVF,ok?"OK":"VIOL"
      if(!ok)bad++
      printf "HOSTN_R%s=%d\n",R,HN
      exit bad?1:0
    }' "$OUT/clean-r$R.txt" >> "$V" || NULLO=1
  grep -a "^s151props " "$OUT/clean-r$R.txt" | sed "s/^/PROPS r$R /" >> "$V"
done

grep -a "^s151tot " "$OUT/clean-r1.txt" | sort > "$OUT/tot-r1.txt"
grep -a "^s151tot " "$OUT/clean-r2.txt" | sort > "$OUT/tot-r2.txt"
if diff -q "$OUT/tot-r1.txt" "$OUT/tot-r2.txt" > /dev/null; then
  echo "repliche: s151tot r1 == r2 (identiche, precedente 0,000%)" >> "$V"
else
  echo "repliche: s151tot r1 != r2 — DIFFERENZE DICHIARATE:" >> "$V"
  diff "$OUT/tot-r1.txt" "$OUT/tot-r2.txt" | head -12 >> "$V"
fi

for ch in c1 c2 c3 c4 c5; do
  echo "-- top-10 siti $ch (r1, somma pid):" >> "$V"
  awk -v C="$ch" '/^s151site /{c="";s="";n=0; for(i=2;i<=NF;i++){split($i,kv,"="); if(kv[1]=="channel")c=kv[2]; else if(kv[1]=="site")s=kv[2]; else if(kv[1]=="n")n=kv[2]+0} if(c==C)a[s]+=n} END{for(s in a)printf "%d %s\n",a[s],s}' "$OUT/clean-r1.txt" | sort -rn | head -10 | sed 's/^/   /' >> "$V"
done
echo "-- testa hostcall per-NOME top-15 (r1, somma pid):" >> "$V"
awk '/^s149name /{nm="";n=0; for(i=2;i<=NF;i++){split($i,kv,"="); if(kv[1]=="name")nm=kv[2]; else if(kv[1]=="n")n=kv[2]+0} a[nm]+=n} END{for(nm in a)printf "%d %s\n",a[nm],nm}' "$OUT/clean-r1.txt" | sort -rn | head -15 | sed 's/^/   /' >> "$V"

# attese s154 (criterio sonda p.4): residuo non-backtrace 60,9M ±2%;
# debug_backtrace ≈ 0,473M×13 = 6,15M ±10%; conteggi class_exists e
# get_declared_classes RIPORTATI (denominatori p.3).
BTN1=$(awk '/^s149name /{nm="";n=0; for(i=2;i<=NF;i++){split($i,kv,"="); if(kv[1]=="name")nm=kv[2]; else if(kv[1]=="n")n=kv[2]+0} if(nm=="debug_backtrace")s+=n} END{print s+0}' "$OUT/clean-r1.txt")
HN1=$(sed -n 's/^HOSTN_R1=\([0-9][0-9]*\)$/\1/p' "$V" | head -1)
CE1=$(awk '/^s149name /{nm="";n=0; for(i=2;i<=NF;i++){split($i,kv,"="); if(kv[1]=="name")nm=kv[2]; else if(kv[1]=="n")n=kv[2]+0} if(nm=="class_exists")s+=n} END{print s+0}' "$OUT/clean-r1.txt")
GD1=$(awk '/^s149name /{nm="";n=0; for(i=2;i<=NF;i++){split($i,kv,"="); if(kv[1]=="name")nm=kv[2]; else if(kv[1]=="n")n=kv[2]+0} if(nm=="get_declared_classes")s+=n} END{print s+0}' "$OUT/clean-r1.txt")
if [ -n "${HN1:-}" ] && [ "$HN1" -gt 0 ]; then
  awk -v h="$HN1" -v bt="$BTN1" -v ce="$CE1" -v gd="$GD1" 'BEGIN{
    resid=h-bt; a=60900000
    dr=(resid>a?resid-a:a-resid)/a*100
    battL=6150000*0.9; battH=6150000*1.1
    printf "§5-bis-s154: hostcall_n r1=%d · debug_backtrace=%d · residuo non-bt=%d (Δ%% vs 60,9M = %.2f) · class_exists=%d · get_declared_classes=%d\n",h,bt,resid,dr,ce,gd
    okr=(dr<=2)?"DENTRO ±2%":"FUORI ±2% (dichiarare)"
    okb=(bt>=battL && bt<=battH)?"DENTRO ±10% di 6,15M":"FUORI ±10% di 6,15M (dichiarare)"
    printf "§5-bis-s154 attese: residuo %s · debug_backtrace %s\n",okr,okb
  }' >> "$V"
else
  echo "§5-bis-s154: HOSTN_R1 non estratto — giudizio NON emesso (dichiarato)" >> "$V"
fi

if [ "$NULLO" -eq 0 ]; then
  echo "CENSUS TRANCHE-5 VALIDO rc=0 (identità §3 tutte OK su r1 e r2)" >> "$V"
  fin 0
else
  echo "CENSUS NULLO rc=5 (identità §3 violate — vedi righe IDVIOL/CONSVIOL/VIOL)" >> "$V"
  fin 5
fi
