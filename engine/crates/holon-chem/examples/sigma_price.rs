//! Price one sigma application against one Davidson iteration on a pair geometry:
//! `sigma_price <A> <B> <R bohr> [applies]`.
//!
//! The Davidson's per-iteration cost is a sigma application plus subspace work (Gram rows,
//! two deflations, a Ritz update, an m×m Jacobi); if the iteration costs many times the
//! sigma, the subspace work is where the seconds are. The instrument that measured the
//! (O,O) curve's 24 ms iterations at 2,025 determinants against the sigma alone.
use holon_chem::dual::D2;
use holon_chem::elements::by_symbol;
use holon_chem::fci::{ci_ints, Order};
use holon_chem::pair::geometry_problem;
use holon_chem::sigma_op::{CpuProvider, SigmaProvider};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let a = by_symbol(&args[1]).expect("species A");
    let b = by_symbol(&args[2]).expect("species B");
    let r: f64 = args[3].parse().expect("R in bohr");
    let applies: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(100);
    let (space, mo, _) = geometry_problem(
        &[a, b],
        vec![
            [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
            [D2::c(0.0), D2::c(0.0), D2::c(r)],
        ],
    );
    let ci0 = ci_ints(&mo, Order::Value);
    let diag = space.diagonal(&ci0);
    let provider = CpuProvider;
    let mut op = provider.op_for(&space, &ci0).expect("host operator");
    let n = space.n_det;
    let x: Vec<f64> = (0..n).map(|i| ((i * 7919) % 1000) as f64 / 1000.0 - 0.5).collect();
    let mut y = vec![0.0f64; n];
    op.apply(&x, &mut y);
    let t = Instant::now();
    for _ in 0..applies {
        op.apply(&x, &mut y);
    }
    let per_apply_ms = 1e3 * t.elapsed().as_secs_f64() / applies as f64;
    let t = Instant::now();
    let (e, _v, iters, resid, exit) =
        holon_chem::tier::davidson_eigh_from_op(op.as_mut(), &diag, 1e-10, 5000, None);
    let dav = t.elapsed().as_secs_f64();
    println!(
        "{}{} R={r} n_det {n} sigma {per_apply_ms:.3} ms/apply; davidson {dav:.3} s, {iters} it, \
         {:.3} ms/it, resid {resid:.1e}, {exit:?}, E {e:.10}; subspace overhead {:.1}x the sigma",
        a.symbol,
        b.symbol,
        1e3 * dav / iters.max(1) as f64,
        (1e3 * dav / iters.max(1) as f64) / per_apply_ms
    );
}
