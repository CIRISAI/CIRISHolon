#!/usr/bin/env python3
"""RECORD-1 — the 11+1 taxonomy of change on its native carrier: this programme's own
record of corrections, 2026-08-26 .. 2026-09-26.

Subcommands (run from anywhere; every path resolves from this file's location):

  corpus   build RECORD1_CORPUS.jsonl from the declared extraction spec below
           (amendments, notes-on-building / what-the-prereg-got-wrong items, misfit
           occurrences, Nth-instance / CORRECTION lines), dated from the text or from git
  blind    write RECORD1_BLIND.jsonl: corpus + the PL-1 synthetic entries, shuffled, text only
           (ids, files and dates withheld, so a plant cannot be told from a corpus entry;
           the Record flag is judged from the warrant the text itself states) — the input
           the coder receives; the key is committed only with the results
  stats    read RECORD1_CODES.jsonl (written BEFORE this is ever run) and compute S1-S4,
           the plants, the reversal search, and the branch; writes record1_read.txt/.json

stdlib only. No timers. The sibling ontology repository is located by $CIRISONTOLOGY_ROOT
or as a sibling directory of this repository (or of the main checkout of a worktree).
"""
import json, os, pathlib, random, re, subprocess, sys, hashlib, collections

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent.parent
CORPUS = HERE / "RECORD1_CORPUS.jsonl"
BLIND = HERE / "RECORD1_BLIND.jsonl"
BLIND_KEY = HERE / "RECORD1_BLIND_KEY.json"
CODES = HERE / "RECORD1_CODES.jsonl"
READ_TXT = HERE / "record1_read.txt"
READ_JSON = HERE / "record1_read.json"
WINDOW = ("2026-08-26", "2026-09-26")

KINDS = ["Priorities", "Rules", "Manner", "Identity", "Confidence", "Facts",
         "Circumstances", "Process", "Model", "Structure", "Premises"]

# ------------------------------------------------------------------------------------------
# git helpers (dates only; nothing here writes to git)
# ------------------------------------------------------------------------------------------

def git(*args):
    r = subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True, text=True)
    return r.stdout


_blame_cache = {}
def blame_date(rel, lineno):
    key = (rel, lineno)
    if key not in _blame_cache:
        out = git("blame", "--line-porcelain", "-L", f"{lineno},{lineno}", "HEAD", "--", rel)
        m = re.search(r"^author-time (\d+)", out, re.M)
        if m:
            import datetime
            d = datetime.datetime.fromtimestamp(int(m.group(1)), datetime.timezone.utc)
            # the programme's clock is CDT (UTC-5); a UTC date would move late-evening work
            d = d - datetime.timedelta(hours=5)
            _blame_cache[key] = d.strftime("%Y-%m-%d")
        else:
            _blame_cache[key] = None
    return _blame_cache[key]


_add_cache = {}
def added_date(rel):
    if rel not in _add_cache:
        out = git("log", "--diff-filter=A", "--format=%ad", "--date=short", "--", rel).split()
        _add_cache[rel] = out[-1] if out else None
    return _add_cache[rel]


_reg_cache = {}
def registration_date(mid):
    if mid not in _reg_cache:
        out = git("log", "--reverse", "--format=%ad", "--date=short", "-S", f"**{mid}**",
                  "--", "conformance/gravity/MISFITS.md").split()
        _reg_cache[mid] = out[0] if out else None
    return _reg_cache[mid]

# ------------------------------------------------------------------------------------------
# text extraction
# ------------------------------------------------------------------------------------------

_phrase_cache = {}
def phrase_date(rel, phrase):
    if (rel, phrase) not in _phrase_cache:
        out = git("log", "--reverse", "--format=%ad", "--date=short", "-S", phrase, "--", rel).split()
        _phrase_cache[(rel, phrase)] = out[0] if out else None
    return _phrase_cache[(rel, phrase)]


_files = {}
def lines(rel):
    if rel not in _files:
        _files[rel] = (ROOT / rel).read_text().split("\n")
    return _files[rel]


def find(rel, anchor, after=0):
    for i, l in enumerate(lines(rel)):
        if i >= after and anchor in l:
            return i
    raise KeyError(f"anchor not found in {rel}: {anchor!r}")


ITEM = re.compile(r"^(#{1,6} |\d+\. \*\*|\d+\. |\*\*\d+\. |\*\*C\d|\*\*[A-Z][^*]{0,80}\*\*|- \*\*|---\s*$|> \*\*|witness:|\*\*misfits)")

def block(rel, i0, heading=False, cap=2600):
    """From line i0: a section body (heading=True: until the next heading of the same or a
    higher level) or one item (until the next item/heading)."""
    L = lines(rel)
    out = [L[i0]]
    lvl = len(L[i0]) - len(L[i0].lstrip("#")) if L[i0].startswith("#") else None
    for j in range(i0 + 1, len(L)):
        l = L[j]
        if heading and lvl:
            m = re.match(r"^(#{1,6}) ", l)
            if (m and len(m.group(1)) <= lvl) or l.startswith("witness:") or l.startswith("**misfits") \
                    or l.startswith("misfits:") or re.match(r"^---\s*$", l):
                break
        else:
            if ITEM.match(l):
                break
        out.append(l)
    return trim(" ".join(x.strip() for x in out if x.strip()), cap)


REG_IDS = set(re.findall(r"\*\*(M-[A-Z0-9-]+)\*\*", (ROOT / "conformance/gravity/MISFITS.md").read_text()))

def trim(t, cap):
    t = re.sub(r"\s+", " ", t).strip()
    if len(t) > cap:
        t = t[:cap]
        t = t[: t.rfind(" ")] + " [...]"
    # never leave a truncated misfit id behind (CI 10a1 greps the whole conformance tree)
    for m in re.findall(r"\bM-[A-Z0-9-]+\b", t):
        assert m in REG_IDS, f"ghost misfit id in extracted text: {m}"
    return t


def substring(rel, anchor, start, end):
    L = lines(rel)
    i = find(rel, anchor)
    row = L[i]
    a = row.index(start)
    b = row.index(end, a) + len(end)
    return i, trim(row[a:b], 6000)


DATE = re.compile(r"(2026-\d\d-\d\d)")
def stated_date(t):
    d = DATE.findall(t)
    d = [x for x in d if WINDOW[0] <= x <= WINDOW[1]]
    return d[0] if d else None

