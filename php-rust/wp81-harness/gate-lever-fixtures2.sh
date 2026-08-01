#!/bin/bash
# gate-lever-fixtures2.sh — S-81.0 step 6: design79 §9 fixtures F1-F4, F7,
# F9-F13 on the cli-server arm (the honest A/B twin: single accept thread,
# same acquire/park/publish as the worker). Every HIT parity verdict is
# FULL-BODY vs the ORACLE (KS-PP-80-3; auto-equality banned KS-AH-80-2).
# Counters = uc_log events (union build). F5/F8b/F8c/F-oneshot live in
# gate-lever-fixtures.sh; F6 is the in-cargo park-events test; F8 is
# subsumed by F8c (fatal -> fix -> 200 with put==1).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo 'no-git')"
OUTBIN="$HOME/Claude/php-rust-output/release"
ORACLE="/opt/homebrew/opt/php/bin/php"
PORT="${PORT:-8397}"
FAILS=0

TMPD=$(mktemp -d)
SRVPID=""
cleanup() { [ -n "$SRVPID" ] && kill -TERM "$SRVPID" 2>/dev/null; rm -rf "$TMPD"; }
trap cleanup EXIT

count_ev() { local log="$1" ev="$2" sub="${3:-}"
  [ -f "$log" ] || { echo 0; return; }
  if [ -n "$sub" ]; then grep -c "^unitcache $ev .*$sub" "$log" || true
  else grep -c "^unitcache $ev " "$log" || true; fi
}
req() { curl -s -m 5 "http://127.0.0.1:$PORT/$1"; }
okf() { echo "OK  $1"; }
kof() { echo "FAIL: $1"; FAILS=$((FAILS+1)); }
# Oracle body for a fixture file (CLI rendering == cli-server body form).
oracle_body() { ( cd "$DOCROOT" && "$ORACLE" -n "$1" 2>/dev/null ); }

[ -x "$ORACLE" ] || { echo "FAIL: oracle $ORACLE missing"; exit 1; }

DOCROOT="$TMPD/docroot"
mkdir -p "$DOCROOT"
ULOG="$TMPD/f2.uclog"
echo '<?php echo "boot";' > "$DOCROOT/boot.php"
PHPR_UNIT_CACHE_LOG="$ULOG" "$OUTBIN/php-server" --port "$PORT" -t "$DOCROOT" \
  > /dev/null 2> "$TMPD/server.log" &
SRVPID=$!
up=0
for _ in $(seq 1 100); do
  if curl -s -o /dev/null -m 1 "http://127.0.0.1:$PORT/boot.php"; then up=1; break; fi
  sleep 0.1
done
[ "$up" = 1 ] || { echo "FAIL: server did not come up"; exit 1; }

# --- F1: main edited between requests (content+size) -------------------------
echo '<?php echo "v1";' > "$DOCROOT/f1.php"
B1=$(req f1.php); B1b=$(req f1.php)   # second request = HIT on v1
sleep 1.1                              # distinct mtime second (belt; fp is the judge)
echo '<?php echo "v2-different-size";' > "$DOCROOT/f1.php"
B2=$(req f1.php)
if [ "$B1" = "v1" ] && [ "$B1b" = "v1" ] && [ "$B2" = "v2-different-size" ]; then
  np=$(count_ev "$ULOG" main_put f1.php)
  if [ "$np" -eq 2 ]; then okf "F1: edit (content+size) => new body, main_put==2 (v1+v2)";
  else kof "F1 counters: main_put=$np != 2 on f1.php"; fi
else
  kof "F1 bodies: '$B1'/'$B1b'/'$B2' (stale main served? KH80-1 => lever REJECTED)"
fi

