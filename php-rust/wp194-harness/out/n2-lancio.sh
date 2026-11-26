#!/bin/bash
# n2-lancio.sh — S-194 N2 (+B1+B2): the proof arm n2-copertura.sh, then p-coda.sh N2 N1b against the pin reading C4base.
# Generato con assistenza AI (Claude Fable 5.2).
set -u
W="$(cd "$(dirname "$0")/.." && pwd)"
bash "$W/out/n2-copertura.sh"
S194_BASE=s194-primitive-C4base.out bash "$W/out/p-coda.sh" N2 N1b
