#!/usr/bin/env python3
"""ORDER-1, POST-FREEZE and labelled so: PO-4 as frozen (amplitude 0.7) failed on the carrier of
record by 0.004 of R^2. This re-runs the SAME plant on the SAME carrier at other amplitudes to
measure the instrument's sensitivity. It is NOT a plant of record and moves no stake or verdict.
  ORDER1_PLANT_AMP is read at import, so each amplitude is its own process:
  ORDER1_PLANT_AMP=1.0 python3 order1_plant_scan.py DIR"""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import order1_search as O
W = O.walk_features(sys.argv[1])
say = lambda s: print(s) if s.startswith(("PO-4", "# PO-4")) or "O2 tau" in s or "lag      1 ps" in s or "lag      2 ps" in s else None
O.run_plant(W, 1.0, os.path.basename(sys.argv[1].rstrip("/")) + f" at amplitude {O.PLANT_AMP:g} (POST-FREEZE sensitivity scan, NOT the plant of record)", say=say)
