#!/bin/bash
# s176-census-typed.sh — CENSUS «typed» (NEXT §S-176 p.4; criterio s176-criterio-census-typed.md): COPIA DICHIARATA di
# s176-census.sh (manifest s176-census-typed-copia.diff) coi SOLI adattamenti: out census-typed-out/, verdetto
# s176-census-typed-verdetto.out, smoke a esito esatto sulla sezione «prop_set entry (S-176 typed)» (classe typed ⇒
# ic_hit_typed ≥1, plain ⇒ ic_hit_plain ≥1), parser che aggrega anche quella sezione; build op-census da HEAD coi
# contatori (census.rs/run.rs S-176). Testo originale sotto.
# s176-census.sh — CENSUS «op in place + Sweep» su WP media + ORM (NEXT §S-176 p.3; criterio s176-criterio-census.md).
# COPIA DICHIARATA di wp147-harness/s147-census-orm.sh (gamba ORM: untar orm-work.tgz su APFS, phpunit --no-coverage,
# watchdog 1800 s, parità per NOME vs baseline16) + /Volumes/Extreme Pro/Claude/wp53-harness/census53.sh (gamba media:
# wpdev, reset wptests, phpunit --group media) con la guardia uploads di s175-pair.sh (backup-wipe/restore, MAI wipe
# manuale) e la build census di wp109-harness/s109-census-run.sh (feature php-runtime/op-census, target separata
# phpr-census-target). Adattamenti: sorgente = HEAD (flag gc-idle nel tree: NON cambia i conteggi op); dump
# PHPR_OP_CENSUS su FILE (path assoluto ⇒ append per processo) per gamba; sezioni nuove «X -> Sweep (all)» e
# «IncDecSlotJmp -> X (all)» (census.rs S-176) aggregate per NOME su tutti i processi; smoke a esito ESATTO sulle sezioni;
# 1 rep per gamba (conteggi, non tempi). MAI cifra di tempo da qui.
# rc (census-out/census.done): 0 = gambe a parità e sezioni presenti · 1 = parità diverge · 4 = build · 7 = file/pre ·
# 8 = smoke muto · 9 = lock/disco.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp176-harness"; OUT="$H/census-typed-out"; mkdir -p "$OUT"
VERD="$H/s176-census-typed-verdetto.out"; DONE="$OUT/census.done"; rm -f "$DONE"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
WPDEV="/Users/francescotinti/Claude/wpdev"
GUARD="/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
CT="/Volumes/Extreme Pro/Claude/phpr-census-target"; CB="$CT/release/phpr"
SP="/private/tmp/phpr-s176-census-typed"; mkdir -p "$SP"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
fin(){ echo "rc=$1 $(date +%T)" > "$DONE"; exit "$1"; }
: > "$OUT/progress.txt"; : > "$VERD"
grep -qw s176 /private/tmp/phpr-measure.lock 2>/dev/null || { echo "rc=9 lock s176 assente" >> "$VERD"; fin 9; }
for f in "$GATES/orm-work.tgz" "$REPO/wp125-harness/orm-baseline-failnames.txt" "$GUARD" "$WPDEV/vendor/bin/phpunit"; do
  [ -s "$f" ] || { echo "rc=7 path MANCANTE: $f" >> "$VERD"; fin 7; }; done
mysql --socket=/private/tmp/mysql-wp8.sock -uroot -e "SHOW DATABASES" 2>/dev/null | grep -qx wptests || { echo "rc=7 MySQL wp8 giù o senza wptests" >> "$VERD"; fin 7; }
AVX=$(df -k "/Volumes/Extreme Pro" | awk 'NR>1{printf "%.0f", $4/1048576}'); [ "$AVX" -ge 15 ] || { echo "rc=9 Extreme ${AVX}G" >> "$VERD"; fin 9; }
cd "$REPO" || fin 7
SHA=$(git rev-parse HEAD)
p "build census (feature op-census, target $CT) da HEAD ${SHA:0:12}"
( CARGO_TARGET_DIR="$CT" cargo build --release -p php-cli --features php-runtime/op-census ) > "$OUT/census-build.log" 2>&1 \
  || { echo "rc=4 build census fallita (census-out/census-build.log)" >> "$VERD"; fin 4; }
