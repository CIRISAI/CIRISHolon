//! The species registry: hydrogen through neon, and the STO-3G basis they are given.
//!
//! # What is DECLARED here and what is computed elsewhere
//!
//! The ELEMENTS-1 freeze names three declared inputs and this file is where all three
//! live, so a reader can see the entire input surface of the chemical tier in one
//! screen:
//!
//! * the nuclear charge `Z` — an integer, and the only thing that distinguishes one
//!   element from another in the Hamiltonian;
//! * the nuclear mass — a MEASURED input, exactly like `m_e`, used by the sandbox's
//!   dynamics and never by the electronic structure (Born–Oppenheimer);
//! * the STO-3G contraction — a MODEL choice, three exponents and three coefficients
//!   per shell.
//!
//! Everything else — every energy, every bond length, every well depth, and WHICH PAIRS
//! BIND AT ALL — is computed from these and the closed-form integrals. There is no
//! fitted parameter anywhere in this crate and no table of chemical results.
//!
//! # Provenance of the basis
//!
//! STO-3G, the least-squares 3-Gaussian expansion of a Slater orbital:
//!
//! * W. J. Hehre, R. F. Stewart, J. A. Pople, *J. Chem. Phys.* **51**, 2657 (1969) —
//!   hydrogen and helium, and the method;
//! * W. J. Hehre, R. Ditchfield, R. F. Stewart, J. A. Pople, *J. Chem. Phys.* **52**,
//!   2769 (1970) — lithium through neon.
//!
//! The numbers below are the standard tabulation of those two papers as it is
//! distributed today (the Basis Set Exchange's `STO-3G`). Two structural facts of that
//! tabulation are worth stating because they are also the cheapest check on a
//! transcription error, and `tests/elements.rs` asserts both:
//!
//! * the CONTRACTION COEFFICIENTS are universal — every element's 1s shell shares one
//!   triple, and every first-row 2s and 2p shell shares one triple each. Only the
//!   exponents move with the element. That is the design of STO-3G (a fixed expansion of
//!   a Slater function, rescaled), not a coincidence.
//! * 2s and 2p share their EXPONENTS (the "sp shell"), and differ only in coefficients.
//!
//! # Hydrogen is not re-declared
//!
//! [`Species::HYDROGEN`]'s contraction is [`crate::sto3g::H_EXPONENTS`] and
//! [`crate::sto3g::H_COEFFS`] BY REFERENCE, not by a second copy of the same six
//! decimals. The banked H2 curve is pinned against a 50-digit referee at 5e-15 hartree;
//! a second transcription of its basis, even a correct one, would be a second thing that
//! could drift, and the H2 regression in `tests/referee.rs` would be grading two
//! different models.

use crate::sto3g::{H_COEFFS, H_EXPONENTS};

/// Electron masses per unified atomic mass unit, `m_e/u`.
///
/// A MEASURED input, and the same value `holon-render`'s `sim.rs` uses to build its
/// `M_H`; stated once here so the two cannot disagree about what a mass unit is.
pub const M_E_PER_U: f64 = 1822.888486;

/// The universal STO-3G 1s contraction coefficients, at the precision the crate's
/// hydrogen has always declared them.
///
/// Eight decimals rather than the ten the modern tabulation carries, because these ARE
/// [`crate::sto3g::H_COEFFS`] and that constant is what the pinned 50-digit H2 referee
/// was computed against. Taking two more digits here would be a change to the declared
/// model, not an improvement to it, and it would move a number the gate has measured.
pub const C_1S: [f64; 3] = H_COEFFS;

/// The universal STO-3G 2s contraction coefficients. The leading one is NEGATIVE — the
/// radial node of a 2s orbital is built by the contraction, not by the primitives.
pub const C_2S: [f64; 3] = [-0.09996723, 0.39951283, 0.70011547];

/// The universal STO-3G 2p contraction coefficients.
pub const C_2P: [f64; 3] = [0.15591627, 0.60768372, 0.39195739];

