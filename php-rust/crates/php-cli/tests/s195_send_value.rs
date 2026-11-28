//! S-195 (2026-11-28): the value of a plain-variable argument of a dynamic call is the one it
//! has at the SEND. A later argument of the same call that writes the variable must not change
//! what a by-value parameter receives (`$o->m($i, $i = 5)` passes 1 and 5; phpr passed 5 and 5),
//! while a by-reference parameter still aliases the variable. Behaviour tooth of the step
//! (`wp195-harness/s195-invio-comportamento.php`): assignment, `++`, compound assignment, a
//! nested call taking the variable by reference, a by-reference closure capture, methods, static
//! and dynamic calls, `__call`, `__invoke`, closures, constructors, typed parameters, arrays,
//! strings, a variable that already is a reference, an undefined variable, the global scope.
//! The expected string is the oracle's stdout (php 8.5.7 CLI, `php -r`), byte for byte.
use std::process::{Command, Stdio};

#[test]
fn a_variable_argument_is_read_at_the_send() {
    let script = r#"// S-195 behaviour tooth: the value of a plain-variable argument is read at the SEND, not at the bind. Expected = the oracle's output.
namespace S195;
class K {
    function val($a, $b = 0) { return "$a,$b"; }
    function mix($a, &$b) { $r = "$a,$b"; $b = 'w'; return $r; }
    function ref(&$a, $b) { $r = "$a,$b"; $a = 'w'; return $r; }
    static function st($a, $b) { return "$a,$b"; }
    function __call($n, $a) { return "__call:" . implode(',', $a); }
    function __invoke($a, $b) { return "inv:$a,$b"; }
    function self2($a, $b) { return $this->val($a, $b); }
    function own() { $i = 1; return $this->val($i, $i = 5); }
    function typed(int $a, int $b): string { return "$a,$b"; }
    function arr($a, $b) { return json_encode($a) . json_encode($b); }
}
class P { public $v; function __construct($a, $b) { $this->v = "$a,$b"; } }
function inc(&$x) { return ++$x; }
function byv($a, $b) { return "$a,$b"; }
function run() {
    $o = new K;
    $i = 1; echo $o->val($i, $i = 5), " $i\n";
    $i = 1; echo $o->val($i, $i++), " $i\n";
    $i = 1; echo $o->val($i, ++$i), " $i\n";
    $i = 1; echo $o->val($i, $i += 3), " $i\n";
    $i = 1; echo $o->val($i, inc($i)), " $i\n";
    $i = 1; echo $o->val($i, $i), " $i\n";
    $i = 1; echo $o->val($i = 2, $i), " $i\n";
    $i = 1; echo $o->val($i, $i = 5, ), $o->val($i, 7), "\n";
    $i = 1; echo $o->mix($i, $i), " $i\n";
    $i = 1; $j = 2; echo $o->mix($i, $j), " $i $j\n";
    $i = 1; echo $o->ref($i, $i = 5), " $i\n";
    $i = 1; echo $o->ref($i, inc($i)), " $i\n";
    $i = 1; echo K::st($i, $i = 5), " $i\n";
    $c = 'S195\\K'; $i = 1; echo $c::st($i, $i = 5), " $i\n";
    $m = 'val'; $i = 1; echo $o->$m($i, $i = 5), " $i\n";
    $i = 1; echo $o->nope($i, $i = 5), " $i\n"; // __call: by value
    $i = 1; echo $o($i, $i = 5), " $i\n";
    $f = fn($a, $b) => "$a,$b"; $i = 1; echo $f($i, $i = 5), " $i\n";
    $g = function ($a, &$b) { $r = "$a,$b"; $b = 9; return $r; }; $i = 1; $k = 2; echo $g($i, $k), " $i $k\n";
    $i = 1; $p = new P($i, $i = 5); echo $p->v, " $i\n";
    $cn = 'S195\\P'; $i = 1; $p = new $cn($i, $i = 5); echo $p->v, " $i\n";
    $i = 1; echo byv($i, $i = 5), " $i\n";
    $i = 1; echo \S195\byv($i, $i = 5), " $i\n";
    $i = 1; echo $o->self2($i, $i = 5), " ", $o->own(), "\n";
    $i = 1; echo $o->typed($i, $i = 5), " $i\n";
    $a = [1]; echo $o->arr($a, $a[] = 2), " ", json_encode($a), "\n";
    $a = [1]; echo $o->arr($a, $a = [9]), " ", json_encode($a), "\n";
    $s = 'x'; echo $o->val($s, $s .= 'y'), " $s\n";
    $r = 1; $rr = &$r; echo $o->val($r, $r = 5), " $r\n";
    echo $o->val($u, $u = 5), " $u\n";
    $i = 1; echo $o->val($i, $o->val($i, $i = 5)), " $i\n";
    $i = 1; echo $o->val($i, [$i, $i = 4][1]), " $i\n";
    $i = 1; echo $o->val($i, (function () use (&$i) { return $i = 6; })()), " $i\n";
    $i = 1; echo call_user_func([$o, 'val'], $i, $i = 5), " $i\n";
    $i = 1; $j = 1; for ($n = 0; $n < 3; $n++) { echo $o->val($i, $j), $o->val($j, $j = $j + 1), ";"; } echo " $i $j\n";
    $i = 1; $b = $i; echo $o->val($i, 2), " "; $b++; echo "$i $b\n";
}
run();
$gi = 1; $go = new K; echo $go->val($gi, $gi = 5), " $gi\n";
"#;
    let expected = r#"1,5 5
1,1 2
1,2 2
1,4 4
1,2 2
1,1 1
2,2 2
1,55,7
1,1 w
1,2 1 w
5,5 w
2,2 w
1,5 5
1,5 5
1,5 5
__call:1,5 5
inv:1,5 5
1,5 5
1,2 1 9
1,5 5
1,5 5
1,5 5
1,5 5
1,5 1,5
1,5 5
[1]2 [1,2]
[1][9] [9]
x,xy xy
1,5 5

Warning: Undefined variable $u in Command line code on line 49
,5 5
1,1,5 5
1,4 4
1,6 6
1,5 5
1,11,2;1,22,3;1,33,4; 1 4
1,2 1 2
1,5 5"#;
    let out = Command::new(env!("CARGO_BIN_EXE_phpr"))
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .arg("-r")
        .arg(script)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .expect("spawn phpr");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim_end(), expected);
}
