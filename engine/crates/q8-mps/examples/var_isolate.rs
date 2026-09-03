//! Isolate the labelled two-site variance's disagreement: per bond, compare the dense
//! route's (one-site + two-site) against ||Z||^2 - ||A'Z||^2 for BOTH choices of A, and
//! check the block-sparse operator against the dense one on the actual psi.
use q8_mps::mps::{self, Env, TensorSite};
use q8_mps::qcd2::Qcd2;
use q8_mps::symmetric::{random_start, split_two_site_sym, SymConfig};

fn n2(v: &[f64]) -> f64 { v.iter().map(|x| x * x).sum() }
fn adag_z(a: &TensorSite, z: &[f64], n: usize) -> Vec<f64> {
    let (cl, cr) = (a.chi_l, a.chi_r);
    let mut out = vec![0.0; cr * n];
    for s in 0..2 { for l in 0..cl {
        let zr = &z[(s * cl + l) * n..(s * cl + l) * n + n];
        for r in 0..cr { let av = a.get(s, l, r); if av == 0.0 { continue; }
            for (o, zv) in out[r * n..r * n + n].iter_mut().zip(zr) { *o += av * zv; } }
    }}
    out
}

fn main() {
    let (n, b, chi) = (4usize, 0i32, 16usize);
    let q = Qcd2::new(n, 4.0);
    let n_q = q.quarks(b);
    let sector = q.sector(n_q).unwrap();
    let mpo = { let mut u = Qcd2::new(n, 4.0); u.lam = 0.0; u.mpo(n_q) };
    let cfg = SymConfig::amendment(chi, 40);
    let (r, labels) = q.ground_energy_sym_from(&[], n_q, &cfg, Some(random_start(&sector, 256, 7))).unwrap();
    let l = r.tensors.len();
    let mut rights: Vec<Env> = vec![mps::trivial_right_env_mpo(mpo.sites[l - 1].d_r); l + 1];
    for k in (0..l).rev() { rights[k] = mps::grow_right_mpo(&rights[k + 1], &mpo.sites[k], &r.tensors[k]); }
    let mut lefts = mps::trivial_left_env_mpo(mpo.sites[0].d_l);
    let mut ts = r.tensors.clone();
    let mut qs = labels.clone();
    println!("{:>4} {:>10} {:>12} {:>12} {:>12} {:>10}", "bond", "chi_l/mid", "||Z||^2-||A'Z||^2", "A cols(split)", "plan-vs-dense", "psi off-label");
    for j in 0..(l - 1) {
        let (cl, cr) = (ts[j].chi_l, ts[j + 1].chi_r);
        let mid = ts[j].chi_r;
        let mut psi = vec![0.0; cl * 4 * cr];
        for lf in 0..cl { for a in 0..2 { for m in 0..mid {
            let av = ts[j].get(a, lf, m); if av == 0.0 { continue; }
            for bb in 0..2 { for rr in 0..cr { psi[((lf * 2 + a) * 2 + bb) * cr + rr] += av * ts[j + 1].get(bb, m, rr); } }
        }}}
        let (e1, e2) = (sector.site_charge[j], sector.site_charge[j + 1]);
        // is psi label-consistent?
        let mut off = 0.0f64;
        for (li, &ql) in qs[j].iter().enumerate() { for a in 0..2 {
            let qa = if a == 1 { q8_mps::symmetric::charge_add(ql, e1) } else { ql };
            for bb in 0..2 { let qab = if bb == 1 { q8_mps::symmetric::charge_add(qa, e2) } else { qa };
                for (ri, &qr) in qs[j + 2].iter().enumerate() {
                    if qr != qab { off = off.max(psi[((li * 2 + a) * 2 + bb) * cr + ri].abs()); } } }
        }}
        let zd = mps::apply_effective_h_mpo(&lefts, &mpo.sites[j], &mpo.sites[j + 1], &rights[j + 2], &psi, cl, cr);
        let plan = q8_mps::blocks::BlockPlan::build(&qs[j], &qs[j + 2], e1, e2, &lefts, &rights[j + 2]).unwrap();
        let zp = plan.apply(&lefts, &mpo.sites[j], &mpo.sites[j + 1], &rights[j + 2], &psi);
        let dz: f64 = zd.iter().zip(&zp).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max);
        let cols = 2 * cr;
        let mut zr = vec![0.0; 2 * cl * cols];
        for lf in 0..cl { for s in 0..2 { for t in 0..2 { for rr in 0..cr {
            zr[(s * cl + lf) * cols + t * cr + rr] = zd[((lf * 2 + s) * 2 + t) * cr + rr]; }}}}
        let full = 4 * cl.max(cr) + 8;
        let (a, next, _, _, nq, _) = split_two_site_sym(&psi, &qs[j], &qs[j + 2], e1, e2, full, false).unwrap();
        let term = n2(&zr) - n2(&adag_z(&a, &zr, cols));
        println!("{j:>4} {:>10} {term:>12.5e} {:>12} {dz:>12.2e} {off:>10.1e}", format!("{cl}/{mid}"), a.chi_r);
        lefts = mps::grow_left_mpo(&lefts, &mpo.sites[j], &a);
        qs[j + 1] = nq;
        ts[j] = a;
        ts[j + 1] = next;
    }
}
