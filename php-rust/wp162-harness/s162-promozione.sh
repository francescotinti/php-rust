#!/bin/bash
# s162-promozione.sh <cand_hash16> — gate di promozione LEVA L-AM2
# (array_map string-callable UTENTE k=1 senza args-Vec) — S-162. COPIA
# DICHIARATA di wp161-harness/s161-promozione.sh; DIVERGENZE DICHIARATE:
#  (1) tag s162, harness wp162 (H PROPRIO, OUT NUOVA promo-out; verifica
#      POSITIVA dei path in testa, pena STOP); candidato 20c63af44bfd077a
#      (A/B R=5 in s162-am2r5-verdetto.out: giudice strmap VINTO D=+65,0
#      rumore 1,0/1,0, riconc. smoke |2,5|<4,0 in banda; ARBITRATO census in
#      mezzo rc=0-con-appendice (D_smoke +67,5 FUORI banda [4;19] SOPRA):
#      Delta=2 alloc/elemento ESATTE (args-Vec + to_vec del nome) sul solo
#      array_map ⇒ FUORI-UB SOPRA a verbale, coeff PROPRIO sito strmap;
#      18 guardie ok NESSUN morso; braccio A = GEMELLO ec0a636a == pin s161
#      AL BYTE (s162-gemelloA-identita.out: 2 tentativi target dedicato
#      scartati, path-sensibilita' dichiarata); candidato stashato
#      phpr-s162-am2-B);
#  (2) inventario batteria = baseline s125 + i DUE denti dichiarati (come
#      s161): rczval + loc_dente A4 — L-AM2 non aggiunge test; salite loc
#      monolite VM 25831→25847 e host 7708→7726 PRE-dichiarate e committate
#      (edadf29f);
#  (3) corpus: ZERO flip attesi (L-AM2 preserva la semantica per costruzione:
#      fast path solo string-callable a funzione UTENTE simple arita' 1,
#      resto invariato) — rc!=0 = STOP secco, NESSUN flip-handler;
#  (4) fixture chain INVARIATA (10 gate) + fx-ce (bilaterale oracle==pin) +
#      sonda-bt (pin==stash s161: contratto §3.25 invariato, presidio L-AL2
#      ereditato) + fx-refl (pin==stash s161) + fx-am v2 + fx-af (bilaterali
#      oracle==pin) + fx-sm NUOVO (bilaterale oracle==pin, forme
#      string-callable: presidio DIRETTO della leva) + fx-sm-div NUOVO
#      (pin==stash s161 INVARIANTE: 2 divergenze PRE-esistenti a catalogo) —
#      fixture ereditate in BYTE-COPIA wp162 verificata cmp;
#  (5) conferma POST-PIN: m-strmap (R=5 pin s162 vs stash phpr-s161
#      ec0a636ad0c42005, direzione attesa +, D nell'intorno di +65,0 ±
#      rumore+drift; rev. S-161 #5: rumore > attesa/2 ⇒ SOLO SEGNO);
#  (6) disasm run_loop GIA' AGLI ATTI pre-promo (bl A=6033 B=6033, Δ=0 —
#      ab-out/disasm-am2.out);
#  (7) micro R=5 = run-micro.sh come s161 (scoreboard al pin nuovo).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp162-harness"
BIN="$HOME/Claude/php-rust-output/release/phpr"
RUNNER="$HOME/Claude/php-rust-output/release/phpt-runner"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
WD="/Volumes/Extreme Pro/Claude/wp13-harness/run-with-watchdog.sh"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
QUIESCE="$SRC/wp129-harness/s129-quiescenza.sh"
SP="${PROMO_SP:?PROMO_SP (workdir APFS per i gate ORM/hk) richiesto}"
OUT="$H/promo-out"; mkdir -p "$OUT"
VERD="$H/s162-promo-verdetto.out"
CAND_EXP="${1:?uso: s162-promozione.sh <cand_hash16>}"
note(){ echo "$1"; echo "$1" >> "$VERD"; }
stop(){ note "$1"; echo 1 > "$OUT/rcb"; exit 1; }

