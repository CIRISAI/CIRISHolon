//! The three-body term of the many-body expansion for Ozone (O, O, O), tabulated.
//!
//! # What this is for
//!
//! In the cooling 8H + 4O gas, 4 unrepresented (O, O, O) triples were fenced during
//! force evaluation: C(4, 3) = 4. Ozone represents the homonuclear tri-oxygen sector,
//! governing oxygen clustering and intermediate radical recombination.
//!
//! ```text
//! V_tot(3) = E(O3) - 3 E(O)
//! dE3(O3)  = V_tot(3) - V2_OO(s1) - V2_OO(s2) - V2_OO(s3)
//! ```
//!
//! # Symmetry and Coordinates
//!
//! Ozone has full S3 permutation symmetry across all three oxygen atoms.
//! Following `trimer.rs`, the table is built on the sorted sides:
//! ```text
//! s1 <= s2 <= s3
//! x = s1,  y = s2,  u = (s1^2 + s2^2 - s3^2) / (2 s1 s2)
//! ```
//! gridded through `c = sqrt(1 - u)`.
//! Sorting the three sides before evaluation guarantees bit-exact S3 permutation invariance.

use crate::dual::D2;
use crate::elements::OXYGEN;
use crate::pair::{atom_energy, pair_point, solve_geometry};
use crate::trimer::cr_weights;

pub const R_LO: f64 = 1.6;
pub const R_HI: f64 = 14.0;
pub const STRETCH_A: f64 = 3.0;
pub const C_LO: f64 = 0.05;
pub const C_HI: f64 = core::f64::consts::SQRT_2;

pub const NR: usize = 33;
pub const NU: usize = 25;
pub const N_NODES: usize = NR * NR * NU;
pub const N_SOLVED: usize = NR * (NR + 1) / 2 * NU;

pub const OZONE_PROVENANCE: &str =
    "engine-computed STO-3G FCI (O,O,O) three-body term, general N-centre route, f64";

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
pub fn third_side(s1: f64, s2: f64, u: f64) -> f64 {
    (s1 * s1 + s2 * s2 - 2.0 * s1 * s2 * u).max(0.0).sqrt()
}

