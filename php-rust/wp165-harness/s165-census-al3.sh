#!/bin/bash
# s165-census-al3.sh — az.rev. S-164 #2: rerun census AL3 con attesa che
# NOMINA il +1 (criterio s165-census-al3-attesa.md, PRE-registrato). COPIA
# DICHIARATA di s164-census-al3.sh (manifest s165-census-copia.diff);
# adattamenti: (i) il tree e' s165 SENZA AL3 ⇒ probe-A = copia LISCIA,
# probe-B = copia + patch AVANTI (patching=2 rej=0) + DISCRIMINANTE
# `exts: Vec::with_capacity(EXT_POOL_DEPTH)` via Default manuale (python a
# sostituzioni ASSERITE, pattern S-164): il buffer del Vec nasce a VM-init,
# FUORI dal conteggio hostcall; (ii) attesa: class_exists A=200006 B=7
# Delta=199999 ESATTO (il +1 di S-164 DEVE sparire), altri nomi 0; (iii)
# esito = verifica del MECCANISMO (il verdetto leva AL3 p.3b non dipende).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp165-harness"; OUT="$H/census-out"; mkdir -p "$OUT"
VERD="$H/s165-census-al3-verdetto.out"
LOCK=/private/tmp/phpr-measure.lock
PATCH="$REPO/wp164-harness/s164-al3-edit.patch"
DRV="$REPO/wp161-harness/m-missload-census.php"  # driver read-only wp161 (ML-OK 200000)
SRC="/private/tmp/s165-census-al3"
: > "$VERD"; RCF="$OUT/census-al3.done"; rm -f "$RCF"
fin(){ echo "rc=$1 $(date +%T)" > "$RCF"; exit "$1"; }

for f in "$PATCH" "$DRV"; do
  [ -s "$f" ] || { echo "rc=7 path d'ingresso MANCANTE: $f" >> "$VERD"; fin 7; }
done
grep -qi "s-165\|s165" "$LOCK" 2>/dev/null || { echo "rc=9 lock assente/altrui" | tee -a "$VERD"; fin 9; }
echo "== s165 rerun census L-AL3 (attesa 199999 col +1 NOMINATO: with_capacity sposta il buffer del Vec exts FUORI dal conteggio; probe-A = tree s165 LISCIO, probe-B = copia + patch AL3 + discriminante; driver N=200000) ==" >> "$VERD"

rm -rf "$SRC"; mkdir -p "$SRC/A/php-rust" "$SRC/B"
for f in crates Cargo.toml Cargo.lock rust-toolchain.toml .cargo; do
  cp -R "$REPO/$f" "$SRC/A/php-rust/" || { echo "rc=7 copia tree ($f)" >> "$VERD"; fin 7; }
done
cp -R "$SRC/A/php-rust" "$SRC/B/php-rust" || { echo "rc=7 copia B" >> "$VERD"; fin 7; }
( cd "$SRC/B" && patch -p1 --no-backup-if-mismatch < "$PATCH" ) > "$OUT/patchF.log" 2>&1 \
  || { echo "rc=6 patch avanti fallita (log census-out/patchF.log)" >> "$VERD"; fin 6; }
NP=$(grep -c "^patching file" "$OUT/patchF.log"); NREJ=$(/usr/bin/find "$SRC/B" -name "*.rej" | wc -l | tr -d ' ')
{ [ "$NP" = 2 ] && [ "$NREJ" = 0 ]; } || { echo "rc=6 patch guardia: patching=$NP (atteso 2) rej=$NREJ (atteso 0)" >> "$VERD"; fin 6; }
echo "patch avanti: 2 file (vm/mod.rs + vm/calls.rs), zero rej" >> "$VERD"
/opt/homebrew/bin/python3 - "$SRC/B/php-rust/crates/php-runtime/src/vm/mod.rs" >> "$VERD" 2>&1 <<'PYD'
import sys
p = sys.argv[1]; s = open(p).read()
old = "#[derive(Default)]\nstruct FramePool {"
assert s.count(old) == 1, f"derive-anchor count={s.count(old)}"
s = s.replace(old, "struct FramePool {")
anchor = "const EXT_POOL_DEPTH: usize = 8;"
assert s.count(anchor) == 1, f"const-anchor count={s.count(anchor)}"
s = s.replace(anchor, anchor + "\n\nimpl Default for FramePool {\n    fn default() -> Self {\n        FramePool { bufs: Vec::new(), exts: Vec::with_capacity(EXT_POOL_DEPTH) }\n    }\n}")
open(p, "w").write(s)
print("discriminante with_capacity: sostituzioni ASSERITE 2/2 applicate (Default manuale)")
PYD
[ $? -eq 0 ] || { echo "rc=6 discriminante fallito" >> "$VERD"; fin 6; }

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
    grep -q "^ML-OK 200000$" "$OUT/run-$ARM-r$Rr.out" || { echo "rc=8 marcatore ML-OK 200000 assente ($ARM r$Rr)" >> "$VERD"; fin 8; }
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
print("conteggi r1 per-NOME (arbitro: class_exists):")
for k in ("class_exists","spl_autoload_register","count"):
    print(f"  {k}: A={nA.get(k,0)} B={nB.get(k,0)} Delta={nA.get(k,0)-nB.get(k,0)}")
fuori = []
if nA.get("class_exists",0) - nB.get("class_exists",0) != 199999: fuori.append("class_exists")
altri = [k for k in sorted(set(nA) | set(nB)) if k != "class_exists" and nA.get(k,0) != nB.get(k,0)]
dh = hA - hB
print(f"Delta hostcall_n = {dh} (atteso 199999 ESATTO — col buffer del Vec exts spostato a VM-init il +1 di S-164 DEVE sparire)")
print(f"Delta altri nomi != 0: {altri if altri else 'nessuno'}")
if fuori or altri or dh != 199999:
    print(f"CONTEGGI: FUORI ATTESA — nomi_fuori={fuori} altri={altri} — l'attribuzione «+1 = buffer del Vec» e' FALSA: si torna al sorgente, INCIDENTE da contare (seconda spiegazione caduta sullo stesso arbitro)")
    bad = bad or 5
else:
    print("CONTEGGI: attesa ESATTA — il +1 di S-164 e' VERIFICATO come buffer del Vec exts (da post-hoc a PROVATO); meccanismo del verbale AL3 CONFERMATO, criterio coerente in TUTTE le clausole")
sys.exit(bad)
PYC
CRC=$?
[ "$CRC" -eq 0 ] || { echo "rc=$CRC conteggi non validi/fuori attesa" >> "$VERD"; fin "$CRC"; }
echo "ESITO: rc=0 (az.rev. S-164 #2 CHIUSA: +1 nominato e provato)" >> "$VERD"
fin 0
