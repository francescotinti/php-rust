//! S-194 N1 (2026-11-26): substring search answers a one-byte needle and a short haystack without
//! building a searcher per call; object ids are reused without hashing empty per-id maps; a
//! non-Throwable `new` no longer clones its handle to learn it is not a Throwable. Behaviour tooth
//! of the step (`wp194-harness/s194-n1-comportamento.php`): strpos / str_contains / strstr /
//! strrpos / substr_count / stripos on haystacks on both sides of the 256-byte threshold, empty
//! and over-long needles, offsets; Throwables and plain objects created in a loop with id reuse.
//! The expected string is the oracle's stdout (php 8.5.7 CLI, `php -r`), byte for byte.
use std::process::{Command, Stdio};

#[test]
fn substring_search_and_object_reuse_match_the_oracle() {
    let script = r#"// S-194 N1 behaviour tooth: substring search on short and long haystacks, object ids reused. Expected = the oracle's output.
function b($v) { return var_export($v, true); }
$h = str_repeat("ab", 100) . "needle" . str_repeat("cd", 100);
$long = str_repeat("xy", 200) . "needle" . str_repeat("zz", 50);
foreach ([["hello world", "o"], ["hello world", "world"], ["hello", "hello!"], ["hello", "l"], ["", "a"], ["aaa", "aa"], ["abcabcabd", "abd"], ["abc", "c"], ["abc", "abc"],
          [$h, "needle"], [$h, "dc"], [$h, "x"], [$long, "needle"], [$long, "zzz"], [$long, "q"], ["a\0b", "\0b"], ["été", "t"], ["aXbXc", "X"]] as [$s, $n]) {
    echo b(strpos($s, $n)), ' ', b(str_contains($s, $n)), ' ', b(strstr($s, $n) === false ? false : strlen(strstr($s, $n))), ' ', b(strrpos($s, $n)), ' ', b(substr_count($s, $n)), ' ', b(stripos($s, strtoupper($n))), ' ', b(str_starts_with($s, $n)), ' ', b(str_ends_with($s, $n)), "\n";
}
echo b(strpos("abcabc", "c", 3)), ' ', b(strpos("abcabc", "c", -1)), ' ', b(strpos("abcabc", "a", -3)), ' ', b(str_contains("abc", "")), ' ', b(strpos("abc", "")), ' ', b(strstr("user@example.com", "@", true)), ' ', b(explode(",", "a,b,,c")), "\n";
try { strpos("abc", "b", 9); } catch (ValueError $e) { echo get_class($e), ': ', $e->getMessage(), "\n"; }
class E1 extends Exception {} class P1 { public $v = 1; function __destruct() { echo "d"; } }
for ($i = 0; $i < 3; $i++) { $p = new P1; $e = new E1("m$i"); echo ' ', $e->getLine(), ':', basename($e->getFile()) === basename(__FILE__) ? 'f' : 'x', ':', count($e->getTrace()), ' ', spl_object_id($p) > 0 ? 'id' : 'no'; $p = null; }
echo "\n";
function mk() { return new E1("in"); } $x = mk(); echo $x->getLine(), ' ', $x->getTrace()[0]['function'], "\n";
"#;
    let expected = r#"4 true 7 7 2 4 false false
6 true 5 6 1 6 false true
false false false false 0 false false false
2 true 3 3 2 2 false false
false false false false 0 false false false
0 true 3 1 1 0 true true
6 true 3 6 1 6 false true
2 true 1 2 1 2 false true
0 true 3 0 1 0 true true
200 true 206 200 1 200 false false
207 true 199 403 99 207 false false
false false false false 0 false false false
400 true 106 400 1 400 false false
406 true 100 503 33 406 false true
false false false false 0 false false false
1 true 2 1 1 1 false true
2 true 3 2 1 2 false false
1 true 4 3 2 1 false false
5 5 3 true 0 'user' array (
  0 => 'a',
  1 => 'b',
  2 => '',
  3 => 'c',
)
ValueError: strpos(): Argument #3 ($offset) must be contained in argument #1 ($haystack)
 12:f:0 idd 12:f:0 idd 12:f:0 idd
14 mk"#;
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
