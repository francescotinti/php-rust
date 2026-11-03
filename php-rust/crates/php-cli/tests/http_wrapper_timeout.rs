//! §3.39 (S-189, 2026-11-03): the `http://` stream wrapper must honour the
//! stream-context `timeout` (and, absent that, `default_socket_timeout`): a
//! server that accepts the connection and never answers made phpr hang for
//! ever, which under `php-server --axum` parks a worker for good. Oracle probe
//! (php 8.5.7 CLI, silent TCP server): context `timeout => 1.0` → `bool(false)`
//! after 1004 ms with the Warning "file_get_contents(URL): Failed to open
//! stream: HTTP request failed!"; `-d default_socket_timeout=2` → 2000 ms.
//! The dente drives the real binary (`CARGO_BIN_EXE_phpr`, env from scratch as
//! in cli_streams.rs) against a silent listener owned by the test. Only the
//! context path is exercised end to end (the 60 s default is unit-tested in
//! php-builtins `http_timeout`): RED on the pre-cure tree (hang → the wall
//! bound), GREEN with the cure.
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn phpr() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_phpr"));
    cmd.env_clear().env("PATH", "/usr/bin:/bin");
    cmd
}

#[test]
fn http_wrapper_times_out_on_a_silent_server() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    // The silent server: accept, hold the socket, answer nothing, release when told.
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let holder = std::thread::spawn(move || {
        let held = listener.accept().ok();
        let _ = release_rx.recv_timeout(Duration::from_secs(30));
        drop(held);
    });
    let script = format!(
        "$c = stream_context_create(['http' => ['timeout' => 1.0]]);\
         $t = microtime(true);\
         $r = file_get_contents('http://127.0.0.1:{port}/x', false, $c);\
         var_dump($r);\
         echo 'ms=', (int) ((microtime(true) - $t) * 1000), \"\\n\";"
    );
    let t0 = Instant::now();
    let out = phpr()
        .arg("-r")
        .arg(&script)
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn phpr");
    let wall = t0.elapsed();
    let _ = release_tx.send(());
    let _ = holder.join();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        wall < Duration::from_secs(8),
        "phpr did not give up on the silent server within 8 s (§3.39 hang): {wall:?}"
    );
    assert!(out.status.success(), "status {:?}\nstdout:\n{stdout}\nstderr:\n{stderr}", out.status);
    assert!(stdout.contains("bool(false)"), "expected false:\n{stdout}");
    assert!(
        stdout.contains(&format!(
            "file_get_contents(http://127.0.0.1:{port}/x): Failed to open stream: HTTP request failed!"
        )),
        "expected the oracle's timeout Warning:\n{stdout}\n{stderr}"
    );
    let ms: u64 = stdout
        .lines()
        .find_map(|l| l.strip_prefix("ms=").and_then(|v| v.trim().parse().ok()))
        .expect("ms= line");
    assert!((900..4000).contains(&ms), "timeout fired at {ms} ms, expected ≈1000 (context timeout 1.0)");
}
