//! **SATURATION-3 G1: the gate.**
//!
//! Table generation sharded across workers, on the actual table path
//! (`pair::geometry_problem` into `fci::solve_determinant_from`, real species, real
//! geometries — M-FOREIGN-DOMAIN-CORROBORATION), asserting:
//!
//! 1. the assembled table is BIT-IDENTICAL across worker counts 1, 4, N;
//! 2. a deliberately corrupted shard is CONVICTED by the merge digest;
//! 3. clean runs are NEVER convicted, however they were sharded;
//! 4. a wrong warm start VOIDs its node rather than writing a silent number;
//! 5. and the mutation that SHOULD break bit-identity does break it — without which (1)
//!    would pass on an implementation that ignored its inputs.
//!
//! # Which species, and why it matters
//!
//! `S3_GATE_SPECIES` selects the triple; the default is the cheapest system that still
//! exhibits the effect under test. That last clause is the whole difficulty. The gate's
//! "must fire" half needs warm starts to actually perturb the last bits of the answer, and
//! on a small enough space they do not — a 9-determinant H3 solve is effectively exact from
//! any start, so `WorkerLocalWarmStart` would not move the table and the test would pass
//! for the wrong reason, having proved nothing.
//!
//! [`warm_start_sensitivity_is_present`] therefore runs FIRST and asserts the precondition
//! directly: on this species, at this grid, a warm start must change the answer. If it does
//! not, the whole "must fire" half is vacuous and the test says so instead of going green.

use holon_chem::elements::by_symbol;
use holon_chem::elements::Species;
use holon_tables::digest::Certificate;
use holon_tables::{generate, GenSpec, Mutation, NodeStatus, TableGrid, WarmPolicy, VoidReason};

/// The triple under test. Default `(H,H,Cl)` — a staked SATURATION-3 type at 605
/// determinants, the cheapest of the four staked chlorine combinations.
fn species() -> [Species; 3] {
    let spec = std::env::var("S3_GATE_SPECIES").unwrap_or_else(|_| "H,H,Cl".into());
    let parts: Vec<&str> = spec.split(',').collect();
    assert_eq!(parts.len(), 3, "S3_GATE_SPECIES must name three species");
    let mut out = [by_symbol("H").unwrap(); 3];
    for (i, p) in parts.iter().enumerate() {
        out[i] = by_symbol(p).unwrap_or_else(|| panic!("unknown species {p}"));
    }
    out
}

/// The gate's grid. Small, because every node is a real FCI solve, but with enough regions
/// that four workers genuinely contend and enough nodes per region that warm chains exist.
fn grid() -> TableGrid {
    let n: usize = std::env::var("S3_GATE_N")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4);
    // n x n x 2 nodes, in regions of 2 x 2 x 2: four regions at n = 4, each an eight-node
    // warm chain rooted at one cold seed.
    TableGrid::new(n, n, 2, [2, 2, 2], (2.4, 3.2), (2.6, 3.4), (0.1, 0.5))
}

fn spec() -> GenSpec {
    GenSpec::new(species(), grid())
}

/// THE PRECONDITION. If a warm start does not move the answer on this system, then the
/// "must fire" half of the mutation set is vacuous and nothing below it means anything.
///
/// Asserted rather than assumed, because it is a property of the SPACE — measured true on
/// `(H,H,Cl)` at 605 determinants (0 of 5 warm/cold pairs bit-identical, 3.4e-13 to 4.3e-12
/// hartree) and expected false on a space small enough to be solved exactly from anywhere.
#[test]
fn warm_start_sensitivity_is_present() {
    let cold = generate(&spec().with_warm(WarmPolicy::AllCold), 1);
    let warm = generate(&spec().with_warm(WarmPolicy::CanonicalChain), 1);

    assert_eq!(cold.cold_solves, cold.records.len(), "the cold arm warm-started");
    assert!(
        warm.warm_solves > 0,
        "the warm arm warm-started nothing: with {} regions over {} nodes there is no \
         chain, so the gate below would be vacuous",
        grid().n_regions(),
        grid().n_nodes()
    );

    let differing = cold
        .records
        .iter()
        .zip(warm.records.iter())
        .filter(|(c, w)| c.energy_bits != w.energy_bits)
        .count();

    println!(
        "warm-start sensitivity: {differing} of {} nodes differ between the all-cold and \
         canonical-chain tables; cold arm {} Davidson iterations, warm arm {} ({} cold / {} \
         warm solves)",
        cold.records.len(),
        cold.total_davidson_iters,
        warm.total_davidson_iters,
        warm.cold_solves,
        warm.warm_solves
    );

    assert!(
        differing > 0,
        "a warm start did not change ANY node's energy on this species. The space is small \
         enough to be solved exactly from any start, so `WorkerLocalWarmStart` cannot move \
         the table either and the must-fire half of the mutation set would pass vacuously. \
         Choose a species with a larger determinant space via S3_GATE_SPECIES."
    );
}

