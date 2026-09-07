#!/bin/bash
# s172-promozione.sh <cand_hash16> <braccio> — gate di promozione LEVA L-SL2
# («forma sigillata Long» fetta 2 = prop: P1 bigramma fuso PropGetSlotRecv+
# BinaryTCPropSetPop sigillato, P2 BinarySTDst sigillato, corpi esatti #[cold])
# — S-172, criterio wp172-harness/s172-criterio.md p.6.
# COPIA DICHIARATA di wp171-harness/s171-promozione.sh (manifest
# s172-promozione-copia-v3.diff + copia-gate v3); DIVERGENZE DICHIARATE:
#  (1) tag s172, harness wp172 (fixture S-171 ereditate in BYTE-COPIA verificata
#      cmp); candidato = hash argomento + BRACCIO promosso (B=P1, C=P1+P2; A/B R=5
#      in s172-leva-verdetto.out: giudice prop-dq, guardia arith-dq); candidato
#      stashato phpr-s172-sl2-<braccio> via pin-phpr.sh --braccio; stash di
#      riferimento per i gate «pin==stash» = phpr-s171 (b360b2933eddfe18);
#      **fx-sl2 NUOVA bilaterale** (presidio DIRETTO L-SL2, oracle==pin già sul
#      pin s171); conferma post-pin = prop-dq (non arith-dq);
#  (2) inventario batteria: baseline s125 + denti — i denti aggiunti S-126..S-170
#      NON sono elencati qui: nomi NUOVI si DICHIARANO (nota), nomi SPARITI
#      fermano (stop); conteggio a verbale; debug_backtrace_array_fields verde;
#  (3) corpus: ZERO flip attesi (i fast path sono sottoinsiemi ESATTI degli arm
#      Long di binary_fast; ogni miss ricomputa il corpo originale) — rc!=0 = STOP;
#  (4) fixture chain s109 INVARIATA (10 gate) + fx-ce/fx-am/fx-af/fx-sm/fx-au
#      bilaterali + sonda-bt/fx-refl/fx-sm-div/fx-au-div pin==stash s171
#      (ereditate in BYTE-COPIA wp171 verificata cmp) + fx-mck bilaterale e
#      fx-mc/fx-mc2/fx-mc2-fib pin==stash s171 (S-165/166, §3.28/§3.29) +
#      **fx-sl1 NUOVA bilaterale** (presidio DIRETTO L-SL1: overflow, shift,
#      Div/Mod/Pow, Ref/Double/stringa/null/bool, dst==l, typed by-ref, IncDec ai
#      limiti, CmpJmpSC 8 forme; oracle con `-d log_errors=0 -d display_errors=1`
#      perché il CLI oracle DUPLICA i diag su stderr con log_errors — dichiarato,
#      la fixture bilaterale non emette diag) + **fx-sl1-div NUOVA** pin==stash
#      s171 (forme con diag: §3.11 + §3.13 PRE-esistenti, INVARIATE);
#  (5) conferma POST-PIN: arith-dq R=5 pin s172 vs stash phpr-s171 (attesa segno
#      +, intorno del D_dq dell'A/B; rumore > attesa/2 ⇒ SOLO SEGNO);
#  (6) disasm run_loop AGLI ATTI in fase A/B (ab-out/disasm-sl1.out), non qui;
#  (7) micro R=5 = run-micro.sh (scoreboard al pin nuovo) — INVARIATO.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp172-harness"
BIN="$HOME/Claude/php-rust-output/release/phpr"
RUNNER="$HOME/Claude/php-rust-output/release/phpt-runner"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
WD="/Volumes/Extreme Pro/Claude/wp13-harness/run-with-watchdog.sh"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
QUIESCE="$SRC/wp129-harness/s129-quiescenza.sh"
SP="${PROMO_SP:?PROMO_SP (workdir APFS per i gate ORM/hk) richiesto}"
OUT="$H/promo-out"; mkdir -p "$OUT"
VERD="$H/s172-promo-verdetto.out"
CAND_EXP="${1:?cand}"; ARM="${2:?braccio}"; CAND_EXP_PIN="${3:?pin s172 atteso}"
REF="$STASH/phpr-s171"; REF_EXP="b360b2933eddfe18"
note(){ echo "$1"; echo "$1" >> "$VERD"; }
stop(){ note "$1"; echo 1 > "$OUT/rcb"; exit 1; }

