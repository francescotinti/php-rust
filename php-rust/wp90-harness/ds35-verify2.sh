#!/bin/bash
# ds35-verify2.sh — A-DS49/A-DS50/A-DS52 (Concilio WP-91, ordine S-90.0
# p6): PIN v2 delle fixture LSP — fixture v1 (18, wp89) + v2 (19, wp90)
# — PRIMA del codice (KS-DS-91-2: il gate di merge sono le fixture v2
# per NOME, non solo le 18).
#
# CONTRATTO EMENDATO (A-DS48, refutazione oracle-viva Stogov WP-91):
#   r2 by-ref: ESATTA e' la REF-NESS (togliere O aggiungere & = fatal in
#   entrambe le direzioni); il TIPO resta contravariante (widening
#   LEGALE: int&$x -> int|string&$x alive). I messaggi portano le union
#   nell'ordine canonico Zend (es. «P::m(string|int &$x)»).
#
# PIN v2 (A-DS50, supersede del pin `2>&1|head -3` — KS-DS-91-3: quel
# bersaglio era byte-IMPOSSIBILE: teneva solo il blocco stderr che phpr
# NON emette): canali SEPARATI e INTEGRALI — stdout FULL, stderr FULL,
# exit code, per braccio. DECISIONE PER NOME (A-DS50): phpr NON modella
# la log-copy stderr dell'oracle («PHP Fatal error: ...», doppio
# spazio) — divergenza a CATALOGO (§3.3-quinquies); il bersaglio
# byte-fedele dell'implementazione e' lo STDOUT integrale dell'oracle;
# lo stderr phpr atteso e' VUOTO.
#
# A-DS52: PERSIST_FLAGS UNICO condiviso probe/braccio (la recidiva
# flag-drift WP-89 muore qui: una flag caduta cade per ENTRAMBI e il
# binchk la morde); header con le TRE flag per NOME.
set -eu

REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
FIX1="$REPO/wp89-harness/fixtures-ds35"
FIX2="$REPO/wp90-harness/fixtures-ds35-v2"
OUT="$REPO/wp90-harness/ds35-verify2.out"
ORACLE="/opt/homebrew/opt/php/bin/php"
PHPR="$HOME/Claude/php-rust-output/release/phpr"
FC=$(mktemp -d /tmp/ds35v2-filecache.XXXXXX)

# A-DS52: the ONE flag vector — probe and braccio consume THIS, never a
# retyped literal.
PERSIST_FLAGS=(-d opcache.enable_cli=1 -d opcache.file_cache_only=1 -d "opcache.file_cache=$FC")

: > "$OUT"
say() { echo "$@" | tee -a "$OUT"; }

say "== ds35-verify2 — oracle $($ORACLE -v | head -1 | awk '{print $2}') · phpr $(shasum -a 256 "$PHPR" | cut -c1-16) · git $(cd "$REPO" && git rev-parse --short HEAD) =="
say "persist flags (A-DS52, single vector): opcache.enable_cli=1 opcache.file_cache_only=1 opcache.file_cache=<fresh-dir>"
say "pin channels (A-DS50): stdout FULL + stderr FULL + exit, SEPARATE — never 2>&1, never truncated (KS-DS-91-3)"
say "declared by NAME: phpr does NOT model the oracle stderr log-copy — phpr stderr expected EMPTY (catalogo §3.3-quinquies)"
say ""

run_branch() { # <label> <cmd...> — full separate channels into the OUT
  local label="$1"; shift
  local rc=0 so se
  so=$(mktemp); se=$(mktemp)
  "$@" > "$so" 2> "$se" || rc=$?
  {
    echo "--branch=$label exit=$rc"
    echo "--stdout"
    cat "$so"
    echo "--stderr"
    cat "$se"
    echo "--end"
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
    PFC=$(mktemp -d /tmp/ds35v2-probecache.XXXXXX)
    PSD=$(mktemp -d /tmp/ds35v2-probesrc.XXXXXX)
    echo '<?php echo "probe";' > "$PSD/probe.php"
    touch -t 202601010000 "$PSD/probe.php"   # file_update_protection (S-89.0)
    local SAVED_FC="$FC"
    FC="$PFC"
    local PROBE_FLAGS=(-d opcache.enable_cli=1 -d opcache.file_cache_only=1 -d "opcache.file_cache=$PFC")
    "$ORACLE" "${PROBE_FLAGS[@]}" "$PSD/probe.php" >/dev/null 2>&1 || true
    FC="$SAVED_FC"
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

rm -rf "$FC"
say "== ds35-verify2 DONE (37 fixtures, separate-channel pins) =="
