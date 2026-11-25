#!/bin/bash
# c4-lancio.sh — S-194 C4: (1) the battery of the proof-only panic arm of c4-copertura.sh, whole (--no-fail-fast: the copy lacks the
# harness trees some teeth read, those fail by name and are listed); (2) pin reading of the whole probe (pin s189 = the P4 code) as
# s194-primitive-C4base.out; (3) p-coda.sh C4 P4 with that base. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")/.." && pwd)"; BT="$HOME/Claude/phpr-target"
( cd "$BT/s194-srcX" && CARGO_TARGET_DIR="$BT/s194-armX" cargo test --profile dev-release --no-fail-fast > "$W/out/c4-copertura-batteria2.log" 2>&1
  echo "batteria2: rc=$? hits=$(tr -d '\0' < "$W/out/c4-copertura-batteria2.log" | grep -c 'panicked.*missed materialization\|missed materialization funnel$') falliti=$(tr -d '\0' < "$W/out/c4-copertura-batteria2.log" | grep -E '^test .* FAILED' | sort -u | tr '\n' ' ')" > "$W/out/c4-copertura-batteria2.done" )
bash "$W/s194-primitive.sh" C4base
S194_BASE=s194-primitive-C4base.out bash "$W/out/p-coda.sh" C4 P4