# verifica POSITIVA dei path d'ingresso (emenda §3 proposta S-160, lezione #21)
for f in "$H/fx-ce.php" "$H/fx-am.php" "$H/fx-af.php" "$H/fx-refl.php" \
         "$H/fx-sm.php" "$H/fx-sm-div.php" \
         "$H/sonda-bt-autoload.php" "$H/m-arrfilter.php" "$H/m-strmap.php" \
         "$H/empty.php" "$QUIESCE" "$WD" "$GATES/orm-work.tgz" "$GATES/hk-work.tgz" \
         "$SRC/wp125-harness/orm-baseline-failnames.txt" \
         "$SRC/wp125-harness/promo-out/batteria-nomi.txt" \
         "$STASH/phpr-s161" "$STASH/phpr-s162-am2-B"; do
  [ -s "$f" ] || stop "PRE: path d'ingresso MANCANTE: $f — STOP"
done

cd "$SRC" || exit 4
git diff --quiet -- crates/ || stop "PRE: crates/ sporco — STOP"
[ -e /private/tmp/phpr-measure.lock ] && note "lock CI: presente (finestra di sessione, non lo tocco)" || note "lock CI: ASSENTE — la sessione lo doveva creare (proseguo, dichiarato)"

SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build.log" 2>&1
rc=$?; echo "$rc" > "$OUT/build.rc"
[ "$rc" = 0 ] || stop "build rc=$rc"
HB=$(shasum -a 256 "$BIN" | cut -c1-16)
ident_contenuto(){ # $1=fileA $2=fileB — 0 se cluster tutti nominati, cap 160 B
  local le; le=$(otool -l "$1" | awk '/cmd LC_SEGMENT_64/{seg=""} /segname __LINKEDIT/{seg=1} seg && /fileoff/{print $2; exit}')
  python3 - "$1" "$2" "${le:-0}" <<'PY'
import sys
a=open(sys.argv[1],"rb").read(); b=open(sys.argv[2],"rb").read(); le=int(sys.argv[3])
if len(a)!=len(b): print("dimensioni diverse"); sys.exit(1)
d=[i for i,(x,y) in enumerate(zip(a,b)) if x!=y]
if len(d)>160: print(f"{len(d)} byte > cap 160"); sys.exit(1)
cl=[]
for i in d:
    if cl and i-cl[-1][1]<=64: cl[-1][1]=i
    else: cl.append([i,i])
bad=[]
for s,e in cl:
    ctx=a[max(0,s-64):e+64]+b[max(0,s-64):e+64]
    if s<4096 and e-s+1<=16: k="LC_UUID"
    elif le and s>=le: k="firma"
    elif b"Jan  1 1970" in ctx or b"00:00:00" in ctx or b"built on" in ctx: k="banner"
    else: k="NON-CLASSIFICATO"; bad.append((s,e))
    print(f"  cluster 0x{s:x}-0x{e:x}: {k}")
print(f"  tot {len(d)} B in {len(cl)} cluster")
sys.exit(1 if bad else 0)
PY
}
if [ "$HB" = "$CAND_EXP" ]; then
  note "promozione: build ricetta riproduce il candidato $HB AL BYTE"
else
  CSTASH="$STASH/phpr-s162-am2-B"
  [ "$(shasum -a 256 "$CSTASH" | cut -c1-16)" = "$CAND_EXP" ] || stop "stash candidato != $CAND_EXP — STOP"
  ident_contenuto "$BIN" "$CSTASH" > "$OUT/ident-contenuto.txt" 2>&1     || { cat "$OUT/ident-contenuto.txt" >> "$VERD"; stop "build $HB DIVERGE dal candidato OLTRE il meccanismo nominato — STOP"; }
  cat "$OUT/ident-contenuto.txt" >> "$VERD"
  note "promozione: identità candidato a CONTENUTO (cluster LC_UUID/firma/banner) — pin effettivo = $HB (dichiarato)"
fi

CARGO_INCREMENTAL=0 cargo test --release > "$OUT/batteria.log" 2>&1
brc=$?; echo "$brc" > "$OUT/batteria.rc"
cnt=$(awk '/^test result:/{p+=$4; f+=$6; ig+=$8} END{printf "%d/%d/%d", p, f, ig}' "$OUT/batteria.log")
grep -E '^test .* \.\.\. ' "$OUT/batteria.log" | sed 's/^test //; s/ \.\.\..*//' | sort > "$OUT/batteria-nomi.txt"
normline(){ sed 's/(line [0-9][0-9]*)/(line N)/' "$1" | sort; }
normline "$SRC/wp125-harness/promo-out/batteria-nomi.txt" > "$OUT/base-norm.txt"
normline "$OUT/batteria-nomi.txt" > "$OUT/nomi-norm.txt"
NEW_ONLY=$(comm -13 "$OUT/base-norm.txt" "$OUT/nomi-norm.txt" | sort | tr '\n' ' ' | sed 's/ $//')
GONE=$(comm -23 "$OUT/base-norm.txt" "$OUT/nomi-norm.txt")
if [ "$NEW_ONLY" = "nessun_sorgente_rs_oltre_cap rczval_pattern_resta_nel_funnel" ] && [ -z "$GONE" ]; then
  INV="baseline s125 + i DUE denti dichiarati (rczval + loc_dente A4)"
