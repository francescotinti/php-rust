#!/bin/bash
# ds35-verify4.sh — A-DS53/54/55 + A-DS58 (Concilio WP-93, S-92.0 p5) (Concilio WP-92, ordine S-91.0
# p6 fase 0): PIN v3 delle fixture LSP — v1 (18, wp89) + v2 (19, wp90) +
# v3 (8, wp91: i SETTE buchi oracle-morsi di Stogov Q1 + il positivo DNF)
# — committate PRIMA del codice (KS-DS-92-2: merge di A-DS51 senza le
# fixture A-DS53 per NOME nel gate = REJECT).
#
# FORMATO PIN v3 (A-DS54 — KS-DS-92-1: il comparatore a MARCATORI e'
# BANDITO): un programma PHP legale puo' stampare le righe-marcatore del
# v2 («--stdout» in mezzo allo stdout spezzava il parsing per
# costruzione). Qui ogni canale e' LENGTH-PREFIXED:
#   --branch=<label> exit=<rc> stdout_bytes=<N> stderr_bytes=<M>
#   <N byte esatti di stdout, raw>
#   <M byte esatti di stderr, raw>
# Il parser legge N e M byte per OFFSET e trova l'header successivo alla
# posizione esatta — mai una scansione di marcatori dentro il payload.
#
# A-DS55 (per NOME): il bersaglio byte-fedele di t4_hoisted_with_output
# e' il braccio PERSIST (plain = `pre|` PRIMA del fatal; persist = fatal
# PRE-output: phpr col lowering hoisted fatalera' pre-output). A-DS56:
# per v15 prevale «mai skip silenzioso» (§3.3-quinquies) — fatal al bind
# hoisted; i bind dinamici li giudica il gate ORM/hk.
#
# CONTRATTO (A-DS48 invariato) + A-DS52 PERSIST_FLAGS unico + binchk
# same-vector (A-DS47). Dichiarato per NOME: phpr NON modella la
# log-copy stderr dell'oracle — stderr phpr atteso VUOTO (catalogo
# §3.3-quinquies).
set -eu

REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
FIX1="$REPO/wp89-harness/fixtures-ds35"
FIX2="$REPO/wp90-harness/fixtures-ds35-v2"
FIX3="$REPO/wp91-harness/fixtures-ds35-v3"
FIX4="$REPO/wp92-harness/fixtures-ds35-v4"
OUT="$REPO/wp92-harness/ds35-verify4.out"
ORACLE="/opt/homebrew/opt/php/bin/php"
PHPR="$HOME/Claude/php-rust-output/release/phpr"
FC=$(mktemp -d /tmp/ds35v4-filecache.XXXXXX)

PERSIST_FLAGS=(-d opcache.enable_cli=1 -d opcache.file_cache_only=1 -d "opcache.file_cache=$FC")

: > "$OUT"
say() { echo "$@" | tee -a "$OUT"; }

say "== ds35-verify4 — oracle $($ORACLE -v | head -1 | awk '{print $2}') · phpr $(shasum -a 256 "$PHPR" | cut -c1-16) · git $(cd "$REPO" && git rev-parse --short HEAD) =="
say "persist flags (A-DS52, single vector): opcache.enable_cli=1 opcache.file_cache_only=1 opcache.file_cache=<fresh-dir>"
say "pin format (A-DS54/KS-DS-92-1): LENGTH-PREFIXED channels — stdout_bytes/stderr_bytes on the header, payload read by OFFSET; marker-scanning comparators BANNED"
say "t4 target arm (A-DS55, per NOME): PERSIST (plain diverges: pre| before the fatal)"
say "declared by NAME: phpr does NOT model the oracle stderr log-copy — phpr stderr expected EMPTY (catalogo §3.3-quinquies)"
say ""

run_branch() { # <label> <cmd...> — length-prefixed separate channels
  local label="$1"; shift
  local rc=0 so se nb mb
  so=$(mktemp); se=$(mktemp)
  "$@" > "$so" 2> "$se" || rc=$?
  nb=$(wc -c < "$so" | tr -d ' ')
  mb=$(wc -c < "$se" | tr -d ' ')
  {
    echo "--branch=$label exit=$rc stdout_bytes=$nb stderr_bytes=$mb"
    cat "$so"
    cat "$se"
  } >> "$OUT"
  rm -f "$so" "$se"
}

