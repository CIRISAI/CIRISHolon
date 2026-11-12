//! HOUSEGEN-1's carrier: draw households for units read from stdin.
//!
//!   housegen_draw <shares,comma-separated> <sizes,comma-separated> [truncating]
//!
//! stdin: one unit per line, `seed people`. stdout: one line per unit, `households ones`,
//! then `HIST` and the pooled count of households at each size 0..=20, then `DIGEST` and
//! the crate's 1,000-call digest for this row.
use holon_scene::household::{digest, draw, draw_truncating};
use std::io::{BufRead, Write};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let shares: Vec<u64> = a[1].split(',').map(|x| x.parse().unwrap()).collect();
    let sizes: Vec<u8> = a[2].split(',').map(|x| x.parse().unwrap()).collect();
    let plant = a.get(3).map(|s| s == "truncating").unwrap_or(false);
    let mut hist = [0u64; 21];
    let out = std::io::stdout();
    let mut out = std::io::BufWriter::new(out.lock());
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut it = line.split_whitespace();
        let (Some(s), Some(p)) = (it.next(), it.next()) else { continue };
        let (seed, people): (u64, u64) = (s.parse().unwrap(), p.parse().unwrap());
        let v = if plant { draw_truncating(seed, people, &shares, &sizes) } else { draw(seed, people, &shares, &sizes) };
        let ones = v.iter().filter(|&&x| x == 1).count();
        for &x in &v {
            hist[(x as usize).min(20)] += 1;
        }
        writeln!(out, "{} {}", v.len(), ones).unwrap();
    }
    writeln!(out, "HIST {}", hist.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" ")).unwrap();
    writeln!(out, "DIGEST {}", digest(&shares, &sizes)).unwrap();
}
