# Source before any Python in this lane.
#
# ONE BLAS THREAD PER PROCESS.  scipy's bundled OpenBLAS is built with
# MAX_THREADS=64 and, unset, builds a spin-waiting pool of 63 threads in EVERY
# worker.  Measured on the sibling lane's 12-worker pool: 756 threads, 148% CPU
# per worker for 100% of work, six cores of a shared 32-core machine burned on
# nothing, and a load average of 83 that four other campaigns were paying for.
# This work is mpmath-bound; the only BLAS in the hot path is a sparse matvec,
# single-threaded regardless.  Must be set BEFORE numpy is imported.
export OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1
export NUMEXPR_NUM_THREADS=1 VECLIB_MAXIMUM_THREADS=1
