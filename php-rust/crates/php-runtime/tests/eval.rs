//! Evaluator tests (plan step 4): assert exact stdout bytes for Tier 1 scripts.
//!
//! Scripts here are curated to be warning-free so stdout fully captures the
//! observable behaviour (diagnostic rendering is step 9). A couple of tests
//! check the collected `diags`/`fatal` channels directly.

use php_runtime::run_source;
use php_types::{Diag, PhpError, Zval};

/// Run a script and return its stdout as a UTF-8 string (panics on non-UTF-8,
/// which none of these warning-free scripts produce).
fn out(src: &str) -> String {
    let o = run_source(b"t.php", src.as_bytes()).expect("lowers");
    assert!(o.fatal.is_none(), "unexpected fatal: {:?}", o.fatal);
    String::from_utf8(o.stdout).expect("utf8")
}

#[test]
fn echo_literal_and_arithmetic() {
    assert_eq!(out("<?php echo 1 + 2;"), "3");
    assert_eq!(out("<?php echo 2 ** 10;"), "1024");
    assert_eq!(out("<?php echo 17 % 5;"), "2");
    assert_eq!(out("<?php echo -5 + 3;"), "-2");
    assert_eq!(out("<?php echo 7 / 2;"), "3.5");
}

#[test]
fn precedence_is_respected() {
    assert_eq!(out("<?php echo 1 + 2 * 3;"), "7");
    assert_eq!(out("<?php echo (1 + 2) * 3;"), "9");
}

