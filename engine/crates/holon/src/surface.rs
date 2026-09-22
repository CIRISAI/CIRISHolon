//! THE ROTATED SURFACE CODE — the flagship adaptive workload.
//!
//! A distance-`d` rotated surface code: `d²` data qubits on a square grid,
//! `d²−1` ancillas at the plaquettes, `n = 2d²−1` qubits total. Syndrome
//! extraction is the canonical adaptive circuit — entangle each ancilla with
//! its data neighbours, measure it mid-circuit, feed the outcome forward into
//! a correction — which is exactly the shape `adaptive.rs` was built for and
//! exactly what a fault-tolerant machine spends ~99.9% of its time doing.
//!
//! LAYOUT, DERIVED NOT REMEMBERED. The plaquette set is generated from one
//! rule (bulk plaquette `(i,j)` is X-type iff `i+j` is even; a boundary
//! plaquette is the weight-2 remnant of the virtual plaquette one step
//! outside the grid, kept only when its type matches the edge it sits on),
//! and then CHECKED: `verify_commuting` asserts every pair of stabilizers
//! commutes and the count is `d²−1`. A layout that fails those assertions is
//! not a surface code, so the demo refuses to run on one rather than
//! reporting confident nonsense about a state that is not a codestate.
//!
//! Credited: Bombin–Martin-Delgado (rotated lattice), Fowler et al.
//! (arXiv:1208.0928) for the extraction schedule. The interleaved data order
//! (NW, NE, SW, SE for Z; NW, SW, NE, SE for X) is the standard hook-error-
//! avoiding schedule; in a NOISELESS exact simulation it changes no observable,
//! and is kept so the circuit is the real one rather than a convenient one.

/// Which Pauli the plaquette measures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    X,
    Z,
}

/// One stabilizer: its type, its SCHEDULE, and the ancilla that reads it.
///
/// `sched[t]` is the data qubit this plaquette touches at time step `t` of
/// the four-step extraction cycle, or `None` where the plaquette is a
/// weight-2 boundary remnant and that corner falls outside the lattice. The
/// schedule is the load-bearing part: two CNOTs that share a data qubit do
/// NOT commute when one uses it as control and the other as target, so a
/// wrong schedule silently produces wrong syndromes. `verify_schedule`
/// asserts every data qubit is touched at most once per time step.
#[derive(Clone, Debug)]
pub struct Stab {
    pub kind: Kind,
    /// Data qubit per time step; `None` = this corner is off-lattice.
    pub sched: [Option<usize>; 4],
    /// The ancilla qubit index that carries this syndrome.
    pub ancilla: usize,
}

impl Stab {
    /// The data qubits this stabilizer acts on, in schedule order.
    pub fn data(&self) -> Vec<usize> {
        self.sched.iter().flatten().copied().collect()
    }

    /// Weight of the stabilizer.
    pub fn weight(&self) -> usize {
        self.sched.iter().flatten().count()
    }

    pub fn touches(&self, q: usize) -> bool {
        self.sched.iter().flatten().any(|&d| d == q)
    }
}

#[derive(Clone, Debug)]
pub struct SurfaceCode {
    pub d: usize,
    /// Total qubits: `d²` data then `d²−1` ancillas.
    pub n: usize,
    pub stabs: Vec<Stab>,
    /// Grid position `r·d + c` ↦ the qubit index the CIRCUIT uses. The
    /// identity for [`SurfaceCode::new`]; a BAND ordering for
    /// [`SurfaceCode::banded`]. Nothing about the code depends on it — a
    /// relabelling is a renaming, not a different circuit — but where a
    /// qubit SITS in the index space is what decides whether a contiguous
    /// column cut has its CX pairs inside one shard or across two.
    pub label: Vec<usize>,
}

impl SurfaceCode {
    /// Data qubit index for grid position `(r, c)`.
    #[inline]
    pub fn data(&self, r: usize, c: usize) -> usize {
        self.label[r * self.d + c]
    }

    /// Number of data qubits.
    #[inline]
    pub fn n_data(&self) -> usize {
        self.d * self.d
    }

