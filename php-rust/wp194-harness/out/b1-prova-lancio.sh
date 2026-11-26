#!/bin/bash
# b1-prova-lancio.sh — S-194 B1: once the N1b chain has ended, the quick reading prova.sh B1 N1b. Generato con assistenza AI (Claude Fable 5.2).
set -u
W="$(cd "$(dirname "$0")/.." && pwd)"
until [ -f "$W/out/p-coda-N1b.done" ]; do sleep 20; done
bash "$W/out/prova.sh" B1 N1b
