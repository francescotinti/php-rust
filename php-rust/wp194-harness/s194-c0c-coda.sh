#!/bin/bash
# s194-c0c-coda.sh — S-194 package C, step C0c: profile of one whole ORM leg on HEAD (S-192 anatomy tool, unchanged), then its
# second reading (families and who pays). Outcomes: out/c0c-coda.done; wp192-harness/s192-anatomia-s194c.out and -chiamanti-s194c.out.
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")" && pwd)"; H="$W/../wp192-harness"; rm -f "$W/out/c0c-coda.done"
q=0; while [ "$q" -lt 2 ]; do if pgrep -qx cargo || pgrep -qx rustc; then q=0; else q=$((q + 1)); fi; sleep 20; done
python3 "$H/s192-anatomia.py" s194c > "$W/out/c0c-anatomia.log" 2>&1; a=$?
python3 "$H/s192-anatomia-chiamanti.py" s194c > "$W/out/c0c-chiamanti.log" 2>&1; b=$?
echo "anatomia=$a chiamanti=$b" > "$W/out/c0c-coda.done"
