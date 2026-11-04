#!/usr/bin/env python3
# gen-notte-s191.py — genera i lanciatori della NOTTE 04→05/11 (criterio s191-criterio-notte.md) come COPIE DICHIARATE,
# ogni sostituzione con assert di unicità, manifest = diff vecchio/nuovo (<nome>-copia.diff):
#   s191-lancio-notte.sh     ← wp185-harness/s185-lancio-catena.sh   (catena unica: gate di finestra, lock TOKEN s191, 4 passi)
#   s191-orm-coppia-t28.sh   ← wp185-harness/s185-orm-coppia-t27.sh  (coppia dbal+ORM t28 @ pin s184)
#   s191-pair.sh             ← wp184-harness/s184-pair.sh            (coppia WP full+media t26 @ pin s184)
#   s191-catena-fetta1.sh    ← wp189-harness/s189-catena-fetta1.sh   (coppia server: guardia AX + tripwire, t3)
#   s191-guardia-ax.sh       ← wp189-harness/s189-guardia-ax.sh      (AXnew = s190 canonica, AXold = s189 stash)
#   s191-tripwire-w1.sh      ← wp189-harness/s189-tripwire-w1.sh
# Eseguito una volta il 04/11 (mattina) dopo la promozione s184/s190. Generato con assistenza AI (Claude Fable 5).
import os, re
os.chdir(os.path.dirname(os.path.abspath(__file__)) + '/..')

def rep(s, old, new, n=1):
    assert s.count(old) == n, (old[:80], s.count(old), n)
    return s.replace(old, new)

def rep_all(s, old, new, at_least=1):
    c = s.count(old); assert c >= at_least, (old[:80], c)
    return s.replace(old, new)

def head_swap(s, new_head):
    i = s.index('set -u\n'); return new_head + s[i:]

def out(name, text):
    p = 'wp191-harness/' + name
    open(p, 'w', encoding='utf-8').write(text); os.chmod(p, 0o755)

