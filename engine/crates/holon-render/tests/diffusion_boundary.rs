//! THE ONE SEAM between the engine's boundary and the lens's: a total map, checked.
//!
//! `holon-lens` has no path to `holon-render` (its manifest says why at length), so
//! `lens::LensBoundary` is a COPY of `sim::Boundary`'s three cases and the conversion lives
//! at the call site. A copy that drifts is worse than a dependency, so the map is written
//! once, here, in a crate that can see both types, and it is a `match` with no wildcard:
//! the day the engine grows a fourth boundary this file stops compiling, which is exactly
//! when somebody should be asked what the diffusion lens does about it.
//!
//! `examples/liquid2.rs` calls [`lens_boundary`] and never writes the mapping itself.

use holon_lens::lens::{diffusion_periodic, LagWindow, LensBoundary};
use holon_lens::synthetic::{self, Spec};
use holon_lens::traj::BondSet;
use holon_render::sim::Boundary;

/// The engine's boundary as the lens sees it. Total, and deliberately without a wildcard.
pub fn lens_boundary(b: Boundary) -> LensBoundary {
    match b {
        Boundary::Walls => LensBoundary::Walls,
        Boundary::Open => LensBoundary::Open,
        Boundary::Periodic => LensBoundary::Periodic,
    }
}

#[test]
fn every_engine_boundary_maps_to_exactly_one_lens_boundary() {
    assert_eq!(lens_boundary(Boundary::Walls), LensBoundary::Walls);
    assert_eq!(lens_boundary(Boundary::Open), LensBoundary::Open);
    assert_eq!(lens_boundary(Boundary::Periodic), LensBoundary::Periodic);
    // and the two predicates the engine exposes agree with the lens's own reading of them
    assert!(Boundary::Walls.has_walls() && !Boundary::Periodic.has_walls());
    assert!(Boundary::Periodic.wraps() && !Boundary::Walls.wraps());
}

/// THE REVIEW'S CLAIM, at the seam: the wall cap is a statement about `Boundary::Walls`,
/// and told the truth about the boundary the lens now reads the very trajectory it used to
/// refuse.
#[test]
fn the_wall_cap_follows_the_engine_s_boundary_and_not_the_data() {
    // a walk whose displacement runs well past (L/4)^2 in a 30-bohr cell
    let (l, s, n_frames) = (30.0f64, 0.30f64, 1200usize);
    let mut sp = Spec::quench_like(n_frames, vec![8; 32]);
    sp.dims = 3;
    sp.dt = 1.0;
    sp.substeps = 10;
    sp.box_w = l;
    sp.box_h = l;
    sp.box_d = l;
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    let mut unit = move || {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((state >> 11) as f64) / ((1u64 << 53) as f64)
    };
    let traj = synthetic::build(sp, move |t, pos, _v| {
        for p in pos.iter_mut() {
            if t == 0 {
                *p = [0.5 * l, 0.5 * l, 0.5 * l];
            } else {
                for c in 0..3 {
                    let u = unit().max(1e-12);
                    let v = unit();
                    p[c] += s * (-2.0 * u.ln()).sqrt() * (std::f64::consts::TAU * v).cos();
                }
            }
        }
        BondSet::empty()
    });
    let win = LagWindow::new(20, 400);
    assert!(
        diffusion_periodic(&traj, win, lens_boundary(Boundary::Walls)).is_err(),
        "under Walls the cap must still refuse"
    );
    let r = diffusion_periodic(&traj, win, lens_boundary(Boundary::Periodic))
        .expect("under Periodic there is no wall to saturate against");
    assert!(r.d_bohr2_per_fs > 0.0);
    assert!(!r.finite_size.applied, "and finite size is reported, not applied");
    println!(
        "the same 1,200 frames: Walls -> refused by the cap; Periodic -> D = {:.6e} bohr^2/fs, \
         alpha = {:.3}, box edge {:.3} bohr, Yeh-Hummer term/viscosity {:.6e} (NOT applied)",
        r.d_bohr2_per_fs,
        r.alpha,
        r.finite_size.box_edge_bohr,
        r.finite_size.yeh_hummer_over_viscosity.unwrap()
    );
}