# ------------------------------------------------------------------------------------------
# THE EXTRACTION SPEC — declared before any coding; committed with the corpus
# ------------------------------------------------------------------------------------------
# Source 1, amendments. Rule: where the amendment enumerates its changes (A-numbered
# sections, a numbered list of changes, or the found-items list), each is one entry; a
# title clause not covered by them is one entry; a section the amendment marks as found
# AFTER its own freeze ("found on building/launching/first walk/first contact",
# "Correction on building", a prepended CORRECTION) is one entry. Sections that state
# what is NOT changed, plants-only sections and restatements are not entries.
W = "conformance/water_observatory/"
AMEND = [
    # (campaign, file, anchor, heading?)
    ("CARRIER_V2", "conformance/atomworld/CARRIER_V2_AMENDMENT_1.md", "**The mistake the freeze made is nameable", False),
    ("CARRIER_V2", "conformance/atomworld/CARRIER_V2_AMENDMENT_1.md", "## A1.2 ARM B", True),
    ("GF1", "conformance/crystal/GF1_AMENDMENT_1.md", "## A1 — the price", True),
    ("GF1", "conformance/crystal/GF1_AMENDMENT_1.md", "## A2 — the non-local magic", True),
    ("GF1", "conformance/crystal/GF1_AMENDMENT_1.md", "## A3 — the variance gate", True),
    ("GF1", "conformance/crystal/GF1_AMENDMENT_1.md", "## The ladder, re-sized", True),
    ("GF1", "conformance/crystal/GF1_AMENDMENT_1.md", "**C1 —", False),
    ("GF1", "conformance/crystal/GF1_AMENDMENT_1.md", "**C2 —", False),
    ("GF1", "conformance/crystal/GF1_AMENDMENT_1.md", "**C3 —", False),
    ("GF1", "conformance/crystal/GF1_AMENDMENT_1.md", "**C4 —", False),
    ("GF1", "conformance/crystal/GF1_AMENDMENT_1.md", "**C5 —", False),
    ("GF2A", "conformance/crystal/GF2A_AMENDMENT_1.md", "## A1.2 The successor instrument", True),
    ("GF2A", "conformance/crystal/GF2A_AMENDMENT_1.md", "## A1.4 G1′", True),
    ("GF2A", "conformance/crystal/GF2A_AMENDMENT_1.md", "1. **The product start is a fixed point", False),
    ("GF2A", "conformance/crystal/GF2A_AMENDMENT_1.md", "2. **A dropped sector never returns", False),
    ("GF2A", "conformance/crystal/GF2A_AMENDMENT_2.md", "## A2.1 The mechanism, named", True),
    ("GF2A", "conformance/crystal/GF2A_AMENDMENT_2.md", "## A2.4 V1", True),
    ("GF2A", "conformance/crystal/GF2A_AMENDMENT_2.md", "## A2.5 R1", True),
    ("GF2A", "conformance/crystal/GF2A_AMENDMENT_2.md", "## A2.6 The ladder, restaked", True),
    ("GF2A", "conformance/crystal/GF2A_AMENDMENT_2.md", "## A2.8 Plants", True),
    ("GF2A", "conformance/crystal/GF2A_AMENDMENT_3.md", "## A3.1 G0‴", True),
    ("FLUID0", "conformance/mesh/FLUID0_AMENDMENT_1.md", "*Frozen 2026-09-06, committed alone", False),
    ("FLUID0", "conformance/mesh/FLUID0_AMENDMENT_1.md", "Gates unchanged in substance, three letters corrected", False),
    ("QVM_ACUITY2", "conformance/qasm/QVM_ACUITY2_AMENDMENT_1.md", "1. **The MPS probe as frozen", False),
    ("QVM_ACUITY2", "conformance/qasm/QVM_ACUITY2_AMENDMENT_1.md", "2. **The certificate is", False),
    ("QVM_ACUITY2", "conformance/qasm/QVM_ACUITY2_AMENDMENT_1.md", "3. **S3's inequality", False),
    ("QVM_ACUITY2", "conformance/qasm/QVM_ACUITY2_AMENDMENT_1.md", "4. **The grid", False),
    ("QVM_ACUITY2", "conformance/qasm/QVM_ACUITY2_AMENDMENT_1.md", "5. **The prices", False),
    ("QVM_ACUITY2", "conformance/qasm/QVM_ACUITY2_AMENDMENT_1.md", "6. **The by-hand runs", False),
    ("QVM_ACUITY2", "conformance/qasm/QVM_ACUITY2_AMENDMENT_1.md", "7. **The campaign seed", False),
    ("QVM_ACUITY2", "conformance/qasm/QVM_ACUITY2_AMENDMENT_1.md", "8. **What is printed", False),
    ("REASON_SEARCH0", "conformance/reasoning/REASON_SEARCH0_AMENDMENT_1.md", "*Written 2026-09-21, before the re-read", False),
    ("REPLACE1", "conformance/replace1/REPLACE1_AMENDMENT_1.md", "1. **Admission is ORDERED", False),
    ("REPLACE1", "conformance/replace1/REPLACE1_AMENDMENT_1.md", "2. **G3 restated", False),
    ("REPLACE1", "conformance/replace1/REPLACE1_AMENDMENT_1.md", "3. **G4's amplitude clause", False),
    ("CT1", W + "CT1_AMENDMENT_1.md", "*Frozen 2026-09-05, committed alone", False),
    ("FIELD3", W + "FIELD3_AMENDMENT_1.md", "- **G-C0 — the price (amended)", False),
    ("FIELD1", W + "FIELD_AMENDMENT_1.md", "## A1.1 The corrected definition", True),
    ("FIELD1", W + "FIELD_AMENDMENT_2.md", "*Frozen 2026-09-04, committed alone, after G1 was RUN", False),
    ("FIELD1", W + "FIELD_AMENDMENT_3.md", "*Frozen 2026-09-04, committed alone. AMENDMENT 2 moved", False),
    ("FIELD1", W + "FIELD_AMENDMENT_3.md", "- **plant (ii), re-staked.**", False),
    ("LIQUID1", W + "LIQUID1_AMENDMENT_1.md", "1. **The nearest-site distance", False),
    ("LIQUID1", W + "LIQUID1_AMENDMENT_1.md", "2. **The orientation measure", False),
    ("LIQUID1", W + "LIQUID1_AMENDMENT_1.md", "3. **The dry run", False),
    ("LIQUID1", W + "LIQUID1_AMENDMENT_1.md", "4. **Two conversions", False),
    ("LIQUID1", W + "LIQUID1_AMENDMENT_2.md", "1. **The switch", False),
    ("LIQUID1", W + "LIQUID1_AMENDMENT_2.md", "2. **The door", False),
    ("LIQUID1", W + "LIQUID1_AMENDMENT_2.md", "3. **The price of the truncation", False),
    ("LIQUID1", W + "LIQUID1_AMENDMENT_2.md", "4. **The derivative gate", False),
    ("LIQUID2", W + "LIQUID2_AMENDMENT_1.md", "## The amendment", True),
    ("LIQUID2", W + "LIQUID2_AMENDMENT_2.md", "## The repair, and why not the obvious one", True),
    ("MOLSEARCH1", W + "MOLSEARCH1_AMENDMENT_1.md", "PM-1's synthetic carrier", False),
    ("MOLSEARCH1", W + "MOLSEARCH1_AMENDMENT_1.md", "What the molecule tier's object actually is", False),
    ("MOLSEARCH1", W + "MOLSEARCH1_AMENDMENT_1.md", "## A2 — the carrier is 128 waters", True),
    ("MOLSEARCH1", W + "MOLSEARCH1_AMENDMENT_1.md", "## A3 — the fine settle", True),
    ("ORDER1", W + "ORDER1_AMENDMENT_1.md", "## What changes", True),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_1.md", "## A1 — the driven quadrature", True),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_1.md", "## A2 — the cycle-aligned average", True),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_1.md", "## A3 — the density mode starts", True),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_1.md", "## A4 — the print", True),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_1.md", "### Correction on building (2026-09-19", True),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_2.md", "**1. R1's grid", False),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_2.md", "**2. `D_cont` does not fall", False),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_2.md", "**3. The continuity read", False),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_2.md", "## A4 — the midpoint law's own floor", True),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_2.md", "## A5 — the window-mean form", True),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_3.md", "## A1 — R4 and R1′", True),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_3.md", "## A2 — R2 fits both forms", True),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_3.md", "## A3 — two more 200 m/s", True),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_3.md", "## A4 — the density mode's baseline", True),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_4.md", "Amendment 2 derived", False),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_5.md", "## What it means", True),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_5.md", "- **The closed chart is the staggered one**", False),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_5.md", "## Staked on the eight arms", True),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_6.md", "## The run", True),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_6.md", "## What is compared, against what", True),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_6.md", "## The kill", True),
    ("RESPONSE1", W + "RESPONSE1_AMENDMENT_6.md", "## Cost, corrected before the run", True),
    ("RUNG2", W + "RUNG2_AMENDMENT_1.md", "### A1 — the density field is binned", True),
    ("RUNG2", W + "RUNG2_AMENDMENT_1.md", "### A2 — the grid has a third axis", True),
    ("RUNG2", W + "RUNG2_AMENDMENT_1.md", "### A3 — the grid ladder is derived", True),
    ("RUNG2", W + "RUNG2_AMENDMENT_2.md", "## The rule, stated once", True),
    ("RUNG2", W + "RUNG2_AMENDMENT_2.md", "> **CORRECTION, 2026-09-17", False),
    ("RUNG2", W + "RUNG2_AMENDMENT_3.md", "## The rule", True),
    ("RUNG2", W + "RUNG2_AMENDMENT_4.md", "## Route (i)", True),
    ("RUNG2", W + "RUNG2_AMENDMENT_4.md", "## Route (ii)", True),
    ("RUNG2", W + "RUNG2_AMENDMENT_5.md", "## The rule", True),
    ("RUNG2", W + "RUNG2_AMENDMENT_6.md", "## The leg", True),
    ("SEAM1", W + "SEAM_AMENDMENT_1.md", "## A1.2 The floor instrument", True),
    ("SEAM1", W + "SEAM_AMENDMENT_1.md", "## A1.3 The successor clause", True),
    ("SLOW1", W + "SLOW1_AMENDMENT_1.md", "## The dictionary, amended", True),
    ("VIEW_SEARCH", W + "VIEW_SEARCH_AMENDMENT_1.md", "1. **S4's premise was wrong", False),
    ("VIEW_SEARCH", W + "VIEW_SEARCH_AMENDMENT_1.md", "2. **S2 cannot be read", False),
    ("VIEW_SEARCH", W + "VIEW_SEARCH_AMENDMENT_1.md", "3. **The held-out score", False),
    ("VIEW_SEARCH", W + "VIEW_SEARCH_AMENDMENT_1.md", "4. **Features are standardised", False),
    ("VIEW_SEARCH", W + "VIEW_SEARCH_AMENDMENT_1.md", "5. **Per-seed centring", False),
    ("VIEW_SEARCH", W + "VIEW_SEARCH_AMENDMENT_1.md", "6. **PV-1 at", False),
]

