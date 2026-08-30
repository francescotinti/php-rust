#!/bin/bash
# s165-orm-oracle-r5.sh — istruttoria DRIFT oracle ORM (criterio
# s165-istruttoria-ictx-orm.md §B2, PRE-registrato): R=5 gambe ORACLE-only
# in finestra quieta, stesso lancio della gamba oracle della coppia
# (ADATTAMENTO DICHIARATO oracle-only di s164-orm-coppia.sh: run_leg +
# rodaggio + quiescenza per gamba + floor med3; niente phpr, niente ratio).
# Verdetto: |mediana_R5(net) − ORA_REF 4.885| > 0.03 ⇒ RI-FONDAZIONE ORA_REF
# dichiarata alla mediana; altrimenti ORA_REF resta, banda [4.83;4.94] fa
# da sentinella. rc autoritativo = orm-out/oracle-r5.rc
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp165-harness"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
WD="/Volumes/Extreme Pro/Claude/wp13-harness/run-with-watchdog.sh"
ORACLE=/opt/homebrew/opt/php/bin/php
SP="${MAPPA_SP:?MAPPA_SP (workdir APFS) richiesto}"
OUT="$H/orm-out"; mkdir -p "$OUT"
VERD="$H/s165-orm-oracle-r5-verdetto.out"
RC="$OUT/oracle-r5.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE — TAG nuovo (az.rev. S-131 #5)" >&2; exit 7; }
LOCK=/private/tmp/phpr-measure.lock
grep -qi "s-165\|s165" "$LOCK" 2>/dev/null || { echo "lock assente/altrui" | tee -a "$VERD"; echo 9 > "$RC"; exit 9; }
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
quiesce_gate(){ local attempt q
  for attempt in 1 2 3; do
    "$QUIESCE" "$OUT/quiesce-$1.rc" > "$OUT/quiesce-$1.log" 2>&1; q=$?
    [ "$q" = 0 ] && return 0
    /bin/sleep 30
  done
  echo "quiescenza-fail su $1 (3 tentativi)" | tee -a "$VERD"; echo 8 > "$RC"; exit 8
}
ls_sentinel(){ pgrep -fl 'language[_-]server|Antigravity|rust-analyzer|intelephense|serena|gopls|pylsp|solargraph' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' '; }
floor3(){ local f1 f2 f3
  f1=$( { /usr/bin/time -l "$ORACLE" "$SP/orm-work/vendor/bin/phpunit" --version > /dev/null; } 2>&1 | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' )
  f2=$( { /usr/bin/time -l "$ORACLE" "$SP/orm-work/vendor/bin/phpunit" --version > /dev/null; } 2>&1 | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' )
  f3=$( { /usr/bin/time -l "$ORACLE" "$SP/orm-work/vendor/bin/phpunit" --version > /dev/null; } 2>&1 | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' )
  printf '%s\n%s\n%s\n' "$f1" "$f2" "$f3" | sort -n | awk 'NR==2'; }
run_leg(){ # LABEL
  rm -rf "$SP/orm-work"; tar xzf "$GATES/orm-work.tgz" -C "$SP" || return 7
  ( cd "$SP/orm-work" && "$WD" -t 3600 -s 600 -p "$OUT/r5-$1.txt" -o "$OUT" -- \
      /usr/bin/time -l "$ORACLE" -d memory_limit=-1 vendor/bin/phpunit --no-coverage > "$OUT/r5-$1.txt" 2> "$OUT/r5-$1.time" )
  return 0
}
{
echo "== s165 R=5 oracle-only ORM (criterio §B2 PRE-registrato; ORA_REF=4.885; soglia ri-fondazione |Δ|>0.03; banda sentinella [4.83;4.94]) =="
echo "sentinella LS inizio: $(ls_sentinel)"
run_leg rodaggio   # NON giudicante (transitorio primo-run)
FO=$(floor3)
echo "floor_user_med3 oracle=$FO (phpunit --version)"
for i in 1 2 3 4 5; do
  quiesce_gate "r5-$i"
  run_leg "leg$i"
  U=$(awk '/[0-9.]+ *user/{for(j=1;j<=NF;j++) if($j=="user"){print $(j-1)}}' "$OUT/r5-leg$i.time" | tail -1)
  S=$(LC_ALL=C tr -d '\0' < "$OUT/r5-leg$i.txt" | LC_ALL=C grep -aE "^(Tests:|OK)" | tail -1)
  echo "leg$i: user=$U summary=[$S]"
done
python3 - "$OUT" "$FO" <<'PY'
import sys, re
out, fo = sys.argv[1], float(sys.argv[2])
nets = []
for i in range(1, 6):
    u = None
    for l in open(f"{out}/r5-leg{i}.time", errors="replace"):
        m = re.search(r"([0-9.]+) *user", l)
        if m: u = float(m.group(1))
    nets.append(round(u - fo, 3))
s = sorted(nets); med = s[2]
print(f"nets={nets} mediana={med:.3f} ORA_REF=4.885 |delta|={abs(med-4.885):.3f}")
inband = all(4.83 <= x <= 4.94 for x in nets)
print(f"banda sentinella [4.83;4.94]: {'TUTTE dentro' if inband else 'gambe FUORI: ' + str([x for x in nets if not (4.83 <= x <= 4.94)])}")
if abs(med - 4.885) > 0.03:
    print(f"VERDETTO: RI-FONDAZIONE ORA_REF DOVUTA -> {med:.3f} (dichiarata a verbale, storia: s162 4.87/4.90 s163 4.85/4.84 s164 4.86/4.85)")
    sys.exit(3)
print("VERDETTO: ORA_REF=4.885 REGGE (delta <= 0.03); banda sentinella attiva dalla prossima coppia")
sys.exit(0)
PY
prc=$?
echo "sentinella LS fine: $(ls_sentinel)"
echo "$prc" > "$RC"
exit "$prc"
} >> "$VERD" 2>&1
