//! GANTT node D: probe EVERY species in the elements registry for what route, if any, the
//! engine has to its neutral-atom ground state — and state the relativistic fence.
//!
//! # This PROBES the registry; it asserts nothing from memory
//!
//! Every number below is read off `elements::ALL_ELEMENTS` or computed from it by counting.
//! The basis size is the sum of the declared shells' `n_functions()`; the electron split is
//! `pair::electron_counts`, the same function the solver uses; the determinant count is
//! `C(n_orb, n_alpha) * C(n_orb, n_beta)`, the same arithmetic `pair::automatic_route`
//! spends at its door. No solve runs here and none is needed: the classification is a
//! comparison of counts against constants read live out of the crate, printed beside their
//! values so a reader can check the comparison rather than trust it.
//!
//! # Route classification is ARITHMETIC, not a certification
//!
//! `FCI-DIRECT` says the determinant space is small enough that `fci::solve` would take the
//! determinant route. It does NOT say the resulting energy has been computed, converged,
//! checked against a second route, or certified for any use. ELEMENTS-3 measured what that
//! separation costs: indium came back at a 3.98e-1 residual — nine orders above
//! `pair::CONVERGED_RESIDUAL` — in a row otherwise indistinguishable from a measurement.
//! Certification is per-species campaign work and this table is not it.
//!
//! # Why the MPS band is reported with a caveat rather than as a route
//!
//! `pair::MPS_MAX_DETERMINANTS` (the MEASURED reach of the DMRG sweeps) is smaller than
//! `fci::MPS_ROUTE_THRESHOLD` (the size at which `fci::solve` hands work to them). A space
//! big enough to be routed to MPS is therefore necessarily bigger than the reach, so
//! `AutomaticRoute::Mps` is unreachable at the current constants. This example does not
//! take that on faith: it compares the two constants at run time and prints the conclusion
//! that follows, so the day one of them moves the sentence moves with it.
//!
//! # The relativistic fence
//!
//! Staked at Z = 36 (krypton): rows with `Z > 36` carry NON-RELATIVISTIC-MODEL-FENCE
//! whatever their route. The engine's Hamiltonian is the non-relativistic electronic one —
//! `fci`/`md` build kinetic, nuclear-attraction and two-electron Coulomb integrals over
//! real contracted Gaussians, with no mass–velocity, Darwin or spin–orbit term anywhere.
//! The reasoning for the stake is in the generated document; it is a MODEL fence, and its
//! exit is a named rung, not "permanent".
//!
//! Usage:
//!   cargo run --release -p holon-chem --example periodic_availability
//!
//! Output is a fixed-width table on stdout, `#`-commented header lines first, then one row
//! per registered species, then the per-class totals. It is the body of
//! `conformance/atomworld/PERIODIC_AVAILABILITY.md`.

use holon_chem::elements::{Species, ALL_ELEMENTS, MAX_Z};
use holon_chem::fci::{HARD_DETERMINANT_CAP, MAX_ORB, MPS_ROUTE_THRESHOLD};
use holon_chem::pair::{electron_counts, MPS_MAX_DETERMINANTS, MPS_MAX_ORBITALS};

/// Nuclear charge past which every row carries the non-relativistic model fence.
///
/// STAKED at krypton. See the generated document's own section for the one-sentence
/// reasoning; it is stated there because that is what a reader of the table sees.
const RELATIVISTIC_FENCE_Z: u32 = 36;

/// What the engine has, or has not, by way of a route to a species' ground state.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Availability {
    /// `n_det <= fci::MPS_ROUTE_THRESHOLD`: `fci::solve` takes the determinant route.
    FciDirect,
    /// `n_det > fci::MPS_ROUTE_THRESHOLD`: `fci::solve` hands this to the MPS/DMRG arm.
    MpsRoute,
    /// The registry does not carry what a solve would need. The variant names the gap.
    Unavailable(&'static str),
}

impl Availability {
    fn label(self) -> &'static str {
        match self {
            Availability::FciDirect => "FCI-DIRECT",
            Availability::MpsRoute => "MPS-ROUTE",
            Availability::Unavailable(_) => "UNAVAILABLE",
        }
    }
}

/// `C(n, k)` EXACTLY, in `u128`, or `None` when even `u128` cannot hold it.
///
/// Deliberately NOT the saturating form. `pair::choose`'s header spells out why the obvious
/// `acc.saturating_mul(n - i) / (i + 1)` refactor is wrong in the dangerous direction — once
/// the multiply saturates the divide pulls the result back DOWN, so the count UNDERSTATES
/// and a space reads as cheaper than it is. An exact count with an explicit `None` for
/// "cannot be represented" cannot understate at all.
///
/// `u128` is not decoration: this runs over a registry whose heaviest atom has 27 orbitals,
/// where `C(27, 13)` is 20,058,300 and the product fits in `u64` with room to spare — but
/// the loop is written for the registry it will have, not the one it has.
fn choose_exact(n: usize, k: usize) -> Option<u128> {
    if k > n {
        return Some(0);
    }
    let k = k.min(n - k);
    let mut acc: u128 = 1;
    for i in 0..k {
        acc = acc.checked_mul((n - i) as u128)?;
        acc /= (i + 1) as u128;
    }
    Some(acc)
}

