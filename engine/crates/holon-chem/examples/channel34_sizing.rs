//! SCRATCH (not a campaign artifact): sizing channels 3 (pair dispersion) and 4
//! (three-body dispersion) for WATER, to CT1_PREREG.md §4.
//!
//! ```text
//! cargo run --release -p holon-chem --example channel34_sizing -- count|ring|scf3|pa|all [OUT_DIR]
//! ```
//!
//! Nothing here is a harvest. `count` prices spaces at the admission door; `ring` builds
//! the cyclic water trimer and reads its contacts; `scf3` measures the density-embedding
//! fixed points that need only monomer solves; `pa` runs the dimer-in-field solves of
//! `rho_pa_subset` one at a time with a clock on each, and prices (never calls) the exact
//! trimer that `subset_in_field` on the whole set would be.

use holon_chem::budget::{admit, price_determinant, price_mpo};
use holon_chem::density_embed::{
    classical_interaction, embed_densities, partners_except_flagged, solve_in_densities,
    DensityStart, Partner,
};
use holon_chem::elements::{by_symbol, Species};
use holon_chem::embed::{water_centers, Fragment, ANGSTROM_TO_BOHR};
use holon_chem::fci::Strings;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

// ------------------------------------------------------------------ counting

/// C(n, k) exactly, in u128.
fn binom(n: u64, k: u64) -> u128 {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut acc: u128 = 1;
    for i in 0..k {
        acc = acc * (n - i) as u128 / (i + 1) as u128;
    }
    acc
}

fn gib(bytes: u128) -> f64 {
    bytes as f64 / (1024.0 * 1024.0 * 1024.0)
}

struct Space {
    label: &'static str,
    n_orb: u64,
    n_e: u64,
    /// Build the engine's own `Strings` to cross-check the binomial. Off for the spaces
    /// whose string list alone would be tens of gigabytes.
    verify: bool,
}

