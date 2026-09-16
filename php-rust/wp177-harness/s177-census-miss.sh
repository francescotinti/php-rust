#!/bin/bash
# s177-census-miss.sh — CENSUS «IC miss di prop_set per CAUSA» su ORM + WP media (NEXT §S-177 p.2; criterio
# s177-criterio-census-miss.md): COPIA DICHIARATA di ../wp176-harness/s176-census-typed.sh (manifest
# s177-census-miss-copia.diff) coi SOLI adattamenti: (1) sorgente = COMMIT del ramo `s177-census` (git archive su
# /Volumes/Extreme Pro/Claude/s177-census-src: il tree main resta la sorgente del braccio C in gate), build op-census su
# target separata phpr-census-target; (2) out census-miss-out/, verdetto s177-census-miss-verdetto.out, SP
# phpr-s177-census-miss, lock s177; (3) smoke a esito ESATTO sulla sezione «prop_set miss (S-177 causa)»: sito
# polimorfo a 2 classi ⇒ psm_ic_class ≥1, prop dinamica ⇒ psr_dynamic ≥1, classe plain al primo passaggio ⇒
# psm_ic_empty ≥1 e psr_plain_fast ≥1; (4) parser che aggrega anche quella sezione (psm_*/psr_* in % del miss totale
# PROP_SET_MISS della sezione typed). Testo originale S-176 conservato sotto.
# rc (census-miss-out/census.done): 0 = gambe a parità e sezioni presenti · 1 = parità diverge · 4 = build · 7 = file/pre ·
# 8 = smoke muto · 9 = lock/disco.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
GITROOT="/Volumes/Extreme Pro/Claude/php-rust-experiment"
REPO="$GITROOT/php-rust"
H="$REPO/wp177-harness"; OUT="$H/census-miss-out"; mkdir -p "$OUT"
VERD="$H/s177-census-miss-verdetto.out"; DONE="$OUT/census.done"; rm -f "$DONE"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
WPDEV="/Users/francescotinti/Claude/wpdev"
GUARD="/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
CT="/Volumes/Extreme Pro/Claude/phpr-census-target"; CB="$CT/release/phpr"
CSRC="/Volumes/Extreme Pro/Claude/s177-census-src"
SP="/private/tmp/phpr-s177-census-miss"; mkdir -p "$SP"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
fin(){ echo "rc=$1 $(date +%T)" > "$DONE"; exit "$1"; }
: > "$OUT/progress.txt"; : > "$VERD"
grep -qw s177 /private/tmp/phpr-measure.lock 2>/dev/null || { echo "rc=9 lock s177 assente" >> "$VERD"; fin 9; }
for f in "$GATES/orm-work.tgz" "$REPO/wp125-harness/orm-baseline-failnames.txt" "$GUARD" "$WPDEV/vendor/bin/phpunit"; do
  [ -s "$f" ] || { echo "rc=7 path MANCANTE: $f" >> "$VERD"; fin 7; }; done
mysql --socket=/private/tmp/mysql-wp8.sock -uroot -e "SHOW DATABASES" 2>/dev/null | grep -qx wptests || { echo "rc=7 MySQL wp8 giù o senza wptests" >> "$VERD"; fin 7; }
AVX=$(df -k "/Volumes/Extreme Pro" | awk 'NR>1{printf "%.0f", $4/1048576}'); [ "$AVX" -ge 15 ] || { echo "rc=9 Extreme ${AVX}G" >> "$VERD"; fin 9; }
cd "$GITROOT" || fin 7
SHA=$(git rev-parse --verify "s177-census^{commit}") || { echo "rc=7 ramo s177-census assente" >> "$VERD"; fin 7; }
p "archivio ramo s177-census ${SHA:0:12} → $CSRC; build census (feature op-census, target $CT)"
rm -rf "$CSRC"; mkdir -p "$CSRC"
git archive "$SHA" php-rust/crates php-rust/Cargo.toml php-rust/Cargo.lock php-rust/rust-toolchain.toml php-rust/.cargo 2>/dev/null | tar -x -C "$CSRC" \
  || { echo "rc=7 archivio del ramo fallito" >> "$VERD"; fin 7; }
/usr/bin/find "$CSRC" -type f -exec touch {} +
( cd "$CSRC/php-rust" && CARGO_TARGET_DIR="$CT" cargo build --release -p php-cli --features php-runtime/op-census ) > "$OUT/census-build.log" 2>&1 \
  || { echo "rc=4 build census fallita (census-miss-out/census-build.log)" >> "$VERD"; fin 4; }
