#!/bin/bash
# gate-slope-verdict.sh — KG-86-1 (Council WP-86, Gregg Q4): the machine
# that makes every FUTURE slope verdict born fail-closed. A slope claim is
# legal ONLY if this gate PASSes on (verdict-out, raw-dir); without it the
# verdict is NULL d'ufficio. Requirements enforced:
#   (a) A-BG33: the campaign raws contain PER-REQUEST base samples from the
#       driver (`curl -w %{time_total}` lines) — a slope without a
#       per-request base is a ratio over an inferred denominator;
#   (b) A-BG35/KG-86-1: the verdict body defines its regime — a literal
#       `regime=` string in the verdict output;
#   (c) A-PP31/KS-PP-86-1: every `reqns: ` sample line in the raws carries
#       `w=1` in-band (the rejection is re-run HERE, independent of the
#       verdict's own guard — two teeth, one contract);
#   (d) A-BB43/KB-86-3: the verdict names the binary the slope was measured
#       on (a literal `bin=` or `binario` token) — slope figures are
#       binary-bound, transfers need a smoke-slope.
# Usage: gate-slope-verdict.sh <verdict.out> <raw-dir-or-glob-dir>
#        gate-slope-verdict.sh --selftest
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u

check() { # <verdict.out> <rawdir> -> prints failures, returns count
  local out="$1" rawdir="$2" fails=0
  [ -f "$out" ] || { echo "FAIL: verdict output missing: $out"; return 1; }
  [ -d "$rawdir" ] || { echo "FAIL: raw dir missing: $rawdir"; return 1; }
  if ! grep -q "regime=" "$out"; then
    echo "FAIL: no 'regime=' definition in the verdict (A-BG35/KG-86-1)"
    fails=$((fails+1))
  fi
  if ! grep -qE "bin=|binario" "$out"; then
    echo "FAIL: verdict does not name the measured binary (A-BB43/KB-86-3)"
    fails=$((fails+1))
  fi
  if ! grep -rl "time_total" "$rawdir" 2>/dev/null | head -1 | grep -q .; then
    echo "FAIL: no per-request base samples (curl time_total) in raws (A-BG33)"
    fails=$((fails+1))
  fi
  local bad
  bad=$(grep -rh "^reqns: " "$rawdir" 2>/dev/null | grep -cv " w=1 " || true)
  if [ "${bad:-0}" -gt 0 ]; then
    echo "FAIL: $bad reqns sample line(s) without w=1 in-band (A-PP31/KS-PP-86-1)"
    fails=$((fails+1))
  fi
  return $fails
}

if [ "${1:-}" = "--selftest" ]; then
  T=$(mktemp -d)
  mkdir -p "$T/raws"
  # compliant pair
  printf 'slope lever=1 regime=steady-100 bin=deadbeef\n' > "$T/v.out"
  printf 'reqns: 5 w=1 arm=x\ntime_total 0.004\n' > "$T/raws/r1.log"
  if ! check "$T/v.out" "$T/raws" >/dev/null; then
    echo "SELFTEST FAIL: compliant slope verdict rejected (KG-86-1)"; rm -rf "$T"; exit 1
  fi
  # each tooth must bite
  printf 'slope lever=1 bin=deadbeef\n' > "$T/v2.out"
  check "$T/v2.out" "$T/raws" >/dev/null && { echo "SELFTEST FAIL: missing regime= not caught"; rm -rf "$T"; exit 1; }
  printf 'reqns: 5 arm=x\ntime_total 0.004\n' > "$T/raws/r1.log"
  check "$T/v.out" "$T/raws" >/dev/null && { echo "SELFTEST FAIL: unlabeled reqns not caught"; rm -rf "$T"; exit 1; }
  printf 'reqns: 5 w=1 arm=x\n' > "$T/raws/r1.log"
  check "$T/v.out" "$T/raws" >/dev/null && { echo "SELFTEST FAIL: missing time_total base not caught"; rm -rf "$T"; exit 1; }
  rm -rf "$T"
  echo "SELFTEST PASS: all four slope-verdict teeth bite (KG-86-1)"
  exit 0
fi

[ $# -eq 2 ] || { echo "usage: $0 <verdict.out> <raw-dir> | --selftest"; exit 2; }
if check "$1" "$2"; then
  echo "PASS gate-slope-verdict (KG-86-1): regime defined, binary named, per-request base present, reqns labeled"
  exit 0
else
  echo "== SLOPE VERDICT NULL d'ufficio (KG-86-1) =="
  exit 1
fi
