//! `crypt` (step 64) — one-way string hashing.
//!
//! Mirrors `php_crypt`'s salt-prefix dispatch (`ext/standard/crypt.c`) on top of
//! the `pwhash` crate (glibc-compatible DES / BSDi / MD5 / SHA-256 / SHA-512 /
//! bcrypt). PHP's `*0`/`*1` failure convention is layered on top: an invalid salt
//! yields `"*1"` when the salt began with `*0`, otherwise `"*0"`.

use pwhash::unix;

use php_runtime::Ctx;
use php_types::{convert, PhpError, PhpStr, Zval};

/// PHP_MAX_SALT_LEN — the salt is truncated to this many bytes.
const MAX_SALT_LEN: usize = 123;

// SHA-256/512 crypt round bounds (crypt_sha256.c / crypt_sha512.c).
const ROUNDS_MIN: u64 = 1000;
const ROUNDS_MAX: u64 = 999_999_999;

/// For a `$5$`/`$6$` salt, return the explicit `rounds=N` value if present.
/// PHP rejects an out-of-range count with `NULL` (→ `*0`); we must reject it too,
/// both for fidelity and to stop `pwhash` from spinning through a billion rounds
/// (an effective hang).
fn sha_crypt_rounds_out_of_range(salt: &[u8]) -> bool {
    let is_sha = salt.len() >= 3 && salt[0] == b'$' && (salt[1] == b'5' || salt[1] == b'6') && salt[2] == b'$';
    if !is_sha {
        return false;
    }
    let rest = &salt[3..];
    let prefix = b"rounds=";
    if !rest.starts_with(prefix) {
        return false;
    }
    let digits: &[u8] = &rest[prefix.len()..];
    let end = digits.iter().position(|&c| c == b'$').unwrap_or(digits.len());
    // Only an actual `rounds=N$` clause counts (matches the C `*endp == '$'`).
    if end == digits.len() || end == 0 || !digits[..end].iter().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let n: u64 = std::str::from_utf8(&digits[..end])
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(u64::MAX);
    !(ROUNDS_MIN..=ROUNDS_MAX).contains(&n)
}

/// `crypt(string $string, string $salt): string`.
pub fn crypt(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    if args.len() != 2 {
        return Err(PhpError::ArgumentCountError(format!(
            "crypt() expects exactly 2 arguments, {} given",
            args.len()
        )));
    }
    let password = convert::to_zstr(&args[0], ctx.diags);
    let salt_z = convert::to_zstr(&args[1], ctx.diags);
    let salt = salt_z.as_bytes();
    let salt = &salt[..salt.len().min(MAX_SALT_LEN)];

    // PHP returns "*1" if crypt failed on a salt that started with "*0",
    // otherwise "*0".
    let fallback = || {
        let s: &[u8] = if salt.len() >= 2 && salt[0] == b'*' && salt[1] == b'0' {
            b"*1"
        } else {
            b"*0"
        };
        Ok(Zval::Str(PhpStr::new(s.to_vec())))
    };

    // A "*0"/"*1" salt is an immediate failure token in php_crypt.
    if salt.len() >= 2 && salt[0] == b'*' && (salt[1] == b'0' || salt[1] == b'1') {
        return fallback();
    }

    // Reject an out-of-range SHA round count exactly as PHP does.
    if sha_crypt_rounds_out_of_range(salt) {
        return fallback();
    }

    // pwhash takes the setting string as &str; a non-UTF-8 salt cannot be a valid
    // crypt setting, so treat it as a failure.
    let salt_str = match std::str::from_utf8(salt) {
        Ok(s) => s,
        Err(_) => return fallback(),
    };

    match unix::crypt(password.as_bytes(), salt_str) {
        Ok(h) => Ok(Zval::Str(PhpStr::new(h.into_bytes()))),
        Err(_) => fallback(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use php_types::Diags;

    fn call(args: &[Zval]) -> Result<Zval, PhpError> {
        let mut out = Vec::new();
        let mut diags: Diags = Vec::new();
        let mut direct = Vec::new();
        let dbg = std::collections::HashMap::new();
        let mut ctx = Ctx { out: &mut out, diags: &mut diags, direct_out: &mut direct, debug_info: &dbg };
        crypt(args, &mut ctx)
    }

    fn s(x: &str) -> Zval {
        Zval::Str(PhpStr::new(x.as_bytes().to_vec()))
    }

    fn out(args: &[Zval]) -> String {
        match call(args).unwrap() {
            Zval::Str(p) => String::from_utf8_lossy(p.as_bytes()).into_owned(),
            other => panic!("expected string, got {other:?}"),
        }
    }

    #[test]
    fn std_des_md5_sha_and_bcrypt() {
        assert_eq!(out(&[s("rasmuslerdorf"), s("rl")]), "rl.3StKT.4T8M");
        assert_eq!(
            out(&[s("rasmuslerdorf"), s("$1$rasmusle$")]),
            "$1$rasmusle$rISCgZzpwk3UhDidwXvin0"
        );
        assert_eq!(
            out(&[s("Hello world!"), s("$5$saltstring")]),
            "$5$saltstring$5B8vYYiY.CVt1RlTTf8KbXBH3hsxY/GNooZaBBGWEc5"
        );
        assert_eq!(
            out(&[s("rasmuslerdorf"), s("$2a$07$rasmuslerd............")]),
            "$2a$07$rasmuslerd............nIdrcHdxcUxWomQX9j6kvERCFjTg7Ra"
        );
    }

    #[test]
    fn invalid_salt_star_convention() {
        // "*0" salt → "*1"; any other failure → "*0".
        assert_eq!(out(&[s("foo"), s("*0")]), "*1");
        assert_eq!(out(&[s("foo"), s("*1")]), "*0");
        assert_eq!(out(&[s("test"), s("$23$04$1234567890123456789012345")]), "*0");
    }

    #[test]
    fn sha_rounds_out_of_range_is_rejected_without_hanging() {
        // Must not spin through a billion rounds — PHP returns "*0" here.
        assert_eq!(out(&[s("x"), s("$5$rounds=1000000000$roundstoohigh")]), "*0");
        assert_eq!(out(&[s("x"), s("$6$rounds=999$short")]), "*0");
    }

    #[test]
    fn wrong_arity_is_argument_count_error() {
        assert!(matches!(
            call(&[s("only-one")]),
            Err(PhpError::ArgumentCountError(_))
        ));
    }
}