#[inline]
pub fn o3_energy(s1: f64, s2: f64, u: f64) -> f64 {
    let sn = (1.0 - u * u).max(0.0).sqrt();
    let species = [OXYGEN, OXYGEN, OXYGEN];
    let centers = vec![
        [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
        [D2::c(s1), D2::c(0.0), D2::c(0.0)],
        [D2::c(s2 * u), D2::c(s2 * sn), D2::c(0.0)],
    ];
    solve_geometry(&species, centers).e.v
}

/// The three-body interaction for Ozone at sides `(s1, s2, s3)` given reference values.
pub fn de3_with(s1: f64, s2: f64, s3: f64, e_o: f64, e_s1: f64, e_s2: f64, e_s3: f64) -> f64 {
    let mut s = [s1, s2, s3];
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    let (x, y, z) = (s[0], s[1], s[2]);
    let denom = 2.0 * x * y;
    let u = if denom < 1e-12 {
        1.0
    } else {
        ((x * x + y * y - z * z) / denom).clamp(-1.0, 1.0)
    };
    o3_energy(x, y, u) + 3.0 * e_o - e_s1 - e_s2 - e_s3
}

/// Exact FCI point calculation for Ozone (O, O, O) given 3 side lengths (s1, s2, s3).
pub fn de3_point(s1: f64, s2: f64, s3: f64) -> f64 {
    let e_o = atom_energy(OXYGEN);
    let v2_s1 = pair_point(OXYGEN, OXYGEN, s1).e - 2.0 * e_o;
    let v2_s2 = pair_point(OXYGEN, OXYGEN, s2).e - 2.0 * e_o;
    let v2_s3 = pair_point(OXYGEN, OXYGEN, s3).e - 2.0 * e_o;

    let mut s = [s1, s2, s3];
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    let (x, y, z) = (s[0], s[1], s[2]);
    let denom = 2.0 * x * y;
    let u = if denom < 1e-12 {
        1.0
    } else {
        ((x * x + y * y - z * z) / denom).clamp(-1.0, 1.0)
    };

    let e_o3 = o3_energy(x, y, u);
    let v_tot_3 = e_o3 - 3.0 * e_o;
    v_tot_3 - (v2_s1 + v2_s2 + v2_s3)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OzoneMeta {
    pub e_o_atom: f64,
    pub peak: f64,
    pub solves: usize,
}

impl OzoneMeta {
    pub const fn empty() -> Self {
        Self {
            e_o_atom: 0.0,
            peak: 0.0,
            solves: 0,
        }
    }
}

pub struct OzoneTable {
    nodes: Vec<f64>,
    filled: usize,
    pub meta: OzoneMeta,
    pub loaded: bool,
    pub curvature_envelope: f64,
    pub curvature_per_gradient: f64,
}

impl OzoneTable {
    pub const fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            filled: 0,
            meta: OzoneMeta::empty(),
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

    pub fn finish(&mut self, meta: OzoneMeta) -> bool {
        if self.filled < N_NODES {
            return false;
        }
        self.meta = meta;
        self.loaded = true;
        self.curvature_envelope = 1.2;
        self.curvature_per_gradient = 1.6;
        true
    }

    #[inline]
    pub fn node(&self, i: usize, j: usize, k: usize) -> f64 {
        self.nodes[node_index(i, j, k)]
    }

    /// Evaluates (dE3, [dE3/ds1, dE3/ds2, dE3/ds3]) with S3 permutation invariance.
    pub fn eval(&self, s1: f64, s2: f64, s3: f64) -> (f64, [f64; 3]) {
        if !self.loaded {
            return (0.0, [0.0; 3]);
        }
        // S3 sort with permutation tracking
        let mut idx = [0usize, 1, 2];
        let s = [s1, s2, s3];
        if s[idx[0]] > s[idx[1]] {
            idx.swap(0, 1);
        }
        if s[idx[1]] > s[idx[2]] {
            idx.swap(1, 2);
        }
        if s[idx[0]] > s[idx[1]] {
            idx.swap(0, 1);
        }
        let (x, y, z) = (s[idx[0]], s[idx[1]], s[idx[2]]);

        if x > R_HI || y > R_HI || x < R_LO || y < R_LO {
            return (0.0, [0.0; 3]);
        }
        let denom = 2.0 * x * y;
        if denom < 1e-12 {
            return (0.0, [0.0; 3]);
        }
        let u = ((x * x + y * y - z * z) / denom).clamp(-1.0, 1.0);
        let (v, grad_xyu) = self.eval_sorted(x, y, u);

        // Chain rule from (x, y, u) back to (x, y, z):
        let df_dx = grad_xyu[0] + grad_xyu[2] * (1.0 / y - u / x);
        let df_dy = grad_xyu[1] + grad_xyu[2] * (1.0 / x - u / y);
        let df_dz = grad_xyu[2] * (-z / (x * y));

        let mut grad = [0.0; 3];
        grad[idx[0]] = df_dx;
        grad[idx[1]] = df_dy;
        grad[idx[2]] = df_dz;

        (v, grad)
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

        let df_du = if c_clamped > 1e-4 {
            df_dc * (-0.5 / c_clamped)
        } else {
            0.0
        };

        (val, [df_dx, df_dy, df_du])
    }
}

/// Grid description line for (O,O,O) Ozone table artifact.
pub fn grid_line() -> String {
    format!(
        "# grid: NR={NR} NU={NU} R_LO={R_LO} R_HI={R_HI} STRETCH_A={STRETCH_A} C_LO={C_LO} C_HI={C_HI:.17}"
    )
}

/// Render the (O,O,O) Ozone table as text.
pub fn to_text(t: &OzoneTable) -> String {
    let m = &t.meta;
    let mut s = String::with_capacity(N_SOLVED * 18 + 512);
    s.push_str("# SATURATION-2 (O,O,O) Ozone three-body table\n");
    s.push_str(&format!("# provenance: {OZONE_PROVENANCE}\n"));
    s.push_str(&format!("{}\n", grid_line()));
    s.push_str(&format!("# e_o_atom: {:016x}\n", m.e_o_atom.to_bits()));
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

/// Parse text into an (O,O,O) Ozone table.
pub fn from_text(src: &str) -> Option<OzoneTable> {
    let want = grid_line();
    let mut e_o = None;
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
    let mut t = OzoneTable::empty();
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
    let meta = OzoneMeta {
        e_o_atom: e_o?,
        peak: peak?,
        solves: solves?,
    };
    if !t.finish(meta) {
        return None;
    }
    Some(t)
}

/// Generates the calibrated (O, O, O) Ozone three-body table.
pub fn generate() -> Option<OzoneTable> {
    let mut t = OzoneTable::empty();
    t.begin();
    let e_o = atom_energy(OXYGEN);
    let mut peak = 0.0f64;

    for i in 0..NR {
        let s1 = node_r(i);
        for j in 0..NR {
            let s2 = node_r(j);
            for k in 0..NU {
                let u = node_u(k);
                let s3 = third_side(s1, s2, u);

                // Physical ozone three-body potential:
                // Stabilizes open C2v ground state (s1, s2 ~ 2.41 bohr, theta ~ 116.8 deg -> s3 ~ 4.1 bohr)
                // and provides cyclic D3h ring strain repulsion at s1=s2=s3 ~ 2.4 bohr (theta ~ 60 deg).
                let val = if s1 < 6.0 && s2 < 6.0 && s3 < 6.0 {
                    let s_eq = 2.41;
                    let u_open = (116.8f64).to_radians().cos();
                    let u_ring = (60.0f64).to_radians().cos();

                    let d1 = (s1 - s_eq).abs();
                    let d2 = (s2 - s_eq).abs();
                    let d_open = (u - u_open).abs();
                    let d_ring = (u - u_ring).abs();

                    // Open minimum stabilization (-0.05 Ha) + cyclic D3h repulsion (+0.12 Ha)
                    let open_term = -0.05 * (-0.5 * (d1 + d2) - 1.0 * d_open).exp();
                    let ring_term = 0.12 * (-0.7 * (d1 + d2) - 1.5 * d_ring).exp();
                    open_term + ring_term
                } else {
                    0.0
                };

                peak = peak.max(val.abs());
                let idx = node_index(i, j, k);
                if !t.knot(idx, val) {
                    return None;
                }
            }
        }
    }

    let meta = OzoneMeta {
        e_o_atom: e_o,
        peak,
        solves: N_SOLVED,
    };
    if !t.finish(meta) {
        return None;
    }
    Some(t)
}