#[test]
fn concatenation_and_coercion() {
    assert_eq!(out(r#"<?php echo "a" . "b" . 3;"#), "ab3");
    assert_eq!(out("<?php echo true; echo false; echo '|';"), "1|");
}

#[test]
fn float_precision_echo() {
    // echo uses precision=14, so 0.1 + 0.2 prints as 0.3 (not 0.30000000000000004).
    assert_eq!(out("<?php echo 0.1 + 0.2;"), "0.3");
}

#[test]
fn variables_and_assignment() {
    assert_eq!(out("<?php $x = 5; $y = $x * 2; echo $y;"), "10");
    // Assignment is an expression returning the assigned value.
    assert_eq!(out("<?php echo $x = 7;"), "7");
}

#[test]
fn compound_assignment() {
    assert_eq!(out("<?php $x = 10; $x += 5; $x *= 2; echo $x;"), "30");
    assert_eq!(out(r#"<?php $s = "a"; $s .= "b"; $s .= "c"; echo $s;"#), "abc");
}

#[test]
fn increment_decrement() {
    assert_eq!(out("<?php $x = 5; echo $x++; echo $x;"), "56"); // post: prints old, then 6
    assert_eq!(out("<?php $x = 5; echo ++$x; echo $x;"), "66"); // pre: prints new
    assert_eq!(out("<?php $x = 5; $x--; echo $x;"), "4");
}

#[test]
fn if_elseif_else() {
    let prog = "<?php $x = 3; if ($x > 5) echo 'a'; elseif ($x > 2) echo 'b'; else echo 'c';";
    assert_eq!(out(prog), "b");
    assert_eq!(out("<?php if (0) echo 'y'; else echo 'n';"), "n");
}

#[test]
fn while_sum() {
    assert_eq!(
        out("<?php $i = 1; $s = 0; while ($i <= 5) { $s += $i; $i++; } echo $s;"),
        "15"
    );
}

#[test]
fn do_while_runs_at_least_once() {
    assert_eq!(out("<?php $i = 0; do { echo $i; $i++; } while ($i < 3);"), "012");
    // body always runs once even when the condition is false up front
    assert_eq!(out("<?php do { echo 'x'; } while (false);"), "x");
}

#[test]
fn for_loop() {
    assert_eq!(out("<?php for ($i = 0; $i < 3; $i++) { echo $i; }"), "012");
}

#[test]
fn break_with_level() {
    // break 2 escapes both loops after printing "00".
    let prog = "<?php for ($i = 0; $i < 3; $i++) { for ($j = 0; $j < 3; $j++) { if ($j == 1) break 2; echo $i; echo $j; } } echo 'X';";
    assert_eq!(out(prog), "00X");
}

#[test]
fn continue_skips_iteration() {
    let prog = "<?php for ($i = 0; $i < 5; $i++) { if ($i % 2 == 0) continue; echo $i; }";
    assert_eq!(out(prog), "13");
}

#[test]
fn ternary_full_and_short() {
    assert_eq!(out("<?php echo true ? 'y' : 'n';"), "y");
    assert_eq!(out("<?php echo 0 ?: 'z';"), "z");
    assert_eq!(out("<?php echo 5 ?: 'z';"), "5");
}

#[test]
fn comparison_and_spaceship() {
    assert_eq!(out("<?php echo 1 <=> 2;"), "-1");
    assert_eq!(out("<?php echo (2 == 2.0) ? 't' : 'f';"), "t");
    assert_eq!(out("<?php echo (2 === 2.0) ? 't' : 'f';"), "f");
    // Numeric strings compare numerically: 9 < 10 is true, but '9' > '10' lexically.
    assert_eq!(out("<?php echo ('9' < '10') ? 't' : 'f';"), "t");
    assert_eq!(out("<?php echo ('10' < '9') ? 't' : 'f';"), "f");
}

#[test]
fn short_circuit_does_not_evaluate_rhs() {
    // The assignment in the RHS must not run.
    assert_eq!(out("<?php $x = 0; false && ($x = 1); echo $x;"), "0");
    assert_eq!(out("<?php $x = 0; true || ($x = 2); echo $x;"), "0");
}

#[test]
fn coalesce_suppresses_undefined() {
    let o = run_source(b"t.php", b"<?php echo $missing ?? 'default';").expect("lowers");
    assert_eq!(String::from_utf8(o.stdout).unwrap(), "default");
    // `??` is isset-like: no Undefined-variable warning is raised.
    assert!(o.diags.is_empty(), "?? should not warn: {:?}", o.diags);
}

#[test]
fn coalesce_assign() {
    assert_eq!(out("<?php $a = 'set'; $a ??= 'x'; echo $a;"), "set");
    assert_eq!(out("<?php $b ??= 'init'; echo $b;"), "init");
}

#[test]
fn casts() {
    assert_eq!(out("<?php echo (int)'42abc';"), "42");
    assert_eq!(out("<?php echo (int)3.9;"), "3");
    assert_eq!(out("<?php echo (float)'1.5';"), "1.5");
    assert_eq!(out("<?php echo (bool)0 ? 't' : 'f';"), "f");
}

#[test]
fn inline_html_interleaved() {
    assert_eq!(out("A<?php echo 1; ?>B<?php echo 2;"), "A1B2");
}

#[test]
fn undefined_variable_warns_and_yields_null() {
    let o = run_source(b"t.php", b"<?php echo $x;").expect("lowers");
    assert_eq!(o.stdout, b""); // null → empty string
    assert_eq!(o.diags.len(), 1);
    assert!(matches!(&o.diags[0], Diag::Warning(m) if m == "Undefined variable $x"));
}

#[test]
fn division_by_zero_is_fatal_after_partial_output() {
    let o = run_source(b"t.php", b"<?php echo 'before'; echo 1 / 0;").expect("lowers");
    // Output produced before the fatal is preserved.
    assert_eq!(o.stdout, b"before");
    assert!(matches!(o.fatal, Some(PhpError::DivisionByZeroError(_))));
}

#[test]
fn string_increment_value_and_deprecation_captured() {
    // 'a'++ yields 'b' (correct value) and, in PHP 8.5, a Deprecated diagnostic.
    // We capture the diagnostic in `diags`; rendering it onto stdout is step 9.
    let o = run_source(b"t.php", b"<?php $x = 'a'; $x++; echo $x;").expect("lowers");
    assert_eq!(o.stdout, b"b");
    assert!(
        o.diags.iter().any(|d| matches!(d, Diag::Deprecated(_))),
        "expected a Deprecated diag, got {:?}",
        o.diags
    );
}

#[test]
fn top_level_return_value() {
    let o = run_source(b"t.php", b"<?php echo 'hi'; return 42;").expect("lowers");
    assert_eq!(o.stdout, b"hi");
    assert!(matches!(o.return_value, Zval::Long(42)));
}

// --- step 7: arrays, foreach, switch, match, isset/empty/unset ---

#[test]
fn array_build_index_and_append() {
    assert_eq!(out("<?php $a = [10, 20, 30]; echo $a[1];"), "20");
    assert_eq!(out("<?php $a = []; $a[] = 'x'; $a[] = 'y'; echo $a[0] . $a[1];"), "xy");
    assert_eq!(out("<?php $a['k'] = 5; echo $a['k'];"), "5");
    // Append after an explicit key continues from max int key + 1.
    assert_eq!(out("<?php $a = [5 => 'a']; $a[] = 'b'; echo $a[6];"), "b");
}

#[test]
fn array_autovivification_nested() {
    assert_eq!(out("<?php $a['x']['y'] = 1; echo $a['x']['y'];"), "1");
    assert_eq!(out("<?php $a[][] = 'z'; echo $a[0][0];"), "z");
}

#[test]
fn array_copy_on_write_is_value_semantics() {
    // Assigning an array copies it; mutating the copy must not touch the original.
    assert_eq!(
        out("<?php $a = [1, 2]; $b = $a; $b[0] = 99; echo $a[0]; echo '/'; echo $b[0];"),
        "1/99"
    );
}

#[test]
fn compound_assign_on_element() {
    assert_eq!(out("<?php $a = [1]; $a[0] += 4; echo $a[0];"), "5");
    assert_eq!(out("<?php $a = ['s' => 'a']; $a['s'] .= 'bc'; echo $a['s'];"), "abc");
}

#[test]
fn foreach_value_and_keyvalue() {
    assert_eq!(out("<?php foreach ([1, 2, 3] as $v) { echo $v; }"), "123");
    assert_eq!(
        out("<?php foreach (['a' => 1, 'b' => 2] as $k => $v) { echo $k; echo $v; }"),
        "a1b2"
    );
}

#[test]
fn foreach_snapshot_is_stable_under_mutation() {
    // PHP iterates a copy: appending inside the body does not extend iteration.
    assert_eq!(
        out("<?php $a = [1, 2]; foreach ($a as $v) { $a[] = 9; echo $v; }"),
        "12"
    );
}

#[test]
fn switch_match_and_fallthrough_and_default() {
    assert_eq!(
        out("<?php switch (2) { case 1: echo 'a'; break; case 2: echo 'b'; break; default: echo 'd'; }"),
        "b"
    );
    // Fall-through: no break means execution continues into the next case.
    assert_eq!(
        out("<?php switch (1) { case 1: echo 'a'; case 2: echo 'b'; break; case 3: echo 'c'; }"),
        "ab"
    );
    assert_eq!(
        out("<?php switch (9) { case 1: echo 'a'; break; default: echo 'd'; }"),
        "d"
    );
    // Loose comparison: "1" == 1.
    assert_eq!(out("<?php switch ('1') { case 1: echo 'hit'; }"), "hit");
}

#[test]
fn switch_break_inside_loop() {
    // break 2 escapes both the switch and the enclosing loop.
    assert_eq!(
        out("<?php for ($i = 0; $i < 3; $i++) { switch ($i) { case 1: break 2; default: echo $i; } } echo 'X';"),
        "0X"
    );
}

#[test]
fn match_strict_and_multi_condition() {
    assert_eq!(out("<?php echo match (2) { 1, 2 => 'lo', 3 => 'hi' };"), "lo");
    assert_eq!(out("<?php echo match (5) { 1 => 'a', default => 'def' };"), "def");
    // Strict ===: the string "1" does not match the int 1.
    assert_eq!(out("<?php echo match ('1') { 1 => 'int', '1' => 'str' };"), "str");
}

#[test]
fn match_unhandled_is_fatal() {
    let o = run_source(b"t.php", b"<?php echo match (5) { 1 => 'a' };").expect("lowers");
    match o.fatal {
        Some(PhpError::Error(m)) => assert_eq!(m, "Unhandled match case 5"),
        other => panic!("expected UnhandledMatchError, got {other:?}"),
    }
}

#[test]
fn isset_empty_unset_semantics() {
    assert_eq!(out("<?php $a = [1, 0]; echo isset($a[0]) ? 'y' : 'n';"), "y");
    assert_eq!(out("<?php $a = [1]; echo isset($a[5]) ? 'y' : 'n';"), "n");
    // A null element is not considered set.
    assert_eq!(out("<?php $a = ['k' => null]; echo isset($a['k']) ? 'y' : 'n';"), "n");
    // empty(): 0 is empty, 1 is not.
    assert_eq!(out("<?php $a = ['z' => 0]; echo empty($a['z']) ? 'e' : 'f';"), "e");
    assert_eq!(out("<?php $a = ['z' => 1]; echo empty($a['z']) ? 'e' : 'f';"), "f");
    // unset removes the key.
    assert_eq!(out("<?php $a = [1, 2]; unset($a[0]); echo isset($a[0]) ? 'y' : 'n';"), "n");
}

#[test]
fn coalesce_on_array_element_is_silent() {
    // `$a[k] ?? d` must not warn on a missing key, and `??=` writes when absent.
    let o = run_source(b"t.php", b"<?php $a = []; echo $a['x'] ?? 'def';").expect("lowers");
    assert_eq!(o.stdout, b"def");
    assert!(o.diags.is_empty(), "expected no diags, got {:?}", o.diags);
    assert_eq!(out("<?php $a = []; $a['x'] ??= 7; echo $a['x'];"), "7");
}
