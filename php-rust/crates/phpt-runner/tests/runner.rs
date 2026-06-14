//! Tests for the `.phpt` runner: section parsing, the capability scan, the
//! EXPECT/EXPECTF matchers, and the classification rules.

use std::path::Path;

use php_builtins::registry;
use phpt_runner::{parse_sections, run_path, run_phpt, Status};

fn status(src: &str) -> (Status, &'static str) {
    let reg = registry();
    let r = run_phpt(src.as_bytes(), &reg);
    (r.status, r.category)
}

#[test]
fn parses_named_sections_in_order() {
    let s = parse_sections(b"--TEST--\nhi\n--FILE--\n<?php echo 1;\n--EXPECT--\n1\n");
    let names: Vec<_> = s.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["TEST", "FILE", "EXPECT"]);
    assert_eq!(s[1].1.trim(), "<?php echo 1;");
}

#[test]
fn exact_match_passes() {
    let (st, _) = status("--TEST--\nt\n--FILE--\n<?php echo 1 + 2;\n--EXPECT--\n3\n");
    assert_eq!(st, Status::Pass);
}

#[test]
fn exact_mismatch_fails() {
    let (st, cat) = status("--TEST--\nt\n--FILE--\n<?php echo 1 + 2;\n--EXPECT--\n4\n");
    assert_eq!(st, Status::Fail);
    assert_eq!(cat, "mismatch");
}

#[test]
fn expectf_wildcards_match() {
    let src = "--TEST--\nt\n--FILE--\n<?php echo 'id=', 6 * 7;\n--EXPECTF--\nid=%d\n";
    assert_eq!(status(src).0, Status::Pass);
    // %s matches the float text too.
    let src = "--TEST--\nt\n--FILE--\n<?php echo 1.0 / 8;\n--EXPECTF--\n%s\n";
    assert_eq!(status(src).0, Status::Pass);
}

#[test]
fn unsupported_construct_is_skipped() {
    // Enums are supported as of step 23, so use a variadic parameter — still out
    // of scope — as the motivated-skip example.
    let (st, cat) =
        status("--TEST--\nt\n--FILE--\n<?php function f(...$a){} echo 1;\n--EXPECT--\n1\n");
    assert_eq!(st, Status::Skip);
    assert_eq!(cat, "unsupported");
}

#[test]
fn out_of_scope_section_is_skipped() {
    let (st, cat) =
        status("--TEST--\nt\n--EXTENSIONS--\njson\n--FILE--\n<?php echo 1;\n--EXPECT--\n1\n");
    assert_eq!(st, Status::Skip);
    assert_eq!(cat, "section");
}

#[test]
fn undefined_builtin_is_skipped_not_failed() {
    let (st, cat) = status("--TEST--\nt\n--FILE--\n<?php echo nope_fn();\n--EXPECT--\nx\n");
    assert_eq!(st, Status::Skip);
    assert_eq!(cat, "builtin");
}

#[test]
fn expected_diagnostic_is_rendered_and_matched() {
    // Step 9: a warning is rendered inline and compared. EXPECTF's `%s` absorbs
    // the file path, so a faithful diagnostic now PASSES rather than skipping.
    let src = "--TEST--\nt\n--FILE--\n<?php echo $x;\n--EXPECTF--\n\nWarning: Undefined variable $x in %s on line 1\n";
    let (st, _cat) = status(src);
    assert_eq!(st, Status::Pass);
}

#[test]
fn missing_expected_diagnostic_now_fails() {
    // A test that expects a warning the script never raises is a real divergence
    // (no longer hidden behind a step-9 skip).
    let src = "--TEST--\nt\n--FILE--\n<?php echo 1;\n--EXPECT--\nWarning: something in x on line 1\n1\n";
    assert_eq!(status(src).0, Status::Fail);
}

#[test]
fn uncaught_fatal_is_rendered_and_matched() {
    // An uncaught DivisionByZeroError renders the full CLI fatal block.
    let src = "--TEST--\nt\n--FILE--\n<?php $x = 1 % 0;\n--EXPECTF--\n\nFatal error: Uncaught DivisionByZeroError: Modulo by zero in %s:1\nStack trace:\n#0 {main}\n  thrown in %s on line 1\n";
    let (st, _cat) = status(src);
    assert_eq!(st, Status::Pass);
}

#[test]
fn expectregex_is_skipped() {
    let (st, cat) = status("--TEST--\nt\n--FILE--\n<?php echo 1;\n--EXPECTREGEX--\n\\d\n");
    assert_eq!(st, Status::Skip);
    assert_eq!(cat, "expectregex");
}

#[test]
fn closing_tag_eats_one_newline() {
    // `?>` swallows a single trailing newline (Zend lexer rule).
    let src = "--TEST--\nt\n--FILE--\n<?php echo 'a'; ?>\nb\n--EXPECT--\nab\n";
    assert_eq!(status(src).0, Status::Pass);
}

#[test]
fn run_path_over_fixtures_dir() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let summary = run_path(&dir, &registry()).expect("read fixtures");
    // 2 pass (exact + expectf), 2 skip (unsupported fn, EXTENSIONS section).
    assert_eq!(summary.fail, 0, "unexpected failures: {:?}", summary.failures);
    assert_eq!(summary.pass, 2, "summary: {summary:?}");
    assert_eq!(summary.skip, 2, "summary: {summary:?}");
}