# --- F2: SAME-SIZE, SAME-MTIME edit — the content fp is the ONLY judge --------
echo '<?php echo "AAAA";' > "$DOCROOT/f2.php"
touch -r "$DOCROOT/f2.php" "$TMPD/f2.ref" 2>/dev/null || cp -p "$DOCROOT/f2.php" "$TMPD/f2.ref"
M_PRE=$(stat -f "%Fm %z" "$DOCROOT/f2.php")
B1=$(req f2.php); B1b=$(req f2.php)
# same byte-length, different content; RESTORE the original mtime so the
# UnitKey (path, mtime, size) is IDENTICAL — without this the test passes
# vacuously through a key miss (mtime moved), never exercising the fp.
echo '<?php echo "BBBB";' > "$DOCROOT/f2.php"
touch -r "$TMPD/f2.ref" "$DOCROOT/f2.php"
# A-SK21/KS-SK-83-2 (Council WP-83): SAME-KEY proven BY MACHINE — APFS
# `touch -r` can truncate sub-µs mtime (utimes µs vs stat ns): if the
# restored (mtime ns, size) differ, the MISS below is a key-miss and the
# fingerprint was never exercised — F2 must then FAIL, not pass vacuously.
M_POST=$(stat -f "%Fm %z" "$DOCROOT/f2.php")
B2=$(req f2.php)
if [ "$M_PRE" != "$M_POST" ]; then
  kof "F2 same-key NOT proven: (mtime ns, size) '$M_PRE' -> '$M_POST' (A-SK21: fp never exercised, F2 vacuous)"
elif [ "$B1" = "AAAA" ] && [ "$B2" = "BBBB" ]; then
  # counter tooth: both versions PUT, the repeat was a genuine HIT — with
  # key identity proven above, the second put can only be an fp-MISS.
  np2=$(count_ev "$ULOG" main_put f2.php)
  nh2=$(count_ev "$ULOG" main_hit f2.php)
  if [ "$np2" -eq 2 ] && [ "$nh2" -eq 1 ]; then
    okf "F2: SAME-SIZE SAME-KEY edit => MISS via content fp (stat ns identical, put==2, hit==1)"
  else
    kof "F2 counters: main_put=$np2 (pin 2) main_hit=$nh2 (pin 1) on f2.php (A-SK21)"
  fi
else
  kof "F2: same-size edit served stale body ('$B2' after '$B1') — KH80-1: lever REJECTED"
fi

# --- F3: symlink swap of the entry (canonicalize = revalidate_path 1) ---------
mkdir -p "$DOCROOT/v1" "$DOCROOT/v2"
echo '<?php echo "from-v1";' > "$DOCROOT/v1/app.php"
echo '<?php echo "from-v2";' > "$DOCROOT/v2/app.php"
ln -sfn "$DOCROOT/v1/app.php" "$DOCROOT/f3.php"
B1=$(req f3.php); B1b=$(req f3.php)
ln -sfn "$DOCROOT/v2/app.php" "$DOCROOT/f3.php"
B2=$(req f3.php)
if [ "$B1" = "from-v1" ] && [ "$B2" = "from-v2" ]; then
  okf "F3: symlink swap => new canonical path => MISS (revalidate_path=1 PINNED)"
else
  kof "F3: symlink swap served '$B2' after '$B1' (stale canonical binding)"
fi

# --- F4: main with conditional/impure include — body correct, main published --
cat > "$DOCROOT/f4inc.php" <<'EOF'
<?php function f4x() { return "inc"; }
EOF
cat > "$DOCROOT/f4.php" <<'EOF'
<?php
if (($_GET["x"] ?? "") !== "no") { include "f4inc.php"; }
echo function_exists("f4x") ? f4x() : "none";
EOF
B1=$(req "f4.php"); B2=$(req "f4.php?x=no"); B3=$(req "f4.php")
ns=$(count_ev "$ULOG" main_impure_skip f4.php)
if [ "$B1" = "inc" ] && [ "$B2" = "none" ] && [ "$B3" = "inc" ] && [ "$ns" -eq 0 ]; then
  okf "F4: conditional include never wrong on HIT; main_impure_skip==0 (deviation declared: no impure-main path on this tree)"
else
  kof "F4: bodies '$B1'/'$B2'/'$B3' (want inc/none/inc), main_impure_skip=$ns (pin 0)"
fi

