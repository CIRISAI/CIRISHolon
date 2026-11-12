# DEVICEGEN-1 — the record made before any survey row is parsed for it (2026-11-12)

- **The generator exists** (`engine/crates/holon-scene/src/devices.rs`), with the plants
  that live in the crate as tests: persons sum to the size for sizes 0 to 9 with sizes
  above the top code flagged; counts agree with the cell; two phones each; one seed one
  household; a thin stratum falls back and records the level; nobody holds nothing. The
  crate has 73 tests, all passing.
- **The scoring's plants** pass through the shipped binary (`plants.log`).
- **A plant whose first carrier was wrong, corrected before the read.** P-COVARIATE first
  made "nothing to host on" more likely in apartments but still a random subset of them.
  That is not "explained wholly by building": a rival that draws each kind alone given
  building cannot reproduce a random subset's joint, and it read 1.16 against a survey
  2.92. The carrier now makes the have-nothing households exactly the one-person
  apartments; the rival then reads 2.88 against 2.94 and the kill fires, as the plant
  says. The rival and the stake were not changed.
- **Files.** Both survey years, fetched again; their digests match those recorded for
  DEVICES-0. No row parsed for this campaign.
- **Choices the freeze left open:**
  - Building: detached, attached and mobile home are "house"; apartments of 2 to 4 and of
    5 or more units are "apartment".
  - Connection: a fixed line; internet without one (mobile data or other); none.
  - Smartphones per person: three rates per connection by weighted least squares
    (adults 18 to 64, adults over 64, children), capped at 1.6; a person holds two with
    probability `q` times the rate, and `q` is the one number that makes the share of
    households with more phones than persons match FIT's.
  - The other kinds: histograms 0 to 8, written for a stratum with at least 30 FIT
    records; fallback as frozen, with a last level by pooled size alone.
  - A household in a "computer" cell that draws no laptop, desktop or tablet is given one
    laptop; in a "speaker or streaming" cell that draws neither, one streaming device.
  - Television hours: the main television's weekday hours, 0 to 24; zero without one.
  - Seeds: a held-out household's 20 seeds are its row number times 1,000 plus 0 to 19.
  - G0's cell bar of 0.002 is read with the Monte Carlo error of the check (10 seeds over
    the FIT households, about 0.004) added.
  - **The no-draw expectation is NOT computed.** The freeze says each generated value
    carries the table's expectation with no draw beside it. With the consistency step the
    expectation is not a closed form; the 20-seed mean is what is printed. This is a
    departure, stated here.
  - S7's rival is given pooled size, building, any child and any adult over 64, taken
    from the held-out household itself, and draws the connection and each of laptop,
    desktop and tablet alone.
  - S8: 200,000 households with sizes from HOUSEHOLD-0's national US row.
  - S9's census figures are from memory and are labelled so in the output.
