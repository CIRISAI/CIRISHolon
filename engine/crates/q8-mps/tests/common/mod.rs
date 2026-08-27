//! Shared gate infrastructure for G0-1 (`tests/g0_sector_shift.rs`) and G1a/G1b
//! (`tests/g1_mpo_correctness.rs`) — `Q8_MPS_PREREG.md` §3.
//!
//! `mod common;` is compiled once PER integration-test binary, so a function only one of the two
//! binaries calls is "unused" from the other's point of view — not a real dead-code question,
//! the standard shape of a shared `tests/common/mod.rs`.
#![allow(dead_code)]
//!
//! `independent_dense_h` is the G1a cross-check: a brute-force second-quantization build over
//! Fock-basis bitmasks, sharing NO code with `q8_mps::mpo`'s tensor contraction — the whole
//! point of G1a is two independently-derived constructions agreeing. It is written directly
//! from `c†_p c_q = sigma+_p . (Z-string over strictly-between sites) . sigma-_q` (`p<q`), the
//! same derivation `mpo.rs`'s header states, but as flat bitmask arithmetic rather than tensor
//! contraction — a genuinely different code shape, not a copy.

/// `c†_a c_b` acting on `mask` (bit `q` = occupation of JW site `q`, 0-indexed). `None` if the
/// term annihilates. General JW sign: `(-1)^(popcount of mask strictly between a and b)`.
pub fn jw_hop(mask: u32, a: usize, b: usize) -> Option<(u32, f64)> {
    if mask & (1 << b) == 0 {
        return None;
    }
    let new_mask = mask & !(1u32 << b);
    if new_mask & (1 << a) != 0 {
        return None;
    }
    let lo = a.min(b) + 1;
    let hi = a.max(b);
    let sign = if hi > lo {
        let between = (mask >> lo) & ((1u32 << (hi - lo)) - 1);
        if between.count_ones().is_multiple_of(2) {
            1.0
        } else {
            -1.0
        }
    } else {
        1.0
    };
    Some((new_mask | (1 << a), sign))
}

/// Dense `H'` (or bare `H` at `mu=0.0`) over the FULL `2^(2*sites)`-dim Fock space, built
/// directly from `jw_hop` — independent of `q8_mps::mpo`. Gate-only, `sites<=4`.
pub fn independent_dense_h(sites: usize, t: f64, u: f64, mu: f64) -> Vec<f64> {
    let l = 2 * sites;
    let dim = 1usize << l;
    let mut h = vec![0.0; dim * dim];

    for mask in 0..dim as u32 {
        let mut diag = -mu * mask.count_ones() as f64;
        for s in 0..sites {
            let up = (mask >> (2 * s)) & 1;
            let dn = (mask >> (2 * s + 1)) & 1;
            diag += u * (up * dn) as f64;
        }
        h[mask as usize * dim + mask as usize] += diag;

        for j in 0..l.saturating_sub(2) {
            let (a, b) = (j, j + 2);
            if let Some((new_mask, sign)) = jw_hop(mask, a, b) {
                h[new_mask as usize * dim + mask as usize] += -t * sign;
            }
            if let Some((new_mask, sign)) = jw_hop(mask, b, a) {
                h[new_mask as usize * dim + mask as usize] += -t * sign;
            }
        }
    }
    h
}

/// `(<N_up>, <N_down>)` of a real eigenvector over the full `2^(2*sites)` Fock basis — the
/// "measured, not assumed" sector check both G0-1 and G1b rely on.
pub fn sector_of(v: &[f64], sites: usize) -> (f64, f64) {
    let mut nup = 0.0;
    let mut ndn = 0.0;
    for (mask, &amp) in v.iter().enumerate() {
        let w = amp * amp;
        if w == 0.0 {
            continue;
        }
        let m = mask as u32;
        for s in 0..sites {
            if (m >> (2 * s)) & 1 == 1 {
                nup += w;
            }
            if (m >> (2 * s + 1)) & 1 == 1 {
                ndn += w;
            }
        }
    }
    (nup, ndn)
}

/// Brute-force observable profile from a normalized dense eigenvector over the full
/// `2^(2*sites)` Fock basis — the ground truth `tests/observables_check.rs` gates the MPS
/// contraction (`q8_mps::observables`) against at small N, before it is trusted at N=8-12.
pub struct DenseObservables {
    pub occupation: Vec<f64>,
    pub magnetization: Vec<f64>,
    pub double_occ: Vec<f64>,
    pub n_tot: f64,
    pub sz: f64,
    pub sz_sq: f64,
}

