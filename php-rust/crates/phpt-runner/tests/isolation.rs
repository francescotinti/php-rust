//! `--isolate` mode: each `.phpt` runs in its own child process, so a crash
//! (native stack overflow → SIGABRT, or a panic) is contained as one reported
//! failure instead of aborting the whole batch (DevEx hardening).

use std::path::PathBuf;
use std::process::Command;

fn write(dir: &std::path::Path, name: &str, body: &str) {
    std::fs::write(dir.join(name), body).expect("write fixture");
}

#[test]
fn isolate_contains_a_crashing_test_and_still_runs_the_rest() {
    // A unique scratch dir (no external tempdir crate).
    let dir: PathBuf = std::env::temp_dir().join(format!("phpt_iso_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("mkdir");

    // `aa_*` sorts first: an unbounded closure recursion overflows the native
    // stack (SIGABRT) — a crash vector the in-process runner cannot survive.
    write(
        &dir,
        "aa_crash.phpt",
        "--TEST--\naa\n--FILE--\n<?php $f = function($n) use (&$f){ return $f($n + 1); }; echo $f(0);\n--EXPECT--\nx\n",
    );
    // `zz_*` sorts last: a normal passing test that must still run even though
    // `aa_crash` aborts its own child first.
    write(
        &dir,
        "zz_ok.phpt",
        "--TEST--\nzz\n--FILE--\n<?php echo 1 + 1;\n--EXPECT--\n2\n",
    );

    let out = Command::new(env!("CARGO_BIN_EXE_phpt-runner"))
        .arg("--isolate")
        .arg("--list-fails")
        .arg(&dir)
        .output()
        .expect("spawn phpt-runner");
    let _ = std::fs::remove_dir_all(&dir);

    let stdout = String::from_utf8_lossy(&out.stdout);
    // The batch completes: the normal test passed (so the crash did not abort
    // the run), and the crashing test is recorded as one contained failure.
    assert!(stdout.contains("pass:    1"), "stdout:\n{stdout}");
    assert!(stdout.contains("fail:    1"), "stdout:\n{stdout}");
    assert!(
        stdout.contains("isolated worker crashed"),
        "expected a contained crash, stdout:\n{stdout}"
    );
}
