//! Does the diffusion lens ever RETURN, on real data?
//!
//! A refusal that fires at every setting is not a reading, it is a dead branch wearing a
//! measurement's clothes — the M-VACUOUS-SUCCESS shape with the sign flipped. The lens
//! refuses when the fit window is wall-dominated, so the question is whether a shorter
//! window exists on which it reports, and where the boundary sits.

use holon_lens::lens;
use holon_lens::traj::Trajectory;

fn main() {
    for p in std::env::args().skip(1) {
        let t = Trajectory::read(std::path::Path::new(&p)).expect("readable");
        println!("# {}", p.rsplit('/').next().unwrap());
        let mut last_ok = None;
        let mut first_refusal = None;
        for lag in [10usize, 20, 50, 100, 200, 500, 1000, 2000] {
            if lag >= t.frames.len() {
                break;
            }
            match lens::diffusion(&t, lag) {
                Ok(d) => {
                    println!(
                        "#   lag {lag:>5} ({:>8.1} fs): D = {d:.6} bohr^2/fs   MSD = {:.3}   alpha = {:.2}",
                        lens::mean_lag_fs(&t, lag),
                        lens::msd(&t, lag),
                        lens::msd_exponent(&t, lag)
                    );
                    last_ok = Some(lag);
                }
                Err(e) => {
                    if first_refusal.is_none() {
                        first_refusal = Some(lag);
                    }
                    println!(
                        "#   lag {lag:>5}: REFUSED [{}] (alpha = {:.2}) {}",
                        e.gate,
                        lens::msd_exponent(&t, lag),
                        e.reason
                    );
                }
            }
        }
        println!(
            "#   => reports up to lag {:?}, refuses from lag {:?}",
            last_ok, first_refusal
        );
    }
}
