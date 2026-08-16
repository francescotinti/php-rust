#!/bin/bash
# s145-riparse-golden-test.sh — golden del riparse simmetrico (le quote sono
# calcolate A MANO sui campioni sintetici; confronto su stringhe ESATTE).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp145-harness"
T="$(mktemp -d /private/tmp/s145-riparse-golden.XXXXXX)"
trap 'rm -rf "$T"' EXIT
FAIL=0

cat > "$T/oracle.txt" <<'EOF'
Analysis of sampling php (pid 1) every 1 millisecond
Sort by top of stack, same collapsed (when >= 5):
        __workq_kernreturn  (in libsystem_kernel.dylib)        690
        execute_ex  (in php)        100
        memcpy  (in libsystem_platform.dylib)        50
        zval_ptr_dtor  (in php)        30
        sqlite3VdbeExec  (in php)        20
Binary Images:
EOF
# tot=890 idle=690 att=200: vm_inline 100=50%, memops 50=25%, churn 30=15%, other 20=10%
O="$(/usr/bin/python3 "$H/s145-riparse.py" "g1:oracle:$T/oracle.txt")"
echo "$O" | grep -Fq "g1 [oracle]: tot=890 idle=690 attivi=200" || { echo "GOLDEN oracle denominatore FALLITO"; echo "$O"; FAIL=1; }
echo "$O" | grep -Fq "g1 vm_inline: 100 (50.00%)" || { echo "GOLDEN oracle vm_inline FALLITO"; echo "$O"; FAIL=1; }
echo "$O" | grep -Fq "g1 GIUDICE-UNICO: churn_zval=15.00% memops=25.00%" || { echo "GOLDEN oracle giudice FALLITO"; echo "$O"; FAIL=1; }

cat > "$T/phpr.txt" <<'EOF'
Analysis of sampling phpr (pid 2) every 1 millisecond
Sort by top of stack, same collapsed (when >= 5):
        phpr::vm::run_loop  (in phpr)        120
        memmove  (in libsystem_platform.dylib)        40
        _ZvalcloneGT  (in phpr)        30
        kevent  (in libsystem_kernel.dylib)        10
Binary Images:
EOF
# tot=200 idle=10 att=190: vm_inline 120=63,16%, memops 40=21,05%, churn 30=15,79%
P="$(/usr/bin/python3 "$H/s145-riparse.py" "g2:phpr:$T/phpr.txt")"
echo "$P" | grep -Fq "g2 [phpr]: tot=200 idle=10 attivi=190" || { echo "GOLDEN phpr denominatore FALLITO"; echo "$P"; FAIL=1; }
echo "$P" | grep -Fq "g2 GIUDICE-UNICO: churn_zval=15.79% memops=21.05%" || { echo "GOLDEN phpr giudice FALLITO"; echo "$P"; FAIL=1; }

[ "$FAIL" = 0 ] && echo "GOLDEN PASS 2/2" || echo "GOLDEN FAIL"
exit "$FAIL"
