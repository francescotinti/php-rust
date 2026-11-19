//! S-192 — the builtins that read or empty the resolved-path cache
//! (`php_types::realpath_cache`; criterion `wp192-harness/s192-criterio-realpath.md`).
//! `realpath` resolves through it; `unlink`, `rename`, `rmdir`, `chdir` and
//! `clearstatcache` empty it before running, as PHP's `php_clear_stat_cache`
//! does for the same calls. Registered in `lib.rs` in place of the originals.
use php_runtime::Ctx;
use php_types::{PhpError, PhpStr, Zval};

/// `realpath`: the canonical absolute path (symlinks + `..` resolved), or
/// `false` if any component is missing.
pub fn realpath(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use std::os::unix::ffi::OsStrExt;
    let mut p = crate::file::arg_os_path(argv, ctx);
    // PHP quirk: realpath("") resolves the current directory (Symfony Process
    // builds its cwd through this, and "" must NOT read as failure).
    if p.is_empty() {
        p = std::ffi::OsString::from(".");
    }
    match php_types::realpath_cache::canonicalize(std::path::Path::new(&p)) {
        Ok(abs) => Ok(Zval::Str(PhpStr::new(abs.as_os_str().as_bytes().to_vec()))),
        Err(_) => Ok(Zval::Bool(false)),
    }
}

macro_rules! clearing {
    ($($name:ident),*) => {$(
        pub fn $name(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
            php_types::realpath_cache::clear();
            crate::file::$name(argv, ctx)
        }
    )*};
}
clearing!(unlink, rename, rmdir, chdir, clearstatcache);