fn phase_count(out: &PathBuf) {
    println!("\n================ 1. THE SPACES, PRICED AT THE ADMISSION DOOR ================");
    println!("(the hard determinant cap of 2,000,000 was deleted 2026-09-02, fci.rs:1002;");
    println!(" a space is now admitted by budget::price_determinant against a live probe)\n");

    let spaces = [
        Space { label: "water monomer, STO-3G", n_orb: 7, n_e: 5, verify: true },
        Space { label: "water DIMER, STO-3G (of record)", n_orb: 14, n_e: 10, verify: true },
        Space { label: "HF trimer, STO-3G (EMBED-2/3's carrier)", n_orb: 18, n_e: 15, verify: true },
        Space { label: "water TRIMER, STO-3G", n_orb: 21, n_e: 15, verify: true },
        Space { label: "water dimer + 1 p-shell on 2 H", n_orb: 20, n_e: 10, verify: true },
        Space { label: "water dimer + 1 p-shell on 4 H", n_orb: 26, n_e: 10, verify: false },
    ];

    let mut rows = Vec::new();
    for s in spaces.iter() {
        let n_str = binom(s.n_orb, s.n_e);
        let verified = if s.verify {
            let t0 = Instant::now();
            let st = Strings::new(s.n_orb as usize, s.n_e as usize);
            let got = st.masks.len() as u128;
            let secs = t0.elapsed().as_secs_f64();
            drop(st);
            assert_eq!(got, n_str, "binomial disagrees with Strings::new for {}", s.label);
            format!("engine-counted ({:.1} s)", secs)
        } else {
            "hand-counted (string list alone would be >14 GiB)".to_string()
        };
        let n_det = n_str * n_str;
        let price = price_determinant(usize::try_from(n_det).unwrap_or(usize::MAX));
        let verdict = match admit(&price) {
            Ok(what) => format!("ADMITTED ({what})"),
            Err(r) => format!("REFUSED ({})", r.verdict),
        };
        // one sigma needs the vector it reads and the vector it writes, nothing else
        let sigma_floor: u128 = 2 * n_det * 8;
        let mpo = price_mpo(s.n_orb as usize);
        println!("{}", s.label);
        println!("  n_orb {}  n_alpha=n_beta {}   strings C({},{}) = {}   [{}]", s.n_orb, s.n_e, s.n_orb, s.n_e, n_str, verified);
        println!("  n_det = strings^2 = {}  ({:.4e})", n_det, n_det as f64);
        println!("  Davidson working set (2*48+8 vectors x 8 B): {} B = {:.1} GiB  -> {}", price.bytes, gib(price.bytes as u128), verdict);
        println!("  one sigma, the two vectors alone:            {} B = {:.1} GiB", sigma_floor, gib(sigma_floor));
        println!("  MPS route, dense MPO at {} orbitals:         {} B = {:.2} GiB [{}]", s.n_orb, mpo.bytes, gib(mpo.bytes as u128), if mpo.provenance.starts_with("PROVISIONAL") { "PROVISIONAL" } else { "measured" });
        println!();
        rows.push(format!(
            "{{\"label\": \"{}\", \"n_orb\": {}, \"n_e\": {}, \"n_strings\": {}, \"n_det\": {}, \"davidson_bytes\": {}, \"sigma_two_vector_bytes\": {}, \"mpo_bytes\": {}, \"door\": \"{}\", \"counting\": \"{}\"}}",
            s.label, s.n_orb, s.n_e, n_str, n_det, price.bytes, sigma_floor, mpo.bytes, verdict.replace('"', "'"), verified
        ));
    }

    // The Heitler-London product of THREE monomers, priced.
    println!("---- the Heitler-London product of THREE water monomers ----");
    let nq = binom(21, 15); // dimer-analogue: the trimer's strings per spin lane
    let n_prod = binom(7, 5) * binom(7, 5) * binom(7, 5); // one monomer string triple per lane
    let n_det_tri = nq * nq;
    let lane_bytes: u128 = nq * n_prod * 8;
    println!("  heitler_london_undeformed's expansion matrix, per spin lane:");
    println!("    T ({} trimer strings x {} monomer-string triples) x 8 B = {} B = {:.1} GiB, TWO lanes = {:.1} GiB",
        nq, n_prod, lane_bytes, gib(lane_bytes), gib(2 * lane_bytes));
    println!("  the product vector v and one sigma on it: 2 x {} x 8 B = {:.1} GiB", n_det_tri, gib(2 * n_det_tri * 8));
    println!("  floor for ONE energy evaluation <v|H|v>:   {:.1} GiB", gib(2 * lane_bytes + 2 * n_det_tri * 8));
    let det_flops = nq as f64 * n_prod as f64 * 15.0f64.powi(3) * 2.0;
    println!("  the determinants det T[Q,P] alone: {:.1} x {} x O(15^3) ~ {:.2e} flops per lane", nq as f64, n_prod, det_flops);
    println!();

    // The CT-1 closed sector (channel 3's instrument), at three basis sizes.
    println!("---- CT-1's CLOSED sector (channel 3's instrument), by basis ----");
    let sector = |na: u64, nb: u64| -> u128 {
        let s = binom(na, 5) * binom(nb, 5);
        s * s
    };
    for (label, na, nb) in [
        ("STO-3G, 7 + 7 orbitals (CT-1 as frozen)", 7u64, 7u64),
        ("+1 p-shell on one H of each monomer, 10 + 10", 10, 10),
        ("+1 p-shell on both H of one monomer, 13 + 7", 13, 7),
        ("+1 p-shell on every H, 13 + 13", 13, 13),
    ] {
        let n = sector(na, nb);
        let price = price_determinant(usize::try_from(n).unwrap_or(usize::MAX));
        let verdict = match admit(&price) {
            Ok(_) => "ADMITTED".to_string(),
            Err(r) => format!("REFUSED ({})", r.verdict),
        };
        println!("  {label}: sector = [C({na},5)*C({nb},5)]^2 = {n}  ({:.3e}), {:.1} GiB -> {verdict}", n as f64, gib(price.bytes as u128));
    }
    println!();

    fs::write(out.join("counts.json"), format!("[\n  {}\n]\n", rows.join(",\n  "))).unwrap();
}

// ------------------------------------------------------------------ the ring

