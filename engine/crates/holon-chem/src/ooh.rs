//! The three-body term of the many-body expansion for hydroperoxyl (O, O, H), tabulated.
//!
//! # What this is for
//!
//! The 8H + 4O water quench simulation scored 0/8 in the frozen P2 protocol because
//! 52 triples were fenced without a table: 48 (O, O, H) + 4 (O, O, O).
//! (O, O, H) represents the missing reactive channels (OH + O <-> O2 + H) needed for
//! complete water synthesis.
//!
//! ```text
//! V_tot(3) = E(OOH) - 2 E(O) - E(H)
//! dE3(OOH) = V_tot(3) - V2_OO(r_OO) - V2_OH(r_OH1) - V2_OH(r_OH2)
//! ```
//!
//! # Coordinates and Exchange Symmetry
//!
//! Geometry is AB_2 where A is Hydrogen (Z=1) and B1, B2 are Oxygen (Z=8).
//! The coordinates are:
//! ```text
//! x = min(r_OH1, r_OH2),  y = max(r_OH1, r_OH2),  u = cos theta_OHO
//! ```
//! gridded through `c = sqrt(1 - u)`.
//! Permuting the two oxygens swaps r_OH1 <-> r_OH2 and leaves r_OO invariant, so
//! the surface has exact S2 exchange symmetry.

use crate::dual::D2;
use crate::elements::{HYDROGEN, OXYGEN};
use crate::pair::{atom_energy, pair_point, solve_geometry};
use crate::trimer::cr_weights;

pub const R_LO: f64 = 0.8;
pub const R_HI: f64 = 14.0;
pub const STRETCH_A: f64 = 3.0;
pub const C_LO: f64 = 0.05;
pub const C_HI: f64 = core::f64::consts::SQRT_2;
pub const U_FENCE: f64 = 1.0 - C_LO * C_LO;

pub const NR: usize = 33;
pub const NU: usize = 25;
pub const N_NODES: usize = NR * NR * NU;
pub const N_SOLVED: usize = NR * (NR + 1) / 2 * NU;

pub const OOH_PROVENANCE: &str =
    "engine-computed STO-3G FCI (O,O,H) three-body term, general N-centre route, f64";

#[inline]
pub const fn node_index(i: usize, j: usize, k: usize) -> usize {
    i * (NR * NU) + j * NU + k
}

#[inline]
pub fn tau_of_r(r: f64) -> f64 {
    let r_clamped = r.clamp(R_LO, R_HI);
    let frac = (r_clamped - R_LO) / (R_HI - R_LO);
    (1.0 + frac * (STRETCH_A.exp() - 1.0)).ln() / STRETCH_A
}

#[inline]
pub fn r_of_tau(tau: f64) -> f64 {
    let tau_clamped = tau.clamp(0.0, 1.0);
    R_LO + (R_HI - R_LO) * ((STRETCH_A * tau_clamped).exp() - 1.0) / (STRETCH_A.exp() - 1.0)
}

#[inline]
pub fn dtau_dr(r: f64) -> f64 {
    let r_clamped = r.clamp(R_LO, R_HI);
    let frac = (r_clamped - R_LO) / (R_HI - R_LO);
    let denom = (R_HI - R_LO) * (frac + 1.0 / (STRETCH_A.exp() - 1.0)) * STRETCH_A;
    if denom.abs() < 1e-15 {
        0.0
    } else {
        1.0 / denom
    }
}

#[inline]
pub fn node_r(i: usize) -> f64 {
    let tau = (i as f64) / ((NR - 1) as f64);
    r_of_tau(tau)
}

#[inline]
pub fn node_c(k: usize) -> f64 {
    C_LO + (C_HI - C_LO) * (k as f64) / ((NU - 1) as f64)
}

#[inline]
pub fn node_u(k: usize) -> f64 {
    let c = node_c(k);
    1.0 - c * c
}

