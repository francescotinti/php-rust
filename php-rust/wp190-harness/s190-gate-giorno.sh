#!/bin/bash
# s190-gate-giorno.sh <tag> — S-190 cura O(1) v2, gate di GIORNO (criterio s190-criterio-o1v2.md p.5; dev-release,
# NON verdetto): (1) i 5 test del corpus rotti dalla v1, nei DUE modi (PHPR_REG_LOWER=0/1): bug49893 deve PASSARE,
# gli altri quattro devono dare il chunk col digest sha256 == golden congelato (wp109-harness/corpus-gate/golden-content-
# {off,on}.tsv, stessa macchina chunk di scripts/corpus-gate.sh); (2) 9 fixture bilaterali contro l'oracle (fx-mc2 contro
# wp186-harness/fx-mc2.expected-diff, VUOTO = byte-id richiesta). Esito in s190-gate-giorno-<tag>.out; rc 0/1.
# Variabili: PHPR (phpr dev-release), RUNNER (phpt-runner dev-release). Generato con assistenza AI (Claude Fable 5).
# EMENDA 04/11 00:1x (dichiarata il 04/11 10:xx, rilievo 4 revisione S-190): il giudice per DIGEST del chunk vale solo nel run
# intero (la coda del chunk in un run a file singolo differisce), quindi dal passo 2 si giudica il TESTO del chunk contro il
# chunk di riferimento, e il riferimento è a sua volta verificato per digest contro il golden committato. Riferimento IN REPO:
# golden-chunks/{off,on}.norm (estratti il 04/11 dal norm della CI 8d5f1598a41b di main, 02/11, corpus-gate OK); REF_NORM_DIR
# lo sovrascrive. REPLAY=1 (rilievo 5): ri-giudica i log già in gate-out/<tag> senza rilanciare i binari.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp190-harness"
TAG="${1:?tag (es. c1-passo1)}"
PHPR="${PHPR:-$HOME/Claude/phpr-target/dev-output/dev-release/phpr}"
RUNNER="${RUNNER:-$HOME/Claude/phpr-target/dev-output/dev-release/phpt-runner}"
ORACLE=/opt/homebrew/opt/php/bin/php
ZT="/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests"
FROZ="$REPO/wp109-harness/corpus-gate"
OUT="$H/gate-out/$TAG"; mkdir -p "$OUT"
VERD="$H/s190-gate-giorno-$TAG.out"
RC=0
note(){ echo "$1" | tee -a "$VERD"; }
: > "$VERD"
REPLAY="${REPLAY:-0}"
if [ "$REPLAY" = 1 ]; then
  note "== s190 gate di giorno tag=$TAG REPLAY=1: rigiudicato $(date +%F_%H:%M:%S) dai log di gate-out/$TAG (mtime $(stat -f %Sm -t %F_%T "$OUT")); binari NON rilanciati =="
else
  [ -x "$PHPR" ] && [ -x "$RUNNER" ] || { note "FAIL setup: binari dev-release assenti ($PHPR / $RUNNER)"; exit 2; }
  note "== s190 gate di giorno tag=$TAG git=$(git -C "$REPO" rev-parse --short HEAD) phpr=$(shasum -a 256 "$PHPR" | cut -c1-16) runner=$(shasum -a 256 "$RUNNER" | cut -c1-16) $(date +%F_%H:%M:%S) (dev-release: NON verdetto) =="
fi

# (1) i cinque test, due modi
FIVE="bug49893.phpt exceptions/bug47771.phpt exceptions/bug53511.phpt coalesce/assign_coalesce_002.phpt temporary_cleaning/temporary_cleaning_013.phpt"
for m in off on; do
  flag=0; [ "$m" = on ] && flag=1
  for t in $FIVE; do
    f="$ZT/$t"; [ -f "$f" ] || { note "FAIL: manca $f"; RC=1; continue; }
    n=$(basename "$t" .phpt)
    if [ "$REPLAY" = 1 ]; then [ -s "$OUT/$n.$m.log" ] || { note "FAIL [$m] $n: REPLAY senza log"; RC=1; continue; }
    else PHPR_REG_LOWER=$flag perl -e 'alarm 120; exec @ARGV or die' -- "$RUNNER" --list-fails --isolate "$f" 2>&1 | tr -d '\0' > "$OUT/$n.$m.log"; fi
    python3 - "$OUT/$n.$m.log" "$f" "$FROZ/golden-content-$m.tsv" "$FROZ/corpus-s109-$m.fails" "$m" "$n" "${REF_NORM_DIR:-$H/golden-chunks}/$m.norm" <<'PY' | tee -a "$VERD"
