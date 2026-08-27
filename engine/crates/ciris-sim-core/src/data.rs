//! Sealed structural data for the eleven kinds. Generated from CIRISOntology's
//! PANEL-2 curated confusion matrix (248 items, licensed instrument): symmetrised,
//! diagonal zeroed, off-diagonal mean normalised to 1. Embedded as constants because
//! this crate is `no_std` and cannot read files.

/// Number of kinds.
pub const N: usize = 11;

/// The maximal choice object's coordinates, in the canonical ontology order.
///
/// The first eleven are artifact-local sites and index [`KINDS`], [`COUPLING`], and
/// [`DEPTH`]. [`ChoiceKind::Record`] is deliberately not a twelfth graph node: it is the
/// frame relation that asks whether an event can still be established from what survives.
/// This is the executable vocabulary of `Core/WrongKind.lean` and
/// `Core/Generator.lean`, not a second domain ontology for the physics engine.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChoiceKind {
    Priorities = 0,
    Rules = 1,
    Manner = 2,
    Identity = 3,
    Confidence = 4,
    Facts = 5,
    Circumstances = 6,
    Process = 7,
    Model = 8,
    Structure = 9,
    Premises = 10,
    Record = 11,
}

/// Default constitutional treatment of a coordinate under variation.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disposition {
    Vary,
    Hold,
    HoldUnlessStudied,
    ReplaceWithReview,
    CannotVary,
    OutOfScope,
}

impl ChoiceKind {
    pub const ALL: [Self; 12] = [
        Self::Priorities,
        Self::Rules,
        Self::Manner,
        Self::Identity,
        Self::Confidence,
        Self::Facts,
        Self::Circumstances,
        Self::Process,
        Self::Model,
        Self::Structure,
        Self::Premises,
        Self::Record,
    ];

    pub const SITES: [Self; N] = [
        Self::Priorities,
        Self::Rules,
        Self::Manner,
        Self::Identity,
        Self::Confidence,
        Self::Facts,
        Self::Circumstances,
        Self::Process,
        Self::Model,
        Self::Structure,
        Self::Premises,
    ];

    /// Index into the K11 artifact-local arrays, or `None` for the Record relation.
    pub const fn site_index(self) -> Option<usize> {
        match self {
            Self::Record => None,
            _ => Some(self as usize),
        }
    }

    pub const fn plain(self) -> &'static str {
        match self {
            Self::Priorities => "Priorities",
            Self::Rules => "Rules",
            Self::Manner => "Manner",
            Self::Identity => "Identity",
            Self::Confidence => "Confidence",
            Self::Facts => "Facts",
            Self::Circumstances => "Circumstances",
            Self::Process => "Process",
            Self::Model => "Model",
            Self::Structure => "Structure",
            Self::Premises => "Premises",
            Self::Record => "Record",
        }
    }

    pub const fn discriminator(self) -> &'static str {
        match self {
            Self::Priorities => "What becomes more important?",
            Self::Rules => "What becomes allowed or required?",
            Self::Manner => "How is the same thing presented or used?",
            Self::Identity => "What is this said to be?",
            Self::Confidence => "How sure are we, and on what standard?",
            Self::Facts => "What claimed fact becomes wrong?",
            Self::Circumstances => "What just happens to differ here?",
            Self::Process => "What steps or ordering change?",
            Self::Model => "What rule or model are we reasoning under?",
            Self::Structure => "How are the pieces put together?",
            Self::Premises => "What are we taking as given?",
            Self::Record => "Can the event still be established from what survives?",
        }
    }

    pub const fn disposition(self) -> Disposition {
        match self {
            Self::Priorities | Self::Premises => Disposition::Vary,
            Self::Rules => Disposition::ReplaceWithReview,
            Self::Manner | Self::Confidence => Disposition::HoldUnlessStudied,
            Self::Structure => Disposition::CannotVary,
            Self::Circumstances => Disposition::OutOfScope,
            Self::Identity | Self::Facts | Self::Process | Self::Model | Self::Record => {
                Disposition::Hold
            }
        }
    }

    pub const fn is_surface(self) -> bool {
        matches!(
            self,
            Self::Rules | Self::Manner | Self::Identity | Self::Facts
        )
    }
}

/// Kind names, in the canonical order used by every array in this crate.
pub const KINDS: [&str; N] = [
    "Priorities", "Rules", "Manner", "Identity", "Confidence", "Facts",
    "Circumstances", "Process", "Model", "Structure", "Premises",
];

/// The measured symmetric coupling matrix `c_ij`. Zero diagonal.
pub const COUPLING: [[f64; N]; N] = [
    [0.0, 1.45444316278, 0.0, 0.0, 0.0, 0.0, 0.0, 1.09083237208, 0.0, 0.0, 0.0],
    [1.45444316278, 0.0, 0.0, 0.0, 7.03421602361, 0.0, 0.0, 0.0, 1.42892661606, 0.727221581388, 3.61264914625],
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.690244212843, 3.21508488614, 9.01631503026, 0.328422649659],
    [0.0, 0.0, 0.0, 0.0, 0.0, 1.19777672229, 0.299444180572, 0.0, 0.0, 0.0, 1.31369059864],
    [0.0, 7.03421602361, 0.0, 0.0, 0.0, 2.40644232387, 0.0, 0.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 1.19777672229, 2.40644232387, 0.0, 7.58693430616, 0.0, 1.37534186796, 0.0, 5.25476239455],
    [0.0, 0.0, 0.0, 0.299444180572, 0.0, 7.58693430616, 0.0, 0.0, 0.0, 0.0, 2.62738119727],
    [1.09083237208, 0.0, 0.690244212843, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.727221581388, 0.656845299318],
    [0.0, 1.42892661606, 3.21508488614, 0.0, 0.0, 1.37534186796, 0.0, 0.0, 0.0, 0.0, 1.6421132483],
    [0.0, 0.727221581388, 9.01631503026, 0.0, 0.0, 0.0, 0.0, 0.727221581388, 0.0, 0.0, 1.31369059864],
    [0.0, 3.61264914625, 0.328422649659, 1.31369059864, 0.0, 5.25476239455, 2.62738119727, 0.656845299318, 1.6421132483, 1.31369059864, 0.0]
];

/// The two twin pairs, as indices into [`KINDS`]: (Priorities, Process) and
/// (Structure, Circumstances). These generate the order-4 automorphism group.
pub const TWINS: [(usize, usize); 2] = [(0, 7), (9, 6)];

/// Grounding depth per kind (Surface.lean `depth_counts` = [3,2,0,2]).
pub const DEPTH: [u8; N] = [1, 0, 0, 0, 1, 0, 1, 1, 2, 1, 3];

#[cfg(test)]
mod choice_tests {
    use super::*;

    #[test]
    fn executable_choice_vocabulary_matches_the_k11_arrays() {
        for (index, kind) in ChoiceKind::SITES.into_iter().enumerate() {
            assert_eq!(kind.site_index(), Some(index));
            assert_eq!(kind.plain(), KINDS[index]);
            assert_eq!(DEPTH[index] == 0, kind.is_surface());
        }
        assert_eq!(ChoiceKind::Record.site_index(), None);
        assert_eq!(ChoiceKind::ALL.len(), N + 1);
    }
}