#[inline]
pub fn oo_side(roh1: f64, roh2: f64, u: f64) -> f64 {
    (roh1 * roh1 + roh2 * roh2 - 2.0 * roh1 * roh2 * u).max(0.0).sqrt()
}

#[inline]
pub fn ooh_energy(x: f64, y: f64, u: f64) -> f64 {
    let sn = (1.0 - u * u).max(0.0).sqrt();
    let species = [HYDROGEN, OXYGEN, OXYGEN];
    let centers = vec![
        [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
        [D2::c(x), D2::c(0.0), D2::c(0.0)],
        [D2::c(y * u), D2::c(y * sn), D2::c(0.0)],
    ];
    solve_geometry(&species, centers).e.v
}

/// The three-body interaction at O-H sides `(x, y)` and angle cosine `u`, hartree, with
/// the reference energies supplied by the caller.
pub fn de3_with(x: f64, y: f64, u: f64, e_o: f64, e_h: f64, e_ox: f64, e_oy: f64) -> f64 {
    let z = oo_side(x, y, u);
    ooh_energy(x, y, u) + 2.0 * e_o + e_h
        - e_ox
        - e_oy
        - pair_point(OXYGEN, OXYGEN, z).e
}

/// The same, solving everything from scratch.
pub fn de3(x: f64, y: f64, u: f64) -> f64 {
    de3_with(
        x,
        y,
        u,
        atom_energy(OXYGEN),
        atom_energy(HYDROGEN),
        pair_point(OXYGEN, HYDROGEN, x).e,
        pair_point(OXYGEN, HYDROGEN, y).e,
    )
}

/// Exact FCI point calculation for (O, O, H) given 3 side lengths (roh1, roh2, roo).
pub fn de3_point(roh1: f64, roh2: f64, roo: f64) -> f64 {
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    let v2_oo = pair_point(OXYGEN, OXYGEN, roo).e - 2.0 * e_o;
    let v2_oh1 = pair_point(OXYGEN, HYDROGEN, roh1).e - e_o - e_h;
    let v2_oh2 = pair_point(OXYGEN, HYDROGEN, roh2).e - e_o - e_h;

    let u = ((roh1 * roh1 + roh2 * roh2 - roo * roo) / (2.0 * roh1 * roh2)).clamp(-1.0, 1.0);
    let s = (1.0 - u * u).max(0.0).sqrt();

    let species = [HYDROGEN, OXYGEN, OXYGEN];
    let centers = vec![
        [D2::c(0.0), D2::c(0.0), D2::c(0.0)],            // H at origin
        [D2::c(roh1), D2::c(0.0), D2::c(0.0)],           // O1 on x-axis
        [D2::c(roh2 * u), D2::c(roh2 * s), D2::c(0.0)],  // O2 in xy-plane
    ];

    let sol = solve_geometry(&species, centers);
    let e_ooh = sol.e.v;
    let v_tot_3 = e_ooh - 2.0 * e_o - e_h;
    v_tot_3 - (v2_oo + v2_oh1 + v2_oh2)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OohMeta {
    pub e_o_atom: f64,
    pub e_h_atom: f64,
    pub peak: f64,
    pub solves: usize,
}

impl OohMeta {
    pub const fn empty() -> Self {
        Self {
            e_o_atom: 0.0,
            e_h_atom: 0.0,
            peak: 0.0,
            solves: 0,
        }
    }
}

pub struct OohTable {
    nodes: Vec<f64>,
    filled: usize,
    pub meta: OohMeta,
    pub loaded: bool,
    pub curvature_envelope: f64,
    pub curvature_per_gradient: f64,
}

impl OohTable {
    pub const fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            filled: 0,
            meta: OohMeta::empty(),
            loaded: false,
            curvature_envelope: 0.0,
            curvature_per_gradient: 0.0,
        }
    }

    pub fn begin(&mut self) {
        self.nodes.clear();
        self.nodes.resize(N_NODES, 0.0);
        self.filled = 0;
        self.loaded = false;
        self.curvature_envelope = 0.0;
        self.curvature_per_gradient = 0.0;
    }

    pub fn knot(&mut self, index: usize, value: f64) -> bool {
        if index >= N_NODES || self.nodes.is_empty() {
            return false;
        }
        self.nodes[index] = value;
        self.filled += 1;
        true
    }

    pub fn finish(&mut self, meta: OohMeta) -> bool {
        if self.filled < N_NODES {
            return false;
        }
        self.meta = meta;
        self.loaded = true;
        self.curvature_envelope = 1.0;
        self.curvature_per_gradient = 1.5;
        true
    }

    #[inline]
    pub fn node(&self, i: usize, j: usize, k: usize) -> f64 {
        self.nodes[node_index(i, j, k)]
    }

    /// Evaluates (dE3, [dE3/droh1, dE3/droh2, dE3/droo]).
    pub fn eval(&self, roh1: f64, roh2: f64, roo: f64) -> (f64, [f64; 3]) {
        if !self.loaded {
            return (0.0, [0.0; 3]);
        }
        let swapped = roh1 > roh2;
        let (x, y) = if swapped { (roh2, roh1) } else { (roh1, roh2) };
        if x > R_HI || y > R_HI || x < R_LO || y < R_LO {
            return (0.0, [0.0; 3]);
        }
        let denom = 2.0 * x * y;
        if denom < 1e-12 {
            return (0.0, [0.0; 3]);
        }
        let u = ((x * x + y * y - roo * roo) / denom).clamp(-1.0, 1.0);
        let (v, grad_xyu) = self.eval_sorted(x, y, u);

        // Chain rule from (x, y, u) back to (x, y, roo):
        // u = (x^2 + y^2 - roo^2)/(2xy)
        // du/dx = 1/y - u/x
        // du/dy = 1/x - u/y
        // du/droo = -roo / (xy)
        let df_dx = grad_xyu[0] + grad_xyu[2] * (1.0 / y - u / x);
        let df_dy = grad_xyu[1] + grad_xyu[2] * (1.0 / x - u / y);
        let df_droo = grad_xyu[2] * (-roo / (x * y));

        let (g1, g2) = if swapped { (df_dy, df_dx) } else { (df_dx, df_dy) };
        (v, [g1, g2, df_droo])
    }

    fn eval_sorted(&self, x: f64, y: f64, u: f64) -> (f64, [f64; 3]) {
        let tau_x = tau_of_r(x);
        let tau_y = tau_of_r(y);
        let c = (1.0 - u).max(0.0).sqrt();
        let c_clamped = c.clamp(C_LO, C_HI);
        let norm_c = (c_clamped - C_LO) / (C_HI - C_LO);

        let (bx, wx, dwx) = cr_weights(NR, tau_x * (NR - 1) as f64);
        let (by, wy, dwy) = cr_weights(NR, tau_y * (NR - 1) as f64);
        let (bc, wc, dwc) = cr_weights(NU, norm_c * (NU - 1) as f64);

        let dtau_dx = dtau_dr(x) * (NR - 1) as f64;
        let dtau_dy = dtau_dr(y) * (NR - 1) as f64;
        let dnorm_dc = 1.0 / (C_HI - C_LO) * (NU - 1) as f64;

        let mut val = 0.0;
        let mut df_dtau_x = 0.0;
        let mut df_dtau_y = 0.0;
        let mut df_dnorm_c = 0.0;

        for (a, &wxa) in wx.iter().enumerate() {
            let dwxa = dwx[a];
            for (b, &wyb) in wy.iter().enumerate() {
                let dwyb = dwy[b];
                for (d, &wcd) in wc.iter().enumerate() {
                    let dwcd = dwc[d];
                    let node_val = self.node(bx + a, by + b, bc + d);
                    val += wxa * wyb * wcd * node_val;
                    df_dtau_x += dwxa * wyb * wcd * node_val;
                    df_dtau_y += wxa * dwyb * wcd * node_val;
                    df_dnorm_c += wxa * wyb * dwcd * node_val;
                }
            }
        }

        let df_dx = df_dtau_x * dtau_dx;
        let df_dy = df_dtau_y * dtau_dy;
        let df_dc = df_dnorm_c * dnorm_dc;

        // Convert df/dc to df/du: c = sqrt(1-u) => dc/du = -1 / (2c)
        let df_du = if c_clamped > 1e-4 {
            df_dc * (-0.5 / c_clamped)
        } else {
            0.0
        };

        (val, [df_dx, df_dy, df_du])
    }
}

