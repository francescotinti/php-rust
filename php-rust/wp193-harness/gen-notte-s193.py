#!/usr/bin/env python3
# gen-notte-s193.py — genera i lanciatori della NOTTE di S-193 (criterio s193-criterio-notte.md) come COPIE DICHIARATE dei
# sei script di ../wp191-harness, ogni sostituzione con assert di unicità, manifest = diff vecchio/nuovo (<nome>-copia.diff):
#   s193-lancio-notte.sh    <- s191-lancio-notte.sh    (catena unica: gate di finestra, lock TOKEN s193, 4 passi)
#   s193-orm-coppia-t29.sh  <- s191-orm-coppia-t28.sh  (coppia dbal+ORM t29 @ pin s185)
#   s193-pair.sh            <- s191-pair.sh            (coppia WP full+media t27 @ pin s185)
#   s193-catena-fetta1.sh   <- s191-catena-fetta1.sh   (coppia server t4: guardia AX + tripwire)
#   s193-guardia-ax.sh      <- s191-guardia-ax.sh      (AXnew = s191 canonica, AXold = s190 stash)
#   s193-tripwire-w1.sh     <- s191-tripwire-w1.sh
# Eseguito il 20/11 dopo la promozione s185/s191 (rc=0). Generato con assistenza AI (Claude Fable 5.2).
import os, subprocess
os.chdir(os.path.dirname(os.path.abspath(__file__)) + '/..')

def rep(s, old, new, n=1):
    assert s.count(old) == n, (old[:80], s.count(old), n)
    return s.replace(old, new)

def rep_all(s, old, new, at_least=1):
    c = s.count(old); assert c >= at_least, (old[:80], c)
    return s.replace(old, new)

def line_from(s, start):
    assert s.count(start) == 1, (start[:80], s.count(start))
    t = s[s.index(start):]
    return t[:t.index('\n')]

def out(name, src, text):
    p = 'wp193-harness/' + name
    open(p, 'w', encoding='utf-8').write(text); os.chmod(p, 0o755)
    d = subprocess.run(['diff', src, p], capture_output=True, text=True).stdout
    open(p[:-3] + '-copia.diff', 'w', encoding='utf-8').write(d)

