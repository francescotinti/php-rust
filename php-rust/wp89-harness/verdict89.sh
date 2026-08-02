#!/bin/bash
# verdict89.sh — S-89.0: verdetto macchina della campagna measure89.
# Fail-closed, COLLECT-THEN-EMIT per blocco (A-SK44); i blocchi a valle
# consumano i FLAG di pulizia a monte (A-SK47/KS-SK-88-2/KS-SK-89-3).
# Novità WP-90 (Concilio, ordine S-89.0 p3):
#   A-AH51/KS-AH-90-2: il GIUDICE è identificato per sha — righe ledger
#     con judge_sha=sha256(verdict89.sh)[0:16]; VIETATO riusare
#     campaign_sha come marcatore.
#   A-SK59/KS-SK-90-3: mem_hash ri-verificato contro il matrix COMMITTED
#     a HEAD (tether che prima viveva solo nella campagna); le
#     generazioni gG sono governate: g>1 dichiara la supersessione in
#     header e il doc cita SEMPRE la generazione MASSIMA per attempt.
#   A-SK58: `spans=` GIUDICATO in VWARM — stag=>NO-OVERLAP,
#     base/warm=>OVERLAP, INVALID=>FAIL (mai piu riportato-senza-dente).
#   A-BG50/KG-90-2: presence-guard sugli estrattori — riga tag=mi_proc
#     win=0 contata ==1 e C>0 PRIMA dell'echo in TMP; mai `v+0` come
#     sorgente di un aggregato (riga assente => FAIL, mai 0 silenzioso).
#   A-BG51/KG-90-3: srv_boot_epoch GIUDICATO per raw (presente e
#     <= epoch della riga identity; assente/incoerente => raw VOID).
#   A-BG52: ogni label di discriminatore VWARM porta il REGIME di
#     validita IN-BAND (floor-collapse => label VOID-di-significato).
# Novità WP-89 (tutte le sedie):
#   A-SK54/A-BG46: verdetto PER-ATTEMPT e PER-GENERAZIONE
#     (verdict89.aN.gG.out, name-reuse rifiutato); coerenza pointer<->
#     suffixed .done giudicata; esito appeso al LEDGER (A-AH49/KG-89-1).
#   A-SK51: identity line count==1; VARMS TORNA (m89 ha TRE bracci
#     d'ambiente: default / ret0 / eagerpos) — read-back giudicato
#     per-braccio su OGNI raw (A-DL41/A-DL44/KL-90-4).
#   A-BG47/KG-89-2: identity v2 giudicata — server_exit==0, srv_pid=
#     riconciliato con TUTTE le righe pid= del raw.
#   A-PP42/KS-PP-89-2: VDISP esige thr-set == {0..W-1} ESATTO
#     (unicità+completezza), mai la sola cardinalità.
#   A-BG48/KG-89-3: companion mancante = FAIL del raw — mai 0/NA dentro
#     min/LSQ; righe peak dai .log con nul_count= IN-BAND (A-SK52: canale
#     DECLARED-DEVIATION quando nul_count>0, mai strip silenzioso).
#   A-BB57/A-BB58 (KB-90-1/2): mode-census per W; LSQ di b sui MODI
#     DOMINANTI con se(b) e b±2σ IN-BAND + fascia δ=0,15 EX-ANTE sul
#     MARGINALE (fuori fascia => grade ADVISORY); modes==R = census
#     NON-informativo, punto ADVISORY. Banda KL-85-2 RITIRATA (KB-90-2):
#     NESSUN confronto di banda cross-protocollo.
#   VATTR (KL-90-4): attribuzione di b — b_base vs b_ret0 contro le
#     soglie ex-ante P-RET0; verdict-grade SOLO con census per-theap +
#     braccio retain armato entrambi in-band.
#   A-BB55: blocco VARENA — righe tag=mi_arena obbligatorie, parse=FAILED
#     = FAIL; arena==proc = identita di read-path (A-DL45/KL-90-2).
#   A-BB59: blocco VSWEEP — stagger invertito con spans GIUDICATO
#     (dt0=>OVERLAP, dt20=>NO-OVERLAP, INVALID=>FAIL) e discriminatore
#     ordine-vs-fixture con regime in-band (A-BG52).
#   VUCLOG: DECLARED-ABSENT (A-DS45 consumata in m88; F16b in battery).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
OUT="$REPO/wp78-harness/measure-out"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD)"
# A-AH51: the judge identifies ITSELF by sha — every ledger row this
# script appends carries judge_sha (campaign_sha reuse is banned).
JUDGE_SHA=$(shasum -a 256 "$0" | cut -c1-16)
GITPREFIX="$(git -C "$REPO" rev-parse --show-prefix)"

# --- .done consumption (KG-88-3) + A-SK54 coherence -------------------------
DONE="$OUT/m89.done"
[ -f "$DONE" ] || { echo "FAIL: m89.done missing — campaign never completed (KG-88-3)"; exit 1; }
ATT=$(sed -n 's/.* attempt=\([0-9]*\) .*/\1/p' "$DONE")
ADONE="$OUT/m89.a$ATT.done"
[ -f "$ADONE" ] || { echo "FAIL: per-attempt done $ADONE missing (A-SK54)"; exit 1; }
cmp -s "$DONE" "$ADONE" || { echo "FAIL: m89.done pointer != m89.a$ATT.done (A-SK54 coherence)"; exit 1; }
D_GIT=$(sed -n 's/.* git=\([0-9a-f]*\) .*/\1/p' "$DONE")
D_MH=$(sed -n 's/.* mem_hash=\([0-9a-f]*\) .*/\1/p' "$DONE")
if [ -z "$D_GIT" ] || [ -z "$ATT" ] || [ -z "$D_MH" ]; then
  echo "FAIL: m89.done lacks git=/attempt=/mem_hash= fields"; exit 1
