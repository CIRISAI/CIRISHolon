# RECORD-1 — results: the eleven classify our record at one coder; a kind does not persist from one correction to the next

*2026-09-26. Prereg `RECORD1_PREREG.md` (frozen alone, `8975be1`), after the corpus
(`RECORD1_CORPUS.jsonl`, 269 entries, `df27907`) and before any code existed. The codes
(`RECORD1_CODES.jsonl`, one coder, blind order) were committed alone at `60899f6`, before the
statistics code was written. The read is `record1.py stats` (pinned `taskset -c 18-20`, no
timers), printed to `record1_read.txt` and `record1_read.json`. The blind key
(`RECORD1_BLIND_KEY.json`, sha256 `fdc1760e…decb8e` as the prereg stated) is committed with this
document. No stake moved; what the prereg got wrong is appended to it under "Notes on
building".*

## Verdict

**Branch (b): S1 met, S3 killed.** At one coder, every corpus entry that states a change took
exactly one of the eleven kinds. The Record flag held its one-way reading. But a correction's
kind does not predict the next correction's kind in the same campaign: the diagonal sits at
the order-permutation null (`D = 74` against `77.5 ± 6.4`, `p = 0.74`). Kinds cluster by
campaign (`3.5` sd above a global permutation), not by succession inside one. All three plants
pass. The agreement stake is not read, because no second coder was arranged.