# ---------------------------------------------------------------- 1. catena unica
SRC = 'wp191-harness/s191-lancio-notte.sh'
s = open(SRC, encoding='utf-8').read()
head = '''#!/bin/bash
# s193-lancio-notte.sh — NOTTE di S-193 (criterio wp193-harness/s193-criterio-notte.md): CATENA UNICA, un solo lock TOKEN s193,
# gate di finestra EREDITATI, poi in ORDINE: (1) guardia b5 = s186-guardia.sh INVARIATO con A = stash phpr-s183, Z gemello S-186,
# B = binario CANONICO (pin s185): costo K0 TOTALE di s185 rispetto a s183 (b4 su s184 mordeva +4,62 ns/Ret) · (2) coppia
# dbal+ORM t29 @ pin s185 (costanti E5 INVARIATE: Delta_norm legge s185 vs riferimento s182) · (3) coppia server t4: s191
# canonica vs s190 stash (guardia AX + tripwire W=1) · (4) coppia WP full+media t27 @ pin s185 per ULTIMA.
# COPIA DICHIARATA di ../wp191-harness/s191-lancio-notte.sh (manifest s193-lancio-notte-copia.diff, generatore gen-notte-s193.py)
# coi SOLI adattamenti: harness wp193, TOKEN s193, controlli di hash (stash s183/s185, canoniche phpr s185 e server s191, stash
# server s190, gemello Z), blocco finale coi quattro passi di S-193 (tag b5/t29/t4/t27). Testo S-191/S-185/S-184 sotto:
'''
s = head + s[s.index('set -u\n'):]
s = rep(s, 'php-rust/wp191-harness"', 'php-rust/wp193-harness"')
s = rep(s, 'echo "s191 notte b4+t28+server-t3+t26 @ pin s184 $(date', 'echo "s193 notte b5+t29+server-t4+t27 @ pin s185 $(date')
s = rep(s, 'lock scritto: $MLOCK (TOKEN s191)', 'lock scritto: $MLOCK (TOKEN s193)')
s = rep(s, 'catena-out/catena-s191.done"', 'catena-out/catena-s193.done"', 2)
s = rep(s, '== dichiarazione finestra quieta NOTTE S-191 (b4, t28, server t3, t26)', '== dichiarazione finestra quieta NOTTE S-193 (b5, t29, server t4, t27)')
s = rep(s, '— (1) guardia b4, (2) t28, (3) server t3, (4) t26" >> "$LOG"', '— (1) guardia b5, (2) t29, (3) server t4, (4) t27" >> "$LOG"')
anchor = 'PIN183=$(shasum -a 256 "$STASH/phpr-s183" | cut -c1-16)'
assert s.count(anchor) == 1
s = s[:s.index(anchor)] + '''PIN183=$(shasum -a 256 "$STASH/phpr-s183" | cut -c1-16); PIN185=$(shasum -a 256 "$STASH/phpr-s185" | cut -c1-16)
CANON="$HOME/Claude/php-rust-output/release/phpr"
PC=$(shasum -a 256 "$CANON" | cut -c1-16)
SRVC=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/php-server" | cut -c1-16); SRV190=$(shasum -a 256 "$STASH/php-server-s190" | cut -c1-16)
[ "$PIN183" = "9a6ae11bf7e900ef" ] || fine "stash s183 $PIN183 != 9a6ae11bf7e900ef" "rc=9 stash183" 9
[ "$PIN185" = "d9d6429301f60f4e" ] || fine "stash s185 $PIN185 != d9d6429301f60f4e" "rc=9 stash185" 9
[ "$PC" = "$PIN185" ] || fine "pin canonico $PC != stash s185 $PIN185" "rc=9 pin" 9
[ "$SRVC" = "c413398a44f9b0a4" ] || fine "server canonico $SRVC != s191 c413398a44f9b0a4" "rc=9 server" 9
[ "$SRV190" = "721b7802e2d12058" ] || fine "stash server s190 $SRV190 != 721b7802e2d12058" "rc=9 stash190" 9
ZB="$REPO/wp186-harness/ab-out/s186-leva/phpr-Z"
[ "$(shasum -a 256 "$ZB" 2>/dev/null | cut -c1-16)" = "23c0a94c26db62eb" ] || fine "gemello Z assente o diverso ($ZB)" "rc=9 Z" 9
[ -e "$REPO/wp186-harness/s186-guardia-verdetto-b5.out" ] && fine "verdetto b5 ESISTE già: spostarlo prima" "rc=7 verdetto-b5" 7
echo "$(date '+%F %T') pin canonico $PC == stash s185 · stash s183 $PIN183 · server $SRVC / stash s190 $SRV190 · Z ok — (1) GUARDIA b5 (A=stash s183, Z, B=CANONICO s185, R=5)" >> "$LOG"
PIN="$STASH/phpr-s183" Z="$ZB" B="$CANON" R=5 RE3=3 TAG=b5 /bin/bash "$REPO/wp186-harness/s186-guardia.sh"
rc1=$?; GR="$(cat "$REPO/wp186-harness/guardia-out-b5/guardia.done" 2>/dev/null)"
echo "$(date '+%F %T') guardia b5 fine rc=$rc1 ($GR) — verdetto wp186-harness/s186-guardia-verdetto-b5.out — (2) ORM t29 (E5 INVARIATE: ORA_REF2 4,870 SENT [4,81;4,93]; REF2 [34,23;34,31] = riferimento phpr s182: Delta_norm legge s185 vs s182)" >> "$LOG"
SPD=/private/tmp/phpr-s193-orm; mkdir -p "$SPD"
export E5=1 ORA_REF2=4.870 SENT_LO=4.81 SENT_HI=4.93 REF2_MIN=34.23 REF2_MAX=34.31
PIN_ATTESO="$PIN185" MAPPA_SP="$SPD" /bin/bash "$H/s193-orm-coppia-t29.sh"
rc2=$?; TR="$(cat "$H/orm-out/rimisura.done" 2>/dev/null)"
echo "$(date '+%F %T') t29 fine rc=$rc2 ($TR) — verdetto s193-orm-coppia-verdetto-t29.out — (3) coppia server t4: s191 canonica vs s190 stash" >> "$LOG"
T=t4 PIN_ATTESO="$SRVC" PIN_OLD="$SRV190" SRVOLD="$STASH/php-server-s190" /bin/bash "$H/s193-catena-fetta1.sh"
rc3=$?; SR="$(cat "$H/pair-out/catena-fetta1.done" 2>/dev/null)"
echo "$(date '+%F %T') server t4 fine rc=$rc3 ($SR) — (4) WP t27 full+media @ pin s185 (ULTIMA: le gambe media risvegliano mediaanalysisd)" >> "$LOG"
PIN_ATTESO="$PIN185" SRV_ATTESO="$SRVC" /bin/bash "$H/s193-pair.sh" t27
rc4=$?; PR="$(cat "$H/pair-out/pair193-t27.done" 2>/dev/null)"
RC=0; for r in "$rc1" "$rc2" "$rc3" "$rc4"; do [ "$r" -gt "$RC" ] && RC=$r; done
fine "notte fine: b5 rc=$rc1 · t29 rc=$rc2 · server t4 rc=$rc3 · t27 rc=$rc4; lock rimosso" "rc=$RC guardia_b5=[$GR] t29=[$TR] server_t4=[$SR] t27=[$PR]" "$RC"
'''
b = s[s.index('set -u\n'):]
for bad in ('TOKEN s191', 'catena-s191', 'wp191', 's184', 't28', 't26', 'b4'):
    assert bad not in b, bad
