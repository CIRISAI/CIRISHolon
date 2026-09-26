//! The walks on disk, read by the engine — REPLACE-1's readers (`conformance/replace1/`).
//!
//! Three formats, each documented by the Python reader that wrote the banked numbers, and
//! each read here without Python so that the fluid-element tier's admission runs from the
//! engine's own code path:
//!
//! * **`rigid.walk` / `rigid.vwalk`** — header `# walk N L` (`L` in bohr), then one row per
//!   readout of `3n` numbers: unwrapped OXYGEN positions (bohr) / velocities (au). A running
//!   arm's last row may be partial; only complete rows common to both files are used
//!   (`slow1_search.py`'s `load`). [`RigidWalk`].
//! * **`atoms.walk` / `atoms.vwalk`** — header `# awalk N L`, a `# z …` line of atomic numbers
//!   (O, H, H per molecule in ascending index order), rows of `3·atoms` numbers; the velocity
//!   file carries `# dt_fs …` on its second line (`molsearch.py`'s `load`). [`AtomWalk`].
//!
//! On top of them, the dictionaries the banked reads were taken on:
//!
//! * SLOW-1's per-molecule structure — Errington–Debenedetti `q` over the four nearest
//!   oxygens, the counts within 3.3 / 5.0 Å and the O–O "bond" count within 3.5 Å
//!   ([`structure`]), and Piaggi–Parrinello's local pair entropy `s2` ([`pair_entropy_s2`]);
//! * ORDER-1's first-harmonic modes — the 24 density and current columns
//!   ([`fourier_features`]), a per-molecule field coarse-grained at the same `k`
//!   ([`coarse`]), and the per-wavevector chains with their quarter-wave rotations
//!   ([`chains`]);
//! * HBOND-SEARCH-1's per-molecule donated/accepted counts and centre-of-mass velocity
//!   ([`hbond_features`]).
//!
//! Every constant is the Python reader's, including its Bohr: SLOW-1 and ORDER-1 convert with
//! `0.529177210903`, HBOND's criterion with `0.529177` — two readers, two constants, both
//! kept so a banked number is reproduced rather than nudged.

use std::f64::consts::PI;
use std::fs;
use std::path::Path;

/// `slow1_search.py`'s Bohr (Å).
pub const BOHR_A_SLOW1: f64 = 0.529177210903;
/// `hbond_search.py`'s and `r1_closure_test.py`'s Bohr (Å).
pub const BOHR_A_SHORT: f64 = 0.529177;

fn read_rows(path: &Path) -> Result<(Vec<String>, Vec<Vec<f64>>), String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut headers = Vec::new();
    let mut rows = Vec::new();
    for line in text.lines() {
        if line.starts_with('#') {
            headers.push(line.to_string());
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        let r: Result<Vec<f64>, _> = line.split_whitespace().map(|x| x.parse::<f64>()).collect();
        match r {
            Ok(v) => rows.push(v),
            Err(_) => break, // a partial last row being written
        }
    }
    Ok((headers, rows))
}

/// A rigid arm's oxygen walk: `pos[(t·n + i)·3 + a]` in bohr (unwrapped), `vel` in au.
#[derive(Clone, Debug)]
pub struct RigidWalk {
    pub frames: usize,
    pub n: usize,
    pub box_bohr: f64,
    pub pos: Vec<f64>,
    pub vel: Vec<f64>,
    /// `scout.json`'s `readout_fs`, when the directory has one
    pub readout_fs: Option<f64>,
    /// `scout.json`'s `production_temperature_mean_k`, when present
    pub temperature_k: Option<f64>,
}

