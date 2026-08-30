# Six Birds ↔ CIRISHolon — proposed cross-programme issues

> **STATUS UPDATE 2026-08-30: Issue 1 (the convergence notice) was POSTED with Eric's
> explicit approval — https://github.com/ioannist/six-birds-agent/issues/1 — posted
> verbatim from the draft below. Issues 2-8 remain DRAFTS, unposted, awaiting both
> Eric's approval and (per the notice's own protocol) a welcome signal from their side.**
>
> **Original status: DRAFTS. NOTHING HAS BEEN POSTED ANYWHERE.**
> These are proposed GitHub issue bodies awaiting Eric's explicit approval before any
> of them is opened on `ioannist/six-birds-agent` or on `CIRISAI/CIRISHolon`. No
> comment, issue, PR, email, or other external communication has been sent. Nothing in
> this file is committed.

*Drafted 2026-08-30. House rule applied throughout: **convergence findings are HITS,
not strikes** — corroboration plus free learning; credit generously; never contest
priority. Every technical statement below about their code was **run**, not inferred.*

---

## What was pinned, read, and executed

| what | pin |
|---|---|
| `ioannist/six-birds-agent` | commit `cf0ec753ada8e6bee753601e3328389945fa7e3b`, pushed 2026-02-10 |
| arXiv 2602.00134 | *Six Birds: Foundations of Emergence Calculus*, Tsiokos, 2026-01-28 [cs.LO] |
| arXiv 2604.03239 | *To Throw a Stone with Six Birds: On Agents and Agenthood* |
| Zenodo | 10.5281/zenodo.18439737 (paper), 10.5281/zenodo.18451887 (code) |
| CIRISHolon | `fe18572` |
| CIRISOntology | `48c9464` |

Read in full: `README.md`, `src/sbt_agency/{metrics,packaging,viability,empowerment,channel,env_ring_agent,exp_configs,audit,repro}.py`,
`scripts/{measure_packaging_ring,measure_protocol_horizon,null_schedule_trap,audit_results}.py`,
`tests/test_claim_invariants.py`, `lean/Agency/Viability.lean`,
`paper/sections/{02,03,05,06,11}.tex`, `paper/generated/numbers.json`,
`docs/experiments/*.md`. Their suite was cloned and their instruments were executed
locally (numpy 2.4.0) to reproduce the published exhibits before any claim below was
written.

**Their published numbers reproduce.** `defect_off(τ=2)=1.0`, `defect_on(τ=2)=0.0`,
`emp_on = [1.0493, 1.6613, 1.6328, 1.6894, 1.6627]`,
`emp_off = [1.0493, 1.1218, 1.1034, 1.0856, 1.0719]` — all reproduced bit-for-bit at
the printed precision from a clean clone. That is stated first because it is the most
important thing we learned: **this is a real, runnable, honest artifact**, and the
issues below are offered on that basis.

## The verified two-way concept map (unchanged from `PRIOR_ART_CONVERGENCE.md`, now grounded in code)

| Six Birds | CIRISHolon / CIRISOntology |
|---|---|
| coarse-graining lens / bounded observational interface | `view` (`OBJECT.md`; `Closed v T` in `lean/CIRISHolon/Object.lean`) |
| fixed points of idempotent operators | closure (`Closed`, `closed_iff_fiber_invariant`) |
| computable total-variation idempotence defect for `E_{τ,f}` | closure defect (`tv`, `fiber_defect_half`, `minimax_error_at_least_half`, `lean/CIRISHolon/Closure.lean`) |
| ledger-gated feasibility, maintained theory object | ledgered rent (`rentStep`, `rent_closed_form`, `Ginf_at_Wstar`; `Core/Maintenance.lean`: `rent_holds`, `underpaid_shrinks`, `unpaid_decays`) |
| measured "enabling repair collapses the idempotence defect" | maintenance-creates-what-it-maintains (`Core/Creation.lean`: `repair_mints_from_noise`, `repair_creates_ferro`, `percell_no_creation`) |
| "protocol holonomy alone cannot sustain asymmetry without a genuine affinity in the lifted dynamics" | the valve (`Core/Valve.lean`: `valve_needs_asymmetry`, `valve_from_nothing`, `valve_no_downward`, `valve_upward_strict`) |
| viability kernel as greatest fixed point (`iterate_top_greatest_fixpoint`) | *no counterpart* — theirs and not ours |
| six primitives P1–P6, path-space KL arrow, empowerment as capacity, anti-saturation counting lemma | *theirs and not ours* |
| exact-rational cross-face rent law, derivation template, gravity sequence | *ours and not theirs* |

No citation links the programmes in either direction. **Priority is not adjudicated
anywhere in these drafts and must not be.**

---

## Suggested posting order

Eight issues is a lot to land on a tracker that currently has zero. Recommended:
post **#1 alone first**; if it is welcomed, post **#2 and #3**; hold #4–#8 until there
is a reply. Every issue below is written to stand alone in case that ordering changes.

