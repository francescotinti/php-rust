#!/bin/bash
# s181-promozione.sh — gate di PROMOZIONE del tree «pin s180 + L-CR1 Call/Ret magri» (criterio s181-criterio-cr1.md p.5).
# COPIA DICHIARATA di ../wp180-harness/s180-promozione.sh (manifest s181-promozione-copia.diff) coi SOLI adattamenti:
# (1) tag s181 (harness wp181: verdetto s181-promo-verdetto.out, promo-out); (2) CANDIDATO = braccio B dell'A/B
# (ab-out/s181-leva/phpr-B, commit 172814ae): precondizione A/B rc=0 (ab-out/cr1.rc) e HEAD == commit del candidato;
# (3) az.rev. S-180 rilievo 3: HEAD e `git status --porcelain` registrati PRIMA della build; la build ricetta sulla target
# canonica si confronta col braccio B (target separata): hash uguale = identità al byte, diverso = «candidato a
# contenuto» dichiarato (LC_UUID/firma), MAI stop; (4) REF = stash phpr-s180 (884399fc52277119) per i gate invarianti;
# (5) gate bilaterale NUOVO fx-cr1 (marcatore FX-CR1 DONE); (6) corpus: ZERO flip attesi (semantica identica per
# costruzione); (7) conferma post-pin = calls-dq R=5 pin s181 vs stash s180 (STESSA toolchain 1.98.1: cifra attesa ≈ D
# dell'A/B) + prop-dq (guardia); (8) rilievo 4: `hk.rc` nel verdetto accanto al summary; (9) pin-server.sh s181.
# Tutto il resto INVARIATO (testo S-180/S-177 conservato sotto).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp172-harness"
H4="$SRC/wp174-harness"
H8="$SRC/wp181-harness"
BIN="$HOME/Claude/php-rust-output/release/phpr"
RUNNER="$HOME/Claude/php-rust-output/release/phpt-runner"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
WD="/Volumes/Extreme Pro/Claude/wp13-harness/run-with-watchdog.sh"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
QUIESCE="$SRC/wp129-harness/s129-quiescenza.sh"
SP="${PROMO_SP:?PROMO_SP (workdir APFS per i gate ORM/hk) richiesto}"
OUT="$H8/promo-out"; mkdir -p "$OUT"
VERD="$H8/s181-promo-verdetto.out"
disk_guard(){ local g; g=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}'); [ "${g:-0}" -ge 3 ] || stop "guardia disco: Data ${g}G < 3G prima di $1 — STOP"; note "guardia disco: Data ${g}G prima di $1"; }
REF="$STASH/phpr-s180"; REF_EXP="884399fc52277119"
CAND="$H8/ab-out/s181-leva/phpr-B"; CAND_COMMIT="${CAND_COMMIT:-172814ae}"
note(){ echo "$1"; echo "$1" >> "$VERD"; }
stop(){ note "$1"; echo 1 > "$OUT/rcb"; exit 1; }

# verifica POSITIVA dei path d'ingresso (emenda §3 proposta S-160, lezione #21)
for f in "$H/fx-ce.php" "$H/fx-am.php" "$H/fx-af.php" "$H/fx-refl.php" \
         "$H/fx-sm.php" "$H/fx-sm-div.php" "$H/fx-au.php" "$H/fx-au-div.php" \
         "$H/fx-mc.php" "$H/fx-mc2.php" "$H/fx-mc2-fib.php" "$H/fx-mck.php" \
         "$H/fx-sl1.php" "$H/fx-sl1-div.php" "$H/fx-sl2.php" "$H/fx-sl2-div.php" "$H/fx-sl3.php" "$H/fx-sl3-div.php" "$H/prop-dq.php" "$H4/fixtures/fx-sw1.php" \
         "$H8/fx-cr1.php" "$H8/calls-dq.php" "$CAND" "$H8/ab-out/cr1.rc" \
         "$H/sonda-bt-autoload.php" "$SRC/wp164-harness/arith-dq.php" \
         "$H/empty.php" "$QUIESCE" "$WD" "$GATES/orm-work.tgz" "$GATES/hk-work.tgz" \
         "$SRC/wp125-harness/orm-baseline-failnames.txt" \
         "$SRC/wp125-harness/promo-out/batteria-nomi.txt" \
         "$REF"; do
  [ -s "$f" ] || stop "PRE: path d'ingresso MANCANTE: $f — STOP"
