#!/bin/bash
# s190-direzione-k0.sh <tag> — S-190 C2, A/B di DIREZIONE (NON verdetto; il giudice resta la guardia b3 di notte): costo per Ret
# del tree con la cura O(1) v2 (binario RELEASE del tree, env TREE) contro il pin s183 (A), sui driver della guardia S-186:
# ret-buf-dq-k0/k4/k16 (ns/Ret = (user − pavimento)/(N·RETS_PER_ITER)), mcalls-dq e calls-dq (ns/iter), R=3 interallacciati ABAB,
# pavimento `empty.php` med3 per binario, stimatore = mediana delle differenze APPAIATE per giro. Soglia di direzione (criterio p.6):
# D(K0) < 4 ns/Ret e |D(K4) − D(K0)|, |D(K16) − D(K0)| < 4 (K piatto). Esito in s190-direzione-k0-<tag>.out; rc 0 (direzione ok) / 1.
# Generato con assistenza AI (Claude Fable 5).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$P/wp190-harness"; H6="$P/wp186-harness"
TAG="${1:?tag}"
A="${PIN:-$HOME/Claude/php-rust-output/release/phpr}"; AEXP="9a6ae11bf7e900ef"
B="${TREE:?TREE = binario RELEASE del tree (es. ~/Claude/phpr-target/dev-output/release/phpr)}"
R="${R:-3}"
EMPTY="$P/wp160-harness/empty.php"
VERD="$H/s190-direzione-k0-$TAG.out"; OUT="$H/ab-out/direzione-$TAG"; mkdir -p "$OUT"
: > "$VERD"; note(){ echo "$1" | tee -a "$VERD"; }
AM="$(shasum -a 256 "$A" | cut -c1-16)"; BM="$(shasum -a 256 "$B" | cut -c1-16)"
[ "$AM" = "$AEXP" ] || { note "rc=9 A $AM != pin s183 $AEXP"; exit 9; }
[ "$BM" != "$AM" ] || { note "rc=9 B == pin"; exit 9; }
pgrep -x cargo >/dev/null && { note "rc=8 cargo attivo: niente direzione"; exit 8; }
nsrc(){ awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$1"; }
rpi(){ awk 'match($0, /RETS_PER_ITER=[0-9]+/) {print substr($0, RSTART+14, RLENGTH-14); exit}' "$1"; }
ucpu(){ { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$@" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
med3(){ printf '%s\n' "$@" | sort -n | awk '{a[NR]=$1} END{print a[int((NR+1)/2)]}'; }
note "== s190 direzione K0 tag=$TAG A=pin $AM · B=tree $BM · R=$R · $(date '+%F %T') (DIREZIONE, non verdetto) =="
FA=$(med3 $(ucpu "$A" "$EMPTY") $(ucpu "$A" "$EMPTY") $(ucpu "$A" "$EMPTY")); FB=$(med3 $(ucpu "$B" "$EMPTY") $(ucpu "$B" "$EMPTY") $(ucpu "$B" "$EMPTY"))
note "pavimento empty.php med3: A=$FA s · B=$FB s"
RC=0; DK0=""
for d in "k0:$H6/ret-buf-dq-k0.php" "k4:$H6/ret-buf-dq-k4.php" "k16:$H6/ret-buf-dq-k16.php" "mcalls:$P/wp185-harness/mcalls-dq.php" "calls:$P/wp185-harness/calls-dq.php"; do
  n="${d%%:*}"; f="${d#*:}"; N=$(nsrc "$f"); RP=$(rpi "$f"); [ -n "$RP" ] || RP=1
  diffs=""; ta=""; tb=""
  for r in $(seq 1 "$R"); do
    if [ $((r % 2)) = 1 ]; then a=$(ucpu "$A" "$f"); b=$(ucpu "$B" "$f"); else b=$(ucpu "$B" "$f"); a=$(ucpu "$A" "$f"); fi
    ta="$ta $a"; tb="$tb $b"
    diffs="$diffs $(python3 -c 'import sys; a,b,fa,fb,n,rp=map(float,sys.argv[1:7]); print("%.2f" % (((b-fb)-(a-fa))/(n*rp)*1e9))' "$a" "$b" "$FA" "$FB" "$N" "$RP")"
  done
  D=$(med3 $diffs); [ "$n" = k0 ] && DK0="$D"
  note "$n: N=$N rets/iter=$RP · A=[$ta ] s · B=[$tb ] s · D(B−A) appaiato=[$diffs ] ns/Ret-o-iter · mediana $D"
  echo "$D" > "$OUT/$n.D"
done
python3 - "$(cat "$OUT/k0.D")" "$(cat "$OUT/k4.D")" "$(cat "$OUT/k16.D")" >> "$VERD" <<'PY'
import sys
k0, k4, k16 = map(float, sys.argv[1:4])
ok0 = k0 < 4.0; flat = abs(k4 - k0) < 4.0 and abs(k16 - k0) < 4.0
print(f"DIREZIONE: D(K0)={k0:.2f} ns/Ret ({'< 4 OK' if ok0 else '>= 4: K0 ancora caro'}) · K piatto {'OK' if flat else 'NO'} (K4−K0={k4-k0:.2f}, K16−K0={k16-k0:.2f})")
print("rc=" + ("0" if ok0 and flat else "1"))
PY
tail -1 "$VERD" | grep -q 'rc=0' || RC=1
note "fine $(date '+%F %T')"; exit "$RC"
