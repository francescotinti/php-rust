#!/bin/bash
# copia-gate-v3.sh — v3 PER TOKEN (az.rev. S-169 #3): gate MECCANICO per i
# copioni derivati. Cura la classe d'incidente S-169 (4 casi): il v2 assolveva
# la RIGA intera se conteneva il token della sessione attesa, quindi una riga
# con `s169` E `wp168-harness` (etichetta stantia accanto a quella corrente)
# passava. Il v3 giudica OGNI TOKEN: in ogni riga del derivato si cancellano i
# token della famiglia attesa e si guarda se resta un token sessione/harness
# stantio; l'allow (regex) si applica alla riga ORIGINALE (contesto).
# USO: copia-gate-v3.sh <base> <derivato> <manifest_out> <sess_attesa: sNNN>
#                       [allow_regex]
#   1) diff INTERO base→derivato nel manifest;
#   2) per TOKEN: s1XX · pair1XX · wp1XX-harness · t1X fuori famiglia e fuori
#      allow ⇒ RESIDUO (riga:numero, token evidenziato);
#   3) `[ -s ]` su base e derivato (file vuoto = rc 7);
#   esito appeso al manifest; rc: 0=pulito · 3=residui · 7=file mancante.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
BASE="${1:?uso: copia-gate-v3.sh <base> <derivato> <manifest> <sess_attesa> [allow_regex]}"
DER="${2:?derivato}"
MAN="${3:?manifest}"
SESS="${4:?sess attesa (es. s170)}"
ALLOW="${5:-__nessun_allow__}"
NUM="${SESS#s}"
[ -s "$BASE" ] || { echo "rc=7 base mancante o vuota: $BASE"; exit 7; }
[ -s "$DER" ]  || { echo "rc=7 derivato mancante o vuoto: $DER"; exit 7; }
diff "$BASE" "$DER" > "$MAN"
RES="$MAN.residui.$$"; : > "$RES"
python3 - "$DER" "$NUM" "$ALLOW" > "$RES" <<'PY'
import re, sys
der, num, allow = sys.argv[1], sys.argv[2], sys.argv[3]
fam = re.compile(r'(?<![A-Za-z0-9])(s%s|pair%s|wp%s-harness|t[0-9]+)(?![0-9])' % (num, num, num))
stale = re.compile(r'(?<![A-Za-z0-9])(s1[0-9]{2}|pair1[0-9]{2}|wp1[0-9]{2}-harness|t1[0-9])(?![0-9])')
allow_re = re.compile(allow)
for n, line in enumerate(open(der, encoding='utf-8', errors='replace'), 1):
    line = line.rstrip('\n')
    if allow_re.search(line):
        continue
    stripped = fam.sub('', line)
    toks = sorted(set(m.group(1) for m in stale.finditer(stripped)))
    if toks:
        print("%d:[%s] %s" % (n, ",".join(toks), line))
PY
{
echo ""
echo "== copia-gate v3 PER TOKEN (az.rev. S-169) =="
echo "base=$BASE derivato=$DER sess_attesa=$SESS allow=[$ALLOW]"
if [ -s "$RES" ]; then
  echo "RESIDUI PER TOKEN ($(wc -l < "$RES" | tr -d ' ')) — correggere o dichiarare nell'allow UNO A UNO:"
  cat "$RES"
  echo "esito: rc=3"
else
  echo "nessun token stantio fuori allow — esito: rc=0"
fi
} >> "$MAN"
if [ -s "$RES" ]; then rm -f "$RES"; echo "rc=3 residui nel manifest $MAN"; exit 3; fi
rm -f "$RES"; echo "rc=0 pulito (manifest $MAN)"; exit 0