    /// Build the distance-`d` rotated code. `d` must be odd and ≥ 3.
    pub fn new(d: usize) -> Self {
        assert!(d >= 3 && d % 2 == 1, "rotated surface code needs odd d ≥ 3");
        let nd = d * d;
        let mut stabs: Vec<Stab> = Vec::with_capacity(nd - 1);

        let idx = |r: usize, c: usize| r * d + c;

        // ---- bulk: (d−1)² weight-4 plaquettes, centre (i+½, j+½) ----
        //
        // The two hook-avoiding orders (Fowler et al.): a Z plaquette walks
        // NW, NE, SW, SE; an X plaquette walks NW, SW, NE, SE. Those two
        // orders are what make the cycle conflict-free — at step 2 a Z
        // plaquette wants its NE while an X plaquette wants its SW, and the
        // parity rule puts those on plaquettes of the same type, so they
        // never collide. `verify_schedule` checks it rather than trusting it.
        for i in 0..d - 1 {
            for j in 0..d - 1 {
                let kind = if (i + j) % 2 == 0 { Kind::X } else { Kind::Z };
                let sched = match kind {
                    Kind::Z => [
                        Some(idx(i, j)),         // NW
                        Some(idx(i, j + 1)),     // NE
                        Some(idx(i + 1, j)),     // SW
                        Some(idx(i + 1, j + 1)), // SE
                    ],
                    Kind::X => [
                        Some(idx(i, j)),         // NW
                        Some(idx(i + 1, j)),     // SW
                        Some(idx(i, j + 1)),     // NE
                        Some(idx(i + 1, j + 1)), // SE
                    ],
                };
                stabs.push(Stab { kind, sched, ancilla: 0 });
            }
        }

        // ---- boundaries: weight-2 remnants of the virtual plaquettes ----
        // The virtual plaquette one step outside the grid keeps the same
        // parity rule AND the same schedule; we retain it only where its type
        // is the one that edge carries, and the off-lattice corners become
        // `None`. Keeping the parent's time steps is what preserves the
        // conflict-freedom at the boundary.
        for j in 0..d - 1 {
            // top: virtual row i = −1, parity (−1 + j) even ⇔ j odd ⇒ X-type.
            // X order NW,SW,NE,SE over (−1,j),(0,j),(−1,j+1),(0,j+1).
            if j % 2 == 1 {
                stabs.push(Stab {
                    kind: Kind::X,
                    sched: [None, Some(idx(0, j)), None, Some(idx(0, j + 1))],
                    ancilla: 0,
                });
            }
            // bottom: virtual row i = d−1; for odd d, (d−1+j) even ⇔ j even.
            // X order over (d−1,j),(d,j),(d−1,j+1),(d,j+1).
            if j % 2 == 0 {
                stabs.push(Stab {
                    kind: Kind::X,
                    sched: [Some(idx(d - 1, j)), None, Some(idx(d - 1, j + 1)), None],
                    ancilla: 0,
                });
            }
        }
        for i in 0..d - 1 {
            // left: virtual col j = −1, Z-type ⇔ (i−1) odd ⇔ i even.
            // Z order NW,NE,SW,SE over (i,−1),(i,0),(i+1,−1),(i+1,0).
            if i % 2 == 0 {
                stabs.push(Stab {
                    kind: Kind::Z,
                    sched: [None, Some(idx(i, 0)), None, Some(idx(i + 1, 0))],
                    ancilla: 0,
                });
            }
            // right: virtual col j = d−1, Z-type ⇔ (i+d−1) odd ⇔ i odd.
            // Z order over (i,d−1),(i,d),(i+1,d−1),(i+1,d).
            if i % 2 == 1 {
                stabs.push(Stab {
                    kind: Kind::Z,
                    sched: [Some(idx(i, d - 1)), None, Some(idx(i + 1, d - 1)), None],
                    ancilla: 0,
                });
            }
        }

        // Ancillas follow the data block, one per stabilizer.
        for (k, s) in stabs.iter_mut().enumerate() {
            s.ancilla = nd + k;
        }

        let n = nd + stabs.len();
        let code = SurfaceCode { d, n, stabs, label: (0..nd).collect() };
        code.verify_commuting();
        code.verify_schedule();
        code
    }

