#!/bin/bash
# s160-istr-phpr1.sh — istruttoria gamba phpr1 ictx (S-160 p.1; az.rev. S-159 #4).
# grade=ISTRUTTORIA (non di record): 3 gambe phpr ORM consecutive, untar fresco
# prima di OGNI gamba (come s159-orm-coppia.sh), sonda daemon prima/dopo ogni
# gamba + probe idle non bloccante. Esiti in istr-out/, rc autoritativo in
# istr-out/istr.done. Pin atteso s159 f2d17f18c00a4049.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp160-harness"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
WD="/Volumes/Extreme Pro/Claude/wp13-harness/run-with-watchdog.sh"
PHPR="$HOME/Claude/php-rust-output/release/phpr"
SP=/private/tmp/phpr-s160-istr; mkdir -p "$SP"
OUT="$H/istr-out"; mkdir -p "$OUT"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
: > "$OUT/progress.txt"
rm -f "$OUT/istr.done"
PINM="$(shasum -a 256 "$PHPR" | cut -c1-16)"
[ "$PINM" = "f2d17f18c00a4049" ] || { echo "rc=9 pin!=s159" > "$OUT/istr.done"; exit 9; }
LOCK=/private/tmp/phpr-measure.lock
[ -e "$LOCK" ] || { echo "rc=6 measure-lock ASSENTE" > "$OUT/istr.done"; exit 6; }

# sonda daemon: cputime cumulato (s) dei daemon indiziati, per NOME
daemons(){ ps -Ao cputime=,comm= | awk '
  function tosec(t,  a,n,s){ n=split(t,a,":"); s=0; for(i=1;i<=n;i++) s=s*60+a[i]; return s }
  /mds$|mds_stores|mdworker|fseventsd|deleted$|backupd|mediaanalysisd|corespotlightd|photoanalysisd/ {
    n=$2; sub(/.*\//,"",n); tot[n]+=tosec($1) }
  END { for(k in tot) printf "%s=%.0f ", k, tot[k]; print "" }'; }

idle_probe(){ top -l 2 -n 0 -s 2 2>/dev/null | grep -a "CPU usage" | tail -1; }

for leg in 1 2 3; do
  rm -rf "$SP/orm-work"; tar xzf "$GATES/orm-work.tgz" -C "$SP" || { echo "rc=8 untar leg$leg" > "$OUT/istr.done"; exit 8; }
  echo "leg$leg PRE  $(daemons)" >> "$OUT/daemons.txt"
  echo "leg$leg IDLE $(idle_probe)" >> "$OUT/daemons.txt"
  p "leg$leg START"
  ( cd "$SP/orm-work" && "$WD" -t 3600 -s 600 -p "$OUT/leg$leg.txt" -o "$OUT" -- \
      /usr/bin/time -l "$PHPR" vendor/bin/phpunit --no-coverage > "$OUT/leg$leg.txt" 2> "$OUT/leg$leg.time" )
  p "leg$leg rc=$?"
  echo "leg$leg POST $(daemons)" >> "$OUT/daemons.txt"
done

{ echo "== s160 istruttoria phpr1 (grade=ISTRUTTORIA; pin MISURATO $PINM; 3 gambe phpr ORM, untar fresco per gamba) =="
  for leg in 1 2 3; do
    U=$(tr -d '\0' < "$OUT/leg$leg.time" | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}')
    R=$(tr -d '\0' < "$OUT/leg$leg.time" | awk '/[0-9.]+ *real/{for(i=1;i<=NF;i++) if($i=="real"){print $(i-1)}}')
    IC=$(tr -d '\0' < "$OUT/leg$leg.time" | awk '/involuntary/{print $1}')
    PR=$(tr -d '\0' < "$OUT/leg$leg.time" | awk '/page reclaims/{print $1}')
    IN=$(tr -d '\0' < "$OUT/leg$leg.time" | awk '/instructions/{print $1}')
    CY=$(tr -d '\0' < "$OUT/leg$leg.time" | awk '/cycles/{print $1}')
    echo "leg$leg: user=$U real=$R ictx=$IC ictx_s=$(python3 -c "print(f'{$IC/$R:.1f}')" 2>/dev/null) reclaims=$PR instr=$IN cycles=$CY"
  done
  echo "-- daemon deltas (cputime cumulato, s) --"
  cat "$OUT/daemons.txt"
} > "$H/s160-istr-verdetto.out"
echo "rc=0 $(date +%T)" > "$OUT/istr.done"
p "DONE rc=0"
