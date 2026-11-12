//! One prompt's pass through a model, checked: the writes must sum to the state.
//!
//! ```text
//! cargo run --release -p holon-closure --example trace_check -- TRACE.bin [--tolerance 0]
//! ```
//!
//! For the kindscope explorer (`conformance/reasoning/scope0`): standing at a site and
//! watching one prompt pass is only honest if what is drawn adds up. A residual stream is a
//! ledger: at each position the state is the sum of what each block wrote. This program
//! refuses a trace in which that does not hold.
//!
//! # The file
//!
//! One line of text, then raw little-endian `f32`:
//!
//! ```text
//! holon-trace 1 positions P width W writes K <name_1> ... <name_K>\n
//! ```
//!
//! then, for each of the `P` positions in order: `K * W` values (the writes, in the order
//! named, which must be the order the model adds them in), then `W` values (the state the
//! model itself held after the last write, before any final normalisation).
//!
//! # What is checked
//!
//! * **Conservation.** At each position, adding the writes IN THE NAMED ORDER in `f32`
//!   reproduces the state. With the default tolerance `0` this is bit for bit: the same
//!   additions in the same order give the same float. A tolerance above zero is for a frame
//!   known to close only approximately, and is printed with the result.
//! * **Nothing missing.** A file shorter or longer than its header says, a write count of
//!   zero, a repeated write name, or a value that is not finite: REFUSED, with the reason.
//!
//! It does not rebuild logits: that needs the model's head, which is not this crate's.

use std::fmt;

/// A parsed trace: `writes[p][k]` is write `k` at position `p`.
#[derive(Clone, Debug, PartialEq)]
pub struct Trace {
    pub width: usize,
    pub names: Vec<String>,
    pub writes: Vec<Vec<Vec<f32>>>,
    pub states: Vec<Vec<f32>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Refusal {
    Header(String),
    Length { expected: usize, found: usize },
    NotFinite { position: usize, what: String },
    NotConserved { position: usize, coordinate: usize, summed: f32, state: f32, positions_off: usize },
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::Header(m) => write!(f, "header: {m}"),
            Refusal::Length { expected, found } => write!(f, "the header promises {expected} bytes of values, the file holds {found}: a write is missing or extra"),
            Refusal::NotFinite { position, what } => write!(f, "position {position}: a value that is not finite in {what}"),
            Refusal::NotConserved { position, coordinate, summed, state, positions_off } => {
                write!(f, "the writes do not sum to the state at {positions_off} position(s); first at position {position}, coordinate {coordinate}: summed {summed:e}, state {state:e}")
            }
        }
    }
}

pub fn parse(bytes: &[u8]) -> Result<Trace, Refusal> {
    let nl = bytes.iter().position(|&b| b == b'\n').ok_or_else(|| Refusal::Header("no header line".into()))?;
    let head = std::str::from_utf8(&bytes[..nl]).map_err(|_| Refusal::Header("not text".into()))?;
    let t: Vec<&str> = head.split_whitespace().collect();
    let bad = |m: &str| Refusal::Header(m.to_string());
    if t.len() < 9 || t[0] != "holon-trace" || t[1] != "1" || t[2] != "positions" || t[4] != "width" || t[6] != "writes" {
        return Err(bad("expected `holon-trace 1 positions P width W writes K name...`"));
    }
    let num = |s: &str| s.parse::<usize>().map_err(|_| bad("a count that is not a number"));
    let (p, w, k) = (num(t[3])?, num(t[5])?, num(t[7])?);
    let names: Vec<String> = t[8..].iter().map(|s| s.to_string()).collect();
    if k == 0 || w == 0 || names.len() != k {
        return Err(bad("the number of names is not the number of writes, or a count is zero"));
    }
    for (i, n) in names.iter().enumerate() {
        if names[..i].contains(n) {
            return Err(bad("a write is named twice"));
        }
    }
    let body = &bytes[nl + 1..];
    let expected = p * (k + 1) * w * 4;
    if body.len() != expected {
        return Err(Refusal::Length { expected, found: body.len() });
    }
    let val = |i: usize| f32::from_le_bytes([body[4 * i], body[4 * i + 1], body[4 * i + 2], body[4 * i + 3]]);
    let (mut writes, mut states) = (Vec::with_capacity(p), Vec::with_capacity(p));
    for pos in 0..p {
        let base = pos * (k + 1) * w;
        let mut ws = Vec::with_capacity(k);
        for (j, name) in names.iter().enumerate() {
            let v: Vec<f32> = (0..w).map(|c| val(base + j * w + c)).collect();
            if v.iter().any(|x| !x.is_finite()) {
                return Err(Refusal::NotFinite { position: pos, what: name.clone() });
            }
            ws.push(v);
        }
        let s: Vec<f32> = (0..w).map(|c| val(base + k * w + c)).collect();
        if s.iter().any(|x| !x.is_finite()) {
            return Err(Refusal::NotFinite { position: pos, what: "the state".into() });
        }
        writes.push(ws);
        states.push(s);
    }
    Ok(Trace { width: w, names, writes, states })
}

/// What a conserving trace reports.
#[derive(Clone, Debug, PartialEq)]
pub struct Report {
    pub positions: usize,
    pub largest_difference: f32,
    /// Each write's share of the summed squared norm of all writes, in the named order.
    pub share_of_norm: Vec<f64>,
}

