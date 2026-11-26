#!/bin/bash
# n3-lancio.sh — S-194 N3 (criterion s194-criterio-N.md p.7): p-coda.sh N3 T1 against the pin reading C4base.
# Generato con assistenza AI (Claude Fable 5.2).
set -u
W="$(cd "$(dirname "$0")/.." && pwd)"
S194_BASE=s194-primitive-C4base.out bash "$W/out/p-coda.sh" N3 T1
