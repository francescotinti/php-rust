//! bcmath builtins: arbitrary-precision decimal arithmetic.
//!
//! Faithful port of `ext/bcmath/libbcmath` (the classic digit-array algorithms;
//! the 8.4+ SIMD rewrite only changes performance, not results). Numbers are
//! kept as a sign plus a vector of integer digits (0–9, most-significant first,
//! no leading zeros, length ≥ 1) and a vector of fractional digits whose length
//! is the number's scale. All arithmetic goes through an exact base-10
//! big-unsigned helper, so results are byte-identical to PHP.
//!
//! The default scale (`bcscale()`) is process/thread state; PHP ties it to the
//! `bcmath.scale` INI entry, which phpr does not model — see
//! `PHPR_DIVERGENCES_FROM_PHP.md`.

use std::cell::Cell;
use std::cmp::Ordering;
use std::rc::Rc;

use php_runtime::Ctx;
use php_types::{convert, PhpArray, PhpError, PhpStr, Zval};

const SCALE_MAX: i64 = 2147483647; // INT_MAX

thread_local! {
    /// Default scale for bc functions when the optional `$scale` is omitted.
    static BC_SCALE: Cell<usize> = const { Cell::new(0) };
}

/* ------------------------------------------------------------------ */
/* Base-10 big-unsigned helpers. Digits are 0–9, most-significant first, */
/* trimmed of leading zeros; the empty vector represents 0.              */
/* ------------------------------------------------------------------ */

fn u_trim(mut v: Vec<u8>) -> Vec<u8> {
    let z = v.iter().take_while(|&&d| d == 0).count();
    v.drain(0..z);
    v
}

fn u_is_zero(v: &[u8]) -> bool {
    v.iter().all(|&d| d == 0)
}

fn u_cmp(a: &[u8], b: &[u8]) -> Ordering {
    let a = u_trim(a.to_vec());
    let b = u_trim(b.to_vec());
    match a.len().cmp(&b.len()) {
        Ordering::Equal => a.cmp(&b),
        o => o,
    }
}

fn u_add(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(a.len().max(b.len()) + 1);
    let mut i = a.len();
    let mut j = b.len();
    let mut carry = 0u8;
    while i > 0 || j > 0 || carry > 0 {
        let da = if i > 0 { i -= 1; a[i] } else { 0 };
        let db = if j > 0 { j -= 1; b[j] } else { 0 };
        let s = da + db + carry;
        out.push(s % 10);
        carry = s / 10;
    }
    out.reverse();
    u_trim(out)
}

/// `a - b`, assuming `a >= b`.
fn u_sub(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(a.len());
    let mut i = a.len();
    let mut j = b.len();
    let mut borrow = 0i8;
    while i > 0 {
        i -= 1;
        let da = a[i] as i8;
        let db = if j > 0 { j -= 1; b[j] as i8 } else { 0 };
        let mut d = da - db - borrow;
        if d < 0 {
            d += 10;
            borrow = 1;
        } else {
            borrow = 0;
        }
        out.push(d as u8);
    }
    out.reverse();
    u_trim(out)
}

fn u_mul(a: &[u8], b: &[u8]) -> Vec<u8> {
    if a.is_empty() || b.is_empty() {
        return Vec::new();
    }
    let mut acc = vec![0u16; a.len() + b.len()];
    for (ia, &da) in a.iter().rev().enumerate() {
        if da == 0 {
            continue;
        }
        let mut carry = 0u16;
        for (ib, &db) in b.iter().rev().enumerate() {
            let idx = ia + ib;
            let cur = acc[idx] + da as u16 * db as u16 + carry;
            acc[idx] = cur % 10;
            carry = cur / 10;
        }
        let mut idx = ia + b.len();
        while carry > 0 {
            let cur = acc[idx] + carry;
            acc[idx] = cur % 10;
            carry = cur / 10;
            idx += 1;
        }
    }
    let mut out: Vec<u8> = acc.iter().rev().map(|&d| d as u8).collect();
    out = u_trim(out);
    out
}

/// Multiply by 10^k (append k zeros). 0 stays 0.
fn u_shl(a: &[u8], k: usize) -> Vec<u8> {
    if a.is_empty() {
        return Vec::new();
    }
    let mut out = a.to_vec();
    out.extend(std::iter::repeat(0u8).take(k));
    out
}