CH=$(shasum -a 256 "$CB" | cut -c1-16)
# smoke: sezioni nuove a esito ESATTO (≥1 riga X -> Sweep, ≥1 riga IncDecSlotJmp -> X)
cat > "$OUT/smoke176.php" <<'PHP'
<?php
class P { public int $x = 0; }
class Q { public $y = 0; }
$o = new P; $q = new Q; $s = 0;
for ($i = 0; $i < 1000; $i++) { $o->x = $o->x + 1; $q->y = $q->y + 1; $s += $o->x + $q->y; }
echo $s, "\n";
PHP
rm -f "$OUT/smoke-op.txt"
PHPR_OP_CENSUS="$OUT/smoke-op.txt" "$CB" "$OUT/smoke176.php" > "$OUT/smoke176.out" 2>&1
grep -q "^-- bigrams X -> Sweep (all) --" "$OUT/smoke-op.txt" && grep -qE "^ *[0-9]+  [A-Za-z]+ -> Sweep$" "$OUT/smoke-op.txt" \
  && grep -qE "^ *[0-9]+  IncDecSlotJmp -> [A-Za-z]+$" "$OUT/smoke-op.txt" || { echo "rc=8 smoke: sezioni S-176 assenti o vuote (census-typed-out/smoke-op.txt)" >> "$VERD"; fin 8; }
grep -q "^-- prop_set entry (S-176 typed) --" "$OUT/smoke-op.txt" && grep -qE "^ *[1-9][0-9]*  prop_set ic_hit_typed$" "$OUT/smoke-op.txt" \
  && grep -qE "^ *[1-9][0-9]*  prop_set ic_hit_plain$" "$OUT/smoke-op.txt" || { echo "rc=8 smoke typed: ic_hit_typed/ic_hit_plain assenti o 0 (census-typed-out/smoke-op.txt)" >> "$VERD"; fin 8; }
p "smoke PASS ($(grep -c ' -> Sweep$' "$OUT/smoke-op.txt") righe X->Sweep)"

busy(){ { pgrep -x rustc || pgrep -x cargo; } > /dev/null 2>&1 && echo BUSY || echo CLEAN; }
run_wd(){ # $1=secondi $2..=comando (cwd corrente); rc del comando
  "${@:2}" & local PID=$!; ( sleep "$1"; kill -9 "$PID" 2>/dev/null ) & local W=$!
  wait "$PID"; local r=$?; kill "$W" 2>/dev/null; wait "$W" 2>/dev/null; return $r; }

# ---- gamba ORM (copia s147) ----
rm -rf "$SP/orm-work"; tar xzf "$GATES/orm-work.tgz" -C "$SP" || { echo "rc=7 untar orm" >> "$VERD"; fin 7; }
cd "$SP/orm-work" || fin 7
RAWO="$OUT/census-op-orm.txt"; rm -f "$RAWO"
echo "orm pre=$(busy)" >> "$OUT/sentinelle.txt"; p "ORM START"
PHPR_OP_CENSUS="$RAWO" run_wd 1800 "$CB" vendor/bin/phpunit --no-coverage > "$OUT/run-orm.txt" 2>&1; ORC=$?
echo "orm post=$(busy)" >> "$OUT/sentinelle.txt"; p "ORM rc=$ORC"
tr -d '\0' < "$OUT/run-orm.txt" | sed -n 's/^[0-9][0-9]*) \(.*\)$/\1/p' | sort -u > "$OUT/orm.failnames"
RC=0
diff -q "$REPO/wp125-harness/orm-baseline-failnames.txt" "$OUT/orm.failnames" > /dev/null && ORMPAR="== baseline16" || { ORMPAR="DIVERGE (census-out/orm.failnames)"; RC=1; }