    /// THE SAME CODE, NUMBERED IN BANDS — for the column-sharded engine
    /// (`sharded.rs`, MESH-CLIFFORD-1).
    ///
    /// The natural numbering is all `d²` data qubits, then all `d²−1`
    /// ancillas. A shard that is a contiguous range of qubit COLUMNS
    /// therefore splits data from ancilla, and since every CX in the
    /// extraction schedule joins a data qubit to an ancilla, ALMOST EVERY
    /// GATE CROSSES: measured, 0.96–1.00 of them at S ∈ {2,4,8}. The prereg's
    /// §1 asks for boundaries "chosen on the code's row structure"; no choice
    /// of boundary can do it, because the structure is not in the numbering.
    /// It has to be PUT there.
    ///
    /// So: band `i` is data row `i` followed by the ancillas of every
    /// plaquette whose topmost data row is `i`. Each band is `2d−1`
    /// consecutive qubits, a contiguous column range is a run of whole bands,
    /// and a CX crosses only when its plaquette straddles a band boundary —
    /// `2(d−1)` gates per boundary out of ~`4d²`, i.e. `≈ (S−1)/(2d)`.
    ///
    /// This is a RENAMING: same stabilizers, same schedule, same syndromes in
    /// the same order, same logical operators, and `verify_commuting` /
    /// `verify_schedule` run on it exactly as on the natural numbering.
    pub fn banded(d: usize) -> Self {
        let mut code = SurfaceCode::new(d);
        let nd = code.n_data();
        // The band a plaquette belongs to: its topmost data row.
        let band_of = |s: &Stab| -> usize {
            s.sched
                .iter()
                .flatten()
                .map(|&q| q / d)
                .min()
                .expect("a stabilizer with no data qubit is not a stabilizer")
        };
        let mut newidx = vec![usize::MAX; code.n];
        let mut next = 0usize;
        for i in 0..d {
            for c in 0..d {
                newidx[i * d + c] = next;
                next += 1;
            }
            for s in code.stabs.iter().filter(|s| band_of(s) == i) {
                newidx[s.ancilla] = next;
                next += 1;
            }
        }
        assert_eq!(next, code.n, "the band ordering must be a permutation");
        assert!(newidx.iter().all(|&v| v != usize::MAX), "band ordering missed a qubit");
        for s in code.stabs.iter_mut() {
            s.ancilla = newidx[s.ancilla];
            for q in s.sched.iter_mut().flatten() {
                *q = newidx[*q];
            }
        }
        code.label = (0..nd).map(|q| newidx[q]).collect();
        code.verify_commuting();
        code.verify_schedule();
        code
    }

    /// The four-step cycle must be CONFLICT-FREE: at each time step, no data
    /// qubit is touched by two plaquettes. Two CNOTs sharing a data qubit —
    /// one using it as control (Z extraction), the other as target (X
    /// extraction) — do not commute, so a colliding schedule yields wrong
    /// syndromes with no other symptom. Panics rather than proceed.
    pub fn verify_schedule(&self) {
        for t in 0..4 {
            // Sized by `n`, not `n_data`: under a band numbering
            // (`SurfaceCode::banded`) a data qubit's index is no longer
            // bounded by `d²`, and a check that panics on its own indexing
            // is not a check.
            let mut seen = vec![usize::MAX; self.n];
            for (k, s) in self.stabs.iter().enumerate() {
                if let Some(q) = s.sched[t] {
                    assert!(
                        seen[q] == usize::MAX,
                        "schedule collision at step {t}: data {q} touched by \
                         stabilizers {} and {k}",
                        seen[q]
                    );
                    seen[q] = k;
                }
            }
        }
    }

