//! The species registry: hydrogen through xenon, and the STO-3G basis they are given.
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
//!   2769 (1970) — lithium through neon, and sodium through argon;
//! * W. J. Pietro, E. S. Blurock, R. F. Hout, W. J. Hehre, D. J. DeFrees, R. F. Stewart,
//!   *Inorg. Chem.* **19**, 2225 (1980) — potassium through krypton, and the second fits
//!   of 3s/3p described below;
//! * W. J. Pietro, B. A. Levi, W. J. Hehre, R. F. Stewart, *Inorg. Chem.* **19**, 2225
//!   (1980) and the same programme's later extension — rubidium through xenon.
//!
//! The numbers below are the standard tabulation of those papers as it is distributed
//! today (the Basis Set Exchange's `STO-3G`), pinned in
//! `conformance/atomworld/elements3_sto3g.json`. Three structural facts of that
//! tabulation are worth stating because they are also the cheapest check on a
//! transcription error, and `tests/elements.rs` asserts all three:
//!
//! * the CONTRACTION COEFFICIENTS are universal PER FIT — every element's 1s shell shares
//!   one triple, every 2s shell shares another, and so on. Only the exponents move with
//!   the element. That is the design of STO-3G (a fixed expansion of a Slater function,
//!   rescaled), not a coincidence. The qualifier "per fit" is load-bearing: 3s, 3p, 4s and
//!   4p were each fitted TWICE, once where the shell is valence and again where it has
//!   become core, so those four have two triples apiece. See [`C_3S_HEAVY`].
//! * an s shell and its p partner share their EXPONENTS (the "sp shell"), and differ only
//!   in coefficients. From scandium down the d function joins the same set for some rows.
//! * the EXPONENT RATIOS within a shell are element-independent to the precision the
//!   declaration carries, because rescaling one fit cannot change them. This is the check
//!   that caught the oxygen defect, and `tests/elements.rs` runs it over every fit family.
//!
//! # DECLARED: d shells are five spherical components, not six Cartesian
//!
//! The basis is one of the three declared inputs, so the COMPONENT CONVENTION is part of
//! the declaration and is stated here rather than left to be inferred from whichever
//! integral routine a reader happens to open. **A d shell contributes five functions: the
//! five real solid harmonics.**
//!
//! The integral recursions are Cartesian and evaluate a d shell as six components, but the
//! six do not span an `l = 2` space -- they span the five plus
//! `(x^2 + y^2 + z^2) exp(-a r^2)`, which is spherically symmetric and therefore `l = 0`.
//! `md::SPHERICAL_D` projects that sixth function out, and `ShellKind::n_functions` reports
//! five because five is what the basis has.
//!
//! Leaving this implicit is exactly how it went wrong. The convention was never written
//! down, the engine carried six, and ELEMENTS-3's freeze had been written against five --
//! its own arithmetic (xenon's atom at ONE determinant, Br2 at ~1.3e3, Xe2 at 54 orbitals)
//! is derivable under no other convention. Under six, krypton is not a closed shell at all.
//! See AMENDMENT A1.1 of `conformance/atomworld/ELEMENTS3_PREREG.md`.
//!
//! # The rows below argon are generated, not typed
//!
//! Z = 19..54 adds 130 shells and some eight hundred declared digits. At that volume a
//! transcription error is not a risk to be managed but a defect to be scheduled, so the
//! block is emitted by `conformance/atomworld/elements3_transcribe.py` from the pinned
//! tabulation and the pinned NIST mass table, and the tests check the emitted numbers back
//! against both. What a reviewer reads is the generator and the gates, not the digits.
//!
//! # One isotope is a choice rather than a measurement
//!
//! Every `isotope` here is the most abundant one EXCEPT technetium (Z = 43), which has no
//! stable isotope at all: its natural abundance is zero and the declared 97Tc is a
//! representative choice. The mass plays no part in the electronic structure, so nothing
//! computed here depends on it, but a record that let "most abundant" stand for an element
//! that has no abundance would be stating something untrue for free.
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

/// The universal STO-3G 3s contraction coefficients.
pub const C_3S: [f64; 3] = [-0.21962037, 0.22559543, 0.90039843];

/// The universal STO-3G 3p contraction coefficients.
pub const C_3P: [f64; 3] = [0.01058760, 0.59516701, 0.46200101];

/// The universal STO-3G 3d contraction coefficients.
pub const C_3D: [f64; 3] = [0.21976795, 0.65554736, 0.28657326];

// --- the second fit of an already-fitted shell -----------------------------------
//
// STO-3G's coefficients are universal per (n, l) FIT, and 3s, 3p, 4s and 4p were each
// fitted TWICE: once for the row where the shell is the valence shell, and again for the
// rows below it where the same shell has become core. The tabulation carries both, and
// they are genuinely different numbers rather than a rounding of one another — the 3s
// leading coefficient moves from -0.21962037 to -0.22776350.
//
// This is worth stating because the obvious reading of "the coefficients are universal"
// is that ONE triple per shell type serves the whole table, and a gate written to that
// reading would fire honestly on correct data. The universality that actually holds is
// per fit family, and `tests/elements.rs` checks the partition the tabulation has rather
// than the tidier one it does not.

/// The STO-3G 3s coefficients for the rows where 3s is a CORE shell (Sc..Xe).
pub const C_3S_HEAVY: [f64; 3] = [-0.22776350, 0.21754360, 0.91667696];

/// The STO-3G 3p coefficients for the rows where 3p is a CORE shell (Sc..Xe).
pub const C_3P_HEAVY: [f64; 3] = [0.00495151, 0.57776647, 0.48464604];

/// The universal STO-3G 4s contraction coefficients, K..Sr.
pub const C_4S: [f64; 3] = [-0.30884412, 0.01960641, 1.13103444];

/// The universal STO-3G 4p contraction coefficients, K..Sr.
pub const C_4P: [f64; 3] = [-0.12154686, 0.57152276, 0.54989495];

/// The STO-3G 4s coefficients for the rows where 4s is a CORE shell (Y..Xe).
pub const C_4S_HEAVY: [f64; 3] = [-0.33061006, 0.05761095, 1.11557874];

/// The STO-3G 4p coefficients for the rows where 4p is a CORE shell (Y..Xe).
pub const C_4P_HEAVY: [f64; 3] = [-0.12839276, 0.58520476, 0.54394420];

/// The universal STO-3G 4d contraction coefficients.
pub const C_4D: [f64; 3] = [0.12506621, 0.66867856, 0.30524682];

