//! The scale ladder: one zoom from the vacuum to the universe (the lead's ruling, as the CEWP
//! twin's lead relayed it on 2026-11-15: "There should be a switch, on by default, that limits
//! the zoom to street to globe - when off, zoom is universe to vacuum - holon-draw will
//! handle").
//!
//! # The zoom is a logarithm
//!
//! `z` is log10 of the view's HEIGHT in metres, an f64: about -35 at the Planck length and
//! about 26.9 across the observable universe. Nothing on the ladder is ever positioned in
//! absolute metres. Each [`Rung`] has its own UNIT (10^`unit` metres) and its own origin;
//! a rung's marks are f32 in that unit about that origin, and the camera is placed in the
//! same unit in f64 and folded into the matrix on the CPU ([`crate::Camera::relative`]), so
//! precision is relative to the rung at every scale.
//!
//! # What a rung is, and what it is not
//!
//! A rung is a scene drawn over a band of `z`, with a STATUS that says where what it shows
//! comes from. The ladder never passes a drawing off as a computation: a rung that is an
//! illustration says so on screen. The statuses are this repository's (`TIERS.md`) plus two
//! for drawing: a CATALOGUE value (a measured number someone else published) and an
//! ILLUSTRATION (nothing computes it).
//!
//! # Hand-off with a host
//!
//! A host that draws its own band (the twin: street to globe) owns that band's edges and
//! says where they are with [`Band`]. [`host_weight`] is the one function both sides read for
//! the cross-fade, so the two engines cannot disagree about it.

/// Where what a rung shows comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Computed live by an engine of this repository as it is drawn.
    Simulated,
    /// A result of this repository, read from its record.
    Measured,
    /// An input this repository takes as given (`TIERS.md`: DECLARED).
    Declared,
    /// A value published elsewhere, drawn at its measured size.
    Catalogue,
    /// A chosen position with a kill (`TIERS.md`: WAGER).
    Wager,
    /// A picture: nothing here computes it.
    Illustration,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Status::Simulated => "simulated live",
            Status::Measured => "measured",
            Status::Declared => "declared input",
            Status::Catalogue => "catalogue values",
            Status::Wager => "a wager",
            Status::Illustration => "illustration, not computed",
        }
    }
}

/// A rung: a name, the band of `z` it is drawn over (with a fade of [`FADE`] decades inside
/// each edge), its unit, and its status.
#[derive(Clone, Debug, PartialEq)]
pub struct Rung {
    pub name: &'static str,
    pub z_lo: f64,
    pub z_hi: f64,
    /// log10 of the rung's unit in metres.
    pub unit: i32,
    pub status: Status,
}

/// Decades over which one rung fades into the next.
pub const FADE: f64 = 0.5;

/// A rung's weight at `z`: 1 inside, falling linearly to 0 across `FADE` beyond each edge, 0
/// outside. Neighbouring rungs that share an edge cross-fade and their weights sum to 1.
pub fn weight(r: &Rung, z: f64) -> f64 {
    let h = 0.5 * FADE;
    let up = ((z - (r.z_lo - h)) / FADE).clamp(0.0, 1.0);
    let down = (((r.z_hi + h) - z) / FADE).clamp(0.0, 1.0);
    up.min(down)
}

/// The rungs drawn at `z`, with their weights, in ladder order; at most two.
pub fn visible(rungs: &[Rung], z: f64) -> Vec<(usize, f64)> {
    rungs.iter().enumerate().map(|(i, r)| (i, weight(r, z))).filter(|x| x.1 > 0.0).collect()
}

/// A host's own band and its switch.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Band {
    /// The finest `z` the host draws (its street) and the coarsest (its globe).
    pub z_lo: f64,
    pub z_hi: f64,
    /// ON: the zoom is clamped to the host's band and the ladder draws nothing.
    pub clamp: bool,
}

impl Band {
    /// The zoom a host may reach.
    pub fn clamp_z(&self, z: f64) -> f64 {
        if self.clamp {
            z.clamp(self.z_lo, self.z_hi)
        } else {
            z.clamp(Z_MIN, Z_MAX)
        }
    }
    /// The ladder's rungs are drawn over everything outside the host's band; this is the
    /// rung the host's band takes the place of.
    pub fn as_rung(&self) -> Rung {
        Rung { name: "host", z_lo: self.z_lo, z_hi: self.z_hi, unit: 0, status: Status::Measured }
    }
}

