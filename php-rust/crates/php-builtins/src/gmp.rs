//! gmp low-level builtins: arbitrary-precision integers via `num-bigint`.
//!
//! These are the internal primitives (`_gmp_*`) that the PHP-side `GMP` class and
//! `gmp_*` wrappers (in the prelude) delegate to. The canonical exchange form is a
//! base-10 string (the value of a `GMP` object's `num` property), so every
//! primitive takes and returns decimal strings (plus ints/bools where PHP does).
//!
//! num-bigint covers the arithmetic; number-theory and bitwise helpers are hand
//! -rolled below. The GMP random family is intentionally absent (non-deterministic,
//! cannot be byte-matched) — see PHPR_DIVERGENCES_FROM_PHP.md.

use num_bigint::{BigInt, Sign};
use num_integer::Integer;
use num_traits::{One, Signed, Zero};

use php_runtime::Ctx;
use php_types::{convert, PhpError, PhpStr, Zval};

fn to_bi(v: &Zval) -> BigInt {
    let s = match v {
        Zval::Str(s) => s.as_bytes().to_vec(),
        other => {
            // Decimal already (the PHP layer passes canonical strings/ints).
            return BigInt::from(convert::to_long_cast(other, &mut Vec::new()));
        }
    };
    BigInt::parse_bytes(&s, 10).unwrap_or_else(BigInt::zero)
}

fn ret(n: BigInt) -> Result<Zval, PhpError> {
    Ok(Zval::Str(PhpStr::new(n.to_str_radix(10).into_bytes())))
}

/// Parse a GMP integer literal in `base` (0 = auto-detect 0x/0b/0o/0/decimal).
/// Returns the canonical decimal, or `PARSE_ERR` if malformed.
fn parse_int(s: &[u8], base: i64) -> Option<BigInt> {
    // Leading whitespace is allowed by GMP.
    let mut i = 0;
    while i < s.len() && (s[i] == b' ' || s[i] == b'\t' || s[i] == b'\n' || s[i] == b'\r') {
        i += 1;
    }
    // GMP accepts a leading '-' but not '+'.
    let mut neg = false;
    if let Some(b'-') = s.get(i) {
        neg = true;
        i += 1;
    }
    let rest = &s[i..];
    if rest.is_empty() {
        return None; // "" / "+" / "-" are not integer strings
    }
    let (digits, radix): (&[u8], u32) = if base == 0 {
        if rest.len() >= 2 && rest[0] == b'0' && (rest[1] | 0x20) == b'x' {
            (&rest[2..], 16)
        } else if rest.len() >= 2 && rest[0] == b'0' && (rest[1] | 0x20) == b'b' {
            (&rest[2..], 2)
        } else if rest.len() >= 2 && rest[0] == b'0' && (rest[1] | 0x20) == b'o' {
            (&rest[2..], 8)
        } else if rest.len() >= 2 && rest[0] == b'0' {
            (&rest[1..], 8)
        } else {
            (rest, 10)
        }
    } else {
        (rest, base as u32)
    };
    if digits.is_empty() {
        // "0" (and "0x" etc.) — GMP treats a lone/empty tail as 0.
        return Some(BigInt::zero());
    }
    let n = BigInt::parse_bytes(digits, radix)?;
    Some(if neg { -n } else { n })
}

/// `_gmp_parse(string $s, int $base): string|false` — canonical decimal, or
/// `false` when the string is not a valid integer literal (the PHP layer brands
/// the ValueError).
pub fn parse(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr(args.first().unwrap_or(&Zval::Null), ctx.diags);
    let base = convert::to_long_cast(args.get(1).unwrap_or(&Zval::Null), ctx.diags);
    match parse_int(s.as_bytes(), base) {
        Some(n) => ret(n),
        None => Ok(Zval::Bool(false)),
    }
}

