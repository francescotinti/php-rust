#!/bin/bash
# s159-sonda.sh — sonda surplus m-refl (criterio s159-criterio-sonda.md, PRE-
# REGISTRATO). FASE 1 CONTEGGI: probe census A (tree HEAD − patch L-RF2
# inversa) vs B (tree HEAD), driver m-refl-census.php N=200k, attese ESATTE
# p.4. FASE 2 RIMISURA: A/B R=5 ABAB su stash gemelloA vs pin (giudice m-refl
# N=10M, matematica s158 INVARIATA: mediane, floor3, rumore drop-1). FASE 3
# TARATURA: coeff = D/2 ns/alloc-sito; surplus = D − 13,8; drift vs +29/+21.
# Esiti a FILE: verdetto s159-sonda-verdetto.out; rc SOLO da sonda-out/sonda.done.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"  # .cargo/bin: fix rc=7 del primo lancio (cargo not found), dichiarato nel verdetto
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp159-harness"; OUT="$H/sonda-out"; mkdir -p "$OUT"
VERD="$H/s159-sonda-verdetto.out"
LOCK=/private/tmp/phpr-measure.lock
ST="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
A="$ST/phpr-s158-gemelloA"; BB="$ST/phpr-s158"
AEXP=369ee345; BEXP=92b0aea3
PATCH="$REPO/wp158-harness/s158-refl2-edit.patch"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
SRC="/private/tmp/s159-sonda"
: > "$VERD"; RCF="$OUT/sonda.done"; rm -f "$RCF"
fin(){ echo "rc=$1 $(date +%T)" > "$RCF"; exit "$1"; }
ls_sentinel(){ pgrep -fl 'language[_-]server|Antigravity|rust-analyzer|intelephense|serena|gopls|pylsp|solargraph' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' '; }

grep -qi "s-159\|s159" "$LOCK" 2>/dev/null || { echo "rc=9 lock assente/altrui" | tee -a "$VERD"; fin 9; }
AM=$(shasum -a 256 "$A" | cut -c1-8); BM=$(shasum -a 256 "$BB" | cut -c1-8)
[ "$AM" = "$AEXP" ] || { echo "rc=9 stash A $AM != $AEXP" | tee -a "$VERD"; fin 9; }
[ "$BM" = "$BEXP" ] || { echo "rc=9 stash B $BM != $BEXP" | tee -a "$VERD"; fin 9; }
echo "== s159 sonda surplus m-refl (criterio s159-criterio-sonda.md; bracci rimisura MISURATI A=$AM(gemelloA s157) B=$BM(==pin s158); conteggi su probe census A/B da tree HEAD∓patch) ==" >> "$VERD"
echo "sentinella language-server inizio finestra: $(ls_sentinel)" >> "$VERD"

# ---------- FASE 1: CONTEGGI ----------
rm -rf "$SRC"; mkdir -p "$SRC/B/php-rust" "$SRC/A"
for f in crates Cargo.toml Cargo.lock rust-toolchain.toml .cargo; do
  cp -R "$REPO/$f" "$SRC/B/php-rust/" || { echo "rc=7 copia tree ($f)" >> "$VERD"; fin 7; }
done
cp -R "$SRC/B/php-rust" "$SRC/A/php-rust" || { echo "rc=7 copia A" >> "$VERD"; fin 7; }
( cd "$SRC/A" && patch -R -p1 --no-backup-if-mismatch < "$PATCH" ) > "$OUT/patchR.log" 2>&1 \
  || { echo "rc=6 patch -R fallita (log sonda-out/patchR.log)" >> "$VERD"; fin 6; }
