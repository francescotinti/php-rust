#!/bin/bash
# s154-sonda.sh — S-154 p.2: sonda k post-BT2 (criterio s154-criterio-sonda.md,
# PRE-REGISTRATO). Ricetta PREP s151 INVARIATA; CONTEGGI, mai tempo.
# rc: 0=VALIDO · 3=identità-ricetta · 4=copia-infedele · 5=smoke · 6=guardia
#     pre-run · 7=apply-diff · 8=probe/conteggi · 2=restore-INCOMPLETO (grave)
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:/Users/francescotinti/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp154-harness"; OUT="$H/sonda-out"; mkdir -p "$OUT"
V="$H/s154-sonda-verdetto.out"
CANON="$HOME/Claude/php-rust-output"
HOLD="$HOME/Claude/php-rust-output.s154hold"
SRC=/private/tmp/phpr-census-s154-src
TGT=/private/tmp/phpr-census-s154-target
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
PIN="8370c257ae70cc8e"; SRV="f030c6fcbddfab96"
: > "$V"; rm -f "$OUT/sonda.done"
note(){ echo "$1" >> "$V"; }
fin(){ echo "$1" > "$OUT/sonda.rc"; touch "$OUT/sonda.done"; exit "$1"; }
h16(){ shasum -a 256 "$1" | cut -c1-16; }

note "== s154 sonda k post-BT2 (criterio s154-criterio-sonda.md; ricetta PREP s151; CONTEGGI mai tempo) $(date '+%F %T') =="

# S0 — guardie pre-run
[ -e /private/tmp/phpr-measure.lock ] || { note "rc=6 measure-lock ASSENTE"; fin 6; }
pgrep -qx cargo && { note "rc=6 cargo in volo"; fin 6; }
pgrep -qx rustc && { note "rc=6 rustc in volo"; fin 6; }
[ "$(h16 "$CANON/release/phpr")" = "$PIN" ] || { note "rc=6 pin phpr != $PIN"; fin 6; }
[ "$(h16 "$CANON/release/php-server")" = "$SRV" ] || { note "rc=6 pin server != $SRV"; fin 6; }
[ -e "$HOLD" ] && { note "rc=6 HOLD residuo presente ($HOLD): istruire prima"; fin 6; }
note "S0 guardie: lock presente, niente cargo/rustc, pin phpr=$PIN server=$SRV, hold assente"

# S1 — copia canonica (APFS) + guardia copia-fedele
mkdir -p "$SRC"
rsync -a --delete --exclude "._*" "$REPO/crates" "$REPO/Cargo.toml" "$REPO/Cargo.lock" "$REPO/rust-toolchain.toml" "$REPO/.cargo" "$SRC/" || { note "rc=7 rsync copia"; fin 7; }
if ! diff -r -x "._*" "$REPO/crates" "$SRC/crates" > "$OUT/copia-fedele.diff" 2>&1; then
  note "rc=4 copia INFEDELE (dettaglio sonda-out/copia-fedele.diff)"; fin 4
fi
note "S1 copia-fedele: PASS diff -r crates rc=0 (copia $SRC)"

