#!/bin/bash
# t1-lancio.sh — S-194 T1 (criterion s194-criterio-N.md p.5): p-coda.sh T1 N2 against the pin reading C4base.
# Generato con assistenza AI (Claude Fable 5.2).
set -u
W="$(cd "$(dirname "$0")/.." && pwd)"
S194_BASE=s194-primitive-C4base.out bash "$W/out/p-coda.sh" T1 N2
