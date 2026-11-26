#!/bin/bash
# t1-prova-lancio.sh — S-194 T1: once the N2 chain has ended, the quick reading prova.sh T1 N2. Generato con assistenza AI (Claude Fable 5.2).
set -u
W="$(cd "$(dirname "$0")/.." && pwd)"
until [ -f "$W/out/p-coda-N2.done" ]; do sleep 20; done
bash "$W/out/prova.sh" T1 N2