/// Whether every declared shell carries usable primitive data.
///
/// A shell whose exponents are all zero, or non-finite, or whose contraction coefficients
/// are all zero, is a placeholder: the basis function it names has no radial part and the
/// integral machinery would build a zero column for it. This is the probe that turns
/// "missing basis data" from an assertion into a reading.
fn basis_gap(sp: Species) -> Option<&'static str> {
    if sp.shells.is_empty() {
        return Some("no shells declared");
    }
    for s in sp.shells {
        if !s.alpha.iter().all(|a| a.is_finite()) || !s.coeff.iter().all(|c| c.is_finite()) {
            return Some("non-finite primitive");
        }
        if s.alpha.iter().all(|a| *a == 0.0) {
            return Some("zero exponents");
        }
        if s.coeff.iter().all(|c| *c == 0.0) {
            return Some("zero coefficients");
        }
    }
    None
}

/// One probed row.
struct Row {
    z: u32,
    symbol: &'static str,
    n_basis: usize,
    n_elec: usize,
    n_alpha: usize,
    n_beta: usize,
    /// `None` when the count overflowed `u128`, which is a fact worth printing as one.
    n_det: Option<u128>,
    avail: Availability,
    /// Whether `n_det` is past `fci::HARD_DETERMINANT_CAP`, where `solve_determinant` — the
    /// by-hand route the MPS band actually falls back to — refuses outright.
    past_hard_cap: bool,
    fenced_relativistic: bool,
    /// Whether the registry carries a measured homonuclear radius for this species. A
    /// SECOND availability axis: an electronic route without one cannot be placed in a scene.
    has_radius: bool,
}

fn probe(sp: Species) -> Row {
    let n_basis = sp.n_basis();
    let (n_elec, n_alpha, n_beta) = electron_counts(&[sp]);
    let n_det = choose_exact(n_basis, n_alpha)
        .and_then(|a| choose_exact(n_basis, n_beta).and_then(|b| a.checked_mul(b)));

    let avail = if let Some(gap) = basis_gap(sp) {
        Availability::Unavailable(gap)
    } else if n_basis > MAX_ORB {
        // Past the solver's own orbital ceiling: the string machinery is built to MAX_ORB.
        Availability::Unavailable("orbitals past fci::MAX_ORB")
    } else if n_alpha > n_basis {
        // The declared basis cannot hold Z electrons, so `C(n_orb, n_alpha)` is zero and
        // there is no determinant space at all. A minimal basis that under-counts its own
        // element is missing basis data, not a small space.
        Availability::Unavailable("basis too small for Z electrons")
    } else {
        match n_det {
            Some(d) if d <= MPS_ROUTE_THRESHOLD as u128 => Availability::FciDirect,
            // Overflow lands here honestly: a count too large for u128 is certainly past
            // the routing threshold.
            _ => Availability::MpsRoute,
        }
    };

    Row {
        z: sp.z,
        symbol: sp.symbol,
        n_basis,
        n_elec,
        n_alpha,
        n_beta,
        n_det,
        avail,
        past_hard_cap: n_det.map_or(true, |d| d > HARD_DETERMINANT_CAP as u128),
        fenced_relativistic: sp.z > RELATIVISTIC_FENCE_Z,
        has_radius: sp.homonuclear_radius().is_some(),
    }
}

const HEAD: &str = "  Z  sym   nbas   n_elec    na    nb                  n_det  route        hard-cap  relativity                     scene r";

