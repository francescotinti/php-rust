//! Differential test: evaluator stdout vs the real PHP 8.5.7 CLI (the oracle).
//!
//! The corpus is curated to be warning-free and deterministic, so stdout fully
//! captures observable behaviour (diagnostic rendering is step 9). Each snippet
//! is bare PHP (no `<?php` tag): the oracle runs it via `php -r`, and our
//! evaluator runs it wrapped in `<?php`.
//!
//! The oracle binary is `$PHP_ORACLE` or `/tmp/php-src/sapi/cli/php`; if neither
//! exists the test is skipped (it is not built in every environment).

use std::path::Path;
use std::process::Command;

use php_runtime::run_source;

/// Warning-free snippets exercising the Tier 1 evaluator surface.
const CORPUS: &[&str] = &[
    // arithmetic + integer/float result types
    "echo 1 + 2;",
    "echo 10 - 3 * 2;",
    "echo 2 ** 16;",
    "echo 7 / 2;",
    "echo 8 / 4;",
    "echo 17 % 5;",
    "echo -17 % 5;",
    "echo 17 % -5;",
    "echo (1 + 2) * 3;",
    "echo 2 + 3 * 4 - 1;",
    "echo -5;",
    "echo - -5;",
    "echo +5;",
    // float formatting (precision=14)
    "echo 0.1 + 0.2;",
    "echo 1 / 3;",
    "echo 10 / 3;",
    "echo 1.5e3;",
    "echo 100000000000000.0;",
    "echo 0.0001;",
    // bit ops
    "echo 6 & 3;",
    "echo 6 | 1;",
    "echo 6 ^ 3;",
    "echo ~5;",
    "echo 1 << 4;",
    "echo 256 >> 2;",
    // string concat + coercion
    "echo 'a' . 'b' . 'c';",
    "echo 'x' . 1 . 2.5;",
    "echo 'n' . true . false . 'm';",
    "echo '3' + 4;",
    "echo '3.5' + '1.5';",
    "echo '10' . '20';",
    // comparisons
    "echo (1 < 2) ? 't' : 'f';",
    "echo (2 <= 2) ? 't' : 'f';",
    "echo (3 > 2) ? 't' : 'f';",
    "echo (2 >= 3) ? 't' : 'f';",
    "echo 1 <=> 2;",
    "echo 2 <=> 2;",
    "echo 3 <=> 2;",
    "echo (2 == 2.0) ? 't' : 'f';",
    "echo (2 === 2.0) ? 't' : 'f';",
    "echo ('9' < '10') ? 't' : 'f';",
    "echo ('abc' == 'abc') ? 't' : 'f';",
    // boolean / logical
    "echo true && false ? 't' : 'f';",
    "echo true || false ? 't' : 'f';",
    "echo (1 xor 0) ? 't' : 'f';",
    "echo !0 ? 't' : 'f';",
    // casts
    "echo (int)3.9;",
    "echo (int)'42abc';",
    "echo (float)'1.5';",
    "echo (string)42;",
    "echo (int)true;",
    // variables + assignment
    "$x = 5; $y = $x * 2; echo $y;",
    "echo $x = 7;",
    "$x = 10; $x += 5; $x *= 2; echo $x;",
    "$s = 'a'; $s .= 'b'; echo $s;",
    "$x = 5; echo $x++; echo '/'; echo $x;",
    "$x = 5; echo ++$x; echo '/'; echo $x;",
    // control flow
    "$x = 3; if ($x > 5) echo 'a'; elseif ($x > 2) echo 'b'; else echo 'c';",
    "$i = 1; $s = 0; while ($i <= 5) { $s += $i; $i++; } echo $s;",
    "$i = 0; do { echo $i; $i++; } while ($i < 3);",
    "for ($i = 0; $i < 4; $i++) { echo $i; }",
    "for ($i = 0; $i < 5; $i++) { if ($i % 2 == 0) continue; echo $i; }",
    "for ($i = 0; $i < 3; $i++) { for ($j = 0; $j < 3; $j++) { if ($j == 1) break 2; echo $i; echo $j; } } echo 'X';",
    "echo 0 ?: 'z';",
    "echo 5 ?: 'z';",
    "$n = 10; $f = 1; for ($k = 2; $k <= $n; $k++) { $f *= $k; } echo $f;",
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
    let output = Command::new(php)
        .arg("-n") // ignore php.ini → deterministic defaults (precision=14)
        .arg("-r")
        .arg(code)
        .output()
        .expect("spawn oracle");
    output.stdout
}

#[test]
fn evaluator_matches_oracle() {
    let Some(php) = oracle_path() else {
        eprintln!("SKIP: PHP oracle not found (set PHP_ORACLE or build /tmp/php-src)");
        return;
    };

    let mut mismatches = Vec::new();
    for code in CORPUS {
        let expected = oracle_stdout(&php, code);

        let wrapped = format!("<?php {code}");
        let got = match run_source(b"diff.php", wrapped.as_bytes()) {
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
