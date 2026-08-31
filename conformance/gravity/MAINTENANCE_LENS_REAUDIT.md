# M-MAINTENANCE-LENS re-audit — CIRISHolon's own rent/repair/maintenance claims

*The standing obligation from the registry entry, discharged as a TABLE OF FINDINGS.
**No fixes are applied and no verdicts are drawn** — the lead rules after the table
exists. Audited by `saturation3-mesh`, 2026-08-30.*

The two questions, from the entry verbatim:

1. **does the instrument's lens contain the variable the repair/maintenance acts on?**
2. **has a known-null repair (issue-the-command-restore-nothing) been run as the control?**

The entry's own founding case is Six Birds': their `p_repair` knob at 1.0 / 0.5 / 0.0 gives an
identical `Def(E)=0`, because the lens `(y,r,φ)` discards `u` — the repaired variable. A metric
computed on a lens that hides what the repair acts on is *constructively incapable* of measuring
repair efficacy; it can only report that the command was issued.

---

## The table

| # | claim | maintained/repaired variable | what the lens actually reads | LENS CONTAINS IT | NULL CONTROL |
|---|---|---|---|---|---|
| 1 | **Cross-face rent law** — `W(v) = 1 − Σᵢ maxⱼ Pᵢⱼ` is the minimum per-step displaced mass of a repair holding view `v` closed | the microstate `s''` produced by the maintenance kernel `R(s''\|s,s')` (`CROSSFACE1_PREREG.md:98`) | the view-level joint `Pᵢⱼ` built from the **unrepaired** step map (`crossface1.py:216`), then a closed form (`:43`) | **PARTIAL** — the block *label* yes, within-block content no | **ABSENT** |
| 2 | **Ratchet bridge** — `W(v) = 2δ·W*(γ,δ)`; the maintained-holonomy split is the fiber floor | the fiber coordinate `f` (`omega_ratchet1_verify.py:142`) | the fiber marginal `Pr[f=0]` (`:152`); campaign-side, *fidelity* | **YES** | **RUN**, three ways |
| 3 | **Tuner Hold/Degrade** — one axis is HELD and never degrades; nothing degrades silently | the held axis: exactness (bool) or latency (`budget_ms`) | **only `t` vs `Degrade::Scope{max_t}`** (`tune.rs:192`) | **PARTIAL** — exactness yes but not via the selector; **latency NO** | **ABSENT for latency** |
| 4 | **holon-tables plant (iii)** — the variational guard catches a bad warm start; a warm start is an optimisation | the Davidson **start vector**, and through it the converged energy and iteration count | `Solution`: energy, `variational_margin`, `davidson_iters`, residual, exit | **YES** | **RUN (mis-targeted variant); strict no-op ABSENT** |
| 5 | **RESOURCE-1 rent-refresh** — receipts are the rent; a working holder pays by working | the lease's liveness (receipt counter → `Active`/`Idle`, and the reaper's rung 1) | the receipt counter itself (`AtomicU64` per worker), and `Ledger::rent` | **YES** | **RUN (strict no-op)** |

---

## The evidence, per row

### 1. Cross-face rent — the instrument never instantiates a repair

`crossface1.py` contains the word `repair` **exactly once, in its docstring** (`:5`), verified by
direct count. `W` is computed from the joint count matrix of the *unrepaired* step map and a
closed form; no policy object is ever built and no kernel is ever applied.

The programme has already written down why this is structural, and it is sharper than "the lens is
narrow". `OMEGA_RATCHET1_NOTE.md:262–275` (O3): `W(v)` is a **minimum over all kernels**, so it
cannot express a constraint on *which* repairs are available — and on the fiber model the argmin
is attained by the design-blind class. Their own sentence: *"The quantity CROSS-FACE-1 minimises
is minimised by the class of repair that loses the design."* So the lens cannot see restorative
content, and cannot even see the repair command, because the minimisation eliminates the policy
before the instrument runs.

The three plants (`crossface1.py:415–499`) are model-substitution plants; none issues a repair.
No `null|placebo|no-op|p_repair` anywhere in the instrument or its prereg.

**Nearest thing in the family, reported separately so it is not miscounted as CROSSFACE-1's:**
`maintenance_sweep.py:725` (ARM-MISMATCH) drifts a structure and then runs *full upkeep pointed at
the original support* — a mis-targeted repair. It **fired**: P11 FALSIFIED, mismatched upkeep held
the share fully (2.773 of 2.773) against 0.000 with no upkeep
(`MAINTENANCE_SWEEP_RESULTS.md:336`). That is the same shape as Six Birds' finding, found
independently by this programme, on a different instrument.

### 2. Ratchet bridge — the exception, and it was reached deliberately

The lens *is* the discriminating coordinate. `omega_ratchet1_verify.py:152` reads the fiber
marginal, which is exactly what the design-knowing repair moves and the design-blind one does not.

Controls, three registers:
* **design-blind arm at every `q`** (`:163`, `:166`) — the command fires at rate up to 19/20 and
  restores nothing of the fiber; alignment provably tends to `1/|Φ|`, i.e. **no improvement at any
  budget short of total replacement**;