/// The cyclic water trimer: three monomers at EMBED-1's pin, oxygens on an equilateral
/// triangle of side `side` bohr in the xy-plane, each donating one O-H straight at the next
/// oxygen around the ring, the free O-H tilted OUTWARD by the monomer angle in the ring plane.
fn water_ring(o: Species, h: Species, r: f64, theta: f64, side: f64) -> Vec<Fragment> {
    let rc = side / 3.0f64.sqrt(); // circumradius of an equilateral triangle
    let vert = |k: usize| -> [f64; 3] {
        let a = std::f64::consts::TAU * (k as f64) / 3.0;
        [rc * a.cos(), rc * a.sin(), 0.0]
    };
    (0..3)
        .map(|k| {
            let ok = vert(k);
            let on = vert((k + 1) % 3);
            let d = [on[0] - ok[0], on[1] - ok[1], 0.0];
            let dn = (d[0] * d[0] + d[1] * d[1]).sqrt();
            let u = [d[0] / dn, d[1] / dn, 0.0]; // donor direction: at the next oxygen
            let rn = (ok[0] * ok[0] + ok[1] * ok[1]).sqrt();
            let n = [ok[0] / rn, ok[1] / rn, 0.0]; // outward radial: the free H's side
            let h1 = [ok[0] + r * u[0], ok[1] + r * u[1], 0.0];
            let (s, c) = (theta.sin(), theta.cos());
            let h2 = [ok[0] + r * (c * u[0] + s * n[0]), ok[1] + r * (c * u[1] + s * n[1]), 0.0];
            Fragment::new(vec![o, h, h], vec![ok, h1, h2], vec![-2.0, 1.0, 1.0])
        })
        .collect()
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn report_ring(frags: &[Fragment]) -> (f64, f64) {
    println!("\n================ 2. THE RING ================");
    let names = ["A", "B", "C"];
    for (k, f) in frags.iter().enumerate() {
        for (j, c) in f.centers.iter().enumerate() {
            let sym = if j == 0 { "O" } else if j == 1 { "H(donor)" } else { "H(free)" };
            println!("  {}.{:<9} [{:+.6}, {:+.6}, {:+.6}] bohr", names[k], sym, c[0], c[1], c[2]);
        }
    }
    let mut oo = Vec::new();
    for i in 0..3 {
        for j in (i + 1)..3 {
            let d = dist(frags[i].centers[0], frags[j].centers[0]);
            oo.push(d);
            println!("  O({})-O({}) = {:.6} bohr = {:.4} A", names[i], names[j], d, d / ANGSTROM_TO_BOHR);
        }
    }
    let mut shortest_ho = f64::INFINITY;
    let mut shortest_hh = f64::INFINITY;
    for i in 0..3 {
        for j in 0..3 {
            if i == j {
                continue;
            }
            for (hk, hc) in frags[i].centers.iter().enumerate().skip(1) {
                let d = dist(*hc, frags[j].centers[0]);
                let tag = if hk == 1 { "donor" } else { "free " };
                println!("  H({} {})...O({}) = {:.6} bohr = {:.4} A", names[i], tag, names[j], d, d / ANGSTROM_TO_BOHR);
                shortest_ho = shortest_ho.min(d);
            }
        }
    }
    for i in 0..3 {
        for j in (i + 1)..3 {
            for a in 1..3 {
                for b in 1..3 {
                    shortest_hh = shortest_hh.min(dist(frags[i].centers[a], frags[j].centers[b]));
                }
            }
        }
    }
    println!("  SHORTEST cross-unit H...O = {:.6} bohr = {:.4} A", shortest_ho, shortest_ho / ANGSTROM_TO_BOHR);
    println!("  SHORTEST cross-unit H...H = {:.6} bohr = {:.4} A", shortest_hh, shortest_hh / ANGSTROM_TO_BOHR);
    println!("  FIELD-8's closure boundary is H...O = 1.95 bohr; this ring's shortest is {:.2}x that.", shortest_ho / 1.95);
    (oo[0], shortest_ho)
}

// ------------------------------------------------------------------ the embedding

/// The engine's own embedded total on a subset, generalising EMBED-3's dimer composition
/// (`embed3_campaign.rs::run_water`: `E_A[B] + E_B[A] - E_cl(A,B)`):
/// `E_emb(S) = sum_i E_i[fixed-point field of the others in S] - sum_{i<j} E_cl(i,j)`.
fn embedded_total(frags: &[Fragment]) -> (f64, usize, f64, Vec<f64>) {
    let fp = embed_densities(frags, DensityStart::Zero);
    let fi = embed_densities(frags, DensityStart::Isolated);
    let dp = fp
        .densities
        .iter()
        .flatten()
        .zip(fi.densities.iter().flatten())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f64, f64::max);
    let n = frags.len();
    let mut mono = Vec::new();
    let mut acc = 0.0;
    for i in 0..n {
        let e = solve_in_densities(&frags[i], &partners_except_flagged(frags, &fp.densities, &[i], None)).e_total;
        mono.push(e);
        acc += e;
    }
    for i in 0..n {
        for j in (i + 1)..n {
            acc -= classical_interaction(&frags[i], &fp.densities[i], &frags[j], &fp.densities[j]);
        }
    }
    (acc, fp.sweeps, dp, mono)
}