# Source 2, numbered items under "Notes on building" / "What the prereg (freeze, first
# version, campaign) got wrong" in *_RESULTS.md and *_PREREG.md. Dedup rule: where a RESULTS
# list restates the PREREG's notes, the RESULTS list is canonical and PREREG items it does
# not cover are added; where an item restates an amendment's change, the amendment is
# canonical. A matching section with no numbered items is one entry per bold-led paragraph
# that states a correction (SATURATION-3, MIXTURES-1). Dropped duplicates are listed in
# DUPS with the canonical id they fold into.
N = [
    ("MIXTURES1", "conformance/atomworld/MIXTURES1_RESULTS.md", "I named the wrong commit", False),
    ("SATURATION1", "conformance/atomworld/SATURATION1_RESULTS.md", "1. **The envelope was measuring a kink", False),
    ("SATURATION1", "conformance/atomworld/SATURATION1_RESULTS.md", "2. **The count was a worst case", False),
    ("SATURATION2", "conformance/atomworld/SATURATION2_RESULTS.md", "1. **`R_HI = 14`", False),
    ("SATURATION2", "conformance/atomworld/SATURATION2_RESULTS.md", "2. **The tail is not dispersion", False),
    ("SATURATION2", "conformance/atomworld/SATURATION2_RESULTS.md", "3. **A conservation gate passed", False),
    ("SATURATION3", "conformance/atomworld/SATURATION3_RESULTS.md", "**A CORRECTION TO THIS GATE'S FIRST VERSION", False),
    ("SATURATION3", "conformance/atomworld/SATURATION3_RESULTS.md", "**(Cl,Cl,Cl) is over its stated class", False),
]
MP = "conformance/bigqvm/MESH_CLIFFORD_PREREG.md"
MR = "conformance/bigqvm/MESH_CLIFFORD_RESULTS.md"
for a in ["1. **The boundary is snapped", "2. **§1's \"single-qubit gates", "3. **Gates have to be BUFFERED",
          "4. **G2 is KILLED", "5. **The rowsum split", "6. **One direction of the transpose", "7. **Sizes run"]:
    N.append(("MESH_CLIFFORD", MP, a, False))
for a in ["1. **G1 as written cannot catch", "2. **P-cores 8–15", "3. **G3 compares", "4. **G4 says",
          "5. **§5 \"Cost", "6. **G3 names no seed", "7. **G4 assumes", "8. **A record taken", "9. **§4 of the freeze"]:
    N.append(("MESH_CLIFFORD", MR, a, False))
A1P = "conformance/qasm/QVM_ACUITY1_PREREG.md"
for a in ["1. **§3's \"the sum", "2. **The `ε` ladder", "3. **S4's `S = 8`", "4. **S5 does not say",
          "5. **PQ-4 as written", "6. **S2's kill has"]:
    N.append(("QVM_ACUITY1", A1P, a, False))
for a in ["**1. An amplitude's light cone", "**2. The prereg's marginal", "**3. The light cone is SOUND",
          "**4. `y = 0^n`", "**5. \"Equals the tableau", "**6. `holon-qasm` is a DEV", "**7. A marginal is not priced",
          "**8. Where the cheap part's"]:
    N.append(("QVM_ACUITY1", A1P, a, False))
A2P = "conformance/qasm/QVM_ACUITY2_PREREG.md"
for a in ["3. **The label for family T", "4. **Family L cannot show decay", "5. **The prices omit the search",
          "6. **§1's MPS price"]:
    N.append(("QVM_ACUITY2", A2P, a, False))
GP = "conformance/qasm/QVM_GPUFOLD1_RESULTS.md"
for a in ["1. **\"Cold\" lumped", "2. **G1 never contacted", "3. **G1's every-`k`", "4. **PG-2's carrier",
          "5. **The CPU arm is contended", "6. **The round trip", "7. **`holon` now has a runtime"]:
    N.append(("QVM_GPUFOLD1", GP, a, False))
RP = "conformance/rank/QVM_RANK1_PREREG.md"
for a in ["1. **The ring in §1", "2. **\"Project and test", "3. **The Galois filter", "4. **The dual-distance filter",
          "5. **Five-tuple coverage", "6. **\"The pipeline\" at m = 6", "7. **Every member line", "8. **The exhaustive branch",
          "9. **The device carries", "10. **A seven-copy read", "11. **Mid-run reallocation"]:
    N.append(("QVM_RANK1", RP, a, False))
OR_ = "conformance/replace1/OBJECTIVE1_RESULTS.md"
for a in ["1. **PI-3's amplitude", "2. **§4 missed half", "3. **The definedness rule", "4. **Q4's order clause",
          "5. **The lag axis", "6. **PI-2 is read"]:
    N.append(("OBJECTIVE1", OR_, a, False))
RR = "conformance/replace1/REPLACE1_RESULTS.md"
for a in ["1. **G2's network block", "2. **G3 names no lag", "3. **G3's \"others kept\"", "4. **G3b stakes",
          "5. **G4's \"every kept gate", "6. **§0's \"the cell chart", "7. The stakes read the tier's admission"]:
    N.append(("REPLACE1", RR, a, False))
O1R = W + "ORDER1_RESULTS.md"
for a in ["2. **O1's `σ₁ ≥ 0.3`", "3. **O1's residual-ratio leg", "4. **O3's two-time test", "5. **O2's 400 K clause",
          "6. **The lead's literal dictionary", "7. **The heat mode"]:
    N.append(("ORDER1", O1R, a, False))
S2R = W + "SLOW2_RESULTS.md"
for a in ["1. **SPIB's convergence settings", "2. **F at λ = 10", "3. **The branch rule", "4. **Seen before the full run",
          "5. **\"Own σ₁\" of a 2-D output"]:
    N.append(("SLOW2", S2R, a, False))
N.append(("SLOW2", W + "SLOW2_PREREG.md", "3. **SPIB's input normalisation", False))
S3P = W + "SLOW3_PREREG.md"
for a in ["1. **The binary.**", "2. **A 1-epoch, 1-seed code-path smoke", "3. **The selection (",
          "4. **Said before the read", "5. **Frozen.**"]:
    N.append(("SLOW3", S3P, a, False))

# literal items quoted inline in a RESULTS list, not present in the prereg's notes
NT = [
    ("QVM_RANK1", "conformance/rank/QVM_RANK1_RESULTS.md", "Items 1–11 under the prereg's",
     "(9) stabrank's `m = 7` upper bound is 9 (KvWV), not 7 — a witness settles `χ = 6` outright; "
     "The lead's error to own: the prereg was frozen without reading stabrank's `bounds/` and `research/` first (item 9)."),
    ("QVM_RANK1", "conformance/rank/QVM_RANK1_RESULTS.md", "Items 1–11 under the prereg's",
     "(11) the run ended at 30.4 h of a planned 40 by the harness's memory reaper."),
]

