#!/bin/bash
# s137-sonda.sh — ripartizione eccedenza FD1 +13,7 su m-dimwrite (criterio
# s137-criterio-sonda.md, commit PRIMA del run). Metodo di s136-tempo.sh
# INVARIATO (UN segmento per run PHPR_TP, R=3, mediana ns/span, overhead seg9
# sottratto, parita' stdout vs oracle a ogni run) + aggiunte s137: quiescenza
# gate SEPARATO, parita' fixtures-fd1 probe==PIN prima del tempo (criterio
# p.6), run conteggi hit/miss (PHPR_TPC=1) SEPARATO dal tempo (p.4),
# ripartizione vs modello s136-tempo nel giudice (p.3/p.5). Segmenti s137:
# 0=arm 1=pop+keys 2=admission 3=leaf(walk child) 4=post(boundary+gc+drain)
# 5=fast intero (call-site) 9=calibrazione. Contrasto TRA SONDE OMOGENEE
# (probe s135 = s136-tempo vs probe s136 = qui), MAI cifra-leva.
# rc autoritativo = sonda-out/sonda.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp137-harness"
J="$REPO/wp136-harness/micro-bisez/m-dimwrite.php"
FIX="$REPO/wp136-harness/fixtures-fd1.php"
BIN="/Volumes/Extreme Pro/Claude/phpr-s137-target-s136/release/phpr"
PINBIN="$HOME/Claude/php-rust-output/release/phpr"
ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
OUT="$H/sonda-out"; mkdir -p "$OUT"
VERD="$H/s137-sonda-verdetto.out"
RC="$OUT/sonda.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE — tentativo nuovo = file nuovo" >&2; exit 7; }
[ -x "$BIN" ] || { echo "manca il binario tp" >&2; echo 1 > "$RC"; exit 1; }
PIN_ATTESO="1e14793ec0d9650c"
[ "$(shasum -a 256 "$PINBIN" | cut -c1-16)" = "$PIN_ATTESO" ] || { echo "pin mismatch" >&2; echo 9 > "$RC"; exit 9; }
"$QUIESCE" "$OUT/quiesce-sonda.rc" > "$OUT/quiesce-sonda.log" 2>&1 || { echo 8 > "$RC"; echo "quiescenza-fail (rc nel file)" >&2; exit 8; }

