//! Developer logging for the runtime — the [`log`] facade plus a [`log4rs`]
//! backend installed by the binaries (`phpr`, `phpt-runner`, the dev server).
//!
//! Logging is **off by default**: when no level is requested no global logger is
//! installed, so every `log::trace!`/`debug!`/… call site collapses to a cheap
//! disabled-level branch and corpus runs and oracle comparisons are unaffected.
//! **All output goes to stderr or a file — never stdout**, because stdout is the
//! interpreted program's output, compared byte-for-byte against PHP.
//!
//! - `PHPR_LOG` — level: `error` | `warn`(=`warning`) | `info` | `debug` |
//!   `trace`, case-insensitive (unset / empty / `off` = disabled). Anything else
//!   is treated as `debug`.
//! - `PHPR_LOG_FILE` — append to this file instead of stderr.
//! - `PHPR_LOG_CONFIG` — a [`log4rs`] YAML config file; when set it *fully*
//!   overrides the two variables above (the robust, per-target-filterable path).
//!
//! Subsystems log under stable target names so a YAML config can filter them
//! independently: `phpr::gc` (object sweep / destructors), `phpr::call`
//! (function/method entry), `phpr::exc` (exception unwinding), `phpr::include`
//! (include / require / eval), `phpr::compile`, `phpr::run` (top-level run).

use std::fmt;
use std::sync::Once;

use log::LevelFilter;
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

static INIT: Once = Once::new();

/// `time level target - message`, e.g. `12:00:01.234 DEBUG phpr::gc - sweep …`.
const PATTERN: &str = "{d(%H:%M:%S%.3f)} {h({l:<5})} {t} - {m}{n}";

/// Why installing the global logger failed. Only ever produced when logging was
/// actually requested (a level or config file is set) — a disabled logger is a
/// success, not an error.
#[derive(Debug)]
pub enum LoggingError {
    /// The `PHPR_LOG_CONFIG` file or a built appender could not be loaded/created.
    Config(String),
    /// A global logger was already installed (e.g. [`init`] ran twice).
    AlreadyInitialised,
}

impl fmt::Display for LoggingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoggingError::Config(e) => write!(f, "log configuration error: {e}"),
            LoggingError::AlreadyInitialised => write!(f, "a logger was already installed"),
        }
    }
}

impl std::error::Error for LoggingError {}

/// Install the global logger once, honouring the `PHPR_LOG*` environment. Safe to
/// call from several entry points; only the first call takes effect. If logging
/// was requested but could not be installed, a one-line warning is written to
/// stderr (never stdout) and the program continues without logging.
pub fn init() {
    INIT.call_once(|| {
        if let Err(e) = try_init() {
            eprintln!("[phpr] logging disabled: {e}");
        }
    });
}

/// Attempt to install the global logger, returning whether one was installed
/// (`Ok(false)` = logging disabled by configuration, the common default). Unlike
/// [`init`] this is not guarded by a [`Once`], so a caller may retry after a
/// configuration error — though the underlying `log` crate only accepts one
/// successful logger per process.
pub fn try_init() -> Result<bool, LoggingError> {
    // A full YAML config wins outright — the escape hatch for per-target levels,
    // rolling files, multiple appenders, etc.
    if let Some(path) = std::env::var("PHPR_LOG_CONFIG").ok().filter(|p| !p.is_empty()) {
        log4rs::init_file(&path, Default::default()).map_err(|e| LoggingError::Config(e.to_string()))?;
        return Ok(true);
    }
    let Some(level) = parse_level(std::env::var("PHPR_LOG").ok().as_deref()) else {
        return Ok(false); // disabled: no logger installed
    };
    let file = std::env::var("PHPR_LOG_FILE").ok().filter(|p| !p.is_empty());
    let config = build_config(level, file.as_deref())?;
    log4rs::init_config(config).map_err(|_| LoggingError::AlreadyInitialised)?;
    Ok(true)
}

/// Parse a `PHPR_LOG` value into a level filter. `None` means logging is
/// disabled (unset, empty, or an explicit `off`). Matching is case-insensitive
/// and an unrecognised non-empty value falls back to `debug` (so a typo still
/// gives output rather than silence).
pub fn parse_level(spec: Option<&str>) -> Option<LevelFilter> {
    let s = spec?.trim();
    if s.is_empty() {
        return None;
    }
    match s.to_ascii_lowercase().as_str() {
        "off" | "none" => None,
        "error" => Some(LevelFilter::Error),
        "warn" | "warning" => Some(LevelFilter::Warn),
        "info" => Some(LevelFilter::Info),
        "debug" => Some(LevelFilter::Debug),
        "trace" => Some(LevelFilter::Trace),
        _ => Some(LevelFilter::Debug),
    }
}

/// Build a [`log4rs`] config logging at `level` to `file` (when given and
/// creatable) or otherwise to stderr. Never targets stdout.
fn build_config(level: LevelFilter, file: Option<&str>) -> Result<Config, LoggingError> {
    let appender = match file {
        Some(path) => {
            let fa = FileAppender::builder()
                .encoder(Box::new(PatternEncoder::new(PATTERN)))
                .build(path)
                .map_err(|e| LoggingError::Config(e.to_string()))?;
            Appender::builder().build("out", Box::new(fa))
        }
        None => {
            let console = ConsoleAppender::builder()
                .target(Target::Stderr)
                .encoder(Box::new(PatternEncoder::new(PATTERN)))
                .build();
            Appender::builder().build("out", Box::new(console))
        }
    };
    Config::builder()
        .appender(appender)
        .build(Root::builder().appender("out").build(level))
        .map_err(|e| LoggingError::Config(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_disabled_when_unset_empty_or_off() {
        assert!(parse_level(None).is_none());
        assert!(parse_level(Some("")).is_none());
        assert!(parse_level(Some("   ")).is_none());
        assert!(parse_level(Some("off")).is_none());
        assert!(parse_level(Some("OFF")).is_none());
        assert!(parse_level(Some("none")).is_none());
    }

    #[test]
    fn level_parsing_is_case_insensitive_and_trimmed() {
        assert_eq!(parse_level(Some("warn")), Some(LevelFilter::Warn));
        assert_eq!(parse_level(Some("WARN")), Some(LevelFilter::Warn));
        assert_eq!(parse_level(Some("Warning")), Some(LevelFilter::Warn));
        assert_eq!(parse_level(Some("  Debug ")), Some(LevelFilter::Debug));
        assert_eq!(parse_level(Some("TRACE")), Some(LevelFilter::Trace));
        assert_eq!(parse_level(Some("error")), Some(LevelFilter::Error));
        assert_eq!(parse_level(Some("info")), Some(LevelFilter::Info));
    }

    #[test]
    fn unknown_level_falls_back_to_debug() {
        assert_eq!(parse_level(Some("verbose")), Some(LevelFilter::Debug));
    }

    #[test]
    fn build_config_to_stderr_succeeds() {
        // A stderr config must always build (no I/O). Stdout is never a target.
        assert!(build_config(LevelFilter::Debug, None).is_ok());
    }

    #[test]
    fn build_config_to_unwritable_file_errs() {
        // A file under a non-existent directory cannot be created → Config error,
        // surfaced to the caller rather than silently dropped.
        let bogus = "/nonexistent-dir-phpr-xyz/sub/phpr.log";
        assert!(matches!(build_config(LevelFilter::Debug, Some(bogus)), Err(LoggingError::Config(_))));
    }
}
