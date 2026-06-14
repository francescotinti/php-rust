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
fn float_to_int_param_deprecation() {
    // A lossy float → int coercion deprecates (oracle: "...from float 3.7...").
    let o = run_source(b"t.php", b"<?php function f(int $x){} f(3.7);").expect("lowers");
    assert!(o.fatal.is_none(), "unexpected fatal: {:?}", o.fatal);
    assert!(
        o.diags.iter().any(|d| matches!(d, Diag::Deprecated(m)
            if m == "Implicit conversion from float 3.7 to int loses precision")),
        "got {:?}",
        o.diags
    );
}

#[test]
fn float_string_to_int_param_deprecation() {
    // A lossy numeric float-string → int coercion deprecates with the distinct
    // "float-string" wording (oracle).
    let o = run_source(b"t.php", b"<?php function f(int $x){} f('1.5');").expect("lowers");
    assert!(
        o.diags.iter().any(|d| matches!(d, Diag::Deprecated(m)
            if m == "Implicit conversion from float-string \"1.5\" to int loses precision")),
        "got {:?}",
        o.diags
    );
}

#[test]
fn exact_float_to_int_param_no_deprecation() {
    // 3.0 and "1.0" convert without precision loss, so no deprecation fires.
    let o = run_source(b"t.php", b"<?php function f(int $x){} f(3.0); f('1.0');").expect("lowers");
    assert!(
        !o.diags.iter().any(|d| matches!(d, Diag::Deprecated(_))),
        "unexpected deprecation: {:?}",
        o.diags
    );
}

#[test]
fn default_value_is_coerced_to_hint() {
    // The D-NEW-6 case: an int default on a `float` parameter is coerced to
    // float when the default is used (oracle: float(0)).
    assert_eq!(out("<?php function f(float $n = 0){ echo $n === 0.0 ? 'Y' : 'N'; } f();"), "Y");
}

#[test]
fn return_type_hint_coerces() {
    // A scalar return type coerces the returned value (weak), proven by `===`.
    assert_eq!(out("<?php function f(): int { return '5'; } echo f() === 5 ? 'Y' : 'N';"), "Y");
    assert_eq!(
        out("<?php function f(): string { return 42; } echo f() === '42' ? 'Y' : 'N';"),
        "Y"
    );
}

#[test]
fn return_type_hint_error_message() {
    // A non-coercible return value raises a TypeError with PHP's return-value
    // message (distinct format from the argument one).
    let o = run_source(b"t.php", b"<?php function f(): int { return 'x'; } f();").expect("lowers");
    match &o.fatal {
        Some(PhpError::TypeError(m)) => assert_eq!(
            m,
            "f(): Return value must be of type int, string returned in t.php:1"
        ),
        other => panic!("expected TypeError, got {other:?}"),
    }
}

#[test]
fn static_var_accumulates_across_calls() {
    // `static $n` is initialised once and persists across calls (oracle 123).
    assert_eq!(
        out("<?php function f(){ static $n = 0; echo ++$n; } f(); f(); f();"),
        "123"
    );
}

#[test]
fn static_var_shared_across_recursion() {
    // The static cell is shared by every recursive frame (oracle 4).
    assert_eq!(
        out("<?php function f($d){ static $n = 0; $n++; if ($d > 0) f($d - 1); return $n; } \
             echo f(3);"),
        "4"
    );
}

#[test]
fn static_var_is_per_function() {
    // Each function has its own static storage (oracle 11012).
    assert_eq!(
        out("<?php function f(){ static $n = 0; echo ++$n; } \
             function g(){ static $n = 100; echo ++$n; } \
             f(); g(); f();"),
        "11012"
    );
}

#[test]
fn static_var_without_initializer_is_null_then_persists() {
    // `static $a;` starts as null, then keeps whatever it was set to (oracle YN).
    assert_eq!(
        out("<?php function f(){ static $a; echo $a === null ? 'Y' : 'N'; $a = 1; } f(); f();"),
        "YN"
    );
}

