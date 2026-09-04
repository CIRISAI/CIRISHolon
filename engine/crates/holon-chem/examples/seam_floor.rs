//! SEAM-1 AMENDMENT 1 — the arithmetic floor of `r_emb`, measured by a translation null.
//!
//! ```text
//! cargo run --release -p holon-chem --example seam_floor -- [OUT_DIR]
//! ```
//!
//! The physics is translation-invariant and the arithmetic is not: every one of the seven
//! solves behind `r_emb` (the trimer, three dimers, three monomers) is repeated with the
//! whole chain moved by a fixed off-axis vector, the self-consistent charges re-derived on
//! the moved geometry, and `|r_emb − r_emb(moved)|` is the node's floor. A far node is
//! posable for the monotonicity clause only where `|r_emb| ≥ 10 × floor`.
use holon_chem::elements::{by_symbol, Species};
use holon_chem::embed::*;
use holon_chem::seam::*;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

fn sp(s: &str) -> Species {
    by_symbol(s).expect("species")
}

const R_HF: f64 = 1.879437929774;
const SHIFT: [f64; 3] = [0.37, 0.21, 0.5];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = PathBuf::from(args.get(1).cloned().unwrap_or_else(|| "../conformance/water_observatory/seam".to_string()));
    let (f, h) = (sp("F"), sp("H"));
    for r_ang in [5.0, 6.0, 8.0] {
        let path = out.join(format!("floor_R{r_ang:.1}.json"));
        if path.exists() {
            eprintln!("  R={r_ang:.1} Å: exists, skipped");
            continue;
        }
        let t0 = Instant::now();
        let base = hf_chain(f, h, R_HF, r_ang * ANGSTROM_TO_BOHR, 3);
        let moved: Vec<Fragment> = base.iter().map(|fr| fr.translated(SHIFT)).collect();
        let mut rec = Vec::new();
        for (tag, frags) in [("base", &base), ("moved", &moved)] {
            let sm = supermolecule_all(frags);
            let z = embed_many(frags, ChargeModel::DipoleExact, Start::Zero);
            let ee = ee_pa(frags, &z.charges);
            let bare = bare_pa(frags);
            rec.push((tag, sm.e_total, ee.total, bare.total, sm.e_total - ee.total, sm.e_total - bare.total, z.iterations, sm.sol.residual));
        }
        let (b, m) = (&rec[0], &rec[1]);
        let floor = (b.4 - m.4).abs();
        let json = format!(
            "{{\n  \"r_angstrom\": {r_ang:.3}, \"shift_bohr\": [{}, {}, {}],\n  \"base\":  {{\"e_super\": {:.12e}, \"ee_pa\": {:.12e}, \"bare_pa\": {:.12e}, \"r_emb\": {:.12e}, \"de3_bare\": {:.12e}, \"sweeps\": {}, \"residual\": {:.3e}}},\n  \"moved\": {{\"e_super\": {:.12e}, \"ee_pa\": {:.12e}, \"bare_pa\": {:.12e}, \"r_emb\": {:.12e}, \"de3_bare\": {:.12e}, \"sweeps\": {}, \"residual\": {:.3e}}},\n  \"floor_r_emb\": {:.6e}, \"floor_de3\": {:.6e}, \"floor_e_super\": {:.6e}, \"posable_ratio\": {:.3}, \"wall_seconds\": {:.1}\n}}\n",
            SHIFT[0], SHIFT[1], SHIFT[2],
            b.1, b.2, b.3, b.4, b.5, b.6, b.7, m.1, m.2, m.3, m.4, m.5, m.6, m.7,
            floor, (b.5 - m.5).abs(), (b.1 - m.1).abs(), if floor > 0.0 { b.4.abs() / floor } else { f64::INFINITY }, t0.elapsed().as_secs_f64()
        );
        fs::write(&path, json).expect("write");
        eprintln!("  R={r_ang:.1} Å: r_emb {:.3e} vs moved {:.3e} → floor {:.3e} (|r_emb|/floor = {:.1}); ΔE3 floor {:.1e}; E_super floor {:.1e}; wall {:.0}s",
            b.4, m.4, floor, if floor > 0.0 { b.4.abs() / floor } else { f64::INFINITY }, (b.5 - m.5).abs(), (b.1 - m.1).abs(), t0.elapsed().as_secs_f64());
    }
}