import hashlib, re, sys
# Giudice: in un run a file singolo la CODA del chunk (fino a fine output) differisce da quella del run intero, quindi il
# digest grezzo non è confrontabile: si confronta il TESTO del chunk (rstrip) col chunk di riferimento estratto dal norm
# della CI che ha prodotto il golden (8d5f1598, corpus-gate OK) e si verifica che quel riferimento abbia il digest golden.
log, path, golden, frozen, mode, name, refnorm = sys.argv[1:8]
txt = open(log, encoding="utf-8", errors="replace").read()
reftxt = open(refnorm, encoding="utf-8", errors="replace").read()
refchunks = dict(re.findall(r"^--- (.+?) ---\n(.*?)(?=^--- .+? ---$|\Z)", reftxt, re.S | re.M))
m = re.search(r"^failures: (\d+)$", txt, re.M)
mf = re.search(r"^fail:\s+(\d+)$", txt, re.M)
fails = int(m.group(1)) if m else (int(mf.group(1)) if mf else -1)  # the runner omits "failures:" when 0
chunks = {k: v for k, v in re.findall(r"^--- (.+?) ---\n(.*?)(?=^--- .+? ---$|\Z)", txt, re.S | re.M)}
gold = {}
for line in open(golden, encoding="utf-8"):
    line = line.rstrip("\n")
    if "\t" in line:
        d, p = line.split("\t", 1); gold[p] = d
froz = set(l.strip() for l in open(frozen, encoding="utf-8") if l.strip())
expected_fail = path in froz
if not expected_fail:
    ok = fails == 0 and path not in chunks
    print(f"{'OK  ' if ok else 'FAIL'} [{mode}] {name}: atteso PASS (non nel congelato) — failures={fails}" + ("" if ok else " ⇒ pass→fail (KS: zero pass→fail)"))
else:
    if path not in chunks:
        print(f"FAIL [{mode}] {name}: atteso FAIL col golden, invece PASSA (fail→pass: golden da ri-basare SOLO con voce di catalogo)")
        ok = False
    else:
        ref = refchunks.get(path)
        refd = hashlib.sha256(ref.encode("utf-8", "surrogateescape")).hexdigest() if ref is not None else None
        if ref is None or refd != gold.get(path):
            print(f"FAIL [{mode}] {name}: riferimento CI assente o con digest {str(refd)[:16]} != golden {gold.get(path, 'assente')[:16]} (giudice VOID)")
            ok = False
        else:
            ok = chunks[path].rstrip() == ref.rstrip()
            print(f"{'OK  ' if ok else 'FAIL'} [{mode}] {name}: testo del chunk {'==' if ok else '!='} riferimento golden {refd[:16]}")
            if not ok:
                print("    chunk attuale:"); print("    " + chunks[path].replace("\n", "\n    ").rstrip())
                print("    chunk golden:"); print("    " + ref.replace("\n", "\n    ").rstrip())
sys.exit(0 if ok else 1)
PY
    [ "${PIPESTATUS[0]}" = 0 ] || RC=1
  done
done

