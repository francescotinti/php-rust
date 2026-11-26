#!/bin/bash
# n1b-lancio.sh — S-194 N1b (criterion s194-criterio-N.md p.3): p-coda.sh N1b C4d against the pin reading C4base.
# Generato con assistenza AI (Claude Fable 5.2).
set -u
W="$(cd "$(dirname "$0")/.." && pwd)"
S194_BASE=s194-primitive-C4base.out bash "$W/out/p-coda.sh" N1b C4d
