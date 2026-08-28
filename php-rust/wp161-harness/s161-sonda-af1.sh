#!/bin/bash
# s161-sonda-af1.sh — sonda surplus L-AF1 (criterio s161-criterio-sonda-af1.md,
# PRE-REGISTRATO). FASE 1 CONTEGGI: probe census A (tree HEAD − patch L-AF1
# inversa, solo host.rs) vs B (tree HEAD), driver m-arrfilter-census.php
# 200×1000, attese ESATTE p.4. FASE 2 RIMISURA: A/B R=5 ABAB su stash FERMI
# gemelloA(==pin s159) vs af1-B(==pin s160) (giudice m-arrfilter N=10M,
# matematica s158/s159 INVARIATA: mediane, floor3, rumore drop-1). FASE 3
# DECISIONE p.6: cifra a registro + bivio coeff unico / famiglia sdoppiata.
# Pezzi DICHIARATI: struttura+judge math COPIA s159-sonda.sh; parser adattato.
# Esiti a FILE: verdetto s161-sonda-af1-verdetto.out; rc SOLO da sonda-out/sonda.done.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp161-harness"; OUT="$H/sonda-out"; mkdir -p "$OUT"
VERD="$H/s161-sonda-af1-verdetto.out"
LOCK=/private/tmp/phpr-measure.lock
ST="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
A="$ST/phpr-s160-gemelloA"; BB="$ST/phpr-s160-af1-B"
AEXP=f2d17f18; BEXP=ceeb6e76
PATCH="$REPO/wp160-harness/s160-af1-copia.diff"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
MR="$REPO/wp160-harness/m-arrfilter.php"; EMP="$REPO/wp160-harness/empty.php"
DRV="$H/m-arrfilter-census.php"
SRC="/private/tmp/s161-sonda"
: > "$VERD"; RCF="$OUT/sonda.done"; rm -f "$RCF"
fin(){ echo "rc=$1 $(date +%T)" > "$RCF"; exit "$1"; }
ls_sentinel(){ pgrep -fl 'language[_-]server|Antigravity|rust-analyzer|intelephense|serena|gopls|pylsp|solargraph' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' '; }

# verifica POSITIVA dei path d'ingresso (emenda §3 proposta S-160)
for f in "$PATCH" "$QUIESCE" "$MR" "$EMP" "$DRV" "$A" "$BB"; do
  [ -s "$f" ] || { echo "rc=7 path d'ingresso MANCANTE: $f" >> "$VERD"; fin 7; }
done
grep -qi "s-161\|s161" "$LOCK" 2>/dev/null || { echo "rc=9 lock assente/altrui" | tee -a "$VERD"; fin 9; }
AM=$(shasum -a 256 "$A" | cut -c1-8); BM=$(shasum -a 256 "$BB" | cut -c1-8)
[ "$AM" = "$AEXP" ] || { echo "rc=9 stash A $AM != $AEXP" | tee -a "$VERD"; fin 9; }
[ "$BM" = "$BEXP" ] || { echo "rc=9 stash B $BM != $BEXP" | tee -a "$VERD"; fin 9; }
echo "== s161 sonda surplus L-AF1 (criterio s161-criterio-sonda-af1.md; bracci rimisura MISURATI A=$AM(gemelloA==pin s159) B=$BM(af1-B==pin s160); conteggi su probe census A/B da tree HEAD∓patch host.rs) ==" >> "$VERD"
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
{ [ "$NP" = 1 ] && [ "$NREJ" = 0 ]; } || { echo "rc=6 patch guardia: patching=$NP (atteso 1) rej=$NREJ (atteso 0)" >> "$VERD"; fin 6; }
echo "patch inversa: 1 file (host.rs), zero rej (log agli atti; loc_dente.rs escluso DICHIARATO: test, non in -p php-cli)" >> "$VERD"
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
    PHPR_MEM_CENSUS="$RAW" "$P" "$DRV" > "$OUT/run-$ARM-r$Rr.out" 2>&1
    grep -q "^AF-OK 100000$" "$OUT/run-$ARM-r$Rr.out" || { echo "rc=8 marcatore AF-OK 100000 assente ($ARM r$Rr)" >> "$VERD"; fin 8; }
    [ -s "$RAW" ] || { echo "rc=8 probe muto ($ARM r$Rr)" >> "$VERD"; fin 8; }
    tr -d '\0' < "$RAW" > "$OUT/census-$ARM-r$Rr.txt"
  done
