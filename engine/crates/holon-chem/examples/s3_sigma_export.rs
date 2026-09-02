//! SATURATION-3 G2: export the REAL `(O,O,O)` sigma problem, so the GPU arm is measured
//! against the actual table path's data and checked against the actual table path's answer.
//!
//! M-FOREIGN-DOMAIN-CORROBORATION says the GPU gate exercises the real path, never a toy.
//! A GPU kernel benchmarked on synthetic index structures of the right SHAPE would still be
//! a toy: the gather patterns, and therefore the memory behaviour, come from the excitation
//! lists' contents, not from their dimensions. So this writes out the real thing —
//! assembled by `geometry_problem`, the same call the tables make — together with the CPU's
//! own sigma on a fixed input vector, which is what the GPU result is checked against.
//!
//! Layout (little-endian throughout):
//! ```text
//!   u64  n_orb, n_alpha_str, n_beta_str, ns_a, ns_b, n_det
//!   f64  k[n2]                       folded one-electron integrals
//!   f64  g[n2*n2]                    two-electron integrals in CI form
//!   for each alpha string, ns_a x (u32 pq, u32 dst, f64 sign)
//!   for each beta  string, ns_b x (u32 pq, u32 dst, f64 sign)
//!   f64  c[n_det]                    the probe vector
//!   f64  sigma_cpu[n_det]            what `sigma_direct` makes of it
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::by_symbol;
use holon_chem::fci::{ci_ints, sigma_direct, Order};
use holon_chem::pair::geometry_problem;
use std::io::Write;

fn at(x: f64, y: f64, z: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(z)]
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let symbols: Vec<String> = args
        .first()
        .map(|s| s.split(',').map(|t| t.to_string()).collect())
        .unwrap_or_else(|| vec!["O".into(), "O".into(), "O".into()]);
    let out_path = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "/tmp/s3_sigma_ooo.bin".to_string());

    let species: Vec<_> = symbols
        .iter()
        .map(|s| by_symbol(s).unwrap_or_else(|| panic!("unknown species {s}")))
        .collect();

    let s = 2.4_f64;
    let centers = vec![
        at(0.0, 0.0, 0.0),
        at(s, 0.0, 0.0),
        at(0.5 * s, 0.75_f64.sqrt() * s, 0.0),
    ];
    let (space, mo, _nuc) = geometry_problem(&species, centers);
    let n = space.n_orb;
    let n2 = n * n;
    let na = space.alpha().len();
    let nb = space.beta().len();

    // The excitation lists are UNIFORM in length: a string with `e` of `n` orbitals
    // occupied has exactly `e * (n - e) + e` single excitations, every string alike. The
    // GPU layout is a flat array that depends on that, so it is asserted rather than
    // assumed — a ragged list would silently corrupt every stride.
    let ns_a = space.alpha().singles[0].len();
    let ns_b = space.beta().singles[0].len();
    assert!(
        space.alpha().singles.iter().all(|s| s.len() == ns_a),
        "alpha excitation lists are ragged; the flat GPU layout assumes they are not"
    );
    assert!(
        space.beta().singles.iter().all(|s| s.len() == ns_b),
        "beta excitation lists are ragged; the flat GPU layout assumes they are not"
    );

    let ci0 = ci_ints(&mo, Order::Value);

    let mut c = vec![0.0f64; space.n_det];
    let mut seed = 0x243f_6a88_85a3_08d3u64;
    for x in c.iter_mut() {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *x = ((seed >> 33) as f64 / (1u64 << 31) as f64) - 0.5;
    }
    let mut sigma = vec![0.0f64; space.n_det];
    sigma_direct(&space, &ci0, &c, &mut sigma);
    // M-VACUOUS-SUCCESS: an all-zero reference would let any GPU kernel agree with it.
    let nz = sigma.iter().filter(|x| **x != 0.0).count();
    assert!(
        nz == space.n_det,
        "the CPU reference sigma has {nz} of {} entries nonzero; a degenerate reference \
         would let a broken GPU kernel agree with it",
        space.n_det
    );

    let f = std::fs::File::create(&out_path).expect("cannot create the export file");
    let mut w = std::io::BufWriter::new(f);
    for v in [n, na, nb, ns_a, ns_b, space.n_det] {
        w.write_all(&(v as u64).to_le_bytes()).unwrap();
    }
    assert_eq!(ci0.k.len(), n2, "k is not n^2");
    assert_eq!(ci0.g.len(), n2 * n2, "g is not n^4");
    for v in ci0.k.iter().chain(ci0.g.iter()) {
        w.write_all(&v.to_le_bytes()).unwrap();
    }
    for strings in [space.alpha(), space.beta()] {
        for row in strings.singles.iter() {
            for &(pq, sign, dst) in row.iter() {
                w.write_all(&(pq as u32).to_le_bytes()).unwrap();
                w.write_all(&dst.to_le_bytes()).unwrap();
                w.write_all(&sign.to_le_bytes()).unwrap();
            }
        }
    }
    for v in c.iter().chain(sigma.iter()) {
        w.write_all(&v.to_le_bytes()).unwrap();
    }
    w.flush().unwrap();

    println!("species     {}", symbols.join(","));
    println!("n_orb       {n}");
    println!("n_alpha     {na}   singles/string {ns_a}");
    println!("n_beta      {nb}   singles/string {ns_b}");
    println!("n_det       {}", space.n_det);
    println!(
        "mixed-block GEMM per sigma: ({ns_a} x {n2}) * ({n2} x {nb}) batched over {na} = \
         {:.3} GFLOP",
        2.0 * ns_a as f64 * n2 as f64 * nb as f64 * na as f64 / 1e9
    );
    println!("wrote       {out_path}");
}
