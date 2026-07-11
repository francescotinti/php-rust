#!/bin/bash
# Measure phpr's internal-function coverage against the reference PHP 8.5.7
# oracle, grouped by extension. Prints a TSV report plus the headline rollups
# that README.md / COVERAGE.md publish.
#
# Usage: scripts/measure-coverage.sh [ORACLE_PHP] [PHPR_BIN]
#   ORACLE_PHP  reference interpreter (default: /opt/homebrew/opt/php/bin/php)
#   PHPR_BIN    phpr binary          (default: ~/Claude/php-rust-output/release/phpr)
#
# The corpus number is NOT measured here (it takes ~12 min); run separately:
#   phpt-runner --list-fails --isolate "/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests"
set -euo pipefail

ORACLE="${1:-/opt/homebrew/opt/php/bin/php}"
PHPR="${2:-$HOME/Claude/php-rust-output/release/phpr}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# 1. Oracle list: "ext<TAB>function" for every internal function.
"$ORACLE" -r 'foreach (get_defined_functions()["internal"] as $f) {
    $r = new ReflectionFunction($f);
    echo ($r->getExtensionName() ?: "core"), "\t", $f, "\n"; }' \
    | sort > "$WORK/oracle.tsv"

# 2. Probe each with function_exists() inside phpr.
"$ORACLE" -r '
$rows = array_map(fn($l) => explode("\t", $l), file($argv[1], FILE_IGNORE_NEW_LINES));
$out = fopen($argv[2], "w");
fwrite($out, "<?php\n\$fns = [\n");
foreach ($rows as [$ext, $fn]) { fwrite($out, "[\"$ext\", \"$fn\"],\n"); }
fwrite($out, "];\nforeach (\$fns as \$p) { echo \$p[0], \"\\t\", \$p[1], \"\\t\", function_exists(\$p[1]) ? 1 : 0, \"\\n\"; }\n");
' "$WORK/oracle.tsv" "$WORK/probe.php"
"$PHPR" "$WORK/probe.php" > "$WORK/result.tsv"

# 3. Tally per extension + the published rollups.
awk -F'\t' '
{ tot[$1]++; T++; if ($3 == 1) { have[$1]++; H++ } }
END {
    printf "TOTAL\t%d/%d\t%d%%\n", H, T, int(100 * H / T + 0.5)
    core_h = have["standard"] + have["Core"] + have["date"]
    core_t = tot["standard"] + tot["Core"] + tot["date"]
    printf "CORE-STDLIB (standard+Core+date)\t%d/%d\t%d%%\n", core_h, core_t, int(100 * core_h / core_t + 0.5)
    print "---"
    for (e in tot) printf "%s\t%d/%d\t%d%%\n", e, have[e], tot[e], int(100 * (have[e] + 0) / tot[e] + 0.5)
}' "$WORK/result.tsv" | { IFS= read -r l1; echo "$l1"; IFS= read -r l2; echo "$l2"; IFS= read -r sep; echo "$sep"; sort -t/ -k1; }
