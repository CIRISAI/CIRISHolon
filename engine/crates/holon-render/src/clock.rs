//! The three clocks, kept apart on purpose, and the contract that governs what gives
//! when the device cannot keep up.
//!
//! 1. **Physics dt** — DERIVED from the curve, never chosen. See `Timescale::from_table`.
//! 2. **Frame rate** — whatever `requestAnimationFrame` hands over. Never assumed, never
//!    60; the host passes the measured wall interval and the accumulator does the rest.
//! 3. **Sim-speed** — femtoseconds of simulated time per wall-second, user-visible.
//!
//! Conflating any two of these is the classic real-time-physics defect: stretching dt to
//! fit a frame silently rewrites the accuracy contract, and dropping substeps silently
//! rewrites the clock. Neither is permitted here. When the budget cannot be met, time
//! DILATES — the simulation runs slower in wall-clock terms while every step it does
//! take remains exactly as accurate as declared — and the dilation is displayed.

use crate::table::PotentialTable;
use holon::grain::Grain;
use holon::tune::{Degrade, Hold, Policy, PolicyError};

/// One atomic unit of time in femtoseconds (hbar / E_h = 2.4188843265857e-17 s).
pub const AU_TO_FS: f64 = 0.024188843265857;

/// Steps per vibrational period at the reference timestep. STAKED: 64 gives
/// `omega_e * dt = 2*pi/64 = 0.0982`, hence a reference drift bound of
/// `(omega dt)^2/4 = 2.4e-3` of the oscillator energy — comfortably inside the
/// `omega*dt < 2` stability limit of the Verlet map, with better than two decimal
/// digits on the vibration itself.
pub const STEPS_PER_PERIOD: f64 = 64.0;

/// Wall-seconds one vibration should take at the default sim-speed. Chosen so the
/// motion the whole app is about is actually watchable.
pub const WALL_SECONDS_PER_VIBRATION: f64 = 2.0;

/// `omega * dt` beyond which the velocity-Verlet map stops being stable: the conserved
/// quadratic form `1/2 v^2 + 1/2 omega^2 (1 - omega^2 dt^2/4) x^2` loses positive
/// definiteness at exactly 2. Reaching it is a REFUSAL, not a degradation — there is no
/// accuracy left to trade.
pub const STABILITY_LIMIT: f64 = 2.0;

/// Longest wall interval one frame may consume. A backgrounded tab can hand back a
/// multi-second interval, and honouring it would produce a catch-up burst that stalls the
/// page again on the frame after. Capped and REPORTED as dilation, never dropped quietly.
pub const MAX_FRAME_SECONDS: f64 = 0.25;

/// Which rung of the degradation ladder is active. Displayed, always.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rung {
    /// Everything held. The requested sim-speed is being delivered at the derived dt.
    Exact,
    /// Rung (i): the device cannot deliver the requested sim-speed, so TIME DILATES.
    /// Accuracy is untouched — every step taken is as exact as declared; there are
    /// simply fewer of them per wall-second.
    TimeDilated,
    /// Rung (ii): only reachable through the explicit user toggle. dt has grown and the
    /// drift bound has been RE-DERIVED and is being displayed at its new, larger value.
    /// Accuracy degraded declaredly.
    AccuracyDeclared,
    /// Nothing lawful remains: `omega_env * dt` has reached the stability limit.
    Refused,
}

/// What one frame is allowed to do.
pub struct FramePlan {
    pub substeps: u32,
    /// Delivered sim-speed over requested. 1.0 when nothing gave.
    pub dilation: f64,
    pub rung: Rung,
}