fn main() {
    println!("# PERIODIC-TABLE AVAILABILITY — probed from holon_chem::elements::ALL_ELEMENTS");
    println!("#");
    println!("# Constants READ LIVE from the crate, printed so the comparison can be checked:");
    println!("#   fci::MPS_ROUTE_THRESHOLD  = {MPS_ROUTE_THRESHOLD}   (n_det above this: fci::solve routes to MPS/DMRG)");
    println!("#   fci::HARD_DETERMINANT_CAP = {HARD_DETERMINANT_CAP}   (solve_determinant refuses outright above this)");
    println!("#   fci::MAX_ORB              = {MAX_ORB}        (the string machinery's orbital ceiling)");
    println!("#   pair::MPS_MAX_DETERMINANTS= {MPS_MAX_DETERMINANTS}      (MEASURED reach of the DMRG sweeps)");
    println!("#   pair::MPS_MAX_ORBITALS    = {MPS_MAX_ORBITALS}        (MEASURED orbital wall of the MPO build)");
    println!("#   elements::MAX_Z           = {MAX_Z}       (heaviest registered nuclear charge)");
    println!("#   RELATIVISTIC_FENCE_Z      = {RELATIVISTIC_FENCE_Z}       (STAKED here; rows past it carry the model fence)");
    println!("#");
    // The MPS band's real status, DERIVED at run time rather than asserted, so the sentence
    // tracks the constants instead of going stale beside them.
    if MPS_MAX_DETERMINANTS <= MPS_ROUTE_THRESHOLD {
        println!(
            "# MPS-ROUTE IS A BAND, NOT AN AVAILABLE ROUTE: pair::MPS_MAX_DETERMINANTS ({MPS_MAX_DETERMINANTS}) <= \
             fci::MPS_ROUTE_THRESHOLD"
        );
        println!(
            "# ({MPS_ROUTE_THRESHOLD}), so a space large enough to be routed to MPS is necessarily larger than the \
             sweeps' measured"
        );
        println!("# reach. `AutomaticRoute::Mps` is unreachable at these constants. An MPS-ROUTE row's real production");
        println!("# status is 'determinant, BY HAND ONLY (solve_determinant)' — and the hard-cap column reads REFUSED");
        println!("# on the rows where even that by-hand route refuses.");
    } else {
        println!(
            "# MPS-ROUTE IS REACHABLE: pair::MPS_MAX_DETERMINANTS ({MPS_MAX_DETERMINANTS}) > \
             fci::MPS_ROUTE_THRESHOLD ({MPS_ROUTE_THRESHOLD})."
        );
        println!("# The window is n_det in {MPS_ROUTE_THRESHOLD}..={MPS_MAX_DETERMINANTS} at n_orb <= {MPS_MAX_ORBITALS}.");
    }
    println!("#");
    println!("# ROUTE CLASSIFICATION IS ARITHMETIC, NOT A CERTIFICATION. FCI-DIRECT says a space is small");
    println!("# enough for the determinant route; it says nothing about whether that solve converges, agrees");
    println!("# with a second route, or is fit for any use. Certification is per-species campaign work.");
    println!("#");
    println!("# 'scene r' is a SECOND availability axis: Species::homonuclear_radius, measured for ten species");
    println!("# only. A row with an electronic route and no radius cannot be placed in a scene.");
    println!("#");
    println!("{HEAD}");

    let rows: Vec<Row> = ALL_ELEMENTS.iter().copied().map(probe).collect();

    for r in &rows {
        let det = match r.n_det {
            Some(d) => format!("{d}"),
            None => "> u128".to_string(),
        };
        // The hard cap gets its own column rather than a suffix: it is a DIFFERENT fence
        // from the routing threshold — `solve_determinant`, the by-hand fallback an
        // MPS-ROUTE row actually falls to, refuses outright above it — and two fences
        // sharing one cell is how one of them stops being read.
        let cap = if r.past_hard_cap { "REFUSED" } else { "-" };
        let rel = if r.fenced_relativistic {
            "NON-RELATIVISTIC-MODEL-FENCE"
        } else {
            "-"
        };
        let gap = match r.avail {
            Availability::Unavailable(g) => format!("  missing: {g}"),
            _ => String::new(),
        };
        println!(
            "{:>3}  {:<3}  {:>5}   {:>6}  {:>4}  {:>4}  {:>21}  {:<11}  {:<8}  {:<28}  {:<6}{}",
            r.z,
            r.symbol,
            r.n_basis,
            r.n_elec,
            r.n_alpha,
            r.n_beta,
            det,
            r.avail.label(),
            cap,
            rel,
            if r.has_radius { "yes" } else { "NONE" },
            gap,
        );
    }

    let fci = rows.iter().filter(|r| r.avail == Availability::FciDirect).count();
    let mps = rows.iter().filter(|r| r.avail == Availability::MpsRoute).count();
    let unavail = rows
        .iter()
        .filter(|r| matches!(r.avail, Availability::Unavailable(_)))
        .count();
    let fenced = rows.iter().filter(|r| r.fenced_relativistic).count();
    let hard = rows.iter().filter(|r| r.past_hard_cap).count();
    let no_radius = rows.iter().filter(|r| !r.has_radius).count();

    println!();
    println!("# TOTALS over {} registered species", rows.len());
    println!("#   FCI-DIRECT                    {fci}");
    println!("#   MPS-ROUTE                     {mps}");
    println!("#   UNAVAILABLE                   {unavail}");
    println!("#   ---");
    println!("#   past fci::HARD_DETERMINANT_CAP {hard}");
    println!("#   NON-RELATIVISTIC-MODEL-FENCE  {fenced}   (Z > {RELATIVISTIC_FENCE_Z})");
    println!("#   no measured homonuclear radius {no_radius}");
    println!("# rows {}", rows.len());
}
