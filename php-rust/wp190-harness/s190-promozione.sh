#!/bin/bash
# s190-promozione.sh — gate di PROMOZIONE COMPOSTA del tree «pin s183 + cura §3.34/§3.35 (22/10) + O(1) v2 (S-190) + routing
# cli-server in php-types (S-188) + timeout wrapper http §3.39 (S-189) + main 72f688c3 (S-187: PR #3/#4/#7, portabilità Linux)
# + feature census-time 901515b6 (gated, mai attiva nel pin)» → pin phpr s184 (criterio wp190-harness/s190-criterio-o1v2.md
# p.7, NEXT ordine 3). COPIA DICHIARATA di ../wp185-harness/s185-promozione.sh (manifest s190-promozione-copia.diff,
# generatore gen-promozione-s190.py) coi SOLI adattamenti: (1) tag s184 (harness wp190: verdetto s190-promo-verdetto.out,
# promo-out); (2) CANDIDATO = braccio B3 della guardia (ab-out/s190-leva/phpr-B3 ad57fc29, commit 8bff279e): precondizione
# guardia b3 rc=0 (catena-out/catena-s190.done) e commit in HEAD; (3) REF = stash phpr-s183 (9a6ae11bf7e900ef) per i gate
# invarianti; (4) gate wp172 fx-mc2 (§3.28) da INVARIANTE a DICHIARATO: diff(oracle, pin) == fx-mc2-wp172.expected-diff
# (la cura 22/10 ha curato il caso (ii) C4 — == oracle — e mosso la forma del caso (i) C1: né invariante vs s183 né byte-id;
# funzione expdiff NUOVA); gate invariante NUOVO fx-sw2-gc (conteggi GC §3.33 vs stash s183); (5) corpus: ZERO flip attesi
# (corpus-gate ×2 verde sul tree in dev-release — catena c2b — e in CI su cbc3ce5a/a28c376f; nomi 1407 per modo);
# (6) conferma post-pin = ret-buf-dq-k0 R=5 pin s184 vs stash s183 (bersaglio della guardia b3: attesa A−B ≈ −200 ns/iter
# = 65 Ret × −3,08; tetto 65 × 4 = 260) + mcalls-dq (attesa ≈ −2,33) + calls-dq/prop-dq (|D| < 1); (7) pin-server.sh s190
# (il server riporta la cura O(1) v2: s189 portava la v1); (8) ri-dichiarazione delle salite LOC entrate dopo s183 (REGOLE §8).
# Testo S-185/S-184/S-181/S-180/S-177 conservato sotto:
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp172-harness"
H4="$SRC/wp174-harness"
H8="$SRC/wp181-harness"
H9="$SRC/wp184-harness"; H3="$SRC/wp182-harness"
H10="$SRC/wp185-harness"
H11="$SRC/wp190-harness"; H12="$SRC/wp186-harness"
BIN="$HOME/Claude/php-rust-output/release/phpr"
RUNNER="$HOME/Claude/php-rust-output/release/phpt-runner"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
WD="/Volumes/Extreme Pro/Claude/wp13-harness/run-with-watchdog.sh"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
QUIESCE="$SRC/wp129-harness/s129-quiescenza.sh"
SP="${PROMO_SP:?PROMO_SP (workdir APFS per i gate ORM/hk) richiesto}"
OUT="$H11/promo-out"; mkdir -p "$OUT"
VERD="$H11/s190-promo-verdetto.out"
disk_guard(){ local g; g=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}'); [ "${g:-0}" -ge 3 ] || stop "guardia disco: Data ${g}G < 3G prima di $1 — STOP"; note "guardia disco: Data ${g}G prima di $1"; }
REF="$STASH/phpr-s183"; REF_EXP="9a6ae11bf7e900ef"
CAND="$H11/ab-out/s190-leva/phpr-B3"; CAND_COMMIT="${CAND_COMMIT:-8bff279e}"
note(){ echo "$1"; echo "$1" >> "$VERD"; }
stop(){ note "$1"; echo 1 > "$OUT/rcb"; exit 1; }

