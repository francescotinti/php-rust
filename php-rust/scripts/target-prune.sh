#!/bin/bash
# target-prune.sh <pin_phpr16> <pin_server16> — potatura della build canonica
# (decisione utente 2026-08-13: il disco locale non può liberare altro).
# TIENE solo i 3 binari pinnati in release/ (phpr, php-server, phpt-runner:
# ~43MB — il pre-flight di parità continua a leggerli lì); POTA deps/build/
# .fingerprint/examples/incremental (~4,1G). Le build successive sono a freddo
# (~+3 min l'una; la ricetta SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 riproduce
# gli hash al byte — provato dalle catene di promozione e dalla CI).
# COLLAUDO-NELL'ATTO: prima verifica che i binari == pin dichiarati (o li
# RIPRISTINA dallo stash phpr-old-target), pota, ri-verifica gli hash, stampa
# lo spazio liberato. Qualunque discordanza => STOP senza potare.
set -u
export PATH=/usr/bin:/bin:/usr/sbin
T="$HOME/Claude/php-rust-output"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
PIN="${1:?uso: target-prune.sh <pin_phpr16> <pin_server16> [tag-stash]}"
SRV="${2:?pin_server16}"
TAG="${3:-}"
h16(){ shasum -a 256 "$1" 2>/dev/null | cut -c1-16; }

# runner del corpus: nessun pin proprio dichiarato in NEXT_SESSION — si
# conserva quello presente (o lo stash piu' recente) e se ne stampa l'hash.
for b in phpr php-server phpt-runner; do
  [ -f "$T/release/$b" ] || { echo "manca $b in release/ — provo lo stash"; }
done
if [ "$(h16 "$T/release/phpr")" != "$PIN" ]; then
  echo "phpr != pin $PIN — ripristino dallo stash"
  { [ -n "$TAG" ] && cp "$STASH/phpr-$TAG" "$T/release/phpr" 2>/dev/null; } \
    || { echo "STOP: phpr non ripristinabile (passa il tag stash come 3o arg)"; exit 1; }
fi
if [ "$(h16 "$T/release/php-server")" != "$SRV" ]; then
  echo "php-server != pin $SRV — STOP (rilinka con pin-server.sh prima di potare)"; exit 1
fi

BEFORE=$(df -g "$HOME" | awk 'NR==2{print $4}')
mkdir -p "$T/keep-release"
for b in phpr php-server phpt-runner; do
  cp "$T/release/$b" "$T/keep-release/$b" || { echo "STOP: copia $b fallita"; exit 1; }
done
rm -rf "$T/release" "$T/flycheck0" "$T/tmp"
mkdir -p "$T/release"
for b in phpr php-server phpt-runner; do
  mv "$T/keep-release/$b" "$T/release/$b"
  chmod +x "$T/release/$b"
done
rmdir "$T/keep-release"

P2=$(h16 "$T/release/phpr"); S2=$(h16 "$T/release/php-server")
[ "$P2" = "$PIN" ] && [ "$S2" = "$SRV" ] || { echo "STOP: hash post-potatura divergono (phpr=$P2 server=$S2)"; exit 1; }
AFTER=$(df -g "$HOME" | awk 'NR==2{print $4}')
echo "TARGET-PRUNE OK: phpr=$P2 server=$S2 runner=$(h16 "$T/release/phpt-runner") liberi ${BEFORE}G->${AFTER}G"
