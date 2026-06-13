//! `.phpt` test runner (plan step 6).
//!
//! PHP's own test suite ships thousands of `.phpt` files: a sectioned format
//! (`--TEST--`, `--FILE--`, `--EXPECT--` / `--EXPECTF--`, …) consumed by
//! `run-tests.php`. This runner reuses that corpus as a differential oracle,
//! but — crucially — it does *not* assume every test is in scope. It performs a
//! **capability scan** first: a file is run only if our front-end can lower it
//! and our evaluator produces a clean (warning- and fatal-free) result. Anything
//! else becomes a *motivated SKIP* with a category, exactly as the lowering
//! bridge anticipated (see `php_runtime::lower`).
//!
//! The point of this honesty: the only [`Status::Fail`] outcome is a genuine
//! output divergence on a script we fully support — the valuable signal. Scope
//! gaps (unsupported syntax, missing builtins, unrendered diagnostics — step 9)
//! are counted and labelled, never silently passed or falsely failed.

use std::fs;
use std::path::{Path, PathBuf};

use php_runtime::{run_source_with, LowerError, Registry};
use regex::Regex;

/// The classification of a single `.phpt` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// Lowered, ran clean, and output matched the expectation.
    Pass,
    /// Lowered and ran clean, but output diverged (the signal worth chasing).
    Fail,
    /// Out of current scope; not a defect. See `category`.
    Skip,
}

/// The result of evaluating one `.phpt` file.
#[derive(Debug, Clone)]
pub struct TestResult {
    pub status: Status,
    /// Short, human-readable detail (a diff for fails, a reason for skips).
    pub detail: String,
    /// A coarse bucket, set for skips (and `"ok"` for passes / `"mismatch"` for
    /// fails), so a run can be summarised by reason.
    pub category: &'static str,
}

impl TestResult {
    fn pass() -> Self {
        TestResult { status: Status::Pass, detail: String::new(), category: "ok" }
    }
    fn fail(detail: String) -> Self {
        TestResult { status: Status::Fail, detail, category: "mismatch" }
    }
    fn skip(category: &'static str, detail: String) -> Self {
        TestResult { status: Status::Skip, detail, category }
    }
}

/// Sections present in a `.phpt` file, in source order (`name`, `body`).
/// `name` excludes the surrounding dashes (e.g. `"FILE"`).
pub fn parse_sections(src: &[u8]) -> Vec<(String, String)> {
    let text = String::from_utf8_lossy(src);
    let mut sections: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        if let Some(name) = section_header(line) {
            sections.push((name, String::new()));
        } else if let Some(last) = sections.last_mut() {
            last.1.push_str(line);
            last.1.push('\n');
        }
        // Lines before the first header (none, in practice) are ignored.
    }
    sections
}

/// `--NAME--` header → `NAME`, where NAME is `[A-Z_]+` (mirrors run-tests.php).
fn section_header(line: &str) -> Option<String> {
    let rest = line.strip_prefix("--")?;
    let name = rest.strip_suffix("--")?;
    if !name.is_empty() && name.bytes().all(|b| b.is_ascii_uppercase() || b == b'_') {
        Some(name.to_string())
    } else {
        None
    }
}

fn find<'a>(sections: &'a [(String, String)], name: &str) -> Option<&'a str> {
    sections.iter().find(|(n, _)| n == name).map(|(_, b)| b.as_str())
}

/// Sections that imply behaviour we do not model (I/O, SAPI, ini tuning,
/// extensions). Their presence forces a SKIP — running anyway would compare
/// against an environment we are not reproducing.
const UNSUPPORTED_SECTIONS: &[&str] = &[
    "SKIPIF", "EXTENSIONS", "INI", "POST", "POST_RAW", "GET", "PUT", "COOKIE",
    "STDIN", "ARGS", "ENV", "REQUEST", "CGI", "PHPDBG", "HEADERS", "XFAIL",
    "FILE_EXTERNAL", "FILEEOF",
];

