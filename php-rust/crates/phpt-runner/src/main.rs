//! `phpt-runner` — run PHP `.phpt` files through the Rust evaluator (step 6).
//!
//! Usage:
//!   phpt-runner <path>...        # files and/or directories (searched recursively)
//!   phpt-runner --list-fails ... # also print each failing test's diff
//!   phpt-runner --isolate ...    # run each test in its own child process, so a
//!                                # crash (stack overflow / panic) is contained as
//!                                # one FAIL instead of aborting the whole batch
//!
//! Exit code is non-zero iff at least one test FAILED (skips never fail the run).

use std::io::Read;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

use php_builtins::registry;
use phpt_runner::{
    collect_phpt, php_script_name, run_path, run_phpt, Status, Summary,
};

/// The recursive-descent front-end (mago) and our tree-walking evaluator both
/// recurse on the native stack, so pathological tests — e.g. PHP's own
/// `bug64660.phpt`, thousands of nested `[` — can exhaust the default 8 MiB
/// main stack. Run the whole job on a worker thread with a generous stack so
/// such tests are handled (parsed deeply, or run) rather than aborting the run.
const WORKER_STACK: usize = 1 << 30; // 1 GiB

/// Per-test wall-clock cap for `--isolate`. A `.phpt` that drives our evaluator
/// into an unbounded loop (e.g. `while (true) $a[] = 1;`) would otherwise run
/// forever, exhausting RAM and freezing the host — there is no `timeout(1)` on
/// macOS to lean on. We kill the child past this budget and record one FAIL.
/// Override with `PHPT_TIMEOUT_SECS` (0 disables the cap).
fn isolate_timeout() -> Option<Duration> {
    match std::env::var("PHPT_TIMEOUT_SECS").ok().and_then(|s| s.parse::<u64>().ok()) {
        Some(0) => None,
        Some(n) => Some(Duration::from_secs(n)),
        None => Some(Duration::from_secs(10)),
    }
}

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();

    // Hidden child mode: run exactly one test and serialise its result, so the
    // parent (`--isolate`) can survive a crash here as a non-zero exit status.
    if let Some(pos) = args.iter().position(|a| a == "--run-one") {
        let path = args.get(pos + 1).cloned();
        return run_one_child(path);
    }

    let mut list_fails = false;
    let mut list_skips = false;
    let mut isolate = false;
    args.retain(|a| match a.as_str() {
        "--list-fails" => {
            list_fails = true;
            false
        }
        "--list-skips" => {
            list_skips = true;
            false
        }
        "--isolate" => {
            isolate = true;
            false
        }
        _ => true,
    });

    if args.is_empty() {
        eprintln!("usage: phpt-runner [--list-fails] [--list-skips] [--isolate] <path>...");
        return ExitCode::from(2);
    }

    if isolate {
        // The parent only spawns children, so it needs no large stack itself.
        return run_isolated(&args, list_fails, list_skips);
    }

    // In-process (fast) path: run the whole job on a generous-stack worker.
    std::thread::Builder::new()
        .stack_size(WORKER_STACK)
        .spawn(move || run_in_process(&args, list_fails, list_skips))
        .expect("spawn worker")
        .join()
        .expect("worker panicked")
}

/// Run every test in-process under one big-stack worker thread (the default).
fn run_in_process(args: &[String], list_fails: bool, list_skips: bool) -> ExitCode {
    let reg = registry();
    let mut total = Summary::default();
    for arg in args {
        match run_path(Path::new(arg), &reg) {
            Ok(s) => merge(&mut total, s),
            Err(e) => {
                eprintln!("error reading {arg}: {e}");
                return ExitCode::from(2);
            }
        }
    }
    print_summary(&total, list_fails, list_skips);
    exit_for(&total)
}

