#!/usr/bin/env python3
# gen-catena-P.py — S-194: generates s194-catena-P.sh (acceptance chain on HEAD before the group promotion) as a DECLARED COPY of
# s194-catena-A.sh (every substitution asserted, manifest s194-catena-P-copia.diff). Generato con assistenza AI (Claude Fable 5.2).
import os, subprocess
os.chdir(os.path.dirname(os.path.abspath(__file__)))
def rep(s, old, new, n=1):
    assert s.count(old) == n, (old[:70], s.count(old)); return s.replace(old, new)
s = open('s194-catena-A.sh', encoding='utf-8').read()
head_old = s[:s.index('set -u\n')]
s = '''#!/bin/bash
# s194-catena-P.sh — S-194 group promotion: acceptance chain on HEAD (criterion s194-criterio-promozione.md p.2).
# DECLARED COPY of s194-catena-A.sh (manifest s194-catena-P-copia.diff, generator gen-catena-P.py); the ONLY adaptations:
# tag P, base of the probe = the package A reading, no profile step. Outcomes: s194-catena-P.out, out/catena-P.done.
# Generato con assistenza AI (Claude Fable 5.2).
''' + s[len(head_old):]
s = rep(s, 'catena-A', 'catena-P', s.count('catena-A'))
s = rep(s, 's194A', 's194P', 3)
s = rep(s, 'bash "$W/s194-primitive.sh" A --bin "$ARM/release/phpr" --base s194-primitive-F.out', 'bash "$W/s194-primitive.sh" P --bin "$ARM/release/phpr" --base s194-primitive-A.out')
s = rep(s, 'echo "sonda primitive A: rc=$(cat "$W/s194-primitive-A.rc")" >> "$V"', 'echo "sonda primitive P: rc=$(cat "$W/s194-primitive-P.rc")" >> "$V"')
i = s.index('S194_BIN='); j = s.index('fine 0'); s = s[:i] + s[j:]
assert 'profilo' not in s[s.index('set -u'):] and 'catena-A' not in s
open('s194-catena-P.sh', 'w', encoding='utf-8').write(s); os.chmod('s194-catena-P.sh', 0o755)
open('s194-catena-P-copia.diff', 'w', encoding='utf-8').write(subprocess.run(['diff', 's194-catena-A.sh', 's194-catena-P.sh'], capture_output=True, text=True).stdout)
print('ok')
