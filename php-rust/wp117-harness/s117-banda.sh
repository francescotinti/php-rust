#!/bin/bash
# s117-banda.sh — RI-BANDA su A′ (ordine concilio p.2; criterio PRE §5).
# Per OGNI zavorra (s114, s115-2): apply → build sotto pipeline A′ (target
# dedicato, INCREMENTAL=0) → admission (dump ON {main} ×6 vs A′; OFF al byte;
# taglia run_loop + bl/br a verbale — sotto LTO la zavorra può sparire in
# silenzio: l'admission pretende l'esito esatto) → conserva → revert (tree
# pulito verificato) → A/B ABAB vs A′ sei categorie (famiglie 1,3×min, R=5)
# → held-out poly/err/wploop R=5 (|Dnet| = campione banda). Esiti in FILE.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
ROOT="/Volumes/Extreme Pro/Claude/php-rust-experiment"
SRC="$ROOT/php-rust"
TGT="/Volumes/Extreme Pro/Claude/phpr-s117-aprime-target"
M="$H/../wp97-harness/micro"
HD="$H/../wp111-harness/heldout"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s117-aprime"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
OUT="$H/banda-out"; mkdir -p "$OUT"
VERD="$H/s117-banda-verdetto.out"
CATS="arith prop calls str arr re"
note(){ echo "$1"; echo "$1" >> "$VERD"; }
user_cpu() { { /usr/bin/time -p "$1" "$2" >/dev/null; } 2>&1 | awk '/^user/{print $2}'; }
main_only() { awk '/^-- \{main\} /{f=1;next} /^-- /{f=0} f' "$1"; }

infam_count() {
python3 - "$1" "$2" <<'PY'
import sys
rows=[l.split('\t') for l in open(sys.argv[1]) if l.startswith(sys.argv[2]+'\t')]
if not rows: print(0); sys.exit()
na=[(float(t[3])-float(t[5]))/float(t[1])*1e9 for t in rows]
nb=[(float(t[4])-float(t[6]))/float(t[1])*1e9 for t in rows]
ta,tb=1.3*min(na),1.3*min(nb)
print(sum(1 for a,b in zip(na,nb) if a<=ta and b<=tb))
PY
}

