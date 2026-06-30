//! OpenSSL builtins. phpr models TLS via a rustls-backed stream wrapper rather
//! than linking OpenSSL, so these reproduce the *observable* PHP API on top of
//! pure-Rust crates.
//!
//! `openssl_x509_parse` parses a certificate with the `x509-parser` crate and
//! returns a *faithful-core* of OpenSSL's info array: truthy for any valid
//! certificate (the contract Composer's `CaBundle` — `(bool)
//! openssl_x509_parse(...)` — and most callers depend on), and byte-correct on
//! the common fields (version, validity + `_time_t`, signatureType*, and the
//! C/O/CN/OU/ST/L of subject & issuer).
//!
//! It is deliberately NOT a byte-for-byte reproduction of OpenSSL's certificate
//! *text rendering*, which is an OpenSSL-internals rabbit hole the supported
//! surface never reads:
//!   * `hash`, `purposes`, `extensions` — omitted (X509_NAME_hash /
//!     X509_check_purpose / per-extension text formatting).
//!   * `name` and the `0x`-vs-decimal `serialNumber` form follow OpenSSL's
//!     common cases but not every X509_NAME_oneline escaping / large-serial
//!     threshold edge.
//! Emitting approximate values for the omitted keys would be less honest than
//! their absence.

use std::rc::Rc;

use php_runtime::Ctx;
use php_types::{convert, Key, PhpArray, PhpError, PhpStr, Zval};

use x509_parser::asn1_rs::Oid;
use x509_parser::der_parser::num_bigint::BigUint;
use x509_parser::objects::{oid2abbrev, oid2sn, oid_registry};
use x509_parser::prelude::*;

/// OpenSSL OBJ-table entries for the signature algorithms common in TLS chains:
/// (OID dotted string, signatureTypeSN, signatureTypeLN, signatureTypeNID).
/// OpenSSL's SN/LN/NID are not derivable from a generic OID registry (e.g. SN
/// "RSA-SHA256" vs LN "sha256WithRSAEncryption"), so the reproducible ones are
/// tabled explicitly; anything else falls back to the registry short name with
/// the NID omitted.
const SIG_ALGS: &[(&str, &str, &str, i64)] = &[
    ("1.2.840.113549.1.1.5", "RSA-SHA1", "sha1WithRSAEncryption", 65),
    ("1.2.840.113549.1.1.11", "RSA-SHA256", "sha256WithRSAEncryption", 668),
    ("1.2.840.113549.1.1.12", "RSA-SHA384", "sha384WithRSAEncryption", 669),
    ("1.2.840.113549.1.1.13", "RSA-SHA512", "sha512WithRSAEncryption", 670),
    ("1.2.840.10045.4.3.2", "ecdsa-with-SHA256", "ecdsa-with-SHA256", 794),
    ("1.2.840.10045.4.3.3", "ecdsa-with-SHA384", "ecdsa-with-SHA384", 795),
    ("1.2.840.10045.4.3.4", "ecdsa-with-SHA512", "ecdsa-with-SHA512", 796),
    ("1.3.101.112", "ED25519", "ED25519", 1087),
];

/// `openssl_x509_parse(string $certificate, bool $short_names = true)`: parse a
/// PEM/DER certificate into the info array, or `false` on failure.
pub fn openssl_x509_parse(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let pem = convert::to_zstr(&args[0], ctx.diags);
    // `$short_names` (default true) chooses the DN key style: "CN" vs "commonName".
    let short_names = match args.get(1) {
        Some(Zval::Bool(b)) => *b,
        Some(Zval::Long(n)) => *n != 0,
        Some(Zval::Null) => false,
        _ => true,
    };
    match parse_first_cert(pem.as_bytes(), short_names) {
        Some(arr) => Ok(Zval::Array(Rc::new(arr))),
        None => Ok(Zval::Bool(false)),
    }
}

