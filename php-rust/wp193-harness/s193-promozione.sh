#!/bin/bash
# s193-promozione.sh — gate di PROMOZIONE del tree «pin s184 + SEI leve di S-192» → pin phpr s185 (NEXT §S-193 ordine 2;
# ordine deciso dall'utente il 19/11: promozione di giorno, poi la notte la conferma o la smentisce; s184 resta nello stash).
# COPIA DICHIARATA di ../wp190-harness/s190-promozione.sh (manifest s193-promozione-copia.diff, generatore
# gen-promozione-s193.py) coi SOLI adattamenti: (1) tag s185 (harness wp193: verdetto s193-promo-verdetto.out, promo-out);
# (2) CANDIDATO = braccio B dei gemelli s192gc (target s192-armB sulla bundle, df39c0a92937db0f, commit debbca2f = ultimo
# commit di codice: da lì a HEAD solo documenti); precondizione = ultima catena di accettazione S-192 rc=0
# (wp190-harness/s190-c2-catena-s192gc.rc) e commit in HEAD; (3) REF = stash phpr-s184 (60b10d857eb0343a) per i gate
# invarianti; (4) gate di fixture INVARIATI nella forma (dry-run 20/11: 25/25 verdi col candidato), sole etichette s183 → s184;
# (5) corpus: ZERO flip attesi (corpus-gate ×2 verde a ogni leva di S-192); (6) conferma post-pin = ret-buf-dq-k0 R=5 pin s185
# vs stash s184 col SEGNO di S-192 (gemelli g3: K0 −5,08 ns/Ret a favore della leva ⇒ attesa D = A−B ≈ +330 ns/iter) +
# mcalls-dq/calls-dq/prop-dq (guardie, attese piatte); (7) pin-server.sh s191; (8) dichiarazione LOC: S-192 non alza cap.
# Testo S-190/S-185/S-184/S-181/S-180/S-177 conservato sotto:
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp172-harness"
H4="$SRC/wp174-harness"
H8="$SRC/wp181-harness"
H9="$SRC/wp184-harness"; H3="$SRC/wp182-harness"
H10="$SRC/wp185-harness"
H11="$SRC/wp190-harness"; H12="$SRC/wp186-harness"; H13="$SRC/wp193-harness"
BIN="$HOME/Claude/php-rust-output/release/phpr"
RUNNER="$HOME/Claude/php-rust-output/release/phpt-runner"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
WD="/Volumes/Extreme Pro/Claude/wp13-harness/run-with-watchdog.sh"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
QUIESCE="$SRC/wp129-harness/s129-quiescenza.sh"
SP="${PROMO_SP:?PROMO_SP (workdir APFS per i gate ORM/hk) richiesto}"
OUT="$H13/promo-out"; mkdir -p "$OUT"
VERD="$H13/s193-promo-verdetto.out"
disk_guard(){ local g; g=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}'); [ "${g:-0}" -ge 3 ] || stop "guardia disco: Data ${g}G < 3G prima di $1 — STOP"; note "guardia disco: Data ${g}G prima di $1"; }
REF="$STASH/phpr-s184"; REF_EXP="60b10d857eb0343a"
CAND="$HOME/Claude/phpr-target/s192-armB/release/phpr"; CAND_COMMIT="${CAND_COMMIT:-debbca2f}"
note(){ echo "$1"; echo "$1" >> "$VERD"; }
stop(){ note "$1"; echo 1 > "$OUT/rcb"; exit 1; }

# verifica POSITIVA dei path d'ingresso (emenda §3 proposta S-160, lezione #21)
for f in "$H/fx-ce.php" "$H/fx-am.php" "$H/fx-af.php" "$H/fx-refl.php" \
         "$H/fx-sm.php" "$H/fx-sm-div.php" "$H/fx-au.php" "$H/fx-au-div.php" \
         "$H/fx-mc.php" "$H/fx-mc2.php" "$H/fx-mc2-fib.php" "$H/fx-mck.php" \
         "$H/fx-sl1.php" "$H/fx-sl1-div.php" "$H/fx-sl2.php" "$H/fx-sl2-div.php" "$H/fx-sl3.php" "$H/fx-sl3-div.php" "$H/prop-dq.php" "$H4/fixtures/fx-sw1.php" \
         "$H8/fx-cr1.php" "$H9/calls-dq.php" "$H3/fx-rt1.php" "$H9/fx-rt2.php" "$H10/fx-mc2.php" "$H10/mcalls-dq.php" "$H12/ret-buf-dq-k0.php" "$H4/fixtures/fx-sw2-gc.php" "$H11/fx-mc2-wp172.expected-diff" "$CAND" "$H11/s190-c2-catena-s192gc.rc" \
         "$H/sonda-bt-autoload.php" "$SRC/wp164-harness/arith-dq.php" \
         "$H/empty.php" "$QUIESCE" "$WD" "$GATES/orm-work.tgz" "$GATES/hk-work.tgz" \
         "$SRC/wp125-harness/orm-baseline-failnames.txt" \
         "$SRC/wp125-harness/promo-out/batteria-nomi.txt" \
         "$REF"; do
  [ -s "$f" ] || stop "PRE: path d'ingresso MANCANTE: $f — STOP"
