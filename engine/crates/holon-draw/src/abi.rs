//! The wasm boundary: plain `extern "C"` scalars and two kinds of buffer, no binding
//! generator. A host writes a leaf's layout buffers into input buffers it asks for, calls
//! [`hd_build_leaf`], and reads the vertex stream in place from the output buffer.
//!
//! Single-threaded by construction (one static state): a host that builds in a worker
//! instantiates the module there.
use crate::twin::{build_leaf, Leaf, Style};
use std::cell::RefCell;

#[derive(Default)]
struct State {
    input: [Vec<f64>; 5], // roads, buildings, meta, streets, tones
    out: Vec<f32>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

/// Size input buffer `which` (0 roads, 1 buildings, 2 meta, 3 streets, 4 tones) to `len`
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

/// Build the leaf from the input buffers. Returns the number of floats in the output, or 0
/// if the meta buffer holds fewer than 3 numbers or a road or building record is cut short.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn hd_build_leaf(id: f64, ox: f64, oy: f64, sx: f64, pph: f64, road_r: f64, road_g: f64, road_b: f64, has_tones: u32) -> u32 {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let st = &mut *s;
        let [roads, buildings, meta, streets, tones] = &st.input;
        if meta.len() < 3 || roads.len() % 5 != 0 || buildings.len() % 8 != 0 {
            st.out.clear();
            return 0;
        }
        let style = Style { pph, road: [road_r, road_g, road_b], ..Style::default() };
        let leaf = Leaf { id, roads, buildings, meta, streets, tones: if has_tones != 0 { Some(tones) } else { None }, ox, oy, sx };
        st.out = build_leaf(&leaf, &style);
        st.out.len() as u32
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
        hd_in(1, 7); // a building record cut short
        assert_eq!(hd_build_leaf(42.0, 1.0, 2.0, 1.0, 2.5, 0.47, 0.49, 0.53, 0), 0);
    }
}