# (2) nove fixture bilaterali
bilat(){ # $1=nome $2=file $3=marcatore $4..=opzioni oracle
  local n="$1" f="$2" mk="$3"; shift 3
  [ -f "$f" ] || { note "FAIL: fixture $n assente ($f)"; RC=1; return; }
  if [ "$REPLAY" = 1 ]; then [ -s "$OUT/$n-oracle.out" ] && [ -s "$OUT/$n-tree.out" ] || { note "REPLAY $n: output assenti in gate-out (non rigiudicabile)"; return; }
  else
  "$ORACLE" "$@" "$f" > "$OUT/$n-oracle.out" 2>&1
  perl -e 'alarm 300; exec @ARGV or die' -- "$PHPR" "$@" "$f" > "$OUT/$n-tree.out" 2>&1
  fi
  [ -s "$OUT/$n-tree.out" ] || { note "FAIL $n: output tree VUOTO"; RC=1; return; }
  [ -z "$mk" ] || grep -q "$mk" "$OUT/$n-tree.out" || { note "FAIL $n: marcatore $mk assente"; RC=1; return; }
  if [ "$n" = fxmc2 ]; then
    diff "$OUT/$n-oracle.out" "$OUT/$n-tree.out" > "$OUT/$n.diff" || true
    if cmp -s "$OUT/$n.diff" "$REPO/wp186-harness/fx-mc2.expected-diff"; then note "OK   $n: diff == fx-mc2.expected-diff (vuoto ⇒ byte-id)"; else note "FAIL $n: diff ≠ expected-diff ($(grep -c '^[<>]' "$OUT/$n.diff") righe, $OUT/$n.diff)"; RC=1; fi
  elif cmp -s "$OUT/$n-oracle.out" "$OUT/$n-tree.out"; then note "OK   $n == oracle BYTE-ID"
  else diff "$OUT/$n-oracle.out" "$OUT/$n-tree.out" > "$OUT/$n.diff" || true; note "FAIL $n DIVERGE dall'oracle ($(grep -c '^[<>]' "$OUT/$n.diff") righe, $OUT/$n.diff)"; RC=1; fi
}
bilat fxsw1 "$REPO/wp174-harness/fixtures/fx-sw1.php" "FX-SW1 DONE"
# fx-sw2-gc: fixture di CONTEGGI GC (§3.33), confrontata col PIN (invarianza, come s186-leva-build), non con l'oracle
PIN="$HOME/Claude/php-rust-output/release/phpr"
if [ "$REPLAY" != 1 ]; then
perl -e 'alarm 300; exec @ARGV or die' -- "$PIN" "$REPO/wp174-harness/fixtures/fx-sw2-gc.php" > "$OUT/fxsw2-pin.out" 2>&1
perl -e 'alarm 300; exec @ARGV or die' -- "$PHPR" "$REPO/wp174-harness/fixtures/fx-sw2-gc.php" > "$OUT/fxsw2-tree.out" 2>&1
fi
if [ "$REPLAY" = 1 ] && ! [ -s "$OUT/fxsw2-tree.out" ]; then note "REPLAY fxsw2: output assenti in gate-out (non rigiudicabile)"
elif grep -q "FX-SW2 DONE" "$OUT/fxsw2-tree.out" && cmp -s "$OUT/fxsw2-pin.out" "$OUT/fxsw2-tree.out"; then note "OK   fxsw2 == pin BYTE-ID (invarianza GC)"; else diff "$OUT/fxsw2-pin.out" "$OUT/fxsw2-tree.out" > "$OUT/fxsw2.diff" || true; note "FAIL fxsw2 ≠ pin ($(grep -c '^[<>]' "$OUT/fxsw2.diff") righe, $OUT/fxsw2.diff)"; RC=1; fi
bilat fxsl1 "$REPO/wp172-harness/fx-sl1.php" "FX-SL1 DONE" -d log_errors=0 -d display_errors=1
bilat fxsl2 "$REPO/wp172-harness/fx-sl2.php" "FX-SL2 DONE" -d log_errors=0 -d display_errors=1
bilat fxsl3 "$REPO/wp172-harness/fx-sl3.php" "FX-SL3 DONE" -d log_errors=0 -d display_errors=1
bilat fxcr1 "$REPO/wp181-harness/fx-cr1.php" "FX-CR1 DONE" -d log_errors=0 -d display_errors=1
bilat fxrt1 "$REPO/wp182-harness/fx-rt1.php" "FX-RT1 DONE" -d log_errors=0 -d display_errors=1
bilat fxrt2 "$REPO/wp184-harness/fx-rt2.php" "FX-RT2 DONE" -d log_errors=0 -d display_errors=1
bilat fxmc2 "$REPO/wp185-harness/fx-mc2.php" "FX-MC2 DONE" -d log_errors=0 -d display_errors=1

note "== esito rc=$RC $(date +%F_%H:%M:%S) =="
exit "$RC"