done
[ "$(shasum -a 256 "$REF" | cut -c1-16)" = "$REF_EXP" ] || stop "PRE: stash phpr-s184 hash != $REF_EXP — STOP"
[ "$(cat "$H11/s190-c2-catena-s192gc.rc")" = 0 ] || stop "PRE: catena di accettazione s192gc rc=$(cat "$H11/s190-c2-catena-s192gc.rc") ≠ 0 — nessuna promozione senza accettazione verde — STOP"
CH=$(shasum -a 256 "$CAND" | cut -c1-16)
note "PRE: candidato = braccio B $CH (commit $CAND_COMMIT, gemelli ORM s192gc; accettazione S-192 rc=0 su ogni leva: batteria 1773/0/2, corpus-gate ×2, sentinelle); REF = stash s184 $REF_EXP"
note "PRE: DICHIARAZIONE dei cambiamenti del pin s185 rispetto a s184 (per nome, criteri wp192-harness/s192-criterio-*.md): (a) K0 gc_note_this_tail (0ac0b35b); (b) hasher Fx in lower/compile (8b253f18); (c) cache realpath validata (15019848); (d) array_diff a insieme hash (0eca36da, 82244d79); (e) memo __destruct/__construct (1b11e848); (f) registro oggetti vivi IdMap (debbca2f); inoltre contatori census-time (bb0e84dd, cec724cf: gated OFF nel pin). Cifre S-192 = DIREZIONE su una gamba ORM; WordPress mai rimisurato: se t27/t29 differiscono da t26/t28 l'attribuzione va istruita per leva, mai d'ufficio."
note "PRE: LOC (REGOLE §8): S-192 NON alza alcun cap (il dente loc_dente non è toccato dalla promozione s184, a485e626; resta nella batteria di questa catena)."

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

"$SRC/scripts/pin-phpr.sh" s185 > "$OUT/pin.log" 2>&1
prc=$?; echo "$prc" > "$OUT/pin.rc"
[ "$prc" = 0 ] || stop "pin-phpr.sh rc=$prc"
note "promozione: $(tail -1 "$OUT/pin.log")"

disk_guard corpus
"$SRC/scripts/corpus-gate.sh" "$RUNNER" "$OUT/corpus"
crc=$?; echo "$crc" > "$OUT/corpus-rc"
[ "$crc" = 0 ] || stop "corpus-gate rc=$crc — divergenza (6) non ammette flip: STOP"
note "promozione corpus-gate: rc=0 — nomi==congelato (1407 per modo), CONTENUTO==golden, off-on zero (ZERO flip come atteso: corpus-gate ×2 verde a ogni leva di S-192, catene s192*)"

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
# gate INVARIANTE: pin==stash s184 byte-id (+ marcatore quando preteso)
invar(){ # $1=nome $2=file $3=marcatore("" = nessuno) $4=descrizione
  local n="$1" f="$2" m="$3" d="$4"
  "$REF" "$f" > "$OUT/$n-s184.out" 2>&1
  "$BIN" "$f" > "$OUT/$n-pin.out" 2>&1
  [ -s "$OUT/$n-pin.out" ] || stop "gate $n: output pin VUOTO"
  [ -z "$m" ] || grep -q "$m" "$OUT/$n-pin.out" || stop "gate $n: MARCATORE $m assente"
  if diff -q "$OUT/$n-s184.out" "$OUT/$n-pin.out" > /dev/null; then
    note "promozione gate $n: pin==stash s184 BYTE-ID ($d)"
  else
    diff "$OUT/$n-s184.out" "$OUT/$n-pin.out" > "$OUT/$n.diff" || true
    stop "gate $n: pin DIVERGE dallo stash s184 (promo-out/$n.diff)"
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
invar fxsw2gc "$H4/fixtures/fx-sw2-gc.php" "FX-SW2 DONE" "§3.33 fixture di CONTEGGI GC: invariante vs stash s184 (dry-run 20/11 verde anche con IdMap e gc_note_this_tail), non bilaterale"
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

PHPR="$BIN" R=5 "$SRC/wp97-harness/micro/run-micro.sh" > "$OUT/micro-pin-s185.out" 2>&1
note "promozione micro pin s185: $(grep -E '^rapporto_' "$OUT/micro-pin-s185.out" | tr '\n' ' ')"

# ---- conferma POST-PIN: R=5 pin s185 vs stash s184 (STESSA toolchain 1.98.1) su ret-buf-dq-k0 (bersaglio della leva K0) e mcalls-dq/calls-dq/prop-dq (guardie); D = A(stash) − B(pin): POSITIVO = pin più veloce ----
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
note "conferma post-pin ret-buf-dq-k0 (pin s185 vs stash s184, stessa toolchain; ns/iter = 65 Ret): $(conferma "$H12/ret-buf-dq-k0.php" k0) (attesa col segno di S-192: D ≈ +330 ns/iter = 65 × 5,08 dei gemelli g3; D ≤ 0 = leva K0 non confermata sul canonico, da dichiarare)"
note "conferma post-pin mcalls-dq (guardia, pin s185 vs stash s184): $(conferma "$H10/mcalls-dq.php" mcalls) (attesa piatta: gemelli g3 +0,33; |D| < 4)"
note "conferma post-pin calls-dq (guardia, pin s185 vs stash s184): $(conferma "$H9/calls-dq.php" calls) (attesa piatta: gemelli g3 +0,50; |D| < 4)"
note "conferma post-pin prop-dq (guardia, pin s185 vs stash s184): $(conferma "$H/prop-dq.php" prop) (attesa |D| < 1: sola lettura)"

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

"$SRC/scripts/pin-server.sh" s191 > "$OUT/pin-server.log" 2>&1
src_rc=$?; echo "$src_rc" > "$OUT/pin-server.rc"
[ "$src_rc" = 0 ] || stop "pin-server.sh rc=$src_rc"
note "promozione server: $(grep '^PIN server' "$OUT/pin-server.log" | tail -1)"

echo 0 > "$OUT/rcb"
note "PROMOZIONE COMPLETA rc=0 (da promo-out/rcb): pin s185 = $H2"
exit 0