/// The universal STO-3G 5s contraction coefficients. Two negative entries: a 5s orbital
/// has two radial nodes and the contraction builds both.
pub const C_5S: [f64; 3] = [-0.38426426, -0.19725674, 1.37549551];

/// The universal STO-3G 5p contraction coefficients.
pub const C_5P: [f64; 3] = [-0.34816915, 0.62903237, 0.66628327];

/// Which shells an element carries, in the order they enter the basis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellKind {
    /// 1s.
    S1,
    /// 2s.
    S2,
    /// 2p — three Cartesian components, `px`, `py`, `pz`.
    P2,
    /// 3s.
    S3,
    /// 3p — three Cartesian components, `px`, `py`, `pz`.
    P3,
    /// 3d — Cartesian components (xy, yz, zx, x^2-y^2, z^2 or 6 Cartesian components).
    D3,
    /// 4s.
    S4,
    /// 4p.
    P4,
    /// 4d.
    D4,
    /// 5s.
    S5,
    /// 5p.
    P5,
}

impl ShellKind {
    /// Total angular momentum: 0 for s shells, 1 for p shells, 2 for d shells.
    pub fn l(self) -> u8 {
        match self {
            ShellKind::S1 | ShellKind::S2 | ShellKind::S3 | ShellKind::S4 | ShellKind::S5 => 0,
            ShellKind::P2 | ShellKind::P3 | ShellKind::P4 | ShellKind::P5 => 1,
            ShellKind::D3 | ShellKind::D4 => 2,
        }
    }

    /// Principal quantum number. Needed because STO-3G's contraction coefficients are
    /// universal per `(n, l)` FIT and not per shell type alone — see [`C_3S_HEAVY`].
    pub fn n(self) -> u8 {
        match self {
            ShellKind::S1 => 1,
            ShellKind::S2 | ShellKind::P2 => 2,
            ShellKind::S3 | ShellKind::P3 | ShellKind::D3 => 3,
            ShellKind::S4 | ShellKind::P4 | ShellKind::D4 => 4,
            ShellKind::S5 | ShellKind::P5 => 5,
        }
    }

