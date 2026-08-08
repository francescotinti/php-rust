#!/bin/bash
# pair113.sh — S-100: collaudo WordPress full+media stessa-sera + coppia peak,
# versione BIMODALE del contratto di modo (A-KL-101-6). USO: pair113.sh <off|on>
# La gamba phpr spende PHPR_REG_LOWER=0|1 per VALORE ESPLICITO (mai assenza:
# dopo il flip del default l'assenza cambierebbe modo in silenzio). L'oracle
# non è toccato dal flag. Ricetta pair94 INVARIATA per il resto (GAP_TREND
# §Metodo): /usr/bin/time -l, DB reset + uploads azzerati PRIMA di ogni run,
# MIMALLOC_PURGE_DELAY=0, oracle PRIMA, uploads dalla guardia Gregg R7.
# A-KL-101-3 (KS-KL-101-3): assert conteggi↔nomi — i failnames estratti
# devono quadrare con Failures+Errors+Warnings dichiarati dallo strumento
# (calibrato sui raw pair99: 87=1F+86W oracle, 88=2F+86W phpr).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
case "${1:-}" in
  off) REG=0 ;;
  on)  REG=1 ;;
  *) echo "USO: $0 <off|on> — modo register-lowering ESPLICITO (A-KL-101-6)"; exit 64 ;;
esac
MODE="$1"
WPDEV="/Users/francescotinti/Claude/wpdev"
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp113-harness"
OUT="$H/pair-out-$MODE"; mkdir -p "$OUT"
ORACLE=/opt/homebrew/opt/php/bin/php
PHPR="$HOME/Claude/php-rust-output/release/phpr"
GUARD="/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
rm -f "$OUT/pair113.done"
step(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
GATE_VOID=0
reset_env() {
  mysql -h 127.0.0.1 -u root -e "DROP DATABASE IF EXISTS wptests; CREATE DATABASE wptests; GRANT ALL ON wptests.* TO 'wp'@'%';" || exit 3
  find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
}
{
  echo "modo=$MODE (PHPR_REG_LOWER=$REG esplicito sulla gamba phpr)"
  echo "phpr=$(shasum -a 256 "$PHPR" | cut -c1-16)"
  echo "php_server=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/php-server" | cut -c1-16)"
  echo "oracle=$("$ORACLE" -v | head -1)"
  echo "rustc=$(cd "$H/.." && rustc -Vv | tr '\n' ';')"
  echo "head=$(git -C "$H/.." rev-parse --short=12 HEAD)"
  echo "epoch=$(date +%s)"
} > "$OUT/pair113.identity"
"$GUARD" backup-wipe >> "$OUT/progress.txt" 2>&1 || { echo "rc=4 guard-backup" > "$OUT/pair113.done"; exit 4; }
cd "$WPDEV" || exit 2

step "media oracle"
reset_env
MIMALLOC_PURGE_DELAY=0 /usr/bin/time -l "$ORACLE" vendor/bin/phpunit --group media \
  > "$OUT/media-oracle.txt" 2> "$OUT/media-oracle.time"
step "media oracle rc=$?"
step "media phpr modo=$MODE"
reset_env
MIMALLOC_PURGE_DELAY=0 PHPR_REG_LOWER="$REG" /usr/bin/time -l "$PHPR" vendor/bin/phpunit --group media \
  > "$OUT/media-phpr.txt" 2> "$OUT/media-phpr.time"
step "media phpr rc=$?"

step "full oracle"
reset_env
MIMALLOC_PURGE_DELAY=0 /usr/bin/time -l "$ORACLE" vendor/bin/phpunit \
  > "$OUT/full-oracle.txt" 2> "$OUT/full-oracle.time"
step "full oracle rc=$?"
step "full phpr modo=$MODE"
reset_env
MIMALLOC_PURGE_DELAY=0 PHPR_REG_LOWER="$REG" /usr/bin/time -l "$PHPR" vendor/bin/phpunit \
  > "$OUT/full-phpr.txt" 2> "$OUT/full-phpr.time"
step "full phpr rc=$?"

find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
"$GUARD" restore >> "$OUT/progress.txt" 2>&1

