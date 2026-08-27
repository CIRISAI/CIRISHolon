//! S2 arm of COMPOSITION_PREREG.md — region time series + the K measurement.
//!
//! Arms, per the freeze:
//!   N — two INDEPENDENT sessions (different throws): product dynamics by
//!       construction; law 2 predicts both cross-defects at floor.
//!   I — left/right halves of ONE thrown session, coupled through contacts:
//!       law 2 predicts cross-defects above floor.
//!   K — twin deterministic sessions; one takes a second, tiny throw at frame
//!       240; the settled-grain coarse divergence's per-step growth is K
//!       (Aggregation's non-expansiveness), excluding nodes added after the
//!       perturbation so the probe itself is not counted as response.
//!
//! Output: CSVs of per-frame region aggregates. Analysis and adjudication live
//! in Python against the frozen model; this file only measures.
use holon_sandbox::sim::Session;
use holon_sandbox::tier::TierId;
use std::fmt::Write as _;

const DT: f64 = 1.0 / 60.0;
const FRAMES: usize = 24000;
const K_PERTURB_FRAME: usize = 240;

fn aggregates(s: &Session, lo_x: f64, hi_x: f64, max_node: usize) -> (f64, f64, usize) {
    let n = s.nodes();
    let (mut px, mut spd, mut cnt) = (0.0, 0.0, 0usize);
    for i in 0..n.position.len().min(max_node) {
        let x = n.position[i][0];
        if x >= lo_x && x < hi_x {
            px += n.mass_kg[i] * n.velocity[i][0];
            spd += (n.velocity[i][0].powi(2) + n.velocity[i][1].powi(2)).sqrt();
            cnt += 1;
        }
    }
    (px, if cnt > 0 { spd / cnt as f64 } else { 0.0 }, cnt)
}

