//! The merge law tested ONCE, generically, for every ledger — and the
//! transport fence tested as API. These are the DRY guarantees: any tier
//! needing "its own" transaction or transport semantics has to argue with
//! these tests first.

use holon::ledger::Cyc;
use holon::merge::{fold, MergeLedger, RentLedger, SignLedger};
use holon::transport::{transport, CertWitness};
use holon::Certificate;

fn law_check<L: MergeLedger>(items: Vec<L>, eq: impl Fn(&L, &L) -> bool) {
    // associativity + commutativity + identity, via shuffled folds
    let a = fold(items.clone());
    let mut rev = items.clone();
    rev.reverse();
    let b = fold(rev);
    assert!(eq(&a, &b), "merge law violated: order dependence");
    let c = L::empty().merge(a.clone());
    assert!(eq(&a, &c), "merge law violated: identity");
}

#[test]
fn merge_laws_hold_for_every_ledger() {
    let mut seed = 5u64;
    let mut rand = move || {
        seed = seed.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = seed;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    };
    let cycs: Vec<Cyc> = (0..40)
        .map(|_| Cyc {
            c: [
                (rand() % 9) as i128 - 4,
                (rand() % 9) as i128 - 4,
                (rand() % 9) as i128 - 4,
                (rand() % 9) as i128 - 4,
            ],
            m: (rand() % 6) as i32,
        })
        .collect();
    law_check(cycs, |a, b| a == b); // EXACT equality: the exact ring's law
    let signs: Vec<SignLedger> = (0..40).map(|_| SignLedger((rand() % 4) as u8)).collect();
    law_check(signs, |a, b| a == b);
    let rents: Vec<RentLedger> = (0..40)
        .map(|_| RentLedger { paid: (rand() % 1000) as f64 / 7.0 })
        .collect();
    law_check(rents, |a, b| (a.paid - b.paid).abs() < 1e-9); // declared tolerance
}

#[test]
fn transport_fence_holds() {
    let cert = Certificate::exact("v", "t", "h");
    let t1 = transport(41u64, |x| x + 1, &cert, None);
    assert_eq!(t1.value, 42);
    assert!(t1.cert.is_none(), "a certificate must never ride transport for free");
    let t2 = transport(
        41u64,
        |x| x + 1,
        &cert,
        Some(CertWitness { warrant: "battery receipt xyz".into() }),
    );
    assert!(t2.cert.is_some());
    assert!(t2.cert.unwrap().receipt.contains("transported under"));
}
