//! THE ADMISSION DOOR: every priced computation in this crate asks for its resources at
//! the moment of use, and a constant never stands in for the probe.
//!
//! Until 2026-09-02 this crate carried four capability caps — a hard determinant count,
//! a routing threshold between the determinant and MPS routes, an MPS orbital wall and
//! an MPS determinant reach. Each was a PRICE measured in one regime (one box, one
//! afternoon, one of them in wall-clock on a loaded machine) and consumed everywhere as
//! if it were physics. RESOURCE_DESIGN D1 says discovery is a hint and the probe is the
//! authority; D3b says an overflow leases the next tier rather than editing a constant.
//! So the caps are gone and this module is what replaced them: a solve computes its
//! PRICE from its own allocations, the resource layer's probe attempts that amount, and
//! the answer is admission or a refusal that names the price. A bigger box admits a
//! bigger solve; nothing in this crate knows what "too big" means.
//!
//! The one place a measured limit survives is as PROVENANCE on a refusal or a route —
//! `Provisional` where it was never calibrated in work units — never as a gate.

use holon_resource::probe::{AttemptProbe, Probe, ProbeVerdict, ResourceKind};
use std::sync::Mutex;

/// The Davidson driver's subspace bound (`tier.rs`: `max_sub = 48.min(n_det)`), and the
/// vectors it holds beyond the basis and sigma sets (diagonal, residual, start, two
/// candidates, scratch). The price is what the code allocates, not a guess about it.
pub const DAVIDSON_SUBSPACE_MAX: usize = 48;
const DAVIDSON_EXTRA_VECTORS: usize = 8;

/// A computation's price, in the units the resource layer probes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Price {
    /// What is being priced, for the refusal's own words.
    pub what: String,
    /// Bytes of working set the computation will hold.
    pub bytes: u64,
    /// How well the price is known. `Measured` prices are computed from the code's own
    /// allocations; `Provisional` ones are fitted and say so.
    pub provenance: &'static str,
}

/// The determinant route's working set for a space of `n_det` determinants: basis and
/// sigma subspaces of up to [`DAVIDSON_SUBSPACE_MAX`] vectors each, plus the fixed extras,
/// eight bytes per entry. MEASURED in the sense that matters: read off the driver.
pub fn price_determinant(n_det: usize) -> Price {
    price_determinant_with(n_det, DAVIDSON_SUBSPACE_MAX)
}

/// [`price_determinant`] under a caller-stated subspace bound (`tier::davidson_eigh_from_op_sub`):
/// the same allocations, read off the same driver, at the bound the solve will actually run.
pub fn price_determinant_with(n_det: usize, max_sub: usize) -> Price {
    let vectors = 2 * max_sub.max(2).min(n_det.max(1)) + DAVIDSON_EXTRA_VECTORS;
    Price {
        what: format!("determinant route, {n_det} determinants x {vectors} vectors (subspace bound {max_sub})"),
        bytes: (n_det as u64).saturating_mul(vectors as u64).saturating_mul(8),
        provenance: "computed from tier.rs's Davidson allocations (2·max_sub + 8 vectors)",
    }
}

/// The MPS route's dense MPO for `n_orb` spatial orbitals, PROVISIONAL: the bond
/// dimension of q8-mps's channel construction grows as roughly 3.8 n_orb^2 (fitted to
/// the seam lane's one measurement, 1.9 GB dense at 21 orbitals, 2026-09-02) and each
/// site tensor is d_l x d_r x 4 doubles. The exact price exists only once the MPO is
/// planned from real integrals; until the builder exposes its plan this is the pre-door
/// estimate, labelled as such, and the refusal that cites it says so.
pub fn price_mpo(n_orb: usize) -> Price {
    let d = 2.0 + 3.8 * (n_orb as f64) * (n_orb as f64);
    let bytes = (n_orb as f64) * d * d * 4.0 * 8.0;
    Price {
        what: format!("MPS route, dense MPO over {n_orb} orbitals at bond dimension ~{d:.0}"),
        bytes: bytes as u64,
        provenance: "PROVISIONAL: bond dimension fitted to one measurement (1.9 GB at 21 orbitals, mps-seam 2026-09-02); superseded when the MPO builder exposes its plan",
    }
}

