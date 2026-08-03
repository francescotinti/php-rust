#!/bin/bash
# check-campaign-v2.sh — A-AH63 (Concilio WP-93, Hejlsberg: la delibera
# ledger v2 era applicata a metà — «la grammatica campaign v2 non ha alcun
# dente pre-nascita»). Dente PRE-NASCITA della grammatica campaign v2
# (design91-ledger.md: A-AH60 campaign_sha sulle verdict, A-BG58 reason=
# autosufficiente, A-PP-65 authorize del giudice), committato PRIMA di
# qualunque m91: una riga m9[1-9]+ fuori grammatica ⇒ campagna VOID
# (KS-AH-93-2) alla PRIMA consumazione.
#
# PURO: giudica SOLO il file passato come argomento — la provenienza
# (blob a HEAD) è responsabilità del CHIAMANTE (gate-measure-cifre
# estrae DA HEAD sia il ledger sia QUESTO script prima di eseguirlo:
# un checker working-tree patchato non giudica nulla).
#
# Regole v2 (righe della campagna, ordine del file):
#   R1: esattamente UNA riga phase=start, con campaign_sha=<16hex> e
#       judge_sha=<16hex> (il pin del giudice di partenza).
#   R2: ogni riga phase=verdict porta campaign_sha= IDENTICO allo start
#       (A-AH60), generation=g<N>, esito∈{PASS,FAIL} e reason= non vuoto.
#   R3: ogni riga con supersede_of=g<N> porta
#       reason=requalify:<blocco>:<old>-><new> (A-BG58: il ledger da solo
#       rigenera la storia g(n)→g(n+1)).
#   R4: una verdict con judge_sha ≠ start esige una riga phase=authorize
#       judge_sha=<nuovo> PRIMA di essa (A-PP-65/KS-PP-92-2).
# exit 0 = conforme; exit 1 = violazione (righe nominate); exit 2 = uso.
# Scope: v2 governa m91+ (design91-ledger §Scope); i ledger storici
# m87..m90 restano grammar v1 e NON passano da qui (il gate li legge
# come committati — la storia non si rigiudica).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u

if [ "${1:-}" = "--selftest" ]; then
  T=$(mktemp -d)
  cat > "$T/good.ledger" <<'EOF'
attempt_epoch=1 git=abc1234 campaign_sha=aa37730264fcf2fe attempt=1 phase=start judge_sha=1b1e9e96f5019bc0
attempt_epoch=2 git=abc1234 campaign_sha=aa37730264fcf2fe attempt=1 phase=slope.w4.r1 raw=x esito=ok nreq=400
attempt_epoch=3 git=abc1234 campaign_sha=aa37730264fcf2fe attempt=1 phase=verdict generation=g1 judge_sha=1b1e9e96f5019bc0 esito=FAIL reason=see-verdict-file supersede_of=none
attempt_epoch=4 git=abc1234 campaign_sha=aa37730264fcf2fe attempt=1 phase=authorize judge_sha=97a8eff0d9783ee4 gG=g2-requalify
attempt_epoch=5 git=abc1234 campaign_sha=aa37730264fcf2fe attempt=1 phase=verdict generation=g2 judge_sha=97a8eff0d9783ee4 esito=PASS reason=requalify:VCKPT:want-1/3-to-1/1 supersede_of=g1
EOF
  if ! bash "$0" "$T/good.ledger" >/dev/null; then
    echo "SELFTEST FAIL: ledger v2 CONFORME rifiutato (dente vacuo al contrario)"; rm -rf "$T"; exit 1
  fi
  # bad1 — verdict in forma v1 (m89: niente campaign_sha né reason sulla verdict)
  cat > "$T/bad1.ledger" <<'EOF'
attempt_epoch=1 git=abc1234 campaign_sha=aa37730264fcf2fe attempt=1 phase=start judge_sha=1b1e9e96f5019bc0
attempt_epoch=2 git=abc1234 judge_sha=1b1e9e96f5019bc0 attempt=1 phase=verdict esito=PASS generation=g1 verdict_file=x.out
EOF
  if bash "$0" "$T/bad1.ledger" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: verdict v1-style (senza campaign_sha/reason) NON morsa (A-AH60)"; rm -rf "$T"; exit 1
  fi
  # bad2 — supersede senza reason=requalify
  cat > "$T/bad2.ledger" <<'EOF'
attempt_epoch=1 git=abc1234 campaign_sha=aa37730264fcf2fe attempt=1 phase=start judge_sha=1b1e9e96f5019bc0
attempt_epoch=2 git=abc1234 campaign_sha=aa37730264fcf2fe attempt=1 phase=verdict generation=g2 judge_sha=1b1e9e96f5019bc0 esito=PASS reason=all-clean supersede_of=g1
EOF
  if bash "$0" "$T/bad2.ledger" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: supersede_of senza reason=requalify NON morso (A-BG58)"; rm -rf "$T"; exit 1
  fi
  # bad3 — cambio di giudice senza riga authorize (il buco g1->g2 di S-90.0)
  cat > "$T/bad3.ledger" <<'EOF'
