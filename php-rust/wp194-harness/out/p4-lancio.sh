#!/bin/bash
# p4-lancio.sh — S-194 P4: pin reading of the new fixture p_ro_ctor (P4base) joined to Pbase3 as s194-primitive-Pbase4.out, then
# p-coda.sh P4 P3 with that base. Declared copy of p3-lancio.sh (names only). Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")/.." && pwd)"
bash "$W/s194-primitive.sh" P4base --only noop,p_ro_ctor
{ cat "$W/s194-primitive-Pbase3.out"; grep -E '^p_ro_ctor' "$W/s194-primitive-P4base.out"; } > "$W/s194-primitive-Pbase4.out"
S194_BASE=s194-primitive-Pbase4.out bash "$W/out/p-coda.sh" P4 P3