CH=$(shasum -a 256 "$CB" | cut -c1-16)
# smoke: sezione S-177 a esito ESATTO
cat > "$OUT/smoke177.php" <<'PHP'
<?php
class P { public $x = 0; }
class Q { public $x = 0; }
function w($o) { $o->x = $o->x + 1; }
$p = new P; $q = new Q; $d = new stdClass;
for ($i = 0; $i < 200; $i++) { w($p); w($q); }
for ($i = 0; $i < 50; $i++) { $d->k = $i; }
echo $p->x + $q->x + $d->k, "\n";
PHP
rm -f "$OUT/smoke-op.txt"
PHPR_OP_CENSUS="$OUT/smoke-op.txt" "$CB" "$OUT/smoke177.php" > "$OUT/smoke177.out" 2>&1
grep -q "^-- prop_set miss (S-177 causa) --" "$OUT/smoke-op.txt" \
  && grep -qE "^ *[1-9][0-9]*  prop_set psm_ic_class$" "$OUT/smoke-op.txt" \
  && grep -qE "^ *[1-9][0-9]*  prop_set psr_dynamic$" "$OUT/smoke-op.txt" \
  && grep -qE "^ *[1-9][0-9]*  prop_set psm_ic_empty$" "$OUT/smoke-op.txt" \
  && grep -qE "^ *[1-9][0-9]*  prop_set psr_plain_fast$" "$OUT/smoke-op.txt" \
  || { echo "rc=8 smoke causa: sezione S-177 assente o attese a 0 (census-miss-out/smoke-op.txt)" >> "$VERD"; fin 8; }
p "smoke PASS ($(grep -c ' prop_set ps[mr]_' "$OUT/smoke-op.txt") righe psm/psr)"

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
diff -q "$REPO/wp125-harness/orm-baseline-failnames.txt" "$OUT/orm.failnames" > /dev/null && ORMPAR="== baseline16" || { ORMPAR="DIVERGE (census-miss-out/orm.failnames)"; RC=1; }

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
[ -s "$OUT/media.failnames" ] && { MEDPAR="DIVERGE ($(wc -l < "$OUT/media.failnames" | tr -d ' ') nomi, census-miss-out/media.failnames)"; RC=1; } || MEDPAR="OK(vuoto)"

# ---- aggregazione per NOME su tutti i processi (dump appesi) ----
agg(){ python3 - "$1" <<'PY'
import sys, re, collections
tot = 0; ps = collections.Counter(); pm = collections.Counter(); sec = None; dumps = 0
for line in open(sys.argv[1], encoding='utf-8', errors='replace'):
    m = re.match(r"^== PHPR_OP_CENSUS: (\d+) ops dispatched ==", line)
    if m: tot += int(m.group(1)); dumps += 1; sec = None; continue
    if line.startswith("-- prop_set entry (S-176 typed) --"): sec = "ps"; continue
    if line.startswith("-- prop_set miss (S-177 causa) --"): sec = "pm"; continue
    if line.startswith("-- "): sec = None; continue
    mp = re.match(r"^\s*(\d+)\s+prop_set (\S+)$", line)
    if mp and sec == "ps": ps[mp.group(2)] += int(mp.group(1)); continue
    if mp and sec == "pm": pm[mp.group(2)] += int(mp.group(1)); continue
P = sum(ps.values()); M = ps['miss']
print(f"processi(dump)={dumps} ops_totali={tot}")
print(f"prop_set entry (typed): totale={P} " + " ".join(f"{k}={v} ({v/P*100 if P else 0:.1f}%)" for k, v in ps.most_common()))
psm = sum(v for k, v in pm.items() if k.startswith('psm_') and k != 'psm_init_props')
print(f"prop_set MISS per CAUSA (stato al miss, % del miss {M}; Σpsm={psm}):")
for k, v in sorted(((k, v) for k, v in pm.items() if k.startswith('psm_')), key=lambda kv: -kv[1]): print(f"  {v:>12}  {k}  ({v/M*100 if M else 0:.1f}%)")
print(f"prop_set MISS esito della risoluzione (psr_*, non esclusivi, % del miss {M}):")
for k, v in sorted(((k, v) for k, v in pm.items() if k.startswith('psr_')), key=lambda kv: -kv[1]): print(f"  {v:>12}  {k}  ({v/M*100 if M else 0:.1f}%)")
PY
}
{ echo "== s177 CENSUS «IC miss di prop_set per CAUSA» WP media + ORM — CONTEGGI, mai tempo (criterio s177-criterio-census-miss.md) $(date '+%F %T') =="
  echo "census_phpr=$CH (build op-census dal ramo s177-census ${SHA:0:12}: probe, MAI parità)"
  echo "orm run_rc=$ORC parità per NOME vs baseline16: $ORMPAR · summary: $(tr -d '\0' < "$OUT/run-orm.txt" | sed -n 's/^\(Tests: .*\)$/\1/p' | tail -1)"
  echo "media run_rc=$MRC parità (nomi falliti): $MEDPAR · summary: $(tr -d '\0' < "$OUT/run-media.txt" | sed -n 's/^\(Tests: .*\)$/\1/p' | tail -1)"
  echo "--- sentinelle (non-gate) ---"; cat "$OUT/sentinelle.txt"
  echo "--- ORM ---"; agg "$RAWO"
  echo "--- media ---"; agg "$RAWM"
  echo "ESITO rc=$RC (0 = parità entrambe le gambe; 1 = parità diverge: i conteggi restano a verbale) fine $(date '+%F %T')"
} >> "$VERD" 2>&1
rm -rf "$SP/orm-work" "$CSRC"
p "DONE rc=$RC"; fin $RC