out('s193-lancio-notte.sh', SRC, s)

# ---------------------------------------------------------------- 2. ORM t29
SRC = 'wp191-harness/s191-orm-coppia-t28.sh'
o = open(SRC, encoding='utf-8').read()
o = rep(o, '#!/bin/bash\n', '#!/bin/bash\n'
        '# s193-orm-coppia-t29.sh — S-193: coppia dbal+ORM t29 @ pin s185 (DOVUTA: pin nuovo con le sei leve di S-192; t28 era @ s184).\n'
        '# COPIA DICHIARATA di ../wp191-harness/s191-orm-coppia-t28.sh (manifest s193-orm-coppia-t29-copia.diff, generatore\n'
        '# gen-notte-s193.py) coi SOLI adattamenti: harness wp193 (orm-out, verdetto s193-orm-coppia-verdetto-t29.out), PIN_ATTESO\n'
        '# default s185 d9d6429301f60f4e, etichette t29/s185. Costanti E5 dal lanciatore, INVARIATE. Testo S-191/S-185 sotto:\n')
o = rep(o, 'H="$REPO/wp191-harness"', 'H="$REPO/wp193-harness"')
o = rep(o, 'VERD="$H/s191-orm-coppia-verdetto-t28.out"', 'VERD="$H/s193-orm-coppia-verdetto-t29.out"')
o = rep(o, 'PIN_ATTESO="${PIN_ATTESO:-60b10d857eb0343a}"', 'PIN_ATTESO="${PIN_ATTESO:-d9d6429301f60f4e}"')
o = rep(o, 'print("coppia-t28 @ s184 (user CPU;', 'print("coppia-t29 @ s185 (user CPU;')
o = rep(o, '{ echo "== s191 coppia t28 dbal+ORM (pin s184 MISURATO $PINM', '{ echo "== s193 coppia t29 dbal+ORM (pin s185 MISURATO $PINM')
out('s193-orm-coppia-t29.sh', SRC, o)

