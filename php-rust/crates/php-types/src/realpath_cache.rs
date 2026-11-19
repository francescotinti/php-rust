//! S-192 — resolved-path cache (criterion `wp192-harness/s192-criterio-realpath.md`).
//!
//! PHP resolves `realpath()` and every `include` path through the realpath
//! cache of `Zend/zend_virtual_cwd.c` (TTL 120 s), emptied by `unlink`,
//! `rename`, `rmdir`, `chdir` and `clearstatcache(true)`. Without it each
//! resolution costs one system call per path component. This is the same
//! cache in a VALIDATED form: a hit is served only while one `stat` of the
//! given path still succeeds, so a file deleted behind the engine's back is
//! resolved afresh (never staler than PHP's cache, and fresher than it).
//! Relative paths depend on the working directory and are never cached.
use std::cell::RefCell;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// PHP's `REALPATH_CACHE_TTL`.
const TTL: Duration = Duration::from_secs(120);
/// Bound on the table: past it the whole table is dropped (PHP bounds it by bytes).
const MAX_ENTRIES: usize = 16_384;

thread_local! {
    static CACHE: RefCell<rustc_hash::FxHashMap<Vec<u8>, (Vec<u8>, Instant)>> = RefCell::new(Default::default());
}

/// `std::fs::canonicalize` through the cache. Errors are never cached.
pub fn canonicalize(p: &Path) -> std::io::Result<PathBuf> {
    if !p.is_absolute() {
        return std::fs::canonicalize(p);
    }
    let key = p.as_os_str().as_bytes();
    let hit = CACHE.with(|c| {
        c.borrow().get(key).filter(|(_, at)| at.elapsed() < TTL).map(|(real, _)| real.clone())
    });
    if let Some(real) = hit {
        // One system call instead of one per component; it follows links, so a
        // dangling link fails here exactly as the full resolution would.
        if std::fs::metadata(p).is_ok() {
            return Ok(PathBuf::from(OsStr::from_bytes(&real)));
        }
    }
    let real = std::fs::canonicalize(p)?;
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() >= MAX_ENTRIES {
            c.clear();
        }
        c.insert(key.to_vec(), (real.as_os_str().as_bytes().to_vec(), Instant::now()));
    });
    Ok(real)
}

/// Empty the cache (PHP's `php_clear_stat_cache(1, NULL, 0)`).
pub fn clear() {
    CACHE.with(|c| c.borrow_mut().clear());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("phpr-realpath-cache-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::canonicalize(&d).unwrap()
    }

    #[test]
    fn second_resolution_is_the_same_and_a_deleted_file_fails() {
        let d = scratch("del");
        let f = d.join("a.txt");
        std::fs::write(&f, b"x").unwrap();
        let first = canonicalize(&f).unwrap();
        assert_eq!(first, canonicalize(&f).unwrap());
        std::fs::remove_file(&f).unwrap(); // deleted behind the cache's back
        assert!(canonicalize(&f).is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn a_retargeted_link_is_seen_after_clear() {
        let d = scratch("link");
        let (a, b, l) = (d.join("a"), d.join("b"), d.join("l"));
        std::fs::create_dir(&a).unwrap();
        std::fs::create_dir(&b).unwrap();
        std::os::unix::fs::symlink(&a, &l).unwrap();
        assert_eq!(canonicalize(&l).unwrap(), a);
        std::fs::remove_file(&l).unwrap();
        std::os::unix::fs::symlink(&b, &l).unwrap();
        clear(); // what unlink/rename/rmdir/chdir/clearstatcache do
        assert_eq!(canonicalize(&l).unwrap(), b);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn a_relative_path_is_never_cached() {
        clear();
        let _ = canonicalize(Path::new("."));
        assert_eq!(CACHE.with(|c| c.borrow().len()), 0);
    }
}
