#!/bin/bash
# s145-partition-golden-test.sh — golden del parser di partizione (lezione
# S-143/S-144: un parser senza golden non giudica). Due casi sintetici con
# quote calcolate A MANO; il confronto è su stringhe ESATTE del verdetto.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp145-harness"
T="$(mktemp -d /private/tmp/s145-golden.XXXXXX)"
trap 'rm -rf "$T"' EXIT
FAIL=0

mkgolden() { # $1=dir  $2..$10 prezzi: cal mv_scalar mv_str mv_arr mv_obj note_scalar note_cont pair_zcell pair_arr0
  d="$1"; shift
  mkdir -p "$d"
  for r in r1 r2; do
    { i=0
      for k in cal mv_scalar mv_str mv_arr mv_obj note_scalar note_cont_repeat pair_zcell pair_arr0; do
        i=$((i+1)); eval "v=\$$i"
        echo "s145.price.${k}_ns=${v}"
      done
      echo "s145.price.n_mv=1000"
      echo "s145.price.n_pair=100"
    } > "$d/price-$r.txt"
  done
}

# case1: cal=1; netti: mv_scalar=2 str=4 arr=6 obj=8, note_scalar=1 note_cont=10
# conteggi: scalar=10 str=10 arr=10 obj=5 ref=3 rcother=2 (rc=10, tot=40)
#           gcnote_total=20 cont=10
# memcpy=2*40=80; incdec=(4-2)*10+(6-2)*10+(8-2)*10=120; nota=1*10+10*10=110
# den=310 => quote 25,8 / 38,7 / 35,5; inc+nota=74,2 >=60, inc>=nota => B1->B2
mkgolden "$T/c1" 1 3 5 7 9 2 11 16 21
for r in r1 r2; do
  echo "pid=1 tag=exit arr.cum_n=7 obj.cum_n=5 s144.rczval_n=3 s145.clone_scalar_n=10 s145.clone_str_n=10 s145.clone_arr_n=10 s145.clone_obj_n=5 s145.clone_ref_n=3 s145.clone_rcother_n=2" > "$T/c1/census-mem-$r.txt"
  printf 'zvalcensus_s101 propget_val=0 gcnote_total=20 gcnote_scalar=8 gcnote_obj=10\nzvalcensus_s145 gcnote_cont=10\n' > "$T/c1/census-zval-$r.txt"
done
OUT1="$(/usr/bin/python3 "$H/s145-partition.py" "$T/c1")"
echo "$OUT1" | grep -Fq "QUOTE: memcpy=25.8% inc-dec=38.7% nota=35.5%" || { echo "GOLDEN c1 QUOTE FALLITO"; echo "$OUT1"; FAIL=1; }
echo "$OUT1" | grep -Fq "ESITO REGOLA: inc-dec+nota = 74.2% >= 60% => fette B1->B2" || { echo "GOLDEN c1 ESITO FALLITO"; echo "$OUT1"; FAIL=1; }

# case2 (memcpy-dominato): netti mv_scalar=20 str=21 arr=21 obj=21; note 1/1
# conteggi come sopra => memcpy=800; incdec=1*10+1*10+1*10=30; nota=1*10+1*10=20
# den=850 => memcpy 94,1% >=60 => B3/concilio
mkgolden "$T/c2" 1 21 22 22 22 2 2 16 21
cp "$T/c1/census-mem-r1.txt" "$T/c2/census-mem-r1.txt"
cp "$T/c1/census-mem-r2.txt" "$T/c2/census-mem-r2.txt"
cp "$T/c1/census-zval-r1.txt" "$T/c2/census-zval-r1.txt"
cp "$T/c1/census-zval-r2.txt" "$T/c2/census-zval-r2.txt"
OUT2="$(/usr/bin/python3 "$H/s145-partition.py" "$T/c2")"
echo "$OUT2" | grep -Fq "QUOTE: memcpy=94.1% inc-dec=3.5% nota=2.4%" || { echo "GOLDEN c2 QUOTE FALLITO"; echo "$OUT2"; FAIL=1; }
echo "$OUT2" | grep -Fq "B1/B2 NON si aprono, filone conteggi (B3) TORNA AL CONCILIO (KS-B4)" || { echo "GOLDEN c2 ESITO FALLITO"; echo "$OUT2"; FAIL=1; }

# case3 (replica rotta sui prezzi): r2 con mv_scalar +5% => il parser DEVE
# uscire rc=2 e DICHIARARE lo scarto.
mkdir -p "$T/c3"; cp "$T/c1"/*.txt "$T/c3/"
sed -i '' 's/s145.price.mv_scalar_ns=3/s145.price.mv_scalar_ns=3.2/' "$T/c3/price-r2.txt"
if /usr/bin/python3 "$H/s145-partition.py" "$T/c3" > "$T/c3.out" 2>&1; then
  echo "GOLDEN c3 FALLITO: rc=0 con replica rotta"; FAIL=1
else
  grep -Fq "DICHIARA scarto>1% su prezzo mv_scalar" "$T/c3.out" || { echo "GOLDEN c3 dicitura FALLITA"; cat "$T/c3.out"; FAIL=1; }
fi

[ "$FAIL" = 0 ] && echo "GOLDEN PASS 3/3" || echo "GOLDEN FAIL"
exit "$FAIL"
