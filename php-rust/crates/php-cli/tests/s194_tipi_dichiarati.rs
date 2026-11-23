//! S-194 (2026-11-23): declared types on calls and returns — divergences from PHP found by the
//! behaviour teeth of package C (`wp194-harness/s194-c-comportamento.php`) and cured together:
//! a typed by-reference parameter was not checked, a `static` return type was not enforced, a
//! callback called by an internal function was strict in a `strict_types` unit, an undefined
//! variable passed to a method did not warn, a scalar hint given an object said "object given",
//! `iterable` was spelled as written, and a return-conversion Deprecated carried a later line.
//! Each expected string is the oracle's stdout (php 8.5.7 CLI, `php -r`), byte for byte.
use std::process::{Command, Stdio};

fn run(script: &str) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_phpr"))
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .arg("-r")
        .arg(script)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .expect("spawn phpr");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn typed_by_ref_parameter_checks_and_coerces_the_referent() {
    let script = r#"function m($t) { return explode(", called", $t->getMessage())[0]; }
function e(int &$a) { $a++; } function c(array &$a) { $a[] = 1; } function d(?array &$a) { $a[] = 1; }
class P { function r(int &$a) { $a++; } }
$x = "5"; e($x); var_dump($x); $y = 5.0; (new P)->r($y); var_dump($y); d($n); echo count($n), "\n";
foreach ([function() { $w = "x"; e($w); }, function() { c($u); }, function() { $a = []; e($a["k"]); }, function() { (new P)->r($z); }] as $f) { try { $f(); echo "ok\n"; } catch (TypeError $t) { echo m($t), "\n"; } }"#;
    let expected = r#"int(6)
int(6)
1
e(): Argument #1 ($a) must be of type int, string given
c(): Argument #1 ($a) must be of type array, null given
e(): Argument #1 ($a) must be of type int, null given
P::r(): Argument #1 ($a) must be of type int, null given"#;
    assert_eq!(run(script).trim_end(), expected);
}

#[test]
fn static_return_type_is_checked_against_the_called_class() {
    let script = r#"class P { function a(): static { return $this; } function b(): static { return new P; } function c(): ?static { return null; } function d(): static|int { return "x"; } }
class P2 extends P {}
foreach ([fn() => (new P2)->a(), fn() => (new P)->b(), fn() => (new P2)->b(), fn() => (new P2)->c(), fn() => (new P2)->d(), function() { $f = function(): static { return new stdClass; }; return $f(); }] as $f) {
  try { $r = $f(); echo is_object($r) ? get_class($r) : var_export($r, true), "\n"; } catch (TypeError $t) { echo preg_replace('/\{closure.*\}\(\)/', '{closure}()', $t->getMessage()), "\n"; } }"#;
    let expected = r#"P2
P
P::b(): Return value must be of type P2, P returned
NULL
P::d(): Return value must be of type P2|int, string returned
{closure}(): Return value must be of type static, stdClass returned"#;
    assert_eq!(run(script).trim_end(), expected);
}

#[test]
fn a_callback_called_by_an_internal_function_is_weak_in_a_strict_unit() {
    let script = r#"declare(strict_types=1);
function spi(int $a) { return $a; } class K { function m(int $a) { return $a; } }
function t($n, $f) { try { echo $n, ":", json_encode($f()), "\n"; } catch (TypeError $e) { echo $n, ":TypeError\n"; } }
t('array_map', fn() => array_map('spi', ["1"])); t('method', fn() => array_map([new K, 'm'], ["1"]));
t('usort', function() { $a = ["2", "1"]; usort($a, fn(int $x, int $y) => $x <=> $y); return $a; });
t('call_user_func', fn() => call_user_func('spi', "1")); t('call_user_func_array', fn() => call_user_func_array('spi', ["1"]));
t('direct', fn() => spi("1")); t('dynamic', function() { $f = 'spi'; return $f("1"); });
t('reflection', fn() => (new ReflectionFunction('spi'))->invoke("1")); t('return', function() { $a = [2, 1]; usort($a, fn($x, $y): int => "1"); return 1; });"#;
    let expected = r#"array_map:[1]
method:[1]
usort:["1","2"]
call_user_func:call_user_func:TypeError
call_user_func_array:call_user_func_array:TypeError
direct:direct:TypeError
dynamic:dynamic:TypeError
reflection:1
return:return:TypeError"#;
    assert_eq!(run(script).trim_end(), expected);
}

#[test]
fn an_undefined_variable_passed_to_a_method_warns() {
    let script = r#"set_error_handler(function($n, $m) { echo "W:", $m, "\n"; return true; });
class T { function a($x) { return 1; } function b(int $x) { return 2; } function r(&$x) { $x = 7; } function __call($n, $a) { return 3; } }
function g() { $t = new T; echo $t->a($u1), "\n"; try { $t->b($u2); } catch (TypeError $e) { echo "TypeError\n"; } $t->r($u3); echo $u3, "\n"; $m = 'a'; echo $t->$m($u5), "\n"; }
g(); $t = new T; echo $t->a($g1), "\n"; $t->r($g2); echo $g2, "\n"; $s = new ArrayObject([]); $s->append($g3); echo count($s), "\n";"#;
    let expected = r#"W:Undefined variable $u1
1
W:Undefined variable $u2
TypeError
7
W:Undefined variable $u5
1
W:Undefined variable $g1
1
7
W:Undefined variable $g3
1"#;
    assert_eq!(run(script).trim_end(), expected);
}

#[test]
fn type_error_messages_name_the_class_and_spell_iterable_like_php() {
    let script = r#"class Z {} function m($t) { return explode(", called", $t->getMessage())[0]; }
function pin_(int $a) {} function ps(string $a) {} function it(iterable $i) {} function nit(?iterable $i) {} function ui(iterable|int $i) {}
function ri($v): int { return $v; } function rit($v): iterable { return $v; }
foreach ([fn() => pin_(new Z), fn() => ps(new Z), fn() => pin_(fn() => 1), fn() => it(1), fn() => nit(1), fn() => ui("x"), fn() => ri(new Z), fn() => rit(2)] as $f) { try { $f(); } catch (TypeError $t) { echo m($t), "\n"; } }"#;
    let expected = r#"pin_(): Argument #1 ($a) must be of type int, Z given
ps(): Argument #1 ($a) must be of type string, Z given
pin_(): Argument #1 ($a) must be of type int, Closure given
it(): Argument #1 ($i) must be of type Traversable|array, int given
nit(): Argument #1 ($i) must be of type Traversable|array|null, int given
ui(): Argument #1 ($i) must be of type Traversable|array|int, string given
ri(): Return value must be of type int, Z returned
rit(): Return value must be of type Traversable|array, int returned"#;
    assert_eq!(run(script).trim_end(), expected);
}

#[test]
fn a_return_conversion_deprecated_is_reported_at_the_return_line() {
    let script = r#"set_error_handler(function($n, $m, $f, $l) { echo "D:", $m, " @", $l, "\n"; return true; });
function ri($v): int {
  return $v; }
function t($f) {


  return $f(); }
echo t(fn() => ri(5.5)), "\n";"#;
    let expected = r#"D:Implicit conversion from float 5.5 to int loses precision @3
5"#;
    assert_eq!(run(script).trim_end(), expected);
}