else
  INV="DIVERGE (nuovi='$NEW_ONLY' spariti='$GONE' — diff in promo-out/batteria-inv.diff)"
  diff "$SRC/wp125-harness/promo-out/batteria-nomi.txt" "$OUT/batteria-nomi.txt" > "$OUT/batteria-inv.diff" || true
fi
note "promozione batteria: rc=$brc (da promo-out/batteria.rc) · $cnt · inventario: $INV"
[ "$brc" = 0 ] || stop "batteria rc=$brc"
case "$INV" in DIVERGE*) stop "inventario batteria DIVERGE";; esac
grep -E '^test .*debug_backtrace_array_fields' "$OUT/batteria.log" | grep -q ' ok$' \
  && note "promozione batteria: debug_backtrace_array_fields VERDE" \
  || stop "batteria: debug_backtrace_array_fields NON verde/assente"

SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build2.log" 2>&1
rc=$?; echo "$rc" > "$OUT/build2.rc"
[ "$rc" = 0 ] || stop "build2 rc=$rc"
H2=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$H2" = "$HB" ] || stop "re-hash post-batteria $H2 != $HB (churn) — STOP"
note "promozione: churn batteria neutralizzato (build ricetta → $H2 al byte)"

"$SRC/scripts/pin-phpr.sh" s162 > "$OUT/pin.log" 2>&1
prc=$?; echo "$prc" > "$OUT/pin.rc"
[ "$prc" = 0 ] || stop "pin-phpr.sh rc=$prc"
note "promozione: $(tail -1 "$OUT/pin.log")"

"$SRC/scripts/corpus-gate.sh" "$RUNNER" "$OUT/corpus"
crc=$?; echo "$crc" > "$OUT/corpus-rc"
[ "$crc" = 0 ] || stop "corpus-gate rc=$crc — divergenza (3) non ammette flip: STOP"
note "promozione corpus-gate: rc=0 — nomi==congelato (1412), CONTENUTO==golden, off-on zero (ZERO flip come atteso, L-AM2)"

PHPR_PIN_ATTESO="$H2" "$SRC/wp109-harness/s109-fixture-chain.sh" > "$OUT/fixture-chain.out" 2>&1
frc=$?; echo "$frc" > "$OUT/fixture-chain.rc"
[ "$frc" = 0 ] || stop "fixture chain rc=$frc"
FX_ATTESI="hc1 move recv fx20 fx21 w9 preg teardown stash backtrace"
FX_VISTI=$(sed -n 's/^FIXTURE-CHAIN inventario=//p' "$OUT/fixture-chain.out")
[ "$FX_VISTI" = "$FX_ATTESI" ] || stop "fixture chain inventario diverso: visti='$FX_VISTI' attesi='$FX_ATTESI'"
note "promozione fixture chain: rc=0 (10 gate: $FX_VISTI)"

ORACLE=/opt/homebrew/opt/php/bin/php
# ---- gate fx-ce: bilaterale oracle==pin byte-id (MARCATORE preteso) ----
"$ORACLE" "$H/fx-ce.php" > "$OUT/fxce-oracle.out" 2>&1
"$BIN" "$H/fx-ce.php" > "$OUT/fxce-pin.out" 2>&1
[ -s "$OUT/fxce-pin.out" ] || stop "gate fx-ce: output pin VUOTO (file/probe rotto)"
if diff -q "$OUT/fxce-oracle.out" "$OUT/fxce-pin.out" > /dev/null; then
  note "promozione gate fx-ce: oracle==pin BYTE-ID (12 forme class/interface/trait_exists)"
else
  diff "$OUT/fxce-oracle.out" "$OUT/fxce-pin.out" > "$OUT/fxce.diff" || true
  stop "gate fx-ce: DIVERGE (promo-out/fxce.diff)"
