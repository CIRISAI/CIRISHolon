# RECORD-1 — the taxonomy of change on its native carrier: this programme's own corrections

*Frozen 2026-09-26, committed ALONE, after the corpus (`RECORD1_CORPUS.jsonl`, 269 entries,
commit `df27907`) and BEFORE a single entry is coded: at the time of this commit
`RECORD1_CODES.jsonl` does not exist and no statistic has been computed. The corpus is data;
this text is the freeze. The stakes are the lead's; the coding rule, the operational
definitions and the deviations from the brief (§7) are the builder's, stated here so that
none of them can be chosen after the codes exist.*

witness: none (a measured campaign; no gate is a theorem of this repository. The Lean it tests lives in the sibling seed, cited as the object under test and never as evidence: `record_not_site_generated` and `generator_injective` (`CIRISOntology/Core/Generator.lean`), `repairable_does_not_factor` and `repairability_not_intrinsic` (`Core/WrongKind.lean`), `Block.surface` and `block_cards` (`Core/Surface.lean`), and the Record-axis cells of `LEG_A.md` §2.1 (`kinds_are_frame_scalars`, `record_not_computable_from_artifact_reads`, `no_frame_restriction`))

## 0. The object

`OBJECT.md`, "The shape, stated four times": the **11 + 1** — eleven ARTIFACT-LOCAL kinds of
change and one frame relation, the Record, "whether the past can still be proven depends on
what survives AROUND the artifact". The seed proves, GIVEN ITS MODEL, that the eleven are the
bijective image of eleven sites (`generator_image`, `generator_injective`) and that no site
generates the Record (`record_not_site_generated`), because repairability is a relation to a
frame and factors through no artifact-only property (`repairable_does_not_factor`). The seed's
own header puts the risk where it lives: "the question 'are there more kinds?' does not
vanish — it MOVES to 'is this model adequate?', which is answerable by measurement."

The measurement here is on the taxonomy's NATIVE carrier: descriptions, and changes to them.
This programme's record of its own corrections is such a carrier, and it is dated. Every
entry of the corpus is a change the programme made to one of its own descriptions (a prereg,
an amendment, a results document, `OBJECT.md`, the misfit registry) between 2026-08-26 and
2026-09-26. Prior art on the same question, credited: the transition map's LEG C
(`TRANSITION_MAP/LEGC_RESULTS.md`) tried the wild matrix on Stack Exchange revision chains
and read VOID on agreement five times (panel κ 0.25–0.36 on wild single-change units,
against 0.687–0.711 on curated items and 0.831 for humans from the definitions); its
void-run observation was a diagonal share of 0.50–0.67 against a within-chain shuffle.
Nothing here re-reads LEG C, and nothing read here transfers to it (different carrier,
different coder, different grain).

## 1. The corpus (committed, `df27907`; built by `record1.py corpus`)

Four sources, the extraction spec declared in `record1.py` and committed with the data:

| source | rule | entries |
|---|---|---|
| amendments, all 35 `*_AMENDMENT_*.md` | each change the amendment enumerates (A-numbered sections, a numbered change list, a found-items list); a title clause not covered by those; each section it marks as found after its own freeze ("found on building / launching / first walk / first contact", "Correction on building", a prepended CORRECTION) | 97 |
| notes | each numbered item under "Notes on building" or "what the prereg / freeze / first version / campaign got wrong" in `*_RESULTS.md` and `*_PREREG.md`; where a RESULTS list restates the PREREG's notes the RESULTS list is canonical; a matching section with no numbered items gives one entry per bold-led correction | 92 |
| misfit occurrences | each occurrence a `MISFITS.md` row names with enough text to code | 71 |
| instance and CORRECTION lines | "the Nth instance" and CORRECTION lines of `GANTT2.md`, `OBJECT.md`, `STANCE.md` not already in the corpus | 9 |

26 restatements are FOLDED into their canonical entry (amendment > notes > misfit > doc line)
and listed by name in `record1.py` (`DUPS`); they are not entries. Dates: a date the text or
its section heading states; else the amendment's own "Frozen/Written" date; else git (blame
of the line, or the first commit carrying the misfit phrase). Every entry is in the window.
Known holes, stated now: instance ordinals 1–7, 13, 16, 20 and 21 of "a constant is a price
in a regime" are named in none of the three documents source 3 reads, so they enter only if
another source carries them; `M-PLANT-OBS`'s two BRIDGE-6 occurrences carry no separating
text and are one entry.