| stake | reading | verdict |
|---|---|---|
| **S1** coverage | `264 / 264` CHANGE entries take exactly one kind (`1.000`; bar `0.95`, kill `0.85`). NO-FIT `0`, inseparable `0`. Corpus defects `5 / 269` (`0.019`, cap `0.10`). Strict reading with the defects counted as residue: `264 / 269 = 0.981`. **Contested: 188 of 264 (`0.71`) had a defensible runner-up that a tie-break decided.** | **MET**, at one coder (see *What S1 means* below) |
| **S2(a)** flag against warrant | `0` exceptions over 269 codes. Record = TRUE on `207 / 269` (`0.770`) | met. Self-consistency only, as declared |
| **S2(b)(i)** coded reversals | `1` (R1-A083, RUNG2 Amendment 2's CORRECTION restores a formula the same document had rejected). Its own warrant is external (the settled seeds) | not killed |
| **S2(b)(ii)** git reversals | `5` pattern hits, adjudicated below. None undoes a Record-TRUE correction from inside an artifact | not killed |
| **S2(b)(iii)** text at HEAD | VACUOUS BY CONSTRUCTION, as declared | — |
| **S3(a)** forbidden cells | `0` occupied. VACUOUS BY CONSTRUCTION on the binding register (prereg §4) | — |
| **S3(b)** diagonal | 314 document-pair transitions across 29 campaigns (the effective N). `D = 74` (`0.236`) against the order-permutation null `77.5 ± 6.4`, one-sided `p = 0.744` | **KILLED** (at or below the null mean) |
| **S4** distribution | below. The lead's guess fails on both readings as worded. **Premises is the dominant root of the whole corpus (121 of 264).** | reported |
| **PL-1** | `22 / 22` synthetic entries coded to their written kind (must be `≥ 20`) | PASS |
| **PL-2** | shuffled kinds: `D = 39` against `47.9 ± 4.9`, `p = 0.96`. Overlay `139` against `132.0 ± 11.8`. Forbidden `0` | PASS |
| **PL-3** | the two inconsistent synthetic codes are flagged `2 / 2`; the consistent mirrors are flagged `0 / 2` | PASS |
| agreement | no second coder | **not read** |

## S1 — what "met" means at one coder

The eleven took every change. No entry needed a twelfth kind, and none needed two kinds that
the rule could not separate. That is the stake as written. It is also the weakest way it
could have been met. The coder wrote the rule and its five tie-breaks, and a forced-choice
coder with tie-breaks refuses only when no tie-break applies. **71 % of the entries had a
named runner-up.** S1 therefore measures how decisive the rule is, not whether the
taxonomy partitions change. The substance is in which boundaries were contested (reported,
not graded):

| contested pair | n | | contested pair | n |
|---|---|---|---|---|
| Facts / Premises | 18 | | Model / Premises | 11 |
| Model / Rules | 16 | | Confidence / Facts | 11 |
| Premises / Rules | 15 | | Confidence / Rules | 8 |
| Identity / Rules | 12 | | Confidence / Identity | 8 |
| Facts / Model | 11 | | Identity / Model | 7 |

Only 66 of the 188 contested pairs lie inside one `Surface.lean` block. The seed's two named
boundaries are both on the list: Confidence/Facts at 11 and Model/Facts at 11. The two
largest are Facts/Premises (the T2 tie-break: is the claim regime-bound?) and Model/Rules
(the T1-before-T3 order: is a replaced number a criterion or a computation?). On this
material, those are where a second coder would most likely disagree. LEG C's prior art
predicts the disagreement will be large: panels read κ 0.25–0.36 on wild change units,
against 0.83 for humans working from the definitions.

The five corpus defects, listed by text in the read:
- two SPLIT (R1-A030: the campaign seed plus stim's column definition; R1-N006: a gate item
  that absorbed the next unnumbered paragraph);
- three NOT-A-CHANGE (R1-N056, a mathematical lemma recorded; R1-N088 and R1-N090, the
  declared selection's output and the frozen manifest).

## S2 — the Record axis on our record

**(a)** Zero exceptions. By construction the rule cannot produce an exception for a coder who
applies it, and PL-3 shows the checker would catch one. The rates are the content:

| source | n | Record = TRUE |
|---|---|---|
| amendment | 96 | 0.677 |
| notes | 88 | 0.727 |
| misfit | 71 | 0.915 |
| instance/CORRECTION line | 9 | 0.889 |

Amendments carry most of the artifact-internal corrections. Examples: a formula that
contradicted the number beside it (LIQUID-1 A1), two of a prereg's own lines disagreeing
(RESPONSE-1 A1's sin/cos quadrature, GF2a A3's χ against its own rank table), and a criterion
the freeze's own extrapolation could not meet (GF2a A1's G1). The misfit registry is almost
wholly Record-forced: a fault is registered when a read convicts it.

**(b)(i)** One coded reversal: R1-A083. RUNG2 Amendment 2's prepended CORRECTION says a
formula the same document had dismissed as "3–4× too wide" "was right". It restores the
state before an earlier Record-forced dismissal. Its own warrant is external (three settled
seeds, spread `36–44` au), so it is a frame-forced reversal of a frame-forced change, which
the axis allows. It is not a kill.

**(b)(ii)** The five pattern hits, each adjudicated by hand:

| commit | what matched | adjudication |
|---|---|---|
| `8975be1` | "undone" in this campaign's own prereg subject | self-match. Not a reversal |
| `9273777` | "restores the amplitude fidelity" (physics) | lexical. Not a reversal |
| `165ca21` | "Verified in three cases: void, fail, restored" (test cases) | lexical. Not a reversal |
| `ca7d39c` | a parser fix restoring lost tokenization | a code fix. It corrects no description |
| **`05a965c`** | "Restore RULE-B at HEAD to the refutation's own blob" | **the one substantive case: read below** |

`05a965c` is the nearest thing on our record to what S2 forbids. The SELECTOR-5 refutation
(`bfd70e0`, a Record-forced correction: C4 reproduced by `(tag, name, |G|)`; R1-M029 in this
corpus) was committed together with a rewritten `rule_b_sm`. A bare `git add` from a
neighbouring lane had swept the rewrite in. It put a family-name short-circuit back into the
corrective campaign's own label, which is M-TAG-AS-PROPERTY reintroduced. It carried no
warrant, not even an internal one: it was a speed-up that landed by accident. For 84 minutes
the instrument at HEAD read tags again. The refutation's TEXT and its published numbers never
changed. The corrected state was re-established from what survived around the artifact: the
refuter had pinned the blob (`cbaf2b47`), and six of eight checkable cases crashed. **By the
letter of S2(b)(ii) this is not a kill.** The matched commit restores, on an external
warrant, and the regression changed no description. **In substance** it is the Record
relation working as the seed states it. The past stayed provable because the pinned blob
survived around the file. It stayed provable despite what the file said, not because of it.
It is reported at full volume because the next such event may land in a description.

**(b)(iii)** is vacuous as declared: the corpus is read from HEAD. The search has a limit: it
sees reversals whose commit messages say so. A reversal made silently inside a larger edit is
invisible to it, and nothing in this design can reach one.

**Beside S2, not staked:** 8 of the 11 kinds carry both Record values. Priorities, Process
and Circumstances are all-TRUE on 3, 4 and 5 entries. So on this corpus the Record flag is not
a function of the kind, which is what `record_not_computable_from_artifact_reads` says of the
model. This is consistent with the theorem, not a test of it.

## S3 — the transition map on our record

S3(a) is vacuous by construction, as the prereg's §4 declared before the run. LEG A has no
successional cell between two of the eleven, and the Record is a flag here, not a sequence
element. The operational overlay was reported and not graded. It is occupied at the null
rate, as predicted: `140` against `134.0 ± 11.1` over the 71 cells that some channel forbids
and none allows. Succession on our record ignores the operation channels, which is what NOTE
A1.1 of the transition-map prereg said succession counts would do.

**S3(b) is killed.** Over consecutive documents, same-kind pairs occur at the rate a
permutation of kinds inside each campaign gives:

| reading | transitions | diagonal | null | p |
|---|---|---|---|---|
| **documents, all pairs (graded)** | 314 | **74** | 77.5 ± 6.4 | **0.744** |
| entries, consecutive (labelled, not graded) | 194 | 59 | 45.3 ± 4.2 | 0.0016 |
| documents against a GLOBAL permutation (reported after the read) | 314 | 74 | 49.3 ± 7.1 | 3.5 sd above |

Three readings of one number:
- **The entry-level count would have "met" S3.** It meets it on the splitting rule: one
  amendment's enumerated changes sit next to each other and share a kind. That is the artifact
  the prereg's deviation 3 named before the run, and the document unit removes it.
- **The campaign-level excess is composition.** RESPONSE-1 is a Rules-and-Model campaign,
  RUNG-2 a Model campaign (8 of 10), and BRIDGE-1's rows are all Premises. A global
  permutation puts `D` 3.5 sd high. A
  permutation within each campaign puts it at the mean.
- **What persists is the campaign's kind mix, not a kind's succession.** The diagonal mass is
  Rules→Rules 33 and Model→Model 33 and almost nothing else. The largest off-diagonal cells
  are Model→Rules 31, Structure→Rules 19, Facts→Rules 12, Structure→Facts 11, Rules→Model 11
  and Model→Facts 11. A computation replaced and then a criterion re-staked is the commonest
  step on our record.

TM3 of the transition-map prereg staked diagonal dominance on the wild stream, and LEG C's
void run saw it at `p = 0.0008` on Stack Exchange. On this programme's record, read at the
document grain, it does not hold. The two carriers differ (copy-editing streams against
corrections forced by reads), and nothing here re-reads LEG C.

## S4 — the distribution

| kind | site | root | Record-TRUE |
|---|---|---|---|
| Rules | 66 | 18 | 47/66 |
| Model | 61 | 55 | 41/61 |
| Confidence | 30 | 2 | 28/30 |
| Facts | 26 | 11 | 22/26 |
| Premises | 25 | **121** | 21/25 |
| Identity | 23 | 18 | 14/23 |
| Structure | 17 | 16 | 16/17 |
| Circumstances | 5 | 12 | 5/5 |
| Manner | 4 | 5 | 1/4 |
| Process | 4 | 6 | 4/4 |
| Priorities | 3 | 0 | 3/3 |

(264 coded entries. The root is the coder's reading of why each correction was needed.)

- **The misfit registry's recurring faults.** On the 71 misfit occurrences, the sites are
  Confidence 18, Rules 15, Premises 10, Identity 9, Facts 7, Model 6, Structure 6. The roots
  are **Premises 45**, Model 8, Structure 6, Identity 3, Circumstances 3, Rules 2, Process 2,
  Manner 2.
- **The lead's guess — "Premises and Rules dominate" — fails as worded on both readings,
  and each half holds on one.** Rules is the second site. A registered fault is corrected
  first by re-staking or by downgrading a claim (Confidence tops the sites: VOID in substance,
  a verdict retracted). Premises is by far the first root: 45 of 71, and 121 of all 264
  corrections. "A constant is a price in a regime" is the modal reason this programme
  corrects itself. It usually shows up as a re-staked Rule, a replaced Model or a downgraded
  Confidence, not as a Premises edit. "A bar set from the read" is a Rules site with a Rules
  root, as guessed (R1-N074, R1-M007).
- **The builder's prediction fails too.** The commonest sites are Rules and Model, not Facts
  and Rules. Identity is not rare (23 entries: definitions of a water unit, a tier, a closed
  sector, a device-class artifact). Record = TRUE came in at 0.77 against the predicted
  four in five.
- **Surface against depth.** The gross four (Facts, Rules, Identity, Manner) carry 119 of
  264. The depths carry 145, led by Model at 61. This carrier's corrections live largely in
  the depths, unlike the wild traffic `Surface.lean`'s header describes as lopsided toward
  the four. A reading, not a stake.

## Plants

- **PL-1**: 22/22. The coder wrote the plants, so this shows the rule's definitions are
  determinate on their own examples, not that the coder was blind.
- **PL-2**: the shuffled corpus falls to its null on the diagonal (`p = 0.96`). The overlay
  sits at its null. Forbidden cells stay at 0.
- **PL-3**: both inconsistent codes are flagged and both consistent mirrors pass. The first
  build tested the mirror clause against the real codes after filtering them to the
  consistent ones, so that clause could not fail. It was fixed after the first read to test
  the two mirror codes explicitly (M-VACUOUS-SUCCESS, the harness's own instance). The
  verdict did not move.

## Second coder

Not arranged: the brief spawns none from this lane. The blind file (`RECORD1_BLIND.jsonl`,
text only, shuffled, 22 plants hidden among 269 entries) and the rule (prereg §2) are
committed, so the lead can hand them to a second coder. A second pass would be scored by
Cohen's κ on `kind` and raw agreement on `record`, against LEG C's 0.40 floor. It should not
read the PL-1 block in `record1.py` or the key. Until that pass exists, S1's MET is one
coder's decisiveness.

## What this is not

Not a test of the site model's adequacy beyond this carrier. Not a re-read of LEG C. No
physics. `record_not_site_generated` is a theorem about a model. Nothing here supports it or
refutes it: S2 found no counterexample on our record, and the one near-instance (`05a965c`)
shows the frame doing its job.

witness: none (a measured campaign; the Lean it tests is the sibling seed's, `record_not_site_generated` and `repairable_does_not_factor`, cited as the object under test)
**misfits:** M-VACUOUS-SUCCESS (S1 at one coder, S2(a), S3(a) and PL-3's first build, each named), M-TAG-AS-PROPERTY (the `05a965c` regression), M-BASE-RATE-OMITTED (every per-kind Record rate is printed beside its base rate), M-ONE-MODEL-DELTA (the diagonal is graded against one null; the global permutation is reported beside it), M-FOREIGN-DOMAIN-CORROBORATION (nothing transfers to LEG C), M-PLANT-OBS, M-PLANT-SECTOR (the plants ran on the carrier of record), M-HOMOG (contacted by the word "artifact-local"), M-MAINTENANCE-LENS (contacted by the word "repair"), M-COND-PROBE (contacted by "inside the artifact"), M-PLACEMENT-LOTTERY (`taskset`; no timing read), M-PROVENANCE-OVERREACH (the key's sha256), M-STALE-INSTRUMENT (the instrument is committed with this document). These are the registered ids this text contacts, cited.
