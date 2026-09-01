#!/bin/bash
# s169-e2-check.sh — az.rev.2 S-169 (REGOLE §3: il criterio emendato si RIESEGUE):
# colonna e2 di m5/m7 rimisurata (A=m0 stash, B=m5/m7 stash), R=5 interleaved,
# floor per-binario, output CONFRONTATO CON L'ATTESO (oracle) — non solo A==B;
# dump del loop nudo sotto m5 (nessun Nop atteso: E2 non ha statement nel corpo).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"; S="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
A="$S/phpr-s168-m0"; E2="$H/../wp168-harness/arith-e2.php"; EMPTY="$H/../wp164-harness/empty.php"; O=/opt/homebrew/opt/php/bin/php
OUT="$H/ab-out"; VERD="$H/s169-e2-verdetto.out"; RC="$OUT/e2check.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE" >&2; exit 7; }
grep -q "S-169" /private/tmp/phpr-measure.lock 2>/dev/null || { echo "lock S-169 assente" | tee -a "$VERD"; echo 9 > "$RC"; exit 9; }
"$H/../wp129-harness/s129-quiescenza.sh" "$OUT/quiesce-e2.rc" > /dev/null 2>&1 || { echo "quiescenza FAIL" | tee -a "$VERD"; echo 8 > "$RC"; exit 8; }
ucpu(){ { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$@" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3(){ local a b c; a=$(ucpu "$1" "$EMPTY"); b=$(ucpu "$1" "$EMPTY"); c=$(ucpu "$1" "$EMPTY"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }
{
EXP=$("$O" "$E2")
echo "== s169 e2-check — A=$(shasum -a 256 "$A" | cut -c1-8) (m0) vs m5/m7 dagli stash; R=5; output confrontato con l'ATTESO oracle ('$EXP') =="
for t in m5 m7; do B="$S/phpr-s169-$t"; GOT=$("$B" "$E2" 2>&1); [ "$GOT" = "$EXP" ] || { echo "$t: output '$GOT' != atteso '$EXP' — STOP"; echo 2 > "$RC"; exit 2; }; done
echo "loop nudo sotto m5: $(PHPR_DUMP_OPS=1 "$S/phpr-s169-m5" "$E2" 2>&1 | sed -n '/^-- {main}/,/Ret/p' | sed -n '/CmpJmpSC/,/IncDecSlotJmp/p' | awk '{print $2}' | tr '\n' ' ')"
FA=$(floor3 "$A"); F5=$(floor3 "$S/phpr-s169-m5"); F7=$(floor3 "$S/phpr-s169-m7"); echo "floors: A=$FA m5=$F5 m7=$F7"
TSV="$OUT/e2check.tsv"; : > "$TSV"
for i in 1 2 3 4 5; do TA=$(ucpu "$A" "$E2"); T5=$(ucpu "$S/phpr-s169-m5" "$E2"); T7=$(ucpu "$S/phpr-s169-m7" "$E2"); printf '%s\t%s\t%s\n' "$TA" "$T5" "$T7" >> "$TSV"; echo "  giro$i: A=$TA m5=$T5 m7=$T7"; done
python3 - "$TSV" "$FA" "$F5" "$F7" <<'PY'
import sys
rows=[l.split() for l in open(sys.argv[1])]; f=list(map(float,sys.argv[2:5])); N=250e6
def col(i): return sorted((float(r[i])-f[i])/N*1e9 for r in rows)
def dr1(v): m=v[2]; w=sorted(v,key=lambda x:(abs(x-m),x))[:-1]; return max(w)-min(w)
a,m5,m7=col(0),col(1),col(2)
for n,v in (("m5",m5),("m7",m7)):
    d=v[2]-a[2]; g=max(4.0,dr1(a),dr1(v)); print(f"e2 {n}: A={a[2]:.2f} B={v[2]:.2f} |D|={abs(d):.2f} ≤ {g:.2f} -> {'guardia ok' if abs(d)<=g else 'MORDE'} (rumore A'={dr1(a):.2f} B'={dr1(v):.2f})")
print(f"e2_A (per il conto di m5) = {a[2]:.2f} ns/iter MISURATO in questo run")
PY
echo 0 > "$RC"
} >> "$VERD" 2>&1; cat "$VERD"
