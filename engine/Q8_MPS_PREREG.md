# Q8 MPS PREREG — a matrix-product-state engine with ledgered bonds, gated against the exact seam

**Frozen 2026-08-23, before any code exists.** No crate `q8-mps` exists at commit time; no MPO,
no SVD, no ground state has been computed. Every number below is chosen and hereby **STAKED**, or
is a closed form / theorem reused from `q-seam` and cited as such.

**Amendment 1 (2026-08-23, team-lead review, GO granted with two required pins).** D1's
sector-shift argument was verified independently and approved as-is. Two pins landed before build:
the §2 working-Hamiltonian convention now unshifts with the integer `N_target`, never the measured
`⟨N̂_tot⟩` (closes a `U/2`-scaled drift leak into G2); G0-2 gained `⟨(Ŝz_tot)²⟩` to close a
sector-mixing loophole `⟨Ŝz_tot⟩=0` alone does not close. One suggestion adopted: G4 carries a
theoretical prior that its fitted exponent should sit near 1. All three are inlined at their sites
below, not collected in a separate errata section — the amendment changed the stakes themselves,
not just the record of them.

**Amendment 2 (2026-08-24, resource decision, Eric via team-lead — `N=12` demoted MID-RUN, before
its own gates could adjudicate).** Not a scientific finding, a scheduling one, stated with the same
discipline as the rest of this file. The staked `N=8,10,12` grid was launched (full grid gates +
G4's chi ladder, both detached per house rule 5) and stalled: `full_grid_gates` sat 85+ minutes
silent at `N=12`'s exact reference call. Diagnosis (`examples/probe_n12.rs`, committed, does not
touch `q-seam`'s source): a FAITHFUL replica of `q-seam`'s Lanczos cost shape — including the one
step a first, cheaper replica missed, the full `m x m` dense Jacobi re-solve of the growing
tridiagonal EVERY iteration (`lanczos.rs:117-126`) — ran 200 iterations in 67.66 s, extrapolating to
9–18 minutes at the full `MAX_ITERS=400` cap. **The algorithm is bounded; the observed stall is not
an algorithmic defect and is not q-seam's to fix.** The gap is resource contention: two of this
campaign's own `N=12`/`chi=256`-scale jobs were running concurrently, and the host was carrying
substantial unrelated desktop load at the same time (`ps aux` showed browser processes at tens of
percent CPU each) — a bounded ~10–20 minute cost inflated to plural contended hours. Decision: kill
both jobs, close the campaign on `N=8,10` (fully computed, uncontended once serialized — see below),
demote `N=12` rather than spend further wall time on its tail. Logs preserved as final artifacts
(`output/q8_mps/{full_grid_gates,g4_certificate}.log`, `.DONE` marked `KILLED` with the reason, not a
crash).

**Consequences, applied at their sites below, not collected separately:**
- **The validation grid is `N ∈ {8,10}`** for every gate in §3–§7 that named `N=8,10,12`. `N=12`'s
  partial data (it reached `U=0,1,4` before the kill, mid-`U=1` on the main grid job) is reported in
  §-Misfits as a diagnostic, never as a gate input — it did not run its own gate assertions before
  being killed, so there is no "frozen `N=12` reading" to hold alongside an amended one; this is not
  a case for the both-readings rule (that rule applies when two completed adjudications might
  disagree, and only one exists here).
- **G4's held-out set moves to `N=10`; the calibration set becomes `N=8` alone.** Reusing the
  original grid's fewer points is a genuine change to G4's staked design, not a relabeling — the
  original design pooled two `N`'s for calibration precisely so the fit wasn't a single-`N` fluke.
  If the single-`N=8` calibration set is too thin to pass its own `R² ≥ 0.85` check, or the `N=10`
  held-out band fails, **the staked refuse-to-quote policy fires exactly as designed** — a
  certificate that refuses under a thinner calibration set is behaving correctly, not failing.
- **The binding scheduling rule this pass adds** (a genuine addition, not a pin on existing
  wording): **before launching any two campaigns concurrently, probe the environment with one
  timing under load, and serialize by default.** `Q8_MPS_RESULTS.md` reports both stalled logs and
  this rule is carried into `Q9_*_PREREG.md`'s own process section.

**Amendment 3 (2026-08-24, team-lead BINDING RULING + research-manager verification — the
G3-primary re-read and its four-outcome table, written before its readings).** Two causes, kept
separate below. Neither is fitted to a desired outcome; A3.1 is a transcription of a ruling that
predates every G3-primary reading in existence, and A3.2 reports a deviation already in force.

**A3.1 — the G3-primary re-read, and the four outcomes, staked before adjudication.**

*Cause.* The §7 **SWEEP KILL IS RULED FIRED** — both readings (absolute `>2 of the grid`,
proportional `>2/12 of the original grid`), monotone in the unread configurations, and the count
`>2` is **not renegotiated** against Amendment 2's shrunken grid. Ruled by team-lead, on the
superseded run's readings, before this amendment. What that ruling then exposed: **G3-primary was
staked in §4 but was never checkable** — `SweepResult` carried no per-sweep energy trajectory, so
neither of its two clauses (floor at every sweep; monotone non-increase across sweeps) had any data
to run against. Only G3-**secondary** (a single end-of-run comparison) was ever executed. That is a
defect in the instrument, found in verification, not a finding about the physics.

*The fence, and why it is all 8 and not 3.* G3-primary runs **inside** the grid loop, so all 8
configurations' readings come free; restricting the fence to the 3 stalled ones would cost extra
engineering **to keep evidence out of the record**, which is not a thing this programme does. The
re-read is therefore all 8 configurations of the Amendment-2 grid.

*The four outcomes, per configuration, on the two axes `G7 status × G3-primary monotonicity`.
Verbatim from the ruling, and the seductive case is named as such in advance:*

| | **monotone** (worst rise `≤ 1e-9`) | **NON-monotone** (rise `> 1e-9`) |
|---|---|---|
| **stalled** (G7 VOID, no convergence in 20 sweeps) | **(b) SLOW CONVERGENCE.** Configuration stays G7 VOID; the SWEEP KILL stands as ruled; **Q9's premise is intact** — the stall is a convergence-*rate* problem, which is what a chi-warm-start plus a stagnation detector is for. | **(a) OSCILLATION.** Promotes toward the CORRECTNESS KILL and **damages Q9's premise**: a certificate of motion has no monotone trajectory to certify. Reported as loudly as a survival. |
| **converged** (early-stop hit by sweep 20) | **(c) CONVERGENCE UPGRADED.** G3-primary passes on that configuration and the convergence claim becomes **warranted** — it previously rested on the early-stop test alone, which reads only the last two sweeps. | **(d) RIGHT BY REFERENCE, CONVERGENCE CLAIM RETRACTED with an asterisk.** **This is the seductive case and must not be reinterpreted post hoc.** The final energy matching the exact reference is **not** a defence: §4 stakes monotone non-increase at *every* sweep, not at the end. |

*Interaction with §3 and §7, stated now so it cannot be argued afterwards.*

- **(a) and (b) sit on VOID configurations, and §3 is binding**: a G7-VOID configuration is
  excluded as a **gate datum**. So a non-monotone reading there is a **diagnostic** — it convicts
  the sweep schedule of oscillating rather than merely being slow, and it is what decides Q9's
  premise — but it does **not**, by itself, fire §7. "Promotes toward" is the ruling's word and it
  is the right one.
- **(d) sits on a NON-VOID configuration, and there the adjudication could genuinely differ. Both
  readings are recorded now, per the `Q_SEAM_RESULTS.md` §2.5 both-readings rule:**
  **(i) THE LETTER OF §7** — the CORRECTNESS KILL "fires iff any of G-SVD, G0, G1, G2, G3-primary
  fails at any of the validated configurations", G3-primary's monotone clause is part of G3-primary,
  and on a non-VOID configuration it is a gate datum: **the CORRECTNESS KILL FIRES.** STOP; no
  `N~100` runner.
  **(ii) THE RULING'S GLOSS** — (d) retracts the convergence claim with an asterisk and does not by
  itself fire the correctness kill, on the ground that the returned state is right by reference.
  **If (d) is observed, both readings are reported and the kill is treated as UNADJUDICATED until
  team-lead rules. The default while unruled is the STRICTER reading (i): a staked kill is not
  softened by silence.**

*The ordering, recorded rather than smoothed.* **The discipline was bent here and this is the
record of it.** The re-run launched at **12:11:14 CDT**; this table was transcribed and committed
at **12:13 CDT** (`0e0e972`), i.e. **after** the instrument was already running — by two and a half
minutes, not by the nine this file first said. At transcription exactly **one** configuration's
G3-primary readings existed — `N=8, U=0`: converged in 5 sweeps, floor worst margin `-1.45e-12`
(band `≥ -1e-9`), monotonicity worst rise `3.14e-13` (band `≤ 1e-9`) — both inside band, i.e.
**outcome (c)**. The other 7, **including every stalled configuration, which is where the
adjudication actually bites**, were unread. The four outcomes were ruled by team-lead before any
G3-primary reading existed anywhere; this amendment transcribes them and does not construct them.
The correct order was table-then-launch, and the launch came first.

**A3.2 — deviations already in force, declared (found in research-manager verification of the
lane's tree, 2026-08-24).**

- **The exact reference is CACHED, deviating from §9's "called live for every exact comparison
  above — never copied".** Cause: q-seam's pinned Lanczos policy is deterministic, and re-deriving
  the same `(N,U,t)` reference on every re-run of an iteratively-fixed gate file is the campaign's
  single largest wall-clock cost (the superseded run spent 4h39m and never reached `N=10, U=16`).
  Mitigation as built: the cache is a plain text ledger at `output/q8_mps/exact_cache.txt`, and one
  configuration per run is forced **live** and cross-checked against its cache entry, with a
  mismatch `> 1e-12` a hard panic ("determinism is broken somewhere, this outranks every other
  finding"). **The cache file is an artifact of the campaign and is committed with the results.**
- **DEFECT IN THAT MITIGATION, recorded now, not after it matters.** The forced-live configuration
  is always the **first** of the grid (`N=8, U=0`) — the cheapest one. The expensive entries
  (`N=10`, and `U=16` in particular) are therefore **never** re-validated once cached, so the
  spot-check is close to vacuous exactly where a stale entry would cost the most. **Required
  before the cache backs any closeout number: rotate the forced-live configuration** (persist a
  run counter beside the cache, or force-live the last grid entry rather than the first) **and
  record in the log which configuration was validated on that run.** Until then, every cached
  reading in the closeout carries this caveat by name.
- **The SWEEP KILL's two readings are combined by CONJUNCTION in code** (`absolute && proportional`).
  Both fire on the data in hand, so nothing here is unadjudicated — but a conjunction silently
  biases toward *not* firing if the readings ever disagree, and the both-readings rule requires
  disagreement to be **reported as unadjudicated**, not resolved by an `&&`. Noted; not changed
  mid-run, because changing a kill's combining rule while its data is being read is precisely the
  move the rule exists to prevent.

**Amendment 4 (2026-08-24, team-lead ruling on a research-manager finding — G4's undeclared
filter, and the scope retraction that outranks it).** Four parts. **A4.2 is the headline and must
not be reported as a footnote to A4.1.**

**A4.1 — THE FILTER, DECLARED.** `examples/g4_certificate.rs:24` carries `const FLOOR: f64 = 1e-14`,
applied as `.filter(|pt| pt.epsilon > FLOOR && pt.d_energy > FLOOR)` before the log-log fit. **§5
declared no floor and no exclusion rule.** An undeclared filter in code is a prereg defect on its own
terms, independently of whether the filter is a good idea.

*Provenance, and it is exculpatory — recorded because it would be recorded if it were the reverse.*
`FLOOR` was present in `b510a68`, **2026-08-23 20:50:58**, the commit that first created the runner.
The earliest G4 data is `output/q8_mps/g4_certificate.KILLED_RUN.log`, **2026-08-24 05:44** — nearly
nine hours later. **The filter predates every G4 number in existence. It was not fitted to the data.**
The defect is non-declaration, not shopping.

*What it removes, at the one configuration examined (N=8, U=16):* `chi=128` (`ε = 2.595e-17`) and
`chi=256` (`ε = -0e0`). Retained: `chi=16, 32, 64`. **The filter is also numerically forced** — a
log-log fit cannot consume `log(-0.0)` (undefined) or `log(2.595e-17)` (below f64 resolution for a
norm-1 state, `ε_machine ≈ 2.2e-16`). It is defensible. That defence is the next paragraph.

**A4.2 — SCOPE RETRACTION: THE CERTIFICATE IS UNQUOTABLE IN THE DOMAIN IT WAS BUILT FOR.** Neither
"passed" nor "failed" is the honest closeout sentence. §5's policy is to quote `δE ≈ c·ε^p` **at
N~100** — where `chi` is large and `ε` sits at or under the numerical floor, exactly as it does at
`chi=128, 256` here. The staked fallback degenerates in the same place: **reporting `-0e0` as an
error indicator is reporting noise.** So the measurable-`ε` domain and the domain requiring the
certificate are close to disjoint. **This is `Q_SEAM_RESULTS.md`'s shape recurring — the instrument
works where you do not need it** — and it is reported at that prominence, in those words.

**A4.3 — AND THE STAKED FUNCTIONAL FORM IS REFUTED ON THE RETAINED DATA, WITHOUT THE EXCLUDED
POINTS.** This is stronger than the filter complaint and survives it entirely. All three of
`chi=16, 32, 64` pass `FLOOR` and enter the fit:

| chi | ε | δE |
|---|---|---|
| 16 | 2.4742e-6 | 3.4108e-5 |
| 32 | 2.5926e-8 | 8.5945e-7 |
| 64 | 4.2747e-12 | **1.4947e-2** |

`16 → 32`: `ε` falls 95×, `δE` falls 40× — **requires `p > 0`.** `32 → 64`: `ε` falls 6065×, `δE`
**rises** 17392× — **requires `p < 0`.** `c·ε^p` with `c > 0` is monotone in `ε`. **No single `(c,p)`
satisfies both legs, so the staked form is not merely inaccurate here — it is impossible.**

**And §5's staked `R² ≥ 0.85` gate cannot detect this.** The fit pools 20 points (N=8 × 4 `U` × 5
`chi`); the violation is a **sign** violation confined to one `U` column. A pooled goodness-of-fit
statistic can pass while containing a subset on which the form is refuted. **The gate as staked is
the wrong instrument for the failure that actually occurs** — a per-column monotonicity check would
catch it; `R²` averages it away. Recorded as a gate-design defect, not moved: the threshold stays
`0.85` and the closeout reports both.

**A4.4 — THE LAKE ALREADY CARRIES THE SHAPE, and the citation is narrowed to what is actually
proved.** `Core/Stagnation.lean`'s `error_not_computable_from_motion` states
`¬ ∃ f : ℝ → ℝ, ∀ x, |x − target| = f (motion id x)` — distance-to-truth does not factor through an
internal process quantity. **That theorem is about the MOTION residual, not about discarded weight.**
Applying the shape to `ε` needs the general lemma `not_computable_from` **plus a fiber-separation
witness for `ε`**, and ours is **measured, not proved** (the table above: two rungs whose `ε` ordering
and `δE` ordering disagree). So this is **scope-corroboration of an existing shape, not a new theorem
and not a proof about G4** — graded that way deliberately, per the house rule against counting one
principle's instantiations as independent confirmation. What is airtight is the empirical refutation
in A4.3, which needs no theorem at all.

**A4.5 — CONSEQUENCE FOR Q9, before its prereg is written.** The same shape forbids a **certificate of
motion** from being a **certificate of error**. Q9's deliverable must be *"refuses to quote when
motion is uninformative"*, never *"quotes error from motion"*. Load-bearing in the design from here.

**A4.6 — housekeeping.** (i) `ε = -0e0` at `chi=256` remains **unexplained**: `mps.rs:395` computes
`Σ s_i²` and `g4_certificate.rs:68` sums those — a sum of squares cannot carry a sign bit — and the
suggested `1 − Σ(kept)²` subtraction form was searched for across `crates/q8-mps/src` and
`examples/` and **is not present**. Flagged, not theorised. (ii) Two citation upgrades for the
χ-ladder anchor: **`chi=16` is a second witness** (`δE = 3.411e-5`, converged, also beating
64/128/256) — one anomalous rung invites a special-case story, two on the same side does not; and
the `dE`-to-energy ordering transfers **only because every rung sits above the exact energy**, which
**G3-primary's floor supplies** (`worst_margin = −1.526e-11` against slack `1e-9`). Both numbers
belong next to the citation so a reader can check the transfer rather than trust it.

*Scope of every number above: N=8, U=16, from a run killed mid-flight. The per-config lines are
complete readings; the kill truncated the sequence, not the lines.*

The product is not a faster solver. The product is the same refusal discipline `GrainFloor.lean`
names for the sandbox engine, instantiated on a new resource: **a bond's declared dimension `chi`
is its ledger**, its accumulated discarded weight is the honest cost of underpaying that ledger,
and a state that needs more than the declared `chi` gets a typed refusal, never a silently
truncated answer. `Core/SelfAudit.lean`'s lesson governs §5 directly: a certificate built from the
engine's own data (sweep-to-sweep energy drift, say) cannot certify the engine's error against the
truth — only an external reference can, which is exactly why every correctness gate below is read
against `q-seam`'s exact solver, never against this engine's own residuals.

---

## 0. DEVIATIONS FROM THE COMMISSION, DECLARED UP FRONT

**D1 — sector targeting is not optional, and it is not quantum-number block-sparsity.** The
commission scopes `(N,Sz)` block structure as optional, "only if N~100 demands it." That is true
of block-sparse *tensors*. It is not true of sector *targeting*: a plain two-site DMRG sweep, with
no symmetry restriction at all, minimizes energy over the **entire** Fock space, and the half-filled
`Sz=0` sector is **not** always where that minimum sits. Worked example, `N=2` sites (the dimer,
`q-seam`'s own closed form, `t=1`): at `U=16`, the single-particle sector `E(N_tot=1) = -t = -1` is
**lower** than the half-filled sector's `E₀(16) = (16-√272)/2 ≈ -0.2462`. Unconstrained energy
minimization converges to the wrong filling entirely — not a bug, the correct answer to the wrong
question.

The fix costs nothing in tensor machinery. The Hubbard model is particle–hole symmetric under
`c_iσ → (-1)ⁱc†_iσ`, self-dual about the chemical potential `μ = U/2`; the standard consequence
(the Mott plateau in `n(μ)` is centered there — Essler, Frahm, Göhmann, Klümper & Korepin,
*The One-Dimensional Hubbard Model*, 2005, §1) is that **the unrestricted ground state of
`H' = H − (U/2)·N̂`, minimized over every sector, sits exactly at half filling.** Same worked
example, now shifted (`μ=8`): `E'(N_tot=2,half) = -0.2462 - 16 = -16.246`, beating `E'(N_tot=2,
Sz=±1) = -16.000`, `E'(N_tot=1) = E'(N_tot=3) = -9`, and `E'(N_tot=0)=E'(N_tot=4)=0` — half filling
wins outright, and the `Sz` ordering within `N_tot=2` is untouched (the shift is a function of
`N̂_tot` alone, so Lieb's `S=0` uniqueness inside a fixed-`N` sector carries through unchanged).
**§2 makes `H'` the working Hamiltonian; §3 (G0) gates the trick numerically before it is trusted.**
This is a correction to what "optional" covers, not a rejection of it: no charge-blocked tensor is
built anywhere in v1.

**D2 — the workspace manifest is not touched by this commit.** `crates/q8-mps` is built with an
explicit `--manifest-path` until the integrator adds the one-line `members` entry, per the
commission. `Cargo.toml` at `sim_engine/` is not edited here.

**D3 — `N=12`'s exact reference is a fresh computation, not a reuse.** `Q_SEAM_PREREG.md` D3 scoped
`N=12` **out** of the Q5/Q6 campaign because Q6's share pass at that size is a `~9e9`-update job.
That reason does not apply to G2/G3/G6 here: they need only `q-seam::lanczos::ground_state` and
`q-seam::observables::ExactObservables::measure`, not the share statistic. So `N=12`'s exact ground
state is computed fresh, as a dev-dependency call into the live `q-seam` crate, at its own cost:
full reorthogonalization stores every Krylov vector, `400 × 853776 × 8 B ≈ 2.7 GB` at the pinned
`MAX_ITERS`. **Declared now, not amended after the fact**: this run is detached compute (house rule
5 — `setsid` + done-marker + `RESUME.md`), and it inherits `q-seam`'s pinned Lanczos policy
(`START_SEED`, one deterministic restart, `RESIDUAL_GATE`, `G-E6` non-degeneracy) unchanged — no
loosened cap to make `N=12` converge faster.

---

## 1. THE REFERENCE FAMILY AND THE BASIS

**Same physics as `Q_SEAM_PREREG.md` §1**: 1D Hubbard, open boundaries, half filling, `Sz=0`,
`t=1`. The MPS uses a **different basis** than `q-seam`'s spin-factorized one, and the difference
is exactly where the named trap lives.

**The chain is `2N` Jordan–Wigner sites in interleaved order**: site `1 = (1,↑)`, site `2 = (1,↓)`,
site `3 = (2,↑)`, …, site `2N = (N,↓)`. Local dimension 2 (occupied/empty), one spin-orbital per
site. Standard JW: `c_j = (⊗_{k<j} Z_k) ⊗ σ⁻_j`, `Z = 1-2n̂`, so `c†_p c_q` (`p<q`) carries a
`Z`-string over every site **strictly between** `p` and `q` (Jordan & Wigner 1928).

**The derived fact that makes G1 meaningful.** For a real-space nearest-neighbour hop of **either**
spin, `(s,σ)–(s+1,σ)`, the two JW-site indices are `2s-1,2s+1` (spin ↑) or `2s,2s+2` (spin ↓) —
distance 2 in JW-site order, always. In both cases exactly **one** site lies strictly between them:
the *opposite*-spin orbital at site `s` (or `s+1`). So **every hopping term in this model carries a
JW string of length exactly 1, independent of `N`** — the interleaved-ordering analogue of
`hubbard.rs`'s "nearest-neighbour hops carry no sign" fact for the all-up-then-all-down ordering.
It is asserted here, not assumed: G1 checks it against an independently-coded dense build. The
on-site interaction `n_{s↑}n_{s↓}` is a plain 2-site density–density term (sites `2s-1,2s`), no
string, since number operators never anticommute past a `Z`.

**Reused rulers (not recomputed).** `q-seam`'s three closed forms gauge this instrument too, called
live through the dev-dependency, never copied: the `U=0` energy/gap table (§1.1(i)), the `N=2`
dimer column at all `U` (§1.1(ii)), and the two exact theorems of §1.1(iii) — `⟨n_iσ⟩=1/2`
everywhere (particle–hole) and `m_i=0` everywhere. G6 below cites the **lighter** warrant for the
second fact: `Q7_SEAM_PREREG.md` §2.2 derives `m_i=0` from spin-independence of `H` plus uniqueness
of the ground state in the `Sz=0` sector alone — no bipartite lattice, no half filling, no Lieb —
which is the correct citation on this file's own terms (D0-d in `Q_SEAM_RESULTS.md` already flags
Lieb as over-strong for this exact fact).

**The validation grid — fixed now, 12 configurations.** `N ∈ {8, 10, 12}` (JW chain length
`{16,20,24}`) `× U/t ∈ {0, 1, 4, 16}` — free, weakly correlated, strongly correlated, and `q-seam`'s
own plant (deep Mott), reusing `q-seam`'s exact grid values so every comparison is exact, never
interpolated. `N=2,4` are used **only** for G0/G1's small-dense checks below, not for the main
grid — they are too small to exercise a real bond-dimension ledger.

---

## 2. THE ANSATZ: TWO-SITE MPS/DMRG WITH A LEDGERED BOND

**Working Hamiltonian, stated once, used everywhere.** All variational optimization (initial state,
local updates, sweep convergence, G3's floor) runs on `H' = H − (U/2)·N̂_tot` (D1). Every **reported**
energy is unshifted with the **integer target** `N_target = N`, never the measured `⟨N̂_tot⟩`:
`E ≡ E'_MPS + (U/2)·N_target`. **Pinned by amendment (2026-08-23, team-lead review)**: using the
measured expectation instead would leak sub-tolerance particle drift into the energy comparison
scaled by `U/2` — at `U=16` a `1e-7` drift in `⟨N̂_tot⟩` turns into an `8e-7` energy wobble that
would look like a G2 mystery rather than the rounding it is. This substitution is invisible
downstream of §3; it is named here once so no gate statement below has to repeat it.

**Bond ledger.** `2N` sites carry `2N+1` bonds, `b=0..2N`, `chi[0]=chi[2N]=1` (trivial boundary).
v1 declares a **single scalar `chi`** applied uniformly, `chi_max[b] = min(chi, 2^b, 2^{2N-b})` —
the physical cap near the chain ends needs no truncation and produces no discarded weight there by
construction. Per-bond *non-uniform* ledgers are a natural later extension, not built in v1.

**Initial state — fixed, no RNG (constraint given).** The Néel-like product state: site `s` (chain
index, `1..N`) carries an up electron if `s` is odd, a down electron if `s` is even — exactly `N/2`
of each, zero double occupancy, deterministic and replayable. In JW-site terms: site `2s-1` occupied
iff `s` odd, site `2s` occupied iff `s` even.

**Sweep procedure.** Two-site DMRG, alternating left-to-right / right-to-left, each local step:
build the effective Hamiltonian on the active 2-site × environment block, diagonalize (Lanczos,
own implementation, no external solver — same house rule as `q-seam`'s reference), SVD the
resulting 2-site tensor with the crate's own Jacobi SVD (below), truncate to `chi_max[b]`, record
the discarded weight for that truncation.

### 2.1 The SVD — validated before it backs anything else

Zero runtime dependencies (matching `q-seam`'s discipline exactly): the crate's own Jacobi SVD,
fixed seed `SVD_FIXTURE_SEED = 1` for every randomized fixture (stated once, replayable).

| Gate | Fixtures | **STAKED** threshold |
|---|---|---|
| G-SVD1 reconstruction, `max\|A − USVᵀ\|` | 3×3 identity; 5×5 and 20×20 seeded-random; 10×10 rank-6 (deliberately rank-deficient) | `≤ 1e-13` |
| G-SVD2 orthogonality, `max\|UᵀU−I\|`, `max\|VᵀV−I\|` | same four fixtures | `≤ 1e-13` |
| G-SVD3 known spectrum, diagonal matrix's singular values reproduce `\|diag\|` sorted | diag(3,1,4,1,5,9) style fixture | `≤ 1e-14` |

Nothing in §2's sweep runs before all three pass.

---

## 3. FOUNDATIONAL GATES — nothing in §4–§6 runs until these pass

A gate failure VOIDs the configuration (excluded, reported), exactly `q-seam`'s discipline: never a
refusal, never a datum.

| Gate | Statement | **STAKED** threshold |
|---|---|---|
| **G0-1** sector-shift check | `H'`'s **unrestricted** ground state (brute-force dense diag, full Fock space, `N=2,4`, all 4 `U`) lies in `(N_tot,Sz)=(N,0)` exactly (integer count) and its shifted-back energy matches `q-seam`'s/the dimer's exact `E0` | integer exact; energy `≤ 1e-10` abs |
| **G0-2** sector-lock anchor (standing, every run, every `N` incl. 100) | `\|⟨N̂_tot⟩_MPS − N\|`, `\|⟨Ŝz⟩_MPS\|`, `⟨(Ŝz_tot)²⟩` on the converged state | `≤ 1e-6` each |
| **G1a** MPO vs. independent dense build | MPO contracted to dense `H`, `N=2` (dim 16), `N=4` (dim 256), interleaved JW ordering, vs. a **separately coded** brute-force second-quantization build (not sharing code with the MPO construction) — this is the fermionic-sign trap named up front | `max\|ΔH\| ≤ 1e-13` |
| **G1b** sector-projected spectrum vs. `q-seam` | eigenvalues of the `N=2,4` dense `H` above, filtered to `(N,0)` by measured `N↑,N↓` per eigenvector, vs. `q-seam`'s exact eigenvalues (dev-dependency call) at the same `N,U`, all 4 `U` | `≤ 1e-10` abs |

**Why `⟨(Ŝz_tot)²⟩` and not just `⟨Ŝz_tot⟩` (pinned by amendment).** `⟨Ŝz_tot⟩ = 0` alone does not
preclude the variational state **mixing** nearly-degenerate `Sz` sectors — an equal superposition
of `Sz=+1` and `Sz=−1` also averages to `⟨Ŝz_tot⟩=0`. This loophole is not academic here: at large
`U` the singlet–triplet gap in a half-filled Hubbard chain is `J ≈ 4t²/U`, small at the plant
(`U=16`), so it is exactly the regime this campaign stresses hardest. `⟨(Ŝz_tot)²⟩ = Σ_ij ⟨S_i^z
S_j^z⟩` is zero for a true `Sz=0` eigenstate and strictly positive for any such mixture, closing the
gap G6 (site-wise magnetization) cannot: G6 catches *local* spin-symmetry breaking, `⟨(Ŝz_tot)²⟩`
catches *global* sector mixing, and neither substitutes for the other. It costs `O(N²)`
two-point-function contractions, cheap and unchanged in cost at `N~100`.

G0-2 and the particle–hole anchor `\|⟨n_i⟩_MPS − 1/2\| ≤ 1e-6` (theorem-pinned, needs no exact
reference — the `Core/SelfAudit.lean` door: an external theorem supplies the constant, so this is
honest self-certification, never a self-audit against the engine's own residuals) run at **every**
configuration, `N=8,10,12` included, and later at `N~100` where no exact reference exists at all.
Reflection symmetry (`⟨n_i⟩=⟨n_{N+1-i}⟩`, `Q7_SEAM_PREREG.md` §2.3) is **not** added as a separate
gate: on this zero-potential family particle–hole already pins the stronger absolute value, so
reflection would add nothing — evaluated, not merely skipped.

---

## 4. GROUND-STATE GATES

| Gate | Statement | **STAKED** threshold |
|---|---|---|
| **G2** energy match, `N=8,10,12`, generous `chi=256` | `\|E_MPS − E_exact\| / \|E_exact\|` vs. `q-seam` (dev-dep, live call) | `≤ 1e-8` rel at `U∈{1,4,16}`; `≤ 1e-6` rel at `U=0` |
| **G2** density profile | `max_i \|n_i^{MPS} − n_i^{exact}\|` | `≤ 1e-6` |
| **G2** double occupancy | `\|d^{MPS} − d^{exact}\|`, `d=(1/N)Σ⟨n_{i↑}n_{i↓}⟩` | `≤ 1e-6` |
| **G6** magnetization anchor | `max_i \|m_i^{MPS}\|` (theorem `=0`, `Q7_SEAM_PREREG.md` §2.2 — spin-independence + `Sz`-sector uniqueness, **not** Lieb) | `≤ 1e-6` |
| **G3** variational floor, primary | `⟨H'⟩_MPS ≥ E'_exact − 1e-9`, at **every sweep**, monotone non-increasing across sweeps | slack `1e-9`, matching `q-seam`'s G-C3 style |
| **G3** variational floor, secondary | `⟨H⟩_MPS ≥ E_exact − 1e-9` (bare energy) — holds **only** once G0-2 passes; a violation with G0-2 clean is a genuine bug, a violation with G0-2 dirty is sector leakage, and the two are reported separately, never conflated | slack `1e-9` |
| **G7** sweep schedule | fixed cap 20 two-site sweeps, early-stop at `\|E_k − E_{k-1}\| ≤ 1e-10`; **no post-hoc extension** — non-convergence by sweep 20 VOIDs the configuration and is reported as VOID | cap 20, tol `1e-10` |

**Why energy alone is not enough (the reused house lesson).** `q-seam`'s own G-C3 exists because a
variational bound is a cheap, independent bug-catcher orthogonal to any single observable; G3 here
plays the identical role, and because `H'` is what the sweeps actually descend on, the *primary*
floor is unconditional while the bare-energy floor is conditional and diagnostic — this asymmetry
is declared now so a G3-secondary failure is never mistaken for a G3-primary one.

**G3-secondary is not a second theorem (pinned by amendment).** The variational bound is proved for
`H'` alone (that is the operator the sweeps minimize). Subtracting the same constant
`(U/2)·N_target` from both sides — exactly the unshift convention pinned in §2 — carries it to the
bare-energy bound unchanged, valid precisely where the state truly sits at `N_target` (G0-2). It is
shifted, not re-derived, and stated once here so it is never re-derived confused downstream.

**N=100 note.** At production size no exact reference exists, so G2/G3-secondary/G1 cannot run.
What survives unchanged: G3-primary (monotone non-increase needs no reference), G0-2 and the
particle–hole anchor (theorem-pinned), G7 (a property of the sweep schedule alone). This is the
`Core/SelfAudit.lean` shape made concrete: correctness *against ground truth* only extends as far
as ground truth extends; correctness *against a theorem* extends everywhere.

---

## 5. G4 — THE CERTIFICATE: discarded weight versus true error

**The chi ladder**, staked now: `chi ∈ {16, 32, 64, 128, 256}`, run at all 12 `(N,U)` configurations
of §1's grid. Per run: **accumulated discarded weight** `ε` = sum of `Σ_{i>chi} s_i²` over every
truncation in the **final** sweep only (earlier sweeps' truncations are corrected by later
optimization and are not representative of the returned state's error — stated now, not decided
after seeing the numbers); **true error** `δE = |E_MPS(chi) − E_exact|` against the live `q-seam`
call.

**The functional form, staked before any point is measured**: `δE = c · ε^p` (power law; test by
linear regression of `log δE` on `log ε`).

**A theoretical prior on `p`, staked (team-lead suggestion, adopted).** Standard first-order
perturbation theory says the energy error is first-order in the total discarded probability weight,
so `p ≈ 1` is *expected*, not merely fitted. **A fitted `p` far from 1 is a diagnostic to report,
not a fact to absorb** — it says the discarded weight is not behaving like a probability-weighted
energy error on this family, and the fit is used only if `R²` and the held-out band (below) still
pass; the prior does not override either staked check.

- **Calibration set**: `N ∈ {8,10}`, all 4 `U`, all 5 `chi` — pooled fit, `(c,p)` and `R²` reported.
- **STAKED: `R² ≥ 0.85`** on the calibration fit, or the form is rejected outright (§5's policy
  fires, see below) — chosen, not derived, and declared as such.
- **Held-out prediction**: `N=12`, all 4 `U`, all 5 `chi` — predict `δE` from the **calibration**
  `(c,p)` and each point's measured `ε`, compare to the actually measured `δE`.
- **STAKED: median `\|log10(predicted/actual)\| ≤ log10(3)`** over the 20 held-out points — a
  factor-of-3 band on a log scale, chosen as a workable engineering tolerance for an error-bar
  policy, not a scientific claim.

**G4 POLICY, fixed before any result exists.**

> If **both** staked thresholds pass: at `N~100`, quote `δE ≈ c · ε^p` using the calibration
> `(c,p)`, always labelled as an extrapolated estimate, never as a bound.
> If **either** fails: **refuse to quote a derived error bar at `N~100` — report the raw
> accumulated discarded weight only**, per the commission's explicit instruction. This is a
> refusal, not a silent widening of the tolerance; the failure itself is reported with its numbers.

Energy is the primary error quantity (matching the commission's phrasing). Density-profile,
double-occupancy and `m_i` deviations are measured alongside and reported, but **cannot change
G4's verdict** — declared secondaries, same discipline as `Q_SEAM_PREREG.md`'s `B4_mean` raw.

---

## 6. G5 — THE REFUSAL GATE, MUTATION-TESTED

**Taxonomy note, stated as the commission requires**: this refusal is **FLOOR-type**. It says *I
cannot see finely enough at this `chi`*, not *there is nothing finer to see* — the
`GrainFloor.lean` header's distinction between its two refusal kinds, and this engine's refusal is
squarely the first kind: a larger `chi` (a re-root to a finer ledger, in that file's language)
serves the same request the current `chi` refuses. It is not a candidate for `GrainFloor.lean`
itself (a different resource, spatial/temporal grain there vs. entanglement/bond-dimension grain
here) but it is the same *shape*, worth naming for whoever writes the next instance.

**The forcing configuration, staked now.** `N=12`, `U/t=0` (the gapless point — smallest gap in the
whole grid, per `q-seam`'s own `Δ(N) ∝ 1/(N+1)` closed form, hence the largest correlation length
and the hardest case for a small `chi`), `chi` capped at **4** — two orders of magnitude below G2's
generous `chi=256`. (The commission's alternative, "a staked quench state," is out of scope: v1 is
ground-state-only, no real-time evolution, so it is not buildable without expanding scope — declared
here rather than silently dropped.)

**STAKED: at least one bond's discarded weight `≥ 1e-4`** at this configuration — the refusal
threshold, an independent chosen number, deliberately *not* derived from G4's fit (a natural later
unification: derive the threshold from G4's calibrated relation plus a target energy tolerance —
flagged, not built, in v1). On firing, the engine returns a typed `Refusal { bond, weight }` naming
the **worst** bond, never a partially-truncated `Ok`.

**The mutation test — run and reported, never asserted.** A second code path, `RefusalPolicy::
Silent`, shares every numeric routine with the real `RefusalPolicy::Typed` and differs only in
whether the discarded-weight check gates the return value. The **same test** exercises both:

> **PASSES iff** `Typed` returns `Err(Refusal{..})` with weight `≥ 1e-4` at the forcing
> configuration, **and** `Silent` returns `Ok(state)` at the identical configuration, silently.

If `Silent` also refuses (because the underlying truncation numerics were changed to refuse
unconditionally, say), the test proves nothing about discrimination and **that is failure**,
reported as such — the mutant must be shown failing, not asserted to.

**G5 KILL, separable.** Fires iff the joint condition above does not hold — either `Typed` fails to
refuse the forcing configuration, or `Silent` also refuses. Firing kills the refusal *feature* only:
G1–G4, G6, G7 are untouched, but no `N~100` run may claim "no silent truncation happened" until it
is fixed.

---

## 7. THE KILLS — separable, stated absolutely, before any data exists

Four, each taking down its own claim and nothing beneath it (epistemology.md rule 2).

> **CORRECTNESS KILL** fires iff any of G-SVD, G0, G1, G2, G3-primary fails at **any** of the 12
> validated configurations. This is fatal to the whole deliverable: the engine does not reproduce
> the exact seam it is certified against, and no `N~100` number may be reported until it is fixed.
> STOP; do not build the `N~100` runner; report the failing gate and its numbers.

> **CERTIFICATE KILL** fires iff G4's calibration `R²` or held-out band fails. Separable from
> correctness — G1–G3 can all pass while this fires, meaning the *energies* are right but no honest
> error bar can be put on ones that have not been checked against ground truth. Policy: raw
> discarded weight only at `N~100`, per §5.

> **REFUSAL KILL** fires iff G5's joint condition fails. Separable from both of the above —
> energies can be correct and the certificate usable while the refusal mechanism itself is
> decoration. Policy: `N~100` energies may still be reported, but flagged that undetected
> under-`chi` truncation is a live, unmitigated risk until fixed.

> **SWEEP KILL** fires iff more than 2 of the 12 configurations VOID under G7 (fail to converge in
> 20 sweeps). Separable — a schedule problem, not a correctness or certificate problem. Policy:
> report which configurations VOIDed and why; the fixed schedule is not silently extended after
> seeing which ones failed.

None of the four kills implies another. All four are reported in the results file's title line, as
loudly as a survival would be, per house rule (the concise-report discipline: verdict first, then
numbers, misfits, deviations).

---

## 8. CREDITS

DMRG — White, *Phys. Rev. Lett.* 69, 2863 (1992). The modern MPS formulation and the standard
sweep/SVD-truncation machinery — Schollwöck, *Ann. Phys.* 326, 96 (2011) (also the standard
reference for symmetry/quantum-number targeting in DMRG, which v1 deliberately does *not* build).
Area laws, the reason a bounded `chi` can work at all in 1D — Hastings, *J. Stat. Mech.* P08024
(2007). The Jordan–Wigner transformation — Jordan & Wigner, *Z. Phys.* 47, 631 (1928). The
particle–hole / chemical-potential half-filling argument (D1, G0) is standard Hubbard-model
practice, not new here — Essler, Frahm, Göhmann, Klümper & Korepin, *The One-Dimensional Hubbard
Model*, Cambridge (2005), §1; Lieb & Wu, *Phys. Rev. Lett.* 20, 1445 (1968) for the exact solution
the plateau argument rests on. **Not standard, and this programme's own**: the ledger reading of
bond dimension as a declared, auditable resource; the discarded-weight-to-error certificate and its
calibrate/hold-out/refuse-to-quote policy (§5); and G0's specific numeric demonstration on the
`N=2` dimer. The algorithm is White's, Schollwöck's, and Hastings's; the refusal discipline wrapped
around it is `GrainFloor.lean`'s pattern, applied here for the first time to a new resource.

---

## 9. FILES AND DETACHED COMPUTE

Prereg: `sim_engine/Q8_MPS_PREREG.md` (this file). Crate (post-GO): `sim_engine/crates/q8-mps`, no
`[workspace]` stanza, built via `--manifest-path` until the integrator adds the `members` line
(D2). Dev-dependency: `q-seam` (path, no features), called live for every exact comparison above —
never copied, never re-derived by hand.

Two jobs cross the ~5-minute house-rule line and get `setsid` + a done-marker + `RESUME.md`: the
`N=12` exact reference (D3, ~2.7 GB resident, `q-seam`'s pinned Lanczos policy unchanged), and the
G4 chi-ladder sweep (`12 configs × 5 chi = 60` DMRG runs, `N` up to 12 / `2N=24` sites). Neither
runs before the GO on this prereg.