# --- F7: stale-IC — class order varied, 3 requests, body == ORACLE ------------
cat > "$DOCROOT/f7.php" <<'EOF'
<?php
if (($_GET["o"] ?? "a") === "a") {
    class F7A { public $p = "A"; public function m() { return "mA"; } }
    class F7B { public $p = "B"; public function m() { return "mB"; } }
} else {
    class F7B { public $p = "B"; public function m() { return "mB"; } }
    class F7A { public $p = "A"; public function m() { return "mA"; } }
}
$a = new F7A; $b = new F7B;
echo $a->p, $b->p, $a->m(), $b->m();
EOF
WANT_A=$(oracle_body "f7.php")            # oracle with default order (no $_GET)
B1=$(req "f7.php?o=a"); B2=$(req "f7.php?o=b"); B3=$(req "f7.php?o=a")
if [ "$B1" = "$WANT_A" ] && [ "$B2" = "$WANT_A" ] && [ "$B3" = "$WANT_A" ]; then
  okf "F7: class-order variation x3 == oracle (IC epoch holds on HIT)"
else
  kof "F7: bodies vs oracle diverge ('$B1'/'$B2'/'$B3' vs '$WANT_A') — stale IC class"
fi

# --- F9: eviction/supersede must SCATTARE (KG-80-2) ---------------------------
echo '<?php echo "e0";' > "$DOCROOT/f9.php"
req f9.php > /dev/null
for i in 1 2 3 4 5; do
  sleep 0.05
  echo "<?php echo \"e$i\";" > "$DOCROOT/f9.php"
  req f9.php > /dev/null
done
nsup=$(count_ev "$ULOG" supersede f9.php)
if [ "$nsup" -ge 1 ]; then
  okf "F9: supersede fired $nsup times on repeated edits (counter SEEN moving)"
else
  kof "F9: no supersede event after 5 edits of one path (KG-80-2)"
fi

# --- F10: early-binding — main extends P from lib; edit the lib ---------------
cat > "$DOCROOT/f10lib.php" <<'EOF'
<?php class F10P { public function v() { return "old"; } }
EOF
cat > "$DOCROOT/f10.php" <<'EOF'
<?php
require "f10lib.php";
class F10C extends F10P {}
$c = new F10C;
echo $c->v();
EOF
B1=$(req f10.php); B1b=$(req f10.php)
sleep 1.1
cat > "$DOCROOT/f10lib.php" <<'EOF'
<?php class F10P { public function v() { return "new"; } }
EOF
B2=$(req f10.php)
if [ "$B1" = "old" ] && [ "$B1b" = "old" ] && [ "$B2" = "new" ]; then
  okf "F10: lib edit never serves the old binding on main-HIT (KS-DS-81-1)"
else
  kof "F10: '$B1'/'$B1b'/'$B2' (want old/old/new) — STALE early binding => lever REJECTED"
fi

# --- F11: __FILE__/__DIR__ — two spellings, same canonical file ----------------
mkdir -p "$DOCROOT/sub"
cat > "$DOCROOT/sub/f11.php" <<'EOF'
<?php echo __FILE__, "|", __DIR__;
EOF
WANT=$(cd "$DOCROOT" && "$ORACLE" -n "sub/f11.php" 2>/dev/null)
B1=$(req "sub/f11.php"); B2=$(req "sub/../sub/f11.php"); B3=$(req "sub/f11.php")
if [ "$B1" = "$WANT" ] && [ "$B3" = "$WANT" ]; then
  okf "F11: __FILE__/__DIR__ byte-parity with oracle on HIT (spelling 2: '$B2' vs canonical — recorded)"
else
  kof "F11: __FILE__ diverges from oracle on HIT ('$B1'/'$B3' vs '$WANT') — KS-DS-81-2: REVERT"
fi

# --- F12: declare(strict_types=1) in the cached main ---------------------------
cat > "$DOCROOT/f12.php" <<'EOF'
<?php declare(strict_types=1);
function f12t(int $x): int { return $x + 1; }
try { echo f12t("3"); } catch (TypeError $e) { echo "TypeError-caught"; }
EOF
WANT=$(oracle_body "f12.php")
B1=$(req f12.php); B2=$(req f12.php); B3=$(req f12.php)
if [ "$B1" = "$WANT" ] && [ "$B2" = "$WANT" ] && [ "$B3" = "$WANT" ]; then
  okf "F12: strict_types coercion identical on HIT x3 (== oracle: '$WANT')"
