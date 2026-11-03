#!/usr/bin/env python3
"""gen-lancio-s188.py [tentativo] — genera s188-lancio.sh come COPIA DICHIARATA di
wp184-harness/s184-lancio-pair-t25.sh (regola shell-harness: ogni sostituzione asserita UNICA;
manifest s188-lancio-copia.diff prodotto dallo script chiamante). Adattamenti SOLI: nomi
s188/t<N>/pair188, harness wp188, lock TOKEN s188, exec s188-wp-server-pair.sh, attesa post-.done
del watchdog 10 min (nessun ORM). Generato con assistenza AI (Claude Fable 5).
"""
import os
import stat
import sys

T = sys.argv[1] if len(sys.argv) > 1 else "t1"
H = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(H, "..", "wp184-harness", "s184-lancio-pair-t25.sh")
DST = os.path.join(H, "s188-lancio.sh")
s = open(SRC, encoding="utf-8").read()

HEAD_OLD = """# s184-lancio-pair-t25.sh — COPPIA t25 @ pin s182 (criterio s184-criterio-coppia-t25.md p.3): COPIA DICHIARATA di
# ../wp182-harness/s182-lancio-pair-t24.sh (manifest s184-lancio-pair-copia.diff) coi SOLI adattamenti: nomi s184/t25/pair184,
# lock TOKEN s184, s184-pair.sh t25, PIÙ il gate sentinella «processo singolo >50 % CPU» ×2 (az.rev. S-183 #3, come
# s184-lancio-rt2.sh) dopo E2. Il resto INVARIATO (testo S-182 conservato sotto).
"""
HEAD_NEW = f"""# s188-lancio.sh — COPPIA SERVER WP {T} @ pin s183 (criterio s188-criterio-server-pair.md p.6): COPIA DICHIARATA di
# ../wp184-harness/s184-lancio-pair-t25.sh (manifest s188-lancio-copia.diff, generatore gen-lancio-s188.py) coi SOLI
# adattamenti: nomi s188/{T}/pair188, harness wp188, lock TOKEN s188, exec s188-wp-server-pair.sh {T}, attesa post-.done del
# watchdog 10 min (nessun ORM). Il resto INVARIATO (testo S-184 conservato sotto).
"""
SUBS = [
    (HEAD_OLD, HEAD_NEW),
    ('H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp184-harness"',
     'H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp188-harness"'),
    ('LOG="$H/pair-out/lancio-t25.log"', f'LOG="$H/pair-out/lancio-{T}.log"'),
    ('echo "s184 coppia t25 @ pin s182 $(date \'+%F %T\') pid=$$" > "$MLOCK"',
     f'echo "s188 coppia server WP {T} @ pin s183 $(date \'+%F %T\') pid=$$" > "$MLOCK"'),
    ('lock scritto: $MLOCK (TOKEN s184)', 'lock scritto: $MLOCK (TOKEN s188)'),
    ('( WD="$H/pair-out/watchdog-t25.txt"; PD="$H/pair-out/pair184-t25.done"; t0=$(date +%s); tdone=0',
     f'( WD="$H/pair-out/watchdog-{T}.txt"; PD="$H/pair-out/pair188-{T}.done"; t0=$(date +%s); tdone=0'),
    ('{ [ "$tdone" != 0 ] && [ $((now-tdone)) -ge 14400 ]; } && break',
     '{ [ "$tdone" != 0 ] && [ $((now-tdone)) -ge 600 ]; } && break'),
    ('Q="$H/pair-out/quiet-decl-t25.txt"', f'Q="$H/pair-out/quiet-decl-{T}.txt"'),
    ('{ echo "== dichiarazione finestra quieta t25 $(date \'+%F %T\') =="',
     f'{{ echo "== dichiarazione finestra quieta {T} $(date \'+%F %T\') =="'),
    ('quiete raggiunta — dichiarazione in quiet-decl-t25.txt — lancio pair t25',
     f'quiete raggiunta — dichiarazione in quiet-decl-{T}.txt — lancio coppia server WP {T}'),
    ('exec /bin/bash "$H/s184-pair.sh" t25', f'exec /bin/bash "$H/s188-wp-server-pair.sh" {T}'),
]
for old, new in SUBS:
    n = s.count(old)
    assert n == 1, f"sostituzione non unica ({n}): {old[:70]!r}"
    s = s.replace(old, new)
open(DST, "w", encoding="utf-8").write(s)
os.chmod(DST, os.stat(DST).st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
print(f"scritto {DST} ({len(SUBS)} sostituzioni uniche, tentativo {T})")
