//! The water-table door (FSD-W3 WB-10.7): the committed (O, H, H) artifact pushed through
//! the byte buffer is the native parser's table to the bit, and bytes that are not the
//! table are refused by the parser rather than read as numbers.
//!
//! One test function on purpose: the doors act on the crate's one global `Sim`, and two
//! tests in this binary would race for it.

use holon_chem::water::{self, NR, NU};
use holon_render::{
    holon_water_loaded, holon_water_node, holon_water_nodes, holon_water_peak,
    holon_water_table_alloc, holon_water_table_load,
};

fn committed() -> String {
    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../holon-chem/tests/data/s2/s2_water_table.txt");
    std::fs::read_to_string(&p).expect("the committed (O,H,H) table")
}

/// The host's side of the door: reserve, write, load.
fn push(bytes: &[u8]) -> u32 {
    let ptr = holon_water_table_alloc(bytes.len() as u32);
    // SAFETY: the engine reserved exactly `bytes.len()` bytes at `ptr` and nothing else
    // touches the reservation between the reserve and the load.
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len()) };
    holon_water_table_load()
}

#[test]
fn the_door_is_the_native_parser_to_the_bit_and_refuses_what_is_not_the_table() {
    let text = committed();
    let native = water::from_text(&text).expect("the committed table parses natively");
    assert_eq!(holon_water_loaded(), 0, "a fresh engine carries no water table");

    // --- refusals first, each leaving nothing loaded -------------------------------
    // nothing reserved
    assert_eq!(holon_water_table_load(), 0, "an empty reservation is not a table");
    // a short artifact: the node count is the parser's, not the host's
    let cut = &text.as_bytes()[..text.len() / 2];
    assert_eq!(push(cut), 0, "half the table must be refused");
    // a byte that is not UTF-8
    let mut bad = text.clone().into_bytes();
    bad[text.len() / 3] = 0xff;
    assert_eq!(push(&bad), 0, "a non-UTF-8 byte must be refused by name");
    // a foreign grid line: the axes are this build's or the file is not read
    let grid = water::grid_line();
    assert!(text.contains(&grid), "the committed table carries this build's grid line");
    let foreign = text.replacen(&grid, &grid.replace("NR=", "NR=1"), 1);
    assert_eq!(push(foreign.as_bytes()), 0, "a foreign grid rule must be refused");
    assert_eq!(holon_water_loaded(), 0, "four refusals loaded nothing");

    // --- the load ------------------------------------------------------------------
    assert_eq!(push(text.as_bytes()), 1, "the committed table loads through the door");
    assert_eq!(holon_water_loaded(), 1);
    assert_eq!(holon_water_nodes() as usize, water::N_NODES);
    assert_eq!(
        holon_water_peak().to_bits(),
        native.meta.peak.to_bits(),
        "the peak through the door is the native peak to the bit"
    );
    // every node, every index order the door accepts, against the native table
    let mut compared = 0usize;
    for i in 0..NR {
        for j in 0..NR {
            for k in 0..NU {
                let d = holon_water_node(i as u32, j as u32, k as u32);
                let n = native.node(i, j, k);
                assert_eq!(d.to_bits(), n.to_bits(), "node ({i},{j},{k}) differs through the door");
                compared += 1;
            }
        }
    }
    assert_eq!(compared, NR * NR * NU);
    // off the grid is a sentinel, not a read past the end
    assert_eq!(holon_water_node(NR as u32, 0, 0), 0.0);
    assert_eq!(holon_water_node(0, 0, NU as u32), 0.0);

    // --- a planted node: the door carries the bytes it was given ---------------------
    // Flip the last hex digit of the first value line. The parser reads it as a different
    // number, and the door must serve THAT number — a door that cached, re-derived or
    // symmetrised from a stale copy would still serve the committed value.
    let mut lines: Vec<String> = text.lines().map(String::from).collect();
    let first_value = lines
        .iter()
        .position(|l| !l.starts_with('#') && !l.is_empty())
        .expect("a value line");
    let planted_bits = u64::from_str_radix(&lines[first_value], 16).unwrap() ^ 1;
    lines[first_value] = format!("{planted_bits:016x}");
    let planted = lines.join("\n") + "\n";
    let planted_native = water::from_text(&planted).expect("the planted table still parses");
    assert_eq!(push(planted.as_bytes()), 1);
    let mut moved = 0usize;
    for i in 0..NR {
        for j in 0..NR {
            for k in 0..NU {
                let d = holon_water_node(i as u32, j as u32, k as u32);
                assert_eq!(d.to_bits(), planted_native.node(i, j, k).to_bits());
                if d.to_bits() != native.node(i, j, k).to_bits() {
                    moved += 1;
                }
            }
        }
    }
    assert!(moved >= 1, "the planted bit reached no node: the door is not serving what it was given");

    // the committed table again, so the engine ends where a page would leave it
    assert_eq!(push(text.as_bytes()), 1);
    assert_eq!(holon_water_node(0, 0, 0).to_bits(), native.node(0, 0, 0).to_bits());
}
