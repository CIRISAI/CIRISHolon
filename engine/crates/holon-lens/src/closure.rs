//! The H-bond network's phase, through `holon-closure`'s one spanning rule.
//!
//! # What it is
//!
//! LIQUID-1 counted the bonds and read the shell; it did NOT read the spanning cluster, and
//! its own row says so — *"the spanning-cluster fraction is the readout that names it and is
//! OWED (not in LIQUID-1's runner)"*. This module is that readout, built on
//! [`hbonds_periodic`], which is untouched, and on `largest_domain`, which is also
//! untouched and is used here as the check on the shared routine rather than replaced by it.
//!
//! # What it declares
//!
//! * **A node is an oxygen**, in ascending atom order, and the map back to the atom index is
//!   [`oxygen_index`]. One oxygen is one water unit at this tier, which is `holon-render`'s
//!   `units_reading` convention: a unit is named by its oxygen.
//! * **An edge is one hydrogen bond**, donor oxygen to acceptor oxygen, with the INTEGER
//!   wrap count between them as its displacement.
//!
//!   The integer is not a convenience. Around any cycle the raw coordinate differences
//!   telescope to exactly zero, so the summed minimum-image displacement is `−L` times the
//!   summed wrap count; the cycle winds iff that integer sum is nonzero. Testing the
//!   floating-point minimum-image sum against zero instead would make "does the cluster
//!   span" an equality test on rounding error, and on a hundred-thousand-frame arm it would
//!   answer differently on different frames of the same configuration.
//! * **Spanning is a WINDING**, the same rule `holon-lattice::orientation` measured and for
//!   the same reason: the "touches both faces of the box" reading is not invariant under
//!   translating the configuration, and a gate that moves when the scene is shifted is not a
//!   gate.
//!
//! # What it refuses
//!
//! * **It refuses exactly where [`hbonds_periodic`] refuses** and nowhere else — a scene
//!   with no oxygen or no hydrogen has no hydrogen-bond variable, and a zero there would
//!   read as a measured absence. The refusal is propagated, not caught.
//! * **It names no phase.** The reading is a fraction and a winding. "A spanning bond graph
//!   is a liquid" is LIQUID-2's threshold on its own carrier and is not a library's to make.
//! * **It does not replace `largest_domain`.** That routine is a size-only union–find over
//!   an explicit edge set, it is what the closure census calls, and it stays. Here it is the
//!   independent check: [`hbond_phase`]'s `largest` must equal it on the same edge set.

use crate::lens::{hbonds_periodic, HBond, Reading};
use holon_closure::{phase, Phase, PhaseEdge};

/// The oxygens of a scene, ascending — the node list the phase reading is over.
pub fn oxygen_index(z: &[u32]) -> Vec<usize> {
    (0..z.len()).filter(|&i| z[i] == 8).collect()
}

/// The wrap count from `a` to `b` on each axis: how many box lengths the minimum image
/// crossed. An axis whose edge is not a finite positive length is left at zero, which is
/// the open-axis case stated rather than assumed — the same rule `lens::min_image` follows.
fn wraps(a: [f64; 3], b: [f64; 3], cell: [f64; 3]) -> [i32; 3] {
    let mut n = [0i32; 3];
    for k in 0..3 {
        let l = cell[k];
        if l.is_finite() && l > 0.0 {
            n[k] = ((b[k] - a[k]) / l).round() as i32;
        }
    }
    n
}

/// The H-bond census as phase edges over the oxygens.
pub fn hbond_edges(
    bonds: &[HBond],
    pos: &[[f64; 3]],
    oxygens: &[usize],
    cell: [f64; 3],
) -> Vec<PhaseEdge<3>> {
    let id = |atom: usize| -> Option<u32> { oxygens.binary_search(&atom).ok().map(|i| i as u32) };
    bonds
        .iter()
        .filter_map(|b| {
            Some(PhaseEdge {
                a: id(b.donor_o)?,
                b: id(b.acceptor_o)?,
                displacement: wraps(pos[b.donor_o], pos[b.acceptor_o], cell),
            })
        })
        .collect()
}

/// The spanning-cluster reading LIQUID-2 owes: the largest H-bond component as a fraction of
/// the waters, and whether it winds the periodic box.
///
/// Refuses exactly where [`hbonds_periodic`] refuses.
pub fn hbond_phase(pos: &[[f64; 3]], z: &[u32], cell: [f64; 3]) -> Reading<Phase<3>> {
    let bonds = hbonds_periodic(pos, z, cell)?;
    let oxygens = oxygen_index(z);
    let edges = hbond_edges(&bonds, pos, &oxygens, cell);
    Ok(phase(oxygens.len(), &edges))
}