**The unit a transition is counted on.** A *document* is an amendment file or a notes
section at one date (its entries are the author's exposition, not a succession); each misfit
occurrence and each instance/CORRECTION line is its own document. A campaign's documents are
ordered by date, then by amendment number / file order.

## 2. The coding rule (what the coder applies, once, blind)

**Input.** `RECORD1_BLIND.jsonl`: 291 items — the 269 corpus entries and the 22 PL-1 plants
(§5) — shuffled (seed 20260926), text only: no id, file, campaign or date, so a plant cannot
be told from an entry by its metadata and adjacent items are not adjacent in any campaign
(coding in campaign order would let the coder's own momentum manufacture a diagonal). The key
(`RECORD1_BLIND_KEY.json`, sha256 `fdc1760e9de32e12a414e58c05dda6445f39161ede12ba98eecdd47bb5decb8e`)
is withheld from the coder and committed only with the results.

**Output.** `RECORD1_CODES.jsonl`, one line per item, written in full and committed-ready
BEFORE `record1.py stats` exists or runs:
`{bid, status, kind, alt, separable, record, warrant, warrant_class, reverses, root, note}`.

**Step 1 — status.** `CHANGE` (the text states a change to a description — a replacement,
or an addition that fills what the frozen text left open, which is a correction of an
omission), `NOT-A-CHANGE` (it states no change to any description — a result or an event
recorded that neither replaces nor adds to what a description says), or `SPLIT` (two or more independent, co-equal changes, none of which the others
follow from — a corpus-construction defect). `NOT-A-CHANGE` and `SPLIT` are corpus defects,
excluded from S1's denominator, listed by text, and CAPPED (§3, S1).

**Step 2 — the site: which component of the described artifact did the correction
REPLACE?** Code the component whose text is different after the correction, not the reason
the correction was needed (that is `root`, step 5). The eleven components, each with its
site as the seed defines it (`Generator.lean`, `Site`) and its discriminator
(`WrongKind.discriminator`), and the operational reading used here:

| kind | the seed's site | the seed's discriminator | on this carrier |
|---|---|---|---|
| Facts | "an assertive's content changes truth-conditions" (`factContent`) | "What claimed fact becomes wrong?" | a claimed value, reading, count or statement about the world, the instrument or the record is replaced by another |
| Confidence | "a strength/hedging marker on an assertive moves" (`strengthMarker`) | "How sure are we, and on what standard?" | the same claim is held more or less strongly: a finding downgraded to unresolved, VOID-in-substance, an error bar or significance standard changed, a retraction whose content is not shown false |
| Model | "the rule APPLIED to derive downstream content is swapped (use, not mention)" (`appliedRule`) | "What rule or model are we reasoning under?" | the formula, estimator, bound, fit form, derivation or mechanism used to compute an answer is replaced |
| Premises | "the founding assumption other components compose over is swapped" (`foundingAssumption`) | "What are we taking as given?" | an assumption the design composes over is dropped or replaced: the regime, the carrier's nature, a transferred constant, what the data's shape was taken to be |
| Rules | "a directive's permission content changes" (`directiveContent`) | "What becomes allowed or required?" | a stake, bar, gate, kill, verdict rule, obligation or prohibition is added, removed or re-staked |
| Priorities | "the preference ORDER over outcomes is permuted" (`preferenceOrder`) | "What becomes more important?" | which reading, quantity or arm is primary, or the ranking among them, changes |
| Process | "the step ORDER is permuted" (`stepOrder`) | "What steps or ordering change?" | the procedure's steps or their order change: what runs before what, what is committed when, what is re-run |
| Identity | "a declaration's counts-as content changes" (`declarationContent`) | "What is this said to be?" | a definition or classification: what an object, unit, term or run counts as |
| Structure | "the serialization/encoding is altered" (`encoding`) | "How are the pieces put together?" | layout, format, encoding, data structure, code architecture or dependency graph |
| Manner | "the register/presentation wrapper is altered" (`register`) | "How is the same thing presented or used?" | the same content presented differently: wording, labels, print format, a legend, a column shown beside another |
| Circumstances | "an unbound instance token (not intended invariant) differs" (`instanceToken`) | "What just happens to differ here?" | an incidental event or token with no content claim: a crash, an interruption, a placement, a seed that happened to be used |

The seed's two boundaries travel with the names (`WrongKind.lean`): **Confidence vs Facts**
("the proposition can stay identical while warranted confidence changes") and **Model vs
Facts** ("`nomological` is the model APPLIED to derive an answer. A model ASSERTED to be
descriptively true of the world is a Fact"). The builder's tie-breaks, in order, applied
when two rows both read on an entry: (T1) a replaced criterion of acceptance — a number that
decides a verdict — is **Rules**, not Facts; (T2) "X was taken to hold here and does not" is
**Premises**; "X's value is v, not u" with no regime clause is **Facts**; (T3) a replaced
computation is **Model** even when its output number also changes; (T4) an obligation
("must", "never", "every row carries") is **Rules**, the same content re-presented without an
obligation is **Manner**; (T5) a definition is **Identity** even when a rule is re-read
under it. `kind` is the coded site; `alt` is a second kind the coder judged defensible under
the rule (or none); `separable` is true when a tie-break above decides between them and
false when none does (that is the kill in S1). `NO-FIT` is a legal `kind`: a change none of
the eleven describes.

**Step 3 — the Record flag** (the frame relation). `record = TRUE` when the correction was
FORCED by something outside the artifact — a read, a datum, a run, a machine event, another
session's work, prior art, an external review — and could NOT have been produced from the
artifact's own text; `record = FALSE` when the artifact's own text licenses it — an
arithmetic slip in its own numbers, an inconsistency between two of its own lines, a
citation, a rounding, a definition its own theorem already fixed. The criterion is where the
WARRANT lies, not who found it: an external reviewer who points out that a document's
Theorem 3 contradicts its own Theorem 2 found a Record-FALSE correction. `warrant` quotes
the source the text names for the correction; `warrant_class ∈ {external, internal}`. This is
the seed's obligation made operational: a testimonial classification "MUST carry the corpus
against which non-re-derivability is asserted" (`WrongKind.lean`, `testimonialNamesItsCorpus`);
a Record reading "cannot be constructed without a frame" (`reading_record_has_frame`).

**Step 4 — reversals.** `reverses`: when the text undoes an earlier correction (restores the
state before it), a quotation identifying the undone change; else none.

**Step 5 — root.** The kind of the reason the correction was needed, when the text states
one distinct from the site (e.g. a bar re-derived because a transferred constant did not
hold: site Rules, root Premises). Reported in S4 only; enters no stake.

**One coder.** No second coder is arranged by this lane (the brief: "if you cannot arrange a
second coder, code once and say so, and the agreement stake is NOT read"). The coder is the
builder, who wrote the rule and the plants and has read the stakes; the blind file removes
metadata and order, not the coder's knowledge. LEG C's prior art says a single model coder's
decisiveness is not agreement: its panels splintered on exactly this material. The blind
file is committed so the lead can hand it to a second coder; that coder's agreement with
this pass (Cohen's κ on `kind`, raw agreement on `record`) would be read against LEG C's
0.40 floor in a later document.

## 3. Stakes (the lead's)

- **S1 — coverage.** Among entries with `status = CHANGE`, at least **95 %** take exactly one
  kind of the eleven (not `NO-FIT`, not inseparable); the residue is listed by text. **Kill:**
  under **85 %**, or ANY entry coded `separable = false` (two kinds the rule cannot separate).
  **VOID (corpus, not taxonomy):** `NOT-A-CHANGE` + `SPLIT` above **10 %** of the corpus —
  the corpus rule, not the eleven, failed. Reported beside, never graded: the strict reading
  with corpus defects counted as residue, and the contested rate (`alt` set but separable).
- **S2 — the Record axis is one-way on our record.** (a) Every `record = TRUE` code carries
  `warrant_class = external` and every `record = FALSE` carries `internal` — 0 exceptions
  (this is the rule's self-consistency, tautological for a coder who applies it, and PL-3 is
  its plant). (b) NO Record-TRUE correction was later REVERSED by an artifact-internal edit:
  searched (i) within the corpus — every `reverses` code whose undone change is Record-TRUE
  and whose own `record` is FALSE; (ii) in git — every commit from 2026-08-26 to 2026-09-26
  whose subject matches `revert|restor|reinstat|undo|back out|roll ?back|un-?retract` or whose
  body carries "This reverts commit", each adjudicated in the results by hand (does it undo a
  corpus entry's change; is its warrant internal); (iii) the corrected text of each
  Record-TRUE entry still stands at HEAD — VACUOUS BY CONSTRUCTION, since the corpus is read
  from HEAD, and stated as such. **Kill:** one Record-TRUE change undone from inside the
  artifact, by (i) or (ii) — `record_not_site_generated` read on our own record would be
  refuted.
- **S3 — the transition map on our record (LEG C's shape).** Within each campaign, transitions
  are ALL ordered pairs (kind of an entry of document k → kind of an entry of document k+1)
  over consecutive documents, entries with `status = CHANGE` and a kind only. (a) No
  transition lands in a cell LEG A marks FORBIDDEN in the successional register (§4).
  (b) The diagonal count `D` exceeds the order-permutation null: kinds permuted across the
  entries of each campaign (every document keeps its size and place), 10,000 draws (seed
  20260926); MET at one-sided `p ≤ 0.05`. **Kill:** a forbidden cell occupied (named), or `D`
  at or below the null's mean. Between the two — `D` above the null's mean at `p > 0.05` —
  S3 is UNRESOLVED, neither met nor killed. Effective N is the number of campaigns with at
  least two coded documents, reported beside `p`.
- **S4 — the distribution, reported.** The kind histogram (site and root), the Record rate
  per kind and per source, and which kinds carry the misfit registry's recurring faults.
  **The lead's guess, stated so it can be wrong:** among misfit-occurrence entries Premises
  and Rules are the two most frequent kinds ("a constant is a price in a regime" is a
  Premises change; "a bar set from the read" is a Rules change) — read on site kinds and on
  root kinds, separately. Reported beside, not staked: whether `record` is a function of
  `kind` on this corpus (`record_not_computable_from_artifact_reads` says the eleven jointly
  do not determine the Record; a corpus where every kind is all-TRUE or all-FALSE would not
  refute a theorem about frames, but it would be worth saying).

Every bar above is the lead's (95 %, 85 %) or conventional and declared before the data
(`p ≤ 0.05`, the 10 % corpus-defect cap, 10,000 draws); none is set from a reading of this
corpus.

## 4. What LEG A marks FORBIDDEN, and why S3(a) cannot fire here — stated before the run

LEG A (`LEG_A.md` §0) finds "no theorem about temporal succession between kinds". Its
FORBIDDEN cells are of two registers, and the frozen transition-map prereg separates them
by a note binding on all scoring (`TRANSITION_MAP_PREREG.md`, NOTE A1.1): "LEG A's cells are
OPERATIONAL (apply ground/modulate/mention/absorb to a kind-i change, get kind-j), NOT
successional … the operational cells are a separate overlay never scored against succession
counts." The successional FORBIDDEN cells are the Record axis only: Record → each of the
eleven and each of the eleven → Record (22 cells, §2.1/§4.4), plus Record → Record under a
shrinking frame. In this design the Record is a flag on every entry, never an element of a
kind sequence, so no kind → kind transition can land in a Record-axis cell. **S3(a) is
VACUOUS BY CONSTRUCTION on the binding register, and is declared so now; branch (a), if
reached, carries its S3 on the diagonal alone.** The Record axis is tested where it has
content on this corpus: S2.

Reported beside, never graded (NOTE A1.1): the OPERATIONAL overlay — the 71 cells LEG A
marks substantively FORBIDDEN under some channel and ALLOWED under none (§4.3's 73: grounding
never leaves the stack, 28; nothing enters or leaves the carrier layer under mention, 54;
Identity's absorption column, 10; union 73, less Structure → Manner and Circumstances →
Manner, which absorption allows). **Predicted:** occupied, at the rate the permutation null
gives (the overlay describes what an operation on ONE change yields, not what follows it),
so its occupancy is the null's within three null standard deviations.

## 5. Plants (each run on the carrier of record — the blind file and the coded corpus)

- **PL-1 — the rule classifies its own definitions.** 22 synthetic entries, two per kind,
  written from the table in §2 before the freeze (`record1.py`, `PL1`), shuffled into the
  blind file under neutral ids and coded in the same pass. **Must:** at least **20 of 22**
  coded to their written kind (the brief's 18/20 at two per kind for eleven kinds; 22 × 0.9
  = 19.8). Carrier: the blind file; the sector the plant acts on — the `kind` field — is
  nonzero in that carrier by construction (22 items with a known kind). Limit stated: the
  coder wrote them, so this tests the rule's definitions, not the coder's blindness.
- **PL-2 — shuffled kinds fall to the null.** The coded corpus with its kinds permuted across
  ALL entries (one permutation, seed 11), run through S3 unchanged. **Must:** the diagonal
  `p > 0.05` and `D` within three null standard deviations of the null's mean; the
  successional forbidden count 0 (as for any corpus under §4); the operational overlay's
  occupancy within three null standard deviations of the null's mean. Carrier: the coded
  corpus; the sector the plant acts on — the kind sequence within campaigns — is nonzero in
  it (every campaign with two or more documents).
- **PL-3 — the rule refuses a Record flag its warrant does not carry.** Two synthetic codes
  (`record1.py`, `PL3`): `record = TRUE` with an artifact-internal warrant, and
  `record = FALSE` with an external one. **Must:** S2(a)'s check flags both as inconsistent,
  and flags no code that is consistent. Carrier: the S2(a) checker applied to the two codes
  appended to the real ones; the sector — the `record` × `warrant_class` pair — is nonzero in
  it by construction.

## 6. Branches

- **(a)** S1, S2 and S3 met (S3 on the diagonal; §4): the eleven classify this programme's
  record of its own corrections, the Record axis is one-way on it, and the forbidden
  successional cells stay empty — the taxonomy is a measured fact about this programme's
  descriptions, at one coder.
- **(b)** S1 met, S3 killed: the kinds classify, the transition rule (persistence of kind
  across successive corrections) does not hold. **(b′)** S1 met, S3 unresolved.
- **(c)** S1 killed: the eleven do not partition change even on their native carrier.
- **(d)** S2 killed: a Record-forced change was undone from inside the artifact.
- **(e)** a plant fails: its stake is not read; the others are reported as not graded.
- **VOID** S1 by the corpus-defect cap.

Order of precedence: (e), then VOID, then (d), then (c), then (b)/(b′), then (a).
Predicted, so the read can be wrong against it: (a) — S1 near 97 %, Record = TRUE on about
four entries in five, a diagonal modestly above the null; Facts and Rules the commonest
sites; Manner, Priorities and Identity rare. If the lead's S4 guess and the builder's
prediction disagree, both are printed against the read.

## 7. Deviations from the brief, each stated before any code exists

1. **PL-1 is 22 entries, bar 20/22**: two per kind is 22 for eleven kinds.
2. **S3(a) is vacuous by construction** (§4): the brief assumed LEG A marks successional
   cells FORBIDDEN between kinds; it marks none, and its own map's binding note forbids
   scoring the operational cells against succession. The operational overlay is reported.
3. **S3 counts transitions between consecutive documents, all pairs**, not between
   consecutive entries: the corpus splits one amendment into several entries, and
   entry-to-entry adjacency inside one document would manufacture a diagonal out of the
   splitting rule. The entry-level count is reported beside, labelled.
4. **Corpus defects are excluded from S1 with a 10 % cap** (§3), and the strict reading is
   reported beside.
5. **S2(a) is the rule's self-consistency** and says so; the substantive half is S2(b).
6. **One coder; no agreement stake read** (§2).

## 8. Instrument and run

`record1.py` (stdlib): `corpus` and `blind` are committed (`df27907`); `stats` is written
after this freeze and after `RECORD1_CODES.jsonl` exists. It runs pinned with
`taskset -c 18-20`, has no timers, and resolves every path from its own location. It prints
a header of what it read — the number of codes, of each status, the corpus's and the codes'
sha256 — beside its verdicts, so a stale or partial codes file fails loudly rather than
reading as a pass.

**misfits:** M-PLANT-OBS, M-PLANT-SECTOR (all three plants on the carrier of record, carrier
sentences above), M-VACUOUS-SUCCESS (S2(a) and S3(a) declared vacuous where they are, S2(b)
and the diagonal carry the content; the stats header asserts its work count),
M-BAR-FROM-THE-READ (no bar from this corpus; §3's last paragraph), M-JOINT-PASS-REGION (S1,
S2 and S3 read disjoint fields of a code — status/kind, record/warrant, order — so a coding
meeting each meets all jointly; PL-1's synthetic block is such a coding), M-BASE-RATE-OMITTED
(S4 carries the Record base rate and the kind base rates beside every per-kind rate),
M-POPULATION-CHOICE (the corpus is a declared population with its extraction rule, its 26
folds and its known holes named in §1), M-ONE-MODEL-DELTA (the diagonal is graded against
one null, the order permutation, and says so), M-FOREIGN-DOMAIN-CORROBORATION (nothing here
transfers to LEG C's Stack Exchange carrier or back), M-TAG-AS-PROPERTY (the coder sees text,
never the source's file or campaign tag), M-HOMOG (contacted by the word "artifact-local";
nothing spatial), M-MAINTENANCE-LENS (contacted by the word "repair"; no rent or maintenance
claim is made), M-PROVENANCE-OVERREACH (the key's sha256 names the bytes withheld, nothing
inferred beside it), M-PLACEMENT-LOTTERY (`taskset` pins the run; no timing is read),
M-STALE-INSTRUMENT (the instrument's commit is cited for the corpus; `stats` is committed
with the results document), M-COND-PROBE (contacted by the words "inside the artifact", the
brief's own; no operator and no step are involved) — the registered ids this text contacts,
cited.

Carrier-sector statement (M-PLANT-SECTOR): every plant names its carrier (the blind file,
the coded corpus, the S2(a) checker with two appended codes), and the sector the plant acts
on is nonzero in that carrier by construction.

### Notes on building (2026-09-26; appended after the codes (`60899f6`) and the read; no stake, bar, plant or branch above moves)

The read is `record1_read.txt`; the verdict, branch (b), is `RECORD1_RESULTS.md`. Where this
text and the build disagree:

1. **S1 was staked where one coder can hardly fail it.** A forced-choice coder with five
   tie-breaks refuses only when no tie-break applies, so "inseparable" was never going to be
   coded by the rule's own author. S1 read `264/264`, but 188 of those entries (71 %) had a
   named defensible runner-up. The informative number was the contested rate and its pairs,
   and the prereg reported them only "beside". A freeze of this shape should stake coverage
   at two coders or stake the contested rate. M-VACUOUS-SUCCESS, on the stake the brief put
   first.
2. **SPLIT was applied only when the co-equal changes differ in kind.** The text says SPLIT
   for "two or more independent, co-equal changes". Where several co-equal changes share one
   kind (FLUID-0's three gate letters, VIEW-SEARCH-1's two plant parameters), the coder coded
   the kind. That convention was not written here. Under the letter at most two more entries
   would move to SPLIT (G3b's closure statistic beside its missing budget, and MOLSEARCH-1's
   run-on paragraph). The defect rate would stay under `0.03`.
3. **The extractor overlapped three times.** A section block ran on into the next paragraph:
   - MOLSEARCH-1's first clause carries the second clause's paragraph;
   - SATURATION-2's item 3 absorbed the next unnumbered sentence (coded SPLIT);
   - RESPONSE-1 Amendment 5's "What it means" carries the staggered-chart bullet, which is
     also its own entry.

   One change is therefore in the corpus twice. The two readings of it were coded Facts and
   Structure. They sit in one document, so they make no transition with each other, but each
   pairs with the neighbouring documents, so that change carries double weight in two of
   RESPONSE-1's document pairs.
4. **The date order inside the BRIDGE-era campaigns is arbitrary.** Those rows carry their
   registration date (2026-08-28), not their event date, so their documents tie on date and
   fall back to line order. The permutation null is indifferent to any fixed order, so S3 is
   not biased by this. But "succession" means less on those campaigns than on the dated ones.
5. **The git reversal pattern is lexical.** Three of its five hits are words: this prereg's
   own "undone", a physics "restores", and a test named "restored". It cannot see a reversal
   whose message does not announce one. Its one substantive hit (`05a965c`) is a restoration,
   adjudicated in the results.
6. **PL-3's mirror clause was vacuous in the first build.** It filtered the real codes to the
   consistent ones and then asked whether any was inconsistent. It was fixed after the first
   read to test the two consistent mirror codes explicitly. The verdict did not move.
7. **Reported after the first read, labelled there and never graded:** the diagonal against a
   global permutation (it separates campaign composition from succession) and the contested
   boundary pairs.
8. **The builder's prediction was wrong on its specifics.** The commonest sites are Rules and
   Model, not Facts and Rules, and Identity is not rare. The Record rate was `0.77`, not four
   in five.
