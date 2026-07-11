//! `crypt` (step 64) — one-way string hashing.
//!
//! Mirrors `php_crypt`'s salt-prefix dispatch (`ext/standard/crypt.c`) on top of
//! the `pwhash` crate (glibc-compatible DES / BSDi / MD5 / SHA-256 / SHA-512 /
//! bcrypt). PHP's `*0`/`*1` failure convention is layered on top: an invalid salt
//! yields `"*1"` when the salt began with `*0`, otherwise `"*0"`.

use std::rc::Rc;

use pwhash::unix;

use php_runtime::Ctx;
use php_types::{convert, Key, PhpArray, PhpError, PhpStr, Zval};

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

// ---------------------------------------------------------------------------
// password_* (bcrypt) — the only algorithm the oracle build exposes ('2y').
// ---------------------------------------------------------------------------

const BCRYPT_DEFAULT_COST: i64 = 12;
/// bcrypt's radix-64 alphabet (OpenBSD order: `.` `/` then `A-Za-z0-9`).
const BCRYPT_B64: &[u8; 64] = b"./ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

/// Encode raw bytes in bcrypt's radix-64 (OpenBSD `encode_base64`): 16 salt bytes
/// → 22 chars.
fn bcrypt_b64(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity((data.len() * 8 + 5) / 6);
    let mut i = 0;
    while i < data.len() {
        let c1 = data[i];
        i += 1;
        out.push(BCRYPT_B64[(c1 >> 2) as usize]);
        let mut acc = (c1 & 0x03) << 4;
        if i >= data.len() {
            out.push(BCRYPT_B64[acc as usize]);
            break;
        }
        let c2 = data[i];
        i += 1;
        acc |= (c2 >> 4) & 0x0f;
        out.push(BCRYPT_B64[acc as usize]);
        acc = (c2 & 0x0f) << 2;
        if i >= data.len() {
            out.push(BCRYPT_B64[acc as usize]);
            break;
        }
        let c3 = data[i];
        i += 1;
        acc |= (c3 >> 6) & 0x03;
        out.push(BCRYPT_B64[acc as usize]);
        out.push(BCRYPT_B64[(c3 & 0x3f) as usize]);
    }
    out
}

/// `$length` bytes from the OS CSPRNG (`/dev/urandom`), like `random_bytes`.
fn os_random(n: usize) -> Option<Vec<u8>> {
    use std::io::Read;
    let mut buf = vec![0u8; n];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut buf))
        .ok()?;
    Some(buf)
}

/// Constant-time byte equality (result-correct; the timing property is not
/// corpus-observable but matches `password_verify`'s intent).
fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Parse a bcrypt hash (`$2y$NN$...`, also the `$2a$`/`$2b$`/`$2x$` variants),
/// returning its cost. A real bcrypt hash is exactly 60 bytes; anything shorter
/// or malformed is not recognised (`None`), matching `password_get_info`.
fn bcrypt_cost(hash: &[u8]) -> Option<i64> {
    if hash.len() == 60
        && hash[0] == b'$'
        && hash[1] == b'2'
        && matches!(hash[2], b'a' | b'b' | b'x' | b'y')
        && hash[3] == b'$'
        && hash[4].is_ascii_digit()
        && hash[5].is_ascii_digit()
        && hash[6] == b'$'
    {
        Some(((hash[4] - b'0') * 10 + (hash[5] - b'0')) as i64)
    } else {
        None
    }
}

/// `password_hash(string $password, string|int|null $algo, array $options = []): string`
/// — bcrypt only. A non-bcrypt `$algo` is a ValueError; the cost must be 4..=31.
pub fn password_hash(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    if args.len() < 2 {
        return Err(PhpError::ArgumentCountError(format!(
            "password_hash() expects at least 2 arguments, {} given",
            args.len()
        )));
    }
    if matches!(args[0], Zval::Array(_)) {
        return Err(PhpError::TypeError(
            "password_hash(): Argument #1 ($password) must be of type string, array given"
                .to_string(),
        ));
    }
    // ZPP type phase (left-to-right, BEFORE any value validation): $algo must be
    // string|int|null and $options must be an array.
    if matches!(args[1], Zval::Array(_)) {
        return Err(PhpError::TypeError(
            "password_hash(): Argument #2 ($algo) must be of type string|int|null, array given"
                .to_string(),
        ));
    }
    let options = match args.get(2) {
        None => None,
        Some(Zval::Array(a)) => Some(a.clone()),
        Some(other) => {
            return Err(PhpError::TypeError(format!(
                "password_hash(): Argument #3 ($options) must be of type array, {} given",
                other.type_name_for_error()
            )))
        }
    };
    // Value phase.
    let is_bcrypt = match &args[1] {
        Zval::Null | Zval::Long(1) => true,
        v => convert::to_zstr(v, ctx.diags).as_bytes() == b"2y",
    };
    if !is_bcrypt {
        return Err(PhpError::ValueError(
            "password_hash(): Argument #2 ($algo) must be a valid password hashing algorithm"
                .to_string(),
        ));
    }
    let password = convert::to_zstr(&args[0], ctx.diags);
    if password.as_bytes().contains(&0) {
        return Err(PhpError::ValueError(
            "Bcrypt password must not contain null character".to_string(),
        ));
    }
    if let Some(a) = &options {
        if a.get(&Key::from_bytes(b"salt")).is_some() {
            ctx.diags.push(php_types::Diag::Warning(
                "password_hash(): The \"salt\" option has been ignored, since providing a custom \
                 salt is no longer supported"
                    .to_string(),
            ));
        }
    }
    let cost = options
        .as_ref()
        .and_then(|a| a.get(&Key::from_bytes(b"cost")))
        .map(|v| convert::to_long_cast(v, ctx.diags))
        .unwrap_or(BCRYPT_DEFAULT_COST);
    if !(4..=31).contains(&cost) {
        return Err(PhpError::ValueError(format!(
            "Invalid bcrypt cost parameter specified: {cost}"
        )));
    }
    let raw = os_random(16).ok_or_else(|| PhpError::Error("Cannot open source device".to_string()))?;
    let salt = bcrypt_b64(&raw);
    // 16 salt bytes encode to 22 chars; bcrypt uses exactly 22.
    let setting = format!(
        "$2y${:02}${}",
        cost,
        String::from_utf8_lossy(&salt[..22.min(salt.len())])
    );
    match unix::crypt(password.as_bytes(), &setting) {
        Ok(h) => Ok(Zval::Str(PhpStr::new(h.into_bytes()))),
        Err(_) => Err(PhpError::Error("password_hash(): Unexpected failure".to_string())),
    }
}

