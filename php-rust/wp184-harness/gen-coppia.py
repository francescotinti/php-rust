#!/usr/bin/env python3
# gen-coppia.py — genera avvio-coppia-s184.sh, s184-lancio-pair-t25.sh, s184-lancio-orm-t25.sh, s184-pair.sh come COPIE
# DICHIARATE degli omologhi S-182 (ogni sostituzione con assert di unicità; manifest = diff). Eseguito una volta in S-184.
import os
os.chdir(os.path.dirname(os.path.abspath(__file__)) + '/..')

def rep(s, old, new, n=1):
    assert s.count(old) == n, (old[:70], s.count(old))
    return s.replace(old, new)

# ---- s184-pair.sh
s = open('wp182-harness/s182-pair.sh', encoding='utf-8').read()
head_old = s[:s.index('# s182-pair.sh <tentativo>')]
head_new = '''#!/bin/bash
# s184-pair.sh <tentativo> — S-184: coppia full/media WP @ pin s182 (DOVUTA: pin nuovo, promozione COMPOSTA L-RT1+L-RT2; criterio
# s184-criterio-coppia-t25.md). COPIA DICHIARATA di wp182-harness/s182-pair.sh (manifest s184-pair-copia.diff) coi SOLI adattamenti:
# nomi s184/pair184/t25; harness wp184; pin s182 = PIN_ATTESO/SRV_ATTESO dagli stash (avvio-coppia-s184.sh); mediane storiche
# + t24=1,770 (banda giudizio INVARIATA [1,738;1,799]); messaggi GIÙ/REGRESSIONE riferiti a L-RT1+L-RT2 (pin NUOVO: la direzione
# è della leva, sotto-risoluzione dichiarata). Tutto il resto INVARIATO (testo S-182/S-181 conservato sotto).
'''
s = head_new + s[len(head_old):]
s = rep(s, 'H="$REPO/wp182-harness"', 'H="$REPO/wp184-harness"')
s = rep(s, 'VERD="$H/s182-pair-verdetto-$T.out"', 'VERD="$H/s184-pair-verdetto-$T.out"')
s = rep(s, 'DONE="$OUT/pair182-$T.done"', 'DONE="$OUT/pair184-$T.done"')
s = rep(s, 'PIN_ATTESO="${PIN_ATTESO:-PLACEHOLDER-PIN-S181}"', 'PIN_ATTESO="${PIN_ATTESO:-PLACEHOLDER-PIN-S182}"')
s = rep(s, 'SRV_ATTESO="${SRV_ATTESO:-PLACEHOLDER-SRV-S181}"', 'SRV_ATTESO="${SRV_ATTESO:-PLACEHOLDER-SRV-S182}"')
s = rep(s, 'print(f"== s182 ISTRUTTORIA t24: RIMISURA coppia WP full+media sul pin s181 MISURATO phpr={PINH} server={SRVH} tentativo={T} (criteri s132/s146/s148 EREDITATI + s182-criterio-istruttoria.md: giudizio a MEDIANA per finestra, az.rev.3 S-149) ==")',
        'print(f"== s184 coppia t25: WP full+media @ pin s182 MISURATO phpr={PINH} server={SRVH} tentativo={T} (criteri s132/s146/s148 EREDITATI + s184-criterio-coppia-t25.md: giudizio a MEDIANA per finestra, az.rev.3 S-149) ==")')
s = rep(s, 'print(f"grade=VERDICT  # derivazione meccanica dai .time; rc autoritativo = pair-out/pair182-{T}.done")',
        'print(f"grade=VERDICT  # derivazione meccanica dai .time; rc autoritativo = pair-out/pair184-{T}.done")')
s = rep(s, '''        # s182-criterio-istruttoria.md p.4 (az.rev.3 S-149): GIUDIZIO CANONICO a
        # MEDIANA per finestra vs mediane storiche t1=1,799 t2=1,738 t3=1,789 t4=1,781 t5=1,757 t6=1,771 t7=1,795 t8=1,754 t9=1,767 t10=1,760 t11=1,765 t12=1,767 t13=1,763 t14=1,761 t15=1,746 t16=1,749 t17=1,776 t18=1,769 t19=1,772 t20=1,765 t21=1,744 t23=1,799 (t22 abortita);''',
        '''        # s184-criterio-coppia-t25.md p.4 (az.rev.3 S-149): GIUDIZIO CANONICO a
        # MEDIANA per finestra vs mediane storiche t1=1,799 t2=1,738 t3=1,789 t4=1,781 t5=1,757 t6=1,771 t7=1,795 t8=1,754 t9=1,767 t10=1,760 t11=1,765 t12=1,767 t13=1,763 t14=1,761 t15=1,746 t16=1,749 t17=1,776 t18=1,769 t19=1,772 t20=1,765 t21=1,744 t23=1,799 (t22 abortita) t24=1,770;''')
