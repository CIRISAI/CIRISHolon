//! The verdict, in its order: envelope, then floor, then match.
//!
//! This is `SceneCert.verdict` transcribed, and the tests are its theorems run over every
//! input. The order is part of the certificate: a class the reference never covered is
//! refused by name, a class it covered too thinly is unresolved, and the match is not
//! consulted in either case.

/// The three answers. `Unresolved` is neither of the others.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Certified,
    Refused,
    Unresolved,
}

/// `SceneCert.verdict`.
pub fn verdict(in_envelope: bool, resolved: bool, pass: bool) -> Verdict {
    if !in_envelope {
        Verdict::Refused
    } else if !resolved {
        Verdict::Unresolved
    } else if pass {
        Verdict::Certified
    } else {
        Verdict::Refused
    }
}

/// The verdict with the exact half in front: a scene that does not aggregate to its parent
/// is refused before anything statistical is asked.
pub fn verdict_with_section(section_holds: bool, in_envelope: bool, resolved: bool, pass: bool) -> Verdict {
    if !section_holds {
        Verdict::Refused
    } else {
        verdict(in_envelope, resolved, pass)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const B: [bool; 2] = [false, true];

    #[test]
    fn certified_iff_all_three() {
        for e in B { for r in B { for p in B {
            assert_eq!(verdict(e, r, p) == Verdict::Certified, e && r && p);
        }}}
    }

    #[test]
    fn unresolved_ignores_the_match() {
        for p in B {
            assert_eq!(verdict(true, false, p), Verdict::Unresolved);
        }
    }

    #[test]
    fn outside_the_envelope_is_refused_whatever_else() {
        for r in B { for p in B {
            assert_eq!(verdict(false, r, p), Verdict::Refused);
        }}
    }

    #[test]
    fn a_broken_section_is_refused_whatever_else() {
        for e in B { for r in B { for p in B {
            assert_eq!(verdict_with_section(false, e, r, p), Verdict::Refused);
            assert_eq!(verdict_with_section(true, e, r, p), verdict(e, r, p));
        }}}
    }
}
