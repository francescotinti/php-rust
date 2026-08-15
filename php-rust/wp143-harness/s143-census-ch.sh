#!/bin/bash
# s143-census-ch.sh — census CH_* PER CLASSE (eventi cum_n + buffer s143_* +
# denominatore galloc_n STESSA run) su SUITE ORM, monobinario census (feature
# zval-census,mem-census). Criterio: s143-criterio-istruttoria.md (regola di
# decisione PRE-REGISTRATA p.3). MAI cifra di tempo da qui.
# Base DICHIARATA: wp142-harness/s142-census-rd1.sh (adattamenti: doppio dump
# PHPR_MEM_CENSUS+PHPR_ZVAL_CENSUS, smoke su s143.galloc_n, quota python dalla
# regola p.3, path wp143).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp143-harness"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
CENSUS_PHPR="${CENSUS_PHPR:?binario census richiesto}"
SP="${CENSUS_SP:?workdir APFS richiesto}"
OUT="$H/census-out"; mkdir -p "$OUT"
VERD="$H/s143-census-verdetto.out"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
: > "$OUT/progress.txt"
rm -f "$OUT/census.done"

# criterio p.6.iii: smoke — il probe deve emettere i contatori s143.* nel dump
# MEM (oggetto con props + array con elementi ⇒ propsbuf_n>=1, arrbuf_n>=1)
printf '<?php class C { public $x=1; public $y=2; } $o=new C; $o->x=3; $a=[1,"x",2.5]; $a[]=4; echo "ok\n";' > "$SP/smoke143.php"
rm -f "$OUT/smoke-mem.txt"
PHPR_MEM_CENSUS="$OUT/smoke-mem.txt" "$CENSUS_PHPR" "$SP/smoke143.php" > "$SP/smoke143.out" 2>&1
grep -q "s143.galloc_n" "$OUT/smoke-mem.txt" || { echo "rc=8 probe MUTO sui contatori s143.*" > "$OUT/census.done"; exit 8; }
p "smoke PASS (dump s143.* vivo)"

busy(){ { pgrep -x rustc || pgrep -x cargo || pgrep -f rust-analyzer; } > /dev/null 2>&1 && echo BUSY || echo CLEAN; }

for rep in 1 2; do
  rm -rf "$SP/orm-work"; tar xzf "$GATES/orm-work.tgz" -C "$SP" || { echo "rc=7 untar" > "$OUT/census.done"; exit 7; }
  cd "$SP/orm-work" || { echo "rc=7 cd" > "$OUT/census.done"; exit 7; }
  RAW="$OUT/census-mem-r$rep-$(date +%Y%m%d).txt"; rm -f "$RAW"
  RAWZ="$OUT/census-zval-r$rep-$(date +%Y%m%d).txt"; rm -f "$RAWZ"
  echo "rep$rep pre=$(busy)" >> "$OUT/sentinelle.txt"
  p "rep$rep START"
  PHPR_MEM_CENSUS="$RAW" PHPR_ZVAL_CENSUS="$RAWZ" "$CENSUS_PHPR" vendor/bin/phpunit --no-coverage > "$OUT/run$rep.txt" 2>&1 &
  PID=$!
  ( sleep 1800; kill -9 "$PID" 2>/dev/null ) & WDPID=$!
  wait "$PID"; RUNRC=$?
  kill "$WDPID" 2>/dev/null; wait "$WDPID" 2>/dev/null
  echo "rep$rep post=$(busy)" >> "$OUT/sentinelle.txt"
  p "rep$rep rc=$RUNRC"
  echo "rep$rep run_rc=$RUNRC raw=$(basename "$RAW")" >> "$OUT/runrc.txt"
  tr -d '\0' < "$OUT/run$rep.txt" | sed -n 's/^[0-9][0-9]*) \(.*\)$/\1/p' | sort -u > "$OUT/rep$rep.failnames"
done

RC=0
for rep in 1 2; do
  diff -q "$REPO/wp125-harness/orm-baseline-failnames.txt" "$OUT/rep$rep.failnames" > /dev/null || { p "rep$rep parita' DIVERGE"; RC=1; }
done

python3 - "$OUT" > "$OUT/quota.txt" <<'PY'
import glob, re, sys
out = sys.argv[1]
# Chiavi dal dump mem-census (righe tag=exit, somma per-pid) — criterio p.2.
MKEYS = ["str.cum_n","arr.cum_n","obj.cum_n",
         "s143.arrbuf_n","s143.propsbuf_n","s143.galloc_n","s143.gfree_n"]