done
[ "$(shasum -a 256 "$REF" | cut -c1-16)" = "$REF_EXP" ] || stop "PRE: stash phpr-s180 hash != $REF_EXP — STOP"
[ "$(cat "$H8/ab-out/cr1.rc")" = 0 ] || stop "PRE: A/B cr1 rc=$(cat "$H8/ab-out/cr1.rc") ≠ 0 — nessuna promozione senza nomina — STOP"
CH=$(shasum -a 256 "$CAND" | cut -c1-16)
note "PRE: candidato = braccio B $CH (commit $CAND_COMMIT, A/B cr1 rc=0); REF = stash s180 $REF_EXP"

cd "$SRC" || exit 4
git diff --quiet -- crates/ || stop "PRE: crates/ sporco — STOP"
HEAD0=$(git rev-parse --short HEAD); PORC=$(git status --porcelain | grep -v '^??' | wc -l | tr -d ' ')
note "PRE: HEAD PRIMA della build = $HEAD0 · status --porcelain (non-untracked) = $PORC righe (rilievo 3 S-180)"
git merge-base --is-ancestor "$CAND_COMMIT" HEAD || stop "PRE: il commit del candidato $CAND_COMMIT non è in HEAD — STOP"
[ -e /private/tmp/phpr-measure.lock ] && note "lock CI: presente (finestra di sessione, non lo tocco)" || note "lock CI: ASSENTE — la sessione lo doveva creare (proseguo, dichiarato)"

disk_guard build
SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build.log" 2>&1
rc=$?; echo "$rc" > "$OUT/build.rc"
[ "$rc" = 0 ] || stop "build rc=$rc"
HB=$(shasum -a 256 "$BIN" | cut -c1-16)
if [ "$HB" = "$CH" ]; then note "promozione: build ricetta di HEAD $HEAD0 = braccio B $CH AL BYTE"; else note "promozione: build ricetta di HEAD $HEAD0 = $HB ≠ braccio B $CH (target separata: candidato A CONTENUTO, dichiarato — LC_UUID/firma; il pin nasce da QUESTA build)"; fi
disk_guard batteria

CARGO_INCREMENTAL=0 cargo test --release > "$OUT/batteria.log" 2>&1
brc=$?; echo "$brc" > "$OUT/batteria.rc"
cnt=$(awk '/^test result:/{p+=$4; f+=$6; ig+=$8} END{printf "%d/%d/%d", p, f, ig}' "$OUT/batteria.log")
grep -E '^test .* \.\.\. ' "$OUT/batteria.log" | sed 's/^test //; s/ \.\.\..*//' | sort > "$OUT/batteria-nomi.txt"
normline(){ sed 's/(line [0-9][0-9]*)/(line N)/' "$1" | sort; }
normline "$SRC/wp125-harness/promo-out/batteria-nomi.txt" > "$OUT/base-norm.txt"
normline "$OUT/batteria-nomi.txt" > "$OUT/nomi-norm.txt"
NEW_ONLY=$(comm -13 "$OUT/base-norm.txt" "$OUT/nomi-norm.txt" | sort | tr '\n' ' ' | sed 's/ $//')
GONE=$(comm -23 "$OUT/base-norm.txt" "$OUT/nomi-norm.txt")
NNEW=$(printf '%s' "$NEW_ONLY" | wc -w | tr -d ' ')
if [ -z "$GONE" ]; then
  INV="baseline s125 + $NNEW denti (dichiarati: $NEW_ONLY)"
else
  INV="DIVERGE (SPARITI='$(printf '%s' "$GONE" | tr '\n' ' ')' — diff in promo-out/batteria-inv.diff)"
  diff "$SRC/wp125-harness/promo-out/batteria-nomi.txt" "$OUT/batteria-nomi.txt" > "$OUT/batteria-inv.diff" || true
