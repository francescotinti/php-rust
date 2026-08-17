#!/bin/bash
# s152-gonogo.sh — aritmetica GO/NO-GO A3c (criterio s152-criterio-sonde.md §6;
# soglie PRE-registrate s151-criterio-census.md §6). Scritto PRIMA di leggere
# i prezzi (implementazione meccanica del criterio). Input: sonda-out/price-r{1,2}.txt.
# Conteggi census (per replica, wp151-harness/s151-census-verdetto.out):
#   C1=253971737 C2=340931405 C3=6439636 C4=43214340 C5=191157853
#   C2 borrow_mut in [11784675; 60467189] (top-10 + residuo 48682514 agli estremi)
#   C5 quota obj in [105787249; 157382773] (top-10 + residuo 51595524 agli estremi)
# Soglie: S1 banda_netta>=0,50 s · S2 banda_netta>=1,53 s · S3 somma lorda Object>=4,58 s.
set -u
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp152-harness"
OUT="$H/sonda-out"
V="$H/s152-gonogo-verdetto.out"
[ -e "$V" ] && { echo "verdetto ESISTE" >&2; exit 7; }
# EMENDA dichiarata (criterio p.3, t3 dovuta per replica >2% su mv_obj/
# mock_dup_rel/c2_borrow): banda per chiave = [min,max] su TUTTE le repliche
# disponibili — le 2 di record (price-rec-r*, salvate prima della t3) + le 2
# della t3 (price-r* del verdetto -t3).
FILES=""
for f in "$OUT/price-rec-r1.txt" "$OUT/price-rec-r2.txt" "$OUT/price-r1.txt" "$OUT/price-r2.txt"; do
  [ -s "$f" ] && FILES="$FILES $f"
done
[ -n "$FILES" ] || { echo "prezzi assenti" >&2; exit 8; }

get(){ # $1=file $2=chiave completa → valore
  sed -n "s/^$2=\([0-9.]*\)$/\1/p" "$1" | head -1
}
band(){ # $1=chiave → "min max" su tutte le repliche
  { for f in $FILES; do get "$f" "$1"; done; } | awk 'NR==1{lo=$1;hi=$1} {if($1<lo)lo=$1; if($1>hi)hi=$1} END{print lo, hi}'
}

read C2B_LO C2B_HI <<< "$(band s152.price.c2_borrow_ns)"
read C2M_LO C2M_HI <<< "$(band s152.price.c2_borrow_mut_ns)"
read C1_LO  C1_HI  <<< "$(band s152.price.c1_pair_ns)"
read C3_LO  C3_HI  <<< "$(band s152.price.c3_size_pair_ns)"
read MD_LO  MD_HI  <<< "$(band s152.price.mock_deref_ns)"
read MR_LO  MR_HI  <<< "$(band s152.price.mock_dup_rel_ns)"
read MA_LO  MA_HI  <<< "$(band s152.price.mock_alloc_ns)"
read MQ_LO  MQ_HI  <<< "$(band s152.price.mock_decq_ns)"
read MH_LO  MH_HI  <<< "$(band s152.price.miheap_pair_ns)"
read NC_LO  NC_HI  <<< "$(band s145.price.note_cont_repeat_ns)"
read MO_LO  MO_HI  <<< "$(band s145.price.mv_obj_ns)"
read MS_LO  MS_HI  <<< "$(band s145.price.mv_scalar_ns)"
read MST_LO MST_HI <<< "$(band s145.price.mv_str_ns)"
read MAR_LO MAR_HI <<< "$(band s145.price.mv_arr_ns)"

awk -v c2b_lo="$C2B_LO" -v c2b_hi="$C2B_HI" -v c2m_lo="$C2M_LO" -v c2m_hi="$C2M_HI" \
    -v c1_lo="$C1_LO" -v c1_hi="$C1_HI" -v c3_lo="$C3_LO" -v c3_hi="$C3_HI" \
    -v md_lo="$MD_LO" -v md_hi="$MD_HI" -v mr_lo="$MR_LO" -v mr_hi="$MR_HI" \
    -v ma_lo="$MA_LO" -v ma_hi="$MA_HI" -v nc_lo="$NC_LO" -v nc_hi="$NC_HI" \
    -v mo_lo="$MO_LO" -v mo_hi="$MO_HI" -v ms_lo="$MS_LO" -v ms_hi="$MS_HI" \
    -v mst_lo="$MST_LO" -v mst_hi="$MST_HI" -v mar_lo="$MAR_LO" -v mar_hi="$MAR_HI" '