per_rep, zval_rep, zsize = {}, {}, None
for f in sorted(glob.glob(f"{out}/census-mem-r*.txt")):
    rep = re.search(r"census-mem-(r\d)", f).group(1)
    tot = dict.fromkeys(MKEYS, 0)
    for line in open(f, errors="replace"):
        if " tag=exit " not in line and not line.rstrip().endswith("tag=exit"):
            if "tag=exit" not in line: continue
        kv = dict(re.findall(r"([\w.]+)=(-?\d+)", line))
        for k in MKEYS:
            if k in kv: tot[k] += int(kv[k])
        if "s143.zval_size" in kv: zsize = int(kv["s143.zval_size"])
    per_rep[rep] = tot
for f in sorted(glob.glob(f"{out}/census-zval-r*.txt")):
    rep = re.search(r"census-zval-(r\d)", f).group(1)
    tot = {"galloc_bytes":0,"gfree_bytes":0}
    for line in open(f, errors="replace"):
        if not line.startswith("alloccensus "): continue
        kv = dict(re.findall(r"(\w+)=(\d+)", line))
        for k in tot:
            if k in kv: tot[k] += int(kv[k])
    zval_rep[rep] = tot
for rep, tot in sorted(per_rep.items()):
    print(f"{rep}: " + " ".join(f"{k}={tot[k]}" for k in MKEYS))
for rep, tot in sorted(zval_rep.items()):
    print(f"{rep} bytes: galloc={tot['galloc_bytes']} gfree={tot['gfree_bytes']}")
print(f"zval_size={zsize}  # size_of::<Zval>() dichiarato (Hejlsberg R3)")
reps = [per_rep[r] for r in sorted(per_rep)]
if len(reps) == 2:
    for k in MKEYS:
        a, b = reps[0][k], reps[1][k]
        m = max(a, b)
        if m and abs(a-b)/m > 0.01:
            print(f"DICHIARA scarto>1% su {k}: r1={a} r2={b} (criterio p.6.ii)")
if reps:
    t = reps[0]; g = t["s143.galloc_n"] or 1
    q_obj = 100*(t["obj.cum_n"] + t["s143.propsbuf_n"])/g
    q_arr = 100*(t["arr.cum_n"] + t["s143.arrbuf_n"])/g
    q_str = 100*t["str.cum_n"]/g
    q_oth = 100 - q_obj - q_arr - q_str
    print(f"quota_obj={q_obj:.2f}% (box {100*t['obj.cum_n']/g:.2f}% + propsbuf {100*t['s143.propsbuf_n']/g:.2f}%)")
    print(f"quota_arr={q_arr:.2f}% (box {100*t['arr.cum_n']/g:.2f}% + arrbuf {100*t['s143.arrbuf_n']/g:.2f}%)")
    print(f"quota_str={q_str:.2f}%")
    print(f"quota_other={q_oth:.2f}%  # include ref/vecargs/Vec-interni/realloc-alloc: NON attribuiti in questa tranche (DICHIARATO, criterio p.2)")
    print("REGOLA p.3 (pre-registrata): quota_obj >=40% => A-poi-B ; <25% => B sola/B-poi-A ; 25-40% => RICONVOCA su terza via")
    if q_obj >= 40: print(f"ESITO REGOLA: quota_obj {q_obj:.1f}% >= 40% => A-poi-B (A ricondizionata pool+refcount)")
    elif q_obj < 25: print(f"ESITO REGOLA: quota_obj {q_obj:.1f}% < 25% => B sola / B-poi-A")
    else: print(f"ESITO REGOLA: quota_obj {q_obj:.1f}% in 25-40% => RICONVOCA il concilio su terza via")
    print("NOTA: quote = eventi ai funnel di tipo / galloc_n stessa run; clausola memcpy>=60% (Klabnik) fuori perimetro di questa tornata (profilo, non census)")
PY

{ echo "== s143 census CH_* PER CLASSE su SUITE ORM (criterio s143-criterio-istruttoria.md — CONTEGGI, mai tempo) =="
  echo "grade=CENSUS  # monobinario census, quote = eventi/galloc_n stessa run (denominatore dal sorgente)"
  echo "census_phpr=$(shasum -a 256 "$CENSUS_PHPR" | cut -c1-16)  # probe build, MAI parita'"
  cat "$OUT/runrc.txt"
  echo "parita' per NOME vs baseline16: rc=$RC (0=entrambe ==)"
  echo "--- sentinelle (non-gate, criterio p.7) ---"
  cat "$OUT/sentinelle.txt"
  echo "--- contatori e quote ---"
  cat "$OUT/quota.txt"
  for rep in 1 2; do
    echo "summary rep$rep: $(tr -d '\0' < "$OUT/run$rep.txt" | sed -n 's/^\(Tests: .*\)$/\1/p' | tail -1)"
  done
} > "$VERD" 2>&1
echo "rc=$RC $(date +%T)" > "$OUT/census.done"
p "DONE rc=$RC"
