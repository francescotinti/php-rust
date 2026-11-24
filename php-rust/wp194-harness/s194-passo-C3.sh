#!/bin/bash
# s194-passo-C3.sh <tag> [base.out] — S-194 C3 (declared copy of s194-passo-C.sh, see gen-passo-C3.py), one measured step: pin-recipe arm build of the current tree, then the primitive probe
# on the call/return fixtures against the pin s187 reading C3base. Outcome: s194-passo-<tag>.out, out/passo-<tag>.done.
# Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")" && pwd)"; T="$1"; ARM="$HOME/Claude/phpr-target/s192-armD"; V="$W/s194-passo-$T.out"; : > "$V"; rm -f "$W/out/passo-$T.done"
FEED="/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"
for i in $(seq 1 240); do
  if [ "$(tail -1 "$FEED" | cut -d' ' -f1)" = "DONE" ] && ! pgrep -x cargo >/dev/null; then break; fi
  sleep 15
done
cd "$W/.." || exit 2
CARGO_TARGET_DIR="$ARM" CARGO_PROFILE_RELEASE_DEBUG=line-tables-only CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO=packed cargo build --release -p php-cli > "$W/out/passo-$T-braccio.log" 2>&1
b=$?; echo "braccio: rc=$b sha=$(shasum -a 256 "$ARM/release/phpr" | cut -c1-16)" >> "$V"
if [ "$b" != 0 ]; then grep -E -A12 '^(error|warning: unused)' "$W/out/passo-$T-braccio.log" | cut -c1-240 | sed -n '1,60p' >> "$V"; echo 1 > "$W/out/passo-$T.done"; exit 1; fi
bash "$W/s194-primitive.sh" "$T" --bin "$ARM/release/phpr" --base "${2:-s194-primitive-C3base.out}" --only noop,ns_noop,call_user0,call_user2,call_typed,call_method,call_static,call_closure,call_this_inc,new_ctor,ns_user2,ret_typed_only,ret_typed_class,call_typed_class,call_typed_iface,mcall_typed,mcall_typed_class,call_default,mcall_default
echo "sonda: rc=$(cat "$W/s194-primitive-$T.rc")" >> "$V"
echo "fine $(date '+%F %T')" >> "$V"; echo 0 > "$W/out/passo-$T.done"
