# H3ERE2-G restart on CIRISHolon

Status: paused experiment recovered from repository evidence; restart authorized by the
CIRISHolon engine slice, but no response-quality result is claimed here.
Date: 2026-08-22.

## What H3ERE2 was

There are two related efforts in this repository:

1. `scratchpad/TRANSITION_MAP/H3ERE2_TUNING/` is the tuned 11+1 kind decomposition
   instrument using DeepSeek-V3.1, Qwen3-235B, and GLM-4.5. It improved held-out deep-kind
   accuracy but was **not licensed** on its full preregistered criteria.
2. `scratchpad/TRANSITION_MAP/H3ERE2G_DESIGN.md` is H3ERE2-G: a generator in which a small
   language model perceives and articulates while a deterministic structural middle
   generates, evaluates, and selects transformations.

The intended split was:

```text
input --Qwen3-0.6B--> typed state --CIRISHolon--> chosen transformation
      <--Qwen3-0.6B-- rendered action <--------- certificate
```

The language model was deliberately not the safety-critical reasoner. It decomposed input
into typed state and rendered an already-selected result. Reachability, equivalence,
fragility/cost, ordering, conservation, covenant checks, and deferral belonged in the
deterministic middle.

## Qwen3-0.6B work completed before the pause

`crates/ciris-nl` implements a resident native backend (`llama-cpp-2`), a browser/WASM
backend (`rten`), closed-set decoding, and the `NlBridge` boundary. The model-selection and
quantization record is in `NL_BRIDGE.md`.

Measured findings that remain binding:

- SmolLM2-360M was rejected; Qwen3-0.6B was the only sub-gigabyte candidate with measured
  grip on the four surface families.
- The fine-tune was unstable across six runs: mean accuracy 0.763, standard deviation
  0.078, range 0.663–0.880, with 38/92 test items changing answer between runs.
- The V1 Rules instruction was a real shippable improvement for Q4_K_M: 0.641 to 0.717,
  driven by Rules recall, while also improving F16. It raised the curve rather than
  eliminating the quantization gap.
- Full-precision runtimes disagreed as much as some quantized comparisons. Only
  within-runtime, within-tokenizer comparisons are admissible for quantization claims.
- The native bridge still needs the Qwen chat template applied; raw completion was a
  recorded train/serve skew. Browser execution—not merely native loading of the ONNX
  backend—remains a required gate.

## Why the response-quality run was paused

The preregistered A/B/C test was sound in shape:

- A: Qwen3-0.6B answers directly;
- B: the full pipeline with symmetry-preserving scrambled off-diagonal couplings;
- C: the full pipeline with real couplings.

B was the load-bearing placebo because it held architecture, call count, token budget,
renderer, and weight multiset fixed while destroying only the coupling assignment.

The implementation was not yet a valid per-item dynamics experiment:

1. The shipping q4f16 encoder mapped all 170 wild inputs to `Facts`. The resulting 2,040
   responses were declared **void**; every real-arm path was identical.
2. fp32 removed total collapse, yielding 109 `Facts` and 61 `Manner` (V1 added five
   `Rules`), showing quantization damage was the main cause, with residual domain shift.
3. Even a perfect hard encoder still emitted one argmax family. The dynamics therefore
   had at most four initial states and generated category-level paths, not item-level
   reasoning. Amendment A2 correctly required seeding from the full probability
   distribution.
4. The old graph integrator group-averaged the coupling matrix. Its striking twin motion
   was consequently imposed by symmetrization and survived scrambled couplings. That
   negative result is now fenced by `Core/TwinTransport.lean`; it must not be rediscovered
   as evidence for reasoning.

The response runner and its 2,040 response artifacts are not committed as an executable
pipeline. What is committed is the Qwen bridge, the H3ERE2-G design, the preregistration,
both amendments, and the invalidation record.

## CIRISHolon restart

The restart uses the new engine as an actual structural middle rather than renaming the old
fixed path ordering:

1. Apply the correct Qwen chat template and choose a model/checkpoint on a held-out,
   seed-robust criterion.
2. Preserve the complete calibrated surface distribution and uncertainty as whole-state;
   do not collapse it to an argmax.
3. Build one root holon per item. Typed commitments are child holons; REG+ gross state,
   whole-only state, channels, boundaries, and grain are explicit.
4. Generate candidate transformations under the ontology's reachability and
   irreversibility rules. Evaluate covenant constraints on the resulting root state, never
   as independent per-move scores. `defer` is a first-class outcome.
5. Allocate a macro error budget and let CIRISHolon refine only boundaries whose residual
   can change the selected transformation. Record the selected grain and certificate.
6. Keep A/B/C, but scramble only the load-bearing dynamics while preserving all declared
   invariants and cost. Add an identity/control arm proving that an isomorphic relabeling
   changes nothing.
7. Before judging prose, require per-item path diversity materially above four, certificate
   validity on every item, no systematic grain-floor censoring, equal compute budgets, and
   deterministic replay.
8. Judge C vs B pairwise, blind, both presentation orders, as preregistered. C vs A remains
   an engineering secondary and cannot rescue a failed real-vs-placebo primary.

This makes H3ERE2-G a good evaluation target for CIRISHolon, not a release dependency. The
engine ships only on its own physical/certificate gates; H3ERE2-G earns its own license from
the response-quality experiment.