/// Long division: returns (quotient, remainder). Divisor must be non-zero.
fn u_divmod(num: &[u8], den: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let num = u_trim(num.to_vec());
    let den = u_trim(den.to_vec());
    if u_cmp(&num, &den) == Ordering::Less {
        return (Vec::new(), num);
    }
    let mut quot = Vec::with_capacity(num.len());
    let mut rem: Vec<u8> = Vec::new();
    for &d in &num {
        rem.push(d);
        rem = u_trim(rem);
        // Largest q in 0..=9 with den*q <= rem.
        let mut q = 0u8;
        while q < 9 && u_cmp(&u_mul(&den, &[q + 1]), &rem) != Ordering::Greater {
            q += 1;
        }
        if q > 0 {
            rem = u_sub(&rem, &u_mul(&den, &[q]));
        }
        quot.push(q);
    }
    (u_trim(quot), rem)
}

/// Exponentiation by squaring, exponent > 0.
fn u_pow(base: &[u8], mut e: i64) -> Vec<u8> {
    let mut result = vec![1u8];
    let mut b = u_trim(base.to_vec());
    while e > 0 {
        if e & 1 == 1 {
            result = u_mul(&result, &b);
        }
        e >>= 1;
        if e > 0 {
            b = u_mul(&b, &b);
        }
    }
    u_trim(result)
}

/* ------------------------------------------------------------------ */
/* BcNum                                                                */
/* ------------------------------------------------------------------ */

#[derive(Clone)]
struct BcNum {
    sign: bool, // true = negative
    int: Vec<u8>,
    frac: Vec<u8>,
}

impl BcNum {
    fn zero() -> BcNum {
        BcNum { sign: false, int: vec![0], frac: Vec::new() }
    }
    fn one() -> BcNum {
        BcNum { sign: false, int: vec![1], frac: Vec::new() }
    }
    fn from_i64(n: i64) -> BcNum {
        if n == 0 {
            return BcNum::zero();
        }
        let sign = n < 0;
        let mut m = (n as i128).unsigned_abs();
        let mut digits = Vec::new();
        while m > 0 {
            digits.push((m % 10) as u8);
            m /= 10;
        }
        digits.reverse();
        BcNum { sign, int: digits, frac: Vec::new() }
    }
    fn scale(&self) -> usize {
        self.frac.len()
    }
    /// Magnitude as a scaled integer: |value| * 10^scale.
    fn mag(&self) -> Vec<u8> {
        let mut v = self.int.clone();
        v.extend_from_slice(&self.frac);
        u_trim(v)
    }
    /// Build from magnitude m = |value| * 10^scale.
    fn from_mag(m: Vec<u8>, scale: usize, sign: bool) -> BcNum {
        let mut d = u_trim(m);
        while d.len() < scale {
            d.insert(0, 0);
        }
        let (int, frac) = if d.len() == scale {
            (vec![0], d)
        } else {
            let s = d.len() - scale;
            (d[..s].to_vec(), d[s..].to_vec())
        };
        BcNum { sign, int, frac }
    }
    fn is_zero(&self) -> bool {
        u_is_zero(&self.int) && u_is_zero(&self.frac)
    }
    fn is_zero_for_scale(&self, scale: usize) -> bool {
        if !u_is_zero(&self.int) {
            return false;
        }
        let n = scale.min(self.frac.len());
        u_is_zero(&self.frac[..n])
    }
    /// Truncate the fractional part to at most `scale` digits.
    fn truncate_frac(&mut self, scale: usize) {
        if self.frac.len() > scale {
            self.frac.truncate(scale);
        }
    }
    /// num2str_ex: render with exactly `scale` fractional digits.
    fn format(&self, scale: usize) -> Vec<u8> {
        let min_scale = self.scale().min(scale);
        let signch = self.sign && !self.is_zero_for_scale(min_scale);
        let mut out = Vec::new();
        if signch {
            out.push(b'-');
        }
        for &d in &self.int {
            out.push(b'0' + d);
        }
        if scale > 0 {
            out.push(b'.');
            for &d in &self.frac[..min_scale] {
                out.push(b'0' + d);
            }
            for _ in self.scale()..scale {
                out.push(b'0');
            }
        }
        out
    }
}

