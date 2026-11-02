//! GitHub issues #1 and #2 (S-187, 2026-11-02): the CLI as a process.
//!
//! #1 — `phpr` held every byte of program output in memory until the script
//! ended; `flush()` was a no-op and a killed process lost its output. PHP's CLI
//! SAPI writes each chunk straight to fd 1 (`sapi_cli_ub_write`, implicit
//! flush). The dente kills the process from inside the script right after an
//! `echo`, an `fwrite(STDOUT)` and a `flush()`: the bytes must already be out.
//!
//! #2 — `fread(STDIN, 1)` pulled up to 8 KiB into a buffer that `stream_select`
//! could not see, so bytes that had arrived together were reported as "no data".
//! PHP counts buffered bytes as readable. The dente feeds `abc` in one write.
//!
//! Both drive the real binary in a subprocess (`CARGO_BIN_EXE_phpr`, env built
//! from scratch as in absent_eq_one.rs) and judge stdout byte for byte; the
//! expected strings are the oracle's (php 8.5.7 CLI).
use std::io::Write;
use std::process::{Command, Stdio};

fn phpr() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_phpr"));
    cmd.env_clear().env("PATH", "/usr/bin:/bin");
    cmd
}

#[test]
fn output_reaches_stdout_before_a_sigkill() {
    let out = phpr()
        .arg("-r")
        .arg("echo \"before kill\\n\"; fwrite(STDOUT, \"direct\\n\"); echo 'no newline'; flush(); posix_kill(getmypid(), 9);")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .expect("spawn phpr");
    assert!(!out.status.success(), "the script kills itself: {:?}", out.status);
    assert_eq!(String::from_utf8_lossy(&out.stdout), "before kill\ndirect\nno newline");
}

#[test]
fn stream_select_counts_buffered_stdin_bytes() {
    let script = "$got = '';
        for ($i = 0; $i < 3; $i++) {
            $r = [STDIN]; $w = $e = null;
            if (stream_select($r, $w, $e, 0, 200000) < 1) { break; }
            $got .= fread(STDIN, 1);
        }
        echo \"read: '\", $got, \"'\\n\";";
    let mut child = phpr()
        .arg("-r")
        .arg(script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn phpr");
    // One write of three bytes, then the pipe stays open until the child is done
    // reading (the handle is dropped only after wait_with_output collects stdout).
    let mut stdin = child.stdin.take().expect("piped stdin");
    stdin.write_all(b"abc").expect("write abc");
    stdin.flush().expect("flush");
    let out = child.wait_with_output().expect("wait phpr");
    drop(stdin);
    assert!(out.status.success(), "{:?}", out.status);
    assert_eq!(String::from_utf8_lossy(&out.stdout), "read: 'abc'\n");
}