fn json_number(text: &str, key: &str) -> Option<f64> {
    let k = format!("\"{key}\"");
    let at = text.find(&k)? + k.len();
    let rest = text[at..].trim_start().strip_prefix(':')?.trim_start();
    let end = rest.find(|c: char| !(c.is_ascii_digit() || "+-.eE".contains(c))).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

impl RigidWalk {
    /// Read `dir/rigid.walk` and `dir/rigid.vwalk`; `max_frames` truncates.
    pub fn load(dir: &Path, max_frames: Option<usize>) -> Result<RigidWalk, String> {
        let (hp, pr) = read_rows(&dir.join("rigid.walk"))?;
        let (_, vr) = read_rows(&dir.join("rigid.vwalk"))?;
        let head: Vec<&str> = hp.first().ok_or("rigid.walk: no header")?.split_whitespace().collect();
        let box_bohr: f64 = head.get(3).ok_or("rigid.walk: header has no box")?.parse().map_err(|e| format!("{e}"))?;
        let n3 = pr.first().ok_or("rigid.walk: no rows")?.len();
        let mut k = pr.len().min(vr.len());
        while k > 0 && (pr[k - 1].len() != n3 || vr[k - 1].len() != n3) {
            k -= 1;
        }
        if let Some(m) = max_frames {
            k = k.min(m);
        }
        let mut pos = Vec::with_capacity(k * n3);
        let mut vel = Vec::with_capacity(k * n3);
        for t in 0..k {
            pos.extend_from_slice(&pr[t]);
            vel.extend_from_slice(&vr[t]);
        }
        let scout = fs::read_to_string(dir.join("scout.json")).unwrap_or_default();
        Ok(RigidWalk {
            frames: k,
            n: n3 / 3,
            box_bohr,
            pos,
            vel,
            readout_fs: json_number(&scout, "readout_fs"),
            temperature_k: json_number(&scout, "production_temperature_mean_k"),
        })
    }

    #[inline]
    pub fn p(&self, t: usize, i: usize, a: usize) -> f64 {
        self.pos[(t * self.n + i) * 3 + a]
    }

    #[inline]
    pub fn v(&self, t: usize, i: usize, a: usize) -> f64 {
        self.vel[(t * self.n + i) * 3 + a]
    }
}

/// Python's `%` for a positive modulus (the result has the modulus's sign).
#[inline]
pub fn pymod(x: f64, l: f64) -> f64 {
    let r = x % l;
    if r < 0.0 {
        r + l
    } else {
        r
    }
}

/// The minimum image of `d` on a periodic box of edge `l`, with numpy's round-half-even.
#[inline]
pub fn min_image(d: f64, l: f64) -> f64 {
    d - l * (d / l).round_ties_even()
}

/// A walk's frames in Å, wrapped into the box (SLOW-1's `P % L` after `load`'s conversion).
pub fn wrapped_angstrom(w: &RigidWalk, t: usize) -> Vec<[f64; 3]> {
    let l = w.box_bohr * BOHR_A_SLOW1;
    (0..w.n)
        .map(|i| {
            [
                pymod(w.p(t, i, 0) * BOHR_A_SLOW1, l),
                pymod(w.p(t, i, 1) * BOHR_A_SLOW1, l),
                pymod(w.p(t, i, 2) * BOHR_A_SLOW1, l),
            ]
        })
        .collect()
}

/// Per-molecule structure of one frame (Å, wrapped): `[q, n33, n50, nb]` per molecule —
/// `slow1_search.py`'s `structure`, row for row.
pub fn structure(x: &[[f64; 3]], l: f64) -> Vec<[f64; 4]> {
    let n = x.len();
    let mut out = vec![[0.0; 4]; n];
    let mut r = vec![0.0f64; n];
    let mut d = vec![[0.0f64; 3]; n];
    for i in 0..n {
        for j in 0..n {
            for a in 0..3 {
                d[j][a] = min_image(x[j][a] - x[i][a], l);
            }
            r[j] = if i == j { f64::INFINITY } else { (d[j][0] * d[j][0] + d[j][1] * d[j][1] + d[j][2] * d[j][2]).sqrt() };
        }
        // the four nearest (the set, not the order, enters q)
        let mut nn: Vec<usize> = Vec::with_capacity(5);
        for j in 0..n {
            if nn.len() < 4 {
                nn.push(j);
                nn.sort_by(|&a, &b| r[a].partial_cmp(&r[b]).unwrap());
            } else if r[j] < r[nn[3]] {
                nn[3] = j;
                nn.sort_by(|&a, &b| r[a].partial_cmp(&r[b]).unwrap());
            }
        }
        let u: Vec<[f64; 3]> = nn
            .iter()
            .map(|&j| {
                let m = r[j];
                [d[j][0] / m, d[j][1] / m, d[j][2] / m]
            })
            .collect();
        let mut s = 0.0;
        for a in 0..4 {
            for b in a + 1..4 {
                let c = u[a][0] * u[b][0] + u[a][1] * u[b][1] + u[a][2] * u[b][2];
                s += (c + 1.0 / 3.0) * (c + 1.0 / 3.0);
            }
        }
        out[i][0] = 1.0 - 3.0 / 8.0 * s;
        out[i][1] = r.iter().filter(|&&v| v < 3.3).count() as f64;
        out[i][2] = r.iter().filter(|&&v| v < 5.0).count() as f64;
        out[i][3] = r.iter().filter(|&&v| v < 3.5).count() as f64;
    }
    out
}

/// SLOW-1 Amendment 1's local pair entropy `s2` per molecule of one frame (units of `k_B`):
/// the per-molecule `g_i(r)` histogrammed at `0.03` Å, smoothed with a unit-area Gaussian of
/// width `0.15` Å (41 taps), and `s2 = −2πρ ∫₀^{7.5} (g ln g − g + 1) r² dr`
/// (`slow1_search.py`'s `_g_frames` + `pair_entropy`, the unweighted column).
pub fn pair_entropy_s2(x: &[[f64; 3]], l: f64) -> Vec<f64> {
    const RM: f64 = 7.5;
    const SIG: f64 = 0.15;
    const DR: f64 = 0.03;
    let n = x.len();
    let rho = n as f64 / (l * l * l);
    // edges = np.arange(0, RM + 4 SIG + DR, DR): 271 edges, 270 bins (checked against numpy)
    let n_edges = ((RM + 4.0 * SIG + DR) / DR).ceil() as usize;
    let edges: Vec<f64> = (0..n_edges).map(|i| i as f64 * DR).collect();
    let nb = n_edges - 1;
    let rc: Vec<f64> = (0..nb).map(|b| 0.5 * (edges[b + 1] + edges[b])).collect();
    let m = (4.0 * SIG / DR) as i64; // int(4 SIG / DR) = 20
    let ker: Vec<f64> = (-m..=m).map(|q| (-(q as f64 * DR).powi(2) / (2.0 * SIG * SIG)).exp()).collect();
    let ks: f64 = ker.iter().sum::<f64>() * DR;
    let ker: Vec<f64> = ker.iter().map(|k| k / ks).collect();
    let rmax = edges[nb];
    let keep: Vec<usize> = (0..nb).filter(|&b| rc[b] <= RM).collect();
    let mut out = vec![0.0; n];
    let mut hs = vec![0.0f64; nb];
    for i in 0..n {
        hs.iter_mut().for_each(|v| *v = 0.0);
        for j in 0..n {
            if i == j {
                continue;
            }
            let dx = min_image(x[j][0] - x[i][0], l);
            let dy = min_image(x[j][1] - x[i][1], l);
            let dz = min_image(x[j][2] - x[i][2], l);
            let r = (dx * dx + dy * dy + dz * dz).sqrt();
            if r < rmax {
                let b = (r / DR) as i64;
                for (q, kv) in ker.iter().enumerate() {
                    let t = b + q as i64 - m;
                    if t >= 0 && (t as usize) < nb {
                        hs[t as usize] += kv;
                    }
                }
            }
        }
        let mut s = 0.0;
        for &b in &keep {
            let g = hs[b] / (4.0 * PI * rho * rc[b] * rc[b]);
            let gl = if g > 0.0 { g * g.ln() } else { 0.0 };
            s += (gl - g + 1.0) * rc[b] * rc[b];
        }
        out[i] = -2.0 * PI * rho * s * DR;
    }
    out
}

/// ORDER-1's hydrodynamic sector: per frame, the first harmonic along each axis,
/// `[ρ cos, ρ sin, jx cos, jx sin, jy cos, jy sin, jz cos, jz sin]` per axis (24 columns),
/// positions in Å, `k = 2π/L` (`view_search.py`'s `fourier_features`).
pub fn fourier_features(w: &RigidWalk) -> Vec<[f64; 24]> {
    let l = w.box_bohr * BOHR_A_SLOW1;
    let k = 2.0 * PI / l;
    (0..w.frames)
        .map(|t| {
            let mut row = [0.0f64; 24];
            for ax in 0..3 {
                for i in 0..w.n {
                    let ph = k * pymod(w.p(t, i, ax) * BOHR_A_SLOW1, l);
                    let (s, c) = ph.sin_cos();
                    row[ax * 8] += c;
                    row[ax * 8 + 1] += s;
                    for comp in 0..3 {
                        let v = w.v(t, i, comp);
                        row[ax * 8 + 2 + 2 * comp] += v * c;
                        row[ax * 8 + 3 + 2 * comp] += v * s;
                    }
                }
            }
            row
        })
        .collect()
}

/// `Σ_i w_i cos/sin(2π x_{i,a} / L)` per axis: `[x cos, x sin, y cos, y sin, z cos, z sin]`
/// for a per-molecule field `w[t][i]` (ORDER-1's `coarse`).
pub fn coarse(w: &RigidWalk, field: &[Vec<f64>]) -> Vec<[f64; 6]> {
    let l = w.box_bohr * BOHR_A_SLOW1;
    let k = 2.0 * PI / l;
    (0..w.frames)
        .map(|t| {
            let mut row = [0.0f64; 6];
            for a in 0..3 {
                for (i, &fi) in field[t].iter().enumerate().take(w.n) {
                    let ph = k * pymod(w.p(t, i, a) * BOHR_A_SLOW1, l);
                    let (s, c) = ph.sin_cos();
                    row[2 * a] += fi * c;
                    row[2 * a + 1] += fi * s;
                }
            }
            row
        })
        .collect()
}

/// Subtract each column's time mean in place (ORDER-1's per-seed centring).
pub fn centre<const C: usize>(x: &mut [[f64; C]]) {
    let n = x.len() as f64;
    for c in 0..C {
        let m = x.iter().map(|r| r[c]).sum::<f64>() / n;
        x.iter_mut().for_each(|r| r[c] -= m);
    }
}

/// The density columns of [`fourier_features`] (`FOURIER_DENSITY`).
pub const FOURIER_DENSITY: [usize; 6] = [0, 1, 8, 9, 16, 17];

/// ORDER-1's per-wavevector chains: for each axis `a`, the chain
/// `[ρ_c, ρ_s, jL_c, jL_s, jT1_c, jT1_s, jT2_c, jT2_s, X¹_c, X¹_s, X²_c, …]` and its quarter-
/// wave rotation (every `(c, s)` pair mapped to `(s, −c)`) — six chains, as rows of one
/// `Vec<f64>` each. `fields` are coarse fields at the same `k` ([`coarse`]).
pub fn chains(h: &[[f64; 24]], fields: &[&[[f64; 6]]]) -> Vec<(usize, Vec<f64>)> {
    let t_len = h.len();
    let width = 8 + 2 * fields.len();
    let mut out = Vec::with_capacity(6);
    for a in 0..3 {
        let b = 8 * a;
        let mut cols = vec![b, b + 1, b + 2 + 2 * a, b + 3 + 2 * a];
        for c in 0..3 {
            if c != a {
                cols.push(b + 2 + 2 * c);
                cols.push(b + 3 + 2 * c);
            }
        }
        let mut m = Vec::with_capacity(t_len * width);
        for t in 0..t_len {
            for &c in &cols {
                m.push(h[t][c]);
            }
            for f in fields {
                m.push(f[t][2 * a]);
                m.push(f[t][2 * a + 1]);
            }
        }
        let mut r = vec![0.0; t_len * width];
        for t in 0..t_len {
            for p in 0..width / 2 {
                r[t * width + 2 * p] = m[t * width + 2 * p + 1];
                r[t * width + 2 * p + 1] = -m[t * width + 2 * p];
            }
        }
        out.push((width, m));
        out.push((width, r));
    }
    out
}

/// An all-atom walk (`atoms.walk` / `atoms.vwalk`): positions bohr, velocities au.
#[derive(Clone, Debug)]
pub struct AtomWalk {
    pub frames: usize,
    pub atoms: usize,
    pub box_bohr: f64,
    pub z: Vec<u32>,
    pub dt_fs: f64,
    pub pos: Vec<f64>,
    pub vel: Vec<f64>,
}

impl AtomWalk {
    pub fn load(dir: &Path) -> Result<AtomWalk, String> {
        let (hp, pr) = read_rows(&dir.join("atoms.walk"))?;
        let (hv, vr) = read_rows(&dir.join("atoms.vwalk"))?;
        let box_bohr: f64 = hp[0].split_whitespace().nth(3).ok_or("atoms.walk: no box")?.parse().map_err(|e| format!("{e}"))?;
        let z: Vec<u32> = hp[1].split_whitespace().skip(2).map(|s| s.parse().unwrap()).collect();
        let dt_fs: f64 = hv[1].split_whitespace().nth(2).ok_or("atoms.vwalk: no dt")?.parse().map_err(|e| format!("{e}"))?;
        let k = pr.len().min(vr.len());
        let mut pos = Vec::new();
        let mut vel = Vec::new();
        for t in 0..k {
            pos.extend_from_slice(&pr[t]);
            vel.extend_from_slice(&vr[t]);
        }
        Ok(AtomWalk { frames: k, atoms: z.len(), box_bohr, z, dt_fs, pos, vel })
    }

    #[inline]
    fn p(&self, t: usize, i: usize) -> [f64; 3] {
        let o = (t * self.atoms + i) * 3;
        [self.pos[o], self.pos[o + 1], self.pos[o + 2]]
    }

    #[inline]
    fn v(&self, t: usize, i: usize) -> [f64; 3] {
        let o = (t * self.atoms + i) * 3;
        [self.vel[o], self.vel[o + 1], self.vel[o + 2]]
    }
}

/// Per molecule per frame `[donated, accepted, vcom_x, vcom_y, vcom_z]` — HBOND-SEARCH-1's
/// `BOND + VEL` columns: an H-bond is O–O `< 3.5` Å (with `hbond_search.py`'s Bohr) and the
/// angle between O→H and O→O `< 30°`; molecules are `(O, O+1, O+2)` for each oxygen.
/// Returns one `frames × 5` row-major block per molecule.
pub fn hbond_features(w: &AtomWalk) -> Vec<Vec<f64>> {
    const M_O: f64 = 15.9949146196 * 1822.888486;
    const M_H: f64 = 1.00782503223 * 1822.888486;
    let oxy: Vec<usize> = (0..w.atoms).filter(|&i| w.z[i] == 8).collect();
    let n = oxy.len();
    let l = w.box_bohr;
    let rc = 3.5 / BOHR_A_SHORT;
    let cos30 = (30.0f64).to_radians().cos();
    let mut out = vec![vec![0.0; w.frames * 5]; n];
    for t in 0..w.frames {
        let po: Vec<[f64; 3]> = oxy.iter().map(|&o| w.p(t, o)).collect();
        let mut don = vec![0.0; n];
        let mut acc = vec![0.0; n];
        for i in 0..n {
            for hoff in [1usize, 2] {
                let ph = w.p(t, oxy[i] + hoff);
                let mut v = [0.0; 3];
                for a in 0..3 {
                    v[a] = min_image(ph[a] - po[i][a], l);
                }
                let vn = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
                for j in 0..n {
                    if j == i {
                        continue;
                    }
                    // d[i,j] = O_i − O_j (minimum image); the O_i → O_j direction is −d
                    let mut d = [0.0; 3];
                    for a in 0..3 {
                        d[a] = min_image(po[i][a] - po[j][a], l);
                    }
                    let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                    if r >= rc {
                        continue;
                    }
                    let c = -(v[0] * d[0] + v[1] * d[1] + v[2] * d[2]) / (vn * r);
                    if c > cos30 {
                        don[i] += 1.0;
                        acc[j] += 1.0;
                    }
                }
            }
        }
        for i in 0..n {
            let o = oxy[i];
            let (vo, v1, v2) = (w.v(t, o), w.v(t, o + 1), w.v(t, o + 2));
            let mt = M_O + 2.0 * M_H;
            let row = &mut out[i][t * 5..t * 5 + 5];
            row[0] = don[i];
            row[1] = acc[i];
            for a in 0..3 {
                row[2 + a] = (M_O * vo[a] + M_H * v1[a] + M_H * v2[a]) / mt;
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------------------------
// The fluid-element tier's admission sample (REPLACE-1 §1): chains -> the gate's units
// ---------------------------------------------------------------------------------------------

use holon_closure::removable::{self, Mat, NumpyPcg64, Unit};

/// Chain columns, in [`order_chains`]' layout: the hydrodynamic block…
pub const C_RHO: [usize; 2] = [0, 1];
pub const C_JL: [usize; 2] = [2, 3];
/// `(ρ, j_L)` — ORDER-1's target and kept set (`C_LS`)
pub const C_LS: [usize; 4] = [0, 1, 2, 3];
/// …and the structural block, one coarse field per pair: `Q` (q), `S2` (s2), `N33`, `N50`
/// (density counts), `NB` (the O–O bond count).
pub const C_Q: [usize; 2] = [8, 9];
pub const C_STRUCT: [usize; 10] = [8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
pub const STRUCT_FIELDS: [&str; 5] = ["q", "s2", "n33", "n50", "nb"];

/// ORDER-1's features of one walk, centred per seed: the 24 hydrodynamic columns and the
/// structural block's five coarse fields at the same `k`.
#[derive(Clone, Debug)]
pub struct OrderFeatures {
    pub h: Vec<[f64; 24]>,
    /// `Q, S2, N33, N50, NB` in [`STRUCT_FIELDS`] order
    pub fields: Vec<Vec<[f64; 6]>>,
    pub dt_fs: f64,
    pub frames: usize,
    pub q_mean: f64,
}

/// Build [`OrderFeatures`]: per frame the SLOW-1 structure and `s2`, each per-molecule field
/// centred by its walk mean (`q − ⟨q⟩`, ORDER-1's `coarse(P, q − q.mean(), L)`), coarse-
/// grained, and every column centred in time.
pub fn order_features(w: &RigidWalk) -> OrderFeatures {
    let l = w.box_bohr * BOHR_A_SLOW1;
    // per field, per frame, per molecule: q, s2, n33, n50, nb
    let mut per: Vec<Vec<Vec<f64>>> = (0..5).map(|_| Vec::with_capacity(w.frames)).collect();
    for t in 0..w.frames {
        let x = wrapped_angstrom(w, t);
        let st = structure(&x, l);
        per[0].push(st.iter().map(|s| s[0]).collect());
        per[1].push(pair_entropy_s2(&x, l));
        per[2].push(st.iter().map(|s| s[1]).collect());
        per[3].push(st.iter().map(|s| s[2]).collect());
        per[4].push(st.iter().map(|s| s[3]).collect());
    }
    let mut q_mean = 0.0;
    let mut fields = Vec::with_capacity(5);
    for (f, field) in per.iter_mut().enumerate() {
        let mean = field.iter().flatten().sum::<f64>() / (w.frames * w.n) as f64;
        if f == 0 {
            q_mean = mean;
        }
        field.iter_mut().flatten().for_each(|v| *v -= mean);
        let mut c = coarse(w, field);
        centre(&mut c);
        fields.push(c);
    }
    let mut h = fourier_features(w);
    centre(&mut h);
    OrderFeatures { h, fields, dt_fs: w.readout_fs.unwrap_or(20.0), frames: w.frames, q_mean }
}

/// The six chains as gate matrices: `[ρ, jL, jT1, jT2 | Q, S2, N33, N50, NB]` (18 columns
/// when all five fields are passed).
pub fn order_chains(h: &[[f64; 24]], fields: &[Vec<[f64; 6]>]) -> Vec<Mat> {
    let fr: Vec<&[[f64; 6]]> = fields.iter().map(|f| f.as_slice()).collect();
    chains(h, &fr).into_iter().map(|(wd, data)| Mat { rows: data.len() / wd, cols: wd, data }).collect()
}

/// The gate's units from chains: `kept` and `target` column lists.
pub fn units(chains: &[Mat], kept: &[usize], target: &[usize]) -> Vec<Unit> {
    chains.iter().map(|c| Unit { kept: c.select_cols(kept), target: c.select_cols(target) }).collect()
}

/// A candidate's per-unit columns from chains.
pub fn cand(chains: &[Mat], cols: &[usize]) -> Vec<Mat> {
    chains.iter().map(|c| c.select_cols(cols)).collect()
}

fn col_sd(v: impl Iterator<Item = f64> + Clone) -> f64 {
    let n = v.clone().count() as f64;
    let m = v.clone().sum::<f64>() / n;
    (v.map(|x| (x - m) * (x - m)).sum::<f64>() / n).sqrt()
}

/// ORDER-1's PO-4, re-derived for the engine (REPLACE-1 PR-1): six independent OU fields `Z`
/// with `tau_rows` memory (one per axis and quadrature), `Z(t)` added to the density column
/// it is paired with at `amp ×` that column's SD, and the planted field `Q' = sd(Q)·(Z(t +
/// latency) + noise of `noise` of its variance)` — so the planted field leads the density by
/// the latency. Returns `(h', Q')`, both centred. The Gaussian draws are this engine's (Box–
/// Muller on numpy's PCG64 stream), not numpy's ziggurat: the plant is re-derived, not replayed
/// (M-PLANT-OBS).
pub fn plant_po4(
    h: &[[f64; 24]],
    q: &[[f64; 6]],
    tau_rows: f64,
    latency_rows: usize,
    noise: f64,
    amp: f64,
    seed: u64,
) -> (Vec<[f64; 24]>, Vec<[f64; 6]>) {
    let t_len = h.len();
    let mut rng = NumpyPcg64::new(seed);
    let z = removable::ou_field(t_len + latency_rows, 6, tau_rows, &mut rng);
    let mut hp = h.to_vec();
    for (j, &c) in FOURIER_DENSITY.iter().enumerate() {
        let sd = col_sd(h.iter().map(|r| r[c]));
        for (t, row) in hp.iter_mut().enumerate() {
            row[c] += amp * sd * z.at(t, j);
        }
    }
    let q_sd = col_sd(q.iter().flat_map(|r| r.iter().copied()));
    let mut qp: Vec<[f64; 6]> = (0..t_len)
        .map(|t| {
            let mut r = [0.0; 6];
            for (j, v) in r.iter_mut().enumerate() {
                *v = (z.at(t + latency_rows, j) + rng.normal() * noise.sqrt()) * q_sd;
            }
            r
        })
        .collect();
    centre(&mut hp);
    centre(&mut qp);
    (hp, qp)
}
