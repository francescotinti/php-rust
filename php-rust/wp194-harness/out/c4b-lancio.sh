#!/bin/bash
# c4b-lancio.sh — S-194 C4b (lighter general funnels for step-less places): p-coda.sh C4b C4 against the pin reading C4base.
# Generato con assistenza AI (Claude Fable 5.2).
set -u
W="$(cd "$(dirname "$0")/.." && pwd)"
S194_BASE=s194-primitive-C4base.out bash "$W/out/p-coda.sh" C4b C4