    /// Basis functions this shell contributes: 1 for s, 3 for p, 5 for d.
    ///
    /// # Five for d, not six
    ///
    /// The integral recursions are Cartesian and a d shell computes as six components, but
    /// the sixth is `(x^2+y^2+z^2) exp(-a r^2)` — spherically symmetric, so `l = 0` — and
    /// the engine projects it out (see `md::SPHERICAL_D`). What a shell contributes to the
    /// BASIS is therefore five, and this function answers that question, because every
    /// caller of it is asking how big the problem is rather than how the integrals are
    /// evaluated. `md::cartesian_components` answers the other question.
    ///
    /// These two counts disagreeing is not hypothetical: this function returned six for a
    /// while after the projection landed, which made `pair::feasibility` overstate xenon by
    /// two orbitals and the palette report a basis size the engine never builds.
    /// `tests/spherical_d.rs` now ties it to what `build_basis` actually assembles.
    pub fn n_functions(self) -> usize {
        match self {
            ShellKind::S1 | ShellKind::S2 | ShellKind::S3 | ShellKind::S4 | ShellKind::S5 => 1,
            ShellKind::P2 | ShellKind::P3 | ShellKind::P4 | ShellKind::P5 => 3,
            ShellKind::D3 | ShellKind::D4 => 5,
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

    /// The element's derived homonuclear radius in bohr, from `species_palette.json`.
    pub const fn homonuclear_radius(self) -> f64 {
        match self.z {
            1 => 0.6943470090088878,
            2 => 1.9943329366683962,
            3 => 2.4955224091314747,
            4 => 3.0220328441262247,
            5 => 1.411590082275827,
            6 => 1.1913654992774316,
            7 => 1.1283647365796021,
            8 => 1.2210401490051894,
            9 => 1.3110267541113547,
            10 => 2.0646444203853607,
            _ => 0.6943470090088878,
        }
    }

    /// The element's declared colour as `#rrggbb` hex string, from `species_palette.json`.
    pub const fn colour_hex(self) -> &'static str {
        match self.z {
            1 => "#72c0b0",
            2 => "#6bbf9b",
            3 => "#63bf82",
            4 => "#5cbe66",
            5 => "#62be54",
            6 => "#76be4c",
            7 => "#8dbe44",
            8 => "#a7bc3e",
            9 => "#b9b03a",
            10 => "#b58f36",
            _ => "#72c0b0",
        }
    }

    /// The element's declared colour as linear sRGB `(r, g, b)` floats in `[0.0, 1.0]`.
    pub const fn colour_rgb(self) -> (f32, f32, f32) {
        match self.z {
            1 => (0.44705883, 0.7529412, 0.6901961),
            2 => (0.41960785, 0.7490196, 0.60784316),
            3 => (0.3882353, 0.7490196, 0.50980395),
            4 => (0.36078432, 0.74509805, 0.4),
            5 => (0.38431373, 0.74509805, 0.32941177),
            6 => (0.4627451, 0.74509805, 0.29803923),
            7 => (0.5529412, 0.74509805, 0.26666668),
            8 => (0.654902, 0.7372549, 0.24313726),
            9 => (0.7254902, 0.6901961, 0.22745098),
            10 => (0.70980394, 0.56078434, 0.21176471),
            _ => (0.44705883, 0.7529412, 0.6901961),
        }
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

impl PartialEq for Species {
    fn eq(&self, other: &Self) -> bool {
        self.z == other.z
    }
}

impl Eq for Species {}

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

/// Build the five-shell declaration for second-row elements (Na..Ar).
macro_rules! second_row {
    ($name:ident, $sym:literal, $z:literal, $mass:literal, $iso:literal,
     s1 = $s1:expr, sp2 = $sp2:expr, sp3 = $sp3:expr) => {
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
                    alpha: $sp2,
                    coeff: C_2S,
                },
                Shell {
                    kind: ShellKind::P2,
                    alpha: $sp2,
                    coeff: C_2P,
                },
                Shell {
                    kind: ShellKind::S3,
                    alpha: $sp3,
                    coeff: C_3S,
                },
                Shell {
                    kind: ShellKind::P3,
                    alpha: $sp3,
                    coeff: C_3P,
                },
            ],
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
    s1 = [130.70932140, 23.80886605, 6.44360831],
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

second_row!(
    SODIUM, "Na", 11, 22.9897692820, "23Na",
    s1 = [250.77243000, 45.67851117, 12.36238776],
    sp2 = [12.04019274, 2.79788186, 0.90995802],
    sp3 = [1.47874062, 0.41256488, 0.16147510]
);
second_row!(
    MAGNESIUM, "Mg", 12, 23.985041697, "24Mg",
    s1 = [299.23741370, 54.50646845, 14.75157752],
    sp2 = [15.12182352, 3.51398658, 1.14285750],
    sp3 = [1.39544829, 0.38932653, 0.15237977]
);
second_row!(
    ALUMINUM, "Al", 13, 26.98153853, "27Al",
    s1 = [351.42147670, 64.01186067, 17.32410761],
    sp2 = [18.89939621, 4.39181323, 1.42835397],
    sp3 = [1.39544829, 0.38932653, 0.15237977]
);
second_row!(
    SILICON, "Si", 14, 27.976926535, "28Si",
    s1 = [407.79755140, 74.28083305, 20.10329229],
    sp2 = [23.19365606, 5.38970687, 1.75289995],
    sp3 = [1.47874062, 0.41256488, 0.16147510]
);
second_row!(
    PHOSPHORUS, "P", 15, 30.973761998, "31P",
    s1 = [468.36563780, 85.31338559, 23.08913156],
    sp2 = [28.03263958, 6.51418258, 2.11861435],
    sp3 = [1.74310323, 0.48632138, 0.19034289]
);
second_row!(
    SULFUR, "S", 16, 31.972071174, "32S",
    s1 = [533.12573590, 97.10951830, 26.28162542],
    sp2 = [33.32975173, 7.74511752, 2.51895260],
    sp3 = [2.02919427, 0.56614005, 0.22158338]
);
second_row!(
    CHLORINE, "Cl", 17, 34.968852682, "35Cl",
    s1 = [601.34561360, 109.53585420, 29.64467686],
    sp2 = [38.96041889, 9.05356348, 2.94449983],
    sp3 = [2.12938650, 0.59409343, 0.23252414]
);
second_row!(
    ARGON, "Ar", 18, 39.9623831225, "40Ar",
    s1 = [674.44651840, 122.85127530, 33.24834945],
    sp2 = [45.16424392, 10.49519900, 3.41336445],
    sp3 = [2.62136652, 0.73135461, 0.28624724]
);

// ---------------------------------------------------------------------------------
// Z = 19..54, GENERATED. Do not hand-edit: `conformance/atomworld/elements3_transcribe.py`
// emits this block from the two pinned source files beside it, and `tests/elements.rs`
// checks every number below against them. Eight hundred digits is past the volume at which
// transcription is a risk to be managed rather than a defect to be scheduled, and the
// oxygen defect in this module's header is what one of them costs.
//
// The shells are in ascending (n, l) order, which is NOT the order the Basis Set Exchange
// lists them in: gallium's third listed shell is 4s4p and its fourth is 3s3p3d, because
// STO-3G groups each d function with the sp set that shares its exponents. The generator
// derives (n, l) from the contraction coefficients, which identify the fit, and checks the
// result against aufbau occupancy and against exponent ordering before emitting anything.
// ---------------------------------------------------------------------------------

pub const POTASSIUM: Species = Species {
    symbol: "K",
    z: 19,
    mass_u: 38.9637064864,
    isotope: "39K",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [771.51036810, 140.53157660, 38.03332899],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [52.40203979, 12.17710710, 3.96037316],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [52.40203979, 12.17710710, 3.96037316],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [3.65158398, 1.01878266, 0.39874463],
            coeff: C_3S,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [3.65158398, 1.01878266, 0.39874463],
            coeff: C_3P,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [0.50398225, 0.18600115, 0.08214007],
            coeff: C_4S,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [0.50398225, 0.18600115, 0.08214007],
            coeff: C_4P,
        },
    ],
};
pub const CALCIUM: Species = Species {
    symbol: "Ca",
    z: 20,
    mass_u: 39.962590863,
    isotope: "40Ca",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [854.03249510, 155.56308510, 42.10144179],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [59.56029944, 13.84053270, 4.50137080],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [59.56029944, 13.84053270, 4.50137080],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [4.37470626, 1.22053194, 0.47770793],
            coeff: C_3S,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [4.37470626, 1.22053194, 0.47770793],
            coeff: C_3P,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [0.45584898, 0.16823694, 0.07429521],
            coeff: C_4S,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [0.45584898, 0.16823694, 0.07429521],
            coeff: C_4P,
        },
    ],
};
pub const SCANDIUM: Species = Species {
    symbol: "Sc",
    z: 21,
    mass_u: 44.95590828,
    isotope: "45Sc",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [941.66242500, 171.52498620, 46.42135516],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [67.17668771, 15.61041754, 5.07699228],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [67.17668771, 15.61041754, 5.07699228],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [4.69815923, 1.43308831, 0.55293002],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [4.69815923, 1.43308831, 0.55293002],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [0.55170007, 0.16828611, 0.06493001],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [0.63093284, 0.23285390, 0.10283074],
            coeff: C_4S,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [0.63093284, 0.23285390, 0.10283074],
            coeff: C_4P,
        },
    ],
};
pub const TITANIUM: Species = Species {
    symbol: "Ti",
    z: 22,
    mass_u: 47.94794198,
    isotope: "48Ti",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [1033.57124500, 188.26629260, 50.95220601],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [75.25120460, 17.48676162, 5.68723761],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [75.25120460, 17.48676162, 5.68723761],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [5.39553547, 1.64581030, 0.63500478],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [5.39553547, 1.64581030, 0.63500478],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [1.64598119, 0.50207673, 0.19371681],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [0.71226402, 0.26287022, 0.11608626],
            coeff: C_4S,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [0.71226402, 0.26287022, 0.11608626],
            coeff: C_4P,
        },
    ],
};
pub const VANADIUM: Species = Species {
    symbol: "V",
    z: 23,
    mass_u: 50.94395704,
    isotope: "51V",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [1130.76251700, 205.96980410, 55.74346711],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [83.78385011, 19.46956493, 6.33210678],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [83.78385011, 19.46956493, 6.33210678],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [6.14115128, 1.87324688, 0.72275688],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [6.14115128, 1.87324688, 0.72275688],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [2.96481793, 0.90436397, 0.34893173],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [0.71226402, 0.26287022, 0.11608626],
            coeff: C_4S,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [0.71226402, 0.26287022, 0.11608626],
            coeff: C_4P,
        },
    ],
};
pub const CHROMIUM: Species = Species {
    symbol: "Cr",
    z: 24,
    mass_u: 51.94050623,
    isotope: "52Cr",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [1232.32045000, 224.46870820, 60.74999251],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [92.77462423, 21.55882749, 7.01159981],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [92.77462423, 21.55882749, 7.01159981],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [6.89948810, 2.10456378, 0.81200613],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [6.89948810, 2.10456378, 0.81200613],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [4.24147924, 1.29378636, 0.49918300],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [0.75477805, 0.27856057, 0.12301529],
            coeff: C_4S,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [0.75477805, 0.27856057, 0.12301529],
            coeff: C_4P,
        },
    ],
};
pub const MANGANESE: Species = Species {
    symbol: "Mn",
    z: 25,
    mass_u: 54.93804391,
    isotope: "55Mn",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [1337.15326600, 243.56413650, 65.91796062],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [102.02200210, 23.70771923, 7.71048610],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [102.02200210, 23.70771923, 7.71048610],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [7.70196092, 2.34934357, 0.90644979],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [7.70196092, 2.34934357, 0.90644979],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [5.42695046, 1.65539287, 0.63870203],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [0.67098229, 0.24763466, 0.10935808],
            coeff: C_4S,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [0.67098229, 0.24763466, 0.10935808],
            coeff: C_4P,
        },
    ],
};
pub const IRON: Species = Species {
    symbol: "Fe",
    z: 26,
    mass_u: 55.93493633,
    isotope: "56Fe",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [1447.40041100, 263.64579160, 71.35284019],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [111.91948910, 26.00768236, 8.45850549],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [111.91948910, 26.00768236, 8.45850549],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [8.54856975, 2.60758625, 1.00608784],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [8.54856975, 2.60758625, 1.00608784],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [6.41180348, 1.95580443, 0.75461015],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [0.59211568, 0.21852793, 0.09650424],
            coeff: C_4S,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [0.59211568, 0.21852793, 0.09650424],
            coeff: C_4P,
        },
    ],
};
pub const COBALT: Species = Species {
    symbol: "Co",
    z: 27,
    mass_u: 58.93319429,
    isotope: "59Co",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [1560.83467000, 284.30798350, 76.94483567],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [122.27510470, 28.41410473, 9.24114873],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [122.27510470, 28.41410473, 9.24114873],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [9.43931459, 2.87929182, 1.11092030],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [9.43931459, 2.87929182, 1.11092030],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [7.66452739, 2.33792515, 0.90204421],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [0.59211568, 0.21852793, 0.09650424],
            coeff: C_4S,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [0.59211568, 0.21852793, 0.09650424],
            coeff: C_4P,
        },
    ],
};
pub const NICKEL: Species = Species {
    symbol: "Ni",
    z: 28,
    mass_u: 57.93534241,
    isotope: "58Ni",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [1679.77102800, 305.97238960, 82.80806943],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [132.85888990, 30.87354878, 10.04103627],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [132.85888990, 30.87354878, 10.04103627],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [10.33074335, 3.15120600, 1.21583324],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [10.33074335, 3.15120600, 1.21583324],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [8.62772276, 2.63173044, 1.01540342],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [0.63093284, 0.23285390, 0.10283074],
            coeff: C_4S,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [0.63093284, 0.23285390, 0.10283074],
            coeff: C_4P,
        },
    ],
};
pub const COPPER: Species = Species {
    symbol: "Cu",
    z: 29,
    mass_u: 62.92959772,
    isotope: "63Cu",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [1801.80673000, 328.20134500, 88.82409228],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [144.12121840, 33.49067173, 10.89220588],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [144.12121840, 33.49067173, 10.89220588],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [11.30775402, 3.44922540, 1.33081839],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [11.30775402, 3.44922540, 1.33081839],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [9.64791193, 2.94292065, 1.13547028],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [0.63093284, 0.23285390, 0.10283074],
            coeff: C_4S,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [0.63093284, 0.23285390, 0.10283074],
            coeff: C_4P,
        },
    ],
};
pub const ZINC: Species = Species {
    symbol: "Zn",
    z: 30,
    mass_u: 63.92914201,
    isotope: "64Zn",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [1929.43230100, 351.44850210, 95.11568021],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [155.84167550, 36.21425391, 11.77799934],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [155.84167550, 36.21425391, 11.77799934],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [12.28152744, 3.74625733, 1.44542254],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [12.28152744, 3.74625733, 1.44542254],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [10.94737077, 3.33929702, 1.28840460],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [0.88971389, 0.32836038, 0.14500741],
            coeff: C_4S,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [0.88971389, 0.32836038, 0.14500741],
            coeff: C_4P,
        },
    ],
};
pub const GALLIUM: Species = Species {
    symbol: "Ga",
    z: 31,
    mass_u: 68.9255735,
    isotope: "69Ga",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [2061.42453200, 375.49105170, 101.62253240],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [167.76186800, 38.98425028, 12.67888813],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [167.76186800, 38.98425028, 12.67888813],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [12.61505520, 3.84799393, 1.48467568],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [12.61505520, 3.84799393, 1.48467568],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [12.61505520, 3.84799393, 1.48467568],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [0.79852437, 0.29470571, 0.13014515],
            coeff: C_4S,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [0.79852437, 0.29470571, 0.13014515],
            coeff: C_4P,
        },
    ],
};
pub const GERMANIUM: Species = Species {
    symbol: "Ge",
    z: 32,
    mass_u: 73.921177761,
    isotope: "74Ge",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [2196.38422900, 400.07412920, 108.27567260],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [180.38903800, 41.91853304, 13.63320795],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [180.38903800, 41.91853304, 13.63320795],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [14.19665619, 4.33043264, 1.67081554],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [14.19665619, 4.33043264, 1.67081554],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [14.19665619, 4.33043264, 1.67081554],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [0.98583256, 0.36383422, 0.16067303],
            coeff: C_4S,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [0.98583256, 0.36383422, 0.16067303],
            coeff: C_4P,
        },
    ],
};
pub const ARSENIC: Species = Species {
    symbol: "As",
    z: 33,
    mass_u: 74.92159457,
    isotope: "75As",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [2337.06567300, 425.69942980, 115.21087900],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [193.19705350, 44.89484040, 14.60119548],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [193.19705350, 44.89484040, 14.60119548],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [15.87163584, 4.84135482, 1.86794520],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [15.87163584, 4.84135482, 1.86794520],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [15.87163584, 4.84135482, 1.86794520],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [1.10768146, 0.40880412, 0.18053221],
            coeff: C_4S,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [1.10768146, 0.40880412, 0.18053221],
            coeff: C_4P,
        },
    ],
};
pub const SELENIUM: Species = Species {
    symbol: "Se",
    z: 34,
    mass_u: 79.9165218,
    isotope: "80Se",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [2480.62681400, 451.84927080, 122.28804640],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [206.15787800, 47.90665727, 15.58073180],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [206.15787800, 47.90665727, 15.58073180],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [17.63999414, 5.38076046, 2.07606467],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [17.63999414, 5.38076046, 2.07606467],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [17.63999414, 5.38076046, 2.07606467],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [1.21464430, 0.44828014, 0.19796523],
            coeff: C_4S,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [1.21464430, 0.44828014, 0.19796523],
            coeff: C_4P,
        },
    ],
};
pub const BROMINE: Species = Species {
    symbol: "Br",
    z: 35,
    mass_u: 78.9183376,
    isotope: "79Br",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [2629.99747100, 479.05732240, 129.65160700],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [219.83502550, 51.08493222, 16.61440546],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [219.83502550, 51.08493222, 16.61440546],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [19.50173109, 5.94864958, 2.29517394],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [19.50173109, 5.94864958, 2.29517394],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [19.50173109, 5.94864958, 2.29517394],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [1.39603749, 0.51522563, 0.22752907],
            coeff: C_4S,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [1.39603749, 0.51522563, 0.22752907],
            coeff: C_4P,
        },
    ],
};
pub const KRYPTON: Species = Species {
    symbol: "Kr",
    z: 36,
    mass_u: 83.9114977282,
    isotope: "84Kr",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [2782.16005500, 506.77392700, 137.15280190],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [233.95141180, 54.36527681, 17.68127533],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [233.95141180, 54.36527681, 17.68127533],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [21.45684671, 6.54502216, 2.52527302],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [21.45684671, 6.54502216, 2.52527302],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [21.45684671, 6.54502216, 2.52527302],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [1.59004934, 0.58682821, 0.25914952],
            coeff: C_4S,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [1.59004934, 0.58682821, 0.25914952],
            coeff: C_4P,
        },
    ],
};
pub const RUBIDIUM: Species = Species {
    symbol: "Rb",
    z: 37,
    mass_u: 84.9117897379,
    isotope: "85Rb",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [2938.60152900, 535.26993680, 144.86493420],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [248.50703690, 57.74769105, 18.78134142],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [248.50703690, 57.74769105, 18.78134142],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [23.50534097, 7.16987820, 2.76636191],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [23.50534097, 7.16987820, 2.76636191],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [23.50534097, 7.16987820, 2.76636191],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [2.24779682, 0.82957839, 0.36635057],
            coeff: C_4S,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [2.24779682, 0.82957839, 0.36635057],
            coeff: C_4P,
        },
        Shell {
            kind: ShellKind::S5,
            alpha: [0.48699399, 0.26221616, 0.11582549],
            coeff: C_5S,
        },
        Shell {
            kind: ShellKind::P5,
            alpha: [0.48699399, 0.26221616, 0.11582549],
            coeff: C_5P,
        },
    ],
};
pub const STRONTIUM: Species = Species {
    symbol: "Sr",
    z: 38,
    mass_u: 87.9056125,
    isotope: "88Sr",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [3100.98395100, 564.84809780, 152.86993890],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [263.50190070, 61.23217493, 19.91460372],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [263.50190070, 61.23217493, 19.91460372],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [25.57886692, 7.80236971, 3.01039679],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [25.57886692, 7.80236971, 3.01039679],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [25.57886692, 7.80236971, 3.01039679],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [2.46103240, 0.90827573, 0.40110414],
            coeff: C_4S,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [2.46103240, 0.90827573, 0.40110414],
            coeff: C_4P,
        },
        Shell {
            kind: ShellKind::S5,
            alpha: [0.43708048, 0.23534082, 0.10395418],
            coeff: C_5S,
        },
        Shell {
            kind: ShellKind::P5,
            alpha: [0.43708048, 0.23534082, 0.10395418],
            coeff: C_5P,
        },
    ],
};
pub const YTTRIUM: Species = Species {
    symbol: "Y",
    z: 39,
    mass_u: 88.9058403,
    isotope: "89Y",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [3266.02686900, 594.91087120, 161.00609860],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [277.93772440, 64.58674989, 21.00561561],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [277.93772440, 64.58674989, 21.00561561],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [28.96238417, 8.83445031, 3.40860558],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [28.96238417, 8.83445031, 3.40860558],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [28.96238417, 8.83445031, 3.40860558],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [2.52727488, 0.98410774, 0.43320665],
            coeff: C_4S_HEAVY,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [2.52727488, 0.98410774, 0.43320665],
            coeff: C_4P_HEAVY,
        },
        Shell {
            kind: ShellKind::D4,
            alpha: [0.45763239, 0.17819968, 0.07844394],
            coeff: C_4D,
        },
        Shell {
            kind: ShellKind::S5,
            alpha: [0.43708048, 0.23534082, 0.10395418],
            coeff: C_5S,
        },
        Shell {
            kind: ShellKind::P5,
            alpha: [0.43708048, 0.23534082, 0.10395418],
            coeff: C_5P,
        },
    ],
};
pub const ZIRCONIUM: Species = Species {
    symbol: "Zr",
    z: 40,
    mass_u: 89.9046977,
    isotope: "90Zr",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [3435.34867700, 625.75304980, 169.35319580],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [293.78302920, 68.26885797, 22.20315144],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [293.78302920, 68.26885797, 22.20315144],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [30.73293103, 9.37452354, 3.61698262],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [30.73293103, 9.37452354, 3.61698262],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [30.73293103, 9.37452354, 3.61698262],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [2.82760782, 1.10105583, 0.48468749],
            coeff: C_4S_HEAVY,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [2.82760782, 1.10105583, 0.48468749],
            coeff: C_4P_HEAVY,
        },
        Shell {
            kind: ShellKind::D4,
            alpha: [0.88783019, 0.34571647, 0.15218524],
            coeff: C_4D,
        },
        Shell {
            kind: ShellKind::S5,
            alpha: [0.48699399, 0.26221616, 0.11582549],
            coeff: C_5S,
        },
        Shell {
            kind: ShellKind::P5,
            alpha: [0.48699399, 0.26221616, 0.11582549],
            coeff: C_5P,
        },
    ],
};
pub const NIOBIUM: Species = Species {
    symbol: "Nb",
    z: 41,
    mass_u: 92.906373,
    isotope: "93Nb",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [3610.74286400, 657.70132010, 177.99964450],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [310.06757280, 72.05303569, 23.43388348],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [310.06757280, 72.05303569, 23.43388348],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [33.01997858, 10.07214594, 3.88614703],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [33.01997858, 10.07214594, 3.88614703],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [33.01997858, 10.07214594, 3.88614703],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [3.14479843, 1.22456821, 0.53905794],
            coeff: C_4S_HEAVY,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [3.14479843, 1.22456821, 0.53905794],
            coeff: C_4P_HEAVY,
        },
        Shell {
            kind: ShellKind::D4,
            alpha: [1.34487887, 0.52368886, 0.23052913],
            coeff: C_4D,
        },
        Shell {
            kind: ShellKind::S5,
            alpha: [0.48699399, 0.26221616, 0.11582549],
            coeff: C_5S,
        },
        Shell {
            kind: ShellKind::P5,
            alpha: [0.48699399, 0.26221616, 0.11582549],
            coeff: C_5P,
        },
    ],
};
pub const MOLYBDENUM: Species = Species {
    symbol: "Mo",
    z: 42,
    mass_u: 97.90540482,
    isotope: "98Mo",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [3788.66611500, 690.11026230, 186.77076910],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [326.43095670, 75.85553420, 24.67057401],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [326.43095670, 75.85553420, 24.67057401],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [35.46948129, 10.81932234, 4.17443091],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [35.46948129, 10.81932234, 4.17443091],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [35.46948129, 10.81932234, 4.17443091],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [3.49689519, 1.36167286, 0.59941175],
            coeff: C_4S_HEAVY,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [3.49689519, 1.36167286, 0.59941175],
            coeff: C_4P_HEAVY,
        },
        Shell {
            kind: ShellKind::D4,
            alpha: [1.70211232, 0.66279371, 0.29176342],
            coeff: C_4D,
        },
        Shell {
            kind: ShellKind::S5,
            alpha: [0.51296251, 0.27619860, 0.12200178],
            coeff: C_5S,
        },
        Shell {
            kind: ShellKind::P5,
            alpha: [0.51296251, 0.27619860, 0.12200178],
            coeff: C_5P,
        },
    ],
};
pub const TECHNETIUM: Species = Species {
    symbol: "Tc",
    z: 43,
    mass_u: 96.9063667,
    isotope: "97Tc",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [3970.86825700, 723.29860980, 195.75283110],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [343.58463230, 79.84167952, 25.96699219],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [343.58463230, 79.84167952, 25.96699219],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [38.08991983, 11.61863962, 4.48283237],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [38.08991983, 11.61863962, 4.48283237],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [38.08991983, 11.61863962, 4.48283237],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [3.82975271, 1.49128585, 0.65646770],
            coeff: C_4S_HEAVY,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [3.82975271, 1.49128585, 0.65646770],
            coeff: C_4P_HEAVY,
        },
        Shell {
            kind: ShellKind::D4,
            alpha: [2.10137323, 0.81826384, 0.36020176],
            coeff: C_4D,
        },
        Shell {
            kind: ShellKind::S5,
            alpha: [0.46169998, 0.24859690, 0.10980962],
            coeff: C_5S,
        },
        Shell {
            kind: ShellKind::P5,
            alpha: [0.46169998, 0.24859690, 0.10980962],
            coeff: C_5P,
        },
    ],
};
pub const RUTHENIUM: Species = Species {
    symbol: "Ru",
    z: 44,
    mass_u: 101.9043441,
    isotope: "102Ru",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [4159.27421000, 757.61698940, 205.04072390],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [360.79865610, 83.84184843, 27.26797127],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [360.79865610, 83.84184843, 27.26797127],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [40.71751678, 12.42014044, 4.79207630],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [40.71751678, 12.42014044, 4.79207630],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [40.71751678, 12.42014044, 4.79207630],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [4.19751637, 1.63449112, 0.71950701],
            coeff: C_4S_HEAVY,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [4.19751637, 1.63449112, 0.71950701],
            coeff: C_4P_HEAVY,
        },
        Shell {
            kind: ShellKind::D4,
            alpha: [2.39089576, 0.93100242, 0.40982956],
            coeff: C_4D,
        },
        Shell {
            kind: ShellKind::S5,
            alpha: [0.41313548, 0.22244792, 0.09825916],
            coeff: C_5S,
        },
        Shell {
            kind: ShellKind::P5,
            alpha: [0.41313548, 0.22244792, 0.09825916],
            coeff: C_5P,
        },
    ],
};
pub const RHODIUM: Species = Species {
    symbol: "Rh",
    z: 45,
    mass_u: 102.905498,
    isotope: "103Rh",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [4350.07779400, 792.37210050, 214.44681330],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [378.43342640, 87.93978981, 28.60074899],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [378.43342640, 87.93978981, 28.60074899],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [43.52179455, 13.27553454, 5.12211394],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [43.52179455, 13.27553454, 5.12211394],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [43.52179455, 13.27553454, 5.12211394],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [4.54085741, 1.76818634, 0.77835998],
            coeff: C_4S_HEAVY,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [4.54085741, 1.76818634, 0.77835998],
            coeff: C_4P_HEAVY,
        },
        Shell {
            kind: ShellKind::D4,
            alpha: [2.77906609, 1.08215393, 0.47636682],
            coeff: C_4D,
        },
        Shell {
            kind: ShellKind::S5,
            alpha: [0.41313548, 0.22244792, 0.09825916],
            coeff: C_5S,
        },
        Shell {
            kind: ShellKind::P5,
            alpha: [0.41313548, 0.22244792, 0.09825916],
            coeff: C_5P,
        },
    ],
};
pub const PALLADIUM: Species = Species {
    symbol: "Pd",
    z: 46,
    mass_u: 105.9034804,
    isotope: "106Pd",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [4545.16026900, 827.90661680, 224.06384020],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [396.48894330, 92.13550365, 29.96532535],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [396.48894330, 92.13550365, 29.96532535],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [46.41945097, 14.15941211, 5.46314138],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [46.41945097, 14.15941211, 5.46314138],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [46.41945097, 14.15941211, 5.46314138],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [4.91910459, 1.91547383, 0.84319630],
            coeff: C_4S_HEAVY,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [4.91910459, 1.91547383, 0.84319630],
            coeff: C_4P_HEAVY,
        },
        Shell {
            kind: ShellKind::D4,
            alpha: [3.02597745, 1.17829993, 0.51869053],
            coeff: C_4D,
        },
        Shell {
            kind: ShellKind::S5,
            alpha: [0.43708048, 0.23534082, 0.10395418],
            coeff: C_5S,
        },
        Shell {
            kind: ShellKind::P5,
            alpha: [0.43708048, 0.23534082, 0.10395418],
            coeff: C_5P,
        },
    ],
};
pub const SILVER: Species = Species {
    symbol: "Ag",
    z: 47,
    mass_u: 106.9050916,
    isotope: "107Ag",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [4744.52163400, 864.22053830, 233.89180450],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [414.96520690, 96.42898995, 31.36170035],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [414.96520690, 96.42898995, 31.36170035],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [49.41048605, 15.07177314, 5.81515863],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [49.41048605, 15.07177314, 5.81515863],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [49.41048605, 15.07177314, 5.81515863],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [5.29023045, 2.05998832, 0.90681193],
            coeff: C_4S_HEAVY,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [5.29023045, 2.05998832, 0.90681193],
            coeff: C_4P_HEAVY,
        },
        Shell {
            kind: ShellKind::D4,
            alpha: [3.28339567, 1.27853725, 0.56281525],
            coeff: C_4D,
        },
        Shell {
            kind: ShellKind::S5,
            alpha: [0.43708048, 0.23534082, 0.10395418],
            coeff: C_5S,
        },
        Shell {
            kind: ShellKind::P5,
            alpha: [0.43708048, 0.23534082, 0.10395418],
            coeff: C_5P,
        },
    ],
};
pub const CADMIUM: Species = Species {
    symbol: "Cd",
    z: 48,
    mass_u: 113.90336509,
    isotope: "114Cd",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [4950.26190500, 901.69638560, 244.03423130],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [433.44693850, 100.72374690, 32.75848861],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [433.44693850, 100.72374690, 32.75848861],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [52.59279235, 16.04247800, 6.18968674],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [52.59279235, 16.04247800, 6.18968674],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [52.59279235, 16.04247800, 6.18968674],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [5.67485180, 2.20975788, 0.97274086],
            coeff: C_4S_HEAVY,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [5.67485180, 2.20975788, 0.97274086],
            coeff: C_4P_HEAVY,
        },
        Shell {
            kind: ShellKind::D4,
            alpha: [3.64296398, 1.41855129, 0.62444977],
            coeff: C_4D,
        },
        Shell {
            kind: ShellKind::S5,
            alpha: [0.59491510, 0.32032500, 0.14149319],
            coeff: C_5S,
        },
        Shell {
            kind: ShellKind::P5,
            alpha: [0.59491510, 0.32032500, 0.14149319],
            coeff: C_5P,
        },
    ],
};
pub const INDIUM: Species = Species {
    symbol: "In",
    z: 49,
    mass_u: 114.903878776,
    isotope: "115In",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [5158.22471400, 939.57707070, 254.28622310],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [452.33132230, 105.11207160, 34.18570799],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [452.33132230, 105.11207160, 34.18570799],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [55.97539769, 17.07428044, 6.58778820],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [55.97539769, 17.07428044, 6.58778820],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [55.97539769, 17.07428044, 6.58778820],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [5.04854918, 1.96587888, 0.86538472],
            coeff: C_4S_HEAVY,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [5.04854918, 1.96587888, 0.86538472],
            coeff: C_4P_HEAVY,
        },
        Shell {
            kind: ShellKind::D4,
            alpha: [5.04854918, 1.96587888, 0.86538472],
            coeff: C_4D,
        },
        Shell {
            kind: ShellKind::S5,
            alpha: [0.56692306, 0.30525302, 0.13483563],
            coeff: C_5S,
        },
        Shell {
            kind: ShellKind::P5,
            alpha: [0.56692306, 0.30525302, 0.13483563],
            coeff: C_5P,
        },
    ],
};
pub const TIN: Species = Species {
    symbol: "Sn",
    z: 50,
    mass_u: 119.90220163,
    isotope: "120Sn",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [5370.46641300, 978.23716110, 264.74915220],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [472.05153220, 109.69462430, 35.67609636],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [472.05153220, 109.69462430, 35.67609636],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [59.15141188, 18.04306600, 6.96157579],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [59.15141188, 18.04306600, 6.96157579],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [59.15141188, 18.04306600, 6.96157579],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [5.58313853, 2.17404520, 0.95702005],
            coeff: C_4S_HEAVY,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [5.58313853, 2.17404520, 0.95702005],
            coeff: C_4P_HEAVY,
        },
        Shell {
            kind: ShellKind::D4,
            alpha: [5.58313853, 2.17404520, 0.95702005],
            coeff: C_4D,
        },
        Shell {
            kind: ShellKind::S5,
            alpha: [0.62358164, 0.33576016, 0.14831117],
            coeff: C_5S,
        },
        Shell {
            kind: ShellKind::P5,
            alpha: [0.62358164, 0.33576016, 0.14831117],
            coeff: C_5P,
        },
    ],
};
pub const ANTIMONY: Species = Species {
    symbol: "Sb",
    z: 51,
    mass_u: 120.903812,
    isotope: "121Sb",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [5586.98700200, 1017.67665700, 275.42301890],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [492.19248880, 114.37494940, 37.19828336],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [492.19248880, 114.37494940, 37.19828336],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [62.52179775, 19.07114112, 7.35823913],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [62.52179775, 19.07114112, 7.35823913],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [62.52179775, 19.07114112, 7.35823913],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [6.12069315, 2.38336619, 1.04916366],
            coeff: C_4S_HEAVY,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [6.12069315, 2.38336619, 1.04916366],
            coeff: C_4P_HEAVY,
        },
        Shell {
            kind: ShellKind::D4,
            alpha: [6.12069315, 2.38336619, 1.04916366],
            coeff: C_4D,
        },
        Shell {
            kind: ShellKind::S5,
            alpha: [0.65292269, 0.35155850, 0.15528957],
            coeff: C_5S,
        },
        Shell {
            kind: ShellKind::P5,
            alpha: [0.65292269, 0.35155850, 0.15528957],
            coeff: C_5P,
        },
    ],
};
pub const TELLURIUM: Species = Species {
    symbol: "Te",
    z: 52,
    mass_u: 129.906222748,
    isotope: "130Te",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [5810.06159100, 1058.30997200, 286.41997970],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [512.75419200, 119.15304710, 38.75226900],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [512.75419200, 119.15304710, 38.75226900],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [65.98556227, 20.12769970, 7.76589228],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [65.98556227, 20.12769970, 7.76589228],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [65.98556227, 20.12769970, 7.76589228],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [6.70795692, 2.61204366, 1.14982805],
            coeff: C_4S_HEAVY,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [6.70795692, 2.61204366, 1.14982805],
            coeff: C_4P_HEAVY,
        },
        Shell {
            kind: ShellKind::D4,
            alpha: [6.70795692, 2.61204366, 1.14982805],
            coeff: C_4D,
        },
        Shell {
            kind: ShellKind::S5,
            alpha: [0.70127135, 0.37759127, 0.16678870],
            coeff: C_5S,
        },
        Shell {
            kind: ShellKind::P5,
            alpha: [0.70127135, 0.37759127, 0.16678870],
            coeff: C_5P,
        },
    ],
};
pub const IODINE: Species = Species {
    symbol: "I",
    z: 53,
    mass_u: 126.9044719,
    isotope: "127I",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [6035.18362300, 1099.31623100, 297.51787370],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [533.73664180, 124.02891710, 40.33805328],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [533.73664180, 124.02891710, 40.33805328],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [69.54270545, 21.21274175, 8.18453523],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [69.54270545, 21.21274175, 8.18453523],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [69.54270545, 21.21274175, 8.18453523],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [7.29599120, 2.84102115, 1.25062451],
            coeff: C_4S_HEAVY,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [7.29599120, 2.84102115, 1.25062451],
            coeff: C_4P_HEAVY,
        },
        Shell {
            kind: ShellKind::D4,
            alpha: [7.29599120, 2.84102115, 1.25062451],
            coeff: C_4D,
        },
        Shell {
            kind: ShellKind::S5,
            alpha: [0.79003646, 0.42538579, 0.18790038],
            coeff: C_5S,
        },
        Shell {
            kind: ShellKind::P5,
            alpha: [0.79003646, 0.42538579, 0.18790038],
            coeff: C_5P,
        },
    ],
};
pub const XENON: Species = Species {
    symbol: "Xe",
    z: 54,
    mass_u: 131.9041550856,
    isotope: "132Xe",
    shells: &[
        Shell {
            kind: ShellKind::S1,
            alpha: [6264.58454600, 1141.10189500, 308.82670520],
            coeff: C_1S,
        },
        Shell {
            kind: ShellKind::S2,
            alpha: [555.13983810, 129.00255970, 41.95563620],
            coeff: C_2S,
        },
        Shell {
            kind: ShellKind::P2,
            alpha: [555.13983810, 129.00255970, 41.95563620],
            coeff: C_2P,
        },
        Shell {
            kind: ShellKind::S3,
            alpha: [73.07773504, 22.29103845, 8.60057562],
            coeff: C_3S_HEAVY,
        },
        Shell {
            kind: ShellKind::P3,
            alpha: [73.07773504, 22.29103845, 8.60057562],
            coeff: C_3P_HEAVY,
        },
        Shell {
            kind: ShellKind::D3,
            alpha: [73.07773504, 22.29103845, 8.60057562],
            coeff: C_3D,
        },
        Shell {
            kind: ShellKind::S4,
            alpha: [7.90872828, 3.07961780, 1.35565534],
            coeff: C_4S_HEAVY,
        },
        Shell {
            kind: ShellKind::P4,
            alpha: [7.90872828, 3.07961780, 1.35565534],
            coeff: C_4P_HEAVY,
        },
        Shell {
            kind: ShellKind::D4,
            alpha: [7.90872828, 3.07961780, 1.35565534],
            coeff: C_4D,
        },
        Shell {
            kind: ShellKind::S5,
            alpha: [0.89101014, 0.47975388, 0.21191572],
            coeff: C_5S,
        },
        Shell {
            kind: ShellKind::P5,
            alpha: [0.89101014, 0.47975388, 0.21191572],
            coeff: C_5P,
        },
    ],
};