/// The ends of the ladder: the Planck length, and the diameter of the observable universe
/// (about 8.8e26 m), each with a little room.
pub const Z_MIN: f64 = -35.0;
pub const Z_MAX: f64 = 27.0;

/// The host's weight at `z` (1 inside its band, fading across `FADE` beyond each edge). The
/// ladder draws at `1 - host_weight` beyond the band and not at all inside it; with the
/// switch ON the zoom never leaves the band, so the host's weight is always 1.
pub fn host_weight(b: &Band, z: f64) -> f64 {
    if b.clamp {
        return 1.0;
    }
    weight(&b.as_rung(), z)
}

/// The camera for one rung at `z`: an orbit about the rung's origin (plus `pan`, in the rung's
/// unit) whose view height is 10^z metres at the focus.
#[allow(clippy::too_many_arguments)]
pub fn rung_camera(r: &Rung, z: f64, pan: [f64; 3], yaw: f64, pitch: f64, fov_y: f64, width: f64, height: f64) -> crate::Camera {
    let view = 10f64.powf(z - r.unit as f64); // view height in the rung's unit
    let dist = 0.5 * view / (0.5 * fov_y).tan();
    crate::Camera::orbit(pan, yaw, pitch, dist, fov_y, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ladder() -> Vec<Rung> {
        vec![
            Rung { name: "atoms", z_lo: -10.5, z_hi: -8.0, unit: -10, status: Status::Simulated },
            Rung { name: "drop", z_lo: -8.0, z_hi: -2.0, unit: -6, status: Status::Illustration },
            Rung { name: "solar system", z_lo: 7.5, z_hi: 13.5, unit: 11, status: Status::Catalogue },
        ]
    }

    #[test]
    fn neighbours_cross_fade_and_their_weights_sum_to_one() {
        let l = ladder();
        for k in 0..=100 {
            let z = -8.5 + k as f64 * 0.01;
            let v = visible(&l, z);
            let sum: f64 = v.iter().map(|x| x.1).sum();
            assert!((sum - 1.0).abs() < 1e-12, "{z}: {v:?}");
            assert!(v.len() <= 2);
        }
        assert_eq!(visible(&l, -9.0), vec![(0, 1.0)]);
        assert_eq!(visible(&l, -8.0), vec![(0, 0.5), (1, 0.5)]);
        assert!(visible(&l, 5.0).is_empty(), "a gap draws nothing");
    }

    #[test]
    fn the_switch_clamps_the_zoom_and_the_host_fades_only_when_it_is_off() {
        let b = Band { z_lo: 0.0, z_hi: 7.15, clamp: true };
        assert_eq!(b.clamp_z(-20.0), 0.0);
        assert_eq!(b.clamp_z(30.0), 7.15);
        assert_eq!(host_weight(&b, 7.15), 1.0);
        let off = Band { clamp: false, ..b };
        assert_eq!(off.clamp_z(-40.0), Z_MIN);
        assert_eq!(host_weight(&off, 3.0), 1.0);
        assert_eq!(host_weight(&off, 7.15), 0.5);
        assert_eq!(host_weight(&off, 7.4), 0.0);
        // a ladder rung that meets the host's edge carries the other half
        let space = Rung { name: "earth and moon", z_lo: 7.15, z_hi: 9.0, unit: 7, status: Status::Catalogue };
        for k in 0..=50 {
            let z = 6.9 + k as f64 * 0.01;
            assert!((host_weight(&off, z) + weight(&space, z) - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn a_rung_camera_shows_its_view_height_at_every_scale() {
        // at z = -9.5 in a rung of unit 1e-10 m the view is 3.16 units high; at z = 26 in a
        // rung of unit 1e24 m it is 100 units high: a unit sphere spans height / view pixels
        for (z, unit) in [(-9.5f64, -10i32), (26.0, 24), (-33.0, -35)] {
            let r = Rung { name: "", z_lo: z - 1.0, z_hi: z + 1.0, unit, status: Status::Illustration };
            let c = rung_camera(&r, z, [0.0; 3], 0.4, 0.3, 0.8, 1000.0, 800.0);
            let view = 10f64.powf(z - unit as f64);
            assert!(((c.scale * view) / 800.0 - 1.0).abs() < 1e-9, "{z}: {}", c.scale * view);
            let p = c.project([0.0, 0.0, 0.0]).unwrap();
            assert!((p[0] - 500.0).abs() < 1e-6 && (p[1] - 400.0).abs() < 1e-6);
        }
    }
}
