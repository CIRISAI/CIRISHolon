# The retirement battery — RULED 2026-09-02

*Drafted 2026-09-02 by workbench-engine at the lead's ruling. §9c says "the old UI retires
when the 3D workbench is green under its full gate battery — no gap where neither serves."
That battery was a PHRASE. A phrase cannot gate a promotion: it has no owner, no command,
and no way to be false. This enumerates it.*

**Status: RULED and in force.** The lead's rulings are folded in below. R3 has since gone
FULL (1a1b302) and now passes. What remains unbuilt is R5's manual receipt, R6's
registration sweep, R7 (a Route B deliverable and a hard blocker), and R9 (the lead's).

A battery whose every row already passes would be describing the present rather than
gating a change, so the failing rows are the useful half.

---

## How to read a row

Every row names its **command** and its **owner**, and every row states what the command
would **NOT** catch. That last column is the one that earns the battery its name. This
lane has now shipped five checks that passed while establishing nothing — an empty-string
match, an existence test standing in for a tracking test, a threshold at zero, a readout
that could not change, and a field-order regex that broke when its subject grew. Each was
green. The one-directional column is where the next one gets caught.

---

## The battery

| # | gate | command | owner | PASSES TODAY | what it would NOT catch |
|---|---|---|---|---|---|
| **R1** | the 3D shell compiles in the configuration the site ships | `cargo check --manifest-path crates/holon-render-3d/Cargo.toml --target wasm32-unknown-unknown --features render` (ci-gates 15b) | workbench-engine | **yes** | that it LINKS, that wasm-bindgen matches the lockfile, or that the bundle RUNS. It is a type check. It exists because gate 15's `--features headless` is structurally blind to `hud.rs`/`pick.rs`/`render.rs`, which hid a two-day compile break |
| **R2** | the shipped workbench artifact runs, and the page's claims hold against it | `node docs/workbench/smoke.mjs` (ci-gates 17b) — 91 checks, ~30 s | workbench-engine | **yes** | anything about the BEVY artifact, which it never loads; and anything about how the page LOOKS. It drives the cdylib through the raw ABI and reads the page's source, not its pixels |
| **R3** | every figure and fence the page displays resolves to its cited artifact | inside R2 — the `RECORD`, `LADDER`, `FENCE_REGISTER` and `NOT_SERVED` passes | workbench-engine | **yes — FULL since 1a1b302** | that a cited row says what the page says ABOUT it. The gate checks a row EXISTS and that a figure is on its line; it does not read the row's prose back. A register row that is itself wrong would pass |
| **R4** | the page cannot deny a capability the engine has | inside R2 — the `PROSE_CLAIMS` parity table | workbench-engine | **yes** | the reverse: a capability the page CLAIMS and the engine lacks. The absence list covers named exports; a page asserting a physics claim with no export behind it would pass both |
| **R5a** | AUTOMATED: the Bevy artifact builds, links, and wasm-bindgen matches the lockfile | `bash engine/crates/holon-render-3d/build-web.sh` in CI; outcome stamped to `docs/build-status.json` and read at promotion time | workbench-engine | **no — stamp exists, promotion-time read does not** | that the bundle RUNS. Compiles-and-links genuinely does not say runs; that is R5b, and splitting them is the lead's ruling |
| **R5b** | MANUAL: the bundle actually runs, with a receipt | a named command run in a real browser by a human before the promotion commit; outcome recorded IN the promotion commit message — page loaded, sim stepped, N frames, backend used | workbench-engine (runs it) / lead (accepts the receipt) | **no** | anything after the moment it was run. A receipt is a measurement at a time, not a standing property — which is why R5a exists beside it rather than instead of it. Headless-browser CI is a separate node nobody owns yet; it does not block this promotion |
| **R6** | every fence the page renders carries an owner and an exit, and cites a FENCES.md row that exists | inside R2 since 1a1b302 | workbench-engine (page) / bank-fences (register) | **mostly** | that the register row is CORRECT, and that no fence exists which the page fails to display. Most rows were already filed by bank-fences (P2, P13, P14, P16, C3); what remains is a sweep for page-local fences that still have no row of their own |
| **R7** | the Bevy shell renders the SAME scene the cdylib is stepping | not built — needs Route B | workbench-engine | **no** | this is the one-Sim law made checkable. Until Route B lands there is nothing to check; after it, the check is that the drawn atom count and positions match the ABI's, which is the only thing standing between "one Sim" and "two Sims that agree for now" |
| **R8** | a seeded scene replays bit-identically on the shipped artifact | inside R2 — the FNV digest over raw f64 bits, two runs | workbench-engine | **yes, for the cdylib** | the BEVY artifact's determinism, and cross-device replay. WB-5.4 is per device class and the class is declared, not verified across machines |
| **R9** | no gap: the old UI still serves until the moment the new one is promoted | **the lead's, executed and recorded by them.** RULED: the promotion is ONE revertable commit (the pages.yml root switch); the old UI is RETAINED in the tree at its current path; retiring those files is a SEPARATE later commit after the workbench root has served green; rollback is a one-command revert | lead | **owned, not yet executed** | a Pages build that fails after the switch — which the revert covers, and which is why the retirement is a separate commit rather than part of the switch |

---

## The rulings, as given

1. **R5 splits in two.** Automated: builds + links + bindgen matches. Manual: a
   human-run browser receipt recorded in the promotion commit message. Headless-browser
   CI is a later node and does not block; the manual receipt does.
2. **R6: the fence law stands** — FENCES.md is the single register, every displayed fence
   gets a row, the page carries owner+exit and cites the row.
3. **R7 is a HARD promotion blocker**, recommendation adopted. It is a Route B
   deliverable, so the ordering costs nothing.
4. **R9 is the lead's**, with its content ruled as above.
5. **R3 goes FULL before promotion** — done at 1a1b302.

## Superseded — what I had asked the ruling to settle

1. **R5's threshold.** "The artifact builds" is checkable. "The bundle runs" needs a
   headless browser in CI, which this repo does not currently have. I can gate the build
   and stamp the run, or add a browser step. The second is more honest and more expensive.
2. **R6's scope.** Registering every runtime fence in `FENCES.md` is bank-fences' register
   and their law. I can carry owner+exit ON the page and cite the register row; whether
   every page fence must have a register row is theirs to say, not mine.
3. **R7 is the real gate** and it does not exist yet because Route B does not. I would not
   promote without it: "one Sim" is currently an architectural intention, and the failure
   mode it guards against — a drawn scene and an instrumented scene that agree until they
   do not — is invisible to every other row here.
4. **R9 has no owner in my lane.** Someone has to own the sequencing.

## What this battery deliberately does NOT gate

The three coarse bands being FENCED. That is the launch spec, not a gap: §9c's ladder ships
with the molecular band live and the rest wearing their fences, and the rungs
(rung1-network, rung2-continuum) produce the certificates that flip them later. The
citation-gated flip is already wired and planted — a band cannot go live without a
resolving certificate, and a resolving certificate cannot sit in a fenced band.

## A citation the addendum got wrong, corrected here

The continuum-face lineage is real and I verified it: **`engine/MESH_DESIGN.md` §2.1**,
prior-art block at lines 101 (Frisch, Hasslacher & Pomeau — FHP-6, the chart the engine
already wears) and 105 (d'Humières, Lallemand & Frisch — FCHC-24, the chart the design
adopts). The addendum cited `sim_engine/MESH_DESIGN.md`, which does not exist in this
repository — `sim_engine/` is CIRISOntology's tree. The content is exactly as described;
only the path differs. Recorded because the cube's fence text will cite this, and my own
citation gate would have rejected the path as given.