pub struct Timescale {
    // ---- derived from the model ----
    /// Reduced mass of the pair whose vibration sets `dt` — the FASTEST active mode, not
    /// the stiffest curve. See `Sim::adopt_table_timescale`.
    pub mu: f64,
    /// The SMALLEST reduced mass over the scene's active pairs.
    ///
    /// Separate from `mu` because the two answer different questions and in a mixed scene
    /// they are different pairs. `mu` sets the timestep, which the fastest vibration
    /// decides. `mu_min` sets the ENVELOPE frequency, which is a bound and therefore wants
    /// the mass that makes it largest. In a pure scene there is one pair type and they are
    /// the same number, so nothing about a single-species scene moves.
    pub mu_min: f64,
    /// Harmonic angular frequency at the well minimum: sqrt(|U''(R_e)| / mu).
    pub omega_e: f64,
    /// Vibrational period, atomic time units.
    pub period: f64,
    /// The reference timestep: period / STEPS_PER_PERIOD.
    pub dt_reference: f64,
    /// The timestep in force. Equal to the reference under an exactness hold unless the
    /// envelope forces refinement; larger only on rung (ii).
    pub dt: f64,

    // ---- the curvature envelope (fence 3) ----
    /// Largest |U''| reachable at the largest pair energy seen so far.
    pub k_env: f64,
    /// sqrt(k_env / mu): the frequency the drift bound is actually derived from.
    pub omega_env: f64,
    /// Innermost separation reachable at that energy.
    pub r_inner: f64,
    /// The energy the envelope was computed for. Monotone: the envelope is recomputed
    /// whenever a pair exceeds it, never relaxed, so the bound covers the whole history.
    pub e_rel_max: f64,

    // ---- the sim-speed clock ----
    pub sim_speed_fs_per_wallsec: f64,
    /// Simulated time owed to the user, in atomic units. Fractional remainder is CARRIED,
    /// never rounded away — that is what keeps the clock honest frame to frame.
    accumulator: f64,

    // ---- device capacity ----
    /// Measured on this device by the calibration burst. Zero until then.
    pub substeps_per_second: f64,
    pub calibrated: bool,

    // ---- policy ----
    /// The explicit user toggle that unlocks rung (ii). Off by default.
    pub allow_dt_growth: bool,
    pub rung: Rung,
    pub dilation: f64,
    /// The schedule the holon layer and the gates run on.
    pub grain: Grain,
}

impl Timescale {
    pub const fn empty() -> Self {
        Self {
            mu: 0.0,
            mu_min: 0.0,
            omega_e: 0.0,
            period: 0.0,
            dt_reference: 1.0,
            dt: 1.0,
            k_env: 0.0,
            omega_env: 0.0,
            r_inner: 0.0,
            e_rel_max: 0.0,
            sim_speed_fs_per_wallsec: 0.0,
            accumulator: 0.0,
            substeps_per_second: 0.0,
            calibrated: false,
            allow_dt_growth: false,
            rung: Rung::Exact,
            dilation: 1.0,
            grain: Grain {
                period: 1,
                exact_at: &[0],
                source: "placeholder until the table derives the schedule",
            },
        }
    }

    /// DERIVE the timescale from the potential itself.
    ///
    /// `omega_e = sqrt(|U''(R_e)| / mu)` reads the curvature straight off the table's own
    /// minimum, so a different curve gives a different timestep with no code change — the
    /// same property the forces have. `dt = period / STEPS_PER_PERIOD`.
    ///
    /// Nothing here is a hardcoded number: change the file and every clock below moves.
    pub fn from_table(&mut self, table: &PotentialTable, mu: f64) {
        self.mu = mu;
        // Default the envelope mass to the timestep mass. A caller with a mixed scene
        // overwrites it immediately (see `Sim::adopt_table_timescale`); a caller with a
        // pure one is already correct, because the two pairs are the same pair.
        self.mu_min = mu;
        let k_e = table.curvature(table.r_e).abs();
        self.omega_e = (k_e / mu).sqrt();
        self.period = if self.omega_e > 0.0 {
            core::f64::consts::TAU / self.omega_e
        } else {
            0.0
        };
        self.dt_reference = self.period / STEPS_PER_PERIOD;
        self.dt = self.dt_reference;
        // Default sim-speed: one vibration per WALL_SECONDS_PER_VIBRATION.
        self.sim_speed_fs_per_wallsec = (self.period * AU_TO_FS) / WALL_SECONDS_PER_VIBRATION;
        self.accumulator = 0.0;
        self.e_rel_max = 0.0;
        self.refresh_envelope(table, 0.0);
        // The holon layer and the ledger gates close on FRAME boundaries. The period
        // recorded here is the nominal substep count per frame at the default sim-speed
        // and a 60 Hz frame — provenance for the schedule, NOT a universal clock (the
        // fence grain.rs states and this module obeys: the real boundary is whatever the
        // frame turns out to be, which is measured per frame, never assumed).
        self.grain = Grain {
            period: 1,
            exact_at: &[0],
            source: "holon-render: closure at frame boundaries; substeps/frame measured per frame, never assumed",
        };
    }