# ---------------------------------------------------------------- 1. catena unica
s = open('wp185-harness/s185-lancio-catena.sh', encoding='utf-8').read()
head = '''#!/bin/bash
# s191-lancio-notte.sh — NOTTE 04→05/11 (criterio wp191-harness/s191-criterio-notte.md): CATENA UNICA, un solo lock TOKEN s191,
# gate di finestra EREDITATI (quiete CI, anti-flare 6x30 s, loadavg1 <3 x6, E2 <150 % x4, processo singolo >50 % x2, dichiarazione
# uptime+top), poi in ORDINE: (1) guardia b4 = s186-guardia.sh con A = stash phpr-s183, Z gemello S-186, B = stash phpr-s184
# (scioglie la riserva K0 della conferma diurna) · (2) coppia dbal+ORM t28 @ pin s184 (s191-orm-coppia-t28.sh, costanti E5 fisse:
# ORA_REF2/SENT = oracle invariato, REF2 = riferimento phpr s182 così Delta_norm legge s184 vs s182) · (3) coppia server t3:
# s190 canonica vs s189 stash (s191-catena-fetta1.sh: guardia AX + tripwire W=1) · (4) coppia WP full+media t26 @ pin s184
# (s191-pair.sh t26) per ULTIMA: le gambe media risvegliano mediaanalysisd e stallerebbero i gate dei passi successivi.
# COPIA DICHIARATA di ../wp185-harness/s185-lancio-catena.sh (manifest s191-lancio-notte-copia.diff, generatore gen-notte-s191.py)
# coi SOLI adattamenti: harness wp191, TOKEN s191, watchdog disco 7 h (era 3 h: quattro passi), controlli di hash su stash
# s183/s184/s189 + canoniche phpr s184 e server s190 + gemello Z, il blocco finale a 4 passi (al posto di istruttoria + t27),
# fine() con i quattro rc (rc complessivo = massimo). Nessuna igiene FS qui. Testo S-185/S-184 conservato sotto:
'''
s = head_swap(s, head)
s = rep(s, 'H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp185-harness"', 'H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp191-harness"')
s = rep(s, 'echo "s185 catena istruttoria+t27 @ pin s182 $(date \'+%F %T\') pid=$$" > "$MLOCK"', 'echo "s191 notte b4+t28+server-t3+t26 @ pin s184 $(date \'+%F %T\') pid=$$" > "$MLOCK"')
s = rep(s, 'lock scritto: $MLOCK (TOKEN s185)', 'lock scritto: $MLOCK (TOKEN s191)')
s = rep(s, 'PD="$H/catena-out/catena-s185.done"', 'PD="$H/catena-out/catena-s191.done"')
s = rep(s, '[ $((now-t0)) -ge 10800 ] && break', '[ $((now-t0)) -ge 25200 ] && break')
s = rep(s, '== dichiarazione finestra quieta catena S-185 (istruttoria + t27)', '== dichiarazione finestra quieta NOTTE S-191 (b4, t28, server t3, t26)')
s = rep(s, 'quiete raggiunta — dichiarazione in quiet-decl-catena.txt — istruttoria 3 bracci, poi t27', 'quiete raggiunta — dichiarazione in quiet-decl-catena.txt — (1) guardia b4, (2) t28, (3) server t3, (4) t26')
s = rep(s, 'DONE="$H/catena-out/catena-s185.done"; rm -f "$DONE"', 'DONE="$H/catena-out/catena-s191.done"; rm -f "$DONE"')
anchor = 'PIN_ATTESO=$(shasum -a 256 "$STASH/phpr-s182" | cut -c1-16)'
assert s.count(anchor) == 1
s = s[:s.index(anchor)] + '''PIN183=$(shasum -a 256 "$STASH/phpr-s183" | cut -c1-16); PIN184=$(shasum -a 256 "$STASH/phpr-s184" | cut -c1-16)
PC=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)
SRVC=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/php-server" | cut -c1-16); SRV189=$(shasum -a 256 "$STASH/php-server-s189" | cut -c1-16)
[ "$PIN183" = "9a6ae11bf7e900ef" ] || fine "stash s183 $PIN183 != 9a6ae11bf7e900ef" "rc=9 stash183" 9
[ "$PIN184" = "60b10d857eb0343a" ] || fine "stash s184 $PIN184 != 60b10d857eb0343a" "rc=9 stash184" 9
[ "$PC" = "$PIN184" ] || fine "pin canonico $PC != stash s184 $PIN184" "rc=9 pin" 9
[ "$SRVC" = "721b7802e2d12058" ] || fine "server canonico $SRVC != s190 721b7802e2d12058" "rc=9 server" 9
[ "$SRV189" = "80eea401b71c2be7" ] || fine "stash server s189 $SRV189 != 80eea401b71c2be7" "rc=9 stash189" 9
ZB="$REPO/wp186-harness/ab-out/s186-leva/phpr-Z"
[ "$(shasum -a 256 "$ZB" 2>/dev/null | cut -c1-16)" = "23c0a94c26db62eb" ] || fine "gemello Z assente o diverso ($ZB)" "rc=9 Z" 9
[ -e "$REPO/wp186-harness/s186-guardia-verdetto-b4.out" ] && fine "verdetto b4 ESISTE già: spostarlo prima" "rc=7 verdetto-b4" 7
echo "$(date '+%F %T') pin $PC == stash s184 · stash s183 $PIN183 · server $SRVC / stash s189 $SRV189 · Z ok — (1) GUARDIA b4 (A=stash s183, Z, B=stash s184, R=5)" >> "$LOG"
PIN="$STASH/phpr-s183" Z="$ZB" B="$STASH/phpr-s184" R=5 RE3=3 TAG=b4 /bin/bash "$REPO/wp186-harness/s186-guardia.sh"
rc1=$?; GR="$(cat "$REPO/wp186-harness/guardia-out-b4/guardia.done" 2>/dev/null)"
echo "$(date '+%F %T') guardia b4 fine rc=$rc1 ($GR) — verdetto wp186-harness/s186-guardia-verdetto-b4.out — (2) ORM t28 (E5: ORA_REF2 4,870 SENT [4,81;4,93] = oracle invariato; REF2 [34,23;34,31] = riferimento phpr s182: Delta_norm legge s184 vs s182)" >> "$LOG"
SPD=/private/tmp/phpr-s191-orm; mkdir -p "$SPD"
export E5=1 ORA_REF2=4.870 SENT_LO=4.81 SENT_HI=4.93 REF2_MIN=34.23 REF2_MAX=34.31
PIN_ATTESO="$PIN184" MAPPA_SP="$SPD" /bin/bash "$H/s191-orm-coppia-t28.sh"
rc2=$?; TR="$(cat "$H/orm-out/rimisura.done" 2>/dev/null)"
echo "$(date '+%F %T') t28 fine rc=$rc2 ($TR) — verdetto s191-orm-coppia-verdetto-t28.out — (3) coppia server t3: s190 canonica vs s189 stash" >> "$LOG"
T=t3 PIN_ATTESO="$SRVC" PIN_OLD="$SRV189" SRVOLD="$STASH/php-server-s189" /bin/bash "$H/s191-catena-fetta1.sh"
rc3=$?; SR="$(cat "$H/pair-out/catena-fetta1.done" 2>/dev/null)"
echo "$(date '+%F %T') server t3 fine rc=$rc3 ($SR) — (4) WP t26 full+media @ pin s184 (ULTIMA: le gambe media risvegliano mediaanalysisd)" >> "$LOG"
PIN_ATTESO="$PIN184" SRV_ATTESO="$SRVC" /bin/bash "$H/s191-pair.sh" t26
rc4=$?; PR="$(cat "$H/pair-out/pair191-t26.done" 2>/dev/null)"
RC=0; for r in "$rc1" "$rc2" "$rc3" "$rc4"; do [ "$r" -gt "$RC" ] && RC=$r; done
fine "notte fine: b4 rc=$rc1 · t28 rc=$rc2 · server t3 rc=$rc3 · t26 rc=$rc4; lock rimosso" "rc=$RC guardia_b4=[$GR] t28=[$TR] server_t3=[$SR] t26=[$PR]" "$RC"
'''
out('s191-lancio-notte.sh', s)