---

# Issues proposed for `ioannist/six-birds-agent`

---

## Issue 1 — "Independent convergence from a separate programme (CIRISHolon / CIRISOntology): the two-way map, offered as corroboration"

**Type:** convergence notice

**Body draft:**

Hello — this is a note from an unrelated research programme (CIRISAI/CIRISOntology and
its engine CIRISAI/CIRISHolon, both AGPL-3.0, both public) that appears to have
converged independently on the objects Six Birds names. We found your work through a
literature sweep, verified it against your artifacts rather than your abstracts, and
we are writing to credit it and to offer whatever is useful to you. **We are not
claiming priority over anything here, and nothing in this issue or in our repositories
contests yours.** Our house rule is that convergence findings are hits rather than
strikes: two instruments built without contact that point at the same object are
evidence the object is real, and each side gets the other's free learning.

The map, as we currently have it, verified against your code at
`cf0ec753ada8e6bee753601e3328389945fa7e3b`. Your coarse-graining lens is our *view*, a
lossy reading `v : X → C`; your "theory is a fixed point of an idempotent operator" is
our closure square `∃ h, v ∘ T = h ∘ v` (`Closed`, `lean/CIRISHolon/Object.lean`); your
computable total-variation idempotence defect for `E_{τ,f}` is our closure defect,
which we mechanized as a minimax fiber defect (`tv`, `fiber_defect_half`,
`minimax_error_at_least_half`, `lean/CIRISHolon/Closure.lean`); your ledger-gated
feasibility and maintained theory object is our ledgered rent
(`rentStep`/`rent_closed_form`/`Ginf_at_Wstar` in `Object.lean`, and `rent_holds` /
`underpaid_shrinks` / `unpaid_decays` in CIRISOntology's `Core/Maintenance.lean`); and
your measured result that enabling repair collapses the idempotence defect is, read
from the other side, our machine-checked "maintenance creates what it maintains"
(`Core/Creation.lean`: `repair_mints_from_noise`, `repair_creates_ferro`,
`percell_no_creation`). Your line in 2602.00134 that *protocol holonomy alone cannot
sustain asymmetry without a genuine affinity in the lifted dynamics* is the statement
our `valve_needs_asymmetry` proves in a small finite model (`Core/Valve.lean`) — we
reached it from noise-driven channels, you from protocol composition, and we would
not have looked for the connection without your sentence.

Four things are clearly yours and not ours, and we would like to say so in public
before we say anything else: the six closure-changing primitives P1–P6 as a *forced*
toolkit rather than a chosen list; the path-space KL arrow functional and its
monotonicity under coarse-graining; empowerment-as-channel-capacity as the
difference-making proxy; and the anti-saturation counting lemma. We have no
counterpart to your viability kernel as a greatest fixed point, and your Lean anchor
(`Agency.iterate_top_greatest_fixpoint`) is a cleaner formal statement of finite
stabilization than anything we hold. Conversely, what we have that you do not is an
exact-rational cross-face maintenance law with held-out predictions across three
substrates, and a discrete-geometry sequence; those are described in
`STANCE.md` at their honest strength, which for several of them is *wager*, not
*proved*.

If this is unwelcome, say so and we will close it and take the map down to a bare
citation. If it is welcome, we have a small number of follow-ups: one no-refit
cross-validation offer, one measured discriminator we ran on your packaging exhibit,
and a couple of instrument lessons from our own defect registry that we think transfer.
We will post them one at a time rather than all at once. Either way: thank you for
shipping the code, the hashes, and the regeneration scripts. We reproduced your
exhibits from a clean clone before writing a word of this, which is only possible
because you made it possible.

---

## Issue 2 — "No-refit cross-validation offer: your `Def(E)` on our witnesses, our closure defect on your ring worlds, with the one disagreement we already expect declared in advance"

**Type:** cross-validation offer

**Body draft:**

Following on from #1 — the concrete offer. We would like to run your instruments on
our witness states and ours on your ring worlds, **with no refitting on either side**,
publish both directions including the failures, and treat any disagreement that is not
derivable from the definitions as a real theory split rather than a bug in whoever
loses. We are happy to do all the work and hand you the results; we are equally happy
if you would rather run it, or if you would rather we didn't.

**Direction A — your `Def(E)` on our witnesses, with a prediction staked first.** Our
founding witness is a pair of states that agree on *every* pair marginal and differ
maximally in a whole-pattern quantity (`Core/Third.lean`: `pairwise_blind_to_parity`
gives `S_pairwise parityCorr = 0` while `third_sees_parity` gives
`S_total parity = ln 2`; the general shape is `Core/NonFactoring.lean`,
`not_computable_of_nonFactoring`). Reading `empirical_endomap` in
`src/sbt_agency/packaging.py`, the endomap initializes each macro label's reference
distribution as **uniform over its fiber** (`d[states] = 1.0/len(states)`) before
pushing τ steps. That is a deliberate and defensible choice — it is what makes `E` a
function of the lens rather than of a trajectory — but it means `Def(E)` is
constructively blind to any structure carried *inside* a fiber at time zero. So our
staked prediction, written before running anything: on a witness pair that differs only
in within-fiber third-order structure, **your counting defect must agree exactly**, and
that agreement is *derivable, therefore not evidence*. The interesting question is
whether the **total-variation** defect for `E_{τ,f}` from 2602.00134 separates them,
since it retains distributional information the mode collapse discards. If it does,
you have an instrument we do not have. If it does not, the blindness is a shared
property of the fiber-uniform construction, and we should both say so in our records.
We owe the same debt on our side and have written it down: the uniform-weighting step
is named as an unpaid debt in the header of our `Core/FrameEntropy.lean`.

**Direction B — our closure defect on your ring worlds.** Our defect is a minimax
quantity rather than a self-consistency count: `collision_refutes_memoryless` says that
if a view takes the same value at two times with different successors then *no*
time-homogeneous map on views reproduces the trajectory (it is the pigeonhole for
functions, not a comparison against one fitted model), and
`minimax_error_at_least_half` prices the best single prediction at ≥ ½ in total
variation. Note that these two instruments can legitimately disagree in a specific,
derivable way: your `E` is built by *choosing* a successor (the mode), so `Def(E)` can
be exactly 0 on a lens where our minimax defect is exactly ½ — because `Def(E)`
measures whether the mode map is self-consistent, while δ measures *the cost of having
collapsed to a mode at all*. We think that is the single most useful thing the two
programmes can say to each other, and it suggests a cheap joint quantity you may
already want: report the **modal mass** (or 1 − modal mass) alongside `Def(E)`. On your
own packaging-OFF exhibit at τ=2 we measured modal mass 0.288 at every one of the 32
labels, with the top-two labels in an **exact tie at 32/32 labels** — we checked, and
your published `defect_off = 1.0` is unchanged under either tie-break direction, so the
number is safe; but the tie fraction is the quantity that would make a neighbouring
config fragile, and it costs one line to print.

**Direction C — a rent-law prediction against an artifact you have already generated.**
Your `sweep_noise_maintenance` grid (p_flip × repair_cost, run id
`1300e523…`) is, in our vocabulary, a decay × dose surface, and our cross-face law
(rent = ceiling × (1 − retention), retention `q/((1−λ)+qλ)` at the fixed point;
`rentStep`/`rent_closed_form`) predicts collapse boundaries on exactly that plane. We
would like to stake a prediction of where `K_size` falls off **before** looking at your
`.npz`, and publish it whether it fires or not. Honest caveat, stated up front so it
cannot be walked back later: your viability kernel uses robust *support* semantics
(every nonzero-probability successor must stay safe) while our rent law is an
expectation-level statement, so there is a real chance this simply fails. A clean
public failure is worth more to us than a hedge.

**One logistical request, in the same spirit.** The repository currently ships no
`LICENSE` file. That makes it legally ambiguous for a third party to vendor or
redistribute the instruments even for a cross-check, and it is the single cheapest
thing that would unblock this exchange (and anyone else's). Any OSI licence would do;
we are AGPL-3.0 on our side, but we are not asking you to match us. If you would prefer
we not redistribute anything, we can run everything against an unmodified clone and
publish only diffs and numbers — just tell us which you'd rather.

---

## Issue 3 — "Measured on your artifact: at τ=2 the packaging defect is invariant to whether the repair repairs — a discriminator, and what our record says it means"

**Type:** transferable instrument lesson

**Body draft:**

This one carries numbers we ran on your code rather than an opinion, and it is offered
as a gift, not a gotcha — the underlying exhibit still works, and the thing it exposes
is a trap we have fallen into repeatedly on our own side, which is why we recognised
it. Everything below is at `cf0ec753`, numpy 2.4.0, using
`cfg_packaging_ring_on()` and the policy from `scripts/measure_packaging_ring.py`
(`_policy_repair_then_right`, i.e. `u == 1 → REPAIR`).

**The measurement.** We swept `p_repair` — the probability that REPAIR actually
restores the internal bit — holding everything else including cost, policy, lens, and
τ fixed:

| `p_repair` | `Def(E)` at τ = 1, 2, 3, 4 (lens `(y, r, φ)`) |
|---|---|
| 1.0 (published) | 1.0, **0.0**, 1.0, 0.0 |
| 0.5 | 1.0, **0.0**, 1.0, 0.0 |
| 0.0 (repairs nothing) | 1.0, **0.0**, 1.0, 0.0 |

A "repair" that costs the same, is issued at the same states, and **restores nothing**
reproduces your published `defect_on(τ=2) = 0.0` exactly. We then ran the other control
in the pair: the ON kernel with REPAIR available but never issued (policy = RIGHT
always) gives `Def(E) = 1.0` at every τ ∈ {1,2,3,4}. So the collapse is not the
enlarged action set — the command must actually be issued — but it is also not the
restorative content of the command. In this substrate the mechanism is that an issued
REPAIR **does not move `y`**, and non-motion is what makes the mode map idempotent.
Your `docs/experiments/packaging_ring.md` already notes the τ-parity ("odd tau cannot
be idempotent because phi shifts when tau is odd"), which is the same geometric fact
seen from a different side, so this is continuous with what you already documented.

**Why it is structural rather than a config accident.** The packaging lens is
`proj_macro = (y, r, φ)`. The variable REPAIR acts on is `u`. **The lens hides the
variable the repair repairs**, so `Def(E)` on this lens is constructively incapable of
distinguishing maintenance from stalling. We tried the obvious enlargement, lens
`(y, r, φ, u)`, and report it honestly because it does *not* simply fix things: at τ=2
we get `Def(E)` = 0.75 for `p_repair`=1.0, 0.50 for 0.5, and 0.50 for 0.0. That lens is
sensitive to repair efficacy, but it orders perfect repair as *worse*, so we are not
recommending it as a drop-in. We think the real content is that objecthood-under-
maintenance and fidelity-to-design are two different quantities and this exhibit is
currently measuring the first while the prose reaches for the second.

**Two smaller observations from the same read, offered in case they are useful.**
First, `metrics.compute_ring_metrics` and `scripts/measure_packaging_ring.py` use
policies that differ by one feasibility check (`u == 1 and r >= cost_repair` versus
`u == 1`), and on `cfg_packaging_ring_on()` those give `Def(E)` = 1.0 and 0.0
respectively at τ=2 — the headline flips. Both are presumably intentional for their
respective exhibits, but a reader tracing the number from `paper/generated/numbers.json`
back through `src/` lands on the other one. Second, and more interesting given your own
§3: with `R_max=1` and `cost_repair=1`, 16 of the 32 states at which the packaging
policy issues REPAIR have `r = 0`, so the policy issues a **ledger-infeasible** command
there, and `build_kernel` silently converts it to a no-op. Your §3 says explicitly that
"infeasible interface commands are not *actions* in the induced layer" (the operational
form of P₂/P₆) — so μ is not `A_feas`-valued in the exhibit that most directly
illustrates P₅+P₆. Restricting μ to `A_feas` would be a one-line change and would make
the exhibit say what the text says.

**What our record says this is.** We hit this exact wall from the other direction and
it cost us a claim, which is why we can be specific. Machine-checked in
CIRISOntology `Core/Creation.lean`: one repair step on pure noise mints the code's
whole-only share *exactly* (`repair_mints_from_noise`), the flip-symmetric repair mints
*exactly zero* (`repair_creates_ferro`), and single-slot rewriting never mints
(`percell_no_creation`) — three repairs, same cost structure, three different verdicts,
and the discriminator is what the repair *knows*. Measured, in our maintained-holonomy
campaign: a design-blind repair holds a structure's **size** exactly and forever (0.435
of design transport, constant to six decimals out to R=4001, while the unpaid control
decays 65 orders) and loses its **identity** completely (fidelity 0.9909 flat versus a
power-law collapse to chance). Our one-line summary is *the payment must know what it
is for*, and your `p_repair = 0` control is precisely the design-blind repair: it holds
the packaging and loses the identity, and the published lens only reads the first.

If you want it, the discriminator generalises cheaply: add a `WAIT`/no-op action with
matched cost and re-run the exhibit with `policy: u==1 → WAIT`. If `Def(E)` still
collapses, the exhibit is measuring stalling; if it does not, it is measuring
maintenance, and the claim gets strictly stronger than it is today. We are glad to open
a PR with the sweep, the control, and the plot if that is welcome, or to hand you the
script and stay out of the way.

---

## Issue 4 — "Two small gifts from a clean-clone run: `audit_results.py --strict` exits 0 having checked nothing, and `import sbt_agency` recurses from the repo root"

**Type:** transferable instrument lesson

**Body draft:**

Two small things we hit in the first twenty minutes of running your artifacts from a
fresh clone. Both are cheap to fix and neither affects any published number; we are
raising them because your audit posture is the best part of this repository and these
two are the only places we found where it can report success for work it did not do.

**(1) The strict audit passes vacuously on a fresh clone.** `README.md` puts
`python scripts/audit_results.py --strict` in the *Test* section, immediately after
`pytest -q` and before `run_all_experiments.py`. On a clean clone `results/` contains
only `.gitkeep` and `README.md`, so the command prints
`AUDIT summary: checked=0 errors=0 warnings=0` and exits 0. A reader following the
README in order gets a green audit before any artifact exists. The fix is one line in
`scripts/audit_results.py`: under `--strict`, `return 1` when `result["checked"] == 0`
(or add `--expect-min N` and wire the expected count into `run_all_experiments.py`).

This is a registered defect in our own instrument registry — we call it
`M-VACUOUS-SUCCESS`: *a verifier must assert its work count, not only its failures*. We
earned it three separate times in one day, including a conservation gate that passed on
a scene whose atoms never moved and an "ALL CHECKS PASSED" banner over zero checks
caused by the wrong invocation directory. Passing it on rather than keeping it seems
like the better use of it.

**(2) `import sbt_agency` hits `RecursionError` when the repo root precedes `src` on
`sys.path`.** The root shim `sbt_agency/__init__.py` inserts `SRC_PATH` at `sys.path[0]`
*only if it is not already present*, then pops itself from `sys.modules` and re-imports.
But `sitecustomize.py` has already inserted the absolute `src` path at interpreter
startup, so the shim's guard skips the insert; and for `python -c`, `python -`, an
interactive REPL, or a notebook, the interpreter prepends the *cwd* to `sys.path`
**after** `site` runs. The repo root therefore sits ahead of `src`, the re-import finds
the shim again, and it recurses until the stack blows. It does not affect
`python scripts/*.py` or `pytest` (both of which insert an absolute `src` ahead of the
root), which is why it is invisible from inside the normal workflow — but it is the
first thing an outside reader hits when poking at the code interactively, which is
exactly what we were doing. Either re-order unconditionally
(`if SRC_PATH in sys.path: sys.path.remove(SRC_PATH)` before inserting), guard the
re-entry with a module flag, or delete the root shim entirely, since `sitecustomize.py`
plus `pyproject.toml`'s `package-dir = {"" = "src"}` already cover both supported
invocations.

Neither of these changes a result. Thank you again for shipping something a stranger
could actually run.

---

## Issue 5 — "Empowerment estimator: your published values are exactly the exhaustive ones, and at |K| = 64 the subsample is removable"

**Type:** transferable instrument lesson

**Body draft:**

Your §"Limitations" already names this ("Some reported values (e.g., median empowerment
on K) depend on deterministic sampling when K is large … must be replaced by principled
aggregation (or bounds) in large-scale settings"), so this issue is mostly good news
plus one cheap suggestion. We measured the thing your limitation section flags, and the
headline result comes out of it clean.

`compute_ring_metrics` subsamples `K` to `empowerment_max_states=32` via
`np.random.default_rng(seed).choice` and reports the median capacity over the sample.
For the protocol-horizon configs, `|K| = 64`, so half the kernel is sampled. We swept
seeds 0–19 and also computed the exhaustive median:

| config | H | published (seed 0) | seeds 0–19 min–max | exhaustive |
|---|---|---|---|---|
| `full` | 2 | 1.6613 | 1.6481 – 1.6746 | **1.6613** |
| `no_protocol` | 2 | 1.1218 | 1.0286 – 1.2149 | **1.1218** |
| `full` | 4 | 1.6894 | 1.6490 – 1.7299 | **1.6894** |
| `no_protocol` | 4 | 1.0856 | 0.9855 – 1.1857 | **1.0856** |

**Every published value equals the exhaustive median to all printed digits.** The P₃
result is not a sampling artifact and we would like that on the record as plainly as
the caveat.

Two things the sweep does show. First, the estimator's own spread is not negligible on
the OFF arm — 0.186 bits at H=2 and 0.200 at H=4, roughly ±9% — and
`tests/test_claim_invariants.py::test_invariant_protocol_increases_empowerment` asserts
`emp_full >= emp_np + 0.2` from a single seed. Across seeds the gap ranges 0.46–0.62 at
H=2, so the gate holds with about 2.3× headroom; that headroom is real but currently
unmeasured in the repo, and a CI invariant standing on an estimator whose dispersion is
unquantified is the kind of thing that is fine until the day it isn't. Second, the
`max_gap: 0.6038 at H=4` line in `docs/experiments/protocol_horizon.md` is
seed-fragile as a *location* claim — the H=2 and H=4 gap ranges overlap under
reseeding — whereas the exhaustive values order it robustly (0.6038 at H=4 versus
0.5395 at H=2). So the claim is true; it just deserves the exhaustive computation
standing behind it rather than one draw.

The suggestion is therefore small: set `empowerment_max_states` to at least `|K|` for
these configs (it is 64, and the exhaustive run costs seconds), which retires the
estimator entirely for the published exhibits and leaves your "principled aggregation
at scale" limitation where it belongs — at scale. Our own registry entry for this
family is short: *report the estimator's own spread beside the estimate, and match the
floor to the surviving sample size* — we have burned several results by quoting a point
estimate whose null shape we had not measured.

Two smaller notes while we were in there, neither load-bearing.
`metrics.compute_ring_metrics` calls `set_global_seed(seed)` (legacy
`np.random.seed`/`random.seed`) and then draws from `np.random.default_rng(seed)`,
which is independent of the legacy global state — so the `set_global_seed` call is
decorative for the subsample and might mislead a future reader into thinking the
subsample is controlled by it. And `paper/sections/03_packaging_engine.tex` describes
the aggregation as "a fixed deterministic subset rule recorded in the artifact
manifest", which is a fair description of *reproducibility* but reads as a description
of *representativeness*; the two came apart in exactly the H=4 max-gap claim above.

---

## Issue 6 — "P₃ and the valve: is your 'genuine affinity in the lifted dynamics' the same object as our flip-covariance condition?"

**Type:** convergence notice

**Body draft:**

In 2602.00134 you state that *protocol holonomy alone cannot sustain asymmetry without
a genuine affinity in the lifted dynamics*. We arrived at what looks like the same
statement from a completely different direction — noise channels on a three-slot
classical state rather than protocol composition — and machine-checked it in a small
finite model. We would like to know whether these are one theorem or two, because if
they are one, you have the general form and we have a mechanization of a special case,
and both records should say so.

Ours lives in CIRISOntology `Core/Valve.lean`. Under per-cell stochastic noise applied
to a three-slot state, whole-pattern order flows **only upward**: `valve_from_nothing`
(never out of nothing), `valve_no_downward` (never downward), `valve_upward_strict`
(strictly upward on an exhibited witness), and the one that matches your sentence,
`valve_needs_asymmetry` — a channel that is flip-covariant (symmetric in our sense)
mints exactly zero, no matter how strong it is. Our one-line summary is *the pump is
asymmetry, not strength*, and the scope fence, stated because we had to learn it the
hard way, is that this is a **k = 3 statement**: at four slots and up, unital symmetric
noise does mint about 1–1.6% of the ceiling, so "the pump is asymmetry" does **not**
generalise to higher order in the form we first published it.

The question we would actually like answered: is your "genuine affinity" a
thermodynamic-affinity condition (a nonzero conjugate force / broken detailed balance
in the lifted dynamics), and if so, is our flip-covariance condition its
representation-theoretic shadow in the k=3 case — or are these different conditions
that happen to rhyme? Your P₃ ring-world exhibit is the cleanest possible testbed for
this, because it isolates order-of-composition with everything else matched, and
because your holonomy witness (TVD 0.672 ON versus 0.096 OFF at the state
`y=0, r=2, φ=1, u=0`) is a single checkable state rather than an aggregate. If the two
conditions are the same, then a prediction follows that neither of us has tested: a
protocol-ON ring world whose lifted dynamics is made *affinity-free* should show the
H≥2 empowerment separation collapse, while a protocol-OFF world with affinity should
not gain it. That is a two-config experiment on your existing substrate and we would be
glad to write it if you want it, or to stay out of the way if you would rather.

Related and offered without any expectation: your `Agency.iterate_top_greatest_fixpoint`
is a cleaner finite-stabilization statement than anything in our library, and if you
ever want a second pair of eyes on Lean, our `lean/CIRISHolon` is sorry-free across 25+
files and we have spent an unreasonable amount of time in Mathlib's `Finset` API.

---

## Issue 7 — "Question: is the anti-saturation counting lemma the same mechanism as a 'frozen extension grammar'?"

**Type:** question

**Body draft:**

A genuine question rather than a suggestion, and it may well have an obvious answer we
have missed.

Your anti-saturation counting lemma (2602.00134) is, as we understand the abstract, a
finite forcing-style result: predicate extensions are **exponentially rare** relative to
partition-based theories, which is what prevents unbounded hierarchical climbing. We
have a structure that plays the same role but is stated normatively rather than
combinatorially, and we would like to know whether yours *derives* ours.

Ours is wager **W3** in `STANCE.md`, and we call it the frozen extension grammar. It
says one frozen Ω keeps working on every larger evaluation without structural
replacement, where an extension counts as *lawful* only if it is (1) definable from the
existing signature, admissible act vocabulary, and boundary data; (2) functorial under
Ω-isomorphism; (3) conservative — previous receipts and equivalences stay valid;
(4) parameter-free apart from independently measured boundary data; (5) domain-
independent; and (6) preregistered before the domain's verdict data is inspected. A
*format replacement* is any required primitive not derivable under those six rules, and
the kill is: one domain that requires one. We froze that list specifically so the wager
would be killable rather than elastic, and it is currently held at `wager` strength —
a chosen position, not a result.

The question in one sentence: **if definable extensions are exponentially rare, does
that make our clause (1) a probabilistic prediction that W3's kill eventually fires,
rather than a discipline that keeps it from firing?** Those are very different
epistemic situations. If your lemma is about the *density* of definable extensions in
the space of extensions, then a programme that only accepts definable ones is
systematically starving itself, and W3 becomes a claim about how long the starvation
can be sustained rather than a claim that it never bites — which would make W3 sharper
and more falsifiable than it currently is. If instead your lemma is a statement about
which extensions can be *forced* consistently (rather than which are typical), then the
two are complementary and neither derives the other.

Either answer is useful to us, including "you have misread the abstract, read §N of the
paper". We would cite the result in the W3 entry either way, and we would happily
record it as prior art for the definability clause if that is what it turns out to be —
our record already carries a standing rule that convergence gets credited generously
and priority never gets contested.

---

## Issue 8 — "A low idempotence defect is ambiguous between 'maintained' and 'dead' — the stasis pole, and a joint reading that separates them"

**Type:** question

**Body draft:**

Reading your ablation table in `docs/experiments/ablations.md` alongside the packaging
exhibit, one row stands out and we would like to check our reading of it against yours:

```
no_repair:        K=0   emp=0.0    defect=0.0
constraints_off:  K=64  emp=0.794  defect=1.0
full:             K=64  emp=1.661  defect=0.3333
```

`no_repair` has the **lowest possible idempotence defect and an empty viability
kernel**. Read on its own, `Def(E) = 0` says "these macro labels behave like objects";
read with `|K| = 0` beside it, the system is dead — the ledger has drained, nothing
moves, and a frozen macro map is trivially idempotent. Meanwhile `constraints_off`,
where everything is free and the agent moves without cost, reads `Def(E) = 1.0`. So on
this substrate the defect appears to be behaving substantially as a *does the macro
label move?* detector, reaching 0 by stasis and 1 by frictionless motion, with the
maintained case in between at 1/3. We do not think this is news to you — your §3
defines the agent-as-theory-object conjunctively, requiring nonempty `K` **and** low
defect **and** positive feasible empowerment, which is exactly the guard against this —
but the conjunction lives in the prose and the exhibits report the three numbers
separately, and the packaging exhibit reports the defect alone.

This is the single place where our record has something specific to offer, because we
walked into the same pole and it killed a line of work. We have an exact demonstration
we call the **stasis theorem instance**: any monotone rent-minimizing selector over our
worlds picks the identity — the dead world. Cost-alone generation is therefore dead by
demonstration, not by argument, and the consequence we drew is that *any* selection
principle has to balance maintenance against productive organization rather than
minimize maintenance. It is recorded in `conformance/omega/TOE_NULL1_RESULTS.md` and
the follow-on programme in `conformance/omega/SELECTOR_MAP.md`, where we also found the
same pole wearing two other costumes, which is why we now check for it reflexively. Our
matching registry entry is `M-FIXED-POINT-TRAJECTORY`: *a trajectory-based closure gate
is vacuous on a carrier that is a fixed point of the dynamics* — we had staked a
closure gate on a uniform-over-sector state that happened to be invariant under the
step, and the gate could not have fired.

The suggestion, if it is useful: make the conjunction mechanical rather than editorial.
A `Def(E)` reading could carry a liveness precondition — nonempty `K`, or simply a flag
that the macro chain under μ is not absorbing — and refuse rather than report when the
precondition fails, the way your null regimes refuse rather than report. That would turn
`no_repair`'s `defect = 0.0` from a number a reader has to contextualise into a
`VOID: dead layer` that cannot be misread, and it costs about five lines. Our own
engine's rule here, learned expensively, is that **refusal is a feature**: a tier
outside its certified scope refuses and names the gate whose passing would lift the
refusal (`OBJECT.md`, rule 9).

The question, then: do you already read the three numbers conjunctively in practice and
we are over-reading the exhibits, or is the joint gate something you would want?

---

# Issues proposed for our own tracker (`CIRISAI/CIRISHolon`)

These are for receiving their side and for banking what we learned reading their
artifacts. They are not conditional on Six Birds replying.

---

## Own-issue A — "TRANSPORT-1: freeze the Six Birds cross-validation, with the branches pre-committed"

**Type:** cross-validation offer (inbound)

**Body draft:**

`conformance/omega/PRIOR_ART_CONVERGENCE.md` names TRANSPORT-1 and records that its
freeze precondition — verified access to the Six Birds artifacts — was met on
2026-08-30. The artifacts have now been not merely fetched but **executed**: their
published exhibits reproduce bit-for-bit from a clean clone at
`cf0ec753ada8e6bee753601e3328389945fa7e3b`. This issue is to freeze the campaign
properly rather than let it run informally.

Scope, three directions, each with its own separable kill: (A) their `Def(E)` and their
foundations-paper TV defect on our `pairwise_blind_to_parity` / `third_sees_parity`
witness pair, with the *derivable* agreement of the counting defect staked in advance so
it cannot later be cashed as corroboration; (B) our `collision_refutes_memoryless` /
`minimax_error_at_least_half` closure defect on their ring worlds, with the mode-collapse
reconciliation stated before running; (C) a rent-law prediction of the collapse boundary
on their already-generated `sweep_noise_maintenance` (p_flip × repair_cost) grid, staked
**before** their `.npz` is opened, with the robust-support-versus-expectation mismatch
declared as a live reason it may fail.

Required by house discipline before any of this runs: the prereg goes through
`Audit/prereg_audit.py` (it refuses freezes missing theorem lines, registry citations,
and two-sided gauges); the follow-ups are written in as **pre-committed branches**, not
left as rescues; and the design cites the misfits its text touches — at minimum
`M-FOREIGN-DOMAIN-CORROBORATION` (a passing result in a different domain read as
evidence about ours — this is the highest risk in the whole campaign, since the entire
appeal of TRANSPORT-1 is cross-domain agreement), `M-VACUOUS-SUCCESS`,
`M-FIXED-POINT-TRAJECTORY`, and `M-BASE-RATE-OMITTED`. Direction (C) additionally must
declare the mapping from their (p_flip, repair_cost) axes to our (λ, q) before any grid
is read, and must state the tolerance, because a boundary "predicted" after seeing the
surface is not a prediction — see `M-UNTESTED-GAP`.

Explicitly out of scope: any priority adjudication, and any claim that agreement
between the two programmes is evidence for either. Convergence is scored as
corroboration that the object is real, per the house convergent-art rule, and nothing
stronger.

---

## Own-issue B — "Register `M-MAINTENANCE-LENS-HIDES-THE-REPAIRED-VARIABLE` and re-audit our own maintenance exhibits against it"

**Type:** transferable instrument lesson (inbound)

**Body draft:**

Reading Six Birds' packaging exhibit produced a defect worth registering, and the
reason to register it is that **we are exposed to it too**.

The finding, measured on their artifact: their `Def(E)` at τ=2 collapses to 0.0
identically for `p_repair` ∈ {1.0, 0.5, 0.0} — a repair that restores nothing produces
the same "objecthood" reading as a perfect one — because the packaging lens is
`(y, r, φ)` while the repair acts on `u`. The lens hides the variable the repair
repairs, so the instrument cannot see whether maintenance maintained anything. Their
`p_repair = 0` control is, in our vocabulary, the design-blind repair, and our own
maintained-holonomy campaign already measured exactly this split from the other side:
design-blind repair holds **size** exactly and forever (0.435 constant to six decimals
to R=4001) and loses **identity** completely (fidelity 0.9909 flat versus a power-law
collapse to chance). "The payment must know what it is for" is currently a *finding*
in our record; this issue proposes promoting it to a *registry rule*, because a rule is
grep-armed against future freezes and a finding is not.

Proposed entry:

> **M-MAINTENANCE-LENS** — a maintenance claim read through a lens that does not
> contain the repaired degree of freedom cannot distinguish repair from stalling: the
> discriminator is a matched no-op / design-blind repair at identical cost and identical
> policy, and a maintenance freeze must name which lens coordinate carries the repaired
> quantity. *Born in:* the Six Birds packaging exhibit, 2026-08-30, measured.

The re-audit this obliges: every claim in `STANCE.md` that rests on repair or rent —
the cross-face law, the ratchet bridge, GINI RENT, and the `rentStep`/`Ginf_at_Wstar`
family in `lean/CIRISHolon/Object.lean` — gets checked for whether its declared view
contains the quantity the payment acts on, and whether a matched design-blind control
was ever run. Where it was not, the record says so rather than assuming the answer.
Our own `OBJECT.md` rule 5 already carries the rider ("the repair must know the design
— a design-blind repair holds a structure's size while its identity decays"), so the
expectation is that most of these pass; the point of the audit is that "we already
knew" and "we already applied it" are different states, and only the second one counts.

---

## Appendix — things worth knowing about their setup, for whoever posts these

- **Their repo has no `LICENSE`.** Reuse and redistribution are legally ambiguous.
  Raised inside Issue 2 as a request, not a complaint.
- **Their tracker has zero issues and last push was 2026-02-10** (~6 months before this
  draft), while the agency preprint carries an April 2026 arXiv id. They may be slow to
  respond, or the repo may be a frozen artifact for the paper rather than a live
  project. Post #1 alone and wait.
- **Sole author, contactable**: Ioannis Tsiokos, `ioannis@automorph.io`, and the
  discussion section discloses LLM use for scaffolding with author-owned scientific
  content. If the tracker is dead, one short email is probably the right channel — but
  that is Eric's call, not ours, and nothing has been sent.
- **`results/` ships empty** (`.gitkeep`); artifacts are regenerable via
  `scripts/run_all_experiments.py --clean`, and only the summaries in
  `docs/experiments/*.md` and `paper/generated/numbers.json` are committed. Any hash
  exchange should therefore be over regenerated artifacts, and their
  `sbt_agency.repro.stable_hash` (sorted-keys canonical JSON → sha256) is a perfectly
  good shared canonicalization for us to adopt on our side of the exchange.
- **Their honesty posture is strong and should be matched, not lectured.** The papers
  state plainly that results are "witnesses, not general theorems", that empowerment is
  a difference-making proxy and not a goal theory, that packaging is lens- and
  horizon-dependent, and that median-empowerment sampling needs replacing at scale.
  Several of the issues above land inside limitations they already named. Every draft
  says so where that is true.
