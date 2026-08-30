#!/bin/bash
# Final assembly and verification, once every stage's energies are cached.
cd "$(dirname "$0")" || exit 1
PY=../venv/bin/python
set -e
echo "=== integral tests ==="
$PY -u test_integrals.py 2>&1 | grep -vE "Warning|rho_cur|return x|alpha ="
echo
echo "=== FCI tests ==="
$PY -u test_fci.py 2>&1 | grep -vE "Warning|rho_cur|return x|alpha ="
echo
echo "=== the guards, each fired on the case it exists for ==="
$PY -u test_runlock.py
$PY -u test_pmap_safety.py
$PY -u test_emit_refusals.py
$PY -u test_cache_kinds.py
$PY -u test_seed_ladder.py
$PY -u test_route_c_enumeration.py
$PY -u test_verify_sections.py
echo
echo "=== spin audit at every geometry (the emitter refuses without it) ==="
NPROC_LIGHT=12 NPROC_HEAVY=9 $PY -u build_curves.py --spin \
    H2 LiH Li2 HF N2 F2 CO He2 Ne2 2>&1 \
    | grep -vE "Warning|rho_cur|return x|alpha ="
echo
echo "=== recertify any geometry whose bound does not cover 50 digits ==="
NPROC_LIGHT=12 NPROC_HEAVY=9 $PY -u build_curves.py --recertify \
    H2 LiH Li2 HF N2 F2 CO He2 Ne2 2>&1 \
    | grep -vE "Warning|rho_cur|return x|alpha ="
echo
echo "=== assemble all nine curves ==="
NPROC_LIGHT=12 NPROC_HEAVY=9 $PY -u build_curves.py --assemble \
    H2 LiH Li2 HF N2 F2 CO He2 Ne2 2>&1 \
    | grep -vE "Warning|rho_cur|return x|alpha ="
echo
echo "=== emit the engine drop, quoting the computed diff ==="
$PY -u emit_engine.py engine_handoff/elements1 2>&1 | grep -vE "Warning|rho_cur"
echo
echo "=== verify ==="
$PY -u verify_elements.py "$@" 2>&1 | grep -vE "Warning|rho_cur|return x|alpha ="