fi
note "promozione batteria: rc=$brc (da promo-out/batteria.rc) · $cnt · inventario: $INV"
[ "$brc" = 0 ] || stop "batteria rc=$brc"
case "$INV" in DIVERGE*) stop "inventario batteria DIVERGE (nomi spariti)";; esac
grep -E '^test .*debug_backtrace_array_fields' "$OUT/batteria.log" | grep -q ' ok$' \
  && note "promozione batteria: debug_backtrace_array_fields VERDE" \
  || stop "batteria: debug_backtrace_array_fields NON verde/assente"

SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build2.log" 2>&1
rc=$?; echo "$rc" > "$OUT/build2.rc"
[ "$rc" = 0 ] || stop "build2 rc=$rc"
H2=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$H2" = "$HB" ] || stop "re-hash post-batteria $H2 != $HB (churn) — STOP"
note "promozione: churn batteria neutralizzato (build ricetta → $H2 al byte)"

"$SRC/scripts/pin-phpr.sh" s181 > "$OUT/pin.log" 2>&1
prc=$?; echo "$prc" > "$OUT/pin.rc"
[ "$prc" = 0 ] || stop "pin-phpr.sh rc=$prc"
note "promozione: $(tail -1 "$OUT/pin.log")"

disk_guard corpus
"$SRC/scripts/corpus-gate.sh" "$RUNNER" "$OUT/corpus"
crc=$?; echo "$crc" > "$OUT/corpus-rc"
[ "$crc" = 0 ] || stop "corpus-gate rc=$crc — divergenza (6) non ammette flip: STOP"
note "promozione corpus-gate: rc=0 — nomi==congelato (1412), CONTENUTO==golden, off-on zero (ZERO flip come atteso: L-CR1 a semantica identica per costruzione)"

PHPR_PIN_ATTESO="$H2" "$SRC/wp109-harness/s109-fixture-chain.sh" > "$OUT/fixture-chain.out" 2>&1
frc=$?; echo "$frc" > "$OUT/fixture-chain.rc"
[ "$frc" = 0 ] || stop "fixture chain rc=$frc"
FX_ATTESI="hc1 move recv fx20 fx21 w9 preg teardown stash backtrace"
FX_VISTI=$(sed -n 's/^FIXTURE-CHAIN inventario=//p' "$OUT/fixture-chain.out")
[ "$FX_VISTI" = "$FX_ATTESI" ] || stop "fixture chain inventario diverso: visti='$FX_VISTI' attesi='$FX_ATTESI'"
note "promozione fixture chain: rc=0 (10 gate: $FX_VISTI)"

