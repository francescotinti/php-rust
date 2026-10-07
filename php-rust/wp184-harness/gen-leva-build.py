#!/usr/bin/env python3
# gen-leva-build.py — genera s184-leva-build.sh come COPIA DICHIARATA di ../wp182-harness/s183-leva-build.sh
# (ogni sostituzione con assert di unicità; manifest = diff dei due script). Eseguito una volta in S-184.
import os
os.chdir(os.path.dirname(os.path.abspath(__file__)) + '/..')
src = open('wp182-harness/s183-leva-build.sh', encoding='utf-8').read()

def rep(s, old, new, n=1):
    assert s.count(old) == n, (old[:70], s.count(old))
    return s.replace(old, new)

s = src
head_old = s[:s.index('set -u\n')]
head_new = '''#!/bin/bash
# s184-leva-build.sh — bracci Z (gemello del pin s181) e B (tree = L-RT1 + L-RT2) + MUTANTE (criterio s184-criterio-rt2.md p.3/p.6):
# COPIA DICHIARATA di ../wp182-harness/s183-leva-build.sh (manifest s184-leva-build-copia.diff) coi SOLI adattamenti: token s184,
# tag s184-leva, fixture fx-rt2 (NUOVA) in riferimento + parità + bersaglio del mutante, MUTANTE = «ammissione del Ret fuso SENZA i bit
# di forma» (shape/flags/ret_cell tolti dall'ammissione; esito esatto: M diverge dall'oracle su fx-rt2 su ≥1 riga e TUTTE le righe
# cambiate sono etichettate RT2-), DISASM come GATE (az.rev. S-183 #2): |Δbl| ≤ 10 e |Δsp_refs| ≤ 40 di B vs tree-RT1 9ef70777
# (ab-out/s183-leva/phpr-B), altrimenti rc=5 «meccanismo da rileggere». Testo S-183/S-181 conservato sotto:
'''
s = head_new + s[len(head_old):]
s = rep(s, 'H="$REPO/php-rust/wp182-harness"; H1="$REPO/php-rust/wp181-harness"; OUT="$H/ab-out/s183-leva"; mkdir -p "$OUT"',
        'H="$REPO/php-rust/wp184-harness"; H1="$REPO/php-rust/wp181-harness"; H3="$REPO/php-rust/wp182-harness"; OUT="$H/ab-out/s184-leva"; mkdir -p "$OUT"')
s = rep(s, 'VERD="$H/s183-leva-build-verdetto.out"; DONE="$H/ab-out/s183-leva.done"', 'VERD="$H/s184-leva-build-verdetto.out"; DONE="$H/ab-out/s184-leva.done"')
s = rep(s, 'FXRT="$H/fx-rt1.php"', 'FXRT="$H3/fx-rt1.php"; FXRT2="$H/fx-rt2.php"; TREE_RT1="$H3/ab-out/s183-leva/phpr-B"')
s = rep(s, 'SRC="/Volumes/Extreme Pro/Claude/s183-leva"; BUNDLE_MP="$HOME/Claude/phpr-target"; TGT="$BUNDLE_MP/s183-leva-tgt"',
        'SRC="/Volumes/Extreme Pro/Claude/s184-leva"; BUNDLE_MP="$HOME/Claude/phpr-target"; TGT="$BUNDLE_MP/s184-leva-tgt"')
s = rep(s, 'grep -qw s183 "$LOCK" 2>/dev/null || { note "rc=9 lock s183 assente (per TOKEN)"; fin 9; }',
        'grep -qw s184 "$LOCK" 2>/dev/null || { note "rc=9 lock s184 assente (per TOKEN)"; fin 9; }')
s = rep(s, 'for f in "$FX2" "$FX1" "$H2/fx-sl1.php" "$H2/fx-sl2.php" "$H2/fx-sl3.php" "$FXCR" "$FXRT"; do',
        'for f in "$FX2" "$FX1" "$H2/fx-sl1.php" "$H2/fx-sl2.php" "$H2/fx-sl3.php" "$FXCR" "$FXRT" "$FXRT2" "$TREE_RT1"; do')