{
echo "== s137 sonda ripartizione eccedenza FD1 su m-dimwrite (criterio s137-criterio-sonda.md) =="
echo "tp_bin=$(shasum -a 256 "$BIN" | cut -c1-16) (build emendata, MAI pinnabile)"
echo "quiescenza: rc=$(cat "$OUT/quiesce-sonda.rc") (file: sonda-out/quiesce-sonda.rc)"
FAIL=0
# parita' fixtures-fd1 (criterio p.6): probe == PIN (stesso sorgente al netto dei timer)
"$BIN" "$FIX" > "$OUT/fix-probe.txt" 2>&1
"$PINBIN" "$FIX" > "$OUT/fix-pin.txt" 2>&1
if diff -q "$OUT/fix-probe.txt" "$OUT/fix-pin.txt" > /dev/null; then
  echo "parita_fixtures_fd1: OK (probe==pin)"
else
  echo "parita_fixtures_fd1: ROTTA"; FAIL=1
fi
EXP=$("$ORACLE" "$J")
# conteggi hit/miss (criterio p.4): run SEPARATO dal tempo
GOT=$(PHPR_TPC=1 PHPR_TP_OUT="$OUT/conteggi.tp" "$BIN" "$J")
[ "$GOT" = "$EXP" ] || { echo "conteggi: PARITA' STDOUT ROTTA"; FAIL=1; }
HITS=$(awk '{for(i=1;i<=NF;i++){if($i ~ /^hit=/)h=substr($i,5)}} END{print h+0}' "$OUT/conteggi.tp")
MISSES=$(awk '{for(i=1;i<=NF;i++){if($i ~ /^miss=/)m=substr($i,6)}} END{print m+0}' "$OUT/conteggi.tp")
echo "conteggi: hit=$HITS miss=$MISSES (atteso hit~3e6, miss=O(1))"
if [ "$MISSES" -gt 10 ] || [ "$HITS" -lt 2900000 ]; then
  echo "conteggi: MISS RICORRENTI o hit sotto scala — sonda FALLITA (criterio p.4)"; FAIL=1
fi
for seg in 0 1 2 3 4 5 9; do
  vals=()
  for r in 1 2 3; do
    GOT=$(PHPR_TP=$seg PHPR_TP_OUT="$OUT/seg$seg-r$r.tp" "$BIN" "$J")
    [ "$GOT" = "$EXP" ] || { echo "seg$seg r$r: PARITA' STDOUT ROTTA"; FAIL=1; }
    vals+=("$(awk '{for(i=1;i<=NF;i++){if($i ~ /^ns=/){ns=substr($i,4)}; if($i ~ /^spans=/){sp=substr($i,7)}}; if(sp>0) printf "%.3f", ns/sp}' "$OUT/seg$seg-r$r.tp")")
  done
  M=$(printf '%s\n' "${vals[@]}" | sort -n | awk 'NR==2')
  echo "seg${seg}_ns_span_med3=$M (r: ${vals[*]})"
  eval "S$seg=$M"
done
python3 - "$S0" "$S1" "$S2" "$S3" "$S4" "$S5" "$S9" <<'PY'
import sys
s0, s1, s2, s3, s4, s5, s9 = map(float, sys.argv[1:8])
t = lambda x: x - s9
arm, pop, adm, leaf, post, fast = t(s0), t(s1), t(s2), t(s3), t(s4), t(s5)
disp = arm - pop - fast
altro = fast - adm - leaf - post
print(f"overhead_coppia_ns={s9:.1f} (seg9, sottratto)")
print(f"arm_totale={arm:.1f}  pop+keys={pop:.1f}  fast_intero={fast:.1f}")
print(f"  admission={adm:.1f}  leaf_walk={leaf:.1f}  post_plumbing={post:.1f}")
print(f"derivate: dispatch+push(0-1-5)={disp:.1f}  fast_altro(5-2-3-4)={altro:.1f}")
somma = pop + fast
print(f"chiusura=(1+5)/0 = {somma:.1f}/{arm:.1f} = {somma/arm*100 if arm>0 else 0:.0f}%  (criterio p.3: >=90% o modello INCOMPLETO)")
# modello s136-tempo (probe @ s135, s136-tempo-verdetto.out): canali kept attesi
ARM_PRE, D_AB, UB, BANDA = 118.2, 83.3, 69.6, 13.3
POP_M, DISP_M, PLUMB_M, LEAF_M = 4.5, 7.0, 17.6, 18.9
print(f"gain_tra_sonde={ARM_PRE:.1f}-{arm:.1f}={ARM_PRE-arm:.1f} (contrasto tra sonde OMOGENEE; A/B resta D=+{D_AB})")
d_pop, d_disp, d_plumb, d_leaf = POP_M-pop, DISP_M-disp, PLUMB_M-post, LEAF_M-leaf
print("ripartizione per NOME (modello kept s136 -> misura s137; + = restituito oltre l'UB):")
print(f"  pop+keys: {POP_M:.1f}->{pop:.1f} delta={d_pop:+.1f}")
print(f"  dispatch+push: {DISP_M:.1f}->{disp:.1f} delta={d_disp:+.1f}")
print(f"  plumbing (set-entry -> post_plumbing kept): {PLUMB_M:.1f}->{post:.1f} delta={d_plumb:+.1f}")
print(f"  leaf (walk child vs leaf_index): {LEAF_M:.1f}->{leaf:.1f} delta={d_leaf:+.1f}")
print(f"  costo AGGIUNTO admission+altro: -{adm+altro:.1f}")
ecc = d_pop + d_disp + d_plumb + d_leaf - adm - altro
print(f"eccedenza_spiegata_dalla_sonda={ecc:+.1f} (attesa ~+13,7)")
gain_mod = UB + ecc
print(f"identita': UB {UB} + eccedenza {ecc:+.1f} = {gain_mod:.1f} vs D_A/B {D_AB} -> scarto {gain_mod-D_AB:+.1f} (banda {BANDA})")
if abs(gain_mod - D_AB) <= BANDA and somma/arm >= 0.9:
    dom = max([("plumbing", d_plumb), ("leaf", d_leaf), ("dispatch", d_disp), ("pop", d_pop)], key=lambda kv: kv[1])
    print(f"ESITO SONDA: ECCEDENZA ATTRIBUITA per NOME (canale dominante: {dom[0]} {dom[1]:+.1f})")
else:
    print("ESITO SONDA: NON CHIUSA — il blocco az.rev. S-136 #4 sulle leve dim-write PERSISTE")
PY
if [ "$FAIL" = 0 ]; then echo "SONDA ACQUISITA (parita' ok)"; echo 0 > "$RC"; else echo 1 > "$RC"; fi
} > "$VERD" 2>&1
exit "$(cat "$RC")"
