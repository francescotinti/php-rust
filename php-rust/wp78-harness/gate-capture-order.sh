#!/bin/bash
# KS-PP-2 grep-gate (A-PP2/A-TH1, Council WP-78): in the php-server crate no
# function may read `.rendered` / `.stdout` lexically AFTER calling
# `request_end()`. The permanent rule (Pedersen/Stogov, affirmed 9/9) says the
# reset boundary clears those buffers: a post-reset capture reads zeroes.
# Violation anywhere → reject (the boundary discipline closes over ALL verbs,
# not per-fix — WP-70 lesson).
#
# Self-test: the detector is first run against a known-bad snippet and MUST
# flag it (WP-72 lesson: an all-zero counter is a failed positive control).
export PATH=/usr/bin:/bin:/usr/sbin:$PATH
set -u
REPO="$(cd "$(dirname "$0")/.." && pwd)"

detect() {
  # awk: reset the seen-flag at each fn definition; after a request_end() line,
  # any .rendered/.stdout read within the same fn is a violation.
  awk '
    /^[[:space:]]*\/\// { next }          # comments do not arm nor violate
    /fn [A-Za-z0-9_]+ *(<[^>]*>)? *\(/ { seen = 0 }
    /request_end\(\)/ { seen = 1; next }
    seen && /\.(rendered|stdout)/ {
      printf "%s:%d: capture after request_end(): %s\n", FILENAME, FNR, $0
      v = 1
    }
    END { exit v }
  ' "$@"
}

# --- positive control -------------------------------------------------------
SELFTEST=$(mktemp -t ksppa2XXXX).rs
cat > "$SELFTEST" <<'EOF'
fn bad() {
    vm.request_end();
    let out = vm.rendered.clone();
}
EOF
if detect "$SELFTEST" > /dev/null 2>&1; then
  echo "SELF-TEST BROKEN: detector did not flag the known-bad snippet"
  rm -f "$SELFTEST"; exit 2
fi
rm -f "$SELFTEST"

# --- real scan --------------------------------------------------------------
FAIL=0
for f in "$REPO"/crates/php-server/src/*.rs; do
  out=$(detect "$f")
  rc=$?
  [ -n "$out" ] && echo "$out"
  [ $rc -ne 0 ] && FAIL=1
done

if [ "$FAIL" = 0 ]; then
  echo "== KS-PP-2 grep-gate PASS (no capture after request_end in php-server) =="
  exit 0
else
  echo "== KS-PP-2 grep-gate FAIL: capture after request_end() → reject =="
  exit 1
fi
