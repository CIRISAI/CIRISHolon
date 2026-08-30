# CIRISHolon

**A decentralized physics and graphics unified engine, designed for
scalability and performance — from WASM in a browser up through large
clusters — in which every level of detail is a certified view of the level
below.**

One recursive object — the holon — carries every tier: bit-exact classical
and quantum simulation at the bottom (measured against the field's own
tools; see `conformance/BENCHMARKS.md`), matter and graphics tiers above,
one merge law and one transport square binding them, and the same
deterministic, auditable results at every scale from a browser tab to a
thousand-shard mesh. The tier ladder, its referees, and its benchmark
targets live in `TIERS.md`.

## What that means, concretely

The engine's unit is the **holon**: a component that is simultaneously a
whole (it runs its own dynamics) and a part (it is a coarse reading of finer
holons). A **tier** — grain, sandbox, landscape, up the ladder — is not a
separately-authored physics module. It is a *view* `v` of the tier below,
and it is admitted into the engine only when its square commutes: some
update `h` on the coarse readings reproduces what the fine dynamics `T`
does, `v ∘ T = h ∘ v`, within a stated budget. That property is called
**Closed**, it is defined in `lean/CIRISHolon/Object.lean`, and it is the
acceptance criterion for every tier boundary, level-of-detail scheme, and
chart in this engine.

The payoff is engineering, not philosophy:

- **Scaling is correct by construction.** A tier that passes its closure
  gate may be run *in place of* the tier below it. The budget for the
  approximation (`K ≤ 1` non-expansiveness) is tested, not assumed.
- **Failures are certificates, not mysteries.** When a coarse view is wrong,
  it is wrong in exactly one way: two fine states the view cannot tell apart
  that evolve to readings it can. The conformance harness hunts those
  witness pairs directly (`nonfactoring_iff_not_closed`).
- **Charts carry their condition numbers.** A near-cancelling aggregate
  (e.g. net momentum of a settled region) amplifies noise by the inverse of
  its coherence — exactly, provably. Such quantities are never exposed as
  engine state without their conditioning declared.
- **Maintenance is priced, never tuned.** Refresh/decay budgets follow a
  closed-form law (`Ginf`, `Wstar`): choose a retention target, the dose is
  computed.
- **The quantum boundary is honest.** The classical engine is the diagonal
  retract of a quantum carrier; classical circuits run at mesh cost,
  stabilizer circuits in a closed tableau view, and past the point where
  coherence makes closure impossible (a theorem, not a policy) the engine
  spends known-exponential resources, delegates to hardware, or **refuses
  by name**. The QASM conformance target exercises exactly this boundary.

## Repository layout

- `lean/` — the object: the engine's contract as machine-checked theorems,
  verified under Lean 4 / Mathlib (toolchain and revision pinned).
  `Object.lean` (the question, conditioning, rent, the diagonal and its
  wall), `Tiers.lean` (stacking algebra), `Tier.lean` (the certified
  boundary as a bundle), `Transport.lean` (re-rooting and the certificate
  fence), `Mixing.lean` (why stochastic tiers self-heal and deterministic
  ones never do), `Budget.lean` (error composition; the K ≤ 1 linear
  budget), `Stabilizer.lean` (the Clifford stratum's closure kernel and the
  magic wall).
- `OBJECT.md` — the contract in prose: definitions, design rules, and the
  conformance obligations every tier owes.
- `LESSONS.md` — binding design rules distilled from the predecessor
  sandbox's measured failures. Read before writing any tier.
- `conformance/` — the acceptance battery, transplanted with its bands.
- `engine/` — the predecessor crates, grandfathered (see `engine/ENGINE.md`):
  working and gated by their own CI script, not yet certified tier by tier;
  any crate touched enters the battery at that touch.

## License and ethical use

CIRISHolon is licensed under the **GNU AGPL-3.0** (see `LICENSE`). This
project is intended for **ethical use cases only**, as understood by the
[CIRIS Covenant](https://ciris.ai). We state precisely how that intention
binds, because pretending a copyright license can carry it would be false:

- **The code license is pure AGPL-3.0, with no added restrictions.** The AGPL
  grants its freedoms without discrimination by field of use, and its terms
  (§7, §10) do not permit bolting an ethics clause onto it; a license modified
  to do so would no longer be the AGPL and would be incompatible with the
  ecosystem this project belongs to. We do not pretend otherwise. The AGPL's
  network copyleft is itself the structural safeguard we rely on most: anyone
  operating a modified CIRISHolon as a service must offer its source to that
  service's users.
- **The name is where the ethics gate is enforceable.** "CIRISHolon" and the
  CIRIS marks may be used only by deployments consistent with the CIRIS
  Covenant. Forks are welcome under the AGPL; calling an unethical deployment
  by this project's name is not.
- **The community is governed by the Covenant.** Contribution, maintainer
  support, partnership, and any commercial licensing beyond the AGPL are
  extended only to Covenant-aligned uses, at the maintainers' discretion.

If your use case needs a grant the AGPL does not give, the ethical-use
review IS the licensing conversation: contact the maintainers.

## Relationship to CIRISOntology

The theorems in `lean/` are transplanted verbatim from
[CIRISAI/CIRISOntology](https://github.com/CIRISAI/CIRISOntology), where
they were developed, machine-checked, and pressed against measurement
campaigns (a quantum device, a thermodynamic memory dataset, and the
predecessor engine). That repository is the research programme; this one is
the engine. They are deliberately decoupled: the only dependency is the
pinned conformance contract, and misfits found here are reported back there.

## Convergent work

An independent 2026 programme, **"Six Birds"** (Ioannis Tsiokos — arXiv
[2602.00134](https://arxiv.org/abs/2602.00134),
[2604.03239](https://arxiv.org/abs/2604.03239), code at
[ioannist/six-birds-agent](https://github.com/ioannist/six-birds-agent)),
arrived at much of the same object from its own direction: objects as
induced closures under a coarse-graining lens, a computable idempotence
defect where we compute a closure defect, ledger-gated maintenance where we
charge rent, and a measured "enabling repair collapses the defect" where we
prove that maintenance creates what it maintains. No citation links the two
programmes in either direction; we treat the convergence as corroboration
that the underlying object is real, not as a priority contest. The verified
two-way concept map and the cross-validation protocol live in
`conformance/omega/PRIOR_ART_CONVERGENCE.md`.
