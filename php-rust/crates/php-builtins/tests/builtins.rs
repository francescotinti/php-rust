//! Functional tests for the Tier 1 builtins (plan step 5): assert exact stdout
//! for scripts that call them via the evaluator + injected registry.

use php_builtins::registry;
use php_runtime::run_source_with;
use php_types::PhpError;

fn out(src: &str) -> String {
    let reg = registry();
    let o = run_source_with(b"t.php", src.as_bytes(), &reg).expect("lowers");
    assert!(o.fatal.is_none(), "unexpected fatal: {:?}", o.fatal);
    String::from_utf8(o.stdout).expect("utf8")
}

#[test]
fn var_dump_scalars() {
    assert_eq!(out("<?php var_dump(42);"), "int(42)\n");
    assert_eq!(out("<?php var_dump(-7);"), "int(-7)\n");
    assert_eq!(out("<?php var_dump(1.5);"), "float(1.5)\n");
    assert_eq!(out("<?php var_dump(1.0);"), "float(1)\n");
    assert_eq!(out("<?php var_dump(0.1 + 0.2);"), "float(0.30000000000000004)\n");
    assert_eq!(out("<?php var_dump('abc');"), "string(3) \"abc\"\n");
    assert_eq!(out("<?php var_dump('');"), "string(0) \"\"\n");
    assert_eq!(out("<?php var_dump(true);"), "bool(true)\n");
    assert_eq!(out("<?php var_dump(false);"), "bool(false)\n");
    assert_eq!(out("<?php var_dump(null);"), "NULL\n");
}

#[test]
fn var_dump_is_variadic() {
    assert_eq!(out("<?php var_dump(1, 'x', true);"), "int(1)\nstring(1) \"x\"\nbool(true)\n");
}

#[test]
fn var_dump_array() {
    // (array) cast is the only way to build an array in Tier 1 so far.
    assert_eq!(
        out("<?php var_dump((array)5);"),
        "array(1) {\n  [0]=>\n  int(5)\n}\n"
    );
}

#[test]
fn strlen_and_coercion() {
    assert_eq!(out("<?php echo strlen('hello');"), "5");
    assert_eq!(out("<?php echo strlen('');"), "0");
    assert_eq!(out("<?php echo strlen(12345);"), "5"); // int coerces to "12345"
}

#[test]
fn gettype_names() {
    assert_eq!(
        out("<?php echo gettype(1), '/', gettype(1.5), '/', gettype('x'), '/', gettype(true), '/', gettype(null);"),
        "integer/double/string/boolean/NULL"
    );
}

#[test]
fn type_predicates() {
    assert_eq!(out("<?php echo is_int(1) ? 't' : 'f';"), "t");
    assert_eq!(out("<?php echo is_int(1.0) ? 't' : 'f';"), "f");
    assert_eq!(out("<?php echo is_float(1.0) ? 't' : 'f';"), "t");
    assert_eq!(out("<?php echo is_string('x') ? 't' : 'f';"), "t");
    assert_eq!(out("<?php echo is_bool(true) ? 't' : 'f';"), "t");
    assert_eq!(out("<?php echo is_null(null) ? 't' : 'f';"), "t");
    assert_eq!(out("<?php echo is_array((array)1) ? 't' : 'f';"), "t");
    assert_eq!(out("<?php echo is_scalar(1) ? 't' : 'f';"), "t");
    assert_eq!(out("<?php echo is_scalar((array)1) ? 't' : 'f';"), "f");
}

#[test]
fn is_numeric_cases() {
    assert_eq!(out("<?php echo is_numeric('123') ? 't' : 'f';"), "t");
    assert_eq!(out("<?php echo is_numeric('1.5e3') ? 't' : 'f';"), "t");
    assert_eq!(out("<?php echo is_numeric('12abc') ? 't' : 'f';"), "f");
    assert_eq!(out("<?php echo is_numeric('abc') ? 't' : 'f';"), "f");
    assert_eq!(out("<?php echo is_numeric(42) ? 't' : 'f';"), "t");
}

#[test]
fn value_casts() {
    assert_eq!(out("<?php echo intval('42abc');"), "42");
    assert_eq!(out("<?php echo intval(3.9);"), "3");
    assert_eq!(out("<?php echo floatval('1.5x');"), "1.5");
    assert_eq!(out("<?php var_dump(strval(42));"), "string(2) \"42\"\n");
    assert_eq!(out("<?php echo boolval(0) ? 't' : 'f';"), "f");
    assert_eq!(out("<?php echo boolval('a') ? 't' : 'f';"), "t");
}

#[test]
fn undefined_function_is_fatal_after_output() {
    let reg = registry();
    let o = run_source_with(b"t.php", b"<?php echo 'a'; nope();", &reg).expect("lowers");
    assert_eq!(o.stdout, b"a");
    match o.fatal {
        Some(PhpError::Error(m)) => assert!(
            m.contains("Call to undefined function nope"),
            "message was: {m}"
        ),
        other => panic!("expected undefined-function Error, got {other:?}"),
    }
}