# Source 3a, misfit OCCURRENCES in MISFITS.md: one entry per occurrence the row names with
# enough text to be coded; an occurrence already in the corpus from sources 1-2 folds into
# that entry (DUPS). (campaign, id, start phrase or None for the whole row, end phrase)
MF = "conformance/gravity/MISFITS.md"
MIS = [
    ("BRIDGE1", "M-GAUGE-LAUNDER", None, None, None),
    ("BRIDGE1", "M-PARITY-PROTECT", None, None, None),
    ("BRIDGE1", "M-LOOP-BLIND", None, None, None),
    ("BRIDGE0_V2", "M-PLANT-OBS", "observability is instrument-relative", "ON THE CARRIER OF RECORD", "occurrence: BRIDGE0-V2"),
    ("BRIDGE5", "M-PLANT-OBS", "observability is instrument-relative", "ON THE CARRIER OF RECORD", "occurrence: BRIDGE-5"),
    ("BRIDGE6", "M-PLANT-OBS", "observability is instrument-relative", "ON THE CARRIER OF RECORD", "occurrence: BRIDGE-6 (twice; the row gives no text that separates the two)"),
    ("REPLACE1", "M-BAR-FROM-THE-READ", "a stake's bar set from the very reading", "never from the read", "occurrence: REPLACE-1 Amendment 1's bars"),
    ("BRIDGE6", "M-PLANT-SECTOR", None, None, None),
    ("BRIDGE5", "M-BARE-CHARGE", None, None, None),
    ("BRIDGE3", "M-HOMOG", None, None, "occurrence: BRIDGE-3 (founding)"),
    ("BRIDGE6", "M-HOMOG", None, None, "occurrence: BRIDGE-6 (confirmed)"),
    ("SCHWINGER2", "M-VOLUME-SCALE", None, None, None),
    ("BRIDGE3", "M-COND-PROBE", None, None, None),
    ("BRIDGE2", "M-ONE-MODEL-DELTA", None, None, None),
    ("BRIDGE7B", "M-KINEMATIC-NONLOCAL", None, None, None),
    ("WILSON1", "M-ELECTRIC-BASIS", None, None, None),
    ("WILSON2", "M-NULL-MISSTAKE", None, None, None),
    ("LOCAL1B", "M-PROBE-EIGENSTATE", None, None, None),
    ("LOCAL1C", "M-RING-MIXING", None, None, None),
    ("CLOSURE2", "M-GAUGE-UNIFORM-MOMENTUM", None, None, None),
    ("CI_RECORD", "M-STALE-INSTRUMENT", "a results document without its instrument's commit", "fresh-runs the fast ones.", None),
    ("CI_RECORD", "M-STALE-INSTRUMENT", "WIDENED 2026-09-01, the inverted variant", "session-keyed paths.", None),
    ("SATURATION3", "M-STALE-INSTRUMENT", "SECOND VARIANT, 2026-09-01", "converting a dead citation into a re-run result.", None),
    ("CI_RECORD", "M-STALE-INSTRUMENT", "THIRD VARIANT, 2026-09-01, at file level", "the founding case is this lane's error against their file", None),
    ("MIXTURES1", "M-STALE-INSTRUMENT", "CASE LAW from the same interval", "I 27 moved).", None),
    ("EINSTEIN_ADM1", "M-FIXED-POINT-TRAJECTORY", None, None, None),
    ("CLOSURE3", "M-FINAL-VIEW-COLLISIONS", None, None, None),
    ("CLOSURE3", "M-NONBIJECTIVE-STEP", None, None, None),
    ("SELECTOR5", "M-TAG-AS-PROPERTY", None, None, None),
    ("SELECTOR5", "M-BASE-RATE-OMITTED", None, None, None),
    ("SELECTOR5", "M-PRESENTATION-VERDICT", None, None, None),
    ("SELECTOR5", "M-POPULATION-CHOICE", None, None, None),
    ("SELECTOR5", "M-CONJUNCTION-MONOTONE", None, None, None),
    ("ELEMENTS3", "M-SORTS-NOT-SEPARATES", None, None, None),
    ("ELEMENTS3", "M-EXIT-DISCRIMINATOR", None, None, None),
    ("MPS_LADDER", "M-MAX-OVER-SUCCESSES", None, None, None),
    ("SELECTOR6", "M-BUDGET-LAUNDER", None, None, None),
    ("ELEMENTS3", "M-UNTESTED-GAP", None, None, None),
    ("FDC_1", "M-FOREIGN-DOMAIN-CORROBORATION", "a passing result in a different domain", "masks;", "occurrence 1 of 4"),
    ("FDC_2", "M-FOREIGN-DOMAIN-CORROBORATION", "a passing result in a different domain", "DMRG\");", "occurrence 2 of 4"),
    ("FDC_3", "M-FOREIGN-DOMAIN-CORROBORATION", "a passing result in a different domain", "not verified,", "occurrence 3 of 4"),
    ("FDC_4", "M-FOREIGN-DOMAIN-CORROBORATION", "a passing result in a different domain", "second source", "occurrence 4 of 4"),
    ("SELECTOR6", "M-IMPORT-EXECUTES", None, None, None),
    ("VS_1", "M-VACUOUS-SUCCESS", "a verifier must assert its WORK COUNT", "(wrong invocation directory),", "occurrence: zero checks under an ALL CHECKS PASSED banner"),
    ("VS_3", "M-VACUOUS-SUCCESS", "a verifier must assert its WORK COUNT", "not done, differing only by where it ran", "occurrence: a guard on an export that never existed"),
    ("ELEMENTS1", "M-CACHE-KIND", None, None, None),
    ("SATURATION3", "M-DEVICE-CLASS", None, None, None),
    ("RESOURCE1", "M-IDLE-CALIBRATED-TIMEOUT", "RESOURCE-1's reaper is the first:", "(0 after the grace was made self-calibrating).", None),
    ("BIGQVM", "M-IDLE-CALIBRATED-TIMEOUT", "bigqvm's quiet-window gate is the second:", "keeps the rejected d=45 gate as the transferable failure).", None),
    ("BIGQVM", "M-IDLE-CALIBRATED-TIMEOUT", "THE DEAD MECHANISMS, KEPT VISIBLE", "(4.30x vs 4.22x).", None),
    ("RESOURCE1", "M-PROBE-THE-RESOURCE", None, None, None),
    ("SIX_BIRDS", "M-MAINTENANCE-LENS", "a defect metric computed on a lens", "mechanism confirmed from source at cf0ec753.", None),
    ("MAINTENANCE_SWEEP", "M-MAINTENANCE-LENS", "CORROBORATION, and it is the strongest kind:", "(`MAINTENANCE_SWEEP_RESULTS.md:336`).", None),
    ("ELEMENTS3", "M-PROVENANCE-OVERREACH", None, None, None),
    ("BIGQVM", "M-PLACEMENT-LOTTERY", "Measured 2026-08-30 with both arms pinned", "so the minimum was never a floor.", None),
    ("BIGQVM", "M-PLACEMENT-LOTTERY", "Remedy, CORRECTED TWICE (2026-09-01)", "one bare row shows nothing.", None),
    ("BIGQVM", "M-PLACEMENT-LOTTERY", "And **quiet and pinned are NOT interchangeable**", "quiet does the work pinning cannot.", None),
    ("BIGQVM", "M-PLACEMENT-LOTTERY", "but pinning to a P-core does NOT define a reproducible condition", "repeat across cores and sessions.", None),
    ("BIGQVM", "M-PLACEMENT-LOTTERY", "THIRD RUNG, measured 2026-08-31", "which is what to size runs against.", None),
    ("BIGQVM", "M-PLACEMENT-LOTTERY", "DISCIPLINE THIS ROW LEARNED ABOUT ITSELF", "so remedies get measured before they ship as advice.", None),
    ("T3_ENGINE_CORE", "M-PLACEMENT-LOTTERY", "FOURTH INSTANCE, 2026-09-01 (T3 engine-core)", "so correctness landed and the speedup did not", None),
    ("OZONE_P2", "M-CHEAPER-THAN-ITS-PRICE", None, None, None),
    ("GF2A", "M-TRUNCATION-AS-ERRORBAR", None, None, None),
    ("EMBED2", "M-FORMAT-FLOOR", None, None, None),
    ("FIELD2", "M-EMPTY-SECTOR", None, None, None),
    ("FIELD7", "M-EXTRAPOLATED-HOLE", None, None, None),
    ("CT1", "M-FIRST-VIOLATION-ONLY", None, None, None),
    ("CROSSFACE1", "M-INEQUALITY-READ-AS-EQUALITY", None, None, None),
    ("OBJECT", "M-BOUND-HYPOTHESIS-UNMET", None, None, None),
    ("COMPARE0", "M-RIGID-REFERENCE-PLACEMENT", None, None, None),
    ("LIQUID2", "M-VALIDATED-NOT-WIRED", None, None, None),
]

# Source 3b, "the Nth instance" / CORRECTION lines in GANTT2.md, OBJECT.md, STANCE.md not
# already in the corpus. (campaign, file, first line, last line, a phrase that must occur, date)
DOC = [
    ("RESPONSE1", "GANTT2.md", 1006, 1008, "the eleventh instance", None),
    ("VIEW_SEARCH", "GANTT2.md", 927, 931, "the fifteenth", None),
    ("REASON_SEARCH0", "GANTT2.md", 917, 919, "seventeenth and eighteenth", None),
    ("REPLACE0", "GANTT2.md", 1264, 1295, "CORRECTION (2026-09-13", None),
    ("OBJECT", "OBJECT.md", 45, 56, "Corrected: a tier is a Closed view", "2026-09-25"),
    ("OBJECT", "OBJECT.md", 57, 63, "Corrected: the object is found", "2026-09-25"),
    ("OBJECT", "OBJECT.md", 64, 68, "Corrected: PRICE for", "2026-09-25"),
    ("OBJECT", "OBJECT.md", 69, 87, "Corrected table", "2026-09-25"),
    ("OBJECT", "OBJECT.md", 88, 100, "Move 4's join was MEASURED", "2026-09-25"),
]