#[test]
fn static_var_initializer_runs_once() {
    // A non-constant initializer is evaluated only on the first call, so a later
    // change to what it read does not re-run it (oracle 1111).
    assert_eq!(
        out("<?php $g = 10; function f(){ static $x = $GLOBALS['g'] + 1; echo $x; } \
             f(); $GLOBALS['g'] = 99; f();"),
        "1111"
    );
}

#[test]
fn static_var_multiple_items() {
    // `static $a = 1, $b = 2;` declares several statics in one statement (oracle 35).
    assert_eq!(
        out("<?php function f(){ static $a = 1, $b = 2; echo $a + $b; $a++; $b++; } f(); f();"),
        "35"
    );
}

#[test]
fn strict_types_accepts_exact_type() {
    // Under strict_types an exactly-typed argument passes unchanged.
    assert_eq!(
        out("<?php declare(strict_types=1); function f(int $x){ echo $x; } f(5);"),
        "5"
    );
}

#[test]
fn strict_types_widens_int_to_float() {
    // The one implicit conversion allowed in strict mode: int → float widening.
    assert_eq!(
        out("<?php declare(strict_types=1); function f(float $x){ echo $x === 5.0 ? 'Y' : 'N'; } f(5);"),
        "Y"
    );
}

#[test]
fn strict_types_nullable_accepts_null() {
    assert_eq!(
        out("<?php declare(strict_types=1); function f(?int $x){ echo $x === null ? 'Y' : 'N'; } f(null);"),
        "Y"
    );
}

#[test]
fn strict_types_rejects_coercible_string() {
    // A numeric string is rejected in strict mode (no coercion), unlike weak.
    let o = run_source(
        b"t.php",
        b"<?php declare(strict_types=1); function f(int $x){} f('5');",
    )
    .expect("lowers");
    assert!(
        matches!(&o.fatal, Some(PhpError::TypeError(m)) if m.contains("must be of type int, string given")),
        "got {:?}",
        o.fatal
    );
}

#[test]
fn strict_types_rejects_float_to_int() {
    let o = run_source(
        b"t.php",
        b"<?php declare(strict_types=1); function f(int $x){} f(5.0);",
    )
    .expect("lowers");
    assert!(
        matches!(&o.fatal, Some(PhpError::TypeError(m)) if m.contains("must be of type int, float given")),
        "got {:?}",
        o.fatal
    );
}

#[test]
fn strict_types_rejects_int_to_string() {
    let o = run_source(
        b"t.php",
        b"<?php declare(strict_types=1); function f(string $x){} f(5);",
    )
    .expect("lowers");
    assert!(
        matches!(&o.fatal, Some(PhpError::TypeError(m)) if m.contains("must be of type string, int given")),
        "got {:?}",
        o.fatal
    );
}

#[test]
fn strict_types_return_value_is_strict() {
    let o = run_source(
        b"t.php",
        b"<?php declare(strict_types=1); function f(): int { return 'x'; } f();",
    )
    .expect("lowers");
    assert!(
        matches!(&o.fatal, Some(PhpError::TypeError(m)) if m.contains("Return value must be of type int, string returned")),
        "got {:?}",
        o.fatal
    );
}

