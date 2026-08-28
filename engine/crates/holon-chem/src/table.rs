//! Turning the model into the renderer's contract: a knot grid, the values on it, and
//! the four scalars the viewer needs.
//!
//! # Two entry points, one of them allocation-free
//!
//! [`stream_table`] computes the curve one knot at a time and hands each to a callback.
//! It allocates nothing, which is what lets `holon-render` call it inside the browser
//! and push straight into its fixed-size interpolator — no `Vec`, no allocator in the
//! wasm, no intermediate copy of the curve. [`generate_table`] is the native
//! convenience: same numbers, collected into a [`Table`] that can be written out as the
//! JSON contract for the file-fallback path.

use crate::h2::{asymptote, equilibrium, h2_point, h_atom_energy};

/// The scalars that describe the whole curve rather than one knot.
#[derive(Clone, Copy, Debug)]
pub struct Meta {
    pub n: usize,
    pub r_min: f64,
    pub r_max: f64,
    pub r_e: f64,
    pub d_e: f64,
    pub e_asymptote: f64,
    pub e_at_r_e: f64,
    pub e_h_atom: f64,
}

/// The knot grid: uniform in `u = R^{-1/4}`.
///
/// # Why that variable and not `R` or `log R`
///
/// This is a derivation, not a preference. The cubic Hermite error on an interval is
/// `|E''''| h^4 / 384`, and over most of the range `E''''` is dominated by the nuclear
/// repulsion's `d^4(1/R)/dR^4 = 24 R^{-5}`. Equidistributing `h^4 R^{-5}` needs `h`
/// proportional to `R^{5/4}`, which is exactly uniform spacing in `u = R^{-1/4}`. A
/// uniform-in-`R` grid would put the same spacing on the wall as on the tail and spend
/// its knots where the curve is a straight line; uniform-in-`log R` would still be a
/// factor `R^{1/4}` off.
///
/// The endpoints are pinned to `r_min` and `r_max` exactly rather than being left to
/// `powf` round-tripping through `u`, so the table's declared range is its actual range.
pub fn grid_point(r_min: f64, r_max: f64, n: usize, i: usize) -> f64 {
    if i == 0 {
        return r_min;
    }
    if i + 1 >= n {
        return r_max;
    }
    let u_hi = r_min.powf(-0.25);
    let u_lo = r_max.powf(-0.25);
    let u = u_hi + (u_lo - u_hi) * (i as f64) / ((n - 1) as f64);
    u.powf(-4.0)
}

/// Compute the curve knot by knot, handing each to `push` as
/// `(index, R, E, F, d2E/dR2)`. Returns `None` if `push` refuses a knot or the request
/// is not a usable grid.
///
/// `F` is the FORCE (`-dE/dR`), matching the renderer's contract; the sign is applied
/// here so no consumer has to know it.
pub fn stream_table<F>(r_min: f64, r_max: f64, n: usize, mut push: F) -> Option<Meta>
where
    F: FnMut(usize, f64, f64, f64, f64) -> bool,
{
    // The negations are deliberate, not stylistic: `!(a > b)` rejects NaN as well as an
    // out-of-order pair, where `a <= b` would silently accept NaN and hand it to `powf`.
    // `table.rs` in `holon-render` rejects a NaN grid the same way and for the same
    // reason; this is the earlier of the two fences.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if n < 2 || !(r_min > 0.0) || !(r_max > r_min) {
        return None;
    }
    for i in 0..n {
        let r = grid_point(r_min, r_max, n, i);
        let p = h2_point(r);
        if !push(i, r, p.e, p.f, p.e2) {
            return None;
        }
    }
    let (r_e, d_e, e_at_r_e) = equilibrium();
    Some(Meta {
        n,
        r_min,
        r_max,
        r_e,
        d_e,
        e_asymptote: asymptote(),
        e_at_r_e,
        e_h_atom: h_atom_energy(),
    })
}

/// The whole curve, collected.
#[derive(Clone, Debug)]
pub struct Table {
    pub r: Vec<f64>,
    pub e: Vec<f64>,
    pub f: Vec<f64>,
    pub e2: Vec<f64>,
    pub meta: Meta,
}

/// The label this crate puts on every curve it computes. It says the model, the
/// arithmetic and the route, because those three are what a reader would otherwise have
/// to take on trust — and it deliberately does NOT say "exact", which would be a claim
/// about the world rather than about the basis.
pub const PROVENANCE: &str = "engine-computed STO-3G FCI, f64";

pub fn generate_table(r_min: f64, r_max: f64, n: usize) -> Option<Table> {
    let mut r = Vec::with_capacity(n);
    let mut e = Vec::with_capacity(n);
    let mut f = Vec::with_capacity(n);
    let mut e2 = Vec::with_capacity(n);
    let meta = stream_table(r_min, r_max, n, |_, rr, ee, ff, cc| {
        r.push(rr);
        e.push(ee);
        f.push(ff);
        e2.push(cc);
        true
    })?;
    Some(Table { r, e, f, e2, meta })
}