/// General base conversion for `gmp_strval` (bases 2..=62 and -2..=-36).
fn to_radix(n: &BigInt, base: i64) -> String {
    let abase = base.unsigned_abs();
    if !(2..=62).contains(&abase) {
        return n.to_str_radix(10);
    }
    let digit = |d: u64| -> u8 {
        if d < 10 {
            b'0' + d as u8
        } else if base < 0 || base > 36 {
            // upper-case for 10..=35 (negative base, or 37..62)
            if d < 36 {
                b'A' + (d - 10) as u8
            } else {
                b'a' + (d - 36) as u8
            }
        } else {
            b'a' + (d - 10) as u8
        }
    };
    if n.is_zero() {
        return "0".into();
    }
    let neg = n.sign() == Sign::Minus;
    let mut m = n.abs();
    let b = BigInt::from(abase);
    let mut out = Vec::new();
    while !m.is_zero() {
        let (q, r) = m.div_rem(&b);
        let rd: u64 = r.to_str_radix(10).parse().unwrap_or(0);
        out.push(digit(rd));
        m = q;
    }
    if neg {
        out.push(b'-');
    }
    out.reverse();
    String::from_utf8(out).unwrap()
}

/// `_gmp_strval(string $a, int $base): string`.
pub fn strval(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    let base = convert::to_long_cast(args.get(1).unwrap_or(&Zval::Long(10)), ctx.diags);
    Ok(Zval::Str(PhpStr::new(to_radix(&a, base).into_bytes())))
}

/// `_gmp_intval(string $a): int` — truncated to platform int (wrapping like GMP).
pub fn intval(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    // GMP returns the low 64 bits as a signed long.
    let (sign, digits) = a.to_u64_digits();
    let low = digits.first().copied().unwrap_or(0);
    let v = if sign == Sign::Minus {
        (low as i64).wrapping_neg()
    } else {
        low as i64
    };
    Ok(Zval::Long(v))
}

/// `_gmp_cmp(string $a, string $b): int` (-1/0/1).
pub fn cmp(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    let b = to_bi(args.get(1).unwrap_or(&Zval::Null));
    Ok(Zval::Long(match a.cmp(&b) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }))
}

/// `_gmp_sign(string $a): int`.
pub fn sign(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    Ok(Zval::Long(match a.sign() {
        Sign::Minus => -1,
        Sign::NoSign => 0,
        Sign::Plus => 1,
    }))
}

/* ---- division helpers (rounding modes: 0=zero, 1=+inf, 2=-inf) ---- */

fn div_round(a: &BigInt, b: &BigInt, mode: i64) -> (BigInt, BigInt) {
    // truncated quotient/remainder (toward zero)
    let (mut q, mut r) = a.div_rem(b);
    match mode {
        1 => {
            // toward +inf: round up when remainder non-zero and result positive
            if !r.is_zero() && (r.sign() == b.sign()) {
                q += 1;
                r -= b;
            }
        }
        2 => {
            // toward -inf: round down when remainder non-zero and signs differ
            if !r.is_zero() && (r.sign() != b.sign()) {
                q -= 1;
                r += b;
            }
        }
        _ => {} // toward zero (num-bigint default)
    }
    let _ = &mut q;
    let _ = &mut r;
    (q, r)
}

/// `_gmp_divq(a, b, mode): string`.
pub fn divq(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    let b = to_bi(args.get(1).unwrap_or(&Zval::Null));
    let mode = convert::to_long_cast(args.get(2).unwrap_or(&Zval::Long(0)), ctx.diags);
    ret(div_round(&a, &b, mode).0)
}

/// `_gmp_divr(a, b, mode): string`.
pub fn divr(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    let b = to_bi(args.get(1).unwrap_or(&Zval::Null));
    let mode = convert::to_long_cast(args.get(2).unwrap_or(&Zval::Long(0)), ctx.diags);
    ret(div_round(&a, &b, mode).1)
}

/// `_gmp_divqr(a, b, mode): string` — "q r".
pub fn divqr(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    let b = to_bi(args.get(1).unwrap_or(&Zval::Null));
    let mode = convert::to_long_cast(args.get(2).unwrap_or(&Zval::Long(0)), ctx.diags);
    let (q, r) = div_round(&a, &b, mode);
    Ok(Zval::Str(PhpStr::new(
        format!("{} {}", q.to_str_radix(10), r.to_str_radix(10)).into_bytes(),
    )))
}

/// `_gmp_mod(a, b): string` — always non-negative (mpz_mod: |b|).
pub fn modulo(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    let b = to_bi(args.get(1).unwrap_or(&Zval::Null)).abs();
    ret(a.mod_floor(&b))
}