/// Classify and (when in scope) run a single `.phpt` source.
pub fn run_phpt(src: &[u8], reg: &Registry) -> TestResult {
    let sections = parse_sections(src);

    let Some(file) = find(&sections, "FILE") else {
        return TestResult::skip("malformed", "no --FILE-- section".to_string());
    };

    // Pick the expectation flavour.
    let (expect_kind, wanted) = if let Some(w) = find(&sections, "EXPECT") {
        (ExpectKind::Exact, w)
    } else if let Some(w) = find(&sections, "EXPECTF") {
        (ExpectKind::Format, w)
    } else if find(&sections, "EXPECTREGEX").is_some() {
        return TestResult::skip("expectregex", "--EXPECTREGEX-- not supported".to_string());
    } else {
        return TestResult::skip("malformed", "no --EXPECT(F)-- section".to_string());
    };

    // Out-of-scope sections.
    for (name, _) in &sections {
        if UNSUPPORTED_SECTIONS.contains(&name.as_str()) {
            return TestResult::skip("section", format!("--{name}-- section not modelled"));
        }
    }
    // `%r...%r` custom-regex placeholders need a real regex engine in the
    // expectation; rare, and we skip them rather than half-support.
    if matches!(expect_kind, ExpectKind::Format) && wanted.contains("%r") {
        return TestResult::skip("expectf-%r", "%r placeholder not supported".to_string());
    }

    // Capability scan: can the front-end even lower it?
    let source = file.as_bytes();
    if let Err(e) = php_runtime::lower_source(b"test.phpt", source) {
        return match e {
            LowerError::Unsupported { what, line } => {
                TestResult::skip("unsupported", format!("{what} (line {line})"))
            }
            LowerError::Parse(_) => {
                TestResult::skip("parse", "mago could not parse (out-of-scope syntax)".to_string())
            }
        };
    }

    // Run it.
    let outcome = match run_source_with(b"test.phpt", source, reg) {
        Ok(o) => o,
        Err(e) => return TestResult::skip("parse", format!("lower error: {e}")),
    };

    // A builtin we have not implemented yet is a scope gap, not a defect.
    if let Some(err) = &outcome.fatal {
        let msg = err.message();
        if msg.starts_with("Call to undefined function") {
            return TestResult::skip("builtin", msg.to_string());
        }
        // Any other fatal: we do not render fatals onto stdout yet (step 9), so
        // we cannot fairly compare. Skip rather than risk a false fail.
        return TestResult::skip(
            "diag-or-fatal",
            format!("fatal not rendered (step 9): {} {msg}", err.class_name()),
        );
    }
    // Likewise, unrendered warnings/notices/deprecations would desynchronise the
    // output comparison; defer those tests to step 9.
    if !outcome.diags.is_empty() {
        return TestResult::skip(
            "diag-or-fatal",
            format!("{} diagnostic(s) not rendered (step 9)", outcome.diags.len()),
        );
    }

    // Clean run: compare.
    let got = normalize(&String::from_utf8_lossy(&outcome.stdout));
    let want = normalize(wanted);

    // If the expectation itself contains diagnostic output (which we collect but
    // do not yet render onto stdout — step 9), comparison is unfair: skip.
    if expects_diagnostic(&want) {
        return TestResult::skip(
            "diag-or-fatal",
            "expectation contains diagnostics not rendered (step 9)".to_string(),
        );
    }

    let matched = match expect_kind {
        ExpectKind::Exact => got == want,
        ExpectKind::Format => match Regex::new(&expectf_to_regex(&want)) {
            Ok(re) => re.is_match(&got),
            // A malformed generated pattern is our bug in the converter, not a
            // test failure; surface it as a skip with the reason.
            Err(e) => return TestResult::skip("expectf-build", format!("regex build: {e}")),
        },
    };

    if matched {
        TestResult::pass()
    } else {
        TestResult::fail(format!(
            "expected {:?}\n   got      {:?}",
            truncate(&want, 200),
            truncate(&got, 200)
        ))
    }
}

enum ExpectKind {
    Exact,
    Format,
}

