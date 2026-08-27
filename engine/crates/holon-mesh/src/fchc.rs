//! The 3D mode set, enumerated rather than asserted.
//!
//! `MESH_DESIGN.md` §2.1 picks **FCHC-24** as the 3D REG+ chart and quotes three numbers for
//! it: 16,777,216 local states, **72,047** `(N, P)` sectors, largest sector dimension 11,740.
//! §8's M-G3 says those stay **engine-checked**, because `2^24` is past the Lean kernel's
//! reach by `decide` at this project's discipline and `native_decide` is not house style.
//!
//! This module is that check. Until it existed the claim was carried by a throwaway script,
//! which is the same defect as a constant that no test derives — `tier.rs`'s `REG_PLUS_MAX`
//! declared a momentum maximum of 3 for exactly that reason, and the enumeration refuted it.
//!
//! # The control is the point
//!
//! The same enumerator is run over the **FHP-6** directions of `ciris_sim_core::regplus`, and
//! must return **53 sectors with dimension histogram 44 / 7 / 2** — the object
//! `CIRISOntology/Core/Lattice.lean` proves and `regplus.rs`'s own test already pins. An
//! instrument that reproduces the known 2D answer is one you can believe about the unknown 3D
//! one; an instrument checked only against its own output is not.
//!
//! # Why FCHC and not something cheaper
//!
//! Credit, per the house pattern and stated in the form `MESH_DESIGN.md` §2.1 carries:
//!
//! * **FHP-6** — Frisch, Hasslacher & Pomeau, *Lattice-gas automata for the Navier–Stokes
//!   equation*, Phys. Rev. Lett. **56** (1986) 1505. The hexagonal lattice's fourth-order
//!   isotropy is theirs, and it is the whole warrant of the founding 64-state object.
//! * **FCHC-24** — d'Humières, Lallemand & Frisch, *Lattice gas models for 3D hydrodynamics*,
//!   Europhys. Lett. **2** (1986) 291. The face-centred **hyper**-cubic lattice in 4D,
//!   projected onto 3D, adopted precisely because no 3D Bravais lattice with a single speed
//!   has an isotropic fourth-rank tensor.
//!
//! Ours is the enumeration and the ledger-cap consequence, nothing more. Taking D3Q6 or FCC-12
//! because they are cheaper would silently drop the property that makes the 2D chart mean
//! anything, in a lane where nothing in the engine would notice.
//!
//! # Scope
//!
//! This module **enumerates a mode set**. It does not implement a 3D chart, a collision rule,
//! or a 3D ledger — `GrossState`'s momentum arity is still 2 (M-G1), and nothing here changes
//! that. It exists so the numbers in the design document have a witness that runs.

use ciris_sim_core::regplus::DIRECTIONS as FHP_DIRECTIONS;

/// Momentum components carried by the FCHC chart. The lattice lives in 4D and is projected
/// onto 3D; the fourth component is conserved by the projection and is a **spurious
/// invariant** — a known property of the model, named here rather than hidden. See
/// `MESH_DESIGN.md` §2.1 for why the recommendation is to carry it in the ledger anyway: a
/// dropped lane is a conservation claim we stop being able to check.
pub const FCHC_COMPONENTS: usize = 4;

/// The 24 FCHC directions: 4D vectors with exactly two non-zero components, each `±1`.
///
/// Built by [`fchc_directions`] rather than typed out, so the set cannot acquire a typo that
/// every downstream number would then inherit.
pub fn fchc_directions() -> Vec<[i64; FCHC_COMPONENTS]> {
    let mut out = Vec::with_capacity(24);
    for i in 0..FCHC_COMPONENTS {
        for j in (i + 1)..FCHC_COMPONENTS {
            for si in [1i64, -1] {
                for sj in [1i64, -1] {
                    let mut v = [0i64; FCHC_COMPONENTS];
                    v[i] = si;
                    v[j] = sj;
                    out.push(v);
                }
            }
        }
    }
    out
}

/// The six FHP directions, lifted into the 4-component frame so one enumerator serves both
/// charts. The extra components are identically zero, so the sector count is unchanged.
pub fn fhp_directions() -> Vec<[i64; FCHC_COMPONENTS]> {
    FHP_DIRECTIONS
        .iter()
        .map(|d| [d[0], d[1], 0, 0])
        .collect()
}