/// Grid description line for (O,O,H) table artifact.
pub fn grid_line() -> String {
    format!(
        "# grid: NR={NR} NU={NU} R_LO={R_LO} R_HI={R_HI} STRETCH_A={STRETCH_A} C_LO={C_LO} C_HI={C_HI:.17}"
    )
}

/// Render the (O,O,H) table as text.
pub fn to_text(t: &OohTable) -> String {
    let m = &t.meta;
    let mut s = String::with_capacity(N_SOLVED * 18 + 512);
    s.push_str("# SATURATION-2 (O,O,H) three-body table\n");
    s.push_str(&format!("# provenance: {OOH_PROVENANCE}\n"));
    s.push_str(&format!("{}\n", grid_line()));
    s.push_str(&format!("# e_o_atom: {:016x}\n", m.e_o_atom.to_bits()));
    s.push_str(&format!("# e_h_atom: {:016x}\n", m.e_h_atom.to_bits()));
    s.push_str(&format!("# peak: {:016x}\n", m.peak.to_bits()));
    s.push_str(&format!("# solves: {}\n", m.solves));
    s.push_str(&format!("# nodes_stored: {N_SOLVED}\n"));
    for i in 0..NR {
        for j in i..NR {
            for k in 0..NU {
                s.push_str(&format!("{:016x}\n", t.node(i, j, k).to_bits()));
            }
        }
    }
    s
}

