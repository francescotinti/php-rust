#!/usr/bin/env python3
# gen-orm-leg1.py — genera s184-orm-leg1-rerun.sh come COPIA DICHIARATA di ../wp176-harness/s176-orm-coppia.sh
# (S-184 bis: rilancio della SOLA gamba 1 ORM). Ogni sostituzione e' asserita UNICA; manifest s184-orm-leg1-rerun-copia.diff.
import os, subprocess
H = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(H, "..", "wp176-harness", "s176-orm-coppia.sh")
DST = os.path.join(H, "s184-orm-leg1-rerun.sh")
s = open(SRC).read()
def sub(old, new):
    global s
    assert s.count(old) == 1, f"NON unico ({s.count(old)}): {old[:60]!r}"
    s = s.replace(old, new)
sub("#!/bin/bash\n# EMENDA S-184 bis",
    "#!/bin/bash\n# s184-orm-leg1-rerun.sh — S-184 bis (2026-10-08): RILANCIO della SOLA gamba 1 ORM (oracle1+phpr1) della coppia t25 @ pin\n"
    "# s182, dopo che il phpr della gamba 1 notturna ha completato la suite (output identico alla gamba 2) e NON e' uscito\n"
    "# (ucciso dal watchdog dopo 600 s, rc=124, .time senza user). COPIA DICHIARATA di ../wp176-harness/s176-orm-coppia.sh\n"
    "# (manifest s184-orm-leg1-rerun-copia.diff) coi SOLI adattamenti: (1) gambe giudicate = solo leg 1 (leg 2 ORM e dbal 1-2\n"
    "# della notte restano in orm-out e il giudice li rilegge); (2) WORKLOADS default = orm; (3) progress.txt in APPEND;\n"
    "# (4) i file orm-*1 della notte conservati come *.t25-hang PRIMA del run, con verifica ls; (5) intestazione del verdetto\n"
    "# marcata RERUN. H/OUT restano wp176-harness (stessa cartella dati, stesso lock TOKEN s184). Tutto il resto INVARIATO.\n"
    "# EMENDA S-184 bis")
sub(': > "$OUT/progress.txt"\nrm -f "$OUT/rimisura.done"\n',
    'echo "== RERUN gamba 1 ORM (S-184 bis) $(date \'+%F %T\') ==" >> "$OUT/progress.txt"\nrm -f "$OUT/rimisura.done"\n'
    'for f in orm-oracle1.txt orm-oracle1.time orm-phpr1.txt orm-phpr1.time orm-phpr1.failnames; do [ -e "$OUT/$f" ] && mv "$OUT/$f" "$OUT/$f.t25-hang"; done\n'
    'ls "$OUT/orm-phpr1.time.t25-hang" > /dev/null 2>&1 || { echo "rc=8 conserva-notte (orm-phpr1.time.t25-hang assente)" > "$OUT/rimisura.done"; exit 8; }\n'
    '[ -e "$OUT/orm-phpr1.time" ] && { echo "rc=8 orm-phpr1.time ancora presente dopo mv" > "$OUT/rimisura.done"; exit 8; }\n')
sub('WLS="${WORKLOADS:-orm dbal}"', 'WLS="${WORKLOADS:-orm}"')
sub('for leg in 1 2; do\n  case " $WLS " in *" orm "*)\n    quiesce_gate "orm-leg$leg"',
    'for leg in 1; do\n  case " $WLS " in *" orm "*)\n    quiesce_gate "orm-leg$leg"')
sub('{ echo "== s175 coppia dbal+ORM (pin s175 MISURATO $PINM vs oracle 8.5.7; criterio s175-criterio-orm.md) =="',
    '{ echo "== s175 coppia dbal+ORM (pin s175 MISURATO $PINM vs oracle 8.5.7; criterio s175-criterio-orm.md) == [RERUN S-184 bis $(date \'+%F %T\'): SOLO gamba 1 ORM rimisurata; leg2 ORM e dbal 1-2 dalla finestra notturna t25, stesso lock TOKEN s184]"')
open(DST, "w").write(s); os.chmod(DST, 0o755)
with open(os.path.join(H, "s184-orm-leg1-rerun-copia.diff"), "w") as m:
    subprocess.run(["diff", SRC, DST], stdout=m)
print("generato", DST)