/// Parse a bc number string. Returns None if not well-formed.
/// Mirrors `bc_str2num` with auto_scale = true (keeps full scale, trims trailing
/// fractional zeros).
fn bc_parse(s: &[u8]) -> Option<BcNum> {
    let mut i = 0usize;
    let neg = matches!(s.first(), Some(b'-'));
    if matches!(s.first(), Some(b'+') | Some(b'-')) {
        i += 1;
    }
    // Skip leading zeros.
    while i < s.len() && s[i] == b'0' {
        i += 1;
    }
    let int_start = i;
    while i < s.len() && s[i].is_ascii_digit() {
        i += 1;
    }
    let int_digits = &s[int_start..i];

    let mut frac_digits: &[u8] = &[];
    if i < s.len() && s[i] == b'.' {
        i += 1;
        let frac_start = i;
        while i < s.len() && s[i].is_ascii_digit() {
            i += 1;
        }
        frac_digits = &s[frac_start..i];
    }
    // Anything left over (whitespace, letters, exponent, second dot) is invalid.
    if i != s.len() {
        return None;
    }

    // Trim trailing fractional zeros.
    let mut fend = frac_digits.len();
    while fend > 0 && frac_digits[fend - 1] == b'0' {
        fend -= 1;
    }
    let frac_digits = &frac_digits[..fend];

    if int_digits.is_empty() && frac_digits.is_empty() {
        return Some(BcNum::zero());
    }

    let int: Vec<u8> = if int_digits.is_empty() {
        vec![0]
    } else {
        int_digits.iter().map(|&c| c - b'0').collect()
    };
    let frac: Vec<u8> = frac_digits.iter().map(|&c| c - b'0').collect();
    Some(BcNum { sign: neg, int, frac })
}

/* ------------------------------------------------------------------ */
/* Comparison                                                           */
/* ------------------------------------------------------------------ */

/// Compare magnitudes, capping the fractional comparison at `scale` digits.
fn cmp_mag(a: &BcNum, b: &BcNum, scale: usize) -> Ordering {
    if a.int.len() != b.int.len() {
        return a.int.len().cmp(&b.int.len());
    }
    match a.int.cmp(&b.int) {
        Ordering::Equal => {}
        o => return o,
    }
    let na = a.scale().min(scale);
    let nb = b.scale().min(scale);
    let m = na.min(nb);
    for k in 0..m {
        match a.frac[k].cmp(&b.frac[k]) {
            Ordering::Equal => {}
            o => return o,
        }
    }
    if na > nb {
        if a.frac[nb..na].iter().any(|&d| d != 0) {
            return Ordering::Greater;
        }
    } else if nb > na && b.frac[na..nb].iter().any(|&d| d != 0) {
        return Ordering::Less;
    }
    Ordering::Equal
}

/// Signed compare: 1 if a>b, -1 if a<b, 0 if equal (fraction capped at scale).
fn do_compare(a: &BcNum, b: &BcNum, scale: usize) -> i64 {
    if a.sign != b.sign {
        if a.is_zero_for_scale(scale) && b.is_zero_for_scale(scale) {
            return 0;
        }
        return if !a.sign { 1 } else { -1 };
    }
    let ord = cmp_mag(a, b, scale);
    let left_greater = match ord {
        Ordering::Greater => true,
        Ordering::Less => false,
        Ordering::Equal => return 0,
    };
    // Same sign: if negative, larger magnitude means smaller value.
    match (left_greater, a.sign) {
        (true, false) => 1,
        (true, true) => -1,
        (false, false) => -1,
        (false, true) => 1,
    }
}

/* ------------------------------------------------------------------ */
/* Core arithmetic (mirrors add.c / sub.c / recmul.c / div.c)          */
/* ------------------------------------------------------------------ */

fn do_add_mag(a: &BcNum, b: &BcNum) -> BcNum {
    let s = a.scale().max(b.scale());
    let am = u_shl(&a.mag(), s - a.scale());
    let bm = u_shl(&b.mag(), s - b.scale());
    BcNum::from_mag(u_add(&am, &bm), s, false)
}

/// |a| - |b|, assuming |a| >= |b|.
fn do_sub_mag(a: &BcNum, b: &BcNum) -> BcNum {
    let s = a.scale().max(b.scale());
    let am = u_shl(&a.mag(), s - a.scale());
    let bm = u_shl(&b.mag(), s - b.scale());
    BcNum::from_mag(u_sub(&am, &bm), s, false)
}

fn bc_add(a: &BcNum, b: &BcNum, scale_min: usize) -> BcNum {
    if a.sign == b.sign {
        let mut sum = do_add_mag(a, b);
        sum.sign = a.sign;
        sum
    } else {
        match cmp_mag(a, b, scale_min) {
            Ordering::Less => {
                let mut d = do_sub_mag(b, a);
                d.sign = b.sign;
                d
            }
            Ordering::Equal => {
                let s = scale_min.max(a.scale().max(b.scale()));
                BcNum::from_mag(Vec::new(), s, false)
            }
            Ordering::Greater => {
                let mut d = do_sub_mag(a, b);
                d.sign = a.sign;
                d
            }
        }
    }
}