ORACLE=/opt/homebrew/opt/php/bin/php
# gate BILATERALE: oracle==pin byte-id (+ marcatore quando preteso)
bilat(){ # $1=nome $2=file $3=marcatore("" = nessuno) $4=descrizione $5..=opzioni oracle
  local n="$1" f="$2" m="$3" d="$4"; shift 4
  "$ORACLE" "$@" "$f" > "$OUT/$n-oracle.out" 2>&1
  "$BIN" "$f" > "$OUT/$n-pin.out" 2>&1
  [ -s "$OUT/$n-pin.out" ] || stop "gate $n: output pin VUOTO (file/probe rotto)"
  [ -z "$m" ] || grep -q "$m" "$OUT/$n-pin.out" || stop "gate $n: MARCATORE $m assente (lezione #21)"
  if diff -q "$OUT/$n-oracle.out" "$OUT/$n-pin.out" > /dev/null; then
    note "promozione gate $n: oracle==pin BYTE-ID ($d)"
  else
    diff "$OUT/$n-oracle.out" "$OUT/$n-pin.out" > "$OUT/$n.diff" || true
    stop "gate $n: DIVERGE (promo-out/$n.diff)"
  fi
}
# gate INVARIANTE: pin==stash s180 byte-id (+ marcatore quando preteso)
invar(){ # $1=nome $2=file $3=marcatore("" = nessuno) $4=descrizione
  local n="$1" f="$2" m="$3" d="$4"
  "$REF" "$f" > "$OUT/$n-s180.out" 2>&1
  "$BIN" "$f" > "$OUT/$n-pin.out" 2>&1
  [ -s "$OUT/$n-pin.out" ] || stop "gate $n: output pin VUOTO"
  [ -z "$m" ] || grep -q "$m" "$OUT/$n-pin.out" || stop "gate $n: MARCATORE $m assente"
  if diff -q "$OUT/$n-s180.out" "$OUT/$n-pin.out" > /dev/null; then
    note "promozione gate $n: pin==stash s180 BYTE-ID ($d)"
  else
    diff "$OUT/$n-s180.out" "$OUT/$n-pin.out" > "$OUT/$n.diff" || true
    stop "gate $n: pin DIVERGE dallo stash s180 (promo-out/$n.diff)"
  fi
}
bilat fxce "$H/fx-ce.php" "" "12 forme class/interface/trait_exists"
invar sonda-bt "$H/sonda-bt-autoload.php" "" "contratto autoload/backtrace §3.25 invariato"
invar fxrefl "$H/fx-refl.php" "" "contratto __reflect_* L-RF2 invariato"
bilat fxam "$H/fx-am.php" "FXAM-END" "20 forme array_map v2, fast E pieno"
bilat fxaf "$H/fx-af.php" "FXAF-END" "13 forme array_filter, fast E pieno"
bilat fxsm "$H/fx-sm.php" "FX-SM DONE" "forme string-callable, fast E pieno — presidio L-AM2"
invar fxsmdiv "$H/fx-sm-div.php" "FX-SM-DIV DONE" "3 divergenze §3.26 PRE-esistenti INVARIATE"
bilat fxau "$H/fx-au.php" "FX-AU DONE" "10 forme loader array-callable — presidio L-AU1"
invar fxaudiv "$H/fx-au-div.php" "FX-AU-DIV DONE" "divergenza §3.27 PRE-esistente INVARIATA"
bilat fxmck "$H/fx-mck.php" "" "method-call k=3/k=5 — presidio L-MCk (S-166: ==oracle)"
invar fxmc "$H/fx-mc.php" "" "method-call fast path L-MC1 (S-165) invariato"
invar fxmc2 "$H/fx-mc2.php" "" "§3.28 ordine SEND_VAR_EX/dtor temp PRE-esistente INVARIATO"
invar fxmc2fib "$H/fx-mc2-fib.php" "" "§3.29 Fiber non final PRE-esistente INVARIATO"
bilat fxsl1 "$H/fx-sl1.php" "FX-SL1 DONE" "presidio DIRETTO L-SL1: overflow/shift/Div/Mod/Pow/Ref/Double/stringa/null/bool/dst==l/typed-ref/IncDec limiti/CmpJmpSC 8 forme" -d log_errors=0 -d display_errors=1
invar fxsl1div "$H/fx-sl1-div.php" "FX-SL1-DIV DONE" "forme con diag §3.11/§3.13 PRE-esistenti INVARIATE"
bilat fxsl2 "$H/fx-sl2.php" "FX-SL2 DONE" "presidio DIRETTO L-SL2 (forme P1 in loop a 2 iterazioni = IC calda): P1 bigramma fuso e P2 BinarySTDst — overflow/shift/Div/Mod/Pow/Concat/Double/stringa/null/bool/Ref/typed int-float/readonly (anche in scope)/private/hook set/__set/ereditata/dinamica" -d log_errors=0 -d display_errors=1
invar fxsl2div "$H/fx-sl2-div.php" "FX-SL2-DIV DONE" "§3.30 default float da literal int PRE-esistente INVARIATO"
bilat fxsl3 "$H/fx-sl3.php" "FX-SL3 DONE" "presidio DIRETTO fetta 3 (loop a 2 = IC calda): P3 peephole \$s OP= \$o->x e P4 borrow unico recv==slot — overflow/shift/Div/Mod/Pow/Concat/Double/stringa/null/bool/Ref/typed-ref/__get/hook get/enum/dinamica/dst==slot/controlli negativi" -d log_errors=0 -d display_errors=1
invar fxsl3div "$H/fx-sl3-div.php" "FX-SL3-DIV DONE" "§3.32 riga del Deprecated prop dinamica PRE-esistente INVARIATA"
bilat fxsw1 "$H4/fixtures/fx-sw1.php" "FX-SW1 DONE" "presidio DIRETTO Sweep-in-op: ordine statement/distruttori dopo gli op fusi in place, forme fuori dominio (Double/null/concat/overflow/div), P3/P1/P4 con nota a monte, sweep light e finestra main, IN_DESTRUCTOR, pressione GC; back-edge fuso: for Long/Int, miss (Double/str/CmpJmpSS/const-lhs/step2/while), overflow al bordo"
bilat fxcr1 "$H8/fx-cr1.php" "FX-CR1 DONE" "presidio DIRETTO L-CR1 (S-181): arità esatta (ip=1), troppo pochi argomenti (ArgumentCountError Zend su funzione/metodo/statico/3-arietà), surplus+func_get_args, default, by-ref, ricorsione, dtor a fine frame, metodi IC/ereditati, closure/dinamica/cuf/cufa, hint, Ref che decade" -d log_errors=0 -d display_errors=1

