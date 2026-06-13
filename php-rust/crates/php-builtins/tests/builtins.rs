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

/// Run a snippet, returning the fatal error (panics if none was raised).
fn fatal(src: &str) -> PhpError {
    let reg = registry();
    let o = run_source_with(b"t.php", src.as_bytes(), &reg).expect("lowers");
    o.fatal.expect("expected a fatal error")
}

#[test]
fn count_arrays() {
    assert_eq!(out("<?php echo count([1, 2, 3]);"), "3");
    assert_eq!(out("<?php echo count([]);"), "0");
    assert_eq!(out("<?php echo count(['a' => 1, 'b' => 2]);"), "2");
    // Default (COUNT_NORMAL) does not descend into nested arrays.
    assert_eq!(out("<?php echo count([1, [2, 3], 4]);"), "3");
}

#[test]
fn count_recursive() {
    // mode 1 == COUNT_RECURSIVE: counts nested containers AND their elements.
    assert_eq!(out("<?php echo count([1, [2, 3], 4], 1);"), "5");
    assert_eq!(out("<?php echo count([[1, 2], [3, 4]], 1);"), "6");
}

#[test]
fn count_scalar_is_type_error() {
    match fatal("<?php count(5);") {
        PhpError::TypeError(m) => assert_eq!(
            m,
            "count(): Argument #1 ($value) must be of type Countable|array, int given"
        ),
        other => panic!("expected TypeError, got {other:?}"),
    }
    match fatal("<?php count(null);") {
        PhpError::TypeError(m) => assert!(m.contains("null given"), "message was: {m}"),
        other => panic!("expected TypeError, got {other:?}"),
    }
}

#[test]
fn array_keys_all() {
    assert_eq!(
        out("<?php var_dump(array_keys(['a' => 1, 'b' => 2, 7 => 3]));"),
        "array(3) {\n  [0]=>\n  string(1) \"a\"\n  [1]=>\n  string(1) \"b\"\n  [2]=>\n  int(7)\n}\n"
    );
    assert_eq!(out("<?php var_dump(array_keys([]));"), "array(0) {\n}\n");
}

#[test]
fn array_keys_with_search() {
    // Loose search returns positions of matching values, reindexed.
    assert_eq!(
        out("<?php var_dump(array_keys([1, 2, 1, 3, 1], 1));"),
        "array(3) {\n  [0]=>\n  int(0)\n  [1]=>\n  int(2)\n  [2]=>\n  int(4)\n}\n"
    );
    // Strict search: only the int 1 matches (not "1" or true).
    assert_eq!(
        out("<?php var_dump(array_keys(['1', 1, true], 1, true));"),
        "array(1) {\n  [0]=>\n  int(1)\n}\n"
    );
}

#[test]
fn array_values_reindexes() {
    assert_eq!(
        out("<?php var_dump(array_values(['a' => 10, 5 => 20, 'c' => 30]));"),
        "array(3) {\n  [0]=>\n  int(10)\n  [1]=>\n  int(20)\n  [2]=>\n  int(30)\n}\n"
    );
}

#[test]
fn in_array_loose_and_strict() {
    assert_eq!(out("<?php var_dump(in_array('1', [1, 2, 3]));"), "bool(true)\n");
    assert_eq!(out("<?php var_dump(in_array('1', [1, 2, 3], true));"), "bool(false)\n");
    assert_eq!(out("<?php var_dump(in_array(2, [1, 2, 3], true));"), "bool(true)\n");
    assert_eq!(out("<?php var_dump(in_array(9, [1, 2, 3]));"), "bool(false)\n");
}

#[test]
fn in_array_non_array_haystack_is_type_error() {
    match fatal("<?php in_array(1, 'x');") {
        PhpError::TypeError(m) => assert_eq!(
            m,
            "in_array(): Argument #2 ($haystack) must be of type array, string given"
        ),
        other => panic!("expected TypeError, got {other:?}"),
    }
}

#[test]
fn array_merge_reindexes_ints_overwrites_strings() {
    assert_eq!(
        out("<?php var_dump(array_merge([1, 2], ['a' => 3, 4], ['a' => 9]));"),
        "array(4) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(2)\n  [\"a\"]=>\n  int(9)\n  [2]=>\n  int(4)\n}\n"
    );
    assert_eq!(
        out("<?php var_dump(array_merge([5 => 'a'], [10 => 'b']));"),
        "array(2) {\n  [0]=>\n  string(1) \"a\"\n  [1]=>\n  string(1) \"b\"\n}\n"
    );
    assert_eq!(out("<?php var_dump(array_merge());"), "array(0) {\n}\n");
}

#[test]
fn array_merge_non_array_is_type_error() {
    match fatal("<?php array_merge([1], 'x');") {
        PhpError::TypeError(m) => assert_eq!(
            m,
            "array_merge(): Argument #2 must be of type array, string given"
        ),
        other => panic!("expected TypeError, got {other:?}"),
    }
}

#[test]
fn implode_joins() {
    assert_eq!(out("<?php echo implode(',', [1, 2, 3]);"), "1,2,3");
    assert_eq!(out("<?php echo implode([1, 2, 3]);"), "123"); // glue defaults to ""
    // Each element is string-coerced: true->"1", null->"", 2.5->"2.5".
    assert_eq!(out("<?php echo implode('-', [1, 'a', true, null, 2.5]);"), "1-a-1--2.5");
    assert_eq!(out("<?php echo implode(',', []);"), "");
}

#[test]
fn implode_array_separator_is_type_error() {
    // The legacy implode($array, $glue) order was removed in PHP 8.
    match fatal("<?php implode([1, 2], '-');") {
        PhpError::TypeError(m) => assert_eq!(
            m,
            "implode(): Argument #1 ($separator) must be of type string, array given"
        ),
        other => panic!("expected TypeError, got {other:?}"),
    }
}

#[test]
fn explode_splits() {
    assert_eq!(
        out("<?php var_dump(explode(',', 'a,b,c'));"),
        "array(3) {\n  [0]=>\n  string(1) \"a\"\n  [1]=>\n  string(1) \"b\"\n  [2]=>\n  string(1) \"c\"\n}\n"
    );
    // No separator occurrence: single-element array holding the whole string.
    assert_eq!(
        out("<?php var_dump(explode(',', 'abc'));"),
        "array(1) {\n  [0]=>\n  string(3) \"abc\"\n}\n"
    );
    // Multichar separator.
    assert_eq!(
        out("<?php echo implode('|', explode('::', 'a::b::c'));"),
        "a|b|c"
    );
}

#[test]
fn explode_limit() {
    // Positive limit: last element holds the unsplit remainder.
    assert_eq!(
        out("<?php var_dump(explode(',', 'a,b,c,d', 2));"),
        "array(2) {\n  [0]=>\n  string(1) \"a\"\n  [1]=>\n  string(5) \"b,c,d\"\n}\n"
    );
    // limit 0 behaves like 1: whole string.
    assert_eq!(out("<?php echo implode('|', explode(',', 'a,b,c', 0));"), "a,b,c");
    // Negative limit drops |limit| trailing pieces.
    assert_eq!(out("<?php echo implode('|', explode(',', 'a,b,c,d', -1));"), "a|b|c");
    assert_eq!(out("<?php echo count(explode(',', 'a,b,c', -5));"), "0");
}

#[test]
fn explode_empty_separator_is_value_error() {
    match fatal("<?php explode('', 'abc');") {
        PhpError::ValueError(m) => assert_eq!(
            m,
            "explode(): Argument #1 ($separator) must not be empty"
        ),
        other => panic!("expected ValueError, got {other:?}"),
    }
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
