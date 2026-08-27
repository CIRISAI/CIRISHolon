//! ℝ-plane tiers: the SoA layout of the physics engine and the statevector
//! carrier, as the same object with f64 planes — where CONDITIONING becomes
//! load-bearing (lean/Object.lean `sum_perturb_le`): a chart aggregate ships
//! with its coherence MEASURED, never assumed.

#[derive(Clone, Debug)]
pub struct RealPlane {
    pub lanes: Vec<f64>,
}

impl RealPlane {
    /// Coherence |Σa| / Σ|a| of an aggregate over this plane — the measured
    /// condition number of exposing its sum as engine state.
    pub fn coherence(&self) -> f64 {
        let s: f64 = self.lanes.iter().sum();
        let a: f64 = self.lanes.iter().map(|x| x.abs()).sum();
        if a == 0.0 {
            1.0
        } else {
            s.abs() / a
        }
    }
}

/// A physics-tier holon: SoA planes plus a chart whose conditioning is
/// COMPUTED from the planes at construction — the declaration is data.
pub struct RealHolon {
    pub planes: Vec<(String, RealPlane)>,
    pub conditioning: Vec<(String, f64)>,
}

impl RealHolon {
    pub fn new(planes: Vec<(String, RealPlane)>) -> Self {
        let conditioning = planes
            .iter()
            .map(|(name, p)| (name.clone(), p.coherence()))
            .collect();
        RealHolon { planes, conditioning }
    }
}

/// The entangled-bulk shape: per-site tensors (left, phys, right) as flat
/// f64 planes — the bond indices ARE the ledger of this tier. Minimal
/// viability instance: product states, single-site ops, exact contraction.
pub struct MpsHolon {
    pub tensors: Vec<(usize, usize, usize, Vec<f64>)>, // (dl, d, dr, data)
}

impl MpsHolon {
    pub fn product_state(bits: &[bool]) -> Self {
        MpsHolon {
            tensors: bits
                .iter()
                .map(|&b| (1, 2, 1, if b { vec![0.0, 1.0] } else { vec![1.0, 0.0] }))
                .collect(),
        }
    }

    pub fn apply_x(&mut self, site: usize) {
        let (dl, _, dr, ref mut data) = self.tensors[site];
        for l in 0..dl {
            for r in 0..dr {
                data.swap(l * 2 * dr + r, l * 2 * dr + dr + r);
            }
        }
    }

    /// Amplitude of a basis state: contract left to right (dl=dr=1 chain of
    /// products for this minimal instance; general contraction is the
    /// python DMRG's job upstream).
    pub fn amplitude(&self, bits: &[bool]) -> f64 {
        let mut acc = 1.0;
        for (site, &(dl, _, dr, ref data)) in self.tensors.iter().enumerate() {
            assert!(dl == 1 && dr == 1, "minimal instance is bond-1");
            acc *= data[if bits[site] { 1 } else { 0 }];
        }
        acc
    }
}