misura_zavorra() { # $1=tag $2=patch-path
  local TAG="$1" PATCH="$2"
  local B="$STASH/phpr-s117-$TAG"
  local TSV="$OUT/$TAG-runs.tsv"
  note "== zavorra $TAG =="

  # apply + build sotto A′
  cd "$ROOT" || return 4
  git apply "$PATCH" || { note "$TAG: git apply FALLITO"; return 4; }
  cd "$SRC"
  CARGO_TARGET_DIR="$TGT" CARGO_INCREMENTAL=0 cargo build --release > "$OUT/$TAG-build.log" 2>&1
  rc=$?; echo "$rc" > "$OUT/$TAG-build.rc"
  if [ "$rc" != 0 ]; then note "$TAG: build FALLITA rc=$rc"; cd "$ROOT"; git apply -R "$PATCH"; return 4; fi
  cp "$TGT/release/phpr" "$B"
  # revert SUBITO e verifica tree pulito
  cd "$ROOT"; git apply -R "$PATCH" || { note "$TAG: revert FALLITO — STOP"; return 4; }
  git diff --quiet -- '*.rs' || { note "$TAG: tree NON pulito dopo revert — STOP"; return 4; }
  note "$TAG: build rc=0, candidato conservato phpr-s117-$TAG=$(shasum -a 256 "$B"|cut -c1-8), revert verificato"

  # admission: dump ON {main} ×6 e OFF al byte vs A′; run_loop size/bl a verbale
  local ARC=0
  for c in $CATS; do
    PHPR_DUMP_OPS=1 "$B" "$M/$c.php" 2> "$OUT/$TAG-on-$c.dump" > /dev/null
    PHPR_DUMP_OPS=1 "$A" "$M/$c.php" 2> "$OUT/aprime-on-$c.dump" > /dev/null
    main_only "$OUT/$TAG-on-$c.dump" > "$OUT/$TAG-on-$c.main"
    main_only "$OUT/aprime-on-$c.dump" > "$OUT/aprime-on-$c.main"
    cmp -s "$OUT/$TAG-on-$c.main" "$OUT/aprime-on-$c.main" || { note "$TAG admission: {main} ON diverge su $c"; ARC=1; }
    PHPR_REG_LOWER=0 PHPR_DUMP_OPS=1 "$B" "$M/$c.php" 2> "$OUT/$TAG-off-$c.dump" > /dev/null
    PHPR_REG_LOWER=0 PHPR_DUMP_OPS=1 "$A" "$M/$c.php" 2> "$OUT/aprime-off-$c.dump" > /dev/null
    cmp -s "$OUT/$TAG-off-$c.dump" "$OUT/aprime-off-$c.dump" || { note "$TAG admission: OFF diverge su $c"; ARC=1; }
  done
  for side in "$TAG:$B" "aprime:$A"; do
    bin="${side#*:}"
    sz=$(nm -n "$bin" 2>/dev/null | grep -A1 "8run_loop17h" | head -2 | python3 -c '
import sys
lines = sys.stdin.read().split("\n")
a = int(lines[0].split()[0], 16); b = int(lines[1].split()[0], 16)
print(b - a)')
    note "$TAG admission: run_loop_size(${side%%:*})=$sz B"
  done
  echo "$ARC" > "$OUT/$TAG-admission.rc"
  if [ "$ARC" != 0 ]; then note "$TAG: ADMISSION FALLITA rc=$ARC (zavorra non nulla o svanita sotto LTO)"; return 4; fi
  note "$TAG admission: dump ON {main} ×6 e OFF al byte IDENTICI ad A′ (rc=0)"

  # parità output (gate PRIMA del cronometro)
  for c in $CATS; do
    "$A" "$M/$c.php" > "$OUT/$TAG-par-$c-A.out" 2>&1
    "$B" "$M/$c.php" > "$OUT/$TAG-par-$c-B.out" 2>&1
    diff -q "$OUT/$TAG-par-$c-A.out" "$OUT/$TAG-par-$c-B.out" > /dev/null || { note "$TAG: output diverge su $c — VIOLAZIONE"; return 2; }
  done

  # A/B ABAB sei categorie
  : > "$TSV"
  local INVALID=""
  for c in $CATS; do
    N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$M/$c.php")
    [ "$c" = arr ] && N=6000000
    FA=$(user_cpu "$A" "$M/empty.php"); FB=$(user_cpu "$B" "$M/empty.php")
    echo "cat=$c N=$N floor_A=$FA floor_B=$FB"
    i=0; x=0
    while [ $i -lt 5 ]; do i=$((i+1))
      TA=$(user_cpu "$A" "$M/$c.php"); TB=$(user_cpu "$B" "$M/$c.php")
      printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$c" "$N" "$i" "$TA" "$TB" "$FA" "$FB" >> "$TSV"
    done
    while [ "$(infam_count "$TSV" "$c")" -lt 5 ] && [ $x -lt 6 ]; do
      x=$((x+1)); i=$((i+1))
      TA=$(user_cpu "$A" "$M/$c.php"); TB=$(user_cpu "$B" "$M/$c.php")
      printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$c" "$N" "$i" "$TA" "$TB" "$FA" "$FB" >> "$TSV"
    done
    [ "$(infam_count "$TSV" "$c")" -lt 5 ] && INVALID="$INVALID $c"
  done
  if [ -n "$INVALID" ]; then note "$TAG: MISURA_INVALIDA:$INVALID"; return 3; fi

  python3 - "$TSV" "$TAG" >> "$VERD" <<'PY'
import sys
from statistics import median
rows = {}
for line in open(sys.argv[1]):
    c, n, i, ta, tb, fa, fb = line.split('\t')
    n, ta, tb, fa, fb = float(n), float(ta), float(tb), float(fa), float(fb)
    rows.setdefault(c, []).append((int(i), (ta-fa)/n*1e9, (tb-fb)/n*1e9))
g = 0.0
for c in ["arith", "prop", "calls", "str", "arr", "re"]:
    na = [r[1] for r in rows[c]]; nb = [r[2] for r in rows[c]]
    tA, tB = 1.3*min(na), 1.3*min(nb)
    fam = [(i, a, b) for i, a, b in rows[c] if a <= tA and b <= tB]
    for i, a, b in rows[c]:
        if not (a <= tA and b <= tB):
            print(f"{sys.argv[2]} {c}: ESCLUSA coppia{i} (A={a:.2f} B={b:.2f}; fam A<={tA:.2f} B<={tB:.2f})")
    Ds = [a-b for _, a, b in fam]
    dmed = median(Ds); banda = abs(round(dmed, 2)); g = max(g, banda)
    pos = sum(1 for d in Ds if d > 0)
    print(f"{sys.argv[2]} {c}: fam={len(Ds)} D_med={dmed:+.2f} segni +{pos}/{len(Ds)} banda={banda:.2f}")
print(f"{sys.argv[2]} BANDA_GLOBALE={g:.2f}")
PY
  tail -8 "$VERD"

  # held-out: |Dnet| R=5 per-binario
  for c in poly err wploop; do
    fa=(); fb=(); ta=(); tb=()
    for i in 1 2 3 4 5; do fa+=("$(user_cpu "$A" "$M/empty.php")"); fb+=("$(user_cpu "$B" "$M/empty.php")"); done
    for i in 1 2 3 4 5; do ta+=("$(user_cpu "$A" "$HD/$c.php")"); tb+=("$(user_cpu "$B" "$HD/$c.php")"); done
    python3 - "$TAG" "$c" "${fa[@]}" "${fb[@]}" "${ta[@]}" "${tb[@]}" >> "$VERD" <<'PY'
import sys
from statistics import median
tag, c = sys.argv[1], sys.argv[2]; v = list(map(float, sys.argv[3:])); R = len(v)//4
fa, fb, ta, tb = v[:R], v[R:2*R], v[2*R:3*R], v[3*R:]
na = median(ta)-median(fa); nb = median(tb)-median(fb)
print(f"{tag} heldout {c}: net_A={na:.2f}s net_B={nb:.2f}s Dnet={na-nb:+.2f}s banda_ho={abs(na-nb):.2f}s")
PY
  done
  tail -3 "$VERD"
  return 0
}

: > "$VERD"
echo "# S-117 ri-banda su A′ (A=$(shasum -a 256 "$A"|cut -c1-8)); zavorre s114+s115-2 RICOSTRUITE sotto lto=fat+cgu=1" >> "$VERD"
RCT=0
misura_zavorra "nulla" "$SRC/wp114-harness/s114-zavorra.patch" || RCT=$?
if [ "$RCT" = 0 ]; then misura_zavorra "nulla2" "$SRC/wp115-harness/s115-zavorra2.patch" || RCT=$?; fi
echo "$RCT" > "$OUT/banda-rc"
note "RI-BANDA rc=$RCT (da banda-out/banda-rc)"
exit "$RCT"