else
  kof "F12: strict_types on HIT diverges ('$B1'/'$B2'/'$B3' vs '$WANT')"
fi

# --- F13 (A-DS13): include ORDER varied across requests on HIT -----------------
cat > "$DOCROOT/f13a.php" <<'EOF'
<?php class F13A { public static function w() { return "A"; } }
EOF
cat > "$DOCROOT/f13b.php" <<'EOF'
<?php class F13B { public static function w() { return "B"; } }
EOF
cat > "$DOCROOT/f13.php" <<'EOF'
<?php
if (($_GET["o"] ?? "ab") === "ab") { include "f13a.php"; include "f13b.php"; }
else { include "f13b.php"; include "f13a.php"; }
$a = new F13A; $b = new F13B;
echo F13A::w(), F13B::w(), ($a instanceof F13A ? "ia" : "-"), ($b instanceof F13B ? "ib" : "-"), get_class($a), get_class($b);
EOF
WANT=$(oracle_body "f13.php")
B1=$(req "f13.php?o=ab"); B2=$(req "f13.php?o=ba"); B3=$(req "f13.php?o=ab"); B4=$(req "f13.php?o=ba")
if [ "$B1" = "$WANT" ] && [ "$B2" = "$WANT" ] && [ "$B3" = "$WANT" ] && [ "$B4" = "$WANT" ]; then
  okf "F13: include order varied x4 on HIT == oracle (baked-id class 2 holds, A-DS12)"
else
  kof "F13: '$B1'/'$B2'/'$B3'/'$B4' vs oracle '$WANT' — baked run-scoped id => lever REJECTED (KS-DS-82-1)"
fi
# F13 POSITIVE control (KS-AH-80-2: auto-equality banned — the comparator
# must BITE): a fixture that deliberately varies per request must FAIL the
# same body==oracle check.
cat > "$DOCROOT/f13x.php" <<'EOF'
<?php echo isset($_GET["v"]) ? "varies" : "base";
EOF
WX=$(oracle_body "f13x.php")
BX=$(req "f13x.php?v=1")
if [ "$BX" = "$WX" ]; then
  kof "F13 positive control: deliberately-varying body did NOT diverge — comparator vacuous"
else
  okf "F13 positive control: comparator bites on a varying body (KS-AH-80-2)"
fi

# --- F14 (A-DS18, Council WP-83): conditional class IN the main + eval-mint
# + extends-deferred (parent from include), include order varied, >=3
# requests == oracle. F13 alone does not discriminate the conditional/
# deferred surfaces (KS-DS-83-3: F14 red or absent in an A/B citing class 2
# => class 2 REOPENED).
cat > "$DOCROOT/f14a.php" <<'EOF'
<?php class F14P { public function v() { return "P"; } }
EOF
cat > "$DOCROOT/f14b.php" <<'EOF'
<?php class F14Q { public function v() { return "Q"; } }
EOF
cat > "$DOCROOT/f14.php" <<'EOF'
<?php
if (($_GET["o"] ?? "ab") === "ab") { include "f14a.php"; include "f14b.php"; }
else { include "f14b.php"; include "f14a.php"; }
if (($_GET["c"] ?? "y") === "y") {
    class F14C { public function w() { return "C1"; } }
} else {
    class F14C { public function w() { return "C2"; } }
}
eval('class F14E { public function e() { return "E"; } }');
class F14D extends F14P {}
$c = new F14C; $e = new F14E; $d = new F14D; $q = new F14Q;
echo $c->w(), $e->e(), $d->v(), $q->v(),
     ($d instanceof F14P ? "iP" : "-"), get_class($d), get_class($e);
