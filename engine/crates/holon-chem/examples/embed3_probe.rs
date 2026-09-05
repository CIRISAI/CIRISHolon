//! EMBED-3 diagnostic: ONE trimer-in-field solve at a given R_CD, reporting the solver's
//! exit, iteration count and residual — the fields the node JSON did not carry.
use holon_chem::density_embed::*;
use holon_chem::elements::by_symbol;
use holon_chem::embed::*;
use holon_chem::seam::hf_chain;
use std::time::Instant;
fn main() {
    let r_cd: f64 = std::env::args().nth(1).and_then(|x| x.parse().ok()).unwrap_or(8.0);
    let (f, h) = (by_symbol("F").unwrap(), by_symbol("H").unwrap());
    let mut frags = hf_chain(f, h, 1.879437929774, 5.0 * ANGSTROM_TO_BOHR, 3);
    frags.push(frags[0].translated([0.0, 0.0, (10.0 + r_cd) * ANGSTROM_TO_BOHR]));
    let t0 = Instant::now();
    let z = embed_densities(&frags, DensityStart::Zero);
    eprintln!("fixed point: {} sweeps, converged {}", z.sweeps, z.converged);
    let ds = subset_in_field(&frags, &z.densities, &[0, 1, 2], None);
    eprintln!("R_CD={r_cd}: E_ABC[ρ_D] = {:.12e}, exit {:?}, davidson_iters {}, residual {:.3e}, wall {:.0}s", ds.e_total, ds.sol.exit, ds.sol.davidson_iters, ds.sol.residual, t0.elapsed().as_secs_f64());
}
