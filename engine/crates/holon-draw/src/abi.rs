//! The wasm boundary: plain `extern "C"` scalars and two kinds of buffer, no binding
//! generator. A host writes a leaf's layout buffers into input buffers it asks for, calls
//! [`hd_build_leaf`], and reads the vertex stream in place from the output buffer.
//!
//! Single-threaded by construction (one static state): a host that builds in a worker
//! instantiates the module there.
//!
//! MEMORY CAN MOVE: `hd_in` and `hd_build_leaf` may grow the module's memory, which detaches
//! every view a host holds on the old buffer. Take `memory.buffer` AFTER the call that returns
//! the pointer, never before (the twin found this on its first integration).
use crate::twin::{build_leaf, build_leaf_instances, Leaf, Style, GROUPS};
use std::cell::RefCell;

#[derive(Default)]
struct State {
    input: [Vec<f64>; 6], // roads, buildings, meta, streets, tones, style
    out: Vec<f32>,
    style: Option<Style>,
    /// The last leaf as packed instance records, in group order, and each group's
    /// (first record, count).
    inst: Vec<u8>,
    groups: [(u32, u32); 8],
    /// The last leaf's records, unpacked, with where each began in the vertex stream.
    recs: Vec<crate::twin::Rec>,
    /// The kind code each record's primitive carries in the vertex stream.
    codes: Vec<f32>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

/// Size input buffer `which` (0 roads, 1 buildings, 2 meta, 3 streets, 4 tones, 5 the
/// style as one flat array, then [`hd_style`]) to `len`
/// doubles and return where to write them. A `which` out of range returns null.
#[no_mangle]
pub extern "C" fn hd_in(which: u32, len: u32) -> *mut f64 {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        match s.input.get_mut(which as usize) {
            Some(b) => {
                b.clear();
                b.resize(len as usize, 0.0);
                b.as_mut_ptr()
            }
            None => std::ptr::null_mut(),
        }
    })
}

/// Adopt the style written to input buffer 5 (in `twin::STYLE_FIELDS` order). Returns 1,
/// or 0 if it is not `twin::style_len()` long, in which case the twin's defaults at its
/// `bb58ce2` stay in force. A leaf's road colour and persons per household are still the
/// arguments of [`hd_build_leaf`].
#[no_mangle]
pub extern "C" fn hd_style() -> u32 {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let st = crate::twin::Style::from_flat(&s.input[5]);
        let ok = st.is_some() as u32;
        if st.is_some() {
            s.style = st;
        }
        ok
    })
}

/// The length of the flat style array.
#[no_mangle]
pub extern "C" fn hd_style_len() -> u32 {
    crate::twin::style_len() as u32
}

/// Build the leaf from the input buffers. Returns the number of floats in the output, or 0
/// if the meta buffer holds fewer than 3 numbers or a road or building record is cut short.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn hd_build_leaf(id: f64, ox: f64, oy: f64, sx: f64, pph: f64, road_r: f64, road_g: f64, road_b: f64, has_tones: u32) -> u32 {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let st = &mut *s;
        let [roads, buildings, meta, streets, tones, _] = &st.input;
        if meta.len() < 3 || roads.len() % 5 != 0 || buildings.len() % 8 != 0 {
            st.out.clear();
            return 0;
        }
        let style = Style { pph, road: [road_r, road_g, road_b], ..st.style.clone().unwrap_or_default() };
        let leaf = Leaf { id, roads, buildings, meta, streets, tones: if has_tones != 0 { Some(tones) } else { None }, ox, oy, sx };
        st.out = build_leaf(&leaf, &style);
        st.out.len() as u32
    })
}

/// Build the leaf as INSTANCE records for `draw.js` (same arguments as [`hd_build_leaf`]).
/// Returns the number of 52-byte records, or 0 on the same refusals. The records are in the
/// twin's paint order by group (`twin::GROUPS`: ground, mottling and scattered trees, yards,
/// garden trees, casings, streets, shadows, roofs), each group one kind; positions are in the
/// tile's frame (the leaf's offset and east-west stretch applied).
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn hd_build_leaf_instances(id: f64, ox: f64, oy: f64, sx: f64, pph: f64, road_r: f64, road_g: f64, road_b: f64, has_tones: u32) -> u32 {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let st = &mut *s;
        let [roads, buildings, meta, streets, tones, _] = &st.input;
        st.inst.clear();
        st.groups = [(0, 0); 8];
        if meta.len() < 3 || roads.len() % 5 != 0 || buildings.len() % 8 != 0 {
            return 0;
        }
        let style = Style { pph, road: [road_r, road_g, road_b], ..st.style.clone().unwrap_or_default() };
        let leaf = Leaf { id, roads, buildings, meta, streets, tones: if has_tones != 0 { Some(tones) } else { None }, ox, oy, sx };
        let recs = build_leaf_instances(&leaf, &style);
        // the vertex stream's kind code for each record (for hd_inst_expand), from the stream itself
        let v = build_leaf(&leaf, &style);
        st.codes = recs.iter().map(|r| v.get(r.start + 9).copied().unwrap_or(0.0)).collect();
        for (i, r) in recs.iter().enumerate() {
            let g = &mut st.groups[r.group];
            if g.1 == 0 {
                g.0 = i as u32;
            }
            g.1 += 1;
            r.inst.pack(&mut st.inst);
        }
        st.recs = recs;
        recs_len(st)
    })
}