EOF
W14=$(oracle_body "f14.php")
B1=$(req "f14.php?o=ab&c=y"); B2=$(req "f14.php?o=ba&c=y"); B3=$(req "f14.php?o=ab&c=y"); B4=$(req "f14.php?o=ba&c=y")
if [ -n "$W14" ] && [ "$B1" = "$W14" ] && [ "$B2" = "$W14" ] && [ "$B3" = "$W14" ] && [ "$B4" = "$W14" ]; then
  okf "F14: conditional-class + eval-mint + extends-deferred, order varied x4 on HIT == oracle (A-DS18)"
else
  kof "F14: '$B1'/'$B2'/'$B3'/'$B4' vs oracle '$W14' — conditional/deferred surface stale => lever REJECTED (KS-DS-83-3)"
fi
# F14 positive control (same comparator law as F13x: it must BITE) — the
# c=n arm declares a DIFFERENT F14C body: oracle (no $_GET) vs c=n differ.
BX=$(req "f14.php?o=ab&c=n")
if [ "$BX" = "$W14" ]; then
  kof "F14 positive control: c=n arm did NOT diverge from oracle — comparator vacuous"
else
  okf "F14 positive control: comparator bites on the varied conditional arm"
fi

# --- F14b (A-DS21/A-SK31, Council WP-84): pin-HIT + full (o,c) matrix --------
# A-SK31/KS-SK-84-4: the c=n control above proves inequality only — it would
# pass on ANY body, a fatal included. Pin the EXPECTED C2 body: it is W14
# with the one delta the c=n arm declares (F14C::w() returns C2, not C1).
W14N="${W14/C1/C2}"
B5=$(req "f14.php?o=ab&c=n"); B6=$(req "f14.php?o=ba&c=n")
if [ -n "$W14N" ] && [ "$W14N" != "$W14" ] && [ "$B5" = "$W14N" ] && [ "$B6" = "$W14N" ]; then
  okf "F14b: (o,c) matrix complete — c=n arms == PINNED expected C2 body, both orders (A-SK31)"
else
  kof "F14b: c=n arms '$B5'/'$B6' != pinned expected '$W14N' (A-SK31/KS-SK-84-4)"
fi
# A-DS21/KS-DS-84-1: pin-HIT — without this a silent per-URL re-MISS makes
# the whole F14 parity green and VACUOUS w.r.t. the lever. One put (first
# request), everything after is a REAL cache HIT on the same (key, fp):
# B2 B3 B4 BX B5 B6 = 6 hits minimum.
NP14=$(count_ev "$ULOG" main_put f14.php)
NH14=$(count_ev "$ULOG" main_hit f14.php)
if [ "$NP14" -eq 1 ] && [ "$NH14" -ge 6 ]; then
  okf "F14b: pin-HIT main_put==1 / main_hit==$NH14>=6 — the parity matrix ran on REAL HITs (A-DS21)"
else
  kof "F14b: main_put=$NP14/main_hit=$NH14 (want 1/>=6) — silent re-MISS per URL: F14 vacuous, class 2 REOPENED (KS-DS-84-1)"
fi

# --- F15 (A-DS20 -> FLIPPED per A-DS24, S-83.0 p6): same file as MAIN and
# as include under 5 distinct include-chain-fps. PRE-mitigation this
# evicted the hot main (FIFO, 5 fps on 4 mixed ways — the S-82.0 PASS
# documented it). POST-mitigation (A-MS24 partition-by-TYPE: main lane
# separate, includes keep the ways) the pins FLIP: main_evicted == 0
# (KS-DS-84-4: >0 anywhere = campaign VOID) with the eviction machinery
# PROVEN ALIVE on the include lane of the SAME key (5th fp evicts an
# include: evict-fp event >= 1) — never a vacuous zero (KG-79.A).
cat > "$DOCROOT/t15.php" <<'EOF'
<?php echo "T15";
EOF
for i in 1 2 3 4 5; do
  cat > "$DOCROOT/f15h$i.php" <<EOF
<?php class F15H$i { public static function h() { return "h$i"; } }
EOF
  cat > "$DOCROOT/f15w$i.php" <<EOF
