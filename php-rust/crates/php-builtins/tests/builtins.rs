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
fn var_dump_object_public_props() {
    // Step 19-1: public-property objects dump in the exact 8.5 form.
    assert_eq!(
        out(
            "<?php class Point { public $x = 0; public $y = 0; \
             function __construct($x, $y) { $this->x = $x; $this->y = $y; } } \
             var_dump(new Point(3, 4));"
        ),
        "object(Point)#1 (2) {\n  [\"x\"]=>\n  int(3)\n  [\"y\"]=>\n  int(4)\n}\n"
    );
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
fn substr_offsets_and_lengths() {
    assert_eq!(out("<?php echo substr('hello', 1, 3);"), "ell");
    assert_eq!(out("<?php echo substr('hello', -2);"), "lo");
    assert_eq!(out("<?php echo substr('hello', 1, -1);"), "ell");
    assert_eq!(out("<?php var_dump(substr('hello', 10));"), "string(0) \"\"\n");
    assert_eq!(out("<?php echo substr('hello', -10);"), "hello");
    assert_eq!(out("<?php var_dump(substr('hello', 2, 0));"), "string(0) \"\"\n");
    assert_eq!(out("<?php var_dump(substr('hello', 1, -10));"), "string(0) \"\"\n");
}

#[test]
fn strpos_finds_and_misses() {
    assert_eq!(out("<?php var_dump(strpos('hello world', 'o'));"), "int(4)\n");
    assert_eq!(out("<?php var_dump(strpos('hello world', 'o', 5));"), "int(7)\n");
    assert_eq!(out("<?php var_dump(strpos('hello', 'z'));"), "bool(false)\n");
    assert_eq!(out("<?php var_dump(strpos('hello world', 'o', -5));"), "int(7)\n");
    assert_eq!(out("<?php var_dump(strpos('abc', ''));"), "int(0)\n");
}

#[test]
fn strpos_offset_out_of_range_is_value_error() {
    match fatal("<?php strpos('abc', 'a', 10);") {
        PhpError::ValueError(m) => assert_eq!(
            m,
            "strpos(): Argument #3 ($offset) must be contained in argument #1 ($haystack)"
        ),
        other => panic!("expected ValueError, got {other:?}"),
    }
}

#[test]
fn str_replace_scalar_and_arrays() {
    assert_eq!(out("<?php echo str_replace('o', '0', 'foobar');"), "f00bar");
    // Array search + array replace, applied element-wise and sequentially.
    assert_eq!(out("<?php echo str_replace(['a', 'b'], ['1', '2'], 'abc');"), "12c");
    assert_eq!(out("<?php echo str_replace(['a', 'b'], ['b', 'c'], 'a');"), "c");
    // Array search + scalar replace: same replacement for each search term.
    assert_eq!(out("<?php echo str_replace(['a', 'b'], 'X', 'abc');"), "XXc");
}

#[test]
fn str_replace_array_subject_returns_array() {
    assert_eq!(
        out("<?php var_dump(str_replace('a', 'X', ['abc', 'aaa']));"),
        "array(2) {\n  [0]=>\n  string(3) \"Xbc\"\n  [1]=>\n  string(3) \"XXX\"\n}\n"
    );
}

#[test]
fn sprintf_integers() {
    assert_eq!(out("<?php echo sprintf('%d', 42);"), "42");
    assert_eq!(out("<?php echo sprintf('[%5d]', 42);"), "[   42]");
    assert_eq!(out("<?php echo sprintf('[%05d]', 42);"), "[00042]");
    assert_eq!(out("<?php echo sprintf('[%-5d]', 42);"), "[42   ]");
    assert_eq!(out("<?php echo sprintf('%+d %+d', 5, -5);"), "+5 -5");
    assert_eq!(out("<?php echo sprintf('%d', 3.9);"), "3"); // float truncates
    assert_eq!(out("<?php echo sprintf('%u', -1);"), "18446744073709551615");
}

#[test]
fn sprintf_strings_and_padding() {
    assert_eq!(out("<?php echo sprintf('[%10s]', 'hi');"), "[        hi]");
    assert_eq!(out("<?php echo sprintf('[%-10s]', 'hi');"), "[hi        ]");
    assert_eq!(out("<?php echo sprintf('[%.3s]', 'hello');"), "[hel]");
    assert_eq!(out("<?php echo sprintf(\"[%'*10d]\", 42);"), "[********42]");
    assert_eq!(out("<?php echo sprintf('100%%');"), "100%");
}

#[test]
fn sprintf_floats() {
    assert_eq!(out("<?php echo sprintf('%f', 3.14159);"), "3.141590");
    assert_eq!(out("<?php echo sprintf('%.2f', 3.14159);"), "3.14");
    assert_eq!(out("<?php echo sprintf('[%08.2f]', 3.14159);"), "[00003.14]");
    assert_eq!(out("<?php echo sprintf('[%+08.2f]', -3.1);"), "[-0003.10]");
    // PHP exponent has a sign and no leading zeros: e+4, not e+04.
    assert_eq!(out("<?php echo sprintf('%e', 12345.678);"), "1.234568e+4");
}

#[test]
fn sprintf_bases_and_char() {
    assert_eq!(out("<?php echo sprintf('%x|%X|%o|%b', 255, 255, 8, 5);"), "ff|FF|10|101");
    assert_eq!(out("<?php echo sprintf('%c', 65);"), "A");
}

#[test]
fn sprintf_positional() {
    assert_eq!(out("<?php echo sprintf('%2$s %1$s', 'a', 'b');"), "b a");
}

#[test]
fn printf_writes_and_returns_length() {
    assert_eq!(out("<?php $n = printf('ab%d', 7); echo '|' . $n;"), "ab7|3");
}

#[test]
fn sprintf_missing_arg_is_argument_count_error() {
    match fatal("<?php sprintf('%d %d', 1);") {
        PhpError::ArgumentCountError(m) => {
            assert_eq!(m, "3 arguments are required, 2 given")
        }
        other => panic!("expected ArgumentCountError, got {other:?}"),
    }
}

#[test]
fn abs_numbers_and_strings() {
    assert_eq!(out("<?php var_dump(abs(-5));"), "int(5)\n");
    assert_eq!(out("<?php var_dump(abs(-5.5));"), "float(5.5)\n");
    assert_eq!(out("<?php var_dump(abs('-7'));"), "int(7)\n"); // numeric string -> int
    assert_eq!(out("<?php var_dump(abs('-3.5'));"), "float(3.5)\n");
    assert_eq!(out("<?php var_dump(abs(5));"), "int(5)\n");
}

#[test]
fn abs_non_numeric_is_type_error() {
    match fatal("<?php abs('abc');") {
        PhpError::TypeError(m) => assert_eq!(
            m,
            "abs(): Argument #1 ($num) must be of type int|float, string given"
        ),
        other => panic!("expected TypeError, got {other:?}"),
    }
}

#[test]
fn max_and_min() {
    assert_eq!(out("<?php var_dump(max(1, 5, 3));"), "int(5)\n");
    assert_eq!(out("<?php var_dump(max([1, 5, 3]));"), "int(5)\n");
    assert_eq!(out("<?php var_dump(min(4, 2, 8));"), "int(2)\n");
    assert_eq!(out("<?php var_dump(min([3, '1', 2]));"), "string(1) \"1\"\n");
    assert_eq!(out("<?php var_dump(max(1, 1.5));"), "float(1.5)\n");
    // Returned value keeps its original type; comparison is loose (numeric).
    assert_eq!(out("<?php var_dump(max(1, '10', 2));"), "string(2) \"10\"\n");
    assert_eq!(out("<?php var_dump(max('apple', 'banana'));"), "string(6) \"banana\"\n");
}

#[test]
fn max_empty_array_is_value_error() {
    match fatal("<?php max([]);") {
        PhpError::ValueError(m) => assert_eq!(
            m,
            "max(): Argument #1 ($value) must contain at least one element"
        ),
        other => panic!("expected ValueError, got {other:?}"),
    }
}

#[test]
fn max_no_args_is_argument_count_error() {
    match fatal("<?php max();") {
        PhpError::ArgumentCountError(m) => {
            assert_eq!(m, "max() expects at least 1 argument, 0 given")
        }
        other => panic!("expected ArgumentCountError, got {other:?}"),
    }
}

#[test]
fn max_single_non_array_is_type_error() {
    match fatal("<?php max(5);") {
        PhpError::TypeError(m) => assert_eq!(
            m,
            "max(): Argument #1 ($value) must be of type array, int given"
        ),
        other => panic!("expected TypeError, got {other:?}"),
    }
}

#[test]
fn print_r_scalars() {
    assert_eq!(out("<?php print_r(42);"), "42");
    assert_eq!(out("<?php print_r('hi');"), "hi");
    assert_eq!(out("<?php print_r(true);"), "1");
    assert_eq!(out("<?php print_r(false);"), "");
    assert_eq!(out("<?php print_r(null);"), "");
}

#[test]
fn print_r_simple_array() {
    assert_eq!(
        out("<?php print_r([1, 2, 3]);"),
        "Array\n(\n    [0] => 1\n    [1] => 2\n    [2] => 3\n)\n"
    );
}

#[test]
fn print_r_nested_array() {
    assert_eq!(
        out("<?php print_r(['a' => 1, 'b' => [2, 3]]);"),
        "Array\n(\n    [a] => 1\n    [b] => Array\n        (\n            [0] => 2\n            [1] => 3\n        )\n\n)\n"
    );
}

#[test]
fn print_r_return_mode() {
    // With a truthy second argument, the output is returned, not printed.
    assert_eq!(
        out("<?php $s = print_r([1, 2], true); echo '[' . $s . ']';"),
        "[Array\n(\n    [0] => 1\n    [1] => 2\n)\n]"
    );
}

// --- by-reference array builtins (step 11c) ---

#[test]
fn array_push_appends_and_returns_count() {
    // Mutates the caller's array by reference and returns the new element count.
    assert_eq!(
        out("<?php $a=[1,2]; $n=array_push($a,3,4); echo $n; echo '|'; echo implode(',',$a);"),
        "4|1,2,3,4"
    );
    // Pushing onto an empty array.
    assert_eq!(out("<?php $a=[]; echo array_push($a,1);"), "1");
}

// --- var_dump / print_r of reference elements (step 11d-4) ---

#[test]
fn var_dump_marks_shared_reference_element() {
    // A reference element shared with a live alias prints with an `&` marker.
    assert_eq!(
        out("<?php $x=5; $a=[1,2]; $a[0]=&$x; var_dump($a);"),
        "array(2) {\n  [0]=>\n  &int(5)\n  [1]=>\n  int(2)\n}\n"
    );
}

#[test]
fn var_dump_no_marker_when_reference_not_shared() {
    // After the other alias is unset the element is the sole holder of the cell,
    // so var_dump prints it as a plain value (no `&`).
    assert_eq!(
        out("<?php $x=5; $a=[0]; $a[0]=&$x; unset($x); var_dump($a);"),
        "array(1) {\n  [0]=>\n  int(5)\n}\n"
    );
}

#[test]
fn var_dump_marks_reference_to_array_element() {
    assert_eq!(
        out("<?php $x=[1,2]; $a=['k'=>0]; $a['k']=&$x; var_dump($a);"),
        "array(1) {\n  [\"k\"]=>\n  &array(2) {\n    [0]=>\n    int(1)\n    [1]=>\n    int(2)\n  }\n}\n"
    );
}

#[test]
fn print_r_does_not_mark_references() {
    // print_r is transparent: it derefs without an `&` marker.
    assert_eq!(
        out("<?php $x=5; $a=[1,2]; $a[0]=&$x; print_r($a);"),
        "Array\n(\n    [0] => 5\n    [1] => 2\n)\n"
    );
}

#[test]
fn print_r_recurses_into_reference_to_array() {
    assert_eq!(
        out("<?php $x=[1,2]; $a=['k'=>0]; $a['k']=&$x; print_r($a);"),
        "Array\n(\n    [k] => Array\n        (\n            [0] => 1\n            [1] => 2\n        )\n\n)\n"
    );
}

#[test]
fn array_push_on_non_array_is_type_error() {
    let err = fatal("<?php $x=5; array_push($x,1);");
    match err {
        PhpError::TypeError(m) => assert_eq!(
            m,
            "array_push(): Argument #1 ($array) must be of type array, int given"
        ),
        other => panic!("expected TypeError, got {other:?}"),
    }
}

#[test]
fn sort_orders_values_and_reindexes() {
    assert_eq!(out("<?php $a=[3,1,2]; sort($a); echo implode(',',$a);"), "1,2,3");
    // String keys are dropped and values reindexed from 0.
    assert_eq!(
        out("<?php $a=['b'=>2,'a'=>1]; sort($a); echo implode(',',$a);"),
        "1,2"
    );
}

#[test]
fn array_pop_removes_last_and_returns_it() {
    assert_eq!(
        out("<?php $a=[1,2,3]; $x=array_pop($a); echo $x; echo '|'; echo implode(',',$a); echo '|'; echo count($a);"),
        "3|1,2|2"
    );
    // Popping an empty array yields NULL and leaves it empty.
    assert_eq!(out("<?php $a=[]; var_dump(array_pop($a));"), "NULL\n");
}

#[test]
fn array_shift_removes_first_and_reindexes() {
    assert_eq!(
        out("<?php $a=[1,2,3]; $x=array_shift($a); echo $x; echo '|'; echo implode(',',$a);"),
        "1|2,3"
    );
    assert_eq!(out("<?php $a=[]; var_dump(array_shift($a));"), "NULL\n");
}

#[test]
fn array_shift_reindexes_int_keys_preserves_string_keys() {
    // The remaining integer keys renumber from 0; string keys are kept as-is.
    assert_eq!(
        out("<?php $a=['x'=>1,7=>2,'y'=>3]; array_shift($a); echo implode(',',array_keys($a));"),
        "0,y"
    );
}

#[test]
fn array_pop_preserves_remaining_keys() {
    // array_pop does not reindex: the survivors keep their original keys.
    assert_eq!(
        out("<?php $a=[5=>10,2=>20]; array_pop($a); echo implode(',',array_keys($a));"),
        "5"
    );
}

// --- step 17-1: string case ---

#[test]
fn strtoupper_strtolower_ascii_only() {
    assert_eq!(out("<?php echo strtoupper('Hello, World! 123');"), "HELLO, WORLD! 123");
    assert_eq!(out("<?php echo strtolower('Hello, WORLD!');"), "hello, world!");
    // Bytes >= 0x80 are left untouched (ASCII-only, C locale).
    assert_eq!(out("<?php var_dump(strtoupper('héllo'));"), "string(6) \"HéLLO\"\n");
    assert_eq!(out("<?php echo strtoupper('');"), "");
}

#[test]
fn ucfirst_lcfirst_ucwords() {
    assert_eq!(out("<?php echo ucfirst('hello');"), "Hello");
    assert_eq!(out("<?php var_dump(ucfirst(''));"), "string(0) \"\"\n");
    assert_eq!(out("<?php echo lcfirst('Hello');"), "hello");
    assert_eq!(out("<?php echo ucwords('hello world foo');"), "Hello World Foo");
    assert_eq!(out("<?php echo ucwords(\"a\tb\nc\");"), "A\tB\nC");
}

// --- step 17-2: string build (repeat/pad/chr/ord) ---

#[test]
fn str_repeat_repeats() {
    assert_eq!(out("<?php echo str_repeat('ab', 3);"), "ababab");
    assert_eq!(out("<?php var_dump(str_repeat('x', 0));"), "string(0) \"\"\n");
    assert_eq!(out("<?php echo str_repeat('-', 5);"), "-----");
}

#[test]
fn str_repeat_negative_is_value_error() {
    match fatal("<?php str_repeat('ab', -1);") {
        PhpError::ValueError(m) => assert_eq!(
            m,
            "str_repeat(): Argument #2 ($times) must be greater than or equal to 0"
        ),
        other => panic!("expected ValueError, got {other:?}"),
    }
}

#[test]
fn str_pad_types() {
    // Named constants aren't lowered yet, so use the literal values:
    // STR_PAD_LEFT=0, STR_PAD_RIGHT=1 (default), STR_PAD_BOTH=2.
    // Default: pad on the right; length <= strlen returns input unchanged.
    assert_eq!(out("<?php var_dump(str_pad('x', 1));"), "string(1) \"x\"\n");
    assert_eq!(out("<?php echo str_pad('5', 4, '0', 0);"), "0005");
    assert_eq!(out("<?php echo str_pad('5', 6, '-=', 2);"), "-=5-=-");
    assert_eq!(out("<?php echo str_pad('5', 4);"), "5   ");
}

#[test]
fn str_pad_empty_pad_is_value_error() {
    match fatal("<?php str_pad('x', 5, '');") {
        PhpError::ValueError(m) => assert_eq!(
            m,
            "str_pad(): Argument #3 ($pad_string) must not be empty"
        ),
        other => panic!("expected ValueError, got {other:?}"),
    }
}

#[test]
fn chr_and_ord() {
    assert_eq!(out("<?php echo chr(65);"), "A");
    assert_eq!(out("<?php var_dump(chr(321));"), "string(1) \"A\"\n"); // 321 % 256 == 65
    assert_eq!(out("<?php var_dump(ord('A'));"), "int(65)\n");
    assert_eq!(out("<?php var_dump(ord('AB'));"), "int(65)\n"); // first byte
}

// --- step 17-3: trim / ltrim / rtrim ---

#[test]
fn trim_default_whitespace() {
    assert_eq!(out("<?php var_dump(trim(\"  hi \n\"));"), "string(2) \"hi\"\n");
    assert_eq!(out("<?php var_dump(ltrim('  hi  '));"), "string(4) \"hi  \"\n");
    assert_eq!(out("<?php var_dump(rtrim('  hi  '));"), "string(4) \"  hi\"\n");
    assert_eq!(out("<?php var_dump(trim(\"\\t\\n x \\0\"));"), "string(1) \"x\"\n");
}

#[test]
fn trim_custom_charlist() {
    assert_eq!(out("<?php echo trim('xxhixx', 'x');"), "hi");
    assert_eq!(out("<?php echo trim('[hi]', '[]');"), "hi");
    // A range expands to the inclusive byte interval (PHP feature, not literal).
    assert_eq!(out("<?php echo trim('a1b2c', 'a..c');"), "1b2");
}

// --- step 17-4: math (intdiv/pow/sqrt/floor/ceil/round) ---

#[test]
fn intdiv_truncates_toward_zero() {
    assert_eq!(out("<?php var_dump(intdiv(7, 2));"), "int(3)\n");
    assert_eq!(out("<?php var_dump(intdiv(-7, 2));"), "int(-3)\n");
    assert_eq!(out("<?php var_dump(intdiv(7, -2));"), "int(-3)\n");
}

#[test]
fn intdiv_by_zero_is_division_error() {
    match fatal("<?php intdiv(1, 0);") {
        PhpError::DivisionByZeroError(m) => assert_eq!(m, "Division by zero"),
        other => panic!("expected DivisionByZeroError, got {other:?}"),
    }
}

#[test]
fn intdiv_min_by_neg_one_is_arithmetic_error() {
    // PHP_INT_MIN isn't a lowered constant yet; build it as i64::MIN.
    match fatal("<?php intdiv(-9223372036854775807 - 1, -1);") {
        PhpError::ArithmeticError(m) => {
            assert_eq!(m, "Division of PHP_INT_MIN by -1 is not an integer")
        }
        other => panic!("expected ArithmeticError, got {other:?}"),
    }
}

#[test]
fn pow_int_and_float() {
    assert_eq!(out("<?php var_dump(pow(2, 3));"), "int(8)\n");
    assert_eq!(out("<?php var_dump(pow(2, 10));"), "int(1024)\n");
    assert_eq!(out("<?php var_dump(pow(2, -1));"), "float(0.5)\n"); // negative exp -> float
    assert_eq!(out("<?php var_dump(pow(2, 0.5));"), "float(1.4142135623730951)\n");
    assert_eq!(out("<?php var_dump(pow(-2, 3));"), "int(-8)\n");
}

#[test]
fn sqrt_floor_ceil_round() {
    assert_eq!(out("<?php var_dump(sqrt(16));"), "float(4)\n");
    assert_eq!(out("<?php var_dump(floor(4.7));"), "float(4)\n");
    assert_eq!(out("<?php var_dump(ceil(4.2));"), "float(5)\n");
    assert_eq!(out("<?php var_dump(round(3.14159, 2));"), "float(3.14)\n");
    assert_eq!(out("<?php var_dump(round(2.5));"), "float(3)\n"); // half away from zero
    assert_eq!(out("<?php var_dump(round(1234.5, -2));"), "float(1200)\n");
}

// --- step 17-5: array (range/slice/reverse/unique/sum) ---

#[test]
fn range_int_char_and_step() {
    assert_eq!(out("<?php echo implode(',', range(1, 5));"), "1,2,3,4,5");
    assert_eq!(out("<?php echo implode(',', range(5, 1));"), "5,4,3,2,1"); // auto descending
    assert_eq!(out("<?php echo implode(',', range(5, 1, 2));"), "5,3,1");
    assert_eq!(out("<?php echo implode(',', range('a', 'e'));"), "a,b,c,d,e");
    // Float step makes a float range.
    assert_eq!(out("<?php var_dump(range(0, 1, 0.5));"), "array(3) {\n  [0]=>\n  float(0)\n  [1]=>\n  float(0.5)\n  [2]=>\n  float(1)\n}\n");
}

#[test]
fn range_step_errors() {
    match fatal("<?php range(1, 5, 0);") {
        PhpError::ValueError(m) => assert_eq!(m, "range(): Argument #3 ($step) cannot be 0"),
        other => panic!("expected ValueError, got {other:?}"),
    }
    match fatal("<?php range(1, 5, -1);") {
        PhpError::ValueError(m) => assert_eq!(
            m,
            "range(): Argument #3 ($step) must be greater than 0 for increasing ranges"
        ),
        other => panic!("expected ValueError, got {other:?}"),
    }
}

#[test]
fn array_slice_offset_length_preserve() {
    assert_eq!(out("<?php echo implode(',', array_slice([10,20,30,40], 1, 2));"), "20,30");
    // Negative offset from the end; default reindexes int keys.
    assert_eq!(out("<?php echo implode(',', array_slice([10,20,30,40], -2));"), "30,40");
    // preserve_keys keeps original int keys.
    assert_eq!(
        out("<?php var_dump(array_slice([10,20,30], 1, null, true));"),
        "array(2) {\n  [1]=>\n  int(20)\n  [2]=>\n  int(30)\n}\n"
    );
}

#[test]
fn array_reverse_and_preserve() {
    assert_eq!(out("<?php echo implode(',', array_reverse([1,2,3]));"), "3,2,1");
    assert_eq!(
        out("<?php var_dump(array_reverse(['a'=>1, 2, 3], true));"),
        "array(3) {\n  [1]=>\n  int(3)\n  [0]=>\n  int(2)\n  [\"a\"]=>\n  int(1)\n}\n"
    );
}

#[test]
fn array_unique_keeps_first_and_keys() {
    // SORT_STRING: 1 and "1" compare equal, first kept; keys preserved.
    assert_eq!(
        out("<?php var_dump(array_unique([1, '1', 2, 2, 3]));"),
        "array(3) {\n  [0]=>\n  int(1)\n  [2]=>\n  int(2)\n  [4]=>\n  int(3)\n}\n"
    );
}

#[test]
fn array_sum_int_and_float() {
    assert_eq!(out("<?php var_dump(array_sum([1, 2, 3]));"), "int(6)\n");
    assert_eq!(out("<?php var_dump(array_sum([1.5, 2]));"), "float(3.5)\n");
    assert_eq!(out("<?php var_dump(array_sum([]));"), "int(0)\n");
    assert_eq!(out("<?php var_dump(array_sum(['1', '2', 3]));"), "int(6)\n");
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

// --- step 18-1: closure gettype (needs the real registry) ---

#[test]
fn closure_gettype_is_object() {
    assert_eq!(out("<?php $f = function(){}; echo gettype($f);"), "object");
    assert_eq!(out("<?php $f = function($x){ return $x; }; echo gettype($f);"), "object");
}

// --- step 18-3: string callables / call_user_func / is_callable on builtins ---

#[test]
fn string_callable_to_builtin() {
    assert_eq!(out("<?php $f = 'strlen'; echo $f('hello');"), "5");
}

#[test]
fn call_user_func_to_builtin() {
    assert_eq!(out("<?php echo call_user_func('strlen', 'abcd');"), "4");
}

#[test]
fn is_callable_builtin_name() {
    assert_eq!(out("<?php echo is_callable('strlen') ? 'y' : 'n';"), "y");
    assert_eq!(out("<?php echo is_callable('definitely_not_a_fn') ? 'y' : 'n';"), "n");
}

// --- step 18-4: engine constants make flag-taking builtins ergonomic ---

#[test]
fn str_pad_with_named_flag_constant() {
    // STR_PAD_LEFT now resolves, so the natural call works (step 17 unlock).
    assert_eq!(out("<?php echo str_pad('5', 3, '0', STR_PAD_LEFT);"), "005");
    assert_eq!(out("<?php echo str_pad('5', 3, '0', STR_PAD_BOTH);"), "050");
}

// --- step 18-5: higher-order builtins with string-builtin callables ---

#[test]
fn array_map_with_builtin_string_callable() {
    assert_eq!(
        out("<?php echo implode(',', array_map('strtoupper', ['a', 'b', 'c']));"),
        "A,B,C"
    );
}

#[test]
fn array_filter_with_builtin_string_callable() {
    // Keep only non-empty strings via strlen as the predicate.
    assert_eq!(
        out("<?php $r = array_filter(['', 'x', '', 'yz'], 'strlen'); echo $r[1], $r[3];"),
        "xyz"
    );
}

// --- step 18-6: first-class callable of a builtin ---

#[test]
fn first_class_callable_builtin() {
    assert_eq!(out("<?php $f = strlen(...); echo $f('hello');"), "5");
    assert_eq!(out("<?php echo gettype(strlen(...));"), "object");
}

#[test]
fn first_class_callable_in_array_map() {
    assert_eq!(
        out("<?php echo implode(',', array_map(strtoupper(...), ['a', 'b', 'c']));"),
        "A,B,C"
    );
}

// --- step 18-7: exact closure var_dump / print_r format ---

#[test]
fn var_dump_closure_no_params() {
    assert_eq!(
        out("<?php $f = function(){}; var_dump($f);"),
        "object(Closure)#1 (3) {\n  \
           [\"name\"]=>\n  string(17) \"{closure:t.php:1}\"\n  \
           [\"file\"]=>\n  string(5) \"t.php\"\n  \
           [\"line\"]=>\n  int(1)\n}\n"
    );
}

#[test]
fn var_dump_closure_with_params() {
    assert_eq!(
        out("<?php $f = function($x, $y = 1){ return $x; }; var_dump($f);"),
        "object(Closure)#1 (4) {\n  \
           [\"name\"]=>\n  string(17) \"{closure:t.php:1}\"\n  \
           [\"file\"]=>\n  string(5) \"t.php\"\n  \
           [\"line\"]=>\n  int(1)\n  \
           [\"parameter\"]=>\n  array(2) {\n    \
             [\"$x\"]=>\n    string(10) \"<required>\"\n    \
             [\"$y\"]=>\n    string(10) \"<optional>\"\n  }\n}\n"
    );
}

#[test]
fn var_dump_first_class_callable() {
    assert_eq!(
        out("<?php function dbl($x){ return $x * 2; } $f = dbl(...); var_dump($f);"),
        "object(Closure)#1 (2) {\n  \
           [\"function\"]=>\n  string(3) \"dbl\"\n  \
           [\"parameter\"]=>\n  array(1) {\n    \
             [\"$x\"]=>\n    string(10) \"<required>\"\n  }\n}\n"
    );
}

#[test]
fn var_dump_object_ids_increment_for_live_closures() {
    let s = out("<?php $a = function(){}; $b = function(){}; var_dump($a); var_dump($b);");
    assert!(s.contains("object(Closure)#1 "), "{s}");
    assert!(s.contains("object(Closure)#2 "), "{s}");
}

#[test]
fn print_r_closure_with_params() {
    assert_eq!(
        out("<?php $f = function($x){}; print_r($f);"),
        "Closure Object\n(\n    \
           [name] => {closure:t.php:1}\n    \
           [file] => t.php\n    \
           [line] => 1\n    \
           [parameter] => Array\n        (\n            \
             [$x] => <required>\n        )\n\n)\n"
    );
}
