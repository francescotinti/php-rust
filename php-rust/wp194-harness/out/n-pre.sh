#!/bin/bash
# n-pre.sh — S-194 night pre-flight (file-based by rule): stashes, twin, MySQL, bundle, port, disk.
S="/Volumes/Extreme Pro/Claude/phpr-old-target/release"; R="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
for b in phpr-s183 phpr-s185 phpr-s186; do echo "$b $(shasum -a 256 "$S/$b" | cut -c1-16)"; done
echo "canonica $(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16) server $(shasum -a 256 "$HOME/Claude/php-rust-output/release/php-server" | cut -c1-16)"
echo "Z $(shasum -a 256 "$R/wp186-harness/ab-out/s186-leva/phpr-Z" | cut -c1-16)"
ls "$R/wp186-harness/s186-guardia-verdetto-b6.out" 2>&1 | tail -1
mysql --socket=/private/tmp/mysql-wp8.sock -uroot -e "SHOW DATABASES" 2>&1 | tr '\n' ' '; echo
/sbin/mount | grep -c phpr-target; lsof -nP -iTCP:8080 -sTCP:LISTEN 2>/dev/null | wc -l
df -g /System/Volumes/Data | awk 'NR==2{print "Data", $4 "G"}'; ls /private/tmp/phpr-measure.lock 2>&1 | tail -1
tail -3 "/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"; ls "/Volumes/Extreme Pro/Claude/phpr-ci/queue" 2>/dev/null | wc -l