# Folded duplicates: (where, the text's handle) -> canonical source handle. Listed so the
# fold is auditable; nothing here is coded.
DUPS = [
    ("GANTT2.md:1047 (CORRECTION, GF1, two errors) and :1061 (the eighth instance)", "GF1_AMENDMENT_1 A1 and A2"),
    ("GANTT2.md:1063 (the ninth)", "GF1_AMENDMENT_1 C1"),
    ("GANTT2.md:1017 (the tenth instance)", "RESPONSE1_AMENDMENT_2 found item 2"),
    ("GANTT2.md:981 / STANCE.md:306 (the twelfth instance)", "VIEW_SEARCH_AMENDMENT_1 item 1"),
    ("STANCE.md:302 / OBJECT.md:452 (the fifteenth instance)", "GANTT2.md:927 entry"),
    ("GANTT2.md:918 eighteenth instance (the parent link one field away)", "REASON_SEARCH0_AMENDMENT_1"),
    ("GANTT2.md:890 / STANCE.md:274 (the nineteenth instance)", "RESPONSE1_AMENDMENT_5 'What it means'"),
    ("STANCE.md:211 (twenty-second and twenty-third instances)", "QVM_ACUITY2_PREREG notes 3 and 4"),
    ("STANCE.md:482 (M-BAR-FROM-THE-READ third instance)", "MISFITS M-BAR-FROM-THE-READ REPLACE-1 occurrence"),
    ("STANCE.md:484 (Corrected 2026-09-25)", "OBJECT.md CORRECTION items 1-4"),
    ("MISFITS M-PLANT-OBS ORDER-1 occurrence / ORDER1_RESULTS item 1 / ORDER1_PREREG bullet PO-4", "ORDER1_AMENDMENT_1"),
    ("MISFITS M-BAR-FROM-THE-READ ACUITY-2 occurrence", "QVM_ACUITY2_PREREG note 3"),
    ("MISFITS M-BAR-FROM-THE-READ ORDER-1 occurrence", "ORDER1_RESULTS item 2"),
    ("MISFITS M-JOINT-PASS-REGION (ORDER-1)", "ORDER1_RESULTS item 3"),
    ("MISFITS M-STALE-INSTRUMENT 'baseline wearing a control's clothes'", "MIXTURES1_RESULTS 'The attribution I got wrong first'"),
    ("MISFITS M-VACUOUS-SUCCESS 'conservation gate on a scene whose atoms never moved'", "SATURATION2_RESULTS item 3"),
    ("MISFITS M-FLOOR-UNSTAKED (SEAM-1)", "SEAM_AMENDMENT_1"),
    ("MISFITS M-BAND-FROM-A-SUPPRESSED-SCATTER", "LIQUID2_AMENDMENT_1"),
    ("MISFITS M-BAR-AGAINST-A-WORKING-THERMOSTAT", "LIQUID2_AMENDMENT_2"),
    ("QVM_ACUITY1_PREREG cheap-part note 9 (S5's exponent is TWO numbers)", "QVM_ACUITY1_PREREG hard-part note 4"),
    ("QVM_ACUITY1_PREREG cheap-part unnumbered paragraph (the ε ladder)", "QVM_ACUITY1_PREREG hard-part note 2"),
    ("QVM_ACUITY2_PREREG notes 1, 2, 7, 8", "QVM_ACUITY2_AMENDMENT_1 items 1, 2, 3, 6"),
    ("QVM_RANK1_RESULTS (1)-(8), (10)", "QVM_RANK1_PREREG notes 1-6, 9-11"),
    ("OBJECTIVE1_PREREG notes 1-3", "OBJECTIVE1_RESULTS items 1-3"),
    ("REPLACE1_PREREG notes 1-5", "REPLACE1_RESULTS items 1-6"),
    ("SLOW2_PREREG notes 1, 2, 4-8", "SLOW2_RESULTS items 1-5"),
]


def build_corpus():
    rows = []

    def add(src, campaign, rel, lineno, text, date, basis, extra=None):
        rows.append(dict(src=src, campaign=campaign, file=rel, line=lineno + 1, text=text,
                         date=date, date_basis=basis, **(extra or {})))

    def dated(rel, i, text, heading_text=None):
        d = stated_date(text[:400])
        if d:
            return d, "stated"
        if heading_text:
            d = stated_date(heading_text)
            if d:
                return d, "stated-in-section"
        d = blame_date(rel, i + 1)
        return d, "git-blame"

    for camp, rel, anchor, heading in AMEND:
        i = find(rel, anchor)
        t = block(rel, i, heading=heading)
        title = lines(rel)[0].lstrip("# ").strip()
        # date: the enclosing section heading's date (a late-found section dates itself),
        # else the amendment's own "Frozen/Written YYYY-MM-DD" in its opening, else git add
        sec = next((lines(rel)[j] for j in range(i, -1, -1)
                    if lines(rel)[j].startswith("#") or lines(rel)[j].startswith("> **CORRECTION")), "")
        opening = " ".join(lines(rel)[1:10])
        m = re.search(r"(?:Frozen|Written)\s+(2026-\d\d-\d\d)", opening)
        if stated_date(sec):
            d, basis = stated_date(sec), "stated-in-section"
        elif m:
            d, basis = m.group(1), "stated-in-opening"
        else:
            d, basis = added_date(rel), "git-add"
        add("amendment", camp, rel, i, t, d, basis, {"doc_title": title})
    for camp, rel, anchor, heading in N:
        i = find(rel, anchor)
        t = block(rel, i, heading=heading)
        # the enclosing notes section's heading date, if the section states one
        sec = None
        for j in range(i, -1, -1):
            if lines(rel)[j].startswith("#") or lines(rel)[j].startswith("**After the read"):
                sec = lines(rel)[j]
                break
        d, basis = dated(rel, i, t, sec)
        add("notes", camp, rel, i, t, d, basis)
    for camp, rel, anchor, text in NT:
        i = find(rel, anchor)
        add("notes", camp, rel, i, trim(text, 3000), blame_date(rel, i + 1), "git-blame")
    for camp, mid, a, b, tag in MIS:
        i = find(MF, f"**{mid}**")
        if a is None:
            t = trim(lines(MF)[i], 6000)
        else:
            _, t = substring(MF, f"**{mid}**", a, b)
        if tag:
            t = f"[{mid}; {tag}] " + t
        else:
            t = f"[{mid}] " + t
        if stated_date(t):
            d, basis = stated_date(t), "stated"
        elif a is not None and not tag:
            d, basis = phrase_date(MF, a), "git-first-commit-of-phrase"
        else:
            d, basis = registration_date(mid), "registration"
        add("misfit", camp, MF, i, t, d, basis, {"misfit": mid})
    for camp, rel, a, b, must, when in DOC:
        seg = " ".join(x.strip() for x in lines(rel)[a - 1:b])
        assert must in seg, (rel, a, must)
        t = trim(seg, 4000)
        # the section's date (a GANTT2 section heading, or the CORRECTION's own date)
        sec = next((lines(rel)[j] for j in range(a - 1, -1, -1) if lines(rel)[j].startswith("#")), "")
        if when:
            d, basis = when, "stated-in-section"
        elif stated_date(sec):
            d, basis = stated_date(sec), "stated-in-section"
        else:
            d, basis = blame_date(rel, a), "git-blame"
        add("doc", camp, rel, a - 1, t, d, basis)

    # window and ids
    keep = []
    dropped = []
    for r in rows:
        if r["date"] and WINDOW[0] <= r["date"] <= WINDOW[1]:
            keep.append(r)
        else:
            dropped.append(r)
    pre = {"amendment": "A", "notes": "N", "misfit": "M", "doc": "D"}
    count = collections.Counter()
    for r in keep:
        count[r["src"]] += 1
        r["id"] = f"R1-{pre[r['src']]}{count[r['src']]:03d}"
    # document key for S3: amendments and notes are grouped by (file, date); a misfit
    # occurrence and a doc line are each their own document
    for r in keep:
        r["doc"] = f"{r['file']}@{r['date']}" if r["src"] in ("amendment", "notes") else r["id"]
    with open(CORPUS, "w") as f:
        for r in keep:
            f.write(json.dumps({k: r[k] for k in ["id", "src", "campaign", "date", "date_basis", "file",
                                                   "line", "doc", "text"] + [k for k in ("misfit", "doc_title") if k in r]},
                               ensure_ascii=False) + "\n")
    print(f"corpus: {len(keep)} entries written to {CORPUS.relative_to(ROOT)}; "
          f"{len(dropped)} outside the window {WINDOW}")
    for r in dropped:
        print(f"  outside window: {r['file']}:{r['line']} {r['date']} {r['text'][:80]}")
    print(" by source:", dict(count))
    print(" duplicates folded (not entries):", len(DUPS))
    return keep