/// Binary numeric op dispatcher: 0 add,1 sub,2 mul,6 divexact,7 gcd,8 lcm,
/// 9 and,10 or,11 xor,12 pow(b=exp),14 binomial(n,k).
pub fn bin(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let op = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), ctx.diags);
    let a = to_bi(args.get(1).unwrap_or(&Zval::Null));
    let b = to_bi(args.get(2).unwrap_or(&Zval::Null));
    let r = match op {
        0 => a + b,
        1 => a - b,
        2 => a * b,
        6 => a / b, // divexact (exact division; matches / when exact)
        7 => a.gcd(&b),
        8 => a.lcm(&b),
        9 => a & b,
        10 => a | b,
        11 => a ^ b,
        12 => {
            let e: u32 = b.try_into().unwrap_or(0);
            a.pow(e)
        }
        _ => BigInt::zero(),
    };
    ret(r)
}

/// Unary op dispatcher: 0 neg, 1 abs, 2 com (~a = -a-1), 3 sqrt(floor).
pub fn un(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let op = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), ctx.diags);
    let a = to_bi(args.get(1).unwrap_or(&Zval::Null));
    let r = match op {
        0 => -a,
        1 => a.abs(),
        2 => -(a + BigInt::one()),
        3 => a.sqrt(),
        _ => BigInt::zero(),
    };
    ret(r)
}

/* ================= Phase 2: number theory ================= */

/// `_gmp_powm(a, e, m): string` — modular exponentiation (e ≥ 0, m ≠ 0 checked
/// by the PHP layer). Result in [0, |m|).
pub fn powm(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    let e = to_bi(args.get(1).unwrap_or(&Zval::Null));
    let m = to_bi(args.get(2).unwrap_or(&Zval::Null));
    let mut r = a.modpow(&e, &m);
    if r.sign() == Sign::Minus {
        r += m.abs();
    }
    ret(r)
}

/// Extended Euclid: returns (g, s, t) with a*s + b*t = g, g ≥ 0.
fn egcd(a: &BigInt, b: &BigInt) -> (BigInt, BigInt, BigInt) {
    let (mut old_r, mut r) = (a.clone(), b.clone());
    let (mut old_s, mut s) = (BigInt::one(), BigInt::zero());
    let (mut old_t, mut t) = (BigInt::zero(), BigInt::one());
    while !r.is_zero() {
        let q = &old_r / &r;
        let nr = &old_r - &q * &r;
        old_r = std::mem::replace(&mut r, nr);
        let ns = &old_s - &q * &s;
        old_s = std::mem::replace(&mut s, ns);
        let nt = &old_t - &q * &t;
        old_t = std::mem::replace(&mut t, nt);
    }
    if old_r.sign() == Sign::Minus {
        old_r = -old_r;
        old_s = -old_s;
        old_t = -old_t;
    }
    (old_r, old_s, old_t)
}

/// `_gmp_gcdext(a, b): string` — "g s t".
pub fn gcdext(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    let b = to_bi(args.get(1).unwrap_or(&Zval::Null));
    let (g, s, t) = egcd(&a, &b);
    Ok(Zval::Str(PhpStr::new(
        format!("{} {} {}", g.to_str_radix(10), s.to_str_radix(10), t.to_str_radix(10)).into_bytes(),
    )))
}

/// `_gmp_invert(a, m): string|false` — modular inverse, or false if none.
pub fn invert(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    let m = to_bi(args.get(1).unwrap_or(&Zval::Null)).abs();
    let (g, s, _) = egcd(&a, &m);
    if g != BigInt::one() {
        return Ok(Zval::Bool(false));
    }
    ret(s.mod_floor(&m))
}

fn nth_root(a: &BigInt, n: u32) -> BigInt {
    if a.sign() == Sign::Minus {
        // odd root of a negative is the negative root; an even root of a
        // negative is a domain error rejected by the PHP layer.
        -(-a).nth_root(n)
    } else {
        a.nth_root(n)
    }
}

/// `_gmp_root(a, n): string` — integer n-th root (toward zero). n>0 by PHP.
pub fn root(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    let n = convert::to_long_cast(args.get(1).unwrap_or(&Zval::Null), ctx.diags) as u32;
    ret(nth_root(&a, n))
}

/// `_gmp_rootrem(a, n): string` — "root rem" (rem = a - root^n).
pub fn rootrem(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    let n = convert::to_long_cast(args.get(1).unwrap_or(&Zval::Null), ctx.diags) as u32;
    let r = nth_root(&a, n);
    let rem = &a - r.pow(n);
    Ok(Zval::Str(PhpStr::new(
        format!("{} {}", r.to_str_radix(10), rem.to_str_radix(10)).into_bytes(),
    )))
}

