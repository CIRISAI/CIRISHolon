//! Probe the identity the two implementations rest on, at bond 0:
//! Z·B† must equal H_eff^{1site} M, and then ‖Z‖²−‖A†Z‖² = one-site + two-site.
use q8_mps::mps::{self, Env};
use q8_mps::qcd2::Qcd2;
use q8_mps::symmetric::{random_start, SymConfig};

fn main() {
    let (n, b, chi) = (4usize, 0i32, 16usize);
    let q = Qcd2::new(n, 4.0);
    let n_q = q.quarks(b);
    let sector = q.sector(n_q).unwrap();
    let mpo = { let mut u = Qcd2::new(n, 4.0); u.lam = 0.0; u.mpo(n_q) };
    let cfg = SymConfig::amendment(chi, 40);
    let (r, _labels) = q.ground_energy_sym_from(&[], n_q, &cfg, Some(random_start(&sector, 256, 7))).unwrap();
    let t = r.tensors;
    let l = t.len();
    let mut rights: Vec<Env> = vec![mps::trivial_right_env_mpo(mpo.sites[l - 1].d_r); l + 1];
    for k in (0..l).rev() { rights[k] = mps::grow_right_mpo(&rights[k + 1], &mpo.sites[k], &t[k]); }
    let lefts = mps::trivial_left_env_mpo(mpo.sites[0].d_l);
    println!("norm^2 = {:.12}", q8_mps::observables::norm_squared(&t));
    println!("<H> = {:.12}  (sweep said {:.12})", q8_mps::variance::expectation_mpo(&t, &mpo), r.energy);
    // right-canonicality of tensors 1..: identity defect of R[FINISH]
    for j in 1..l.min(4) {
        println!("  right env {j} identity defect on FINISH channel: {:.3e}", mps::identity_defect(&rights[j], mpo.sites[j].d_r - 1));
    }
    println!("  left env identity defect on START: {:.3e}", mps::identity_defect(&lefts, 0));
    // bond 0
    let (chi_l, chi_r) = (t[0].chi_l, t[1].chi_r);
    let mid = t[0].chi_r;
    println!("bond 0: chi_l {chi_l} mid {mid} chi_r {chi_r}");
    let cols = 2 * chi_r;
    let mut psi_lab = vec![0.0; chi_l * 4 * chi_r];
    for lft in 0..chi_l { for a in 0..2 { for m in 0..mid {
        let av = t[0].get(a, lft, m); if av == 0.0 { continue; }
        for bb in 0..2 { for rr in 0..chi_r {
            psi_lab[((lft * 2 + a) * 2 + bb) * chi_r + rr] += av * t[1].get(bb, m, rr);
        }}}}}
    let z = mps::apply_effective_h_mpo(&lefts, &mpo.sites[0], &mpo.sites[1], &rights[2], &psi_lab, chi_l, chi_r);
    let mut z2 = vec![0.0; 2 * chi_l * cols];
    for lft in 0..chi_l { for s in 0..2 { for tt in 0..2 { for rr in 0..chi_r {
        z2[(s * chi_l + lft) * cols + tt * chi_r + rr] = z[((lft * 2 + s) * 2 + tt) * chi_r + rr];
    }}}}
    // Z B^dagger
    let mut zb = vec![0.0; 2 * chi_l * mid];
    for row in 0..2 * chi_l { for s in 0..2 { for rr in 0..chi_r {
        let zv = z2[row * cols + s * chi_r + rr]; if zv == 0.0 { continue; }
        for m in 0..mid { zb[row * mid + m] += zv * t[1].get(s, m, rr); }
    }}}
    let n2 = |v: &[f64]| v.iter().map(|x| x * x).sum::<f64>();
    println!("||Z||^2      = {:.9e}", n2(&z2));
    println!("||Z B†||^2   = {:.9e}", n2(&zb));
    // psi norm and <psi|Z> = E
    let mut psi_rows = vec![0.0; 2 * chi_l * cols];
    for lft in 0..chi_l { for s in 0..2 { for tt in 0..2 { for rr in 0..chi_r {
        psi_rows[(s * chi_l + lft) * cols + tt * chi_r + rr] = psi_lab[((lft * 2 + s) * 2 + tt) * chi_r + rr];
    }}}}
    let dot: f64 = psi_rows.iter().zip(&z2).map(|(a, b)| a * b).sum();
    println!("||Psi||^2 = {:.9e}, <Psi|Z> = {:.9e}", n2(&psi_rows), dot);
}