/// What an enumeration of a mode set found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Enumeration {
    /// `2^directions` — every local occupancy pattern.
    pub local_states: u64,
    /// Distinct `(N, P)` conservation labels.
    pub sectors: usize,
    /// How many local states the largest sector holds.
    pub largest_sector: u64,
    /// Sectors holding exactly one local state.
    pub singleton_sectors: usize,
    /// Maximum occupancy of one cell — the number of directions.
    pub max_occupancy: usize,
    /// Maximum `|p|` in any single momentum component, over all local states.
    pub max_momentum_per_component: i64,
}

/// Half-width of the momentum range the accumulator allows per component. Six is the FCHC
/// maximum (six directions carry `+1` in any given component); the enumerator refuses rather
/// than wrapping if a mode set ever exceeds it.
const MOMENTUM_HALF: i64 = 6;
const MOMENTUM_SPAN: usize = (2 * MOMENTUM_HALF + 1) as usize;

/// Enumerate every local state of a mode set and group them into `(N, P)` sectors.
///
/// Dynamic programming over the directions: the table is keyed by the label, so the cost is
/// the number of reachable labels rather than `2^n`. It still visits all `2^24` states'
/// worth of counts for FCHC — it just never materialises them.
///
/// Returns `None` if a momentum component leaves the accumulator's range, which is a refusal
/// rather than a wrong number.
pub fn enumerate(directions: &[[i64; FCHC_COMPONENTS]]) -> Option<Enumeration> {
    let n = directions.len();
    let occ_slots = n + 1;
    let stride = MOMENTUM_SPAN.pow(FCHC_COMPONENTS as u32);
    let mut current = vec![0u64; occ_slots * stride];
    let mut next = vec![0u64; occ_slots * stride];

    let index = |occ: usize, p: [i64; FCHC_COMPONENTS]| -> Option<usize> {
        let mut off = 0usize;
        for c in p.iter() {
            if c.abs() > MOMENTUM_HALF {
                return None;
            }
            off = off * MOMENTUM_SPAN + (c + MOMENTUM_HALF) as usize;
        }
        Some(occ * stride + off)
    };
    let unindex = |slot: usize| -> (usize, [i64; FCHC_COMPONENTS]) {
        let occ = slot / stride;
        let mut off = slot % stride;
        let mut p = [0i64; FCHC_COMPONENTS];
        for c in (0..FCHC_COMPONENTS).rev() {
            p[c] = (off % MOMENTUM_SPAN) as i64 - MOMENTUM_HALF;
            off /= MOMENTUM_SPAN;
        }
        (occ, p)
    };

    current[index(0, [0; FCHC_COMPONENTS])?] = 1;
    for direction in directions {
        next.iter_mut().for_each(|slot| *slot = 0);
        for slot in 0..current.len() {
            let count = current[slot];
            if count == 0 {
                continue;
            }
            let (occ, p) = unindex(slot);
            // Direction empty: the label is unchanged.
            next[slot] += count;
            // Direction occupied: one more quantum, momentum shifted by the direction.
            let mut q = p;
            for c in 0..FCHC_COMPONENTS {
                q[c] += direction[c];
            }
            next[index(occ + 1, q)?] += count;
        }
        core::mem::swap(&mut current, &mut next);
    }

    let mut local_states = 0u64;
    let mut sectors = 0usize;
    let mut largest_sector = 0u64;
    let mut singleton_sectors = 0usize;
    let mut max_occupancy = 0usize;
    let mut max_momentum = 0i64;
    for slot in 0..current.len() {
        let count = current[slot];
        if count == 0 {
            continue;
        }
        let (occ, p) = unindex(slot);
        local_states += count;
        sectors += 1;
        largest_sector = largest_sector.max(count);
        if count == 1 {
            singleton_sectors += 1;
        }
        max_occupancy = max_occupancy.max(occ);
        for c in p.iter() {
            max_momentum = max_momentum.max(c.abs());
        }
    }

    Some(Enumeration {
        local_states,
        sectors,
        largest_sector,
        singleton_sectors,
        max_occupancy,
        max_momentum_per_component: max_momentum,
    })
}

