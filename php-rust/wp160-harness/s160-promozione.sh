#!/bin/bash
# s160-promozione.sh <cand_hash16> — gate di promozione LEVA L-AF1
# (array_filter plumbing 0-alloc per-elemento) — S-160. COPIA DICHIARATA
# di wp159-harness/s159-promozione.sh; DIVERGENZE DICHIARATE:
#  (1) tag s160, harness wp160; candidato ceeb6e76e4ef5ace (A/B R=5 in
#      s160-af1-verdetto.out: giudice arrfilter VINTO D=+16,0 rumore 1,0/1,0,
#      riconc. smoke in banda 2,0<4,0, riconc. UB FUORI-SOPRA DICHIARATO
#      (UB 12,0 tarato ±2,5: eccedenza ~0,5-3,0 su componenti non prezzate
#      nominate ⇒ sonda surplus DOVUTA entro S-161, orologio §4); BANDA SMOKE
#      VINCOLANTE [8;22] rispettata (+14,0); 17 guardie ok, NESSUN morso
#      (arrmap +2,0: L-AM1 presidiata); braccio A = GEMELLO f2d17f18 == pin
#      s159 AL BYTE a freddo N=2 (s160-gemelloA-identita.out); candidato
#      stashato phpr-s160-af1-B);
#  (2) inventario batteria = baseline s125 + i DUE denti dichiarati (come
#      s159): rczval + loc_dente A4 — L-AF1 non aggiunge test; salita loc
#      host.rs 7683→7708 PRE-dichiarata e committata (dcab1059), mod.rs
#      25810 INVARIATO;
#  (3) corpus: ZERO flip attesi (L-AF1 preserva la semantica per costruzione:
#      fast path solo closure anonima simple arita' 1 + mode==0, resto
#      invariato) — rc!=0 = STOP secco, NESSUN flip-handler;
#  (4) fixture chain INVARIATA (10 gate) + gate fx-ce (bilaterale oracle==pin
#      byte-id; byte-copia in wp160) + gate sonda-bt (pin==stash s159 byte-id:
#      contratto autoload §3.25 invariato) + gate fx-refl (pin==stash s159
#      byte-id: contratto __reflect_* L-RF2 invariato; byte-copia in wp160)
#      + gate fx-am v2 (bilaterale oracle==pin byte-id, 20 forme array_map —
#      presidio L-AM1) + gate fx-af NUOVO (bilaterale oracle==pin byte-id,
#      13 forme array_filter — fast E pieno; gia' byte-id pre-promo);
#  (5) conferma POST-PIN: m-arrfilter (R=5 pin s160 vs stash phpr-s159
#      f2d17f18c00a4049, direzione attesa +, D nell'intorno di +16,0
#      ± rumore+drift);
#  (6) disasm run_loop GIA' AGLI ATTI pre-promo (criterio p.8: bl A=6033
#      B=6033, Δ=0 — ab-out/disasm-af1.out);
#  (7) micro R=5 = run-micro.sh come s159 (scoreboard al pin nuovo).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp159-harness"
BIN="$HOME/Claude/php-rust-output/release/phpr"
RUNNER="$HOME/Claude/php-rust-output/release/phpt-runner"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
WD="/Volumes/Extreme Pro/Claude/wp13-harness/run-with-watchdog.sh"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
QUIESCE="$SRC/wp129-harness/s129-quiescenza.sh"
M149="$SRC/wp149-harness"
SP="${PROMO_SP:?PROMO_SP (workdir APFS per i gate ORM/hk) richiesto}"
OUT="$H/promo-out"; mkdir -p "$OUT"
VERD="$H/s160-promo-verdetto.out"
CAND_EXP="${1:?uso: s160-promozione.sh <cand_hash16>}"
note(){ echo "$1"; echo "$1" >> "$VERD"; }
stop(){ note "$1"; echo 1 > "$OUT/rcb"; exit 1; }

cd "$SRC" || exit 4
git diff --quiet -- crates/ || stop "PRE: crates/ sporco — STOP"
[ -e /private/tmp/phpr-measure.lock ] && note "lock CI: presente (finestra di sessione, non lo tocco)" || note "lock CI: ASSENTE — la sessione lo doveva creare (proseguo, dichiarato)"

SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build.log" 2>&1
rc=$?; echo "$rc" > "$OUT/build.rc"
[ "$rc" = 0 ] || stop "build rc=$rc"
HB=$(shasum -a 256 "$BIN" | cut -c1-16)
# EMENDA t2 (dichiarata dopo lo STOP t1): il candidato fu costruito in target
# DEDICATO; la ricetta in target canonica differisce SOLO per LC_UUID+firma
# (±banner mimalloc) — meccanismo nominato (PREP s151, sonda §1-bis). Il gate
# si chiude a CONTENUTO contro il candidato STASHATO; il pin effettivo è $HB.
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
  CSTASH="$STASH/phpr-s160-af1-B"
  [ "$(shasum -a 256 "$CSTASH" | cut -c1-16)" = "$CAND_EXP" ] || stop "stash candidato != $CAND_EXP — STOP"
  ident_contenuto "$BIN" "$CSTASH" > "$OUT/ident-contenuto.txt" 2>&1     || { cat "$OUT/ident-contenuto.txt" >> "$VERD"; stop "build $HB DIVERGE dal candidato OLTRE il meccanismo nominato — STOP"; }
  cat "$OUT/ident-contenuto.txt" >> "$VERD"
  note "promozione: identità candidato a CONTENUTO (emenda t2: cluster LC_UUID/firma/banner) — pin effettivo = $HB (dichiarato)"
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

"$SRC/scripts/pin-phpr.sh" s160 > "$OUT/pin.log" 2>&1
prc=$?; echo "$prc" > "$OUT/pin.rc"
[ "$prc" = 0 ] || stop "pin-phpr.sh rc=$prc"
note "promozione: $(tail -1 "$OUT/pin.log")"

"$SRC/scripts/corpus-gate.sh" "$RUNNER" "$OUT/corpus"
crc=$?; echo "$crc" > "$OUT/corpus-rc"
[ "$crc" = 0 ] || stop "corpus-gate rc=$crc — CE1 non ammette flip (divergenza (3)): STOP"
note "promozione corpus-gate: rc=0 — nomi==congelato (1412), CONTENUTO==golden, off-on zero (ZERO flip come atteso, L-AF1)"

PHPR_PIN_ATTESO="$H2" "$SRC/wp109-harness/s109-fixture-chain.sh" > "$OUT/fixture-chain.out" 2>&1
frc=$?; echo "$frc" > "$OUT/fixture-chain.rc"
[ "$frc" = 0 ] || stop "fixture chain rc=$frc"
FX_ATTESI="hc1 move recv fx20 fx21 w9 preg teardown stash backtrace"
FX_VISTI=$(sed -n 's/^FIXTURE-CHAIN inventario=//p' "$OUT/fixture-chain.out")
[ "$FX_VISTI" = "$FX_ATTESI" ] || stop "fixture chain inventario diverso: visti='$FX_VISTI' attesi='$FX_ATTESI'"
note "promozione fixture chain: rc=0 (10 gate: $FX_VISTI)"

# ---- gate fx-ce AGGIUNTO (divergenza (4)): bilaterale oracle==pin byte-id ----
ORACLE=/opt/homebrew/opt/php/bin/php
"$ORACLE" "$H/fx-ce.php" > "$OUT/fxce-oracle.out" 2>&1
"$BIN" "$H/fx-ce.php" > "$OUT/fxce-pin.out" 2>&1
if diff -q "$OUT/fxce-oracle.out" "$OUT/fxce-pin.out" > /dev/null; then
  note "promozione gate fx-ce: oracle==pin BYTE-ID (12 forme class/interface/trait_exists)"
else
  diff "$OUT/fxce-oracle.out" "$OUT/fxce-pin.out" > "$OUT/fxce.diff" || true
  stop "gate fx-ce: DIVERGE (promo-out/fxce.diff)"
fi
# ---- gate sonda-bt (divergenza (4)): pin==stash s159 byte-id (§3.25 invariato) ----
"$STASH/phpr-s159" "$H/sonda-bt-autoload.php" > "$OUT/sonda-bt-s159.out" 2>&1
"$BIN" "$H/sonda-bt-autoload.php" > "$OUT/sonda-bt-pin.out" 2>&1
if diff -q "$OUT/sonda-bt-s159.out" "$OUT/sonda-bt-pin.out" > /dev/null; then
  note "promozione gate sonda-bt: pin==stash s159 BYTE-ID (contratto autoload/backtrace §3.25 invariato)"
else
  stop "gate sonda-bt: pin DIVERGE dallo stash s159 (promo-out/sonda-bt-pin.out)"
