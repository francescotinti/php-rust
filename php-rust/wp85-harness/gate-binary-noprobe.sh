#!/bin/bash
# gate-binary-noprobe.sh — KH86-1 (Council WP-86, Hoare Q1): a campaign
# figure from a binary whose symbols/strings contain `vm_gate_probe` is
# VOID. Rationale: `cargo build` never unifies the dev-dep feature
# (resolver=2), but `cargo test --release` DOES unify it on that
# invocation's unit-graph — a binary picked from the target dir AFTER a
# test run can contain the probe. The cfg-gate guarantee was therefore one
# of PROTOCOL (which verb produced the binary); this tooth judges the
# BINARY itself, never the source and never the verb.
#
# 🔵 MEASURED LIMIT (S-85.0, positive control): a php-server binary built
# WITH php-runtime/vm-gate-probe hashes DIFFERENT (b55e2f78 vs 15fb6b46)
# yet carried NO vm_gate_probe symbol/string — the probe fn has no caller
# in the bin target and the linker dead-strips it. So the nm tooth alone
# was NECESSARY but NOT SUFFICIENT.
#
# A-TH37 (Council WP-87, S-86.0): the leaf module now carries a `#[used]`
# static marker (`phpr_vm_gate_probe_tainted_a_th37`, symbol
# VM_GATE_PROBE_TAINT) cfg-gated under the probe feature — llvm.used
# survives -dead_strip, so the nm/strings half is SUFFICIENT again as an
# INTRINSIC judge of the binary. KH87-2 positive control VERIFIED in
# S-86.0: tainted build => this gate FAILS (both halves detect: nm symbol
# + strings payload); clean rebuild => PASS. The hash-vs-matrix ENFORCE
# stays as belt (KS-AH-87-1 pins WHICH matrix).
#
# Usage: gate-binary-noprobe.sh <binary> [expected-sha256-prefix]
#        gate-binary-noprobe.sh --selftest
# Exit 0 = no probe symbol AND (if given) hash matches; !=0 = VOID.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u

probe_in() { # <file> -> 0 if the probe marker is present
  nm -- "$1" 2>/dev/null | grep -qi 'vm_gate_probe' && return 0
  strings -- "$1" 2>/dev/null | grep -q 'vm_gate_probe' && return 0
  return 1
}

if [ "${1:-}" = "--selftest" ]; then
  TMP=$(mktemp -d)
  printf 'GARBAGE\x00vm_gate_probe\x00MORE' > "$TMP/tainted.bin"
  printf 'GARBAGE\x00clean_symbol\x00MORE' > "$TMP/clean.bin"
  if ! probe_in "$TMP/tainted.bin"; then
    echo "SELFTEST FAIL: embedded vm_gate_probe marker NOT detected (KH86-1)"
    rm -rf "$TMP"; exit 1
  fi
  if probe_in "$TMP/clean.bin"; then
    echo "SELFTEST FAIL: clean binary flagged (KH86-1)"
    rm -rf "$TMP"; exit 1
  fi
  rm -rf "$TMP"
  echo "SELFTEST PASS: probe marker detection bites and clean passes (KH86-1)"
  exit 0
fi

[ $# -ge 1 ] || { echo "usage: $0 <binary> [expected-sha256-prefix] | --selftest"; exit 2; }
BIN="$1"; WANT="${2:-}"
FAILS=0
if [ ! -f "$BIN" ]; then
  echo "FAIL: binary missing: $BIN"; exit 1
fi
if probe_in "$BIN"; then
  echo "FAIL: vm_gate_probe present in $BIN — campaign VOID (KH86-1)"
  FAILS=$((FAILS+1))
else
  echo "OK  no vm_gate_probe symbol/string in $BIN (KH86-1, necessary half)"
fi
if [ -n "$WANT" ]; then
  GOT=$(shasum -a 256 "$BIN" | cut -d' ' -f1)
  case "$GOT" in
    "$WANT"*) echo "OK  binary hash matches expected prefix $WANT (KH86-1, sufficient half)";;
    *) echo "FAIL: binary hash $GOT != expected prefix $WANT — not the clean-build binary (KH86-1)"
       FAILS=$((FAILS+1));;
  esac
else
  echo "note: no expected hash supplied — only the necessary (nm) half ran; pair with the matrix ENFORCE"
fi
exit $FAILS