/// Run each test in its own child process (`--run-one`). A child that exits
/// abnormally (signal from a stack overflow, or a panic) is recorded as one
/// FAIL with the cause, instead of aborting the whole batch (DevEx hardening).
fn run_isolated(args: &[String], list_fails: bool, list_skips: bool) -> ExitCode {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("cannot find own executable: {e}");
            return ExitCode::from(2);
        }
    };
    let mut total = Summary::default();
    for arg in args {
        let paths = match collect_phpt(Path::new(arg)) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("error reading {arg}: {e}");
                return ExitCode::from(2);
            }
        };
        let timeout = isolate_timeout();
        for path in paths {
            let out = run_one_isolated(&exe, &path, timeout);
            match out {
                // Clean run: the child serialised a result on stdout.
                Ok(IsolatedRun::Done(o)) if o.status.success() => {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    let (header, detail) = match stdout.split_once('\n') {
                        Some((h, d)) => (h, d.to_string()),
                        None => (stdout.as_ref(), String::new()),
                    };
                    let mut fields = header.split('\t');
                    match (fields.next(), fields.next()) {
                        (Some("PASS"), _) => total.pass += 1,
                        (Some("FAIL"), _) => {
                            total.fail += 1;
                            total.failures.push((path, detail));
                        }
                        (Some("SKIP"), Some(cat)) => {
                            total.skip += 1;
                            total.skips.push((path.clone(), format!("{cat}\t{detail}")));
                            *total.skip_by_category.entry(cat.to_string()).or_insert(0) += 1;
                            // Same breakdown as the in-process path, so --isolate
                            // (crash-surviving) also reports it (step 48).
                            match cat {
                                "unsupported" => {
                                    let what = detail
                                        .rsplit_once(" (line")
                                        .map_or(detail.as_str(), |(w, _)| w);
                                    *total
                                        .unsupported_by_what
                                        .entry(what.to_string())
                                        .or_insert(0) += 1;
                                }
                                "builtin" => {
                                    let name = detail
                                        .strip_prefix("Call to undefined function ")
                                        .map_or(detail.as_str(), |n| n.trim_end_matches("()"));
                                    *total.builtin_missing.entry(name.to_string()).or_insert(0) += 1;
                                }
                                "vm-unsupported" => {
                                    *total
                                        .vm_unsupported_by_what
                                        .entry(phpt_runner::vm_unsupported_key(&detail))
                                        .or_insert(0) += 1;
                                }
                                _ => {}
                            }
                        }
                        _ => {
                            total.fail += 1;
                            total
                                .failures
                                .push((path, "isolated worker: unparseable result".to_string()));
                        }
                    }
                }
                // Abnormal exit: signal (e.g. SIGABRT from stack overflow) or panic.
                Ok(IsolatedRun::Done(o)) => {
                    total.fail += 1;
                    total
                        .failures
                        .push((path, format!("isolated worker crashed ({})", o.status)));
                }
                // Ran past the wall-clock budget — almost always an unbounded loop
                // in the evaluator. Killed before it could exhaust host memory.
                Ok(IsolatedRun::TimedOut) => {
                    total.fail += 1;
                    total
                        .failures
                        .push((path, "isolated worker timed out (likely infinite loop)".to_string()));
                }
                Err(e) => {
                    total.fail += 1;
                    total.failures.push((path, format!("spawn failed: {e}")));
                }
            }
        }
    }
    print_summary(&total, list_fails, list_skips);
    exit_for(&total)
}

/// Outcome of one isolated child: it either finished (cleanly or via crash) and
/// we hold its captured output, or it blew the wall-clock budget and was killed.
enum IsolatedRun {
    Done(std::process::Output),
    TimedOut,
}

/// Spawn one `--run-one` child and wait for it, enforcing `timeout` if set.
/// stdout is drained on a side thread so a child that emits more than one pipe
/// buffer (large diff) can't deadlock against our wait/kill loop.
fn run_one_isolated(
    exe: &Path,
    path: &Path,
    timeout: Option<Duration>,
) -> std::io::Result<IsolatedRun> {
    let mut cmd = Command::new(exe);
    cmd.arg("--run-one").arg(path);
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    // Drain stdout concurrently; the reader returns when the child closes it.
    let stdout = child.stdout.take();
    let reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut s) = stdout {
            let _ = s.read_to_end(&mut buf);
        }
        buf
    });

    let start = Instant::now();
    let status = loop {
        match child.try_wait()? {
            Some(status) => break status,
            None => {
                if let Some(limit) = timeout {
                    if start.elapsed() >= limit {
                        let _ = child.kill();
                        let _ = child.wait();
                        let _ = reader.join();
                        return Ok(IsolatedRun::TimedOut);
                    }
                }
                std::thread::sleep(Duration::from_millis(25));
            }
        }
    };

    let stdout = reader.join().unwrap_or_default();
    Ok(IsolatedRun::Done(std::process::Output {
        status,
        stdout,
        stderr: Vec::new(),
    }))
}