# verifica POSITIVA dei path d'ingresso (emenda §3 proposta S-160, lezione #21)
for f in "$H/fx-ce.php" "$H/fx-am.php" "$H/fx-af.php" "$H/fx-refl.php" \
         "$H/fx-sm.php" "$H/fx-sm-div.php" "$H/fx-au.php" "$H/fx-au-div.php" \
         "$H/fx-mc.php" "$H/fx-mc2.php" "$H/fx-mc2-fib.php" "$H/fx-mck.php" \
         "$H/fx-sl1.php" "$H/fx-sl1-div.php" "$H/fx-sl2.php" "$H/fx-sl2-div.php" "$H/prop-dq.php" \
         "$H/sonda-bt-autoload.php" "$SRC/wp164-harness/arith-dq.php" \
         "$H/empty.php" "$QUIESCE" "$WD" "$GATES/orm-work.tgz" "$GATES/hk-work.tgz" \
         "$SRC/wp125-harness/orm-baseline-failnames.txt" \
         "$SRC/wp125-harness/promo-out/batteria-nomi.txt" \
         "$REF" "$STASH/phpr-s172-sl2-$ARM"; do
  [ -s "$f" ] || stop "PRE: path d'ingresso MANCANTE: $f — STOP"
done
[ "$(shasum -a 256 "$REF" | cut -c1-16)" = "$REF_EXP" ] || stop "PRE: stash phpr-s171 hash != $REF_EXP — STOP"

cd "$SRC" || exit 4
git diff --quiet -- crates/ || stop "PRE: crates/ sporco — STOP"
[ -e /private/tmp/phpr-measure.lock ] && note "lock CI: presente (finestra di sessione, non lo tocco)" || note "lock CI: ASSENTE — la sessione lo doveva creare (proseguo, dichiarato)"