# ---------------------------------------------------------------- 3. WP t27
SRC = 'wp191-harness/s191-pair.sh'
p = open(SRC, encoding='utf-8').read()
p = rep(p, '#!/bin/bash\n', '#!/bin/bash\n'
        '# s193-pair.sh <tentativo> — S-193: coppia full/media WP t27 @ pin s185 (DOVUTA: pin nuovo, WordPress mai rimisurato dopo le\n'
        '# sei leve di S-192; t26 era @ s184). COPIA DICHIARATA di ../wp191-harness/s191-pair.sh (manifest s193-pair-copia.diff,\n'
        '# generatore gen-notte-s193.py) coi SOLI adattamenti: nomi s193/pair193/t27; harness wp193; mediane storiche + t26=1,751\n'
        '# (banda di giudizio INVARIATA [1,738;1,799]); messaggi riferiti al pin s185 (attribuzione per leva). Testo S-191 sotto:\n')
p = rep(p, 'H="$REPO/wp191-harness"', 'H="$REPO/wp193-harness"')
p = rep(p, 'VERD="$H/s191-pair-verdetto-$T.out"', 'VERD="$H/s193-pair-verdetto-$T.out"')
p = rep_all(p, 'pair191-', 'pair193-')
p = rep(p, '== s191 coppia t26: WP full+media @ pin s184 MISURATO', '== s193 coppia t27: WP full+media @ pin s185 MISURATO')
p = rep(p, "vs t25=1.747: {'≤' if med_t26 <= 1.747 else '>'}", "vs t26=1.751: {'≤' if med_t26 <= 1.751 else '>'}")
p = rep(p, 't24=1.770 t25=1.747', 't24=1.770 t25=1.747 t26=1.751', p.count('t24=1.770 t25=1.747'))
assert p.count('t26=1.751') >= 2
p = rep_all(p, 'med_t26', 'med_t27')
p = rep(p, 'mediana_t26=', 'mediana_t27=')
p = rep_all(p, 'pin s184 (§3.35+O(1) v2, php-types, §3.39: attribuzione per braccio)', 'pin s185 (sei leve di S-192: attribuzione per leva)')
p = rep_all(p, 'pin NUOVO s184', 'pin NUOVO s185')
out('s193-pair.sh', SRC, p)

# ---------------------------------------------------------------- 4. server: catena + guardia AX + tripwire
SRC = 'wp191-harness/s191-catena-fetta1.sh'
c = open(SRC, encoding='utf-8').read()
c = rep(c, '#!/bin/bash\n', '#!/bin/bash\n'
        '# s193-catena-fetta1.sh — S-193 coppia server s191 (AXnew, canonica) vs s190 (AXold, stash), tentativo t4, nella finestra\n'
        '# della notte (lock TOKEN s193). COPIA DICHIARATA di ../wp191-harness/s191-catena-fetta1.sh (manifest\n'
        '# s193-catena-fetta1-copia.diff, generatore gen-notte-s193.py) coi SOLI adattamenti: harness wp193, PIN_OLD default s190,\n'
        '# script s193-*, done ga193/tw193. Testo S-191/S-189 sotto:\n')
c = rep(c, 'php-rust/wp191-harness"', 'php-rust/wp193-harness"')
c = rep(c, 'export PIN_ATTESO PIN_OLD="${PIN_OLD:-80eea401b71c2be7}"', 'export PIN_ATTESO PIN_OLD="${PIN_OLD:-721b7802e2d12058}"')
c = rep(c, '/bin/bash "$H/s191-guardia-ax.sh" "$T"', '/bin/bash "$H/s193-guardia-ax.sh" "$T"')
c = rep(c, '/bin/bash "$H/s191-tripwire-w1.sh" "$T"', '/bin/bash "$H/s193-tripwire-w1.sh" "$T"')
c = rep_all(c, 'ga191-', 'ga193-'); c = rep_all(c, 'tw191-', 'tw193-')
out('s193-catena-fetta1.sh', SRC, c)