# E2 (emenda S-177, lezione: gate s129 cieco ai pesi utente): calma CPU TOTALE <150 % per 4 campioni a 30 s
# PRIMA della quiescenza; fino a 30 tentativi, poi STOP senza micro.
cpu_tot(){ ps -Ao %cpu= | awk '{s+=$1} END{printf "%.0f", s}'; }
E2OK=1
for t in $(seq 1 30); do
  ok=1; smp=""
  for k in 1 2 3 4; do c=$(cpu_tot); smp="$smp $c"; [ "$c" -lt 150 ] || ok=0; sleep 30; done
  if [ "$ok" = 1 ]; then E2OK=0; note "promozione E2 calma CPU: PASS al tentativo $t (campioni:$smp %)"; break; fi
  note "promozione E2 calma CPU: tentativo $t FALLITO (campioni:$smp %)"
done
[ "$E2OK" = 0 ] || stop "E2 calma CPU MAI raggiunta in 30 tentativi — STOP prima delle misure"

QOK=1
for t in $(seq 1 30); do
  if "$QUIESCE" "$OUT/quiesce.rc" > "$OUT/quiesce-t$t.log" 2>&1; then QOK=0; note "promozione quiescenza: PASS al tentativo $t"; break; fi
  sleep 60
done
[ "$QOK" = 0 ] || stop "quiescenza MAI PASS in 30 tentativi — STOP prima delle misure"

PHPR="$BIN" R=5 "$SRC/wp97-harness/micro/run-micro.sh" > "$OUT/micro-pin-s181.out" 2>&1
note "promozione micro pin s181: $(grep -E '^rapporto_' "$OUT/micro-pin-s181.out" | tr '\n' ' ')"

