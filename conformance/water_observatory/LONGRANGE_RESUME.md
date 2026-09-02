# B1 long-range audit — detached compute resume

Lane: `longrange-audit`. Worktree: a CIRISHolon worktree on branch `lane/longrange-audit`,
private `CARGO_TARGET_DIR` at `.target-lr` inside it. Session death must kill narration
only, never computation.

## What is running

```
./.target-lr/release/examples/longrange_audit --class=hydrogen --plant --stride=400
./.target-lr/release/examples/longrange_audit --class=fenced   --plant --stride=400
```

Both launched with `setsid nohup ... < /dev/null &` from the worktree root, output
redirected to `lr_hydrogen.log` / `lr_fenced.log` in this session's scratchpad.

## Cost, and why the mixed class is slow

The instrument regenerates the pair curves through the protocol's own
`generate_pair_table(a, b, 96)` — the same call, the same knot count, the same load door.
`CLASS-H` needs H–H only (seconds). `CLASS-MIX-FENCED` needs H–H, O–H and O–O, and the
committed arm log prices O–O at 2596.2 s.

## Done when

Each log's last line begins `VERDICT`. A log without one is an unfinished run, not a
verdict: re-launch rather than reading a partial sweep.

## Resume

Both runs are pure functions of committed inputs — the parked artifacts, their manifest,
and the committed curve generator — so there is no checkpoint and none is needed.
Re-launch the same command line.

## The plant demonstration (P1), run separately

A staged root holds one pristine and one one-bit-flipped copy of a parked trajectory,
plus the unmodified arm log:

```
./.target-lr/release/examples/longrange_audit --class=hydrogen --root=<scratch>/plantroot
```

The expected output is one `ADMITTED` line and one `REFUSED` line with both digests
printed — the gate discriminating, not refusing everything.