fi

# per-attempt, per-generation verdict file (A-BG46: generations NEVER
# overwrite — a re-judgment gets a fresh gG name).
G=1
while [ -e "$HERE/verdict89.a$ATT.g$G.out" ]; do G=$((G+1)); done
VOUT="$HERE/verdict89.a$ATT.g$G.out"
: > "$VOUT"
say() { echo "$@" | tee -a "$VOUT"; }
FAILS=0
say "== verdict89 — campaign git=$D_GIT attempt=$ATT mem_hash=$D_MH (judge_sha=$JUDGE_SHA, judged at HEAD=$GIT_REV, generation g$G) =="
# A-SK59 generation governance: a later generation SUPERSEDES the earlier
# ones on the same attempt — declared in-band; any doc citing gN while a
# divergent gM>N exists voids its claim (KS-SK-90-3).
[ "$G" -gt 1 ] && say "generation g$G SUPERSEDES g1..g$((G-1)) on attempt=$ATT (KS-SK-90-3: docs must cite the MAXIMUM generation)"

# A-SK59: re-verify mem_hash against the COMMITTED matrix at HEAD — the
# tether lived only in the campaign (KS-AH-87-1); the judge now re-bites.
MTX_NAME=$(sed -n "s/.* attempt=$ATT phase=identity .* matrix=\([^ ]*\).*/\1/p" "$OUT/m89.campaign.ledger" | tail -1)
if [ -z "$MTX_NAME" ]; then
  say "FAIL: no phase=identity ledger row with matrix= for attempt=$ATT (A-SK59)"; exit 1
fi
MTX_MH=$(git -C "$REPO" show "HEAD:${GITPREFIX}wp78-harness/matrix-archive/$MTX_NAME" 2>/dev/null | tr -d '\0' | sed -n 's/^bin\[mem-census\] sha256\[0:16\]=\([0-9a-f]*\).*/\1/p')
if [ "$MTX_MH" != "$D_MH" ]; then
  say "FAIL: mem_hash=$D_MH != committed matrix bin[mem-census]=$MTX_MH (matrix=$MTX_NAME at HEAD) — judge-side tether (A-SK59/KS-AH-87-1)"; exit 1
fi
say "matrix tether: mem_hash==bin[mem-census] of COMMITTED $MTX_NAME (A-SK59)"

nul_free() { perl -0777 -ne 'exit(index($_,"\0")>=0?1:0)' "$1"; }
nul_count() { perl -0777 -ne 'my $c=()=/\0/g; print $c' "$1"; }

WS="4 8 12 16"
DTS="0 1 2 5 10 20"
SLOPE_RAWS=""
for w in $WS; do for r in 1 2 3 4 5; do SLOPE_RAWS="$SLOPE_RAWS m89.slope.w$w.r$r.a$ATT.memcensus"; done; done
SLOPE0_RAWS=""
for w in $WS; do for r in 1 2 3 4 5; do SLOPE0_RAWS="$SLOPE0_RAWS m89.slope0.w$w.r$r.a$ATT.memcensus"; done; done
EAGER_RAW="m89.eagerpos.w4.r1.a$ATT.memcensus"
CAL_RAWS="m89.cala.r1.a$ATT.memcensus m89.cala.r2.a$ATT.memcensus m89.calb.r1.a$ATT.memcensus m89.calb.r2.a$ATT.memcensus"
SWEEP_RAWS=""
for dt in $DTS; do for o in afirst bfirst; do SWEEP_RAWS="$SWEEP_RAWS m89.sweep.dt$dt.$o.a$ATT.memcensus"; done; done
ALL_RAWS="$SLOPE_RAWS $SLOPE0_RAWS $EAGER_RAW $CAL_RAWS $SWEEP_RAWS"

