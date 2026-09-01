//! **The device class, defined once.**
//!
//! RESOURCE_DESIGN **D0**: *device class is part of the ARTIFACT, not of the schedule.* A
//! workload whose output is gated on bit-identity declares a class, and dispatch may choose
//! freely only within it.
//!
//! The warrant is a measurement, not a worry. SATURATION-3 G2 ran the `(O,O,O)` FCI sigma on
//! both devices: they agree to **3.033e-15** relative and **91.0% of the 207,025 entries differ
//! BITWISE**. Both answers are correct. They are not the same artifact. So a dispatcher that
//! sends work to the GPU above a measured size crossover and to the CPU below it makes the last
//! bits of every result a function of where that crossover happens to sit — and the crossover
//! moves with machine load, driver version and library kernel selection.
//!
//! # Why this is its own crate
//!
//! Two crates that cannot depend on each other both need to name the class: `holon-chem`
//! produces the artifact (and ships into a browser, so it cannot take a std-only dependency),
//! and `holon-resource` dispatches on it (and sits under everything, so it cannot take a heavy
//! one). Declaring the enum twice with a conversion between them would be M-DEVICE-CLASS's own
//! shape one level down: the day a third class is added to one side and not the other, one
//! artifact gets two different answers about what produced it and every test still passes.
//!
//! # What this crate does NOT do
//!
//! It does not decide anything. There is no dispatch policy here, no throughput, no probe — a
//! class is a LABEL that travels with an artifact, and the moment a label starts choosing is the
//! moment it has become a schedule.

#![no_std]

use core::fmt;

/// Which arithmetic produced — or would produce — an artifact.
///
/// Carried BY the artifact. Two artifacts of different classes may agree to any tolerance you
/// like and still be different artifacts, so a bit-gated table may never mix them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DeviceClass {
    /// Host arithmetic: the CPU mesh, `sigma_direct` and its Davidson driver.
    Cpu,
    /// Device arithmetic: CUDA, and whatever kernel selection the pinned configuration made.
    Gpu,
}

impl DeviceClass {
    /// The short token that goes in a filename, a digest key or a table header.
    ///
    /// Deliberately lowercase and stable: it is written into artifacts, so changing it would
    /// re-key every table ever produced.
    pub const fn tag(self) -> &'static str {
        match self {
            DeviceClass::Cpu => "cpu",
            DeviceClass::Gpu => "gpu",
        }
    }

    /// Every class, so a caller enumerating them cannot silently miss one that was added later.
    pub const ALL: [DeviceClass; 2] = [DeviceClass::Cpu, DeviceClass::Gpu];

    /// Parse a tag back. `None` rather than a default: an unrecognised class on a stored
    /// artifact means the artifact was produced by something this build does not know about,
    /// and guessing `Cpu` would silently admit it into a bit-gated table.
    pub fn from_tag(s: &str) -> Option<DeviceClass> {
        match s {
            "cpu" => Some(DeviceClass::Cpu),
            "gpu" => Some(DeviceClass::Gpu),
            _ => None,
        }
    }
}

impl fmt::Display for DeviceClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.tag())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The round trip, over `ALL` rather than over a hand-written list — so a class added to the
    /// enum without a tag fails here instead of in a table six months later.
    #[test]
    fn every_class_round_trips_through_its_tag() {
        for c in DeviceClass::ALL {
            assert_eq!(DeviceClass::from_tag(c.tag()), Some(c), "{c} lost its tag");
        }
        assert_eq!(DeviceClass::ALL.len(), 2, "a class was added; audit every match on it");
    }

    /// An unknown tag is refused, not defaulted. This is the half that matters: a default would
    /// admit a foreign artifact into a bit-gated table under this build's own class.
    #[test]
    fn an_unknown_tag_is_refused_rather_than_defaulted() {
        assert_eq!(DeviceClass::from_tag("tpu"), None);
        assert_eq!(DeviceClass::from_tag("CPU"), None);
        assert_eq!(DeviceClass::from_tag(""), None);
    }
}
