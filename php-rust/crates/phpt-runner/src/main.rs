//! `phpt-runner` — run PHP `.phpt` files through the Rust evaluator (step 6).
//!
//! Usage:
//!   phpt-runner <path>...        # files and/or directories (searched recursively)
//!   phpt-runner --list-fails ... # also print each failing test's diff
//!
//! Exit code is non-zero iff at least one test FAILED (skips never fail the run).

use std::path::Path;
use std::process::ExitCode;

use php_builtins::registry;
use phpt_runner::{run_path, Summary};

/// The recursive-descent front-end (mago) and our tree-walking evaluator both
/// recurse on the native stack, so pathological tests — e.g. PHP's own
/// `bug64660.phpt`, thousands of nested `[` — can exhaust the default 8 MiB
/// main stack. Run the whole job on a worker thread with a generous stack so
/// such tests are handled (parsed deeply, or run) rather than aborting the run.
const WORKER_STACK: usize = 1 << 30; // 1 GiB

fn main() -> ExitCode {
    std::thread::Builder::new()
        .stack_size(WORKER_STACK)
        .spawn(run)
        .expect("spawn worker")
        .join()
        .expect("worker panicked")
}

fn run() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut list_fails = false;
    args.retain(|a| {
        if a == "--list-fails" {
            list_fails = true;
            false
        } else {
            true
        }
    });

    if args.is_empty() {
        eprintln!("usage: phpt-runner [--list-fails] <path>...");
        return ExitCode::from(2);
    }

    let reg = registry();
    let mut total = Summary::default();
    for arg in &args {
        match run_path(Path::new(arg), &reg) {
            Ok(s) => merge(&mut total, s),
            Err(e) => {
                eprintln!("error reading {arg}: {e}");
                return ExitCode::from(2);
            }
        }
    }

    print_summary(&total, list_fails);
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