bf=0; buf=""
emit() { buf="$buf$*
"; }
flushb() { printf '%s' "$buf" | tee -a "$VOUT" >/dev/null; FAILS=$((FAILS+bf)); buf=""; }

# --- Block VIDENT v2 (A-SK51 + A-BG47 + KG-89-2) ----------------------------
IDENT_CLEAN=0
bf=0
for raw in $ALL_RAWS; do
  F="$OUT/$raw"
  if [ ! -f "$F" ]; then emit "FAIL VIDENT: raw missing: $raw"; bf=$((bf+1)); continue; fi
  if ! nul_free "$F"; then emit "FAIL VIDENT: NUL byte in $raw — raw REFUSED, never stripped (KS-SK-88-3)"; bf=$((bf+1)); continue; fi
  NID=$(grep -c "^mem_hash=" "$F")
  if [ "$NID" != 1 ]; then emit "FAIL VIDENT: $raw has $NID identity lines, expected exactly 1 (A-SK51)"; bf=$((bf+1)); continue; fi
  ID=$(grep "^mem_hash=" "$F")
  case "$ID" in
    *"mem_hash=$D_MH git=$D_GIT"*" attempt=$ATT "*) : ;;
    *) emit "FAIL VIDENT: identity mismatch in $raw: $ID"; bf=$((bf+1)); continue;;
  esac
  SEXIT=$(echo "$ID" | sed -n 's/.* server_exit=\([0-9-]*\) .*/\1/p')
  [ "$SEXIT" = 0 ] || { emit "FAIL VIDENT: $raw server_exit=$SEXIT != 0 (A-BG47/KG-89-2 — the a1 orphan signal, now judged)"; bf=$((bf+1)); continue; }
  SPID=$(echo "$ID" | sed -n 's/.* srv_pid=\([0-9]*\) .*/\1/p')
  [ -n "$SPID" ] || { emit "FAIL VIDENT: $raw identity lacks srv_pid= (A-BG47)"; bf=$((bf+1)); continue; }
  # A-BG51/KG-90-3: srv_boot_epoch JUDGED — present and coherent (boot
  # precedes the identity-row epoch); absent/incoherent => raw VOID.
  BEPOCH=$(echo "$ID" | sed -n 's/.* srv_boot_epoch=\([0-9]*\) .*/\1/p')
  REPOCH=$(echo "$ID" | sed -n 's/.* epoch=\([0-9]*\).*/\1/p')
  if [ -z "$BEPOCH" ] || [ -z "$REPOCH" ]; then
    emit "FAIL VIDENT: $raw srv_boot_epoch=/epoch= missing — raw VOID (A-BG51/KG-90-3)"; bf=$((bf+1)); continue
  fi
  if [ "$BEPOCH" -gt "$REPOCH" ]; then
    emit "FAIL VIDENT: $raw srv_boot_epoch=$BEPOCH > epoch=$REPOCH — incoherent boot (A-BG51/KG-90-3)"; bf=$((bf+1)); continue
  fi
  BADPID=$(awk -v want="$SPID" '/^pid=/ { split($1,a,"="); if (a[2]+0 != want+0) { print; exit } }' "$F")
  [ -z "$BADPID" ] || { emit "FAIL VIDENT: $raw census pid rows not reconciled with srv_pid=$SPID (KG-89-2): $BADPID"; bf=$((bf+1)); continue; }
done
if [ "$bf" = 0 ]; then emit "VIDENT PASS: $(echo $ALL_RAWS | wc -w | tr -d ' ') raws present, NUL-free, identity v2 (count==1, server_exit==0, srv_pid reconciled; mem_hash=$D_MH git=$D_GIT attempt=$ATT)"; IDENT_CLEAN=1; fi
flushb

# --- Block VDISP (A-PP37 + A-PP42/KS-PP-89-2: exact thr-set) ----------------
DISP_CLEAN=0
bf=0
if [ "$IDENT_CLEAN" = 1 ]; then
  for raw in $ALL_RAWS; do
    F="$OUT/$raw"
    ID=$(grep "^mem_hash=" "$F")
    W=$(echo "$ID" | sed -n 's/.* w=\([0-9]*\) .*/\1/p')
    NREQ=$(echo "$ID" | sed -n 's/.* nreq=\([0-9]*\) .*/\1/p')
    MAP=$(awk '/tag=worker_dispatch/ { t=""; c=""; for (i=1;i<=NF;i++) { if ($i ~ /^thr=/) {t=$i; sub("thr=","",t)} if ($i ~ /^count=/) {c=$i; sub("count=","",c)} } print t":"c }' "$F" | sort -t: -k1,1n)
    THRSET=$(echo "$MAP" | cut -d: -f1 | tr '\n' ',' | sed 's/,$//')
    WANTSET=$(seq 0 $((W-1)) | tr '\n' ',' | sed 's/,$//')
    if [ "$THRSET" != "$WANTSET" ]; then
      emit "FAIL VDISP: $raw thr-set {$THRSET} != {$WANTSET} — uniqueness+completeness, not cardinality (A-PP42/KS-PP-89-2)"; bf=$((bf+1)); continue
    fi
    PER=$((NREQ / W))
    BAD=$(echo "$MAP" | awk -F: -v per=$PER '$2+0 != per {print}')
    if [ -n "$BAD" ]; then emit "FAIL VDISP: $raw per-thread map != $PER each: $(echo $BAD | tr '\n' ' ')"; bf=$((bf+1)); fi
  done
  if [ "$bf" = 0 ]; then emit "VDISP PASS: thr-set == {0..W-1} exact and per-thread count == nreq/W on every raw (A-PP37+A-PP42)"; DISP_CLEAN=1; fi
else
  emit "VDISP SKIPPED: upstream VIDENT not clean (KS-SK-88-2)"; bf=1
fi
flushb

# --- Block VORD (A-SK45 + A-DL36 + A-DS39; mode-aware for warm) -------------
ORD_CLEAN=0
bf=0
if [ "$IDENT_CLEAN" = 1 ]; then
  for raw in $SLOPE_RAWS $SLOPE0_RAWS $EAGER_RAW $CAL_RAWS $SWEEP_RAWS; do
    F="$OUT/$raw"
    ID=$(grep "^mem_hash=" "$F")
    W=$(echo "$ID" | sed -n 's/.* w=\([0-9]*\) .*/\1/p')
    EXP_PER_THR=1   # m89: nessuna fase warm (una richiesta per worker ovunque)
    NME=$(grep -c "tag=unitcache_main_entry" "$F" || true)
    WANTME=$((W * EXP_PER_THR))
    if [ "$NME" != "$WANTME" ]; then emit "FAIL VORD: $raw has $NME main-entry rows, expected $WANTME"; bf=$((bf+1)); continue; fi
    BADORD=$(awk -v maxo=$EXP_PER_THR '/tag=unitcache_main_entry/ { ok=0; cl=""; for (i=1;i<=NF;i++) { if ($i ~ /^ord=/) { o=$i; sub("ord=","",o); if (o+0>=1 && o+0<=maxo) ok=1 } if ($i ~ /^clamped=/) cl=$i } if (!ok || cl!="clamped=0") print NR": "$0 }' "$F")
    if [ -n "$BADORD" ]; then emit "FAIL VORD: $raw main-entry row outside ord profile (1..$EXP_PER_THR) or clamped!=0:"; emit "$BADORD"; bf=$((bf+1)); fi
    BADQ=$(awk -v e="entries=$EXP_PER_THR" -v m="mains=$EXP_PER_THR" '/tag=unitcache_thr/ { ee=""; mm=""; for (i=1;i<=NF;i++) { if ($i ~ /^entries=/) ee=$i; if ($i ~ /^mains=/) mm=$i } if (ee!=e || mm!=m) print NR": "$0 }' "$F")
    if [ -n "$BADQ" ]; then emit "FAIL VORD: $raw thread cache not at expected profile ($EXP_PER_THR/$EXP_PER_THR):"; emit "$BADQ"; bf=$((bf+1)); fi
  done
  if [ "$bf" = 0 ]; then emit "VORD PASS: main-entry ord profile + clamped=0 + thread-cache profile hold on every judged raw (warm profile 2/2 declared)"; ORD_CLEAN=1; fi
