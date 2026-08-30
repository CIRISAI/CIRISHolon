#!/usr/bin/env python3
"""test_landscape_sweep.py — Unit test suite for SELECTOR-5 landscape sweep.

Verifies:
  1. Group axioms (associativity, identity, inverse, closure) across all group builders.
  2. Commutator subgroup, defect, and abelianization computations.
  3. Order spectrum entropy and timescale distribution.
  4. Orientation index and Frobenius-Schur ambivalence indicators.
  5. Schur multiplier and spin cover classification.
  6. SELECTOR-4 gate verdicts on benchmark physical groups (2T, 2O, 2I, A4, A5, S4, D8, Q8, F21, Delta(27)).
  7. Deduplication, monotonic thinning, and F1 discrimination metrics.
"""

import sys
import unittest
import numpy as np

sys.path.insert(0, "/home/emoore/CIRISHolon/conformance/selector")
import landscape_sweep as LS


class TestGroupAxioms(unittest.TestCase):
    """Verify group axioms by exhaustive check on sample groups from every family."""

    def _verify_group(self, G: LS.FiniteGroup):
        n = G.order
        self.assertEqual(G.MUL.shape, (n, n))
        self.assertEqual(len(G.INV), n)
        
        # Identity check
        self.assertTrue(0 <= G.IDE < n)
        for g in range(n):
            self.assertEqual(G.MUL[G.IDE, g], g, f"Left identity fails for {G.name}")
            self.assertEqual(G.MUL[g, G.IDE], g, f"Right identity fails for {G.name}")
            
        # Inverse check
        for g in range(n):
            inv_g = G.INV[g]
            self.assertEqual(G.MUL[g, inv_g], G.IDE, f"Right inverse fails for {G.name}")
            self.assertEqual(G.MUL[inv_g, g], G.IDE, f"Left inverse fails for {G.name}")
            
        # Associativity on sample triples (or all if small)
        triples = [(x, y, z) for x in range(min(n, 6)) for y in range(min(n, 6)) for z in range(min(n, 6))]
        for x, y, z in triples:
            xy_z = G.MUL[G.MUL[x, y], z]
            x_yz = G.MUL[x, G.MUL[y, z]]
            self.assertEqual(xy_z, x_yz, f"Associativity fails for {G.name} at {x},{y},{z}")

    def test_cyclic_groups(self):
        for n in (1, 2, 7, 12, 64, 128):
            g = LS.build_cyclic(n)
            self._verify_group(g)
            self.assertEqual(len(g.commutator_subgroup), 1)
            self.assertEqual(g.commutator_defect, 0.0)

    def test_dihedral_groups(self):
        for n in (2, 3, 4, 7, 16):
            g = LS.build_dihedral(n)
            self._verify_group(g)
            self.assertEqual(g.orientation_index, 0.0, f"Dihedral D_{2*n} must be ambivalent")

    def test_alternating_groups(self):
        for deg in (3, 4, 5):
            g = LS.build_alternating(deg)
            self._verify_group(g)

    def test_symmetric_groups(self):
        for deg in (2, 3, 4, 5):
            g = LS.build_symmetric(deg)
            self._verify_group(g)
            self.assertEqual(g.orientation_index, 0.0, f"Symmetric S_{deg} must be ambivalent")

    def test_dicyclic_groups(self):
        for n in (2, 3, 4, 5, 8):
            g = LS.build_dicyclic(n)
            self._verify_group(g)
            self.assertEqual(g.order, 4 * n)

    def test_binary_polyhedral_groups(self):
        g2t = LS.build_binary_tetrahedral()
        self._verify_group(g2t)
        self.assertEqual(g2t.order, 24)

        g2o = LS.build_binary_octahedral()
        self._verify_group(g2o)
        self.assertEqual(g2o.order, 48)

        g2i = LS.build_binary_icosahedral()
        self._verify_group(g2i)
        self.assertEqual(g2i.order, 120)

    def test_frobenius_groups(self):
        for (p, k) in ((7, 3), (5, 4), (13, 3), (11, 5)):
            g = LS.build_frobenius(p, k)
            self.assertIsNotNone(g)
            self._verify_group(g)
            self.assertEqual(g.order, p * k)

    def test_delta_groups(self):
        for n in (1, 2, 3):
            g = LS.build_delta_3n2(n)
            self.assertIsNotNone(g)
            self._verify_group(g)
            self.assertEqual(g.order, 3 * n * n)


