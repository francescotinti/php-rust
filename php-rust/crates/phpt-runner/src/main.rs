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

use std::path::Path;
use std::process::{Command, ExitCode};

use php_builtins::registry;
use phpt_runner::{collect_phpt, php_script_name, run_path, run_phpt, Status, Summary};

/// The recursive-descent front-end (mago) and our tree-walking evaluator both
/// recurse on the native stack, so pathological tests — e.g. PHP's own
/// `bug64660.phpt`, thousands of nested `[` — can exhaust the default 8 MiB
/// main stack. Run the whole job on a worker thread with a generous stack so
/// such tests are handled (parsed deeply, or run) rather than aborting the run.
const WORKER_STACK: usize = 1 << 30; // 1 GiB

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();

    // Hidden child mode: run exactly one test and serialise its result, so the
    // parent (`--isolate`) can survive a crash here as a non-zero exit status.
    if let Some(pos) = args.iter().position(|a| a == "--run-one") {
        let path = args.get(pos + 1).cloned();
        return run_one_child(path);
    }

    let mut list_fails = false;
    let mut isolate = false;
    args.retain(|a| match a.as_str() {
        "--list-fails" => {
            list_fails = true;
            false
        }
        "--isolate" => {
            isolate = true;
            false
        }
        _ => true,
    });

    if args.is_empty() {
        eprintln!("usage: phpt-runner [--list-fails] [--isolate] <path>...");
        return ExitCode::from(2);
    }

    if isolate {
        // The parent only spawns children, so it needs no large stack itself.
        return run_isolated(&args, list_fails);
    }

    // In-process (fast) path: run the whole job on a generous-stack worker.
    std::thread::Builder::new()
        .stack_size(WORKER_STACK)
        .spawn(move || run_in_process(&args, list_fails))
        .expect("spawn worker")
        .join()
        .expect("worker panicked")
}

/// Run every test in-process under one big-stack worker thread (the default).
fn run_in_process(args: &[String], list_fails: bool) -> ExitCode {
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
    print_summary(&total, list_fails);
    exit_for(&total)
}

/// Run each test in its own child process (`--run-one`). A child that exits
/// abnormally (signal from a stack overflow, or a panic) is recorded as one
/// FAIL with the cause, instead of aborting the whole batch (DevEx hardening).
fn run_isolated(args: &[String], list_fails: bool) -> ExitCode {
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
        for path in paths {
            let out = Command::new(&exe).arg("--run-one").arg(&path).output();
            match out {
                // Clean run: the child serialised a result on stdout.
                Ok(o) if o.status.success() => {
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
                            *total.skip_by_category.entry(cat.to_string()).or_insert(0) += 1;
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
                Ok(o) => {
                    total.fail += 1;
                    total
                        .failures
                        .push((path, format!("isolated worker crashed ({})", o.status)));
                }
                Err(e) => {
                    total.fail += 1;
                    total.failures.push((path, format!("spawn failed: {e}")));
                }
            }
        }
    }
    print_summary(&total, list_fails);
    exit_for(&total)
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
    into.failures.extend(other.failures);
}

fn print_summary(s: &Summary, list_fails: bool) {
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