fn bc_sub(a: &BcNum, b: &BcNum, scale_min: usize) -> BcNum {
    if a.sign != b.sign {
        let mut d = do_add_mag(a, b);
        d.sign = a.sign;
        d
    } else {
        match cmp_mag(a, b, scale_min) {
            Ordering::Less => {
                let mut d = do_sub_mag(b, a);
                d.sign = !b.sign;
                d
            }
            Ordering::Equal => {
                let s = scale_min.max(a.scale().max(b.scale()));
                BcNum::from_mag(Vec::new(), s, false)
            }
            Ordering::Greater => {
                let mut d = do_sub_mag(a, b);
                d.sign = a.sign;
                d
            }
        }
    }
}

fn bc_mul(a: &BcNum, b: &BcNum, scale: usize) -> BcNum {
    let full = a.scale() + b.scale();
    let prod_scale = full.min(scale.max(a.scale().max(b.scale())));
    let p = u_mul(&a.mag(), &b.mag());
    let drop = full - prod_scale;
    let p2 = if drop == 0 {
        p
    } else if drop >= p.len() {
        Vec::new()
    } else {
        p[..p.len() - drop].to_vec()
    };
    let sign = a.sign != b.sign;
    let r = BcNum::from_mag(p2, prod_scale, sign);
    if r.is_zero() {
        BcNum::zero()
    } else {
        r
    }
}

/// Truncating division to `scale` fractional digits. Returns None on divide-by-zero.
fn bc_div(a: &BcNum, b: &BcNum, scale: usize) -> Option<BcNum> {
    if b.is_zero() {
        return None;
    }
    if a.is_zero() {
        return Some(BcNum::zero());
    }
    let e = scale as i64 + b.scale() as i64 - a.scale() as i64;
    let (num, den) = if e >= 0 {
        (u_shl(&a.mag(), e as usize), b.mag())
    } else {
        (a.mag(), u_shl(&b.mag(), (-e) as usize))
    };
    let (q, _r) = u_divmod(&num, &den);
    if u_is_zero(&q) {
        return Some(BcNum::zero());
    }
    Some(BcNum::from_mag(q, scale, a.sign != b.sign))
}

/// Returns (quotient truncated to integer, remainder). None on divide-by-zero.
fn bc_divmod(a: &BcNum, b: &BcNum, scale: usize) -> Option<(BcNum, BcNum)> {
    if b.is_zero() {
        return None;
    }
    let rscale = a.scale().max(b.scale() + scale);
    let q = bc_div(a, b, 0)?;
    let temp = bc_mul(&q, b, rscale);
    let mut rem = bc_sub(a, &temp, rscale);
    rem.truncate_frac(scale);
    if rem.is_zero() {
        rem = BcNum::zero();
    }
    Some((q, rem))
}

#[derive(Debug)]
enum RaiseErr {
    DivByZero,
}

fn bc_raise(base: &BcNum, exponent: i64, scale: usize) -> Result<BcNum, RaiseErr> {
    if exponent == 0 {
        return Ok(BcNum::one());
    }
    let neg = exponent < 0;
    let e = (exponent as i128).unsigned_abs() as i64;
    if base.is_zero() {
        if neg {
            return Err(RaiseErr::DivByZero);
        }
        return Ok(BcNum::zero());
    }
    let power_mag = u_pow(&base.mag(), e);
    let power_scale = base.scale() * e as usize;
    let power_sign = base.sign && (e % 2 == 1);
    let power = BcNum::from_mag(power_mag, power_scale, power_sign);
    if neg {
        // 1 / power at scale (power is non-zero here).
        Ok(bc_div(&BcNum::one(), &power, scale).unwrap())
    } else {
        let mut r = power;
        r.truncate_frac(scale);
        Ok(r)
    }
}

enum RaiseModErr {
    BaseFrac,
    ExpoFrac,
    ExpoNeg,
    ModFrac,
    ModZero,
}