/// A named refusal: the price that was asked, and what the probe said.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refused {
    pub price: Price,
    pub verdict: &'static str,
}

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "REFUSED at the admission door: {} would hold {} bytes of RAM and the probe said \
             \"{}\" (price provenance: {}). Nothing in this crate caps the size; a machine \
             that admits the reservation runs it. The escalation is a lease on a bigger \
             box or the next tier, never an edit to a constant.",
            self.price.what, self.price.bytes, self.verdict, self.price.provenance
        )
    }
}

impl std::error::Error for Refused {}

static ADMISSION: Mutex<Option<Box<dyn Probe + Send>>> = Mutex::new(None);

/// Install the probe every admission in this process consults. `None` restores the
/// resource layer's [`AttemptProbe`] — a real reservation, page-touched. Tests install
/// the resource layer's `ScriptedProbe` to PLANT a refusal or an admission, which is how
/// a routing rule is demonstrated to gate rather than assumed to.
pub fn set_admission(probe: Option<Box<dyn Probe + Send>>) {
    *ADMISSION.lock().unwrap_or_else(|e| e.into_inner()) = probe;
}

/// Ask the door. `Ok` carries what the probe verified; `Err` is the named refusal.
pub fn admit(price: &Price) -> Result<&'static str, Refused> {
    let mut guard = ADMISSION.lock().unwrap_or_else(|e| e.into_inner());
    let verdict = match guard.as_mut() {
        Some(p) => p.probe(ResourceKind::Ram, price.bytes),
        None => AttemptProbe::new(scratch_dir()).probe(ResourceKind::Ram, price.bytes),
    };
    match verdict {
        ProbeVerdict::Pass(what) => Ok(what),
        ProbeVerdict::Fail(why) => Err(Refused {
            price: price.clone(),
            verdict: why,
        }),
    }
}

/// The seam lane's measured record of the MPS route's reach, kept as PROVENANCE and not
/// as a gate: chi = 32 under a 300 s WALL-CLOCK budget on a loaded box (M-PLACEMENT-LOTTERY's
/// own 2x oversubscription), the route reached LiH's 225 determinants to 1e-8 in three
/// sweeps and did not reach S2's 23,409; the MPO build was driven at nine orbitals and not
/// at ten. A cap defined by a wall-clock timer on a loaded machine is a measurement of the
/// queue, not the method (`regime-inherited-constant`), which is why these numbers stopped
/// gating anything on 2026-09-02 and the seam node re-measures them in work units.
pub const MPS_REACH_RECORD: &str = "MPS reach, superseded record: chi=32, 300 s wall-clock on a loaded box; \
     reached 225 determinants (LiH), not 23,409 (S2); MPO driven at 9 orbitals, not 10. \
     Provenance: pair-route sweeps 2026-08-3x; re-measured in work units by the MPS seam node.";

#[cfg(test)]
mod tests {
    use super::*;
    use holon_resource::probe::ScriptedProbe;

    #[test]
    fn a_planted_refusal_names_its_price() {
        set_admission(Some(Box::new(ScriptedProbe::always_fail("planted"))));
        let r = admit(&price_determinant(1_000)).unwrap_err();
        assert_eq!(r.verdict, "planted");
        assert!(r.to_string().contains("1000 determinants"));
        set_admission(None);
    }

    #[test]
    fn the_real_probe_admits_a_small_space_and_refuses_an_absurd_one() {
        set_admission(None);
        assert!(admit(&price_determinant(1_000)).is_ok());
        let absurd = Price {
            what: "a space no machine holds".into(),
            bytes: u64::MAX / 4,
            provenance: "test",
        };
        assert!(admit(&absurd).is_err());
    }

    #[test]
    fn prices_are_monotone_and_the_mpo_price_is_labelled_provisional() {
        assert!(price_determinant(2_000_000).bytes > price_determinant(50_000).bytes);
        assert!(price_mpo(21).bytes > price_mpo(9).bytes);
        assert!(price_mpo(9).provenance.starts_with("PROVISIONAL"));
    }
}

/// Where the default probe's disk arm would write. `std::env::temp_dir` PANICS on
/// wasm32-unknown-unknown ("no filesystem on this platform"), and the browser found
/// that the first time the door was asked a price (2026-09-02): the RAM arm never
/// touches the path, so on wasm the door carries a nominal one.
fn scratch_dir() -> std::path::PathBuf {
    #[cfg(target_arch = "wasm32")]
    {
        std::path::PathBuf::from("/")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::temp_dir()
    }
}