fn main() {
    let out_dir = "../../../scratchpad/composition/s2";
    std::fs::create_dir_all(out_dir).unwrap();
    let tier = TierId::Sandbox;

    // Domain width for the half-split: read from a fresh session's node spread.
    let probe = Session::new(tier);
    let xs: Vec<f64> = probe.nodes().position.iter().map(|p| p[0]).collect();
    let (xmin, xmax) = (xs.iter().cloned().fold(f64::MAX, f64::min),
                       xs.iter().cloned().fold(f64::MIN, f64::max));
    let mid = 0.5 * (xmin + xmax);
    eprintln!("domain x: [{xmin:.3}, {xmax:.3}] mid={mid:.3} nodes={}", xs.len());

    // ---- Arms N and I ----
    let mut s1 = Session::new(tier);
    let mut s2 = Session::new(tier);
    s1.throw(mid - 0.2 * (xmax - xmin), 0.8, 0.5);
    s2.throw(mid + 0.25 * (xmax - xmin), 0.7, 0.6); // different throw: independent dynamics
    let mut csv = String::from("frame,n1_px,n1_spd,n2_px,n2_spd,i_l_px,i_l_spd,i_r_px,i_r_spd\n");
    for f in 0..FRAMES {
        s1.step(DT);
        s2.step(DT);
        let (a_px, a_sp, _) = aggregates(&s1, f64::MIN, f64::MAX, usize::MAX);
        let (b_px, b_sp, _) = aggregates(&s2, f64::MIN, f64::MAX, usize::MAX);
        let (l_px, l_sp, _) = aggregates(&s1, f64::MIN, mid, usize::MAX);
        let (r_px, r_sp, _) = aggregates(&s1, mid, f64::MAX, usize::MAX);
        writeln!(csv, "{f},{a_px:.9e},{a_sp:.9e},{b_px:.9e},{b_sp:.9e},{l_px:.9e},{l_sp:.9e},{r_px:.9e},{r_sp:.9e}").unwrap();
    }
    std::fs::write(format!("{out_dir}/arms_NI.csv"), csv).unwrap();
    eprintln!("arms N/I written");

    // ---- OMEGA-KILL B1: SHAM twins — no probe at all; divergence must be
    // EXACTLY zero at every frame (deterministic engine, geometry-seeded). ----
    {
        let mut sa = Session::new(tier);
        let mut sb = Session::new(tier);
        sa.throw(mid, 0.8, 0.5);
        sb.throw(mid, 0.8, 0.5);
        let mut sham = String::from("frame,div_pos,div_px\n");
        for f in 0..1200 {
            sa.step(DT);
            sb.step(DT);
            let na = sa.nodes();
            let nb = sb.nodes();
            let m = na.position.len().min(nb.position.len());
            let (mut dpx, mut dpos) = (0.0, 0.0);
            for i in 0..m {
                dpx += (na.mass_kg[i] * na.velocity[i][0] - nb.mass_kg[i] * nb.velocity[i][0]).abs();
                dpos += ((na.position[i][0] - nb.position[i][0]).powi(2)
                       + (na.position[i][1] - nb.position[i][1]).powi(2)).sqrt();
            }
            writeln!(sham, "{f},{dpos:.9e},{dpx:.9e}").unwrap();
        }
        std::fs::write(format!("{out_dir}/arm_sham.csv"), sham).unwrap();
        eprintln!("sham arm written");
    }

    // ---- Arm K: deterministic twins, tiny second throw on B ----
    let mut ka = Session::new(tier);
    let mut kb = Session::new(tier);
    ka.throw(mid, 0.8, 0.5);
    kb.throw(mid, 0.8, 0.5);
    // OMEGA-KILL-2 B3': INDEX-FREE sector aggregates (D-IDENT mitigation) —
    // per-sector kinetic energy sums for BOTH sessions, every frame including
    // pre-probe. Sums over regions are invariant under node renumbering, so
    // certify_at cannot poison them.
    let mut acsv = String::from("frame,keL_a,keR_a,keL_b,keR_b\n");
    let ke_sectors = |s: &Session| {
        let n = s.nodes();
        let (mut kl, mut kr) = (0.0, 0.0);
        for i in 0..n.position.len() {
            let ke = 0.5 * n.mass_kg[i] * (n.velocity[i][0].powi(2) + n.velocity[i][1].powi(2));
            if n.position[i][0] < mid { kl += ke } else { kr += ke }
        }
        (kl, kr)
    };
    let mut kcsv = String::from("frame,div_px,div_pos,div_pos_l,div_pos_r\n");
    let mut base_nodes = 0usize;
    for f in 0..FRAMES {
        ka.step(DT);
        kb.step(DT);
        {
            let (la, ra) = ke_sectors(&ka);
            let (lb, rb) = ke_sectors(&kb);
            writeln!(acsv, "{f},{la:.9e},{ra:.9e},{lb:.9e},{rb:.9e}").unwrap();
        }
        if f == K_PERTURB_FRAME {
            base_nodes = ka.nodes().position.len().min(kb.nodes().position.len());
            // OMEGA-KILL B3: the probe lands in the LEFT sector
            kb.throw(mid - 0.2 * (xmax - xmin), 0.9, 0.05); // tiny probe, LEFT
        }
        // OMEGA-KILL B2: record the PRE-probe window too (must be exactly zero)
        if f < K_PERTURB_FRAME {
            let na = ka.nodes();
            let nb = kb.nodes();
            let m = na.position.len().min(nb.position.len());
            let (mut dpx, mut dpos) = (0.0, 0.0);
            for i in 0..m {
                dpx += (na.mass_kg[i] * na.velocity[i][0] - nb.mass_kg[i] * nb.velocity[i][0]).abs();
                dpos += ((na.position[i][0] - nb.position[i][0]).powi(2)
                       + (na.position[i][1] - nb.position[i][1]).powi(2)).sqrt();
            }
            writeln!(kcsv, "{f},{dpx:.9e},{dpos:.9e},0,0").unwrap();
        }
        if f >= K_PERTURB_FRAME {
            let na = ka.nodes();
            let nb = kb.nodes();
            let m = base_nodes.min(na.position.len()).min(nb.position.len());
            let (mut dpx, mut dpos, mut dl, mut dr) = (0.0, 0.0, 0.0, 0.0);
            for i in 0..m {
                let d = ((na.position[i][0] - nb.position[i][0]).powi(2)
                       + (na.position[i][1] - nb.position[i][1]).powi(2)).sqrt();
                dpx += (na.mass_kg[i] * na.velocity[i][0] - nb.mass_kg[i] * nb.velocity[i][0]).abs();
                dpos += d;
                if na.position[i][0] < mid { dl += d } else { dr += d }
            }
            writeln!(kcsv, "{f},{dpx:.9e},{dpos:.9e},{dl:.9e},{dr:.9e}").unwrap();
        }
    }
    std::fs::write(format!("{out_dir}/arm_K.csv"), kcsv).unwrap();
    std::fs::write(format!("{out_dir}/arm_agg.csv"), acsv).unwrap();
    eprintln!("arm K + agg written");
}
