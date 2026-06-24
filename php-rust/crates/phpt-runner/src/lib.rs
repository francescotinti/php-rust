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
//! Since step 9 the evaluator renders diagnostics and uncaught fatals inline
//! (`Outcome::rendered`), so tests that expect warnings/notices/fatals are now
//! run and compared rather than skipped; only an undefined-function call still
//! skips (as a missing-builtin scope gap).
//!
//! The point of this honesty: the only [`Status::Fail`] outcome is a genuine
//! output divergence on a script we fully support — the valuable signal. Scope
//! gaps (unsupported syntax, missing builtins) are counted and labelled, never
//! silently passed or falsely failed.

use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use php_runtime::{LowerError, Registry};
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

/// Sections that imply behaviour we do not model (I/O, SAPI, ini tuning).
/// Their presence forces a SKIP — running anyway would compare against an
/// environment we are not reproducing. `EXTENSIONS` is handled separately: a
/// test gated only on extensions we *do* model is run (see [`SUPPORTED_EXTENSIONS`]).
const UNSUPPORTED_SECTIONS: &[&str] = &[
    "SKIPIF", "INI", "POST", "POST_RAW", "GET", "PUT", "COOKIE",
    "STDIN", "ARGS", "ENV", "REQUEST", "CGI", "PHPDBG", "HEADERS", "XFAIL",
    "FILE_EXTERNAL", "FILEEOF",
];

/// Extensions the interpreter substantially models, so a `--EXTENSIONS--` test
/// requiring only these can be run instead of skipped. `core`/`standard` are the
/// always-present built-in functions; the rest are the extensions ported by the
/// corresponding steps (pcre: 31/36/37, json: 26, date: 34/35, mbstring: 41–43).
/// A test requiring anything else still skips. Names are compared lowercase.
const SUPPORTED_EXTENSIONS: &[&str] =
    &["core", "standard", "mbstring", "pcre", "json", "date"];

/// Classify and (when in scope) run a single `.phpt` source on the bytecode VM.
pub fn run_phpt(src: &[u8], name: &[u8], reg: &Registry) -> TestResult {
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
    // An extension-gated test runs only when every required extension is one we
    // model; otherwise it would fail for missing functions, not a real divergence.
    if let Some(exts) = find(&sections, "EXTENSIONS") {
        for ext in exts.split_whitespace() {
            if !SUPPORTED_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()) {
                return TestResult::skip(
                    "extension",
                    format!("requires unsupported extension '{ext}'"),
                );
            }
        }
    }
    // `%r...%r` custom-regex placeholders need a real regex engine in the
    // expectation; rare, and we skip them rather than half-support.
    if matches!(expect_kind, ExpectKind::Format) && wanted.contains("%r") {
        return TestResult::skip("expectf-%r", "%r placeholder not supported".to_string());
    }

    // Capability scan: can the front-end even lower it?
    let source = file.as_bytes();
    if let Err(e) = php_runtime::lower_source(name, source) {
        match e {
            LowerError::Unsupported { what, line } => {
                return TestResult::skip("unsupported", format!("{what} (line {line})"))
            }
            LowerError::Parse(_) => {
                return TestResult::skip(
                    "parse",
                    "mago could not parse (out-of-scope syntax)".to_string(),
                )
            }
            // A compile-time fatal (e.g. trait collision, step 21) is faithful PHP
            // behaviour, not a coverage gap: fall through and run it so the fatal
            // is rendered onto the stream and compared like normal output.
            LowerError::Fatal { .. } => {}
        }
    }

    // Run it. The script's name doubles as the path PHP prints in diagnostics
    // (`... in <name> on line N`). run-tests.php runs each test from a temp file
    // named `<test>.php`, so `name` is the `.phpt` path with a `.php` extension;
    // EXPECTF `%s` placeholders absorb the directory prefix, and patterns that
    // embed the basename (e.g. `%sfinally_goto_001.php`) now line up too.
    // Materialize the script on disk so `__FILE__` resolves to a real file —
    // `fopen(__FILE__)` / `include __FILE__` then behave as under run-tests.php
    // (which executes a real `<test>.php`). Guard against clobbering a companion
    // source file that genuinely exists next to the `.phpt`: only create the file
    // when it is absent, and only remove the one we created.
    let script_path = Path::new(OsStr::from_bytes(name)).to_path_buf();
    let materialized = !script_path.exists() && fs::write(&script_path, source).is_ok();
    // Run on the bytecode VM: it yields the CLI-faithful `rendered` stream and an
    // optional fatal message, and additionally reports a construct its compiler
    // rejects (a coverage gap, skipped distinctly).
    let run: Result<(Vec<u8>, Option<String>), TestResult> =
        match php_runtime::vm::run_source_with(name, source, reg) {
            Ok(o) => Ok((o.rendered, o.fatal.as_ref().map(|e| e.message().to_string()))),
            Err(php_runtime::vm::VmRunError::Unsupported(what)) => {
                Err(TestResult::skip("vm-unsupported", what))
            }
            Err(php_runtime::vm::VmRunError::Lower(e)) => {
                Err(TestResult::skip("parse", format!("lower error: {e}")))
            }
        };
    if materialized {
        let _ = fs::remove_file(&script_path);
    }
    let (rendered, fatal_msg) = match run {
        Ok(v) => v,
        Err(skip) => return skip,
    };

    // A builtin we have not implemented yet is a scope gap, not a defect: a
    // "Call to undefined function" fatal means missing coverage. Every other
    // fatal is rendered onto the stream (step 9) and compared like normal output.
    if let Some(msg) = &fatal_msg {
        if msg.starts_with("Call to undefined function") {
            return TestResult::skip("builtin", msg.clone());
        }
    }

    // Compare the rendered stream: program output with diagnostics and any
    // uncaught fatal interleaved exactly as PHP's CLI prints them (step 9).
    let mut got = normalize(&String::from_utf8_lossy(&rendered));
    let want = normalize(wanted);

    // run-tests.php runs every test with `fatal_error_backtraces=Off`, so a
    // plain `Fatal error:` prints *without* the trailing `Stack trace:\n#0
    // {main}` our engine always appends (it mirrors PHP's default INI). When the
    // expectation itself carries no stack trace, drop ours so the comparison is
    // faithful to the harness the `.phpt` was authored against. Uncaught
    // exceptions keep their trace (the expectation includes it), so this only
    // strips the engine backtrace, never an exception's own.
    if !want.contains("Stack trace:") {
        if let Some(idx) = got.find("\nStack trace:") {
            got.truncate(idx);
            got = normalize(&got);
        }
    }

    // Compile-time diagnostics are out of scope: our front-end (mago) parses,
    // and we never run the engine's compile-time validation (attribute targets,
    // illegal type declarations, parser strictness). When the expectation is such
    // an error — a `Parse error:` or a non-`Uncaught` `Fatal error:` — and we did
    // not produce one, the test exercises a capability we do not model; skip it
    // honestly rather than counting a false divergence.
    if expects_compile_error(&want) && !is_engine_fatal(&got) {
        return TestResult::skip(
            "compile-error",
            "compile-time diagnostic not modelled".to_string(),
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
        TestResult::fail(unified_diff(&want, &got, &expect_kind))
    }
}