s = rep(s, 'med_t24 = statistics.median(props)', 'med_t25 = statistics.median(props)')
s = rep(s, 'print(f"mediana_t24={med_t24:.3f} (N={len(props)} coppie proprie ON PULITE; mediane storiche t1=1.799 t2=1.738 t3=1.789 t4=1.781 t5=1.757 t6=1.771 t7=1.795 t8=1.754 t9=1.767 t10=1.760 t11=1.765 t12=1.767 t13=1.763 t14=1.761 t15=1.746 t16=1.749 t17=1.776 t18=1.769 t19=1.772 t20=1.765 t21=1.744 t23=1.799)")',
        'print(f"mediana_t25={med_t25:.3f} (N={len(props)} coppie proprie ON PULITE; mediane storiche t1=1.799 t2=1.738 t3=1.789 t4=1.781 t5=1.757 t6=1.771 t7=1.795 t8=1.754 t9=1.767 t10=1.760 t11=1.765 t12=1.767 t13=1.763 t14=1.761 t15=1.746 t16=1.749 t17=1.776 t18=1.769 t19=1.772 t20=1.765 t21=1.744 t23=1.799 t24=1.770; riferimento L-RT1+L-RT2: t24=1.770)")')
s = rep(s, 'if MED_LO <= med_t24 <= MED_HI:', 'if MED_LO <= med_t25 <= MED_HI:')
s = rep(s, 'print(f"giudizio_MEDIANA (canonico, criterio istruttoria p.4): COMPATIBILE (mediana in [{MED_LO}; {MED_HI}]) — nessun claim")',
        'print(f"giudizio_MEDIANA (canonico, criterio coppia-t25 p.4): COMPATIBILE (mediana in [{MED_LO}; {MED_HI}]) — L-RT1+L-RT2 sotto-risoluzione su WP (dichiarato); vs t24=1.770: {\'≤\' if med_t25 <= 1.770 else \'>\'} (direzione, non cifra)")')
s = rep(s, 'elif med_t24 < MED_LO:', 'elif med_t25 < MED_LO:')
s = rep(s, 'print(f"giudizio_MEDIANA (canonico, criterio istruttoria p.4): direzione GIU\' SEGNALATA (mediana {med_t24:.3f} < {MED_LO}) — stesso pin s181 (rimisura di istruttoria): lettura, non è una leva")',
        'print(f"giudizio_MEDIANA (canonico, criterio coppia-t25 p.4): direzione GIU\' SEGNALATA (mediana {med_t25:.3f} < {MED_LO}) — pin NUOVO s182: coerente con L-RT1+L-RT2 (attesa ≤0), magnitudine non ripartita")')
s = rep(s, 'print(f"giudizio_MEDIANA (canonico, criterio istruttoria p.4): REGRESSIONE SEGNALATA (mediana {med_t24:.3f} > {MED_HI}) — in finestra pulita = REPERTO contro L-CR1 (criterio istruttoria p.5)")',
        'print(f"giudizio_MEDIANA (canonico, criterio coppia-t25 p.4): REGRESSIONE SEGNALATA (mediana {med_t25:.3f} > {MED_HI}) — in finestra pulita = REPERTO contro L-RT1+L-RT2 (criterio coppia-t25 p.5: regola 4, indagine prima di ogni leva)")')
open('wp184-harness/s184-pair.sh', 'w', encoding='utf-8').write(s)

