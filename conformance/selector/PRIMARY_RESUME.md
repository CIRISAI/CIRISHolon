# SELECTOR-6 primary — resume note

Detached run of `primary.py` over the 319-group census. Session death kills
narration only.

- launch: `cd conformance/selector && setsid nohup nice -n 19 python3 primary.py > primary.log 2>&1 < /dev/null &`
- done marker: `primary.DONE` (contains the E1 branch letter); output
  `primary.json` (per-group record), `primary.log`
- prerequisites already green: census S1 exact (319/319, zero VOIDs, both
  cross-audit legs clean); all four plants fired (plants.log)
- pins: criterion s4core from selector4 blob d33f0469; label ruleb from
  refute_lib blob cbaf2b47 (tag-free by gate)
- the primary verdict is under MIN_DECIDED_RUNG = 3 (ruling 3 as proposed).
  The coarse-rung fallback is a labelled sensitivity line only, never an
  alternative headline. This was written before the run.
- if it dies: rerun from scratch. Do NOT resume partially -- a half-scored
  census must never reach E1.
