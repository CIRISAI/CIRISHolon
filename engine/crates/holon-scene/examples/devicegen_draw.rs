//! DEVICEGEN-1's carrier: generate households read from stdin with tables read from a file.
//!
//!   devicegen_draw <tables.txt>
//!
//! tables.txt: one row per line, `key v1 v2 ...`. stdin: `seed persons building` per line.
//! stdout per line: adults seniors children phones(3) counts(8) connection tv_hours weekdays_home
//! then `DIGEST` and a number folding 1,000 fixed calls.
use holon_scene::devices::{generate_with, Tables};
use holon_scene::seed::mix;
use std::io::{BufRead, Write};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let text = std::fs::read_to_string(&a[1]).unwrap();
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
        let h = generate_with(&t, v[0], v[1] as u32, v[2] as u8);
        let c = h.counts;
        writeln!(out, "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}", h.adults, h.seniors, h.children, h.phones[0], h.phones[1], h.phones[2],
            c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7], h.connection, h.tv_hours, h.weekdays_someone_home).unwrap();
    }
    let mut d = 0u64;
    for i in 0..1000u64 {
        let h = generate_with(&t, mix(i), (i % 9) as u32, (i % 3) as u8);
        d = mix(d ^ h.persons() as u64);
        for x in h.counts.iter().chain(h.phones.iter()) {
            d = mix(d ^ *x as u64);
        }
        d = mix(d ^ h.connection as u64);
    }
    writeln!(out, "DIGEST {}", d).unwrap();
}