else
  emit "VORD SKIPPED: upstream VIDENT not clean (KS-SK-88-2)"; bf=1
fi
flushb

# --- Block VARENA (A-BB55: in-band arena/commit term) -----------------------
ARENA_CLEAN=0
bf=0
if [ "$IDENT_CLEAN" = 1 ]; then
  for raw in $SLOPE_RAWS $SLOPE0_RAWS $EAGER_RAW; do
    F="$OUT/$raw"
    grep -q "tag=mi_arena_json win=0" "$F" || { emit "FAIL VARENA: $raw lacks mi_arena_json win=0 row (A-BB55)"; bf=$((bf+1)); continue; }
    grep -q "tag=mi_arena win=0 .*parse=FAILED" "$F" && { emit "FAIL VARENA: $raw has parse=FAILED mi_arena rows"; bf=$((bf+1)); continue; }
    grep -q "tag=mi_arena win=0 key=committed " "$F" || { emit "FAIL VARENA: $raw lacks extracted committed row"; bf=$((bf+1)); continue; }
  done
  if [ "$bf" = 0 ]; then emit "VARENA PASS: mi_arena_json + extracted rows present, no parse failures, on every slope-arm raw (arena==proc = READ-PATH IDENTITY, never a cross-check — A-DL45/KL-90-2)"; ARENA_CLEAN=1; fi
else
  emit "VARENA SKIPPED: upstream VIDENT not clean (KS-SK-88-2)"; bf=1
fi
flushb

# --- Block VARMS (A-DL41/A-DL44/KL-90-4: per-arm read-back judged) -----------
ARMS_CLEAN=0
bf=0
if [ "$IDENT_CLEAN" = 1 ]; then
  for raw in $ALL_RAWS; do
    F="$OUT/$raw"
    PD=$(sed -n 's/.*tag=mi_option win=0 name=purge_delay ord=15 val=\([0-9-]*\).*/\1/p' "$F" | head -1)
    RT=$(sed -n 's/.*tag=mi_option win=0 name=page_full_retain ord=36 val=\([0-9-]*\).*/\1/p' "$F" | head -1)
    EG=$(sed -n 's/.*tag=mi_option win=0 name=arena_eager_commit ord=4 val=\([0-9-]*\).*/\1/p' "$F" | head -1)
    [ "$PD" = 0 ] || { emit "FAIL VARMS: $raw purge_delay read-back '$PD' != 0 (A-DL41)"; bf=$((bf+1)); continue; }
    case "$raw" in
      *slope0.*)
        # discrimination arm: ord 36 must read 0 against default 2 —
        # the positive that bites on its own (KL-90-4).
        [ "$RT" = 0 ] || { emit "FAIL VARMS: $raw page_full_retain read-back '$RT' != 0 on the RET0 arm — mute arm (A-DL41/KL-90-4)"; bf=$((bf+1)); continue; }
        ;;
      *eagerpos.*)
        # A-DL44: the ordinal-4 positive — armed value 1 vs default 2.
        [ "$EG" = 1 ] || { emit "FAIL VARMS: $raw arena_eager_commit read-back '$EG' != 1 on the EAGER arm (A-DL44)"; bf=$((bf+1)); continue; }
        [ "$RT" = 2 ] || { emit "FAIL VARMS: $raw page_full_retain read-back '$RT' != default 2 (A-DL41)"; bf=$((bf+1)); continue; }
        ;;
      *)
        [ "$RT" = 2 ] || { emit "FAIL VARMS: $raw page_full_retain read-back '$RT' != default 2 on a default-arm raw (A-DL41)"; bf=$((bf+1)); continue; }
        ;;
    esac
  done
  if [ "$bf" = 0 ]; then emit "VARMS PASS: per-arm mi_option read-back judged on EVERY raw (purge=0 all; ord36: 0 on ret0 / 2 elsewhere; ord4: 1 on eagerpos — A-DL41/A-DL44/KL-90-4)"; ARMS_CLEAN=1; fi
else
  emit "VARMS SKIPPED: upstream VIDENT not clean (KS-SK-88-2)"; bf=1
fi
flushb

