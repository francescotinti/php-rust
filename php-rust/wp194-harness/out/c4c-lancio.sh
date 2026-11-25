#!/bin/bash
# c4c-lancio.sh — S-194 C4c (general funnels settle a step-less place before the general resolver): waits for the C4b chain to end,
# then p-coda.sh C4c C4b against the pin reading C4base. Generato con assistenza AI (Claude Fable 5.2).
set -u
W="$(cd "$(dirname "$0")/.." && pwd)"
until [ -f "$W/out/p-coda-C4b.done" ]; do sleep 20; done
S194_BASE=s194-primitive-C4base.out bash "$W/out/p-coda.sh" C4c C4b