/// **(1) The gate.** Bit-identical across worker counts, with the schedule genuinely
/// varying: regions are handed out from a shared counter, so which worker takes which
/// region differs from run to run.
#[test]
fn table_is_bit_identical_across_worker_counts() {
    let s = spec();
    let one = generate(&s, 1);
    let four = generate(&s, 4);
    let many = generate(&s, 8);

    for (label, run) in [("1", &one), ("4", &four), ("8", &many)] {
        assert!(
            run.certificate.is_clean(),
            "the {label}-worker run's own certificate was not clean: {:?}",
            run.certificate
        );
        // M-VACUOUS-SUCCESS: a run that solved nothing would compare equal to another run
        // that solved nothing.
        assert_eq!(run.records.len(), grid().n_nodes());
        assert!(run.total_davidson_iters > 0, "the {label}-worker run did no Davidson work");
    }

    assert_eq!(
        one.table_bytes(),
        four.table_bytes(),
        "the 1-worker and 4-worker tables differ"
    );
    assert_eq!(
        one.table_bytes(),
        many.table_bytes(),
        "the 1-worker and 8-worker tables differ"
    );
    assert_eq!(one.digest(), four.digest());
    assert_eq!(one.digest(), many.digest());

    println!(
        "bit-identical over {} nodes at 1/4/8 workers; digest {}",
        one.records.len(),
        one.digest().hex()
    );
}

/// **(5a) The reordering that must NOT fire.** `holon-mesh`'s header names this trap: a
/// reorder over an exact carrier produces the identical result, so this half is the control
/// that stops the pair from being satisfiable by an implementation that ignores its inputs.
#[test]
fn reordering_the_regions_does_not_move_the_table() {
    let base = generate(&spec(), 4);
    let reversed = generate(&spec().with_mutation(Some(Mutation::ReverseRegionOrder)), 4);
    assert_eq!(
        base.table_bytes(),
        reversed.table_bytes(),
        "reversing the region order moved the table, so the schedule is reaching the numbers"
    );
    assert!(reversed.certificate.is_clean());
}

/// **(5b) The mutation that MUST fire.** Warm-starting from whatever the WORKER solved last
/// — the natural implementation — makes the table a function of the worker count. If this
/// does not fire, the canonical region decomposition is unnecessary and this crate's
/// central design argument is wrong.
#[test]
fn worker_local_warm_start_breaks_bit_identity() {
    // THE PRECONDITION. With a single region there is one chunk, so every worker count
    // processes it in the same canonical order and `worker_last` follows exactly the same
    // chain as the region's own — the mutation cannot change anything, and this test would
    // fail for a configuration reason rather than a code one. Measured: it did, at
    // S3_GATE_N=2, where a 2x2x2 grid in 2x2x2 regions is ONE region.
    //
    // Refused rather than asserted-around, in the same shape as plant (iii)'s empty sector:
    // a gate that cannot fire must say so instead of reporting either colour.
    assert!(
        grid().n_regions() >= 2,
        "this grid has {} region(s), so the worker count cannot change which chain a node \
         sits on and this mutation has nothing to move. Raise S3_GATE_N (or shrink the \
         region shape) so the grid cuts into at least two regions.",
        grid().n_regions()
    );
    let m = Some(Mutation::WorkerLocalWarmStart);
    let one = generate(&spec().with_mutation(m), 1);
    let four = generate(&spec().with_mutation(m), 4);

    assert_ne!(
        one.table_bytes(),
        four.table_bytes(),
        "worker-local warm starts did NOT make the table depend on the worker count. Either \
         the warm start does not affect the answer on this species (see \
         warm_start_sensitivity_is_present) or the mutation is not wired through — and in \
         both cases table_is_bit_identical_across_worker_counts is passing vacuously."
    );
    println!(
        "the must-fire half fires: worker-local warm starts give different tables at 1 and \
         4 workers ({} vs {} Davidson iterations)",
        one.total_davidson_iters, four.total_davidson_iters
    );
}