# ---------------------------------------------------------------- 2. ORM t28
o = open('wp185-harness/s185-orm-coppia-t27.sh', encoding='utf-8').read()
o = rep(o, '#!/bin/bash\n# s185-orm-coppia-t27.sh — S-185:', '#!/bin/bash\n'
        '# s191-orm-coppia-t28.sh — S-191: coppia dbal+ORM t28 @ pin s184 (DOVUTA: pin nuovo; t27 era @ s182, nessuna coppia su s183).\n'
        '# COPIA DICHIARATA di ../wp185-harness/s185-orm-coppia-t27.sh (manifest s191-orm-coppia-t28-copia.diff, generatore\n'
        '# gen-notte-s191.py) coi SOLI adattamenti: harness wp191 (orm-out, verdetto s191-orm-coppia-verdetto-t28.out), PIN_ATTESO\n'
        '# default s184 60b10d857eb0343a, etichette t28/s184. Costanti E5 dal lanciatore (ORA_REF2/SENT = oracle invariato;\n'
        '# REF2 = riferimento phpr s182: Delta_norm legge s184 vs s182, companion). Il resto INVARIATO; testo S-185 sotto:\n'
        '# s185-orm-coppia-t27.sh — S-185:')
o = rep(o, 'H="$REPO/wp185-harness"', 'H="$REPO/wp191-harness"')
o = rep(o, 'VERD="$H/s185-orm-coppia-verdetto-t27.out"', 'VERD="$H/s191-orm-coppia-verdetto-t28.out"')
o = rep(o, 'PIN_ATTESO="${PIN_ATTESO:-159eee40be506171}"', 'PIN_ATTESO="${PIN_ATTESO:-60b10d857eb0343a}"')
o = rep(o, 'print("coppia-t27 @ s182 (user CPU;', 'print("coppia-t28 @ s184 (user CPU;')
o = rep(o, '{ echo "== s185 coppia t27 dbal+ORM (pin s182 MISURATO $PINM', '{ echo "== s191 coppia t28 dbal+ORM (pin s184 MISURATO $PINM')
out('s191-orm-coppia-t28.sh', o)

# ---------------------------------------------------------------- 3. WP t26
p = open('wp184-harness/s184-pair.sh', encoding='utf-8').read()
p = rep(p, '#!/bin/bash\n# s184-pair.sh <tentativo> — S-184:', '#!/bin/bash\n'
        '# s191-pair.sh <tentativo> — S-191: coppia full/media WP t26 @ pin s184 (DOVUTA: pin nuovo con TRE cambiamenti per nome — cura\n'
        '# §3.35 + O(1) v2, routing php-types S-188, §3.39 — più main 72f688c3; t25 era @ s182). COPIA DICHIARATA di\n'
        '# ../wp184-harness/s184-pair.sh (manifest s191-pair-copia.diff, generatore gen-notte-s191.py) coi SOLI adattamenti: nomi\n'
        '# s191/pair191/t26; harness wp191; PIN_ATTESO/SRV_ATTESO dal lanciatore (s184 / server s190); mediane storiche + t25=1,747\n'
        '# (banda giudizio INVARIATA [1,738;1,799]); messaggi riferiti al pin s184 (tre cause: attribuzione per braccio, mai d\'ufficio\n'
        '# alla cura O(1)). Tutto il resto INVARIATO (testo S-184/S-182/S-181 conservato sotto).\n'
        '# s184-pair.sh <tentativo> — S-184:')
