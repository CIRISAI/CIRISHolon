/-
lean/Audit/AxiomAudit.lean — the load-bearing verification gate for this lake.

Run as `lake env lean Audit/AxiomAudit.lean` from `lean/`. This file's
ELABORATION is the gate: anything that fails below is a build error, not a
warning. Ported from CIRISOntology/Audit/AxiomAudit.lean, with one change made
on purpose and stated here.

WHY THIS FILE RATHER THAN A GREP. Three weaker gates are commonly used and each
is defeatable: a textual search for the admitted-gap keyword flags prose that
merely MENTIONS it and misses gaps introduced by a tactic rather than by the
literal keyword; `lake build --wfail` relies on Lean's own warning, which
`#guard_msgs` swallows; and neither sees a gap inherited transitively from an
imported declaration. Asking the proof assistant what a theorem actually depends
on is the check that cannot be talked around — check the artifact, not the text
describing the artifact.

THE CHANGE FROM THE PREDECESSOR. CIRISOntology pins theorems BY NAME, one
`assert_no_sorry` line each. A name list has a defect this lake would feel
immediately: a new theorem is clean until someone forgets to add its line, and
nothing says so. `assert_namespace_clean` instead SWEEPS — every declaration in
the `CIRISHolon` namespace, whether or not anyone remembered it — and prints the
count it checked, so the coverage is visible rather than assumed. The named pins
below are kept anyway, for the load-bearing claims: they are what a reader greps
for, and they fail with a specific name instead of a count.
-/
import CIRISHolon

open Lean Elab Command

/-- Fail if `n` transitively depends on the admitted-gap axiom. Mathlib ships
    the same eight lines as `assert_no_sorry`; inlined here so the gate has no
    dependency that could be relaxed elsewhere. -/
elab "assert_no_sorry " n:ident : command => do
  let name ← liftCoreM <| realizeGlobalConstNoOverloadWithInfo n
  let axs ← liftCoreM <| collectAxioms name
  if axs.contains ``sorryAx then
    throwError "AUDIT FAILURE: {n} transitively depends on sorryAx"

/-- Fail if `n` depends on any axiom outside the standard three. Catches
    `Lean.ofReduceBool` / `native_decide` creep, which a sorry-only check
    cannot see. -/
elab "assert_standard_axioms " n:ident : command => do
  let name ← liftCoreM <| realizeGlobalConstNoOverloadWithInfo n
  let axs ← liftCoreM <| collectAxioms name
  for a in axs do
    unless a == ``propext || a == ``Classical.choice || a == ``Quot.sound do
      throwError "AUDIT FAILURE: {n} depends on non-standard axiom {a}"

/-- Sweep an entire namespace: no declaration under `root` may carry an admitted
    gap or a non-standard axiom. Reports the number of declarations actually
    checked, because a gate whose coverage is invisible drifts behind the thing
    it gates. -/
elab "assert_namespace_clean " root:ident : command => do
  let env ← getEnv
  let rootName := root.getId
  let mut checked := 0
  for (n, _) in env.constants.toList do
    unless rootName.isPrefixOf n do continue
    -- Internal names (`_cstage1`/`_cstage2` compiler stages, `_elambda_*`
    -- lambda-liftings, `_proof_*` extracted subproofs) are skipped. This is a
    -- SCOPE statement, not a relaxation: the compiler's output has no logical
    -- content, and an extracted subproof carrying a gap puts `sorryAx` into its
    -- PARENT's closure, which this sweep checks under the parent's own name.
    if n.isInternal then continue
    let axs ← liftCoreM <| collectAxioms n
    if axs.contains ``sorryAx then
      throwError "AUDIT FAILURE: {n} transitively depends on sorryAx"
    for a in axs do
      unless a == ``propext || a == ``Classical.choice || a == ``Quot.sound do
        throwError "AUDIT FAILURE: {n} depends on non-standard axiom {a}"
    checked := checked + 1
  logInfo m!"AUDIT: {checked} declarations under {rootName} are sorry-free and \
use only propext / Classical.choice / Quot.sound"

section Gate

-- (0) THE SWEEP. Everything in the lake, named or not.
assert_namespace_clean CIRISHolon

-- (1) The object contract — the vertical axis (`Object.lean`).
assert_no_sorry CIRISHolon.Object.closed_iff_fiber_invariant
assert_no_sorry CIRISHolon.Object.nonfactoring_iff_not_closed
assert_no_sorry CIRISHolon.Object.rent_closed_form
assert_no_sorry CIRISHolon.Object.Ginf_at_Wstar
assert_no_sorry CIRISHolon.Object.diag_view_closed_of_classical
assert_no_sorry CIRISHolon.Object.diag_not_closed_under_coherence
assert_standard_axioms CIRISHolon.Object.diag_not_closed_under_coherence

-- (2) The collision theorem — no memoryless map, not merely a worse one.
assert_no_sorry CIRISHolon.Closure.collision_refutes_memoryless
assert_no_sorry CIRISHolon.Closure.minimax_error_at_least_half

-- (3) The carrier tower — the horizontal axis (`Carrier.lean`, WB-8.1).
--     The fiber, the certificate, the composition, and the price.
assert_no_sorry CIRISHolon.Carrier.eval_total
assert_no_sorry CIRISHolon.Transport.picture_total
assert_no_sorry CIRISHolon.Transport.refl_comp
assert_no_sorry CIRISHolon.Transport.comp_refl
assert_no_sorry CIRISHolon.Transport.comp_assoc
assert_no_sorry CIRISHolon.Transport.closed_transports
assert_no_sorry CIRISHolon.Tower.climb_square
assert_no_sorry CIRISHolon.Tower.climb_total
assert_no_sorry CIRISHolon.select_admissible
assert_no_sorry CIRISHolon.select_min
assert_no_sorry CIRISHolon.select_eq_none_iff
assert_no_sorry CIRISHolon.cheap_but_over_budget_not_selected
assert_standard_axioms CIRISHolon.Tower.climb_total
assert_standard_axioms CIRISHolon.Transport.comp_assoc

end Gate

/-! ### The gate, demonstrated FIRING

A gate that has never failed has never gated — the same standard `tower.rs`'s
`compile_fail` doctest and `Carrier.lean`'s `#guard_msgs` fiber refusal are held
to. Below, a declaration carrying an admitted gap is planted UNDER the swept
namespace and the gate is asked about it; the refusal is captured, so this file
fails if the gate ever stops refusing.

Placement is load-bearing: the plant is declared AFTER `assert_namespace_clean`
above, so the sweep never sees it. Nothing here is imported by `CIRISHolon.lean`
— `lake build` does not compile this file, only `lake env lean` does. -/

namespace CIRISHolon.AuditSelfTest

/-- warning: declaration uses 'sorry' -/
#guard_msgs in
theorem planted_gap : False := by sorry

end CIRISHolon.AuditSelfTest

/-- error: AUDIT FAILURE: CIRISHolon.AuditSelfTest.planted_gap transitively depends on sorryAx -/
#guard_msgs in
assert_no_sorry CIRISHolon.AuditSelfTest.planted_gap

/-- error: AUDIT FAILURE: CIRISHolon.AuditSelfTest.planted_gap depends on non-standard axiom sorryAx -/
#guard_msgs in
assert_standard_axioms CIRISHolon.AuditSelfTest.planted_gap
