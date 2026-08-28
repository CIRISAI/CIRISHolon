//! Reading the pinned referee, and comparing against it WITHOUT throwing its precision
//! away.
//!
//! # The trap this module exists to avoid
//!
//! The obvious way to compare a 50-digit referee to an f64 is `referee.parse::<f64>()`
//! and subtract. That silently rounds the referee to f64 first, so the comparison can
//! never resolve anything below half an ulp of the value — about `1.1e-16` hartree here.
//! At the bounds this crate stakes (`5e-15`) that is a 2% contamination, which is
//! survivable; at any tighter bound the gate would be measuring its own rounding and
//! reporting it as agreement. [`decimal_minus_f64`] does the subtraction in exact
//! fixed-point decimal instead and only converts the (tiny) difference to f64, so the
//! residual it returns is the real one however small it gets.

use std::path::PathBuf;

pub fn referee_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/referee_h2_sto3g_fci.json")
}

pub fn referee_bytes() -> Vec<u8> {
    std::fs::read(referee_path()).expect("pinned referee curve present")
}

pub fn referee_text() -> String {
    String::from_utf8(referee_bytes()).expect("referee file is UTF-8")
}

/// Extract a JSON array of decimal STRINGS. Deliberately not a general parser: the file
/// is a fixture this crate owns, and a general parser would be more code to trust than
/// the thing it is checking.
pub fn string_array(src: &str, key: &str) -> Vec<String> {
    let needle = format!("\"{key}\":");
    let at = src
        .find(&needle)
        .unwrap_or_else(|| panic!("referee is missing \"{key}\""))
        + needle.len();
    let rest = src[at..].trim_start();
    let rest = rest
        .strip_prefix('[')
        .unwrap_or_else(|| panic!("\"{key}\" is not an array"));
    let end = rest
        .find(']')
        .unwrap_or_else(|| panic!("\"{key}\" is unterminated"));
    rest[..end]
        .split(',')
        .map(|t| t.trim().trim_matches('"').to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

/// Extract a single JSON string value.
pub fn string_scalar(src: &str, key: &str) -> String {
    let needle = format!("\"{key}\":");
    let at = src
        .find(&needle)
        .unwrap_or_else(|| panic!("referee is missing \"{key}\""))
        + needle.len();
    let rest = src[at..].trim_start();
    let rest = rest
        .strip_prefix('"')
        .unwrap_or_else(|| panic!("\"{key}\" is not a string"));
    let end = rest.find('"').expect("unterminated string");
    rest[..end].to_string()
}

const INT_DIGITS: usize = 4;
const FRAC_DIGITS: usize = 64;
const TOTAL: usize = INT_DIGITS + FRAC_DIGITS;

/// A plain decimal (no exponent) as a sign plus a fixed-point digit string.
fn to_fixed(s: &str) -> (bool, [u8; TOTAL]) {
    let s = s.trim();
    let (neg, body) = match s.strip_prefix('-') {
        Some(b) => (true, b),
        None => (false, s.strip_prefix('+').unwrap_or(s)),
    };
    assert!(
        !body.contains('e') && !body.contains('E'),
        "exponent notation is not handled: {s:?}"
    );
    let (int_part, frac_part) = match body.split_once('.') {
        Some((i, f)) => (i, f),
        None => (body, ""),
    };
    assert!(
        int_part.len() <= INT_DIGITS,
        "integer part too wide for the fixed point: {s:?}"
    );
    assert!(
        frac_part.len() <= FRAC_DIGITS,
        "fraction too long for the fixed point: {s:?}"
    );
    let mut out = [0u8; TOTAL];
    for (k, c) in int_part.bytes().rev().enumerate() {
        out[INT_DIGITS - 1 - k] = c - b'0';
    }
    for (k, c) in frac_part.bytes().enumerate() {
        out[INT_DIGITS + k] = c - b'0';
    }
    (neg, out)
}

fn cmp_mag(a: &[u8; TOTAL], b: &[u8; TOTAL]) -> std::cmp::Ordering {
    for i in 0..TOTAL {
        match a[i].cmp(&b[i]) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    std::cmp::Ordering::Equal
}

fn sub_mag(a: &[u8; TOTAL], b: &[u8; TOTAL]) -> [u8; TOTAL] {
    let mut out = [0u8; TOTAL];
    let mut borrow = 0i16;
    for i in (0..TOTAL).rev() {
        let mut d = a[i] as i16 - b[i] as i16 - borrow;
        if d < 0 {
            d += 10;
            borrow = 1;
        } else {
            borrow = 0;
        }
        out[i] = d as u8;
    }
    assert_eq!(borrow, 0, "magnitude subtraction underflowed");
    out
}

fn fixed_to_f64(neg: bool, d: &[u8; TOTAL]) -> f64 {
    let mut s = String::with_capacity(TOTAL + 3);
    if neg {
        s.push('-');
    }
    for c in d.iter().take(INT_DIGITS) {
        s.push((b'0' + c) as char);
    }
    s.push('.');
    for c in d.iter().skip(INT_DIGITS) {
        s.push((b'0' + c) as char);
    }
    s.parse::<f64>().expect("fixed point is a decimal literal")
}

fn add_signed(a: (bool, [u8; TOTAL]), b: (bool, [u8; TOTAL])) -> (bool, [u8; TOTAL]) {
    if a.0 == b.0 {
        let mut out = [0u8; TOTAL];
        let mut carry = 0u8;
        for i in (0..TOTAL).rev() {
            let s = a.1[i] + b.1[i] + carry;
            out[i] = s % 10;
            carry = s / 10;
        }
        assert_eq!(carry, 0, "fixed-point addition overflowed");
        (a.0, out)
    } else if cmp_mag(&a.1, &b.1) == std::cmp::Ordering::Less {
        (b.0, sub_mag(&b.1, &a.1))
    } else {
        (a.0, sub_mag(&a.1, &b.1))
    }
}

/// `decimal - x`, computed in exact fixed-point decimal and returned as the f64 nearest
/// the true difference.
///
/// `x` is rendered with `{:.64}`, which for an f64 of this magnitude is its own decimal
/// expansion rounded at the 64th place — an error of at most `1e-64`, i.e. 48 orders
/// below anything this gate resolves. The subtraction itself is exact.
pub fn decimal_minus_f64(decimal: &str, x: f64) -> f64 {
    let a = to_fixed(decimal);
    let b_str = format!("{:.*}", FRAC_DIGITS, x);
    let mut b = to_fixed(&b_str);
    b.0 = !b.0;
    let (neg, mag) = add_signed(a, b);
    fixed_to_f64(neg, &mag)
}

/// Worst absolute residual of `mine` against a column of referee decimals, with the R
/// where it occurred.
pub fn worst(referee: &[String], mine: &[f64]) -> (f64, usize) {
    assert_eq!(referee.len(), mine.len(), "column length mismatch");
    let mut w = 0.0f64;
    let mut at = 0usize;
    for (i, (r, m)) in referee.iter().zip(mine.iter()).enumerate() {
        let d = decimal_minus_f64(r, *m).abs();
        if d > w {
            w = d;
            at = i;
        }
    }
    (w, at)
}