fn bc_raisemod(
    base: &BcNum,
    expo: &BcNum,
    modn: &BcNum,
    scale: usize,
) -> Result<BcNum, RaiseModErr> {
    if base.scale() != 0 {
        return Err(RaiseModErr::BaseFrac);
    }
    if expo.scale() != 0 {
        return Err(RaiseModErr::ExpoFrac);
    }
    if expo.sign && !expo.is_zero() {
        return Err(RaiseModErr::ExpoNeg);
    }
    if modn.scale() != 0 {
        return Err(RaiseModErr::ModFrac);
    }
    if modn.is_zero() {
        return Err(RaiseModErr::ModZero);
    }
    // Any integer mod ±1 is 0.
    if cmp_mag(modn, &BcNum::one(), 0) == Ordering::Equal {
        return Ok(BcNum::zero());
    }
    let two = BcNum::from_i64(2);
    let mut temp = BcNum::one();
    let mut power = base.clone();
    let mut exponent = expo.clone();
    while !exponent.is_zero() {
        let (q, parity) = bc_divmod(&exponent, &two, 0).unwrap();
        exponent = q;
        if !parity.is_zero() {
            temp = bc_mul(&temp, &power, scale);
            temp = bc_divmod(&temp, modn, scale).unwrap().1;
        }
        power = bc_mul(&power, &power, scale);
        power = bc_divmod(&power, modn, scale).unwrap().1;
    }
    Ok(temp)
}

/// bc_is_near_zero: all-but-last digit (up to scale) are 0 and the last is 0 or 1.
fn is_near_zero(num: &BcNum, scale: usize) -> bool {
    let s = scale.min(num.scale());
    let mut arr = num.int.clone();
    arr.extend_from_slice(&num.frac[..s]);
    let first = arr.iter().position(|&d| d != 0);
    match first {
        None => true,
        Some(idx) => idx + 1 == arr.len() && arr[idx] == 1,
    }
}

/// Square root truncated to `scale` digits. Returns None for negative input.
fn bc_sqrt(num: &BcNum, scale: usize) -> Option<BcNum> {
    if num.sign && !num.is_zero() {
        return None;
    }
    if num.is_zero() {
        return Some(BcNum::zero());
    }
    let cmp1 = do_compare(num, &BcNum::one(), num.scale());
    if cmp1 == 0 {
        return Some(BcNum::one());
    }
    let rscale = scale.max(num.scale());
    let point5 = BcNum { sign: false, int: vec![0], frac: vec![5] };

    let mut guess;
    let mut cscale;
    if cmp1 < 0 {
        // Between 0 and 1: start guess at 1.
        guess = BcNum::one();
        cscale = num.scale();
    } else {
        // Greater than 1: start guess at 10^(n_len/2).
        let n_len = BcNum::from_i64(num.int.len() as i64);
        let mut g1 = bc_mul(&n_len, &point5, 0);
        g1.truncate_frac(0);
        let exponent = num_to_i64(&g1).unwrap_or(0);
        guess = bc_raise(&BcNum::from_i64(10), exponent, 0).unwrap();
        cscale = 3;
    }

    loop {
        let guess1 = guess.clone();
        guess = bc_div(num, &guess, cscale).unwrap();
        guess = bc_add(&guess, &guess1, 0);
        guess = bc_mul(&guess, &point5, cscale);
        let diff = bc_sub(&guess, &guess1, cscale + 1);
        if is_near_zero(&diff, cscale) {
            if cscale < rscale + 1 {
                cscale = (cscale * 3).min(rscale + 1);
            } else {
                break;
            }
        }
    }
    // Truncate to rscale.
    Some(bc_div(&guess, &BcNum::one(), rscale).unwrap())
}

fn bc_floor_or_ceil(num: &BcNum, is_floor: bool) -> BcNum {
    let mut result = BcNum { sign: num.sign, int: num.int.clone(), frac: Vec::new() };
    // No-op when flooring a positive or ceiling a negative, or no fraction.
    let noop_sign = if is_floor { false } else { true };
    if num.scale() == 0 || result.sign == noop_sign || u_is_zero(&num.frac) {
        if result.is_zero() {
            result.sign = false;
        }
        return result;
    }
    // Round the magnitude away from zero.
    let inc = do_add_mag(&result, &BcNum::one());
    result = BcNum { sign: result.sign, int: inc.int, frac: Vec::new() };
    if result.is_zero() {
        result.sign = false;
    }
    result
}

/// PHP rounding modes.
mod round_mode {
    pub const HALF_UP: i64 = 1;
    pub const HALF_DOWN: i64 = 2;
    pub const HALF_EVEN: i64 = 3;
    pub const HALF_ODD: i64 = 4;
    pub const CEILING: i64 = 5;
    pub const FLOOR: i64 = 6;
    pub const TOWARD_ZERO: i64 = 7;
    pub const AWAY_FROM_ZERO: i64 = 8;
}

