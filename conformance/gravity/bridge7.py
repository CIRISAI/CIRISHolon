#!/usr/bin/env python3
"""BRIDGE-7 instrument. Prereg ADMITTED by Audit/prereg_audit.py and frozen
before this file. Live off-channel carrier + pendant plaquette."""
import sys
import numpy as np
sys.path.insert(0, "/home/emoore/CIRISHolon/conformance/gravity")
from bridge1 import MUL, INV, CLASS, base_graph
from bridge6 import Model6, nonzero

def pendant_graph():
    """BRIDGE-7B amendment (frozen in BRIDGE7_RESULTS.md before this rerun):
    the 3-edge pendant OOM'd at 8^9 configs, so the pendant is a BIGON --
    two new parallel edges 3->d1 and d1->3. The meaning-bearing property is
    intact: its holonomy uses edges no fan plaquette touches. 8 edges =
    8^8 ~ 16.7M configs, the size the refined graph already ran."""
    edges, plaq, loop, p0, e_star = base_graph()
    base_e = len(edges)
    edges = edges + [(3, "d1"), ("d1", 3)]
    pend = [(base_e, +1), (base_e + 1, +1)]
    plaq = plaq + [pend]
    return edges, plaq, loop, p0, e_star

class Model7(Model6):
    def off_channel_vacuum(self):
        """R2' carrier: generic physical state with the dressed channel
        projected OUT -- the construction whose absence voided BRIDGE-6."""
        rng = np.random.default_rng(11)
        re = rng.integers(-2, 3, size=(4, self.N)).astype(np.int64)
        im = np.zeros_like(re)
        st = (re, im)
        for v in self.verts:
            ar = np.zeros_like(re); ai = np.zeros_like(im)
            for g in range(8):
                a, b = self.joint(st, v, g); ar += a; ai += b
            st = (ar, ai)
        sre, sim, ore, oim = self._split(st[0], st[1])
        return (np.array(ore), np.array(oim))

    def off_component_nonzero(self, st):
        _, _, ore, oim = self._split(st[0], st[1])
        return int(sum(np.count_nonzero(ore[m]) + np.count_nonzero(oim[m])
                       for m in range(4))) > 0

def run(tag, mk, pendant_far=None):
    M = Model7(mk)
    rep = {}
    probe = M.dressed_vacuum(+1)
    if nonzero(probe) == 0:
        print(f"[{tag}] G0=FIRE (dressed sector empty)"); return {"G0": "FIRE"}
    held, why = M.gauss_holds(probe)
    after_pump = M.gauss_holds(M.k_charge(probe))[0]
    rep["G0"] = "PASS" if held and after_pump else f"FIRE ({why}, pump={after_pump})"
    if rep["G0"] != "PASS":
        print(f"[{tag}] G0={rep['G0']}"); return rep

    reg = [("dressed", probe)]
    # R1 on this graph
    ws = [M.channel_weight(probe)]; c = probe
    for k in range(1, 4):
        c = M.step(c); reg.append((f"T{k}", c)); ws.append(M.channel_weight(c))
    rep["R1"] = "PASS" if len(set(ws)) > 1 else "FIRE (inert)"

    # R2': LIVE off-channel carrier
    off = M.off_channel_vacuum()
    if not M.off_component_nonzero(off):
        rep["R2'"] = "VOID (off-channel carrier empty -- cannot pose)"
    else:
        reg.append(("off0", off))
        a, b = probe, off
        seen = False
        for _ in range(3):
            a = M.step(a); b = M.step(b)
            if M.loop_classes(a) != M.loop_classes(b):
                seen = True
        rep["R2'"] = "PASS" if seen else "FIRE (charged channel does not reach the loop)"

    # R3': pendant locality (only on the pendant graph)
    if pendant_far is not None:
        p1 = M.step(probe)
        base_w = M.channel_weight(p1)
        near = M.channel_weight(M.g_geo(p1, M.p0))
        far = M.channel_weight(M.g_geo(p1, pendant_far))
        if near == base_w:
            rep["R3'"] = "VOID (geometry term inert on this graph -- cannot pose)"
        else:
            rep["R3'"] = ("PASS" if far == base_w
                          else f"FIRE far moved: base={str(base_w)[:10]} far={str(far)[:10]}")

    bad = [nm for nm, s in reg for h, _ in [M.gauss_holds(s)] if not h]
    rep["B3"] = "PASS" if not bad else f"FIRE {bad}"
    print(f"[{tag}] " + "  ".join(f"{k}={v}" for k, v in sorted(rep.items())), flush=True)
    return rep

def plants():
    ok = True
    Mok = Model7(base_graph)
    carrier = Mok.dressed_vacuum(+1)
    assert nonzero(carrier) > 0, "plant (i) carrier empty"
    Mb = Model7(base_graph, broken_action=True)
    held, why = Mok.gauss_holds(Mb.dressed_vacuum(+1))
    print(f"[plant i] wrong-side action -> B3 {'FIRES' if not held else 'MISSED'} ({why})")
    ok &= not held
    # plant (ii): carrier is the OFF-CHANNEL state, sector asserted (M-PLANT-SECTOR)
    Mp = Model7(base_graph, broken_pump=True)
    off = Mp.off_channel_vacuum()
    assert Mp.off_component_nonzero(off), "plant (ii) carrier has no off-channel component"
    held2, why2 = Mp.gauss_holds(Mp.k_charge(off))
    print(f"[plant ii] non-central pump on LIVE off-channel carrier -> B3 "
          f"{'FIRES' if not held2 else 'MISSED'} ({why2})")
    return ok and not held2

if __name__ == "__main__":
    r_base = run("base", base_graph)
    edges, plaq, loop, p0, e_star = pendant_graph()
    pend_idx = len(plaq) - 1
    r_pend = run("pendant", pendant_graph, pendant_far=pend_idx)
    ok = plants()
    gates = all(v == "PASS" for r in (r_base, r_pend) for v in r.values())
    print(f"\nVERDICT: gates {'ALL PASS' if gates else 'FIRED/VOID'}; "
          f"plants {'both FIRE' if ok else 'VOID - plant missed'}")
    sys.exit(0 if (gates and ok) else 1)