/// `_gmp_sqrtrem(a): string` — "s r" (r = a - s^2).
pub fn sqrtrem(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    let s = a.sqrt();
    let r = &a - &s * &s;
    Ok(Zval::Str(PhpStr::new(
        format!("{} {}", s.to_str_radix(10), r.to_str_radix(10)).into_bytes(),
    )))
}

/// `_gmp_fact(n): string` — n! (n ≥ 0).
pub fn fact(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let n = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), ctx.diags);
    let mut r = BigInt::one();
    let mut i = 2i64;
    while i <= n {
        r *= i;
        i += 1;
    }
    ret(r)
}

/// `_gmp_binomial(n, k): string` — C(n, k).
pub fn binomial(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let n = to_bi(args.first().unwrap_or(&Zval::Null));
    let k = convert::to_long_cast(args.get(1).unwrap_or(&Zval::Null), ctx.diags);
    if k < 0 {
        return ret(BigInt::zero());
    }
    let mut num = BigInt::one();
    let mut den = BigInt::one();
    let mut top = n;
    for i in 0..k {
        num *= &top;
        top -= 1;
        den *= i + 1;
    }
    ret(num / den)
}

fn is_probable_prime(n: &BigInt) -> bool {
    if n < &BigInt::from(2) {
        return false;
    }
    let small: [u64; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    for &p in &small {
        let bp = BigInt::from(p);
        if n == &bp {
            return true;
        }
        if (n % &bp).is_zero() {
            return false;
        }
    }
    // Miller-Rabin, deterministic for n < 3.3e24 with these bases.
    let one = BigInt::one();
    let n1 = n - &one;
    let mut d = n1.clone();
    let mut r = 0u32;
    while d.is_even() {
        d /= 2;
        r += 1;
    }
    let two = BigInt::from(2);
    'witness: for &a in &small {
        let a = BigInt::from(a);
        let mut x = a.modpow(&d, n);
        if x == one || x == n1 {
            continue;
        }
        for _ in 0..r.saturating_sub(1) {
            x = x.modpow(&two, n);
            if x == n1 {
                continue 'witness;
            }
        }
        return false;
    }
    true
}

/// `_gmp_probprime(n, reps): int` — 0 composite, 1 probably prime, 2 prime.
pub fn probprime(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let n = to_bi(args.first().unwrap_or(&Zval::Null));
    if !is_probable_prime(&n) {
        return Ok(Zval::Long(0));
    }
    // GMP proves primality by trial division up to ~sqrt(5e15); a survivor below
    // that is "definitely prime" (2), above it "probably prime" (1). The exact
    // cutoff is an internal GMP detail — see PHPR_DIVERGENCES.
    let proven = n <= BigInt::from(5_000_000_000_000_000i64);
    Ok(Zval::Long(if proven { 2 } else { 1 }))
}

/// `_gmp_nextprime(n): string` — smallest prime strictly greater than n.
pub fn nextprime(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let mut n = to_bi(args.first().unwrap_or(&Zval::Null));
    let two = BigInt::from(2);
    if n < two {
        return ret(two);
    }
    n += 1;
    if n.is_even() {
        n += 1;
    }
    while !is_probable_prime(&n) {
        n += 2;
    }
    ret(n)
}

fn kronecker_sym(a: &BigInt, b: &BigInt) -> i32 {
    if b.is_zero() {
        return if a.abs() == BigInt::one() { 1 } else { 0 };
    }
    let mut a = a.clone();
    let mut b = b.clone();
    let mut result = 1i32;
    if b.sign() == Sign::Minus {
        b = -&b;
        if a.sign() == Sign::Minus {
            result = -result;
        }
    }
    let eight = BigInt::from(8);
    let four = BigInt::from(4);
    let three = BigInt::from(3);
    let five = BigInt::from(5);
    let mut e = 0u32;
    while b.is_even() {
        b /= 2;
        e += 1;
    }
    if e % 2 == 1 {
        let m8 = a.mod_floor(&eight);
        if m8 == three || m8 == five {
            result = -result;
        }
    }
    // b is now odd and positive; reduce a into [0, b) and run Jacobi.
    a = a.mod_floor(&b);
    loop {
        if a.is_zero() {
            return if b == BigInt::one() { result } else { 0 };
        }
        let mut e = 0u32;
        while a.is_even() {
            a /= 2;
            e += 1;
        }
        if e % 2 == 1 {
            let m8 = b.mod_floor(&eight);
            if m8 == three || m8 == five {
                result = -result;
            }
        }
        if a.mod_floor(&four) == three && b.mod_floor(&four) == three {
            result = -result;
        }
        std::mem::swap(&mut a, &mut b);
        a = a.mod_floor(&b);
    }
}