binchk() { # A-DS47 + A-DS52: same-vector probe when the fixture is compile-fatal
  local NBIN
  NBIN=$(find "$FC" -name '*.bin' -type f | wc -l | tr -d ' ')
  if [ "$NBIN" -ge 1 ]; then
    say "  binchk : ok nbin=$NBIN (A-DS47 persist ANCORATO; A-DS52 single vector)"
  else
    local PFC PSD NPRB
    PFC=$(mktemp -d /tmp/ds35v4-probecache.XXXXXX)
    PSD=$(mktemp -d /tmp/ds35v4-probesrc.XXXXXX)
    echo '<?php echo "probe";' > "$PSD/probe.php"
    touch -t 202601010000 "$PSD/probe.php"   # file_update_protection (S-89.0)
    local PROBE_FLAGS=(-d opcache.enable_cli=1 -d opcache.file_cache_only=1 -d "opcache.file_cache=$PFC")
    "$ORACLE" "${PROBE_FLAGS[@]}" "$PSD/probe.php" >/dev/null 2>&1 || true
    NPRB=$(find "$PFC" -name '*.bin' -type f | wc -l | tr -d ' ')
    rm -rf "$PFC" "$PSD"
    if [ "$NPRB" -ge 1 ]; then
      say "  binchk : ok-via-probe nbin=0 nprobe=$NPRB (compile-fatal fixture: .bin N/A; arm NOT crippled)"
    else
      say "  binchk : FAIL nbin=0 nprobe=0 — crippled persist arm (KS-DS-90-2): observable UNANCHORED"
      exit 1
    fi
  fi
}

judge_set() { # <dir> <set-label> <fixtures...>
  local dir="$1" setl="$2"; shift 2
  for f in "$@"; do
    say "fixture=$f.php set=$setl"
    run_branch "plain" "$ORACLE" "$dir/$f.php"
    rm -rf "$FC"; mkdir -p "$FC"
    PERSIST_FLAGS=(-d opcache.enable_cli=1 -d opcache.file_cache_only=1 -d "opcache.file_cache=$FC")
    run_branch "persist" "$ORACLE" "${PERSIST_FLAGS[@]}" "$dir/$f.php"
    binchk
    run_branch "phpr" "$PHPR" "$dir/$f.php"
    say ""
  done
}

judge_set "$FIX1" v1 \
  n1_return_incompat n2_param_narrow n3_static_widen n4_union_superset \
  n5_static_mismatch n6_visibility n7_final n8_prop_type n9_trait_abstract \
  p1_covariant_return p2_param_widen p3_static_return p4_union_subset \
  p5_tentative_return p6_rtwc_suppress p7_never_covariant p8_ctor_exempt \
  p9_private_exempt

judge_set "$FIX2" v2 \
  v1_byref_widen v2_byref_narrow v3_ref_removed v4_ref_added \
  v5_required_grows v6_variadic_absorbs v7_nullable_grafia v8_mixed_narrow \
  v9_void_exact v10_static_inverse v11_readonly v12_prop_typed_on_untyped \
  v13_iface_ctor v14_abstract_ctor v15_unresolvable \
  t1_parent_after t2_conditional_executed t3_conditional_skipped \
  t4_hoisted_with_output

# A-DS53 (Stogov WP-92 Q1, tutti oracle-morsi dal vivo): multi-iface in
# conflitto (messaggio con l'IFACE colpevole I2), iface-extends-iface
# (fatal SENZA classi), enum-implements, self-vs-static (self RISOLTO nel
# messaggio: C::m(): C), hook-subtype (forma C::$x::get()), final-const,
# readonly-PROMOTED (stesso messaggio di v11, path di lowering diverso) +
# il POSITIVO di regressione DNF (A&B)|string -> string LEGALE.
judge_set "$FIX3" v3 \
  w1_multi_iface w2_iface_extends w3_enum_implements w4_self_vs_static \
  w5_hook_subtype w6_final_const w7_readonly_promoted w8_dnf_positive

# A-DS58 (Concilio WP-93, Stogov Q1 — i quattro buchi residui + x5/x6):
# x1 famiglia NUOVA enum-abstract («Enum method E::m() must not be
# abstract»), x2 famiglia NUOVA hook-set (giudicato contro il TIPO DELLA
# PROPERTY, senza forma «Declaration of»), x3 negativo intersection
# return-widen (A -> A&B), x4 by-ref hook get (prefisso «& » CON SPAZIO,
# fuori dal vincolo w5), x5 classname UTF-8 (regressione anti-wc -m: il
# pin length-prefixed conta BYTE, KS-DS-93-3), x6 vincolo iface-ctor
# PROPAGATO al nipote (due livelli sotto l'implements).
judge_set "$FIX4" v4 \
  x1_enum_abstract x2_hook_set_param x3_intersection_return_widen \
  x4_byref_hook_get x5_utf8_classname x6_iface_ctor_grandchild

rm -rf "$FC"
say "== ds35-verify4 DONE (51 fixtures = 18 v1 + 19 v2 + 8 v3 + 6 v4, length-prefixed channel pins) =="
