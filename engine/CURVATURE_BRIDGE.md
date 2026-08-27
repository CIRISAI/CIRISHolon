# CURVATURE_BRIDGE — the certified curved tier chart

Status: DESIGN, awaiting integrator review before implementation.
Foundation: `crates/ciris-sim-core/{relativity,curvature}.rs` (T5, commit 922ce47 — all gates
green, measured numbers in the T5 report). Binding context: `MESH_DESIGN.md` §1 (single-tier
fence), `holon-sandbox/src/tier.rs` (the ladder and its `NoGravityChart` refusals),
`Core/GrainFloor.lean` (floor/ceiling taxonomy, `cert_does_not_transport_across_reroot`),
`Core/Locality.lean` (the staked kill that names gravity), OBJECT.md R1 (admissibility is
factoring only on nested ladders).
Date: 2026-08-24.

## 0. Verdict first

**A tier adopts `(g0, Φ)` the way it adopts `g0` today: gravity is CHART DATA, static and
declared, and the geodesic stepper consumes it locally.** That one sentence is the whole
design; everything below is its certificate, its screens, and its refusals. It unblocks the
three `NoGravityChart` tiers honestly:

| tier | scene ε = max(\|Φ\|/c², v²/c²) | chart-error bound K·ε² (K=10, probed) | verdict |
|---|---:|---:|---|
| planet (ballistic + orbital) | 7.0e-10 | 4.9e-18 | certified; curved-vs-flat near the arithmetic floor, and the certificate SAYS so |
| galactic (nuclear orbits) | wide S2-class 4e-5; FULL S2 envelope 6.6e-4 (see A1 — the original 4.4e-5 was orbit-averaged, the defect A1 cites) | 1.9e-8 / 4.3e-6 | wide class certified under the frozen stake; full S2 refuses under it and certifies under A1 — both readings gated |
| cosmic (comoving patch ≤ ~30 Mpc) | ~1e-5–1e-4 | ≤ 1e-7 | certified per patch; expansion-scale claims REFUSE (floor, FRW unlock named) |

No new object class, no new holon field: `Φ` is values on the scene chart (the frame decision
of 2026-08-23 — "gravity is CHART data" — made executable), the certificate is a function, and
the refusals are typed by the existing taxonomy.

## 1. The chart

**Declaration.** A curved tier declares `(g0, ChartPhi)` where

```
ChartPhi { uniform_g: f64, centers: [(pos, GM); N_MAX] }   // fixed capacity, no_std
```