fn phase_scf3(frags: &[Fragment], out: &PathBuf) {
    println!("\n================ 3. THE EMBEDDING'S THREE-BODY TERM THAT NEEDS NO TRIMER SOLVE ================");
    let t0 = Instant::now();
    let e0: Vec<f64> = frags.iter().map(|f| solve_in_densities(f, &[]).e_total).collect();
    println!("  isolated monomers: {:.12e}  {:.12e}  {:.12e} Ha  ({:.1} s)", e0[0], e0[1], e0[2], t0.elapsed().as_secs_f64());

    let t1 = Instant::now();
    let (e_abc, sw3, dp3, mono3) = embedded_total(frags);
    let d_abc = e_abc - e0.iter().sum::<f64>();
    println!("  E_emb(ABC) = {:.12e} Ha, sweeps {}, both starts agree to {:.1e}", e_abc, sw3, dp3);
    println!("    E_i[field of the other two]: {:.12e}  {:.12e}  {:.12e}", mono3[0], mono3[1], mono3[2]);
    println!("    dE(ABC) = {:.6e} Ha = {:.4} mHa   ({:.1} s)", d_abc, d_abc * 1e3, t1.elapsed().as_secs_f64());

    let names = ["AB", "AC", "BC"];
    let pairs = [(0usize, 1usize), (0, 2), (1, 2)];
    let mut dsum = 0.0;
    let mut prs = Vec::new();
    for (k, &(i, j)) in pairs.iter().enumerate() {
        let t = Instant::now();
        let sub = vec![frags[i].clone(), frags[j].clone()];
        let (e, sw, dp, _) = embedded_total(&sub);
        let d = e - e0[i] - e0[j];
        dsum += d;
        println!("  E_emb({}) = {:.12e} Ha, sweeps {}, dp {:.1e}, dE = {:.6e} Ha = {:.4} mHa  ({:.1} s)", names[k], e, sw, dp, d, d * 1e3, t.elapsed().as_secs_f64());
        prs.push(format!("{{\"pair\": \"{}\", \"e_emb\": {:.17e}, \"delta\": {:.17e}, \"sweeps\": {}}}", names[k], e, d, sw));
    }
    let e3 = d_abc - dsum;
    println!("\n  THREE-BODY INDUCTION (channel 2's non-additivity, density embedding):");
    println!("    E_3^SCF = dE(ABC) - [dE(AB)+dE(AC)+dE(BC)] = {:.6e} - {:.6e}", d_abc, dsum);
    println!("            = {:.9e} Ha = {:.5} mHa = {:.4e} of dE(ABC)", e3, e3 * 1e3, e3 / d_abc);
    println!("    (this is NOT channel 4: the correlated trimer is what carries three-body DISPERSION)");
    println!("  wall for the whole of phase 3: {:.1} s", t0.elapsed().as_secs_f64());
    fs::write(
        out.join("scf3.json"),
        format!(
            "{{\n  \"e0\": [{:.17e}, {:.17e}, {:.17e}],\n  \"e_emb_abc\": {:.17e},\n  \"delta_abc\": {:.17e},\n  \"pairs\": [{}],\n  \"delta_pairs_sum\": {:.17e},\n  \"e3_scf\": {:.17e},\n  \"wall_seconds\": {:.3}\n}}\n",
            e0[0], e0[1], e0[2], e_abc, d_abc, prs.join(", "), dsum, e3, t0.elapsed().as_secs_f64()
        ),
    )
    .unwrap();
}

// ------------------------------------------------------------------ the pairwise leg