* **C-RAND** (`holonomy_rent.py:141`, run at `:317`) — identical pipeline, matched `q`, deposits a
  fixed *random* unitary. Gain plateaus indistinguishably from the genuine arm while fidelity sits
  at chance. Their own conclusion (`HOLONOMY_RENT_RESULTS.md:250`): *"Without this control, R-POL's
  flat gain would read as success."* It also corrected the assumed floor from `1/√d` to `1/d`;
* **mechanized** — `GiniRent.lean:462` `equivariant_uniform_invariant`.

Caveat carried forward from the note itself (`:379–395`): the finite model reproduces the *shape*
of the split but not the rate law, and none of §7.4's checks is a confirmed advance prediction.

### 3. Tuner Hold/Degrade — verified at source, and the latency face fails both questions

`select_at` (`tune.rs:184–226`) reads **exactly one quantity**: `t` against
`Degrade::Scope{max_t}`. Verified directly:

* `let _ = n;` (`:191`) — width discarded;
* `decomp` is the constant `Decomp::Pruned` (`:203`);
* `degraded` is a **string literal** (`:211–214`), and the latency arm binds nothing:
  `Hold::Latency { .. }`;
* `budget_ms` appears in `engine/crates` only at its declaration (`tune.rs:40`) and one test
  construction (`:280`). It is **never read by any decision path**. (The `budget_ms` hits in
  `engine-compare` are an unrelated symbol.)
* under a latency hold `select_at` returns `Ok(...)` unconditionally — there is no cost model, no
  candidate list, no clock read, and **no path on which it can refuse**.

A model/implementation gap sits underneath this and is load-bearing: `Tune.lean:37` gives `Config`
a `cost` field and `:47` defines `holds (.latency b) c := c.cost ≤ b`. There is no `cost` field and
no `Config` list in the Rust selector, so `select_sound`/`select_complete` are proved about an
object the Rust does not implement.

The exactness face is genuinely held — but by *route agreement* (`tuned.rs:55` asserts the tuned
amplitude equals the untuned truth), i.e. because no approximate route exists to choose, not
because the selector checks anything. The `AccuracyUnderExactness` refusal is enforced at
construction and is tested (`tune.rs:233`).

Null control for latency: **absent, and would fail today** — no test declares a `Hold::Latency`
with an unmeetable budget, and `select_at` could not refuse it if one did.

### 4. holon-tables plant (iii) — lens yes, control is mis-targeted rather than null

The guard reads the energy via `Solution::variational_margin`, and the energy is precisely what a
bad start corrupts; the benefit claim reads `davidson_iters`, precisely what a warm start is meant
to improve. Both are in the lens.

The control is `wrong_start` (`generate.rs:126`): a deterministic **random** vector supplied as a
warm start. Command issued, no restorative content — and the metric reported the truth: iterations
went *up* (361 against 97) and 12 of 32 nodes landed 7.47 Ha wrong and VOIDed.

**But it is a MIS-TARGETED repair, not a strict no-op**, and the distinction is the one this
misfit is about. The strict null — warm-start from the *cold* start vector, so the command is
issued and the state is unchanged — has **not** been run. On this instrument I expect it to be
benign, but expecting is what the obligation exists to replace.

### 5. RESOURCE-1 rent-refresh — the only strict no-op in the set

The receipt counter *is* the maintained variable: rent is paid in the same quantity the reaper's
rung 1 reads, so there is no gap between what is restored and what is measured — by construction
rather than by luck.

`plants.rs:220` issues `pay_rent(id, Receipt::ZERO)` — the command, restoring nothing — and asserts
the lease moves to `Idle` and the rent total does **not** move. That is the required shape exactly:
a heartbeat with no work product is not rent.

---

## What the audit found that it was not looking for

* **The expectation in the proposal was not met.** `SIX_BIRDS_ISSUE_PROPOSALS.md:639` anticipated
  *"most of these pass; the point of the audit is that 'we already knew' and 'we already applied
  it' are different states."* Of five families, **one passes cleanly** (ratchet bridge), two are
  partial with no control, one has a mis-targeted control, and one passes.
* **The tuner's latency hold is the sharpest instance and it is not a measurement claim at all** —
  it is a contract the code cannot honour, with a Lean proof about an object the Rust does not
  implement. It is the only row where the gap is between a *declared guarantee* and code, rather
  than between a metric and a lens.
* **This programme found the Six Birds shape independently, before importing it** —
  ARM-MISMATCH falsified P11 on exactly the mis-targeted-repair question
  (`MAINTENANCE_SWEEP_RESULTS.md:336`). That is convergent-art evidence for the misfit itself.
* **A strict no-op and a mis-targeted repair are different controls**, and only rows 2 and 5 have
  the strict one. Row 4 is mine and I have marked my own as the weaker kind rather than claiming
  the stronger.

## What is NOT claimed here

No verdicts, no fixes, no severity ordering. Rows 1 and 2 are other lanes' instruments and I have
reported them from source with line references so the owners can check me rather than take my
word. Row 3's reading was verified directly against `tune.rs` rather than accepted from a survey.