# ---- RIPRESA S-172 (copia dichiarata della coda di s172-promozione.sh dal corpus-gate in poi) ----
# Tentativo 2: build ricetta (identità a contenuto), batteria rc=0 1748, re-build al byte e
# pin-phpr.sh s172 COMPLETATI (binario 5f2dff7d17ebed79 in posto, stash phpr-s172, registro
# committato fc632c8b); pin-phpr.sh ha reso rc=1 SOLO sul push (collisione col push manuale del
# catalogo §3.30 durante la catena = incidente di processo #2, contato). Il push è stato rifatto
# a mano (origin == main). Si riparte dal corpus-gate con PRE-condizioni verificate qui sotto.
H2=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$H2" = "$CAND_EXP_PIN" ] || stop "RIPRESA: binario canonico $H2 != pin s172 atteso $CAND_EXP_PIN — STOP"
[ "$(shasum -a 256 "$STASH/phpr-s172" | cut -c1-16)" = "$H2" ] || stop "RIPRESA: stash phpr-s172 != $H2 — STOP"
grep -q "5f2dff7d17ebed79" "$SRC/PIN_REGISTRY.md" || stop "RIPRESA: registro senza il pin s172 — STOP"
note "RIPRESA dal corpus-gate: pin s172 = $H2 (binario == stash phpr-s172 == registro), tentativo 2 dopo pin-phpr rc=1 sul solo push (incidente #2 dichiarato)"
"$SRC/scripts/corpus-gate.sh" "$RUNNER" "$OUT/corpus"
crc=$?; echo "$crc" > "$OUT/corpus-rc"
[ "$crc" = 0 ] || stop "corpus-gate rc=$crc — divergenza (3) non ammette flip: STOP"
note "promozione corpus-gate: rc=0 — nomi==congelato (1412), CONTENUTO==golden, off-on zero (ZERO flip come atteso, L-SL1)"

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
# gate INVARIANTE: pin==stash s171 byte-id (+ marcatore quando preteso)
invar(){ # $1=nome $2=file $3=marcatore("" = nessuno) $4=descrizione
  local n="$1" f="$2" m="$3" d="$4"
  "$REF" "$f" > "$OUT/$n-s171.out" 2>&1
  "$BIN" "$f" > "$OUT/$n-pin.out" 2>&1
  [ -s "$OUT/$n-pin.out" ] || stop "gate $n: output pin VUOTO"
  [ -z "$m" ] || grep -q "$m" "$OUT/$n-pin.out" || stop "gate $n: MARCATORE $m assente"
  if diff -q "$OUT/$n-s171.out" "$OUT/$n-pin.out" > /dev/null; then
    note "promozione gate $n: pin==stash s171 BYTE-ID ($d)"
  else
    diff "$OUT/$n-s171.out" "$OUT/$n-pin.out" > "$OUT/$n.diff" || true
    stop "gate $n: pin DIVERGE dallo stash s171 (promo-out/$n.diff)"
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

QOK=1
for t in $(seq 1 30); do
  if "$QUIESCE" "$OUT/quiesce.rc" > "$OUT/quiesce-t$t.log" 2>&1; then QOK=0; note "promozione quiescenza: PASS al tentativo $t"; break; fi
  sleep 60
done
[ "$QOK" = 0 ] || stop "quiescenza MAI PASS in 30 tentativi — STOP prima delle misure"

PHPR="$BIN" R=5 "$SRC/wp97-harness/micro/run-micro.sh" > "$OUT/micro-pin-s172.out" 2>&1
note "promozione micro pin s172: $(grep -E '^rapporto_' "$OUT/micro-pin-s172.out" | tr '\n' ' ')"

# ---- conferma POST-PIN prop-dq (divergenza (5)): R=5 pin s172 vs stash s171 ----
DQ="$H/prop-dq.php"
NDQ=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$DQ")
ucpu(){ { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$1" "$2" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3(){ local a b c; a=$(ucpu "$1" "$2"); b=$(ucpu "$1" "$2"); c=$(ucpu "$1" "$2"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }
FA=$(floor3 "$REF" "$SRC/wp160-harness/empty.php"); FB=$(floor3 "$BIN" "$SRC/wp160-harness/empty.php")
CT="$OUT/conferma-runs.tsv"; : > "$CT"
for i in 1 2 3 4 5; do
  if [ $((i % 2)) -eq 1 ]; then TA=$(ucpu "$REF" "$DQ"); TB=$(ucpu "$BIN" "$DQ"); else TB=$(ucpu "$BIN" "$DQ"); TA=$(ucpu "$REF" "$DQ"); fi
  printf '%s\t%s\t%s\t%s\n' "$TA" "$TB" "$FA" "$FB" >> "$CT"
done
CONF=$(python3 - "$CT" "$NDQ" <<'PY'
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
)
note "conferma post-pin prop-dq (pin s172 vs stash s171): $CONF (attesa: segno +, D nell'intorno del D_prop del braccio promosso nell'A/B; rumore > attesa/2 ⇒ SOLO SEGNO)"

# ---- gate ORM per NOME ----
rm -rf "$SP/orm-work" && mkdir -p "$SP" && tar xzf "$GATES/orm-work.tgz" -C "$SP" || stop "untar orm-work"
( cd "$SP/orm-work" && "$WD" -t 2400 -s 600 -p "$OUT/orm.log" -o "$OUT" -- \
    "$BIN" vendor/bin/phpunit --no-coverage > "$OUT/orm.log" 2>&1 )
orc=$?; echo "$orc" > "$OUT/orm.rc"
SUMM=$(tr -d '\0' < "$OUT/orm.log" | grep -E "^(Tests:|OK)" | tail -1)
tr -d '\0' < "$OUT/orm.log" | sed -n 's/^[0-9][0-9]*) \(.*\)$/\1/p' | sort -u > "$OUT/orm-failnames.txt"
if diff -q "$SRC/wp125-harness/orm-baseline-failnames.txt" "$OUT/orm-failnames.txt" > /dev/null; then
  note "promozione gate ORM: fail-set per NOME == baseline (16 nomi) · $SUMM"
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
  note "promozione gate http-kernel: 0E/0F · $HSUMM"
else
  stop "gate http-kernel: E=$HE F=$HF (attesi 0/0) · $HSUMM"
fi

"$SRC/scripts/pin-server.sh" s172 > "$OUT/pin-server.log" 2>&1
src_rc=$?; echo "$src_rc" > "$OUT/pin-server.rc"
[ "$src_rc" = 0 ] || stop "pin-server.sh rc=$src_rc"
note "promozione server: $(grep '^PIN server' "$OUT/pin-server.log" | tail -1)"

echo 0 > "$OUT/rcb"
note "PROMOZIONE COMPLETA rc=0 (da promo-out/rcb): pin s172 = $H2"
exit 0
