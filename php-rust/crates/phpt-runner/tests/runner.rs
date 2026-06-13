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
    // A class declaration is still out of scope (step 8 added functions, not
    // OOP) → motivated skip.
    let (st, cat) = status("--TEST--\nt\n--FILE--\n<?php class C {} echo 1;\n--EXPECT--\n1\n");
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
fn expected_diagnostic_is_skipped() {
    // We collect diagnostics but do not render them onto stdout yet (step 9):
    // a test expecting a warning is out of scope rather than a failure.
    let src = "--TEST--\nt\n--FILE--\n<?php echo 1;\n--EXPECT--\nWarning: something in x on line 1\n1\n";
    let (st, cat) = status(src);
    assert_eq!(st, Status::Skip);
    assert_eq!(cat, "diag-or-fatal");
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
