#!/bin/bash
# s175-catena.sh — catena post-promozione S-175 (ordine NEXT §S-175 p.1-3a), detached via daemonize.pl:
# 1) attende promo-out/rcb (promozione C): se != 0 si ferma (rc=1);
# 2) inserisce l'hash MISURATO del server pinnato al posto del PLACEHOLDER in s175-pair.sh (DICHIARAZIONE in coda
#    a s175-criterio-pair-t20.md) e committa (solo quei due file);
# 3) esegue s175-mutante-gc.sh (build MS+M3 su Extreme Pro, ~10 min: PRIMA della coppia, mai durante);
# 4) daemonizza s175-lancio-pair.sh (quiete CI + anti-flare → pair t20), s175-lancio-orm.sh (attende pair174-t20.done)
#    e s175-lancio-mock.sh (attende orm-out/rimisura.done). Esiti: promo-out/rcb, ab-out/s175-mutgc.done,
#    pair-out/pair174-t20.done, orm-out/rimisura.done, ab-out/mock.rc. Log: ab-out/catena.log.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment"
H="$REPO/php-rust/wp174-harness"
DZ="/Volumes/Extreme Pro/Claude/wp54-harness/daemonize.pl"
LOG="$H/ab-out/catena.log"; mkdir -p "$H/ab-out"
l(){ echo "$(date '+%F %T') $*" >> "$LOG"; }
l "attesa promo-out/rcb"
while [ ! -e "$H/promo-out/rcb" ]; do sleep 60; done
RCB=$(cat "$H/promo-out/rcb")
[ "$RCB" = 0 ] || { l "promozione rcb=$RCB ≠ 0 — catena FERMA"; exit 1; }
SRV=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/php-server" | cut -c1-16)
PIN=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)
l "promozione rc=0: pin phpr $PIN server $SRV"
[ "$PIN" = "5de14d6856d760a8" ] || { l "pin phpr $PIN ≠ 5de14d6856d760a8 — catena FERMA"; exit 1; }
grep -q 'SRV_ATTESO="SRV_S175_PLACEHOLDER"' "$H/s175-pair.sh" || { l "placeholder server assente in s175-pair.sh — catena FERMA"; exit 1; }
sed -i '' "s/SRV_ATTESO=\"SRV_S175_PLACEHOLDER\"/SRV_ATTESO=\"$SRV\"/" "$H/s175-pair.sh"
printf '\nDICHIARAZIONE (S-175, dopo la promozione rc=0 — %s): pin s175 = phpr %s + server %s inseriti in s175-pair.sh (PIN_ATTESO/SRV_ATTESO) prima del lancio t20.\n' "$(date '+%F %T')" "$PIN" "$SRV" >> "$H/s175-criterio-pair-t20.md"
( cd "$REPO" && git add php-rust/wp174-harness/s175-pair.sh php-rust/wp174-harness/s175-criterio-pair-t20.md \
  && printf 'S-175 coppia t20: server pinnato %s inserito in s175-pair.sh (dichiarazione nel criterio) — catena post-promozione\n\nCo-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>\n' "$SRV" > "$H/ab-out/catena-msg.txt" \
  && git commit -q -F "$H/ab-out/catena-msg.txt" && git push -q ) >> "$LOG" 2>&1 && l "commit server hash OK" || l "commit server hash FALLITO (dichiarato; la coppia parte comunque)"
l "mutante gc: lancio (build MS+M3)"
/bin/bash "$H/s175-mutante-gc.sh" >> "$LOG" 2>&1
l "mutante gc: $(cat "$H/ab-out/s175-mutgc.done" 2>/dev/null)"
sleep 30
mkdir -p "$H/pair-out" "$H/orm-out"
/usr/bin/perl "$DZ" "$H/pair-out/lancio-pair-daemon.log" /bin/bash "$H/s175-lancio-pair.sh"
/usr/bin/perl "$DZ" "$H/orm-out/lancio-orm-daemon.log" /bin/bash "$H/s175-lancio-orm.sh"
/usr/bin/perl "$DZ" "$H/ab-out/lancio-mock-daemon.log" /bin/bash "$H/s175-lancio-mock.sh"
l "lanciatori pair/orm/mock daemonizzati — catena conclusa"
exit 0