s = rep(s, 'note "== s183 bracci Z/B/M leva L-RT1 — riferimento A = pin s181 $PH',
        'note "== s184 bracci Z/B/M leva COMPOSTA L-RT1+L-RT2 — riferimento A = pin s181 $PH, tree-RT1 $(shasum -a 256 "$TREE_RT1" | cut -c1-8)')
anchor = 'diff -q "$OUT/fxrt1-oracle.out" "$OUT/fxrt1-pin.out" > /dev/null || { note "rc=7 fx-rt1: pin ≠ oracle (fixture non bilaterale a monte)"; fin 7; }\n'
add = '''"$ORACLE" -d log_errors=0 -d display_errors=1 "$FXRT2" > "$OUT/fxrt2-oracle.out" 2>&1
perl -e 'alarm 120; exec @ARGV or die' -- "$PIN" "$FXRT2" > "$OUT/fxrt2-pin.out" 2>&1
grep -q "FX-RT2 DONE" "$OUT/fxrt2-pin.out" || { note "rc=7 riferimento: marcatore FX-RT2 assente sul pin"; fin 7; }
diff -q "$OUT/fxrt2-oracle.out" "$OUT/fxrt2-pin.out" > /dev/null || { note "rc=7 fx-rt2: pin ≠ oracle (fixture non bilaterale a monte)"; fin 7; }
'''
s = rep(s, anchor, anchor + add)
s = rep(s, '· fx-cr1 pin==oracle BYTE-ID (righe ACE: $NACE)"', '· fx-cr1 pin==oracle BYTE-ID (righe ACE: $NACE) · fx-rt1/fx-rt2 pin==oracle BYTE-ID"')
for lab in ('Z', 'B', 'M'):
    s = rep(s, 'ab-out/s183-leva/build-%s.log' % lab, 'ab-out/s184-leva/build-%s.log' % lab)
# mutante
RUNRS = 'crates/php-runtime/src/vm/' + 'run' + '.rs'
old_mut = s[s.index('# M = B + mutante: fast path di Ret SENZA le note GC'):s.index('note "M: binario $MH (B + mutante: Ret fast path senza note GC)"')]
new_mut = '''# M = B + mutante: ammissione del Ret FUSO senza i bit di FORMA (shape/flags/ret_cell) — sostituzione sull'ARCHIVIO, mai sul tree — DEVE divergere su fx-rt2 (righe RT2-)
python3 - "$SRC/php-rust/%s" <<'PY' || { echo "mutante non applicato" >> "$OUT/mutante.err"; }
import sys
p = sys.argv[1]; s = open(p).read()
old = ('                        f.func.ret_shape == 0\\n'
       '                            && f.flags.bits() == 0\\n'
       '                            && f.ret_cell.is_none()\\n'
       '                            && f.this.is_none()\\n')
assert s.count(old) == 1, 'sito ammissione Ret fuso non unico'
s = s.replace(old, '                        // MUTANTE s184: ammissione SENZA shape/flags/ret_cell\\n                        f.this.is_none()\\n')
open(p, 'w').write(s)
print('mutante applicato')
PY
[ -e "$OUT/mutante.err" ] && { note "rc=7 mutante non applicabile (sito non trovato)"; fin 7; }
grep -q 'MUTANTE s184' "$SRC/php-rust/%s" || { note "rc=7 mutante NON scritto (verifica grep dopo la patch, lezione S-183 #3)"; fin 7; }
MH=$(build M) || { note "rc=4 M: build FALLITA (ab-out/s184-leva/build-M.log)"; fin 4; }
[ "$MH" != "$BH" ] || { note "rc=7 M: binario == B (mutante NON entrato)"; fin 7; }
''' % (RUNRS, RUNRS)
s = s.replace(old_mut, new_mut)
s = rep(s, 'note "M: binario $MH (B + mutante: Ret fast path senza note GC)"', 'note "M: binario $MH (B + mutante: ammissione del Ret fuso senza shape/flags/ret_cell)"')
s = rep(s, 'bilat fxrt1 "$FXRT" "FX-RT1 DONE" -d log_errors=0 -d display_errors=1\n',
        'bilat fxrt1 "$FXRT" "FX-RT1 DONE" -d log_errors=0 -d display_errors=1\nbilat fxrt2 "$FXRT2" "FX-RT2 DONE" -d log_errors=0 -d display_errors=1\n')
