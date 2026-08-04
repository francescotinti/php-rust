#!/bin/bash
# check-liveness-fixtures.sh — A-SK-97-3 (Klabnik, Concilio WP-97): gli smoke
# di S-95.0 erano MANUALI e non committati, quindi il negativo che aveva morso
# era gia' irripetibile a macchina. «Un controllo che non puo' piu' mordere non
# e' un controllo, e' un ricordo.» Qui i conteggi attesi sono ESATTI e il
# controllo positivo e' esplicito: se una fixture smette di mordere, questo
# script lo dice invece di tacere.
#
# Uso:  check-liveness-fixtures.sh <phpr-censimento> [phpr-pre-fix]
# Il secondo argomento e' facoltativo: se c'e', si verifica anche il MORSO del
# fix A-TH-97-1 (i contatori DEVONO cambiare fra pre e post, e l'output del
# programma DEVE restare identico — la fase e' di sola misura).
#
# COME SI RICOSTRUISCE IL BINARIO PRE-FIX (un controllo la cui riproduzione non
# e' scritta e' un controllo che scade). Il fix di liveness e' atterrato nel
# commit «S-96.0 A-ZV2 passo 1+2»; il commit PRECEDENTE (l'apparato
# A-SK-93..97) e' quindi lo stato pre-fix:
#
#   git worktree add --detach /tmp/phpr-prefix <commit-apparato-A-SK-93..97>
#   cd /tmp/phpr-prefix/php-rust
#   cargo build --release --features zval-census \
#       --target-dir "/Volumes/Extreme Pro/Claude/phpr-pre-target"
#
# Il binario cosi' ottenuto in S-96.0 aveva sha e318fbfc248a8e35, e il post-fix
# 3e0e861c5fdbcb9b. Le due sha NON sono un contratto (dipendono dal toolchain e
# da tutto cio' che sta nel mezzo): il contratto e' che i contatori di t4
# DIFFERISCANO. Se un giorno coincidono, questo script FALLISCE, ed e' quello
# che deve fare.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
FIX="$H/fixtures"
POST="${1:-/Volumes/Extreme Pro/Claude/phpr-census-target/release/phpr}"
PRE="${2:-}"
ORACLE="/opt/homebrew/opt/php/bin/php"
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
rc=0

# Estrae un contatore dalla riga di censimento di un run.
count() { # <binario> <fixture> <chiave>
  local raw="$TMP/c.raw"; rm -f "$raw"
  PHPR_ZVAL_CENSUS="$raw" "$1" "$FIX/$2" > "$TMP/o.txt" 2>&1
  grep '^zvalcensus ' "$raw" | sed -n "s/.*[[:space:]]$3=\([0-9]*\).*/\1/p" | head -1
}

# ---- attesi ESATTI, post-fix (A-SK-97-3) -----------------------------------
# formato: fixture|chiave|valore atteso|che cosa prova
EXPECT='
t2-backedge.php|would_take|4|lo slot vivo sul BACK-EDGE non e'"'"'e un ultimo uso: dei 4 siti eseguiti nessuno in piu'"'"' viene contato
t2-backedge.php|sites_movable|2|i due siti movibili sono fuori dal ciclo, non la lettura di $acc dentro
t3-renounce.php|would_take|4|le letture movibili ci sono...
t3-renounce.php|would_take_safe|2|...ma compact/global/use(&$x) ne tolgono meta'"'"' dal perimetro F2
t4-first-op-def.php|would_take|4|A-TH-97-1: la def non uccide piu'"'"' sul contributo dell'"'"'arco eccezionale
t4-first-op-def.php|would_take_safe|0|nessun sito sopravvive: gli slot sono anche CONDIVISI (unset/&/global)
t4-first-op-def.php|sites_movable|6|due siti in meno rispetto al giudizio insound
'

echo "== fixture attese (post-fix), binario $(shasum -a 256 "$POST" | cut -c1-16)"
while IFS='|' read -r f k want why; do
  [ -n "${f:-}" ] || continue
  got=$(count "$POST" "$f" "$k")
  if [ "${got:-}" != "$want" ]; then
    echo "FAIL $f: $k=$got, atteso $want — $why"; rc=1
  else
    echo "  ok  $f: $k=$want ($why)"
  fi
done <<< "$EXPECT"

# ---- parita' con l'oracle: una fixture che DIVERGE non misura phpr ----------
echo "== parita' con l'oracle (una fixture che diverge non sta misurando phpr)"
for f in t1-hoare-exc.php t2-backedge.php t3-renounce.php t4-first-op-def.php; do
  "$POST" "$FIX/$f" > "$TMP/phpr.txt" 2>&1
  "$ORACLE" "$FIX/$f" > "$TMP/ora.txt" 2>&1
  if cmp -s "$TMP/phpr.txt" "$TMP/ora.txt"; then echo "  ok  $f byte-identico"
  else echo "FAIL $f: phpr diverge dall'oracle"; diff "$TMP/ora.txt" "$TMP/phpr.txt" | head -6; rc=1; fi
done

# ---- il MORSO del fix (solo se e' stato dato un binario pre-fix) -----------
if [ -n "$PRE" ] && [ -x "$PRE" ]; then
  echo "== morso del fix A-TH-97-1, binario pre-fix $(shasum -a 256 "$PRE" | cut -c1-16)"
  bit=0
  for k in would_take would_take_rc sites_movable; do
    a=$(count "$PRE" t4-first-op-def.php "$k"); b=$(count "$POST" t4-first-op-def.php "$k")
    if [ "$a" = "$b" ]; then echo "  (nessuna differenza su $k: $a)"
    else echo "  ok  $k: pre=$a -> post=$b (il fix cambia il giudizio)"; bit=1; fi
  done
  if [ "$bit" != 1 ]; then
    echo "FAIL: il fix A-TH-97-1 NON cambia nessun contatore su t4 — o la fixture ha smesso di mordere, o il fix e' inerte. In entrambi i casi non e' provato."; rc=1
  fi
  "$PRE"  "$FIX/t4-first-op-def.php" > "$TMP/pre.txt"  2>&1
  "$POST" "$FIX/t4-first-op-def.php" > "$TMP/post.txt" 2>&1
  if cmp -s "$TMP/pre.txt" "$TMP/post.txt"; then echo "  ok  output pre==post (la fase e' di SOLA MISURA)"
  else echo "FAIL: l'output del programma e' cambiato — F1/F2 non sono piu' di sola misura"; rc=1; fi
else
  echo "== morso del fix: SALTATO (nessun binario pre-fix passato come 2o argomento)"
fi

[ "$rc" = 0 ] && echo "CHECK-LIVENESS-FIXTURES PASS" || echo "CHECK-LIVENESS-FIXTURES FAIL"
exit $rc
