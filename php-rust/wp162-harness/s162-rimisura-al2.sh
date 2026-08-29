#!/bin/bash
# s162-rimisura-al2.sh — rimisura AL2 su stash FERMI (criterio
# s162-criterio-al2-rimisura.md, PRE-REGISTRATO; rev. S-161 #1-#2).
# Bracci: A=phpr-s161-gemelloA(==pin s160) vs B=phpr-s161(==pin s161).
# Giudice m-missload N=10M; R=5 ABAB, floor3, mediane, rumore drop-1
# (matematica s158 INVARIATA — struttura+judge math COPIA DICHIARATA di
# s161-sonda-af1.sh FASE 2/3, parser adattato al sito autoload).
# Esiti a FILE: s162-al2-rimisura-verdetto.out; rc SOLO da al2rim-out/rim.done.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp162-harness"; OUT="$H/al2rim-out"; mkdir -p "$OUT"
VERD="$H/s162-al2-rimisura-verdetto.out"
LOCK=/private/tmp/phpr-measure.lock
ST="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
A="$ST/phpr-s161-gemelloA"; BB="$ST/phpr-s161"
AEXP=ceeb6e76; BEXP=ec0a636a
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
MR="$REPO/wp161-harness/m-missload.php"; EMP="$REPO/wp161-harness/empty.php"
: > "$VERD"; RCF="$OUT/rim.done"; rm -f "$RCF"
fin(){ echo "rc=$1 $(date +%T)" > "$RCF"; exit "$1"; }
ls_sentinel(){ pgrep -fl 'language[_-]server|Antigravity|rust-analyzer|intelephense|serena|gopls|pylsp|solargraph' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' '; }

# verifica POSITIVA dei path d'ingresso (emenda §3 praticata)
for f in "$QUIESCE" "$MR" "$EMP" "$A" "$BB"; do
  [ -s "$f" ] || { echo "rc=7 path d'ingresso MANCANTE: $f" >> "$VERD"; fin 7; }
done
grep -qi "s-162\|s162" "$LOCK" 2>/dev/null || { echo "rc=9 lock assente/altrui" | tee -a "$VERD"; fin 9; }
AM=$(shasum -a 256 "$A" | cut -c1-8); BM=$(shasum -a 256 "$BB" | cut -c1-8)
[ "$AM" = "$AEXP" ] || { echo "rc=9 stash A $AM != $AEXP" | tee -a "$VERD"; fin 9; }
[ "$BM" = "$BEXP" ] || { echo "rc=9 stash B $BM != $BEXP" | tee -a "$VERD"; fin 9; }
echo "== s162 rimisura L-AL2 su stash FERMI (criterio s162-criterio-al2-rimisura.md; bracci MISURATI A=$AM(gemelloA==pin s160) B=$BM(==pin s161); giudice m-missload N=10M) ==" >> "$VERD"
echo "sentinella language-server inizio finestra: $(ls_sentinel)" >> "$VERD"

"$QUIESCE" "$OUT/quiesce.rc" > "$OUT/quiesce.log" 2>&1
QRC=$(cat "$OUT/quiesce.rc" 2>/dev/null || echo MANCANTE)
echo "quiescenza rimisura: rc=$QRC" >> "$VERD"
[ "$QRC" = 0 ] || { echo "rc=7 quiescenza fallita" >> "$VERD"; fin 7; }
ucpu() { { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$1" "$2" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3() { local a b c; a=$(ucpu "$1" "$2"); b=$(ucpu "$1" "$2"); c=$(ucpu "$1" "$2"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }
"$A" "$MR" > "$OUT/rim-A.out" 2>&1; "$BB" "$MR" > "$OUT/rim-B.out" 2>&1
grep -q "^ML-OK 10000000$" "$OUT/rim-A.out" || { echo "rc=2 marcatore ML-OK 10000000 assente (A)" >> "$VERD"; fin 2; }
grep -q "^ML-OK 10000000$" "$OUT/rim-B.out" || { echo "rc=2 marcatore ML-OK 10000000 assente (B)" >> "$VERD"; fin 2; }
diff -q "$OUT/rim-A.out" "$OUT/rim-B.out" > /dev/null || { echo "rc=2 output A/B DIVERGE" >> "$VERD"; fin 2; }
FA=$(floor3 "$A" "$EMP"); FB=$(floor3 "$BB" "$EMP")
TSV="$OUT/rimisura.tsv"; : > "$TSV"
echo "rimisura: giudice m-missload N=10000000 floor_A=$FA floor_B=$FB" >> "$VERD"
for i in 1 2 3 4 5; do
  if [ $((i % 2)) -eq 1 ]; then TA=$(ucpu "$A" "$MR"); TB=$(ucpu "$BB" "$MR"); ord=AB
  else TB=$(ucpu "$BB" "$MR"); TA=$(ucpu "$A" "$MR"); ord=BA; fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$i" "$TA" "$TB" "$FA" "$FB" "$ord" >> "$TSV"
  echo "  coppia$i [$ord]: rawA=$TA rawB=$TB" >> "$VERD"
done

/opt/homebrew/bin/python3 - "$TSV" >> "$VERD" <<'PYT'
import sys
N = 10_000_000
rows = [l.rstrip("\n").split("\t") for l in open(sys.argv[1])]
na = [(float(t[1])-float(t[3]))/N*1e9 for t in rows]
nb = [(float(t[2])-float(t[4]))/N*1e9 for t in rows]
def med(v):
    s = sorted(v); n = len(s)
    return s[n//2] if n % 2 else (s[n//2-1]+s[n//2])/2
def trange(v):
    m = med(v)
    w = sorted(v, key=lambda x: (abs(x - m), x))[:-1]
    return max(w) - min(w)
ma, mb = med(na), med(nb)
D = ma - mb
ra, rb = trange(na), trange(nb)
noise = max(ra, rb)
print(f"RIMISURA m-missload: A={ma:.1f} B={mb:.1f} ns/miss D={D:+.1f} (rumore drop-1: A'={ra:.1f} B'={rb:.1f})")
print(f"drift vs R=5 s161 (+5.0): {D-5.0:+.1f} · vs post-pin (+3.0): {D-3.0:+.1f} (banda rumore {noise:.1f})")
if D <= noise or D <= 0:
    print(f"DECISIONE p.5: NESSUNA cifra (D={D:+.1f} non sopra il rumore col segno +) — registro resta «direzione firmata, magnitudine NON tarata», si dichiara")
    sys.exit(0)
print(f"REGISTRO p.6: coeff sito-autoload = {D:.1f} ± {noise:.1f} ns/miss su stash FERMI s160/s161 — TABELLA PER-SITO: refl 12.0±2.5 · arrfilter 17.0±2.0 · autoload {D:.1f}±{noise:.1f}")
if abs(D - 5.0) > 2.0 + noise:
    print(f"INTORNO p.7: |D-5.0|={abs(D-5.0):.1f} > 2.0+rumore({noise:.1f}) — reperto FUORI-INTORNO a verbale (nessuna taratura post-hoc)")
else:
    print(f"INTORNO p.7: |D-5.0|={abs(D-5.0):.1f} <= 2.0+rumore({noise:.1f}) — cifra COERENTE col R=5 di promozione")
PYT
echo "sentinella language-server fine finestra: $(ls_sentinel)" >> "$VERD"
echo "ESITO: rc=0 (rimisura + decisione agli atti)" >> "$VERD"
fin 0