pub fn dense_observables(v: &[f64], sites: usize) -> DenseObservables {
    let mut occupation = vec![0.0; sites];
    let mut magnetization = vec![0.0; sites];
    let mut double_occ = vec![0.0; sites];
    let mut n_tot = 0.0;
    let mut sz = 0.0;
    let mut sz_sq = 0.0;

    for (mask, &amp) in v.iter().enumerate() {
        let w = amp * amp;
        if w == 0.0 {
            continue;
        }
        let m = mask as u32;
        let (mut nup, mut ndn) = (0i32, 0i32);
        for cs in 0..sites {
            let up = (m >> (2 * cs)) & 1;
            let dn = (m >> (2 * cs + 1)) & 1;
            occupation[cs] += w * (up + dn) as f64;
            magnetization[cs] += w * (up as i32 - dn as i32) as f64;
            double_occ[cs] += w * (up * dn) as f64;
            nup += up as i32;
            ndn += dn as i32;
        }
        n_tot += w * (nup + ndn) as f64;
        let s = (nup - ndn) as f64;
        sz += w * s;
        sz_sq += w * s * s;
    }

    DenseObservables { occupation, magnetization, double_occ, n_tot, sz, sz_sq }
}

pub fn max_abs_diff(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max)
}

/// The validation grid's `U/t` values, `Q8_MPS_PREREG.md` §1.
pub const U_GRID: [f64; 4] = [0.0, 1.0, 4.0, 16.0];

/// A cached q-seam exact reference — declared deviation from `Q8_MPS_PREREG.md` §9's "called
/// live for every exact comparison" (research-manager verification, 2026-08-24): q-seam's
/// pinned Lanczos policy is deterministic, so re-deriving the SAME (N,U,t) reference across
/// repeated re-runs during iterative fixing is pure waste against the speed directive. Gated:
/// every run live-validates ONE configuration against its cache entry and panics on mismatch —
/// a cache that goes stale is a finding, not something to silently trust.
pub struct CachedExact {
    pub energy: f64,
    pub s_squared: f64,
    pub density: Vec<f64>,
    pub magnetization: Vec<f64>,
    pub double_occ: Vec<f64>,
}

fn cache_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../output/q8_mps/exact_cache.txt")
}

fn rotation_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../output/q8_mps/live_validation_rotation.txt")
}

/// The forced-live spot-check DEFECT fix (research-manager-2, A3.2): the naive "always the
/// first config" pins the check to the cheapest entry forever, so N=10/U=16 are never
/// re-validated once cached. Persists a counter across runs and returns the grid index (0-based,
/// enumeration order) to force live THIS run; every call also advances the counter for next
/// time, so repeated runs sweep the whole grid rather than the same one point.
pub fn next_rotation_index(total: usize) -> usize {
    let total = total.max(1);
    let path = rotation_path();
    let prev: usize =
        std::fs::read_to_string(&path).ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
    let this_run = prev % total;
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, ((prev + 1) % total).to_string());
    this_run
}

/// `key(sites, u)` -> cached entry, loaded once. Missing file or missing key is `None`, never
/// an error — caller falls back to a live call either way.
pub fn load_exact_cache() -> std::collections::HashMap<(usize, u64), CachedExact> {
    let mut map = std::collections::HashMap::new();
    let Ok(text) = std::fs::read_to_string(cache_path()) else {
        return map;
    };
    for line in text.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() < 4 {
            continue;
        }
        let sites: usize = f[0].parse().unwrap();
        let u: f64 = f[1].parse().unwrap();
        let energy: f64 = f[2].parse().unwrap();
        let s_squared: f64 = f[3].parse().unwrap();
        let rest: Vec<f64> = f[4..].iter().map(|x| x.parse().unwrap()).collect();
        assert_eq!(rest.len(), 3 * sites, "cache line malformed for N={sites} U={u}");
        let density = rest[0..sites].to_vec();
        let magnetization = rest[sites..2 * sites].to_vec();
        let double_occ = rest[2 * sites..3 * sites].to_vec();
        map.insert((sites, u.to_bits()), CachedExact { energy, s_squared, density, magnetization, double_occ });
    }
    map
}

pub fn append_exact_cache(sites: usize, u: f64, e: &CachedExact) {
    use std::io::Write;
    let path = cache_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut line = format!("{sites} {u} {:e} {:e}", e.energy, e.s_squared);
    for v in e.density.iter().chain(e.magnetization.iter()).chain(e.double_occ.iter()) {
        line.push(' ');
        line.push_str(&format!("{v:e}"));
    }
    line.push('\n');
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
    }
}
