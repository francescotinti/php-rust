//! Differential test: every (value, op, value) case is evaluated by the
//! reference php 8.5.7 CLI (built from the local source tree) and by our
//! operators; outputs must match byte-for-byte, diagnostics included.
//!
//! Skipped (with a message) when the oracle binary is missing.
//! Override the path with PHP_ORACLE=/path/to/php.

use std::fmt::Write as _;
use std::process::Command;
use std::rc::Rc;

use php_types::convert::{to_bool, to_zstr_cast};
use php_types::dtoa::double_to_shortest;
use php_types::ops;
use php_types::{Diag, Diags, Key, PhpArray, PhpError, Zval};

fn oracle() -> Option<String> {
    let path = std::env::var("PHP_ORACLE").unwrap_or_else(|_| "/tmp/php-src/sapi/cli/php".into());
    if std::path::Path::new(&path).exists() {
        Some(path)
    } else {
        None
    }
}

/// (php literal, our value)
fn corpus() -> Vec<(String, Zval)> {
    let mut v: Vec<(String, Zval)> = vec![
        ("null".into(), Zval::Null),
        ("true".into(), Zval::Bool(true)),
        ("false".into(), Zval::Bool(false)),
        ("0".into(), Zval::Long(0)),
        ("1".into(), Zval::Long(1)),
        ("-1".into(), Zval::Long(-1)),
        ("5".into(), Zval::Long(5)),
        ("100".into(), Zval::Long(100)),
        ("9223372036854775807".into(), Zval::Long(i64::MAX)),
        ("PHP_INT_MIN".into(), Zval::Long(i64::MIN)),
        ("0.0".into(), Zval::Double(0.0)),
        ("-0.0".into(), Zval::Double(-0.0)),
        ("0.5".into(), Zval::Double(0.5)),
        ("-1.5".into(), Zval::Double(-1.5)),
        ("0.1".into(), Zval::Double(0.1)),
        ("1e14".into(), Zval::Double(1e14)),
        ("1e15".into(), Zval::Double(1e15)),
        ("1e100".into(), Zval::Double(1e100)),
        ("7.9".into(), Zval::Double(7.9)),
        ("NAN".into(), Zval::Double(f64::NAN)),
        ("INF".into(), Zval::Double(f64::INFINITY)),
        ("-INF".into(), Zval::Double(f64::NEG_INFINITY)),
    ];
    for s in [
        "", "0", "1", "01", " 1", "1 ", "abc", "5abc", "0.5", "1e2", "-1", "0x1A", ".5", "5.",
        "9223372036854775807", "9223372036854775808", "99999999999999999999",
        "99999999999999999998", "z", "a9", "Zz", "10",
    ] {
        v.push((format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")), Zval::str_from(s)));
    }
    // Arrays
    v.push(("[]".into(), Zval::Array(Rc::new(PhpArray::new()))));
    let mut a1 = PhpArray::new();
    a1.append(Zval::Long(1)).unwrap();
    a1.append(Zval::Long(2)).unwrap();
    v.push(("[1, 2]".into(), Zval::Array(Rc::new(a1))));
    let mut a2 = PhpArray::new();
    a2.insert(Key::from_bytes(b"a"), Zval::Long(1));
    v.push(("[\"a\" => 1]".into(), Zval::Array(Rc::new(a2))));
    v
}

const BINOPS: &[&str] = &[
    "+", "-", "*", "/", "%", "**", ".", "==", "===", "<", "<=", "<=>", "&", "|", "^", "<<", ">>",
];

/// Render diagnostics + outcome exactly like the PHP harness does.
fn render_rust_case(op: &str, a: &Zval, b: &Zval) -> String {
    let mut diags = Diags::new();
    let outcome: Result<Zval, PhpError> = match op {
        "+" => ops::add(a, b, &mut diags),
        "-" => ops::sub(a, b, &mut diags),
        "*" => ops::mul(a, b, &mut diags),
        "/" => ops::div(a, b, &mut diags),
        "%" => ops::modulo(a, b, &mut diags),
        "**" => ops::pow(a, b, &mut diags),
        "." => ops::concat(a, b, &mut diags),
        "==" => Ok(Zval::Bool(ops::loose_eq(a, b))),
        "===" => Ok(Zval::Bool(ops::identical(a, b))),
        "<" => Ok(Zval::Bool(ops::smaller(a, b))),
        "<=" => Ok(Zval::Bool(ops::smaller_or_equal(a, b))),
        "<=>" => Ok(Zval::Long(ops::compare(a, b) as i64)),
        "&" => ops::bw_and(a, b, &mut diags),
        "|" => ops::bw_or(a, b, &mut diags),
        "^" => ops::bw_xor(a, b, &mut diags),
        "<<" => ops::shl(a, b, &mut diags),
        ">>" => ops::shr(a, b, &mut diags),
        _ => unreachable!(),
    };
    render(diags, outcome)
}

fn render_rust_unary(op: &str, val: &Zval) -> String {
    let mut diags = Diags::new();
    let outcome: Result<Zval, PhpError> = match op {
        "++" => {
            let mut v = val.clone();
            ops::increment(&mut v, &mut diags).map(|_| v)
        }
        "--" => {
            let mut v = val.clone();
            ops::decrement(&mut v, &mut diags).map(|_| v)
        }
        "neg" => ops::neg(val, &mut diags),
        "~" => ops::bw_not(val, &mut diags),
        "!" => Ok(Zval::Bool(!to_bool(val, &mut diags))),
        "(string)" => {
            let s = to_zstr_cast(val, &mut diags);
            Ok(Zval::Str(s))
        }
        _ => unreachable!(),
    };
    render(diags, outcome)
}

fn render(diags: Diags, outcome: Result<Zval, PhpError>) -> String {
    let mut out = String::new();
    for d in &diags {
        match d {
            Diag::Warning(m) => writeln!(out, "D:Warning:{m}").unwrap(),
            Diag::Deprecated(m) => writeln!(out, "D:Deprecated:{m}").unwrap(),
            Diag::Notice(m) => writeln!(out, "D:Notice:{m}").unwrap(),
        }
    }
    match outcome {
        Ok(v) => {
            out.push_str("R:");
            var_dump(&v, 0, &mut out);
        }
        Err(e) => writeln!(out, "E:{}:{}", e.class_name(), e.message()).unwrap(),
    }
    out
}

/// Minimal var_dump replica (scalars + arrays), shortest float mode.
fn var_dump(v: &Zval, level: usize, out: &mut String) {
    let pad = "  ".repeat(level);
    match v {
        Zval::Undef | Zval::Null => writeln!(out, "{pad}NULL").unwrap(),
        Zval::Bool(b) => writeln!(out, "{pad}bool({})", if *b { "true" } else { "false" }).unwrap(),
        Zval::Long(l) => writeln!(out, "{pad}int({l})").unwrap(),
        Zval::Double(d) => writeln!(
            out,
            "{pad}float({})",
            String::from_utf8_lossy(&double_to_shortest(*d))
        )
        .unwrap(),
        Zval::Str(s) => writeln!(
            out,
            "{pad}string({}) \"{}\"",
            s.len(),
            String::from_utf8_lossy(s.as_bytes())
        )
        .unwrap(),
        Zval::Array(a) => {
            writeln!(out, "{pad}array({}) {{", a.len()).unwrap();
            let inner = "  ".repeat(level + 1);
            for (k, val) in a.iter() {
                match k {
                    Key::Int(i) => writeln!(out, "{inner}[{i}]=>").unwrap(),
                    Key::Str(s) => {
                        writeln!(out, "{inner}[\"{}\"]=>", String::from_utf8_lossy(s.as_bytes()))
                            .unwrap()
                    }
                }
                var_dump(val, level + 1, out);
            }
            writeln!(out, "{pad}}}").unwrap();
        }
        // The differential corpus exercises only scalar/array ops, never refs
        // or closures.
        Zval::Ref(_) => unreachable!("differential corpus produces no references"),
        Zval::Closure(_) => unreachable!("differential corpus produces no closures"),
        Zval::Object(_) => unreachable!("differential corpus produces no objects"),
        Zval::Generator(_) => unreachable!("differential corpus produces no generators"),
        Zval::Resource(_) => unreachable!("differential corpus produces no resources"),
    }
}

/// Build the PHP script computing every case in the same block format.
fn build_php_script(cases: &[(String, String)]) -> String {
    let mut s = String::from(
        r#"<?php
error_reporting(E_ALL);
set_error_handler(function ($no, $msg) {
    $sev = match (true) {
        (bool)($no & (E_WARNING | E_USER_WARNING)) => "Warning",
        (bool)($no & (E_DEPRECATED | E_USER_DEPRECATED)) => "Deprecated",
        (bool)($no & (E_NOTICE | E_USER_NOTICE)) => "Notice",
        default => "Other$no",
    };
    echo "D:$sev:$msg\n";
    return true;
});
"#,
    );
    for (id, expr) in cases {
        s.push_str(&format!(
            "echo \"===CASE {id}===\\n\";\ntry {{ $r = ({expr}); echo \"R:\"; var_dump($r); }} catch (\\Throwable $e) {{ echo \"E:\", get_class($e), \":\", $e->getMessage(), \"\\n\"; }}\n"
        ));
    }
    s
}

fn run_php(php: &str, script: &str) -> String {
    let dir = std::env::temp_dir().join("php_types_differential");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("cases.php");
    std::fs::write(&file, script).unwrap();
    let out = Command::new(php)
        .arg("-n") // no php.ini: defaults (precision=14, serialize_precision=-1)
        .arg(&file)
        .output()
        .expect("oracle php run");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn parse_blocks(output: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let mut current: Option<String> = None;
    let mut buf = String::new();
    for line in output.lines() {
        if let Some(rest) = line.strip_prefix("===CASE ").and_then(|l| l.strip_suffix("===")) {
            if let Some(id) = current.take() {
                map.insert(id, std::mem::take(&mut buf));
            }
            current = Some(rest.to_string());
        } else if current.is_some() {
            buf.push_str(line);
            buf.push('\n');
        }
    }
    if let Some(id) = current {
        map.insert(id, buf);
    }
    map
}

#[test]
fn differential_operators_vs_oracle() {
    let Some(php) = oracle() else {
        eprintln!("SKIP: php oracle not found (build /tmp/php-src or set PHP_ORACLE)");
        return;
    };
    let corpus = corpus();
    let mut php_cases: Vec<(String, String)> = Vec::new();
    let mut rust_results: Vec<(String, String)> = Vec::new();

    // Binary ops over the full corpus product.
    for (i, (la, va)) in corpus.iter().enumerate() {
        for (j, (lb, vb)) in corpus.iter().enumerate() {
            for op in BINOPS {
                let id = format!("b{i}_{j}_{op}");
                php_cases.push((id.clone(), format!("({la}) {op} ({lb})")));
                rust_results.push((id, render_rust_case(op, va, vb)));
            }
        }
    }
    // Unary ops.
    for (i, (la, va)) in corpus.iter().enumerate() {
        for (op, expr) in [
            ("++", format!("(function() {{ $v = {la}; $v++; return $v; }})()")),
            ("--", format!("(function() {{ $v = {la}; $v--; return $v; }})()")),
            ("neg", format!("-({la})")),
            ("~", format!("~({la})")),
            ("!", format!("!({la})")),
            ("(string)", format!("(string)({la})")),
        ] {
            let id = format!("u{i}_{op}");
            php_cases.push((id.clone(), expr));
            rust_results.push((id, render_rust_unary(op, va)));
        }
    }

    let script = build_php_script(&php_cases);
    let oracle_out = run_php(&php, &script);
    let oracle_blocks = parse_blocks(&oracle_out);

    let mut mismatches = Vec::new();
    let case_by_id: std::collections::HashMap<_, _> =
        php_cases.iter().map(|(id, e)| (id.clone(), e.clone())).collect();
    for (id, ours) in &rust_results {
        let theirs = oracle_blocks.get(id).cloned().unwrap_or_default();
        if &theirs != ours {
            mismatches.push(format!(
                "case {id}: {}\n--- oracle ---\n{theirs}--- ours ---\n{ours}",
                case_by_id[id]
            ));
        }
    }
    let total = rust_results.len();
    if !mismatches.is_empty() {
        let report = std::env::temp_dir().join("php_types_differential/report.txt");
        std::fs::write(&report, mismatches.join("\n")).unwrap();
        let shown: Vec<_> = mismatches.iter().take(5).cloned().collect();
        panic!(
            "{} / {} differential mismatches. Full report: {}\n\n{}",
            mismatches.len(),
            total,
            report.display(),
            shown.join("\n")
        );
    }
    println!("differential: {total} cases, 0 mismatches");
}
