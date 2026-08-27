//! Calibrates the `sweeps` determinism test: is that count a LIVE branch, or is it so
//! far from the convergence threshold that "the targets agree on it" would be true of
//! anything at all? Run: `cargo run --release --example sweep_sensitivity`.
//!
//! Answer, measured: **a one-ULP change to an input does NOT move the count** — 33
//! consecutive ULP perturbations give 8 sweeps at N=64 and 6 at N=11, every time. The
//! count IS input-dependent (7 for 11 of 200 random couplings at N=64, 8 for the other
//! 189), so the branch is live across inputs but blunt within one. That is why
//! `sweep_boundary_bits` exists: it bisects to the exact double where the count flips,
//! so the targets can be compared on a branch a single ULP really does control.
//!
//! Perturbs one entry of the coupling by k ULPs and records the resulting sweep count.
//! If the count never moves, agreeing on it is weak evidence. If it moves, the
//! `off_sq <= tol_sq` branch is genuinely reachable and cross-target agreement is real.
use ciris_sim_core::linalg::{jacobi_eigen, laplacian};

struct Rng(u64);
impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn unit2(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64) * (2.0 / 9_007_199_254_740_992.0)
    }
}

fn synth<const M: usize>(seed: u64) -> [[f64; M]; M] {
    let mut r = Rng(seed);
    let mut m = [[0.0f64; M]; M];
    for i in 0..M {
        for j in (i + 1)..M {
            let v = r.unit2();
            m[i][j] = v;
            m[j][i] = v;
        }
    }
    m
}

fn main() {
    const M: usize = 64;
    println!("A. one entry perturbed by k ULPs, N={M}, base seed 0x5EED1234ABCD0001");
    let base = synth::<M>(0x5EED_1234_ABCD_0001);
    let mut counts = [0usize; 33];
    for k in 0..33u64 {
        let mut c = base;
        let b = c[3][7].to_bits() + k;
        c[3][7] = f64::from_bits(b);
        c[7][3] = c[3][7];
        counts[k as usize] = jacobi_eigen(&laplacian(&c)).sweeps;
    }
    let lo = counts.iter().min().unwrap();
    let hi = counts.iter().max().unwrap();
    println!("   sweeps over k=0..32: min {lo}, max {hi}, distinct {}",
        { let mut v = counts.to_vec(); v.sort(); v.dedup(); v.len() });
    println!("   sequence: {counts:?}");

    println!("\nB. 200 INDEPENDENT random couplings, N={M} — what range does sweeps take?");
    let mut hist = [0usize; 16];
    for s in 0..200u64 {
        let m = synth::<M>(0xABCD_0000 ^ s);
        let sw = jacobi_eigen(&laplacian(&m)).sweeps;
        if sw < 16 { hist[sw] += 1; }
    }
    for (sw, n) in hist.iter().enumerate() {
        if *n > 0 { println!("   sweeps={sw}: {n} of 200"); }
    }

    print!("   seeds giving 7: ");
    for s in 0..200u64 {
        let m = synth::<M>(0xABCD_0000 ^ s);
        if jacobi_eigen(&laplacian(&m)).sweeps == 7 { print!("{s} "); }
    }
    println!();

    println!("\nC. the same at N=11 on the measured coupling, perturbed");
    let mut c11 = [0usize; 33];
    for k in 0..33u64 {
        let mut c = ciris_sim_core::COUPLING;
        c[1][4] = f64::from_bits(c[1][4].to_bits() + k);
        c[4][1] = c[1][4];
        c11[k as usize] = jacobi_eigen(&laplacian(&c)).sweeps;
    }
    println!("   sequence: {c11:?}");
}