# verifica POSITIVA dei path d'ingresso (emenda §3 proposta S-160, lezione #21)
for f in "$H/fx-ce.php" "$H/fx-am.php" "$H/fx-af.php" "$H/fx-refl.php" \
         "$H/fx-sm.php" "$H/fx-sm-div.php" "$H/fx-au.php" "$H/fx-au-div.php" \
         "$H/fx-mc.php" "$H/fx-mc2.php" "$H/fx-mc2-fib.php" "$H/fx-mck.php" \
         "$H/fx-sl1.php" "$H/fx-sl1-div.php" "$H/fx-sl2.php" "$H/fx-sl2-div.php" "$H/fx-sl3.php" "$H/fx-sl3-div.php" "$H/prop-dq.php" "$H4/fixtures/fx-sw1.php" \
         "$H8/fx-cr1.php" "$H9/calls-dq.php" "$H3/fx-rt1.php" "$H9/fx-rt2.php" "$H10/fx-mc2.php" "$H10/mcalls-dq.php" "$H12/ret-buf-dq-k0.php" "$H4/fixtures/fx-sw2-gc.php" "$H11/fx-mc2-wp172.expected-diff" "$CAND" "$H11/catena-out/catena-s190.done" \
         "$H/sonda-bt-autoload.php" "$SRC/wp164-harness/arith-dq.php" \
         "$H/empty.php" "$QUIESCE" "$WD" "$GATES/orm-work.tgz" "$GATES/hk-work.tgz" \
         "$SRC/wp125-harness/orm-baseline-failnames.txt" \
         "$SRC/wp125-harness/promo-out/batteria-nomi.txt" \
         "$REF"; do
  [ -s "$f" ] || stop "PRE: path d'ingresso MANCANTE: $f — STOP"
done
[ "$(shasum -a 256 "$REF" | cut -c1-16)" = "$REF_EXP" ] || stop "PRE: stash phpr-s183 hash != $REF_EXP — STOP"
grep -q '^rc=0 ' "$H11/catena-out/catena-s190.done" || stop "PRE: guardia b3 NON verde ($(cat "$H11/catena-out/catena-s190.done")) — nessuna promozione senza guardia rc=0 — STOP"
CH=$(shasum -a 256 "$CAND" | cut -c1-16)
note "PRE: candidato = braccio B3 $CH (commit $CAND_COMMIT, guardia b3 rc=0 04/11 02:45: K0 +3,08 ns/Ret, K piatto, mcalls +2,33, E3 2,17, micro 6/6); REF = stash s183 $REF_EXP"
note "PRE: DICHIARAZIONE dei cambiamenti del pin s184 rispetto a s183 (per nome): (a) cura §3.34/§3.35 (4aa30319, 22/10: dtor dei locali al leave) + cura O(1) v2 (S-190: cursore gc_probe_mark, vm/gc_probe, nota operandi binop lento, leva iv); (b) routing cli-server spostato in php-types (S-188 C1); (c) timeout del wrapper http §3.39 (S-189, php-builtins/http_timeout); inoltre main 72f688c3 (S-187: PR #3/#4/#7 portabilità Linux) e feature census-time (901515b6, gated OFF nel pin). Se t26/t28 differiscono da t25/t27 l'attribuzione va istruita per braccio, mai d'ufficio alla cura O(1)."
note "PRE: salite LOC entrate dopo s183 (REGOLE §8, ri-dichiarate qui come in S-186 #4): vm/mod.rs cap 25990 → 26082 e vm/run.rs cap 7543 → 7611 (commit 4aa30319 §3.35, 901515b6 census-time, f600e32f O(1) v1); S-190 NON alza alcun cap (mod.rs 26072 ≤ 26082, run.rs 7611 = cap, gc_probe.rs nuovo 78)."

