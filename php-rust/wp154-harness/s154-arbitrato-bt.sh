#!/bin/bash
# s154-arbitrato-bt.sh — arbitrato della guardia backtrace (emenda §6-bis del
# criterio ce1): m-backtrace-hi (N=600000, tick 16,7), R=5 ABAB, pavimenti
# med3, soglia max(4; rumore drop-1). Argv neutro; bracci da copie verificate.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp154-harness"
OUT="$H/ab-out"; V="$H/s154-arbitrato-bt-verdetto.out"
A=/private/tmp/s154-armA/armA
B=/private/tmp/s154-armB/candB
: > "$V"; rm -f "$OUT/arbitrato-bt.rc"
fin(){ echo "$1" > "$OUT/arbitrato-bt.rc"; exit "$1"; }
[ "$(shasum -a 256 "$A" | cut -c1-8)" = "2023cbb9" ] || { echo "A!=2023cbb9" >> "$V"; fin 9; }
[ "$(shasum -a 256 "$B" | cut -c1-8)" = "e634d95c" ] || { echo "B!=e634d95c" >> "$V"; fin 9; }
[ -e /private/tmp/phpr-measure.lock ] || { echo "lock assente" >> "$V"; fin 9; }
Q="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp129-harness/s129-quiescenza.sh"
"$Q" "$OUT/quiesce-arbitrato.rc" > "$OUT/quiesce-arbitrato.log" 2>&1
[ "$(cat "$OUT/quiesce-arbitrato.rc")" = 0 ] || { echo "quiescenza FALLITA" >> "$V"; fin 1; }
ucpu(){ { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$1" "$2" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3(){ local a b c; a=$(ucpu "$1" "$2"); b=$(ucpu "$1" "$2"); c=$(ucpu "$1" "$2"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }
MB="$H/m-backtrace-hi.php"
OA=$("$A" "$MB"); OB=$("$B" "$MB")
[ "$OA" = "1200000" ] && [ "$OB" = "1200000" ] || { echo "parita' output ROTTA: A='$OA' B='$OB' attesi 1200000" >> "$V"; fin 2; }
FA=$(floor3 "$A" "$H/empty.php"); FB=$(floor3 "$B" "$H/empty.php")
{
echo "== s154 arbitrato guardia backtrace (emenda §6-bis): N=600000 tick 16,7; A=2023cbb9 B=e634d95c R=5 ABAB; soglia max(4, drop-1) =="
echo "floor_A=$FA floor_B=$FB parita' 1200000==1200000 ok"
TSV="$OUT/arbitrato-bt.tsv"; : > "$TSV"
for i in 1 2 3 4 5; do
  if [ $((i % 2)) -eq 1 ]; then TA=$(ucpu "$A" "$MB"); TB=$(ucpu "$B" "$MB"); ord=AB
  else TB=$(ucpu "$B" "$MB"); TA=$(ucpu "$A" "$MB"); ord=BA; fi
  printf '%s\t%s\t%s\t%s\t%s\n' "$i" "$TA" "$TB" "$ord" "600000" >> "$TSV"
  echo "  coppia$i [$ord]: rawA=$TA rawB=$TB"
done
python3 - "$TSV" "$FA" "$FB" <<'PY'
import sys
rows = [l.split("\t") for l in open(sys.argv[1]) if l.strip()]
fa, fb = float(sys.argv[2]), float(sys.argv[3])
n = 600000.0
na = [(float(r[1])-fa)/n*1e9 for r in rows]
nb = [(float(r[2])-fb)/n*1e9 for r in rows]
def med(v):
    s = sorted(v); m = len(s)
    return s[m//2] if m % 2 else (s[m//2-1]+s[m//2])/2
def trange(v):
    m = med(v)
    w = sorted(v, key=lambda x: (abs(x-m), x))[:-1]
    return max(w) - min(w)
ma, mb = med(na), med(nb)
d = ma - mb
ra, rb = trange(na), trange(nb)
thr = max(4.0, ra, rb)
print(f"ARBITRATO backtrace: A={ma:.1f} B={mb:.1f} ns/iter D={d:+.1f} soglia={thr:.1f} (rumore drop-1 A'={ra:.1f} B'={rb:.1f})")
if d <= -thr:
    print("ESITO: REGRESSIONE REALE — leva in istruttoria, niente promo"); sys.exit(5)
elif abs(d) < thr:
    print("ESITO: TICK-FLIP REFUTATO — guardia dichiarata ok (emenda §6-bis), giudice ab-ce1b VINTO"); sys.exit(0)
else:
    print("ESITO: D POSITIVO oltre soglia (layout a favore) — guardia ok, si dichiara"); sys.exit(0)
PY
prc=$?
echo "$prc" > "$OUT/arbitrato-bt.rc"
exit "$prc"
} >> "$V" 2>&1
