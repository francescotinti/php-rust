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

// --- step 6 regressions: bugs surfaced by the .phpt import ---

#[test]
fn coalesce_on_string_offset() {
    // Bug #69889: `??` on string offsets must treat out-of-range / non-integer
    // offsets as unset (fall through), not as the empty string / coerced char.
    assert_eq!(out(r#"<?php $s = "test"; echo $s[0] ?? "d";"#), "t");
    assert_eq!(out(r#"<?php $s = "test"; echo $s[5] ?? "d";"#), "d");
    assert_eq!(out(r#"<?php $s = "test"; echo $s["str"] ?? "d";"#), "d");
    // Negative offset in range still resolves.
    assert_eq!(out(r#"<?php $s = "test"; echo $s[-1] ?? "d";"#), "t");
}

#[test]
fn huge_integer_literal_overflows_to_inf() {
    // Bug #74947: a literal too large even for u64 promotes to float → INF,
    // not the u64-clamped ~1.8e19.
    let big = "2".to_string() + &"0".repeat(320);
    assert_eq!(out(&format!("<?php echo {big};")), "INF");
    assert_eq!(out(&format!("<?php echo -{big};")), "-INF");
    // A literal just past i64::MAX still promotes to the right finite float.
    assert_eq!(out("<?php echo 9223372036854775808;"), "9.2233720368548E+18");
}

// --- step 8: user-defined functions ---

#[test]
fn function_declare_and_call() {
    assert_eq!(
        out("<?php function greet() { echo 'hi'; } greet();"),
        "hi"
    );
    // A function with a single parameter and a return value.
    assert_eq!(
        out("<?php function sq($n) { return $n * $n; } echo sq(7);"),
        "49"
    );
}

#[test]
fn function_is_hoisted_before_declaration() {
    // PHP hoists top-level function declarations: a call may precede the body.
    assert_eq!(
        out("<?php echo f(); function f() { return 'ok'; }"),
        "ok"
    );
}

#[test]
fn function_name_is_case_insensitive() {
    assert_eq!(
        out("<?php function Foo() { return 'x'; } echo FOO(); echo foo();"),
        "xx"
    );
}

#[test]
fn function_local_scope_is_isolated() {
    // The function's `$x` is a distinct slot from the caller's `$x`.
    let prog = "<?php $x = 'outer'; function f() { $x = 'inner'; echo $x; } f(); echo $x;";
    assert_eq!(out(prog), "innerouter");
}

#[test]
fn function_default_parameter() {
    assert_eq!(
        out("<?php function inc($n, $by = 1) { return $n + $by; } echo inc(5); echo '/'; echo inc(5, 10);"),
        "6/15"
    );
    // A default that is a constant expression is evaluated at call time.
    assert_eq!(
        out("<?php function f($a = 2 * 3) { return $a; } echo f();"),
        "6"
    );
}

#[test]
fn function_extra_arguments_are_ignored() {
    // Without a variadic, surplus positional arguments are silently dropped.
    assert_eq!(
        out("<?php function f($a) { return $a; } echo f(1, 2, 3);"),
        "1"
    );
}

#[test]
fn function_missing_required_argument_is_fatal() {
    let o = run_source(b"t.php", b"<?php function f($a) { return $a; } f();").expect("lowers");
    match o.fatal {
        Some(PhpError::Error(m)) => assert!(
            m.contains("Too few arguments to function f()"),
            "unexpected message: {m}"
        ),
        other => panic!("expected ArgumentCountError, got {other:?}"),
    }
}

#[test]
fn function_recursion_factorial() {
    let prog = "<?php function fact($n) { return $n <= 1 ? 1 : $n * fact($n - 1); } echo fact(5);";
    assert_eq!(out(prog), "120");
}

#[test]
fn global_reads_and_writes_outer() {
    // `global $x` aliases the global cell; a write through it is visible outside
    // (step 12-2, oracle 9).
    assert_eq!(
        out("<?php $x = 5; function f(){ global $x; $x = 9; } f(); echo $x;"),
        "9"
    );
}

#[test]
fn global_reads_existing_value() {
    // Reading through `global` sees the current global value (oracle 42).
    assert_eq!(
        out("<?php $x = 42; function f(){ global $x; echo $x; } f();"),
        "42"
    );
}

#[test]
fn global_creates_global_from_function() {
    // A global declared & assigned only inside a function still exists at the top
    // level afterwards (oracle 7).
    assert_eq!(
        out("<?php function f(){ global $x; $x = 7; } f(); echo $x;"),
        "7"
    );
}

#[test]
fn global_persists_across_calls() {
    // The alias targets one shared cell, so repeated calls accumulate (oracle 3).
    assert_eq!(
        out("<?php $x = 1; function f(){ global $x; $x++; } f(); f(); echo $x;"),
        "3"
    );
}

#[test]
fn global_multiple_variables() {
    // `global $a, $b;` aliases each named global in one statement (oracle 3_99).
    assert_eq!(
        out("<?php $a = 1; $b = 2; function f(){ global $a, $b; $a = $a + $b; $b = 99; } \
             f(); echo $a; echo '_'; echo $b;"),
        "3_99"
    );
}

#[test]
fn globals_array_writes_outer() {
    // `$GLOBALS['x'] = 8` inside a function writes the global (oracle 8).
    assert_eq!(
        out("<?php $x = 3; function f(){ $GLOBALS['x'] = 8; } f(); echo $x;"),
        "8"
    );
}

#[test]
fn globals_array_reads_outer() {
    // Reading `$GLOBALS['x']` sees the global value (oracle 10).
    assert_eq!(
        out("<?php $x = 10; function f(){ echo $GLOBALS['x']; } f();"),
        "10"
    );
}

#[test]
fn globals_array_creates_global() {
    // Assigning a never-seen `$GLOBALS['n']` creates the bare global (oracle 5).
    assert_eq!(
        out("<?php function f(){ $GLOBALS['n'] = 5; } f(); echo $n;"),
        "5"
    );
}

#[test]
fn globals_array_compound_assignment() {
    // `$GLOBALS['x'] += 4` reads then writes the global cell (oracle 5).
    assert_eq!(
        out("<?php $x = 1; function f(){ $GLOBALS['x'] += 4; } f(); echo $x;"),
        "5"
    );
}

#[test]
fn globals_array_nested_element_write() {
    // `$GLOBALS['a'][0] = 9` reaches into the global array element (oracle 9).
    assert_eq!(
        out("<?php $a = [1, 2]; function f(){ $GLOBALS['a'][0] = 9; } f(); echo $a[0];"),
        "9"
    );
}

#[test]
fn globals_array_isset_is_silent() {
    // `isset($GLOBALS['z'])` is false with no warning for an unset global;
    // a set global is true (oracle nY).
    assert_eq!(
        out("<?php $x = 1; function f(){ \
             echo isset($GLOBALS['z']) ? 'y' : 'n'; \
             echo isset($GLOBALS['x']) ? 'Y' : 'N'; } f();"),
        "nY"
    );
}

#[test]
fn globals_array_at_top_level() {
    // At global scope `$GLOBALS['x']` is just the variable `$x` (oracle 7).
    assert_eq!(out("<?php $GLOBALS['x'] = 7; echo $x;"), "7");
}

#[test]
fn return_by_ref_aliases_global() {
    // `$y = &f()` on a by-ref function aliases the cell f returned, so writing
    // through $y writes the global (oracle 99).
    assert_eq!(
        out("<?php $x = 1; function &f(){ global $x; return $x; } \
             $y = &f(); $y = 99; echo $x;"),
        "99"
    );
}

#[test]
fn return_by_ref_aliases_array_element() {
    // A by-ref function returning an element of a by-ref param; aliasing the
    // result writes back into the caller's array (oracle 99).
    assert_eq!(
        out("<?php function &first(&$a){ return $a[0]; } \
             $arr = [10, 20]; $r = &first($arr); $r = 99; echo $arr[0];"),
        "99"
    );
}

#[test]
fn return_by_ref_in_value_context_derefs() {
    // Using a by-ref function's result in a value context yields a copy, not the
    // reference (oracle 5).
    assert_eq!(
        out("<?php $x = 5; function &f(){ global $x; return $x; } echo f();"),
        "5"
    );
}

#[test]
fn return_by_ref_assigned_by_value_copies() {
    // `$y = f()` (no `&`) copies even though f returns by reference, so a later
    // write to $y does not touch the global (oracle 1).
    assert_eq!(
        out("<?php $x = 1; function &f(){ global $x; return $x; } \
             $y = f(); $y = 99; echo $x;"),
        "1"
    );
}

#[test]
fn return_by_ref_non_lvalue_notices() {
    // A by-ref function returning a non-lvalue raises a Notice but still yields
    // the value (oracle: Notice + 3).
    let o = run_source(
        b"t.php",
        b"<?php function &f(){ return 1 + 2; } $y = &f(); echo $y;",
    )
    .expect("lowers");
    assert_eq!(String::from_utf8(o.stdout).unwrap(), "3");
    assert!(
        o.diags.iter().any(|d| matches!(d, Diag::Notice(m)
            if m == "Only variable references should be returned by reference")),
        "expected return-by-ref notice, got {:?}",
        o.diags
    );
}

#[test]
fn return_by_ref_bare_return_notices() {
    // Even a bare `return;` in a by-ref function raises the same Notice.
    let o = run_source(b"t.php", b"<?php function &f(){ return; } $y = &f();").expect("lowers");
    assert!(
        o.diags.iter().any(|d| matches!(d, Diag::Notice(m)
            if m == "Only variable references should be returned by reference")),
        "expected return-by-ref notice, got {:?}",
        o.diags
    );
}

#[test]
fn assign_ref_to_non_ref_function_notices() {
    // `$y = &f()` where f is NOT by-reference raises "Only variables should be
    // assigned by reference" and copies the value (oracle: Notice + 5).
    let o = run_source(
        b"t.php",
        b"<?php function f(){ return 5; } $y = &f(); echo $y;",
    )
    .expect("lowers");
    assert_eq!(String::from_utf8(o.stdout).unwrap(), "5");
    assert!(
        o.diags.iter().any(|d| matches!(d, Diag::Notice(m)
            if m == "Only variables should be assigned by reference")),
        "expected assign-by-ref notice, got {:?}",
        o.diags
    );
}

#[test]
fn function_mutual_recursion() {
    // Both functions are hoisted, so even-before-odd resolution works.
    let prog = "<?php \
        function is_even($n) { return $n == 0 ? true : is_odd($n - 1); } \
        function is_odd($n) { return $n == 0 ? false : is_even($n - 1); } \
        echo is_even(10) ? 'y' : 'n'; echo is_odd(7) ? 'y' : 'n';";
    assert_eq!(out(prog), "yy");
}

#[test]
fn function_falling_off_end_returns_null() {
    let o = run_source(b"t.php", b"<?php function f() { $x = 1; } return f();").expect("lowers");
    // `return f()` with no explicit return inside f yields NULL.
    assert!(matches!(o.return_value, Zval::Null));
}

#[test]
fn assignment_evaluates_lvalue_offsets_before_rhs() {
    // Bug surfaced by the .phpt import (engine_assignExecutionOrder_005): the
    // offset expressions of an assignment target run left-to-right *before* the
    // RHS. Chained `$a[f()] = $b[g()] = h()` must print f, g, h in order.
    let prog = "<?php \
        function p($s) { echo $s; return 0; } \
        $a = [[0]]; $b = [[1]]; $c = [[2]]; \
        $a[p('1')][p('2')] = $b[p('3')][p('4')] = $c[p('5')][p('6')]; \
        echo '='; echo $a[0][0]; echo $b[0][0];";
    assert_eq!(out(prog), "123456=22");
}

#[test]
fn scalar_hint_coerces_int_float_string_bool() {
    // Weak-mode scalar coercion of arguments to the parameter's declared type
    // (step 14, oracle-verified). Strict `===` proves the coerced *type*, not
    // just the value.
    assert_eq!(out("<?php function f(int $x){ echo $x === 123 ? 'Y' : 'N'; } f('123');"), "Y");
    assert_eq!(out("<?php function f(int $x){ echo $x === 1 ? 'Y' : 'N'; } f(true);"), "Y");
    assert_eq!(out("<?php function f(float $x){ echo $x === 7.0 ? 'Y' : 'N'; } f(7);"), "Y");
    assert_eq!(out("<?php function f(float $x){ echo $x === 1000.0 ? 'Y' : 'N'; } f('1e3');"), "Y");
    assert_eq!(out("<?php function f(string $x){ echo $x === '42' ? 'Y' : 'N'; } f(42);"), "Y");
    assert_eq!(out("<?php function f(string $x){ echo $x === '1' ? 'Y' : 'N'; } f(true);"), "Y");
    assert_eq!(out("<?php function f(bool $x){ echo $x === false ? 'Y' : 'N'; } f(0);"), "Y");
    assert_eq!(out("<?php function f(bool $x){ echo $x === true ? 'Y' : 'N'; } f('x');"), "Y");
}

#[test]
fn nullable_scalar_hint() {
    // `?int` accepts null verbatim, and still coerces a non-null argument.
    assert_eq!(out("<?php function f(?int $x){ echo $x === null ? 'Y' : 'N'; } f(null);"), "Y");
    assert_eq!(out("<?php function f(?int $x){ echo $x === 5 ? 'Y' : 'N'; } f('5');"), "Y");
}

#[test]
fn scalar_hint_type_error_message() {
    // A non-coercible argument raises a TypeError with PHP's exact message
    // (oracle-verified; "Command line code" becomes the test file name).
    let o = run_source(
        b"t.php",
        b"<?php function f(int $x){ return $x; } f('abc');",
    )
    .expect("lowers");
    match &o.fatal {
        Some(PhpError::TypeError(m)) => assert_eq!(
            m,
            "f(): Argument #1 ($x) must be of type int, string given, \
             called in t.php on line 1 and defined in t.php:1"
        ),
        other => panic!("expected TypeError, got {other:?}"),
    }
}

#[test]
fn scalar_hint_type_error_null_and_array() {
    // null and array report their PHP type names in the message.
    let o = run_source(b"t.php", b"<?php function f(int $x){} f(null);").expect("lowers");
    assert!(
        matches!(&o.fatal, Some(PhpError::TypeError(m)) if m.contains("must be of type int, null given")),
        "got {:?}",
        o.fatal
    );
    let o = run_source(b"t.php", b"<?php function f(int $x){} f([1]);").expect("lowers");
    assert!(
        matches!(&o.fatal, Some(PhpError::TypeError(m)) if m.contains("must be of type int, array given")),
        "got {:?}",
        o.fatal
    );
}

#[test]
fn nullable_hint_type_error_shows_question_mark() {
    // The nullable hint is rendered with its `?` in the error message.
    let o = run_source(b"t.php", b"<?php function f(?int $x){} f('z');").expect("lowers");
    assert!(
        matches!(&o.fatal, Some(PhpError::TypeError(m)) if m.contains("must be of type ?int, string given")),
        "got {:?}",
        o.fatal
    );
}

// --- step 9: diagnostic & fatal rendering (interleaved onto the CLI stream) ---

/// Run a script and return its `rendered` (CLI-faithful) stream as a string.
fn rendered(src: &str) -> String {
    let o = run_source(b"t.php", src.as_bytes()).expect("lowers");
    String::from_utf8(o.rendered).expect("utf8")
}

#[test]
fn rendered_clean_script_equals_stdout() {
    let o = run_source(b"t.php", b"<?php echo 'hi'; echo 1 + 2;").expect("lowers");
    assert_eq!(o.rendered, o.stdout);
    assert_eq!(o.rendered, b"hi3");
}

#[test]
fn rendered_warning_is_interleaved_at_point_of_occurrence() {
    // echo "a"; (l2) echo $undef; (l3) echo "b"; (l4) — the warning lands
    // between "a" and "b", carrying line 3, with a leading + trailing newline.
    let src = "<?php\necho 'a';\necho $undef;\necho 'b';\n";
    assert_eq!(
        rendered(src),
        "a\nWarning: Undefined variable $undef in t.php on line 3\nb"
    );
}

#[test]
fn rendered_undefined_array_key_carries_its_line() {
    let src = "<?php\n$a = [1];\necho $a[5];\n";
    assert_eq!(
        rendered(src),
        "\nWarning: Undefined array key 5 in t.php on line 3\n"
    );
}

#[test]
fn rendered_array_to_string_warning_precedes_the_text() {
    // The "Array to string conversion" warning is emitted before the literal
    // "Array" that the conversion yields.
    let src = "<?php\n$a = [1, 2];\necho $a;\n";
    assert_eq!(
        rendered(src),
        "\nWarning: Array to string conversion in t.php on line 3\nArray"
    );
}

#[test]
fn rendered_fatal_appended_after_partial_output() {
    let src = "<?php\necho 'before';\n$x = 1 % 0;\necho 'after';\n";
    assert_eq!(
        rendered(src),
        "before\nFatal error: Uncaught DivisionByZeroError: Modulo by zero in t.php:3\n\
         Stack trace:\n#0 {main}\n  thrown in t.php on line 3\n"
    );
}

#[test]
fn rendered_null_array_offset_deprecation() {
    // Using null as an array offset is deprecated (PHP 8.1+); the key resolves to
    // the empty string, so the write still lands and is read back.
    let src = "<?php\n$a = [];\n$a[null] = 'v';\necho $a[''];\n";
    assert_eq!(
        rendered(src),
        "\nDeprecated: Using null as an array offset is deprecated, use an empty string instead in t.php on line 3\nv"
    );
}

// --- foreach by reference (step 11d-3): `foreach ($a as &$v)` ---

#[test]
fn foreach_by_ref_mutates_source_array() {
    // `&$v` aliases each element, so the body's writes land in the source array.
    assert_eq!(
        out("<?php $a=[1,2,3]; foreach($a as &$v){$v*=10;} unset($v); echo $a[0]; echo $a[1]; echo $a[2];"),
        "102030"
    );
}

#[test]
fn foreach_by_ref_lingering_reference_gotcha() {
    // After a by-ref loop without unset, `$v` still references the last element;
    // a following by-value loop writes through it (the classic PHP gotcha).
    assert_eq!(
        out("<?php $a=[1,2,3]; foreach($a as &$v){} foreach($a as $v){} echo $a[0]; echo $a[1]; echo $a[2];"),
        "122"
    );
}

#[test]
fn foreach_by_ref_with_key() {
    // The key stays by value while `&$v` is by reference.
    assert_eq!(
        out("<?php $a=['x'=>1,'y'=>2]; foreach($a as $k=>&$v){$v=$k;} unset($v); echo $a['x']; echo $a['y'];"),
        "xy"
    );
}

#[test]
fn foreach_by_ref_over_temporary_is_tolerated() {
    // PHP permits `foreach (expr as &$v)` over a non-lvalue; the mutations are
    // simply lost. We must not error.
    assert_eq!(out("<?php foreach([1,2,3] as &$v){$v*=2;} echo 'ok';"), "ok");
}

// --- element-level references (step 11d-2): `$x = &$a[0]`, `$a[0] = &$x` ---

#[test]
fn ref_to_array_element_aliases_it() {
    // `$x = &$a[i]` aliases the element; writing $x writes the element.
    assert_eq!(out("<?php $a=[1,2]; $x=&$a[1]; $x=99; echo $a[1];"), "99");
}

#[test]
fn ref_to_array_element_vivifies() {
    // Binding a reference to a missing element creates it (as NULL, then written).
    assert_eq!(out("<?php $a=[]; $x=&$a['k']; $x=5; echo $a['k'];"), "5");
}

#[test]
fn array_element_ref_to_variable() {
    // `$a[0] = &$x` makes the element alias the variable.
    assert_eq!(out("<?php $x=5; $a=[1,2]; $a[0]=&$x; $x=9; echo $a[0];"), "9");
}

#[test]
fn array_append_ref_to_variable() {
    // `$a[] = &$x` appends a reference element.
    assert_eq!(out("<?php $x=7; $a=[]; $a[]=&$x; $x=8; echo $a[0];"), "8");
}

#[test]
fn ref_to_nested_array_element() {
    assert_eq!(out("<?php $a=[[1]]; $x=&$a[0][0]; $x=9; echo $a[0][0];"), "9");
}

#[test]
fn write_through_existing_ref_element() {
    // Writing `$a[0]` when it is already a reference element writes through the
    // shared cell, so an alias of that element sees the new value.
    assert_eq!(out("<?php $a=[1,2,3]; $b=&$a[0]; $a[0]=50; echo $b;"), "50");
}

#[test]
fn unset_ref_element_keeps_alias_value() {
    // `unset($a[0])` drops the element, but an alias of it keeps the value
    // through the shared cell (oracle: "gone1").
    assert_eq!(
        out("<?php $a=[1,2]; $x=&$a[0]; unset($a[0]); echo isset($a[0])?'set':'gone'; echo $x;"),
        "gone1"
    );
}

// --- by-reference parameters (step 11b): `function f(&$x)` ---

#[test]
fn byref_param_mutates_caller_variable() {
    // A `&$x` parameter binds the caller's variable, so the callee's write is
    // visible after the call returns (D-R6).
    assert_eq!(out("<?php function inc(&$x){$x++;} $n=1; inc($n); echo $n;"), "2");
}

#[test]
fn byref_param_defines_undefined_caller_variable() {
    // Passing an undefined variable by reference defines it (NULL → written),
    // with no undefined-variable warning.
    assert_eq!(out("<?php function f(&$x){$x=10;} f($y); echo $y;"), "10");
}

#[test]
fn byref_params_swap() {
    // Two by-ref params let a callee swap the caller's variables.
    assert_eq!(
        out("<?php function swap(&$a,&$b){$t=$a;$a=$b;$b=$t;} $x=1;$y=2; swap($x,$y); echo $x; echo $y;"),
        "21"
    );
}

#[test]
fn byvalue_param_leaves_caller_untouched() {
    // The contrast case: a plain (by-value) param does not write back.
    assert_eq!(out("<?php function g($x){$x++;} $n=1; g($n); echo $n;"), "1");
}

#[test]
fn byref_param_nonvariable_argument_is_fatal() {
    // PHP 8.x: passing a non-variable (here a literal) to a by-ref parameter is
    // an uncaught Error (oracle-verified message).
    let o = run_source(b"t.php", b"<?php function inc(&$x){$x++;} inc(5);")
        .expect("lowers");
    let err = o.fatal.expect("expected a fatal error");
    assert_eq!(
        err.message(),
        "inc(): Argument #1 ($x) could not be passed by reference"
    );
}

// --- references (step 11a): variable-level `&` binding ---

#[test]
fn reference_write_through_both_directions() {
    // `$b = &$a` aliases the two variables: a later write to either is visible
    // through the other (D-R3 write-through).
    assert_eq!(out("<?php $a = 1; $b = &$a; $a = 5; echo $b;"), "5");
    assert_eq!(out("<?php $a = 1; $b = &$a; $b = 7; echo $a;"), "7");
}

#[test]
fn reference_to_undefined_defines_null() {
    // Binding a reference to an undefined variable defines it as NULL with no
    // "undefined variable" warning (oracle-verified).
    assert_eq!(
        out("<?php $b = &$a; echo $b === null ? 'null' : 'other';"),
        "null"
    );
}

#[test]
fn reference_chain_shares_one_cell() {
    // `$c = &$b` where `$b` is already a reference joins the same binding, so a
    // write through `$c` reaches `$a`.
    assert_eq!(out("<?php $a = 1; $b = &$a; $c = &$b; $c = 8; echo $a;"), "8");
}

#[test]
fn reference_unset_breaks_only_that_alias() {
    // `unset($b)` drops the alias but leaves the shared value intact for `$a`.
    assert_eq!(
        out("<?php $a = 1; $b = &$a; unset($b); $a = 9; echo $b ?? 'gone';"),
        "gone"
    );
    // `unset($a)` likewise leaves `$b` holding the value.
    assert_eq!(out("<?php $a = 1; $b = &$a; unset($a); $b = 3; echo $b;"), "3");
}

#[test]
fn rendered_warning_then_fatal_in_order() {
    // The undefined-variable warning (line 2) renders before the fatal (line 3).
    let src = "<?php\necho $undef;\n$x = 5 / 0;\n";
    assert_eq!(
        rendered(src),
        "\nWarning: Undefined variable $undef in t.php on line 2\n\
         \nFatal error: Uncaught DivisionByZeroError: Division by zero in t.php:3\n\
         Stack trace:\n#0 {main}\n  thrown in t.php on line 3\n"
    );
}
