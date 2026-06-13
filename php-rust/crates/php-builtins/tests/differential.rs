//! Differential test: builtins' stdout vs the real PHP 8.5.7 CLI (the oracle).
//!
//! Focus is var_dump formatting (incl. shortest-roundtrip floats, INF/NAN/-0)
//! plus the inspection/cast builtins. Snippets are bare PHP run via `php -n -r`;
//! arrays are built with `(array)` casts since array literals are not lowered
//! until step 7. Skipped if no oracle is available.

use std::path::Path;
use std::process::Command;

use php_builtins::registry;
use php_runtime::run_source_with;

const CORPUS: &[&str] = &[
    // var_dump scalars + int/float result types
    "var_dump(42);",
    "var_dump(-7);",
    "var_dump(0);",
    "var_dump(1.5);",
    "var_dump(1.0);",
    "var_dump(100.0);",
    "var_dump(0.1 + 0.2);",
    "var_dump(1 / 3);",
    "var_dump(1e20);",
    "var_dump(1e-7);",
    "var_dump(-0.0);",
    "var_dump(1e308 * 10);",                  // INF
    "var_dump(-1e308 * 10);",                 // -INF
    "var_dump((1e308 * 10) - (1e308 * 10));", // NAN
    "var_dump(true);",
    "var_dump(false);",
    "var_dump(null);",
    "var_dump('hello');",
    "var_dump('');",
    "var_dump(1, 'x', true, null);",
    // var_dump arrays via (array) cast
    "var_dump((array)5);",
    "var_dump((array)'x');",
    "var_dump((array)null);",
    // var_dump array literals (step 7): recursive + nested + keyed
    "var_dump([1, 'two', 3.5]);",
    "var_dump(['a' => 1, 'b' => [2, 3]]);",
    "var_dump([]);",
    "var_dump(['x' => true, 'y' => null]);",
    "var_dump([5 => 'a', 'b', 10 => 'c', 'd']);",
    "$a = [1, 2]; $a[] = 3; unset($a[0]); var_dump($a);",
    // strlen / gettype
    "echo strlen('hello');",
    "echo strlen('');",
    "echo strlen(12345);",
    "echo gettype(1), '/', gettype(1.5), '/', gettype('x'), '/', gettype(true), '/', gettype(null);",
    // is_* predicates
    "var_dump(is_int(1), is_int(1.0), is_string('a'), is_bool(true), is_null(null), is_float(1.5));",
    "var_dump(is_array((array)1), is_scalar(1), is_scalar((array)1));",
    "var_dump(is_numeric('123'), is_numeric('1.5e3'), is_numeric('12abc'), is_numeric('abc'), is_numeric(42));",
    // value casts
    "echo intval('42abc'), '/', intval(3.9);",
    "echo floatval('1.5x');",
    "var_dump(strval(42), strval(1.5), strval(true));",
    "var_dump(boolval(0), boolval('a'), boolval(''));",
];

fn oracle_path() -> Option<String> {
    if let Ok(p) = std::env::var("PHP_ORACLE") {
        if Path::new(&p).exists() {
            return Some(p);
        }
    }
    let default = "/tmp/php-src/sapi/cli/php";
    Path::new(default).exists().then(|| default.to_string())
}

fn oracle_stdout(php: &str, code: &str) -> Vec<u8> {
    Command::new(php)
        .arg("-n")
        .arg("-r")
        .arg(code)
        .output()
        .expect("spawn oracle")
        .stdout
}

#[test]
fn builtins_match_oracle() {
    let Some(php) = oracle_path() else {
        eprintln!("SKIP: PHP oracle not found (set PHP_ORACLE or build /tmp/php-src)");
        return;
    };

    let reg = registry();
    let mut mismatches = Vec::new();
    for code in CORPUS {
        let expected = oracle_stdout(&php, code);
        let wrapped = format!("<?php {code}");
        let got = match run_source_with(b"diff.php", wrapped.as_bytes(), &reg) {
            Ok(o) if o.fatal.is_none() => o.stdout,
            Ok(o) => {
                mismatches.push(format!("{code}\n  evaluator fatal: {:?}", o.fatal));
                continue;
            }
            Err(e) => {
                mismatches.push(format!("{code}\n  lower error: {e}"));
                continue;
            }
        };
        if got != expected {
            mismatches.push(format!(
                "{code}\n  oracle:    {:?}\n  evaluator: {:?}",
                String::from_utf8_lossy(&expected),
                String::from_utf8_lossy(&got)
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "{} / {} differential mismatches:\n{}",
        mismatches.len(),
        CORPUS.len(),
        mismatches.join("\n")
    );
}