cd "$SRC" || exit 4
git diff --quiet -- crates/ || stop "PRE: crates/ sporco — STOP"
HEAD0=$(git rev-parse --short HEAD); PORC=$(git status --porcelain | grep -v '^??' | wc -l | tr -d ' ')
note "PRE: HEAD PRIMA della build = $HEAD0 · status --porcelain (non-untracked) = $PORC righe (rilievo 3 S-180)"
git merge-base --is-ancestor "$CAND_COMMIT" HEAD || stop "PRE: il commit del candidato $CAND_COMMIT non è in HEAD — STOP"
[ -e /private/tmp/phpr-measure.lock ] && note "lock CI: presente (finestra di sessione, non lo tocco)" || note "lock CI: ASSENTE — la sessione lo doveva creare (proseguo, dichiarato)"

disk_guard build
# az.rev. S-182 rilievo 1: il config locale punta alla target di sviluppo (bundle);
# la catena §6 costruisce e hasha SOLO nella canonica, e il binario DEVE risultare
# ricostruito (mtime >= inizio build), altrimenti il gate «build ×2» è una tautologia.
export CARGO_TARGET_DIR="$HOME/Claude/php-rust-output"
T0=$(date +%s)
SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build.log" 2>&1
rc=$?; echo "$rc" > "$OUT/build.rc"
[ "$rc" = 0 ] || stop "build rc=$rc"
[ "$(stat -f %m "$BIN")" -ge "$T0" ] || stop "build: $BIN NON ricostruito (mtime < inizio build) — target sbagliata? STOP"
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

T0=$(date +%s)
SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build2.log" 2>&1
rc=$?; echo "$rc" > "$OUT/build2.rc"
[ "$rc" = 0 ] || stop "build2 rc=$rc"
H2=$(shasum -a 256 "$BIN" | cut -c1-16)
if [ "$(stat -f %m "$BIN")" -ge "$T0" ]; then
  [ "$H2" = "$HB" ] || stop "re-hash post-batteria $H2 != $HB (churn) — STOP"
  note "promozione: churn batteria neutralizzato (build ricetta → $H2 al byte)"
else
  # EMENDA S-184 (REGOLE §5, lettera-gate che morde: il controllo «ricostruito» del rilievo 1 S-182 vale per la build 1,
  # che DEVE toccare la canonica; dopo la batteria senza churn cargo è legittimamente un no-op e il gate vero è l'hash)
  [ "$H2" = "$HB" ] || stop "build2 no-op ma hash $H2 != $HB (churn non sanato) — STOP"
  grep -q 'Compiling' "$OUT/build2.log" && stop "build2: cargo ha compilato ma $BIN non è stato riscritto (target sbagliata?) — STOP"
  note "promozione: batteria SENZA churn — build2 no-op dichiarato (binario intatto, hash $H2 == build 1; emenda S-184 del gate rilievo 1 S-182)"
fi

"$SRC/scripts/pin-phpr.sh" s184 > "$OUT/pin.log" 2>&1
prc=$?; echo "$prc" > "$OUT/pin.rc"
[ "$prc" = 0 ] || stop "pin-phpr.sh rc=$prc"
note "promozione: $(tail -1 "$OUT/pin.log")"