/// The first row, indexed by `Z - 1`.
pub const FIRST_ROW: [Species; 10] = [
    HYDROGEN, HELIUM, LITHIUM, BERYLLIUM, BORON, CARBON, NITROGEN, OXYGEN, FLUORINE, NEON,
];

/// The second row, indexed by `Z - 11`.
pub const SECOND_ROW: [Species; 8] = [
    SODIUM, MAGNESIUM, ALUMINUM, SILICON, PHOSPHORUS, SULFUR, CHLORINE, ARGON,
];

/// The third row, indexed by `Z - 19`: potassium through krypton.
pub const THIRD_ROW: [Species; 18] = [
    POTASSIUM, CALCIUM, SCANDIUM, TITANIUM, VANADIUM, CHROMIUM, MANGANESE, IRON, COBALT, NICKEL, COPPER, ZINC, GALLIUM, GERMANIUM, ARSENIC, SELENIUM, BROMINE, KRYPTON,
];

/// The fourth row, indexed by `Z - 37`: rubidium through xenon.
pub const FOURTH_ROW: [Species; 18] = [
    RUBIDIUM, STRONTIUM, YTTRIUM, ZIRCONIUM, NIOBIUM, MOLYBDENUM, TECHNETIUM, RUTHENIUM, RHODIUM, PALLADIUM, SILVER, CADMIUM, INDIUM, TIN, ANTIMONY, TELLURIUM, IODINE, XENON,
];