# --- Block VTHEAP (A-DL46-census/KL-90-4: per-theap census present) ----------
THEAP_CLEAN=0
bf=0
if [ "$IDENT_CLEAN" = 1 ]; then
  for raw in $SLOPE_RAWS $SLOPE0_RAWS $EAGER_RAW; do
    F="$OUT/$raw"
    grep -q "tag=mi_theap_pages win=0 .*visit=FAILED" "$F" && { emit "FAIL VTHEAP: $raw theap census visit=FAILED"; bf=$((bf+1)); continue; }
    grep -q "tag=mi_theap_pages win=0 heap=" "$F" || { emit "FAIL VTHEAP: $raw lacks mi_theap_pages rows (A-DL46-census/KL-90-4)"; bf=$((bf+1)); continue; }
    grep -q "tag=mi_theap_pages win=0 heaps_total=" "$F" || { emit "FAIL VTHEAP: $raw lacks the declared heaps_total trailer (KL-90-3)"; bf=$((bf+1)); continue; }
  done
  if [ "$bf" = 0 ]; then emit "VTHEAP PASS: per-theap page census in-band on every slope-arm raw (heap-by-visit-index + win0-postteardown DECLARED, KL-90-3)"; THEAP_CLEAN=1; fi
else
  emit "VTHEAP SKIPPED: upstream VIDENT not clean (KS-SK-88-2)"; bf=1
fi
flushb