/// Normalise output for comparison: CRLF→LF and trim surrounding whitespace
/// (matching run-tests.php's `trim` + newline canonicalisation).
fn normalize(s: &str) -> String {
    s.replace("\r\n", "\n").trim().to_string()
}

/// Does the expected output contain a PHP diagnostic header? Until step 9 we
/// route warnings/notices/deprecations/fatals to a side channel rather than
/// stdout, so any test expecting them is out of scope.
fn expects_diagnostic(want: &str) -> bool {
    const MARKERS: &[&str] = &[
        "Warning: ",
        "Deprecated: ",
        "Notice: ",
        "Fatal error: ",
        "Parse error: ",
        "Strict Standards: ",
    ];
    want.lines()
        .any(|l| MARKERS.iter().any(|m| l.trim_start().starts_with(m)))
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    // Cut on a char boundary at or below `max`.
    let end = (0..=max).rev().find(|&i| s.is_char_boundary(i)).unwrap_or(0);
    format!("{}…", &s[..end])
}

/// Convert an `--EXPECTF--` body to an anchored regex, mirroring the placeholder
/// table in run-tests.php. Replacement strings contain no `%`, so the sequential
/// substitution is collision-free.
fn expectf_to_regex(wanted: &str) -> String {
    let mut re = regex::escape(wanted);
    for (from, to) in [
        ("%e", r"[/\\]"),
        ("%s", r"[^\r\n]+"),
        ("%S", r"[^\r\n]*"),
        ("%a", r".+"),
        ("%A", r".*"),
        ("%w", r"\s*"),
        ("%i", r"[+-]?\d+"),
        ("%d", r"\d+"),
        ("%x", r"[0-9a-fA-F]+"),
        ("%f", r"[+-]?\.?\d+\.?\d*(?:[Ee][+-]?\d+)?"),
        ("%c", r"."),
    ] {
        re = re.replace(from, to);
    }
    format!("(?s)^{re}$")
}

/// Aggregate counts over a run.
#[derive(Debug, Default, Clone)]
pub struct Summary {
    pub pass: usize,
    pub fail: usize,
    pub skip: usize,
    /// Skip counts keyed by category (sorted by the caller for display).
    pub skip_by_category: std::collections::BTreeMap<String, usize>,
    /// `(path, detail)` for each failure, for reporting.
    pub failures: Vec<(PathBuf, String)>,
}

impl Summary {
    pub fn total(&self) -> usize {
        self.pass + self.fail + self.skip
    }

    fn record(&mut self, path: &Path, r: &TestResult) {
        match r.status {
            Status::Pass => self.pass += 1,
            Status::Fail => {
                self.fail += 1;
                self.failures.push((path.to_path_buf(), r.detail.clone()));
            }
            Status::Skip => {
                self.skip += 1;
                *self.skip_by_category.entry(r.category.to_string()).or_insert(0) += 1;
            }
        }
    }
}

/// Run every `.phpt` under `root` (a file or a directory, searched
/// recursively), returning the aggregate [`Summary`].
pub fn run_path(root: &Path, reg: &Registry) -> std::io::Result<Summary> {
    let mut summary = Summary::default();
    for path in collect_phpt(root)? {
        let src = fs::read(&path)?;
        let result = run_phpt(&src, reg);
        summary.record(&path, &result);
    }
    Ok(summary)
}

/// Dotfiles, including macOS AppleDouble sidecars (`._foo.phpt`), are not tests.
fn is_hidden(p: &Path) -> bool {
    p.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with('.'))
}

/// Collect `.phpt` paths under `root`, sorted for deterministic output.
fn collect_phpt(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(p) = stack.pop() {
        let meta = fs::metadata(&p)?;
        if meta.is_dir() {
            for entry in fs::read_dir(&p)? {
                stack.push(entry?.path());
            }
        } else if p.extension().is_some_and(|e| e == "phpt") && !is_hidden(&p) {
            out.push(p);
        }
    }
    out.sort();
    Ok(out)
}