/// All registered elements, indexed by `Z - 1`. Hydrogen through xenon.
pub const ALL_ELEMENTS: [Species; 54] = [
    HYDROGEN, HELIUM, LITHIUM, BERYLLIUM, BORON, CARBON, NITROGEN, OXYGEN, FLUORINE, NEON,
    SODIUM, MAGNESIUM, ALUMINUM, SILICON, PHOSPHORUS, SULFUR, CHLORINE, ARGON,
    POTASSIUM, CALCIUM, SCANDIUM, TITANIUM, VANADIUM, CHROMIUM, MANGANESE, IRON, COBALT, NICKEL, COPPER, ZINC, GALLIUM, GERMANIUM, ARSENIC, SELENIUM, BROMINE, KRYPTON,
    RUBIDIUM, STRONTIUM, YTTRIUM, ZIRCONIUM, NIOBIUM, MOLYBDENUM, TECHNETIUM, RUTHENIUM, RHODIUM, PALLADIUM, SILVER, CADMIUM, INDIUM, TIN, ANTIMONY, TELLURIUM, IODINE, XENON,
];

/// The heaviest nuclear charge the registry carries. ELEMENTS-3 stops at xenon because the
/// next shell needs f functions (l = 3) and the integral machinery has none — a boundary of
/// the BASIS, stated here rather than left as the length of an array.
pub const MAX_Z: u32 = 54;

/// Look an element up by nuclear charge. `None` outside `1..=MAX_Z`.
pub fn by_z(z: u32) -> Option<Species> {
    if (1..=MAX_Z).contains(&z) {
        Some(ALL_ELEMENTS[(z - 1) as usize])
    } else {
        None
    }
}

/// Look an element up by symbol. Case-sensitive, because "CO" is a molecule and "Co" is
/// not in the table.
pub fn by_symbol(sym: &str) -> Option<Species> {
    ALL_ELEMENTS.iter().copied().find(|s| s.symbol == sym)
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
