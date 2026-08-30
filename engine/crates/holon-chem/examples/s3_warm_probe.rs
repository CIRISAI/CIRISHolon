//! SATURATION-3, the measurement the mesh design turns on: does a warm start change the
//! ANSWER, or only the PATH?
//!
//! G1 requires the assembled table to be bit-identical across shard counts, and the
//! generator is to be warm-started along a locality sweep. Those two requirements meet at
//! a question nobody has measured here: if node B's Davidson starts from node A's
//! converged vector instead of from cold, is B's energy the same f64?
//!
//! It is not obvious either way, and the two answers imply different mesh designs.
//!
//! * The eigenvalue itself is pinned far tighter than an ulp. Davidson exits at residual
//!   `1e-11`, and a Ritz value's error is `~|r|^2 / gap`, so `~1e-22 / 0.1 = 1e-21` Ha —
//!   about seven orders BELOW the `~2.8e-14` ulp at these energies. On that argument the
//!   two starts should agree bit-for-bit.
//! * But `theta` is not read off an exact object. It is computed by `jacobi_eigh` over a
//!   subspace matrix accumulated from a DIFFERENT basis, and that arithmetic carries
//!   `~eps * ||H||`, which at 100-500 Ha is itself about one ulp. On that argument they
//!   should differ in the last bit or two.
//!
//! The first argument is about the mathematics, the second about the floating point, and
//! the second wins whenever it is larger. So this is measured, on the real path, before
//! any sharding is designed around the answer.
//!
//! WHAT THE ANSWER IMPLIES. If warm and cold agree bit-for-bit, a shard may warm-start
//! from whatever it happens to hold and bit-identity across shard counts still holds. If
//! they do NOT, then the warm-start source must be a canonical function of the grid — a
//! fixed region decomposition with a fixed traversal — or the table's contents become a
//! function of the worker count, which is exactly what G1 forbids.
//!
//! Reported either way, at full prominence (M-NULL-MISSTAKE).

use holon_chem::dual::D2;
use holon_chem::elements::by_symbol;
use holon_chem::fci::{solve_determinant, solve_determinant_from};
use holon_chem::pair::geometry_problem;

fn at(x: f64, y: f64, z: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(z)]
}