fi
# ---- gate sonda-bt: pin==stash s161 byte-id (§3.25 — presidio L-AL2 ereditato) ----
"$STASH/phpr-s161" "$H/sonda-bt-autoload.php" > "$OUT/sonda-bt-s161.out" 2>&1
"$BIN" "$H/sonda-bt-autoload.php" > "$OUT/sonda-bt-pin.out" 2>&1
[ -s "$OUT/sonda-bt-pin.out" ] || stop "gate sonda-bt: output pin VUOTO"
if diff -q "$OUT/sonda-bt-s161.out" "$OUT/sonda-bt-pin.out" > /dev/null; then
  note "promozione gate sonda-bt: pin==stash s161 BYTE-ID (contratto autoload/backtrace §3.25 invariato — presidio L-AL2 ereditato)"
else
  stop "gate sonda-bt: pin DIVERGE dallo stash s161 (promo-out/sonda-bt-pin.out)"
fi
# ---- gate fx-refl: pin==stash s161 byte-id ----
"$STASH/phpr-s161" "$H/fx-refl.php" > "$OUT/fxrefl-s161.out" 2>&1
"$BIN" "$H/fx-refl.php" > "$OUT/fxrefl-pin.out" 2>&1
[ -s "$OUT/fxrefl-pin.out" ] || stop "gate fx-refl: output pin VUOTO"
if diff -q "$OUT/fxrefl-s161.out" "$OUT/fxrefl-pin.out" > /dev/null; then
  note "promozione gate fx-refl: pin==stash s161 BYTE-ID (contratto __reflect_* L-RF2 invariato)"
else
  stop "gate fx-refl: pin DIVERGE dallo stash s161 (promo-out/fxrefl-pin.out)"
fi
# ---- gate fx-am: bilaterale oracle==pin byte-id + MARCATORE ----
"$ORACLE" "$H/fx-am.php" > "$OUT/fxam-oracle.out" 2>&1
"$BIN" "$H/fx-am.php" > "$OUT/fxam-pin.out" 2>&1
grep -q "FXAM-END" "$OUT/fxam-pin.out" || stop "gate fx-am: MARCATORE FXAM-END assente (lezione #21)"
if diff -q "$OUT/fxam-oracle.out" "$OUT/fxam-pin.out" > /dev/null; then
  note "promozione gate fx-am: oracle==pin BYTE-ID (20 forme array_map v2, fast E pieno)"
else
  diff "$OUT/fxam-oracle.out" "$OUT/fxam-pin.out" > "$OUT/fxam.diff" || true
  stop "gate fx-am: DIVERGE (promo-out/fxam.diff)"
fi
# ---- gate fx-af: bilaterale oracle==pin byte-id + MARCATORE ----
"$ORACLE" "$H/fx-af.php" > "$OUT/fxaf-oracle.out" 2>&1
"$BIN" "$H/fx-af.php" > "$OUT/fxaf-pin.out" 2>&1
grep -q "FXAF-END" "$OUT/fxaf-pin.out" || stop "gate fx-af: MARCATORE FXAF-END assente (lezione #21)"
if diff -q "$OUT/fxaf-oracle.out" "$OUT/fxaf-pin.out" > /dev/null; then
  note "promozione gate fx-af: oracle==pin BYTE-ID (13 forme array_filter, fast E pieno)"
else
  diff "$OUT/fxaf-oracle.out" "$OUT/fxaf-pin.out" > "$OUT/fxaf.diff" || true
  stop "gate fx-af: DIVERGE (promo-out/fxaf.diff)"
fi
# ---- gate fx-sm: bilaterale oracle==pin byte-id + MARCATORE (presidio DIRETTO L-AM2) ----
"$ORACLE" "$H/fx-sm.php" > "$OUT/fxsm-oracle.out" 2>&1
"$BIN" "$H/fx-sm.php" > "$OUT/fxsm-pin.out" 2>&1
grep -q "FX-SM DONE" "$OUT/fxsm-pin.out" || stop "gate fx-sm: MARCATORE FX-SM DONE assente (lezione #21)"
if diff -q "$OUT/fxsm-oracle.out" "$OUT/fxsm-pin.out" > /dev/null; then
  note "promozione gate fx-sm: oracle==pin BYTE-ID (forme string-callable, fast E pieno — presidio DIRETTO L-AM2)"
else
  diff "$OUT/fxsm-oracle.out" "$OUT/fxsm-pin.out" > "$OUT/fxsm.diff" || true
  stop "gate fx-sm: DIVERGE (promo-out/fxsm.diff)"