fn recs_len(st: &State) -> u32 {
    st.recs.len() as u32
}

/// For the twin's real-leaf witness: expand the last leaf's records back into the twin's
/// 10-float vertex stream (`twin::expand`), in the VERTEX stream's order (each record put back
/// where its primitive began), readable at `hd_out()`. Returns the float count; a host
/// compares it with `hd_build_leaf`'s stream over the same inputs, with the tolerances stated
/// in `twin::tests::the_instance_records_are_the_vertex_streams_geometry`: kind codes exact;
/// positions to f32 rounding plus |sx - 1| times the mark's size; normals and widths 1e-3;
/// colours half an 8-bit step.
#[no_mangle]
pub extern "C" fn hd_inst_expand() -> u32 {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let st = &mut *s;
        let mut order: Vec<usize> = (0..st.recs.len()).collect();
        order.sort_by_key(|&i| st.recs[i].start);
        let mut out = Vec::new();
        for i in order {
            out.extend(crate::twin::expand(&st.recs[i], st.codes[i]));
        }
        st.out = out;
        st.out.len() as u32
    })
}

/// Where the last leaf's records are (52 bytes each). Valid until the next call.
#[no_mangle]
pub extern "C" fn hd_out_instances() -> *const u8 {
    STATE.with(|s| s.borrow().inst.as_ptr())
}

/// The number of paint groups (8).
#[no_mangle]
pub extern "C" fn hd_inst_groups() -> u32 {
    GROUPS.len() as u32
}

/// Group `g`'s kind (holon-draw's `Kind` number), first record, or record count (`which` 0,
/// 1 or 2); 0 for a group out of range.
#[no_mangle]
pub extern "C" fn hd_inst_group(g: u32, which: u32) -> u32 {
    let Some(&(_, kind)) = GROUPS.get(g as usize) else { return 0 };
    STATE.with(|s| {
        let (first, count) = s.borrow().groups[g as usize];
        match which {
            0 => kind as u32,
            1 => first,
            _ => count,
        }
    })
}

/// Where the last built leaf's floats are. Valid until the next call into this module.
#[no_mangle]
pub extern "C" fn hd_out() -> *const f32 {
    STATE.with(|s| s.borrow().out.as_ptr())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_boundary_gives_what_the_function_gives_and_refuses_a_cut_record() {
        let (roads, buildings, meta, streets) = ([0.0, 0.0, 40.0, 0.0, 12.0], [10.0, 8.0, 8.0, 6.0, 0.3, 2.0, 1.0, 0.0], [60.0, 50.0, 4.0], [1.0, 0.0, 1.0]);
        for (i, b) in [&roads[..], &buildings[..], &meta[..], &streets[..]].iter().enumerate() {
            let p = hd_in(i as u32, b.len() as u32);
            unsafe { std::slice::from_raw_parts_mut(p, b.len()).copy_from_slice(b) };
        }
        let n = hd_build_leaf(42.0, 1.0, 2.0, 1.0, 2.5, 0.47, 0.49, 0.53, 0);
        let got = unsafe { std::slice::from_raw_parts(hd_out(), n as usize) }.to_vec();
        let style = Style::default();
        let want = build_leaf(&Leaf { id: 42.0, roads: &roads, buildings: &buildings, meta: &meta, streets: &streets, tones: None, ox: 1.0, oy: 2.0, sx: 1.0 }, &style);
        assert_eq!(got, want);
        assert!(hd_in(9, 4).is_null());
        // the instance path over the same inputs: records in group order, groups adding up
        let n = hd_build_leaf_instances(42.0, 1.0, 2.0, 1.0, 2.5, 0.47, 0.49, 0.53, 0);
        assert!(n > 0);
        let total: u32 = (0..hd_inst_groups()).map(|g| hd_inst_group(g, 2)).sum();
        assert_eq!(total, n);
        assert_eq!(hd_inst_group(0, 0), crate::Kind::Patch as u32);
        assert_eq!(hd_inst_group(5, 0), crate::Kind::Line as u32);
        let bytes = unsafe { std::slice::from_raw_parts(hd_out_instances(), n as usize * crate::INSTANCE_BYTES) };
        assert_eq!(bytes[48], crate::Kind::Patch as u8, "the first record is the ground");
        // the expansion has the stream's length, its kind codes and, here (sx = 1), its positions
        let m = hd_inst_expand();
        let e = unsafe { std::slice::from_raw_parts(hd_out(), m as usize) }.to_vec();
        let n2 = hd_build_leaf(42.0, 1.0, 2.0, 1.0, 2.5, 0.47, 0.49, 0.53, 0);
        let v = unsafe { std::slice::from_raw_parts(hd_out(), n2 as usize) }.to_vec();
        assert_eq!(e.len(), v.len());
        for (a, b) in e.chunks(10).zip(v.chunks(10)) {
            assert_eq!(a[9], b[9]);
            assert!((a[0] - b[0]).abs() < 2e-3 && (a[1] - b[1]).abs() < 2e-3, "{a:?} {b:?}");
        }
        hd_in(1, 7); // a building record cut short
        assert_eq!(hd_build_leaf(42.0, 1.0, 2.0, 1.0, 2.5, 0.47, 0.49, 0.53, 0), 0);
    }
}
