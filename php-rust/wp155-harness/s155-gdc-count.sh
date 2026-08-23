#!/bin/bash
# s155-gdc-count.sh — S-155 p.3: conteggio get_declared_classes (criterio
# s155-criterio-gdc.md, PRE-REGISTRATO). Probe census s155 della sonda p.2
# RIUSATO (hash verificato contro il suo verdetto). CONTEGGI, mai tempo.
# rc: 0=VALIDO · 6=guardie · 8=run/conteggi
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp155-harness"; OUT="$H/gdc-out"; mkdir -p "$OUT"
V="$H/s155-gdc-verdetto.out"
PROBE="$H/census-prep-s155/phpr-census-s155"
: > "$V"; rm -f "$OUT/gdc.done"
note(){ echo "$1" >> "$V"; }
fin(){ echo "$1" > "$OUT/gdc.rc"; touch "$OUT/gdc.done"; exit "$1"; }
h16(){ shasum -a 256 "$1" | cut -c1-16; }

note "== s155 gdc-count (criterio s155-criterio-gdc.md; CONTEGGI mai tempo) $(date '+%F %T') =="

# S0 — guardie
[ -e /private/tmp/phpr-measure.lock ] || { note "rc=6 measure-lock ASSENTE"; fin 6; }
pgrep -qx cargo && { note "rc=6 cargo in volo"; fin 6; }
pgrep -qx rustc && { note "rc=6 rustc in volo"; fin 6; }
[ -f "$PROBE" ] || { note "rc=6 probe ASSENTE ($PROBE): sonda p.2 prima"; fin 6; }
HP="$(h16 "$PROBE")"
HS="$(grep -o 'hash16=[0-9a-f]\{16\}' "$H/s155-sonda-ce-verdetto.out" | cut -d= -f2)"
[ "$HP" = "$HS" ] || { note "rc=6 probe hash $HP != verdetto sonda $HS"; fin 6; }
note "S0 guardie: lock presente, niente cargo/rustc, probe $HP == verdetto sonda"

# S1 — 4 run: N x GDX (stdout ESATTO pena STOP; classi coerenti)
N1=100000; N2=300000
declare -A CLS
for GDX in 0 200; do
  for N in $N1 $N2; do
    PHPR_MEM_CENSUS="$OUT/gdc-$GDX-$N.raw" GDN=$N GDX=$GDX "$PROBE" "$H/gdc-count.php" > "$OUT/gdc-$GDX-$N.stdout" 2> "$OUT/gdc-$GDX-$N.stderr"
    RCN=$?
    GOT="$(cat "$OUT/gdc-$GDX-$N.stdout")"
    [ "$RCN" -eq 0 ] || { note "rc=8 gdc GDX=$GDX N=$N rc=$RCN"; fin 8; }
    C="$(echo "$GOT" | sed -n 's/.*classi=\([0-9]*\).*/\1/p')"
    OKF="$(echo "$GOT" | sed -n 's/.*acc_ok=\([01]\).*/\1/p')"
    [ "$GOT" = "GDC-COUNT-OK n=$N classi=$C acc_ok=1" ] && [ "$OKF" = 1 ] || { note "rc=8 stdout GDX=$GDX N=$N: '$GOT'"; fin 8; }
    [ -s "$OUT/gdc-$GDX-$N.raw" ] || { note "rc=8 raw GDX=$GDX N=$N vuoto"; fin 8; }
    if [ -n "${CLS[$GDX]:-}" ] && [ "${CLS[$GDX]}" != "$C" ]; then note "rc=8 classi instabili GDX=$GDX"; fin 8; fi
    CLS[$GDX]="$C"
  done
done
[ "${CLS[200]}" -eq "$(( ${CLS[0]} + 200 ))" ] || { note "rc=8 classi: C200=${CLS[200]} != C0=${CLS[0]}+200"; fin 8; }
note "S1 gdc-count: stdout ESATTI ai 4 run; C0=${CLS[0]} C200=${CLS[200]} (+200 OK)"

# S2 — aritmetica meccanica (criterio p.3-4)
python3 - "$OUT" "$N1" "$N2" "${CLS[0]}" >> "$V" <<'PY'
import re, sys
out, n1, n2, c0 = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
def gdc(fn):
    for line in open(fn, errors="replace"):
        if "name=get_declared_classes" in line:
            m = re.search(r"\bn=(\d+)\b.*?\bb=(\d+)\b", line)
            if m: return int(m.group(1)), int(m.group(2))
    raise SystemExit(f"riga name=get_declared_classes ASSENTE in {fn}")
dn = n2 - n1
ks = {}
for gdx in (0, 200):
    a1, b1 = gdc(f"{out}/gdc-{gdx}-{n1}.raw"); a2, b2 = gdc(f"{out}/gdc-{gdx}-{n2}.raw")
    k = (a2 - a1) / dn; by = (b2 - b1) / dn
    intero = "INTERO ESATTO" if (a2 - a1) % dn == 0 else "NON intero (dichiarare)"
    ks[gdx] = k
    print(f"k(GDX={gdx}) = {k:.3f} alloc/chiamata ({intero}; Δn={a2-a1} su ΔN={dn}) · b = {by:.1f} B/chiamata")
per_cls = (ks[200] - ks[0]) / 200.0
fisso = ks[0] - per_cls * c0
k_orm = fisso + per_cls * 2393
calls = 4563808 / k_orm if k_orm > 0 else float("nan")
print(f"fit: per_classe = {per_cls:.4f} alloc · fisso = {fisso:.2f} (C0={c0})")
ip = "CONFERMATA" if 0.9 <= per_cls <= 1.1 else "FALSIFICATA (si torna al sorgente prima d'ogni conclusione)"
print(f"ipotesi s152 (array ricostruito per chiamata, per_classe~1): {ip}")
print(f"proiezione ORM: k_ORM = fisso + per_classe x 2393 = {k_orm:.1f} ⇒ chiamate ≈ 4.563.808 / {k_orm:.1f} = {calls:,.0f}")
print(f"nota di scala (nessuna attesa di leva qui): alloc 4,56M x miheap [6,7;6,9] ns ≈ [{4563808*6.7/1e9:.3f}; {4563808*6.9/1e9:.3f}] s puro-alloc — il criterio di LEVA (se mai) è atto separato coi canali copy/free nominati")
PY
PRC=$?
[ "$PRC" -eq 0 ] || { note "rc=8 parser fallito (rc=$PRC)"; fin 8; }
note "S2 aritmetica scritta. rc=0"
fin 0