# ------------------------------------------------------------------------------------------
# plants PL-1 (22 synthetic entries, two per kind) and PL-3 (Record-inconsistent entries)
# ------------------------------------------------------------------------------------------
PL1 = [
    ("Priorities", "The freeze listed three readouts with no order; it now names the momentum field as the primary reading and the density field as secondary, and the verdict is read on the primary first. No readout, bar or instrument changes."),
    ("Priorities", "Where the wall-clock and the CPU-time columns disagree, the record now quotes CPU time first and the wall beside it; both numbers were always printed and neither changes."),
    ("Rules", "The gate's pass bar is moved from a ratio of 0.5 to a ratio of 0.3: a reading now passes only if the ratio is at most 0.3."),
    ("Rules", "A plant that does not fire now VOIDs the arms it gates; before, the arms were read regardless of the plant."),
    ("Manner", "The table prints amplitudes in scientific notation instead of two fixed decimals; the numbers themselves are unchanged."),
    ("Manner", "The section heading 'Findings' is renamed 'What was read' and the paragraph is rewritten in the passive voice; no claim moves."),
    ("Identity", "'Converged' is redefined: it now means the solver's own exit reason is converged, not that the residual is under the publication bar."),
    ("Identity", "The unit the campaign calls 'a molecule' is now the bonded triple the engine's census emits, not any three atoms within a cutoff."),
    ("Confidence", "The claim that the arms differ is downgraded from a finding to 'not resolved at three seeds'; the measured means are unchanged."),
    ("Confidence", "The error bar on the reported slope is widened from one standard error to the seed-to-seed spread; the slope itself is unchanged."),
    ("Facts", "The reported diffusion coefficient was 5.4e-10 m^2/s; the correct value from the same run is 6.5e-10 m^2/s."),
    ("Facts", "The document said the run used 32 threads; it used 24."),
    ("Circumstances", "The arm on core 18 was interrupted when the host rebooted overnight and was restarted from its checkpoint; nothing about the protocol changes."),
    ("Circumstances", "The smoke run happened to use seed 7 because the default was not overridden; the campaign seed is unaffected."),
    ("Process", "The plants now run before the arms instead of after them."),
    ("Process", "The amendment is committed alone before the re-read, and the reader is changed only after that commit."),
    ("Model", "The noise floor is now computed with the two-sided formula (observation and prediction variances) instead of the one-sided formula."),
    ("Model", "The fit uses a two-exponential relaxation form instead of a single exponential to extract the rate."),
    ("Structure", "The result file is split from one JSON object into one JSON line per arm, with the same fields."),
    ("Structure", "The engine function is moved from the lens crate into the closure crate, and the lens crate now depends on it."),
    ("Premises", "The design assumed the box was at equilibrium when the reads began; that assumption is dropped, and every read now composes over a relaxed-state check."),
    ("Premises", "The protocol took the thermostat's scatter as independent of the thermostat; that is no longer taken as given."),
]
PL3 = [
    # a Record=TRUE claim whose only warrant is artifact-internal: the rule must refuse it
    ("PL3-a", True, "internal", "The prereg's section 2 states eight cores and section 5 states seven; the count in section 2 is corrected to seven. [coder asserts Record=TRUE, warrant: the prereg's own section 5]"),
    # and the mirror: a Record=FALSE claim whose warrant is outside the artifact
    ("PL3-b", False, "external", "The bar is moved because the first full arm read 0.735 against it. [coder asserts Record=FALSE, warrant: the arm's results file]"),
]


def build_blind():
    corpus = [json.loads(l) for l in open(CORPUS)]
    items = [dict(src_id=r["id"], campaign=r["campaign"], date=r["date"], file=r["file"],
                  text=r["text"], kind="corpus") for r in corpus]
    for n, (k, t) in enumerate(PL1):
        items.append(dict(src_id=f"PL1-{n+1:02d}", campaign="SYNTHETIC", date=None, file="(synthetic)",
                          text=t, kind="PL1"))
    rng = random.Random(20260926)
    rng.shuffle(items)
    key = {}
    with open(BLIND, "w") as f:
        for n, it in enumerate(items):
            bid = f"B{n+1:03d}"
            key[bid] = it["src_id"]
            f.write(json.dumps(dict(bid=bid, text=it["text"]),
                               ensure_ascii=False) + "\n")
    BLIND_KEY.write_text(json.dumps(key, indent=0))
    print(f"blind: {len(items)} items ({len(corpus)} corpus + {len(PL1)} PL-1) -> {BLIND.relative_to(ROOT)}")


# ------------------------------------------------------------------------------------------
# STATISTICS — written after the freeze (8975be1) and after the codes (60899f6)
# ------------------------------------------------------------------------------------------
N_DRAWS = 10_000
SEED_NULL = 20260926
SEED_PL2 = 11

STACK = {"Premises", "Model", "Facts", "Confidence"}
CARRIER = {"Structure", "Manner", "Circumstances"}


def operational_overlay():
    """LEG A section 4.3: cells FORBIDDEN under some channel and ALLOWED under none (71)."""
    cells = set()
    for i in KINDS:
        for j in KINDS:
            if i == j:
                continue
            if i in STACK and j not in STACK:
                cells.add((i, j))                       # grounding never leaves the stack (28)
            if i in CARRIER or j in CARRIER:
                cells.add((i, j))                       # carrier inert under mention (54)
            if j == "Identity":
                cells.add((i, j))                       # Identity's absorption column (10)
    assert len(cells) == 73, len(cells)
    cells -= {("Structure", "Manner"), ("Circumstances", "Manner")}  # absorption allows these
    return cells


OVERLAY = operational_overlay()
# the successional register (TRANSITION_MAP_PREREG NOTE A1.1): Record-axis cells only; no
# kind -> kind cell is among them, so on a kind sequence this set is empty by construction
SUCCESSIONAL_FORBIDDEN = set()


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def load():
    corpus = {json.loads(l)["id"]: json.loads(l) for l in open(CORPUS)}
    key = json.loads(BLIND_KEY.read_text())
    codes = [json.loads(l) for l in open(CODES)]
    assert len(codes) == len(key) == len(corpus) + len(PL1), (len(codes), len(key), len(corpus))
    by_src = {}
    for c in codes:
        by_src[key[c["bid"]]] = c
    return corpus, key, by_src


def coded_one_kind(c):
    return c["status"] == "CHANGE" and c["kind"] in KINDS and c["separable"]


def campaign_docs(corpus, kinds_of):
    """campaign -> ordered list of documents -> list of kinds of included entries."""
    docs = collections.defaultdict(lambda: collections.defaultdict(list))
    order = {}
    for eid, r in corpus.items():
        k = kinds_of.get(eid)
        if k is None:
            continue
        docs[r["campaign"]][r["doc"]].append(k)
        key = (r["date"], r["file"], r["line"])
        order[(r["campaign"], r["doc"])] = min(order.get((r["campaign"], r["doc"]), key), key)
    out = {}
    for camp, dd in docs.items():
        seq = sorted(dd, key=lambda d: order[(camp, d)])
        out[camp] = [dd[d] for d in seq]
    return out


def transitions(seqs):
    T = collections.Counter()
    for camp, docs in seqs.items():
        for a, b in zip(docs, docs[1:]):
            for i in a:
                for j in b:
                    T[(i, j)] += 1
    return T


def entry_transitions(corpus, kinds_of):
    seq = collections.defaultdict(list)
    for eid, r in sorted(corpus.items(), key=lambda kv: (kv[1]["date"], kv[1]["file"], kv[1]["line"])):
        k = kinds_of.get(eid)
        if k is not None:
            seq[r["campaign"]].append(k)
    T = collections.Counter()
    for camp, ks in seq.items():
        for a, b in zip(ks, ks[1:]):
            T[(a, b)] += 1
    return T, seq


def diag(T):
    return sum(v for (i, j), v in T.items() if i == j)


def occ(T, cells):
    return sum(v for c, v in T.items() if c in cells)


def permutation_null(seqs, draws, seed):
    """Kinds permuted across the entries of each campaign; every document keeps its size."""
    rng = random.Random(seed)
    Ds, Os = [], []
    flat = {c: [k for d in docs for k in d] for c, docs in seqs.items()}
    sizes = {c: [len(d) for d in docs] for c, docs in seqs.items()}
    for _ in range(draws):
        new = {}
        for c in seqs:
            ks = flat[c][:]
            rng.shuffle(ks)
            docs, at = [], 0
            for s in sizes[c]:
                docs.append(ks[at:at + s])
                at += s
            new[c] = docs
        T = transitions(new)
        Ds.append(diag(T))
        Os.append(occ(T, OVERLAY))
    return Ds, Os


def entry_null(seq, draws, seed):
    rng = random.Random(seed)
    Ds = []
    for _ in range(draws):
        d = 0
        for ks in seq.values():
            ks = ks[:]
            rng.shuffle(ks)
            d += sum(1 for a, b in zip(ks, ks[1:]) if a == b)
        Ds.append(d)
    return Ds


def mean_sd(xs):
    m = sum(xs) / len(xs)
    v = sum((x - m) ** 2 for x in xs) / (len(xs) - 1)
    return m, v ** 0.5


def s3_read(seqs):
    T = transitions(seqs)
    D = diag(T)
    O = occ(T, OVERLAY)
    F = occ(T, SUCCESSIONAL_FORBIDDEN)
    Ds, Os = permutation_null(seqs, N_DRAWS, SEED_NULL)
    mD, sD = mean_sd(Ds)
    mO, sO = mean_sd(Os)
    p = (1 + sum(1 for x in Ds if x >= D)) / (1 + len(Ds))
    total = sum(T.values())
    eff = sum(1 for docs in seqs.values() if len(docs) >= 2)
    if F > 0:
        verdict = "KILLED (forbidden cell occupied)"
    elif D <= mD:
        verdict = "KILLED (diagonal at or below the null mean)"
    elif p <= 0.05:
        verdict = "MET"
    else:
        verdict = "UNRESOLVED"
    return dict(T=T, D=D, total=total, O=O, F=F, null_D=(mD, sD), null_O=(mO, sO), p=p,
                eff_campaigns=eff, verdict=verdict)


