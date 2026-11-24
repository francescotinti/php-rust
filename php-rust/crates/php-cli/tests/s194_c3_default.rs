//! S-194 C3a (2026-11-24): calls with fewer arguments than parameters enter the fast call paths
//! (the default prologue fills the missing slots). Behaviour tooth of the step, plus the divergence
//! it found: an `ArgumentCountError` carried line 0 instead of the declaration line.
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
fn an_argument_count_error_reports_the_declaration_line() {
    let script = r#"function d9($a, $b) { return 1; }
class A {
  function need($a, $b = 2, ...$r) { return 1; } }
foreach ([fn() => d9(1), fn() => (new A)->need(), function() { $c = function($x, $y) {}; $c(1); }] as $f) { try { $f(); } catch (ArgumentCountError $e) { echo $e->getMessage(), ' @', $e->getLine(), "\n"; } }"#;
    let expected = r#"Too few arguments to function d9(), 1 passed in Command line code on line 4 and exactly 2 expected @1
Too few arguments to function A::need(), 0 passed in Command line code on line 4 and at least 1 expected @3
Too few arguments to function {closure:{closure:Command line code:4}:4}(), 1 passed in Command line code on line 4 and exactly 2 expected @4"#;
    assert_eq!(run(script).trim_end(), expected);
}

#[test]
fn default_values_fill_the_parameters_not_passed_on_warm_call_sites() {
    let script = r#"class K { const D = 7; }
function d1($a, $b = 1) { return $a + $b; } function d2($a, $b = K::D, $c = [1, 2]) { return "$a|$b|" . count($c); }
function d3(int $a, ?string $s = null, float $f = 1) { return json_encode([$a, $s, $f]); } function d6($a, $b = 10) { return func_num_args() . ':' . implode(',', func_get_args()); }
class A { function m($a, $b = 'A') { return static::class . ":$a:$b"; } function t(int $a, int $b = 2): int { return $a + $b; } function u($x) { return $this->m($x) . '+' . $this->t($x); } }
class B extends A { function m($a, $b = 'B', $c = 'x') { return "B:$a:$b:$c"; } }
$a = new A; $b = new B;
for ($i = 0; $i < 2; $i++) { echo d1(1), ' ', d1(1, 5), ' ', d2('p'), ' ', d3(1), ' ', d3('2', 's'), ' ', d6(1), ' ', d6(1, 2, 3), "\n";
  foreach ([$a, $b, $a] as $o) { echo $o->m(1), ' ', $o->m(1, 2), ' ', $o->t(5), ' ', $o->u(4), "\n"; }
  try { echo $a->t('x'); } catch (TypeError $e) { echo explode(', called', $e->getMessage())[0], "\n"; } }"#;
    let expected = r#"2 6 p|7|2 [1,null,1] [2,"s",1] 1:1 3:1,2,3
A:1:A A:1:2 7 A:4:A+6
B:1:B:x B:1:2:x 7 B:4:B:x+6
A:1:A A:1:2 7 A:4:A+6
A::t(): Argument #1 ($a) must be of type int, string given
2 6 p|7|2 [1,null,1] [2,"s",1] 1:1 3:1,2,3
A:1:A A:1:2 7 A:4:A+6
B:1:B:x B:1:2:x 7 B:4:B:x+6
A:1:A A:1:2 7 A:4:A+6
A::t(): Argument #1 ($a) must be of type int, string given"#;
    assert_eq!(run(script).trim_end(), expected);
}

#[test]
fn dir_magic_constant_of_command_line_code_is_the_working_directory() {
    let script = r#"var_dump(__DIR__ === getcwd(), __DIR__ !== ".", __FILE__);"#;
    let expected = r#"bool(true)
bool(true)
string(17) "Command line code""#;
    assert_eq!(run(script).trim_end(), expected);
}
