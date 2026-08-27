# Q8 canonicality repair receipt

Captured 2026-08-24 on `experiment/q-sota-adapter`, based on `9d5f051`. These are targeted repair
readings, not a replacement for `full_grid_gates.log` or a retroactive change to its adjudication.
All heavy runs were serialized.

## Failure-first regression

Command:

```text
cargo test --release -p q8-mps --test canonical_sweep -- --nocapture
```

Before the SVD repair:

```text
left block basis lost canonical form: defect=1.7995051073022862e-5
test strong_coupling_sweep_preserves_both_canonical_bases ... FAILED
```

After the repair, the same test passed. The two-sweep diagnostic gave:

```text
worst_left_canonical_defect=9.079312803079102e-15
worst_right_canonical_defect=3.82904462531506e-15
worst_lanczos_residual=9.55588075207981e-12
```

## Independent returned-state View, N=8 U=16 chi=32

Command:

```text
cargo run --release -p q8-mps --example object_claim_transport -- 8 16 32
```

```text
Door: q-seam E=-1.262136132334283 residual=1.257e-13 iterations=180

neel:
reported(last local) E=-1.262135297768552
returned-state global E=-1.262135297766171
|difference|=2.381206e-12
|E_state-E_exact|=8.345681e-7
canonical(L/R)=(3.191e-14,7.505e-14)
local-residual=3.360e-8
sweeps=6 converged=true

doublon-hole:
reported(last local) E=-1.262135272906008
returned-state global E=-1.262135272905127
|difference|=8.810730e-13
|E_state-E_exact|=8.594292e-7
canonical(L/R)=(8.998e-14,9.082e-14)
local-residual=3.734e-8
sweeps=10 converged=true
```

Both arms passed observable transport, reported-is-state, and the erased-JW mutant check. Their
returned-state energy separation was `2.486104e-8`.

## High-ledger cold-start replays

Commands have the form:

```text
cargo run --release -p q8-mps --example diag_failing_config -- N U chi 20 energy-only
```

### N=8 U=16 chi=64

```text
energy=-1.262136132207331 converged=true sweeps_used=5 wall=16.002s
energy_history=[-0.9336532916004217, -1.1298482818209834, -1.262036004927964,
                -1.2621361322049864, -1.2621361322073312]
discarded_max=4.619518763121798e-12
canonical(L/R)=(3.191356559387316e-14,1.247890679678676e-13)
worst_lanczos_residual=3.796252748630451e-8
```

### N=8 U=16 chi=128

```text
energy=-1.262136132335229 converged=true sweeps_used=5 wall=33.810s
energy_history=[-0.9336532916004217, -1.1298482818209834, -1.262036004927964,
                -1.2621361323352858, -1.262136132335229]
discarded_max=3.0771238815760366e-19
canonical(L/R)=(6.061817714453355e-14,2.728928194528635e-13)
worst_lanczos_residual=3.796252748630451e-8
```

### N=8 U=16 chi=256

```text
energy=-1.262136132335044 converged=true sweeps_used=5 wall=36.315s
energy_history=[-0.9336532916004217, -1.1298482818209834, -1.262036004927964,
                -1.2621361323353, -1.2621361323350442]
discarded_max=-0e0
canonical(L/R)=(6.17284001691587e-14,2.728928194528635e-13)
worst_lanczos_residual=3.796252748630451e-8
```

Against q-seam `E0=-1.262136132334283`, the chi 64/128/256 absolute errors are respectively
`1.26952e-10`, `9.46e-13`, and `7.61e-13`.

### N=10 U=16 chi=256

```text
energy=-1.602785021944129 converged=true sweeps_used=5 wall=368.485s
energy_history=[-1.0976461447264256, -1.3738665223018387, -1.6018430162842492,
                -1.6027850219423385, -1.602785021944129]
discarded_max=3.5723620351007e-21
canonical(L/R)=(2.3225865675158275e-13,1.1577405700791132e-12)
worst_lanczos_residual=2.438911736853758e-7
```

Against the cached q-seam Door `E0=-1.6027850219406818`, the absolute error is `3.45e-12`.

## Test suite

`cargo test -p q8-mps` passed all 16 non-ignored integration tests plus doc tests. The intentionally
high-cost `full_grid_g2_g3_g6_g0_2_g7` test remained ignored by the normal suite. A complete repaired
eight-configuration grid re-adjudication is therefore still owed.