impl Table {
    /// Serialise to the `h2_potential.json` contract.
    ///
    /// Numbers are written with `{:?}`, which is Rust's shortest representation that
    /// round-trips back to the same f64 — so re-reading this file gives bit-identical
    /// knots, and the file does not carry digits the computation did not have.
    pub fn to_json(&self) -> String {
        let mut s = String::with_capacity(64 * self.r.len() + 2048);
        s.push_str("{\n");
        s.push_str(&format!("  \"provenance\": \"{PROVENANCE}\",\n"));
        s.push_str(
            "  \"units\": \"Hartree atomic units: R in bohr, E in hartree, \
             F in hartree/bohr. F is the FORCE, so dE/dR = -F.\",\n",
        );
        s.push_str(&format!("  \"model\": \"{}\",\n", crate::sto3g::MODEL_NAME));
        s.push_str(
            "  \"note\": \"EXACT-IN-MODEL for STO-3G FCI, computed at load time from \
             closed-form Gaussian integrals in f64. R_e, D_e and the dissociation \
             asymptote are computed by the same code, none are quoted. NOT a prediction \
             of experiment.\",\n",
        );
        s.push_str(&format!("  \"R_e\": {:?},\n", self.meta.r_e));
        s.push_str(&format!("  \"D_e\": {:?},\n", self.meta.d_e));
        s.push_str(&format!("  \"E_asymptote\": {:?},\n", self.meta.e_asymptote));
        s.push_str(&format!("  \"E_at_R_e\": {:?},\n", self.meta.e_at_r_e));
        s.push_str(&format!("  \"E_H_atom\": {:?},\n", self.meta.e_h_atom));
        push_array(&mut s, "R_grid_bohr", &self.r);
        push_array(&mut s, "E_hartree", &self.e);
        push_array(&mut s, "F_hartree_per_bohr", &self.f);
        // The referee's spelling, not the renderer's original `d2E_hartree_per_bohr2`.
        // Emitting BOTH was the first fix and it was the wrong one: it would have left
        // `json.rs`'s inability to read this name undetected, and duplicated the column
        // in every file to hide it. `json.rs` accepts either spelling now, so one column
        // is enough and the round-trip test exercises the reader that needed fixing.
        push_array(&mut s, "E2_hartree_per_bohr2", &self.e2);
        s.push_str(&format!("  \"n_grid\": {}\n", self.r.len()));
        s.push_str("}\n");
        s
    }

    /// Worst absolute error of the piecewise cubic Hermite interpolant this table
    /// defines, measured against the model itself at `samples` interior points of every
    /// interval. Returns `(max |dE|, max |dF|)`.
    ///
    /// This is the number that says whether the grid is dense enough — the renderer
    /// integrates the INTERPOLANT, not the model, so the interpolant's departure from
    /// the model is the error the physics actually carries.
    pub fn hermite_error(&self, samples: usize) -> (f64, f64) {
        let mut worst_e = 0.0f64;
        let mut worst_f = 0.0f64;
        for i in 0..self.r.len() - 1 {
            let (r0, r1) = (self.r[i], self.r[i + 1]);
            let h = r1 - r0;
            let (y0, y1) = (self.e[i], self.e[i + 1]);
            // The stored column is the force; the interpolant is built on dE/dR.
            let (d0, d1) = (-self.f[i], -self.f[i + 1]);
            for k in 1..=samples {
                let t = k as f64 / (samples + 1) as f64;
                let t2 = t * t;
                let t3 = t2 * t;
                let value = (2.0 * t3 - 3.0 * t2 + 1.0) * y0
                    + (t3 - 2.0 * t2 + t) * h * d0
                    + (-2.0 * t3 + 3.0 * t2) * y1
                    + (t3 - t2) * h * d1;
                let slope = ((6.0 * t2 - 6.0 * t) * y0 + (-6.0 * t2 + 6.0 * t) * y1) / h
                    + (3.0 * t2 - 4.0 * t + 1.0) * d0
                    + (3.0 * t2 - 2.0 * t) * d1;
                let exact = h2_point(r0 + t * h);
                worst_e = worst_e.max((value - exact.e).abs());
                worst_f = worst_f.max((-slope - exact.f).abs());
            }
        }
        (worst_e, worst_f)
    }
}

fn push_array(s: &mut String, name: &str, v: &[f64]) {
    s.push_str("  \"");
    s.push_str(name);
    s.push_str("\": [");
    for (i, x) in v.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        if x.is_finite() {
            s.push_str(&format!("{x:?}"));
        } else {
            // Never emit `inf`/`NaN`: they are not JSON, and a parser that accepted them
            // would be accepting a curve nobody can integrate.
            s.push_str("null");
        }
    }
    s.push_str("],\n");
}
