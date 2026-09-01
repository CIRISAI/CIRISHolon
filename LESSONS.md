# Lessons — binding rules from the predecessor sandbox's measured failures

*The predecessor engine (CIRISOntology/sim_engine) accumulated classical-
simulation idioms — global index arrays, re-certification passes, ad-hoc
energy bookkeeping, convergence heuristics — and every one of them
eventually cost a measurement campaign. Each rule below was paid for. The
general form of the mistake was always the same: importing a conventional
simulation technique instead of deriving the mechanism from the object.
These are binding on all CIRISHolon code.*

1. **Never compare by array index; identity is the arena.** A
   re-certification pass that renumbered nodes silently poisoned three
   campaigns (per-node comparisons read ~116 where the physical answer was
   0.018). The arena was append-only all along — the stable key existed and
   the instruments ignored it. Rule: joins by holon id, always; any pass
   that reorders storage must be invisible to every observable.
2. **State-content rewrites must be observable-invisible or declared.**
   Re-certification also *changed state content* at ULP–1e-7 scale
   ("materialization dust"), which fired thresholds meant for physics.
   Rule: onsets and bands are threshold-relative (1% of max), and any pass
   that rewrites state declares its dust scale.
3. **A band names its frame and its units.** One campaign arm died because
   "frame 0" meant pre-step in the band and post-step in the CSV; another
   because a dimensionless series was compared against a raw-unit series.
   Rule: every band carries the units of both sides and which side of the
   first step its zero lives on; gauges must plant through the real
   pipeline's units, never synthetic same-scale series.
4. **Saturating series defeat plateau-including statistics.** A growth-ratio
   gate read 1.0 on a planted expander because the plateau dominated the
   window. Rule: growth statistics run on the rise epoch (onset to 90% of
   max) and VOID when the rise is too short to pose.
5. **One gate per conservation law; comments don't gate scale.** An energy
   gate stayed green while impulse was off 5.3×; five write-scale decisions
   became fatal quadratics at 1000×, three with justifying comments. Rule:
   every conserved quantity gets its own gate with planted mutations; every
   all-residents loop is re-audited on any 10× scale change.
6. **Convergence heuristics lie.** "Stagnation means converged" and absolute
   SVD tolerances hid non-canonical states and cost a verdict. A
   perturbative self-audit reported health exactly where the mean field lied
   most. Rule: convergence and health checks pin to theorem-backed
   observables, not to the solver's own residuals.
7. **Chart-relative conservation.** Balance gates refused wrongly (or
   passed wrongly) until conservation was stated per chart: injectivity is
   information, not energy, and a balance gate must refuse where the chart
   has no time-translation symmetry.
8. **Round-trip tests need a reversible map.** Forward-back error measures
   integrator asymmetry unless the map is symplectic; the confound mimics
   the effect under test.
9. **Planted defects must be observable.** Three of seven planted mutations
   stayed silent for numerical reasons. Rule: when a mutation doesn't fire,
   suspect the mutation before trusting the gate.
10. **Do not bolt on "quantum modes."** The quantum relation is the diagonal
    retract (see OBJECT.md rule 8). Any feature that simulates quantum
    behaviour outside that structure — a special-cased amplitude array, a
    per-feature interpretation — is refused at review. The wall is a
    theorem; respect it in both directions.
11. **A pathspec fences across files, never within one.** `git commit --
    <file>` takes the whole worktree file, so on a shared tree it banks
    whatever a sibling lane left mid-edit in it — under your name, in your
    commit message, undescribed. Measured 2026-09-01: four within-file
    sweeps in one day, one of them ~30 lines of another lane's API-surface
    classifications, each a documented judgement the committer had not made
    and could not vouch for. The tell is that nothing warns you: the commit
    is green, the tests pass, and the record is wrong about who decided
    what. Rule, for every shared-write file (`ci-gates.sh`, `MISFITS.md`,
    `LESSONS.md`, `BENCHMARKS`, any register or RESULTS doc): run `git diff
    -- <file>` immediately before committing and account for EVERY hunk as
    yours; a foreign hunk means leave the file out and hand it to its owner.
    Run `git diff --cached --stat` too — the index is shared and may hold a
    sibling's staged file. If you swept someone, never amend the landed
    commit; describe the taken hunk in the next commit message and tell the
    owner. This tree goes foreign in minutes, so "I checked when I started"
    is not a check.

12. **A gate that compares one region does not cover the file.** The
    trajectory runner and its reference were byte-compared on their frozen
    protocol block; the block matched, and the runner still silently ran
    different physics, because `de4_enabled = true` is set in the reference's
    `main` BELOW that block's closing banner. Regenerating "the dE4 seed"
    with it would have switched the four-body term OFF, written a trajectory
    of different physics under the right filename, and let a closure census
    report a confident failure to certify. Every number would have looked
    reasonable. When two files must agree about PHYSICS, gate the physics —
    inventory what each one sets and require the stand-in to name every knob
    its reference names — not just the region you happened to draw a box
    around. What a runner does not set, it inherits from the constructor's
    defaults, and the constructor is not the protocol.

13. **Symbol ABSENCE is not evidence; only presence is.** Declaring a build
    by its symbol table (`nm -C | grep -c <fn>`) is sound when the symbol is
    THERE — it proves the code is reached. It proves nothing when the symbol
    is missing: a build that calls `quaternary::de4_ohhh_fci` shows ZERO such
    symbols because the call is inlined. Two builds were correctly separated
    this way when one's source could not call the function at all, and the
    method was adopted house-wide on that success — which is exactly how a
    one-directional check becomes a rule that misleads. Where the question is
    "did this term actually fire", use a counter the physics itself
    increments and read it from the run.

14. **A done-marker records an exit code, and 128+N is a signal, not a result.**
    Two detached arms wrote their markers and stopped; the markers said
    `143`, which is SIGTERM — they had been killed about a minute in, having
    generated one pair curve each. Had the check been "does the marker
    exist", both would have been read as finished runs with empty output
    directories, and the census would have reported on nothing. The detached
    pattern also needs its detach VERIFIED rather than assumed: plain
    `setsid cmd &` inside a tool call that then went on to do other work did
    not survive, while `setsid nohup cmd & disown` in a call that exits
    immediately did. Launch, then confirm the process is alive in its own
    session before trusting that it will be there later.

15. **A detached pipeline's tail runs on a corpse.** The lesson underneath
    (14): SIGTERM killed the work and the cleanup marched on anyway, moving
    an empty directory to the durable path, appending a hash of a truncated
    log to a manifest that already had a line for that name, and writing
    `usage: census <dir-or-file>` under a filename that promises a verdict.
    Three artifacts, each plausible, each about nothing. **The tail checks
    the exit code FIRST**: nonzero writes a KILLED marker naming the signal
    and touches nothing else — never parks, never hashes, never adjudicates.
    A zero exit with an empty output directory gets the same treatment, since
    a success nobody can check is not one.
    `conformance/water_observatory/detached_run.sh` is the pattern, with its
    three plants (killed, succeeded, empty-success) checked in all three
    directions.
