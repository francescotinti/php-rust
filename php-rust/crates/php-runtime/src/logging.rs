//! Developer logging for the runtime — the [`log`] facade plus a [`log4rs`]
//! backend installed by the binaries (`phpr`, `phpt-runner`, the dev server).
//!
//! Logging is **off by default**: when no level is requested no global logger is
//! installed, so every `log::trace!`/`debug!`/… call site collapses to a cheap
//! disabled-level branch and corpus runs and oracle comparisons are unaffected.
//! It is enabled per-run through environment variables, and **all output goes to
//! stderr or a file — never stdout**, because stdout is the interpreted program's
//! output, compared byte-for-byte against PHP.
//!
//! - `PHPR_LOG` — level: `error` | `warn` | `info` | `debug` | `trace`
//!   (unset / empty / `off` = disabled). Anything else is treated as `debug`.
//! - `PHPR_LOG_FILE` — append to this file instead of stderr.
//! - `PHPR_LOG_CONFIG` — a [`log4rs`] YAML config file; when set it *fully*
//!   overrides the two variables above (the robust, per-target-filterable path).
//!
//! Subsystems log under stable target names so a YAML config can filter them
//! independently: `phpr::gc` (object sweep / destructors), `phpr::call`
//! (function/method entry), `phpr::exc` (exception unwinding), `phpr::include`
//! (include / require / eval), `phpr::compile`, `phpr::run` (top-level run).

use std::sync::Once;

use log::LevelFilter;
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

static INIT: Once = Once::new();

/// `time level target - message`, e.g. `12:00:01.234 DEBUG phpr::gc - sweep …`.
const PATTERN: &str = "{d(%H:%M:%S%.3f)} {h({l:<5})} {t} - {m}{n}";

/// Install the global logger once, honouring the `PHPR_LOG*` environment. Safe to
/// call from several entry points; only the first call takes effect.
pub fn init() {
    INIT.call_once(install);
}

fn install() {
    // A full YAML config wins outright — the escape hatch for per-target levels,
    // rolling files, multiple appenders, etc.
    if let Some(path) = std::env::var("PHPR_LOG_CONFIG").ok().filter(|p| !p.is_empty()) {
        let _ = log4rs::init_file(&path, Default::default());
        return;
    }
    let level = match std::env::var("PHPR_LOG").ok().as_deref().map(str::trim) {
        None | Some("") | Some("off") => return, // disabled: no logger installed
        Some("error") => LevelFilter::Error,
        Some("warn") => LevelFilter::Warn,
        Some("info") => LevelFilter::Info,
        Some("debug") => LevelFilter::Debug,
        Some("trace") => LevelFilter::Trace,
        Some(_) => LevelFilter::Debug,
    };
    // A file appender when `PHPR_LOG_FILE` is set and buildable, else stderr.
    let to_file = std::env::var("PHPR_LOG_FILE")
        .ok()
        .filter(|p| !p.is_empty())
        .and_then(|path| {
            FileAppender::builder()
                .encoder(Box::new(PatternEncoder::new(PATTERN)))
                .build(&path)
                .ok()
                .and_then(|fa| {
                    Config::builder()
                        .appender(Appender::builder().build("out", Box::new(fa)))
                        .build(Root::builder().appender("out").build(level))
                        .ok()
                })
        });
    let config = to_file.unwrap_or_else(|| {
        let console = ConsoleAppender::builder()
            .target(Target::Stderr)
            .encoder(Box::new(PatternEncoder::new(PATTERN)))
            .build();
        Config::builder()
            .appender(Appender::builder().build("out", Box::new(console)))
            .build(Root::builder().appender("out").build(level))
            .expect("static stderr log config is valid")
    });
    let _ = log4rs::init_config(config);
}