/// Parse the first certificate (PEM block, else raw DER) and build the info
/// array; `None` on any parse failure (PHP returns `false`).
fn parse_first_cert(bytes: &[u8], short_names: bool) -> Option<PhpArray> {
    // PEM first — the common case: a "-----BEGIN CERTIFICATE-----" block, possibly
    // within a bundle; only the first is parsed, mirroring openssl_x509_parse.
    if let Some(first) = Pem::iter_from_buffer(bytes).next() {
        return match first {
            Ok(pem) => pem.parse_x509().ok().map(|c| build_info(&c, short_names)),
            Err(_) => None,
        };
    }
    // Fall back to raw DER.
    X509Certificate::from_der(bytes)
        .ok()
        .map(|(_, cert)| build_info(&cert, short_names))
}

fn build_info(cert: &X509Certificate, short_names: bool) -> PhpArray {
    let mut arr = PhpArray::new();

    arr.insert(Key::from_bytes(b"name"), str_zval(&dn_oneline(cert.subject())));
    arr.insert(
        Key::from_bytes(b"subject"),
        Zval::Array(Rc::new(dn_array(cert.subject(), short_names))),
    );
    arr.insert(
        Key::from_bytes(b"issuer"),
        Zval::Array(Rc::new(dn_array(cert.issuer(), short_names))),
    );
    arr.insert(Key::from_bytes(b"version"), Zval::Long(cert.version().0 as i64));

    // OpenSSL prints `serialNumber` as decimal for a positive integer but as
    // "0x"+hex when the DER integer is negative (first content byte's high bit
    // set); `serialNumberHex` is always the magnitude hex. x509-parser exposes
    // the serial as an unsigned BigUint, so the sign is read from the raw bytes.
    let serial = &cert.tbs_certificate.serial;
    let hex = serial_hex(serial);
    let negative = cert.raw_serial().first().map_or(false, |b| b & 0x80 != 0);
    let serial_number = if negative {
        format!("0x{hex}")
    } else {
        serial.to_str_radix(10)
    };
    arr.insert(Key::from_bytes(b"serialNumber"), str_zval(serial_number.as_bytes()));
    arr.insert(Key::from_bytes(b"serialNumberHex"), str_zval(hex.as_bytes()));

    let nb = cert.validity().not_before;
    let na = cert.validity().not_after;
    arr.insert(Key::from_bytes(b"validFrom"), str_zval(asn1_time(&nb).as_bytes()));
    arr.insert(Key::from_bytes(b"validTo"), str_zval(asn1_time(&na).as_bytes()));
    arr.insert(Key::from_bytes(b"validFrom_time_t"), Zval::Long(nb.timestamp()));
    arr.insert(Key::from_bytes(b"validTo_time_t"), Zval::Long(na.timestamp()));

    let (sn, ln, nid) = sig_alg(cert);
    arr.insert(Key::from_bytes(b"signatureTypeSN"), str_zval(sn.as_bytes()));
    arr.insert(Key::from_bytes(b"signatureTypeLN"), str_zval(ln.as_bytes()));
    if let Some(nid) = nid {
        arr.insert(Key::from_bytes(b"signatureTypeNID"), Zval::Long(nid));
    }
    arr
}

fn str_zval(b: &[u8]) -> Zval {
    Zval::Str(PhpStr::new(b.to_vec()))
}

/// OpenSSL short name ("CN"/"C"/"O") or long name ("commonName") for a DN OID.
/// With `short_names`, OpenSSL uses the abbreviation when one exists, else the
/// long name (e.g. 2.5.4.97 has no abbreviation, so it prints
/// "organizationIdentifier"); the dotted OID is the final fallback.
fn dn_key(oid: &Oid, short_names: bool) -> String {
    let reg = oid_registry();
    let primary = if short_names {
        oid2abbrev(oid, reg)
    } else {
        oid2sn(oid, reg)
    };
    primary
        .or_else(|_| oid2sn(oid, reg))
        .map(|s| s.to_string())
        .unwrap_or_else(|_| oid.to_id_string())
}

