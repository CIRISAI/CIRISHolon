# Pre-registration — CLOSURE-2: the phase-space channel

*Frozen 2026-08-28, committed ALONE. Requirement 3's measurable half, on
the machinery WILSON-2B validated. The Lean half is `ClosureDerives.lean`
(inheritance, uniqueness, the finite symplectic instance) — the derivation
TEMPLATE; this campaign measures the channels the template speaks about,
on an exact model. Einstein-on-ADM remains the named far rung.*

misfits: contacts M-PROBE-EIGENSTATE (the carrier is the ZERO-FLUX dressed
vacuum, named), M-BARE-CHARGE (dressed pair unchanged), M-ONE-MODEL-DELTA
(the defect below is the COLLISION/minimax form, never a chosen-model
comparison), M-NULL-MISSTAKE (every conservation gate is staked per-arm on
the quantity unitarity actually conserves), M-LOOP-BLIND and
M-GAUGE-LAUNDER (both channels are built from ORIENTED weight triples),
M-PLANT-OBS and M-PLANT-SECTOR (plants state carriers and sectors),
M-COND-PROBE (dynamics is WILSON-2B's T, all inside the step),
M-ELECTRIC-BASIS (no new operator is introduced; the electric CHANNEL
reads shift-eigenspace weights, an observable, not a dynamics term),
M-HOMOG (no locality claim is staked in this campaign; the auto-promotion
machinery is inherited from the LOCAL-1D amendment and nothing else).

## Model and channels

WILSON-2B's frozen model and T, exactly (Z3 fan disk, 10 edges,
Eisenstein-exact, dressed pair, covariant pump). Carrier: the zero-flux
dressed vacuum. Trajectory: x_k = T^k(carrier), k = 0..8, with the
LOCAL-1D auto-promotion (int64 → arbitrary precision) so depth is not
capacity-limited.

- **v_conf** (the configuration channel): the tuple of oriented weight
  triples of all six plaquette holonomies (five fan + the rim loop),
  normalized exactly by cross-multiplication (scale-free).
- **v_PS** (the phase-space channel): v_conf PLUS the per-spoke ELECTRIC
  triples — the weights of the three shift-eigenspace projections
  `P_k(e) = (1/3)Σ_j ω^{−kj} L_j(e)`, computed exactly (×3 to stay in the
  ring). Configuration AND momentum, the (h, π) analogue.

## Gates

- **G0** (EXACT): carrier nonzero, Gauss held; first. witness: none
  (instrument-checked)
- **C1 — the configuration collision** (EXACT, two-branch): search all
  pairs i < j ≤ 8 with v_conf(x_i) = v_conf(x_j). Branch (a): a collision
  exists with v_conf(x_{i+1}) ≠ v_conf(x_{j+1}) → the minimax defect of
  v_conf is POSITIVE, exactly, by `collision_refutes_memoryless`
  (Closure.lean); its value is the staked reading. Branch (b): no
  collision at depth 8 → "v_conf not refuted at this depth", recorded,
  NOT a fire. witness: collision_refutes_memoryless
- **C2 — the separation claim** (EXACT, posed only on C1 branch (a)): at
  every C1 collision pair, v_PS(x_i) ≠ v_PS(x_j) — the momentum half is
  exactly the memory the configuration channel discarded. Fires if some
  colliding pair is v_PS-blind too. witness: none (measured gate)
- **C3 — inheritance, the derived gate** (EXACT): the microdynamics
  conserves total weight up to the exact per-step ring scale; the pushed-
  down quantity (total weight through either channel, scale-normalized)
  must be conserved along the coarse trajectory at every step, both
  channels. This is `closed_view_inherits_conservation` wearing its
  measured face. witness: closed_view_inherits_conservation
- **C4 — single-valuedness ledger** (EXACT): for every collision (either
  channel) whose successors AGREE, record consistency; the induced F is
  single-valued on all observed data or C1(a) has already said otherwise.
  Ties to the uniqueness theorem: what closure determines, the data
  cannot contradict. witness: closure_determines_dynamics
- **B3** (EXACT): joint Gauss on every trajectory state. witness: none

## plants (carrier and sector per M-PLANT-SECTOR)

Each plant's carrier is asserted nonzero in the sector the plant acts on.
- **(i) channel-blindness control**: two states differing ONLY in electric
  content (the carrier and its U_E-image at one edge — configuration
  marginals provably unchanged by a diagonal-in-flux check) must be
  v_conf-EQUAL and v_PS-DIFFERENT. Convicts both channels' definitions at
  once. Carrier: the zero-flux vacuum; sector: the electric difference,
  asserted nonzero.
- **(ii) conservation mutant**: score C3 with a deliberately mis-scaled
  step (drop one ring factor); the conservation gate must FIRE on the
  mutant. Carrier: the trajectory; sector: total weight, nonzero.
A missed plant VOIDs.

## Meaning

C1(a)+C2+C3 → "on an exact constrained gauge model: the configuration
channel is provably non-autonomous with a computed minimax defect, the
phase-space channel carries exactly the discarded memory at every measured
collision, and the coarse dynamics inherits its conserved generator —
derived by theorem, measured by instrument." That is requirement 3's
structure, complete on the toy. What it is NOT: Einstein dynamics — the
ADM instantiation is the named successor. C2 firing would be the deeper
find: even (h, π) insufficient on this model — recorded either way.
