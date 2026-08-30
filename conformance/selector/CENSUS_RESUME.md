# SELECTOR-6 census — resume note

Detached run of `census.py` (orders 1..63). Session death kills narration only.

- launch: `cd conformance/selector && setsid nohup nice -n 19 python3 census.py > census.log 2>&1 < /dev/null &`
- done marker: `census.DONE`; output: `census.json` (per-order tables), `census.log`
- staged validation already passed: orders 1..32 give 144 types, exactly the pin
  total; S1 VOIDs NONE; abelian cross-audit NONE; Holder cross-audit NONE.
  Order 32 alone: 51 types from 3084 candidates in 148 s (the declared
  Aut(Z_2^4)=20160 cost).
- if it dies: rerun from scratch (the build is deterministic and ~20 min); there
  is no partial-resume state, by design -- a half-census must never be scored.