def server_copy(src, tag_old, tag_new, own, title):
    g = open(src, encoding='utf-8').read()
    g = rep(g, '#!/bin/bash\n', '#!/bin/bash\n' + title, 1)
    g = rep_all(g, '/wp191-harness"', '/wp193-harness"')
    g = rep_all(g, tag_old, tag_new)
    g = rep(g, "grep -q 's191' \"$MLOCK\"", "grep -q 's193' \"$MLOCK\"")
    g = rep_all(g, 'php-server-s189}', 'php-server-s190}')
    g = rep_all(g, '${PIN_OLD:-80eea401b71c2be7}', '${PIN_OLD:-721b7802e2d12058}')
    g = rep(g, 'shasum -a 256 "$H/' + own.replace('s193', 's189') + '" "$RU"', 'shasum -a 256 "$H/' + own + '" "$RU"')
    return g

SRC = 'wp191-harness/s191-guardia-ax.sh'
g = server_copy(SRC, 'ga191-', 'ga193-', 's193-guardia-ax.sh',
    '# s193-guardia-ax.sh <tentativo> — S-193 coppia server: guardia di NON-REGRESSIONE del throughput AXnew = pin s191 (canonica,\n'
    '# c413398a44f9b0a4) vs AXold = pin s190 (stash, 721b7802e2d12058), stesso giudice di S-189/S-191. COPIA DICHIARATA di\n'
    '# ../wp191-harness/s191-guardia-ax.sh (manifest s193-guardia-ax-copia.diff, generatore gen-notte-s193.py) coi SOLI adattamenti:\n'
    '# harness wp193, done ga193, lock TOKEN s193, SRVOLD/PIN_OLD default s190, etichette s190/s191, e DRIVER_SHA calcolato su\n'
    '# QUESTO script (in S-191 puntava a un file s189 assente dalla cartella: etichetta rotta, non un giudice). Testo sotto:\n')
g = rep(g, 'VERD="$H/s191-guardia-ax-verdetto-$T.out"', 'VERD="$H/s193-guardia-ax-verdetto-$T.out"')
g = rep(g, 'pin s189 {OLD} (stash)", "AXnew": f"php-server --axum --workers 4 pin s190 {NEW} (canonica)"}', 'pin s190 {OLD} (stash)", "AXnew": f"php-server --axum --workers 4 pin s191 {NEW} (canonica)"}')
g = rep(g, 'print(f"== s191 coppia server guardia AX (NON-REGRESSIONE; in banda ⇒ s190 server di record): AXnew s190 {NEW} vs AXold s189 {OLD}',
        'print(f"== s193 coppia server guardia AX (NON-REGRESSIONE; in banda ⇒ s191 server di record): AXnew s191 {NEW} vs AXold s190 {OLD}')
out('s193-guardia-ax.sh', SRC, g)

SRC = 'wp191-harness/s191-tripwire-w1.sh'
t = server_copy(SRC, 'tw191-', 'tw193-', 's193-tripwire-w1.sh',
    '# s193-tripwire-w1.sh <tentativo> — S-193 coppia server: tripwire W=1 Anew = pin s191 (canonica) vs Aold = pin s190 (stash),\n'
    '# |Anew/Aold − 1| ≤ 2 %. COPIA DICHIARATA di ../wp191-harness/s191-tripwire-w1.sh (manifest s193-tripwire-w1-copia.diff,\n'
    '# generatore gen-notte-s193.py) coi SOLI adattamenti: harness wp193, done tw193, lock TOKEN s193, SRVOLD/PIN_OLD default s190,\n'
    '# etichette s190/s191, DRIVER_SHA su QUESTO script. Testo S-191/S-189 sotto:\n')
t = rep(t, 'VERD="$H/s191-tripwire-w1-verdetto-$T.out"', 'VERD="$H/s193-tripwire-w1-verdetto-$T.out"')
t = rep(t, 'print(f"== s191 coppia server tripwire W=1 (NON-REGRESSIONE, mai cifra): Anew s190 {NEW} vs Aold s189 {OLD}',
        'print(f"== s193 coppia server tripwire W=1 (NON-REGRESSIONE, mai cifra): Anew s191 {NEW} vs Aold s190 {OLD}')
out('s193-tripwire-w1.sh', SRC, t)
print('ok')
