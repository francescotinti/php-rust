#!/bin/bash
# c4d-lancio.sh — S-194 C4d = C4c with the frame read only when a place is present (C4c: acceptance red, two frozen-set tests panicked
# on a callback run with no frame): p-coda.sh C4d C4b against the pin reading C4base. Generato con assistenza AI (Claude Fable 5.2).
set -u
W="$(cd "$(dirname "$0")/.." && pwd)"
S194_BASE=s194-primitive-C4base.out bash "$W/out/p-coda.sh" C4d C4b