p = rep(p, 'H="$REPO/wp184-harness"', 'H="$REPO/wp191-harness"')
p = rep(p, 'VERD="$H/s184-pair-verdetto-$T.out"', 'VERD="$H/s191-pair-verdetto-$T.out"')
p = rep(p, 'DONE="$OUT/pair184-$T.done"', 'DONE="$OUT/pair191-$T.done"')
p = rep(p, 'PIN_ATTESO="${PIN_ATTESO:-PLACEHOLDER-PIN-S182}"', 'PIN_ATTESO="${PIN_ATTESO:-PLACEHOLDER-PIN-S184}"')
p = rep(p, 'SRV_ATTESO="${SRV_ATTESO:-PLACEHOLDER-SRV-S182}"', 'SRV_ATTESO="${SRV_ATTESO:-PLACEHOLDER-SRV-S190}"')
p = rep(p, '== s184 coppia t25: WP full+media @ pin s182 MISURATO', '== s191 coppia t26: WP full+media @ pin s184 MISURATO')
p = rep(p, 'rc autoritativo = pair-out/pair184-{T}.done', 'rc autoritativo = pair-out/pair191-{T}.done')
p = rep(p, "vs t24=1.770: {'≤' if med_t25 <= 1.770 else '>'}", "vs t25=1.747: {'≤' if med_t25 <= 1.747 else '>'}")
p = rep(p, 't24=1.770', 't24=1.770 t25=1.747', p.count('t24=1.770'))   # le liste delle mediane storiche (commento + print)
p = rep_all(p, 'med_t25', 'med_t26')
p = rep(p, 'mediana_t25=', 'mediana_t26=')
p = rep_all(p, 'L-RT1+L-RT2', 'pin s184 (§3.35+O(1) v2, php-types, §3.39: attribuzione per braccio)')
p = rep_all(p, 'pin NUOVO s182', 'pin NUOVO s184')
assert '$this' not in p.split('s191-pair.sh <tentativo>')[1].split('\n')[0]
out('s191-pair.sh', p)

# ---------------------------------------------------------------- 4. server: catena + guardia AX + tripwire
c = open('wp189-harness/s189-catena-fetta1.sh', encoding='utf-8').read()
c = rep(c, '#!/bin/bash\n# s189-catena-fetta1.sh — S-189 FETTA 1', '#!/bin/bash\n'
        '# s191-catena-fetta1.sh — S-191 coppia server s190 (AXnew, canonica) vs s189 (AXold, stash), tentativo t3, nella finestra\n'
        '# della notte (lock TOKEN s191). COPIA DICHIARATA di ../wp189-harness/s189-catena-fetta1.sh (manifest\n'
        '# s191-catena-fetta1-copia.diff, generatore gen-notte-s191.py) coi SOLI adattamenti: harness wp191, PIN_OLD default s189,\n'
        '# script s191-*, done ga191/tw191, exit col massimo dei due rc (prima usciva sempre 0). Testo S-189 sotto:\n'
        '# s189-catena-fetta1.sh — S-189 FETTA 1')
c = rep(c, 'H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp189-harness"', 'H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp191-harness"')
c = rep(c, 'export PIN_ATTESO PIN_OLD="${PIN_OLD:-97b58fd225ca7289}"', 'export PIN_ATTESO PIN_OLD="${PIN_OLD:-80eea401b71c2be7}"')
c = rep(c, '/bin/bash "$H/s189-guardia-ax.sh" "$T"', '/bin/bash "$H/s191-guardia-ax.sh" "$T"')
c = rep(c, 'G=$(cat "$OUT/ga189-$T.done"', 'G=$(cat "$OUT/ga191-$T.done"')
c = rep(c, '/bin/bash "$H/s189-tripwire-w1.sh" "$T"', '/bin/bash "$H/s191-tripwire-w1.sh" "$T"')
c = rep(c, 'W=$(cat "$OUT/tw189-$T.done"', 'W=$(cat "$OUT/tw191-$T.done"')
c = rep(c, 'echo "guardia_$T=[$G] tripwire_$T=[$W] $(date \'+%F %T\')" > "$OUT/catena-fetta1.done"',
        'echo "guardia_$T=[$G] tripwire_$T=[$W] $(date \'+%F %T\')" > "$OUT/catena-fetta1.done"\n'
        'rg=$(printf \'%s\' "$G" | sed -n \'s/^rc=\\([0-9]*\\).*/\\1/p\'); rw=$(printf \'%s\' "$W" | sed -n \'s/^rc=\\([0-9]*\\).*/\\1/p\')\n'
        'RC=${rg:-99}; [ "${rw:-99}" -gt "$RC" ] && RC=${rw:-99}; exit "$RC"')