/// **(2) and (3): plant (iv).** One corrupted node must be convicted by the digest, and
/// clean runs must never be convicted however they were sharded.
#[test]
fn plant_iv_corrupted_shard_is_convicted_with_no_false_positives() {
    // Zero false positives first, over every worker count the gate uses. If a clean run
    // could be convicted, a conviction below would carry no information.
    for workers in [1usize, 2, 3, 4, 8] {
        let run = generate(&spec(), workers);
        assert!(
            run.certificate.is_clean(),
            "a CLEAN {workers}-worker run was convicted: {:?}",
            run.certificate
        );
    }

    // Now the plant. The lowest mantissa bit of one node's energy — the hardest corruption
    // to notice and the one a float tolerance would forgive.
    let target = (grid().n_nodes() / 2) as u32;
    let run = generate(
        &spec().with_mutation(Some(Mutation::CorruptNode {
            node: target,
            bit: 0,
        })),
        4,
    );
    match run.certificate {
        Certificate::Convicted {
            assembled,
            folded_shards,
        } => {
            assert_ne!(assembled, folded_shards);
            println!(
                "plant (iv) convicted: node {target}, one flipped mantissa bit. assembled \
                 {} vs shards {}",
                assembled.hex(),
                folded_shards.hex()
            );
        }
        Certificate::Clean { .. } => panic!(
            "plant (iv) was NOT convicted: a one-bit corruption of node {target} passed the \
             merge digest. The certificate is decorative."
        ),
    }
}

