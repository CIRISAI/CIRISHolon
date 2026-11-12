//! PRESENCEGEN-1's carrier: generate days read from stdin with tables read from a file.
//!
//!   presencegen_draw <tables.txt> <chain|frame>
//!
//! stdin: `seed employed day_type` per line. stdout per line: 24 states, kind, changes, fallback.
//! Then `DIGEST` and a number folding 1,000 fixed calls.
use holon_scene::devices::Tables;
use holon_scene::presence::{day_with, Generator};
use holon_scene::seed::mix;
use std::io::{BufRead, Write};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let text = std::fs::read_to_string(&a[1]).unwrap();
    let g = if a[2] == "chain" { Generator::Chain } else { Generator::Frame };
    let mut owned: Vec<(u32, Vec<u32>)> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let mut it = l.split_whitespace().map(|x| x.parse::<u32>().unwrap());
            (it.next().unwrap(), it.collect())
        })
        .collect();
    owned.sort_by_key(|r| r.0);
    let rows: Vec<(u32, &[u32])> = owned.iter().map(|r| (r.0, r.1.as_slice())).collect();
    let t = Tables { rows: &rows };
    let out = std::io::stdout();
    let mut out = std::io::BufWriter::new(out.lock());
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        let v: Vec<u64> = line.split_whitespace().map(|x| x.parse().unwrap()).collect();
        if v.len() < 3 {
            continue;
        }
        let d = day_with(&t, g, v[0], v[1] == 1, v[2] as u8, 0, false);
        let s: Vec<String> = d.states.iter().map(|x| x.to_string()).collect();
        writeln!(out, "{} {} {} {}", s.join(" "), d.kind, d.place_changes, d.fallback as u8).unwrap();
    }
    let mut dg = 0u64;
    for i in 0..1000u64 {
        let d = day_with(&t, g, mix(i), i % 2 == 0, (i % 4) as u8, mix(i / 5), i % 3 == 0);
        for x in d.states.iter() {
            dg = mix(dg ^ *x as u64);
        }
        dg = mix(dg ^ d.kind as u64 ^ ((d.place_changes as u64) << 8));
    }
    writeln!(out, "DIGEST {}", dg).unwrap();
}