    /// The code is only a code if the count is right and every pair commutes.
    /// Panics otherwise — a wrong layout must never reach a measurement.
    pub fn verify_commuting(&self) {
        let d = self.d;
        assert_eq!(
            self.stabs.len(),
            d * d - 1,
            "rotated code must have d²−1 stabilizers"
        );
        assert_eq!(self.n, 2 * d * d - 1, "n must be 2d²−1");
        // Two stabilizers of the SAME type always commute. Different types
        // commute iff they share an EVEN number of data qubits.
        for a in 0..self.stabs.len() {
            for b in a + 1..self.stabs.len() {
                let (sa, sb) = (&self.stabs[a], &self.stabs[b]);
                if sa.kind == sb.kind {
                    continue;
                }
                let (da, db) = (sa.data(), sb.data());
                let shared = da.iter().filter(|q| db.contains(q)).count();
                assert_eq!(
                    shared % 2,
                    0,
                    "stabilizers {a} and {b} anticommute (share {shared} data qubits)"
                );
            }
        }
    }

    /// The logical Z operator: a Z-string spanning the lattice. For this
    /// layout (X on top/bottom boundaries, Z on left/right) the logical Z runs
    /// along a ROW, commuting with every X plaquette it meets in two places.
    pub fn logical_z(&self) -> Vec<usize> {
        (0..self.d).map(|c| self.data(0, c)).collect()
    }

    /// The logical X operator: an X-string down a COLUMN.
    pub fn logical_x(&self) -> Vec<usize> {
        (0..self.d).map(|r| self.data(r, 0)).collect()
    }