/// **(4): plant (iii).** A deliberately wrong warm start must not silently write a WRONG
/// answer. On a space large enough to get lost in, it VOIDs.
///
/// # The prereg's wording, and the case the measurement added to it
///
/// Plant (iii) is staked as "must yield the bit-identical converged energy or VOID the
/// node". The measurement that motivated this crate falsifies the first disjunct as
/// something that can ever happen: warm and cold solves of the same geometry were
/// bit-identical in 0 of 5 pairs, differing by 3.4e-13 to 4.3e-12 hartree. A warm start
/// ALWAYS moves the last bits. If "bit-identical" were the only alternative to VOID, every
/// warm-started node in the campaign would have to VOID, which is not a table.
///
/// So there are three outcomes, not two, and only the third is the failure:
///
/// 1. the wrong start converges to the CORRECT eigenvector, differing at ulp scale — benign,
///    and what happens on a space too small to get lost in;
/// 2. it converges to a DIFFERENT eigenvector and the node VOIDs — the plant firing;
/// 3. it converges to a different eigenvector and the node SCORES — the silent wrong entry
///    the plant exists to forbid.
///
/// # M-PLANT-SECTOR: the carrier is asserted before the plant is scored
///
/// Outcome 1 means the plant's sector is EMPTY on this species: there is no wrong
/// eigenvector nearby to be trapped by, so a passing VOID check would prove nothing about
/// the guard. Measured: on H3 (9 determinants) a random start still lands on the ground
/// state to 3.1e-15 hartree, so H3 cannot host this plant at all. A plant on an empty
/// sector VOIDs rather than passes, and this test says so instead of going green.
#[test]
fn plant_iii_wrong_warm_start_voids_every_node_it_traps() {
    let clean = generate(&spec(), 1);
    assert_eq!(
        clean.voided, 0,
        "the unplanted run already VOIDed {} nodes; the plant's conviction would be \
         ambiguous",
        clean.voided
    );

    // Every node wrong-started at once, so the trap is measured over the whole grid rather
    // than sampled at one geometry.
    let planted = generate(&spec().with_mutation(Some(Mutation::WrongWarmStartAll)), 1);

    // The carrier threshold sits far above the measured warm-start scatter (4.3e-12) and
    // far below the measured trapping error (7.47 hartree), so it cannot confuse the two.
    const CARRIER_FLOOR: f64 = 1e-6;

    let mut trapped = 0usize;
    let mut trapped_and_voided = 0usize;
    let mut escaped = Vec::new();
    let mut worst = 0.0f64;

    for (p, c) in planted.records.iter().zip(clean.records.iter()) {
        assert_eq!(p.node, c.node);
        let carrier = (p.energy() - c.energy()).abs();
        if carrier < CARRIER_FLOOR {
            // The wrong start found the right eigenvector anyway. Nothing to catch here,
            // and the node is correctly NOT voided.
            assert!(
                p.is_ok(),
                "node {} VOIDed even though its wrong start reached the right answer \
                 ({carrier:.3e} Ha); the guard is firing on good solves",
                p.node
            );
            continue;
        }
        trapped += 1;
        worst = worst.max(carrier);
        match p.status {
            NodeStatus::Void(reason) => {
                assert!(
                    matches!(reason, VoidReason::AboveLowestDiagonal { .. }),
                    "node {} VOIDed for {:?} rather than the variational bound",
                    p.node,
                    reason
                );
                trapped_and_voided += 1;
            }
            NodeStatus::Ok => escaped.push((p.node, carrier, p.energy(), c.energy())),
        }
    }

    println!(
        "plant (iii) over {} nodes: {trapped} trapped by a wrong warm start (worst \
         {worst:.3e} Ha), {trapped_and_voided} of those VOIDed by the variational bound; \
         {} converged correctly anyway",
        planted.records.len(),
        planted.records.len() - trapped
    );

    // M-PLANT-SECTOR: a plant on an empty sector VOIDs rather than passes.
    assert!(
        trapped > 0,
        "plant (iii) has an EMPTY SECTOR across the whole grid: no wrong warm start was \
         trapped anywhere, so the guard had nothing to catch and this test proves nothing. \
         Re-run on a species whose determinant space is large enough to get lost in."
    );

    assert!(
        escaped.is_empty(),
        "the variational guard MISSED {} trapped node(s), each of which wrote a silently \
         wrong table entry: {:?}",
        escaped.len(),
        escaped
            .iter()
            .map(|(n, d, got, want)| format!(
                "node {n}: wrote {got:.9} want {want:.9} (off by {d:.3e} Ha)"
            ))
            .collect::<Vec<_>>()
    );
}

/// The warm start's actual value, reported rather than assumed: Davidson iterations saved
/// along the canonical chain. Not an assertion about a speedup — a measurement of one, and
/// the cold-seed fraction it was bought at.
#[test]
fn warm_start_iteration_saving_is_reported() {
    let cold = generate(&spec().with_warm(WarmPolicy::AllCold), 4);
    let warm = generate(&spec().with_warm(WarmPolicy::CanonicalChain), 4);

    let saved = cold.total_davidson_iters as f64 - warm.total_davidson_iters as f64;
    let pct = 100.0 * saved / cold.total_davidson_iters as f64;
    println!(
        "warm-start saving: {} -> {} Davidson iterations over {} nodes ({pct:.1}%), at a \
         cold-seed fraction of {}/{} = {:.2}",
        cold.total_davidson_iters,
        warm.total_davidson_iters,
        warm.records.len(),
        warm.cold_solves,
        warm.records.len(),
        warm.cold_solves as f64 / warm.records.len() as f64
    );

    // The only thing asserted is that the accounting is real; whether the saving is large
    // is a measurement for the report, not a gate.
    assert_eq!(
        cold.cold_solves + cold.warm_solves,
        cold.records.len(),
        "the cold arm's solve accounting does not add up"
    );
    assert_eq!(
        warm.cold_solves + warm.warm_solves,
        warm.records.len(),
        "the warm arm's solve accounting does not add up"
    );
    assert_eq!(
        warm.cold_solves,
        grid().n_regions(),
        "the canonical chain should have exactly one cold seed per region"
    );
}