    /// Recompute the curvature envelope if a pair has exceeded the energy it was last
    /// computed for. Monotone by construction — the bound must cover the whole history
    /// since reset, so the envelope only ever widens.
    ///
    /// This is fence 3 in force: the bound the UI shows is derived from the stiffest
    /// curvature the pair can REACH at its current energy, not from the equilibrium
    /// curvature, so it stays valid through a collision instead of going green right
    /// across the encounter that violates it.
    pub fn refresh_envelope(&mut self, table: &PotentialTable, e_rel_max: f64) -> bool {
        self.refresh_envelope_over(e_rel_max, |e| table.curvature_envelope(e))
    }

    /// The same, over MORE THAN ONE curve.
    ///
    /// `envelope` is handed the energy and returns the pair `(k, r_inner)` the caller
    /// wants the bound built from — for a mixed scene, the LARGEST curvature and the
    /// INNERMOST reach across every table the scene's atoms can meet each other on. The
    /// callback shape is what keeps the monotonicity rule in ONE place: a caller that
    /// looped over tables calling `refresh_envelope` per table would be stopped by the
    /// guard below after the first one, and would silently bound a mixed scene by whichever
    /// curve it happened to visit first.
    ///
    /// With a single curve the callback is that curve's own `curvature_envelope` and every
    /// float below is what it always was.
    pub fn refresh_envelope_over(
        &mut self,
        e_rel_max: f64,
        mut envelope: impl FnMut(f64) -> (f64, f64),
    ) -> bool {
        if e_rel_max <= self.e_rel_max && self.k_env > 0.0 {
            return false;
        }
        self.e_rel_max = e_rel_max.max(self.e_rel_max);
        let (k, r_inner) = envelope(self.e_rel_max);
        self.k_env = k;
        self.r_inner = r_inner;
        // `mu_min`, not `mu`: the envelope is a BOUND, so it takes the mass that makes
        // the frequency largest. Equal to `mu` in a pure scene, so this line is the same
        // float it was.
        self.omega_env = if self.mu_min > 0.0 {
            (k / self.mu_min).sqrt()
        } else {
            0.0
        };
        self.hold_exactness();
        true
    }

    /// The accuracy target, `omega * dt`, that the reference timestep meets by
    /// construction at the well bottom: `omega_e * dt_reference = 2*pi/64`.
    pub fn accuracy_target(&self) -> f64 {
        core::f64::consts::TAU / STEPS_PER_PERIOD
    }

    /// HOLD = EXACTNESS. When the envelope stiffens past what the reference timestep can
    /// carry, dt is REFINED (halved) until the accuracy target is met again. That is the
    /// hold doing its job, not a degradation: it costs substeps, and the extra substeps
    /// are what the degradation ladder then has to find room for.
    ///
    /// Under the rung-(ii) toggle this does nothing: there, dt is deliberately allowed to
    /// exceed the target and the enlarged bound is displayed instead.
    fn hold_exactness(&mut self) {
        if self.allow_dt_growth {
            return;
        }
        let target = self.accuracy_target();
        self.dt = self.dt_reference;
        if self.omega_env <= 0.0 {
            return;
        }
        // Bounded refinement: 2^-20 of the reference is far past any physical need and
        // keeps a pathological table from spinning here.
        for _ in 0..20 {
            if self.omega_env * self.dt <= target {
                break;
            }
            self.dt *= 0.5;
        }
    }

