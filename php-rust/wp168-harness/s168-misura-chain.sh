#!/bin/bash
# s168-misura-chain.sh — finestra di misura S-168 in SEQUENZA (run pesanti
# sequenziali e detached, REGOLE §10): (1) braccio pin-phpr.sh --braccio per
# m1/m2/m3/m123 (smoke parità + stash + registro; m0 già fatto); (2) A/B
# s168-ab-mock.sh: m0 vs pin-stash (braccio nullo), poi m1/m2/m3/m123 vs m0;
# (3) s168-stride.sh; (4) s168-xctrace.sh. Esiti SOLO da file: ab-out/*.rc;
# questo copione scrive ab-out/chain.log e ab-out/chain.done. STOP solo su
# rc di apparato (1 pin, 2 output diverge, 7 file, 9 lock); i verdetti (0/4/5/8)
# si registrano e si prosegue.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"; R="$H/.."; OUT="$H/ab-out"
LOG="$OUT/chain.log"; : > "$LOG"; rm -f "$OUT/chain.done"
BIN=/private/tmp/s168-mock-bin; STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
h8(){ shasum -a 256 "$1" | cut -c1-8; }
say(){ echo "$(date '+%T') $*" >> "$LOG"; }
for t in m1 m2 m3 m123; do
  [ "$(cat "$OUT/build-$t.rc" 2>/dev/null)" = 0 ] || { say "STOP: build $t non verde"; touch "$OUT/chain.done"; exit 7; }
  if [ -e "$STASH/phpr-s168-$t" ]; then say "braccio $t già a stash"; else
    "$R/scripts/pin-phpr.sh" --braccio "s168-$t" "$BIN/phpr-$t" >> "$LOG" 2>&1 || { say "STOP: braccio $t fallito"; touch "$OUT/chain.done"; exit 1; }
  fi
done
M0="$BIN/phpr-m0"; PIN="$STASH/phpr-s166"
# criterio p.5: disasm bl run_loop A/B per ogni braccio (pin vs m0, m0 vs mN)
"$R/wp160-harness/s160-disasm-bl.sh" "$PIN" "$M0" "$OUT/disasm-m0.out" >> "$LOG" 2>&1
for t in m1 m2 m3 m123; do "$R/wp160-harness/s160-disasm-bl.sh" "$M0" "$BIN/phpr-$t" "$OUT/disasm-$t.out" >> "$LOG" 2>&1; done
run_ab(){ # <A> <B> <tag>
  "$H/s168-ab-mock.sh" "$1" "$(h8 "$1")" "$2" "$(h8 "$2")" "$3"; rc=$?
  say "ab $3 rc=$rc"; case "$rc" in 1|2|7|9) say "STOP apparato"; touch "$OUT/chain.done"; exit "$rc";; esac
}
run_ab "$PIN" "$M0" m0
for t in m1 m2 m3 m123; do run_ab "$M0" "$BIN/phpr-$t" "$t"; done
"$H/s168-stride.sh"; say "stride rc=$?"
"$H/s168-xctrace.sh"; say "xctrace rc=$?"
touch "$OUT/chain.done"; say "fine"