def git_reversal_search():
    out = git("log", "--since=2026-08-26", "--until=2026-09-27", "--format=%H%x1f%ad%x1f%s%x1f%b%x1e", "--date=short")
    pat = re.compile(r"revert|restor|reinstat|undo|back out|roll ?back|un-?retract", re.I)
    hits = []
    for rec in out.split("\x1e"):
        f = rec.strip("\n").split("\x1f")
        if len(f) < 4:
            continue
        h, d, subj, body = f[0].strip(), f[1], f[2], f[3]
        if pat.search(subj) or "This reverts commit" in body:
            hits.append((h[:7], d, subj))
    return hits


def stats():
    corpus, key, by_src = load()
    lines_out = []
    say = lines_out.append
    res = {}

    ids = list(corpus)
    codes = {e: by_src[e] for e in ids}
    say("RECORD-1 read")
    say(f"  corpus  {CORPUS.name} sha256 {sha(CORPUS)}  entries {len(ids)}")
    say(f"  codes   {CODES.name} sha256 {sha(CODES)}  codes {len(by_src)} (corpus {len(codes)}, PL-1 {len(PL1)})")
    say(f"  key     {BLIND_KEY.name} sha256 {sha(BLIND_KEY)}")
    st = collections.Counter(c["status"] for c in codes.values())
    say(f"  status  {dict(st)}")

    # ---------------- S1 ----------------
    defects = [e for e in ids if codes[e]["status"] != "CHANGE"]
    change = [e for e in ids if codes[e]["status"] == "CHANGE"]
    one = [e for e in change if coded_one_kind(codes[e])]
    nofit = [e for e in change if codes[e]["kind"] == "NO-FIT"]
    insep = [e for e in change if not codes[e]["separable"]]
    contested = [e for e in one if codes[e]["alt"]]
    cov = len(one) / len(change)
    strict = len(one) / len(ids)
    defect_rate = len(defects) / len(ids)
    if defect_rate > 0.10:
        s1 = "VOID (corpus defects over 10 %)"
    elif insep or cov < 0.85:
        s1 = "KILLED"
    elif cov >= 0.95:
        s1 = "MET"
    else:
        s1 = "NOT MET (between 85 % and 95 %)"
    say("")
    say(f"S1 coverage: {len(one)}/{len(change)} CHANGE entries take exactly one kind = {cov:.4f}  -> {s1}")
    say(f"   corpus defects {len(defects)}/{len(ids)} = {defect_rate:.4f} (cap 0.10); strict reading {len(one)}/{len(ids)} = {strict:.4f}")
    say(f"   NO-FIT {len(nofit)}; inseparable {len(insep)}; contested (runner-up named, separated) {len(contested)}/{len(one)} = {len(contested)/len(one):.3f}")
    for e in defects:
        say(f"   defect {e} [{codes[e]['status']}] {corpus[e]['text'][:150]}")
    for e in nofit + insep:
        say(f"   residue {e} {corpus[e]['text'][:150]}")
    res["S1"] = dict(verdict=s1, coverage=cov, one=len(one), change=len(change), defects=defects,
                     defect_rate=defect_rate, strict=strict, nofit=nofit, inseparable=insep,
                     contested=len(contested))

    # ---------------- S2 ----------------
    def inconsistent(c):
        return (c["record"] and c["warrant_class"] != "external") or (not c["record"] and c["warrant_class"] != "internal")
    s2a_exc = [e for e in ids if inconsistent(codes[e])]
    rev = [e for e in ids if codes[e]["reverses"]]
    rev_kill = [e for e in rev if not codes[e]["record"]]
    hits = git_reversal_search()
    n_true = sum(1 for e in ids if codes[e]["record"])
    say("")
    say(f"S2(a) Record flag against the warrant's class: {len(s2a_exc)} exceptions over {len(ids)} codes"
        f" (Record = TRUE on {n_true}/{len(ids)} = {n_true/len(ids):.3f})")
    say(f"S2(b)(i) coded reversals: {len(rev)}; reversals whose own warrant is internal: {len(rev_kill)}")
    for e in rev:
        c = codes[e]
        say(f"   {e} record={c['record']} undoes: {c['reverses']}")
    say(f"S2(b)(ii) git subjects/bodies matching the reversal pattern, 2026-08-26..2026-09-26: {len(hits)}")
    for h in hits:
        say(f"   {h[0]} {h[1]} {h[2][:170]}")
    say("S2(b)(iii) every Record-TRUE entry's text stands at HEAD: VACUOUS BY CONSTRUCTION (the corpus is read from HEAD)")
    res["S2"] = dict(a_exceptions=s2a_exc, record_true=n_true, reversals=rev, reversal_kills=rev_kill,
                     git_hits=hits)

    # ---------------- S3 ----------------
    kinds_of = {e: codes[e]["kind"] for e in ids if coded_one_kind(codes[e])}
    seqs = campaign_docs(corpus, kinds_of)
    r3 = s3_read(seqs)
    Te, eseq = entry_transitions(corpus, kinds_of)
    De = diag(Te)
    eDs = entry_null(eseq, N_DRAWS, SEED_NULL)
    emD, esD = mean_sd(eDs)
    ep = (1 + sum(1 for x in eDs if x >= De)) / (1 + len(eDs))
    say("")
    say(f"S3 (documents, all pairs): transitions {r3['total']}, diagonal D = {r3['D']} "
        f"({r3['D']/r3['total']:.3f}); null {r3['null_D'][0]:.1f} +- {r3['null_D'][1]:.1f}; "
        f"one-sided p = {r3['p']:.4f}; campaigns with >= 2 documents: {r3['eff_campaigns']}")
    say(f"   S3(a) successional forbidden cells occupied: {r3['F']} (VACUOUS BY CONSTRUCTION, section 4)")
    say(f"   S3 -> {r3['verdict']}")
    say(f"   operational overlay (71 cells, reported, not graded): occupancy {r3['O']} "
        f"({r3['O']/r3['total']:.3f}); null {r3['null_O'][0]:.1f} +- {r3['null_O'][1]:.1f}")
    say(f"   entry-level (labelled, not graded): transitions {sum(Te.values())}, diagonal {De}, "
        f"null {emD:.1f} +- {esD:.1f}, p = {ep:.4f}")
    diag_by_kind = {k: r3["T"].get((k, k), 0) for k in KINDS}
    say(f"   diagonal by kind: {diag_by_kind}")
    top_off = sorted(((v, c) for c, v in r3["T"].items() if c[0] != c[1]), reverse=True)[:8]
    say(f"   largest off-diagonal cells: " + "; ".join(f"{c[0]}->{c[1]} {v}" for v, c in top_off))
    # reported, not graded (added after the first read, labelled): the diagonal against a
    # GLOBAL permutation (kinds shuffled across all campaigns), which separates campaign
    # composition from succession inside a campaign
    grng = random.Random(SEED_NULL)
    ges = list(kinds_of)
    gks = [kinds_of[e] for e in ges]
    gD = []
    for _ in range(2000):
        gks2 = gks[:]
        grng.shuffle(gks2)
        gD.append(diag(transitions(campaign_docs(corpus, dict(zip(ges, gks2))))))
    gm, gs = mean_sd(gD)
    say(f"   [reported after the first read, not graded] diagonal against a GLOBAL permutation "
        f"(2,000 draws): null {gm:.1f} +- {gs:.1f}; D = {r3['D']} is {(r3['D']-gm)/gs:.1f} sd above it "
        f"-> kinds cluster by campaign, not by succession within one")
    res_global = (gm, gs)
    per_camp = {c: len(d) for c, d in seqs.items() if len(d) >= 2}
    say(f"   documents per campaign (>= 2): {per_camp}")
    res["S3"] = dict(verdict=r3["verdict"], D=r3["D"], total=r3["total"], null=r3["null_D"], p=r3["p"],
                     overlay=r3["O"], overlay_null=r3["null_O"], forbidden=r3["F"],
                     eff_campaigns=r3["eff_campaigns"], entry=dict(D=De, total=sum(Te.values()),
                                                                   null=(emD, esD), p=ep),
                     T={f"{i}->{j}": v for (i, j), v in sorted(r3["T"].items())},
                     global_null=res_global)
    # reported, not graded (added after the first read, labelled): the coder's contested
    # boundaries, kind -> runner-up, and whether each crosses a Surface.lean block
    BLOCK = {k: b for b, ks in (("assertive", ["Facts", "Confidence", "Model", "Premises"]),
                                ("directive", ["Rules", "Priorities", "Process"]),
                                ("declaration", ["Identity"]),
                                ("carrier", ["Manner", "Structure", "Circumstances"])) for k in ks}
    pairs = collections.Counter(tuple(sorted((codes[e]["kind"], codes[e]["alt"]))) for e in kinds_of if codes[e]["alt"])
    within = sum(v for (a, b), v in pairs.items() if BLOCK[a] == BLOCK[b])
    say(f"   [reported, not graded] contested boundaries (kind with a named runner-up), unordered: " +
        "; ".join(f"{a}/{b} {v}" for (a, b), v in pairs.most_common(10)))
    say(f"   within one Surface.lean block: {within} of {sum(pairs.values())}")
    res["S1"]["contested_pairs"] = {f"{a}/{b}": v for (a, b), v in pairs.most_common()}
    res["S1"]["contested_within_block"] = within

    # ---------------- S4 ----------------
    say("")
    hist = collections.Counter(kinds_of.values())
    rhist = collections.Counter(codes[e]["root"] for e in kinds_of if codes[e]["root"])
    say("S4 site-kind histogram (corpus, one kind): " + ", ".join(f"{k} {hist.get(k, 0)}" for k in KINDS))
    say("   root-kind histogram:                    " + ", ".join(f"{k} {rhist.get(k, 0)}" for k in KINDS))
    for src in ("amendment", "notes", "misfit", "doc"):
        es = [e for e in kinds_of if corpus[e]["src"] == src]
        h = collections.Counter(kinds_of[e] for e in es)
        rt = sum(1 for e in es if codes[e]["record"])
        say(f"   {src:9s} n={len(es):3d} Record-TRUE {rt/len(es):.3f}  " +
            ", ".join(f"{k} {v}" for k, v in h.most_common()))
    say("   Record-TRUE rate per kind (base rate "
        f"{sum(1 for e in kinds_of if codes[e]['record'])/len(kinds_of):.3f}): " +
        ", ".join(f"{k} {sum(1 for e in kinds_of if kinds_of[e]==k and codes[e]['record'])}/{hist.get(k,0)}"
                  for k in KINDS if hist.get(k)))
    mixed = [k for k in KINDS if hist.get(k) and 0 < sum(1 for e in kinds_of if kinds_of[e] == k and codes[e]["record"]) < hist[k]]
    say(f"   kinds carrying both Record values: {len(mixed)} of {sum(1 for k in KINDS if hist.get(k))} present"
        f" -> the Record flag is {'NOT ' if mixed else ''}a function of the kind on this corpus")
    mis = [e for e in kinds_of if corpus[e]["src"] == "misfit"]
    mh = collections.Counter(kinds_of[e] for e in mis)
    mr = collections.Counter(codes[e]["root"] for e in mis if codes[e]["root"])
    top2 = [k for k, _ in mh.most_common(2)]
    rtop2 = [k for k, _ in mr.most_common(2)]
    guess_site = set(top2) == {"Premises", "Rules"}
    guess_root = set(rtop2) == {"Premises", "Rules"}
    say(f"   misfit occurrences (n={len(mis)}) site: " + ", ".join(f"{k} {v}" for k, v in mh.most_common()))
    say(f"   misfit occurrences root: " + ", ".join(f"{k} {v}" for k, v in mr.most_common()))
    say(f"   the lead's guess (Premises and Rules the top two): site {'HOLDS' if guess_site else 'FAILS'} "
        f"(top two {top2}); root {'HOLDS' if guess_root else 'FAILS'} (top two {rtop2})")
    top_all = [k for k, _ in hist.most_common(2)]
    rare = sorted(hist, key=lambda k: hist[k])[:3]
    say(f"   the builder's prediction: Facts and Rules the commonest sites -> top two {top_all}; "
        f"Manner, Priorities, Identity rare -> rarest three {rare} (Manner {hist.get('Manner',0)}, "
        f"Priorities {hist.get('Priorities',0)}, Identity {hist.get('Identity',0)})")
    res["S4"] = dict(hist=dict(hist), root=dict(rhist), misfit_site=dict(mh), misfit_root=dict(mr),
                     guess_site=guess_site, guess_root=guess_root, mixed_record_kinds=mixed)

    # ---------------- plants ----------------
    say("")
    pl1 = []
    for n, (k, t) in enumerate(PL1):
        c = by_src[f"PL1-{n+1:02d}"]
        pl1.append((k, c["kind"], c["kind"] == k))
    n_ok = sum(1 for x in pl1 if x[2])
    pl1_pass = n_ok >= 20
    say(f"PL-1: {n_ok}/22 synthetic entries coded to their written kind (must >= 20) -> {'PASS' if pl1_pass else 'FAIL'}")
    for k, got, ok in pl1:
        if not ok:
            say(f"   miss: written {k}, coded {got}")
    # PL-2: kinds permuted across ALL coded corpus entries
    rng = random.Random(SEED_PL2)
    es = list(kinds_of)
    ks = [kinds_of[e] for e in es]
    rng.shuffle(ks)
    shuffled = dict(zip(es, ks))
    r2 = s3_read(campaign_docs(corpus, shuffled))
    pl2_d = r2["p"] > 0.05 and abs(r2["D"] - r2["null_D"][0]) <= 3 * r2["null_D"][1]
    pl2_o = abs(r2["O"] - r2["null_O"][0]) <= 3 * r2["null_O"][1]
    pl2_pass = pl2_d and pl2_o and r2["F"] == 0
    say(f"PL-2: shuffled kinds: D = {r2['D']} against null {r2['null_D'][0]:.1f} +- {r2['null_D'][1]:.1f}, "
        f"p = {r2['p']:.4f}; overlay {r2['O']} against {r2['null_O'][0]:.1f} +- {r2['null_O'][1]:.1f}; "
        f"forbidden {r2['F']} -> {'PASS' if pl2_pass else 'FAIL'}")
    # PL-3: the checker refuses the two inconsistent synthetic codes and nothing consistent
    synth = [dict(record=rec, warrant_class=wc) for _, rec, wc, _ in PL3]
    flagged = [inconsistent(c) for c in synth]
    # the consistent mirror codes must NOT be flagged (the first build tested this clause against
    # the real codes after filtering them to the consistent ones, which could not fail; fixed
    # after the first read and labelled in the results)
    mirrors = [dict(record=True, warrant_class="external"), dict(record=False, warrant_class="internal")]
    false_flags = [m for m in mirrors if inconsistent(m)]
    pl3_pass = all(flagged) and not false_flags
    say(f"PL-3: synthetic inconsistent codes flagged {sum(flagged)}/2; consistent mirror codes flagged "
        f"{len(false_flags)}/2 -> {'PASS' if pl3_pass else 'FAIL'}")
    res["plants"] = dict(PL1=dict(ok=n_ok, passed=pl1_pass, misses=[x for x in pl1 if not x[2]]),
                         PL2=dict(D=r2["D"], null=r2["null_D"], p=r2["p"], overlay=r2["O"],
                                  overlay_null=r2["null_O"], passed=pl2_pass),
                         PL3=dict(passed=pl3_pass))

    # ---------------- branch ----------------
    s2_kill = bool(s2a_exc) and False  # S2(a) exceptions are reported; the kill is S2(b)
    s2b_kill = bool(rev_kill)
    if not (pl1_pass and pl2_pass and pl3_pass):
        branch = "(e) a plant fails"
    elif s1.startswith("VOID"):
        branch = "VOID (S1 corpus-defect cap)"
    elif s2b_kill:
        branch = "(d) S2 killed"
    elif s1 == "KILLED":
        branch = "(c) S1 killed"
    elif r3["verdict"].startswith("KILLED"):
        branch = "(b) S1 met, S3 killed" if s1 == "MET" else "S1 not met, S3 killed"
    elif r3["verdict"] == "UNRESOLVED":
        branch = "(b') S1 met, S3 unresolved" if s1 == "MET" else "S1 not met, S3 unresolved"
    elif s1 == "MET" and not s2a_exc:
        branch = "(a) S1-S3 met (S3 on the diagonal; S3(a) vacuous by construction)"
    else:
        branch = "not (a): S1 " + s1 + ", S2(a) exceptions " + str(len(s2a_exc))
    say("")
    say(f"BRANCH: {branch}")
    res["branch"] = branch
    READ_TXT.write_text("\n".join(lines_out) + "\n")
    READ_JSON.write_text(json.dumps(res, indent=1, default=str))
    print("\n".join(lines_out))


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "help"
    if cmd == "corpus":
        build_corpus()
    elif cmd == "blind":
        build_blind()
    elif cmd == "stats":
        # written after RECORD1_PREREG.md (8975be1) and RECORD1_CODES.jsonl (60899f6) were committed
        stats()
    else:
        print(__doc__)
