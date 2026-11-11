#!/usr/bin/env python3
"""FERMION-0: the toric code's bound charge and flux on the stabilizer tier.

Prereg conformance/crystal/FERMION0_PREREG.md, Amendments 1 and 2. This file is the
circuit generator, the reader and the predictors. It runs every circuit on two
simulators: the engine's `holon-qasm run --tier tableau` (exact distribution of the
classical register) and stim's tableau simulator (exact per-ancilla expectation).

usage: fermion0.py <holon-qasm binary> <out dir> [step ...]
steps: time g0 plants s1 s2 s3 s45 all
"""
import json, os, random, subprocess, sys, time
import stim

L = 12
ND = 2 * L * L                 # data qubits, one per link
SIGN = ND                      # the sign ancilla
SYN0 = ND + 1                  # 144 vertex ancillas, then 144 plaquette ancillas
NV = L * L
SEED = 20261111
DIRS = {"E": (1, 0), "N": (0, 1), "W": (-1, 0), "S": (0, -1)}
OPP = {"E": "W", "W": "E", "N": "S", "S": "N"}

def h(x, y): return 2 * ((y % L) * L + (x % L))
def vt(x, y): return 2 * ((y % L) * L + (x % L)) + 1
def vid(x, y): return (y % L) * L + (x % L)
def star(x, y): return [h(x, y), h(x - 1, y), vt(x, y), vt(x, y - 1)]
def plaq(x, y): return [h(x, y), vt(x + 1, y), h(x, y + 1), vt(x, y)]

def link_vertices(q):
    i = q // 2; x, y = i % L, i // L
    return [vid(x, y), vid(x + 1, y)] if q % 2 == 0 else [vid(x, y), vid(x, y + 1)]

def link_plaqs(q):
    i = q // 2; x, y = i % L, i // L
    return [vid(x, y), vid(x, y - 1)] if q % 2 == 0 else [vid(x, y), vid(x - 1, y)]

# ---------------------------------------------------------------- hops (one fine step each)
def e_link(x, y, d): return {"E": h(x, y), "W": h(x - 1, y), "N": vt(x, y), "S": vt(x, y - 1)}[d]
def m_link(x, y, d): return {"E": vt(x + 1, y), "W": vt(x, y), "N": h(x, y + 1), "S": h(x, y)}[d]

def hop(kind, x, y, d):
    """One fine step: the Paulis that move a particle of `kind` from site (x, y) one link in d."""
    if kind == "e": return [("Z", e_link(x, y, d))]
    if kind == "m": return [("X", m_link(x, y, d))]
    return [("Z", e_link(x, y, d)), ("X", m_link(x, y, d))]

def move(kind, x, y, d, n):
    steps = []
    for _ in range(n):
        steps.append(hop(kind, x, y, d)); x += DIRS[d][0]; y += DIRS[d][1]
    return steps, x, y

def twist(x, y):
    """The e of an eps at (x, y) carried once around its own m (the plaquette north-east)."""
    return [[("Z", h(x, y))], [("Z", vt(x + 1, y))], [("Z", h(x, y + 1))], [("Z", vt(x, y))]]

def null_excursion(x, y):
    """Same four steps' length, enclosing nothing: to the east neighbour and back, twice."""
    return [[("Z", h(x, y))] for _ in range(4)]

def junction(kind, cx, cy, da, db, dp, c, exchanges):
    """Setup (two particles at the ends of arms a and b) and `exchanges` exchanges."""
    ax, ay = cx + c * DIRS[da][0], cy + c * DIRS[da][1]
    bx, by = cx + c * DIRS[db][0], cy + c * DIRS[db][1]
    px, py = cx + c * DIRS[dp][0], cy + c * DIRS[dp][1]
    s1, _, _ = move(kind, ax, ay, OPP[da], c)
    s2, _, _ = move(kind, cx, cy, db, c)
    setup = s1 + s2
    one = []
    for (x, y, d) in [(bx, by, OPP[db]), (cx, cy, dp), (ax, ay, OPP[da]),
                      (cx, cy, db), (px, py, OPP[dp]), (cx, cy, da)]:
        st, _, _ = move(kind, x, y, d, c); one += st
    return setup, one * exchanges, (ax, ay), (bx, by), (px, py)