#[test]
fn strict_types_zero_is_weak() {
    // `declare(strict_types=0)` is the default weak mode: coercion still happens.
    assert_eq!(
        out("<?php declare(strict_types=0); function f(int $x){ echo $x === 5 ? 'Y' : 'N'; } f('5');"),
        "Y"
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

// --- step 18-1: closures, use captures, dynamic calls ---

#[test]
fn closure_basic_call() {
    assert_eq!(out("<?php $f = function(){ return 42; }; echo $f();"), "42");
    assert_eq!(
        out("<?php $f = function($a, $b){ return $a + $b; }; echo $f(3, 4);"),
        "7"
    );
}

#[test]
fn closure_default_parameter() {
    assert_eq!(
        out("<?php $f = function($a, $b = 10){ return $a + $b; }; echo $f(5);"),
        "15"
    );
}

#[test]
fn closure_use_by_value_captures_at_definition() {
    // A by-value `use` snapshots the variable at definition; a later write to the
    // outer variable does not change what the closure sees.
    assert_eq!(
        out("<?php $x = 10; $f = function() use ($x) { return $x; }; $x = 20; echo $f();"),
        "10"
    );
}

#[test]
fn closure_use_by_reference_sees_later_writes() {
    assert_eq!(
        out("<?php $x = 10; $f = function() use (&$x) { return $x; }; $x = 20; echo $f();"),
        "20"
    );
}

#[test]
fn closure_use_by_reference_writes_through() {
    // Writing the captured reference inside the closure is visible outside.
    assert_eq!(
        out("<?php $x = 1; $f = function() use (&$x) { $x = 99; }; $f(); echo $x;"),
        "99"
    );
}

#[test]
fn closure_immediately_invoked() {
    assert_eq!(out("<?php echo (function($x){ return $x * 2; })(21);"), "42");
}

#[test]
fn closure_stored_in_array_and_called() {
    assert_eq!(
        out(r#"<?php $a = ["f" => function($x){ return $x + 1; }]; echo $a["f"](9);"#),
        "10"
    );
}

#[test]
fn closure_nested_capture_chain() {
    // A closure that returns a closure capturing the outer parameter by value.
    assert_eq!(
        out(
            "<?php $mk = function($n){ return function($x) use ($n) { return $x + $n; }; }; \
             $add5 = $mk(5); echo $add5(10);"
        ),
        "15"
    );
}

#[test]
fn closure_does_not_capture_implicitly() {
    // A plain `function` does NOT see the enclosing scope without `use`.
    let o = run_source(
        b"t.php",
        b"<?php $x = 5; $f = function(){ return $x; }; echo $f();",
    )
    .expect("lowers");
    // `$x` inside is undefined -> NULL, echoes nothing; a warning is raised.
    assert_eq!(String::from_utf8(o.stdout).unwrap(), "");
    assert!(o.fatal.is_none());
}

#[test]
fn dynamic_call_on_non_callable_is_fatal() {
    let o = run_source(b"t.php", b"<?php $f = 5; $f();").expect("lowers");
    match o.fatal {
        Some(PhpError::Error(m)) => assert_eq!(m, "Value of type int is not callable"),
        other => panic!("expected Error, got {other:?}"),
    }
}

#[test]
fn closure_too_few_arguments_is_fatal() {
    let o = run_source(
        b"t.php",
        b"<?php $f = function($a, $b){ return $a + $b; }; $f(1);",
    )
    .expect("lowers");
    match o.fatal {
        Some(PhpError::Error(m)) => {
            assert!(m.contains("Too few arguments"), "{m}");
            assert!(m.contains("{closure"), "{m}");
        }
        other => panic!("expected Error, got {other:?}"),
    }
}

// --- step 18-2: arrow functions ---

#[test]
fn arrow_basic() {
    assert_eq!(out("<?php $f = fn($x) => $x * 2; echo $f(21);"), "42");
    assert_eq!(out("<?php $f = fn($a, $b) => $a + $b; echo $f(3, 4);"), "7");
}

#[test]
fn arrow_auto_captures_by_value() {
    // `fn` auto-captures `$a` by value at definition; the later write to `$a`
    // does not change what the arrow sees.
    assert_eq!(
        out("<?php $a = 5; $f = fn($y) => $a + $y; $a = 100; echo $f(3);"),
        "8"
    );
}

#[test]
fn arrow_captures_with_no_params() {
    assert_eq!(out("<?php $x = 7; $f = fn() => $x; echo $f();"), "7");
}

#[test]
fn arrow_immediately_invoked() {
    assert_eq!(out("<?php echo (fn($x) => $x + 1)(9);"), "10");
}

#[test]
fn arrow_nested_capture_is_transitive() {
    // The outer arrow must capture `$a`/`$b` so the inner arrow can see them.
    assert_eq!(
        out("<?php $a = 1; $b = 2; $f = fn() => fn() => $a + $b; echo $f()();"),
        "3"
    );
}

#[test]
fn arrow_returns_closure_that_captures() {
    // An arrow whose body is a `function () use (...)` — the closure captures
    // from the arrow's frame (so the arrow must capture from its enclosing one).
    assert_eq!(
        out(
            "<?php $mk = fn($n) => function($x) use ($n) { return $x + $n; }; \
             $add5 = $mk(5); echo $add5(100);"
        ),
        "105"
    );
}

// --- step 18-3: string callables, is_callable, call_user_func[_array] ---

#[test]
fn string_callable_to_user_function() {
    assert_eq!(
        out("<?php function inc($x){ return $x + 1; } $f = 'inc'; echo $f(5);"),
        "6"
    );
}

#[test]
fn call_user_func_with_closure() {
    assert_eq!(out("<?php echo call_user_func(fn($x) => $x * 2, 21);"), "42");
}

#[test]
fn call_user_func_with_user_function_name() {
    assert_eq!(
        out("<?php function dbl($x){ return $x * 2; } echo call_user_func('dbl', 21);"),
        "42"
    );
}

#[test]
fn call_user_func_array_with_closure() {
    assert_eq!(
        out("<?php echo call_user_func_array(fn($a, $b) => $a + $b, [3, 4]);"),
        "7"
    );
}

#[test]
fn is_callable_closure_and_non_callable() {
    assert_eq!(out("<?php echo is_callable(fn() => 1) ? 'y' : 'n';"), "y");
    assert_eq!(
        out("<?php $f = function(){}; echo is_callable($f) ? 'y' : 'n';"),
        "y"
    );
    assert_eq!(out("<?php echo is_callable(5) ? 'y' : 'n';"), "n");
    assert_eq!(out("<?php echo is_callable('nope_xyz') ? 'y' : 'n';"), "n");
}

#[test]
fn is_callable_user_function_name() {
    assert_eq!(
        out("<?php function foo(){} echo is_callable('foo') ? 'y' : 'n';"),
        "y"
    );
}

#[test]
fn callable_type_hint_accepts_closure() {
    assert_eq!(
        out("<?php function apply(callable $f, $v){ return $f($v); } echo apply(fn($x) => $x * 3, 4);"),
        "12"
    );
}

// --- step 18-4: named engine constants (ConstFetch) ---

#[test]
fn const_php_int_limits() {
    assert_eq!(out("<?php echo PHP_INT_MAX;"), "9223372036854775807");
    assert_eq!(out("<?php echo PHP_INT_MIN;"), "-9223372036854775808");
    assert_eq!(out("<?php echo PHP_INT_SIZE;"), "8");
}

#[test]
fn const_str_pad_flags() {
    assert_eq!(
        out("<?php echo STR_PAD_LEFT, '|', STR_PAD_RIGHT, '|', STR_PAD_BOTH;"),
        "0|1|2"
    );
}

#[test]
fn const_array_filter_and_sort_flags() {
    assert_eq!(
        out("<?php echo ARRAY_FILTER_USE_KEY, '|', ARRAY_FILTER_USE_BOTH;"),
        "2|1"
    );
    assert_eq!(out("<?php echo SORT_STRING, SORT_FLAG_CASE;"), "28");
    assert_eq!(out("<?php echo COUNT_RECURSIVE;"), "1");
}

#[test]
fn const_php_eol_is_newline() {
    assert_eq!(out("<?php echo PHP_EOL === \"\\n\" ? 'y' : 'n';"), "y");
}

#[test]
fn const_math_pi() {
    assert_eq!(out("<?php echo M_PI > 3.14 && M_PI < 3.15 ? 'y' : 'n';"), "y");
}

#[test]
fn const_true_false_null_case_insensitive() {
    assert_eq!(out("<?php echo TRUE === true ? 'a' : 'b';"), "a");
    assert_eq!(out("<?php echo NULL === null ? 'a' : 'b';"), "a");
}

#[test]
fn unknown_constant_is_unsupported() {
    // User-defined constants are not lowered yet: the script becomes a SKIP.
    assert!(run_source(b"t.php", b"<?php echo NOPE_UNDEFINED_CONST;").is_err());
}

// --- step 18-5: array_map / array_filter / usort ---

#[test]
fn array_map_single_array() {
    assert_eq!(
        out("<?php $r = array_map(fn($x) => $x * 2, [1, 2, 3]); echo $r[0], $r[1], $r[2];"),
        "246"
    );
}

#[test]
fn array_map_preserves_string_keys() {
    assert_eq!(
        out("<?php $r = array_map(fn($x) => $x + 1, ['a' => 1, 'b' => 2]); echo $r['a'], $r['b'];"),
        "23"
    );
}

#[test]
fn array_map_multiple_arrays_reindexes() {
    assert_eq!(
        out("<?php $r = array_map(fn($a, $b) => $a + $b, [1, 2], [10, 20]); echo $r[0], '|', $r[1];"),
        "11|22"
    );
}

#[test]
fn array_map_null_callback_zips() {
    assert_eq!(
        out("<?php $r = array_map(null, [1, 2], [3, 4]); echo $r[0][0], $r[0][1], $r[1][0], $r[1][1];"),
        "1324"
    );
}

#[test]
fn array_filter_with_callback_preserves_keys() {
    assert_eq!(
        out("<?php $r = array_filter([1, 2, 3, 4], fn($x) => $x % 2 == 0); echo $r[1], $r[3];"),
        "24"
    );
}

#[test]
fn array_filter_no_callback_keeps_truthy() {
    assert_eq!(
        out("<?php $r = array_filter([0, 1, '', 2, null, 3]); echo $r[1], $r[3], $r[5];"),
        "123"
    );
}

#[test]
fn array_filter_use_key_mode() {
    assert_eq!(
        out(
            "<?php $r = array_filter(['a' => 1, 'b' => 2, 'c' => 3], \
             fn($k) => $k !== 'b', ARRAY_FILTER_USE_KEY); \
             echo $r['a'], $r['c'], isset($r['b']) ? 'set' : 'unset';"
        ),
        "13unset"
    );
}

#[test]
fn array_filter_use_both_mode() {
    assert_eq!(
        out(
            "<?php $r = array_filter(['a' => 1, 'b' => 2], \
             fn($v, $k) => $v > 1 && $k === 'b', ARRAY_FILTER_USE_BOTH); \
             echo $r['b'], isset($r['a']) ? 'set' : 'unset';"
        ),
        "2unset"
    );
}

#[test]
fn usort_ascending_and_return_value() {
    assert_eq!(
        out("<?php $a = [3, 1, 2]; $ok = usort($a, fn($x, $y) => $x <=> $y); echo $a[0], $a[1], $a[2], $ok ? 'y' : 'n';"),
        "123y"
    );
}

#[test]
fn usort_descending() {
    assert_eq!(
        out("<?php $a = [1, 2, 3]; usort($a, fn($x, $y) => $y <=> $x); echo $a[0], $a[1], $a[2];"),
        "321"
    );
}

#[test]
fn usort_reindexes_keys() {
    assert_eq!(
        out("<?php $a = ['x' => 3, 'y' => 1]; usort($a, fn($p, $q) => $p <=> $q); echo $a[0], $a[1];"),
        "13"
    );
}

// --- step 18-6: first-class callable syntax f(...) ---

#[test]
fn first_class_callable_user_function() {
    assert_eq!(
        out("<?php function dbl($x){ return $x * 2; } $f = dbl(...); echo $f(21);"),
        "42"
    );
}

#[test]
fn first_class_callable_in_usort() {
    assert_eq!(
        out(
            "<?php function cmp($a, $b){ return $a <=> $b; } \
             $a = [3, 1, 2]; usort($a, cmp(...)); echo $a[0], $a[1], $a[2];"
        ),
        "123"
    );
}

#[test]
fn first_class_callable_is_callable() {
    assert_eq!(
        out("<?php function foo(){} echo is_callable(foo(...)) ? 'y' : 'n';"),
        "y"
    );
}

// --- Step 19-1: OOP infrastructure (classes, new, $this, methods, properties) ---

#[test]
fn oop_method_call_basic() {
    assert_eq!(
        out("<?php class A { function m() { return 5; } } $a = new A; echo $a->m();"),
        "5"
    );
}

#[test]
fn oop_new_with_args_and_constructor() {
    assert_eq!(
        out(
            "<?php class Point { public $x; public $y; \
             function __construct($x, $y) { $this->x = $x; $this->y = $y; } \
             function dist() { return $this->x + $this->y; } } \
             $p = new Point(3, 4); echo $p->dist();"
        ),
        "7"
    );
}

#[test]
fn oop_new_without_parens() {
    assert_eq!(
        out("<?php class A { function m() { return 9; } } $a = new A; echo $a->m();"),
        "9"
    );
}

#[test]
fn oop_object_handle_semantics() {
    // Assigning an object copies the *handle*: both variables see the mutation.
    assert_eq!(
        out(
            "<?php class C { public $x = 1; } \
             $p = new C; $q = $p; $q->x = 99; echo $p->x;"
        ),
        "99"
    );
}

#[test]
fn oop_property_default_value() {
    assert_eq!(
        out("<?php class A { public $n = 5; } $a = new A; echo $a->n;"),
        "5"
    );
}

#[test]
fn oop_method_calls_method_via_this() {
    assert_eq!(
        out(
            "<?php class A { function a() { return $this->b() + 1; } \
             function b() { return 10; } } echo (new A)->a();"
        ),
        "11"
    );
}

#[test]
fn oop_property_read_and_write_via_this() {
    assert_eq!(
        out(
            "<?php class Counter { public $n = 0; \
             function inc() { $this->n = $this->n + 1; } \
             function get() { return $this->n; } } \
             $c = new Counter; $c->inc(); $c->inc(); echo $c->get();"
        ),
        "2"
    );
}

#[test]
fn oop_undefined_property_warns_and_yields_null() {
    let o = run_source(b"t.php", b"<?php class A {} $a = new A; echo $a->nope ?? 'x';").expect("lowers");
    assert!(o.fatal.is_none());
    assert_eq!(String::from_utf8(o.stdout).unwrap(), "x");
}

#[test]
fn oop_this_outside_method_is_fatal() {
    let o = run_source(b"t.php", b"<?php echo $this->x;").expect("lowers");
    match o.fatal {
        Some(PhpError::Error(m)) => assert_eq!(m, "Using $this when not in object context"),
        other => panic!("expected $this fatal, got {other:?}"),
    }
}

// --- Step 19-2: full property write-path (compound, inc/dec, ??=, array, nested) ---

#[test]
fn oop_prop_compound_add() {
    assert_eq!(
        out("<?php class C { public $n = 10; } $c = new C; $c->n += 5; echo $c->n;"),
        "15"
    );
}

#[test]
fn oop_prop_inc_dec() {
    assert_eq!(
        out("<?php class C { public $n = 0; } $c = new C; $c->n++; ++$c->n; echo $c->n;"),
        "2"
    );
}

#[test]
fn oop_prop_concat_assign() {
    assert_eq!(
        out("<?php class C { public $s = 'a'; } $c = new C; $c->s .= 'bc'; echo $c->s;"),
        "abc"
    );
}

#[test]
fn oop_prop_coalesce_assign() {
    assert_eq!(
        out("<?php class C { public $x = null; } $c = new C; $c->x ??= 7; echo $c->x;"),
        "7"
    );
}

#[test]
fn oop_prop_array_push_and_read() {
    assert_eq!(
        out(
            "<?php class C { public $a = []; } $c = new C; \
             $c->a[] = 1; $c->a[] = 2; $c->a[5] = 9; echo $c->a[0], $c->a[1], $c->a[5];"
        ),
        "129"
    );
}

#[test]
fn oop_nested_object_property_write() {
    assert_eq!(
        out(
            "<?php class B { public $v = 0; } class A { public $b; } \
             $a = new A; $a->b = new B; $a->b->v = 42; echo $a->b->v;"
        ),
        "42"
    );
}

#[test]
fn oop_isset_empty_unset_property() {
    assert_eq!(
        out(
            "<?php class C { public $x = 5; public $y = null; } $c = new C; \
             echo isset($c->x) ? 'A' : 'a'; \
             echo isset($c->y) ? 'B' : 'b'; \
             echo isset($c->z) ? 'C' : 'c'; \
             echo empty($c->x) ? 'D' : 'd'; \
             unset($c->x); \
             echo isset($c->x) ? 'E' : 'e';"
        ),
        "Abcde"
    );
}

// --- Step 19-3: inheritance, parent::/self::, visibility ---

#[test]
fn oop_inherited_method() {
    assert_eq!(
        out("<?php class A { function hi() { return 'A'; } } class B extends A {} echo (new B)->hi();"),
        "A"
    );
}

#[test]
fn oop_method_override() {
    assert_eq!(
        out(
            "<?php class A { function hi() { return 'A'; } } \
             class B extends A { function hi() { return 'B'; } } echo (new B)->hi();"
        ),
        "B"
    );
}

#[test]
fn oop_parent_call() {
    assert_eq!(
        out(
            "<?php class A { function hi() { return 'A'; } } \
             class B extends A { function hi() { return parent::hi() . 'B'; } } echo (new B)->hi();"
        ),
        "AB"
    );
}

#[test]
fn oop_self_resolves_to_defining_class() {
    // `self::who()` binds to the class that *defines* the method (A), not the
    // runtime class (B) — that distinction is late static binding (`static::`).
    assert_eq!(
        out(
            "<?php class A { function who() { return 'A'; } function call() { return self::who(); } } \
             class B extends A { function who() { return 'B'; } } echo (new B)->call();"
        ),
        "A"
    );
}

#[test]
fn oop_parent_constructor_chain() {
    assert_eq!(
        out(
            "<?php class A { public $x; function __construct($x) { $this->x = $x; } } \
             class B extends A { public $y; \
             function __construct($x, $y) { parent::__construct($x); $this->y = $y; } } \
             $b = new B(1, 2); echo $b->x, $b->y;"
        ),
        "12"
    );
}

#[test]
fn oop_inherited_constructor() {
    // A subclass with no constructor inherits the parent's.
    assert_eq!(
        out(
            "<?php class A { public $x; function __construct($x) { $this->x = $x; } } \
             class B extends A {} $b = new B(7); echo $b->x;"
        ),
        "7"
    );
}

#[test]
fn oop_private_property_from_outside_is_fatal() {
    let o = run_source(b"t.php", b"<?php class A { private $s = 1; } $a = new A; echo $a->s;")
        .expect("lowers");
    match o.fatal {
        Some(PhpError::Error(m)) => assert_eq!(m, "Cannot access private property A::$s"),
        other => panic!("expected private-access fatal, got {other:?}"),
    }
}

#[test]
fn oop_protected_property_accessible_from_subclass() {
    assert_eq!(
        out(
            "<?php class A { protected $p = 9; } \
             class B extends A { function get() { return $this->p; } } echo (new B)->get();"
        ),
        "9"
    );
}

#[test]
fn oop_private_method_from_outside_is_fatal() {
    let o = run_source(
        b"t.php",
        b"<?php class A { private function s() { return 1; } } $a = new A; echo $a->s();",
    )
    .expect("lowers");
    match o.fatal {
        Some(PhpError::Error(m)) => {
            assert_eq!(m, "Call to private method A::s() from global scope")
        }
        other => panic!("expected private-method fatal, got {other:?}"),
    }
}

// --- Step 19-4: class constants, static properties, static calls, LSB ---

#[test]
fn oop_class_constant_and_self_const() {
    assert_eq!(
        out("<?php class C { const FOO = 42; const BAR = self::FOO + 1; } echo C::FOO, '-', C::BAR;"),
        "42-43"
    );
}

#[test]
fn oop_constant_inheritance() {
    assert_eq!(
        out("<?php class A { const X = 7; } class B extends A {} echo B::X;"),
        "7"
    );
}

#[test]
fn oop_self_const_in_method() {
    assert_eq!(
        out("<?php class C { const N = 5; function f() { return self::N; } } echo (new C)->f();"),
        "5"
    );
}

#[test]
fn oop_static_property_via_static_method() {
    assert_eq!(
        out(
            "<?php class C { public static $n = 0; static function inc() { self::$n++; } } \
             C::inc(); C::inc(); echo C::$n;"
        ),
        "2"
    );
}

#[test]
fn oop_static_property_direct_compound() {
    assert_eq!(
        out("<?php class C { public static $n = 10; } C::$n += 5; echo C::$n;"),
        "15"
    );
}

#[test]
fn oop_static_method_call() {
    assert_eq!(
        out("<?php class C { static function hi() { return 'hi'; } } echo C::hi();"),
        "hi"
    );
}

#[test]
fn oop_shared_static_property_across_instances() {
    assert_eq!(
        out(
            "<?php class C { public static $c = 0; function __construct() { self::$c++; } } \
             new C; new C; echo C::$c;"
        ),
        "2"
    );
}

#[test]
fn oop_new_static_late_static_binding() {
    // `new static()` in a parent method creates the *called* class's instance.
    assert_eq!(
        out(
            "<?php class A { static function make() { return new static(); } function tag() { return 'A'; } } \
             class B extends A { function tag() { return 'B'; } } echo B::make()->tag();"
        ),
        "B"
    );
}

#[test]
fn oop_static_call_late_static_binding() {
    // `static::who()` dispatches on the called class (B), unlike `self::`.
    assert_eq!(
        out(
            "<?php class A { static function who() { return 'A'; } static function call() { return static::who(); } } \
             class B extends A { static function who() { return 'B'; } } echo B::call();"
        ),
        "B"
    );
}

#[test]
fn oop_class_constant_name() {
    assert_eq!(
        out("<?php class Foo {} echo Foo::class;"),
        "Foo"
    );
}

// --- Step 19-5: instanceof, interfaces, abstract ---

#[test]
fn oop_instanceof_class_and_parent() {
    assert_eq!(
        out(
            "<?php class A {} class B extends A {} $b = new B; \
             echo ($b instanceof B) ? '1' : '0'; \
             echo ($b instanceof A) ? '1' : '0'; \
             echo ($b instanceof C) ? '1' : '0';"
        ),
        "110"
    );
}

#[test]
fn oop_instanceof_interface() {
    assert_eq!(
        out(
            "<?php interface I {} class C implements I {} $c = new C; \
             echo ($c instanceof I) ? 'y' : 'n';"
        ),
        "y"
    );
}

#[test]
fn oop_instanceof_transitive_interface() {
    assert_eq!(
        out(
            "<?php interface A {} interface B extends A {} class C implements B {} \
             echo ((new C) instanceof A) ? 'y' : 'n';"
        ),
        "y"
    );
}

#[test]
fn oop_interface_const_and_implemented_method() {
    assert_eq!(
        out(
            "<?php interface Shape { const SIDES = 4; function area(); } \
             class Sq implements Shape { public $s = 2; function area() { return $this->s * $this->s; } } \
             $q = new Sq; echo $q->area(), '-', Shape::SIDES, '-', ($q instanceof Shape ? 'y' : 'n');"
        ),
        "4-4-y"
    );
}

#[test]
fn oop_abstract_method_with_concrete_subclass() {
    assert_eq!(
        out(
            "<?php abstract class A { abstract function f(); function g() { return $this->f() + 1; } } \
             class B extends A { function f() { return 10; } } echo (new B)->g();"
        ),
        "11"
    );
}

#[test]
fn oop_cannot_instantiate_abstract_class() {
    let o = run_source(b"t.php", b"<?php abstract class A {} $a = new A;").expect("lowers");
    match o.fatal {
        Some(PhpError::Error(m)) => assert_eq!(m, "Cannot instantiate abstract class A"),
        other => panic!("expected abstract-instantiate fatal, got {other:?}"),
    }
}

#[test]
fn oop_cannot_instantiate_interface() {
    let o = run_source(b"t.php", b"<?php interface I {} $i = new I;").expect("lowers");
    match o.fatal {
        Some(PhpError::Error(m)) => assert_eq!(m, "Cannot instantiate interface I"),
        other => panic!("expected interface-instantiate fatal, got {other:?}"),
    }
}