    /// Data qubits touched by each Z-type stabilizer, indexed by stabilizer.
    /// The decoder's lookup: which Z-plaquettes does an X error on data `q`
    /// flip? (An X error anticommutes with the Z stabilizers containing `q`.)
    pub fn z_stabs_touching(&self, q: usize) -> Vec<usize> {
        self.stabs
            .iter()
            .enumerate()
            .filter(|(_, s)| s.kind == Kind::Z && s.touches(q))
            .map(|(i, _)| i)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_is_a_code_at_every_distance_we_use() {
        for d in [3usize, 5, 7, 9, 11, 21, 31] {
            let c = SurfaceCode::new(d); // verify_commuting runs inside
            assert_eq!(c.stabs.len(), d * d - 1);
            assert_eq!(c.n, 2 * d * d - 1);
            let xs = c.stabs.iter().filter(|s| s.kind == Kind::X).count();
            let zs = c.stabs.iter().filter(|s| s.kind == Kind::Z).count();
            assert_eq!(xs, zs, "d={d}: X and Z stabilizer counts must match");
            assert_eq!(xs + zs, d * d - 1);
        }
    }

    #[test]
    fn boundary_plaquettes_are_weight_two_and_bulk_weight_four() {
        let c = SurfaceCode::new(7);
        let w4 = c.stabs.iter().filter(|s| s.weight() == 4).count();
        let w2 = c.stabs.iter().filter(|s| s.weight() == 2).count();
        assert_eq!(w4, 36, "(d−1)² bulk plaquettes");
        assert_eq!(w2, 12, "2(d−1) boundary plaquettes");
        assert_eq!(w4 + w2, c.stabs.len());
    }

    /// Every data qubit must be read by at least one stabilizer of each type,
    /// or the code cannot detect both error kinds there.
    #[test]
    fn every_data_qubit_is_covered_by_both_types() {
        let c = SurfaceCode::new(9);
        for q in 0..c.n_data() {
            let has_x = c
                .stabs
                .iter()
                .any(|s| s.kind == Kind::X && s.touches(q));
            let has_z = c
                .stabs
                .iter()
                .any(|s| s.kind == Kind::Z && s.touches(q));
            assert!(has_x, "data qubit {q} has no X stabilizer");
            assert!(has_z, "data qubit {q} has no Z stabilizer");
        }
    }

    /// The logical operators must commute with every stabilizer (they are in
    /// the normalizer) and anticommute with each other.
    #[test]
    fn logicals_commute_with_stabilizers_and_anticommute_with_each_other() {
        for d in [3usize, 5, 7, 9] {
            let c = SurfaceCode::new(d);
            let lz = c.logical_z();
            let lx = c.logical_x();
            for (i, s) in c.stabs.iter().enumerate() {
                // logical Z (Z-string) vs X-type stabilizer: overlap must be even
                if s.kind == Kind::X {
                    let ov = s.data().iter().filter(|q| lz.contains(q)).count();
                    assert_eq!(ov % 2, 0, "d={d}: logical Z anticommutes with X stab {i}");
                }
                // logical X (X-string) vs Z-type stabilizer: overlap must be even
                if s.kind == Kind::Z {
                    let ov = s.data().iter().filter(|q| lx.contains(q)).count();
                    assert_eq!(ov % 2, 0, "d={d}: logical X anticommutes with Z stab {i}");
                }
            }
            let ov = lz.iter().filter(|q| lx.contains(q)).count();
            assert_eq!(ov % 2, 1, "d={d}: logical X and Z must anticommute");
        }
    }

    /// The BAND numbering must be a renaming and nothing more: a permutation
    /// of the qubits, the same stabilizer set in the same order with the same
    /// weights and types, the same schedule, and the code's own checks
    /// passing on it — and its bands must be the contiguous runs the column
    /// cut needs (every qubit of band `i` before every qubit of band `i+1`).
    #[test]
    fn the_band_numbering_is_a_permutation_and_nothing_else() {
        for d in [3usize, 5, 7, 9, 21] {
            let nat = SurfaceCode::new(d);
            let ban = SurfaceCode::banded(d); // verify_* run inside
            assert_eq!(nat.n, ban.n);
            assert_eq!(nat.stabs.len(), ban.stabs.len());
            let mut seen = vec![false; ban.n];
            for q in ban.label.iter().chain(ban.stabs.iter().map(|s| &s.ancilla)) {
                assert!(!std::mem::replace(&mut seen[*q], true), "d={d}: index {q} reused");
            }
            assert!(seen.iter().all(|&b| b), "d={d}: the band numbering is not onto");
            for (a, b) in nat.stabs.iter().zip(&ban.stabs) {
                assert_eq!(a.kind, b.kind, "d={d}: stabilizer types moved");
                assert_eq!(a.weight(), b.weight(), "d={d}: stabilizer weights moved");
                for t in 0..4 {
                    assert_eq!(
                        a.sched[t].is_some(),
                        b.sched[t].is_some(),
                        "d={d}: schedule shape moved"
                    );
                    // The relabelled slot must be the SAME qubit under the map.
                    if let (Some(qa), Some(qb)) = (a.sched[t], b.sched[t]) {
                        let img = if qa < nat.n_data() {
                            ban.label[qa]
                        } else {
                            ban.stabs[nat
                                .stabs
                                .iter()
                                .position(|s| s.ancilla == qa)
                                .expect("ancilla exists")]
                            .ancilla
                        };
                        assert_eq!(qb, img, "d={d}: schedule slot {t} is not the image");
                    }
                }
            }
            // THE PROPERTY THE CUT NEEDS: each data row is a contiguous run,
            // the rows ascend, and a plaquette's ancilla sits between its own
            // data row and the next one — so a contiguous column range is a
            // run of whole bands and only straddling plaquettes cross.
            let mut prev_end = 0usize;
            for i in 0..d {
                let lo = ban.data(i, 0);
                assert!(lo >= prev_end, "d={d}: band {i} starts before band {} ends", i - 1);
                for c in 0..d {
                    assert_eq!(ban.data(i, c), lo + c, "d={d}: data row {i} is not contiguous");
                }
                prev_end = lo + d;
                let next_start = if i + 1 < d { ban.data(i + 1, 0) } else { ban.n };
                for s in &ban.stabs {
                    let top = s
                        .sched
                        .iter()
                        .flatten()
                        .copied()
                        .min()
                        .expect("non-empty");
                    if top >= lo && top < lo + d {
                        assert!(
                            s.ancilla >= lo + d && s.ancilla < next_start,
                            "d={d}: band {i}'s plaquette ancilla {} is outside its band",
                            s.ancilla
                        );
                    }
                }
            }
        }
    }

    /// An X error on any data qubit must flip a NON-EMPTY set of Z syndromes —
    /// otherwise the decoder has nothing to see and the demo's verification
    /// would be vacuous.
    #[test]
    fn every_single_x_error_is_visible_to_some_z_stabilizer() {
        let c = SurfaceCode::new(11);
        for q in 0..c.n_data() {
            assert!(
                !c.z_stabs_touching(q).is_empty(),
                "X error on data {q} is invisible to every Z stabilizer"
            );
        }
    }
}