<?php include "f15h$i.php"; include "t15.php"; echo F15H$i::h();
EOF
done
BM1=$(req t15.php)                      # main entry for t15's UnitKey
for i in 1 2 3 4 5; do req "f15w$i.php" > /dev/null; done  # 5 include-fps, same key
NEV=$(count_ev "$ULOG" main_evicted t15.php)
NEVICT=$(count_ev "$ULOG" "evict fp" t15.php)
BM2=$(req t15.php)                      # post-thrash: the main must be a HIT now
NH15=$(count_ev "$ULOG" main_hit t15.php)
if [ "$BM1" = "T15" ] && [ "$BM2" = "T15" ]; then
  if [ "$NEV" -eq 0 ] && [ "$NEVICT" -ge 1 ] && [ "$NH15" -ge 1 ]; then
    okf "F15 FLIPPED: main_evicted==0 with include-lane eviction ALIVE ($NEVICT evict-fp) and post-thrash main HIT ($NH15) — A-MS24 partition holds (A-DS24/KS-DS-84-4)"
  else
    kof "F15: main_evicted=$NEV (want 0) evict-fp=$NEVICT (want >=1) main_hit=$NH15 (want >=1) — partition regressed or machinery blind (KS-DS-84-4)"
  fi
else
  kof "F15: bodies '$BM1'/'$BM2' != 'T15' under include-lane thrash"
fi

# --- F15b (A-DS24 -> FLIPPED, S-83.0 p6): INTERLEAVED discriminant — the
# main is TOUCHED between every include-fp insert. PRE-mitigation the FIFO
# evicted it regardless of recency (S-83.0 p4 documented main_evicted==1
# interleaved). POST-mitigation the discriminant reads EXEMPTION: the main
# survives BY LANE, not by recency — main_evicted==0, every touch after
# the first is a HIT (main_put==1, main_hit>=5), include-lane eviction
# alive on the same key (5th fp).
cat > "$DOCROOT/t15b.php" <<'EOF'
<?php echo "T15B";
EOF
for i in 1 2 3 4 5; do
  cat > "$DOCROOT/f15bh$i.php" <<EOF
<?php class F15BH$i { public static function h() { return "b$i"; } }
EOF
  cat > "$DOCROOT/f15bw$i.php" <<EOF
<?php include "f15bh$i.php"; include "t15b.php"; echo F15BH$i::h();
EOF
done
BM1=$(req t15b.php)
for i in 1 2 3 4 5; do
  req "f15bw$i.php" > /dev/null
  req t15b.php > /dev/null        # touch the main between inserts (recency)
done
NEVB=$(count_ev "$ULOG" main_evicted t15b.php)
NEVICTB=$(count_ev "$ULOG" "evict fp" t15b.php)
NPB=$(count_ev "$ULOG" main_put t15b.php)
NHB=$(count_ev "$ULOG" main_hit t15b.php)
BM2=$(req t15b.php)
if [ "$BM1" = "T15B" ] && [ "$BM2" = "T15B" ]; then
  if [ "$NEVB" -eq 0 ] && [ "$NPB" -eq 1 ] && [ "$NHB" -ge 5 ] && [ "$NEVICTB" -ge 1 ]; then
    okf "F15b FLIPPED: exemption BY LANE — main_put==1/main_hit==$NHB>=5/main_evicted==0 with include eviction alive ($NEVICTB) (A-DS24/KS-DS-84-3)"
  else
    kof "F15b: put=$NPB hit=$NHB evicted=$NEVB evict-fp=$NEVICTB (want 1/>=5/0/>=1) — exemption incomplete or machinery blind (KS-DS-84-3/4)"
  fi
else
  kof "F15b: bodies '$BM1'/'$BM2' != 'T15B' under interleaved thrash"
fi

kill -TERM "$SRVPID" 2>/dev/null; wait "$SRVPID" 2>/dev/null; SRVPID=""

if [ "$FAILS" = 0 ]; then
  echo "== GATE-LEVER-FIXTURES2 PASS (F1-F4, F7, F9-F15 + F14b/F15b + positive controls) [git $GIT_REV] =="
  exit 0
else
  echo "== GATE-LEVER-FIXTURES2 FAIL($FAILS) [git $GIT_REV] =="
  exit 1
fi
