# SATURATION-3 G2: THE FAIR CPU ARM.
#
# The GPU arm does not just run on a GPU. It also reformulates sigma as three GEMMs and
# hands them to a tuned library. Quoting its speedup against holon-chem's hand-written
# sigma_direct would therefore be quoting a REFORMULATION-plus-library against a plain
# loop, and attributing the whole difference to the device. holon-gpu/src/cpu.rs's own
# header warns about exactly this: "so the GPU's speedup is quoted against the best CPU and
# not against the most convenient one".
#
# This runs the IDENTICAL three-GEMM formulation on the CPU through OpenBLAS, on the real
# exported (O,O,O) index structures, and checks the answer against the same CPU reference
# the GPU is checked against. Whatever it reaches is the number the GPU has to beat.
import numpy as np, time, os, sys

path = sys.argv[1] if len(sys.argv) > 1 else "ooo.bin"
reps = int(os.environ.get("REPS", "5"))

with open(path, "rb") as f:
    n, na, nb, ns_a, ns_b, n_det = np.frombuffer(f.read(48), dtype="<u8")
    n, na, nb, ns_a, ns_b, n_det = map(int, (n, na, nb, ns_a, ns_b, n_det))
    n2 = n * n
    k = np.frombuffer(f.read(n2 * 8), dtype="<f8").copy()
    g = np.frombuffer(f.read(n2 * n2 * 8), dtype="<f8").copy().reshape(n2, n2)
    sd = np.dtype([("pq", "<u4"), ("dst", "<u4"), ("sign", "<f8")])
    a_s = np.frombuffer(f.read(na * ns_a * 16), dtype=sd).reshape(na, ns_a).copy()
    b_s = np.frombuffer(f.read(nb * ns_b * 16), dtype=sd).reshape(nb, ns_b).copy()
    c = np.frombuffer(f.read(n_det * 8), dtype="<f8").copy().reshape(na, nb)
    ref = np.frombuffer(f.read(n_det * 8), dtype="<f8").copy().reshape(na, nb)

print(f"loaded  n_orb {n}, na {na}, nb {nb}, ns_a {ns_a}, n_det {n_det}")

# ---- the same c-independent precomputation the GPU does ----
src_jb = np.full((n2, nb), -1, dtype=np.int64)
src_sign = np.zeros((n2, nb))
for jb in range(nb):
    r = b_s[jb]
    src_jb[r["pq"], r["dst"]] = jb
    src_sign[r["pq"], r["dst"]] = r["sign"]
assert (src_jb >= 0).sum() == nb * ns_b, "the beta gather map is not injective"

A = g[a_s["pq"].reshape(-1)].reshape(na, ns_a, n2)

at_ja = np.zeros((na, ns_a), dtype=np.int64)
at_m = np.zeros((na, ns_a), dtype=np.int64)
at_sign = np.zeros((na, ns_a))
fill = np.zeros(na, dtype=np.int64)
for ja in range(na):
    for m in range(ns_a):
        ia = int(a_s[ja, m]["dst"])
        t = fill[ia]; fill[ia] += 1
        at_ja[ia, t] = ja; at_m[ia, t] = m; at_sign[ia, t] = a_s[ja, m]["sign"]
assert (fill == ns_a).all(), "the alpha transpose is ragged"

Fb = np.zeros((nb, nb)); Fa = np.zeros((na, na))
for jb in range(nb):
    for t in range(ns_b):
        s1 = b_s[jb, t]
        np.add.at(Fb[jb], s1["dst"], s1["sign"] * k[s1["pq"]])
        s2 = b_s[int(s1["dst"])]
        np.add.at(Fb[jb], s2["dst"], 0.5 * s1["sign"] * s2["sign"] * g[s2["pq"], s1["pq"]])
for ja in range(na):
    for t in range(ns_a):
        s1 = a_s[ja, t]
        np.add.at(Fa[ja], s1["dst"], s1["sign"] * k[s1["pq"]])
        s2 = a_s[int(s1["dst"])]
        np.add.at(Fa[ja], s2["dst"], 0.5 * s1["sign"] * s2["sign"] * g[s2["pq"], s1["pq"]])

flat_jb = src_jb.reshape(-1)
D_idx = (at_ja * ns_a + at_m).reshape(-1)

def sigma():
    S = c @ Fb
    S += Fa.T @ c
    T = c[:, flat_jb].reshape(na, n2, nb) * src_sign[None, :, :]   # the gather
    D = (A @ T).reshape(na * ns_a, nb)                              # batched GEMM
    S += np.einsum("it,itb->ib", at_sign, D[D_idx].reshape(na, ns_a, nb), optimize=True)
    return S

got = sigma()
scale = np.abs(ref).max()
err = np.abs(got - ref).max()
print(f"\nagreement with the CPU reference: max abs {err:.3e}, relative {err/scale:.3e}")
if not err / scale < 1e-12:
    print("REFUSED: the reformulated CPU sigma does not reproduce sigma_direct.")
    sys.exit(1)

sigma()
t0 = time.perf_counter()
for _ in range(reps):
    sigma()
dt = (time.perf_counter() - t0) / reps
gflop = (2*na*ns_a*n2*nb + 2*na*nb*nb + 2*na*na*nb) / 1e9
print(f"\nOPENBLAS_NUM_THREADS={os.environ.get('OPENBLAS_NUM_THREADS','default')}")
print(f"per sigma   {dt*1e3:.2f} ms  ->  {1/dt:.2f} sigma/s, {gflop/dt:.1f} GFLOP/s FP64")
print(f"loadavg     {os.getloadavg()[0]:.2f}")