disk_guard corpus
"$SRC/scripts/corpus-gate.sh" "$RUNNER" "$OUT/corpus"
crc=$?; echo "$crc" > "$OUT/corpus-rc"
[ "$crc" = 0 ] || stop "corpus-gate rc=$crc — divergenza (6) non ammette flip: STOP"
note "promozione corpus-gate: rc=0 — nomi==congelato (1407 per modo), CONTENUTO==golden, off-on zero (ZERO flip come atteso: corpus-gate ×2 verde sul tree, catena c2b, e CI OK su cbc3ce5a/a28c376f)"

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
# gate INVARIANTE: pin==stash s183 byte-id (+ marcatore quando preteso)
invar(){ # $1=nome $2=file $3=marcatore("" = nessuno) $4=descrizione
  local n="$1" f="$2" m="$3" d="$4"
  "$REF" "$f" > "$OUT/$n-s183.out" 2>&1
  "$BIN" "$f" > "$OUT/$n-pin.out" 2>&1
  [ -s "$OUT/$n-pin.out" ] || stop "gate $n: output pin VUOTO"
  [ -z "$m" ] || grep -q "$m" "$OUT/$n-pin.out" || stop "gate $n: MARCATORE $m assente"
  if diff -q "$OUT/$n-s183.out" "$OUT/$n-pin.out" > /dev/null; then
    note "promozione gate $n: pin==stash s183 BYTE-ID ($d)"
  else
    diff "$OUT/$n-s183.out" "$OUT/$n-pin.out" > "$OUT/$n.diff" || true
    stop "gate $n: pin DIVERGE dallo stash s183 (promo-out/$n.diff)"
  fi
}
# gate DICHIARATO (S-190): diff(oracle, pin) == expected-diff committato — per un caso che la cura 22/10 ha MOSSO
# (né invariante vs stash né byte-id con l'oracle): la forma residua è dichiarata per nome nel catalogo.
expdiff(){ # $1=nome $2=file $3=expected-diff $4=descrizione
  local n="$1" f="$2" e="$3" d="$4"
  "$ORACLE" "$f" > "$OUT/$n-oracle.out" 2>&1
  "$BIN" "$f" > "$OUT/$n-pin.out" 2>&1
  [ -s "$OUT/$n-pin.out" ] || stop "gate $n: output pin VUOTO"
  diff "$OUT/$n-oracle.out" "$OUT/$n-pin.out" > "$OUT/$n.diff" || true
  if cmp -s "$OUT/$n.diff" "$e"; then note "promozione gate $n: diff(oracle,pin) == expected-diff ($d)"; else stop "gate $n: diff(oracle,pin) ≠ expected-diff dichiarato (promo-out/$n.diff vs $e)"; fi
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
expdiff fxmc2 "$H/fx-mc2.php" "$H11/fx-mc2-wp172.expected-diff" "§3.28: (ii) C4 CURATA dalla cura 22/10 (== oracle), (i) C1 residua nella forma post-22/10 — expected-diff dichiarato S-190"
invar fxmc2fib "$H/fx-mc2-fib.php" "" "§3.29 Fiber non final PRE-esistente INVARIATO"
bilat fxsl1 "$H/fx-sl1.php" "FX-SL1 DONE" "presidio DIRETTO L-SL1: overflow/shift/Div/Mod/Pow/Ref/Double/stringa/null/bool/dst==l/typed-ref/IncDec limiti/CmpJmpSC 8 forme" -d log_errors=0 -d display_errors=1
invar fxsl1div "$H/fx-sl1-div.php" "FX-SL1-DIV DONE" "forme con diag §3.11/§3.13 PRE-esistenti INVARIATE"
bilat fxsl2 "$H/fx-sl2.php" "FX-SL2 DONE" "presidio DIRETTO L-SL2 (forme P1 in loop a 2 iterazioni = IC calda): P1 bigramma fuso e P2 BinarySTDst — overflow/shift/Div/Mod/Pow/Concat/Double/stringa/null/bool/Ref/typed int-float/readonly (anche in scope)/private/hook set/__set/ereditata/dinamica" -d log_errors=0 -d display_errors=1
invar fxsl2div "$H/fx-sl2-div.php" "FX-SL2-DIV DONE" "§3.30 default float da literal int PRE-esistente INVARIATO"
bilat fxsl3 "$H/fx-sl3.php" "FX-SL3 DONE" "presidio DIRETTO fetta 3 (loop a 2 = IC calda): P3 peephole \$s OP= \$o->x e P4 borrow unico recv==slot — overflow/shift/Div/Mod/Pow/Concat/Double/stringa/null/bool/Ref/typed-ref/__get/hook get/enum/dinamica/dst==slot/controlli negativi" -d log_errors=0 -d display_errors=1
invar fxsl3div "$H/fx-sl3-div.php" "FX-SL3-DIV DONE" "§3.32 riga del Deprecated prop dinamica PRE-esistente INVARIATA"
bilat fxsw1 "$H4/fixtures/fx-sw1.php" "FX-SW1 DONE" "presidio DIRETTO Sweep-in-op: ordine statement/distruttori dopo gli op fusi in place, forme fuori dominio (Double/null/concat/overflow/div), P3/P1/P4 con nota a monte, sweep light e finestra main, IN_DESTRUCTOR, pressione GC; back-edge fuso: for Long/Int, miss (Double/str/CmpJmpSS/const-lhs/step2/while), overflow al bordo"
invar fxsw2gc "$H4/fixtures/fx-sw2-gc.php" "FX-SW2 DONE" "§3.33 fixture di CONTEGGI GC: invariante vs stash s183 (come in s186/s190-leva-build), non bilaterale"
bilat fxcr1 "$H8/fx-cr1.php" "FX-CR1 DONE" "presidio DIRETTO L-CR1 (S-181): arità esatta (ip=1), troppo pochi argomenti (ArgumentCountError Zend su funzione/metodo/statico/3-arietà), surplus+func_get_args, default, by-ref, ricorsione, dtor a fine frame, metodi IC/ereditati, closure/dinamica/cuf/cufa, hint, Ref che decade" -d log_errors=0 -d display_errors=1
bilat fxrt1 "$H3/fx-rt1.php" "FX-RT1 DONE" "presidio DIRETTO L-RT1 (S-183): cicli locali array/oggetto + gc_collect_cycles, ricorsione, foreach, dinamiche, metodo" -d log_errors=0 -d display_errors=1
bilat fxrt2 "$H9/fx-rt2.php" "FX-RT2 DONE" "presidio DIRETTO L-RT2 (S-184): ammissione del Ret fuso — hint int/nullable/TypeError, by-ref alias e deref, forma 0 annidata/null/locali, ret_cell thunk statico, INIT_PROPS, __toString, dtor con chiamata" -d log_errors=0 -d display_errors=1
bilat fxmc2s185 "$H10/fx-mc2.php" "FX-MC2 DONE" "presidio DIRETTO L-MR1+L-MC2 (S-185): Ret fuso dei metodi (nota GC del ricevitore: dtor-late, ordine slots poi ricevitore, cascata del ricevitore risorto), MethodCall in pila (by-ref, default, ArgPlace sul cammino storico, dtor di temporanei in istruzione propria)" -d log_errors=0 -d display_errors=1

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

PHPR="$BIN" R=5 "$SRC/wp97-harness/micro/run-micro.sh" > "$OUT/micro-pin-s184.out" 2>&1
note "promozione micro pin s184: $(grep -E '^rapporto_' "$OUT/micro-pin-s184.out" | tr '\n' ' ')"

# ---- conferma POST-PIN: R=5 pin s184 vs stash s183 (STESSA toolchain 1.98.1) su ret-buf-dq-k0 (bersaglio della guardia b3) e mcalls-dq/calls-dq/prop-dq (guardie); D = A(stash) − B(pin): negativo = pin più lento ----
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
note "conferma post-pin ret-buf-dq-k0 (pin s184 vs stash s183, stessa toolchain; ns/iter = 65 Ret): $(conferma "$H12/ret-buf-dq-k0.php" k0) (attesa: D ≈ −200 ns/iter = 65 × −3,08 della guardia b3; tetto |D| < 260 = 65 × 4)"
note "conferma post-pin mcalls-dq (guardia, pin s184 vs stash s183): $(conferma "$H10/mcalls-dq.php" mcalls) (attesa: D ≈ −2,33 della guardia b3, |D| < 4)"
note "conferma post-pin calls-dq (guardia, pin s184 vs stash s183): $(conferma "$H9/calls-dq.php" calls) (attesa |D| < 1: guardia b3 +0,33)"
note "conferma post-pin prop-dq (guardia, pin s184 vs stash s183): $(conferma "$H/prop-dq.php" prop) (attesa |D| < 1: sola lettura)"

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

"$SRC/scripts/pin-server.sh" s190 > "$OUT/pin-server.log" 2>&1
src_rc=$?; echo "$src_rc" > "$OUT/pin-server.rc"
[ "$src_rc" = 0 ] || stop "pin-server.sh rc=$src_rc"
note "promozione server: $(grep '^PIN server' "$OUT/pin-server.log" | tail -1)"

echo 0 > "$OUT/rcb"
note "PROMOZIONE COMPLETA rc=0 (da promo-out/rcb): pin s184 = $H2"
exit 0