/// A triangle with the two short sides `x`, `y` and the cosine `u` between them — the
/// trimer table's own coordinates, so this probe walks the surface the tables walk.
fn triangle(x: f64, y: f64, u: f64) -> Vec<[D2; 3]> {
    let s = (1.0 - u * u).max(0.0).sqrt();
    vec![at(0.0, 0.0, 0.0), at(x, 0.0, 0.0), at(y * u, y * s, 0.0)]
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let symbols: Vec<String> = if args.is_empty() {
        vec!["H".into(), "H".into(), "Cl".into()]
    } else {
        args[0].split(',').map(|s| s.to_string()).collect()
    };
    let species: Vec<_> = symbols
        .iter()
        .map(|s| by_symbol(s).unwrap_or_else(|| panic!("unknown species {s}")))
        .collect();
    assert_eq!(species.len(), 3, "this probe walks TRIPLES");

    println!("species {}", symbols.join(","));

    // A short walk along one coordinate, of the kind a locality sweep makes.
    let base = (2.6_f64, 2.8_f64, 0.30_f64);
    let steps = [0.02_f64, 0.05, 0.10, 0.20, 0.40];

    // Node A, cold. Its vector is what every warm start below begins from.
    let (space_a, mo_a, _) = geometry_problem(&species, triangle(base.0, base.1, base.2));
    println!("n_det   {}", space_a.n_det);
    let sol_a = solve_determinant(&space_a, &mo_a);
    println!(
        "node A  E = {:.15} Ha   iters {}  exit {}  resid {:.2e}",
        sol_a.e.v,
        sol_a.davidson_iters,
        sol_a.exit.label(),
        sol_a.residual
    );
    println!();

    println!(
        "{:>6}  {:>4} {:>4}  {:>24} {:>24}  {:>11}  {}",
        "step", "cold", "warm", "E(cold)", "E(warm)", "|dE|", "bits"
    );

    let mut n_bit_identical = 0usize;
    let mut n_total = 0usize;
    let mut worst = 0.0f64;

    for &d in &steps {
        // Step in x only: the neighbour a canonical traversal would come from.
        let (space_b, mo_b, _) = geometry_problem(&species, triangle(base.0 + d, base.1, base.2));
        assert_eq!(
            space_b.n_det, space_a.n_det,
            "the two geometries must share a determinant space for the vector to transfer"
        );

        let cold = solve_determinant(&space_b, &mo_b);
        let warm = solve_determinant_from(&space_b, &mo_b, Some(&sol_a.vector));

        let same = cold.e.v.to_bits() == warm.e.v.to_bits();
        let de = (cold.e.v - warm.e.v).abs();
        if same {
            n_bit_identical += 1;
        }
        n_total += 1;
        worst = worst.max(de);

        println!(
            "{d:>6.2}  {:>4} {:>4}  {:>24.15} {:>24.15}  {:>11.3e}  {:>9}  {} / {}",
            cold.davidson_iters,
            warm.davidson_iters,
            cold.e.v,
            warm.e.v,
            de,
            if same { "IDENTICAL" } else { "DIFFER" },
            cold.exit.label(),
            warm.exit.label()
        );
        // M-EXIT-DISCRIMINATOR: the exit reason is DATA and is reported per row, not
        // asserted away. What is refused is the one exit that would make the comparison
        // meaningless — a solve stopped by the iteration cap has not finished at all, and
        // comparing two unfinished solves measures the cap rather than the warm start.
        for (which, sol) in [("cold", &cold), ("warm", &warm)] {
            assert_ne!(
                sol.exit.label(),
                "iteration cap",
                "{which} solve at step {d} hit the iteration cap (resid {:.2e}); comparing \
                 unfinished solves would measure the cap, not the warm start",
                sol.residual
            );
        }
    }

    // ---- plant (iii): a DELIBERATELY WRONG warm start.
    //
    // Not a perturbed guess — a vector with no relationship to the answer at all. The
    // prereg's requirement is that it still reaches the identical energy or the node
    // VOIDs; what it must never do is produce a silently different table entry.
    println!();
    println!("--- plant (iii): deliberately wrong warm start ---");
    let (space_b, mo_b, _) = geometry_problem(&species, triangle(base.0 + 0.10, base.1, base.2));
    let reference = solve_determinant(&space_b, &mo_b);

    let mut wrong = vec![0.0f64; space_b.n_det];
    let mut seed = 0xdead_beef_1234_5678u64;
    for x in wrong.iter_mut() {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *x = ((seed >> 33) as f64 / (1u64 << 31) as f64) - 0.5;
    }
    // M-PLANT-OBS: the plant's carrier is asserted nonzero before it is scored. A "wrong"
    // start that happened to be the right answer would test nothing.
    let overlap: f64 = wrong
        .iter()
        .zip(reference.vector.iter())
        .map(|(a, b)| a * b)
        .sum::<f64>()
        .abs()
        / wrong.iter().map(|x| x * x).sum::<f64>().sqrt();
    assert!(
        overlap < 0.5,
        "the 'wrong' start has overlap {overlap:.3} with the answer; it is not wrong \
         enough to be a plant"
    );
    println!("planted start overlap with the true vector: {overlap:.4}");

    // Several planted starts, because ONE plant that fires tells us the plant works and
    // nothing about what separates a good warm start from a bad one. The guard the mesh
    // needs is a DISCRIMINATOR, and a discriminator has to be measured against both
    // classes.
    println!(
        "{:>18}  {:>22} {:>11} {:>5}  {:>10}  {}",
        "start", "E", "|dE| vs cold", "iters", "residual", "exit"
    );
    println!(
        "{:>18}  {:>22.15} {:>11} {:>5}  {:>10.3e}  {}",
        "cold (reference)",
        reference.e.v,
        "-",
        reference.davidson_iters,
        reference.residual,
        reference.exit.label()
    );

    let good = solve_determinant_from(&space_b, &mo_b, Some(&sol_a.vector));
    for (name, v) in [
        ("neighbour (good)", &good.vector),
        ("random (planted)", &wrong),
    ] {
        let s = solve_determinant_from(&space_b, &mo_b, Some(v));
        println!(
            "{name:>18}  {:>22.15} {:>11.3e} {:>5}  {:>10.3e}  {}",
            s.e.v,
            (s.e.v - reference.e.v).abs(),
            s.davidson_iters,
            s.residual,
            s.exit.label()
        );
    }
    let planted = solve_determinant_from(&space_b, &mo_b, Some(&wrong));
    println!();
    println!(
        "planted vs reference: |dE| = {:.3e}   {}",
        (planted.e.v - reference.e.v).abs(),
        if planted.e.v.to_bits() == reference.e.v.to_bits() {
            "BIT-IDENTICAL"
        } else {
            "DIFFERS -- a wrong warm start moved the answer"
        }
    );
    println!(
        "planted residual {:.3e} vs reference residual {:.3e}  --  ratio {:.1}x",
        planted.residual,
        reference.residual,
        planted.residual / reference.residual
    );
    println!(
        "planted exit reason is {:?}, reference exit reason is {:?}  --  {}",
        planted.exit.label(),
        reference.exit.label(),
        if planted.exit.label() == reference.exit.label() {
            "THE SAME. The exit reason cannot discriminate; a guard must read the residual."
        } else {
            "different, so the exit reason discriminates."
        }
    );

    // ---- THE DISCRIMINATOR.
    //
    // The residual cannot separate these: a residual is small for ANY eigenvector, and the
    // planted start converged cleanly onto the WRONG one. That is the carbon failure again,
    // reached through the warm-start channel instead of through a spin sector.
    //
    // What does separate them is free and rigorous. `E_ground <= min_i H_ii`, because a
    // single determinant is itself a trial vector and the variational principle applies to
    // it. `diag` is already computed for the preconditioner, so this costs one pass over an
    // array the solve has in hand. A converged theta ABOVE the lowest diagonal is not the
    // ground state, whatever its residual says.
    println!();
    println!("--- the discriminator: E <= min(diag), free and rigorous ---");
    let ci0 = holon_chem::fci::ci_ints(&mo_b, holon_chem::fci::Order::Value);
    let diag = space_b.diagonal(&ci0);
    let min_diag = diag.iter().copied().fold(f64::INFINITY, f64::min);
    println!("min(diag)              {min_diag:.15} Ha");
    for (name, e) in [
        ("cold (reference)", reference.e.v),
        ("neighbour (good)", good.e.v),
        ("random (planted)", planted.e.v),
    ] {
        let ok = e <= min_diag;
        println!(
            "{name:>18}  E = {e:>22.15}   E - min(diag) = {:>11.3e}   {}",
            e - min_diag,
            if ok { "passes" } else { "VOID -- above the lowest diagonal" }
        );
    }
    let guard_fires = planted.e.v > min_diag;
    let guard_clean = reference.e.v <= min_diag && good.e.v <= min_diag;
    println!(
        "guard verdict: fires on the plant = {guard_fires}, clean on both good solves = \
         {guard_clean}  --  {}",
        if guard_fires && guard_clean {
            "USABLE: it separates the classes with zero false positives here"
        } else if !guard_fires {
            "NOT SUFFICIENT: the plant slips past it; a second guard is owed"
        } else {
            "BROKEN: it fires on good solves too"
        }
    );
    println!(
        "(iteration count is a second, weaker signal: {} for the plant against {} and {} \
         for the two good solves -- suggestive, not a bound)",
        planted.davidson_iters, reference.davidson_iters, good.davidson_iters
    );

    println!();
    println!("=== VERDICT ===");
    println!(
        "bit-identical on {n_bit_identical} of {n_total} warm/cold pairs; worst |dE| {worst:.3e} Ha"
    );
    if n_bit_identical == n_total {
        println!(
            "Warm starts do NOT move the answer on this path. A shard could warm-start \
             from anything it holds."
        );
    } else {
        println!(
            "Warm starts DO move the answer, in the last bits. The mesh therefore CANNOT \
             let the warm-start source depend on worker count: the region decomposition \
             and its traversal must be a canonical function of the grid, or the assembled \
             table stops being bit-identical across shard counts."
        );
    }
}
