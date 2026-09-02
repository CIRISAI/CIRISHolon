# The retirement battery — DRAFT for ruling

*Drafted 2026-09-02 by workbench-engine at the lead's ruling. §9c says "the old UI retires
when the 3D workbench is green under its full gate battery — no gap where neither serves."
That battery was a PHRASE. A phrase cannot gate a promotion: it has no owner, no command,
and no way to be false. This enumerates it.*

**Status: DRAFT. Not a gate until ruled.** Rows R1–R4 pass today; R5–R9 do not, and three
of them are not yet built. A battery whose every row already passes would be describing
the present rather than gating a change, so the failing rows are the useful half.

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
| **R3** | every figure and fence the page displays resolves to its cited artifact | inside R2 — the `RECORD` and `LADDER` citation passes | workbench-engine | **partial** | **the fences in the page's own fence register and NOT-SERVED list carry no citations at all.** RECORD figures (6) and band cites (4+1 certificate) are gated; the runtime fence list is prose. Closing this is R6 |
| **R4** | the page cannot deny a capability the engine has | inside R2 — the `PROSE_CLAIMS` parity table | workbench-engine | **yes** | the reverse: a capability the page CLAIMS and the engine lacks. The absence list covers named exports; a page asserting a physics claim with no export behind it would pass both |
| **R5** | the Bevy artifact BUILDS — link, bindgen match, bundle runs | `bash engine/crates/holon-render-3d/build-web.sh` in CI, outcome stamped to `docs/build-status.json` | workbench-engine | **no — not a gate yet** | the stamp records the outcome and the page fences on it, but nothing FAILS a promotion on it. Needs a promotion-time read of the stamp. Also: "builds" is not "runs correctly" — see R7 |
| **R6** | every fence the page renders carries an owner and an exit, and is registered | not built | workbench-engine | **no** | the band fences already carry owner+exit and are gated; the runtime fence register and NOT-SERVED list are not. Registration in `FENCES.md` is the second half — M8/M9/M11 exist, the rest do not |
| **R7** | the Bevy shell renders the SAME scene the cdylib is stepping | not built — needs Route B | workbench-engine | **no** | this is the one-Sim law made checkable. Until Route B lands there is nothing to check; after it, the check is that the drawn atom count and positions match the ABI's, which is the only thing standing between "one Sim" and "two Sims that agree for now" |
| **R8** | a seeded scene replays bit-identically on the shipped artifact | inside R2 — the FNV digest over raw f64 bits, two runs | workbench-engine | **yes, for the cdylib** | the BEVY artifact's determinism, and cross-device replay. WB-5.4 is per device class and the class is declared, not verified across machines |
| **R9** | no gap: the old UI still serves until the moment the new one is promoted | not built | lead / whoever owns the .io deploy | **no** | this is a deploy-sequencing property, not a code property, and no command I own can check it. Naming it here so it is not discovered during the promotion |

---

## What I would ask the ruling to settle

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