fi
# ---- gate fx-sm-div: pin==stash s161 INVARIANTE + MARCATORE (divergenze pre-esistenti) ----
"$STASH/phpr-s161" "$H/fx-sm-div.php" > "$OUT/fxsmdiv-s161.out" 2>&1
"$BIN" "$H/fx-sm-div.php" > "$OUT/fxsmdiv-pin.out" 2>&1
grep -q "FX-SM-DIV DONE" "$OUT/fxsmdiv-pin.out" || stop "gate fx-sm-div: MARCATORE FX-SM-DIV DONE assente"
if diff -q "$OUT/fxsmdiv-s161.out" "$OUT/fxsmdiv-pin.out" > /dev/null; then
  note "promozione gate fx-sm-div: pin==stash s161 BYTE-ID (2 divergenze PRE-esistenti INVARIATE, a catalogo)"
else
  stop "gate fx-sm-div: pin DIVERGE dallo stash s161 (promo-out/fxsmdiv-pin.out)"
fi

QOK=1
for t in $(seq 1 30); do
  if "$QUIESCE" "$OUT/quiesce.rc" > "$OUT/quiesce-t$t.log" 2>&1; then QOK=0; note "promozione quiescenza: PASS al tentativo $t"; break; fi
  sleep 60
done
[ "$QOK" = 0 ] || stop "quiescenza MAI PASS in 30 tentativi — STOP prima delle misure"

PHPR="$BIN" R=5 "$SRC/wp97-harness/micro/run-micro.sh" > "$OUT/micro-pin-s162.out" 2>&1
note "promozione micro pin s162: $(grep -E '^rapporto_' "$OUT/micro-pin-s162.out" | tr '\n' ' ')"

# ---- conferma POST-PIN m-strmap (divergenza (5)): R=5 pin vs stash s161 ----
AOLD="$STASH/phpr-s161"
[ "$(shasum -a 256 "$AOLD" | cut -c1-16)" = "ec0a636ad0c42005" ] || stop "stash phpr-s161 hash inatteso"
ucpu(){ { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$1" "$2" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3(){ local a b c; a=$(ucpu "$1" "$2"); b=$(ucpu "$1" "$2"); c=$(ucpu "$1" "$2"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }
FA=$(floor3 "$AOLD" "$H/empty.php"); FB=$(floor3 "$BIN" "$H/empty.php")
CT="$OUT/conferma-runs.tsv"; : > "$CT"
for i in 1 2 3 4 5; do
  if [ $((i % 2)) -eq 1 ]; then TA=$(ucpu "$AOLD" "$H/m-strmap.php"); TB=$(ucpu "$BIN" "$H/m-strmap.php"); else TB=$(ucpu "$BIN" "$H/m-strmap.php"); TA=$(ucpu "$AOLD" "$H/m-strmap.php"); fi
  printf '%s\t%s\t%s\t%s\n' "$TA" "$TB" "$FA" "$FB" >> "$CT"
done
CONF=$(python3 - "$CT" <<'PY'
import sys
rows=[l.split("\t") for l in open(sys.argv[1]).read().strip().split("\n")]
n=10000000.0
na=[(float(t[0])-float(t[2]))/n*1e9 for t in rows]
nb=[(float(t[1])-float(t[3]))/n*1e9 for t in rows]
def med(v): s=sorted(v); k=len(s); return s[k//2] if k%2 else (s[k//2-1]+s[k//2])/2
def trange(v):
    m=med(v); w=sorted(v,key=lambda x:(abs(x-m),x))[:-1]; return max(w)-min(w)
d=med(na)-med(nb); noise=max(trange(na),trange(nb))
segni=sum(1 for a,b in zip(na,nb) if a>b)
print(f"D={d:+.1f} rumore={noise:.1f} segni={segni}/5 A={med(na):.1f} B={med(nb):.1f}")
PY
)
note "conferma post-pin m-strmap (pin s162 vs stash s161, drift-tree da dichiarare): $CONF (attesa: segno +, D nell'intorno di +65,0 ± rumore+drift; rev. S-161 #5: rumore > attesa/2 = 32,5 ⇒ SOLO SEGNO)"

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

"$SRC/scripts/pin-server.sh" s162 > "$OUT/pin-server.log" 2>&1
src_rc=$?; echo "$src_rc" > "$OUT/pin-server.rc"
[ "$src_rc" = 0 ] || stop "pin-server.sh rc=$src_rc"
note "promozione server: $(grep '^PIN server' "$OUT/pin-server.log" | tail -1)"

echo 0 > "$OUT/rcb"
note "PROMOZIONE COMPLETA rc=0 (da promo-out/rcb): pin s162 = $H2"
exit 0
