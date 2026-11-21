#!/bin/bash
# s194-fuori.sh — S-194 "left outside" measurements: probe on the cross-file / by-reference / host fixtures (current arm),
# then the ORM instruction count on pin, pre-N commit and N commit. Outcome: s194-fuori.out, out/fuori.done. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")" && pwd)"; ARM="$HOME/Claude/phpr-target/s192-armD"; V="$W/s194-fuori.out"; : > "$V"; rm -f "$W/out/fuori.done"
bash "$W/s194-primitive.sh" X1 --bin "$ARM/release/phpr" --only noop,ns_noop,call_user2,preg_match,ns_user2,ns_xfile_user2,ns_xfile_user2_q,x_global_user2,builtin_reset,ns_reset,builtin_pushpop,ns_pushpop,builtin_fexists,ns_fexists,builtin_inarray,ns_inarray,ns_preg_match
echo "sonda X1: rc=$(cat "$W/s194-primitive-X1.rc")" >> "$V"
: > "$W/s194-orm-istruzioni.out"
python3 "$W/s194-orm-istruzioni.py" pin="$HOME/Claude/php-rust-output/release/phpr" preN=182da85d N=0588daa4 > "$W/out/orm-istruzioni.log" 2>&1
echo "ORM istruzioni: rc=$(cat "$W/out/orm-istruzioni.rc" 2>/dev/null || echo 99)" >> "$V"
echo "fine $(date '+%F %T')" >> "$V"; echo 0 > "$W/out/fuori.done"
