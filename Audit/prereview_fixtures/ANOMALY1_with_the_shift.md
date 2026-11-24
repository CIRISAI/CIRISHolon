# Fixture: ANOMALY-1's reserve, with the shift its review derived written in

*A test fixture for `Audit/test_prereview_check.py`, not a preregistration. Identical to
`ANOMALY1_as_drafted.md` except for its reserve block, which now answers the two questions
the check forces: which mass is meant (the shifted one), and what the lattice adds to a
point named "massless". The shift is the review's identity
(`conformance/crystal/ANOMALY1_REVIEW.md` §1, finding 1): the chain at `μ = 0` carries a
mass term of `¼` in the lattice's own units.*

Once the shift is written the breach is arithmetic: every campaign point of S1 and S2 lies
inside the region the draft reserved for rung IV.

```prereview
[
 {
  "kind": "stake",
  "id": "S2",
  "quantity": "D_view on the window 0.40 of a period",
  "margin": 0.03,
  "margin_reason": "the draft's §5: a static misreading of 0.015 moves D_view by 0.03",
  "regions": [
   {
    "name": "MET",
    "hi": 0.07
   },
   {
    "name": "KILL",
    "lo": 0.16
   }
  ],
  "separates": [
   [
    "closed",
    "open 0.20"
   ],
   [
    "closed",
    "open 0.30"
   ]
  ],
  "table": [
   {
    "hypothesis": "closed",
    "label": "one mode",
    "reads": 0.0,
    "expect": "MET"
   },
   {
    "hypothesis": "closed",
    "label": "one mode, statics misread by 0.015",
    "reads": 0.031,
    "expect": "MET"
   },
   {
    "hypothesis": "closed",
    "label": "one mode, statics misread by 0.03",
    "reads": [
     0.061,
     0.066
    ],
    "expect": "MET"
   },
   {
    "hypothesis": "open 0.05",
    "label": "phi 0.05 at 2w1",
    "reads": 0.043,
    "expect": "MET"
   },
   {
    "hypothesis": "open 0.10",
    "label": "phi 0.10 at 2w1",
    "reads": 0.086,
    "expect": "BETWEEN"
   },
   {
    "hypothesis": "open 0.15",
    "label": "phi 0.15 at 2w1",
    "reads": 0.129,
    "expect": "BETWEEN"
   },
   {
    "hypothesis": "open 0.20",
    "label": "phi 0.20 at 2w1",
    "reads": 0.171,
    "expect": "KILL"
   },
   {
    "hypothesis": "open 0.30",
    "label": "phi 0.30 at 2w1",
    "reads": 0.253,
    "expect": "KILL"
   },
   {
    "hypothesis": "open 0.10 at 3w1",
    "label": "phi 0.10 at 3w1",
    "reads": 0.171,
    "expect": "KILL"
   },
   {
    "hypothesis": "open 0.05 at 6w1",
    "label": "phi 0.05 at 6w1",
    "reads": 0.08,
    "expect": "BETWEEN"
   }
  ]
 },
 {
  "kind": "stake",
  "id": "S1",
  "quantity": "D_sat as read through the banked referee",
  "margin": 0.02,
  "margin_reason": "none stated in the draft for S1; 0.02 is the tolerance of its own gates G-LIN and G-CLEAN",
  "regions": [
   {
    "name": "REFEREE",
    "hi": -0.10001
   },
   {
    "name": "MET",
    "lo": -0.1,
    "hi": 0.1
   },
   {
    "name": "KILL",
    "lo": 0.22
   }
  ],
  "separates": [
   [
    "closed",
    "open 0.40"
   ]
  ],
  "table": [
   {
    "hypothesis": "closed",
    "label": "one mode",
    "reads": [
     -0.067,
     0.065
    ],
    "expect": "MET"
   },
   {
    "hypothesis": "open 0.10",
    "label": "phi 0.10 at 2w1",
    "reads": [
     0.013,
     0.135
    ],
    "expect": "STRADDLES MET/BETWEEN"
   },
   {
    "hypothesis": "open 0.20",
    "label": "phi 0.20 at 2w1",
    "reads": [
     0.093,
     0.205
    ],
    "expect": "STRADDLES MET/BETWEEN"
   },
   {
    "hypothesis": "open 0.30",
    "label": "phi 0.30 at 2w1",
    "reads": [
     0.173,
     0.275
    ],
    "expect": "STRADDLES BETWEEN/KILL"
   },
   {
    "hypothesis": "open 0.40",
    "label": "phi 0.40 at 2w1",
    "reads": [
     0.253,
     0.345
    ],
    "expect": "KILL"
   },
   {
    "hypothesis": "fully screened",
    "label": "w0 = 0 at x = 4",
    "reads": [
     -0.41,
     -0.38
    ],
    "expect": "REFEREE"
   }
  ]
 },
 {
  "kind": "plant",
  "id": "P-NAIVE",
  "stake": "S2",
  "lands": [
   0.35,
   0.502
  ],
  "must": {
   "lo": 0.25
  }
 },
 {
  "kind": "plant",
  "id": "P-NUMBER",
  "stake": "S2",
  "lands": 0.25,
  "must": {
   "lo": 0.2
  }
 },
 {
  "kind": "plant",
  "id": "P-SYNTH",
  "stake": "S2",
  "lands": 0.253,
  "must": "KILL"
 },
 {
  "kind": "branches",
  "first_match": true,
  "outcomes": {
   "gates and plants": [
    "ok",
    "fail or void"
   ],
   "solves": [
    "ok",
    "raise or every x void"
   ],
   "S1": [
    "MET",
    "BETWEEN",
    "KILL",
    "REFEREE",
    "VOID"
   ],
   "S2": [
    "MET",
    "BETWEEN",
    "KILL",
    "VOID"
   ]
  },
  "branches": [
   {
    "id": "e",
    "when": {
     "gates and plants": [
      "fail or void"
     ]
    }
   },
   {
    "id": "v",
    "when": {
     "solves": [
      "raise or every x void"
     ]
    }
   },
   {
    "id": "r",
    "when": {
     "S1": [
      "REFEREE"
     ]
    }
   },
   {
    "id": "k",
    "when": {
     "S1": [
      "KILL"
     ]
    }
   },
   {
    "id": "k",
    "when": {
     "S2": [
      "KILL"
     ]
    }
   },
   {
    "id": "a",
    "when": {
     "S1": [
      "MET"
     ],
     "S2": [
      "MET"
     ]
    }
   },
   {
    "id": "b",
    "when": {}
   }
  ]
 },
 {
  "kind": "numbers",
  "cited": [
   {
    "value": "0.664187",
    "source": "conformance/crystal/SCHWINGER3_RESULTS.md"
   },
   {
    "value": "0.627240",
    "source": "conformance/crystal/SCHWINGER3_RESULTS.md"
   },
   {
    "value": "0.608620",
    "source": "conformance/crystal/SCHWINGER3_RESULTS.md"
   },
   {
    "value": "3.3 %",
    "source": "conformance/crystal/KINK0_ANOMALY0_REVIEW.md"
   }
  ]
 },
 {
  "kind": "reserve",
  "id": "rung IV",
  "region": {
   "x": {
    "lo": 1.5,
    "hi": 25
   },
   "mu": {
    "lo": 0,
    "lo_open": true
   }
  },
  "meaning": {
   "x": "bare",
   "mu": "shifted"
  },
  "shifts": {
   "mu": "mu + 1/4"
  },
  "known_inside": "to first order the field slope is linear in the shifted mass with a known zero, so one reading fixes the small-mass, small-background surface (the review's finding 3)"
 },
 {
  "kind": "points",
  "of": "campaign",
  "label": "S1 statics, massless",
  "points": [
   {
    "x": 4,
    "mu": 0
   },
   {
    "x": 9,
    "mu": 0
   },
   {
    "x": 16,
    "mu": 0
   }
  ]
 },
 {
  "kind": "points",
  "of": "campaign",
  "label": "S2 dynamics, massless",
  "points": [
   {
    "x": 2,
    "mu": 0
   },
   {
    "x": 4,
    "mu": 0
   }
  ]
 },
 {
  "kind": "points",
  "of": "gate",
  "label": "rung 0 at the gate point",
  "points": [
   {
    "x": 0.7,
    "mu": 0.62
   },
   {
    "x": 0.7,
    "mu": 0
   }
  ]
 }
]
```