attempt_epoch=1 git=abc1234 campaign_sha=aa37730264fcf2fe attempt=1 phase=start judge_sha=1b1e9e96f5019bc0
attempt_epoch=2 git=abc1234 campaign_sha=aa37730264fcf2fe attempt=1 phase=verdict generation=g2 judge_sha=97a8eff0d9783ee4 esito=PASS reason=requalify:VCKPT:a-to-b supersede_of=g1
EOF
  if bash "$0" "$T/bad3.ledger" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: judge_sha nuovo senza phase=authorize NON morso (A-PP-65/KS-PP-92-2)"; rm -rf "$T"; exit 1
  fi
  # bad4 — start assente (file vuoto/malformato = fail-closed)
  : > "$T/bad4.ledger"
  if bash "$0" "$T/bad4.ledger" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: ledger senza phase=start NON morso (R1 fail-closed)"; rm -rf "$T"; exit 1
  fi
  rm -rf "$T"
  echo "SELFTEST PASS check-campaign-v2: R1 start-unico + R2 verdict v2 (A-AH60) + R3 requalify (A-BG58) + R4 authorize (A-PP-65) mordono; il conforme passa"
  exit 0
fi

L="${1:-}"
[ -n "$L" ] && [ -f "$L" ] || { echo "usage: $0 <campaign.ledger>|--selftest"; exit 2; }
FAILS=0
bad() { echo "V2-FAIL($1): $2"; FAILS=$((FAILS+1)); }

C=$(tr -d '\0' < "$L")
NSTART=$(printf '%s\n' "$C" | grep -c 'phase=start' || true)
# guardia di numericità (lezione S-91.0: un conteggio non-numerico è un
# giudice rotto)
case "$NSTART" in ''|*[!0-9]*) echo "V2-FAIL(R1): conteggio start NON numerico ('$NSTART') — checker VOID"; exit 1;; esac
[ "$NSTART" = 1 ] || bad R1 "righe phase=start = $NSTART, attese esattamente 1"
SLINE=$(printf '%s\n' "$C" | grep 'phase=start' | head -1)
CSHA=$(printf '%s' "$SLINE" | sed -n 's/.*campaign_sha=\([0-9a-f]\{16\}\).*/\1/p')
JSHA0=$(printf '%s' "$SLINE" | sed -n 's/.*judge_sha=\([0-9a-f]\{16\}\).*/\1/p')
[ -n "$CSHA" ] || bad R1 "riga start senza campaign_sha=<16hex>"
[ -n "$JSHA0" ] || bad R1 "riga start senza judge_sha=<16hex> (pin del giudice)"

AUTHJ=" ${JSHA0:-} "
while IFS= read -r row; do
  [ -n "$row" ] || continue
  case "$row" in
    *phase=authorize*)
      j=$(printf '%s' "$row" | sed -n 's/.*judge_sha=\([0-9a-f]\{16\}\).*/\1/p')
      if [ -n "$j" ]; then AUTHJ="$AUTHJ$j "; else bad R4 "riga authorize senza judge_sha=<16hex>: $row"; fi
      ;;
    *phase=verdict*)
      printf '%s' "$row" | grep -q "campaign_sha=${CSHA:-__nocsha__}" \
        || bad R2 "verdict senza campaign_sha= dello start (A-AH60): $row"
      printf '%s' "$row" | grep -qE 'generation=g[0-9]+' \
        || bad R2 "verdict senza generation=g<N>: $row"
      printf '%s' "$row" | grep -qE 'esito=(PASS|FAIL)( |$)' \
        || bad R2 "verdict con esito fuori {PASS,FAIL}: $row"
      printf '%s' "$row" | grep -qE 'reason=[^ ]+' \
        || bad R2 "verdict senza reason= (A-BG58): $row"
      j=$(printf '%s' "$row" | sed -n 's/.*judge_sha=\([0-9a-f]\{16\}\).*/\1/p')
      if [ -n "$j" ]; then
        case "$AUTHJ" in
          *" $j "*) : ;;
          *) bad R4 "verdict con judge_sha=$j MAI autorizzato da una riga phase=authorize precedente (A-PP-65/KS-PP-92-2): $row";;
        esac
      fi
      ;;
  esac
  case "$row" in
    *supersede_of=g[0-9]*)
      printf '%s' "$row" | grep -qE 'reason=requalify:[^ :]+:[^ ]+' \
        || bad R3 "supersede_of senza reason=requalify:<blocco>:<old>-><new> (A-BG58): $row"
      ;;
  esac
done <<EOF
$C
EOF

if [ "$FAILS" = 0 ]; then
  echo "V2-OK: $L conforme alla grammatica campaign v2 (A-AH60/A-BG58/A-PP-65)"
  exit 0
else
  echo "V2-VIOLAZIONI: $FAILS — campagna VOID alla consumazione (A-AH63/KS-AH-93-2)"
  exit 1
fi