/// Child mode: run a single test on a big-stack worker and serialise its result
/// as `STATUS\tCATEGORY\n` followed by the (possibly multi-line) detail. A crash
/// or panic here exits the process abnormally, which the parent detects.
fn run_one_child(path: Option<String>) -> ExitCode {
    let Some(path) = path else {
        eprintln!("--run-one needs a path");
        return ExitCode::from(2);
    };
    let src = match std::fs::read(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("read {path}: {e}");
            return ExitCode::from(2);
        }
    };
    let name = php_script_name(std::path::Path::new(&path));
    let result = std::thread::Builder::new()
        .stack_size(WORKER_STACK)
        .spawn(move || {
            let reg = registry();
            run_phpt(&src, &name, &reg)
        })
        .expect("spawn worker")
        .join()
        .expect("worker panicked");
    let status = match result.status {
        Status::Pass => "PASS",
        Status::Fail => "FAIL",
        Status::Skip => "SKIP",
    };
    println!("{status}\t{}", result.category);
    print!("{}", result.detail);
    ExitCode::SUCCESS
}

fn exit_for(total: &Summary) -> ExitCode {
    if total.fail == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn merge(into: &mut Summary, other: Summary) {
    into.pass += other.pass;
    into.fail += other.fail;
    into.skip += other.skip;
    for (k, v) in other.skip_by_category {
        *into.skip_by_category.entry(k).or_insert(0) += v;
    }
    for (k, v) in other.unsupported_by_what {
        *into.unsupported_by_what.entry(k).or_insert(0) += v;
    }
    for (k, v) in other.builtin_missing {
        *into.builtin_missing.entry(k).or_insert(0) += v;
    }
    for (k, v) in other.vm_unsupported_by_what {
        *into.vm_unsupported_by_what.entry(k).or_insert(0) += v;
    }
    into.failures.extend(other.failures);
    into.skips.extend(other.skips);
}

/// Print the top entries of a count map (descending, ties broken by key).
fn print_top(title: &str, map: &std::collections::BTreeMap<String, usize>, top: usize) {
    if map.is_empty() {
        return;
    }
    println!("\n{title}");
    let mut v: Vec<_> = map.iter().collect();
    v.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (k, n) in v.into_iter().take(top) {
        println!("  {n:>6}  {k}");
    }
}

fn print_summary(s: &Summary, list_fails: bool, list_skips: bool) {
    // With `--list-skips`, emit every skipped test's path + `category\tdetail` so a
    // specific bucket (e.g. `unsupported`/assignment target) can be inspected.
    if list_skips {
        println!("\n=== skips: {} ===", s.skips.len());
        for (path, detail) in &s.skips {
            println!("{}\t{}", path.display(), detail);
        }
    }
    let total = s.total();
    let runnable = s.pass + s.fail;
    println!("\n=== phpt-runner ===");
    println!("total:   {total}");
    println!("pass:    {}", s.pass);
    println!("fail:    {}", s.fail);
    println!("skip:    {}", s.skip);
    if runnable > 0 {
        let pct = 100.0 * s.pass as f64 / runnable as f64;
        println!("pass rate (of runnable): {:.1}% ({}/{})", pct, s.pass, runnable);
    }

    if !s.skip_by_category.is_empty() {
        println!("\nskips by category:");
        let mut cats: Vec<_> = s.skip_by_category.iter().collect();
        cats.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        for (cat, n) in cats {
            println!("  {n:>6}  {cat}");
        }
    }

    // Breakdowns that steer prioritisation (step 48): which specific construct
    // dominates the `unsupported` bucket, and which builtins are most wanted.
    print_top("unsupported by construct (top 20):", &s.unsupported_by_what, 20);
    print_top("missing builtins (top 20):", &s.builtin_missing, 20);
    // VM-only (E4): which constructs the bytecode compiler still defers to eval.
    print_top("vm-unsupported by construct (top 20):", &s.vm_unsupported_by_what, 20);

    if s.fail > 0 {
        println!("\nfailures: {}", s.fail);
        if list_fails {
            for (path, detail) in &s.failures {
                println!("\n--- {} ---\n{detail}", path.display());
            }
        } else {
            for (path, _) in &s.failures {
                println!("  {}", path.display());
            }
            println!("(pass --list-fails for diffs)");
        }
    }
}