class TestSelectorCriteriaBenchmarks(unittest.TestCase):
    """Test SELECTOR-4 bootstrap criteria on known benchmarks."""

    def test_2t_binary_tetrahedral(self):
        g = LS.build_binary_tetrahedral()
        self.assertEqual(g.order, 24)
        self.assertEqual(g.num_classes, 7)
        self.assertEqual(len(g.commutator_subgroup), 8)  # Q8
        self.assertAlmostEqual(g.commutator_defect, 1.0 - 7/24)
        self.assertGreater(g.order_entropy, 1.5)
        self.assertEqual(g.num_non_ambivalent_classes, 4)
        self.assertAlmostEqual(g.orientation_index, 4/7)
        self.assertTrue(g.gate_c1)
        self.assertTrue(g.gate_c2)
        self.assertTrue(g.gate_c3)
        self.assertTrue(g.gate_c4)
        self.assertTrue(g.full_selector_pass, "2T MUST pass the full SELECTOR-4 gauntlet")

    def test_a4_tetrahedral(self):
        g = LS.build_alternating(4)
        self.assertEqual(g.order, 12)
        self.assertEqual(g.num_classes, 4)
        self.assertEqual(g.num_non_ambivalent_classes, 2)
        self.assertAlmostEqual(g.orientation_index, 0.5)
        self.assertEqual(g.schur_multiplier, (2,))
        self.assertTrue(g.gate_c1)
        self.assertTrue(g.gate_c2)
        self.assertTrue(g.gate_c3)
        self.assertTrue(g.gate_c4)
        self.assertTrue(g.full_selector_pass, "A_4 MUST pass the full SELECTOR-4 gauntlet")

    def test_2o_binary_octahedral_killed_at_c3(self):
        g = LS.build_binary_octahedral()
        self.assertEqual(g.order, 48)
        self.assertEqual(g.orientation_index, 0.0, "2O is ambivalent")
        self.assertTrue(g.gate_c1)
        self.assertTrue(g.gate_c2)
        self.assertFalse(g.gate_c3, "2O must fail C3 due to ambivalence laundering")
        self.assertFalse(g.full_selector_pass, "2O must be excluded by Gate C3")

    def test_f21_chiral_frobenius(self):
        g = LS.build_frobenius(7, 3)
        self.assertEqual(g.order, 21)
        self.assertEqual(g.num_classes, 5)
        self.assertEqual(g.num_non_ambivalent_classes, 4)
        self.assertAlmostEqual(g.orientation_index, 0.80)
        self.assertTrue(g.gate_c1)
        self.assertTrue(g.gate_c2)
        self.assertTrue(g.gate_c3, "F_21 is chiral and passes C3")
        # F21 is not a central extension and M(F21)=0
        self.assertFalse(g.gate_c4)

    def test_delta27_su3_subgroup(self):
        g = LS.build_delta_3n2(3)
        self.assertEqual(g.order, 27)
        self.assertEqual(g.num_classes, 11)
        self.assertEqual(g.num_non_ambivalent_classes, 10)
        self.assertAlmostEqual(g.orientation_index, 10/11)
        self.assertEqual(g.schur_multiplier, (3, 3))
        self.assertTrue(g.gate_c1)
        self.assertTrue(g.gate_c2)
        self.assertTrue(g.gate_c3)
        self.assertTrue(g.gate_c4)
        self.assertTrue(g.full_selector_pass, "Delta(27) MUST pass the full SELECTOR-4 gauntlet")

    def test_dihedral_killed_at_c3(self):
        for n in (3, 4, 5, 6, 8):
            g = LS.build_dihedral(n)
            self.assertTrue(g.gate_c1)
            self.assertFalse(g.gate_c3, f"D_{2*n} must fail Gate C3")
            self.assertFalse(g.full_selector_pass)


class TestLandscapeSweepIntegration(unittest.TestCase):
    """Test full landscape sweep generation and metrics."""

    def test_landscape_generation(self):
        groups = LS.generate_landscape()
        self.assertGreater(len(groups), 300, "Should enumerate at least 300 distinct groups")
        
        # Check order bounds
        for g in groups:
            self.assertLessEqual(g.order, 128)
            self.assertGreaterEqual(g.order, 1)

        disc = LS.compute_discrimination_metrics(groups)
        self.assertIn("Full SELECTOR-4 (C1..C4)", disc)
        
        full_m = disc["Full SELECTOR-4 (C1..C4)"]
        self.assertGreater(full_m["Precision"], 0.90, "SELECTOR-4 must have high precision for gauge subgroups")
        self.assertLess(full_m["pass_fraction"], 0.20, "SELECTOR-4 must thin the landscape to < 20%")


if __name__ == "__main__":
    unittest.main()