# ---- s184-lancio-pair-t25.sh
l = open('wp182-harness/s182-lancio-pair-t24.sh', encoding='utf-8').read()
head_old = l[:l.index('set -u\n')]
head_new = '''#!/bin/bash
# s184-lancio-pair-t25.sh — COPPIA t25 @ pin s182 (criterio s184-criterio-coppia-t25.md p.3): COPIA DICHIARATA di
# ../wp182-harness/s182-lancio-pair-t24.sh (manifest s184-lancio-pair-copia.diff) coi SOLI adattamenti: nomi s184/t25/pair184,
# lock TOKEN s184, s184-pair.sh t25, PIÙ il gate sentinella «processo singolo >50 % CPU» ×2 (az.rev. S-183 #3, come
# s184-lancio-rt2.sh) dopo E2. Il resto INVARIATO (testo S-182 conservato sotto).
'''
l = head_new + l[len(head_old):]
l = rep(l, 'H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp182-harness"', 'H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp184-harness"')
l = rep(l, 'LOG="$H/pair-out/lancio-t24.log"', 'LOG="$H/pair-out/lancio-t25.log"')
l = rep(l, 'echo "s183 istruttoria t24 (apparato S-182) $(date \'+%F %T\') pid=$$" > "$MLOCK"', 'echo "s184 coppia t25 @ pin s182 $(date \'+%F %T\') pid=$$" > "$MLOCK"')
l = rep(l, 'echo "$(date \'+%F %T\') lock scritto: $MLOCK (TOKEN s183)" >> "$LOG"', 'echo "$(date \'+%F %T\') lock scritto: $MLOCK (TOKEN s184)" >> "$LOG"')
anchor = '''  echo "$(date '+%F %T') E2 calma CPU: tentativo $t FALLITO (campioni:$smp %)" >> "$LOG"
done
'''
add = '''# GATE sentinella (az.rev. S-183 #3): nessun processo singolo > 50 % CPU al SECONDO campione di `top -l 2`, ×2 consecutivi
hog(){ /usr/bin/top -l 2 -n 1 -o cpu -stats cpu,command 2>/dev/null | tail -1 | awk '$1+0 > 50 {print}'; }
hc=0
while [ "$hc" -lt 2 ]; do hg=$(hog); if [ -z "$hg" ]; then hc=$((hc+1)); else hc=0; echo "$(date '+%F %T') processo singolo >50 % CPU: $hg — attesa" >> "$LOG"; fi; [ "$hc" -lt 2 ] && sleep 30; done
'''
l = rep(l, anchor, anchor + add)
l = rep(l, '( WD="$H/pair-out/watchdog-t24.txt"; PD="$H/pair-out/pair182-t24.done"', '( WD="$H/pair-out/watchdog-t25.txt"; PD="$H/pair-out/pair184-t25.done"')
l = rep(l, 'Q="$H/pair-out/quiet-decl-t24.txt"', 'Q="$H/pair-out/quiet-decl-t25.txt"')
l = rep(l, '{ echo "== dichiarazione finestra quieta t24 $(date \'+%F %T\') =="', '{ echo "== dichiarazione finestra quieta t25 $(date \'+%F %T\') =="')
l = rep(l, 'echo "$(date \'+%F %T\') quiete raggiunta — dichiarazione in quiet-decl-t24.txt — lancio pair t24" >> "$LOG"\nexec /bin/bash "$H/s182-pair.sh" t24',
        'echo "$(date \'+%F %T\') quiete raggiunta — dichiarazione in quiet-decl-t25.txt — lancio pair t25" >> "$LOG"\nexec /bin/bash "$H/s184-pair.sh" t25')
open('wp184-harness/s184-lancio-pair-t25.sh', 'w', encoding='utf-8').write(l)

# ---- s184-lancio-orm-t25.sh
o = open('wp182-harness/s182-lancio-orm-t24.sh', encoding='utf-8').read()
o = rep(o, '# s182-lancio-orm-t24.sh — ISTRUTTORIA S-182: attende pair182-t24.done; SOLO se rc=0',
        '# s184-lancio-orm-t25.sh — COPPIA S-184 @ pin s182: attende pair184-t25.done; SOLO se rc=0')
o = rep(o, '# COPIA DICHIARATA di s181-lancio-orm-t23.sh (manifest s182-lancio-orm-copia.diff): soli nomi s182/t24/pair182.',
        '# COPIA DICHIARATA di ../wp182-harness/s182-lancio-orm-t24.sh (manifest s184-lancio-orm-copia.diff): soli nomi s184/t25/pair184 + archivio del verdetto ORM in wp184-harness/orm-out.')
o = rep(o, 'REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$REPO/wp182-harness"', 'REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$REPO/wp184-harness"')
o = rep(o, 'LOG="$H/orm-out/lancio-orm-t24.log"', 'LOG="$H/orm-out/lancio-orm-t25.log"')
o = rep(o, 'PD="$H/pair-out/pair182-t24.done"', 'PD="$H/pair-out/pair184-t25.done"')
o = rep(o, 'echo "$(date \'+%F %T\') pair t24 NON rc=0 ($(cat "$PD")) — ORM NON lanciato" >> "$LOG"', 'echo "$(date \'+%F %T\') pair t25 NON rc=0 ($(cat "$PD")) — ORM NON lanciato" >> "$LOG"')
o = rep(o, 'SPD=/private/tmp/phpr-s182-orm; mkdir -p "$SPD"', 'SPD=/private/tmp/phpr-s184-orm; mkdir -p "$SPD"')
o = rep(o, 'PIN_ATTESO="${PIN_ATTESO:?pin s181 atteso (istruttoria S-182)}" MAPPA_SP="$SPD" exec /bin/bash "$REPO/wp176-harness/s176-orm-coppia.sh"',
        'PIN_ATTESO="${PIN_ATTESO:?pin s182 atteso (coppia S-184)}" MAPPA_SP="$SPD" /bin/bash "$REPO/wp176-harness/s176-orm-coppia.sh"\nrc=$?\ncp "$REPO/wp176-harness/s176-orm-coppia-verdetto.out" "$H/orm-out/s184-orm-coppia-verdetto-t25.out" 2>/dev/null\ncp "$REPO/wp176-harness/orm-out/rimisura.done" "$H/orm-out/rimisura-t25.done" 2>/dev/null\necho "$(date \'+%F %T\') ORM fine rc=$rc ($(cat "$REPO/wp176-harness/orm-out/rimisura.done" 2>/dev/null)) — verdetto archiviato in wp184-harness/orm-out" >> "$LOG"\nexit "$rc"')
