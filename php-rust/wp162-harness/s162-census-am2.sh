#!/bin/bash
# s162-census-am2.sh — arbitrato census L-AM2 (criterio s162-criterio-am2.md
# p.5, dovuto: smoke +67,5 FUORI banda [4;19] SOPRA). Probe-B = tree HEAD con
# edit; probe-A = copia − s162-am2-edit.patch (4 file, loc_dente.rs = test,
# fuori da -p php-cli, DICHIARATO). Driver m-strmap-census.php 200×1000;
# attesa PRE-REGISTRATA: Delta hostcall_n = 200000 ESATTO su array_map, altri
# nomi zero. Struttura+parser COPIA DICHIARATA della FASE 1 di
# s161-sonda-af1.sh; adattamento SPAZIO dichiarato: build/run SEQUENZIALI con
# rm del target A prima del build B (Data ~12G). Esiti a FILE:
# s162-census-am2-verdetto.out; rc SOLO da census-out/census.done.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp162-harness"; OUT="$H/census-out"; mkdir -p "$OUT"
VERD="$H/s162-census-am2-verdetto.out"
LOCK=/private/tmp/phpr-measure.lock
PATCH="$H/s162-am2-edit.patch"
DRV="$H/m-strmap-census.php"
SRC="/private/tmp/s162-census"
: > "$VERD"; RCF="$OUT/census.done"; rm -f "$RCF"
fin(){ echo "rc=$1 $(date +%T)" > "$RCF"; exit "$1"; }

for f in "$PATCH" "$DRV"; do
  [ -s "$f" ] || { echo "rc=7 path d'ingresso MANCANTE: $f" >> "$VERD"; fin 7; }
done
grep -qi "s-162\|s162" "$LOCK" 2>/dev/null || { echo "rc=9 lock assente/altrui" | tee -a "$VERD"; fin 9; }
echo "== s162 arbitrato census L-AM2 (criterio am2 p.5; probe-B = tree HEAD con edit, probe-A = copia − patch 4 file; driver 200×1000; smoke +67,5 FUORI banda SOPRA) ==" >> "$VERD"

rm -rf "$SRC"; mkdir -p "$SRC/B/php-rust" "$SRC/A"
for f in crates Cargo.toml Cargo.lock rust-toolchain.toml .cargo; do
  cp -R "$REPO/$f" "$SRC/B/php-rust/" || { echo "rc=7 copia tree ($f)" >> "$VERD"; fin 7; }
done
cp -R "$SRC/B/php-rust" "$SRC/A/php-rust" || { echo "rc=7 copia A" >> "$VERD"; fin 7; }
( cd "$SRC/A/php-rust" && patch -R -p1 --no-backup-if-mismatch < "$PATCH" ) > "$OUT/patchR.log" 2>&1 \
  || { echo "rc=6 patch -R fallita (log census-out/patchR.log)" >> "$VERD"; fin 6; }
NP=$(grep -c "^patching file" "$OUT/patchR.log"); NREJ=$(/usr/bin/find "$SRC/A" -name "*.rej" | wc -l | tr -d ' ')
{ [ "$NP" = 4 ] && [ "$NREJ" = 0 ]; } || { echo "rc=6 patch guardia: patching=$NP (atteso 4) rej=$NREJ (atteso 0)" >> "$VERD"; fin 6; }
echo "patch inversa: 4 file (calls/mod/host + loc_dente=test fuori da -p php-cli, DICHIARATO), zero rej" >> "$VERD"

HASHES=""
for ARM in A B; do
  ( cd "$SRC/$ARM/php-rust" && SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$SRC/tgt$ARM" \
    cargo build --release -p php-cli --features mem-census ) > "$OUT/build-$ARM.log" 2>&1 \
    || { echo "rc=7 build probe-$ARM fallita (log census-out/build-$ARM.log)" >> "$VERD"; fin 7; }
  P="$SRC/tgt$ARM/release/phpr"
  HASHES="$HASHES $ARM=$(shasum -a 256 "$P" | cut -c1-16)"
  for Rr in 1 2; do
    RAW="$OUT/census-$ARM-r$Rr.raw"; rm -f "$RAW"
    PHPR_MEM_CENSUS="$RAW" "$P" "$DRV" > "$OUT/run-$ARM-r$Rr.out" 2>&1
    grep -q "^SM-OK 200200$" "$OUT/run-$ARM-r$Rr.out" || { echo "rc=8 marcatore SM-OK 200200 assente ($ARM r$Rr)" >> "$VERD"; fin 8; }
    [ -s "$RAW" ] || { echo "rc=8 probe muto ($ARM r$Rr)" >> "$VERD"; fin 8; }
    tr -d '\0' < "$RAW" > "$OUT/census-$ARM-r$Rr.txt"
  done
  rm -rf "$SRC/tgt$ARM"   # adattamento SPAZIO dichiarato: conteggi già su file
done
echo "probe MISURATI:$HASHES (census -p php-cli --features mem-census, target dedicati, run sequenziali)" >> "$VERD"
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
print("conteggi r1 per-NOME (arbitro: array_map):")
for k in ("array_map","range","count"):
    print(f"  {k}: A={nA.get(k,0)} B={nB.get(k,0)} Delta={nA.get(k,0)-nB.get(k,0)}")
fuori = []
if nA.get("array_map",0) - nB.get("array_map",0) != 200000: fuori.append("array_map")
altri = [k for k in sorted(set(nA) | set(nB)) if k != "array_map" and nA.get(k,0) != nB.get(k,0)]
dh = hA - hB
print(f"Delta hostcall_n = {dh} (atteso 200000 ESATTO)")
print(f"Delta altri nomi != 0: {altri if altri else 'nessuno'}")
if fuori or altri or dh != 200000:
    print(f"CONTEGGI: FUORI ATTESA (p.5) — nomi_fuori={fuori} altri={altri} — dichiarare e tornare al sorgente, NESSUNA taratura")
    bad = bad or 5
else:
    print("CONTEGGI: attese p.5 ESATTE (Delta 1 passaggio vec-args/elemento CONFERMATO su array_map; resto zero) — il fast path MORDE il SOLO cammino atteso; D_smoke misura il bundle PROPRIO del sito strmap (to_vec nome + find_fn_ci + vec + bind), FUORI-UB SOPRA resta reperto A VERBALE per il R=5")
sys.exit(bad)
PYC
CRC=$?
[ "$CRC" -eq 0 ] || { echo "rc=$CRC conteggi non validi/fuori attesa" >> "$VERD"; fin "$CRC"; }
echo "ESITO: rc=0 (arbitrato agli atti — R=5 AUTORIZZATO col reperto FUORI-UB dichiarato)" >> "$VERD"
fin 0