out('s191-catena-fetta1.sh', c)

def server_copy(src, dst, tag_old, tag_new, title):
    g = open(src, encoding='utf-8').read()
    g = rep(g, '#!/bin/bash\n', '#!/bin/bash\n' + title, 1)
    g = rep_all(g, '/wp189-harness"', '/wp191-harness"')
    g = rep_all(g, tag_old, tag_new)
    g = rep(g, "grep -q 's189' \"$MLOCK\"", "grep -q 's191' \"$MLOCK\"")
    g = rep_all(g, 'php-server-s188}', 'php-server-s189}')
    g = rep_all(g, '${PIN_OLD:-97b58fd225ca7289}', '${PIN_OLD:-80eea401b71c2be7}')
    return g

g = server_copy('wp189-harness/s189-guardia-ax.sh', 's191-guardia-ax.sh', 'ga189-', 'ga191-',
    '# s191-guardia-ax.sh <tentativo> — S-191 coppia server: guardia di NON-REGRESSIONE del throughput AXnew = pin s190 (canonica,\n'
    '# 721b7802e2d12058) vs AXold = pin s189 (stash, 80eea401b71c2be7), stesso giudice di S-189 (bande [0,95;1,05] a C=1,\n'
    '# [0,97;1,05] a C=4/8). COPIA DICHIARATA di ../wp189-harness/s189-guardia-ax.sh (manifest s191-guardia-ax-copia.diff,\n'
    '# generatore gen-notte-s191.py) coi SOLI adattamenti: harness wp191, done ga191, lock TOKEN s191, SRVOLD/PIN_OLD default s189;\n'
    '# le etichette «pin s188/s189» nel testo S-189 sotto si leggono «s189 (AXold) / s190 (AXnew)». Testo S-189 conservato:\n')
g = rep(g, 'VERD="$H/s189-guardia-ax-verdetto-$T.out"', 'VERD="$H/s191-guardia-ax-verdetto-$T.out"')
g = rep(g, 'label = {"AXold": f"php-server --axum --workers 4 pin s188 {OLD} (stash)", "AXnew": f"php-server --axum --workers 4 pin s189 {NEW} (canonica)"}',
        'label = {"AXold": f"php-server --axum --workers 4 pin s189 {OLD} (stash)", "AXnew": f"php-server --axum --workers 4 pin s190 {NEW} (canonica)"}')
g = rep(g, 'print(f"== s189 FETTA 1 guardia AX (NON-REGRESSIONE, mai record): AXnew {NEW} vs AXold {OLD}',
        'print(f"== s191 coppia server guardia AX (NON-REGRESSIONE; in banda ⇒ s190 server di record): AXnew s190 {NEW} vs AXold s189 {OLD}')
out('s191-guardia-ax.sh', g)

t = server_copy('wp189-harness/s189-tripwire-w1.sh', 's191-tripwire-w1.sh', 'tw189-', 'tw191-',
    '# s191-tripwire-w1.sh <tentativo> — S-191 coppia server: tripwire W=1 Anew = pin s190 (canonica) vs Aold = pin s189 (stash),\n'
    '# |Anew/Aold − 1| ≤ 2 % (KS-189-6). COPIA DICHIARATA di ../wp189-harness/s189-tripwire-w1.sh (manifest\n'
    '# s191-tripwire-w1-copia.diff, generatore gen-notte-s191.py) coi SOLI adattamenti: harness wp191, done tw191, lock TOKEN s191,\n'
    '# SRVOLD/PIN_OLD default s189; etichette «s188/s189» del testo S-189 sotto = «s189 (Aold) / s190 (Anew)». Testo conservato:\n')
t = rep(t, 'VERD="$H/s189-tripwire-w1-verdetto-$T.out"', 'VERD="$H/s191-tripwire-w1-verdetto-$T.out"')
t = rep(t, 'print(f"== s189 FETTA 1 tripwire W=1 (NON-REGRESSIONE, mai cifra): Anew {NEW} vs Aold {OLD}',
        'print(f"== s191 coppia server tripwire W=1 (NON-REGRESSIONE, mai cifra): Anew s190 {NEW} vs Aold s189 {OLD}')
out('s191-tripwire-w1.sh', t)
print('ok')
