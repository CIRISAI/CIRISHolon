# GRID-1A, the adversarial review of the draft (2026-11-12)

One reviewer subagent read `GRID1A_AMENDMENT_DRAFT.md`, the frozen parent and its script,
and counted from the version-3 table. Nothing was fetched. Verdict: freeze after named
fixes. Each is applied in `GRID1A_AMENDMENT.md`.

Must-fix:

1. Branch (e) did not cover the extractor's other stops (a repeated row, an unknown area,
   another unit, an id collision, a missing column, a failed fetch, a failed keep check on
   the real merge); by the draft's text they fell to (a). (e) now names them.
2. R2 did not say what an absent place or year counts as and could pass on 0 comparisons.
   Now: matched on the published name and year; an absent one counts against; the number
   compared must be 700; plant P-ABSENT.
3. The same-file bar of 0.0005 sat on a rounding edge and its reason was false: the table
   stores the file's value unrounded. Now exact equality.
4. A second row with `latest_for_place: true` for one place would break a filter on that
   field and leave 32 pointers stale. Now no new row carries it; plant P-ONELATEST.
5. On another edition a place's series would mix two editions under one id pattern. Now
   the full series is written under the new edition with suffixed ids; plant P-EDITION.
6. The "Why" counts were wrong (111 places lack a 2024 row, 95 of them with 2025 as
   latest and 16 with 2023 or 2022; 99 have 2024 and no 2025), and "withdrawn in
   GRID1_RESULTS.md" pointed at a correction not yet written.

Should-fix, applied: the factor-of-2 tripwire is split by `small_system` (155 of the 700
rows are small systems); "its latest year" is the file now read; a year-shift misread is
named under vacuity with plant P-SAME; the edition string never depends on one header;
the merge appends only and P-KEEP4 compares the key set.

Notes kept: on the same file R2 is a tautology unless the script changed, and (m) is then
the only test of the new script on real data; a version-3 latest row that a later edition
moves still carries `latest_for_place`, and the sidecar's lists are where a consumer checks.
