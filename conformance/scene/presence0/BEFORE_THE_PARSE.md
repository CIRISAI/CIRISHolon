# PRESENCE-0 — the record made before any diary is parsed (2026-11-11)

- **Plants.** All nine pass on synthetic diaries (`plants.log`).
- **A plant that says more than "pass".** P-NULL-YEAR draws a held-out year from the same
  table at about the real sizes (16,000 fitted, 8,000 scored). At those sizes NO cell's
  gap is under half the `0.03` budget, so no cell can carry a verdict and the plant's
  certified share is empty. By the freeze's own rule S6a and S6b will then be reported as
  untestable at this budget, with their differences and errors printed. The plant's
  pass condition as coded (no spurious refusals) is weaker than its name.
- **Files.** Five archives, fetched through a browser with the lead's leave because the
  publisher's site refuses plain downloads; digests in `raw_file_sha256.txt`. Looked at so
  far: each file's header line, the info texts and the read-in programs' labels. No row.
- **How the files were fetched, for the record.** The first attempt started five
  downloads at once and three were cut short; those partial files were this session's own
  and were deleted, and the three were fetched again one at a time. Each archive passes
  its integrity test.
- **Read from the files' own programs:** durations are the 24-hour fields
  (`TUACTDUR24`); the place is `TEWHERE`; the activity is the pooled six-digit code.
- **Choices the freeze left open, stated before the read:**
  - Asleep is activity code 0101xx (sleeping and sleeplessness).
  - Travel is place codes 12 to 21 and 99, or an activity in the travel tier. Sleep while
    travelling counts as travelling.
  - Workplace or school is place 2 or 8; home is 1; a negative place code is unlocated.
  - An unlocated episode takes the place of the located episode before it in its
    travel-free segment, or the first one after it if none is before.
  - Employed is labour-force status 1 or 2. Household size is the respondent file's count.
    Metropolitan status is the linked file's, for the respondent's line.
  - The replicate-weight variance is 4/160 times the summed squared deviations (the
    survey's 160 half-sample replicates with the factor one half; from memory of the
    survey's guide, not re-read). If the replicate error exceeds the bootstrap's, each
    bootstrap error is multiplied by the median ratio.
  - S3 and S4 are judged on point estimates, with the gap printed.
  - S6a's classes are age band by employment (six); its strata are household sizes 1, 2
    and 3 or more.
  - Holidays stay on their weekday.