pub fn check(t: &Trace, tolerance: f32) -> Result<Report, Refusal> {
    let mut largest = 0f32;
    let mut first: Option<(usize, usize, f32, f32)> = None;
    let mut off = 0usize;
    let mut norms = vec![0f64; t.names.len()];
    for (p, (ws, s)) in t.writes.iter().zip(&t.states).enumerate() {
        let mut bad_here = false;
        for c in 0..t.width {
            let mut acc = 0f32;
            for w in ws {
                acc += w[c];
            }
            let d = (acc - s[c]).abs();
            largest = largest.max(d);
            if d > tolerance {
                bad_here = true;
                first.get_or_insert((p, c, acc, s[c]));
            }
        }
        off += bad_here as usize;
        for (j, w) in ws.iter().enumerate() {
            norms[j] += w.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>();
        }
    }
    if let Some((position, coordinate, summed, state)) = first {
        return Err(Refusal::NotConserved { position, coordinate, summed, state, positions_off: off });
    }
    let total: f64 = norms.iter().sum();
    Ok(Report { positions: t.states.len(), largest_difference: largest, share_of_norm: norms.iter().map(|n| if total > 0.0 { n / total } else { 0.0 }).collect() })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: trace_check TRACE.bin [--tolerance T]");
        std::process::exit(2);
    }
    let tol: f32 = args.iter().position(|a| a == "--tolerance").and_then(|i| args.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let bytes = std::fs::read(&args[1]).unwrap_or_else(|e| {
        eprintln!("{}: {e}", args[1]);
        std::process::exit(2)
    });
    match parse(&bytes).and_then(|t| check(&t, tol).map(|r| (t, r))) {
        Err(r) => {
            println!("REFUSED  {r}");
            std::process::exit(1)
        }
        Ok((t, r)) => {
            println!("CONSERVED  {} positions x {} writes x {} wide; tolerance {tol:e}; largest difference {:e}", r.positions, t.names.len(), t.width, r.largest_difference);
            for (n, s) in t.names.iter().zip(&r.share_of_norm) {
                println!("  {n:<10} {:.4} of the writes' squared norm", s);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A trace whose state IS the sequential f32 sum of its writes.
    fn trace(positions: usize, width: usize, names: &[&str], spoil: Option<(usize, usize)>) -> Vec<u8> {
        let mut out = format!("holon-trace 1 positions {positions} width {width} writes {} {}\n", names.len(), names.join(" ")).into_bytes();
        let mut z = 0x9E3779B97F4A7C15u64;
        let mut next = || {
            z ^= z << 13;
            z ^= z >> 7;
            z ^= z << 17;
            ((z >> 40) as f32 / 16_777_216.0 - 0.5) * 8.0
        };
        for p in 0..positions {
            let ws: Vec<Vec<f32>> = names.iter().map(|_| (0..width).map(|_| next()).collect()).collect();
            let mut s = vec![0f32; width];
            for w in &ws {
                for c in 0..width {
                    s[c] += w[c];
                }
            }
            if let Some((sp, sc)) = spoil {
                if sp == p {
                    s[sc] = f32::from_bits(s[sc].to_bits() ^ 1); // one unit in the last place
                }
            }
            for w in &ws {
                for x in w {
                    out.extend_from_slice(&x.to_le_bytes());
                }
            }
            for x in &s {
                out.extend_from_slice(&x.to_le_bytes());
            }
        }
        out
    }

    const NAMES: [&str; 5] = ["embed", "attn0", "mlp0", "attn1", "mlp1"];

    #[test]
    fn a_trace_that_adds_up_is_conserved_bit_for_bit() {
        let r = check(&parse(&trace(7, 16, &NAMES, None)).unwrap(), 0.0).unwrap();
        assert_eq!((r.positions, r.largest_difference), (7, 0.0));
        assert!((r.share_of_norm.iter().sum::<f64>() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn one_unit_in_the_last_place_is_caught_and_located() {
        let t = parse(&trace(7, 16, &NAMES, Some((4, 9)))).unwrap();
        match check(&t, 0.0) {
            Err(Refusal::NotConserved { position: 4, coordinate: 9, positions_off: 1, .. }) => {}
            other => panic!("{other:?}"),
        }
        // a stated tolerance admits it, and the report says how far off it was
        assert!(check(&t, 1e-5).unwrap().largest_difference > 0.0);
    }

    #[test]
    fn the_order_of_the_writes_is_part_of_the_claim() {
        // the same writes named in another order sum, in f32, to a different float somewhere
        let good = trace(40, 64, &NAMES, None);
        let mut t = parse(&good).unwrap();
        for ws in t.writes.iter_mut() {
            ws.reverse();
        }
        assert!(matches!(check(&t, 0.0), Err(Refusal::NotConserved { .. })));
    }

    #[test]
    fn a_missing_write_a_repeated_name_and_a_bad_value_are_refused() {
        let mut short = trace(3, 8, &NAMES, None);
        short.truncate(short.len() - 8 * 4);
        assert!(matches!(parse(&short), Err(Refusal::Length { .. })));
        assert!(matches!(parse(&trace(3, 8, &["embed", "attn0", "attn0"], None)), Err(Refusal::Header(_))));
        let mut nan = trace(3, 8, &NAMES, None);
        let at = nan.iter().position(|&b| b == b'\n').unwrap() + 1 + 4 * 11;
        nan[at..at + 4].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(matches!(parse(&nan), Err(Refusal::NotFinite { position: 0, .. })));
        assert!(matches!(parse(b"holon-trace 2 positions 1 width 1 writes 1 a\n"), Err(Refusal::Header(_))));
    }
}
