#!/bin/bash
# s161-census-al2.sh — ARBITRATO census L-AL2 (criterio al2 p.5: D_smoke=+5,5
# FUORI banda [8;22] SOTTO ⇒ conteggi PRIMA del R=5). COPIA DICHIARATA della
# FASE 1 di s161-sonda-af1.sh coi SOLI adattamenti: probe-B = tree HEAD (CON
# edit L-AL2 non committato), probe-A = copia − patch s161-al2-edit.patch
# (2 file: vm monolite + dente loc, il dente è test e non entra in -p php-cli
# — NP=2 atteso); driver m-missload-census.php 200k; attesa ESATTA:
# Δ(A−B) name=class_exists = 200000 (1 passaggio vec-args/miss), resto zero.
# Esiti a FILE: verdetto s161-census-al2-verdetto.out; rc da census-out/census.done.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp161-harness"; OUT="$H/census-out"; mkdir -p "$OUT"
VERD="$H/s161-census-al2-verdetto.out"
LOCK=/private/tmp/phpr-measure.lock
PATCH="$H/s161-al2-edit.patch"
DRV="$H/m-missload-census.php"
SRC="/private/tmp/s161-census-al2"
: > "$VERD"; RCF="$OUT/census.done"; rm -f "$RCF"
fin(){ echo "rc=$1 $(date +%T)" > "$RCF"; exit "$1"; }
for f in "$PATCH" "$DRV"; do
  [ -s "$f" ] || { echo "rc=7 path d'ingresso MANCANTE: $f" >> "$VERD"; fin 7; }
done
grep -qi "s-161\|s161" "$LOCK" 2>/dev/null || { echo "rc=9 lock assente/altrui" | tee -a "$VERD"; fin 9; }
echo "== s161 arbitrato census L-AL2 (criterio al2 p.5; probe-B = tree HEAD con edit, probe-A = copia − patch; driver 200k) ==" >> "$VERD"
rm -rf "$SRC"; mkdir -p "$SRC/B/php-rust" "$SRC/A"
for f in crates Cargo.toml Cargo.lock rust-toolchain.toml .cargo; do
  cp -R "$REPO/$f" "$SRC/B/php-rust/" || { echo "rc=7 copia tree ($f)" >> "$VERD"; fin 7; }
done
cp -R "$SRC/B/php-rust" "$SRC/A/php-rust" || { echo "rc=7 copia A" >> "$VERD"; fin 7; }
( cd "$SRC/A" && patch -R -p1 --no-backup-if-mismatch < "$PATCH" ) > "$OUT/patchR.log" 2>&1 \
  || { echo "rc=6 patch -R fallita (log census-out/patchR.log)" >> "$VERD"; fin 6; }
NP=$(grep -c "^patching file" "$OUT/patchR.log"); NREJ=$(/usr/bin/find "$SRC/A" -name "*.rej" | wc -l | tr -d ' ')
{ [ "$NP" = 2 ] && [ "$NREJ" = 0 ]; } || { echo "rc=6 patch guardia: patching=$NP (atteso 2) rej=$NREJ (atteso 0)" >> "$VERD"; fin 6; }
echo "patch inversa: 2 file, zero rej (dente loc = test, fuori da -p php-cli, DICHIARATO)" >> "$VERD"
for ARM in A B; do
  ( cd "$SRC/$ARM/php-rust" && SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$SRC/tgt$ARM" \
    cargo build --release -p php-cli --features mem-census ) > "$OUT/build-$ARM.log" 2>&1 \
    || { echo "rc=7 build probe-$ARM fallita (log census-out/build-$ARM.log)" >> "$VERD"; fin 7; }
done
PA="$SRC/tgtA/release/phpr"; PB="$SRC/tgtB/release/phpr"
echo "probe MISURATI: A=$(shasum -a 256 "$PA" | cut -c1-16) B=$(shasum -a 256 "$PB" | cut -c1-16) (census -p php-cli --features mem-census, target dedicati)" >> "$VERD"
for ARM in A B; do
  P="$SRC/tgt$ARM/release/phpr"
  for Rr in 1 2; do
    RAW="$OUT/census-$ARM-r$Rr.raw"; rm -f "$RAW"
    PHPR_MEM_CENSUS="$RAW" "$P" "$DRV" > "$OUT/run-$ARM-r$Rr.out" 2>&1
    grep -q "^ML-OK 200000$" "$OUT/run-$ARM-r$Rr.out" || { echo "rc=8 marcatore ML-OK 200000 assente ($ARM r$Rr)" >> "$VERD"; fin 8; }
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
    print(f"repliche {arm}: r1{'==' if same else '!='}r2" + ("" if same else " — DIFFERENZE DICHIARATE"))
    if not same:
        d1, d2 = data[(arm,1)][0], data[(arm,2)][0]
        for k in sorted(set(d1) | set(d2)):
            if d1.get(k,0) != d2.get(k,0): print(f"  {k}: r1={d1.get(k,0)} r2={d2.get(k,0)}")
nA, hA = data[("A",1)]; nB, hB = data[("B",1)]
print("conteggi r1 per-NOME (arbitro: class_exists):")
for k in ("class_exists","spl_autoload_register"):
    print(f"  {k}: A={nA.get(k,0)} B={nB.get(k,0)} Delta={nA.get(k,0)-nB.get(k,0)}")
fuori = []
if nA.get("class_exists",0) - nB.get("class_exists",0) != 200000: fuori.append("class_exists")
altri = [k for k in sorted(set(nA) | set(nB)) if k != "class_exists" and nA.get(k,0) != nB.get(k,0)]
dh = hA - hB
print(f"Delta hostcall_n = {dh} (atteso 200000 ESATTO)")
print(f"Delta altri nomi != 0: {altri if altri else 'nessuno'}")
if fuori or altri or dh != 200000:
    print(f"ARBITRATO: FUORI ATTESA — nomi_fuori={fuori} altri={altri} — tornare al sorgente, R=5 NON parte")
    bad = bad or 5
else:
    print("ARBITRATO: Delta 1 passaggio vec-args/miss CONFERMATO ESATTO — il fast path MORDE; D_smoke=+5,5 misura il bundle PROPRIO del sito (vec+dispatch, SENZA mode-match/doppio-clone di arrfilter): il coeff di famiglia si rivela PER-SITO. R=5 autorizzato col modello RIVISTO nel verdetto di sessione.")
sys.exit(bad)
PYC
CRC=$?
[ "$CRC" -eq 0 ] || { echo "rc=$CRC conteggi non validi/fuori attesa" >> "$VERD"; fin "$CRC"; }
echo "ESITO: rc=0 (arbitrato agli atti)" >> "$VERD"
fin 0