    /// Grow dt by a factor. Only meaningful on rung (ii); the caller must have set
    /// `allow_dt_growth`, which is the explicit user toggle.
    pub fn set_dt_multiplier(&mut self, multiplier: f64) {
        if !(self.allow_dt_growth && multiplier.is_finite() && multiplier > 0.0) {
            return;
        }
        self.dt = self.dt_reference * multiplier;
    }

    /// `omega_env * dt` — the single number that says how much accuracy the current
    /// timestep is buying, and whether the map is stable at all.
    pub fn omega_dt(&self) -> f64 {
        self.omega_env * self.dt
    }

    /// The relative drift bound, `(omega_env * dt)^2 / 4`.
    ///
    /// Derived in `sim.rs::drift_bound`, which carries the derivation; this is the
    /// dimensionless factor, and it is recomputed from the CURRENT dt and the CURRENT
    /// envelope every time it is asked for. A changed dt cannot leave a stale bound
    /// behind, because there is no stored bound to go stale.
    pub fn relative_drift_bound(&self) -> f64 {
        let x = self.omega_dt();
        0.25 * x * x
    }

    /// The declared policy, built from the engine's own tuner shapes.
    ///
    /// The default is `Hold::Exactness` degrading latency without limit: time dilates,
    /// accuracy never does. The toggle switches to `Hold::Latency` degrading accuracy to
    /// a declared epsilon — and it has to be a DIFFERENT policy, because
    /// `Policy::new` REFUSES `Degrade::Accuracy` under `Hold::Exactness`
    /// (`PolicyError::AccuracyUnderExactness`). The amendment's "declaredly, never
    /// silently" is therefore enforced by the constructor rather than by our care.
    pub fn policy(&self, frame_budget_ms: f64) -> Result<Policy, PolicyError> {
        if self.allow_dt_growth {
            Policy::new(
                Hold::Latency {
                    budget_ms: frame_budget_ms,
                },
                vec![Degrade::Accuracy {
                    eps: self.relative_drift_bound(),
                }],
            )
        } else {
            Policy::new(
                Hold::Exactness,
                vec![Degrade::Latency {
                    up_to_factor: f64::INFINITY,
                }],
            )
        }
    }

    /// How many substeps this device can afford inside `wall_dt` seconds.
    /// Uncalibrated, the answer is "as many as asked for" — a guessed budget would be
    /// worse than none, because it would dilate time for no measured reason.
    pub fn substep_budget(&self, wall_dt: f64) -> u32 {
        if !self.calibrated || self.substeps_per_second <= 0.0 {
            return u32::MAX;
        }
        (self.substeps_per_second * wall_dt).max(1.0) as u32
    }