NP=$(grep -c "^patching file" "$OUT/patchR.log"); NREJ=$(/usr/bin/find "$SRC/A" -name "*.rej" | wc -l | tr -d ' ')
{ [ "$NP" = 2 ] && [ "$NREJ" = 0 ]; } || { echo "rc=6 patch guardia: patching=$NP (atteso 2) rej=$NREJ (atteso 0)" >> "$VERD"; fin 6; }
echo "patch inversa: 2 file, zero rej (log agli atti)" >> "$VERD"
for ARM in A B; do
  ( cd "$SRC/$ARM/php-rust" && SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$SRC/tgt$ARM" \
    cargo build --release -p php-cli --features mem-census ) > "$OUT/build-$ARM.log" 2>&1 \
    || { echo "rc=7 build probe-$ARM fallita (log sonda-out/build-$ARM.log)" >> "$VERD"; fin 7; }
done
PA="$SRC/tgtA/release/phpr"; PB="$SRC/tgtB/release/phpr"
echo "probe MISURATI: A=$(shasum -a 256 "$PA" | cut -c1-16) B=$(shasum -a 256 "$PB" | cut -c1-16) (census -p php-cli --features mem-census, target dedicati)" >> "$VERD"
for ARM in A B; do
  P="$SRC/tgt$ARM/release/phpr"
  for Rr in 1 2; do
    RAW="$OUT/census-$ARM-r$Rr.raw"; rm -f "$RAW"
    PHPR_MEM_CENSUS="$RAW" "$P" "$H/m-refl-census.php" > "$OUT/run-$ARM-r$Rr.out" 2>&1
    grep -q "^RF-OK 400000$" "$OUT/run-$ARM-r$Rr.out" || { echo "rc=8 parita' RF-OK rotta ($ARM r$Rr)" >> "$VERD"; fin 8; }
    [ -s "$RAW" ] || { echo "rc=8 probe muto ($ARM r$Rr)" >> "$VERD"; fin 8; }
    tr -d '\0' < "$RAW" > "$OUT/census-$ARM-r$Rr.txt"
  done
done
/opt/homebrew/bin/python3 - "$OUT" >> "$VERD" <<'PYC'
import sys, os
out = sys.argv[1]
REFL = ["__reflect_method_info","__reflect_prop_details","__reflect_prop_attr_new",
        "__reflect_class_real_name","__reflect_method_names","__reflect_class_loc"]
data = {}
bad = 0
for arm in "AB":
    for r in (1,2):
        names = {}; hn = sn = un = ovf = 0
        for l in open(f"{out}/census-{arm}-r{r}.txt"):
            t = l.split()
            if not t: continue
            if t[0] == "s149name":
                kv = dict(x.split("=",1) for x in t[1:] if "=" in x)
                names[kv.get("name","?")] = names.get(kv.get("name","?"),0) + int(kv.get("n",0))
            elif t[0] == "s149sum":
                kv = dict(x.split("=",1) for x in t[1:] if "=" in x)
                hn += int(kv.get("hostcall_n",0)); sn += int(kv.get("sum_name_n",0))
                un += int(kv.get("unnamed_n",0)); ovf += int(kv.get("overflow",0))
        ok = (hn == sn and un == 0 and ovf == 0)
        print(f"identita' s149sum {arm} r{r}: hostcall_n={hn} sum_name={sn} unnamed={un} overflow={ovf} {'OK' if ok else 'VIOL'}")
        if not ok: bad = 5
        data[(arm,r)] = (names, hn)
for arm in "AB":
    same = data[(arm,1)][0] == data[(arm,2)][0]
    print(f"repliche {arm}: r1{'==' if same else '!='}r2" + ("" if same else " — DIFFERENZE DICHIARATE (diff nomi a seguire)"))
    if not same:
        d1, d2 = data[(arm,1)][0], data[(arm,2)][0]
        for k in sorted(set(d1) | set(d2)):
            if d1.get(k,0) != d2.get(k,0): print(f"  {k}: r1={d1.get(k,0)} r2={d2.get(k,0)}")
nA, hA = data[("A",1)]; nB, hB = data[("B",1)]
print("conteggi r1 per-NOME (famiglia __reflect_*):")
for k in REFL:
    print(f"  {k}: A={nA.get(k,0)} B={nB.get(k,0)} Delta={nA.get(k,0)-nB.get(k,0)}")
att = {"__reflect_method_info":200000, "__reflect_class_real_name":200000}
fuori = []
for k in REFL:
    exp = att.get(k, 0)
    if nA.get(k,0) - nB.get(k,0) != exp: fuori.append(k)
altri = [k for k in sorted(set(nA) | set(nB)) if k not in REFL and nA.get(k,0) != nB.get(k,0)]
dh = hA - hB
print(f"Delta hostcall_n = {dh} (atteso 400000 ESATTO)")
print(f"Delta nomi NON-reflect != 0: {altri if altri else 'nessuno'}")
if fuori or altri or dh != 400000:
    print(f"CONTEGGI: FUORI ATTESA (p.4) — nomi_fuori={fuori} — dichiarare e tornare al sorgente, NESSUNA taratura")
    bad = bad or 5
else:
    print("CONTEGGI: attese p.4 ESATTE (Delta 2 alloc/iter CONFERMATO: 1 method_info + 1 class_real_name; resto zero)")
sys.exit(bad)
PYC
CRC=$?
[ "$CRC" -eq 0 ] || { echo "rc=$CRC conteggi non validi/fuori attesa" >> "$VERD"; fin "$CRC"; }

# ---------- FASE 2: RIMISURA ----------
"$QUIESCE" "$OUT/quiesce.rc" > "$OUT/quiesce.log" 2>&1
QRC=$(cat "$OUT/quiesce.rc" 2>/dev/null || echo MANCANTE)
echo "quiescenza rimisura: rc=$QRC" >> "$VERD"
[ "$QRC" = 0 ] || { echo "rc=7 quiescenza fallita" >> "$VERD"; fin 7; }
ucpu() { { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$1" "$2" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3() { local a b c; a=$(ucpu "$1" "$2"); b=$(ucpu "$1" "$2"); c=$(ucpu "$1" "$2"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }
MR="$REPO/wp158-harness/m-refl.php"; EMP="$REPO/wp158-harness/empty.php"
"$A" "$MR" > "$OUT/rim-A.out" 2>&1; "$BB" "$MR" > "$OUT/rim-B.out" 2>&1
diff -q "$OUT/rim-A.out" "$OUT/rim-B.out" > /dev/null || { echo "rc=2 output A/B DIVERGE" >> "$VERD"; fin 2; }
FA=$(floor3 "$A" "$EMP"); FB=$(floor3 "$BB" "$EMP")
TSV="$OUT/rimisura.tsv"; : > "$TSV"
echo "rimisura: giudice m-refl N=10000000 floor_A=$FA floor_B=$FB" >> "$VERD"
for i in 1 2 3 4 5; do
  if [ $((i % 2)) -eq 1 ]; then TA=$(ucpu "$A" "$MR"); TB=$(ucpu "$BB" "$MR"); ord=AB
  else TB=$(ucpu "$BB" "$MR"); TA=$(ucpu "$A" "$MR"); ord=BA; fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$i" "$TA" "$TB" "$FA" "$FB" "$ord" >> "$TSV"
  echo "  coppia$i [$ord]: rawA=$TA rawB=$TB" >> "$VERD"
done

# ---------- FASE 3: TARATURA ----------
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
print(f"RIMISURA m-refl: A={ma:.1f} B={mb:.1f} ns/iter D={D:+.1f} (rumore drop-1: A'={ra:.1f} B'={rb:.1f})")
print(f"drift vs finestra s158 (+29.0): {D-29.0:+.1f} · vs post-pin (+21.0): {D-21.0:+.1f} (banda rumore {noise:.1f})")
if D <= noise or D <= 0:
    print(f"TARATURA: NON eseguita (D={D:+.1f} non sopra il rumore col segno + — criterio p.6); registro resta [21;29]")
    sys.exit(0)
coeff = D / 2.0
surplus = D - 13.8
print(f"TARATURA: coeff cammino vec![args] = D/2 = {coeff:.1f} ns/alloc-sito (IC dal rumore: ±{noise/2:.1f}) — vs miheap 6.9: surplus totale {surplus:+.1f} ns/iter ({surplus/2:.1f}/sito)")
print(f"REGISTRO L-RF2 (criterio p.6): D_rimisura={D:+.1f} ± {noise:.1f} su stash FERMI s157/s158 — fonda l'UB della leva p.3 a ~{coeff:.1f} ns/alloc-sito rimosso")
PYT
echo "sentinella language-server fine finestra: $(ls_sentinel)" >> "$VERD"
echo "ESITO: rc=0 (conteggi ESATTI + rimisura agli atti)" >> "$VERD"
fin 0
