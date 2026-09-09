# The 14-block instrument set at 1x, moved aside

This is the pilot set `DRIFT_NOTE.md` and `LIQUID2_PREREG_DRAFT.md` describe as `pilot_1x/`:
three pilots at `1x`, `14` blocks each, matched to the `8x` demonstration set in physical time.
It was moved here on 2026-09-09 so that `pilot_1x/` could hold the DECLARED 60-block set
(`liquid2_pay.sh`). It stays because it is the record the fourth review read (block 14 of
each: 299.2 / 294.9 / 298.4 K, 2.78 / 2.83 / 2.78 Å, 3.06 / 3.09 / 3.08 both-ends) and
because the structure arm's `Sprice` gate refuses to size an arm on it (`blocks_run` 14 of
60): `read_pilot_precision` visits `pilot_1x/` first by name and reads this directory only
when that one holds no pilot at this step.
