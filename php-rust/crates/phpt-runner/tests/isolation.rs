//! `--isolate` mode: each `.phpt` runs in its own child process, so a crash
//! (a panic, OOM, or a runtime that aborts) is contained as one reported failure
//! instead of aborting the whole batch (DevEx hardening). The bytecode VM runs
//! pure PHP recursion iteratively (an explicit frame stack), so unbounded
//! recursion no longer overflows the native stack — it is caught as a clean
//! "Maximum call stack depth" fatal, which the runner still reports per-test.

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

    // `aa_*` sorts first: an unbounded closure recursion. The VM contains it as a
    // clean "Maximum call stack depth" fatal (an explicit frame stack, no native
    // overflow), so it surfaces as a normal per-test failure, not a host crash.
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
    // The batch completes: the normal test passed (so the runaway did not abort
    // the run), and the runaway test is recorded as one contained failure with the
    // VM's call-depth fatal in its detail.
    assert!(stdout.contains("pass:    1"), "stdout:\n{stdout}");
    assert!(stdout.contains("fail:    1"), "stdout:\n{stdout}");
    assert!(
        stdout.contains("Maximum call stack depth"),
        "expected a contained call-depth fatal, stdout:\n{stdout}"
    );
}
