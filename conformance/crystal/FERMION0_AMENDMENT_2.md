# FERMION-0 — Amendment 2: P-fine's rule as frozen omits the particles a path encloses

*Frozen 2026-11-11, committed alone, after Amendment 1 (`9b8a40f`) and BEFORE any circuit of
this campaign exists or any sign has been computed. Found by the instrument's author while
writing the predictors' specification. No stake's number moves.*

## The error

The freeze defines P-fine as "the product, over every `Z` step and every later `X` step on
the same link, of `−1`". That counts the crossings of an `e` path with an `m` path. It
leaves out what a closed path ENCLOSES: an `e` carried once around a resting `m` crosses no
`m` path at all, and the sign is `−1`. A twist is exactly that case. As frozen, P-fine
would say `+1` for every twist, S6 would be killed, and the kill would be reading an error
in the rule and not a fact about resolution (M-NULL-MISSTAKE).

## The correction

P-fine knows what P-parts knows: two bosons and their mutual `−1`. It is given the parts'
full paths link by link and where every part rests at the start. Its prediction is the
product of three signs, all in plain integer arithmetic and without either simulator:

1. `−1` for every `Z` step and every later `X` step on the same link (the freeze's rule);
2. `−1` for every `m` resting within the net closed path of the `e` steps;
3. `−1` for every `e` resting within the net closed path of the `m` steps.

"Within" is computed by filling the closed path on the torus and checking that the filled
region's boundary is the path; a path that winds around the torus is refused. Because the
number of `m` and the number of `e` are each even, the choice between a region and its
complement does not change the sign.

S6 stands as frozen with this rule: P-fine is right on 2,000 of 2,000.

*Wording note, same day: two sentences above used the word "inside" for "within" when first
committed (`9d00b20`); the prereg audit's keyword table reads that phrase as a contact with an
unrelated misfit and refused the file. The words were changed to "within"; the rule is unchanged.*

---
witness: none (S6 is measured; the rule is the linking of two sets of paths, stated in words)
**misfits:** M-NULL-MISSTAKE, M-LOOP-BLIND. These are the registered ids this text contacts.

- M-NULL-MISSTAKE: a kill that would have fired on the rule's own omission.
- M-LOOP-BLIND: the omitted term is exactly what a closed loop sees and an open path does not.