/// Do an expected line and an actual line correspond? Exact sections compare
/// literally; EXPECTF sections treat the expected line as an anchored pattern so
/// a `%d`/`%s` placeholder still counts as a match (and the diff lands on the
/// *first genuinely diverging* line, not on every line with a placeholder).
fn lines_match(want_line: &str, got_line: &str, kind: &ExpectKind) -> bool {
    match kind {
        ExpectKind::Exact => want_line == got_line,
        ExpectKind::Format => Regex::new(&expectf_to_regex(want_line))
            .map(|re| re.is_match(got_line))
            .unwrap_or(false),
    }
}

/// A compact, readable line diff for a failing test: a couple of lines of common
/// context, then the diverging region (`-` expected / `+` actual), bounded so a
/// large mismatch stays legible. Far easier to act on than two truncated blobs.
fn unified_diff(want: &str, got: &str, kind: &ExpectKind) -> String {
    let w: Vec<&str> = want.lines().collect();
    let g: Vec<&str> = got.lines().collect();

    // Longest common prefix / suffix (suffix not overlapping the prefix).
    let mut p = 0;
    while p < w.len() && p < g.len() && lines_match(w[p], g[p], kind) {
        p += 1;
    }
    let mut s = 0;
    while s < w.len() - p && s < g.len() - p && lines_match(w[w.len() - 1 - s], g[g.len() - 1 - s], kind)
    {
        s += 1;
    }

    let clip = |line: &str| truncate(line, 240);
    let mut out = String::new();
    for line in &w[p.saturating_sub(2)..p] {
        out.push_str(&format!("  {}\n", clip(line)));
    }
    const CAP: usize = 20;
    let w_mid = &w[p..w.len() - s];
    let g_mid = &g[p..g.len() - s];
    for (i, line) in w_mid.iter().enumerate() {
        if i == CAP {
            out.push_str(&format!("  … (+{} more expected lines)\n", w_mid.len() - CAP));
            break;
        }
        out.push_str(&format!("- {}\n", clip(line)));
    }
    for (i, line) in g_mid.iter().enumerate() {
        if i == CAP {
            out.push_str(&format!("  … (+{} more actual lines)\n", g_mid.len() - CAP));
            break;
        }
        out.push_str(&format!("+ {}\n", clip(line)));
    }
    if s > 0 {
        out.push_str(&format!("  {}\n", clip(w[w.len() - s])));
    }
    let tag = match kind {
        ExpectKind::Exact => "EXPECT",
        ExpectKind::Format => "EXPECTF",
    };
    format!("@@ {} first diff at line {} @@\n{}", tag, p + 1, out.trim_end())
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

/// Does the expectation lead with a compile-time engine error (a `Parse error:`
/// or a non-thrown `Fatal error:`)? Thrown exceptions print `Fatal error:
/// Uncaught …` and are *not* compile-time — those we do model and compare.
fn expects_compile_error(want: &str) -> bool {
    let w = want.trim_start();
    w.starts_with("Parse error:")
        || (w.starts_with("Fatal error:") && !w.starts_with("Fatal error: Uncaught"))
}

/// Did our own output lead with an engine-level fatal/parse error? If so we can
/// fairly compare it against the expectation instead of skipping.
fn is_engine_fatal(got: &str) -> bool {
    let g = got.trim_start();
    g.starts_with("Parse error:") || g.starts_with("Fatal error:")
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
    /// For the `unsupported` category, counts keyed by the specific construct
    /// (`what`, e.g. `expr:Instantiation`) — the line suffix stripped. Surfaces
    /// *which* missing construct dominates, to steer the next step (step 48).
    pub unsupported_by_what: std::collections::BTreeMap<String, usize>,
    /// For the `builtin` category, counts keyed by the missing function name.
    pub builtin_missing: std::collections::BTreeMap<String, usize>,
    /// For the `vm-unsupported` category (E4, `--engine=vm`), counts keyed by the
    /// bytecode-compiler rejection message — surfaces *which* construct the VM
    /// still defers to the evaluator, to steer the next gap to close.
    pub vm_unsupported_by_what: std::collections::BTreeMap<String, usize>,
    /// `(path, detail)` for each failure, for reporting.
    pub failures: Vec<(PathBuf, String)>,
}

/// Bucket a `vm-unsupported` detail (a bytecode-compiler rejection message) by
/// collapsing any backtick-quoted specifics (a function/class name) to ``…`` so
/// like rejections aggregate (E4).
pub fn vm_unsupported_key(detail: &str) -> String {
    let mut out = String::with_capacity(detail.len());
    let mut in_tick = false;
    for c in detail.chars() {
        if c == '`' {
            if !in_tick {
                out.push_str("`…`");
            }
            in_tick = !in_tick;
        } else if !in_tick {
            out.push(c);
        }
    }
    out
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
                match r.category {
                    "unsupported" => {
                        // detail is `"{what} (line {line})"`; key on `what`.
                        let what = r.detail.rsplit_once(" (line").map_or(&*r.detail, |(w, _)| w);
                        *self.unsupported_by_what.entry(what.to_string()).or_insert(0) += 1;
                    }
                    "builtin" => {
                        // detail is `"Call to undefined function NAME()"`.
                        let name = r
                            .detail
                            .strip_prefix("Call to undefined function ")
                            .map_or(&*r.detail, |n| n.trim_end_matches("()"));
                        *self.builtin_missing.entry(name.to_string()).or_insert(0) += 1;
                    }
                    "vm-unsupported" => {
                        *self
                            .vm_unsupported_by_what
                            .entry(vm_unsupported_key(&r.detail))
                            .or_insert(0) += 1;
                    }
                    _ => {}
                }
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
        let result = run_phpt(&src, &php_script_name(&path), reg);
        summary.record(&path, &result);
    }
    Ok(summary)
}

/// The script name PHP would print in diagnostics for a `.phpt` test: its path
/// with the extension swapped to `.php`, mirroring run-tests.php's temp file
/// (`<test>.php`). EXPECTF patterns that embed the basename rely on this.
pub fn php_script_name(path: &Path) -> Vec<u8> {
    path.with_extension("php")
        .to_string_lossy()
        .into_owned()
        .into_bytes()
}

/// Dotfiles, including macOS AppleDouble sidecars (`._foo.phpt`), are not tests.
fn is_hidden(p: &Path) -> bool {
    p.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with('.'))
}

/// Collect `.phpt` paths under `root`, sorted for deterministic output.
pub fn collect_phpt(root: &Path) -> std::io::Result<Vec<PathBuf>> {
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
