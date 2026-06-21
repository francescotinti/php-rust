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

/// Like [`out`] but returns the raw stdout bytes (for non-UTF-8 output such as
/// transcoded strings).
fn out_bytes(src: &str) -> Vec<u8> {
    let reg = registry();
    let o = run_source_with(b"t.php", src.as_bytes(), &reg).expect("lowers");
    assert!(o.fatal.is_none(), "unexpected fatal: {:?}", o.fatal);
    o.stdout
}

/// Run a script and return `(stdout, raw diagnostic messages)` — for asserting
/// the exact text of Warnings a builtin raises (step 52 filesystem mutators).
fn out_diags(src: &str) -> (String, Vec<String>) {
    let reg = registry();
    let o = run_source_with(b"t.php", src.as_bytes(), &reg).expect("lowers");
    assert!(o.fatal.is_none(), "unexpected fatal: {:?}", o.fatal);
    let warns = o.diags.iter().map(|d| d.message().to_string()).collect();
    (String::from_utf8(o.stdout).expect("utf8"), warns)
}

/// Run a script and return `(stdout, exit_code)` — for `exit`/`die` tests where
/// the process exit code matters (step 46).
fn out_exit(src: &str) -> (String, Option<u8>) {
    let reg = registry();
    let o = run_source_with(b"t.php", src.as_bytes(), &reg).expect("lowers");
    assert!(o.fatal.is_none(), "unexpected fatal: {:?}", o.fatal);
    (String::from_utf8(o.stdout).expect("utf8"), o.exit_code)
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
fn var_dump_object_visibility() {
    // Step 19-7: protected and private annotations (private shows declaring class).
    assert_eq!(
        out("<?php class A { public $a = 1; protected $b = 2; private $c = 3; } var_dump(new A);"),
        "object(A)#1 (3) {\n  [\"a\"]=>\n  int(1)\n  [\"b\":protected]=>\n  int(2)\n  [\"c\":\"A\":private]=>\n  int(3)\n}\n"
    );
}

#[test]
fn var_dump_inherited_private_declaring_class() {
    // A private property inherited from A is annotated with A as declaring class.
    assert_eq!(
        out("<?php class A { private $x = 1; } class B extends A { public $y = 2; } var_dump(new B);"),
        "object(B)#1 (2) {\n  [\"x\":\"A\":private]=>\n  int(1)\n  [\"y\"]=>\n  int(2)\n}\n"
    );
}

#[test]
fn var_dump_object_recursion() {
    assert_eq!(
        out("<?php class A { public $self; } $a = new A; $a->self = $a; var_dump($a);"),
        "object(A)#1 (1) {\n  [\"self\"]=>\n  *RECURSION*\n}\n"
    );
}

#[test]
fn print_r_object_visibility() {
    assert_eq!(
        out("<?php class A { public $a = 1; protected $b = 2; private $c = 3; } print_r(new A);"),
        "A Object\n(\n    [a] => 1\n    [b:protected] => 2\n    [c:A:private] => 3\n)\n"
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

// --- step 21-5: var_dump / print_r of trait-flattened properties ---

#[test]
fn var_dump_trait_props_after_own() {
    // PHP lists the class's own properties first, then trait-supplied ones, with
    // the trait property's visibility annotation preserved.
    assert_eq!(
        out("<?php trait T { public $a = 1; protected $b = 2; public static $s = 9; } \
             class C { use T; public $c = 3; } var_dump(new C());"),
        "object(C)#1 (3) {\n  [\"c\"]=>\n  int(3)\n  [\"a\"]=>\n  int(1)\n  \
         [\"b\":protected]=>\n  int(2)\n}\n"
    );
}

#[test]
fn var_dump_nested_trait_prop_order() {
    // Own first, then this trait's own, then the nested trait's: c, b, a.
    assert_eq!(
        out("<?php trait A { public $a = 1; } trait B { use A; public $b = 2; } \
             class C { use B; public $c = 3; } var_dump(new C());"),
        "object(C)#1 (3) {\n  [\"c\"]=>\n  int(3)\n  [\"b\"]=>\n  int(2)\n  \
         [\"a\"]=>\n  int(1)\n}\n"
    );
}

#[test]
fn print_r_trait_props() {
    assert_eq!(
        out("<?php trait T { public $a = 1; } class C { use T; public $c = 3; } print_r(new C());"),
        "C Object\n(\n    [c] => 3\n    [a] => 1\n)\n"
    );
}

// --- step 23: enum var_dump / print_r ---

#[test]
fn var_dump_enum_pure() {
    assert_eq!(
        out("<?php enum Suit { case Hearts; } var_dump(Suit::Hearts);"),
        "enum(Suit::Hearts)\n"
    );
}

#[test]
fn var_dump_enum_backed_hides_value() {
    assert_eq!(
        out("<?php enum Status: string { case Active = 'A'; } var_dump(Status::Active);"),
        "enum(Status::Active)\n"
    );
    assert_eq!(
        out("<?php enum Size: int { case Big = 9; } var_dump(Size::Big);"),
        "enum(Size::Big)\n"
    );
}

#[test]
fn var_dump_enum_in_array() {
    assert_eq!(
        out("<?php enum Suit { case Hearts; case Spades; } var_dump([Suit::Hearts, Suit::Spades]);"),
        "array(2) {\n  [0]=>\n  enum(Suit::Hearts)\n  [1]=>\n  enum(Suit::Spades)\n}\n"
    );
}

#[test]
fn print_r_enum_pure() {
    assert_eq!(
        out("<?php enum Suit { case Hearts; } print_r(Suit::Hearts);"),
        "Suit Enum\n(\n    [name] => Hearts\n)\n"
    );
}

#[test]
fn print_r_enum_backed() {
    assert_eq!(
        out("<?php enum Status: string { case Active = 'A'; } print_r(Status::Active);"),
        "Status Enum:string\n(\n    [name] => Active\n    [value] => A\n)\n"
    );
    assert_eq!(
        out("<?php enum Size: int { case Big = 9; } print_r(Size::Big);"),
        "Size Enum:int\n(\n    [name] => Big\n    [value] => 9\n)\n"
    );
}

// --- Step 26: json_encode / json_decode ---

#[test]
fn json_encode_scalars() {
    assert_eq!(out("<?php echo json_encode(42);"), "42");
    assert_eq!(out("<?php echo json_encode(true), json_encode(false), json_encode(null);"), "truefalsenull");
    assert_eq!(out("<?php echo json_encode(1.5), '|', json_encode(1.0), '|', json_encode(0.1);"), "1.5|1|0.1");
}

#[test]
fn json_encode_string_escaping() {
    assert_eq!(out("<?php echo json_encode(\"a\\\"b/c\\n\");"), "\"a\\\"b\\/c\\n\"");
    assert_eq!(out("<?php echo json_encode('a/b', JSON_UNESCAPED_SLASHES);"), "\"a/b\"");
}

#[test]
fn json_encode_unicode() {
    // é = U+00E9 -> é by default, raw with JSON_UNESCAPED_UNICODE.
    assert_eq!(out("<?php echo json_encode(\"é\");"), "\"\\u00e9\"");
    assert_eq!(out("<?php echo json_encode(\"é\", JSON_UNESCAPED_UNICODE);"), "\"é\"");
}

#[test]
fn json_encode_arrays() {
    assert_eq!(out("<?php echo json_encode([1,2,3]);"), "[1,2,3]");
    assert_eq!(out("<?php echo json_encode([]);"), "[]");
    assert_eq!(out("<?php echo json_encode(['a'=>1,'b'=>[2,3]]);"), "{\"a\":1,\"b\":[2,3]}");
    assert_eq!(out("<?php echo json_encode([0=>'x',2=>'y']);"), "{\"0\":\"x\",\"2\":\"y\"}");
}

#[test]
fn json_encode_object_public_props() {
    assert_eq!(
        out("<?php class C { public $x=1; public $y='z'; private $h=9; } echo json_encode(new C);"),
        "{\"x\":1,\"y\":\"z\"}"
    );
}

#[test]
fn json_encode_pretty_print() {
    assert_eq!(
        out("<?php echo json_encode(['x'=>1], JSON_PRETTY_PRINT);"),
        "{\n    \"x\": 1\n}"
    );
}

#[test]
fn json_decode_assoc_arrays() {
    assert_eq!(out("<?php $v=json_decode('{\"a\":1,\"b\":[2,3]}', true); echo $v['a'], '|', $v['b'][1];"), "1|3");
    assert_eq!(out("<?php $v=json_decode('[1,\"x\",true,null]', true); echo $v[0], $v[1], ($v[2]?'T':'F'), gettype($v[3]);"), "1xTNULL");
}

#[test]
fn json_decode_scalars_and_errors() {
    assert_eq!(out("<?php $v=json_decode('\"hi\"'); echo $v, '|', gettype($v);"), "hi|string");
    assert_eq!(out("<?php $v=json_decode('3.14'); echo $v, '|', gettype($v);"), "3.14|double");
    assert_eq!(out("<?php var_dump(json_decode('null'));"), "NULL\n");
    assert_eq!(out("<?php var_dump(json_decode('not json'));"), "NULL\n");
}

#[test]
fn json_decode_default_stdclass() {
    assert_eq!(
        out("<?php $o=json_decode('{\"a\":1,\"b\":\"z\"}'); echo get_class($o), '|', $o->a, $o->b;"),
        "stdClass|1z"
    );
}

#[test]
fn json_encode_pretty_nested() {
    // Matches PHP 8.5.7 byte-for-byte (4-space indent per depth).
    assert_eq!(
        out("<?php echo json_encode([1,2,['a'=>3]], JSON_PRETTY_PRINT);"),
        "[\n    1,\n    2,\n    {\n        \"a\": 3\n    }\n]"
    );
}

// --- Step 29-1: pure string builtins ---------------------------------------

#[test]
fn strrev_reverses_bytes() {
    assert_eq!(out("<?php echo strrev('Hello');"), "olleH");
    assert_eq!(out("<?php echo strrev('');"), "");
    assert_eq!(out("<?php echo strrev('a');"), "a");
}

#[test]
fn str_contains_cases() {
    assert_eq!(out("<?php var_dump(str_contains('abc', 'b'));"), "bool(true)\n");
    assert_eq!(out("<?php var_dump(str_contains('abc', 'x'));"), "bool(false)\n");
    // Empty needle is always found (PHP 8 semantics).
    assert_eq!(out("<?php var_dump(str_contains('abc', ''));"), "bool(true)\n");
}

#[test]
fn str_starts_and_ends_with_cases() {
    assert_eq!(out("<?php var_dump(str_starts_with('abc.php', 'abc'));"), "bool(true)\n");
    assert_eq!(out("<?php var_dump(str_starts_with('abc.php', 'php'));"), "bool(false)\n");
    assert_eq!(out("<?php var_dump(str_starts_with('abc', ''));"), "bool(true)\n");
    assert_eq!(out("<?php var_dump(str_ends_with('abc.php', '.php'));"), "bool(true)\n");
    assert_eq!(out("<?php var_dump(str_ends_with('abc.php', 'abc'));"), "bool(false)\n");
    assert_eq!(out("<?php var_dump(str_ends_with('abc', ''));"), "bool(true)\n");
}

#[test]
fn str_split_cases() {
    assert_eq!(
        out("<?php echo implode('|', str_split('abcde'));"),
        "a|b|c|d|e"
    );
    assert_eq!(
        out("<?php echo implode('|', str_split('abcde', 2));"),
        "ab|cd|e"
    );
    // chunk longer than the input yields a single element.
    assert_eq!(out("<?php echo implode('|', str_split('ab', 5));"), "ab");
    // Empty string yields an empty array (PHP 8.2+).
    assert_eq!(out("<?php var_dump(str_split(''));"), "array(0) {\n}\n");
}

#[test]
fn str_split_non_positive_length_is_value_error() {
    match fatal("<?php str_split('ab', 0);") {
        PhpError::ValueError(m) => {
            assert_eq!(m, "str_split(): Argument #2 ($length) must be greater than 0");
        }
        other => panic!("expected ValueError, got {other:?}"),
    }
}

#[test]
fn substr_count_cases() {
    assert_eq!(out("<?php echo substr_count('hello world hello', 'hello');"), "2");
    // Non-overlapping match.
    assert_eq!(out("<?php echo substr_count('aaa', 'aa');"), "1");
    assert_eq!(out("<?php echo substr_count('abc', 'x');"), "0");
}

#[test]
fn number_format_cases() {
    assert_eq!(out("<?php echo number_format(1234567.891);"), "1,234,568");
    assert_eq!(out("<?php echo number_format(1234567.891, 2);"), "1,234,567.89");
    assert_eq!(out("<?php echo number_format(1234567.891, 2, ',', '.');"), "1.234.567,89");
    assert_eq!(out("<?php echo number_format(-1234.5678, 2);"), "-1,234.57");
    assert_eq!(out("<?php echo number_format(1000, 2);"), "1,000.00");
    assert_eq!(out("<?php echo number_format(0, 2);"), "0.00");
    // Round half away from zero.
    assert_eq!(out("<?php echo number_format(2.5);"), "3");
    assert_eq!(out("<?php echo number_format(0.5);"), "1");
    // Rounded-to-zero result drops the negative sign.
    assert_eq!(out("<?php echo number_format(-0.01, 0);"), "0");
    assert_eq!(out("<?php echo number_format(-0.5, 0);"), "-1");
}

// --- Step 29-2: pure array builtins ----------------------------------------

#[test]
fn array_key_exists_cases() {
    assert_eq!(out("<?php var_dump(array_key_exists(0, [1, 2]));"), "bool(true)\n");
    assert_eq!(out("<?php var_dump(array_key_exists(5, [1, 2]));"), "bool(false)\n");
    // Unlike isset(), a null value still counts as an existing key.
    assert_eq!(out("<?php var_dump(array_key_exists('a', ['a' => null]));"), "bool(true)\n");
    assert_eq!(out("<?php var_dump(array_key_exists('a', ['b' => 1]));"), "bool(false)\n");
}

#[test]
fn array_search_cases() {
    // Loose by default: "1" matches int 1.
    assert_eq!(out("<?php var_dump(array_search('1', [0, 1, 2]));"), "int(1)\n");
    // Strict mode rejects the cross-type match.
    assert_eq!(out("<?php var_dump(array_search('1', [0, 1, 2], true));"), "bool(false)\n");
    assert_eq!(out("<?php var_dump(array_search('z', [0, 1, 2]));"), "bool(false)\n");
    // String keys come back as strings.
    assert_eq!(out("<?php var_dump(array_search(2, ['a' => 1, 'b' => 2]));"), "string(1) \"b\"\n");
}

#[test]
fn array_fill_cases() {
    assert_eq!(out("<?php echo implode(',', array_fill(5, 3, 'x'));"), "x,x,x");
    assert_eq!(
        out("<?php echo implode(',', array_keys(array_fill(5, 3, 'x')));"),
        "5,6,7"
    );
    // PHP 8: a negative start index increments consecutively.
    assert_eq!(
        out("<?php echo implode(',', array_keys(array_fill(-3, 3, 0)));"),
        "-3,-2,-1"
    );
}

#[test]
fn array_flip_cases() {
    assert_eq!(
        out("<?php $r = array_flip(['a' => 1, 'b' => 2]); echo $r[1], $r[2];"),
        "ab"
    );
    // Int values stay int keys; the new value is the old key.
    assert_eq!(
        out("<?php $r = array_flip([0 => 'x', 1 => 5]); echo $r['x'], '|', $r[5];"),
        "0|1"
    );
}

#[test]
fn array_combine_cases() {
    assert_eq!(
        out("<?php $r = array_combine(['a', 'b'], [1, 2]); echo $r['a'], $r['b'];"),
        "12"
    );
    assert_eq!(out("<?php var_dump(array_combine([], []));"), "array(0) {\n}\n");
    match fatal("<?php array_combine(['a'], [1, 2]);") {
        PhpError::ValueError(m) => assert!(m.contains("same number of elements"), "{m}"),
        other => panic!("expected ValueError, got {other:?}"),
    }
}

#[test]
fn array_pad_cases() {
    assert_eq!(out("<?php echo implode(',', array_pad([1, 2], 4, 0));"), "1,2,0,0");
    assert_eq!(out("<?php echo implode(',', array_pad([1, 2], -4, 0));"), "0,0,1,2");
    // Size <= length returns the input unchanged.
    assert_eq!(out("<?php echo implode(',', array_pad([1, 2], 1, 0));"), "1,2");
}

#[test]
fn array_product_cases() {
    assert_eq!(out("<?php echo array_product([2, 3, 4]);"), "24");
    assert_eq!(out("<?php echo array_product([]);"), "1");
    assert_eq!(out("<?php echo array_product([2, 2.5]);"), "5");
    assert_eq!(out("<?php echo array_product(['2', '3']);"), "6");
}

#[test]
fn array_key_first_last_cases() {
    assert_eq!(out("<?php var_dump(array_key_first(['x' => 1, 'y' => 2]));"), "string(1) \"x\"\n");
    assert_eq!(out("<?php var_dump(array_key_last([5, 6, 7]));"), "int(2)\n");
    assert_eq!(out("<?php var_dump(array_key_first([]));"), "NULL\n");
    assert_eq!(out("<?php var_dump(array_key_last([]));"), "NULL\n");
}

#[test]
fn array_diff_cases() {
    // Keys are preserved; comparison is by string value.
    assert_eq!(
        out("<?php echo implode(',', array_diff([1, 2, 3, 4], [2, 4]));"),
        "1,3"
    );
    assert_eq!(
        out("<?php echo implode(',', array_keys(array_diff([1, 2, 3, 4], [2, 4])));"),
        "0,2"
    );
    // String "2" removes int 2 (string comparison).
    assert_eq!(out("<?php echo implode(',', array_diff([1, 2, 3], ['2']));"), "1,3");
    // Multiple exclusion arrays.
    assert_eq!(out("<?php echo implode(',', array_diff([1, 2, 3, 4], [2], [4]));"), "1,3");
}

#[test]
fn array_intersect_cases() {
    assert_eq!(
        out("<?php echo implode(',', array_intersect([1, 2, 3, 4], [2, 4, 6]));"),
        "2,4"
    );
    assert_eq!(
        out("<?php echo implode(',', array_keys(array_intersect([1, 2, 3, 4], [2, 4, 6])));"),
        "1,3"
    );
}

// --- Step 29-3: (object) cast ----------------------------------------------

#[test]
fn object_cast_from_array() {
    assert_eq!(out("<?php $o = (object)['a' => 1, 'b' => 2]; echo $o->a, $o->b;"), "12");
    assert_eq!(out("<?php echo get_class((object)['a' => 1]);"), "stdClass");
}

#[test]
fn object_cast_numeric_keys_var_dump() {
    // Numeric array keys become string property names.
    assert_eq!(
        out("<?php var_dump((object)[1 => 'x', 2 => 'y']);"),
        "object(stdClass)#1 (2) {\n  [\"1\"]=>\n  string(1) \"x\"\n  [\"2\"]=>\n  string(1) \"y\"\n}\n"
    );
}

#[test]
fn object_cast_scalar_and_null() {
    // A scalar becomes a single "scalar" property.
    assert_eq!(out("<?php $o = (object)42; echo $o->scalar;"), "42");
    assert_eq!(out("<?php $o = (object)'hi'; echo $o->scalar;"), "hi");
    // null becomes an empty stdClass.
    assert_eq!(out("<?php echo get_class((object)null);"), "stdClass");
    assert_eq!(out("<?php var_dump((object)null);"), "object(stdClass)#1 (0) {\n}\n");
}

#[test]
fn object_cast_object_is_identity() {
    assert_eq!(
        out("<?php $a = new stdClass; $a->x = 5; $b = (object)$a; var_dump($a === $b);"),
        "bool(true)\n"
    );
}

// --- Step 31: preg named groups + PREG_* flags -----------------------------

#[test]
fn preg_match_named_groups() {
    // Named groups appear as the name key followed by the numeric index.
    assert_eq!(
        out("<?php preg_match('/(?<y>\\d{4})-(?<m>\\d{2})/', '2026-06', $mm); var_dump($mm);"),
        "array(5) {\n  [0]=>\n  string(7) \"2026-06\"\n  [\"y\"]=>\n  string(4) \"2026\"\n  [1]=>\n  string(4) \"2026\"\n  [\"m\"]=>\n  string(2) \"06\"\n  [2]=>\n  string(2) \"06\"\n}\n"
    );
}

#[test]
fn preg_match_offset_capture() {
    assert_eq!(
        out("<?php preg_match('/(\\d+)/', 'ab123cd', $mm, PREG_OFFSET_CAPTURE); echo $mm[0][0], '@', $mm[0][1];"),
        "123@2"
    );
}

#[test]
fn preg_match_trailing_unmatched_omitted() {
    // An unmatched trailing group is dropped from $matches by default.
    assert_eq!(
        out("<?php preg_match('/(a)(b)?/', 'a', $mm); echo count($mm), '|', isset($mm[2]) ? 'Y' : 'N';"),
        "2|N"
    );
}

#[test]
fn preg_match_unmatched_as_null() {
    assert_eq!(
        out("<?php preg_match('/(a)(b)?/', 'a', $mm, PREG_UNMATCHED_AS_NULL); var_dump($mm[2]);"),
        "NULL\n"
    );
}

#[test]
fn preg_match_all_set_order() {
    assert_eq!(
        out("<?php preg_match_all('/(\\w)(\\d)/', 'a1b2', $mm, PREG_SET_ORDER); echo $mm[0][0], $mm[0][1], $mm[0][2], '/', $mm[1][0], $mm[1][1], $mm[1][2];"),
        "a1a1/b2b2"
    );
}

#[test]
fn preg_split_no_empty() {
    assert_eq!(
        out("<?php $r = preg_split('/,/', 'a,,b', -1, PREG_SPLIT_NO_EMPTY); echo count($r), ':', $r[0], $r[1];"),
        "2:ab"
    );
}

#[test]
fn preg_split_delim_capture() {
    assert_eq!(
        out("<?php $r = preg_split('/(,)/', 'a,b', -1, PREG_SPLIT_DELIM_CAPTURE); echo implode('|', $r);"),
        "a|,|b"
    );
}

// --- Step 32: array by-ref family (array_splice / array_walk) ---------------

#[test]
fn array_splice_remove_and_replace() {
    assert_eq!(
        out("<?php $a=[1,2,3,4,5]; $r=array_splice($a,1,2,['x','y']); echo implode(',',$a),'/',implode(',',$r);"),
        "1,x,y,4,5/2,3"
    );
}

#[test]
fn array_splice_negative_offset() {
    assert_eq!(out("<?php $a=[1,2,3,4]; array_splice($a,-2); echo implode(',',$a);"), "1,2");
}

#[test]
fn array_splice_insert_length_zero() {
    assert_eq!(out("<?php $a=[1,2,3]; array_splice($a,1,0,['x']); echo implode(',',$a);"), "1,x,2,3");
}

#[test]
fn array_splice_string_keys_preserved() {
    // Kept string keys are preserved; replacement/int keys are renumbered.
    assert_eq!(
        out("<?php $a=['a'=>1,'b'=>2,'c'=>3]; array_splice($a,1,1,['z']); var_dump($a);"),
        "array(3) {\n  [\"a\"]=>\n  int(1)\n  [0]=>\n  string(1) \"z\"\n  [\"c\"]=>\n  int(3)\n}\n"
    );
}

#[test]
fn array_splice_scalar_replacement() {
    // A non-array replacement is treated as a single element.
    assert_eq!(out("<?php $a=[1,2,3]; array_splice($a,1,1,'X'); echo implode(',',$a);"), "1,X,3");
}

#[test]
fn array_walk_by_ref_modifies() {
    assert_eq!(
        out("<?php $a=[1,2,3]; array_walk($a, function(&$v,$k){ $v=$v*10+$k; }); echo implode(',',$a);"),
        "10,21,32"
    );
}

#[test]
fn array_walk_by_value_no_change_with_extra() {
    assert_eq!(
        out("<?php $a=[1,2]; $out=''; array_walk($a, function($v,$k,$p) use (&$out){ $out.=\"$k:$v:$p \"; }, 'X'); echo $out, '/', implode(',',$a);"),
        "0:1:X 1:2:X /1,2"
    );
}

// --- Step 33: array key/assoc set-ops + array_column -----------------------

#[test]
fn array_diff_key_cases() {
    // Keep entries whose key is absent from every other array.
    assert_eq!(
        out("<?php $r=array_diff_key(['a'=>1,'b'=>2,'c'=>3],['b'=>9,'c'=>9]); echo implode(',',array_keys($r)),'/',implode(',',$r);"),
        "a/1"
    );
    assert_eq!(
        out("<?php echo implode(',',array_keys(array_diff_key(['a'=>1,'b'=>2,'c'=>3,'d'=>4],['a'=>0],['c'=>0])));"),
        "b,d"
    );
}

#[test]
fn array_diff_assoc_cases() {
    // Remove an entry only when some other array has the same key AND value.
    assert_eq!(
        out("<?php $r=array_diff_assoc(['a'=>1,'b'=>2,'c'=>3],['a'=>1,'b'=>9]); echo implode(',',array_keys($r)),'/',implode(',',$r);"),
        "b,c/2,3"
    );
    assert_eq!(
        out("<?php $r=array_diff_assoc([1,2,3],[1,9,3]); echo implode(',',array_keys($r)),'/',implode(',',$r);"),
        "1/2"
    );
}

#[test]
fn array_intersect_key_cases() {
    assert_eq!(
        out("<?php $r=array_intersect_key(['a'=>1,'b'=>2,'c'=>3],['a'=>9,'c'=>9]); echo implode(',',array_keys($r)),'/',implode(',',$r);"),
        "a,c/1,3"
    );
    assert_eq!(
        out("<?php echo implode(',',array_keys(array_intersect_key(['a'=>1,'b'=>2,'c'=>3],['a'=>0,'b'=>0],['b'=>0,'c'=>0])));"),
        "b"
    );
}

#[test]
fn array_intersect_assoc_cases() {
    assert_eq!(
        out("<?php $r=array_intersect_assoc(['a'=>1,'b'=>2],['a'=>1,'b'=>9]); echo implode(',',array_keys($r)),'/',implode(',',$r);"),
        "a/1"
    );
}

#[test]
fn array_column_cases() {
    assert_eq!(
        out("<?php $r=[['id'=>1,'name'=>'A'],['id'=>2,'name'=>'B']]; echo implode(',',array_column($r,'name'));"),
        "A,B"
    );
    // With index_key the result is keyed by that column.
    assert_eq!(
        out("<?php $r=[['id'=>5,'name'=>'A'],['id'=>6,'name'=>'B']]; $c=array_column($r,'name','id'); echo implode(',',array_keys($c)),'/',implode(',',$c);"),
        "5,6/A,B"
    );
    // A row missing the column is skipped.
    assert_eq!(
        out("<?php $r=[['id'=>1,'name'=>'A'],['id'=>2]]; echo implode(',',array_column($r,'name'));"),
        "A"
    );
    // A null column yields whole rows keyed by index_key.
    assert_eq!(
        out("<?php $r=[['id'=>5,'name'=>'A']]; $c=array_column($r,null,'id'); echo $c[5]['name'];"),
        "A"
    );
}

// --- Step 34-1: date() / gmdate() core formatting -----------------------------
// Oracle: ts 1718452845 = Sat 2024-06-15 12:00:45 UTC (date_default_timezone_set('UTC')).
// Edge-case timestamps below are oracle-computed (mktime is step 34-2).

#[test]
fn date_day_chars() {
    // d (zero-padded), j (no pad), D (3-letter), l (full), N (1=Mon..7=Sun),
    // w (0=Sun..6=Sat), S (ordinal suffix).
    assert_eq!(out("<?php echo date('d',1718452845);"), "15");
    assert_eq!(out("<?php echo date('j',1718452845);"), "15");
    assert_eq!(out("<?php echo date('D',1718452845);"), "Sat");
    assert_eq!(out("<?php echo date('l',1718452845);"), "Saturday");
    assert_eq!(out("<?php echo date('N',1718452845);"), "6");
    assert_eq!(out("<?php echo date('w',1718452845);"), "6");
    assert_eq!(out("<?php echo date('S',1718452845);"), "th");
    // single-digit day: no padding for j, padding for d (2024-06-05).
    assert_eq!(out("<?php echo date('d-j',1717545600);"), "05-5");
    // ordinal suffixes: 1st 2nd 3rd 11th 21st 23rd.
    assert_eq!(out("<?php echo date('jS',1717200000);"), "1st");
    assert_eq!(out("<?php echo date('jS',1717286400);"), "2nd");
    assert_eq!(out("<?php echo date('jS',1717372800);"), "3rd");
    assert_eq!(out("<?php echo date('jS',1718064000);"), "11th");
    assert_eq!(out("<?php echo date('jS',1718928000);"), "21st");
    assert_eq!(out("<?php echo date('jS',1719100800);"), "23rd");
}

#[test]
fn date_month_year_chars() {
    // F (full), M (3-letter), m (zero-padded), n (no pad), t (days in month),
    // L (leap), Y, y, o (ISO year).
    assert_eq!(out("<?php echo date('F',1718452845);"), "June");
    assert_eq!(out("<?php echo date('M',1718452845);"), "Jun");
    assert_eq!(out("<?php echo date('m',1718452845);"), "06");
    assert_eq!(out("<?php echo date('n',1718452845);"), "6");
    assert_eq!(out("<?php echo date('t',1718452845);"), "30");
    assert_eq!(out("<?php echo date('L',1718452845);"), "1");
    assert_eq!(out("<?php echo date('Y',1718452845);"), "2024");
    assert_eq!(out("<?php echo date('y',1718452845);"), "24");
    assert_eq!(out("<?php echo date('o',1718452845);"), "2024");
    // non-leap 2023: L=0, february 28 days; leap 2024: february 29 days.
    assert_eq!(out("<?php echo date('L',1672531200);"), "0");
    assert_eq!(out("<?php echo date('t',1675209600);"), "28");
    assert_eq!(out("<?php echo date('t',1706745600);"), "29");
}

#[test]
fn date_time_chars() {
    // a/A (am/pm), g/G/h/H (12/24 hour), i, s, u, v.
    assert_eq!(out("<?php echo date('a',1718452845);"), "pm");
    assert_eq!(out("<?php echo date('A',1718452845);"), "PM");
    assert_eq!(out("<?php echo date('g',1718452845);"), "12");
    assert_eq!(out("<?php echo date('G',1718452845);"), "12");
    assert_eq!(out("<?php echo date('h',1718452845);"), "12");
    assert_eq!(out("<?php echo date('H',1718452845);"), "12");
    assert_eq!(out("<?php echo date('i',1718452845);"), "00");
    assert_eq!(out("<?php echo date('s',1718452845);"), "45");
    assert_eq!(out("<?php echo date('u',1718452845);"), "000000");
    assert_eq!(out("<?php echo date('v',1718452845);"), "000");
    // midnight (2024-06-15 00:05:00): 12-hour shows 12, am.
    assert_eq!(out("<?php echo date('g:G:h:H a',1718409900);"), "12:0:12:00 am");
    // 09:00:00: AM, single-digit g/G, padded h/H.
    assert_eq!(out("<?php echo date('g:G:h:H A',1718442000);"), "9:9:09:09 AM");
    // 13:00:00: 12-hour 1, 24-hour 13.
    assert_eq!(out("<?php echo date('g:G:h:H',1718456400);"), "1:13:01:13");
}

#[test]
fn date_timezone_iso_chars() {
    // e/T (tz name), I (dst), O/P (offset), Z (offset seconds) — UTC only.
    assert_eq!(out("<?php echo date('e',1718452845);"), "UTC");
    assert_eq!(out("<?php echo date('T',1718452845);"), "UTC");
    assert_eq!(out("<?php echo date('I',1718452845);"), "0");
    assert_eq!(out("<?php echo date('O',1718452845);"), "+0000");
    assert_eq!(out("<?php echo date('P',1718452845);"), "+00:00");
    assert_eq!(out("<?php echo date('Z',1718452845);"), "0");
    assert_eq!(out("<?php echo date('B',1718452845);"), "542");
}

#[test]
fn date_composite_and_doy_week() {
    // c (ISO 8601), r (RFC 2822), U (timestamp), z (day of year 0-based), W (ISO week).
    assert_eq!(out("<?php echo date('c',1718452845);"), "2024-06-15T12:00:45+00:00");
    assert_eq!(out("<?php echo date('r',1718452845);"), "Sat, 15 Jun 2024 12:00:45 +0000");
    assert_eq!(out("<?php echo date('U',1718452845);"), "1718452845");
    assert_eq!(out("<?php echo date('z',1718452845);"), "166");
    assert_eq!(out("<?php echo date('W',1718452845);"), "24");
    // z edges (2024-01-01 and 2024-12-31).
    assert_eq!(out("<?php echo date('z',1704067200);"), "0");
    assert_eq!(out("<?php echo date('z',1735603200);"), "365");
    // ISO week/year edge: 2023-01-01 is ISO week 52 of 2022.
    assert_eq!(out("<?php echo date('W',1672531200);"), "52");
    assert_eq!(out("<?php echo date('o',1672531200);"), "2022");
}

#[test]
fn date_literals_and_escape() {
    // Non-format chars pass through literally; backslash escapes the next char.
    assert_eq!(out("<?php echo date('Y-m-d H:i:s',1718452845);"), "2024-06-15 12:00:45");
    assert_eq!(out("<?php echo date('\\\\Y=Y',1718452845);"), "Y=2024");
    // common combined format
    assert_eq!(out("<?php echo date('D, d M Y',1718452845);"), "Sat, 15 Jun 2024");
}

#[test]
fn gmdate_matches_date_in_utc() {
    // With the default UTC tz, gmdate == date.
    assert_eq!(out("<?php echo gmdate('Y-m-d H:i:s',1718452845);"), "2024-06-15 12:00:45");
    assert_eq!(out("<?php echo gmdate('r',1718452845);"), "Sat, 15 Jun 2024 12:00:45 +0000");
}

// --- Step 34-2: mktime / gmmktime / checkdate ---------------------------------
// Oracle: mktime(h,m,s,month,day,year), UTC. Values oracle-verified.

#[test]
fn mktime_basic() {
    assert_eq!(out("<?php echo mktime(0,0,0,6,15,2024);"), "1718409600");
    assert_eq!(out("<?php echo mktime(12,0,45,6,15,2024);"), "1718452845");
    // gmmktime == mktime in UTC.
    assert_eq!(out("<?php echo gmmktime(0,0,0,6,15,2024);"), "1718409600");
}

#[test]
fn mktime_month_overflow() {
    // month 13 → January next year; month 0 → December prev year; negative wraps.
    assert_eq!(out("<?php echo mktime(0,0,0,13,1,2024);"), "1735689600");
    assert_eq!(out("<?php echo mktime(0,0,0,0,1,2024);"), "1701388800");
    assert_eq!(out("<?php echo mktime(0,0,0,-1,1,2024);"), "1698796800");
}

#[test]
fn mktime_day_time_overflow() {
    // day 32 of January → Feb 1; day 0 of March → Feb 29 (leap).
    assert_eq!(out("<?php echo mktime(0,0,0,1,32,2024);"), "1706745600");
    assert_eq!(out("<?php echo mktime(0,0,0,3,0,2024);"), "1709164800");
    // hour 25 → +1 day +1h; second -1 → previous second.
    assert_eq!(out("<?php echo mktime(25,0,0,6,15,2024);"), "1718499600");
    assert_eq!(out("<?php echo mktime(0,0,-1,6,15,2024);"), "1718409599");
}

#[test]
fn mktime_two_digit_year() {
    // 0..69 → 2000..2069; 70..100 → 1970..2000.
    assert_eq!(out("<?php echo mktime(0,0,0,1,1,24);"), "1704067200");
    assert_eq!(out("<?php echo mktime(0,0,0,1,1,70);"), "0");
    assert_eq!(out("<?php echo mktime(0,0,0,1,1,99);"), "915148800");
    assert_eq!(out("<?php echo mktime(0,0,0,1,1,69);"), "3124224000");
}

#[test]
fn checkdate_validity() {
    assert_eq!(out("<?php var_dump(checkdate(2,29,2024));"), "bool(true)\n");
    assert_eq!(out("<?php var_dump(checkdate(2,29,2023));"), "bool(false)\n");
    assert_eq!(out("<?php var_dump(checkdate(13,1,2024));"), "bool(false)\n");
    assert_eq!(out("<?php var_dump(checkdate(0,1,2024));"), "bool(false)\n");
    assert_eq!(out("<?php var_dump(checkdate(12,31,2024));"), "bool(true)\n");
    assert_eq!(out("<?php var_dump(checkdate(4,31,2024));"), "bool(false)\n");
}

// --- Step 34-3: strtotime (subset) --------------------------------------------
// Oracle base: 1718452845 = 2024-06-15 12:00:45 UTC, passed as $baseTimestamp.

#[test]
fn strtotime_absolute() {
    assert_eq!(out("<?php echo strtotime('2024-06-15 12:00:00',1718452845);"), "1718452800");
    assert_eq!(out("<?php echo strtotime('2024-06-15',1718452845);"), "1718409600");
    assert_eq!(out("<?php echo strtotime('2024/06/15',1718452845);"), "1718409600");
    assert_eq!(out("<?php echo strtotime('2024-06-15T12:00:45',1718452845);"), "1718452845");
}

#[test]
fn strtotime_at_and_now() {
    assert_eq!(out("<?php echo strtotime('@1718452845',1718452845);"), "1718452845");
    assert_eq!(out("<?php echo strtotime('@0',1718452845);"), "0");
    assert_eq!(out("<?php echo strtotime('now',1718452845);"), "1718452845");
}

#[test]
fn strtotime_relative_seconds_based() {
    assert_eq!(out("<?php echo strtotime('+1 day',1718452845);"), "1718539245");
    assert_eq!(out("<?php echo strtotime('-1 day',1718452845);"), "1718366445");
    assert_eq!(out("<?php echo strtotime('+1 week',1718452845);"), "1719057645");
    assert_eq!(out("<?php echo strtotime('+2 weeks',1718452845);"), "1719662445");
    assert_eq!(out("<?php echo strtotime('+3 hours',1718452845);"), "1718463645");
    assert_eq!(out("<?php echo strtotime('+30 minutes',1718452845);"), "1718454645");
    assert_eq!(out("<?php echo strtotime('+10 seconds',1718452845);"), "1718452855");
    assert_eq!(out("<?php echo strtotime('+1 day +2 hours',1718452845);"), "1718546445");
}

#[test]
fn strtotime_relative_calendar() {
    // month/year add use civil arithmetic (overflow normalizes like PHP).
    assert_eq!(out("<?php echo strtotime('+1 month',1718452845);"), "1721044845");
    assert_eq!(out("<?php echo strtotime('+2 months',1718452845);"), "1723723245");
    assert_eq!(out("<?php echo strtotime('+1 year',1718452845);"), "1749988845");
    assert_eq!(out("<?php echo strtotime('-1 year',1718452845);"), "1686830445");
    // Jan 31 + 1 month → Feb 31 → March 2 (PHP does not clamp).
    assert_eq!(out("<?php echo strtotime('+1 month',1706659200);"), "1709337600");
}

#[test]
fn strtotime_failure() {
    assert_eq!(out("<?php var_dump(strtotime('garbage nonsense',1718452845));"), "bool(false)\n");
    assert_eq!(out("<?php var_dump(strtotime('',1718452845));"), "bool(false)\n");
}

// --- Step 34-4: DateTime core -------------------------------------------------
// DateTime is a prelude PHP class holding its epoch internally and delegating
// to the pure date()/mktime()/strtotime() builtins. Oracle-verified.

#[test]
fn datetime_construct_and_format() {
    assert_eq!(
        out("<?php $d=new DateTime('2024-06-15 12:30:45'); echo $d->format('Y-m-d H:i:s');"),
        "2024-06-15 12:30:45"
    );
    assert_eq!(
        out("<?php echo (new DateTime('2024-06-15'))->getTimestamp();"),
        "1718409600"
    );
    assert_eq!(
        out("<?php $d=new DateTime('2024-06-15 12:30:45'); echo $d->getTimestamp();"),
        "1718454645"
    );
}

#[test]
fn datetime_set_timestamp_date_time() {
    assert_eq!(
        out("<?php $d=new DateTime('2024-06-15 12:30:45'); $d->setTimestamp(1718452845); echo $d->format('Y-m-d H:i:s');"),
        "2024-06-15 12:00:45"
    );
    // setDate keeps the time component.
    assert_eq!(
        out("<?php $d=new DateTime('2024-06-15 12:00:45'); $d->setDate(2020,1,31); echo $d->format('Y-m-d H:i:s');"),
        "2020-01-31 12:00:45"
    );
    // setTime keeps the date component.
    assert_eq!(
        out("<?php $d=new DateTime('2020-01-31 12:00:45'); $d->setTime(8,5,9); echo $d->format('Y-m-d H:i:s');"),
        "2020-01-31 08:05:09"
    );
}

#[test]
fn datetime_is_mutable_and_fluent() {
    // Aliasing: mutating $b mutates $a (object handle semantics).
    assert_eq!(
        out("<?php $a=new DateTime('2024-06-15'); $b=$a; $b->setTime(1,2,3); echo $a->format('H:i:s');"),
        "01:02:03"
    );
    // Setters return $this for fluent chaining.
    assert_eq!(
        out("<?php echo (new DateTime('2024-06-15'))->setTime(10,0,0)->format('H:i:s');"),
        "10:00:00"
    );
}

// --- Step 34-5: DateTimeImmutable + modify ------------------------------------

#[test]
fn datetime_modify_mutates() {
    // DateTime::modify mutates in place and returns $this.
    assert_eq!(
        out("<?php $d=new DateTime('2024-06-15 12:00:00'); $r=$d->modify('+1 day'); echo $d->format('Y-m-d H:i:s'), ($r===$d?'/same':'/diff');"),
        "2024-06-16 12:00:00/same"
    );
    assert_eq!(
        out("<?php $d=new DateTime('2024-06-15 12:00:00'); $d->modify('+1 day')->modify('+2 hours'); echo $d->format('Y-m-d H:i:s');"),
        "2024-06-16 14:00:00"
    );
}

#[test]
fn datetimeimmutable_returns_new() {
    // modify returns a new instance, original unchanged.
    assert_eq!(
        out("<?php $a=new DateTimeImmutable('2024-01-01 00:00:00'); $b=$a->modify('+1 month'); echo $a->format('Y-m-d'),'/',$b->format('Y-m-d'),($a===$b?'/same':'/distinct');"),
        "2024-01-01/2024-02-01/distinct"
    );
    // setTime/setDate/setTimestamp all return new instances.
    assert_eq!(
        out("<?php $a=new DateTimeImmutable('2024-01-01 00:00:00'); $b=$a->setTime(10,30,0); echo $a->format('H:i:s'),'/',$b->format('H:i:s');"),
        "00:00:00/10:30:00"
    );
    assert_eq!(
        out("<?php $a=new DateTimeImmutable('2024-01-01'); $b=$a->setDate(2030,5,20); echo $a->format('Y-m-d'),'/',$b->format('Y-m-d');"),
        "2024-01-01/2030-05-20"
    );
    assert_eq!(
        out("<?php $a=new DateTimeImmutable('2024-01-01'); $b=$a->setTimestamp(1718452845); echo $a->getTimestamp(),'/',$b->getTimestamp();"),
        "1704067200/1718452845"
    );
}

#[test]
fn datetime_at_construct_and_interface() {
    // "@N" epoch constructor.
    assert_eq!(
        out("<?php echo (new DateTimeImmutable('@1718452845'))->format('Y-m-d H:i:s');"),
        "2024-06-15 12:00:45"
    );
    // Both classes implement DateTimeInterface.
    assert_eq!(
        out("<?php $d=new DateTime('2024-01-01'); $i=new DateTimeImmutable('2024-01-01'); var_dump($d instanceof DateTimeInterface, $i instanceof DateTimeInterface);"),
        "bool(true)\nbool(true)\n"
    );
}

// --- Step 34-6: DateInterval + add/sub + diff ---------------------------------

#[test]
fn dateinterval_construct_and_props() {
    assert_eq!(
        out("<?php $iv=new DateInterval('P1Y2M3DT4H5M6S'); echo $iv->y,',',$iv->m,',',$iv->d,',',$iv->h,',',$iv->i,',',$iv->s,',',$iv->invert;"),
        "1,2,3,4,5,6,0"
    );
    // days is false when built from a spec.
    assert_eq!(out("<?php var_dump((new DateInterval('P1Y'))->days);"), "bool(false)\n");
    // weeks fold into days.
    assert_eq!(out("<?php echo (new DateInterval('P2W'))->d;"), "14");
    // bad spec throws.
    assert_eq!(
        out("<?php try { new DateInterval('nonsense'); } catch (Exception $e) { echo 'caught'; }"),
        "caught"
    );
}

#[test]
fn dateinterval_format() {
    assert_eq!(
        out("<?php echo (new DateInterval('P1Y2M3DT4H5M6S'))->format('%y years %m months %d days %h:%i:%s');"),
        "1 years 2 months 3 days 4:5:6"
    );
    // zero-padded specifiers.
    assert_eq!(out("<?php echo (new DateInterval('PT5M3S'))->format('%H:%I:%S');"), "00:05:03");
    // %a is (unknown) from a spec; %R is the sign; %% is a literal percent.
    assert_eq!(out("<?php echo (new DateInterval('P1Y'))->format('%R%a%%');"), "+(unknown)%");
}

#[test]
fn datetime_add_sub() {
    // add a calendar month, then subtract a year, then add time.
    assert_eq!(
        out("<?php $d=new DateTime('2024-01-15 10:00:00'); $d->add(new DateInterval('P1M')); echo $d->format('Y-m-d H:i:s');"),
        "2024-02-15 10:00:00"
    );
    assert_eq!(
        out("<?php $d=new DateTime('2024-02-15 10:00:00'); $d->sub(new DateInterval('P1Y')); echo $d->format('Y-m-d H:i:s');"),
        "2023-02-15 10:00:00"
    );
    assert_eq!(
        out("<?php $d=new DateTime('2023-02-15 10:00:00'); $d->add(new DateInterval('PT1H30M')); echo $d->format('Y-m-d H:i:s');"),
        "2023-02-15 11:30:00"
    );
    // immutable add returns a new instance.
    assert_eq!(
        out("<?php $a=new DateTimeImmutable('2024-01-01'); $b=$a->add(new DateInterval('P1M')); echo $a->format('Y-m-d'),'/',$b->format('Y-m-d');"),
        "2024-01-01/2024-02-01"
    );
}

#[test]
fn datetime_diff() {
    assert_eq!(
        out("<?php $a=new DateTime('2024-06-15'); $b=new DateTime('2024-06-20'); $d=$a->diff($b); echo $d->days,'/',$d->d,'/',$d->invert;"),
        "5/5/0"
    );
    // reversed: invert flips, days stays positive.
    assert_eq!(
        out("<?php $a=new DateTime('2024-06-20'); $b=new DateTime('2024-06-15'); $d=$a->diff($b); echo $d->days,'/',$d->invert;"),
        "5/1"
    );
    // y/m/d breakdown + total days.
    assert_eq!(
        out("<?php $d=(new DateTime('2020-01-15'))->diff(new DateTime('2024-03-20')); echo $d->format('%y years %m months %d days'),'/',$d->days;"),
        "4 years 2 months 5 days/1526"
    );
    // borrowing edge: Jan 31 → Mar 1 is 30 days, 0 months.
    assert_eq!(
        out("<?php $d=(new DateTime('2024-01-31'))->diff(new DateTime('2024-03-01')); echo $d->y,'y',$d->m,'m',$d->d,'d';"),
        "0y0m30d"
    );
}

// --- Step 34-7: createFromFormat ----------------------------------------------

#[test]
fn datetime_create_from_format() {
    // Full date+time is deterministic.
    assert_eq!(
        out("<?php echo DateTime::createFromFormat('Y-m-d H:i:s','2024-06-15 12:30:45')->format('Y-m-d H:i:s');"),
        "2024-06-15 12:30:45"
    );
    // '!' resets all fields to the Unix epoch first (time → 00:00:00).
    assert_eq!(
        out("<?php echo DateTime::createFromFormat('!Y-m-d','2024-06-15')->format('Y-m-d H:i:s');"),
        "2024-06-15 00:00:00"
    );
    // '|' resets the remaining (unparsed) fields.
    assert_eq!(
        out("<?php echo DateTime::createFromFormat('Y-m-d|','2024-06-15')->format('Y-m-d H:i:s');"),
        "2024-06-15 00:00:00"
    );
    // 2-digit year, dot separators.
    assert_eq!(
        out("<?php echo DateTime::createFromFormat('!d.m.y','05.03.21')->format('Y-m-d');"),
        "2021-03-05"
    );
    // single-digit-friendly chars j/n/G.
    assert_eq!(
        out("<?php echo DateTime::createFromFormat('!j-n-Y G:i','5-3-2024 9:07')->format('Y-m-d H:i');"),
        "2024-03-05 09:07"
    );
}

#[test]
fn datetime_create_from_format_failure_and_immutable() {
    assert_eq!(
        out("<?php var_dump(DateTime::createFromFormat('Y-m-d','not a date'));"),
        "bool(false)\n"
    );
    // The immutable variant returns a DateTimeImmutable.
    assert_eq!(
        out("<?php $d=DateTimeImmutable::createFromFormat('!Y-m-d','2024-06-15'); echo $d->format('Y-m-d H:i:s'),'/',get_class($d);"),
        "2024-06-15 00:00:00/DateTimeImmutable"
    );
}

#[test]
fn date_default_timezone_utc_scope() {
    // set returns true; get is always UTC (D-DT3). Setting a non-UTC zone is a
    // documented no-op: date() keeps producing UTC.
    assert_eq!(
        out("<?php var_dump(date_default_timezone_set('Europe/London')); echo date_default_timezone_get();"),
        "bool(true)\nUTC"
    );
    assert_eq!(
        out("<?php date_default_timezone_set('America/Chicago'); echo date('H', 1718452845);"),
        "12"
    );
}

// --- Step 35-1: procedural date API — infra + first wrappers ------------------
// The procedural functions (date_create, date_format, ...) are PHP global
// functions authored in the prelude that delegate to the step-34 OOP API. This
// exercises the new infra: prelude global functions are merged into
// `Program.functions` and resolved by name like any user function (D-PD1).

#[test]
fn date_create_and_format() {
    // date_create builds a DateTime; date_format delegates to ->format().
    assert_eq!(
        out("<?php $d=date_create('2024-06-15 12:30:45'); echo get_class($d),'/',date_format($d,'Y-m-d H:i:s');"),
        "DateTime/2024-06-15 12:30:45"
    );
}

#[test]
fn date_create_immutable_class() {
    assert_eq!(
        out("<?php echo get_class(date_create_immutable('2024-01-01'));"),
        "DateTimeImmutable"
    );
}

#[test]
fn date_timestamp_get_proc() {
    // Same epoch as the OOP getTimestamp() (1718454645 = 2024-06-15 12:30:45 UTC).
    assert_eq!(
        out("<?php echo date_timestamp_get(date_create('2024-06-15 12:30:45'));"),
        "1718454645"
    );
}

#[test]
fn date_create_default_arg_is_datetime() {
    // No argument defaults to "now"; the value is non-deterministic but the type
    // is not (D-DT5: we only assert the class, never the instant).
    assert_eq!(out("<?php echo get_class(date_create());"), "DateTime");
}

// --- Step 35-2: procedural mutators + diff -----------------------------------

#[test]
fn date_diff_proc() {
    // date_diff returns a DateInterval; reversed args flip invert.
    assert_eq!(
        out("<?php $d=date_diff(date_create('2024-06-20'),date_create('2024-06-15')); echo $d->days,'/',$d->invert;"),
        "5/1"
    );
    // $absolute=true clears invert (D-PD4), days unchanged.
    assert_eq!(
        out("<?php $d=date_diff(date_create('2024-06-20'),date_create('2024-06-15'),true); echo $d->days,'/',$d->invert;"),
        "5/0"
    );
}

#[test]
fn date_add_sub_proc() {
    // date_add mutates the DateTime in place and returns it (same handle).
    assert_eq!(
        out("<?php $x=date_create('2024-06-15 00:00:00'); $r=date_add($x,new DateInterval('P1D')); echo ($r===$x?'same':'diff'),'/',date_format($x,'Y-m-d');"),
        "same/2024-06-16"
    );
    // date_sub: P1M back from Jun 15 → May 15.
    assert_eq!(
        out("<?php $y=date_create('2024-06-15'); date_sub($y,new DateInterval('P1M')); echo date_format($y,'Y-m-d');"),
        "2024-05-15"
    );
}

#[test]
fn date_modify_and_setters_proc() {
    assert_eq!(
        out("<?php $z=date_create('2024-06-15 12:00:00'); date_modify($z,'+2 hours'); echo date_format($z,'H:i:s');"),
        "14:00:00"
    );
    // date_date_set + date_time_set chain on the same object.
    assert_eq!(
        out("<?php $w=date_create('2024-06-15 12:00:00'); date_date_set($w,2020,2,29); date_time_set($w,8,30,0); echo date_format($w,'Y-m-d H:i:s');"),
        "2020-02-29 08:30:00"
    );
    assert_eq!(
        out("<?php $w=date_create('2024-06-15'); date_timestamp_set($w,1718452845); echo date_timestamp_get($w);"),
        "1718452845"
    );
}

#[test]
fn date_add_on_immutable_proc() {
    // On DateTimeImmutable, add returns a NEW instance; date_add forwards it.
    assert_eq!(
        out("<?php $a=date_create_immutable('2024-01-01'); $b=date_add($a,new DateInterval('P1M')); echo date_format($a,'Y-m-d'),'/',date_format($b,'Y-m-d');"),
        "2024-01-01/2024-02-01"
    );
}

// --- Step 35-3: procedural createFromFormat + interval helpers ----------------

#[test]
fn date_create_from_format_proc() {
    // '!' resets fields to the Unix epoch first → time is 00:00:00.
    assert_eq!(
        out("<?php echo date_create_from_format('!d/m/Y','15/06/2024')->format('Y-m-d H:i:s');"),
        "2024-06-15 00:00:00"
    );
    // Immutable variant yields a DateTimeImmutable.
    assert_eq!(
        out("<?php $d=date_create_immutable_from_format('!Y-m-d','2024-06-15'); echo get_class($d),'/',$d->format('Y-m-d');"),
        "DateTimeImmutable/2024-06-15"
    );
    // A non-matching value returns false.
    assert_eq!(
        out("<?php var_dump(date_create_from_format('Y-m-d','nope'));"),
        "bool(false)\n"
    );
}

#[test]
fn date_interval_format_proc() {
    assert_eq!(
        out("<?php $iv=date_diff(date_create('2024-06-15'),date_create('2024-06-20')); echo date_interval_format($iv,'%d days');"),
        "5 days"
    );
}

#[test]
fn date_interval_create_from_date_string_proc() {
    // Single unit; days flag is false when built from a relative string.
    assert_eq!(
        out("<?php $i=date_interval_create_from_date_string('1 day'); echo $i->d,'/',$i->y,'/',$i->m,'/',$i->h,'/',$i->i,'/',$i->s; var_dump($i->days);"),
        "1/0/0/0/0/0bool(false)\n"
    );
    // Weeks fold into days; multiple units accumulate.
    assert_eq!(
        out("<?php $i=date_interval_create_from_date_string('2 weeks 3 hours'); echo $i->d,'/',$i->h;"),
        "14/3"
    );
    // Years/months stay separate (no normalization).
    assert_eq!(
        out("<?php $i=date_interval_create_from_date_string('1 year 2 months'); echo $i->y,'/',$i->m;"),
        "1/2"
    );
    // Applying it to a date advances by the parsed amount.
    assert_eq!(
        out("<?php $x=date_create('2024-06-15'); date_add($x,date_interval_create_from_date_string('10 days')); echo date_format($x,'Y-m-d');"),
        "2024-06-25"
    );
}

// --- Step 35-4: getdate / localtime (pure builtins, D-PD2) --------------------
// Oracle ts 1718452845 = Sat 2024-06-15 12:00:45 UTC.

#[test]
fn getdate_components() {
    assert_eq!(
        out("<?php $g=getdate(1718452845); echo $g['seconds'],'/',$g['minutes'],'/',$g['hours'],'/',$g['mday'],'/',$g['wday'],'/',$g['mon'],'/',$g['year'],'/',$g['yday'],'/',$g['weekday'],'/',$g['month'],'/',$g[0];"),
        "45/0/12/15/6/6/2024/166/Saturday/June/1718452845"
    );
}

#[test]
fn getdate_var_dump_order() {
    // The exact key order + the trailing numeric 0 (print_r is order-sensitive).
    assert_eq!(
        out("<?php print_r(getdate(1718452845));"),
        "Array\n(\n    [seconds] => 45\n    [minutes] => 0\n    [hours] => 12\n    [mday] => 15\n    [wday] => 6\n    [mon] => 6\n    [year] => 2024\n    [yday] => 166\n    [weekday] => Saturday\n    [month] => June\n    [0] => 1718452845\n)\n"
    );
}

#[test]
fn localtime_numeric() {
    // [sec,min,hour,mday,mon(0-based),year-1900,wday,yday,isdst]
    assert_eq!(
        out("<?php echo implode(',', localtime(1718452845));"),
        "45,0,12,15,5,124,6,166,0"
    );
}

#[test]
fn localtime_associative() {
    assert_eq!(
        out("<?php $t=localtime(1718452845,true); echo $t['tm_sec'],'/',$t['tm_mon'],'/',$t['tm_year'],'/',$t['tm_wday'],'/',$t['tm_yday'],'/',$t['tm_isdst'];"),
        "45/5/124/6/166/0"
    );
}

// --- Step 39: generators ---

#[test]
fn var_dump_generator() {
    // A Generator dumps with its `function` pseudo-property (step 39-7).
    assert_eq!(
        out("<?php function g(){yield 1;} var_dump(g());"),
        "object(Generator)#1 (1) {\n  [\"function\"]=>\n  string(1) \"g\"\n}\n"
    );
}

#[test]
fn print_r_generator() {
    assert_eq!(
        out("<?php function g(){yield 1;} print_r(g());"),
        "Generator Object\n(\n    [function] => g\n)\n"
    );
}

// --- mbstring batch 1 (UTF-8 core) — mb-1: length / substr / split ---

#[test]
fn mb_strlen_counts_code_points() {
    assert_eq!(out("<?php echo mb_strlen('città');"), "5");
    assert_eq!(out("<?php echo mb_strlen('日本語');"), "3");
    // e + combining acute = two code points (not one grapheme).
    assert_eq!(out("<?php echo mb_strlen('e\u{301}');"), "2");
    assert_eq!(out("<?php echo mb_strlen('');"), "0");
    // Each invalid byte counts as one unit (oracle: a \xFF \xFE b == 4).
    assert_eq!(out("<?php echo mb_strlen(\"a\\xFF\\xFEb\");"), "4");
}

#[test]
fn mb_substr_by_code_point() {
    assert_eq!(out("<?php echo mb_substr('日本語', 1, 1);"), "本");
    assert_eq!(out("<?php echo mb_substr('città', -3, 2);"), "tt");
    // Omitted length runs to the end.
    assert_eq!(out("<?php echo mb_substr('città', 2);"), "ttà");
    assert_eq!(out("<?php echo mb_substr('日本語', -1);"), "語");
}

#[test]
fn mb_str_split_groups_code_points() {
    assert_eq!(out("<?php echo implode('|', mb_str_split('abcde', 2));"), "ab|cd|e");
    // Default length is 1.
    assert_eq!(out("<?php echo implode('|', mb_str_split('日本語'));"), "日|本|語");
    // Empty input yields an empty array.
    assert_eq!(out("<?php echo count(mb_str_split(''));"), "0");
}

#[test]
fn mb_unknown_encoding_is_value_error() {
    match fatal("<?php mb_strlen('abc', 'NO-SUCH-ENC');") {
        PhpError::ValueError(m) => assert_eq!(
            m,
            "mb_strlen(): Argument #2 ($encoding) must be a valid encoding, \"NO-SUCH-ENC\" given"
        ),
        other => panic!("expected ValueError, got {other:?}"),
    }
}

// --- mb-2: case (strtoupper / strtolower / convert_case / ucfirst / lcfirst) ---

#[test]
fn mb_strtoupper_full_unicode() {
    // ß expands to SS; accented and Greek (final sigma → Σ) map per Unicode.
    assert_eq!(out("<?php echo mb_strtoupper('ß');"), "SS");
    assert_eq!(out("<?php echo mb_strtoupper('café');"), "CAFÉ");
    assert_eq!(out("<?php echo mb_strtoupper('σίσυφος');"), "ΣΊΣΥΦΟΣ");
}

#[test]
fn mb_strtolower_full_unicode() {
    // İ (dotted capital I) lowercases to "i" + combining dot above (two cp).
    assert_eq!(out("<?php echo mb_strtolower('İ');"), "i\u{307}");
    assert_eq!(out("<?php echo mb_strtolower('CAFÉ');"), "café");
}

#[test]
fn mb_convert_case_modes() {
    assert_eq!(out("<?php echo mb_convert_case('café', MB_CASE_UPPER);"), "CAFÉ");
    assert_eq!(out("<?php echo mb_convert_case('CAFÉ', MB_CASE_LOWER);"), "café");
    assert_eq!(out("<?php echo mb_convert_case('HELLO wÖRLD', MB_CASE_TITLE);"), "Hello Wörld");
    // Digits and hyphens are word boundaries.
    assert_eq!(out("<?php echo mb_convert_case('abc123def', MB_CASE_TITLE);"), "Abc123Def");
    assert_eq!(out("<?php echo mb_convert_case('über-fluß', MB_CASE_TITLE);"), "Über-Fluß");
    assert_eq!(out("<?php echo mb_convert_case('', MB_CASE_TITLE);"), "");
}

#[test]
fn mb_convert_case_invalid_mode_is_value_error() {
    match fatal("<?php mb_convert_case('x', 99);") {
        PhpError::ValueError(m) => assert_eq!(
            m,
            "mb_convert_case(): Argument #2 ($mode) must be one of the MB_CASE_* constants"
        ),
        other => panic!("expected ValueError, got {other:?}"),
    }
}

#[test]
fn mb_ucfirst_lcfirst() {
    assert_eq!(out("<?php echo mb_ucfirst('straße');"), "Straße");
    // ucfirst only touches the first code point; the rest is verbatim.
    assert_eq!(out("<?php echo mb_ucfirst('ÉLAN');"), "ÉLAN");
    assert_eq!(out("<?php echo mb_lcfirst('ÉLAN');"), "éLAN");
}

// --- mb-3: search (strpos family / strstr family / substr_count) ---

#[test]
fn mb_strpos_code_point_offsets() {
    assert_eq!(out("<?php echo mb_strpos('日本語日本', '本');"), "1");
    assert_eq!(out("<?php echo mb_strpos('日本語日本', '本', 2);"), "4");
    assert_eq!(out("<?php var_dump(mb_strpos('abc', 'z'));"), "bool(false)\n");
    assert_eq!(out("<?php echo mb_strpos('abc', '');"), "0");
    assert_eq!(out("<?php echo mb_strpos('ababa', 'a', -2);"), "4");
    assert_eq!(out("<?php echo mb_stripos('HÉLLO', 'é');"), "1");
}

#[test]
fn mb_strrpos_last_occurrence() {
    assert_eq!(out("<?php echo mb_strrpos('日本語日本', '本');"), "4");
    assert_eq!(out("<?php echo mb_strripos('ABABA', 'b');"), "3");
    assert_eq!(out("<?php var_dump(mb_strrpos('abc', 'z'));"), "bool(false)\n");
}

#[test]
fn mb_strstr_family() {
    assert_eq!(out("<?php echo mb_strstr('foobar café', 'bar');"), "bar café");
    assert_eq!(out("<?php echo mb_strstr('foobar', 'bar', true);"), "foo");
    assert_eq!(out("<?php var_dump(mb_strstr('foo', 'z'));"), "bool(false)\n");
    assert_eq!(out("<?php echo mb_stristr('Café au lait', 'CAFÉ');"), "Café au lait");
    // strrchr uses the whole needle and the last occurrence.
    assert_eq!(out("<?php echo mb_strrchr('a/b/c', '/');"), "/c");
    assert_eq!(out("<?php echo mb_strrchr('a/b/c', '/b');"), "/b/c");
    assert_eq!(out("<?php echo mb_strrichr('A.b.C', 'B');"), "b.C");
}

#[test]
fn mb_substr_count_non_overlapping() {
    assert_eq!(out("<?php echo mb_substr_count('ababab', 'ab');"), "3");
    assert_eq!(out("<?php echo mb_substr_count('aaa', 'aa');"), "1");
}

// --- mb-4: ord / chr / str_pad / trim family / check_encoding ---

#[test]
fn mb_ord_and_chr() {
    assert_eq!(out("<?php echo mb_ord('語');"), "35486");
    assert_eq!(out("<?php echo mb_chr(26085);"), "日");
    // Surrogates and out-of-range code points are not valid → false.
    assert_eq!(out("<?php var_dump(mb_chr(0xD800));"), "bool(false)\n");
    assert_eq!(out("<?php var_dump(mb_chr(0x110000));"), "bool(false)\n");
}

#[test]
fn mb_ord_empty_is_value_error() {
    match fatal("<?php mb_ord('');") {
        PhpError::ValueError(m) => {
            assert_eq!(m, "mb_ord(): Argument #1 ($string) must not be empty")
        }
        other => panic!("expected ValueError, got {other:?}"),
    }
}

#[test]
fn mb_str_pad_by_code_point() {
    assert_eq!(out("<?php echo mb_str_pad('é', 4, '*', STR_PAD_LEFT);"), "***é");
    assert_eq!(out("<?php echo mb_str_pad('x', 5, '-', STR_PAD_BOTH);"), "--x--");
    // Default pad is a space, default type is STR_PAD_RIGHT.
    assert_eq!(out("<?php echo mb_str_pad('日', 3);"), "日  ");
    // No padding when already long enough.
    assert_eq!(out("<?php echo mb_str_pad('abc', 2, '*');"), "abc");
}

#[test]
fn mb_trim_family() {
    assert_eq!(out("<?php echo mb_trim('  héllo \n');"), "héllo");
    assert_eq!(out("<?php echo mb_trim('xxabcxx', 'x');"), "abc");
    assert_eq!(out("<?php echo mb_rtrim('café…', '…');"), "café");
    assert_eq!(out("<?php echo mb_ltrim('…café', '…');"), "café");
}

#[test]
fn mb_check_encoding_utf8() {
    assert_eq!(out("<?php var_dump(mb_check_encoding('café', 'UTF-8'));"), "bool(true)\n");
    assert_eq!(out("<?php var_dump(mb_check_encoding(\"a\\xFF\", 'UTF-8'));"), "bool(false)\n");
}

// --- step 42b: width (mb_strwidth / mb_strimwidth / mb_strcut) ---

#[test]
fn mb_strwidth_counts_east_asian_width() {
    // ASCII = 1 each, CJK ideographs = 2 each.
    assert_eq!(out("<?php echo mb_strwidth('ABC日本語');"), "9");
    assert_eq!(out("<?php echo mb_strwidth('hello');"), "5");
    // Combining mark, zero-width space, ambiguous-width: all width 1 (mbfl rule).
    assert_eq!(out("<?php echo mb_strwidth('e\u{301}');"), "2");
    assert_eq!(out("<?php echo mb_strwidth('\u{200B}');"), "1");
    assert_eq!(out("<?php echo mb_strwidth('\u{00B1}');"), "1");
    // Emoji, fullwidth and halfwidth forms, Hangul syllable.
    assert_eq!(out("<?php echo mb_strwidth('\u{1F600}');"), "2");
    assert_eq!(out("<?php echo mb_strwidth('\u{FF21}');"), "2");
    assert_eq!(out("<?php echo mb_strwidth('\u{FF61}');"), "1");
    assert_eq!(out("<?php echo mb_strwidth('\u{AC00}');"), "2");
    // Each invalid byte becomes one replacement unit of width 1.
    assert_eq!(out("<?php echo mb_strwidth(\"\\xFF\\xFE\");"), "2");
}

#[test]
fn mb_strimwidth_trims_to_width() {
    // Truncated: marker width counts toward the limit (こ=2, ...=3, fits in 6).
    assert_eq!(out("<?php echo mb_strimwidth('こんにちは', 0, 6, '...');"), "こ...");
    // Whole string fits → no marker appended.
    assert_eq!(out("<?php echo mb_strimwidth('こんにちは', 0, 10, '...');"), "こんにちは");
    // start is a code-point offset; tail fits exactly → no marker.
    assert_eq!(out("<?php echo mb_strimwidth('こんにちは', 2, 6, '...');"), "にちは");
    assert_eq!(out("<?php echo mb_strimwidth('A日本', 1, 4);"), "日本");
    // Empty default marker.
    assert_eq!(out("<?php echo mb_strimwidth('こんにちは', 0, 6);"), "こんに");
    // Marker wider than the limit → output is the marker.
    assert_eq!(out("<?php echo mb_strimwidth('こんにちは', 0, 2, '....');"), "....");
    assert_eq!(out("<?php echo mb_strimwidth('ABCDE', 0, 5, 'x');"), "ABCDE");
    assert_eq!(out("<?php echo mb_strimwidth('Hello World', 0, 8, '...');"), "Hello...");
    // Negative start counts from the end.
    assert_eq!(out("<?php echo mb_strimwidth('こんにちは', -2, 4);"), "ちは");
    // start == length is allowed and yields the empty string.
    assert_eq!(out("<?php var_dump(mb_strimwidth('こんにちは', 5, 4, 'x'));"), "string(0) \"\"\n");
}

#[test]
fn mb_strimwidth_start_out_of_range_is_value_error() {
    match fatal("<?php mb_strimwidth('こんにちは', 10, 4, 'x');") {
        PhpError::ValueError(m) => {
            assert_eq!(m, "mb_strimwidth(): Argument #2 ($start) is out of range")
        }
        other => panic!("expected ValueError, got {other:?}"),
    }
}

#[test]
fn mb_strcut_is_byte_oriented() {
    // Byte length 4 requested but never splits a multibyte char (stops at 日).
    assert_eq!(out("<?php echo mb_strcut('日本語', 0, 4);"), "日");
    // start byte 1 falls inside 日 → rounds down to its boundary.
    assert_eq!(out("<?php echo mb_strcut('日本語', 1, 5);"), "日");
    // length is measured from the rounded-down start.
    assert_eq!(out("<?php echo mb_strcut('日本語', 4, 5);"), "本");
    assert_eq!(out("<?php echo mb_strcut('日本語', 2, 7);"), "日本");
    // A char that does not fully fit is excluded.
    assert_eq!(out("<?php echo mb_strcut('日本語', 0, 2);"), "");
    // ASCII behaves like a plain byte cut.
    assert_eq!(out("<?php echo mb_strcut('Hello', 1, 3);"), "ell");
    assert_eq!(out("<?php echo mb_strcut('Hello', -3, 2);"), "ll");
    // Omitted length runs to the end (rounded to a boundary).
    assert_eq!(out("<?php echo mb_strcut('日本語', 3);"), "本語");
    assert_eq!(out("<?php echo mb_strcut('日本語', 2);"), "日本語");
    // start beyond the string → empty.
    assert_eq!(out("<?php echo mb_strcut('日本語', 100, 5);"), "");
}

// --- step 42a: encoding (mb_convert_encoding / mb_detect_encoding) ---

#[test]
fn mb_convert_encoding_transcodes() {
    // UTF-8 → ISO-8859-1: é (U+00E9) becomes the single byte 0xE9.
    assert_eq!(
        out_bytes("<?php echo mb_convert_encoding('café', 'ISO-8859-1', 'UTF-8');"),
        b"caf\xe9"
    );
    // from_encoding omitted defaults to UTF-8.
    assert_eq!(
        out_bytes("<?php echo mb_convert_encoding('café', 'ISO-8859-1');"),
        b"caf\xe9"
    );
    // ISO-8859-1 → UTF-8.
    assert_eq!(
        out_bytes("<?php echo mb_convert_encoding(\"\\xE9\", 'UTF-8', 'ISO-8859-1');"),
        b"\xc3\xa9"
    );
    // True Latin-1: 0x80 → U+0080 (NOT windows-1252's € — D-MB-enc-latin1).
    assert_eq!(
        out_bytes("<?php echo mb_convert_encoding(\"\\x80\", 'UTF-8', 'ISO-8859-1');"),
        b"\xc2\x80"
    );
    // Windows-1252: 0x80 → € (U+20AC).
    assert_eq!(
        out_bytes("<?php echo mb_convert_encoding(\"\\x80\", 'UTF-8', 'Windows-1252');"),
        b"\xe2\x82\xac"
    );
    // Un-encodable target char → substitute '?' (0x3F), not an HTML entity.
    assert_eq!(
        out_bytes("<?php echo mb_convert_encoding('€', 'ISO-8859-1', 'UTF-8');"),
        b"?"
    );
    // UTF-16: bare name is big-endian; LE/BE explicit.
    assert_eq!(
        out_bytes("<?php echo mb_convert_encoding('AB', 'UTF-16', 'UTF-8');"),
        b"\x00A\x00B"
    );
    assert_eq!(
        out_bytes("<?php echo mb_convert_encoding('AB', 'UTF-16LE', 'UTF-8');"),
        b"A\x00B\x00"
    );
    // from_encoding as a detect-list: UTF-8 is picked, then encoded to UTF-16BE.
    assert_eq!(
        out_bytes("<?php echo mb_convert_encoding('café', 'UTF-16', 'UTF-8, ISO-8859-1');"),
        b"\x00c\x00a\x00f\x00\xe9"
    );
    // Round-trip through Shift-JIS (multibyte CJK).
    assert_eq!(
        out_bytes("<?php echo mb_convert_encoding('日', 'SJIS', 'UTF-8');"),
        b"\x93\xfa"
    );
    assert_eq!(
        out_bytes("<?php echo mb_convert_encoding(\"\\x93\\xfa\", 'UTF-8', 'SJIS');"),
        "日".as_bytes()
    );
}

#[test]
fn mb_convert_encoding_unknown_is_value_error() {
    match fatal("<?php mb_convert_encoding('x', 'BOGUS', 'UTF-8');") {
        PhpError::ValueError(m) => assert_eq!(
            m,
            "mb_convert_encoding(): Argument #2 ($to_encoding) must be a valid encoding, \"BOGUS\" given"
        ),
        other => panic!("expected ValueError, got {other:?}"),
    }
    match fatal("<?php mb_convert_encoding('x', 'UTF-8', 'BOGUS');") {
        PhpError::ValueError(m) => assert_eq!(
            m,
            "mb_convert_encoding(): Argument #3 ($from_encoding) contains invalid encoding \"BOGUS\""
        ),
        other => panic!("expected ValueError, got {other:?}"),
    }
}

#[test]
fn mb_detect_encoding_picks_candidate() {
    // Default order is ASCII, UTF-8.
    assert_eq!(out("<?php echo mb_detect_encoding('hello');"), "ASCII");
    assert_eq!(out("<?php echo mb_detect_encoding('café');"), "UTF-8");
    // No candidate validates → fall back to the first candidate (ASCII).
    assert_eq!(out("<?php echo mb_detect_encoding(\"\\xE9\");"), "ASCII");
    // Explicit comma-list and array, with canonical names returned.
    assert_eq!(out("<?php echo mb_detect_encoding('hello', 'UTF-8, ASCII');"), "UTF-8");
    assert_eq!(out("<?php echo mb_detect_encoding(\"\\xE9\", 'UTF-8, ISO-8859-1');"), "ISO-8859-1");
    assert_eq!(
        out("<?php echo mb_detect_encoding(\"\\xE9\", ['UTF-8', 'SJIS', 'ISO-8859-1']);"),
        "ISO-8859-1"
    );
    // Non-strict never returns false: falls back to the first candidate.
    assert_eq!(out("<?php echo mb_detect_encoding(\"\\xFF\\xFE\", ['UTF-8']);"), "UTF-8");
    // Strict mode returns false when nothing fully validates.
    assert_eq!(out("<?php var_dump(mb_detect_encoding('café', ['ASCII'], true));"), "bool(false)\n");
    assert_eq!(
        out("<?php echo mb_detect_encoding(\"\\xE9\", ['UTF-8', 'ISO-8859-1'], true);"),
        "ISO-8859-1"
    );
}

#[test]
fn mb_detect_encoding_invalid_or_empty_list_is_value_error() {
    match fatal("<?php mb_detect_encoding('x', []);") {
        PhpError::ValueError(m) => assert_eq!(
            m,
            "mb_detect_encoding(): Argument #2 ($encodings) must specify at least one encoding"
        ),
        other => panic!("expected ValueError, got {other:?}"),
    }
    match fatal("<?php mb_detect_encoding('x', 'BOGUS');") {
        PhpError::ValueError(m) => assert_eq!(
            m,
            "mb_detect_encoding(): Argument #2 ($encodings) contains invalid encoding \"BOGUS\""
        ),
        other => panic!("expected ValueError, got {other:?}"),
    }
}

// --- step 44: class-A fixes surfaced by the ext/mbstring corpus import ---

#[test]
fn mb_strpos_offset_out_of_range_is_value_error() {
    for (f, ci, rev) in [
        ("mb_strpos", false, false),
        ("mb_stripos", true, false),
        ("mb_strrpos", false, true),
        ("mb_strripos", true, true),
    ] {
        let _ = (ci, rev);
        let src = format!("<?php {f}('f', 'bar', 3);");
        match fatal(&src) {
            PhpError::ValueError(m) => assert_eq!(
                m,
                format!("{f}(): Argument #3 ($offset) must be contained in argument #1 ($haystack)")
            ),
            other => panic!("expected ValueError for {f}, got {other:?}"),
        }
        let src = format!("<?php {f}('f', 'bar', -3);");
        assert!(matches!(fatal(&src), PhpError::ValueError(_)), "{f} negative");
    }
    // An in-range offset still works.
    assert_eq!(out("<?php echo mb_strpos('ababa', 'a', -2);"), "4");
}

#[test]
fn mb_detect_encoding_empty_string_list_is_value_error() {
    match fatal("<?php mb_detect_encoding('Hello', '');") {
        PhpError::ValueError(m) => assert_eq!(
            m,
            "mb_detect_encoding(): Argument #2 ($encodings) must specify at least one encoding"
        ),
        other => panic!("expected ValueError, got {other:?}"),
    }
}

#[test]
fn mb_convert_encoding_empty_from_list_is_value_error() {
    match fatal("<?php mb_convert_encoding('Hello', 'UTF-8', '');") {
        PhpError::ValueError(m) => assert_eq!(
            m,
            "mb_convert_encoding(): Argument #3 ($from_encoding) must specify at least one encoding"
        ),
        other => panic!("expected ValueError, got {other:?}"),
    }
}

// --- step 43a: mbstring regex family (mb_ereg* / mb_split / mb_regex_*) ---

#[test]
fn mb_ereg_captures_and_named_groups() {
    assert_eq!(
        out("<?php mb_ereg('(\\d+)-(\\d+)', 'x12-34y', $m); echo $m[0].'|'.$m[1].'|'.$m[2];"),
        "12-34|12|34"
    );
    assert_eq!(out("<?php var_dump(mb_ereg('(\\d+)-(\\d+)', 'x12-34y', $m));"), "bool(true)\n");
    // No match: returns false and sets $regs to an empty array.
    assert_eq!(
        out("<?php var_dump(mb_ereg('z+', 'abc', $m)); echo '['.implode(',', $m).']';"),
        "bool(false)\n[]"
    );
    // Case-insensitive variant.
    assert_eq!(out("<?php mb_eregi('ABC', 'xxabcyy', $m); echo $m[0];"), "abc");
    // Named group is appended by string key, alongside the numbered group.
    assert_eq!(
        out("<?php mb_ereg('(?<y>\\d+)', '2024', $m); echo $m[0].'|'.$m[1].'|'.$m['y'];"),
        "2024|2024|2024"
    );
    // Backreference inside the pattern.
    assert_eq!(out("<?php var_dump(mb_ereg('(a)\\1', 'aa', $m));"), "bool(true)\n");
    // A non-participating optional group is false.
    assert_eq!(
        out("<?php mb_ereg('(a)(b)?', 'a', $m); var_dump($m[2]);"),
        "bool(false)\n"
    );
}

#[test]
fn mb_ereg_replace_backreferences() {
    assert_eq!(out("<?php echo mb_ereg_replace('(\\w)(\\w)', '\\2\\1', 'abcd');"), "badc");
    assert_eq!(out("<?php echo mb_ereg_replace('(\\w)', '[\\1]', 'ab');"), "[a][b]");
    assert_eq!(out("<?php echo mb_ereg_replace('\\w+', '<\\0>', 'hi');"), "<hi>");
    assert_eq!(out("<?php echo mb_ereg_replace('x+', 'Y', 'axxxb');"), "aYb");
    assert_eq!(out("<?php echo mb_eregi_replace('a', 'X', 'AaA');"), "XXX");
}

#[test]
fn mb_ereg_replace_callback_doubles_digits() {
    assert_eq!(
        out("<?php echo mb_ereg_replace_callback('\\d+', fn($m) => $m[0] * 2, 'a5b10');"),
        "a10b20"
    );
    // Multi-digit matches, the callback returning a built string.
    assert_eq!(
        out("<?php echo mb_ereg_replace_callback('\\d+', fn($m) => '[' . ($m[0] + 1) . ']', 'a1b22c333');"),
        "a[2]b[23]c[334]"
    );
}

#[test]
fn mb_split_keeps_empty_fields_and_limit() {
    assert_eq!(out("<?php echo implode('|', mb_split('\\s+', 'a  b   c'));"), "a|b|c");
    assert_eq!(out("<?php echo implode('|', mb_split(',', 'a,b,c', 2));"), "a|b,c");
    assert_eq!(out("<?php echo implode('|', mb_split(',', ',a,,b,'));"), "|a||b|");
    assert_eq!(out("<?php echo implode('|', mb_split('\\d', 'abc'));"), "abc");
}

#[test]
fn mb_ereg_match_anchored_and_posix_classes() {
    assert_eq!(out("<?php var_dump(mb_ereg_match('\\d+', '123abc'));"), "bool(true)\n");
    assert_eq!(out("<?php var_dump(mb_ereg_match('\\d+', 'abc'));"), "bool(false)\n");
    // Anchored at the start: 'a' is not at the start of 'xa'.
    assert_eq!(out("<?php var_dump(mb_ereg_match('a', 'xa'));"), "bool(false)\n");
    // POSIX character classes work in the default (Ruby) dialect.
    assert_eq!(out("<?php var_dump(mb_ereg_match('[[:digit:]]+', '123'));"), "bool(true)\n");
}

#[test]
fn mb_regex_encoding_and_options_defaults() {
    assert_eq!(out("<?php echo mb_regex_encoding();"), "UTF-8");
    assert_eq!(out("<?php echo mb_regex_set_options();"), "pr");
    assert_eq!(out("<?php var_dump(mb_regex_encoding('UTF-8'));"), "bool(true)\n");
}

#[test]
fn mb_ereg_invalid_pattern_returns_false() {
    // A compile error yields false (PHP also emits a warning, on the side channel).
    assert_eq!(out("<?php var_dump(mb_ereg('(', 'x', $m));"), "bool(false)\n");
}

// --- step 43b: stateful search family (mb_ereg_search_*) ---

#[test]
fn mb_ereg_search_cursor_walks_matches() {
    // Repeated search_pos walks every match as [pos, len], then returns false.
    let src = "<?php mb_ereg_search_init('a1b2c3', '\\d'); $o = []; \
        while (($r = mb_ereg_search_pos()) !== false) { $o[] = $r[0] . ':' . $r[1]; } \
        echo implode(',', $o);";
    assert_eq!(out(src), "1:1,3:1,5:1");
    // Multi-character matches: the cursor advances past the whole match.
    let multi = "<?php mb_ereg_search_init('xx11yy22zz', '\\d+'); $o = []; \
        while (($r = mb_ereg_search_pos()) !== false) { $o[] = $r[0] . ',' . $r[1]; } \
        echo implode(' ', $o);";
    assert_eq!(out(multi), "2,2 6,2");
}

#[test]
fn mb_ereg_search_regs_getregs_and_pos() {
    // search() advances the cursor; getpos returns the byte offset after the match.
    assert_eq!(
        out("<?php mb_ereg_search_init('a1b2c3', '\\d'); var_dump(mb_ereg_search()); \
             echo mb_ereg_search_getpos();"),
        "bool(true)\n2"
    );
    // getregs returns the last successful match's $regs.
    assert_eq!(
        out("<?php mb_ereg_search_init('a1b2c3', '\\d'); mb_ereg_search(); \
             $g = mb_ereg_search_getregs(); echo $g[0];"),
        "1"
    );
    // setpos repositions the cursor; search_regs then matches from there.
    assert_eq!(
        out("<?php mb_ereg_search_init('a1b2c3', '\\d'); mb_ereg_search_setpos(4); \
             $r = mb_ereg_search_regs(); echo $r[0];"),
        "3"
    );
}

// --- goto / labels (step 45) -----------------------------------------------

/// Run a script expected to abort at *compile time* with a `Fatal error:`
/// (undefined label, into-loop, or duplicate label). Returns the CLI-rendered
/// stream and asserts no program output was produced (PHP detects these before
/// running, so nothing prints).
fn goto_fatal(src: &str) -> String {
    let reg = registry();
    let o = run_source_with(b"Command line code", src.as_bytes(), &reg).expect("lowers to outcome");
    assert!(o.stdout.is_empty(), "expected no output, got {:?}", o.stdout);
    String::from_utf8(o.rendered).expect("utf8")
}

#[test]
fn goto_forward_skips() {
    // Forward jump skips the statements between the goto and its label.
    assert_eq!(out("<?php goto a; echo 'skip'; a: echo 'A';"), "A");
    // Jumping forward over a whole block is fine (blocks are transparent).
    assert_eq!(out("<?php goto a; if (true) { echo 'x'; } a: echo 'Y';"), "Y");
}

#[test]
fn goto_backward_loop() {
    // A label before the goto makes a back-edge: a hand-rolled loop.
    assert_eq!(out("<?php $i=0; loop: echo $i; $i++; if ($i<3) goto loop;"), "012");
    // Back-edge with the conditional jump nested inside an `if` block.
    assert_eq!(
        out("<?php $i=0; top: $i++; if ($i<3) { echo $i; goto top; } echo 'end';"),
        "12end"
    );
}

#[test]
fn goto_out_of_loop() {
    // `goto` may leave a loop (only jumping *in* is disallowed).
    assert_eq!(
        out("<?php for ($i=0;$i<5;$i++) { if ($i==2) goto done; echo $i; } done: echo '|done';"),
        "01|done"
    );
    // Out of two nested loops at once.
    assert_eq!(
        out(
            "<?php for ($i=0;$i<3;$i++) { for ($j=0;$j<3;$j++) { if ($j==1) goto out; \
             echo \"$i$j \"; } } out: echo 'OUT';"
        ),
        "00 OUT"
    );
}

#[test]
fn goto_label_fallthrough_is_noop() {
    // Reaching a label by normal fall-through does nothing.
    assert_eq!(out("<?php echo 'a'; x: echo 'b';"), "ab");
}

#[test]
fn goto_runs_finally_on_jump_out_of_try() {
    // Jumping out of a `try` body runs its `finally` first, then lands on the
    // target — the delicate case the corpus `finally_goto_*` tests exercise.
    assert_eq!(
        out("<?php try { echo 't'; goto done; } finally { echo 'f'; } echo 'X'; done: echo 'D';"),
        "tfD"
    );
}

#[test]
fn goto_back_to_label_before_try_runs_finally() {
    // Corpus `finally_goto_005`: the label is *before* the try; the goto inside
    // the try body jumps out, so `finally` runs and its `return` ends the
    // function — output is just the finally's "success".
    assert_eq!(
        out("<?php function f(){ label: try { goto label; } finally { echo 'success'; return; } } f();"),
        "success"
    );
}

#[test]
fn goto_within_finally_is_allowed() {
    // Corpus `finally_goto_003`: a goto and its label both inside the same
    // `finally` block are fine (not a jump *into* the block from outside).
    assert_eq!(
        out("<?php function f(){ try {} finally { goto t; t: } } f(); echo 'okey';"),
        "okey"
    );
}

#[test]
fn goto_into_finally_is_compile_fatal() {
    // Corpus `finally_goto_001/002/004`: jumping into a `finally` block from
    // outside it is a distinct compile-time fatal.
    for src in [
        "<?php function f(){ goto t; try {} finally { t: } } f();",
        "<?php function f(){ try { goto t; } finally { t: } } f();",
        "<?php function f(){ try {} finally { t: } goto t; } f();",
    ] {
        let r = goto_fatal(src);
        assert!(
            r.contains("jump into a finally block is disallowed"),
            "got: {r}"
        );
    }
}

#[test]
fn goto_into_transparent_block_is_scoped_out() {
    // D-45.1: jumping *into* an `if`/`try`/`catch`/plain block is valid PHP
    // (would print "x") but the tree-walker cannot land mid-block. Lowering
    // allows it (PHP-faithful: no compile fatal), and the unresolved jump is
    // surfaced as a deterministic runtime fatal rather than silent wrong output.
    let r = goto_fatal("<?php goto a; if (true) { a: echo 'x'; }");
    assert!(
        r.contains("'goto' into a block is not supported") && r.contains("D-45.1"),
        "got: {r}"
    );
}

#[test]
fn goto_is_function_scoped() {
    // A label inside a function is reachable from a goto in the same function.
    assert_eq!(
        out("<?php function f(){ goto e; echo 'no'; e: echo 'yes'; } f();"),
        "yes"
    );
}

#[test]
fn goto_undefined_label_is_compile_fatal() {
    // Undefined label is caught at compile time: no output, then the fatal.
    let r = goto_fatal("<?php echo 'X'; goto nope; echo 1;");
    assert_eq!(
        r,
        "\nFatal error: 'goto' to undefined label 'nope' in Command line code on line 1\n\
         Stack trace:\n#0 {main}\n"
    );
}

#[test]
fn goto_into_loop_is_compile_fatal() {
    let r = goto_fatal("<?php goto inside; for ($i=0;$i<3;$i++) { inside: echo $i; }");
    assert_eq!(
        r,
        "\nFatal error: 'goto' into loop or switch statement is disallowed \
         in Command line code on line 1\nStack trace:\n#0 {main}\n"
    );
}

#[test]
fn goto_into_switch_is_compile_fatal() {
    let r = goto_fatal("<?php goto inside; switch (1) { case 1: inside: echo 'x'; }");
    assert!(
        r.contains("'goto' into loop or switch statement is disallowed"),
        "got: {r}"
    );
}

#[test]
fn goto_duplicate_label_is_compile_fatal() {
    let r = goto_fatal("<?php a: echo 1; a: echo 2;");
    assert!(r.contains("Label 'a' already defined"), "got: {r}");
}

// --- print / exit / die (step 46) ------------------------------------------

#[test]
fn print_emits_and_returns_one() {
    // `print` is an expression: emits the value, evaluates to int(1).
    assert_eq!(out("<?php print 'x';"), "x");
    assert_eq!(out("<?php $x = print 'hi'; echo \"|$x\";"), "hi|1");
    // Usable mid-expression: (print "a") yields 1, then 1 + 10 = 11.
    assert_eq!(out("<?php echo (print 'a') + 10;"), "a11");
}

#[test]
fn exit_int_sets_code_no_output() {
    // An int argument is the process exit code; nothing extra is printed.
    let (out, code) = out_exit("<?php echo 'a'; exit(5); echo 'b';");
    assert_eq!(out, "a");
    assert_eq!(code, Some(5));
}

#[test]
fn exit_string_prints_and_code_zero() {
    // A string argument is a message: printed, with exit code 0.
    assert_eq!(out_exit("<?php echo 'a'; exit('msg');"), ("amsg".into(), Some(0)));
    // Even a numeric-looking string is a message, not a code.
    assert_eq!(out_exit("<?php exit('5');"), ("5".into(), Some(0)));
    // `die` is an exact alias of `exit`.
    assert_eq!(out_exit("<?php die('bye');"), ("bye".into(), Some(0)));
}

#[test]
fn exit_bare_is_code_zero() {
    assert_eq!(out_exit("<?php echo 'a'; exit; echo 'b';"), ("a".into(), Some(0)));
    assert_eq!(out_exit("<?php echo 'a'; die();"), ("a".into(), Some(0)));
}

#[test]
fn exit_code_wraps_to_byte() {
    // PHP truncates the status to a byte: 256 → 0, -1 → 255.
    assert_eq!(out_exit("<?php exit(256);").1, Some(0));
    assert_eq!(out_exit("<?php exit(-1);").1, Some(255));
    assert_eq!(out_exit("<?php exit(254);").1, Some(254));
}

#[test]
fn exit_as_expression() {
    // `exit`/`die` are expressions: `false or die(...)` runs the die.
    assert_eq!(out_exit("<?php false or die('DEAD'); echo 'after';"), ("DEAD".into(), Some(0)));
}

#[test]
fn exit_does_not_run_finally() {
    // Unlike `return`/`throw`, `exit` does NOT run `finally` (oracle-verified).
    assert_eq!(out_exit("<?php try { echo 't'; exit('X'); } finally { echo 'f'; }"), ("tX".into(), Some(0)));
    assert_eq!(out_exit("<?php try { echo 't'; exit; } finally { echo 'f'; }"), ("t".into(), Some(0)));
    // It also terminates across function boundaries.
    assert_eq!(
        out_exit("<?php function g(){ try { echo 't'; exit('Z'); } finally { echo 'f'; } } g(); echo 'after';"),
        ("tZ".into(), Some(0))
    );
}

#[test]
fn exit_is_not_catchable() {
    // A `catch (\Throwable)` never sees an `exit`.
    assert_eq!(
        out_exit("<?php try { exit('E'); } catch (\\Throwable $e) { echo 'caught'; }"),
        ("E".into(), Some(0))
    );
}

#[test]
fn no_exit_leaves_code_none() {
    // A script that runs to completion has no explicit exit code.
    assert_eq!(out_exit("<?php echo 'done';"), ("done".into(), None));
}

#[test]
fn exit_int_like_args_coerce_to_code() {
    // `string|int`: bool/float/null take the int branch (exit code), not the
    // string branch — nothing is printed.
    assert_eq!(out_exit("<?php exit(true);"), (String::new(), Some(1)));
    assert_eq!(out_exit("<?php exit(false);"), (String::new(), Some(0)));
    assert_eq!(out_exit("<?php exit(1.9);"), (String::new(), Some(1)));
    assert_eq!(out_exit("<?php exit(null);"), (String::new(), Some(0)));
}

#[test]
fn exit_stringable_object_is_message() {
    // An object with `__toString` joins the string branch: printed, code 0.
    assert_eq!(
        out_exit("<?php class S { function __toString() { return 'STR'; } } exit(new S);"),
        ("STR".into(), Some(0))
    );
}

#[test]
fn exit_non_scalar_arg_is_type_error() {
    // array / non-stringable object are outside `string|int` → TypeError
    // (a normal catchable engine error, unlike the exit itself).
    assert_eq!(
        out("<?php try { exit(new stdClass); } catch (\\TypeError $e) { echo $e->getMessage(); }"),
        "exit(): Argument #1 ($status) must be of type string|int, stdClass given"
    );
    assert_eq!(
        out("<?php try { exit([]); } catch (\\TypeError $e) { echo $e->getMessage(); }"),
        "exit(): Argument #1 ($status) must be of type string|int, array given"
    );
}

// --- var_export (step 47) --------------------------------------------------

#[test]
fn var_export_scalars() {
    assert_eq!(out("<?php echo var_export(null, true);"), "NULL");
    assert_eq!(out("<?php echo var_export(true, true);"), "true");
    assert_eq!(out("<?php echo var_export(false, true);"), "false");
    assert_eq!(out("<?php echo var_export(42, true);"), "42");
    assert_eq!(out("<?php echo var_export(-7, true);"), "-7");
}

#[test]
fn var_export_floats_always_have_point() {
    // var_export must round-trip as a float literal: 1.0, not 1.
    assert_eq!(out("<?php echo var_export(1.5, true);"), "1.5");
    assert_eq!(out("<?php echo var_export(1.0, true);"), "1.0");
    assert_eq!(out("<?php echo var_export(1e20, true);"), "1.0E+20");
    assert_eq!(out("<?php echo var_export(INF, true);"), "INF");
    assert_eq!(out("<?php echo var_export(NAN, true);"), "NAN");
}

#[test]
fn var_export_strings_single_quoted() {
    // Only `'` and `\` are escaped; other bytes are verbatim.
    assert_eq!(out("<?php echo var_export('plain', true);"), "'plain'");
    assert_eq!(out("<?php echo var_export(\"a'b\", true);"), "'a\\'b'");
    // One backslash in, two out.
    assert_eq!(out("<?php echo var_export('a\\\\b', true);"), "'a\\\\b'");
}

#[test]
fn var_export_arrays() {
    assert_eq!(out("<?php echo var_export([], true);"), "array (\n)");
    assert_eq!(
        out("<?php echo var_export([1, 2, 3], true);"),
        "array (\n  0 => 1,\n  1 => 2,\n  2 => 3,\n)"
    );
    // Numeric-string key normalises to an unquoted int; string key is quoted.
    assert_eq!(
        out("<?php echo var_export(['7' => 'a', 'x' => 'b'], true);"),
        "array (\n  7 => 'a',\n  'x' => 'b',\n)"
    );
}

#[test]
fn var_export_nested_array() {
    // A nested array's value goes on a new line, indented one level deeper.
    assert_eq!(
        out("<?php echo var_export(['a' => 1, 'b' => [2, 3]], true);"),
        "array (\n  'a' => 1,\n  'b' => \n  array (\n    0 => 2,\n    1 => 3,\n  ),\n)"
    );
}

#[test]
fn var_export_stdclass() {
    // stdClass renders as a `(object) array(...)` cast, members at 3 spaces.
    assert_eq!(
        out("<?php $o = new stdClass; $o->x = 1; $o->y = 'z'; echo var_export($o, true);"),
        "(object) array(\n   'x' => 1,\n   'y' => 'z',\n)"
    );
}

#[test]
fn var_export_user_object() {
    // A user class renders via `__set_state`; all props by value, no markers.
    assert_eq!(
        out("<?php class P { public $a = 1; protected $b = 2; } echo var_export(new P, true);"),
        "\\P::__set_state(array(\n   'a' => 1,\n   'b' => 2,\n))"
    );
}

#[test]
fn var_export_nul_byte_string() {
    // A NUL byte can't live in a single-quoted literal: PHP concatenates a
    // double-quoted `"\0"` between the single-quoted segments.
    assert_eq!(
        out("<?php echo var_export(\"\\0Hi\\0\", true);"),
        "'' . \"\\0\" . 'Hi' . \"\\0\" . ''"
    );
}

#[test]
fn var_export_print_mode() {
    // Without the return flag, var_export writes straight to stdout.
    assert_eq!(out("<?php var_export([1]);"), "array (\n  0 => 1,\n)");
}

// --- get_class_methods / get_object_vars (step 47) -------------------------

#[test]
fn get_class_methods_public_from_global() {
    // From global scope only public methods, child→parent order, by name or obj.
    let src = "<?php class A { public function a1(){} private function ap(){} \
               protected function aq(){} } class B extends A { public function b1(){} } ";
    assert_eq!(out(&format!("{src} echo implode(',', get_class_methods('B'));")), "b1,a1");
    assert_eq!(out(&format!("{src} echo implode(',', get_class_methods(new B));")), "b1,a1");
}

#[test]
fn get_class_methods_sees_private_from_inside() {
    // Called from within the class, private/protected are included too.
    assert_eq!(
        out("<?php class A { public function a1(){} private function ap(){} \
             function d(){ return implode(',', get_class_methods($this)); } } echo (new A)->d();"),
        "a1,ap,d"
    );
}

#[test]
fn get_object_vars_public_from_global() {
    assert_eq!(
        out("<?php class A { public $p = 1; protected $q = 2; private $r = 3; public $z = 4; } \
             $o = new A; $s = ''; foreach (get_object_vars($o) as $k => $v) { $s .= \"$k=$v;\"; } echo $s;"),
        "p=1;z=4;"
    );
}

#[test]
fn get_object_vars_all_from_inside() {
    assert_eq!(
        out("<?php class A { public $p = 1; protected $q = 2; private $r = 3; \
             function d(){ $s = ''; foreach (get_object_vars($this) as $k => $v) { $s .= \"$k=$v;\"; } return $s; } } \
             echo (new A)->d();"),
        "p=1;q=2;r=3;"
    );
}

// --- dynamic class references (step 48) ------------------------------------

#[test]
fn dynamic_new_from_string() {
    assert_eq!(
        out("<?php class Foo { public $x = 5; } $c = 'Foo'; $o = new $c(); echo $o->x;"),
        "5"
    );
    // No parentheses form, and a leading namespace separator is stripped.
    assert_eq!(
        out("<?php class Foo { public $x = 1; } $c = '\\\\Foo'; $o = new $c; echo $o->x;"),
        "1"
    );
}

#[test]
fn dynamic_new_from_object() {
    // `new $obj` instantiates the object's class.
    assert_eq!(
        out("<?php class Foo { public $x = 3; } $a = new Foo; $b = new $a; echo $b->x;"),
        "3"
    );
}

#[test]
fn dynamic_static_const_method_prop() {
    assert_eq!(
        out("<?php class Foo { const K = 9; static function m(){ return 'M'; } } \
             $c = 'Foo'; echo $c::K, $c::m();"),
        "9M"
    );
    assert_eq!(
        out("<?php class Foo { static $s = 7; } $c = 'Foo'; echo $c::$s;"),
        "7"
    );
}

#[test]
fn dynamic_static_call_via_object() {
    assert_eq!(
        out("<?php class Foo { static function m(){ return 'ok'; } } $o = new Foo; echo $o::m();"),
        "ok"
    );
}

#[test]
fn dynamic_instanceof() {
    assert_eq!(
        out("<?php class Foo {} $c = 'Foo'; $o = new Foo; var_dump($o instanceof $c);"),
        "bool(true)\n"
    );
    assert_eq!(
        out("<?php class A {} class B extends A {} $c = 'A'; var_dump(new B instanceof $c);"),
        "bool(true)\n"
    );
}

// --- @ error-control operator (step 48) ------------------------------------

/// Run a script and return its CLI-rendered stream (output with diagnostics
/// interleaved) — for asserting that `@` suppresses a warning.
fn rendered(src: &str) -> String {
    let reg = registry();
    let o = run_source_with(b"t.php", src.as_bytes(), &reg).expect("lowers");
    String::from_utf8(o.rendered).expect("utf8")
}

#[test]
fn error_suppression_silences_warning() {
    // `@$x` yields NULL and the "Undefined variable" warning is suppressed.
    assert_eq!(out("<?php var_dump(@$x);"), "NULL\n");
    let r = rendered("<?php var_dump(@$x);");
    assert!(!r.contains("Undefined variable"), "warning leaked: {r}");
    // Control: without `@`, the warning is rendered.
    assert!(rendered("<?php var_dump($x);").contains("Undefined variable"));
}

#[test]
fn error_suppression_on_array_key() {
    assert_eq!(out("<?php $a = []; var_dump(@$a['k']);"), "NULL\n");
    assert!(!rendered("<?php $a = []; var_dump(@$a['k']);").contains("Undefined"));
}

#[test]
fn error_suppression_does_not_swallow_throwable() {
    // `@` silences warnings, not engine errors: a DivisionByZeroError still
    // propagates (caught here to observe it fired).
    assert_eq!(
        out("<?php try { echo @(1 % 0); } catch (\\DivisionByZeroError $e) { echo 'caught'; }"),
        "caught"
    );
}

#[test]
fn dynamic_new_unknown_class_is_fatal() {
    // An unresolved dynamic class name is a runtime fatal, no output.
    let reg = registry();
    let o = run_source_with(b"t.php", b"<?php echo 'a'; $c = 'Nope'; new $c();", &reg).expect("lowers");
    assert_eq!(o.stdout, b"a");
    assert!(
        String::from_utf8_lossy(&o.rendered).contains("Class \"Nope\" not found"),
        "got: {}",
        String::from_utf8_lossy(&o.rendered)
    );
}

// ---- step 50a: serialize() (verified byte-exact against the PHP 8.5 oracle) ----

#[test]
fn serialize_scalars() {
    assert_eq!(out("<?php echo serialize(null);"), "N;");
    assert_eq!(out("<?php echo serialize(true);"), "b:1;");
    assert_eq!(out("<?php echo serialize(false);"), "b:0;");
    assert_eq!(out("<?php echo serialize(42);"), "i:42;");
    assert_eq!(out("<?php echo serialize(-7);"), "i:-7;");
    assert_eq!(out("<?php echo serialize(3.14);"), "d:3.14;");
    // serialize_precision = -1: 1.0 has no fractional part in the shortest repr.
    assert_eq!(out("<?php echo serialize(1.0);"), "d:1;");
    assert_eq!(out("<?php echo serialize(2.5);"), "d:2.5;");
}

#[test]
fn serialize_strings_byte_length() {
    assert_eq!(out("<?php echo serialize('hello');"), "s:5:\"hello\";");
    assert_eq!(out("<?php echo serialize('');"), "s:0:\"\";");
    // Multibyte: byte length, not codepoint count ('é' is 2 bytes).
    assert_eq!(out("<?php echo serialize('héllo');"), "s:6:\"héllo\";");
}

#[test]
fn serialize_arrays_nested_and_ordered() {
    assert_eq!(
        out("<?php echo serialize([1,2,3]);"),
        "a:3:{i:0;i:1;i:1;i:2;i:2;i:3;}"
    );
    assert_eq!(
        out("<?php echo serialize([1,'a'=>2.5,3=>[true,null]]);"),
        "a:3:{i:0;i:1;s:1:\"a\";d:2.5;i:3;a:2:{i:0;b:1;i:1;N;}}"
    );
}

#[test]
fn serialize_object_stdclass() {
    assert_eq!(
        out("<?php $o=new stdClass; $o->x=1; $o->y='z'; echo serialize($o);"),
        "O:8:\"stdClass\":2:{s:1:\"x\";i:1;s:1:\"y\";s:1:\"z\";}"
    );
}

// ---- step 50b: unserialize() (round-trips verified against the PHP 8.5 oracle) ----

/// serialize(unserialize(S)) == S for every canonical serialized form.
#[test]
fn unserialize_roundtrips_canonical() {
    for s in [
        "i:42;",
        "i:-7;",
        "d:3.14;",
        "d:1;",
        "b:1;",
        "b:0;",
        "N;",
        "s:5:\"hello\";",
        "s:0:\"\";",
        "a:3:{i:0;i:1;i:1;i:2;i:2;i:3;}",
        "a:2:{s:1:\"a\";i:1;s:1:\"b\";i:2;}",
        "a:1:{i:0;a:2:{i:0;b:1;i:1;N;}}",
        "O:8:\"stdClass\":2:{s:1:\"x\";i:1;s:1:\"y\";s:1:\"z\";}",
    ] {
        let src = format!("<?php echo serialize(unserialize('{s}'));");
        assert_eq!(out(&src), s, "round-trip mismatch for {s}");
    }
}

#[test]
fn unserialize_values_observable() {
    assert_eq!(out("<?php echo unserialize('i:42;') + 1;"), "43");
    assert_eq!(out("<?php $a = unserialize('a:2:{i:0;i:10;i:1;i:20;}'); echo $a[0]+$a[1];"), "30");
    assert_eq!(out("<?php $o = unserialize('O:8:\"stdClass\":1:{s:1:\"n\";i:5;}'); echo $o->n;"), "5");
}

#[test]
fn unserialize_malformed_is_false() {
    assert_eq!(out("<?php echo unserialize('z') === false ? 'F' : 'T';"), "F");
    assert_eq!(out("<?php echo unserialize('') === false ? 'F' : 'T';"), "F");
    // Trailing garbage after a valid value is rejected too.
    assert_eq!(out("<?php echo unserialize('i:1;XX') === false ? 'F' : 'T';"), "F");
}

/// Round-trip a freshly serialized value through unserialize and back.
#[test]
fn serialize_unserialize_roundtrip_values() {
    let prog = "<?php echo serialize(unserialize(serialize([1, 'a' => 2.5, 3 => [true, null]])));";
    assert_eq!(out(prog), "a:3:{i:0;i:1;s:1:\"a\";d:2.5;i:3;a:2:{i:0;b:1;i:1;N;}}");
}

// ---- step 51a: fopen / fread / fwrite / fclose on real files + Resource type ----

/// A unique path under the system temp dir for a single test (avoids collisions
/// when tests run in parallel). Cleaned up by the test after use.
fn tmp_path(tag: &str) -> String {
    let mut p = std::env::temp_dir();
    p.push(format!("phpr_51a_{tag}"));
    p.to_string_lossy().into_owned()
}

#[test]
fn fopen_write_read_roundtrip() {
    let p = tmp_path("roundtrip");
    let _ = std::fs::remove_file(&p);
    let src = format!(
        "<?php $w=fopen('{p}','w'); echo fwrite($w,'hello world'); fclose($w); \
         $r=fopen('{p}','r'); echo '|',fread($r,5),'|',fread($r,100); fclose($r);"
    );
    assert_eq!(out(&src), "11|hello| world");
    let _ = std::fs::remove_file(&p);
}

#[test]
fn resource_var_dump_echo_and_casts() {
    let p = tmp_path("dump");
    let _ = std::fs::remove_file(&p);
    let src = format!(
        "<?php $f=fopen('{p}','w'); var_dump($f); echo $f,\"\\n\"; \
         echo gettype($f),\"\\n\"; echo (int)$f,' ',(bool)$f?'1':'0',\"\\n\"; fclose($f);"
    );
    assert_eq!(
        out(&src),
        "resource(5) of type (stream)\nResource id #5\nresource\n5 1\n"
    );
    let _ = std::fs::remove_file(&p);
}

#[test]
fn resource_closed_observable() {
    let p = tmp_path("closed");
    let _ = std::fs::remove_file(&p);
    let src = format!(
        "<?php $f=fopen('{p}','w'); fclose($f); var_dump($f); echo gettype($f);"
    );
    assert_eq!(out(&src), "resource(5) of type (Unknown)\nresource (closed)");
    let _ = std::fs::remove_file(&p);
}

#[test]
fn resource_identity_and_alias() {
    let p = tmp_path("ident");
    let _ = std::fs::remove_file(&p);
    // $g aliases $f (shared handle): closing $g closes $f.
    let src = format!(
        "<?php $f=fopen('{p}','w'); $g=$f; var_dump($f===$f,$f===$g); \
         fclose($g); echo gettype($f);"
    );
    assert_eq!(out(&src), "bool(true)\nbool(true)\nresource (closed)");
    let _ = std::fs::remove_file(&p);
}

#[test]
fn resource_compare_by_id() {
    let (a, b) = (tmp_path("cmpA"), tmp_path("cmpB"));
    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
    let src = format!(
        "<?php $x=fopen('{a}','w'); $y=fopen('{b}','w'); \
         var_dump($x<$y,$x==$x,$x==$y); fclose($x); fclose($y);"
    );
    assert_eq!(out(&src), "bool(true)\nbool(true)\nbool(false)\n");
    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
}

#[test]
fn fopen_missing_file_is_false() {
    // Suppressed with @ so the warning does not interleave into stdout.
    assert_eq!(
        out("<?php echo @fopen('/no_such_dir_phpr/x','r') === false ? 'F' : 'T';"),
        "F"
    );
}

#[test]
fn fputs_is_fwrite_alias() {
    let p = tmp_path("fputs");
    let _ = std::fs::remove_file(&p);
    let src = format!(
        "<?php $f=fopen('{p}','w'); echo fputs($f,'abc'); fclose($f); \
         $r=fopen('{p}','r'); echo fread($r,3); fclose($r);"
    );
    assert_eq!(out(&src), "3abc");
    let _ = std::fs::remove_file(&p);
}

#[test]
fn fopen_c_mode_keeps_content_pos_zero() {
    let p = tmp_path("cmode");
    let _ = std::fs::write(&p, "ABCD");
    // `c` opens without truncating, position 0 → write overwrites the head.
    // (Read back via fread; file_get_contents is step 51c.)
    let src = format!(
        "<?php $f=fopen('{p}','c'); fwrite($f,'X'); fclose($f); \
         $r=fopen('{p}','r'); echo fread($r,10); fclose($r);"
    );
    assert_eq!(out(&src), "XBCD");
    let _ = std::fs::remove_file(&p);
}

#[test]
fn serialize_resource_is_int_zero() {
    let p = tmp_path("ser");
    let _ = std::fs::remove_file(&p);
    let src = format!("<?php $f=fopen('{p}','w'); echo serialize($f); fclose($f);");
    assert_eq!(out(&src), "i:0;");
    let _ = std::fs::remove_file(&p);
}

// ---- step 51b: fgets/fgetc/feof/fseek/ftell/rewind/fflush + php:// wrappers ----

#[test]
fn php_memory_roundtrip_ftell_rewind() {
    let src = "<?php $m=fopen('php://memory','w+'); echo fwrite($m,'abcdef'),';',ftell($m),';'; \
               rewind($m); echo fgetc($m),fgetc($m),';',ftell($m),';'; echo fgets($m); \
               var_dump(feof($m)); echo fread($m,100); var_dump(feof($m));";
    assert_eq!(out(src), "6;6;ab;2;cdefbool(true)\nbool(true)\n");
}

#[test]
fn fseek_whence_and_constants() {
    let src = "<?php $m=fopen('php://memory','w+'); fwrite($m,'0123456789'); \
               fseek($m,2); echo fread($m,2),';'; fseek($m,-3,SEEK_END); echo fread($m,3),';'; \
               fseek($m,1,SEEK_CUR); echo ftell($m),';',SEEK_SET,SEEK_CUR,SEEK_END;";
    assert_eq!(out(src), "23;789;11;012");
}

#[test]
fn fgets_length_cap() {
    // fgets($f, 4) reads at most 4-1 = 3 bytes.
    let src = "<?php $m=fopen('php://memory','w+'); fwrite($m,\"abcdef\\nghi\"); rewind($m); \
               var_dump(fgets($m,4));";
    assert_eq!(out(src), "string(3) \"abc\"\n");
}

#[test]
fn fgets_stops_at_newline() {
    let src = "<?php $m=fopen('php://memory','w+'); fwrite($m,\"l1\\nl2\\nl3\"); rewind($m); \
               echo fgets($m); echo '|'; echo fgets($m); echo '|'; echo fgets($m); \
               var_dump(fgets($m));";
    assert_eq!(out(src), "l1\n|l2\n|l3bool(false)\n");
}

#[test]
fn php_memory_read_mode_not_writable() {
    // php://memory opened "r" honours the mode: not writable, empty to read.
    let src = "<?php $m=fopen('php://memory','r'); var_dump(@fwrite($m,'x')); var_dump(fread($m,5));";
    assert_eq!(out(src), "bool(false)\nstring(0) \"\"\n");
}

#[test]
fn php_stdout_writes_to_output() {
    let src = "<?php $o=fopen('php://stdout','w'); fwrite($o,'OUT'); fclose($o);";
    assert_eq!(out(src), "OUT");
}

#[test]
fn feof_on_closed_is_type_error() {
    let reg = registry();
    let o = run_source_with(
        b"t.php",
        b"<?php $m=fopen('php://memory','r'); fclose($m); feof($m);",
        &reg,
    )
    .expect("lowers");
    let fatal = o.fatal.expect("expected a fatal TypeError");
    assert_eq!(fatal.class_name(), "TypeError");
    assert_eq!(
        fatal.message(),
        "feof(): Argument #1 ($stream) must be an open stream resource"
    );
}

#[test]
fn unsupported_wrapper_is_false() {
    assert_eq!(
        out("<?php echo @fopen('http://example.com/','r') === false ? 'F' : 'T';"),
        "F"
    );
}

// ---- step 51c: file_get_contents / file_put_contents (pure builtins) ----

#[test]
fn file_put_get_contents_roundtrip() {
    let p = tmp_path("fgc_round");
    let _ = std::fs::remove_file(&p);
    let src = format!(
        "<?php var_dump(file_put_contents('{p}','hello')); var_dump(file_get_contents('{p}'));"
    );
    assert_eq!(out(&src), "int(5)\nstring(5) \"hello\"\n");
    let _ = std::fs::remove_file(&p);
}

#[test]
fn file_put_contents_append_and_array() {
    let p = tmp_path("fgc_append");
    let _ = std::fs::remove_file(&p);
    let src = format!(
        "<?php file_put_contents('{p}','A'); file_put_contents('{p}','B',FILE_APPEND); \
         echo file_get_contents('{p}'); echo ';'; \
         var_dump(file_put_contents('{p}',['x','y','z'])); echo file_get_contents('{p}');"
    );
    assert_eq!(out(&src), "AB;int(3)\nxyz");
    let _ = std::fs::remove_file(&p);
}

#[test]
fn file_get_contents_offset_length() {
    let p = tmp_path("fgc_ol");
    let _ = std::fs::write(&p, "0123456789");
    let src = format!("<?php var_dump(file_get_contents('{p}',false,null,3,4));");
    assert_eq!(out(&src), "string(4) \"3456\"\n");
    let _ = std::fs::remove_file(&p);
}

#[test]
fn file_get_contents_missing_is_false() {
    assert_eq!(
        out("<?php var_dump(@file_get_contents('/no_dir_phpr_zz/x'));"),
        "bool(false)\n"
    );
}

#[test]
fn file_put_contents_from_stream_resource() {
    let (src_p, dst_p) = (tmp_path("fgc_src"), tmp_path("fgc_dst"));
    let _ = std::fs::write(&src_p, "copied!");
    let _ = std::fs::remove_file(&dst_p);
    let prog = format!(
        "<?php $r=fopen('{src_p}','r'); var_dump(file_put_contents('{dst_p}',$r)); fclose($r); \
         echo file_get_contents('{dst_p}');"
    );
    assert_eq!(out(&prog), "int(7)\ncopied!");
    let _ = std::fs::remove_file(&src_p);
    let _ = std::fs::remove_file(&dst_p);
}

#[test]
fn fwrite_length_clamps() {
    let p = tmp_path("fw_len");
    let _ = std::fs::remove_file(&p);
    // Negative length writes 0 bytes; over-large writes everything (oracle).
    let src = format!(
        "<?php $f=fopen('{p}','w'); var_dump(fwrite($f,'data',-1)); \
         var_dump(fwrite($f,'data',100000)); fclose($f); \
         $r=fopen('{p}','r'); echo fread($r,10); fclose($r);"
    );
    assert_eq!(out(&src), "int(0)\nint(4)\ndata");
    let _ = std::fs::remove_file(&p);
}

// ---- step 52a: basename / dirname / pathinfo (path-string, pure) ----

#[test]
fn basename_cases() {
    assert_eq!(out("<?php echo basename('/a/b/c.php');"), "c.php");
    assert_eq!(out("<?php echo basename('/a/b/');"), "b");
    assert_eq!(out("<?php echo '['.basename('/').']';"), "[]");
    assert_eq!(out("<?php echo basename('a//b//c');"), "c");
    assert_eq!(out("<?php echo basename('/x/.hidden');"), ".hidden");
    assert_eq!(out("<?php echo basename('a.tar.gz','.gz');"), "a.tar");
    assert_eq!(out("<?php echo basename('test.php','.php');"), "test");
    // Suffix equal to the whole basename is not stripped.
    assert_eq!(out("<?php echo basename('.php','.php');"), ".php");
}

#[test]
fn dirname_cases() {
    assert_eq!(out("<?php echo dirname('/a/b/c');"), "/a/b");
    assert_eq!(out("<?php echo dirname('/a/b/');"), "/a");
    assert_eq!(out("<?php echo dirname('c');"), ".");
    assert_eq!(out("<?php echo dirname('/');"), "/");
    assert_eq!(out("<?php echo dirname('a//b');"), "a");
    assert_eq!(out("<?php echo dirname('/a/b/c/');"), "/a/b");
    assert_eq!(out("<?php echo dirname('/a/b/c/d',2);"), "/a/b");
    assert_eq!(out("<?php echo dirname('/a/b/c/d',3);"), "/a");
}

#[test]
fn pathinfo_array_and_flags() {
    assert_eq!(
        out("<?php $i=pathinfo('/a/b/file.tar.gz'); echo $i['dirname'],'|',$i['basename'],'|',$i['extension'],'|',$i['filename'];"),
        "/a/b|file.tar.gz|gz|file.tar"
    );
    assert_eq!(out("<?php echo pathinfo('/a/b/c.txt', PATHINFO_EXTENSION);"), "txt");
    assert_eq!(out("<?php echo pathinfo('/a/b/c.txt', PATHINFO_FILENAME);"), "c");
    // No extension → no 'extension' key, filename == basename.
    assert_eq!(
        out("<?php $i=pathinfo('noext'); echo $i['dirname'],'|',$i['basename'],'|',$i['filename'],'|',isset($i['extension'])?'Y':'N';"),
        ".|noext|noext|N"
    );
    // Leading-dot file: extension is the tail, filename empty.
    assert_eq!(
        out("<?php $i=pathinfo('/dir/.hidden'); echo $i['extension'],'|','['.$i['filename'].']';"),
        "hidden|[]"
    );
}

// ---- step 52b: existence/type predicates + realpath + cwd ----

#[test]
fn existence_and_type_predicates() {
    use std::os::unix::fs::symlink;
    let dir = tmp_path("b52_dir");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file = format!("{dir}/f.txt");
    std::fs::write(&file, "hi").unwrap();
    let link = format!("{dir}/sl");
    let _ = symlink(&file, &link);
    let broken = format!("{dir}/broken");
    let _ = symlink(format!("{dir}/nope"), &broken);
    let missing = format!("{dir}/none");

    // `echo` of a bool false is "", so map every predicate through a ternary.
    let src = format!(
        "<?php $b=fn($x)=>$x?'1':'0'; \
         echo $b(file_exists('{file}')),$b(is_file('{file}')),$b(is_dir('{file}')),$b(is_link('{file}')),'|'; \
         echo $b(file_exists('{dir}')),$b(is_file('{dir}')),$b(is_dir('{dir}')),'|'; \
         echo $b(file_exists('{link}')),$b(is_file('{link}')),$b(is_link('{link}')),'|'; \
         echo $b(file_exists('{broken}')),$b(is_link('{broken}')),'|'; \
         echo $b(file_exists('{missing}'));"
    );
    // file: ex,isf,!isd,!isl | dir: ex,!isf,isd | link→file: ex,isf,isl |
    // broken: !ex,isl | missing: !ex
    assert_eq!(out(&src), "1100|101|111|01|0");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn filetype_kinds() {
    use std::os::unix::fs::symlink;
    let dir = tmp_path("b52_ft");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file = format!("{dir}/f");
    std::fs::write(&file, "x").unwrap();
    let link = format!("{dir}/l");
    let _ = symlink(&file, &link);
    let src = format!(
        "<?php echo filetype('{file}'),'|',filetype('{dir}'),'|',filetype('{link}'),'|'; \
         var_dump(@filetype('{dir}/none'));"
    );
    assert_eq!(out(&src), "file|dir|link|bool(false)\n");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn realpath_and_cwd() {
    let dir = tmp_path("b52_rp");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file = format!("{dir}/f.txt");
    std::fs::write(&file, "x").unwrap();
    // realpath of a missing path is false; of an existing file, basename matches.
    let src = format!(
        "<?php var_dump(realpath('{dir}/none')); echo basename(realpath('{file}')); \
         echo '|', strlen(sys_get_temp_dir())>0?'T':'F'; \
         echo '|', substr(sys_get_temp_dir(),-1)==='/'?'slash':'noslash'; \
         echo '|', strlen(getcwd())>0?'T':'F';"
    );
    assert_eq!(out(&src), "bool(false)\nf.txt|T|noslash|T");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn access_predicates_respect_mode() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tmp_path("b52_acc");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let plain = format!("{dir}/plain");
    std::fs::write(&plain, "x").unwrap();
    std::fs::set_permissions(&plain, std::fs::Permissions::from_mode(0o644)).unwrap();
    let exe = format!("{dir}/exe");
    std::fs::write(&exe, "x").unwrap();
    std::fs::set_permissions(&exe, std::fs::Permissions::from_mode(0o755)).unwrap();
    // 0644: readable + writable, not executable. 0755: executable.
    // `access(X_OK)` on a file with no exec bits fails even for root, so these
    // assertions hold regardless of the test user's uid.
    let src = format!(
        "<?php $b=fn($x)=>$x?'1':'0'; \
         echo $b(is_readable('{plain}')),$b(is_writable('{plain}')),$b(is_executable('{plain}')),'|'; \
         echo $b(is_executable('{exe}')),'|'; \
         echo $b(is_readable('{dir}/none')),$b(is_writable('{dir}/none'));"
    );
    assert_eq!(out(&src), "110|1|00");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn clearstatcache_is_noop_null() {
    assert_eq!(
        out("<?php var_dump(clearstatcache()); var_dump(clearstatcache(true, '/x'));"),
        "NULL\nNULL\n"
    );
}

#[test]
fn stat_array_shape_and_fields() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tmp_path("b52_stat");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file = format!("{dir}/f");
    std::fs::write(&file, "hello\n").unwrap(); // 6 bytes
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
    // 26 keys; integer key 7 aliases 'size'; regular 0644 file → mode 33188 (0100644).
    let src = format!(
        "<?php $s=stat('{file}'); \
         echo count($s),'|',$s[7]===$s['size']?'Y':'N','|',$s['size'],'|',$s['mode'],'|'; \
         echo filesize('{file}'),'|',is_int(filemtime('{file}'))?'I':'X','|'; \
         printf('%o', fileperms('{file}')); echo '|'; \
         var_dump(@stat('{dir}/none'));"
    );
    assert_eq!(out(&src), "26|Y|6|33188|6|I|100644|bool(false)\n");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn lstat_vs_stat_on_symlink() {
    use std::os::unix::fs::symlink;
    let dir = tmp_path("b52_lstat");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file = format!("{dir}/f");
    std::fs::write(&file, "abc").unwrap();
    let link = format!("{dir}/l");
    let _ = symlink(&file, &link);
    // stat follows → regular-file type bits (0100000); lstat → symlink (0120000).
    let src = format!(
        "<?php echo (stat('{link}')['mode'] & 0xF000)===0x8000?'reg':'?','|'; \
         echo (lstat('{link}')['mode'] & 0xF000)===0xA000?'lnk':'?';"
    );
    assert_eq!(out(&src), "reg|lnk");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fstat_on_file_and_memory() {
    let dir = tmp_path("b52_fstat");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file = format!("{dir}/f");
    std::fs::write(&file, "hello\n").unwrap();
    let src = format!(
        "<?php $f=fopen('{file}','r'); $s=fstat($f); echo count($s),'|',$s['size'],'|'; fclose($f); \
         $m=fopen('php://memory','r+'); fwrite($m,'abcd'); $t=fstat($m); \
         echo count($t),'|',$t['size'],'|',$t['mode']; fclose($m);"
    );
    assert_eq!(out(&src), "26|6|26|4|33206");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn mutators_roundtrip() {
    let dir = tmp_path("b52_mut");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let src = format!(
        "<?php $b=fn($x)=>$x?'1':'0'; \
         echo $b(mkdir('{dir}/a')),$b(mkdir('{dir}/x/y/z',0777,true)),$b(is_dir('{dir}/x/y/z')),'|'; \
         file_put_contents('{dir}/f','hello'); \
         echo $b(touch('{dir}/f')),$b(touch('{dir}/new')),$b(is_file('{dir}/new')),'|'; \
         echo $b(copy('{dir}/f','{dir}/f2')),filesize('{dir}/f2'),'|'; \
         echo $b(rename('{dir}/f2','{dir}/f3')),$b(file_exists('{dir}/f2')),$b(file_exists('{dir}/f3')),'|'; \
         chmod('{dir}/f',0600); printf('%o', fileperms('{dir}/f')&0777); echo '|'; \
         symlink('{dir}/f','{dir}/sl'); echo basename(readlink('{dir}/sl')),'|'; \
         echo $b(unlink('{dir}/f')),$b(rmdir('{dir}/a'));"
    );
    assert_eq!(out(&src), "111|111|15|101|600|f|11");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn touch_sets_explicit_mtime() {
    let dir = tmp_path("b52_touch");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let src = format!(
        "<?php touch('{dir}/t', 1000000000); echo filemtime('{dir}/t');"
    );
    assert_eq!(out(&src), "1000000000");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn csv_str_getcsv() {
    assert_eq!(out("<?php echo json_encode(str_getcsv('a,b,c'));"), "[\"a\",\"b\",\"c\"]");
    assert_eq!(out("<?php echo json_encode(str_getcsv('\"a,b\",c'));"), "[\"a,b\",\"c\"]");
    assert_eq!(out("<?php echo json_encode(str_getcsv('\"x\"\"y\"'));"), "[\"x\\\"y\"]");
    assert_eq!(out("<?php echo json_encode(str_getcsv('a,,c'));"), "[\"a\",\"\",\"c\"]");
    assert_eq!(out("<?php echo json_encode(str_getcsv(''));"), "[null]");
    assert_eq!(out("<?php echo json_encode(str_getcsv('a;b', ';'));"), "[\"a\",\"b\"]");
}

#[test]
fn csv_escape_deprecation() {
    let (_, w) = out_diags("<?php str_getcsv('a,b');");
    assert_eq!(
        w,
        vec!["str_getcsv(): the $escape parameter must be provided as its default value will change"]
    );
    // Passing $escape explicitly suppresses it.
    let (_, w2) = out_diags("<?php str_getcsv('a,b', ',', '\"', '\\\\');");
    assert!(w2.is_empty(), "unexpected diags: {w2:?}");
}

#[test]
fn csv_fputcsv_fgetcsv_roundtrip() {
    // fputcsv quotes space/comma fields, returns byte count.
    let put = "<?php $m=fopen('php://memory','r+'); \
        $n=fputcsv($m,['a','b c','d,e'],',','\"','\\\\'); rewind($m); echo $n,'|',fread($m,100);";
    assert_eq!(out(put), "14|a,\"b c\",\"d,e\"\n");
    // Roundtrip through fgetcsv; second read is EOF → false.
    let trip = "<?php $m=fopen('php://memory','r+'); \
        fputcsv($m,['a','b,c','x y'],',','\"','\\\\'); rewind($m); \
        echo json_encode(fgetcsv($m,0,',','\"','\\\\')),'|'; \
        var_dump(fgetcsv($m,0,',','\"','\\\\'));";
    assert_eq!(out(trip), "[\"a\",\"b,c\",\"x y\"]|bool(false)\n");
}

#[test]
fn fscanf_lines_and_eof() {
    let dir = tmp_path("b54_fscanf");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let p = format!("{dir}/n.txt");
    std::fs::write(&p, "10 20\n30 40\n").unwrap();
    // Loop terminates on EOF (fscanf → false); by-ref mode on the last line.
    let src = format!(
        "<?php $f=fopen('{p}','r'); \
         while(($r=fscanf($f,'%d %d'))!==false){{ echo json_encode($r),'|'; }} \
         rewind($f); $n=fscanf($f,'%d %d',$a,$b); echo \"$n:$a:$b\"; fclose($f);"
    );
    assert_eq!(out(&src), "[10,20]|[30,40]|2:10:20");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn file_lines_and_flags() {
    let dir = tmp_path("b55_file");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let p = format!("{dir}/f.txt");
    std::fs::write(&p, "line1\nline2\n\nline4\n").unwrap();
    // Default keeps newlines (4 entries incl. the bare "\n").
    assert_eq!(
        out(&format!("<?php echo json_encode(file('{p}'));")),
        "[\"line1\\n\",\"line2\\n\",\"\\n\",\"line4\\n\"]"
    );
    // FILE_IGNORE_NEW_LINES strips them; the empty line stays as "".
    assert_eq!(
        out(&format!("<?php echo json_encode(file('{p}', FILE_IGNORE_NEW_LINES));")),
        "[\"line1\",\"line2\",\"\",\"line4\"]"
    );
    // + FILE_SKIP_EMPTY_LINES drops the empty line.
    assert_eq!(
        out(&format!(
            "<?php echo json_encode(file('{p}', FILE_IGNORE_NEW_LINES|FILE_SKIP_EMPTY_LINES));"
        )),
        "[\"line1\",\"line2\",\"line4\"]"
    );
    assert_eq!(out(&format!("<?php var_dump(@file('{dir}/none'));")), "bool(false)\n");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn readfile_and_fpassthru() {
    let dir = tmp_path("b55_rf");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let p = format!("{dir}/f.txt");
    std::fs::write(&p, "line1\nline2\nline3\n").unwrap();
    // readfile echoes the whole file and returns the byte count.
    assert_eq!(
        out(&format!("<?php $n=readfile('{p}'); echo '|',$n;")),
        "line1\nline2\nline3\n|18"
    );
    // fpassthru echoes the rest of the stream after one fgets.
    assert_eq!(
        out(&format!("<?php $f=fopen('{p}','r'); fgets($f); $n=fpassthru($f); echo '|',$n; fclose($f);")),
        "line2\nline3\n|12"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn stream_get_contents_copy_truncate() {
    let dir = tmp_path("b55_stream");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let p = format!("{dir}/f.txt");
    std::fs::write(&p, "line1\nline2\n\nline4\n").unwrap(); // 19 bytes
    // stream_get_contents reads the rest after one fgets; and maxlength caps it.
    assert_eq!(
        out(&format!(
            "<?php $f=fopen('{p}','r'); fgets($f); echo '[',stream_get_contents($f),']'; \
             rewind($f); echo '|',stream_get_contents($f,5); fclose($f);"
        )),
        "[line2\n\nline4\n]|line1"
    );
    // stream_copy_to_stream copies the whole source into a memory stream.
    assert_eq!(
        out(&format!(
            "<?php $s=fopen('{p}','r'); $d=fopen('php://memory','r+'); \
             $n=stream_copy_to_stream($s,$d); rewind($d); \
             echo $n,'|',stream_get_contents($d); fclose($s); fclose($d);"
        )),
        "19|line1\nline2\n\nline4\n"
    );
    // ftruncate shrinks the file; filesize reflects it.
    let tp = format!("{dir}/t.txt");
    std::fs::write(&tp, "hello world").unwrap();
    assert_eq!(
        out(&format!(
            "<?php $f=fopen('{tp}','r+'); var_dump(ftruncate($f,4)); fclose($f); \
             echo file_get_contents('{tp}'),'|',filesize('{tp}');"
        )),
        "bool(true)\nhell|4"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn sscanf_array_mode() {
    // Mixed conversions, types preserved.
    assert_eq!(
        out("<?php var_dump(sscanf('age:42 pi:3.14 name:bob','age:%d pi:%f name:%s'));"),
        "array(3) {\n  [0]=>\n  int(42)\n  [1]=>\n  float(3.14)\n  [2]=>\n  string(3) \"bob\"\n}\n"
    );
    // json_encode for compact checks (NULL → null).
    assert_eq!(out("<?php echo json_encode(sscanf('ab12cd','%2s%2d%2s'));"), "[\"ab\",12,\"cd\"]");
    assert_eq!(out("<?php echo json_encode(sscanf('abc123','%[a-z]%[0-9]'));"), "[\"abc\",\"123\"]");
    assert_eq!(out("<?php echo json_encode(sscanf('0x1A','%i'));"), "[26]");
    assert_eq!(out("<?php echo json_encode(sscanf('ff','%i'));"), "[null]");
    assert_eq!(out("<?php echo json_encode(sscanf('12 34 56','%d %*d %d'));"), "[12,56]");
    assert_eq!(out("<?php echo json_encode(sscanf('12 only','%d %d'));"), "[12,null]");
    assert_eq!(out("<?php echo json_encode(sscanf('xyz','a%d'));"), "[null]");
    assert_eq!(out("<?php echo json_encode(sscanf('hello','%3c'));"), "[\"hel\"]");
    assert_eq!(out("<?php echo json_encode(sscanf('-42 +7','%d %d'));"), "[-42,7]");
}

#[test]
fn sscanf_byref_mode() {
    assert_eq!(
        out("<?php $n=sscanf('12 f0 0x1A 777','%d %x %x %o',$a,$b,$c,$d); echo \"$n|$a|$b|$c|$d\";"),
        "4|12|240|26|511"
    );
    // Stops at first failed conversion; count reflects successes, rest NULL.
    assert_eq!(
        out("<?php $n=sscanf('only','%d %s',$a,$b); echo $n; var_dump($a,$b);"),
        "0NULL\nNULL\n"
    );
}

#[test]
fn strstr_family() {
    assert_eq!(out("<?php var_dump(strstr('hello@world.com','@'));"), "string(10) \"@world.com\"\n");
    assert_eq!(out("<?php var_dump(strstr('hello@world.com','@',true));"), "string(5) \"hello\"\n");
    assert_eq!(out("<?php var_dump(strstr('hello','xyz'));"), "bool(false)\n");
    assert_eq!(out("<?php var_dump(strstr('hello','LL'));"), "bool(false)\n");
    assert_eq!(out("<?php var_dump(stristr('HELLO@x','ll'));"), "string(5) \"LLO@x\"\n");
    assert_eq!(out("<?php var_dump(strrchr('a/b/c.txt','/'));"), "string(6) \"/c.txt\"\n");
    assert_eq!(out("<?php var_dump(strrchr('abc','x'));"), "bool(false)\n");
    // strrchr uses only the first byte of the needle.
    assert_eq!(out("<?php var_dump(strrchr('path/to','/x'));"), "string(3) \"/to\"\n");
    assert_eq!(out("<?php var_dump(strchr('a@b','@'));"), "string(2) \"@b\"\n");
}

#[test]
fn get_resource_type_stream() {
    let p = tmp_path("b53_grt");
    let _ = std::fs::remove_file(&p);
    let src = format!(
        "<?php $f=fopen('{p}','w'); echo get_resource_type($f),'|'; fclose($f); \
         echo get_resource_type($f);"
    );
    assert_eq!(out(&src), "stream|Unknown");
    let _ = std::fs::remove_file(&p);
}

#[test]
fn fprintf_and_vfprintf_to_stream() {
    let src = "<?php $m=fopen('php://memory','r+'); \
        $n=fprintf($m,'%05.2f-%s',3.14159,'hi'); rewind($m); echo $n,'|',fread($m,100),'|'; \
        $m2=fopen('php://memory','r+'); $k=vfprintf($m2,'%d/%d',[7,9]); rewind($m2); echo $k,'|',fread($m2,100); \
        fclose($m); fclose($m2);";
    assert_eq!(out(src), "8|03.14-hi|3|7/9");
}

#[test]
fn opendir_readdir_roundtrip() {
    let dir = tmp_path("b53_dir");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(format!("{dir}/a"), "").unwrap();
    std::fs::write(format!("{dir}/b"), "").unwrap();
    let src = format!(
        "<?php $d=opendir('{dir}'); echo get_resource_type($d),'|'; \
         $n=[]; while(($e=readdir($d))!==false){{ $n[]=$e; }} sort($n); echo implode(',',$n),'|'; \
         rewinddir($d); echo readdir($d)!==false?'got':'none','|'; \
         closedir($d); echo gettype($d);"
    );
    assert_eq!(out(&src), "stream|.,..,a,b|got|resource (closed)");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fstat_on_dir_handle_is_false_not_panic() {
    // Regression (step 53c): a dir handle reaching a stream builtin must not
    // panic. fstat → false; a byte-stream builtin → clean TypeError.
    let dir = tmp_path("b53_dirfstat");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let src = format!(
        "<?php $d=opendir('{dir}'); var_dump(fstat($d)); \
         try {{ fread($d, 4); }} catch (\\TypeError $e) {{ echo 'TypeError'; }} closedir($d);"
    );
    assert_eq!(out(&src), "bool(false)\nTypeError");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn opendir_missing_warns_false() {
    let (_, w) = out_diags("<?php opendir('/no/such/zz');");
    assert_eq!(
        w,
        vec!["opendir(/no/such/zz): Failed to open directory: No such file or directory"]
    );
    assert_eq!(out("<?php var_dump(@opendir('/no/such/zz'));"), "bool(false)\n");
}

#[test]
fn scandir_sort_orders_and_error() {
    let dir = tmp_path("b52_scan");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(format!("{dir}/b.txt"), "").unwrap();
    std::fs::write(format!("{dir}/a.txt"), "").unwrap();
    std::fs::write(format!("{dir}/c.log"), "").unwrap();
    std::fs::create_dir(format!("{dir}/sub")).unwrap();
    let src = format!(
        "<?php echo implode(',',scandir('{dir}')),'|'; echo implode(',',scandir('{dir}',1));"
    );
    assert_eq!(out(&src), ".,..,a.txt,b.txt,c.log,sub|sub,c.log,b.txt,a.txt,..,.");
    // Missing directory → false + the two oracle Warnings (errno 2 = ENOENT).
    let (_, w) = out_diags("<?php scandir('/no/such/zz');");
    assert_eq!(
        w,
        vec![
            "scandir(/no/such/zz): Failed to open directory: No such file or directory",
            "scandir(): (errno 2): No such file or directory",
        ]
    );
    assert_eq!(out("<?php var_dump(@scandir('/no/such/zz'));"), "bool(false)\n");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn glob_patterns_and_flags() {
    let dir = tmp_path("b52_glob");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(format!("{dir}/a.txt"), "").unwrap();
    std::fs::write(format!("{dir}/b.txt"), "").unwrap();
    std::fs::write(format!("{dir}/c.log"), "").unwrap();
    std::fs::create_dir(format!("{dir}/sub")).unwrap();
    let src = format!(
        "<?php $bn=fn($a)=>implode(',',array_map('basename',$a)); \
         echo $bn(glob('{dir}/*.txt')),'|'; \
         echo $bn(glob('{dir}/[ab].txt')),'|'; \
         echo $bn(glob('{dir}/?.txt')),'|'; \
         echo count(glob('{dir}/nomatch*')),'|'; \
         echo $bn(glob('{dir}/*', GLOB_ONLYDIR)),'|'; \
         echo $bn(glob('{dir}/{{a,c}}.*', GLOB_BRACE)),'|'; \
         echo glob('{dir}/*.txt')[0]==='{dir}/a.txt'?'abs':'rel';"
    );
    assert_eq!(out(&src), "a.txt,b.txt|a.txt,b.txt|a.txt,b.txt|0|sub|a.txt,c.log|abs");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn tempnam_creates_unique_file() {
    let dir = tmp_path("b52_tmpnam");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let src = format!(
        "<?php $f=tempnam('{dir}','pre'); \
         echo file_exists($f)?'E':'X'; echo '|'; echo substr(basename($f),0,3); \
         echo '|'; echo (is_string($f)&&$f!=='')?'S':'?'; unlink($f);"
    );
    assert_eq!(out(&src), "E|pre|S");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn tmpfile_roundtrip_and_autoremove() {
    // tmpfile() mints an unlinked read/write stream resource.
    let src = "<?php $f=tmpfile(); echo gettype($f),'|'; \
        echo fwrite($f,'hello'),'|'; rewind($f); echo fread($f,100); fclose($f);";
    assert_eq!(out(src), "resource|5|hello");
}

#[test]
fn mutator_warning_messages() {
    // Exact PHP Warning text (oracle-verified): each mutator frames the path /
    // strerror differently — mkdir omits the path, copy says "Failed to open
    // stream", touch says "Unable to create file …".
    let (_, w) = out_diags(
        "<?php unlink('/no/such/zz'); rmdir('/no/such/zz'); mkdir('/no/such/deep/zz'); \
         rename('/no/such/a','/no/such/b'); copy('/no/such/a','/tmp/zzz_phpr'); \
         readlink('/no/such/zz');",
    );
    assert_eq!(
        w,
        vec![
            "unlink(/no/such/zz): No such file or directory",
            "rmdir(/no/such/zz): No such file or directory",
            "mkdir(): No such file or directory",
            "rename(/no/such/a,/no/such/b): No such file or directory",
            "copy(/no/such/a): Failed to open stream: No such file or directory",
            "readlink(): No such file or directory",
        ]
    );
}
