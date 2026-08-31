# Open defects — found, evidenced, and NOT yet fixed

*A defect is a place where the engine does not do what it says it does. This file exists so
such a place is **carried openly with its evidence** rather than living in a lane's message
history until someone rediscovers it. An entry leaves only by being fixed or by being ruled
not-a-defect, and either way the entry stays with the outcome recorded.*

**Not the same as `LESSONS.md`**, which carries binding rules already paid for. These are
open, and each names an owner or says that ownership is unassigned.

---

## D-1 — `Hold::Latency` declares a guarantee the selector structurally cannot honour

| | |
|---|---|
| **found** | 2026-08-30, by the M-MAINTENANCE-LENS re-audit (`conformance/gravity/MAINTENANCE_LENS_REAUDIT.md`, row 3) |
| **verified** | at source, independently of the survey that surfaced it |
| **owner** | **UNASSIGNED — engine-core.** Routed to the lead for assignment |
| **status** | OPEN. Not fixed, deliberately: the fix is a ruling, not an edit (see below) |

### What is claimed

`tune.rs:1–32` and `lean/CIRISHolon/Tune.lean:10–18` state the contract: one axis is HELD and
never degrades, the rest degrade in declared order, refusal is the total fallback, and
**nothing degrades silently**. `Tune.lean:96` names the graphics face explicitly — *under a
latency hold, anything returned fits the budget*.

### What the code does

`select_at` (`engine/crates/holon/src/tune.rs:184–226`) reads **exactly one quantity**: the
T-count `t` against `Degrade::Scope { max_t }` (`:192–202`). Everything else is discarded or
hardcoded:

* `:191` — `let _ = n;` the width is thrown away;
* `:203–210` — `decomp` is the constant `Decomp::Pruned`;
* `:211–214` — `degraded` is a **string literal**, and the latency arm binds nothing:
  `Hold::Latency { .. } => vec![]`;
* `budget_ms` occurs in `engine/crates` at its declaration (`tune.rs:40`) and one test
  construction (`:280`) and **nowhere else**. No decision path reads it. (The `budget_ms` in
  `engine-compare` is an unrelated symbol.)

**Under a latency hold `select_at` returns `Ok(...)` unconditionally.** There is no cost
model, no candidate list, no clock read, and no branch on which it can refuse.

### The specification-boundary gap, which is the serious half

`Tune.lean:35–37` gives `Config` a `cost : ℕ` field; `:47–49` defines
`holds (.latency b) c := c.cost ≤ b`. The Rust selector has **no `cost` field and no `Config`
list** — `grep -n cost engine/crates/holon/src/tune.rs` returns nothing.

So `select_sound` (`:63`), `select_complete` (`:76`) and `exact_never_degraded` (`:92`) are
proved about an object the Rust does not implement. The proofs are true; they are true of a
model, and the model is not the code. That is the self-consistent-lie shape at the
specification boundary: every artifact is internally sound and the composition asserts
something no component checks.

### Why this is a DEFECT and not a lens gap

The re-audit's other rows are metrics that cannot see what a repair acts on. This one is
different in kind and the lead's verdict says so: it is not a measurement that reads the wrong
variable, it is a **declared guarantee the code cannot honour**. A caller that constructs
`Hold::Latency { budget_ms: 16.6 }` is told an axis is held; nothing in the engine will ever
tell that caller otherwise, including when the budget is missed by any margin.

### What is NOT wrong

The **exactness** face is genuinely held — but by *route agreement*, not by the selector:
`holon/tests/tuned.rs:55–61` asserts the tuned amplitude equals the untuned truth. It is held
because no approximate route exists to choose, not because `select_at` checks anything. The
`AccuracyUnderExactness` refusal is enforced at construction and is tested (`tune.rs:233–240`).

### Missing control

No test declares a `Hold::Latency` with a budget the work cannot meet and confirms a refusal
or a reported degradation. Such a test, written today, **would fail** — `select_at` cannot
refuse on latency. The only refusal test is Scope (`tuned.rs:90–98`).

### The fix is a ruling, not an edit

Two routes, and they are not equivalent:

1. **implement `cost` in the selector** — give it a cost model and a candidate list so a
   latency hold can actually refuse. This makes the Lean's object real and is the larger
   change;
2. **narrow the Lean claim** — drop or qualify the latency face so the mechanized statement
   matches what the code does, and say plainly that `Hold::Latency` is a declaration of intent
   rather than an enforced hold.

Picking (2) quietly would be the worse outcome dressed as the smaller one. The choice is owed
after a reading of `Tune.lean`; this lane has not made it and has changed nothing.