done
/opt/homebrew/bin/python3 - "$OUT" >> "$VERD" <<'PYC'
import sys
out = sys.argv[1]
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
print("conteggi r1 per-NOME (arbitro: array_filter):")
for k in ("array_filter","count","range"):
    print(f"  {k}: A={nA.get(k,0)} B={nB.get(k,0)} Delta={nA.get(k,0)-nB.get(k,0)}")
fuori = []
if nA.get("array_filter",0) - nB.get("array_filter",0) != 200000: fuori.append("array_filter")
altri = [k for k in sorted(set(nA) | set(nB)) if k != "array_filter" and nA.get(k,0) != nB.get(k,0)]
dh = hA - hB
print(f"Delta hostcall_n = {dh} (atteso 200000 ESATTO)")
print(f"Delta altri nomi != 0: {altri if altri else 'nessuno'}")
if fuori or altri or dh != 200000:
    print(f"CONTEGGI: FUORI ATTESA (p.4) — nomi_fuori={fuori} altri={altri} — dichiarare e tornare al sorgente, NESSUNA taratura")
    bad = bad or 5
else:
    print("CONTEGGI: attese p.4 ESATTE (Delta 1 passaggio vec-args/elemento CONFERMATO su array_filter; resto zero)")
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
"$A" "$MR" > "$OUT/rim-A.out" 2>&1; "$BB" "$MR" > "$OUT/rim-B.out" 2>&1
grep -q "^AF-OK 5000000$" "$OUT/rim-A.out" || { echo "rc=2 marcatore AF-OK 5000000 assente (A)" >> "$VERD"; fin 2; }
grep -q "^AF-OK 5000000$" "$OUT/rim-B.out" || { echo "rc=2 marcatore AF-OK 5000000 assente (B)" >> "$VERD"; fin 2; }
diff -q "$OUT/rim-A.out" "$OUT/rim-B.out" > /dev/null || { echo "rc=2 output A/B DIVERGE" >> "$VERD"; fin 2; }
FA=$(floor3 "$A" "$EMP"); FB=$(floor3 "$BB" "$EMP")
TSV="$OUT/rimisura.tsv"; : > "$TSV"
echo "rimisura: giudice m-arrfilter N=10000000 floor_A=$FA floor_B=$FB" >> "$VERD"
for i in 1 2 3 4 5; do
  if [ $((i % 2)) -eq 1 ]; then TA=$(ucpu "$A" "$MR"); TB=$(ucpu "$BB" "$MR"); ord=AB
  else TB=$(ucpu "$BB" "$MR"); TA=$(ucpu "$A" "$MR"); ord=BA; fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$i" "$TA" "$TB" "$FA" "$FB" "$ord" >> "$TSV"
  echo "  coppia$i [$ord]: rawA=$TA rawB=$TB" >> "$VERD"
done

# ---------- FASE 3: DECISIONE ----------
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
print(f"RIMISURA m-arrfilter: A={ma:.1f} B={mb:.1f} ns/elemento D={D:+.1f} (rumore drop-1: A'={ra:.1f} B'={rb:.1f})")
print(f"drift vs finestra s160 (+16.0): {D-16.0:+.1f} · vs post-pin (+15.0): {D-15.0:+.1f} (banda rumore {noise:.1f})")
if D <= noise or D <= 0:
    print(f"DECISIONE: NON eseguita (D={D:+.1f} non sopra il rumore col segno + — criterio p.6); registro resta [15;16], reperto NON risolto")
    sys.exit(0)
print(f"REGISTRO L-AF1 (criterio p.6): D_rimisura={D:+.1f} ± {noise:.1f} ns/elemento su stash FERMI s159/s160 — costo passaggio CLOSURE-vec (Delta census 1/elemento) = {D:.1f} ± {noise:.1f} ns")
if abs(D - 12.0) <= 2.5 + noise:
    print(f"BIVIO p.6 = (a): |D-12.0|={abs(D-12.0):.1f} <= 2.5+rumore({noise:.1f}) — coeff UNICO 12.0±2.5 REGGE; reperto «al bordo» RISOLTO DENTRO")
else:
    print(f"BIVIO p.6 = (b): |D-12.0|={abs(D-12.0):.1f} > 2.5+rumore({noise:.1f}) — famiglia SDOPPIATA: hostcall-vec=12.0±2.5 (s159) · closure-vec={D:.1f}±{noise:.1f} (s161); surplus dispatch closure = {D-12.0:+.1f} ns/passaggio")
PYT
echo "sentinella language-server fine finestra: $(ls_sentinel)" >> "$VERD"
echo "ESITO: rc=0 (conteggi + rimisura + decisione agli atti)" >> "$VERD"
fin 0
