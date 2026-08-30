//! MIXTURES-1 gate B1's reference dump: every number an all-hydrogen scene produces,
//! printed as RAW BITS.
//!
//! # Why bits and not digits
//!
//! B1 stakes that a hydrogen scene through the pair-table BANK is what the single-table
//! sandbox produced "BIT-FOR-BIT". Printed decimal — even at 17 significant digits —
//! round-trips for `f64`, but a comparison written against a decimal string is a
//! comparison a reader has to trust the formatter for. `{:016x}` of `to_bits()` is the
//! number itself.
//!
//! # Why this is a module and not an example
//!
//! Two callers need the identical bytes: `examples/b1_reference.rs`, which was run in a
//! git worktree at the commit BEFORE the bank landed to produce
//! `tests/data/b1_hydrogen_reference.txt`, and `tests/mixtures.rs`, which re-derives it
//! and asserts equality. Two implementations of one dump is how the two of them come to
//! disagree about what they are comparing, so there is one, included by both through
//! `#[path]`.
//!
//! The dump was checked to be identical in the release and debug profiles before the
//! comparison was made a gate — otherwise the test would have been a profile detector.
#![allow(dead_code)]

use holon_render::sim::{Boundary, Dims, Sim};

fn bits(x: f64) -> String {
    format!("{:016x}", x.to_bits())
}

/// The three staked all-hydrogen scenes, dumped in order.
pub fn dump_all() -> String {
    let mut out = String::new();
    run(&mut out, "pair-2H-2D", 2, Dims::Two, false, 200, 64);
    run(&mut out, "quench-16H-2D", 16, Dims::Two, true, 100, 64);
    run(&mut out, "quench-16H-3D", 16, Dims::Three, true, 100, 64);
    out
}

fn run(out: &mut String, name: &str, n: usize, dims: Dims, trimer: bool, frames: usize, substeps: u32) {
    let mut s = Sim::empty();
    s.dims = dims;
    s.boundary = Boundary::Walls;
    let status = holon_render::generate_table(&mut s, 0.3, 10.0, 492);
    if trimer {
        holon_render::generate_trimer_table(&mut s);
    }
    s.reset(n);

    out.push_str(&format!("# scene {name}  n={n}  frames={frames}  substeps={substeps}  status={status}"));
    out.push('\n');
    out.push_str(&format!("# every value is the f64's bit pattern"));
    out.push('\n');
    out.push_str(&format!(
        "dt {}  dt_reference {}  omega_e {}  period {}",
        bits(s.dt()),
        bits(s.timescale.dt_reference),
        bits(s.timescale.omega_e),
        bits(s.timescale.period)
    ));
    out.push('\n');
    out.push_str(&format!(
        "k_env {}  omega_env {}  r_inner {}  mu {}",
        bits(s.timescale.k_env),
        bits(s.timescale.omega_env),
        bits(s.timescale.r_inner),
        bits(s.timescale.mu)
    ));
    out.push('\n');
    out.push_str(&format!("l0 {}  e_ref {}", bits(s.l0), bits(s.e_ref)));
    out.push('\n');

    for f in 0..frames {
        s.step_frame(substeps);
        // Every frame, not only the last: a divergence that appears and cancels is still a
        // divergence, and an end-state comparison cannot see one.
        let (px, py, pz) = s.momentum();
        out.push_str(&format!(
            "f{f} kin {} pair {} three {} wall {} spring {} wext {} E {} L {} drift {} peak {} bound {} eref {} erelmax {} kpair {} kthree {} px {} py {} pz {} pres {} bonded {} clusters {}",
            bits(s.e_kin),
            bits(s.e_pair),
            bits(s.e_three),
            bits(s.e_wall),
            bits(s.e_spring),
            bits(s.w_ext),
            bits(s.energy()),
            bits(s.ledger()),
            bits(s.drift()),
            bits(s.drift_peak),
            bits(s.drift_bound()),
            bits(s.e_ref),
            bits(s.e_rel_max),
            bits(s.k_pair_max()),
            bits(s.k_three_max()),
            bits(px),
            bits(py),
            bits(pz),
            bits(s.momentum_residual_peak),
            s.bonded_count(),
            s.cluster_count().0,
        ));
    out.push('\n');
    }

    // The full final state: positions, velocities, and every pair reading.
    for i in 0..s.n {
        let a = s.atoms[i];
        out.push_str(&format!(
            "atom{i} x {} y {} z {} vx {} vy {} vz {}",
            bits(a.x),
            bits(a.y),
            bits(a.z),
            bits(a.vx),
            bits(a.vy),
            bits(a.vz)
        ));
    out.push('\n');
    }
    for k in 0..s.pair_count {
        let p = s.pairs[k];
        out.push_str(&format!(
            "pair{k} i {} j {} r {} e_rel {} r_outer {} bonded {}",
            p.i,
            p.j,
            bits(p.r),
            bits(p.e_rel),
            bits(p.r_outer),
            u8::from(p.bonded)
        ));
    out.push('\n');
    }
    out.push_str(&format!("# end {name}"));
    out.push('\n');
}