function max(a,b){return a>b?a:b}
BEGIN{
  C1=253971737; C2=340931405; C3=6439636; C4=43214340; C5=191157853
  BM_LO=11784675; BM_HI=60467189       # quota borrow_mut C2
  OBJ_LO=105787249; OBJ_HI=157382773   # quota obj C5
  NS=1e-9
  # --- banda_netta (canali NON-movimento; netti clampati a >=0) ---
  # C2: quota borrow_mut all_estremo che massimizza/minimizza (borrow_mut piu caro del borrow di norma)
  n_lo = max(0,(C2-BM_LO)*(c2b_lo-md_hi)) + max(0,BM_LO*(c2m_lo-md_hi))
  n_hi = max(0,(C2-BM_HI)*(c2b_hi-md_lo)) + max(0,BM_HI*(c2m_hi-md_lo))
  # se borrow_mut>borrow, per l_estremo hi conviene BM_HI; per lo BM_LO: coerente sopra
  c2n_lo=n_lo; c2n_hi=n_hi
  c4n_lo = max(0, C4*(nc_lo-md_hi));  c4n_hi = max(0, C4*(nc_hi-md_lo))
  c5n_lo = max(0, OBJ_LO*(mo_lo-mr_hi)); c5n_hi = max(0, OBJ_HI*(mo_hi-mr_lo))
  c3n_lo = max(0, C3*(c3_lo-ma_hi));  c3n_hi = max(0, C3*(c3_hi-ma_lo))
  bn_lo = (c2n_lo+c4n_lo+c5n_lo+c3n_lo)*NS
  bn_hi = (c2n_hi+c4n_hi+c5n_hi+c3n_hi)*NS
  # --- S3: somma LORDA prezzo-corrente x conteggio, C1 incluso ---
  # C2 lordo: mix borrow/borrow_mut agli estremi; C5 lordo: mix specie agli estremi
  s3_lo = ( C1*c1_lo + ((C2-BM_LO)*c2b_lo+BM_LO*c2m_lo) + C3*c3_lo + C4*nc_lo \
          + (OBJ_LO*mo_lo + (C5-OBJ_LO)*ms_lo) )*NS
  s3_hi = ( C1*c1_hi + ((C2-BM_HI)*c2b_hi+BM_HI*c2m_hi) + C3*c3_hi + C4*nc_hi \
          + (OBJ_HI*mo_hi + (C5-OBJ_HI)*mo_hi) )*NS
  printf "banda_netta = [%.3f; %.3f] s (C2 [%.3f;%.3f] + C4 [%.3f;%.3f] + C5obj [%.3f;%.3f] + C3 [%.3f;%.3f])\n", \
    bn_lo, bn_hi, c2n_lo*NS, c2n_hi*NS, c4n_lo*NS, c4n_hi*NS, c5n_lo*NS, c5n_hi*NS, c3n_lo*NS, c3n_hi*NS
  printf "S3 somma lorda Object (C1..C5) = [%.3f; %.3f] s\n", s3_lo, s3_hi
  S1=0.50; S2=1.53; S3=4.58
  printf "soglie: S1>=%.2f S2>=%.2f (banda_netta) · S3>=%.2f (lorda)\n", S1, S2, S3
  # giudizio pre-registrato (criterio s152 §6): (a) hi fallisce una => NO-GO robusto
  if (bn_hi < S1 || bn_hi < S2 || s3_hi < S3) {
    fails=""
    if (bn_hi < S1) fails=fails" S1"
    if (bn_hi < S2) fails=fails" S2"
    if (s3_hi < S3) fails=fails" S3"
    printf "VERDETTO: NO-GO ROBUSTO a fortiori (estremo hi fallisce:%s) — A3c CHIUSA (stile veto NaN-boxing); restano A3a/A3b micro-judged e leve per NOME\n", fails
  } else if (bn_lo >= S1 && bn_lo >= S2 && s3_lo >= S3) {
    print "VERDETTO: GO (estremo lo supera tutte; caveat mock-ottimistico: criterio A3c pieno con cache/re-entrancy/weak PRIMA di ogni codice)"
  } else {
    print "VERDETTO: INDECIDIBILE alla risoluzione — dovute terza replica + mock working-set (~180k slot) PRIMA di ogni giudizio (criterio §6c)"
  }
}' > "$V".body
{
echo "== s152 GO/NO-GO A3c (criterio s152-criterio-sonde.md §6; soglie s151 §6; conteggi census s151) =="
echo "prezzi: sonda-out/price-r{1,2}.txt del verdetto s152-sonda-canali (probe $(get "$OUT/price-r1.txt" s152.price.obj_size >/dev/null 2>&1; shasum -a 256 /private/tmp/phpr-sonda152-target/release/phpr 2>/dev/null | cut -c1-16))"
cat "$V".body
} > "$V"
rm -f "$V".body
cat "$V"