/// Which shells an element carries, in the order they enter the basis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellKind {
    /// 1s.
    S1,
    /// 2s.
    S2,
    /// 2p — three Cartesian components, `px`, `py`, `pz`.
    P2,
}

impl ShellKind {
    /// Total angular momentum: 0 for either s shell, 1 for the p shell.
    pub fn l(self) -> u8 {
        match self {
            ShellKind::S1 | ShellKind::S2 => 0,
            ShellKind::P2 => 1,
        }
    }

    /// Cartesian components: 1 for s, 3 for p.
    pub fn n_functions(self) -> usize {
        match self {
            ShellKind::S1 | ShellKind::S2 => 1,
            ShellKind::P2 => 3,
        }
    }
}

/// One declared shell: three primitive exponents and their three coefficients.
#[derive(Clone, Copy, Debug)]
pub struct Shell {
    pub kind: ShellKind,
    pub alpha: [f64; 3],
    pub coeff: [f64; 3],
}

/// A first-row element, with everything the engine is allowed to be told about it.
#[derive(Clone, Copy, Debug)]
pub struct Species {
    /// Chemical symbol. A LABEL — nothing computes from it.
    pub symbol: &'static str,
    /// Nuclear charge. DECLARED INPUT: an integer, and the whole of what makes an
    /// element that element as far as the Hamiltonian is concerned.
    pub z: u32,
    /// Nuclear mass of the most abundant isotope, in unified atomic mass units.
    /// DECLARED INPUT, and a MEASURED one — it plays no part in the electronic
    /// structure (Born–Oppenheimer) and only enters the sandbox's dynamics.
    pub mass_u: f64,
    /// The isotope the mass above belongs to. Recorded because "the mass of carbon" is
    /// ambiguous and the natural-abundance average is a different number.
    pub isotope: &'static str,
    /// Shells, in basis order. Two entries for H and He, three for Li..Ne.
    pub shells: &'static [Shell],
}

impl Species {
    /// Neutral-atom electron count. Equal to `Z`: this crate has no ions.
    pub fn n_electrons(self) -> u32 {
        self.z
    }

    /// Nuclear mass in ELECTRON masses, the unit the whole engine works in.
    ///
    /// The NUCLEUS plus its electrons — the atom — because the pair curves are
    /// Born–Oppenheimer and the electrons ride with the nuclei, which is the same
    /// convention `sim.rs` states for `M_H` and for the same reason.
    pub fn mass_me(self) -> f64 {
        self.mass_u * M_E_PER_U
    }

    /// Number of contracted basis functions this element contributes.
    pub fn n_basis(self) -> usize {
        self.shells.iter().map(|s| s.kind.n_functions()).sum()
    }

    pub const HYDROGEN: Species = Species {
        symbol: "H",
        z: 1,
        mass_u: 1.00782503207,
        isotope: "1H",
        shells: &[Shell {
            kind: ShellKind::S1,
            alpha: H_EXPONENTS,
            coeff: C_1S,
        }],
    };
}

/// Build the two-shell (H, He) or three-shell (Li..Ne) declaration in one line each.
macro_rules! first_row {
    ($name:ident, $sym:literal, $z:literal, $mass:literal, $iso:literal,
     s1 = $s1:expr, sp = $sp:expr) => {
        pub const $name: Species = Species {
            symbol: $sym,
            z: $z,
            mass_u: $mass,
            isotope: $iso,
            shells: &[
                Shell {
                    kind: ShellKind::S1,
                    alpha: $s1,
                    coeff: C_1S,
                },
                Shell {
                    kind: ShellKind::S2,
                    alpha: $sp,
                    coeff: C_2S,
                },
                Shell {
                    kind: ShellKind::P2,
                    alpha: $sp,
                    coeff: C_2P,
                },
            ],
        };
    };
    ($name:ident, $sym:literal, $z:literal, $mass:literal, $iso:literal, s1 = $s1:expr) => {
        pub const $name: Species = Species {
            symbol: $sym,
            z: $z,
            mass_u: $mass,
            isotope: $iso,
            shells: &[Shell {
                kind: ShellKind::S1,
                alpha: $s1,
                coeff: C_1S,
            }],
        };
    };
}

