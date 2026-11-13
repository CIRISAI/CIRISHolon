//! BUILDGEN-2's carrier: generate dwellings read from stdin with keyed tables read from a file.
//!
//!   buildgen_draw2 <tables.txt>
//!
//! tables.txt: one row per line, `key own division v1 v2 ...`. stdin: `seed state urban` per line.
//! stdout per line: kind vintage fuel wall roof storeys split area kwh nonelec total heating cooling
//! own_fuel division_fuel matrix_of_division; or `REFUSED` where the function returns none.
//! Then `DIGEST` and a number folding 1,000 fixed calls.
use holon_scene::keyed::{generate_keyed, Tables2};
use holon_scene::seed::mix;
use std::io::{BufRead, Write};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let text = std::fs::read_to_string(&a[1]).unwrap();
    let mut owned: Vec<(u64, Vec<u32>)> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let mut it = l.split_whitespace();
            let k: u64 = it.next().unwrap().parse().unwrap();
            (k, it.map(|x| x.parse::<i64>().unwrap() as u32).collect())
        })
        .collect();
    owned.sort_by_key(|r| r.0);
    let rows: Vec<(u64, &[u32])> = owned.iter().map(|r| (r.0, r.1.as_slice())).collect();
    let t = Tables2 { rows: &rows };
    let out = std::io::stdout();
    let mut out = std::io::BufWriter::new(out.lock());
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        let v: Vec<u64> = line.split_whitespace().map(|x| x.parse().unwrap()).collect();
        if v.len() < 3 {
            continue;
        }
        match generate_keyed(&t, v[0], v[1] as u8, v[2] as u8) {
            None => writeln!(out, "REFUSED").unwrap(),
            Some(k) => {
                let d = &k.dwelling;
                writeln!(out, "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}", d.kind, d.vintage, d.fuel, d.wall, d.roof, d.storeys, d.split_level, d.area_sqft, d.kwh, d.nonelec_kbtu, d.total_kbtu, d.heating_kbtu, d.cooling_kbtu, k.own[2], k.division[2], k.matrix_of_division as u8).unwrap()
            }
        }
    }
    let mut dg = 0u64;
    for i in 0..1000u64 {
        let s = [1u8, 6, 12, 36, 48, 56, 11, 15][(i % 8) as usize];
        if let Some(k) = generate_keyed(&t, mix(i), s, (i % 3) as u8) {
            let d = &k.dwelling;
            for x in [d.kind as u64, d.vintage as u64, d.fuel as u64, d.wall as u64, d.roof as u64, d.storeys as u64, d.area_sqft as u64, d.kwh as u64, d.total_kbtu as u64] {
                dg = mix(dg ^ x);
            }
        }
    }
    writeln!(out, "DIGEST {}", dg).unwrap();
}