# ---- chiusura per NOME (KS-GR-100-1) + assert conteggi↔nomi (A-KL-101-3).
# Il phpunit text output numera i difetti "N) Classe::metodo"; i path del
# progetto contengono SPAZI: niente awk-$1 (lezione WP-96/S-98).
names() { sed -n 's/^[0-9][0-9]*) \(.*\)$/\1/p' "$1" | sort; }
declared() { # Failures+Errors+Warnings+Risky dalla riga di sommario (default 0)
  local f="$1" s
  s="$(tr -d '\0' < "$f" | grep -E '^(Tests:|OK)' | tail -1)"
  local tot=0 k v
  for k in Failures Errors Warnings Risky; do
    v="$(printf '%s' "$s" | sed -n "s/.*$k: \([0-9]*\).*/\1/p")"
    tot=$((tot + ${v:-0}))
  done
  echo "$tot"
}
for g in media full; do
  for side in oracle phpr; do
    names "$OUT/$g-$side.txt" > "$OUT/$g-$side.failnames"
    n_decl="$(declared "$OUT/$g-$side.txt")"
    n_extr="$(wc -l < "$OUT/$g-$side.failnames" | tr -d ' ')"
    if [ "$n_decl" != "$n_extr" ]; then
      step "$g-$side: GATE VOID — nomi estratti ($n_extr) != dichiarati ($n_decl) (KS-KL-101-3)"
      GATE_VOID=1
    fi
  done
  if diff "$OUT/$g-oracle.failnames" "$OUT/$g-phpr.failnames" > "$OUT/$g.failnames.diff"; then
    step "$g: failure IDENTICI per NOME ($(wc -l < "$OUT/$g-oracle.failnames" | tr -d ' ') nomi)"
  else
    step "$g: failure DIVERSI per NOME — vedi $g.failnames.diff"
  fi
done
step "pair113 modo=$MODE DONE gate_void=$GATE_VOID"
echo "rc=$GATE_VOID modo=$MODE $(date +%T)" > "$OUT/pair113.done"

# rapporti = output MACCHINA dai .time (mai aritmetica in prosa)
ratios() {
  perl - "$OUT" <<'PERL'
use strict; use warnings;
my $out = shift @ARGV;
sub t { my ($f) = @_; open my $h, "<", "$out/$f.time" or die "$f: $!";
  my %v; while (my $l = <$h>) {
    $v{user} = $1 if $l =~ /([\d.]+)\s+user/; $v{sys} = $1 if $l =~ /([\d.]+)\s+sys/;
    $v{pf}   = $1 if $l =~ /^\s*(\d+)\s+peak memory footprint/;
    $v{rss}  = $1 if $l =~ /^\s*(\d+)\s+maximum resident set size/; }
  return \%v; }
# NB: mai `while (<$h>)` dentro `map { t($_) }` ($_ read-only, morso S-99).
my ($mo,$mp,$fo,$fp) = map { t($_) } qw(media-oracle media-phpr full-oracle full-phpr);
printf "media_user_cpu_oracle=%s media_user_cpu_phpr=%s ratio=%.3f\n", $mo->{user}, $mp->{user}, $mp->{user}/$mo->{user};
printf "media_peak_footprint_oracle=%d media_peak_footprint_phpr=%d ratio=%.3f\n", $mo->{pf}, $mp->{pf}, $mp->{pf}/$mo->{pf};
printf "full_master_cpu_oracle=%.2f full_master_cpu_phpr=%.2f ratio=%.3f\n", $fo->{user}+$fo->{sys}, $fp->{user}+$fp->{sys}, ($fp->{user}+$fp->{sys})/($fo->{user}+$fo->{sys});
printf "full_peak_footprint_oracle=%d full_peak_footprint_phpr=%d ratio=%.3f\n", $fo->{pf}, $fp->{pf}, $fp->{pf}/$fo->{pf};
printf "full_peak_footprint_phpr_MiB=%.2f\n", $fp->{pf}/1048576;
PERL
}
{ echo "pair113-ratios modo=$MODE: rapporti dai .time (output macchina, non prosa)"
  echo "grade=VERDICT  # derivazione meccanica riproducibile dai raw"
  echo "formato=ascii-nudo"
  ratios
} > "$H/pair113-ratios-$MODE.out"