# ---- conferma POST-PIN: R=5 pin s181 vs stash s180 (STESSA toolchain 1.98.1) su calls-dq (bersaglio) e prop-dq (guardia) ----
ucpu(){ { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$1" "$2" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3(){ local a b c; a=$(ucpu "$1" "$2"); b=$(ucpu "$1" "$2"); c=$(ucpu "$1" "$2"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }
FA=$(floor3 "$REF" "$SRC/wp160-harness/empty.php"); FB=$(floor3 "$BIN" "$SRC/wp160-harness/empty.php")
conferma(){ # $1=driver $2=etichetta
  local DQ="$1" NDQ CT
  NDQ=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$DQ")
  CT="$OUT/conferma-$2-runs.tsv"; : > "$CT"
  for i in 1 2 3 4 5; do
    if [ $((i % 2)) -eq 1 ]; then TA=$(ucpu "$REF" "$DQ"); TB=$(ucpu "$BIN" "$DQ"); else TB=$(ucpu "$BIN" "$DQ"); TA=$(ucpu "$REF" "$DQ"); fi
    printf '%s\t%s\t%s\t%s\n' "$TA" "$TB" "$FA" "$FB" >> "$CT"
  done
  python3 - "$CT" "$NDQ" <<'PY'
import sys
rows=[l.split("\t") for l in open(sys.argv[1]).read().strip().split("\n")]
n=float(sys.argv[2])
na=[(float(t[0])-float(t[2]))/n*1e9 for t in rows]
nb=[(float(t[1])-float(t[3]))/n*1e9 for t in rows]
def med(v): s=sorted(v); k=len(s); return s[k//2] if k%2 else (s[k//2-1]+s[k//2])/2
def trange(v):
    m=med(v); w=sorted(v,key=lambda x:(abs(x-m),x))[:-1]; return max(w)-min(w)
d=med(na)-med(nb); noise=max(trange(na),trange(nb))
segni=sum(1 for a,b in zip(na,nb) if a>b)
print(f"D={d:+.2f} rumore={noise:.2f} segni={segni}/5 A={med(na):.2f} B={med(nb):.2f} ns/iter (N={int(n)} dal driver)")
PY
}
note "conferma post-pin calls-dq (pin s181 vs stash s180, stessa toolchain): $(conferma "$H8/calls-dq.php" calls) (attesa: D ≈ cifra dell'A/B cr1)"
note "conferma post-pin prop-dq (guardia, pin s181 vs stash s180): $(conferma "$H/prop-dq.php" prop) (attesa |D| < 1: sola lettura)"

# ---- gate ORM per NOME ----
rm -rf "$SP/orm-work" && mkdir -p "$SP" && tar xzf "$GATES/orm-work.tgz" -C "$SP" || stop "untar orm-work"
( cd "$SP/orm-work" && "$WD" -t 2400 -s 600 -p "$OUT/orm.log" -o "$OUT" -- \
    "$BIN" vendor/bin/phpunit --no-coverage > "$OUT/orm.log" 2>&1 )
orc=$?; echo "$orc" > "$OUT/orm.rc"
SUMM=$(tr -d '\0' < "$OUT/orm.log" | grep -E "^(Tests:|OK)" | tail -1)
tr -d '\0' < "$OUT/orm.log" | sed -n 's/^[0-9][0-9]*) \(.*\)$/\1/p' | sort -u > "$OUT/orm-failnames.txt"
if diff -q "$SRC/wp125-harness/orm-baseline-failnames.txt" "$OUT/orm-failnames.txt" > /dev/null; then
  note "promozione gate ORM: fail-set per NOME == baseline (16 nomi) · $SUMM · orm.rc=$orc (exit PHPUnit, non gate)"
else
  diff "$SRC/wp125-harness/orm-baseline-failnames.txt" "$OUT/orm-failnames.txt" > "$OUT/orm-failnames.diff" || true
  stop "gate ORM: fail-set DIVERGE per NOME (promo-out/orm-failnames.diff) · $SUMM"
fi

# ---- gate http-kernel: 0E/0F ----
rm -rf "$SP/hk-work" && tar xzf "$GATES/hk-work.tgz" -C "$SP" || stop "untar hk-work"
( cd "$SP/hk-work" && "$WD" -t 1800 -s 600 -p "$OUT/hk.log" -o "$OUT" -- \
    "$BIN" vendor/bin/phpunit --no-coverage > "$OUT/hk.log" 2>&1 )
hrc=$?; echo "$hrc" > "$OUT/hk.rc"
HSUMM=$(tr -d '\0' < "$OUT/hk.log" | grep -E "^(Tests:|OK)" | tail -1)
HE=$(printf '%s' "$HSUMM" | sed -n 's/.*Errors: \([0-9]*\).*/\1/p'); HE=${HE:-0}
HF=$(printf '%s' "$HSUMM" | sed -n 's/.*Failures: \([0-9]*\).*/\1/p'); HF=${HF:-0}
if [ "$HE" = 0 ] && [ "$HF" = 0 ] && [ -n "$HSUMM" ]; then
  note "promozione gate http-kernel: 0E/0F · $HSUMM · hk.rc=$hrc (exit PHPUnit per warning/deprecation: il gate è sul summary — rilievo 4 S-180, dichiarato)"
else
  stop "gate http-kernel: E=$HE F=$HF (attesi 0/0) · $HSUMM · hk.rc=$hrc"
fi

"$SRC/scripts/pin-server.sh" s181 > "$OUT/pin-server.log" 2>&1
src_rc=$?; echo "$src_rc" > "$OUT/pin-server.rc"
[ "$src_rc" = 0 ] || stop "pin-server.sh rc=$src_rc"
note "promozione server: $(grep '^PIN server' "$OUT/pin-server.log" | tail -1)"

echo 0 > "$OUT/rcb"
note "PROMOZIONE COMPLETA rc=0 (da promo-out/rcb): pin s181 = $H2"
exit 0