/// Largest constituent count a `u64` occupancy lane can hold when each terminal holon writes
/// `per_leaf` units into it. The 3D cap of `MESH_DESIGN.md` §2.3, computed rather than quoted.
pub fn occupancy_lane_cap(per_leaf: u64) -> u128 {
    if per_leaf == 0 {
        return u128::MAX;
    }
    u64::MAX as u128 / per_leaf as u128
}

/// Largest constituent count an `i64` momentum lane can hold — half the range before the sign
/// bit, which is why a chart writing momentum has no more headroom than one writing occupancy.
pub fn momentum_lane_cap(per_leaf: u64) -> u128 {
    if per_leaf == 0 {
        return u128::MAX;
    }
    i64::MAX as u128 / per_leaf as u128
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **THE CONTROL, and it is what makes the 3D number believable.**
    ///
    /// The same enumerator, run over the six FHP directions, must reproduce the object
    /// `Core/Lattice.lean` proves and `regplus.rs::runtime_sector_table_matches_the_lean_theorem`
    /// already pins: 64 local states, **53 sectors**, dimension histogram **44 / 7 / 2**.
    #[test]
    fn the_enumerator_reproduces_the_lean_theorems_53_sectors_on_fhp_6() {
        let e = enumerate(&fhp_directions()).expect("FHP-6 is in range");
        assert_eq!(e.local_states, 64, "FHP-6 has 2^6 local states");
        assert_eq!(e.sectors, 53, "Core/Lattice.lean proves 53 sectors");
        assert_eq!(e.max_occupancy, 6);

        // The full 44 / 7 / 2 histogram, not just the count — a wrong grouping could still
        // total 53.
        let (mut d1, mut d2, mut d3) = (0, 0, 0);
        for count in sector_histogram(&fhp_directions()) {
            match count {
                1 => d1 += 1,
                2 => d2 += 1,
                3 => d3 += 1,
                other => panic!("unexpected FHP-6 sector dimension {other}"),
            }
        }
        assert_eq!(
            (d1, d2, d3),
            (44, 7, 2),
            "FHP-6 dimension histogram must match Core/Lattice.lean"
        );
    }

    /// The FHP maximum per momentum component is **2**, independently of `holon-sandbox`.
    ///
    /// `tier.rs::LeafWrites::REG_PLUS_MAX` declared **3** until `ff27476`, which the sandbox
    /// lane's own enumeration refuted; this is a second, separately written instrument
    /// agreeing with that correction. Two enumerators reaching the same 2 is worth more than
    /// either alone, because the value decides a published ledger cap.
    #[test]
    fn the_fhp_momentum_maximum_is_two_not_three() {
        let e = enumerate(&fhp_directions()).expect("FHP-6 is in range");
        assert_eq!(
            e.max_momentum_per_component, 2,
            "no subset of the six axial directions reaches 3 in one component"
        );
    }

    /// The direction set is well formed before any number derived from it is believed.
    #[test]
    fn the_fchc_direction_set_is_twenty_four_distinct_two_nonzero_vectors() {
        let dirs = fchc_directions();
        assert_eq!(dirs.len(), 24);

        let mut seen = std::collections::BTreeSet::new();
        for d in &dirs {
            assert!(seen.insert(*d), "duplicate direction {d:?}");
            let nonzero = d.iter().filter(|c| **c != 0).count();
            assert_eq!(nonzero, 2, "FCHC vectors have exactly two non-zero components");
            assert!(d.iter().all(|c| c.abs() <= 1), "components are 0 or ±1");
        }
        // Closed under negation: a lattice gas needs every direction's opposite, or momentum
        // could not reverse.
        for d in &dirs {
            let neg = [-d[0], -d[1], -d[2], -d[3]];
            assert!(seen.contains(&neg), "no opposite for {d:?}");
        }
    }

    /// **M-G3 closed: the design document's three FCHC numbers, derived here.**
    ///
    /// `MESH_DESIGN.md` §2.1 quotes 16,777,216 local states, 72,047 sectors and a largest
    /// sector of 11,740. If this test fails, the document is wrong and must move; the numbers
    /// no longer live only in a script that ran once.
    #[test]
    fn fchc_24_enumerates_to_the_documented_sector_count() {
        let e = enumerate(&fchc_directions()).expect("FCHC-24 is in range");
        assert_eq!(e.local_states, 1 << 24, "FCHC-24 has 2^24 local states");
        assert_eq!(e.local_states, 16_777_216);
        assert_eq!(e.sectors, 72_047, "MESH_DESIGN §2.1's sector count");
        assert_eq!(e.largest_sector, 11_740, "MESH_DESIGN §2.1's largest sector");
        assert_eq!(e.singleton_sectors, 10_322);
        assert_eq!(e.max_occupancy, 24);
        assert_eq!(
            e.max_momentum_per_component, 6,
            "six directions carry +1 in any given component"
        );
    }

    /// The ledger-cap consequence of §2.3, computed from the enumeration rather than quoted:
    /// occupancy binds in both charts, and 3D is exactly 4× tighter because 24/6 = 4.
    #[test]
    fn the_three_d_ledger_cap_is_four_times_tighter_and_occupancy_binds() {
        let fhp = enumerate(&fhp_directions()).expect("in range");
        let fchc = enumerate(&fchc_directions()).expect("in range");

        let fhp_occ = occupancy_lane_cap(fhp.max_occupancy as u64);
        let fhp_mom = momentum_lane_cap(fhp.max_momentum_per_component as u64);
        let fchc_occ = occupancy_lane_cap(fchc.max_occupancy as u64);
        let fchc_mom = momentum_lane_cap(fchc.max_momentum_per_component as u64);

        assert!(fhp_occ < fhp_mom, "2D: the occupancy lane binds");
        assert!(fchc_occ < fchc_mom, "3D: the occupancy lane binds");
        assert_eq!(fhp_occ, 3_074_457_345_618_258_602);
        assert_eq!(fchc_occ, 768_614_336_404_564_650);
        // Exactly 4x, because 24/6 = 4. Integer division makes this exact, not approximate.
        assert_eq!(fhp_occ / fchc_occ, 4);
    }

    /// The enumerator REFUSES rather than wrapping when a mode set exceeds its accumulator.
    /// A silent wrap here would produce a plausible sector count that is simply wrong.
    #[test]
    fn a_mode_set_beyond_the_momentum_range_is_refused_not_wrapped() {
        // Fourteen copies of the same direction drive one component to 14, past the ±6 range.
        let runaway: Vec<[i64; FCHC_COMPONENTS]> = (0..14).map(|_| [1, 0, 0, 0]).collect();
        assert_eq!(enumerate(&runaway), None);
    }

    /// Per-sector local-state counts, for the histogram assertions.
    fn sector_histogram(directions: &[[i64; FCHC_COMPONENTS]]) -> Vec<u64> {
        use std::collections::BTreeMap;
        let n = directions.len();
        assert!(n <= 20, "the brute-force histogram is for small mode sets only");
        let mut sectors: BTreeMap<(u32, [i64; FCHC_COMPONENTS]), u64> = BTreeMap::new();
        for mask in 0u32..(1 << n) {
            let mut p = [0i64; FCHC_COMPONENTS];
            let occ = mask.count_ones();
            for (k, direction) in directions.iter().enumerate() {
                if mask >> k & 1 == 1 {
                    for c in 0..FCHC_COMPONENTS {
                        p[c] += direction[c];
                    }
                }
            }
            *sectors.entry((occ, p)).or_insert(0) += 1;
        }
        sectors.into_values().collect()
    }

    /// The two enumerators — the dynamic-programming one used for FCHC and the brute-force
    /// one used for the histogram — must agree where both can run. A DP that is subtly wrong
    /// would otherwise pass its own control.
    #[test]
    fn the_dp_and_brute_force_enumerators_agree_on_fhp_6() {
        let dp = enumerate(&fhp_directions()).expect("in range");
        let brute = sector_histogram(&fhp_directions());
        assert_eq!(dp.sectors, brute.len());
        assert_eq!(dp.local_states, brute.iter().sum::<u64>());
        assert_eq!(dp.largest_sector, *brute.iter().max().unwrap());
    }
}
