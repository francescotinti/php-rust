#!/bin/bash
# s195-catena-R.sh — S-195 tetto del modello a registri (criterio s195-criterio-tetto-registri.md p.2): solo misura.
# (1) istruzioni ORM e WordPress del pin a modo acceso/spento (involucri in /private/tmp, percorso senza spazi),
# (2) censimento degli op del commit del pin nei due modi. In coda dietro la catena I. Esiti: out/catena-R.out, .rc, .done.
# Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; O="$P/wp195-harness/out"; W4="$P/wp194-harness"; W5="$P/wp195-harness"; mkdir -p "$O"
V="$O/catena-R.out"; : > "$V"; rm -f "$O/catena-R.rc" "$O/catena-R.done"; RC=0
cd "$P" || exit 2
until [ -f "$O/catena-I.done" ]; do /bin/sleep 30; done
PIN="$HOME/Claude/php-rust-output/release/phpr"; T=/private/tmp
cp "$PIN" "$T/phpr-s195-pin" || exit 2
echo "pin sha $(shasum -a 256 "$T/phpr-s195-pin" | cut -c1-16)" >> "$V"
for m in on off; do
  if [ "$m" = on ]; then v=1; else v=0; fi
  printf '#!/bin/sh\nPHPR_REG_LOWER=%s exec %s/phpr-s195-pin "$@"\n' "$v" "$T" > "$T/phpr-s195-$m"; chmod +x "$T/phpr-s195-$m"
done
[ "$(PHPR_DUMP_OPS=1 "$T/phpr-s195-on" -r '$a=1;$b=$a+2;' 2>&1 | grep -c 'Binary')" != "$(PHPR_DUMP_OPS=1 "$T/phpr-s195-off" -r '$a=1;$b=$a+2;' 2>&1 | grep -c 'Binary')" ] || true
REPS=2 python3 "$W4/s194-orm-istruzioni.py" ON=$T/phpr-s195-on OFF=$T/phpr-s195-off > "$O/catena-R-orm.log" 2>&1; r=$?
echo "istruzioni ORM on/off: rc=$r" >> "$V"; [ "$r" = 0 ] || RC=1
tail -6 "$W4/s194-orm-istruzioni.out" >> "$V"
python3 "$W4/s194-wp-istruzioni.py" ON=$T/phpr-s195-on OFF=$T/phpr-s195-off > "$O/catena-R-wp.log" 2>&1; r=$?
echo "istruzioni WordPress on/off: rc=$r" >> "$V"; [ "$r" = 0 ] || RC=1
tail -6 "$W4/s194-wp-istruzioni.out" >> "$V"
for m in on off; do
  if [ "$m" = on ]; then v=1; else v=0; fi
  PHPR_REG_LOWER=$v python3 "$W4/s194-censimento-op.py" 3b78c61c5b1d34e2acc8844abb8cdf279aa5f269 orm wp > "$O/catena-R-cens-$m.log" 2>&1; r=$?
  echo "censimento $m: rc=$r" >> "$V"; [ "$r" = 0 ] || RC=1
  for w in orm wp; do cp "$W4/s194-censimento-op-$w.txt" "$W5/s195-censimento-op-$w-$m.txt" 2>/dev/null || RC=1; done
done
( cd "$W4" && git checkout -- s194-censimento-op-orm.txt s194-censimento-op-wp.txt 2>/dev/null )
echo "esito rc=$RC $(date '+%F %T')" >> "$V"; echo "$RC" > "$O/catena-R.rc"; : > "$O/catena-R.done"; exit "$RC"