pub const HYDROGEN: Species = Species::HYDROGEN;

first_row!(
    HELIUM, "He", 2, 4.00260325413, "4He",
    s1 = [6.36242139, 1.15892300, 0.31364979]
);
first_row!(
    LITHIUM, "Li", 3, 7.0160034366, "7Li",
    s1 = [16.11957475, 2.93620066, 0.79465049],
    sp = [0.63628975, 0.14786005, 0.04808868]
);
first_row!(
    BERYLLIUM, "Be", 4, 9.012183065, "9Be",
    s1 = [30.16787069, 5.49511531, 1.48719265],
    sp = [1.31483311, 0.30553894, 0.09937075]
);
first_row!(
    BORON, "B", 5, 11.00930536, "11B",
    s1 = [48.79111318, 8.88736217, 2.40526704],
    sp = [2.23695614, 0.51982050, 0.16906176]
);
first_row!(
    CARBON, "C", 6, 12.0, "12C",
    s1 = [71.61683735, 13.04509632, 3.53051216],
    sp = [2.94124936, 0.68348310, 0.22228992]
);
first_row!(
    NITROGEN, "N", 7, 14.0030740044, "14N",
    s1 = [99.10616896, 18.05231239, 4.88566024],
    sp = [3.78045588, 0.87849664, 0.28571437]
);
first_row!(
    OXYGEN, "O", 8, 15.9949146196, "16O",
    s1 = [130.70932000, 23.80886605, 6.44360831],
    sp = [5.03315132, 1.16959612, 0.38038896]
);
first_row!(
    FLUORINE, "F", 9, 18.99840316273, "19F",
    s1 = [166.67913400, 30.36081233, 8.21682067],
    sp = [6.46480325, 1.50228124, 0.48858849]
);
first_row!(
    NEON, "Ne", 10, 19.9924401762, "20Ne",
    s1 = [207.01560700, 37.70815124, 10.20529731],
    sp = [8.24631512, 1.91626629, 0.62322927]
);

/// The first row, indexed by `Z - 1`.
pub const FIRST_ROW: [Species; 10] = [
    HYDROGEN, HELIUM, LITHIUM, BERYLLIUM, BORON, CARBON, NITROGEN, OXYGEN, FLUORINE, NEON,
];

/// Look an element up by nuclear charge. `None` outside 1..=10 — the model is the first
/// row and d functions are a successor's problem, so a silent extrapolation would be a
/// worse answer than a refusal.
pub fn by_z(z: u32) -> Option<Species> {
    if (1..=10).contains(&z) {
        Some(FIRST_ROW[(z - 1) as usize])
    } else {
        None
    }
}

/// Look an element up by symbol. Case-sensitive, because "CO" is a molecule and "Co" is
/// not in the first row.
pub fn by_symbol(sym: &str) -> Option<Species> {
    FIRST_ROW.iter().copied().find(|s| s.symbol == sym)
}

/// Twice the `S_z` sector a species or pair is solved in: 0 for an even electron count,
/// 1 for an odd one.
///
/// # Why the MINIMAL sector and not the physical ground multiplicity
///
/// This is a derivation rather than a convenience. A multiplet of total spin `S` has a
/// component in every sector with `|S_z| <= S`, so the sector with the smallest `|S_z|`
/// consistent with the electron count contains EVERY state of the system, whatever its
/// spin. Solving there therefore cannot miss the ground state — whereas fixing `S_z` to
/// a guessed multiplicity can, and would turn a wrong guess about (say) carbon's term
/// symbol into a wrong energy with nothing to signal it. The cost is that the
/// determinant space is at its largest, which is the price of not having to know the
/// answer in advance.
pub fn sz2_sector(n_electrons: u32) -> u32 {
    n_electrons % 2
}
