#!/bin/bash
# s121-abab-wp.sh — az. rev. S-120 #4: gamba ABAB s119<->s120 STESSA SERA per
# ripartire L-RE1 su WP (tra-pin, al netto del tra-sere). 4 gambe FULL
# intercalate A1->B1->A2->B2: A = stash phpr-s119 (pin s119 350582e5), B =
# stash phpr-s120-re1 (pin s120 885d2c64, = s119 + L-RE1). Modo ON esplicito
# su entrambe (il default spedito); ricetta pair109 per gamba (reset DB +
# uploads azzerati PRIMA di ogni run, /usr/bin/time -l, MIMALLOC_PURGE_DELAY=0,
# uploads sotto guardia). NIENTE oracle: attribuzione pin-to-pin DIRETTA in
# secondi CPU (user+sys) — N=2 per pin = direzione + prima magnitudine.
# Parita': failnames per NOME devono coincidere su TUTTE e 4 le gambe.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp121-harness"
WPDEV="/Users/francescotinti/Claude/wpdev"
BIN_A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s119"
BIN_B="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s120-re1"
GUARD="/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
OUT="$H/abab-out"; mkdir -p "$OUT"
VERD="$H/s121-abab-verdetto.out"
step(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
: > "$OUT/progress.txt"

HA=$(shasum -a 256 "$BIN_A" | cut -c1-16); HB=$(shasum -a 256 "$BIN_B" | cut -c1-16)
if [ "$HA" != "350582e55d21533d" ] || [ "$HB" != "885d2c646ac7ff4c" ]; then
  echo "REFUSE: stash A=$HA (atteso 350582e5...) B=$HB (atteso 885d2c64...)" | tee -a "$OUT/progress.txt"
  echo "rc=65" > "$OUT/abab.done"; exit 65
fi
{
  echo "A=phpr-s119 $HA  B=phpr-s120-re1 $HB  modo=ON esplicito  ordine=A1,B1,A2,B2"
  echo "epoch=$(date +%s)"
} > "$OUT/identity.txt"

"$GUARD" backup-wipe >> "$OUT/progress.txt" 2>&1 || { echo "rc=4 guard-backup" > "$OUT/abab.done"; exit 4; }
restore_uploads(){ "$GUARD" restore >> "$OUT/progress.txt" 2>&1; }
trap restore_uploads EXIT
cd "$WPDEV" || exit 2

reset_env() {
  mysql -h 127.0.0.1 -u root -e "DROP DATABASE IF EXISTS wptests; CREATE DATABASE wptests; GRANT ALL ON wptests.* TO 'wp'@'%';" || exit 3
  find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
}
names() { sed -n 's/^[0-9][0-9]*) \(.*\)$/\1/p' "$1" | sort; }

run_leg(){ # $1=lato(A|B) $2=numero
  local bin; [ "$1" = A ] && bin="$BIN_A" || bin="$BIN_B"
  step "gamba $1$2 START ($(basename "$bin"))"
  reset_env
  MIMALLOC_PURGE_DELAY=0 PHPR_REG_LOWER=1 /usr/bin/time -l "$bin" vendor/bin/phpunit \
    > "$OUT/full-$1$2.txt" 2> "$OUT/full-$1$2.time"
  step "gamba $1$2 rc=$?"
  names "$OUT/full-$1$2.txt" > "$OUT/full-$1$2.failnames"
}

run_leg A 1; run_leg B 1; run_leg A 2; run_leg B 2

trap - EXIT
restore_uploads

PAR=0
for l in B1 A2 B2; do
  if ! diff -q "$OUT/full-A1.failnames" "$OUT/full-$l.failnames" > /dev/null; then
    step "PARITA' VIOLATA: failnames A1 vs $l DIVERSI"; PAR=1
  fi
done

python3 - "$OUT" > "$OUT/abab-riduzione.txt" <<'PY'
import sys, re
out = sys.argv[1]
def t(leg):
    v = {}
    for line in open(f"{out}/full-{leg}.time"):
        m = re.search(r"([\d.]+)\s+user", line);  v['u'] = float(m.group(1)) if m else v.get('u')
        m = re.search(r"([\d.]+)\s+sys", line);   v['s'] = float(m.group(1)) if m else v.get('s')
        m = re.match(r"\s*(\d+)\s+peak memory footprint", line)
        if m: v['pf'] = int(m.group(1))
    return v['u'] + v['s'], v.get('pf', 0) / 1048576
cpu, pk = {}, {}
for leg in ("A1", "B1", "A2", "B2"):
    cpu[leg], pk[leg] = t(leg)
    print(f"full_{leg}_cpu={cpu[leg]:.2f}s peak={pk[leg]:.1f}MiB")
d1, d2 = cpu["A1"] - cpu["B1"], cpu["A2"] - cpu["B2"]
sa = abs(cpu["A1"] - cpu["A2"]); sb = abs(cpu["B1"] - cpu["B2"])
print(f"D_coppia (A-B, + = s120 piu' veloce): D1={d1:+.2f}s D2={d2:+.2f}s")
print(f"spread intra-pin stessa sera: A={sa:.2f}s B={sb:.2f}s")
segno = "CONCORDE" if (d1 > 0) == (d2 > 0) else "DISCORDE"
print(f"L-RE1 su WP (tra-pin, stessa sera, N=2): segno {segno}; magnitudine mediana={(d1+d2)/2:+.2f}s"
      f" — {'sopra' if min(abs(d1),abs(d2)) > max(sa,sb) else 'DENTRO'} lo spread intra-pin (N=2 = direzione, non dimostrazione)")
PY
rrc=$?
cat "$OUT/abab-riduzione.txt"
{
  echo "s121-abab-verdetto — L-RE1 ripartita su WP (az. rev. S-120 #4; $(head -1 "$OUT/identity.txt"))"
  echo "parita' failnames 4 gambe: $([ "$PAR" = 0 ] && echo IDENTICI || echo 'VIOLATA (vedi progress)')"
  cat "$OUT/abab-riduzione.txt"
} >> "$VERD"
RC=$(( (rrc != 0) + PAR ))
echo "rc=$RC" > "$OUT/abab.done"
step "DONE rc=$RC"
exit "$RC"