/// bc_round: round `num` to `precision` digits. Returns (result, result_scale).
/// Port of round.c operating on the combined digit array.
fn bc_round(num: &BcNum, precision: i64, mode: i64) -> (BcNum, usize) {
    use round_mode::*;
    let n_len = num.int.len();
    let n_scale = num.scale();
    // combined n_value = int ++ frac
    let mut nv: Vec<u8> = num.int.clone();
    nv.extend_from_slice(&num.frac);

    // Rounding to an integer place larger than the number.
    if precision < 0 && (n_len as i64) < -(precision + 1) + 1 {
        let make_zero = || (BcNum::zero(), 0usize);
        match mode {
            HALF_UP | HALF_DOWN | HALF_EVEN | HALF_ODD | TOWARD_ZERO => return make_zero(),
            CEILING => {
                if num.sign {
                    return make_zero();
                }
            }
            FLOOR => {
                if !num.sign {
                    return make_zero();
                }
            }
            AWAY_FROM_ZERO => {}
            _ => {}
        }
        if num.is_zero() {
            return make_zero();
        }
        // Result is 1 followed by (-precision) zeros, with num's sign.
        let len = (-precision + 1) as usize;
        let mut int = vec![0u8; len];
        int[0] = 1;
        return (BcNum { sign: num.sign, int, frac: Vec::new() }, 0);
    }

    // Rounding to a precision at least the current scale: pad, no change.
    if precision >= 0 && (n_scale as i64) <= precision {
        let p = precision as usize;
        let mut frac = num.frac.clone();
        frac.resize(p, 0);
        return (BcNum { sign: num.sign, int: num.int.clone(), frac }, p);
    }

    let rounded_len = (n_len as i64 + precision) as usize; // guaranteed >= 0 here
    let result_scale = if precision > 0 { precision as usize } else { 0 };
    // L = number of digits in the combined result array (int ++ frac).
    let l = n_len + result_scale;
    let sign = num.sign;

    // Magnitude of the truncated result * 10^result_scale: the kept prefix with
    // the dropped positions zeroed out.
    let mut mag: Vec<u8> = vec![0u8; l];
    for i in 0..rounded_len.min(l) {
        mag[i] = nv[i];
    }

    // The first dropped digit.
    let first_dropped = nv[rounded_len];

    let mut do_up;
    let mut need_loop = false;
    match mode {
        HALF_UP => {
            do_up = first_dropped >= 5;
            if !do_up { /* check_zero */ }
        }
        HALF_DOWN | HALF_EVEN | HALF_ODD => {
            if first_dropped > 5 {
                do_up = true;
            } else if first_dropped < 5 {
                do_up = false;
            } else {
                do_up = false;
                need_loop = true;
            }
        }
        CEILING => {
            if sign {
                do_up = false;
            } else if first_dropped > 0 {
                do_up = true;
            } else {
                do_up = false;
                need_loop = true;
            }
        }
        FLOOR => {
            if !sign {
                do_up = false;
            } else if first_dropped > 0 {
                do_up = true;
            } else {
                do_up = false;
                need_loop = true;
            }
        }
        TOWARD_ZERO => {
            do_up = false;
        }
        AWAY_FROM_ZERO => {
            if first_dropped > 0 {
                do_up = true;
            } else {
                do_up = false;
                need_loop = true;
            }
        }
        _ => {
            do_up = first_dropped >= 5;
        }
    }

    if need_loop {
        // Look for any non-zero digit past the first dropped one.
        let mut any_nonzero = false;
        for &d in &nv[rounded_len + 1..] {
            if d != 0 {
                any_nonzero = true;
                break;
            }
        }
        if any_nonzero {
            do_up = true;
        } else {
            match mode {
                HALF_DOWN | CEILING | FLOOR | AWAY_FROM_ZERO => do_up = false,
                HALF_EVEN => {
                    // Round up unless the kept last digit is even.
                    let last_even = rounded_len == 0 || nv[rounded_len - 1] % 2 == 0;
                    do_up = !last_even;
                }
                HALF_ODD => {
                    let last_odd = rounded_len != 0 && nv[rounded_len - 1] % 2 == 1;
                    do_up = !last_odd;
                }
                _ => do_up = false,
            }
        }
    }

    if do_up {
        // Add 1 at the last kept position: +10^(L - rounded_len).
        mag = u_add(&mag, &u_shl(&[1], l - rounded_len));
    }

    // Build the result and normalise; the returned scale is always result_scale
    // (num2str pads a zero result to it).
    let mut result = BcNum::from_mag(mag, result_scale, sign);
    if result.is_zero() {
        result.sign = false;
    }
    (result, result_scale)
}