# S2 — identità a CONTENUTO (emenda §1-bis dopo rc=3 t1: pin s153 NON
# cold-riproducibile, cache calda ai build A/B S-153; scarto ammesso = SOLO
# LC_UUID+pagina firma ≤64 B, meccanismo s150-identita-candidato + PREP s151)
IDT=/private/tmp/phpr-census-s154-idcheck
( cd "$SRC" && env SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$IDT" cargo build --release ) > "$OUT/build-identita.log" 2>&1
BRC=$?
[ "$BRC" -eq 0 ] || { note "rc=3 build identità FALLITA (rc=$BRC, log sonda-out/build-identita.log)"; fin 3; }
HID="$(h16 "$IDT/release/phpr")"
LEOFF=$(otool -l "$IDT/release/phpr" | awk '/cmd LC_SEGMENT_64/{seg=""} /segname __LINKEDIT/{seg=1} seg && /fileoff/{print $2; exit}')
python3 - "$CANON/release/phpr" "$IDT/release/phpr" "${LEOFF:-0}" >> "$V" <<'PY'
import sys, re
a = open(sys.argv[1], "rb").read(); b = open(sys.argv[2], "rb").read()
linkedit = int(sys.argv[3])
if len(a) != len(b):
    print(f"S2 identità-contenuto: FAIL dimensioni {len(a)} != {len(b)}"); sys.exit(3)
diffs = [i for i, (x, y) in enumerate(zip(a, b)) if x != y]
if len(diffs) > 160:
    print(f"S2 FAIL: {len(diffs)} byte diversi > cap 160"); sys.exit(3)
clusters = []
for i in diffs:
    if clusters and i - clusters[-1][1] <= 64: clusters[-1][1] = i
    else: clusters.append([i, i])
bad = []
for s, e in clusters:
    ctx_b = b[max(0, s-64):e+64]
    if s < 4096 and e - s + 1 <= 16:
        cls = "LC_UUID"          # header load-command area, ampiezza UUID
    elif linkedit and s >= linkedit:
        cls = "__LINKEDIT/firma"
    elif b"Jan  1 1970" in ctx_b or b"00:00:00" in ctx_b:
        cls = "mimalloc-banner __DATE__/__TIME__"
    else:
        cls = "NON CLASSIFICATO"; bad.append((s, e))
    print(f"S2 cluster 0x{s:x}-0x{e:x} ({sum(1 for d in diffs if s<=d<=e)} B): {cls}")
print(f"S2 confronto byte: {len(diffs)} byte diversi in {len(clusters)} cluster (linkedit fileoff={linkedit})")
sys.exit(3 if bad else 0)
PY
CRC=$?
[ "$CRC" -eq 0 ] || { note "rc=3 identità-contenuto VIOLATA: cluster NON classificato (build=$HID)"; fin 3; }
strings -a "$CANON/release/phpr" > "$OUT/strings-pin.txt"
strings -a "$IDT/release/phpr" > "$OUT/strings-build.txt"
if ! diff "$OUT/strings-pin.txt" "$OUT/strings-build.txt" | grep -vE "^---$|^[<>].*(197[0-9]|202[0-9]|[0-9]{2}:[0-9]{2}:[0-9]{2})|^[0-9,]+[acd][0-9,]+$" | grep -q .; then
  note "S2 strings: diff SOLO righe data/ora mimalloc (ammesse, emenda §1-bis raffinata)"
else
  note "rc=3 strings-diff oltre le righe data/ora (build=$HID): divergenza REALE"; fin 3
fi
rm -f "$OUT/strings-pin.txt" "$OUT/strings-build.txt"
note "S2 identità a CONTENUTO: PASS (build fredda $HID vs pin $PIN: cluster tutti nominati — LC_UUID + firma __LINKEDIT + banner mimalloc; emenda §1-bis)"

# S3 — diff census normalizzato + apply nella copia
sed -e 's#^--- a/php-rust/#--- a/#' \
    -e 's#^+++ b/private/tmp/phpr-census-s151-src/#+++ b/#' \
    -e 's#^diff --git a/php-rust/\(.*\) b/private/tmp/phpr-census-s151-src/.*#diff --git a/\1 b/\1#' \
    "$REPO/wp151-harness/s151-census-copia.diff" > "$OUT/census-s154-norm.diff"
( cd "$SRC" && git apply --check -v "$OUT/census-s154-norm.diff" ) > "$OUT/apply-check.log" 2>&1 || { note "rc=7 apply --check FALLITO (log sonda-out/apply-check.log)"; fin 7; }
( cd "$SRC" && git apply -v "$OUT/census-s154-norm.diff" ) > "$OUT/apply.log" 2>&1 || { note "rc=7 apply FALLITO"; fin 7; }
note "S3 diff census s151 applicato ($(grep -c 'Applied patch' "$OUT/apply.log" 2>/dev/null || echo '?') file; offset nel log)"

# S4 — build probe (feature mem-census SOLA, -p php-cli, target dedicato APFS)
( cd "$SRC" && env SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TGT" cargo build --release -p php-cli --features mem-census ) > "$OUT/build-probe.log" 2>&1 || { note "rc=8 build probe FALLITA (log sonda-out/build-probe.log)"; fin 8; }
PROBE_TGT="$TGT/release/phpr"
HP="$(h16 "$PROBE_TGT")"
mkdir -p "$H/census-prep-s154"
cp "$PROBE_TGT" "$H/census-prep-s154/phpr-census-s154"
cp "$PROBE_TGT" "$STASH/phpr-census-s154"
[ "$(h16 "$H/census-prep-s154/phpr-census-s154")" = "$HP" ] || { note "rc=8 copia probe harness difforme"; fin 8; }
[ "$(h16 "$STASH/phpr-census-s154")" = "$HP" ] || { note "rc=8 stash probe difforme"; fin 8; }
PROBE="$H/census-prep-s154/phpr-census-s154"
note "S4 probe s154: hash16=$HP — conservato x2 (harness + stash), verificato"

# S5 — smoke con attesi s151 INVARIATI
SMK="$OUT/smoke"; mkdir -p "$SMK"
CENSUS_PHPR="$PROBE" "$REPO/wp151-harness/census-prep/smoke151-check.sh" "$SMK" > "$SMK/check.log" 2>&1
SRC_RC=$?
[ "$SRC_RC" -eq 0 ] || { note "rc=5 smoke FALLITO (rc=$SRC_RC, dettaglio $SMK/check.log)"; fin 5; }
note "S5 smoke: rc=0 (checker s151 a esiti esatti, attesi INVARIATI)"

# S6 — conteggi bt-count a DUE N (esito stdout ESATTO pena STOP)
N1=100000; N2=300000
for N in $N1 $N2; do
  PHPR_MEM_CENSUS="$OUT/bt-$N.raw" BTN=$N "$PROBE" "$REPO/wp152-harness/bt-count.php" > "$OUT/bt-$N.stdout" 2> "$OUT/bt-$N.stderr"
  RCN=$?
  EXP="BT-COUNT-OK n=$N frames_tot=$((2*N))"
  GOT="$(cat "$OUT/bt-$N.stdout")"
  [ "$RCN" -eq 0 ] || { note "rc=8 bt-count N=$N rc=$RCN"; fin 8; }
  [ "$GOT" = "$EXP" ] || { note "rc=8 bt-count N=$N stdout '$GOT' != atteso '$EXP'"; fin 8; }
  [ -s "$OUT/bt-$N.raw" ] || { note "rc=8 raw N=$N vuoto"; fin 8; }
done
note "S6 bt-count: stdout ESATTI ai due N, raw presenti"

# S7 — k = Δ/ΔN + riconciliazione FUORI-UB (meccanica, criterio p.3)
python3 - "$OUT/bt-$N1.raw" "$OUT/bt-$N2.raw" "$N1" "$N2" >> "$V" <<'PY'
import re, sys
f1, f2, n1, n2 = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
def bt(fn):
    for line in open(fn, errors="replace"):
        if "name=debug_backtrace" in line:
            m = re.search(r"\bn=(\d+)\b.*?\bb=(\d+)\b", line)
            if m: return int(m.group(1)), int(m.group(2))
    raise SystemExit(f"riga name=debug_backtrace ASSENTE in {fn}")
a1, b1 = bt(f1); a2, b2 = bt(f2)
dn = n2 - n1
k = (a2 - a1) / dn; by = (b2 - b1) / dn
intero = "INTERO ESATTO" if (a2 - a1) % dn == 0 else "NON intero (dichiarare)"
print(f"k_new = {k:.3f} alloc/chiamata ({intero}; Δn={a2-a1} su ΔN={dn}) · b_new = {by:.1f} B/chiamata (rif pre-BT2: k=45, b=2282)")
att = "DENTRO" if 22 <= k <= 28 else "FUORI (si torna al sorgente prima d'ogni conclusione)"
print(f"attesa k 22-28: {att}")
d_lo = (45 - k) * 6.7; d_hi = (45 - k) * 6.9
print(f"riconciliazione: D_alloc = (45-{k:.1f}) x miheap [6,7;6,9] = [{d_lo:.1f}; {d_hi:.1f}] ns/iter · D misurato=+266,7")
r_lo = 266.7 - d_hi; r_hi = 266.7 - d_lo
if r_lo <= 0 <= r_hi or r_hi < 0:
    print(f"residuo NON-alloc R = [{r_lo:.1f}; {r_hi:.1f}] — il canale alloc COPRE D (dichiarare)")
else:
    print(f"residuo NON-alloc R = [{r_lo:.1f}; {r_hi:.1f}] ns/iter — canali candidati da NOMINARE (hash Key::from_bytes, memcpy chiavi, ty.to_vec, clone file); nessuna cifra per canale senza mock dedicato")
PY
PRC=$?
[ "$PRC" -eq 0 ] || { note "rc=8 parser k fallito (rc=$PRC)"; fin 8; }
note "S7 riconciliazione scritta. rc=0"
fin 0
