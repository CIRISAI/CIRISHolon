# Fixture: SEAM-NS-0's rung 0, transcribed into the pre-review convention AS IT WAS DRAFTED

*A test fixture for `Audit/test_prereview_check.py`, not a preregistration. It transcribes
`conformance/water_observatory/SEAM_NS0_PREREG_DRAFT.md` (reviewed REDESIGN, 2026-11-23): its
stake S1, its bars (`T = 0.2`, kill above `0.3`), its plants P1 and P2 and its branches (V),
(a) to (d). The draft carried NO table of what its graded quantity reads under each reading.
The table below is the one its author owed: the review's arithmetic on the record's own two
fits (`conformance/water_observatory/SEAM_NS0_REVIEW.md` §1), where "the damping really
changed" reads `0.15–0.21` on the density mode and `0.25–0.36` on the current mode, and the
open plant P2 reads `0.14` and `0.25`. "One law" is given the review's fit error, at most
`0.05`. The margin is the pooled closure's banked standard error, `0.058`
(`conformance/water_observatory/RESPONSE1_RESULTS.md`). The draft also said its sound speed
`1,497` was "also quoted in" the results file; it is in the preregistration.*

The check must refuse this for the reasons the reviewer gave: the reading astride both bars,
the two readings sharing a region, and the open plant failing on paper.

```prereview
[
 {
  "kind": "stake",
  "id": "S1",
  "quantity": "D_law(L200), the defect of the 200 m/s arms against the 50 m/s linear law",
  "margin": 0.058,
  "margin_reason": "the banked SE of the pooled closure, 0.180 ± 0.058",
  "regions": [
   {
    "name": "MET",
    "hi": 0.2
   },
   {
    "name": "KILL",
    "lo": 0.3
   }
  ],
  "separates": [
   [
    "one law",
    "damping changed"
   ]
  ],
  "table": [
   {
    "hypothesis": "one law",
    "label": "the 50 m/s law at four times the amplitude",
    "reads": [
     0.0,
     0.05
    ],
    "expect": "MET"
   },
   {
    "hypothesis": "damping changed",
    "label": "the record's 200 m/s fit, density mode",
    "reads": [
     0.15,
     0.21
    ],
    "expect": "KILL"
   },
   {
    "hypothesis": "damping changed",
    "label": "the record's 200 m/s fit, current mode",
    "reads": [
     0.25,
     0.36
    ],
    "expect": "KILL"
   }
  ]
 },
 {
  "kind": "plant",
  "id": "P1",
  "stake": "S1",
  "lands": [
   0.0,
   0.05
  ],
  "must": "MET"
 },
 {
  "kind": "plant",
  "id": "P2",
  "stake": "S1",
  "lands": [
   0.14,
   0.25
  ],
  "must": "KILL"
 },
 {
  "kind": "branches",
  "first_match": true,
  "outcomes": {
   "gates and plants": [
    "ok",
    "fail"
   ],
   "S1": [
    "MET",
    "BETWEEN",
    "KILL"
   ],
   "S2": [
    "MET",
    "BETWEEN",
    "KILL"
   ]
  },
  "branches": [
   {
    "id": "V",
    "when": {
     "gates and plants": [
      "fail"
     ]
    }
   },
   {
    "id": "a",
    "when": {
     "S1": [
      "MET"
     ]
    }
   },
   {
    "id": "b",
    "when": {
     "S2": [
      "MET"
     ]
    }
   },
   {
    "id": "c",
    "when": {
     "S2": [
      "KILL"
     ]
    }
   },
   {
    "id": "d",
    "when": {}
   }
  ]
 },
 {
  "kind": "numbers",
  "cited": [
   {
    "value": "0.180",
    "source": "conformance/water_observatory/RESPONSE1_RESULTS.md"
   },
   {
    "value": "0.058",
    "source": "conformance/water_observatory/RESPONSE1_RESULTS.md"
   },
   {
    "value": "1,497",
    "source": "conformance/water_observatory/RESPONSE1_RESULTS.md"
   }
  ]
 }
]
```