# --- Blocks VSLOPE-{BASE,RET0} (A-BB57/A-BB58; KB-90-1/2; A-BG48/50) ---------
# La banda KL-85-2 è RITIRATA (KB-90-2): nessun confronto di banda
# cross-protocollo — il giudizio di m89 è l'ATTRIBUZIONE (VATTR, P-RET0).
B_BASE=""; B_RET0=""
judge_slope() { # <ARM> <raw-prefix>   (usa emit/bf globali; setta B_LAST)
  local ARM="$1" PFX="$2"
  local TMP; TMP=$(mktemp)
  local w r F L NC FP DEVTAG C SL AC AN NPR NAC
  for w in $WS; do
    for r in 1 2 3 4 5; do
      F="$OUT/m89.$PFX.w$w.r$r.a$ATT.memcensus"
      grep -q "exit_collect_mi" "$F" || { emit "FAIL VSLOPE-$ARM: $F lacks exit_collect_mi marker"; bf=$((bf+1)); continue; }
      # A-BG50/KG-90-2: presence-guard BEFORE extraction — an absent row
      # used to become a silent 0 inside TMP and the LSQ (`v+0`).
      NPR=$(grep -c "tag=mi_proc win=0 " "$F" || true)
      [ "$NPR" = 1 ] || { emit "FAIL VSLOPE-$ARM: $F tag=mi_proc win=0 rows == $NPR, expected exactly 1 (A-BG50/KG-90-2)"; bf=$((bf+1)); continue; }
      NAC=$(grep -c "tag=mi_arena win=0 key=committed " "$F" || true)
      [ "$NAC" = 1 ] || { emit "FAIL VSLOPE-$ARM: $F mi_arena committed rows == $NAC, expected exactly 1 (A-BG50/KG-90-2)"; bf=$((bf+1)); continue; }
      C=$(awk '/tag=mi_proc win=0/ { for (i=1;i<=NF;i++) if ($i ~ /^commit=/) {v=$i; sub("commit=","",v)} } END { print v+0 }' "$F")
      [ "$C" -gt 0 ] || { emit "FAIL VSLOPE-$ARM: $F extracted C=$C not > 0 (A-BG50/KG-90-2)"; bf=$((bf+1)); continue; }
      SL=$(awk 'BEGIN{inpost=0} /tag=mi_proc win=0/ { inpost=1; s=0 } inpost && /tag=mi_bin win=0/ { c=0; u=0; for (i=1;i<=NF;i++) { if ($i ~ /^committed=/) {c=$i; sub("committed=","",c)} if ($i ~ /^used_b=/) {u=$i; sub("used_b=","",u)} } s += c-u } END { print s+0 }' "$F")
      AC=$(awk '/tag=mi_arena win=0 key=committed /{ for (i=1;i<=NF;i++) if ($i ~ /^current=/) {v=$i; sub("current=","",v)} } END { print v+0 }' "$F")
      [ "$AC" -gt 0 ] || { emit "FAIL VSLOPE-$ARM: $F extracted arena committed=$AC not > 0 (A-BG50/KG-90-2)"; bf=$((bf+1)); continue; }
      AN=$(awk '/tag=mi_arena win=0 key=arena_count /{ for (i=1;i<=NF;i++) if ($i ~ /^val=/) {v=$i; sub("val=","",v)} } END { print v+0 }' "$F")
      L="$OUT/m89.$PFX.w$w.r$r.a$ATT.log"
      [ -f "$L" ] || { emit "FAIL VSLOPE-$ARM: companion log missing for w=$w r=$r — raw FAILS, never defaulted (A-BG48)"; bf=$((bf+1)); continue; }
      NC=$(nul_count "$L")
      if [ "$NC" = 0 ]; then
        FP=$(awk '/peak memory footprint/{print $1}' "$L")
        DEVTAG=""
      else
        FP=$(tr -d '\0' < "$L" | awk '/peak memory footprint/{print $1}')
        DEVTAG=" DECLARED-DEVIATION nul_count=$NC (A-SK52/KS-SK-89-2: parsed via strip, deviation in-band)"
      fi
      [ -n "$FP" ] || { emit "FAIL VSLOPE-$ARM: peak companion MISSING for w=$w r=$r — FAIL, never 0/NA in an aggregate (A-BG48/KG-89-3)"; bf=$((bf+1)); continue; }
      echo "$w $r $C $SL $FP $AC $AN" >> "$TMP"
      emit "slope-$ARM w=$w r=$r committed_postcollect_win0_bytes=$C slack_committed_minus_used_bytes=$SL peak_memory_footprint_bytes=$FP arena_committed_current_bytes=$AC arena_count=$AN$DEVTAG"
    done
  done
  if [ "$bf" = 0 ]; then
    local REPORT
    REPORT=$(awk -v arm="$ARM" '
      { vals[$1] = vals[$1] " " $3
        if (minc[$1]=="" || $3<minc[$1]) minc[$1]=$3
        an[$1]=$7 }
      END {
        nws=split("4 8 12 16", wsarr, " ")
        R=5
        for (i=1;i<=nws;i++) {
          w=wsarr[i]
          n=split(vals[w], vv, " ")
          delete cnt
          for (j=1;j<=n;j++) cnt[vv[j]]++
          best=""; bestn=0; nm=0
          for (v in cnt) {
            nm++
            printf "mode-census %s W=%d: committed=%d B x%d\n", arm, w, v, cnt[v]
            if (cnt[v]>bestn || (cnt[v]==bestn && (best=="" || v+0<best+0))) { best=v; bestn=cnt[v] }
          }
          dom[w]=best
          # A-BB58 (KB-90-1): modes==R = census NON-informativo, punto ADVISORY
          adv = (nm==R ? " [A-BB58: modes==R, census NON-informative => point ADVISORY]" : (nm>=2 ? " [min-of-R ADVISORY: >=2 modes, KB-89-2]" : ""))
          printf "mode-census %s W=%d: modes=%d dominant=%d B (x%d) min-of-R=%d B%s\n", arm, w, nm, best, bestn, minc[w], adv
          printf "%s W=%d granules64k: dominant=%.2f residue_dominant=%d B | arena_count=%d\n", arm, w, dom[w]/65536, dom[w]%65536, an[w]
        }
        # LSQ b on dominant modes + A-BB57: se(b), b +/- 2sigma, delta-fascia
        n=0; sx=0; sy=0; sxx=0; sxy=0
        for (i=1;i<=nws;i++) { w=wsarr[i]; n++; sx+=w; sy+=dom[w]; sxx+=w*w; sxy+=w*dom[w] }
        b=(n*sxy-sx*sy)/(n*sxx-sx*sx)
        a=(sy-b*sx)/n
        sse=0
        for (i=1;i<=nws;i++) { w=wsarr[i]; rres=dom[w]-(a+b*w); sse+=rres*rres }
        wbar=sx/n; sww=0
        for (i=1;i<=nws;i++) { w=wsarr[i]; sww+=(w-wbar)*(w-wbar) }
        seb=sqrt((sse/(n-2))/sww)
        grade="verdict-grade-candidate"
        # A-BB57 fascia del marginale, delta=0,15 EX-ANTE (KB-90-1)
        for (i=2;i<=nws;i++) {
          d=dom[wsarr[i]]-dom[wsarr[i-1]]
          m=d/(wsarr[i]-wsarr[i-1])
          inband = (m >= b*0.85 && m <= b*1.15) ? "IN" : "OUT"
          if (inband=="OUT") grade="ADVISORY (marginal outside the ex-ante delta=0.15 band, KB-90-1)"
          printf "delta %s W%d->W%d = %d B, marginal %.0f B/worker [%s ex-ante band b*(1+/-0.15)]\n", arm, wsarr[i-1], wsarr[i], d, m, inband
        }
        printf "VSLOPE-%s b (LSQ dominant-mode, metric=committed_postcollect_win0, W in {4, 8, 12, 16}): %.0f B/worker se=%.0f B twosigma=[%.0f, %.0f] a=%.0f B grade=%s\n", arm, b, seb, b-2*seb, b+2*seb, a, grade
      }' "$TMP")
    emit "$REPORT"
    B_LAST=$(echo "$REPORT" | sed -n "s/^VSLOPE-$ARM b (LSQ[^:]*): \([0-9]*\) B\/worker.*/\1/p")
    emit "VSLOPE-$ARM PASS: protocol clean (collect armed, presence-guarded extractors, mode-census + b+/-2sigma + delta-fascia in-band)"
  fi
  rm -f "$TMP"
}
bf=0
if [ "$IDENT_CLEAN" = 1 ] && [ "$DISP_CLEAN" = 1 ] && [ "$ORD_CLEAN" = 1 ] && [ "$ARENA_CLEAN" = 1 ] && [ "$ARMS_CLEAN" = 1 ] && [ "$THEAP_CLEAN" = 1 ]; then
  judge_slope BASE slope
  [ "$bf" = 0 ] && B_BASE="$B_LAST"
else
  emit "VSLOPE-BASE SKIPPED: upstream blocks not clean (KS-SK-88-2)"; bf=$((bf+1))
fi
flushb
bf=0
if [ "$IDENT_CLEAN" = 1 ] && [ "$DISP_CLEAN" = 1 ] && [ "$ORD_CLEAN" = 1 ] && [ "$ARENA_CLEAN" = 1 ] && [ "$ARMS_CLEAN" = 1 ] && [ "$THEAP_CLEAN" = 1 ]; then
  judge_slope RET0 slope0
  [ "$bf" = 0 ] && B_RET0="$B_LAST"
else
  emit "VSLOPE-RET0 SKIPPED: upstream blocks not clean (KS-SK-88-2)"; bf=$((bf+1))
fi
flushb

# --- Block VATTR (KL-90-4: l'attribuzione di b, P-RET0 ex-ante) --------------
bf=0
if [ -n "$B_BASE" ] && [ -n "$B_RET0" ]; then
  HALF=$((B_BASE / 2))
  EIGHT=$((B_BASE * 8 / 10))
  if [ "$B_RET0" -le "$HALF" ]; then
    ATTR="ATTRIBUTED-to-page_full_retain (P-RET0 CONFIRMED at the ex-ante threshold b_ret0 <= 0,5*b_base)"
  elif [ "$B_RET0" -ge "$EIGHT" ]; then
    ATTR="NOT-attributed-to-retention (P-RET0 REFUTED: b survives retain=0 — residual driver OPEN, to be NAMED at council)"
  else
    ATTR="PARTIAL (between the ex-ante thresholds 0,5 and 0,8 — mixed drivers, decomposition needed)"
  fi
  emit "VATTR: b_base=$B_BASE B/worker | b_ret0=$B_RET0 B/worker -> $ATTR [verdict-grade: theap census + retain arm both in-band, KL-90-4; grade inherits the slope grades above]"
  emit "VATTR PASS: attribution judged against the ex-ante P-RET0 thresholds"
else
  emit "VATTR SKIPPED: slope arms not both clean (KL-90-4: without census+arm the attribution stays envelope)"; bf=$((bf+1))
fi
flushb

# --- Block VSWEEP (A-BB59 + A-SK58 + A-BG52) ---------------------------------
# Lo sweep sostituisce il blocco conc di m88: coppie {pad87a,pad87b} a
# dt in {0,1,2,5,10,20} ms, ordine afirst/bfirst. Mapping A-SK58
# DICHIARATO: dt=0 => OVERLAP obbligatorio; dt=20 => NO-OVERLAP
# obbligatorio; INVALID => FAIL ovunque; dt intermedi = stato EMPIRICO
# riportato. Ogni label di discriminazione porta il REGIME in-band
# (A-BG52). I net concorrenti restano VOID come cifre per-thread
# (KB-88-1): qui discriminano il DRIVER, mai una cifra.
bf=0
if [ "$IDENT_CLEAN" = 1 ] && [ "$DISP_CLEAN" = 1 ] && [ "$ORD_CLEAN" = 1 ]; then
  getnet() { awk -v pat="$2" '/tag=unitcache_main_entry/ && $0 ~ pat { for (i=1;i<=NF;i++) { if ($i ~ /^net=/) {n=$i; sub("net=","",n)} if ($i ~ /^floor_inc=/) {f=$i; sub("floor_inc=","",f)} } print n" "f }' "$1"; }
  CA1=$(getnet "$OUT/m89.cala.r1.a$ATT.memcensus" "pad87a[.]php")
  CA2=$(getnet "$OUT/m89.cala.r2.a$ATT.memcensus" "pad87a[.]php")
  CB1=$(getnet "$OUT/m89.calb.r1.a$ATT.memcensus" "pad87b[.]php")
  CB2=$(getnet "$OUT/m89.calb.r2.a$ATT.memcensus" "pad87b[.]php")
  if [ -z "$CA1" ] || [ -z "$CB1" ]; then emit "FAIL VSWEEP: calibration rows missing"; bf=$((bf+1)); fi
  [ "$CA1" = "$CA2" ] || { emit "FAIL VSWEEP: cala r1 ($CA1) != r2 ($CA2) — not byte-reproduced"; bf=$((bf+1)); }
  [ "$CB1" = "$CB2" ] || { emit "FAIL VSWEEP: calb r1 ($CB1) != r2 ($CB2) — not byte-reproduced"; bf=$((bf+1)); }
  if [ "$bf" = 0 ]; then
    CALA_NET=${CA1%% *}; CALA_FI=${CA1##* }
    CALB_NET=${CB1%% *}; CALB_FI=${CB1##* }
    SUM=$((CALA_NET + CALB_NET))
    emit "VSWEEP cal: pad87a net=$CALA_NET B floor_inc=$CALA_FI B | pad87b net=$CALB_NET B floor_inc=$CALB_FI B [metric=net-at-lower, net_window=process-counters, W=1 sequential]"
    NOVERLAP=0; NORDMATCH=0; NAPAD=0; DT20OK=0; DT20N=0
    for dt in $DTS; do
      for o in afirst bfirst; do
        F="$OUT/m89.sweep.dt$dt.$o.a$ATT.memcensus"
        OV=$(awk '
          /tag=lower_span/ {
            tid=""; t0=""; t1=""
            for (i=1;i<=NF;i++) {
              if ($i ~ /^tid=/) tid=$i
              if ($i ~ /^t0_us=/) { v=$i; sub("t0_us=","",v); t0=v+0 }
              if ($i ~ /^t1_us=/) { v=$i; sub("t1_us=","",v); t1=v+0 }
            }
            if ($NF ~ /pad87a[.]php$/) { a0=t0; a1=t1; atid=tid }
            else if ($NF ~ /pad87b[.]php$/) { b0=t0; b1=t1; btid=tid }
          }
          END {
            if (atid=="" || btid=="" || atid==btid) { print "INVALID"; exit }
            print (a0<b1 && b0<a1) ? "OVERLAP" : "NO-OVERLAP"
          }' "$F")
        case "$dt" in
          0)  WANT_OV="OVERLAP" ;;
          20) WANT_OV="NO-OVERLAP" ;;
          *)  WANT_OV="REPORT" ;;
        esac
        if [ "$OV" = "INVALID" ]; then
          emit "FAIL VSWEEP: dt$dt.$o spans=INVALID (A-SK58: INVALID always fails)"; bf=$((bf+1)); continue
        fi
        if [ "$WANT_OV" != "REPORT" ] && [ "$OV" != "$WANT_OV" ]; then
          emit "FAIL VSWEEP: dt$dt.$o spans=$OV, protocol requires $WANT_OV (A-SK58 mapping declared in campaign header)"; bf=$((bf+1)); continue
        fi
        NA=$(getnet "$F" "pad87a[.]php"); NB=$(getnet "$F" "pad87b[.]php")
        [ -n "$NA" ] && [ -n "$NB" ] || { emit "FAIL VSWEEP: dt$dt.$o pad entries missing"; bf=$((bf+1)); continue; }
        NA_NET=${NA%% *}; NB_NET=${NB%% *}
        DA=$((NA_NET - SUM)); DB=$((NB_NET - SUM))
        FIRSTPAD=a; [ "$o" = bfirst ] && FIRSTPAD=b
        ABSA=${DA#-}; ABSB=${DB#-}
        SURPLUS=a; [ "$ABSB" -gt "$ABSA" ] && SURPLUS=b
        emit "VSWEEP dt$dt $o: spans=$OV padA net=$NA_NET B dA=$DA B | padB net=$NB_NET B dB=$DB B | first=$FIRSTPAD surplus_side=$SURPLUS [nets VOID per-thread, KB-88-1]"
        if [ "$OV" = "OVERLAP" ]; then
          NOVERLAP=$((NOVERLAP+1))
          [ "$SURPLUS" = "$FIRSTPAD" ] && NORDMATCH=$((NORDMATCH+1))
          [ "$SURPLUS" = a ] && NAPAD=$((NAPAD+1))
        fi
        if [ "$dt" = 20 ]; then
          DT20N=$((DT20N+2))
          [ "$NA_NET" = "$CALA_NET" ] && DT20OK=$((DT20OK+1))
          [ "$NB_NET" = "$CALB_NET" ] && DT20OK=$((DT20OK+1))
        fi
      done
    done
    if [ "$bf" = 0 ]; then
      # P-DT20 (ancora m88): net==cal AL BYTE a finestre disgiunte.
      if [ "$DT20OK" = "$DT20N" ] && [ "$DT20N" -gt 0 ]; then
        emit "VSWEEP P-DT20 CONFIRMED: net==cal AL BYTE on $DT20OK/$DT20N dt=20 sides (zero-swallow reproduced)"
      else
        emit "VSWEEP P-DT20 REFUTED: net==cal only $DT20OK/$DT20N at dt=20 — the m88 anchor did not reproduce (label, not gate)"
      fi
      # P-ORD (A-BB59): il surplus segue l'ORDINE, non il fixture.
      # Regime in-band (A-BG52): il discriminatore vale SOLO sui run in
      # OVERLAP (a finestre disgiunte non esiste un surplus da attribuire).
      if [ "$NOVERLAP" -gt 0 ]; then
        if [ "$NORDMATCH" = "$NOVERLAP" ]; then ORDVER="ORDER-DRIVEN (P-ORD confirmed $NORDMATCH/$NOVERLAP)"
        elif [ "$NAPAD" = "$NOVERLAP" ]; then ORDVER="FIXTURE-DRIVEN-padA (P-ORD refuted: surplus sticks to pad87a $NAPAD/$NOVERLAP)"
        elif [ "$NAPAD" = 0 ]; then ORDVER="FIXTURE-DRIVEN-padB (P-ORD refuted: surplus sticks to pad87b)"
        else ORDVER="UNSTABLE (order-match $NORDMATCH/$NOVERLAP, padA-side $NAPAD/$NOVERLAP — timing-attached, m87/m88 class)"
        fi
        emit "VSWEEP discrimination: $ORDVER [regime=OVERLAP-only (A-BG52); overlap runs=$NOVERLAP]"
      else
        emit "VSWEEP discrimination: NO overlap runs — discriminator not applicable [regime declared, A-BG52]"
      fi
      emit "VSWEEP PASS: calibrations byte-reproduced, spans judged (A-SK58 mapping), predictions judged against the ex-ante header (A-BB59)"
    fi
  fi
else
  emit "VSWEEP SKIPPED: upstream not clean (KS-SK-88-2)"; bf=$((bf+1))
fi
flushb

# --- Block VUCLOG: NESSUNA fase armata in m89 (DICHIARATO) -------------------
# A-DS45 consumata in m88 (VUCLOG PASS su log di produzione); il positivo
# >=1-coppia vive in F16b (battery, ARMATO). Nessun canale
# PHPR_UNIT_CACHE_LOG in questa campagna => KS-DS-90-1 non innescabile.
emit "VUCLOG DECLARED-ABSENT: no armed-log phase in measure89 BY DESIGN (A-DS45 consumed in m88; >=1-pair positive lives in F16b)"
bf=0
flushb

LEDGER="$OUT/m89.campaign.ledger"
# A-AH51/KS-AH-90-2: the verdict rows carry judge_sha — the placeholder
# `campaign_sha=judge` (semantic reuse) is BANNED.
if [ "$FAILS" = 0 ]; then
  say "== VERDICT89 PASS (git=$D_GIT attempt=$ATT, generation g$G, judge_sha=$JUDGE_SHA) =="
  echo "attempt_epoch=$(date +%s) git=$GIT_REV judge_sha=$JUDGE_SHA attempt=$ATT phase=verdict esito=PASS generation=g$G verdict_file=wp89-harness/verdict89.a$ATT.g$G.out" >> "$LEDGER"
  exit 0
else
  say "== VERDICT89 FAIL($FAILS) =="
  echo "attempt_epoch=$(date +%s) git=$GIT_REV judge_sha=$JUDGE_SHA attempt=$ATT phase=verdict esito=FAIL fails=$FAILS generation=g$G verdict_file=wp89-harness/verdict89.a$ATT.g$G.out" >> "$LEDGER"
  exit 1
fi
