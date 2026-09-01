# SATURATION-3 G2: the tightest honest CPU arm.
#
# The GPU arm wins by reformulating sigma as three GEMMs. That reformulation is not a GPU
# trick -- it would work on the CPU too, against a tuned BLAS. Quoting the GPU speedup
# against holon-chem's hand-written sigma_direct while the GPU gets a reformulation plus
# cuBLAS would be quoting it against a baseline defect, which is exactly what
# holon-gpu/src/cpu.rs's header warns about.
#
# So this measures the SAME three GEMMs on the CPU through OpenBLAS, at the real (O,O,O)
# shapes, to bound what a reformulated CPU sigma could reach.
import numpy as np, time, os

na = nb = 455; ns_a = 48; n2 = 225
reps = int(os.environ.get("REPS", "10"))

rng = np.random.default_rng(7)
A  = rng.standard_normal((na, ns_a, n2))
T  = rng.standard_normal((na, n2, nb))
C  = rng.standard_normal((na, nb))
Fa = rng.standard_normal((na, na))
Fb = rng.standard_normal((nb, nb))

def sigma_once():
    S = C @ Fb                 # beta same-spin
    S += Fa.T @ C              # alpha same-spin
    D = A @ T                  # mixed, batched over alpha strings
    return S, D

sigma_once()
t0 = time.perf_counter()
for _ in range(reps):
    sigma_once()
dt = (time.perf_counter() - t0) / reps

gflop = (2*na*ns_a*n2*nb + 2*na*nb*nb + 2*na*na*nb) / 1e9
print(f"threads available   {os.cpu_count()}  (OPENBLAS_NUM_THREADS={os.environ.get('OPENBLAS_NUM_THREADS','default')})")
print(f"flops per sigma     {gflop:.3f} GFLOP")
print(f"per sigma           {dt*1e3:.2f} ms  ->  {1/dt:.1f} sigma/s, {gflop/dt:.1f} GFLOP/s FP64")
print(f"loadavg             {os.getloadavg()[0]:.2f}")