    /// Plan one frame. `wall_dt` is the MEASURED interval since the last frame.
    ///
    /// The accumulator converts the user's sim-speed into whole substeps and carries the
    /// remainder. dt is never stretched to fit. If the budget cannot cover what the
    /// sim-speed asked for, the shortfall is declared as time dilation and the clock is
    /// re-based rather than allowed to accrue an invisible debt — a growing accumulator
    /// would be silent substep-dropping wearing a different hat.
    pub fn plan_frame(&mut self, wall_dt: f64, budget: u32) -> FramePlan {
        if self.omega_dt() >= STABILITY_LIMIT {
            self.rung = Rung::Refused;
            self.dilation = 0.0;
            return FramePlan {
                substeps: 0,
                dilation: 0.0,
                rung: Rung::Refused,
            };
        }
        // A long stall (a backgrounded tab, a blocked main thread) must not turn into a
        // catch-up burst that freezes the page again. The interval is capped -- and the
        // capping is REPORTED, because sim-time dropped on the floor is exactly the kind
        // of quiet clock-rewriting this module exists to refuse. It is dilation, and it
        // says so.
        let raw_dt = wall_dt;
        let wall_dt = wall_dt.clamp(0.0, MAX_FRAME_SECONDS);
        let clamp_dilation = if raw_dt > wall_dt && raw_dt > 0.0 {
            wall_dt / raw_dt
        } else {
            1.0
        };
        let requested_fs = self.sim_speed_fs_per_wallsec * wall_dt;
        self.accumulator += requested_fs / AU_TO_FS;

        let wanted = (self.accumulator / self.dt).floor().max(0.0);
        let wanted_u = if wanted > u32::MAX as f64 {
            u32::MAX
        } else {
            wanted as u32
        };

        let (substeps, dilation, rung) = if wanted_u > budget {
            let d = if wanted > 0.0 {
                budget as f64 / wanted
            } else {
                1.0
            };
            let r = if self.allow_dt_growth {
                Rung::AccuracyDeclared
            } else {
                Rung::TimeDilated
            };
            (budget, d, r)
        } else {
            let r = if self.allow_dt_growth {
                Rung::AccuracyDeclared
            } else {
                Rung::Exact
            };
            (wanted_u, 1.0, r)
        };

        self.accumulator -= substeps as f64 * self.dt;
        // Declare the shortfall instead of banking it. Carrying more than one step of
        // debt would mean the wall clock and the sim clock had quietly decoupled.
        if self.accumulator > self.dt {
            self.accumulator = self.dt;
        }
        let dilation = dilation * clamp_dilation;
        let rung = if clamp_dilation < 1.0 && rung == Rung::Exact {
            Rung::TimeDilated
        } else {
            rung
        };
        self.rung = rung;
        self.dilation = dilation;
        FramePlan {
            substeps,
            dilation,
            rung,
        }
    }

    /// Substeps per wall-second the CURRENT sim-speed demands. The denominator of the
    /// capacity question.
    pub fn required_substeps_per_second(&self) -> f64 {
        if self.dt <= 0.0 {
            return 0.0;
        }
        (self.sim_speed_fs_per_wallsec / AU_TO_FS) / self.dt
    }
}

/// The largest atom count this device can carry at the current sim-speed and accuracy.
///
/// The force loop is O(N^2) in the pair count, so with `P` affordable pair-evaluations
/// per substep, `N(N-1)/2 <= P`, giving `N <= (1 + sqrt(1 + 8P)) / 2` — solved exactly
/// rather than approximated.
///
/// ATOMWORLD.md banks this as `N_max ~ sqrt(pair-throughput / substep-rate)`, i.e.
/// `sqrt(P)`. The exact solution is asymptotically `sqrt(2P)`, so the banked form
/// UNDERSTATES capacity by a factor of sqrt(2) — the 2 from `pairs = N^2/2`. Recorded
/// here rather than quietly corrected, because the projection table in that document was
/// computed from the banked form.
pub fn n_max(pairs_per_second: f64, required_substeps_per_second: f64) -> f64 {
    if required_substeps_per_second <= 0.0 || pairs_per_second <= 0.0 {
        return 0.0;
    }
    let p = pairs_per_second / required_substeps_per_second;
    (0.5 * (1.0 + (1.0 + 8.0 * p).sqrt())).floor()
}

impl Timescale {
    /// The carried fractional remainder of simulated time owed to the user.
    ///
    /// Private to this module because nothing outside it may WRITE the accumulator during a
    /// run — carrying the remainder is the whole point, and a caller that reset it would be
    /// rounding time away silently. Exposed read/write to the checkpoint alone, which is
    /// not "during a run": a restore whose accumulator started at zero would replay a
    /// different frame schedule from the one that was checkpointed, and the replay gate
    /// would fail for a reason that has nothing to do with the physics.
    pub fn accumulator(&self) -> f64 {
        self.accumulator
    }

    pub fn set_accumulator(&mut self, v: f64) {
        self.accumulator = v;
    }
}
