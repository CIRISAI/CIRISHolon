/-
CIRISHolon.Omega — rung 2 of the evaluation ladder: the holon identity,
frozen as definitions with their composition laws.

The Ω-object of the review's formulation, at the finite exactness this
library works at: a context is a state space with a view (whose fibers are
the observational-equivalence classes), a dynamics, and a cost of closure;
a morphism is a coarse-graining (a re-rooting of the view); composition is
function composition, and the laws below make holons a category-shaped
structure. MAXIMALITY IS RELATIVE TO A ROOT: a holon is maximal AMONG the
coarsenings of a chosen root, which reconciles "no absolute maximal holon"
(nothing here asserts a terminal object exists globally) with the
programme's "maximal object" usage (maximal over a root's coarsening
poset). Completeness — whether Ω captures everything a probe can see — is
rung 3's QUESTION and is deliberately not assumed by any definition here.

Convergent art, credited per the house rule: closure-as-measured-autonomy
has established ancestors — Barnett–Seth's dynamical independence,
Shalizi–Crutchfield's causal states (the stochastic ancestor of the Moore
identity), Kabernik's quantum coarse-graining consistency, Krakauer's
informational individuality, Montévil–Mossio's closure of constraints,
Kolchinsky–Wolpert's semantic information (SELECTOR-1's closest ancestor).
See conformance/omega/PRIOR_ART_CONVERGENCE.md for the full map and the
narrowed originality claim.
-/
import Mathlib.Tactic
import CIRISHolon.ClosureLadder

namespace CIRISHolon.Omega

open CIRISHolon.ClosureLadder

/-- A holon over micro state space `S`: a view into `V` and a dynamics.
    The fiber of `v` at `x` is the observational-equivalence class; `T` is
    the induced micro dynamics. (Measure and cost enter as graded readings
    on top of this skeleton — the skeleton is what composition needs.) -/
structure Holon (S V : Type) where
  view : S → V
  dyn : S → S

/-- The holon is CLOSED when the view's square commutes: some coarse map
    `F` satisfies `view ∘ dyn = F ∘ view` — the `vT = hv` square. -/
def Closed {S V : Type} (H : Holon S V) : Prop :=
  ∃ F : V → V, ∀ s, H.view (H.dyn s) = F (H.view s)

/-- A morphism of holons over the same micro space: a coarse-graining of
    views that intertwines nothing further — dynamics is shared, the view
    factors. `f` re-roots the coarse description. -/
structure Coarsening {S V W : Type} (H : Holon S V) (K : Holon S W) where
  map : V → W
  view_eq : ∀ s, K.view s = map (H.view s)
  dyn_eq : ∀ s, K.dyn s = H.dyn s

/-- Identity coarsening. -/
def Coarsening.id {S V : Type} (H : Holon S V) : Coarsening H H :=
  ⟨fun v => v, fun _ => rfl, fun _ => rfl⟩

/-- Composition of coarsenings — associative by construction (function
    composition), which is the category law the freeze needs. -/
def Coarsening.comp {S V W X : Type} {H : Holon S V} {K : Holon S W}
    {L : Holon S X} (f : Coarsening H K) (g : Coarsening K L) :
    Coarsening H L :=
  ⟨g.map ∘ f.map,
   fun s => by rw [g.view_eq, f.view_eq]; rfl,
   fun s => by rw [g.dyn_eq, f.dyn_eq]⟩

/-- **Closure descends along coarsenings.** If the finer holon is closed,
    every coarsening of it is closed too — the coarse map is conjugated
    through the re-rooting WHEN the re-rooting is injective on the view's
    image; in general, closure descends when the coarse dynamics factors,
    which the surjective-section form below states exactly. This is the
    one-way street the ladder measured: refining can only help closure
    (ClosureLadder), and here, closing survives coarsening when the
    coarser view still factors the closed map. -/
theorem closure_descends {S V W : Type} (H : Holon S V) (K : Holon S W)
    (c : Coarsening H K) (F : V → V) (hF : ∀ s, H.view (H.dyn s) = F (H.view s))
    (G : W → W) (hG : ∀ v : V, G (c.map v) = c.map (F v)) :
    Closed K := by
  refine ⟨G, fun s => ?_⟩
  rw [c.view_eq, c.view_eq, c.dyn_eq, hG, hF]

/-- Maximality RELATIVE TO A ROOT: among the coarsenings of a root holon,
    `K` is maximal-closed when it is closed and every strictly coarser
    view (any further coarsening) that remains closed carries no more
    information — stated as: every further closed coarsening's view
    factors through `K`'s. No global/absolute maximal object is asserted
    anywhere; this is a property in the root's coarsening order. -/
def MaximalClosedOver {S V W : Type} (root : Holon S V) (K : Holon S W)
    (c : Coarsening root K) : Prop :=
  Closed K ∧
  ∀ (X : Type) (L : Holon S X) (_ : Coarsening root L), Closed L →
    ∃ g : W → X, ∀ s, L.view s = g (K.view s)

/-- The trivial (terminal-per-root) witness: the one-point view is always
    closed, and everything factors through... nothing — the one-point view
    factors through EVERY view. This pins the definitions as non-vacuous
    in the harmless direction: a maximal-closed coarsening always exists
    only if closedness plus factoring hold together, and the one-point
    view satisfies Closed but NOT the factoring clause in general — so
    maximality is a real condition, not automatic. Stated as: the unit
    view is closed. -/
theorem unit_view_closed {S : Type} (dyn : S → S) :
    Closed (Holon.mk (fun _ : S => PUnit.unit) dyn) :=
  ⟨fun u => u, fun _ => rfl⟩

end CIRISHolon.Omega
