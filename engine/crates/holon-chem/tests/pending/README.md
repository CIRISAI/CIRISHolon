# tests/pending — gates written ahead of their data

Cargo discovers test targets only at tests/ top level, so files here COMPILE
NOTHING and GATE NOTHING. This directory exists for exactly one situation: a
gate authored before the record it reads has finished computing. A test that
fails because its data has not arrived is a red suite for every lane on the
tree — the elements3 lane's PENDING_ file under engine/output/ is the same
discipline; this is the shared, discoverable version of it.

RULES: a file lands here only with a RESUME note (in the owning lane's
output directory) saying exactly what moves it back; it moves back in the
SAME commit as the data that makes it green; and nothing here counts toward
any gate's discharge — a pending gate is an undischarged gate.