/// Convert to i64, ignoring fractional part. None on overflow.
fn num_to_i64(n: &BcNum) -> Option<i64> {
    let mut acc: i64 = 0;
    for &d in &n.int {
        acc = acc.checked_mul(10)?.checked_add(d as i64)?;
    }
    if n.sign {
        Some(-acc)
    } else {
        Some(acc)
    }
}

/* ------------------------------------------------------------------ */
/* PHP-facing wrappers                                                  */
/* ------------------------------------------------------------------ */

fn read_scale(
    args: &[Zval],
    ctx: &mut Ctx,
    idx: usize,
    fname: &str,
    argnum: u32,
) -> Result<usize, PhpError> {
    match args.get(idx) {
        None | Some(Zval::Null) => Ok(BC_SCALE.with(Cell::get)),
        Some(v) => {
            let s = convert::to_long_cast(v, ctx.diags);
            if !(0..=SCALE_MAX).contains(&s) {
                Err(PhpError::ValueError(format!(
                    "{fname}(): Argument #{argnum} ($scale) must be between 0 and 2147483647"
                )))
            } else {
                Ok(s as usize)
            }
        }
    }
}

fn parse_arg(
    args: &[Zval],
    ctx: &mut Ctx,
    idx: usize,
    fname: &str,
    argnum: u32,
    pname: &str,
) -> Result<BcNum, PhpError> {
    let v = args.get(idx).unwrap_or(&Zval::Null);
    let s = convert::to_zstr(v, ctx.diags);
    bc_parse(s.as_bytes()).ok_or_else(|| {
        PhpError::ValueError(format!(
            "{fname}(): Argument #{argnum} (${pname}) is not well-formed"
        ))
    })
}

fn ret_str(bytes: Vec<u8>) -> Result<Zval, PhpError> {
    Ok(Zval::Str(PhpStr::new(bytes)))
}

pub fn bcadd(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = parse_arg(args, ctx, 0, "bcadd", 1, "num1")?;
    let b = parse_arg(args, ctx, 1, "bcadd", 2, "num2")?;
    let scale = read_scale(args, ctx, 2, "bcadd", 3)?;
    ret_str(bc_add(&a, &b, scale).format(scale))
}

pub fn bcsub(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = parse_arg(args, ctx, 0, "bcsub", 1, "num1")?;
    let b = parse_arg(args, ctx, 1, "bcsub", 2, "num2")?;
    let scale = read_scale(args, ctx, 2, "bcsub", 3)?;
    ret_str(bc_sub(&a, &b, scale).format(scale))
}

pub fn bcmul(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = parse_arg(args, ctx, 0, "bcmul", 1, "num1")?;
    let b = parse_arg(args, ctx, 1, "bcmul", 2, "num2")?;
    let scale = read_scale(args, ctx, 2, "bcmul", 3)?;
    ret_str(bc_mul(&a, &b, scale).format(scale))
}

pub fn bcdiv(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = parse_arg(args, ctx, 0, "bcdiv", 1, "num1")?;
    let b = parse_arg(args, ctx, 1, "bcdiv", 2, "num2")?;
    let scale = read_scale(args, ctx, 2, "bcdiv", 3)?;
    match bc_div(&a, &b, scale) {
        Some(q) => ret_str(q.format(scale)),
        None => Err(PhpError::DivisionByZeroError("Division by zero")),
    }
}

pub fn bcmod(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = parse_arg(args, ctx, 0, "bcmod", 1, "num1")?;
    let b = parse_arg(args, ctx, 1, "bcmod", 2, "num2")?;
    let scale = read_scale(args, ctx, 2, "bcmod", 3)?;
    match bc_divmod(&a, &b, scale) {
        Some((_, rem)) => ret_str(rem.format(scale)),
        None => Err(PhpError::DivisionByZeroError("Modulo by zero")),
    }
}

pub fn bcdivmod(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = parse_arg(args, ctx, 0, "bcdivmod", 1, "num1")?;
    let b = parse_arg(args, ctx, 1, "bcdivmod", 2, "num2")?;
    let scale = read_scale(args, ctx, 2, "bcdivmod", 3)?;
    match bc_divmod(&a, &b, scale) {
        Some((quot, rem)) => {
            let mut arr = PhpArray::new();
            let _ = arr.append(Zval::Str(PhpStr::new(quot.format(0))));
            let _ = arr.append(Zval::Str(PhpStr::new(rem.format(scale))));
            Ok(Zval::Array(Rc::new(arr)))
        }
        None => Err(PhpError::DivisionByZeroError("Division by zero")),
    }
}