old_bite = s[s.index("# mutante: DEVE divergere dall'oracle su fx-cr1"):s.index('# disasm agli atti (p.6)')]
new_bite = '''# mutante: DEVE divergere dall'oracle su fx-rt2 con ≥1 riga cambiata e TUTTE le righe cambiate etichettate RT2- (esito esatto, criterio-rt2 p.3)
perl -e 'alarm 120; exec @ARGV or die' -- "$OUT/phpr-M" "$FXRT2" > "$OUT/fxrt2-M.out" 2>&1
diff "$OUT/fxrt2-oracle.out" "$OUT/fxrt2-M.out" > "$OUT/fxrt2-M.diff" || true
MCH=$(grep -c '^[<>]' "$OUT/fxrt2-M.diff"); MBAD=$(grep '^[<>]' "$OUT/fxrt2-M.diff" | grep -vc '^[<>] RT2-')
if [ "$MCH" -gt 0 ] && [ "$MBAD" -eq 0 ]; then
  note "MUTANTE: morde — fx-rt2 righe cambiate $MCH, tutte RT2- (non RT2- $MBAD): etichette $(grep '^[<>]' "$OUT/fxrt2-M.diff" | sed 's/^[<>] \\(RT2-[a-z-]*\\).*/\\1/' | sort -u | tr '\\n' ' ') — la fixture presidia l'ammissione"
else
  note "MUTANTE: NON morde nella forma attesa (fx-rt2 cambiate $MCH, non RT2- $MBAD; ab-out/s184-leva/fxrt2-M.diff) -> rc=3"; [ "$RC" -eq 0 ] && RC=3
fi

'''
s = s.replace(old_bite, new_bite)
old_dis = s[s.index('note "DISASM run_loop: pin s181'):s.index('rm -rf "$TGT" "$SRC"')]
new_dis = '''DPIN=$(dis "$PIN" pin); DZ=$(dis "$OUT/phpr-Z" Z); DB=$(dis "$OUT/phpr-B" B); DT=$(dis "$TREE_RT1" treeRT1)
note "DISASM run_loop: pin s181 $DPIN · Z $DZ · tree-RT1 9ef70777 $DT · B $DB"
num(){ echo "$1" | sed -n "s/.*$2=\\([0-9]*\\).*/\\1/p"; }
BLB=$(num "$DB" bl); BLT=$(num "$DT" bl); SPB=$(num "$DB" sp_refs); SPT=$(num "$DT" sp_refs)
if [ -n "$BLB" ] && [ -n "$BLT" ] && [ -n "$SPB" ] && [ -n "$SPT" ]; then
  DBL=$((BLB-BLT)); DSP=$((SPB-SPT))
  if [ "${DBL#-}" -le 10 ] && [ "${DSP#-}" -le 40 ]; then note "GATE disasm (az.rev. S-183 #2): Δbl=$DBL (|·|≤10) Δsp_refs=$DSP (|·|≤40) vs tree-RT1 ⇒ RISPETTATO"; else note "GATE disasm: Δbl=$DBL Δsp_refs=$DSP FUORI tolleranza (|Δbl|≤10, |Δsp_refs|≤40) ⇒ meccanismo da rileggere, niente misura -> rc=5"; [ "$RC" -eq 0 ] && RC=5; fi
else
  note "GATE disasm: conteggi non leggibili (B='$DB' tree='$DT') -> rc=5"; [ "$RC" -eq 0 ] && RC=5
fi
'''
s = s.replace(old_dis, new_dis)
s = rep(s, 'note "ESITO rc=$RC (0 = B a parità e mutante che morde: bracci pronti; 2 = B diverge; 3 = mutante non morde) fine',
        'note "ESITO rc=$RC (0 = B a parità, mutante che morde, disasm in tolleranza: bracci pronti; 2 = B diverge; 3 = mutante non morde; 5 = disasm fuori tolleranza) fine')
open('wp184-harness/s184-leva-build.sh', 'w', encoding='utf-8').write(s)
print('ok')
