#!/bin/bash
# s153-sonda-conteggi.sh — sonda conteggi per SITO (A3a istruttoria, piano
# s153-sonda-piano.md). Probe census s151 ab02faec0abfab67, micro a DUE N,
# k = Δ/ΔN. CONTEGGI, mai tempo. rc: 0=ok · 6=guardia · 8=run/raw rotto · 9=lock altrui.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp153-harness"; OUT="$H/sonda-out"; mkdir -p "$OUT"
V="$H/s153-sonda-conteggi-verdetto.out"
PROBE="$REPO/wp151-harness/census-prep/phpr-census-s151"
LOCK=/private/tmp/phpr-measure.lock
: > "$V"; rm -f "$OUT/sonda.done"
fin(){ echo "$1" > "$OUT/sonda.rc"; touch "$OUT/sonda.done"; exit "$1"; }

HP=$(shasum -a 256 "$PROBE" | cut -c1-16)
[ "$HP" = "ab02faec0abfab67" ] || { echo "rc=6 probe $HP != ab02faec0abfab67" >> "$V"; fin 6; }
if [ -e "$LOCK" ]; then echo "rc=9 lock di misura ALTRUI presente: non lo tocco" >> "$V"; fin 9; fi
echo "s153-sonda pid=$$ avvio $(date '+%F %T')" > "$LOCK"
trap 'if grep -q "s153-sonda" "$LOCK" 2>/dev/null; then rm -f "$LOCK"; fi' EXIT

N1=100000; N2=300000; DN=$((N2-N1))
echo "== s153 sonda conteggi per SITO (probe ab02faec0abfab67; k=Δ/ΔN, N1=$N1 N2=$N2; canale c2) ==" >> "$V"
RC=0
for M in td-this td-local td-die ps-set; do
  for N in $N1 $N2; do
    RAW="$OUT/$M-n$N.raw"; rm -f "$RAW"
    TDN=$N PHPR_MEM_CENSUS="$RAW" "$PROBE" "$H/$M.php" > "$OUT/$M-n$N.log" 2>&1
    R=$?
    if [ "$R" -ne 0 ] || [ ! -s "$RAW" ] || ! grep -q -- "-OK n=$N" "$OUT/$M-n$N.log"; then
      echo "rc=8 $M n$N: run rc=$R raw=$([ -s "$RAW" ] && echo ok || echo VUOTO) marker=$(grep -c -- "-OK" "$OUT/$M-n$N.log" 2>/dev/null)" >> "$V"
      RC=8
    fi
  done
  [ "$RC" -ne 0 ] && break
  echo "-- $M: k per sito (canale c2, tutte le righe con Δ!=0; Δ su ΔN=$DN):" >> "$V"
  awk -v dn=$DN 'FNR==1{f++}
    /^s151site /{c="";s="";n=0;for(i=2;i<=NF;i++){split($i,kv,"=");if(kv[1]=="channel")c=kv[2];else if(kv[1]=="site")s=kv[2];else if(kv[1]=="n")n=kv[2]+0}
      if(c=="c2"){if(f==1)a[s]=n; else b[s]+=n}}
    END{for(s in b){d=b[s]-a[s]; if(d!=0)printf "   %s k=%.4f (Δ=%d)\n",s,d/dn,d}
        for(s in a){if(!(s in b)&&a[s]!=0)printf "   %s SOLO-N1 n=%d\n",s,a[s]}}' \
    <(tr -d '\0' < "$OUT/$M-n$N1.raw") <(tr -d '\0' < "$OUT/$M-n$N2.raw") | sort >> "$V"
done
if [ "$RC" -eq 0 ]; then
  echo "SONDA VALIDA rc=0 (k esatti se Δ multiplo intero di ΔN — si legge nel confronto col piano)" >> "$V"
fi
fin "$RC"