# ---- gamba media (copia census53 + guardia uploads di s175-pair.sh) ----
"$GUARD" backup-wipe >> "$OUT/progress.txt" 2>&1 || { echo "rc=7 guard-backup" >> "$VERD"; fin 7; }
cd "$WPDEV" || fin 7
mysql -h 127.0.0.1 -u root -e "DROP DATABASE IF EXISTS wptests; CREATE DATABASE wptests; GRANT ALL ON wptests.* TO 'wp'@'%';" || { "$GUARD" restore >> "$OUT/progress.txt" 2>&1; echo "rc=7 mysql reset" >> "$VERD"; fin 7; }
find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
RAWM="$OUT/census-op-media.txt"; rm -f "$RAWM"
echo "media pre=$(busy)" >> "$OUT/sentinelle.txt"; p "MEDIA START"
PHPR_OP_CENSUS="$RAWM" MIMALLOC_PURGE_DELAY=0 run_wd 1800 "$CB" vendor/bin/phpunit --group media > "$OUT/run-media.txt" 2>&1; MRC=$?
echo "media post=$(busy)" >> "$OUT/sentinelle.txt"; p "MEDIA rc=$MRC"
find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
"$GUARD" restore >> "$OUT/progress.txt" 2>&1
tr -d '\0' < "$OUT/run-media.txt" | sed -n 's/^[0-9][0-9]*) \(.*\)$/\1/p' | sort -u > "$OUT/media.failnames"
[ -s "$OUT/media.failnames" ] && { MEDPAR="DIVERGE ($(wc -l < "$OUT/media.failnames" | tr -d ' ') nomi, census-out/media.failnames)"; RC=1; } || MEDPAR="OK(vuoto)"

# ---- aggregazione per NOME su tutti i processi (dump appesi) ----
agg(){ python3 - "$1" <<'PY'
import sys, re, collections
tot = 0; sw = collections.Counter(); idj = collections.Counter(); ps = collections.Counter(); sec = None; dumps = 0
for line in open(sys.argv[1], encoding='utf-8', errors='replace'):
    m = re.match(r"^== PHPR_OP_CENSUS: (\d+) ops dispatched ==", line)
    if m: tot += int(m.group(1)); dumps += 1; sec = None; continue
    if line.startswith("-- bigrams X -> Sweep (all) --"): sec = "sw"; continue
    if line.startswith("-- bigrams IncDecSlotJmp -> X (all) --"): sec = "idj"; continue
    if line.startswith("-- prop_set entry (S-176 typed) --"): sec = "ps"; continue
    if line.startswith("-- "): sec = None; continue
    mp = re.match(r"^\s*(\d+)\s+prop_set (\S+)$", line)
    if mp and sec == "ps": ps[mp.group(2)] += int(mp.group(1)); continue
    m = re.match(r"^\s*(\d+)\s+(\S+) -> (\S+)$", line)
    if m and sec == "sw": sw[m.group(2)] += int(m.group(1))
    elif m and sec == "idj": idj[m.group(3)] += int(m.group(1))
S = sum(sw.values())
print(f"processi(dump)={dumps} ops_totali={tot} Sweep_dispatch(=Σ X->Sweep)={S} ({S/tot*100 if tot else 0:.2f}% degli op)")
print("X -> Sweep (tutte le righe, % dei Sweep):")
for k, v in sw.most_common(): print(f"  {v:>12}  {k} -> Sweep  ({v/S*100 if S else 0:.1f}%)")
print("IncDecSlotJmp -> X:")
for k, v in idj.most_common(): print(f"  {v:>12}  IncDecSlotJmp -> {k}")
P = sum(ps.values())
print(f"prop_set entry (typed): totale={P} " + " ".join(f"{k}={v} ({v/P*100 if P else 0:.1f}%)" for k, v in ps.most_common()) + f"  quota_typed={ps['ic_hit_typed']/P*100 if P else 0:.1f}%")
PY
}
{ echo "== s176 CENSUS «typed» (prop_set IC plain/typed/miss) + Sweep WP media + ORM — CONTEGGI, mai tempo (criterio s176-criterio-census-typed.md) $(date '+%F %T') =="
  echo "census_phpr=$CH (build op-census da HEAD ${SHA:0:12}: probe, MAI parità)"
  echo "orm run_rc=$ORC parità per NOME vs baseline16: $ORMPAR · summary: $(tr -d '\0' < "$OUT/run-orm.txt" | sed -n 's/^\(Tests: .*\)$/\1/p' | tail -1)"
  echo "media run_rc=$MRC parità (nomi falliti): $MEDPAR · summary: $(tr -d '\0' < "$OUT/run-media.txt" | sed -n 's/^\(Tests: .*\)$/\1/p' | tail -1)"
  echo "--- sentinelle (non-gate) ---"; cat "$OUT/sentinelle.txt"
  echo "--- ORM ---"; agg "$RAWO"
  echo "--- media ---"; agg "$RAWM"
  echo "ESITO rc=$RC (0 = parità entrambe le gambe; 1 = parità diverge: i conteggi restano a verbale) fine $(date '+%F %T')"
} >> "$VERD" 2>&1
rm -rf "$SP/orm-work"
p "DONE rc=$RC"; fin $RC