/// Parse text into an (O,O,H) table.
pub fn from_text(src: &str) -> Option<OohTable> {
    let want = grid_line();
    let mut e_o = None;
    let mut e_h = None;
    let mut peak = None;
    let mut solves = None;
    let mut grid_ok = false;
    let mut vals: Vec<f64> = Vec::with_capacity(N_SOLVED);
    for line in src.lines() {
        let line = line.trim_end();
        if let Some(rest) = line.strip_prefix('#') {
            let rest = rest.trim();
            if line == want {
                grid_ok = true;
            } else if let Some(h) = rest.strip_prefix("e_o_atom:") {
                e_o = u64::from_str_radix(h.trim(), 16).ok().map(f64::from_bits);
            } else if let Some(h) = rest.strip_prefix("e_h_atom:") {
                e_h = u64::from_str_radix(h.trim(), 16).ok().map(f64::from_bits);
            } else if let Some(h) = rest.strip_prefix("peak:") {
                peak = u64::from_str_radix(h.trim(), 16).ok().map(f64::from_bits);
            } else if let Some(h) = rest.strip_prefix("solves:") {
                solves = h.trim().parse::<usize>().ok();
            }
            continue;
        }
        if line.is_empty() {
            continue;
        }
        vals.push(f64::from_bits(u64::from_str_radix(line, 16).ok()?));
    }
    if !grid_ok || vals.len() != N_SOLVED {
        return None;
    }
    let mut t = OohTable::empty();
    t.begin();
    let mut at = 0usize;
    for i in 0..NR {
        for j in i..NR {
            for k in 0..NU {
                let v = vals[at];
                at += 1;
                if !t.knot(node_index(i, j, k), v) {
                    return None;
                }
                if i != j && !t.knot(node_index(j, i, k), v) {
                    return None;
                }
            }
        }
    }
    let meta = OohMeta {
        e_o_atom: e_o?,
        e_h_atom: e_h?,
        peak: peak?,
        solves: solves?,
    };
    if !t.finish(meta) {
        return None;
    }
    Some(t)
}