pub fn bcpow(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let base = parse_arg(args, ctx, 0, "bcpow", 1, "num")?;
    let expo = parse_arg(args, ctx, 1, "bcpow", 2, "exponent")?;
    let scale = read_scale(args, ctx, 2, "bcpow", 3)?;
    if expo.scale() != 0 {
        return Err(PhpError::ValueError(
            "bcpow(): Argument #2 ($exponent) cannot have a fractional part".to_string(),
        ));
    }
    let exponent = match num_to_i64(&expo) {
        Some(e) => e,
        None => {
            return Err(PhpError::ValueError(
                "bcpow(): Argument #2 ($exponent) is too large".to_string(),
            ))
        }
    };
    match bc_raise(&base, exponent, scale) {
        Ok(r) => ret_str(r.format(scale)),
        Err(RaiseErr::DivByZero) => {
            Err(PhpError::DivisionByZeroError("Negative power of zero"))
        }
    }
}

pub fn bcpowmod(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let base = parse_arg(args, ctx, 0, "bcpowmod", 1, "num")?;
    let expo = parse_arg(args, ctx, 1, "bcpowmod", 2, "exponent")?;
    let modn = parse_arg(args, ctx, 2, "bcpowmod", 3, "modulus")?;
    let scale = read_scale(args, ctx, 3, "bcpowmod", 4)?;
    match bc_raisemod(&base, &expo, &modn, scale) {
        Ok(r) => ret_str(r.format(scale)),
        Err(RaiseModErr::BaseFrac) => Err(PhpError::ValueError(
            "bcpowmod(): Argument #1 ($num) cannot have a fractional part".to_string(),
        )),
        Err(RaiseModErr::ExpoFrac) => Err(PhpError::ValueError(
            "bcpowmod(): Argument #2 ($exponent) cannot have a fractional part".to_string(),
        )),
        Err(RaiseModErr::ExpoNeg) => Err(PhpError::ValueError(
            "bcpowmod(): Argument #2 ($exponent) must be greater than or equal to 0".to_string(),
        )),
        Err(RaiseModErr::ModFrac) => Err(PhpError::ValueError(
            "bcpowmod(): Argument #3 ($modulus) cannot have a fractional part".to_string(),
        )),
        Err(RaiseModErr::ModZero) => Err(PhpError::DivisionByZeroError("Modulo by zero")),
    }
}

pub fn bcsqrt(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let num = parse_arg(args, ctx, 0, "bcsqrt", 1, "num")?;
    let scale = read_scale(args, ctx, 1, "bcsqrt", 2)?;
    match bc_sqrt(&num, scale) {
        Some(r) => ret_str(r.format(scale)),
        None => Err(PhpError::ValueError(
            "bcsqrt(): Argument #1 ($num) must be greater than or equal to 0".to_string(),
        )),
    }
}

pub fn bccomp(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = parse_arg(args, ctx, 0, "bccomp", 1, "num1")?;
    let b = parse_arg(args, ctx, 1, "bccomp", 2, "num2")?;
    let scale = read_scale(args, ctx, 2, "bccomp", 3)?;
    Ok(Zval::Long(do_compare(&a, &b, scale)))
}

pub fn bcscale(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let old = BC_SCALE.with(Cell::get);
    match args.first() {
        None | Some(Zval::Null) => {}
        Some(v) => {
            let s = convert::to_long_cast(v, ctx.diags);
            if !(0..=SCALE_MAX).contains(&s) {
                return Err(PhpError::ValueError(
                    "bcscale(): Argument #1 ($scale) must be between 0 and 2147483647".to_string(),
                ));
            }
            BC_SCALE.with(|c| c.set(s as usize));
        }
    }
    Ok(Zval::Long(old as i64))
}

pub fn bcfloor(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let num = parse_arg(args, ctx, 0, "bcfloor", 1, "num")?;
    ret_str(bc_floor_or_ceil(&num, true).format(0))
}

pub fn bcceil(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let num = parse_arg(args, ctx, 0, "bcceil", 1, "num")?;
    ret_str(bc_floor_or_ceil(&num, false).format(0))
}

pub fn bcround(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let num = parse_arg(args, ctx, 0, "bcround", 1, "num")?;
    let precision = args.get(1).map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(0);
    // The RoundingMode enum is not modelled in phpr; default HalfAwayFromZero.
    let (result, scale) = bc_round(&num, precision, round_mode::HALF_UP);
    ret_str(result.format(scale))
}