Φ(x) = uniform_g·z + Σᵢ −GMᵢ/|x−cᵢ|, consumed by the existing `curvature.rs` machinery
(metric A = 1+2Φ/c²+2β_ppn(Φ/c²)², B = 1−2Φ/c²; β_ppn = 1 is the chart, β_ppn = 0 remains
constructible only as the perihelion gate's predicted-signature plant). Linear superposition of
centers is licensed BY the weak-field premise itself: the superposition error is O(ε²), inside
the certificate's remainder. v1 supports exactly this family — it lights all three tiers
(uniform: planet ballistic; central: planet orbital, galactic nuclear, cosmic cluster infall).

**Scoped out BY NAME** (each a values/chart gap, not a silent limit): strong field (screened
out numerically, §2), horizons, dynamical Φ (self-gravity — A3's field-equation half),
FRW background/expansion (see cosmic screen), gravitational radiation, rotation
(Lense–Thirring), logarithmic/NFW potentials (v2, needed for flat rotation curves — a
galactic-disk scene cannot be DECLARED in v1 and the doc says so rather than approximating).

**Locality, faced head-on.** `Core/Locality.lean`'s staked kill names "the gravity chart's
global Poisson solve" as the known candidate for a certified update exceeding its declared
radius. v1 does not trigger it BY CONSTRUCTION: Φ is frame data, fixed at adoption, never
updated — the geodesic step reads Φ and ∇Φ at the worldline's own position only, so every
certified update has dependence radius 0 in the field and the existing `locality.rs` horizon
arithmetic applies unchanged to the matter. The kill becomes live exactly when Φ becomes
dynamical (self-gravity): at that point the design MUST either declare the global solve
honestly as a chart-cadence global datum with its aggregation warrant re-earned (the kill's own
words), or use a local field formulation. That decision is deferred WITH the A3 field half and
recorded here so it cannot be smuggled later. If the certificate machinery ever needs an
omitted-mode error term (e.g. coarse holons under the curved chart), the named object is
`locality.rs::HorizonLocality::influence_bound` — not a new one.

**Fences on readings** (cited, not re-derived): `ThermalScale` — no positive scale-covariant
entropy-only temperature exists, so nothing here may be read as a temperature; `DMVacuum` —
raw cut-edge count is not the entropy area functional, so no area-entropy reading off these
charts. This design makes neither reading.

## 2. The certificate

```
WeakFieldCertificate {
  status: Certified | GrainFloor | RefinementUnavailable,   // the existing enum
  epsilon: f64,               // max over the DECLARED scene envelope of |Φ|/c² and v²/c²
  remainder_bound: f64,       // K·ε², K = 10 (staked; probe below)
  tolerance: f64,             // the tier's declared fractional dynamics tolerance
}
```

The scene must declare its envelope: r_min to every center, domain height, v_max. A scene that
cannot declare these cannot be screened and is refused as undeclarable (not as a physics
verdict).

**The staked remainder band, with ALL its probes RUN before this freeze** (two-solve rule):
the chart's own error is second order. Probed on the perihelion instrument at ε = 1.000e-4
and 9.995e-4: deviation of the 1PN observable from 6πGM/(a(1−e²)c²) = +1.5052e-4 and
+1.5038e-3 — scaling exponent 1.000, coefficient dev/ε = 1.505 at both points (4 digits).
Extended probe at ε = 1e-5 (the galactic/cosmic ε_max/10 point): deviation +1.7112e-5, i.e.
dev/ε = 1.711 — the coefficient is NOT constant to better than ~14% across ε ∈ [1e-5, 1e-3],
and that drift is recorded rather than smoothed. Converted to trajectory phase: omitted-order
error = (dev/ε)·6πε²/((1−e²)·2π) — worst measured 5.2·ε² fractional per orbit. **K staked at
10** (1.9× headroom over the WORST probed coefficient, 2.2× over the calibration points).
Planet-scale probe (uniform-drop first-order coefficient): ratio 1.00001 at 2Φ₀/c² = 1e-8 and
1.00117 at 1e-9 — the second-order residual is below the measurement floor there, exactly the
M27-adjacent smallness the planet-tier certificate asserts. Reproduction:
`scratchpad t5report/src/bin/{probe_k,probe_tiers}.rs`; the implementation re-runs the
two-point K probe as a permanent gate.

**Screens** (certified iff ALL pass):
1. `ε ≤ sqrt(tolerance / K)` — the tolerance-derived cap.
2. `ε ≤ 1e-3` — the ABSOLUTE cap at the probed boundary. The band is measured on ε ∈
   [1e-4, 1e-3]; certifying beyond the probed range would be extrapolation, so the screen
   refuses there even if the tolerance arithmetic would allow it.
3. Cosmic only: the background term `ε_bg = (H·L/c)²` (H declared chart data, L the patch
   size) enters ε. This is where the FRW gap surfaces NUMERICALLY: at L ≈ 40 Mpc,
   ε_bg ≈ 1e-4; expansion-scale patches fail screen 2 by arithmetic, not by fiat.

**Refusal typing** (per the GrainFloor.lean taxonomy; every refusal names its unlock):
- Screens 1–2 exceeded → **FLOOR**. A stronger-field chart family (Schwarzschild exact for a
  single center) could serve the claim; that family is named future work. Frame-relative,
  lifted by a chart re-root — exactly `admissibility_change_is_reroot`'s shape.
- Cosmic expansion-scale claim → **FLOOR**. The FRW chart family could serve it; named
  future work.
- A claim requiring a spacelike signal (influence outside the light cone) → **CEILING**,
  justified: the light-cone partial order is invariant under every chart in the metric-theory
  family (the EP premise fixes Lorentzian signature), so no re-root within the family serves
  it. This is T5's chart-free causality export, typed. Nothing else is claimed as ceiling.
- ε below the arithmetic floor (curved and flat charts indistinguishable at f64, the M27
  branch: corrections < 2·eps_f64) → **GrainFloor**, and the FLAT chart is the licensed
  answer — this is the seam's admissibility test (§4), not a failure.

**Per-tier stakes** (scene values measured from the declared ladders; the εmax probes are
RUN — perihelion at 1e-4/1e-3 and 1e-5, uniform coefficient at 1e-8/1e-9, results above —
so these stakes are frozen WITH their probes, none deferred):

| tier | measured scene ε | staked ε_max | tolerance | note |
|---|---:|---:|---:|---|
| planet | 7.0e-10 (surface Φ 6.96e-10; LEO v² 6.7e-10; ball 3.6e-15) | 1e-8 | 1e-6 | deep inside; certificate's real content is the M27 flat/curved seam |
| galactic | wide S2-class 4e-5; FULL S2 envelope 6.6e-4 (pericenter r_p ~120 AU, v_p ~7.7e6 m/s — the row's original 4.4e-5 was the orbit-averaged GM/(ac²), the category error A1 cites) | ~~1e-4 (frozen)~~ → **1e-3 (A1)** | 1e-4 | BOTH readings gated in B1: full S2 REFUSES under the frozen stake, CERTIFIES under A1 (remainder 4.3e-6 ≤ tol); wide class certifies under both |
| cosmic | 1e-5 (δΦ/c² perturbations; cluster infall v~1e6 m/s) | 1e-4 | 1e-4 | patch-declared; ε_bg screens out L ≳ 40 Mpc |

## 3. Validation instruments — all analytic, every gate mutation-tested

Existing (kept, with their plants; measured values in the T5 report):

| gate | closed-form reference | plant(s), all firing |
|---|---|---|
| perihelion precession (ratio 1.00015) | 6πGM/(a(1−e²)c²) | β_ppn=0 → 1.33401 vs predicted 4/3; force-power ε=1e-3 → predicted ~2.6 band |
| uniform Newtonian limit (0.99933, 1.00000) | r = −(2Φ₀/c²)(gt²/2) + g³t⁴/(3c²) | B≡1 → predicted (~0, 1/2); sign flip → >1e3 off |
| ballistic proper time (0.999990) | (τ−T)/T = v₀²/(6c²) | Γ⁰ flip → derived and measured −7 |
| universality (bit-identical) | no mass parameter exists | 1e-6 fifth force fires 1e-12 gate |
| Killing drift 1.6e-13 / normalization 2.1e-14 | static-chart conserved e, l | Euler control ~1e-3 |

New in this slice:
1. **Screen gate**: a declared scene at ε = 2×ε_max MUST return RefinementUnavailable with the
   floor typing and the named unlock — the certificate must be able to fail. Plant: weaken the
   screen (ε_max×10) and the gate detects the wrongly-certified scene by measured remainder.
2. **K-band gate**: re-run the two-point probe (ε = 1e-4, 1e-3) inside the test suite; assert
   scaling exponent 1.00 ± 0.05 and coefficient inside [1.0, 3.0] (probed 1.505). Plant: K
   staked at 0.1 must be exceeded by the measured remainder at ε = 1e-3.
3. **Superposition gate**: two-center chart vs single-center closed forms in each center's
   near zone; cross-term error measured ≤ K·ε². Plant: sign-flipped second center.
4. **Circular-orbit instrument**: Ω vs Kepler √(GM/r³) with the O(ε) coefficient measured at
   two ε before its band freezes (two-solve; band staked at implementation, not here).
5. **Clock cross-check** (real-world corroboration, planet tier): `static_clock_rate` two-height
   ratio at GPS values (ΔΦ/c² ≈ 5.3e-10) against the published 45.7 μs/day gravitational term,
   band ±1% — data, not a simulator, as the sim-to-real linchpin gate asks.

## 4. The seam: flat ↔ curved is a RE-ROOT

A chart change over the same holon is a re-root (OBJECT.md: a view not comparable in the
current chain; R1 — the flat and curved charts are not a nested ladder).
`cert_does_not_transport_across_reroot` therefore applies verbatim: **no certificate crosses
the seam.** A tier that switches chart re-certifies every claim on the new side; the ledger is
the one thing that must survive (tier.rs's `Reroot` relation is reused unchanged — the seam is
a values case of it, `child.domain == parent.domain`, grains equal, charts differing).

**Admissibility test for the seam** — when may a scene stay flat: `certify_newton_chart`
(T5's A4) already answers the force-free half (Certified / GrainFloor between v = 6 and
7 m/s / RefinementUnavailable at β²/2 > tol). The curved half: the flat chart is admissible
exactly when the curved corrections are below tolerance or below f64 resolution — the
WeakFieldCertificate's GrainFloor branch computes it. The two certificates meet in the middle
and neither transports: the flat side never asserts anything about curved claims, and the
demo's thrown ball gets the honest sentence "Newton chart licensed; curved remainder ≤
2Φ_max/c² × (gt²/2) ≈ nm over the flight."

## 5. The speed clause (owed since T5, reading (a))

Bench target `crates/ciris-sim-core/benches/newton_vs_sr.rs` (harness = false, std allowed,
house report style): force-free and uniform-boost segments; for each stepper (Newton RK4 on
6-dof (x,v) vs SR RK4 on 8-dof (x^μ,u^μ)) find the dt achieving a target tolerance against
the SAME closed form, then wall-clock at those dts. **Integrator-tolerance budget, never an
ulp bar** (the two-fp32-runtimes lesson). Two tolerances (1e-9, 1e-12) — the clause's own
two-point probe. PASS bar: ratio ≥ 2.0. Falsifiable: γ overhead is only ~1.5–2.5×, so failure
is live; consequence, per §3.5: failure BLOCKS the speed half of the Newton-chart license —
`certify_newton_chart`'s docs drop the ≥2× claim and the certificate stands on accuracy alone.
The result is recorded in the bench header either way.

## 6. Sandbox integration (coordinated through the integrator — tier.rs is the sandbox lane's)

1. New `Evaluator::GeodesicChart` variant (or the sandbox lane's preferred name) carrying the
   adopted `ChartPhi`; the three `NoGravityChart` refusals lift ONLY where a tier declares a
   scene inside its screen. Refusal text change: planet's "no certified way to make weight
   pull" → the certificate line with its measured ε and remainder.
2. First certified scenes, per tier: **planet** — the thrown ball under real g = 9.81
   (UniformChart), certified with the honest nm-scale curved remainder plus the GPS clock
   cross-check; this is the demo Eric sees. **galactic** — S2 around Sgr A* (CentralChart,
   ε = 4.4e-5), the perihelion instrument doubling as the scene. **cosmic** — cluster infall
   in a declared patch (ε ≈ 1e-5), with a deliberately oversized patch shown REFUSING on the
   ε_bg screen as the demo of honesty.
3. `holon-ball-game` remains untouched; its CHART_GRAVITY bridge statement (already in
   curvature.rs) is cashed when the game slice reopens.

## 7. Implementation plan (after review)

`bridge.rs` in ciris-sim-core (ChartPhi, WeakFieldCertificate, screens — no_std, no new deps);
new gates of §3 in curvature.rs/bridge.rs tests; the bench of §5; two-point probes for every
band staked "at implementation" run and recorded BEFORE those bands freeze. ci-gates + full
test suite green before commit; pathspec commit; report per house format.

**STOP — this document awaits integrator review before any implementation.**
*(Review completed 2026-08-24: APPROVED with four attachments; implemented at 29eff39.)*

## Amendment A1 — galactic ε_max 1e-4 → 1e-3 (2026-08-24, post-freeze, integrator-ruled)

**Cause, cited as required:** the frozen galactic stake was derived from a MIS-MEASURED
scene value. The table row quoted ε = GM/(a c²) = 4.4e-5 — the orbit-averaged parameter —
where the certificate screens the ENVELOPE maximum; full S2's pericenter values
(r_p ≈ 120 AU, v_p ≈ 7.7e6 m/s) give ε = 6.6e-4. A category error about the scene, not a
choice about the certificate; found by the implementation's own gate (B1) and reported
before any code moved.

**Why the replacement is legitimate where fitting would not be:** 1e-3 is NOT fitted to
S2 — it is the independently probed boundary. The K band's pre-freeze probes ran AT
ε = 1e-3 (deviation coefficient 1.505, inside the [1.0, 3.0] band; in-suite gate B3
re-runs it permanently), so the amendment moves the stake to a point whose behavior was
measured before the defect was known.

**The cap, restated exactly as at the freeze:** the absolute screen cap
`WEAK_FIELD_EPS_CAP` stays 1e-3. A1 moves the galactic stake TO the probed boundary,
never past it; certifying beyond the probed range remains refused whatever the tolerance
arithmetic says.

**Both readings kept:** the per-tier table above shows the frozen and amended verdicts on
one row, and gate B1 asserts BOTH — full S2 refuses under `GALACTIC_EPS_MAX_FROZEN`
(kept as a constant, not erased) and certifies under A1's `GALACTIC_EPS_MAX`.