/// `password_verify(string $password, string $hash): bool`.
pub fn password_verify(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    if args.len() != 2 {
        return Err(PhpError::ArgumentCountError(format!(
            "password_verify() expects exactly 2 arguments, {} given",
            args.len()
        )));
    }
    let password = convert::to_zstr(&args[0], ctx.diags);
    let hash = convert::to_zstr(&args[1], ctx.diags);
    // A hash that is not valid UTF-8, or too short to be a real setting, fails.
    let Ok(hash_str) = std::str::from_utf8(hash.as_bytes()) else {
        return Ok(Zval::Bool(false));
    };
    if hash_str.len() < 13 {
        return Ok(Zval::Bool(false));
    }
    match unix::crypt(password.as_bytes(), hash_str) {
        Ok(computed) => Ok(Zval::Bool(ct_eq(computed.as_bytes(), hash.as_bytes()))),
        Err(_) => Ok(Zval::Bool(false)),
    }
}

/// `password_get_info(string $hash): array` — `['algo'=>, 'algoName'=>, 'options'=>]`.
pub fn password_get_info(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let hash = convert::to_zstr(args.first().unwrap_or(&Zval::Null), ctx.diags);
    let mut info = PhpArray::new();
    match bcrypt_cost(hash.as_bytes()) {
        Some(cost) => {
            info.insert(Key::from_bytes(b"algo"), Zval::Str(PhpStr::new(b"2y".to_vec())));
            info.insert(Key::from_bytes(b"algoName"), Zval::Str(PhpStr::new(b"bcrypt".to_vec())));
            let mut opts = PhpArray::new();
            opts.insert(Key::from_bytes(b"cost"), Zval::Long(cost));
            info.insert(Key::from_bytes(b"options"), Zval::Array(Rc::new(opts)));
        }
        None => {
            info.insert(Key::from_bytes(b"algo"), Zval::Null);
            info.insert(Key::from_bytes(b"algoName"), Zval::Str(PhpStr::new(b"unknown".to_vec())));
            info.insert(Key::from_bytes(b"options"), Zval::Array(Rc::new(PhpArray::new())));
        }
    }
    Ok(Zval::Array(Rc::new(info)))
}

/// `password_needs_rehash(string $hash, string|int|null $algo, array $options = []): bool`.
pub fn password_needs_rehash(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    if args.len() < 2 {
        return Err(PhpError::ArgumentCountError(format!(
            "password_needs_rehash() expects at least 2 arguments, {} given",
            args.len()
        )));
    }
    if matches!(args[0], Zval::Array(_)) {
        return Err(PhpError::TypeError(
            "password_needs_rehash(): Argument #1 ($hash) must be of type string, array given"
                .to_string(),
        ));
    }
    if matches!(args[1], Zval::Array(_)) {
        return Err(PhpError::TypeError(
            "password_needs_rehash(): Argument #2 ($algo) must be of type string|int|null, array given"
                .to_string(),
        ));
    }
    if let Some(other) = args.get(2) {
        if !matches!(other, Zval::Array(_)) {
            return Err(PhpError::TypeError(format!(
                "password_needs_rehash(): Argument #3 ($options) must be of type array, {} given",
                other.type_name_for_error()
            )));
        }
    }
    let hash = convert::to_zstr(&args[0], ctx.diags);
    let want_bcrypt = match args.get(1) {
        None | Some(Zval::Null) | Some(Zval::Long(1)) => true,
        Some(v) => convert::to_zstr(v, ctx.diags).as_bytes() == b"2y",
    };
    match bcrypt_cost(hash.as_bytes()) {
        // A bcrypt hash needs a rehash if a different algorithm is requested or the
        // cost differs from the requested one.
        Some(cost) if want_bcrypt => {
            let want_cost = match args.get(2) {
                Some(Zval::Array(a)) => a
                    .get(&Key::from_bytes(b"cost"))
                    .map(|v| convert::to_long_cast(v, ctx.diags))
                    .unwrap_or(BCRYPT_DEFAULT_COST),
                _ => BCRYPT_DEFAULT_COST,
            };
            Ok(Zval::Bool(cost != want_cost))
        }
        _ => Ok(Zval::Bool(true)),
    }
}

/// `password_algos(): array` — the identifiers available (bcrypt only here).
pub fn password_algos(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let mut arr = PhpArray::new();
    let _ = arr.append(Zval::Str(PhpStr::new(b"2y".to_vec())));
    Ok(Zval::Array(Rc::new(arr)))
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
        let strf = std::collections::HashMap::new();
        let mut ctx = Ctx { out: &mut out, diags: &mut diags, direct_out: &mut direct, debug_info: &dbg, stringify: &strf };
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
