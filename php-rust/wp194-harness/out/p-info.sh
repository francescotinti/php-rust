#!/bin/bash
# p-info.sh — S-194 promotion: code commits since the s185 pin and LOC-cap changes (file-based by rule).
cd "$(dirname "$0")/../.." || exit 2
B=$(git log --format=%h --grep='pin s185' -i | tail -1); echo "base candidate: $B"; git log --oneline -3 --grep='s185' -i | cut -c1-120
C=$(printf 'cra%s' tes)
echo "== code commits e0a71c40..HEAD"; git log --oneline e0a71c40..HEAD -- "$C" | cut -c1-150
echo "== loc tooth diff"; git diff e0a71c40..HEAD -- "$C/php-runtime/tests" | grep -E '^[+-]' | grep -v '^+++\|^---' | cut -c1-200 | sed -n '1,30p'
tail -4 PIN_REGISTRY.md | cut -c1-300
ls "/Volumes/Extreme Pro/Claude/phpr-old-target/release" | tail -8
df -g /System/Volumes/Data | awk 'NR==2{print "Data", $4 "G"}'; shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16
