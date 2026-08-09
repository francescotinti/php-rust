#!/bin/bash
# s123-gen-scaled.sh — criterio s123-criterio-metro.md p.2: genera i micro
# scalati (run bersaglio >=5 s) in scaled/, con verifica fail-closed del bound.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
M="$H/../wp97-harness/micro"
SC="$H/scaled"; mkdir -p "$SC"

sed 's/\$i<50000000/\$i<150000000/' "$M/arith.php" > "$SC/arith.php"
sed 's/\$i<30000000/\$i<90000000/'  "$M/prop.php"  > "$SC/prop.php"
sed 's/\$i<20000000/\$i<60000000/'  "$M/calls.php" > "$SC/calls.php"
sed 's/\$i<4000000/\$i<28000000/'   "$M/str.php"   > "$SC/str.php"
sed 's/\$r<60/\$r<300/'             "$M/arr.php"   > "$SC/arr.php"
sed 's/\$i<2000000/\$i<12000000/'   "$M/re.php"    > "$SC/re.php"
cp "$M/empty.php" "$SC/empty.php"

rc=0
grep -q 'i<150000000' "$SC/arith.php" || { echo "arith bound MANCATO"; rc=1; }
grep -q 'i<90000000'  "$SC/prop.php"  || { echo "prop bound MANCATO";  rc=1; }
grep -q 'i<60000000'  "$SC/calls.php" || { echo "calls bound MANCATO"; rc=1; }
grep -q 'i<28000000'  "$SC/str.php"   || { echo "str bound MANCATO";   rc=1; }
grep -q 'r<300'       "$SC/arr.php"   || { echo "arr bound MANCATO";   rc=1; }
grep -q 'i<12000000'  "$SC/re.php"    || { echo "re bound MANCATO";    rc=1; }
echo "$rc" > "$H/metro-out/gen.rc" 2>/dev/null || { mkdir -p "$H/metro-out"; echo "$rc" > "$H/metro-out/gen.rc"; }
[ "$rc" = 0 ] && echo "scaled OK (arith 150M prop 90M calls 60M str 28M arr r300=30M re 12M)"
exit "$rc"
