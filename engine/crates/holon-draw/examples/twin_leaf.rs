//! Build leaves read from stdin and print each vertex stream as hexadecimal 32-bit words.
//!
//! One leaf per input line, as 9 space-separated groups, numbers within a group separated by
//! commas (an empty group is `-`): id; ox,oy,sx; road colour; roads; buildings; meta; streets;
//! tones; persons per household. One output line per leaf. Used by `tools/twin_leaf_check.mjs`.
use holon_draw::twin::{build_leaf, Leaf, Style};
use std::io::{BufRead, Write};

fn nums(s: &str) -> Vec<f64> {
    if s == "-" {
        return Vec::new();
    }
    s.split(',').map(|x| x.parse().unwrap()).collect()
}

fn main() {
    let out = std::io::stdout();
    let mut out = std::io::BufWriter::new(out.lock());
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        let g: Vec<&str> = line.split_whitespace().collect();
        if g.len() < 9 {
            continue;
        }
        let (place, road, roads, buildings, meta, streets, tones) = (nums(g[1]), nums(g[2]), nums(g[3]), nums(g[4]), nums(g[5]), nums(g[6]), nums(g[7]));
        let style = Style { pph: g[8].parse().unwrap(), road: [road[0], road[1], road[2]], ..Style::default() };
        let leaf = Leaf {
            id: g[0].parse().unwrap(),
            roads: &roads,
            buildings: &buildings,
            meta: &meta,
            streets: &streets,
            tones: if g[7] == "-" { None } else { Some(&tones) },
            ox: place[0],
            oy: place[1],
            sx: place[2],
        };
        let v = build_leaf(&leaf, &style);
        let mut s = String::with_capacity(v.len() * 8);
        for x in v {
            s.push_str(&format!("{:08x}", x.to_bits()));
        }
        writeln!(out, "{s}").unwrap();
    }
}