fn phase_pa(frags: &[Fragment], out: &PathBuf) {
    println!("\n================ 4. rho_pa_subset's DIMER-IN-FIELD SOLVES, ONE AT A TIME ================");
    let t0 = Instant::now();
    let fp = embed_densities(frags, DensityStart::Zero);
    println!("  the three-fragment fixed point: {} sweeps, converged {}, ({:.1} s)", fp.sweeps, fp.converged, t0.elapsed().as_secs_f64());

    let mut mono = Vec::new();
    for i in 0..3 {
        let e = solve_in_densities(&frags[i], &partners_except_flagged(frags, &fp.densities, &[i], None)).e_total;
        println!("  E_{}[field of the other two] = {:.12e} Ha", i, e);
        mono.push(e);
    }

    // the exact trimer term, PRICED and not called
    let n_det_tri = binom(21, 15) * binom(21, 15);
    let price = price_determinant(usize::try_from(n_det_tri).unwrap_or(usize::MAX));
    match admit(&price) {
        Ok(_) => println!("  the exact trimer would be ADMITTED (unexpected) -- not run by this scratch example anyway"),
        Err(r) => {
            println!("\n  subset_in_field(frags, dens, [0,1,2]) IS the exact trimer supermolecule.");
            println!("  {}\n", r);
        }
    }

    let pairs = [(0usize, 1usize), (0, 2), (1, 2)];
    let mut lines = Vec::new();
    for &(i, j) in pairs.iter() {
        let (sp, ce) = holon_chem::seam::joined(&frags[i], &frags[j]);
        let mut w = frags[i].weights.clone();
        w.extend_from_slice(&frags[j].weights);
        let dimer = Fragment::new(sp, ce, w);
        let pts: Vec<Partner> = partners_except_flagged(frags, &fp.densities, &[i, j], None);
        let t = Instant::now();
        let c0 = cpu_seconds();
        let ds = solve_in_densities(&dimer, &pts);
        let wall = t.elapsed().as_secs_f64();
        println!(
            "  E_{}{}[field of the third] = {:.12e} Ha   n_det {}  {} iters  residual {:.2e}  wall {:.1} s  cpu {:.1} s",
            i, j, ds.e_total, ds.gp.space.n_det, ds.sol.davidson_iters, ds.sol.residual, wall, cpu_seconds() - c0
        );
        lines.push(format!(
            "{{\"pair\": [{i}, {j}], \"e\": {:.17e}, \"n_det\": {}, \"davidson_iters\": {}, \"residual\": {:.3e}, \"wall_seconds\": {:.2}, \"cpu_seconds\": {:.2}}}",
            ds.e_total, ds.gp.space.n_det, ds.sol.davidson_iters, ds.sol.residual, wall, cpu_seconds() - c0
        ));
        fs::write(
            out.join("pa_partial.json"),
            format!("{{\n  \"mono\": [{:.17e}, {:.17e}, {:.17e}],\n  \"dimers\": [{}],\n  \"wall_seconds\": {:.2}\n}}\n", mono[0], mono[1], mono[2], lines.join(", "), t0.elapsed().as_secs_f64()),
        )
        .unwrap();
    }
    println!("  phase 4 wall: {:.1} s", t0.elapsed().as_secs_f64());
}

fn cpu_seconds() -> f64 {
    let s = fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let tail = &s[s.rfind(')').map(|i| i + 2).unwrap_or(0)..];
    let f: Vec<&str> = tail.split_whitespace().collect();
    let ut: f64 = f.get(11).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    let st: f64 = f.get(12).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    (ut + st) / 100.0
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let what = args.get(1).map(String::as_str).unwrap_or("all");
    let out = PathBuf::from(
        args.get(2)
            .cloned()
            .unwrap_or_else(|| "/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/ch34".to_string()),
    );
    fs::create_dir_all(&out).expect("out dir");
    println!("threads: lane_threads() = {}", holon_chem::lanes::lane_threads());

    let (o, h) = (by_symbol("O").expect("O"), by_symbol("H").expect("H"));
    // EMBED-1's water pin
    const H2O_R: f64 = 1.9435738400;
    const H2O_THETA: f64 = 1.6887434037;
    let _ = water_centers(H2O_R, H2O_THETA);
    let side = 2.9 * ANGSTROM_TO_BOHR;
    let frags = water_ring(o, h, H2O_R, H2O_THETA, side);

    if what == "count" || what == "all" {
        phase_count(&out);
    }
    if what == "ring" || what == "scf3" || what == "pa" || what == "all" {
        report_ring(&frags);
    }
    if what == "scf3" || what == "all" {
        phase_scf3(&frags, &out);
    }
    if what == "pa" {
        phase_pa(&frags, &out);
    }
}