/// DN attribute value as a String. `as_str()` covers the usual UTF8String /
/// PrintableString / IA5String; other ASN.1 string types (T61String, etc.) are
/// best-effort lossy-decoded from the raw content bytes rather than dropped.
fn dn_value(attr: &x509_parser::x509::AttributeTypeAndValue) -> String {
    match attr.as_str() {
        Ok(s) => s.to_string(),
        Err(_) => String::from_utf8_lossy(attr.attr_value().data).into_owned(),
    }
}

/// Build the subject/issuer associative array. A repeated attribute type
/// collapses to a nested array of its values, as OpenSSL does.
fn dn_array(name: &X509Name, short_names: bool) -> PhpArray {
    let mut arr = PhpArray::new();
    for attr in name.iter_attributes() {
        let key = dn_key(attr.attr_type(), short_names);
        let val = dn_value(attr);
        let k = Key::from_bytes(key.as_bytes());
        let merged = match arr.get(&k).cloned() {
            None => str_zval(val.as_bytes()),
            Some(Zval::Array(existing)) => {
                let mut a = (*existing).clone();
                let _ = a.append(str_zval(val.as_bytes()));
                Zval::Array(Rc::new(a))
            }
            Some(prev) => {
                let mut a = PhpArray::new();
                let _ = a.append(prev);
                let _ = a.append(str_zval(val.as_bytes()));
                Zval::Array(Rc::new(a))
            }
        };
        arr.insert(k, merged);
    }
    arr
}

/// OpenSSL one-line DN: `/C=US/O=Org/CN=Name`, attributes in encoding order.
fn dn_oneline(name: &X509Name) -> Vec<u8> {
    let mut out = Vec::new();
    for attr in name.iter_attributes() {
        let key = dn_key(attr.attr_type(), true);
        let val = dn_value(attr);
        out.push(b'/');
        out.extend_from_slice(key.as_bytes());
        out.push(b'=');
        out.extend_from_slice(val.as_bytes());
    }
    out
}

/// Uppercase hex of the serial's value bytes (no separators), matching
/// OpenSSL's `serialNumberHex` (BN_bn2hex of the positive bignum).
fn serial_hex(serial: &BigUint) -> String {
    let bytes = serial.to_bytes_be();
    if bytes.is_empty() {
        // BN_bn2hex(0) is "0", not "00".
        return "0".to_string();
    }
    bytes.iter().map(|b| format!("{b:02X}")).collect()
}

/// Render an ASN.1 validity time as OpenSSL prints it: UTCTime
/// `YYMMDDHHMMSSZ` for years 1950..2050, GeneralizedTime `YYYYMMDDHHMMSSZ`
/// otherwise.
fn asn1_time(t: &ASN1Time) -> String {
    let dt = t.to_datetime();
    let (year, mo, d, h, mi, s) = (
        dt.year(),
        u8::from(dt.month()),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second(),
    );
    if (1950..2050).contains(&year) {
        let yy = year.rem_euclid(100);
        format!("{yy:02}{mo:02}{d:02}{h:02}{mi:02}{s:02}Z")
    } else {
        format!("{year:04}{mo:02}{d:02}{h:02}{mi:02}{s:02}Z")
    }
}

/// OpenSSL signatureTypeSN/LN/NID for the certificate's signature algorithm.
fn sig_alg(cert: &X509Certificate) -> (String, String, Option<i64>) {
    let oid = cert.signature_algorithm.oid();
    let oid_str = oid.to_id_string();
    for (o, sn, ln, nid) in SIG_ALGS {
        if *o == oid_str {
            return (sn.to_string(), ln.to_string(), Some(*nid));
        }
    }
    let sn = oid2sn(oid, oid_registry())
        .map(|s| s.to_string())
        .unwrap_or_else(|_| oid_str.clone());
    (sn.clone(), sn, None)
}
