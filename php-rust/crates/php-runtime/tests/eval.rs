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
    // Inc/dec now raises its diagnostic synchronously through the error-handler
    // chokepoint (so a `set_error_handler` runs before the write-back), which
    // renders it into `rendered` rather than buffering it in `diags`.
    let o = run_source(b"t.php", b"<?php $x = 'a'; $x++; echo $x;").expect("lowers");
    assert_eq!(o.stdout, b"b");
    let rendered = String::from_utf8(o.rendered).expect("utf8");
    assert!(
        rendered.contains("Increment on non-numeric string is deprecated"),
        "expected a Deprecated render, got {rendered:?}",
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
        // The VM raises `ArgumentCountError` (PHP's actual type, a TypeError
        // subclass); the tree-walker raised a plain `Error` — the VM is correct.
        Some(PhpError::ArgumentCountError(m)) => assert!(
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
#[ignore = "VM gap (deferred): emit the null-array-offset deprecation on the write \
            path (coerce_key_silent → a warning variant threaded with &mut diags) and \
            stamp it with the write line, not the next flush point's line (per-diag \
            line tracking). High blast radius for one deprecation message."]
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
        // VM raises the correct `ArgumentCountError` (eval raised a plain `Error`).
        Some(PhpError::ArgumentCountError(m)) => {
            assert!(m.contains("Too few arguments"), "{m}");
            assert!(m.contains("{closure"), "{m}");
        }
        other => panic!("expected ArgumentCountError, got {other:?}"),
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
fn unknown_constant_is_runtime_error() {
    // A bare name that is neither an engine constant nor `define()`d now lowers
    // to a runtime `Const` read and fatals at eval like PHP 8 (step 49c), rather
    // than being an unsupported-lowering SKIP.
    let o = run_source(b"t.php", b"<?php echo NOPE_UNDEFINED_CONST;").expect("lowers");
    assert!(o.fatal.is_some(), "expected a runtime fatal");
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

// --- Step 19-6: __toString, closure $this binding, static closures ---

#[test]
fn oop_to_string_in_echo_concat_cast() {
    assert_eq!(
        out(
            "<?php class M { public $v; function __construct($v) { $this->v = $v; } \
             function __toString() { return 'M(' . $this->v . ')'; } } \
             $m = new M(5); echo $m, ' ', $m . '!', ' ', (string)$m;"
        ),
        "M(5) M(5)! M(5)"
    );
}

#[test]
fn oop_to_string_missing_is_fatal() {
    let o = run_source(b"t.php", b"<?php class A {} echo new A;").expect("lowers");
    match o.fatal {
        Some(PhpError::Error(m)) => {
            assert_eq!(m, "Object of class A could not be converted to string")
        }
        other => panic!("expected to-string fatal, got {other:?}"),
    }
}

#[test]
fn oop_closure_in_method_binds_this() {
    assert_eq!(
        out(
            "<?php class C { public $v = 7; function make() { return function() { return $this->v; }; } } \
             $f = (new C)->make(); echo $f();"
        ),
        "7"
    );
}

#[test]
fn oop_static_closure_has_no_this() {
    assert_eq!(out("<?php $f = static function() { return 42; }; echo $f();"), "42");
}

#[test]
fn oop_closure_bindto() {
    assert_eq!(
        out(
            "<?php class C { public $v = 9; } \
             $f = function() { return $this->v; }; $g = $f->bindTo(new C); echo $g();"
        ),
        "9"
    );
}

#[test]
fn oop_closure_bind_static() {
    assert_eq!(
        out(
            "<?php class C { public $v = 3; } \
             $f = function() { return $this->v; }; $g = Closure::bind($f, new C); echo $g();"
        ),
        "3"
    );
}

#[test]
fn oop_closure_from_callable_string() {
    assert_eq!(
        out("<?php function dbl($x) { return $x * 2; } $f = Closure::fromCallable('dbl'); echo $f(21);"),
        "42"
    );
}

// --- step 20-1: exceptions — throw, try/catch, hierarchy, accessors ---

#[test]
fn exc_basic_try_catch() {
    assert_eq!(
        out("<?php try { throw new Exception('boom'); } catch (Exception $e) { echo $e->getMessage(); }"),
        "boom"
    );
}

#[test]
fn exc_get_code_default_and_set() {
    assert_eq!(
        out("<?php try { throw new Exception('m', 42); } catch (Exception $e) { echo $e->getMessage().'|'.$e->getCode(); }"),
        "m|42"
    );
    assert_eq!(
        out("<?php try { throw new Exception('x'); } catch (Exception $e) { echo $e->getCode(); }"),
        "0"
    );
}

#[test]
fn exc_default_message_empty() {
    assert_eq!(
        out("<?php try { throw new Exception(); } catch (Exception $e) { echo '['.$e->getMessage().']'; }"),
        "[]"
    );
}

#[test]
fn exc_error_not_caught_by_exception() {
    // TypeError is-a Error, not Exception: the first catch is skipped.
    assert_eq!(
        out("<?php try { throw new TypeError('t'); } catch (Exception $e) { echo 'E'; } catch (Error $e) { echo 'Err:'.$e->getMessage(); }"),
        "Err:t"
    );
}

#[test]
fn exc_throwable_catches_all() {
    assert_eq!(
        out("<?php try { throw new RuntimeException('r'); } catch (Throwable $e) { echo $e->getMessage(); }"),
        "r"
    );
}

#[test]
fn exc_hierarchy_instanceof() {
    assert_eq!(
        out("<?php $e = new RuntimeException('x'); echo ($e instanceof Exception)?'1':'0'; echo ($e instanceof Throwable)?'1':'0';"),
        "11"
    );
    assert_eq!(
        out("<?php echo (new InvalidArgumentException('x') instanceof LogicException)?'1':'0';"),
        "1"
    );
}

#[test]
fn exc_multi_catch() {
    assert_eq!(
        out("<?php try { throw new InvalidArgumentException('i'); } catch (LogicException | RuntimeException $e) { echo $e->getMessage(); }"),
        "i"
    );
}

#[test]
fn exc_catch_without_variable() {
    assert_eq!(
        out("<?php try { throw new Exception('x'); } catch (Exception) { echo 'caught'; }"),
        "caught"
    );
}

#[test]
fn exc_propagates_across_function_call() {
    assert_eq!(
        out("<?php function g(){ throw new RuntimeException('deep'); } try { g(); } catch (RuntimeException $e) { echo 'c:'.$e->getMessage(); }"),
        "c:deep"
    );
}

#[test]
fn exc_uncaught_renders_fatal() {
    let o = run_source(b"t.php", b"<?php throw new Exception('nope');").expect("lowers");
    assert!(matches!(o.fatal, Some(PhpError::Thrown(_))), "fatal: {:?}", o.fatal);
    assert_eq!(
        String::from_utf8(o.rendered).unwrap(),
        "\nFatal error: Uncaught Exception: nope in t.php:1\nStack trace:\n#0 {main}\n  thrown in t.php on line 1\n"
    );
}

#[test]
fn exc_get_line_is_creation_line() {
    let src = "<?php\ntry {\n  throw new Exception('x');\n} catch (Exception $e) { echo $e->getLine(); }";
    assert_eq!(out(src), "3");
}

// --- step 20-2: finally ---

#[test]
fn exc_finally_order_with_catch() {
    assert_eq!(
        out("<?php try { echo 't'; throw new Exception('e'); } catch (Exception $e) { echo 'c'; } finally { echo 'f'; } echo 'after';"),
        "tcfafter"
    );
}

#[test]
fn exc_finally_without_exception() {
    assert_eq!(out("<?php try { echo 't'; } finally { echo 'f'; } echo 'a';"), "tfa");
}

#[test]
fn exc_finally_runs_on_return() {
    assert_eq!(
        out("<?php function f(){ try { return 't'; } finally { echo 'f'; } } echo f();"),
        "ft"
    );
}

#[test]
fn exc_finally_overrides_return() {
    assert_eq!(
        out("<?php function f(){ try { return 'try'; } finally { return 'fin'; } } echo f();"),
        "fin"
    );
}

#[test]
fn exc_finally_runs_then_rethrows() {
    assert_eq!(
        out("<?php try { try { throw new Exception('inner'); } finally { echo 'F'; } } catch (Exception $e) { echo 'C:'.$e->getMessage(); }"),
        "FC:inner"
    );
}

#[test]
fn exc_finally_with_break_in_loop() {
    assert_eq!(
        out("<?php for($i=0;$i<3;$i++){ try { if($i==1) break; echo $i; } finally { echo 'f'; } }"),
        "0ff"
    );
}

#[test]
fn exc_finally_with_continue_in_loop() {
    assert_eq!(
        out("<?php for($i=0;$i<3;$i++){ try { if($i==1) continue; echo $i; } finally { echo 'f'; } }"),
        "0ff2f"
    );
}

#[test]
fn exc_finally_overrides_thrown_with_return() {
    // A `return` in finally swallows an in-flight exception (PHP semantics).
    assert_eq!(
        out("<?php function f(){ try { throw new Exception('x'); } finally { return 'ok'; } } echo f();"),
        "ok"
    );
}

// --- step 20-3: engine errors catchable, user subclasses, previous, throw-expr ---

#[test]
fn exc_engine_division_by_zero_catchable() {
    assert_eq!(
        out("<?php try { $x = 1 % 0; } catch (DivisionByZeroError $e) { echo $e->getMessage(); }"),
        "Modulo by zero"
    );
    assert_eq!(
        out("<?php try { $x = 5 / 0; } catch (DivisionByZeroError $e) { echo $e->getMessage(); }"),
        "Division by zero"
    );
}

#[test]
fn exc_engine_error_caught_as_throwable() {
    assert_eq!(
        out("<?php try { $x = 5 / 0; } catch (Throwable $e) { echo 'caught'; }"),
        "caught"
    );
}

#[test]
fn exc_engine_type_error_catchable() {
    assert_eq!(
        out("<?php function f(int $x){ return $x; } try { f([]); } catch (TypeError $e) { echo 'T'; }"),
        "T"
    );
}

#[test]
fn exc_user_subclass_caught_by_parent() {
    assert_eq!(
        out("<?php class MyExc extends Exception {} try { throw new MyExc('c'); } catch (Exception $e) { echo $e->getMessage(); }"),
        "c"
    );
}

#[test]
fn exc_user_subclass_parent_construct() {
    assert_eq!(
        out("<?php class MyExc extends Exception { public function __construct() { parent::__construct('fixed'); } } try { throw new MyExc(); } catch (MyExc $e) { echo $e->getMessage(); }"),
        "fixed"
    );
}

#[test]
fn exc_get_previous_chain() {
    assert_eq!(
        out("<?php try { try { throw new Exception('inner'); } catch (Exception $e) { throw new RuntimeException('outer', 0, $e); } } catch (Exception $e) { echo $e->getMessage().'<-'.$e->getPrevious()->getMessage(); }"),
        "outer<-inner"
    );
}

#[test]
fn exc_throw_expression() {
    assert_eq!(
        out("<?php function f($x){ return $x ?: throw new RuntimeException('empty'); } try { echo f(0); } catch (RuntimeException $e) { echo $e->getMessage(); }"),
        "empty"
    );
}

// --- step 20 coda: stdClass, get_class, get_parent_class ---

#[test]
fn exc_get_class_object_and_this() {
    assert_eq!(out("<?php class C{} echo get_class(new C);"), "C");
    assert_eq!(
        out("<?php class C{ function n(){ return get_class($this); } } echo (new C)->n();"),
        "C"
    );
}

#[test]
fn exc_get_class_on_exception() {
    assert_eq!(
        out("<?php try { throw new RuntimeException('x'); } catch (Exception $e) { echo get_class($e); }"),
        "RuntimeException"
    );
}

#[test]
fn exc_get_parent_class_string_and_object() {
    assert_eq!(out("<?php class A{} class B extends A{} echo get_parent_class('B');"), "A");
    assert_eq!(out("<?php echo get_parent_class(new RuntimeException('x'));"), "Exception");
}

#[test]
fn exc_get_parent_class_none_is_false() {
    assert_eq!(
        out("<?php class A{} echo get_parent_class(new A) === false ? 'f' : 'x';"),
        "f"
    );
}

#[test]
fn stdclass_dynamic_property() {
    assert_eq!(out("<?php $o = new stdClass; $o->x = 5; echo $o->x;"), "5");
}

// --- step 21-1: traits, core flatten ---

#[test]
fn trait_basic_flatten_method_and_prop() {
    assert_eq!(
        out("<?php trait T { public function hello(){ return 'hi'; } public $x = 5; } \
             class C { use T; } $c = new C(); echo $c->hello(), $c->x;"),
        "hi5"
    );
}

#[test]
fn trait_method_reads_this_property() {
    // A trait method operates on the consuming object's properties.
    assert_eq!(
        out("<?php trait T { public function dbl(){ return $this->n * 2; } } \
             class C { use T; public $n = 21; } $c = new C(); echo $c->dbl();"),
        "42"
    );
}

#[test]
fn trait_get_class_in_method_is_consumer() {
    // `$this` in a flattened method is the consuming class, so get_class returns it.
    assert_eq!(
        out("<?php trait T { public function who(){ return get_class($this); } } \
             class C { use T; } echo (new C())->who();"),
        "C"
    );
}

#[test]
fn trait_class_method_overrides_trait() {
    // A method declared on the class takes precedence over the trait's.
    assert_eq!(
        out("<?php trait T { public function f(){ return 'trait'; } } \
             class C { use T; public function f(){ return 'class'; } } echo (new C())->f();"),
        "class"
    );
}

#[test]
fn trait_method_overrides_inherited_parent_method() {
    // The flattened trait method is the class's own, so it wins over the parent's.
    assert_eq!(
        out("<?php class P { public function f(){ return 'parent'; } } \
             trait T { public function f(){ return 'trait'; } } \
             class C extends P { use T; } echo (new C())->f();"),
        "trait"
    );
}

#[test]
fn trait_multiple_disjoint_members_merge() {
    assert_eq!(
        out("<?php trait A { public function a(){ return 'a'; } } \
             trait B { public function b(){ return 'b'; } } \
             class C { use A, B; } $c = new C(); echo $c->a(), $c->b();"),
        "ab"
    );
}

// --- step 21-2: traits with static props, constants, self/static ---

#[test]
fn trait_static_prop_is_per_consuming_class() {
    // Each class using the trait gets its OWN static-property cell.
    assert_eq!(
        out("<?php trait Counter { public static int $n = 0; \
             public static function next(): int { return ++self::$n; } } \
             class A { use Counter; } class B { use Counter; } \
             echo A::next(), A::next(), B::next();"),
        "121"
    );
}

#[test]
fn trait_static_kw_in_trait_method() {
    // `static::` from a flattened method resolves to the consuming class.
    assert_eq!(
        out("<?php trait T { public static int $count = 0; \
             public static function inc(){ static::$count++; } } \
             class C { use T; } C::inc(); C::inc(); echo C::$count;"),
        "2"
    );
}

#[test]
fn trait_constant_is_flattened() {
    assert_eq!(
        out("<?php trait T { const FOO = 42; } class C { use T; } echo C::FOO;"),
        "42"
    );
}

#[test]
fn trait_new_static_in_trait_method() {
    // late static binding: `new static()` builds the consuming class.
    assert_eq!(
        out("<?php trait T { public static function create(){ return new static(); } } \
             class C { use T; } echo get_class(C::create());"),
        "C"
    );
}

#[test]
fn trait_abstract_method_satisfied_by_class() {
    // An abstract method in the trait is fulfilled by the consumer; the trait's
    // concrete method calls it via $this.
    assert_eq!(
        out("<?php trait T { abstract public function name(): string; \
             public function greet(){ return 'Hi ' . $this->name(); } } \
             class C { use T; public function name(): string { return 'C'; } } \
             echo (new C())->greet();"),
        "Hi C"
    );
}

#[test]
fn trait_mixed_instance_and_static_members() {
    assert_eq!(
        out("<?php trait T { public $tag = 't'; public static int $seen = 0; \
             public function mark(){ self::$seen++; return $this->tag; } } \
             class C { use T; } $c = new C(); \
             echo $c->mark(), $c->mark(), C::$seen;"),
        "tt2"
    );
}

// --- step 21-3: conflict resolution (insteadof / as / collision) ---

#[test]
fn trait_insteadof_and_as_alias() {
    assert_eq!(
        out("<?php trait A { public function say(){ return 'A'; } } \
             trait B { public function say(){ return 'B'; } } \
             class C { use A, B { A::say insteadof B; B::say as sayB; } } \
             $c = new C(); echo $c->say(), $c->sayB();"),
        "AB"
    );
}

#[test]
fn trait_unresolved_collision_is_compile_fatal() {
    let src = "<?php\ntrait A { public function say(){ return 'A'; } }\n\
               trait B { public function say(){ return 'B'; } }\n\
               class C { use A, B; }\necho (new C())->say();\n";
    assert_eq!(
        rendered(src),
        "\nFatal error: Trait method B::say has not been applied as C::say, \
         because of collision with A::say in t.php on line 4\n\
         Stack trace:\n#0 {main}\n"
    );
}

#[test]
fn trait_as_alias_with_rename_callable_internally() {
    // `f as protected g;` exposes `g` (protected) callable from inside the class.
    assert_eq!(
        out("<?php trait T { public function f(){ return 'f'; } } \
             class C { use T { f as protected g; } public function call(){ return $this->g(); } } \
             echo (new C())->call();"),
        "f"
    );
}

#[test]
fn trait_as_protected_rename_blocks_external_call() {
    let src = "<?php\ntrait T { public function f(){ return 'f'; } }\n\
               class C { use T { f as protected g; } }\n$c = new C();\necho $c->g();\n";
    assert_eq!(
        rendered(src),
        "\nFatal error: Uncaught Error: Call to protected method C::g() from global scope \
         in t.php:5\nStack trace:\n#0 {main}\n  thrown in t.php on line 5\n"
    );
}

#[test]
fn trait_as_visibility_only_no_rename() {
    // `f as protected;` keeps the name but changes visibility.
    let src = "<?php\ntrait T { public function f(){ return 'f'; } }\n\
               class C { use T { f as protected; } }\n$c = new C();\necho $c->f();\n";
    assert_eq!(
        rendered(src),
        "\nFatal error: Uncaught Error: Call to protected method C::f() from global scope \
         in t.php:5\nStack trace:\n#0 {main}\n  thrown in t.php on line 5\n"
    );
}

// --- step 21-4: nested traits, abstract requirement, instanceof ---

#[test]
fn trait_using_trait_is_flattened_transitively() {
    assert_eq!(
        out("<?php trait A { public function a(){ return 'a'; } } \
             trait B { use A; public function b(){ return 'b' . $this->a(); } } \
             class C { use B; } $c = new C(); echo $c->a(), $c->b();"),
        "aba"
    );
}

#[test]
fn trait_cross_trait_method_call() {
    // A method in one trait calls a method supplied by another trait on the
    // same consumer (both flattened into the class).
    assert_eq!(
        out("<?php trait A { public function a(){ return $this->b() . 'A'; } } \
             trait B { public function b(){ return 'B'; } } \
             class C { use A, B; } echo (new C())->a();"),
        "BA"
    );
}

#[test]
fn trait_abstract_unimplemented_is_compile_fatal() {
    let src = "<?php\ntrait T { abstract public function f(): string; }\n\
               class C { use T; }\nnew C();\n";
    assert_eq!(
        rendered(src),
        "\nFatal error: Class C contains 1 abstract method and must therefore be declared \
         abstract or implement the remaining method (C::f) in t.php on line 3\n\
         Stack trace:\n#0 {main}\n"
    );
}

#[test]
fn trait_two_abstract_unimplemented_plural_fatal() {
    let src = "<?php\ntrait T { abstract public function f(): string; abstract public function g(): int; }\n\
               class C { use T; }\nnew C();\n";
    assert_eq!(
        rendered(src),
        "\nFatal error: Class C contains 2 abstract methods and must therefore be declared \
         abstract or implement the remaining methods (C::f, C::g) in t.php on line 3\n\
         Stack trace:\n#0 {main}\n"
    );
}

#[test]
fn trait_is_not_a_type_for_instanceof() {
    assert_eq!(
        out("<?php trait T {} class C { use T; } \
             echo (new C()) instanceof T ? 'y' : 'n';"),
        "n"
    );
}

// --- step 22-1: __get / __set ---

#[test]
fn magic_set_then_get() {
    // __set doubles the stored value, so the output distinguishes the magic path
    // from a plain dynamic property (which would echo 5).
    assert_eq!(
        out("<?php class C { private $d = []; \
             function __get($n){ return $this->d[$n] ?? null; } \
             function __set($n,$v){ $this->d[$n] = $v * 2; } } \
             $c = new C(); $c->foo = 5; echo $c->foo;"),
        "10"
    );
}

#[test]
fn magic_get_not_called_for_accessible_existing_prop() {
    // A real, accessible property is read directly — no __get.
    assert_eq!(
        out("<?php class C { public $real = 'R'; \
             function __get($n){ return 'magic'; } } \
             $c = new C(); echo $c->real;"),
        "R"
    );
}

#[test]
fn magic_get_for_inaccessible_private_from_outside() {
    // A private property read from outside the class routes to __get.
    assert_eq!(
        out("<?php class C { private $secret = 'hidden'; \
             function __get($n){ return 'g:' . $n; } } \
             $c = new C(); echo $c->secret;"),
        "g:secret"
    );
}

#[test]
fn magic_set_for_inaccessible_private_from_outside() {
    assert_eq!(
        out("<?php class C { private $secret = 'hidden'; public $log = ''; \
             function __set($n,$v){ $this->log = $n . '=' . $v; } } \
             $c = new C(); $c->secret = 'x'; echo $c->log;"),
        "secret=x"
    );
}

#[test]
fn magic_compound_assign_uses_get_then_set() {
    assert_eq!(
        out("<?php class C { private $d = ['n' => 10]; \
             function __get($k){ return $this->d[$k]; } \
             function __set($k,$v){ $this->d[$k] = $v; } } \
             $c = new C(); $c->n += 5; echo $c->n;"),
        "15"
    );
}

#[test]
fn magic_get_recursion_guard_same_property() {
    // Inside __get('foo'), reading $this->foo (same name) bypasses the magic
    // method and hits the (missing) real property → null, no infinite loop.
    assert_eq!(
        out("<?php class C { \
             function __get($n){ return $this->foo === null ? 'guarded' : 'x'; } } \
             $c = new C(); echo $c->foo;"),
        "guarded"
    );
}

// --- step 22-2: __isset / __unset / coalesce ---

#[test]
fn magic_isset_basic() {
    assert_eq!(
        out("<?php class C { private $d = ['has' => 1]; \
             function __isset($n){ return isset($this->d[$n]); } } \
             $c = new C(); echo isset($c->has) ? '1' : '0'; echo isset($c->no) ? '1' : '0';"),
        "10"
    );
}

#[test]
fn magic_isset_does_not_call_get() {
    // Bare isset() only triggers __isset, never __get.
    assert_eq!(
        out("<?php class C { function __isset($n){ return true; } \
             function __get($n){ echo 'GET'; return 1; } } \
             $c = new C(); echo isset($c->x) ? '1' : '0';"),
        "1"
    );
}

#[test]
fn magic_empty_uses_isset_then_get() {
    assert_eq!(
        out("<?php class C { private $d = ['z' => 0, 'v' => 5]; \
             function __isset($n){ return isset($this->d[$n]); } \
             function __get($n){ return $this->d[$n]; } } \
             $c = new C(); echo empty($c->z) ? '1' : '0'; echo empty($c->v) ? '1' : '0'; \
             echo empty($c->missing) ? '1' : '0';"),
        "101"
    );
}

#[test]
fn magic_coalesce_read() {
    assert_eq!(
        out("<?php class C { private $d = ['a' => 'A']; \
             function __isset($n){ return isset($this->d[$n]); } \
             function __get($n){ return $this->d[$n]; } } \
             $c = new C(); echo ($c->a ?? 'D'); echo ($c->b ?? 'D');"),
        "AD"
    );
}

#[test]
fn magic_coalesce_does_not_get_when_unset() {
    assert_eq!(
        out("<?php class C { function __isset($n){ return false; } \
             function __get($n){ echo 'G'; return 1; } } \
             $c = new C(); echo ($c->x ?? 'D');"),
        "D"
    );
}

#[test]
fn magic_coalesce_assign_existing_does_not_set() {
    assert_eq!(
        out("<?php class C { public $log = ''; private $d = ['a' => 'A']; \
             function __isset($n){ return isset($this->d[$n]); } \
             function __get($n){ return $this->d[$n]; } \
             function __set($n,$v){ $this->log = 'SET'; } } \
             $c = new C(); $c->a ??= 'X'; \
             echo $c->log === '' ? 'noset' : 'set'; echo ':'; echo $c->a;"),
        "noset:A"
    );
}

#[test]
fn magic_coalesce_assign_missing_sets() {
    assert_eq!(
        out("<?php class C { private $d = []; \
             function __isset($n){ return isset($this->d[$n]); } \
             function __get($n){ return $this->d[$n] ?? null; } \
             function __set($n,$v){ $this->d[$n] = $v; } } \
             $c = new C(); $c->x ??= 'NEW'; echo $c->x;"),
        "NEW"
    );
}

#[test]
fn magic_unset_basic() {
    assert_eq!(
        out("<?php class C { public $log = ''; private $d = ['a' => 1]; \
             function __isset($n){ return isset($this->d[$n]); } \
             function __unset($n){ $this->log = 'U:' . $n; unset($this->d[$n]); } } \
             $c = new C(); unset($c->a); echo $c->log; echo isset($c->a) ? '1' : '0';"),
        "U:a0"
    );
}

#[test]
fn magic_unset_for_inaccessible_private() {
    assert_eq!(
        out("<?php class C { private $secret = 1; public $log = ''; \
             function __unset($n){ $this->log = 'U:' . $n; } } \
             $c = new C(); unset($c->secret); echo $c->log;"),
        "U:secret"
    );
}

#[test]
fn magic_isset_recursion_guard() {
    // isset($this->foo) inside __isset('foo') bypasses magic → direct (absent).
    assert_eq!(
        out("<?php class C { function __isset($n){ return isset($this->foo); } } \
             $c = new C(); echo isset($c->foo) ? '1' : '0';"),
        "0"
    );
}

#[test]
fn magic_set_writes_real_property_under_guard() {
    // __set writing the *same* property name creates the real property directly
    // (guard active), so a subsequent read sees it without re-entering __get.
    assert_eq!(
        out("<?php class C { public $hits = 0; \
             function __get($n){ $this->hits++; return 99; } \
             function __set($n,$v){ $this->foo = $v; } } \
             $c = new C(); $c->foo = 7; echo $c->foo, ':', $c->hits;"),
        "7:0"
    );
}

// --- step 22-3: __call / __callStatic ---

#[test]
fn magic_call_undefined_method() {
    assert_eq!(
        out("<?php class C { function __call($n,$a){ return $n . '/' . $a[0] . $a[1]; } } \
             $c = new C(); echo $c->foo('x','y');"),
        "foo/xy"
    );
}

#[test]
fn magic_call_for_inaccessible_private_method() {
    assert_eq!(
        out("<?php class C { private function sec(){ return 'D'; } \
             function __call($n,$a){ return 'C:' . $n; } } \
             $c = new C(); echo $c->sec();"),
        "C:sec"
    );
}

#[test]
fn magic_call_not_used_for_accessible_method() {
    assert_eq!(
        out("<?php class C { function pub(){ return 'P'; } \
             function __call($n,$a){ return 'C'; } } \
             $c = new C(); echo $c->pub();"),
        "P"
    );
}

#[test]
fn magic_call_not_used_for_private_from_inside() {
    assert_eq!(
        out("<?php class C { private function p(){ return 'P'; } \
             function __call($n,$a){ return 'C'; } \
             function go(){ return $this->p(); } } \
             $c = new C(); echo $c->go();"),
        "P"
    );
}

#[test]
fn magic_callstatic_undefined_method() {
    assert_eq!(
        out("<?php class C { static function __callStatic($n,$a){ return 'S:' . $n . ':' . $a[0]; } } \
             echo C::foo(9);"),
        "S:foo:9"
    );
}

#[test]
fn magic_callstatic_for_inaccessible_private_static() {
    assert_eq!(
        out("<?php class C { private static function s(){ return 'D'; } \
             static function __callStatic($n,$a){ return 'CS:' . $n; } } \
             echo C::s();"),
        "CS:s"
    );
}

// --- step 22-4: __invoke ---

#[test]
fn magic_invoke_object_as_function() {
    assert_eq!(
        out("<?php class C { function __invoke($x){ return $x * 2; } } \
             $c = new C(); echo $c(21);"),
        "42"
    );
}

#[test]
fn magic_invoke_via_call_user_func() {
    assert_eq!(
        out("<?php class C { function __invoke($x){ return 'i:' . $x; } } \
             echo call_user_func(new C(), 'z');"),
        "i:z"
    );
}

#[test]
fn magic_invoke_is_callable() {
    assert_eq!(
        out("<?php class C { function __invoke(){ return 1; } } class D {} \
             echo is_callable(new C()) ? '1' : '0'; \
             echo is_callable(new D()) ? '1' : '0';"),
        "10"
    );
}

#[test]
fn magic_empty_silent_when_isset_true_no_get() {
    // bug #44899: empty() with __isset true but no __get reads the value
    // silently — no "Undefined property" warning.
    assert_eq!(
        rendered(
            "<?php class C { private $d = ['foo' => '']; \
             function __isset($n){ return isset($this->d[$n]); } } \
             $c = new C(); echo empty($c->foo) ? 'E' : 'N';"
        ),
        "E"
    );
}

#[test]
fn magic_call_via_parent_in_object_context() {
    // bug #53826: an inaccessible/undefined parent:: call inside a method has
    // $this, so it routes to __call (instance), not __callStatic.
    assert_eq!(
        out("<?php class A { public function __call($m,$a){ return 'call:' . $m; } \
             public static function __callStatic($m,$a){ return 'static:' . $m; } } \
             class B extends A { public function go(){ return parent::missing(); } } \
             echo (new B())->go();"),
        "call:missing"
    );
}

#[test]
fn magic_invoke_object_without_invoke_not_callable() {
    let o = run_source(
        b"t.php",
        b"<?php class C {} $c = new C(); $c();",
    )
    .expect("lowers");
    match o.fatal {
        Some(PhpError::Error(msg)) => assert_eq!(msg, "Object of type C is not callable"),
        other => panic!("expected not-callable Error, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Step 23-1 — pure enum core
// ---------------------------------------------------------------------------

const SUIT: &str = "enum Suit { case Hearts; case Diamonds; case Clubs; case Spades; } ";

#[test]
fn enum_case_name() {
    assert_eq!(
        out(&format!("<?php {SUIT} echo Suit::Hearts->name;")),
        "Hearts"
    );
}

#[test]
fn enum_case_singleton_identity() {
    assert_eq!(
        out(&format!("<?php {SUIT} echo Suit::Hearts === Suit::Hearts ? 'y' : 'n';")),
        "y"
    );
}

#[test]
fn enum_distinct_cases_not_identical() {
    assert_eq!(
        out(&format!(
            "<?php {SUIT} echo Suit::Hearts === Suit::Spades ? 'y' : 'n'; echo Suit::Hearts !== Suit::Spades ? 'y' : 'n';"
        )),
        "ny"
    );
}

#[test]
fn enum_match_on_case() {
    assert_eq!(
        out(&format!(
            "<?php {SUIT} $x = Suit::Clubs; echo match($x) {{ Suit::Hearts, Suit::Diamonds => 'red', Suit::Clubs, Suit::Spades => 'black', }};"
        )),
        "black"
    );
}

#[test]
fn enum_instanceof_self_and_unitenum() {
    assert_eq!(
        out(&format!(
            "<?php {SUIT} $x = Suit::Hearts; echo $x instanceof Suit ? 'y' : 'n'; echo $x instanceof UnitEnum ? 'y' : 'n';"
        )),
        "yy"
    );
}

#[test]
fn enum_class_constant() {
    assert_eq!(out(&format!("<?php {SUIT} echo Suit::class;")), "Suit");
}

#[test]
fn enum_cannot_instantiate() {
    let o = run_source(
        b"t.php",
        b"<?php enum Suit { case Hearts; } $x = new Suit();",
    )
    .expect("lowers");
    match o.fatal {
        Some(PhpError::Error(msg)) => assert_eq!(msg, "Cannot instantiate enum Suit"),
        other => panic!("expected cannot-instantiate Error, got {other:?}"),
    }
}

#[test]
fn object_identity_handle_semantics() {
    // D-23.3: object === follows handle identity (shared Rc), distinct instances differ.
    assert_eq!(
        out("<?php class C {} $a = new C(); $b = $a; $c = new C(); \
             echo $a === $b ? 'y' : 'n'; echo $a === $c ? 'y' : 'n';"),
        "yn"
    );
}

// ---------------------------------------------------------------------------
// Step 23-2 — backed enum
// ---------------------------------------------------------------------------

const STATUS: &str = "enum Status: string { case Active = 'A'; case Inactive = 'I'; } ";
const SIZE: &str = "enum Size: int { case Small = 1; case Large = 3; } ";

#[test]
fn enum_backed_value_string() {
    assert_eq!(out(&format!("<?php {STATUS} echo Status::Active->value;")), "A");
}

#[test]
fn enum_backed_value_int() {
    assert_eq!(out(&format!("<?php {SIZE} echo Size::Large->value;")), "3");
}

#[test]
fn enum_instanceof_backedenum() {
    // A backed case is both UnitEnum and BackedEnum; a pure case is not BackedEnum.
    assert_eq!(
        out(&format!(
            "<?php {STATUS}{SUIT} echo Status::Active instanceof BackedEnum ? 'y' : 'n'; \
             echo Status::Active instanceof UnitEnum ? 'y' : 'n'; \
             echo Suit::Hearts instanceof BackedEnum ? 'y' : 'n';"
        )),
        "yyn"
    );
}

#[test]
fn enum_from_hit_string_and_int() {
    assert_eq!(
        out(&format!(
            "<?php {STATUS}{SIZE} echo Status::from('I') === Status::Inactive ? 'y' : 'n'; \
             echo Size::from(3) === Size::Large ? 'y' : 'n';"
        )),
        "yy"
    );
}

#[test]
fn enum_tryfrom_hit_and_miss() {
    assert_eq!(
        out(&format!(
            "<?php {STATUS} echo Status::tryFrom('A') === Status::Active ? 'y' : 'n'; \
             echo Status::tryFrom('X') === null ? 'y' : 'n';"
        )),
        "yy"
    );
}

#[test]
fn enum_from_miss_throws_valueerror_string() {
    assert_eq!(
        out(&format!(
            "<?php {STATUS} try {{ Status::from('X'); }} catch (\\ValueError $e) {{ echo $e->getMessage(); }}"
        )),
        "\"X\" is not a valid backing value for enum Status"
    );
}

#[test]
fn enum_from_miss_throws_valueerror_int() {
    assert_eq!(
        out(&format!(
            "<?php {SIZE} try {{ Size::from(9); }} catch (\\ValueError $e) {{ echo $e->getMessage(); }}"
        )),
        "9 is not a valid backing value for enum Size"
    );
}

#[test]
fn enum_from_undefined_on_pure_enum() {
    // Pure enums have no from()/tryFrom(); calling one is an undefined-method error.
    let o = run_source(
        b"t.php",
        b"<?php enum Suit { case Hearts; } Suit::from('x');",
    )
    .expect("lowers");
    assert!(matches!(o.fatal, Some(PhpError::Error(_))), "expected error, got {:?}", o.fatal);
}

// ---------------------------------------------------------------------------
// Step 23-3 — cases() + user methods / constants
// ---------------------------------------------------------------------------

#[test]
fn enum_cases_returns_all_in_order() {
    assert_eq!(
        out(&format!(
            "<?php {SUIT} $cs = Suit::cases(); $n = 0; foreach ($cs as $c) {{ echo $c->name; $n++; }} echo $n;"
        )),
        "HeartsDiamondsClubsSpades4"
    );
}

#[test]
fn enum_cases_yields_singletons() {
    assert_eq!(
        out(&format!("<?php {SUIT} echo Suit::cases()[0] === Suit::Hearts ? 'y' : 'n';")),
        "y"
    );
}

#[test]
fn enum_instance_method_with_this_match() {
    assert_eq!(
        out(
            "<?php enum Suit { case Hearts; case Spades; \
             public function color(): string { return match($this) { \
               Suit::Hearts => 'Red', Suit::Spades => 'Black', }; } } \
             echo Suit::Hearts->color(), Suit::Spades->color();"
        ),
        "RedBlack"
    );
}

#[test]
fn enum_static_method() {
    assert_eq!(
        out(
            "<?php enum Suit { case Hearts; \
             public static function default(): Suit { return Suit::Hearts; } } \
             echo Suit::default()->name;"
        ),
        "Hearts"
    );
}

#[test]
fn enum_constant_referencing_self_case() {
    assert_eq!(
        out(
            "<?php enum Status: string { case Active = 'A'; case Inactive = 'I'; \
             const DEFAULT = self::Active; } \
             echo Status::DEFAULT->value; echo Status::DEFAULT === Status::Active ? 'y' : 'n';"
        ),
        "Ay"
    );
}

#[test]
fn enum_method_calls_self_const_and_case() {
    assert_eq!(
        out(
            "<?php enum Suit { case Hearts; case Spades; \
             public function label(): string { return $this->name . '!'; } } \
             echo Suit::Spades->label();"
        ),
        "Spades!"
    );
}

// ---------------------------------------------------------------------------
// Step 23-4 — corpus fixes: loose == on objects/enums, interface constants
// ---------------------------------------------------------------------------

#[test]
fn enum_loose_equals_is_identity() {
    assert_eq!(
        out(&format!(
            "<?php {SUIT} echo Suit::Hearts == Suit::Hearts ? 'y' : 'n'; \
             echo Suit::Hearts == Suit::Spades ? 'y' : 'n';"
        )),
        "yn"
    );
}

#[test]
fn object_loose_equals_same_class_and_props() {
    assert_eq!(
        out("<?php class P { public $x; function __construct($x){ $this->x = $x; } } \
             echo (new P(1)) == (new P(1)) ? 'y' : 'n'; \
             echo (new P(1)) == (new P(2)) ? 'y' : 'n';"),
        "yn"
    );
}

#[test]
fn enum_inherits_interface_constants() {
    // gh7821: interface constants are inherited; case values may reference them.
    assert_eq!(
        out("<?php interface I { const A = 'A'; const B = 'B'; } \
             enum E: string implements I { case C = I::A; case D = self::B; } \
             echo E::A, E::B, E::C->value, E::D->value;"),
        "ABAB"
    );
}

#[test]
fn class_inherits_interface_constants() {
    assert_eq!(
        out("<?php interface HasMax { const MAX = 100; } class C implements HasMax {} echo C::MAX;"),
        "100"
    );
}

// ---------------------------------------------------------------------------
// Step 23-5 — enum case immutability (readonly props, no dynamic, no unset)
// ---------------------------------------------------------------------------

#[test]
fn enum_modify_existing_prop_is_readonly_error() {
    assert_eq!(
        out(&format!(
            "<?php {SUIT} $h = Suit::Hearts; try {{ $h->name = 'x'; }} catch (\\Error $e) {{ echo $e->getMessage(); }}"
        )),
        "Cannot modify readonly property Suit::$name"
    );
}

#[test]
fn enum_create_dynamic_prop_error() {
    assert_eq!(
        out(&format!(
            "<?php {SUIT} $h = Suit::Hearts; try {{ $h->value = 1; }} catch (\\Error $e) {{ echo $e->getMessage(); }}"
        )),
        "Cannot create dynamic property Suit::$value"
    );
}

#[test]
fn enum_backed_modify_value_is_readonly_error() {
    assert_eq!(
        out(&format!(
            "<?php {STATUS} $a = Status::Active; \
             try {{ $a->value = 'Z'; }} catch (\\Error $e) {{ echo $e->getMessage(); }} \
             try {{ $a->other = 1; }} catch (\\Error $e) {{ echo '|', $e->getMessage(); }}"
        )),
        "Cannot modify readonly property Status::$value|Cannot create dynamic property Status::$other"
    );
}

#[test]
fn enum_unset_prop_is_readonly_error() {
    assert_eq!(
        out(&format!(
            "<?php {SUIT} $h = Suit::Hearts; try {{ unset($h->name); }} catch (\\Error $e) {{ echo $e->getMessage(); }}"
        )),
        "Cannot unset readonly property Suit::$name"
    );
}

// --- Step 24-1: Stringable auto-interface ---

#[test]
fn stringable_auto_implementation() {
    // A class with __toString is automatically Stringable, without an explicit
    // `implements` clause; a class without __toString is not.
    assert_eq!(
        out(
            "<?php class A { public function __toString(): string { return 'x'; } } \
             class B {} \
             echo ((new A) instanceof Stringable) ? '1' : '0'; \
             echo ((new B) instanceof Stringable) ? '1' : '0';"
        ),
        "10"
    );
}

#[test]
fn stringable_explicit_implements() {
    assert_eq!(
        out(
            "<?php class C implements Stringable { public function __toString(): string { return 'c'; } } \
             echo ((new C) instanceof Stringable) ? 'y' : 'n';"
        ),
        "y"
    );
}

#[test]
fn stringable_inherited_tostring() {
    // __toString inherited from a parent still satisfies instanceof Stringable.
    assert_eq!(
        out(
            "<?php class P { public function __toString(): string { return 'p'; } } \
             class Q extends P {} \
             echo ((new Q) instanceof Stringable) ? '1' : '0';"
        ),
        "1"
    );
}

// --- Step 24-2: __destruct end-of-script shutdown ---

#[test]
fn destruct_runs_at_end_of_script() {
    // Destructors of objects alive at the end run after the script body, in
    // reverse creation order (PHP shutdown is LIFO).
    assert_eq!(
        out(
            "<?php class A { public $n; function __construct($n){$this->n=$n;} \
             function __destruct(){ echo 'd' . $this->n; } } \
             $a=new A(1); $b=new A(2); $c=new A(3); echo 'end';"
        ),
        "endd3d2d1"
    );
}

#[test]
fn destruct_object_in_array_survives() {
    assert_eq!(
        out(
            "<?php class A { function __destruct(){ echo 'dtor'; } } \
             $arr=[new A()]; echo 'end';"
        ),
        "enddtor"
    );
}

#[test]
fn destruct_object_as_property_survives() {
    assert_eq!(
        out(
            "<?php class H { public $x; } class A { function __destruct(){ echo 'dtor'; } } \
             $h=new H(); $h->x=new A(); echo 'end';"
        ),
        "enddtor"
    );
}

#[test]
fn destruct_inherited_from_parent() {
    assert_eq!(
        out(
            "<?php class P { function __destruct(){ echo 'd'; } } class Q extends P {} \
             $q=new Q(); echo 'end';"
        ),
        "endd"
    );
}

// --- Step 24-3: immediate __destruct on refcount-zero ---

#[test]
fn destruct_on_unset() {
    assert_eq!(
        out(
            "<?php class A { function __destruct(){ echo 'dtor'; } } \
             $a=new A(); echo 'before'; unset($a); echo 'after';"
        ),
        "beforedtorafter"
    );
}

#[test]
fn destruct_on_reassign() {
    // The old object is released when the variable is reassigned: its destructor
    // runs at that statement, the new object's at shutdown.
    assert_eq!(
        out(
            "<?php class A { public $n; function __construct($n){$this->n=$n;} \
             function __destruct(){ echo 'd' . $this->n; } } \
             $a=new A(1); $a=new A(2); echo 'end';"
        ),
        "d1endd2"
    );
}

#[test]
fn destruct_shared_ref_waits_for_last() {
    // Two handles to one object: the destructor runs only when the *last* one is
    // released.
    assert_eq!(
        out(
            "<?php class A { function __destruct(){ echo 'dtor'; } } \
             $a=new A(); $b=$a; unset($a); echo 'mid'; unset($b); echo 'end';"
        ),
        "middtorend"
    );
}

#[test]
fn destruct_temp_expression() {
    // An object with no binding is released at the end of its statement.
    assert_eq!(
        out(
            "<?php class A { public $n; function __construct($n){$this->n=$n;} \
             function __destruct(){ echo 'd' . $this->n; } } \
             new A(1); echo 'end';"
        ),
        "d1end"
    );
}

#[test]
fn destruct_function_local_scope_exit() {
    // A local that goes out of scope when the function returns is destructed then.
    assert_eq!(
        out(
            "<?php class A { function __destruct(){ echo 'dtor'; } } \
             function f(){ $x=new A(); echo 'infn'; } f(); echo 'end';"
        ),
        "infndtorend"
    );
}

#[test]
fn destruct_transitive_array_release() {
    // Unsetting the array that holds the only reference frees the object too.
    assert_eq!(
        out(
            "<?php class A { function __destruct(){ echo 'dtor'; } } \
             $arr=[new A()]; echo 'before'; unset($arr); echo 'after';"
        ),
        "beforedtorafter"
    );
}

#[test]
fn destruct_cascade_through_property() {
    // Releasing a container runs its destructor first, then cascades to the
    // object held only by its property.
    assert_eq!(
        out(
            "<?php class A { function __destruct(){ echo 'A'; } } \
             class B { public $a; function __destruct(){ echo 'B'; } } \
             $b=new B(); $b->a=new A(); unset($b); echo '|end';"
        ),
        "BA|end"
    );
}

#[test]
fn destruct_reassign_chain() {
    assert_eq!(
        out(
            "<?php class A { public $n; function __construct($n){$this->n=$n;} \
             function __destruct(){ echo 'd' . $this->n; } } \
             $a=new A(1); $a=new A(2); $a=new A(3); echo '|';"
        ),
        "d1d2|d3"
    );
}

// --- Step 25: double-quoted string interpolation ---

#[test]
fn interp_simple_variable() {
    assert_eq!(out(r#"<?php $x="W"; echo "hi $x!";"#), "hi W!");
}

#[test]
fn interp_forces_string_type() {
    // Interpolation always yields a string, even for a lone int variable.
    assert_eq!(
        out(r#"<?php $x=5; $y="$x"; echo ($y === "5") ? 'T' : 'F';"#),
        "T"
    );
}

#[test]
fn interp_array_bareword_key() {
    assert_eq!(
        out(r#"<?php $a=['k'=>'V']; echo "x $a[k] y";"#),
        "x V y"
    );
}

#[test]
fn interp_array_int_and_var_key() {
    assert_eq!(
        out(r#"<?php $a=[0=>'Z']; $i=0; echo "$a[0]/$a[$i]";"#),
        "Z/Z"
    );
}

#[test]
fn interp_property_access() {
    assert_eq!(
        out(r#"<?php class C { public $p="P"; } $o=new C; echo "v $o->p!";"#),
        "v P!"
    );
}

#[test]
fn interp_braced_complex() {
    assert_eq!(
        out(
            r#"<?php $x="W"; $a=['k'=>'V']; class C { public $p="P"; } $o=new C; echo "{$x}{$a['k']}{$o->p}";"#
        ),
        "WVP"
    );
}

#[test]
fn interp_braced_object_tostring() {
    assert_eq!(
        out(
            r#"<?php class C { function __toString(){ return "S"; } } $o=new C; echo "v {$o}.";"#
        ),
        "v S."
    );
}

#[test]
fn interp_multiple_parts_and_literals() {
    assert_eq!(
        out(r#"<?php $a="A"; $b="B"; echo "[$a-$b]";"#),
        "[A-B]"
    );
}

#[test]
fn interp_processes_escapes_in_literals() {
    // Step 29-4 (D-NEW): escape sequences in the literal parts of an
    // interpolated string must be unescaped (\t, \n, \$, \\), not emitted raw.
    assert_eq!(out(r#"<?php $v="W"; echo "x\t$v\n\$z\\end";"#), "x\tW\n$z\\end");
    // \x hex and octal escapes.
    assert_eq!(out(r#"<?php $v="W"; echo "$v\x41\101";"#), "WAA");
}

// --- Step 30: heredoc / nowdoc ---

#[test]
fn heredoc_strips_trailing_newline() {
    let src = "<?php $h = <<<EOD\nHello, world\nEOD;\necho '[', $h, ']';";
    assert_eq!(out(src), "[Hello, world]");
}

#[test]
fn heredoc_keeps_internal_newlines() {
    let src = "<?php $h = <<<EOD\nline1\nline2\nEOD;\necho $h;";
    assert_eq!(out(src), "line1\nline2");
}

#[test]
fn heredoc_interpolates_and_unescapes() {
    // \t in a heredoc body IS processed (unlike nowdoc); $n interpolates.
    let src = "<?php $n=\"W\"; $h = <<<EOD\nhi $n\\tend\nEOD;\necho $h;";
    assert_eq!(out(src), "hi W\tend");
}

#[test]
fn heredoc_backslash_quote_is_literal() {
    // Heredoc does NOT process \" (double quotes are literal); keeps \".
    let src = "<?php $h = <<<EOD\na\\\"b\nEOD;\necho $h;";
    assert_eq!(out(src), "a\\\"b");
}

#[test]
fn nowdoc_is_literal() {
    // Nowdoc: no interpolation, no escape processing.
    let src = "<?php $n=\"W\"; $h = <<<'EOD'\nhi $n\\tend\nEOD;\necho $h;";
    assert_eq!(out(src), "hi $n\\tend");
}

#[test]
fn heredoc_flexible_dedent() {
    // Closing marker indented 4 spaces -> strip 4 leading ws from each line.
    let src = "<?php $h = <<<EOD\n    line1\n      line2\n    EOD;\necho $h;";
    assert_eq!(out(src), "line1\n  line2");
}

#[test]
fn nowdoc_flexible_dedent() {
    let src = "<?php $h = <<<'EOD'\n        a\n          b\n        EOD;\necho $h;";
    assert_eq!(out(src), "a\n  b");
}

#[test]
fn heredoc_empty_body() {
    let src = "<?php $h = <<<EOD\nEOD;\necho '[', $h, ']';";
    assert_eq!(out(src), "[]");
}

// --- Step 27: preg_* regular expressions ---

#[test]
fn preg_match_captures() {
    assert_eq!(
        out(r#"<?php preg_match('/(\d+)-(\d+)/', 'ab 12-34 cd', $m); echo $m[0], '/', $m[1], '/', $m[2];"#),
        "12-34/12/34"
    );
}

#[test]
fn preg_match_return_value() {
    assert_eq!(out(r#"<?php echo preg_match('/x/', 'axb'), preg_match('/z/', 'axb');"#), "10");
}

#[test]
fn preg_match_no_match_empties_matches() {
    assert_eq!(
        out(r#"<?php preg_match('/z/', 'abc', $m); echo empty($m) ? 'E' : 'N';"#),
        "E"
    );
}

#[test]
fn preg_match_bad_pattern_returns_false() {
    assert_eq!(out(r#"<?php echo (preg_match('/[/', 'x') === false) ? 'F' : 'T';"#), "F");
}

#[test]
fn preg_replace_simple() {
    assert_eq!(out(r#"<?php echo preg_replace('/\d+/', '#', 'a1b22c333');"#), "a#b#c#");
}

#[test]
fn preg_replace_backreferences() {
    assert_eq!(out(r#"<?php echo preg_replace('/(\w)(\w)/', '$2$1', 'abcd');"#), "badc");
}

#[test]
fn preg_replace_case_insensitive_flag() {
    assert_eq!(out(r#"<?php echo preg_replace('/a/i', 'X', 'AaA');"#), "XXX");
}

#[test]
fn preg_split_basic() {
    assert_eq!(out(r#"<?php $p = preg_split('/,/', 'a,b,c'); echo $p[0], $p[1], $p[2];"#), "abc");
}

#[test]
fn preg_quote_escapes_metachars() {
    assert_eq!(out(r#"<?php echo preg_quote('a.b*c');"#), "a\\.b\\*c");
}

#[test]
fn preg_match_all_pattern_order() {
    assert_eq!(
        out(r#"<?php $n = preg_match_all('/\d/', 'a1b2c3', $all); echo $n, '|', $all[0][0], $all[0][1], $all[0][2];"#),
        "3|123"
    );
}

#[test]
fn preg_replace_callback_basic() {
    assert_eq!(
        out(r#"<?php echo preg_replace_callback('/\d/', function($m){ return $m[0] * 2; }, 'a1b2');"#),
        "a2b4"
    );
}

// --- Step 36: preg backreferences / lookaround (fancy-regex auto-fallback) ---

#[test]
fn preg_match_backreference() {
    // `\1` is a backreference the `regex` crate cannot compile; the engine must
    // fall back to fancy-regex. Oracle: preg_match('/(a)\1/', 'aa') == 1.
    assert_eq!(out(r#"<?php echo preg_match('/(a)\1/', 'aa');"#), "1");
}

#[test]
fn preg_match_named_backreference() {
    // `\k<c>` named backreference (fancy-regex). Oracle == 1.
    assert_eq!(out(r#"<?php echo preg_match('/(?<c>a)\k<c>/', 'aa');"#), "1");
}

#[test]
fn preg_match_lookbehind() {
    // `(?<=foo)` positive lookbehind. Oracle: preg_match('/(?<=foo)bar/','foobar') == 1.
    assert_eq!(out(r#"<?php echo preg_match('/(?<=foo)bar/', 'foobar');"#), "1");
}

#[test]
fn preg_match_lookahead() {
    // `(?=bar)` positive lookahead. Oracle: preg_match('/foo(?=bar)/','foobar') == 1.
    assert_eq!(out(r#"<?php echo preg_match('/foo(?=bar)/', 'foobar');"#), "1");
}

#[test]
fn preg_match_negative_lookahead() {
    // `(?!bar)` negative lookahead. Oracle: preg_match('/foo(?!bar)/','foobaz') == 1.
    assert_eq!(out(r#"<?php echo preg_match('/foo(?!bar)/', 'foobaz');"#), "1");
}

#[test]
fn preg_match_atomic_group() {
    // `(?>..)` atomic group. Oracle: preg_match('/a(?>bc|b)c/','abcc') == 1.
    assert_eq!(out(r#"<?php echo preg_match('/a(?>bc|b)c/', 'abcc');"#), "1");
}

#[test]
fn preg_replace_backreference_pattern() {
    // Backreference in the *pattern* (not the replacement). Oracle:
    // preg_replace('/(\w)\1/','X','aabb') == 'XX'.
    assert_eq!(out(r#"<?php echo preg_replace('/(\w)\1/', 'X', 'aabb');"#), "XX");
}

#[test]
fn preg_match_all_backreference() {
    // captures_iter on the fancy engine. Oracle: 3 matches, group columns a/b/c.
    assert_eq!(
        out(r#"<?php $n = preg_match_all('/(\w)\1/', 'aabbcc', $m); echo $n, '|', $m[0][0], $m[0][1], $m[0][2], '|', $m[1][0], $m[1][1], $m[1][2];"#),
        "3|aabbcc|abc"
    );
}

#[test]
fn preg_split_lookahead() {
    // split on a zero-width lookahead via the fancy engine. Oracle: a / ,b / ,c.
    assert_eq!(
        out(r#"<?php $p = preg_split('/(?=,)/', 'a,b,c'); echo $p[0], '~', $p[1], '~', $p[2];"#),
        "a~,b~,c"
    );
}

// Features fancy-regex 0.14 turns out to support — wider than step 27's scope-out
// note assumed. Each matches the oracle byte-for-byte (D-36 discovery).

#[test]
fn preg_match_recursion() {
    // `(?R)` whole-pattern recursion (balanced parens). Oracle: 1 when a
    // parenthesised group is present, 0 with no parens at all.
    assert_eq!(
        out(r#"<?php echo preg_match('/\((?:[^()]|(?R))*\)/', '(a(b)c)'), preg_match('/\((?:[^()]|(?R))*\)/', 'abc');"#),
        "10"
    );
}

#[test]
fn preg_match_conditional() {
    // `(?(1)yes|no)` conditional on group 1. Oracle == 1.
    assert_eq!(out(r#"<?php echo preg_match('/(a)?(?(1)b|c)/', 'ab');"#), "1");
}

#[test]
fn preg_match_keep_k() {
    // `\K` resets the match start. Oracle: $m[0] == 'bar'.
    assert_eq!(out(r#"<?php preg_match('/foo\Kbar/', 'foobar', $m); echo $m[0];"#), "bar");
}

#[test]
fn preg_match_all_g_anchor() {
    // `\G` anchors at the previous match end. Oracle: 3 consecutive 'a's.
    assert_eq!(out(r#"<?php echo preg_match_all('/\Ga/', 'aaab');"#), "3");
}

// Genuine scope-out (D-36.2): neither engine compiles these, so the `preg_*`
// function returns false/null while PHP would match. Documented, no crash.

#[test]
fn preg_scopeout_subroutine_returns_false() {
    // `(?1)` subroutine call — oracle matches (1); we return false.
    assert_eq!(out(r#"<?php echo (preg_match('/(a)(?1)/', 'aa') === false) ? 'F' : 'T';"#), "F");
}

#[test]
fn preg_scopeout_control_verb_returns_false() {
    // `(*SKIP)` backtracking control verb — oracle matches (1); we return false.
    assert_eq!(out(r#"<?php echo (preg_match('/a(*SKIP)b/', 'ab') === false) ? 'F' : 'T';"#), "F");
}

#[test]
fn preg_scopeout_callout_returns_false() {
    // `(?C1)` callout — oracle matches (1); we return false.
    assert_eq!(out(r#"<?php echo (preg_match('/a(?C1)b/', 'ab') === false) ? 'F' : 'T';"#), "F");
}

// Step 36-3 / 37-1: bug41638's pattern carries the `U` flag. BEFORE step 37 we
// ignored `U`, so `.*` stayed greedy → fancy-regex blew past the backtrack limit
// and 36-3 kept it from hanging by returning a bounded no-match (the D-36.4
// divergence). NOW that step 37-1 honours `U`, `.*` is lazy → the pattern is no
// longer catastrophic and matches PHP byte-for-byte, RESOLVING D-36.4.

#[test]
fn preg_match_all_bug41638_matches_php_with_ungreedy() {
    // Oracle (/tmp/t1.php): preg_match_all → 1.
    let src = r##"<?php echo preg_match_all('/([\'"])((.*(\\\1)*)*)\1/sU', "repeater id='loopt' dataSrc=subject columns=2", $m);"##;
    assert_eq!(out(src), "1");
}

#[test]
fn preg_replace_bug41638_matches_php_with_ungreedy() {
    // Oracle (/tmp/t2.php): the 'loopt' run is replaced by X.
    let src = r##"<?php echo preg_replace('/([\'"])((.*(\\\1)*)*)\1/sU', 'X', "repeater id='loopt' dataSrc=subject columns=2");"##;
    assert_eq!(out(src), "repeater id=X dataSrc=subject columns=2");
}

// Step 36-3 guard with a GENUINELY catastrophic pattern (no `U` to save it):
// greedy nested quantifiers + a backref forcing the fancy engine, with no match
// possible. Must terminate (backtrack-limit error → bounded no-match), never
// hang (captures_iter stop-on-Err) or panic (replace_all via try_replacen).

#[test]
fn preg_match_all_catastrophic_no_ungreedy_is_bounded() {
    // Oracle: preg_match_all('/(a+)+b\1/', '<28 a's>') == 0.
    let src = r#"<?php echo preg_match_all('/(a+)+b\1/', 'aaaaaaaaaaaaaaaaaaaaaaaaaaaa', $m);"#;
    assert_eq!(out(src), "0");
}

#[test]
fn preg_replace_catastrophic_no_ungreedy_is_bounded() {
    // No match → subject returned unchanged (no panic).
    let src = r#"<?php echo preg_replace('/(a+)+b\1/', 'X', 'aaaaaaaaaaaaaaaaaaaaaaaaaaaa');"#;
    assert_eq!(out(src), "aaaaaaaaaaaaaaaaaaaaaaaaaaaa");
}

// --- Step 37: PCRE modifier flags U / A / X / D ($ leniency) ---

#[test]
fn preg_flag_ungreedy_match() {
    // `U` swaps greediness: `.*` becomes lazy. Oracle: $m[0] == '<a>'.
    assert_eq!(out(r#"<?php preg_match('/<.*>/U', '<a> <b>', $m); echo $m[0];"#), "<a>");
}

#[test]
fn preg_flag_ungreedy_explicit_marker_flips_back() {
    // Under `U`, an explicit `?` flips that quantifier back to greedy. Oracle:
    // $m[0] == '<a> <b>'.
    assert_eq!(out(r#"<?php preg_match('/<.*?>/U', '<a> <b>', $m); echo $m[0];"#), "<a> <b>");
}

#[test]
fn preg_flag_ungreedy_replace() {
    // `U` through preg_replace. Oracle: 'X X'.
    assert_eq!(out(r#"<?php echo preg_replace('/<.*>/U', 'X', '<a> <b>');"#), "X X");
}

#[test]
fn preg_flag_anchored_matches_at_start() {
    // `A` (PCRE_ANCHORED): match only if it starts at offset 0. Oracle: 1.
    assert_eq!(out(r#"<?php echo preg_match('/foo/A', 'foobar');"#), "1");
}

#[test]
fn preg_flag_anchored_rejects_mid_string() {
    // Oracle: both 0 — `foo` not at start, `bar` not at start.
    assert_eq!(out(r#"<?php echo preg_match('/foo/A', 'xfoobar');"#), "0");
    assert_eq!(out(r#"<?php echo preg_match('/bar/A', 'foobar');"#), "0");
}

#[test]
fn preg_flag_anchored_with_backref_uses_fancy() {
    // `A` must anchor on the fancy fallback too (the backref forces fancy).
    // Oracle: 1 at start, 0 mid-string.
    assert_eq!(out(r#"<?php echo preg_match('/(a)\1/A', 'aa');"#), "1");
    assert_eq!(out(r#"<?php echo preg_match('/(a)\1/A', 'xaa');"#), "0");
}

#[test]
fn preg_flag_extra_x_is_noop() {
    // Uppercase `X` (PCRE_EXTRA) is deprecated in PCRE2 (PHP's engine) → no-op.
    // It must NOT strip whitespace the way lowercase `x` does. Oracle: matches
    // 'foo bar' literally (1), not 'foobar' (0).
    assert_eq!(out(r#"<?php echo preg_match('/foo bar/X', 'foo bar');"#), "1");
    assert_eq!(out(r#"<?php echo preg_match('/foo bar/X', 'foobar');"#), "0");
}

#[test]
fn preg_dollar_matches_before_trailing_newline_zero_width() {
    // PCRE default `$` (no D, no m) matches at end OR just before a single
    // trailing \n, and is ZERO-WIDTH. Oracle: $m[0] == 'foo' (not "foo\n").
    assert_eq!(out("<?php preg_match('/foo$/', \"foo\\n\", $m); echo $m[0];"), "foo");
}

#[test]
fn preg_dollar_default_leniency_counts() {
    // Oracle: 1 (trailing \n), 1 (bare), 0 (\n not final).
    assert_eq!(out("<?php echo preg_match('/foo$/', \"foo\\n\");"), "1");
    assert_eq!(out("<?php echo preg_match('/foo$/', 'foo');"), "1");
    assert_eq!(out("<?php echo preg_match('/foo$/', \"foo\\nbar\");"), "0");
}

#[test]
#[allow(non_snake_case)]
fn preg_dollar_endonly_D_flag() {
    // `D` (PCRE_DOLLAR_ENDONLY): `$` only at the absolute end. Oracle: 0, 1.
    assert_eq!(out("<?php echo preg_match('/foo$/D', \"foo\\n\");"), "0");
    assert_eq!(out("<?php echo preg_match('/foo$/D', 'foo');"), "1");
}

#[test]
fn preg_dollar_multiline_unaffected() {
    // `m` → `$` per line (and D is ignored under m). Oracle: 1.
    assert_eq!(out("<?php echo preg_match('/foo$/m', \"foo\\nbar\");"), "1");
}

#[test]
fn preg_dollar_in_char_class_is_literal() {
    // A `$` inside `[...]` is a literal dollar, NOT an anchor — must not be
    // rewritten. Oracle: preg_match('/[a$]+/', 'a$a') matches 'a$a'.
    assert_eq!(out(r#"<?php preg_match('/[a$]+/', 'a$a', $m); echo $m[0];"#), "a$a");
}

// --- Step 38: named arguments (nullsafe `?->` already landed in step 19) ---

#[test]
fn nullsafe_property_and_method_chain() {
    // Lock-in for step-19 nullsafe: a null receiver short-circuits `?->` to null
    // (no warning); a non-null receiver behaves normally. Oracle: 5|N|7|NM.
    let src = r#"<?php
class A { public $x = 5; public function m(){ return 7; } }
$a = new A(); $n = null;
echo $a?->x, "|", ($n?->x ?? 'N'), "|", $a?->m(), "|", ($n?->m() ?? 'NM');"#;
    assert_eq!(out(src), "5|N|7|NM");
}

#[test]
fn named_args_reorder() {
    // All named, out of order → bound by name. Oracle: 1-2-3.
    let src = r#"<?php function f($a,$b,$c){ return "$a-$b-$c"; } echo f(c:3, a:1, b:2);"#;
    assert_eq!(out(src), "1-2-3");
}

#[test]
fn named_args_mixed_positional_then_named() {
    // Leading positional, trailing named. Oracle: 1-2-3.
    let src = r#"<?php function f($a,$b,$c){ return "$a-$b-$c"; } echo f(1, c:3, b:2);"#;
    assert_eq!(out(src), "1-2-3");
}

#[test]
fn named_args_skip_to_default() {
    // A named arg may leave an earlier defaulted param at its default. Oracle:
    // 5/9 (b defaulted) and 1/2 (both named, reordered).
    let src = r#"<?php function g($a,$b=9){ return "$a/$b"; } echo g(a:5), '|', g(b:2, a:1);"#;
    assert_eq!(out(src), "5/9|1/2");
}

#[test]
fn named_args_unknown_parameter_is_catchable_error() {
    // Oracle: Uncaught Error "Unknown named parameter $z" (catchable).
    let src = r#"<?php function f($a){} try { f(z:1); } catch (Error $e) { echo $e->getMessage(); }"#;
    assert_eq!(out(src), "Unknown named parameter $z");
}

#[test]
fn named_args_overwrite_previous_is_catchable_error() {
    // Positional then named targeting the same param. Oracle: Error
    // "Named parameter $a overwrites previous argument".
    let src = r#"<?php function f($a){} try { f(1, a:2); } catch (Error $e) { echo $e->getMessage(); }"#;
    assert_eq!(out(src), "Named parameter $a overwrites previous argument");
}

#[test]
fn named_args_constructor_reorder() {
    // Named args to a constructor (step 38-2). Oracle: 1,2.
    let src = r#"<?php
class P { public $x; public $y; function __construct($x, $y){ $this->x=$x; $this->y=$y; } }
$p = new P(y:2, x:1); echo $p->x, ',', $p->y;"#;
    assert_eq!(out(src), "1,2");
}

#[test]
fn named_args_constructor_skip_to_default() {
    // Oracle: 5/9 (b defaulted), 1/2 (both named).
    let src = r#"<?php
class P { public $a; public $b; function __construct($a, $b=9){ $this->a=$a; $this->b=$b; } }
$p = new P(a:5); $q = new P(b:2, a:1); echo $p->a, '/', $p->b, '|', $q->a, '/', $q->b;"#;
    assert_eq!(out(src), "5/9|1/2");
}

#[test]
fn named_args_instance_method() {
    // Named args to an instance method (step 38-3). Oracle: 10 - 1 = 9.
    let src = r#"<?php
class C { function sub($a, $b){ return $a - $b; } }
$c = new C(); echo $c->sub(b:1, a:10);"#;
    assert_eq!(out(src), "9");
}

#[test]
fn named_args_static_method() {
    // Named args to a static method. Oracle: 10 - 1 = 9.
    let src = r#"<?php
class C { static function sub($a, $b){ return $a - $b; } }
echo C::sub(b:1, a:10);"#;
    assert_eq!(out(src), "9");
}

#[test]
fn named_args_by_reference_parameter() {
    // A named arg targeting a by-ref parameter binds the caller's variable
    // (step 38-4). Oracle: $n becomes 6.
    let src = r#"<?php function inc(&$x){ $x++; } $n = 5; inc(x: $n); echo $n;"#;
    assert_eq!(out(src), "6");
}

#[test]
fn variadic_param_collects_positional() {
    // `...$n` collects all positional args into a 0-indexed array (step 38-5).
    let src = r#"<?php function sum(...$n){ $t=0; foreach($n as $v) $t+=$v; return $t; } echo sum(1,2,3,4);"#;
    assert_eq!(out(src), "10");
}

#[test]
fn variadic_param_after_fixed() {
    // A fixed param then `...$rest`; rest may be empty. Oracle: 1,2,3 | 9.
    let src = r#"<?php function f($a, ...$rest){ $s = "$a"; foreach($rest as $v) $s .= ",$v"; return $s; } echo f(1,2,3), '|', f(9);"#;
    assert_eq!(out(src), "1,2,3|9");
}

#[test]
fn variadic_param_keys_are_sequential() {
    // The collected array is keyed 0,1,2… Oracle: 0a1b2c.
    let src = r#"<?php function f(...$xs){ $s=''; foreach($xs as $k=>$v) $s .= "$k$v"; return $s; } echo f('a','b','c');"#;
    assert_eq!(out(src), "0a1b2c");
}

#[test]
fn named_args_positional_after_named_is_compile_fatal() {
    // A positional argument after a named one is a PHP compile-time Fatal error
    // (not a catchable Error). Oracle message verified against PHP 8.5.
    let o = run_source(b"t.php", b"<?php function f($a,$b){} f(a:1, 2);").expect("lowers");
    match o.fatal {
        Some(PhpError::Error(m)) => {
            assert_eq!(m, "Cannot use positional argument after named argument")
        }
        other => panic!("expected compile fatal, got {other:?}"),
    }
}

// --- Step 40: argument unpacking / spread `f(...$arr)` ---

#[test]
fn spread_basic_positional() {
    // Int-keyed array unpacked into positional params. Oracle: 1-2-3.
    let src = r#"<?php function f($a,$b,$c){ return "$a-$b-$c"; } echo f(...[1,2,3]);"#;
    assert_eq!(out(src), "1-2-3");
}

#[test]
fn spread_leading_positional_then_spread() {
    // A plain positional may precede a spread. Oracle: 1-2-3.
    let src = r#"<?php function f($a,$b,$c){ return "$a-$b-$c"; } echo f(1, ...[2,3]);"#;
    assert_eq!(out(src), "1-2-3");
}

#[test]
fn spread_multiple_arrays() {
    // Two spreads concatenate positionally. Oracle: 1-2-3.
    let src = r#"<?php function f($a,$b,$c){ return "$a-$b-$c"; } echo f(...[1], ...[2,3]);"#;
    assert_eq!(out(src), "1-2-3");
}

#[test]
fn spread_into_variadic() {
    // Unpacked int-keyed array collected by `...$args`, re-keyed 0,1,2.
    let src = r#"<?php function f(...$n){ $s=''; foreach($n as $k=>$v) $s.="$k:$v "; return $s; } echo f(...[1,2,3]);"#;
    assert_eq!(out(src), "0:1 1:2 2:3 ");
}

#[test]
fn spread_string_keys_become_named() {
    // String keys map to named parameters by name. Oracle: 1-2-3.
    let src = r#"<?php function f($a,$b,$c){ return "$a-$b-$c"; } echo f(...['a'=>1,'b'=>2,'c'=>3]);"#;
    assert_eq!(out(src), "1-2-3");
}

#[test]
fn spread_partial_string_keys_use_defaults() {
    // String keys fill named params; an omitted middle keeps its default.
    let src = r#"<?php function f($a,$b='B',$c='C'){ return "$a-$b-$c"; } echo f(...['a'=>1,'c'=>3]);"#;
    assert_eq!(out(src), "1-B-3");
}

#[test]
fn spread_int_keys_ignore_key_value() {
    // Non-sequential int keys are appended in iteration order; the key is ignored.
    let src = r#"<?php function f($a,$b,$c){ return "$a-$b-$c"; } echo f(...[5=>'x',2=>'y',9=>'z']);"#;
    assert_eq!(out(src), "x-y-z");
}

#[test]
fn spread_non_array_is_typeerror() {
    // Unpacking a scalar raises a catchable TypeError. Oracle message verified.
    let src = r#"<?php function f($a){} try { f(...5); } catch (\TypeError $e) { echo $e->getMessage(); }"#;
    assert_eq!(out(src), "Only arrays and Traversables can be unpacked, int given");
}

#[test]
fn spread_unknown_named_is_error() {
    // A string key with no matching param raises "Unknown named parameter".
    let src = r#"<?php function f($a){} try { f(...['z'=>1]); } catch (\Error $e) { echo $e->getMessage(); }"#;
    assert_eq!(out(src), "Unknown named parameter $z");
}

#[test]
fn spread_empty_array() {
    // Unpacking an empty array supplies no arguments. Oracle: 0 elems.
    let src = r#"<?php function f(...$n){ $c=0; foreach($n as $v) $c++; return $c; } echo f(...[]);"#;
    assert_eq!(out(src), "0");
}

#[test]
fn spread_overwrite_previous_is_error() {
    // A positional then a spread string key for the same slot overwrites. Oracle Error.
    let src = r#"<?php function f($a,$b,$c){} try { f(1, ...['a'=>9,'b'=>2,'c'=>3]); } catch (\Error $e) { echo $e->getMessage(); }"#;
    assert_eq!(out(src), "Named parameter $a overwrites previous argument");
}

#[test]
fn spread_int_after_string_key_during_unpacking_is_error() {
    // Within unpacking, an int key after a string key is a catchable Error.
    let src = r#"<?php function f($x, ...$r){} try { f(1, ...['k'=>2, 0=>3]); } catch (\Error $e) { echo $e->getMessage(); }"#;
    assert_eq!(
        out(src),
        "Cannot use positional argument after named argument during unpacking"
    );
}

#[test]
fn spread_named_after_spread() {
    // Explicit named arguments may follow a spread. Oracle: 1-2-3.
    let src = r#"<?php function f($a,$b,$c){ return "$a-$b-$c"; } echo f(...[1], c:3, b:2);"#;
    assert_eq!(out(src), "1-2-3");
}

#[test]
fn spread_traversable_generator() {
    // A Traversable (generator) unpacks like an array; int keys → positional.
    let src = r#"<?php
function gen(){ yield 1; yield 2; yield 3; }
function f(...$n){ $t=0; foreach($n as $v) $t+=$v; return $t; }
echo f(...gen());"#;
    assert_eq!(out(src), "6");
}

#[test]
fn spread_generator_string_keys_become_named() {
    // A generator yielding string keys feeds named-into-variadic.
    let src = r#"<?php
function gen(){ yield 'x' => 1; yield 'y' => 2; }
function f(...$n){ $s=''; foreach($n as $k=>$v) $s.="$k:$v "; return $s; }
echo f(...gen());"#;
    assert_eq!(out(src), "x:1 y:2 ");
}

#[test]
fn spread_positional_after_unpacking_is_compile_fatal() {
    // A positional argument after a spread is a compile-time Fatal. Oracle verified.
    let o = run_source(b"t.php", b"<?php function f($a,$b,$c){} f(...[1,2], 3);").expect("lowers");
    match o.fatal {
        Some(PhpError::Error(m)) => {
            assert_eq!(m, "Cannot use positional argument after argument unpacking")
        }
        other => panic!("expected compile fatal, got {other:?}"),
    }
}

#[test]
fn spread_after_named_is_compile_fatal() {
    // Unpacking after a named argument is a compile-time Fatal. Oracle verified.
    let o = run_source(b"t.php", b"<?php function f($a,$b){} f(a:1, ...['b'=>2]);").expect("lowers");
    match o.fatal {
        Some(PhpError::Error(m)) => {
            assert_eq!(m, "Cannot use argument unpacking after named arguments")
        }
        other => panic!("expected compile fatal, got {other:?}"),
    }
}

#[test]
fn spread_into_method_and_constructor() {
    // Spread works for instance methods and `new` (step 40-1).
    let src = r#"<?php
class C {
    public $sum;
    function __construct($a, $b){ $this->sum = $a + $b; }
    function diff($a, $b){ return $a - $b; }
}
$c = new C(...[10, 3]);
echo $c->sum, '|', $c->diff(...[10, 3]);"#;
    assert_eq!(out(src), "13|7");
}

#[test]
fn spread_into_static_method() {
    // Spread works for static method calls (step 40-1).
    let src = r#"<?php
class C { static function add($a, $b){ return $a + $b; } }
echo C::add(...[4, 5]);"#;
    assert_eq!(out(src), "9");
}

// --- Step 40-2: named-into-variadic collection ---

#[test]
fn named_into_variadic_explicit() {
    // Explicit named args with no matching param collect into `...$args` keyed by name.
    let src = r#"<?php function f(...$args){ $s=''; foreach($args as $k=>$v) $s.="$k:$v "; return $s; } echo f(x:1, y:2);"#;
    assert_eq!(out(src), "x:1 y:2 ");
}

#[test]
fn named_into_variadic_after_fixed() {
    // A fixed param then a named arg into the variadic (string-keyed).
    let src = r#"<?php function f($a, ...$rest){ $s="$a|"; foreach($rest as $k=>$v) $s.="$k:$v "; return $s; } echo f(1, k:2);"#;
    assert_eq!(out(src), "1|k:2 ");
}

#[test]
fn spread_named_into_variadic() {
    // Spread string keys collect into the variadic by name (step 40-2).
    let src = r#"<?php function f(...$args){ $s=''; foreach($args as $k=>$v) $s.="$k:$v "; return $s; } echo f(...['x'=>1,'y'=>2]);"#;
    assert_eq!(out(src), "x:1 y:2 ");
}

// --- Step 28: real stack-trace frames ---

#[test]
fn trace_string_nested_functions() {
    let src = "<?php\n\
        function a() { b(); }\n\
        function b() { try { throw new Exception('x'); } catch (Exception $e) { echo $e->getTraceAsString(); } }\n\
        a();\n";
    assert_eq!(out(src), "#0 t.php(2): b()\n#1 t.php(4): a()\n#2 {main}");
}

#[test]
fn trace_string_method() {
    let src = "<?php\n\
        class C {\n\
        function m() { throw new Exception('z'); }\n\
        }\n\
        $c = new C();\n\
        try { $c->m(); } catch (Exception $e) { echo $e->getTraceAsString(); }\n";
    assert_eq!(out(src), "#0 t.php(6): C->m()\n#1 {main}");
}

#[test]
fn get_trace_array_fields() {
    let src = "<?php\n\
        function a() { b(); }\n\
        function b() { try { throw new Exception('x'); } catch (Exception $e) { \
            $t = $e->getTrace(); echo $t[0]['function'], '@', $t[0]['line'], '|', $t[1]['function'], '@', $t[1]['line']; } }\n\
        a();\n";
    assert_eq!(out(src), "b@2|a@4");
}

#[test]
fn uncaught_render_includes_frames() {
    let src = "<?php\n\
        function a() { b(); }\n\
        function b() { throw new Exception('boom'); }\n\
        a();\n";
    let r = rendered(src);
    assert!(
        r.contains("Stack trace:\n#0 t.php(2): b()\n#1 t.php(4): a()\n#2 {main}\n  thrown in t.php on line 3"),
        "rendered was:\n{r}"
    );
}

// ---- Step 39: generators (yield) ----

#[test]
fn generator_basic_current_next() {
    // Calling a generator function does not run the body; it returns a lazy
    // Generator. current() starts it (runs to the first yield); next() advances.
    assert_eq!(
        out("<?php function g(){yield 1;yield 2;} $g=g(); echo $g->current(); $g->next(); echo $g->current();"),
        "12"
    );
}

#[test]
fn generator_lazy_no_run_on_call() {
    // The body must not execute until the generator is first advanced.
    assert_eq!(
        out("<?php function g(){echo 'body'; yield 1;} $g=g(); echo 'made'; $g->current();"),
        "madebody"
    );
}

#[test]
fn generator_key_and_valid() {
    // Auto-keys start at 0 and increment per yield; valid() is false once done.
    assert_eq!(
        out("<?php function g(){yield 10;yield 20;} $g=g(); echo $g->key(),':',$g->current(); $g->next(); echo '|',$g->key(),':',$g->current(); $g->next(); echo $g->valid()?'T':'F';"),
        "0:10|1:20F"
    );
}

#[test]
fn generator_foreach_key_value() {
    // `foreach` drives the generator: start, then current/key -> bind, body, next.
    assert_eq!(
        out(r#"<?php function g(){yield 1;yield 2;yield 3;} foreach(g() as $k=>$v) echo "$k:$v ";"#),
        "0:1 1:2 2:3 "
    );
}

#[test]
fn generator_foreach_value_only() {
    assert_eq!(
        out("<?php function g(){yield 'a';yield 'b';yield 'c';} foreach(g() as $v) echo $v;"),
        "abc"
    );
}

#[test]
fn generator_foreach_break() {
    // `break` stops driving the generator early.
    assert_eq!(
        out("<?php function g(){yield 1;yield 2;yield 3;yield 4;} foreach(g() as $v){ if($v==3) break; echo $v; }"),
        "12"
    );
}

#[test]
fn generator_explicit_string_keys() {
    assert_eq!(
        out(r#"<?php function g(){yield "a"=>1; yield "b"=>2;} foreach(g() as $k=>$v) echo "$k:$v ";"#),
        "a:1 b:2 "
    );
}

#[test]
fn generator_auto_key_counter() {
    // An explicit int key `>=` the counter advances it (array-append rule); a
    // lower one does not. Verified against PHP 8.5: `k:1 0:2 0:3 1:4`.
    assert_eq!(
        out(r#"<?php function g(){yield 'k'=>1; yield 2; yield 0=>3; yield 4;} foreach(g() as $k=>$v) echo "$k:$v ";"#),
        "k:1 0:2 0:3 1:4 "
    );
}

#[test]
fn generator_bare_yield() {
    // `yield;` yields NULL under the auto key (which still advances).
    assert_eq!(
        out("<?php function g(){yield; yield;} $g=g(); echo $g->key(),'='; $g->next(); echo $g->key();"),
        "0=1"
    );
}

#[test]
fn generator_send_delivers_value() {
    // send($v) makes the suspended `yield` expression evaluate to $v.
    assert_eq!(
        out("<?php function g(){ $a = yield 1; echo \"a=$a;\"; $b = yield 2; echo \"b=$b;\"; } \
            $g=g(); $g->current(); $g->send('X'); $g->send('Y');"),
        "a=X;b=Y;"
    );
}

#[test]
fn generator_send_on_unstarted_primes_then_delivers() {
    // send() on a fresh generator runs to the first yield, then delivers the
    // value to it. Verified against PHP 8.5.
    assert_eq!(
        out("<?php function g(){ $a = yield 1; echo \"sa=$a;\"; yield 2; } \
            $g=g(); echo $g->send('Z');"),
        "sa=Z;2"
    );
}

#[test]
fn generator_return_and_get_return() {
    // `return $v` in a generator ends it; getReturn() exposes $v afterwards.
    assert_eq!(
        out("<?php function r(){ yield 1; yield 2; return 99; } \
            $x=r(); foreach($x as $v) echo $v; echo '|'; echo $x->getReturn();"),
        "12|99"
    );
}

#[test]
fn generator_yield_from_array() {
    // `yield from` preserves delegate keys and does NOT advance the outer
    // auto-key counter. Verified against PHP 8.5: `0:1 0:10 1:20 1:2`.
    assert_eq!(
        out(r#"<?php function g(){yield 1; yield from [10,20]; yield 2;} foreach(g() as $k=>$v) echo "$k:$v ";"#),
        "0:1 0:10 1:20 1:2 "
    );
}

#[test]
fn generator_yield_from_string_keys() {
    assert_eq!(
        out(r#"<?php function g(){yield from ['x'=>1, 'y'=>2];} foreach(g() as $k=>$v) echo "$k:$v ";"#),
        "x:1 y:2 "
    );
}

#[test]
fn generator_yield_from_generator_return() {
    // Delegating to a sub-generator re-yields its pairs; the `yield from`
    // expression evaluates to the sub-generator's return value.
    assert_eq!(
        out(r#"<?php
            function inner(){ yield 1; yield 2; return 'R'; }
            function outer(){ $r = yield from inner(); echo "r=$r;"; yield 3; }
            foreach(outer() as $k=>$v) echo "$k:$v ";"#),
        "0:1 1:2 r=R;0:3 "
    );
}

#[test]
fn generator_yield_from_send_forwarding() {
    // send() through a `yield from` reaches the delegated sub-generator.
    assert_eq!(
        out(r#"<?php
            function sub(){ $a = yield 1; echo "sa=$a;"; $b = yield 2; echo "sb=$b;"; }
            function dele(){ yield from sub(); }
            $d = dele(); $d->current(); $d->send('X'); $d->send('Y');"#),
        "sa=X;sb=Y;"
    );
}

#[test]
fn generator_closure() {
    // A closure whose body contains `yield` is itself a generator.
    assert_eq!(
        out("<?php $g = function(){ yield 1; yield 2; yield 3; }; foreach($g() as $v) echo $v;"),
        "123"
    );
}

#[test]
fn generator_closure_captures() {
    // Captured variables are available in the generator body.
    assert_eq!(
        out("<?php $base = 10; $g = function() use ($base){ yield $base+1; yield $base+2; }; foreach($g() as $v) echo $v,' ';"),
        "11 12 "
    );
}

#[test]
fn generator_get_return_auto_primes() {
    // getReturn() on a generator that returns before any yield auto-primes it.
    assert_eq!(
        out("<?php function g(){ return 42; yield 24; } $g=g(); echo $g->getReturn();"),
        "42"
    );
}

#[test]
fn generator_instanceof_interfaces() {
    // A Generator is a Generator/Iterator/Traversable, but not e.g. Countable.
    assert_eq!(
        out("<?php function g(){yield 1;} $g=g(); \
            echo ($g instanceof Generator)?'1':'0', ($g instanceof Iterator)?'1':'0', \
            ($g instanceof Traversable)?'1':'0', ($g instanceof Countable)?'1':'0';"),
        "1110"
    );
}

#[test]
fn generator_rewind_at_start_ok() {
    // rewind() before advancing is allowed (and starts the generator).
    assert_eq!(
        out("<?php function g(){yield 1;yield 2;} $g=g(); $g->rewind(); echo $g->current();"),
        "1"
    );
}

#[test]
fn generator_rewind_after_advance_fatals() {
    // rewind() after the generator has advanced throws. The VM raises a *catchable*
    // \Exception (PHP's actual behaviour, oracle-confirmed), not the tree-walker's
    // plain engine `Error`; uncaught, it renders the fatal banner with the message.
    let o = run_source(
        b"t.php",
        b"<?php function g(){yield 1;yield 2;} $g=g(); $g->next(); $g->rewind();",
    )
    .expect("lowers");
    assert!(
        matches!(&o.fatal, Some(PhpError::Thrown(_))),
        "expected a thrown Exception, got: {:?}",
        o.fatal
    );
    assert!(
        String::from_utf8_lossy(&o.rendered)
            .contains("Cannot rewind a generator that was already run"),
        "rendered was: {}",
        String::from_utf8_lossy(&o.rendered)
    );
}

// --- Tooling hardening: evaluator call-depth guard (prevents host SIGABRT) ---

#[test]
fn deep_recursion_yields_clean_error_not_host_crash() {
    // Runaway recursion must surface a catchable Error (defensive depth guard),
    // not abort the host process via a native stack overflow. Run on a large
    // stack (like the phpt-runner's worker) so the guard fires before the native
    // stack would overflow — on a small thread the native stack would overflow
    // first regardless. RED state (without the guard) is a SIGABRT, which would
    // abort the whole test binary, so it is demonstrated empirically instead.
    // `PhpError`/`Zval` are `Rc`-based (not `Send`), so project the fatal to its
    // message (a `String`, which is `Send`) inside the worker before returning.
    let msg = std::thread::Builder::new()
        .stack_size(1 << 30)
        .spawn(|| match run_source(b"t.php", b"<?php function r($n){ return r($n + 1); } r(0);")
            .expect("lowers")
            .fatal
        {
            Some(PhpError::Error(m)) => Some(m),
            _ => None,
        })
        .expect("spawn worker")
        .join()
        .expect("worker panicked");
    match msg {
        Some(m) => assert!(m.contains("call stack depth"), "unexpected message: {m}"),
        None => panic!("expected depth-guard error, got none"),
    }
}

// ---- step 49: magic constants -------------------------------------------

#[test]
fn magic_line_file_dir() {
    // `__LINE__` is the line of the constant itself; `out` runs `t.php`.
    assert_eq!(out("<?php echo __LINE__;"), "1");
    assert_eq!(out("<?php\n\necho __LINE__;"), "3");
    assert_eq!(out("<?php echo __FILE__;"), "t.php");
    assert_eq!(out("<?php echo __DIR__;"), ".");
}

#[test]
fn magic_function_class_method() {
    // Free function: `__FUNCTION__`/`__METHOD__` are the bare name; `__CLASS__` "".
    assert_eq!(out("<?php function f(){ echo __FUNCTION__; } f();"), "f");
    assert_eq!(out("<?php function f(){ echo __METHOD__; } f();"), "f");
    assert_eq!(out("<?php function f(){ echo '['.__CLASS__.']'; } f();"), "[]");
    // Method: class name, method name, and `Class::method`.
    assert_eq!(
        out("<?php class C { function m(){ echo __CLASS__,'|',__FUNCTION__,'|',__METHOD__; } } (new C)->m();"),
        "C|m|C::m"
    );
}

#[test]
fn magic_top_level_and_closure() {
    // At the top level every name-scoped magic constant is the empty string.
    assert_eq!(out("<?php echo '['.__FUNCTION__.']'.'['.__CLASS__.']';"), "[][]");
    // `__FUNCTION__` inside a closure is PHP's `{closure}`.
    assert_eq!(out("<?php $f = function(){ echo __FUNCTION__; }; $f();"), "{closure}");
}

#[test]
fn magic_trait_and_namespace() {
    // `__TRAIT__` resolves to the trait name in the trait's method body.
    assert_eq!(
        out("<?php trait T { function m(){ echo __TRAIT__; } } class C { use T; } (new C)->m();"),
        "T"
    );
    // No namespaces in Tier 1: `__NAMESPACE__` is always empty.
    assert_eq!(out("<?php echo '['.__NAMESPACE__.']';"), "[]");
}

#[test]
fn predefined_error_and_path_constants() {
    assert_eq!(out("<?php echo E_ALL;"), "30719");
    assert_eq!(out("<?php echo E_WARNING | E_NOTICE;"), "10");
    assert_eq!(out("<?php echo E_ALL & ~E_NOTICE;"), "30711");
    assert_eq!(out("<?php echo DIRECTORY_SEPARATOR;"), "/");
    assert_eq!(out("<?php echo PATH_SEPARATOR;"), ":");
    assert_eq!(out("<?php echo PHP_SAPI;"), "cli");
}

// ---- step 49c: user-defined constants (define/constant/defined) ----------

#[test]
fn user_define_and_read() {
    assert_eq!(out("<?php define('FOO', 42); echo FOO;"), "42");
    assert_eq!(out("<?php define('GREETING', 'hi'); echo GREETING;"), "hi");
    // define() returns true; a second define of the same name returns false.
    assert_eq!(out("<?php echo define('X', 1) ? 't' : 'f';"), "t");
    assert_eq!(out("<?php define('X', 1); echo define('X', 2) ? 't' : 'f'; echo X;"), "f1");
}

#[test]
fn constant_and_defined_builtins() {
    assert_eq!(out("<?php define('A', 7); echo constant('A');"), "7");
    assert_eq!(out("<?php echo defined('NOPE') ? 'y' : 'n';"), "n");
    assert_eq!(out("<?php define('B', 1); echo defined('B') ? 'y' : 'n';"), "y");
    // Engine constants answer to defined()/constant() as well.
    assert_eq!(out("<?php echo defined('PHP_INT_MAX') ? 'y' : 'n';"), "y");
    assert_eq!(out("<?php echo constant('E_ALL');"), "30719");
}

#[test]
fn undefined_constant_is_error() {
    let o = php_runtime::run_source(b"t.php", b"<?php echo MISSING;").expect("lowers");
    let fatal = o.fatal.expect("undefined constant should fatal");
    assert!(
        format!("{fatal:?}").contains("Undefined constant"),
        "unexpected: {fatal:?}"
    );
}