/// The same reading, and `largest_domain`'s own answer beside it.
///
/// The two routines are independent — one is a size-only union–find over an explicit edge
/// set, the other carries a displacement per node — and the whole reason to report both is
/// that they must agree on the size. Returns `(phase, largest_domain)`.
pub fn hbond_phase_checked(
    pos: &[[f64; 3]],
    z: &[u32],
    cell: [f64; 3],
) -> Reading<(Phase<3>, usize)> {
    let bonds = hbonds_periodic(pos, z, cell)?;
    let oxygens = oxygen_index(z);
    let edges = hbond_edges(&bonds, pos, &oxygens, cell);
    let plain: Vec<(usize, usize)> =
        edges.iter().map(|e| (e.a as usize, e.b as usize)).collect();
    Ok((phase(oxygens.len(), &edges), crate::lens::largest_domain(oxygens.len(), &plain)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lens::{largest_domain, HB_ANGLE_DEG};

    /// A ring of `n` waters around one axis of the box, each donating to the next. The
    /// geometry is built to satisfy the Luzar–Chandler criterion the census uses, and the
    /// last bond crosses the face — so the cluster winds by construction.
    fn ring(n: usize, l: f64) -> (Vec<[f64; 3]>, Vec<u32>) {
        let step = l / n as f64;
        assert!(step < 6.0, "the ring's spacing must be inside the O...O criterion");
        let mut pos = Vec::new();
        let mut z = Vec::new();
        for i in 0..n {
            let x = i as f64 * step;
            // O at x; its donor H a short way along +x toward the next O; the second H off
            // the axis so the unit is a water and not a line.
            pos.push([x, 0.0, 0.0]);
            z.push(8);
            pos.push([x + 1.81, 0.0, 0.0]);
            z.push(1);
            pos.push([x - 0.6, 1.71, 0.0]);
            z.push(1);
        }
        (pos, z)
    }

    #[test]
    fn a_ring_of_waters_around_the_box_reads_as_spanning() {
        let l = 20.0;
        let (pos, z) = ring(5, l);
        let cell = [l, l, l];
        let p = hbond_phase(&pos, &z, cell).expect("the scene has oxygens and hydrogens");
        assert_eq!(p.nodes, 5);
        assert_eq!(p.largest, 5, "the ring is one component");
        assert_eq!(p.largest_fraction, 1.0);
        assert!(p.largest_spans, "a ring closed around the box does not read as spanning");
        assert_eq!(p.largest_winds, [true, false, false], "it winds on x and nothing else");
    }

    /// The independent check: the shared routine's largest component is `largest_domain`'s.
    #[test]
    fn the_shared_phase_and_largest_domain_agree_on_the_size() {
        let l = 20.0;
        let (pos, z) = ring(5, l);
        let (p, ld) = hbond_phase_checked(&pos, &z, [l, l, l]).unwrap();
        assert_eq!(p.largest, ld);
        // and `largest_domain` itself is unchanged: called directly, on the same edges
        let bonds = crate::lens::hbonds_periodic(&pos, &z, [l, l, l]).unwrap();
        let ox = oxygen_index(&z);
        let edges: Vec<(usize, usize)> = hbond_edges(&bonds, &pos, &ox, [l, l, l])
            .iter()
            .map(|e| (e.a as usize, e.b as usize))
            .collect();
        assert_eq!(largest_domain(ox.len(), &edges), ld);
    }

    /// A ring that does NOT close around the box is one component and does not wind: the
    /// discriminator between "big" and "spanning".
    #[test]
    fn a_chain_that_does_not_close_is_large_and_does_not_span() {
        // the same ring, in a box twice as long: the last O...O separation is now 10 bohr,
        // outside the criterion, so the closing bond is absent and the chain is open.
        let (pos, z) = ring(5, 20.0);
        let cell = [40.0, 40.0, 40.0];
        let p = hbond_phase(&pos, &z, cell).unwrap();
        assert_eq!(p.nodes, 5);
        assert_eq!(p.largest, 5, "still one chain");
        assert!(!p.largest_spans, "an open chain read as spanning");
        assert!(!p.any_spans);
    }

    /// The census's refusal is propagated and never turned into a zero.
    #[test]
    fn a_scene_with_no_hydrogen_refuses_where_the_census_refuses() {
        let pos = vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0]];
        let z = vec![8, 8];
        assert!(hbond_phase(&pos, &z, [20.0, 20.0, 20.0]).is_err());
        assert!(hbond_phase_checked(&pos, &z, [20.0, 20.0, 20.0]).is_err());
        // and exactly where the census does
        assert!(crate::lens::hbonds_periodic(&pos, &z, [20.0, 20.0, 20.0]).is_err());
        let _ = HB_ANGLE_DEG;
    }
}
