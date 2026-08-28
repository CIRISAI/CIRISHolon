//! THE HOLON — one recursive data object for every tier.
//!
//! `Holon = { planes, ledger, chart, cert, children }` (DATA_OBJECT.md).
//! The planes are the state layout at every tier (bit-packed F₂ at the
//! bottom, f64 lanes at the top); the ledger is what the dynamics pays to
//! keep the representation honest (signs → cyclotomic branch weights → rent);
//! the chart declares its own conditioning; the certificate is a constructor
//! REQUIREMENT — a tier without its square is not a tier; children live in an
//! append-only arena, because identity is the arena index forever.
//!
//! Tier instances in this crate: tier 0 (classical planes), tier 1 (packed
//! Pauli-plane tableau, Aaronson–Gottesman semantics, Stim-style layout —
//! both credited), tier 2 ledger ring (exact Z[ω]). Conformance referees:
//! the certified holon-qasm tiers (QASM-1/2 records) as dev-dependencies.

pub mod affine;
pub mod cyclo;
pub mod cyclon;
pub mod face;
pub mod grain;
pub mod job;
pub mod ledger;
pub mod magic;
pub mod magic5;
pub mod merge;
pub mod mesh;
pub mod phasepoly;
pub mod plane;
pub mod prune;
pub mod qasm;
pub mod real;
pub mod residue;
pub mod run;
pub mod sample;
pub mod simd;
pub mod simplify;
pub mod sliced;
pub mod coltableau;
pub mod tableau;
pub mod zx;
pub mod transport;
pub mod tune;

/// THE INTEGRATION CONTRACT for the magic-tier workstreams (BG decomposition,
/// pruning, sampling, mesh): a branch source enumerates stabilizer branches;
/// consumers fold amplitudes. Exact Z[ω] sums are order-independent, so any
/// sharding of the fold is deterministic — the mesh's whole warrant.
pub trait BranchSource: Sync {
    fn n_branches(&self) -> u64;
    /// coeff_b · ⟨y|φ_b⟩, exact. Cheap to call per (branch, y).
    fn amplitude_of(&self, branch: u64, y: &[bool]) -> ledger::Cyc;
    fn n_qubits(&self) -> usize;
}

use plane::BitPlane;

/// The chart: a partition with its conditioning DECLARED (lean/Object.lean's
/// `coherence`: |Σa| / Σ|a| — an ill-conditioned aggregate never ships as
/// engine state undeclared).
#[derive(Clone, Debug)]
pub struct Chart {
    pub cells: usize,
    /// Declared coherence per exposed aggregate (1.0 = perfectly conditioned,
    /// e.g. all-nonnegative charts; see coherence_of_nonneg).
    pub conditioning: Vec<(String, f64)>,
}

/// The certificate: the square, carried as data. Construction of a `Holon`
/// REQUIRES one; `Certificate::exact` is for tiers exact by construction,
/// battery receipts for the certified-approximate tiers.
#[derive(Clone, Debug)]
pub struct Certificate {
    pub view: &'static str,
    pub step: &'static str,
    pub rate: &'static str,
    /// "exact" or a battery-receipt identifier (prereg/results path upstream).
    pub receipt: String,
}

impl Certificate {
    pub fn exact(view: &'static str, step: &'static str, rate: &'static str) -> Self {
        Certificate { view, step, rate, receipt: "exact-by-construction".into() }
    }
}

/// Append-only arena: push-only, identity = index forever (LESSONS.md rule 1).
#[derive(Clone, Debug, Default)]
pub struct Arena<T> {
    items: Vec<T>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Arena { items: Vec::new() }
    }
    pub fn push(&mut self, t: T) -> usize {
        self.items.push(t);
        self.items.len() - 1
    }
    pub fn get(&self, id: usize) -> &T {
        &self.items[id]
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Tier 0: the classical holon — state as bit-planes, gates as flips.
pub struct ClassicalHolon {
    pub bits: BitPlane,
    pub chart: Chart,
    pub cert: Certificate,
}

impl ClassicalHolon {
    pub fn new(n: usize) -> Self {
        ClassicalHolon {
            bits: BitPlane::zeros(n),
            chart: Chart { cells: n, conditioning: vec![("occupancy".into(), 1.0)] },
            cert: Certificate::exact("bit-plane", "reversible-classical", "bit-plane"),
        }
    }
    pub fn x(&mut self, q: usize) {
        self.bits.flip(q);
    }
    pub fn cx(&mut self, c: usize, t: usize) {
        if self.bits.get(c) {
            self.bits.flip(t);
        }
    }
    pub fn ccx(&mut self, a: usize, b: usize, t: usize) {
        if self.bits.get(a) && self.bits.get(b) {
            self.bits.flip(t);
        }
    }
}

/// Tier 1: the stabilizer holon — the packed tableau IS the planes+ledger.
pub struct StabilizerHolon {
    pub tab: tableau::PackedTableau,
    pub cert: Certificate,
}

impl StabilizerHolon {
    pub fn new(n: usize) -> Self {
        StabilizerHolon {
            tab: tableau::PackedTableau::new(n),
            cert: Certificate::exact("pauli-planes", "clifford", "tableau-update"),
        }
    }
}

/// Two-level recursion: a coarse holon whose plane entries are certified
/// views of child holons. The composition theorem behind it is
/// `closed_comp` / `Tier.stack` in lean/; here it is exercised as data.
pub struct CoarseHolon {
    pub children: Arena<ClassicalHolon>,
    pub cert: Certificate,
}

impl CoarseHolon {
    /// The coarse reading: parity per child — a Closed view of reversible
    /// classical dynamics (parity commutes with X-count mod 2 bookkeeping).
    pub fn read(&self) -> BitPlane {
        let mut p = BitPlane::zeros(self.children.len());
        for i in 0..self.children.len() {
            p.set(i, self.children.get(i).bits.popcount() % 2 == 1);
        }
        p
    }
}
