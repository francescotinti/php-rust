#!/bin/bash
# n1-lancio.sh — S-194 N1 (criterion s194-criterio-N.md): p-coda.sh N1 C4d against the pin reading C4base.
# Generato con assistenza AI (Claude Fable 5.2).
set -u
W="$(cd "$(dirname "$0")/.." && pwd)"
S194_BASE=s194-primitive-C4base.out bash "$W/out/p-coda.sh" N1 C4d