open('wp184-harness/s184-lancio-orm-t25.sh', 'w', encoding='utf-8').write(o)

# ---- avvio-coppia-s184.sh
a = open('wp182-harness/avvio-istruttoria-s182.sh', encoding='utf-8').read()
head_old = a[:a.index('set -u\n')]
head_new = '''#!/bin/bash
# avvio-coppia-s184.sh — S-184 COPPIA t25 @ pin s182 (criterio s184-criterio-coppia-t25.md p.3): derivato da
# ../wp182-harness/avvio-istruttoria-s182.sh (soli nomi: stash s182, harness wp184, lanciatori t25). Attende il DRENAGGIO della CI
# (runner morto ×2 a 60 s, niente cargo/rustc), verifica stash phpr-s182/php-server-s182 == canonica, esporta PIN_ATTESO/SRV_ATTESO e
# daemonizza s184-lancio-pair-t25.sh (→ s184-pair.sh t25, che scrive il lock TOKEN s184) e s184-lancio-orm-t25.sh (ORM dopo pair rc=0).
# Il nome NON contiene «harness/s1NN-» né «phpr» di proposito.
'''
a = head_new + a[len(head_old):]
a = rep(a, 'H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp182-harness"', 'H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp184-harness"')
a = rep(a, 'DZ="/Volumes/Extreme Pro/Claude/wp52-harness/daemonize.pl"', 'DZ="/Volumes/Extreme Pro/Claude/wp58-harness/daemonize.pl"')
a = rep(a, 'LOG="$H/pair-out/avvio-istruttoria.log"', 'LOG="$H/pair-out/avvio-coppia.log"')
a = rep(a, '[ -s "$STASH/phpr-s181" ] && [ -s "$STASH/php-server-s181" ] || { l "stash s181 assenti — istruttoria NON lanciata"; exit 5; }',
        '[ -s "$STASH/phpr-s182" ] && [ -s "$STASH/php-server-s182" ] || { l "stash s182 assenti — coppia NON lanciata"; exit 5; }')
a = rep(a, 'export PIN_ATTESO=$(shasum -a 256 "$STASH/phpr-s181" | cut -c1-16)', 'export PIN_ATTESO=$(shasum -a 256 "$STASH/phpr-s182" | cut -c1-16)')
a = rep(a, 'export SRV_ATTESO=$(shasum -a 256 "$STASH/php-server-s181" | cut -c1-16)', 'export SRV_ATTESO=$(shasum -a 256 "$STASH/php-server-s182" | cut -c1-16)')
a = rep(a, 'l "binari canonici ≠ stash s181 (phpr $PC vs $PIN_ATTESO · server $SC vs $SRV_ATTESO) — istruttoria NON lanciata"; exit 9; fi',
        'l "binari canonici ≠ stash s182 (phpr $PC vs $PIN_ATTESO · server $SC vs $SRV_ATTESO) — coppia NON lanciata"; exit 9; fi')
a = rep(a, 'l "pin s181 phpr=$PIN_ATTESO server=$SRV_ATTESO == canonico — lancio pair t24 + attesa ORM"', 'l "pin s182 phpr=$PIN_ATTESO server=$SRV_ATTESO == canonico — lancio pair t25 + attesa ORM"')
a = rep(a, 'perl "$DZ" "$H/pair-out/lancio-t24.log" /bin/bash "$H/s182-lancio-pair-t24.sh"\nperl "$DZ" "$H/orm-out/lancio-orm-t24.log" /bin/bash "$H/s182-lancio-orm-t24.sh"',
        'perl "$DZ" "$H/pair-out/lancio-t25-dz.log" /bin/bash "$H/s184-lancio-pair-t25.sh"\nperl "$DZ" "$H/orm-out/lancio-orm-t25-dz.log" /bin/bash "$H/s184-lancio-orm-t25.sh"')
a = rep(a, '''l "lanciati (daemonize): $(pgrep -fl 's182-lancio-(pair|orm)-t24' | grep -v pgrep | awk '{print $1}' | tr '\\n' ' ')"''',
        '''l "lanciati (daemonize): $(pgrep -fl 's184-lancio-(pair|orm)-t25' | grep -v pgrep | awk '{print $1}' | tr '\\n' ' ')"''')
open('wp184-harness/avvio-coppia-s184.sh', 'w', encoding='utf-8').write(a)
print('ok')