/// `_gmp_kronecker(a, b): int` — Kronecker symbol (covers jacobi/legendre in
/// their valid domains).
pub fn kronecker(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    let b = to_bi(args.get(1).unwrap_or(&Zval::Null));
    Ok(Zval::Long(kronecker_sym(&a, &b) as i64))
}

/// `_gmp_perfsquare(a): bool`.
pub fn perfsquare(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    if a.sign() == Sign::Minus {
        return Ok(Zval::Bool(false));
    }
    let s = a.sqrt();
    Ok(Zval::Bool(&s * &s == a))
}

/// `_gmp_perfpower(a): bool` — is a = m^k for some k ≥ 2.
pub fn perfpower(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    let abs = a.abs();
    if abs <= BigInt::one() {
        return Ok(Zval::Bool(true));
    }
    let bits = abs.bits();
    for k in 2..=bits as u32 {
        if a.sign() == Sign::Minus && k % 2 == 0 {
            continue;
        }
        let r = nth_root(&a, k);
        if r.pow(k) == a {
            return Ok(Zval::Bool(true));
        }
    }
    Ok(Zval::Bool(false))
}

/* ================= Phase 3: bit manipulation ================= */

/// `_gmp_setbit(a, index, set): string` — set/clear bit `index` (two's complement).
pub fn setbit(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let mut a = to_bi(args.first().unwrap_or(&Zval::Null));
    let idx = convert::to_long_cast(args.get(1).unwrap_or(&Zval::Null), ctx.diags) as u64;
    let set = convert::to_long_cast(args.get(2).unwrap_or(&Zval::Long(1)), ctx.diags) != 0;
    a.set_bit(idx, set);
    ret(a)
}

/// `_gmp_testbit(a, index): bool`.
pub fn testbit(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    let idx = convert::to_long_cast(args.get(1).unwrap_or(&Zval::Null), ctx.diags) as u64;
    Ok(Zval::Bool(a.bit(idx)))
}

/// Scan for the first bit equal to `want` at position ≥ `start`; -1 if none.
fn scan(a: &BigInt, start: u64, want: bool) -> i64 {
    let maxbit = a.bits() + 2;
    let mut idx = start;
    loop {
        if a.bit(idx) == want {
            return idx as i64;
        }
        idx += 1;
        if idx > maxbit {
            // Above the magnitude the tail is all sign bits.
            let tail_is_one = a.sign() == Sign::Minus;
            return if tail_is_one == want { idx as i64 } else { -1 };
        }
    }
}

/// `_gmp_scan0(a, start): int`.
pub fn scan0(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    let start = convert::to_long_cast(args.get(1).unwrap_or(&Zval::Null), ctx.diags) as u64;
    Ok(Zval::Long(scan(&a, start, false)))
}

/// `_gmp_scan1(a, start): int`.
pub fn scan1(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    let start = convert::to_long_cast(args.get(1).unwrap_or(&Zval::Null), ctx.diags) as u64;
    Ok(Zval::Long(scan(&a, start, true)))
}

/// Number of set bits in a non-negative value; `None` for negative (GMP reports
/// SIZE_MAX → PHP -1).
fn count_ones_bi(n: &BigInt) -> Option<u64> {
    if n.sign() == Sign::Minus {
        return None;
    }
    let (_, bytes) = n.to_bytes_le();
    Some(bytes.iter().map(|b| b.count_ones() as u64).sum())
}

/// `_gmp_popcount(a): int` — set-bit count (a ≥ 0; a negative has infinitely many,
/// which GMP reports as SIZE_MAX → PHP -1).
pub fn popcount(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    Ok(Zval::Long(count_ones_bi(&a).map_or(-1, |c| c as i64)))
}

/// `_gmp_hamdist(a, b): int` — Hamming distance = popcount(a XOR b).
pub fn hamdist(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_bi(args.first().unwrap_or(&Zval::Null));
    let b = to_bi(args.get(1).unwrap_or(&Zval::Null));
    Ok(Zval::Long(count_ones_bi(&(a ^ b)).map_or(-1, |c| c as i64)))
}
