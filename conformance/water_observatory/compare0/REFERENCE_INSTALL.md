# COMPARE-0 — the reference models, priced and installed: every command and its result

*The freeze's G-R0 says a reference is scored only if it installs here, and that a failure to
install is a recorded VOID with its exact failure string and never a substituted number. This
file is that record. Nothing here is a summary of what usually works; it is what ran, in order,
with what came back.*

**The machine.** No passwordless sudo. Ubuntu 24.04, `g++ (Ubuntu 13.3.0-6ubuntu2~24.04.1)`,
32 cores, 31 GiB of memory SHARED with other lanes of this session. `python3` 3.12 with
`numpy 2.4.0` and `scipy 1.16.3` already present. Cores 18–23 under `taskset` throughout.

**The scratch.** Everything below lives in this worktree's `scratch/`, which is not tracked.
The install is EPHEMERAL: it is gone with the worktree, and re-running it is the reproduction
route, not a saved binary.

## 1. micromamba, user-space (the route the memory note names)

```console
$ cd .../compare0/scratch
$ curl -sSL https://micro.mamba.pm/api/micromamba/linux-64/latest -o mm.tar.bz2
$ ls -la mm.tar.bz2
-rw-rw-r-- 1 emoore emoore 6988090 ... mm.tar.bz2
$ tar -xvjf mm.tar.bz2 bin/micromamba
bin/micromamba
$ ./bin/micromamba --version
2.9.0
```

**Result: OK.**

## 2. The build environment

First attempt, with python and numpy in the environment:

```console
$ MAMBA_ROOT_PREFIX=.../scratch/mamba ./bin/micromamba create -y -p .../scratch/env \
    -c conda-forge autoconf automake libtool make fftw pkg-config python=3.11 numpy
warning  libmamba Download error (56) Failure when receiving data from the peer
    [https://conda.anaconda.org/conda-forge/linux-64/python-3.11.16-h5f976f7_2_cpython.conda]
    Recv failure: Connection reset by peer
warning  libmamba Retrying in 2 seconds
critical libmamba Download error (28) Timeout was reached
    [https://conda.anaconda.org/conda-forge/linux-64/numpy-2.4.6-py311h2e04523_0.conda]
    Operation too slow. Less than 30 bytes/sec transferred the last 60 seconds
```

**Result: FAILED on the network, not on the solve.** The environment did not need a python at
all — the system `python3` already carries `numpy` and `scipy`, and `references.py` runs on it —
so the retry drops both:

```console
$ MAMBA_ROOT_PREFIX=.../scratch/mamba ./bin/micromamba create -y -p .../scratch/env \
    -c conda-forge autoconf automake libtool make fftw pkg-config
Transaction finished
$ ls .../scratch/env/bin | grep -E 'autoreconf|libtool|^make$'
autoreconf
libtool
libtoolize
make
```

**Result: OK.** It was needed: the machine has `g++ 13.3.0` (MBX asks for `≥ 9.3`) but

```console
$ autoreconf --version
bash: autoreconf: command not found
$ libtoolize --version
bash: libtoolize: command not found
$ ls /usr/include/fftw3.h
ls: cannot access '/usr/include/fftw3.h': No such file or directory
$ ldconfig -p | grep -c fftw
    (runtime shared objects only, no headers)
```

so all three of MBX's stated requirements past the compiler came from this environment.

## 3. MB-pol, through MBX

MB-pol's authoritative implementation is MBX, the Paesani group's own C++ library. The
OpenMM route (`paesanilab/mbpol` on anaconda.org, latest `1.1.2`) was looked at and NOT
taken: every build of it pins `python >=2.7,<2.8` / `>=3.5,<3.6` / `>=3.6,<3.7` and an
OpenMM of that era, so it would have needed a second, older, python environment to serve a
plugin wrapping the same physics MBX implements directly.

```console
$ git clone --depth 1 https://github.com/paesanilab/MBX.git
$ git -C MBX log -1 --format='%H %ad'
0e01b75b47611d7d51f27a34b112bdc5e2090a50  Tue Jul 28 18:07:33 2026 -0600
    (README: "MBX v1.4.0"; J. Chem. Phys. 159, 054802 (2023))
```

Build, exactly as MBX's own README's basic installation prescribes, with `--enable-shared`
because that is what its python plugin requires:

```console
$ PATH=.../env/bin:/usr/bin:/bin \
  CPPFLAGS=-I.../env/include LDFLAGS=-L.../env/lib \
  bash -c 'autoreconf -fi && ./configure --enable-shared --prefix=.../mbx-install \
           && taskset -c 18-23 make -j6 && make install'
...
g++: fatal error: Killed signal terminated program cc1plus
compilation terminated.
make[3]: *** [Makefile:2643: potential/3b/libmbx_la-poly_3b_A1_B1C2X2_B1C2X2_deg4_grad_v1.lo] Error 1
```

**Result: FAILED, and the failure is memory, not the toolchain.** `autoreconf` and
`configure` both succeeded; hundreds of objects compiled; the three-body degree-4 polynomial
sources are enormous single translation units and four concurrent `cc1plus` processes were
holding `4.8`–`5.7` GiB of resident memory each on a 31 GiB machine with 25 GiB already in
use by other lanes of this session. This is a machine-contention failure on a shared box and
it is recorded as one — it is not a statement about MBX.

The retry drops the parallelism and the optimisation level on the sources that had not yet
built. `-O1` is a COMPILE-TIME choice and changes no number the library returns:

```console
$ taskset -c 18-23 make -j1 CXXFLAGS="-fopenmp -O1 -std=c++17" && make install
```

<!-- RESULT-3 -->

## 4. TIP4P/2005

Nothing to install. `conformance/water_observatory/compare0/references.py` implements it
directly from Abascal & Vega, *A general purpose model for the condensed phases of water:
TIP4P/2005*, J. Chem. Phys. **123**, 234505 (2005) — the model-parameter table's TIP4P/2005
column, and the citation travels with the numbers into `references.json` itself. Every unit
conversion is `scipy.constants`' (CODATA), so the only typed numbers in the whole reference
are the paper's five parameters and its two geometry values.

**Result: OK**, with the placement cost measured rather than argued (freeze G-P0).