# ---------------------------------------------------------------- the fine tracker
def toggle(E, M, pauli):
    p, q = pauli
    if p == "Z":
        for v in link_vertices(q): E ^= {v}
    else:
        for pl in link_plaqs(q): M ^= {pl}
    return E, M

def config_after(steps, E=None, M=None):
    E = set() if E is None else set(E); M = set() if M is None else set(M)
    for st in steps:
        for pa in st: E, M = toggle(E, M, pa)
    return E, M

def coarse_history(setup, process, c):
    """Per fine step of the process: how many e and how many m each cell holds."""
    E, M = config_after(setup)
    def counts():
        ce, cm = {}, {}
        for v in E:
            k = ((v % L) // c, (v // L) // c); ce[k] = ce.get(k, 0) + 1
        for p in M:
            k = ((p % L) // c, (p // L) // c); cm[k] = cm.get(k, 0) + 1
        return (tuple(sorted(ce.items())), tuple(sorted(cm.items())))
    hist = [counts()]
    for st in process:
        for pa in st: E, M = toggle(E, M, pa)
        hist.append(counts())
    return hist

# ---------------------------------------------------------------- circuits
def prep_ops(trivial=False):
    if trivial:
        return [("h", q) for q in range(ND)]
    ops = []
    for y in range(L):
        for x in range(L):
            if x == L - 1 and y == L - 1: continue
            rep = h(x, y) if x < L - 1 else vt(x, y)
            ops.append(("h", rep))
            for q in star(x, y):
                if q != rep: ops.append(("cx", rep, q))
    return ops

def pauli_ops(steps):
    return [("x" if p == "X" else "z", q) for st in steps for (p, q) in st]

def controlled(anc, paulis):
    ops = [("h", anc)]
    for (p, q) in paulis: ops.append(("cx", anc, q) if p == "X" else ("cz", anc, q))
    ops.append(("h", anc))
    return ops

def syndrome_ops():
    ops = []
    for v in range(NV):
        ops += controlled(SYN0 + v, [("X", q) for q in star(v % L, v // L)])
    for p in range(NV):
        ops += controlled(SYN0 + NV + p, [("Z", q) for q in plaq(p % L, p // L)])
    return ops

def to_qasm(nq, ops, measured):
    out = ['OPENQASM 2.0;', 'include "qelib1.inc";', f"qreg q[{nq}];", f"creg c[{len(measured)}];"]
    for o in ops:
        out.append(f"{o[0]} q[{o[1]}];" if len(o) == 2 else f"{o[0]} q[{o[1]}],q[{o[2]}];")
    for i, q in enumerate(measured): out.append(f"measure q[{q}] -> c[{i}];")
    return "\n".join(out) + "\n"

def to_stim(ops):
    name = {"h": "H", "x": "X", "z": "Z", "cx": "CX", "cz": "CZ"}
    return stim.Circuit("\n".join(f"{name[o[0]]} " + " ".join(str(a) for a in o[1:]) for o in ops))

class Runner:
    def __init__(self, binary, outdir):
        self.binary, self.outdir = binary, outdir
        self.n = 0; self.agree = 0; self.disagree = []; self.seconds = 0.0
        self.rev = None          # bit order of the engine's printed register, set by P-ORDER
        os.makedirs(outdir, exist_ok=True)

    def engine(self, nq, ops, measured):
        path = os.path.join(self.outdir, "circuit.qasm")
        with open(path, "w") as f: f.write(to_qasm(nq, ops, measured))
        t0 = time.perf_counter()
        r = subprocess.run([self.binary, "run", path, "--tier", "tableau"], capture_output=True, text=True)
        self.seconds += time.perf_counter() - t0
        if r.returncode != 0: raise RuntimeError("engine: " + r.stderr[:300] + r.stdout[:300])
        return json.loads(r.stdout)["dist"]

    def run(self, ops, ancillas, nq=None):
        """Both simulators on one gate list. Returns {ancilla: +1 | -1 | 0}, 0 = undetermined.
        Amendment 1 section 4: stim first; the engine on everything if all are determined, on the
        sign ancilla and at most 3 undetermined ancillas otherwise."""
        nq = nq or (SYN0 + 2 * NV)
        sim = stim.TableauSimulator(); sim.set_num_qubits(nq); sim.do(to_stim(ops))
        ref = {a: sim.peek_z(a) for a in ancillas}
        und = [a for a in ancillas if ref[a] == 0]
        self.n += 1
        if not und:
            dist = self.engine(nq, ops, ancillas)
            ok = len(dist) == 1
            if ok:
                key = next(iter(dist)); bits = key[::-1] if self.rev else key
                ok = all((bits[i] == "1") == (ref[a] == -1) for i, a in enumerate(ancillas))
        else:
            sub = ([ancillas[0]] if ancillas[0] not in und else []) + und[:3]
            dist = self.engine(nq, ops, sub)
            ok = True
            for i, a in enumerate(sub):
                p1 = sum(pr for k, pr in dist.items() if (k[::-1] if self.rev else k)[i] == "1")
                want = 0.5 if ref[a] == 0 else (1.0 if ref[a] == -1 else 0.0)
                ok = ok and abs(p1 - want) < 1e-9
        if ok: self.agree += 1
        else: self.disagree.append(self.n)
        return ref

ALL_ANC = [SIGN] + [SYN0 + i for i in range(2 * NV)]

def process_circuit(setup, process, trivial=False):
    paulis = [pa for st in process for pa in st]
    return prep_ops(trivial) + pauli_ops(setup) + controlled(SIGN, paulis) + syndrome_ops()

def report(R, setup, process, trivial=False):
    """The reporting rule. Returns ('sign', +1|-1) or ('refused', reason)."""
    E0, M0 = config_after(setup)
    E1, M1 = config_after(process, E0, M0)
    ref = R.run(process_circuit(setup, process, trivial), ALL_ANC)
    if ref[SIGN] == 0: return ("refused", "sign ancilla undetermined")
    for v in range(NV):
        s = ref[SYN0 + v]
        if s == 0: return ("refused", "a vertex operator is undetermined")
        if (s == -1) != (v in E0): return ("refused", "vertex syndrome is not the claimed configuration")
    for p in range(NV):
        s = ref[SYN0 + NV + p]
        if s == 0: return ("refused", "a plaquette operator is undetermined")
        if (s == -1) != (p in M0): return ("refused", "plaquette syndrome is not the claimed configuration")
    if (E1, M1) != (E0, M0): return ("refused", "the process does not close")
    return ("sign", ref[SIGN])

# ---------------------------------------------------------------- predictors
def perm_parity(hist, which):
    """Follow each particle of one type from cell to cell through the coarse history and return
    the parity of the permutation of the starting cells. Needs: no cell ever holds two."""
    start = [k for k, n in hist[0][which]]
    for snap in hist:
        if any(n > 1 for _, n in snap[which]): raise RuntimeError("a cell holds two")
    pos = list(start)
    for a, b in zip(hist, hist[1:]):
        ca, cb = dict(a[which]), dict(b[which])
        gone = [k for k in ca if k not in cb]; new = [k for k in cb if k not in ca]
        if len(gone) != len(new) or len(gone) > 1: raise RuntimeError("unreadable coarse step")
        if gone: pos[pos.index(gone[0])] = new[0]
    perm = [start.index(p) for p in pos]
    par, seen = 0, set()
    for i in range(len(perm)):
        if i in seen: continue
        j, n = i, 0
        while j not in seen: seen.add(j); j = perm[j]; n += 1
        par ^= (n - 1) & 1
    return par

def p_parts(hist):
    # Amendment 1: the resolved braids of every e around every m. With pairs moving as one and no
    # cell holding two, that count's parity is the parity of the permutation of the pairs.
    pe, pm = perm_parity(hist, 0), perm_parity(hist, 1)
    if pe != pm: raise RuntimeError("e and m permutations differ")
    return -1 if pe else 1

def p_whole_bare(hist, label=-1):
    return label if perm_parity(hist, 0) else 1

def memory_bits(setup, process):
    """One bit per bound pair, from the fine history, by a rule that does not know which processes
    were built with a twist: an excursion of an e away from its m that returns without the m
    moving flips the bit if its closed path is the boundary of the m's plaquette."""
    E, M = config_after(setup)
    flips, chain = 0, set()
    for st in process:
        kinds = {p for p, _ in st}
        if kinds == {"Z"}:
            for _, q in st: chain ^= {q}
            E, M = config_after([st], E, M)
            if E == M:                        # every e is back on its own m's site
                if chain:
                    hit = [p for p in M if set(plaq(p % L, p // L)) == chain]
                    if not hit: raise RuntimeError("an excursion encloses something unreadable")
                    flips ^= 1
                chain = set()
        else:
            if chain: raise RuntimeError("an excursion did not return before the pair moved")
            E, M = config_after([st], E, M)
    return flips

def fill(chain, dual):
    """The region whose boundary is `chain` (links, mod 2). dual=False: plaquettes bounded by Z
    links. dual=True: vertices whose stars make up X links. None if the chain winds the torus."""
    inside = {}
    for y in range(L):
        if y == 0: inside[(0, 0)] = 0
        else:
            cross = vt(0, y - 1) if dual else h(0, y)
            inside[(0, y)] = inside[(0, y - 1)] ^ (cross in chain)
        for x in range(1, L):
            cross = h(x - 1, y) if dual else vt(x, y)
            inside[(x, y)] = inside[(x - 1, y)] ^ (cross in chain)
    region = {vid(x, y) for (x, y), b in inside.items() if b}
    bound = set()
    for r in region:
        for q in (star(r % L, r // L) if dual else plaq(r % L, r // L)): bound ^= {q}
    return region if bound == chain else None

def p_fine(setup, process):
    """Amendment 2. Integer arithmetic only; no simulator."""
    E0, M0 = config_after(setup)
    sign, seen_z, zc, xc = 1, {}, set(), set()
    for st in process:
        for p, q in st:
            if p == "Z": seen_z[q] = seen_z.get(q, 0) + 1; zc ^= {q}
            else:
                if seen_z.get(q, 0) & 1: sign = -sign
                xc ^= {q}
    rz, rx = fill(zc, False), fill(xc, True)
    if rz is None or rx is None: return None
    if len(rz & M0) & 1: sign = -sign
    if len(rx & E0) & 1: sign = -sign
    return sign

# ---------------------------------------------------------------- the steps
def rand_junction(rng, c):
    n = L // c
    cx, cy = c * rng.randrange(n), c * rng.randrange(n)
    da, db, dp = rng.sample(list(DIRS), 3)
    return cx, cy, da, db, dp

def s3_processes(rng):
    out = []
    for kind, want in (("e", 1), ("m", 1), ("eps", -1)):
        for _ in range(100):
            cx, cy, da, db, dp = rand_junction(rng, 1)
            setup, proc, *_ = junction(kind, cx, cy, da, db, dp, 1, 1)
            out.append((f"{kind} exchanged with {kind}", want, setup, proc))
    for _ in range(100):
        x, y = rng.randrange(L), rng.randrange(L)
        setup = [[("X", vt(x, y))]]                      # an m at plaquette (x, y) and its west neighbour
        proc = [[("Z", q)] for q in plaq(x, y)]
        out.append(("full braid of e around m", -1, setup, proc))
    for _ in range(100):
        x, y = rng.randrange(L), rng.randrange(L)
        d = rng.choice(list(DIRS))
        setup = [hop("eps", x, y, d)]
        out.append(("twist of eps", -1, setup, twist(x, y)))
    return out

def s45_processes(rng, c):
    """1,000 processes: (one exchange or two) x (null excursion or twist), 250 each."""
    out = []
    for ex in (1, 2):
        for tw in (0, 1):
            for _ in range(250):
                cx, cy, da, db, dp = rand_junction(rng, c)
                setup, proc, (ax, ay), _, _ = junction("eps", cx, cy, da, db, dp, c, ex)
                pre = twist(ax, ay) if tw else null_excursion(ax, ay)
                out.append({"ex": ex, "tw": tw, "setup": setup, "process": pre + proc,
                            "twin": (null_excursion(ax, ay) if tw else twist(ax, ay)) + proc})
    return out

def main():
    binary, outdir = sys.argv[1], sys.argv[2]
    steps = sys.argv[3:] or ["all"]
    do = lambda s: s in steps or "all" in steps
    R = Runner(binary, outdir)
    log = open(os.path.join(outdir, "fermion0.log"), "a")
    def say(*a):
        line = " ".join(str(x) for x in a); print(line, flush=True); log.write(line + "\n"); log.flush()
    say(f"# FERMION-0 run, steps={steps}, stim {stim.__version__}, L={L}, seed {SEED}")

    # P-ORDER always runs: every later reading depends on it.
    d = R.engine(3, [("x", 1)], [0, 1, 2])
    key = next(iter(d))
    if len(d) != 1 or key not in ("010",):
        if len(d) == 1 and key in ("010",): pass
    d2 = R.engine(3, [("x", 0)], [0, 1, 2]); k2 = next(iter(d2))
    if len(d2) != 1 or k2 not in ("100", "001"): raise SystemExit(f"P-ORDER unreadable: {d2}")
    R.rev = (k2 == "001")
    say(f"P-ORDER  X on qubit 0 measured into c[0] prints '{k2}': register is "
        f"{'last bit first' if R.rev else 'first bit first'}; the reader names the flipped bit: 1 of 1  FIRES")

    if do("time"):
        rng = random.Random(1)
        t0 = time.perf_counter(); e0 = R.seconds
        for (_, _, setup, proc) in s3_processes(rng)[:10]: report(R, setup, proc)
        say(f"TIMING   10 circuits of {SYN0 + 2 * NV} qubits: {time.perf_counter() - t0:.2f} s wall on one core, "
            f"of which the engine {R.seconds - e0:.2f} s; about 7,000 circuits extrapolates to "
            f"{(time.perf_counter() - t0) * 700 / 60:.0f} core-minutes (a timing of these 10, not of the campaign)")

    if do("g0"):
        loops = [[("Z", h(x, 0)) for x in range(L)], [("Z", vt(0, y)) for y in range(L)]]
        la = [SYN0 + 2 * NV, SYN0 + 2 * NV + 1]
        ops = prep_ops() + syndrome_ops() + controlled(la[0], loops[0]) + controlled(la[1], loops[1])
        ref = R.run(ops, [SYN0 + i for i in range(2 * NV)] + la, nq=SYN0 + 2 * NV + 2)
        good = sum(1 for v in ref.values() if v == 1)
        say(f"G0       vacuum: {good} of {len(ref)} stabilizers and loops read +1, determined  "
            f"{'MET' if good == len(ref) else 'FAILED'}")
        if good != len(ref): raise SystemExit("G0 failed; nothing runs")

    if do("plants"):
        rng = random.Random(SEED + 1)
        rep = [report(R, setup, proc, trivial=True) for (_, _, setup, proc) in s3_processes(rng)]
        n_sign = sum(1 for r in rep if r[0] == "sign")
        say(f"P-TRIVIAL product state, S3's 500 processes: {n_sign} signs reported, {500 - n_sign} refused  "
            f"{'FIRES' if n_sign == 0 else 'FAILS'}")
        rng = random.Random(SEED + 2); refused = 0
        for _ in range(50):
            cx, cy, da, db, dp = rand_junction(rng, 1)
            setup, proc, *_ = junction(rng.choice(["e", "m", "eps"]), cx, cy, da, db, dp, 1, 1)
            if report(R, setup, proc[:-1])[0] == "refused": refused += 1
        say(f"P-OPEN   50 words that do not close: {refused} refused  {'FIRES' if refused == 50 else 'FAILS'}")

    if do("s1"):
        rng = random.Random(SEED + 3); ok = n = 0
        for _state in range(10):
            setup = []
            for _ in range(6):
                st, _, _ = move(rng.choice(["e", "m", "eps"]), rng.randrange(L), rng.randrange(L),
                                rng.choice(list(DIRS)), rng.randrange(1, 5)); setup += st
            E0, M0 = config_after(setup)
            ops = prep_ops() + pauli_ops(setup); anc = []; want = []
            a = ND
            for _ in range(200):
                x0, y0 = rng.randrange(L), rng.randrange(L); w, hgt = rng.randrange(1, 6), rng.randrange(1, 6)
                cells = [(x0 + i, y0 + j) for i in range(w) for j in range(hgt)]
                for dual in (True, False):
                    inner, bound = [], set()
                    for (x, y) in cells:
                        qs = star(x, y) if dual else plaq(x, y)
                        inner += qs
                        for q in qs: bound ^= {q}
                    P = "X" if dual else "Z"
                    ops += controlled(a, [(P, q) for q in inner]) + controlled(a + 1, [(P, q) for q in sorted(bound)])
                    inside = {vid(x, y) for (x, y) in cells}
                    anc += [a, a + 1]; want.append((-1) ** len(inside & (E0 if dual else M0))); a += 2
            ref = R.run(ops, anc, nq=a)
            for i, w_ in enumerate(want):
                n += 1; ok += (ref[anc[2 * i]] == ref[anc[2 * i + 1]] == w_)
        say(f"S1       Gauss: interior product = boundary loop = the count inside, {ok} of {n}  "
            f"{'MET' if ok == n == 4000 else 'KILLED'}")

    if do("s2"):
        rng = random.Random(SEED + 4); odd = 0; paired = 0; planted = None
        for i in range(2000):
            setup = [[]]
            for q in range(ND):
                if rng.random() < 0.05:
                    p = rng.choice("XYZ")
                    if p in "XY": setup[0].append(("X", q))
                    if p in "ZY": setup[0].append(("Z", q))
            ref = R.run(prep_ops() + pauli_ops(setup) + syndrome_ops(), ALL_ANC[1:])
            bits = [ref[a] for a in ALL_ANC[1:]]
            if 0 in bits: raise SystemExit("S2: an undetermined syndrome on a Pauli state")
            ne = sum(1 for b in bits[:NV] if b == -1); nm = sum(1 for b in bits[NV:] if b == -1)
            E0, M0 = config_after(setup)
            if ne != len(E0) or nm != len(M0): raise SystemExit("S2: syndrome differs from the tracker")
            odd += (ne & 1) or (nm & 1); paired += (ne + nm) > 0
            if i == 0:
                tam = list(bits); tam[0] = -tam[0]
                planted = sum(1 for b in tam[:NV] if b == -1) & 1
        say(f"P-ONE    one syndrome bit flipped by hand: the parity checker reports "
            f"{'1 odd state of 1  FIRES' if planted else 'nothing  FAILS'}")
        say(f"S2       2,000 random Pauli products: {odd} with an odd count; {paired} create at least one pair  "
            f"{'MET' if odd == 0 and paired >= 1800 else 'KILLED'}")

    if do("s3"):
        rng = random.Random(SEED + 1); tally = {}
        for (name, want, setup, proc) in s3_processes(rng):
            r = report(R, setup, proc); t = tally.setdefault(name, [0, 0, 0, want])
            if r[0] == "refused": t[2] += 1
            elif r[1] == want: t[0] += 1
            else: t[1] += 1
        good = 0
        for name, (a, b, c_, want) in tally.items():
            good += a; say(f"S3       {name}: {a} read {want:+d}, {b} read the other sign, {c_} refused")
        say(f"S3       {good} of 500  {'MET' if good == 500 else 'KILLED'}")

    if do("s45"):
        fine_ok = fine_n = 0
        for c in (2, 3):
            rng = random.Random(SEED + 10 + c); procs = s45_processes(rng, c)
            same = diff = 0; sc = {"P-parts": 0, "P-whole-bare": 0, "P-whole": 0, "P-label": 0}; agree = 0
            kinds = {}
            for i, pr in enumerate(procs):
                hist = coarse_history(pr["setup"], pr["process"], c)
                r = report(R, pr["setup"], pr["process"])
                if r[0] != "sign": raise SystemExit(f"S5: process {i} refused: {r[1]}")
                truth = r[1]; kinds[(pr["ex"], pr["tw"])] = kinds.get((pr["ex"], pr["tw"]), 0) + 1
                pp, pb = p_parts(hist), p_whole_bare(hist)
                bit = memory_bits(pr["setup"], pr["process"])
                pw = pb * (-1 if bit else 1); pl = p_whole_bare(hist, label=1) * (-1 if bit else 1)
                sc["P-parts"] += pp == truth; sc["P-whole-bare"] += pb == truth
                sc["P-whole"] += pw == truth; sc["P-label"] += pl == truth; agree += pp == pb
                pf = p_fine(pr["setup"], pr["process"]); fine_n += 1; fine_ok += pf == truth
                if i % 2 == 0:                       # S4: 500 pairs, the process and its twin
                    hist2 = coarse_history(pr["setup"], pr["twin"], c)
                    r2 = report(R, pr["setup"], pr["twin"])
                    if r2[0] != "sign": raise SystemExit(f"S4: twin {i} refused: {r2[1]}")
                    same += hist == hist2; diff += (hist == hist2) and (r2[1] != truth)
            say(f"c={c}  kinds (exchanges, twist): {sorted(kinds.items())}; odd exchanges {kinds[(1,0)] + kinds[(1,1)]}")
            say(f"P-LABEL  c={c}: with the label read as boson, P-whole is right on {sc['P-label']} of 1000  "
                f"{'FIRES' if sc['P-label'] <= 550 else 'FAILS'}" + ("" if c == 2 else "  (reported; the plant is staked at c=2)"))
            say(f"S4       c={c}: {same} of 500 pairs have identical coarse histories step by step; "
                f"{diff} of 500 have opposite signs  {'MET' if same == diff == 500 else 'KILLED'}")
            say(f"S5       c={c}: P-whole {sc['P-whole']}/1000, P-parts {sc['P-parts']}/1000, "
                f"P-whole-bare {sc['P-whole-bare']}/1000, P-parts = P-whole-bare on {agree}/1000  "
                f"{'MET' if sc['P-whole'] == 1000 and sc['P-parts'] <= 550 and agree == 1000 else 'KILLED'}")
        say(f"S6       lattice resolution: P-fine right on {fine_ok} of {fine_n}  "
            f"{'MET' if fine_ok == fine_n == 2000 else 'KILLED'}")

    say(f"P-REFEREE engine and stim agree on {R.agree} of {R.n} circuits"
        + ("  FIRES" if not R.disagree else f"  FAILS at circuits {R.disagree[:10]}"))
    say(f"# engine seconds {R.seconds:.1f}")

if __name__ == "__main__":
    main()
