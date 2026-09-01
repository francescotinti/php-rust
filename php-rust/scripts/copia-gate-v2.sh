#!/bin/bash
# copia-gate.sh — v2 (az.rev.1 S-166): gate MECCANICO per i copioni derivati.
# Cura la classe d'incidente S-166 (#1 + 2 recidive): le sostituzioni OMESSE
# nei copioni generati (path di scrittura pair164, VERD vecchio, etichette
# «pin s165» con hash s166) non si vedono a occhio — si grep-ano.
# USO: copia-gate.sh <base> <derivato> <manifest_out> <sess_attesa: sNNN>
#                    [allow_regex]
#   1) diff INTERO base→derivato nel manifest;
#   2) NOMI DI SCRITTURA nel derivato (VERD=/RC=/RCF=/LOG=/OUT=/DONE=/
#      *.done/*.rc/*.log): ogni riga con tag sessione/tentativo DIVERSO
#      dall'atteso e fuori allow ⇒ RESIDUO;
#   3) ETICHETTE nel derivato: ogni occorrenza s1XX / t1X / pair1XX / wp1XX
#      diversa dall'atteso e fuori allow ⇒ RESIDUO (con numero di riga);
#   esito appeso al manifest; rc: 0=pulito · 3=residui (elencati — o si
#   correggono o si dichiarano UNO A UNO nell'allow con motivazione).
# NOTA: l'allow è per i riferimenti LEGITTIMI (provenienza della copia,
# storiche mediane t1..tN, path read-only di harness precedenti).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
BASE="${1:?uso: copia-gate.sh <base> <derivato> <manifest> <sess_attesa> [allow_regex]}"
DER="${2:?derivato}"
MAN="${3:?manifest}"
SESS="${4:?sess attesa (es. s167)}"
ALLOW="${5:-__nessun_allow__}"
NUM="${SESS#s}"
SESSFAM="s${NUM}|pair${NUM}|wp${NUM}-harness|t[0-9]+"  # famiglia della sessione attesa (tentativi inclusi)
[ -s "$BASE" ] || { echo "rc=7 base mancante: $BASE"; exit 7; }
[ -s "$DER" ]  || { echo "rc=7 derivato mancante: $DER"; exit 7; }
diff "$BASE" "$DER" > "$MAN"
RES="$MAN.residui.$$"; : > "$RES"
# (2) nomi di scrittura con tag stantio
grep -nE '(^|[^A-Za-z])(VERD=|RC=|RCF=|LOG=|OUT=|DONE=)|\.done|\.rc([^a-z]|$)|\.log' "$DER" \
 | grep -E 's1[0-9]{2}|t1[0-9]|pair1[0-9]{2}|wp1[0-9]{2}' \
 | grep -v -E "${SESSFAM}|$ALLOW" >> "$RES" || true
# (3) etichette sessione/tentativo/harness stantie: righe INTERE (l'estrazione
# -o del token nudo impediva all'allow di matchare il CONTESTO — fix S-167)
grep -nE 's1[0-9]{2}|pair1[0-9]{2}|wp1[0-9]{2}-harness' "$DER" \
 | grep -v -E "${SESSFAM}|$ALLOW" | sort -u >> "$RES" || true
{
echo ""
echo "== copia-gate v2 (az.rev.1 S-166) =="
echo "base=$BASE derivato=$DER sess_attesa=$SESS allow=[$ALLOW]"
sort -u "$RES" -o "$RES"
if [ -s "$RES" ]; then
  echo "RESIDUI SOSPETTI ($(wc -l < "$RES" | tr -d ' ')) — correggere o dichiarare nell'allow UNO A UNO:"
  cat "$RES"
  echo "esito: rc=3"
else
  echo "nessun residuo fuori allow — esito: rc=0"
fi
} >> "$MAN"
if [ -s "$RES" ]; then rm -f "$RES"; echo "rc=3 residui nel manifest $MAN"; exit 3; fi
rm -f "$RES"; echo "rc=0 pulito (manifest $MAN)"; exit 0
