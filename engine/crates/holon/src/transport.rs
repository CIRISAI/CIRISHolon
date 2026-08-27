//! THE ONE TRANSPORT — the claim-transport square (lean/CIRISHolon/
//! Transport.lean), as API. Moving a reading between roots (shard migration,
//! chart re-rooting, tier stacking, mesh hand-off) is ALWAYS this and never
//! a bespoke path.
//!
//! The fence is the type system's job: a `Certificate` does NOT ride a
//! transport for free — `claim_transport_does_not_grant_certificate` is a
//! theorem, and here it is a struct: the transported value's certificate is
//! `None` unless a `CertWitness` (the second square) is supplied. Code that
//! wants a certified transported value must present the witness; there is no
//! other door.

use crate::Certificate;

/// The witness that the LICENSE survives the re-root (the second square).
/// Constructing one is a claim; it names its warrant.
pub struct CertWitness {
    pub warrant: String,
}

pub struct Transported<T> {
    pub value: T,
    /// None unless the second square was witnessed. Never defaulted.
    pub cert: Option<Certificate>,
}

/// Transport a value across a re-root. The claim square is the caller's
/// `map`; the certificate square is the OPTIONAL witness.
pub fn transport<A, B>(
    value: A,
    map: impl FnOnce(A) -> B,
    cert: &Certificate,
    witness: Option<CertWitness>,
) -> Transported<B> {
    let carried = map(value);
    match witness {
        Some(w) => Transported {
            value: carried,
            cert: Some(Certificate {
                view: cert.view,
                step: cert.step,
                rate: cert.rate,
                receipt: format!("{} | transported under: {}", cert.receipt, w.warrant),
            }),
        },
        None => Transported { value: carried, cert: None },
    }
}
