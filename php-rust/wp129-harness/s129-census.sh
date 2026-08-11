#!/bin/bash
# s129-census.sh — ARBITRO census identità H-cloni (criterio s129-criterio-tempo.md
# p.2, committato+emendato PRIMA di questo codice). Stesso epoch, due fasi:
#   A = clite (baseline: deve RIPRODURRE i numeri S-128)
#   B = clite + name-borrow (variante: deve dare le predizioni ESATTE)
# R=2 identici; per-iter=(cat-empty)/3e6; parità stdout vs pin per ogni sonda.
# rc autoritativo = SOLO census-out/census.rc scritto QUI.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp129-harness"
M="$REPO/wp127-harness/micro-orm"
P="$REPO/wp128-harness/probe"
OUT="$H/census-out"; mkdir -p "$OUT"
CTL="/Volumes/Extreme Pro/Claude/phpr-census-l1-target"
PIN="$HOME/Claude/php-rust-output/release/phpr"
VERD="$H/s129-census-verdetto.out"
fail(){ echo "$1" | tee -a "$VERD"; echo 1 > "$OUT/census.rc"; git -C "$REPO" checkout -- . 2>/dev/null; exit 1; }

cd "$REPO" || exit 2
git diff --quiet || fail "PRE: tree sporco — STOP"
[ "$(shasum -a 256 "$PIN" | cut -c1-16)" = "ccb63dcaf565cffc" ] || fail "pin!=s127b"
CATS="empty objalloc objallocni objdatains objdropdef objmap objchurn"
PRBS="p2-append p3-local p4-int p5-two p6-overwrite"

src_of(){ case "$1" in p2-append|p3-local|p4-int|p5-two|p6-overwrite) echo "$P/$1.php";; *) echo "$M/$1.php";; esac; }

build_phase(){ # $1 = fase (A|B)
  git apply wp119-harness/census-clite.patch || fail "clite.patch NON applica (fase $1)"
  if [ "$1" = B ]; then
    git apply wp129-harness/name-borrow.patch || fail "name-borrow.patch NON applica"
  fi
  CARGO_TARGET_DIR="$CTL" SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 \
    cargo build --release -p php-cli --features mem-census > "$OUT/build-$1.log" 2>&1
  brc=$?
  git checkout -- . || fail "ripristino tree fallito (fase $1)"
  git diff --quiet || fail "POST-build fase $1: tree NON pulito — STOP"
  [ "$brc" = 0 ] || fail "build fase $1 FALLITA (build-$1.log)"
}

run_phase(){ # $1 = fase (A|B)
  LBIN="$CTL/release/phpr"
  for c in $CATS $PRBS; do
    s="$(src_of "$c")"
    "$LBIN" "$s" > "$OUT/$1-$c.out" 2>&1
    if [ "$1" = A ]; then "$PIN" "$s" > "$OUT/pin-$c.out" 2>&1; fi
    diff -q "$OUT/$1-$c.out" "$OUT/pin-$c.out" > /dev/null || fail "fase $1: stdout DIVERGE su $c"
    for r in 1 2; do
      f="$OUT/cen-$1-$c-r$r.census"; rm -f "$f"
      PHPR_MEM_CENSUS="$f" MIMALLOC_PURGE_DELAY=0 "$LBIN" "$s" > /dev/null 2>&1
      grep -q "^gacensus " "$f" || fail "gacensus ASSENTE in $1-$c-r$r"
    done
  done
}

build_phase A; run_phase A
build_phase B; run_phase B

python3 - "$OUT" > "$VERD" <<'PY'
import re, sys
out = sys.argv[1]
N = 3_000_000
BASE = {"objalloc": 7.0, "objallocni": 5.0, "objdatains": 12.0, "objdropdef": 8.0,
        "objmap": 1.0, "objchurn": 13.0, "p2-append": 11.0, "p3-local": 10.0,
        "p4-int": 12.0, "p5-two": 15.0, "p6-overwrite": 16.0}
# predizioni H-cloni EMENDATE (−2 per statement FieldAssign/iter)
VAR = {"objalloc": 7.0, "objallocni": 5.0, "objdatains": 10.0, "objdropdef": 8.0,
       "objmap": 1.0, "objchurn": 11.0, "p2-append": 9.0, "p3-local": 10.0,
       "p4-int": 10.0, "p5-two": 11.0, "p6-overwrite": 12.0}
def counters(ph, cat, r):
    for line in open(f"{out}/cen-{ph}-{cat}-r{r}.census"):
        if line.startswith("gacensus "):
            return {k: int(v) for k, v in (kv.split("=") for kv in re.findall(r"\w+=\d+", line))}
    raise SystemExit(f"gacensus assente: {ph}-{cat}-{r}")
print("== s129 census identità H-cloni (criterio p.2 EMENDATO; baseline S-128 + variante name-borrow, stesso epoch) ==")
print("grade=VERDICT  # conteggi deterministici; per-iter=(cat-empty)/3e6; rc autoritativo = census-out/census.rc")
rc = 0
for ph, exp in (("A", BASE), ("B", VAR)):
    print(f"-- fase {ph} ({'baseline clite' if ph=='A' else 'clite + name-borrow'}) --")
    for cat in ["objalloc", "objallocni", "objdatains", "objdropdef", "objmap", "objchurn",
                "p2-append", "p3-local", "p4-int", "p5-two", "p6-overwrite"]:
        per = []
        for r in (1, 2):
            c, e = counters(ph, cat, r), counters(ph, "empty", r)
            per.append((round((c["galloc_n"]-e["galloc_n"])/N, 2), round((c["gfree_n"]-e["gfree_n"])/N, 2)))
        if per[0] != per[1]:
            print(f"{cat}: R=2 NON identici {per} — NON ACQUISITA"); rc = 3; continue
        a, f = per[0]
        tag = "PRED-OK" if abs(a-exp[cat]) < 0.005 else f"PRED-SCARTO(atteso {exp[cat]:.2f})"
        if "SCARTO" in tag: rc = rc or 4
        print(f"{cat:12s} alloc/iter={a:.2f} free/iter={f:.2f}  [{tag}]")
print("H-CLONI: " + ("CONFERMATA 22/22" if rc == 0 else "NON CONFERMATA — indagine (criterio p.2)"))
sys.exit(rc)
PY
prc=$?
echo "$prc" > "$OUT/census.rc"
echo "done rc=$prc $(date +%T)" > "$OUT/census.done"
exit "$prc"
