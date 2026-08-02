#!/bin/bash
# ds35-verify.sh — A-DS35 fase 1 (Concilio WP-90, ordine S-89.0 p7): PIN
# oracle-vivi delle fixture LSP (contratto A-DS41+A-DS44+A-DS46) — le
# fixture sono committate PRIMA del codice; questo .out e il bersaglio
# byte-fedele dell implementazione. Tre bracci per fixture:
#   plain   = oracle 8.5.7, opcache CARICATO ma enable_cli=Off (default brew)
#   persist = oracle 8.5.7, opcache.enable_cli=1 + file_cache_only=1 (branch PERSIST;
#             run1 basta — l'inversione appare già alla prima esecuzione, Stogov WP-88 Q2)
#   phpr    = binario di parità corrente
# Output: wp89-harness/ds35-verify.out (stdout troncato a 3 righe + exit code per braccio).
# KS-DS-88-2: queste fixture committate sono l'ANCORA delle entry §3.3-ter/quater.
# A-DS47 (Concilio WP-90, Stogov): il braccio persist dichiara SEMPRE le TRE
# flag per NOME (opcache.enable_cli=1 + opcache.file_cache_only=1 +
# opcache.file_cache=<dir>) E verifica che il run1 abbia SCRITTO >=1 file
# `.bin` nel file_cache — il braccio-monco (enable_cli caduta) è SILENZIOSO
# e mima plain (recidiva Stogov WP-89). Senza .bin-check o senza le tre
# flag esplicite l'observable è UNANCHORED (KS-DS-90-2). Fail-closed:
# .bin assente => exit 1.
set -eu

REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
FIX="$REPO/wp89-harness/fixtures-ds35"
OUT="$REPO/wp89-harness/ds35-verify.out"
ORACLE="/opt/homebrew/opt/php/bin/php"
PHPR="$HOME/Claude/php-rust-output/release/phpr"
FC=$(mktemp -d /tmp/ds40-filecache.XXXXXX)

: > "$OUT"
say() { echo "$@" | tee -a "$OUT"; }

say "== ds35-verify — oracle $($ORACLE -v | head -1 | awk '{print $2}') · phpr $(shasum -a 256 "$PHPR" | cut -c1-16) · git $(cd "$REPO" && git rev-parse --short HEAD) =="
say "branches: plain=opcache-loaded/enable_cli=Off · persist=enable_cli=1+file_cache_only=1 (run1) · phpr"
say ""

run_branch() { # <label> <cmd...>
  local label="$1"; shift
  local rc=0 body
  body=$("$@" 2>&1) || rc=$?
  echo "  $label: exit=$rc stdout=$(echo "$body" | head -3 | tr '\n' '|')"
}

for f in n1_return_incompat n2_param_narrow n3_static_widen n4_union_superset n5_static_mismatch n6_visibility n7_final n8_prop_type n9_trait_abstract p1_covariant_return p2_param_widen p3_static_return p4_union_subset p5_tentative_return p6_rtwc_suppress p7_never_covariant p8_ctor_exempt p9_private_exempt; do
  say "fixture=$f.php"
  run_branch "plain  " "$ORACLE" "$FIX/$f.php" | tee -a "$OUT"
  # fresh file-cache dir per fixture: run1 of the persist pass
  rm -rf "$FC"; mkdir -p "$FC"
  run_branch "persist" "$ORACLE" -d opcache.enable_cli=1 -d opcache.file_cache_only=1 -d "opcache.file_cache=$FC" "$FIX/$f.php" | tee -a "$OUT"
  # A-DS47 .bin-check: run1 persist DEVE aver scritto il payload nel
  # file_cache; 0 file .bin = braccio-monco silenzioso => FAIL per NOME.
  # Lane dichiarata (morso dal vivo su tC_lsp_covariance): una fixture che
  # fatala a COMPILE/link time non viene persistita da opcache (nbin=0
  # LEGITTIMO) — lì il monco-check passa a un PROBE persistabile con le
  # STESSE tre flag: probe senza .bin = braccio monco => FAIL.
  NBIN=$(find "$FC" -name '*.bin' -type f | wc -l | tr -d ' ')
  if [ "$NBIN" -ge 1 ]; then
    say "  binchk : ok nbin=$NBIN (A-DS47: persist ANCORATO)"
  else
    PFC=$(mktemp -d /tmp/ds40-probecache.XXXXXX)
    PSD=$(mktemp -d /tmp/ds40-probesrc.XXXXXX)
    echo '<?php echo "probe";' > "$PSD/probe.php"
    # scoperta dal vivo S-89.0: opcache.file_update_protection (default 2s)
    # NON cachea file appena scritti — il probe va retrodatato o il check
    # monco darebbe FAIL spurio su braccio SANO.
    touch -t 202601010000 "$PSD/probe.php"
    "$ORACLE" -d opcache.enable_cli=1 -d opcache.file_cache_only=1 -d "opcache.file_cache=$PFC" "$PSD/probe.php" >/dev/null 2>&1 || true
    NPRB=$(find "$PFC" -name '*.bin' -type f | wc -l | tr -d ' ')
    rm -rf "$PFC" "$PSD"
    if [ "$NPRB" -ge 1 ]; then
      say "  binchk : ok-via-probe nbin=0 nprobe=$NPRB (fixture compile-fatal: .bin N/A; braccio NON monco)"
    else
      say "  binchk : FAIL nbin=0 nprobe=0 — braccio-persist-monco (KS-DS-90-2): observable UNANCHORED"
      exit 1
    fi
  fi
  run_branch "phpr   " "$PHPR" "$FIX/$f.php" | tee -a "$OUT"
  say ""
done
rm -rf "$FC"
say "== ds35-verify DONE =="