fi
# ---- gate fx-refl (divergenza (4)): pin==stash s159 byte-id (contratto L-RF2 invariato) ----
"$STASH/phpr-s159" "$H/fx-refl.php" > "$OUT/fxrefl-s159.out" 2>&1
"$BIN" "$H/fx-refl.php" > "$OUT/fxrefl-pin.out" 2>&1
if diff -q "$OUT/fxrefl-s159.out" "$OUT/fxrefl-pin.out" > /dev/null; then
  note "promozione gate fx-refl: pin==stash s159 BYTE-ID (contratto __reflect_* L-RF2 invariato)"
else
  stop "gate fx-refl: pin DIVERGE dallo stash s159 (promo-out/fxrefl-pin.out)"
fi
# ---- gate fx-am NUOVO (divergenza (4)): bilaterale oracle==pin byte-id ----
"$ORACLE" "$H/fx-am.php" > "$OUT/fxam-oracle.out" 2>&1
"$BIN" "$H/fx-am.php" > "$OUT/fxam-pin.out" 2>&1
if diff -q "$OUT/fxam-oracle.out" "$OUT/fxam-pin.out" > /dev/null; then
  note "promozione gate fx-am: oracle==pin BYTE-ID (20 forme array_map v2, fast E pieno)"
else
  diff "$OUT/fxam-oracle.out" "$OUT/fxam-pin.out" > "$OUT/fxam.diff" || true
  stop "gate fx-am: DIVERGE (promo-out/fxam.diff)"
fi
# ---- gate fx-af NUOVO (divergenza (4)): bilaterale oracle==pin byte-id ----
"$ORACLE" "$H/fx-af.php" > "$OUT/fxaf-oracle.out" 2>&1
"$BIN" "$H/fx-af.php" > "$OUT/fxaf-pin.out" 2>&1
if diff -q "$OUT/fxaf-oracle.out" "$OUT/fxaf-pin.out" > /dev/null; then
  note "promozione gate fx-af: oracle==pin BYTE-ID (13 forme array_filter, fast E pieno)"
else
  diff "$OUT/fxaf-oracle.out" "$OUT/fxaf-pin.out" > "$OUT/fxaf.diff" || true
  stop "gate fx-af: DIVERGE (promo-out/fxaf.diff)"
fi

QOK=1
for t in $(seq 1 30); do
  if "$QUIESCE" "$OUT/quiesce.rc" > "$OUT/quiesce-t$t.log" 2>&1; then QOK=0; note "promozione quiescenza: PASS al tentativo $t"; break; fi
  sleep 60
done
[ "$QOK" = 0 ] || stop "quiescenza MAI PASS in 30 tentativi — STOP prima delle misure"

PHPR="$BIN" R=5 "$SRC/wp97-harness/micro/run-micro.sh" > "$OUT/micro-pin-s160.out" 2>&1
note "promozione micro pin s160: $(grep -E '^rapporto_' "$OUT/micro-pin-s160.out" | tr '\n' ' ')"

# ---- conferma POST-PIN m-refl (divergenza (5)): R=5 pin vs stash s159 ----
AOLD="$STASH/phpr-s159"
[ "$(shasum -a 256 "$AOLD" | cut -c1-16)" = "f2d17f18c00a4049" ] || stop "stash phpr-s159 hash inatteso"
ucpu(){ { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$1" "$2" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3(){ local a b c; a=$(ucpu "$1" "$2"); b=$(ucpu "$1" "$2"); c=$(ucpu "$1" "$2"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }
FA=$(floor3 "$AOLD" "$H/empty.php"); FB=$(floor3 "$BIN" "$H/empty.php")
CT="$OUT/conferma-runs.tsv"; : > "$CT"
for i in 1 2 3 4 5; do
  if [ $((i % 2)) -eq 1 ]; then TA=$(ucpu "$AOLD" "$H/m-arrfilter.php"); TB=$(ucpu "$BIN" "$H/m-arrfilter.php"); else TB=$(ucpu "$BIN" "$H/m-arrfilter.php"); TA=$(ucpu "$AOLD" "$H/m-arrfilter.php"); fi
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
note "conferma post-pin m-arrfilter (pin s160 vs stash s159, drift-tree da dichiarare): $CONF (attesa: segno +, D nell'intorno di +16,0 ± rumore+drift)"

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

"$SRC/scripts/pin-server.sh" s160 > "$OUT/pin-server.log" 2>&1
src_rc=$?; echo "$src_rc" > "$OUT/pin-server.rc"
[ "$src_rc" = 0 ] || stop "pin-server.sh rc=$src_rc"
note "promozione server: $(grep '^PIN server' "$OUT/pin-server.log" | tail -1)"

echo 0 > "$OUT/rcb"
note "PROMOZIONE COMPLETA rc=0 (da promo-out/rcb): pin s160 = $H2"
exit 0
