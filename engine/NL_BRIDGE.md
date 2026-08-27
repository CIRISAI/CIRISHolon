# The NL bridge — engine selection **[decided 2026-08-23]**

The H3ERE2-G architecture puts an SLM at perception and expression only: it decomposes
input into typed tuples and renders a chosen action. The reasoning between is symbolic
and lives in `ciris-sim-core`, which is `no_std` with no allocator. This document picks
the inference engine and states what it costs.

Scouted 2026-08-20/22 against live registries; three load-bearing claims re-verified
independently before adoption.

## THE DECISION — it splits, and that is the finding

| target | engine | version | why |
|---|---|---|---|
| native CPU (priority 1) | **llama-cpp-2** | `0.1.154` | the ONLY candidate that runs current Gemma today, with the only battle-tested quantised CPU kernel story |
| wasm (priority 2) | **rten** + `rten-generate` | `0.25` | 1.69 MB wasm, both targets build clean, WASM SIMD works, leanest pure-Rust dependency graph (47) |
| both | **llguidance** | `1.8` | engine-agnostic constrained decoding — survives an engine swap |

**No single engine covers both targets.** Pretending otherwise would be the easy error.

## VERIFIED INDEPENDENTLY (not taken on report)
- `google/gemma-4-E2B-it-qat-q4_0-gguf`: **`license: apache-2.0`, `gated: False`**. Gemma 4
  (2026-03-31) dropped the bespoke Gemma ToU. **We may ship Gemma 4 GGUF in our own
  installer** under Apache §4(a)-(d). Gemma 3/3n remain gated under the old ToU.
- Versions confirmed on crates.io: llama-cpp-2 0.1.154, rten 0.25.0, llguidance 1.8.0,
  candle-core 0.11.0.

## THE WALL — and it is a product decision, not a library one
**Gemma 4 is not browser-deployable at any quantisation.** Its ONNX export is ~3.6 GB
(1864 MB decoder + 1763 MB embeddings); even the mobile QAT build is ~2.3 GB. No engine
in the field has a memory64 path, so all are capped at **4 GB linear memory**, and
mmap is unavailable on wasm, so weights must land in that same memory.

E2B is also a size trap: its Per-Layer Embeddings table is ~2.35B of 5.1B params, so the
"2B" model ships a **3.35 GB** q4_0 GGUF.

**Consequence, stated plainly: two models forever.** Gemma 4 native, and something
sub-1 GB in the browser — two prompt formats, two quality bars, two eval sets. Viable
browser candidates: SmolLM2-360M-Instruct q4 (**388 MB, Apache-2.0**) or Qwen3-0.6B
(618 MB). Gemma-3-270m is 273 MB but carries the OLD ToU — gated, NOTICE file, and
downstream use restrictions we would have to impose on our own users. **Prefer
SmolLM2 and pay the 115 MB.**

## RULED OUT, with the specific blocker
- **candle** (the popular choice): wasm build **fails on the released crate** with
  `cannot find type CurrentCpuF16` under `+simd128` — the exact flag its own
  `.cargo/config.toml` sets. Open issue #3835, two unmerged fixes. Works only in scalar
  mode, discarding most CPU throughput. Root cause is structural: `grep -rn wasm
  .github/` in huggingface/candle returns **nothing** — zero wasm CI behind 11 in-tree
  browser demos. Also: `gemma4` exists but uses `VarBuilder`/`Linear`, **not `QMatMul`** —
  there is no `quantized_gemma4`, so no GGUF path at all.
- **mistral.rs**: crates.io 0.8.1 is 4 months behind git v0.9.2; zero wasm; architecturally
  a server (unconditional tokio/reqwest/hf-hub).
- **llama-cpp-2 for wasm**: `build.rs` `panic!`s on any triple outside
  Windows/Apple/Linux/Android. Emscripten↔wasm-bindgen ABI mismatch means a naive patch
  would not suffice.

## WE DO NOT NEED vLLM's FEATURES — and this is the argument for rejecting the featureful option
Continuous batching amortises across *concurrent independent* requests; our calls are
sequential within a turn. Paged attention fixes KV fragmentation under long variable
contexts; ours are short prompts and short outputs. What actually matters is low
per-call overhead and **prefix caching** for the fixed system prompt — both backends give
that cheaply. Adopting a server-shaped engine would buy a tokio dependency tree for
nothing measurable.

## COEXISTENCE WITH THE no_std CORE
`resolver = "2"` must be set **explicitly** — a virtual workspace manifest does not
inherit `edition`, so resolver 2 is not implied. But resolver 2 does NOT stop feature
unification between two members built together, so the real protection is procedural:
**never let `cargo build --workspace` be the proof of no_std compliance.** CI must build
the core alone:
```
cargo build -p ciris-sim-core --no-default-features --target wasm32-unknown-unknown
cargo build -p ciris-sim-core --no-default-features --target wasm32-wasip1
```
Feature-gate the engines (`native` → llama-cpp-2, `web` → rten) behind one `NlBridge`
trait, so the symbolic layer never sees an engine and a machine without a C++ toolchain
can still build the physics core.

## THE HOLE — close this before committing
**There are no CPU latency or cold-start numbers for any engine on small models.** Every
published benchmark is GPU, Apple Silicon, or Xeon-AMX. Per-call overhead on generic x86
— precisely our axis — is unmeasured everywhere. That needs a spike, not more desk
research, and it should happen before we write code against either engine.

Second risk worth pricing: `rten-generate` has ~1,211 recent downloads. It is the
least-exercised code in this recommendation and it runs our decode loop. rten has the
best CI discipline of the seven, but expect to read its source.

---

## Model decision VERIFIED against the artifacts — 2026-08-22

`model-scout` recommended **SmolLM2-360M-Instruct**, overturning the earlier
"two models forever" conclusion (which had anchored on Gemma 4 at 3.35GB). I checked its
own flagged open items against the real files rather than accepting the inference. **The
recommendation survives**, and three integration facts came out of the check that the
brief did not have.

### Confirmed by inspecting the actual ONNX graph
Range-fetched the first 6MB of `HuggingFaceTB/SmolLM2-360M-Instruct/onnx/model_q4.onnx`:

| op | count | domain |
|---|---:|---|
| `RotaryEmbedding` | 256 | com.microsoft |
| `MatMulNBits` | 224 | com.microsoft |
| `GroupQueryAttention` | 128 | com.microsoft |
| `SimplifiedLayerNormalization` | 65 | com.microsoft |

**Open item #3 CLOSES CONFIRMED** — the file really is on rten's documented `MatMulNBits`
int4 path. But note what else is in there: this export is **built from ORT fused contrib
ops**, not plain ONNX. rten's `llama.rs` example proves the *architecture*; it does not by
itself prove *this export*. So I checked the op registry directly (rten 0.25.0 source):
all four are implemented and registered, `RotaryEmbeddingMicrosoft` exists as a distinct
contrib variant, and rten's own test fixtures are named `LlamaMSFT`. rten supports this
export deliberately, not incidentally.

### NEW — three facts for whoever wires this up

1. **The contrib ops are behind a Cargo feature.**
   `register_op!("com.microsoft", GroupQueryAttention, feature = "contrib")`.
   It IS in `default`, so a plain dependency works — **but the wasm build must not set
   `default-features = false`**, which is exactly the reflex for trimming browser binary
   size. Doing so drops `contrib` and the model fails to load *at all*. If features are
   trimmed, `contrib` and `onnx_format` must be re-added explicitly.
2. **`wasm_api` is a non-default feature** — it must be enabled for the browser target.
3. **Open item #2 CLOSES NEGATIVE: rten cannot run `q4f16`.**
   `impl Operator for MatMulNBits` requires `TensorView<f32>` for both activations and
   scales, and declares `OutputType::Fixed(DataType::Float)`. There is no f16 path.
   The scout's conservative use of the `q4` files was **correct**, and the hoped-for
   halving of every browser payload **is not available**. The real browser payload is
   **387.94MB**, not ~200MB. This makes the SmolLM2-vs-Qwen3 gap *more* decisive, not
   less: Qwen3-0.6B's browser cost stays at 919MB, which is not viable on mobile Safari.

### Verdict
**Pin SmolLM2-360M-Instruct.** One model, all three platforms, first-party GGUF *and*
first-party ONNX, both apache-2.0 with no re-publisher in the licence chain — the exact
place every other candidate breaks. Native `Q4_K_M` 270.6MB, browser `model_q4.onnx`
387.94MB, tokenizer 2.10MB.

Unchanged and still owed: **no model at any size has been evaluated on an 11-category
typed-tuple taxonomy.** Published IFEval numbers cannot settle SmolLM2-vs-Qwen3 for our
task; only our own eval set can, and we are fine-tuning regardless.

---

## Measured CPU latency, and a loader correction — 2026-08-22

`inference-scout` measured the runtime hole on real hardware (i9-13900HX, AVX2/FMA, **no
AVX-512** — a Xeon/EPYC or Zen4+ box would read better). Nothing below is estimated.

### llama-cpp-2 native — settled, act on all three
| finding | number |
|---|---|
| cold start (process → first token) | **0.5–0.9 s**, and **prefill-dominated**, not load-dominated (GGUF is mmap'd; model_load is only 64ms at 360M) |
| per-call, 200-in / 30-out, 8 threads | SmolLM2-360M **602ms p50 / 690ms p95**; Qwen3-0.6B 742/842 |
| decode throughput | 7.8 ms/tok (SmolLM2 Q4) · 13.3 ms/tok (Qwen3 Q4) |
| **prefix reuse** | **1.56× (Qwen3) to 2.17× (SmolLM2) on p50 — and 2.35× on p95** |

Three binding consequences:
1. **Keep the inference process resident.** Cold start amortises to zero if it lives;
   fork-per-call pays 0.5–0.9s every time. Highest-priority integration constraint.
2. **Prefix reuse belongs in the first implementation, not a later optimisation.** ~10
   lines against `clear_kv_cache_seq`. It beats everything else on the list, it cuts the
   *tail* harder than the median, and the saving grows linearly with system-prompt length
   while its cost stays flat — measured with only a 150-token prefix, ours will be longer.
3. **Set threads to P-core count (8 here), not `num_cpus`.** Prefill and decode scale
   *oppositely* — prefill improves to 32 threads, decode peaks at 8 and degrades as it
   spills onto E-cores. `n_threads` and `n_threads_batch` should be set separately.

### rten — the benchmark measured the WRONG ARTIFACT; numbers are provisional
The scout hit a real failure — `rten-convert` rejected the prebuilt HF ONNX with **161
unconvertible operators**, naming exactly the `com.microsoft` contrib ops — and worked
around it with a self-exported fp32 (1638MB) and a dynamic-QUInt8 (553MB) build. **The
diagnosis was right; the workaround was unnecessary.** rten 0.25.0 has **two loaders with
different op coverage**:

| loader | format | `com.microsoft` registrations |
|---|---|---:|
| `onnx_loader` / `onnx_registry.rs` | `.onnx` **direct** | **7** (incl. GQA, RotaryEmbedding, SimplifiedLayerNorm) |
| `rten_loader` / `rten_registry.rs` | `.rten` (what `rten-convert` emits) | **0** |

`Model::load_file` auto-detects file type and the docs state models load "from either
`.onnx` or `.rten`". So the prebuilt **387.94MB `model_q4.onnx` loads directly**, contrib
ops included, with no conversion step to own. Consequences:

- The 1.6GB fp32 intermediate is not a cost we have to pay.
- **"int8 made rten slower" does NOT transfer to q4.** Dynamic QUInt8 wraps every MatMul
  in quantise/dequantise, which is why it lost at batch-1 GEMV. `MatMulNBits` is a *fused*
  int4 kernel with an `accuracy_level` attribute — a different code path, not the same trade.
- **rten's speed on the shipping artifact is therefore UNMEASURED.** The reported 2.1×
  (fp32) and 4.5× (int8) gaps behind llama.cpp are provisional.
- Re-measure at **our true output length**. The scout's sharpest observation: rten's
  *prefill is faster* than llama.cpp's (250ms vs 364ms) and only *decode* loses (33.7 vs
  7.8 ms/tok). A terse typed-tuple output is prefill-dominated, where rten may win outright.

### Open blocker, independent of the above — ranks above every performance number
`rten-text`'s `Tokenizer::from_file` fails with `BpeError(MissingVocabEntry("Ą"))` on both
the original and a freshly-exported `tokenizer.json` — an rten-text BPE gap, not a corrupt
file. Bypassed for timing with synthetic token IDs. **This is a genuine shipping blocker
for rten with this model family** until the `tokenizers` crate is swapped in.

### Not measured, stated plainly
No Gemma 4 number (3.35GB, impractical here — these are the *shape* of the overhead, not
Gemma-specific; do not extrapolate linearly). No llguidance mask overhead. **No wasm
runtime performance at all** — everything is native x86; browser will be worse
(single-threaded, no mmap). One machine, no AVX-512.

### Direct-ONNX load: VERIFIED BY EXECUTION, not inference — 2026-08-22
The loader correction above was made from source reading. I then ran it, because a
correction issued to another agent on unexecuted evidence is exactly the failure mode we
police elsewhere. `rten 0.25` + `Model::load_file` on the **unmodified prebuilt**
`HuggingFaceTB/SmolLM2-360M-Instruct/onnx/model_q4.onnx` (387,943,246 bytes, size-exact):

```
LOAD OK in 192.012719ms          <- no rten-convert, no self-export, contrib ops included
inputs  = 67   input_ids · attention_mask · position_ids · past_key_values.{0..31}.{key,value}
outputs = 65   logits [batch, seq, 49152] · present.{0..31}.{key,value}
```

Confirmed: the `com.microsoft` contrib ops load through the direct-ONNX path, the KV-cache
plumbing is intact (5 KV heads, head_dim 64), and **load is 192ms — comparable to the
self-exported int8 (187–194ms) and 2.6x faster than the fp32 (487–506ms)**, at a third of
fp32's size. The 1.6GB intermediate and the permanent conversion step are not costs we owe.

---

## The pin flip's browser cost, MEASURED — 2026-08-22

The 4-way surface eval convicted SmolLM2-360M (0.250 = uniform chance, below the 0.370
majority class) and gave Qwen3-0.6B real grip (0.467, p=0.035; McNemar vs SmolLM2
p=0.00119). So the pin flips on capability. The open question was the browser cost, which
had been quoted at 919MB. I proposed a rescue and then tested it. **The rescue fails.**

| candidate | file rten can load | size | `Model::load_file` |
|---|---|---:|---|
| SmolLM2-360M | `onnx/model_q4.onnx` | **387.9 MB** | **OK, 192 ms** |
| Qwen3-0.6B | `onnx/model_q4.onnx` | **919.1 MB** | **OK, 831 ms** |
| Qwen3-0.6B | `onnxruntime/cpu_and_mobile/cpu-int4-kld-block-128/model.onnx` | 524.6 MB | **FAILS** |

```
LOAD FAILED: in node "/model/embed_tokens/GatherBlockQuantized":
  com.microsoft/GatherBlockQuantized operator not supported or not enabled
```

**The hypothesis was right and the conclusion is still negative.** I predicted the 919MB
was inflated by an unquantised embedding table. It is — confirmed by the loaded graph:
Qwen3-0.6B's vocab is **151,936** against SmolLM2's **49,152**, a 3.09x larger embedding
table, and `MatMulNBits` quantises MatMul weights only, never the Gather-based embedding
lookup. ORT's `cpu_and_mobile` export gets to 524.6MB precisely *by* quantising that table
— which requires `GatherBlockQuantized`, an op rten does not implement. So the 1.75x saving
is real and **unavailable to us**.

### What this actually forces
**"One model, every platform" and "a model that works" are now in tension**, and that is
the decision, not a detail:

- **Native is settled.** Qwen3-0.6B GGUF Q4_K_M is 484MB through llama-cpp-2 — fine
  everywhere, and it is the only sub-gig candidate with measured signal.
- **Browser is not.** 919MB is not viable on mobile Safari. SmolLM2 fits at 388MB but
  reads *floor at both granularities*, so shipping it in the browser ships a model that
  cannot do the task — worse than shipping nothing.

Three honest routes, none free: accept desktop-browser-only at 919MB; implement
`GatherBlockQuantized` in rten (an upstream contribution, and the only route that gets
Qwen3 to 524.6MB); or find a model with a small vocabulary and real 4-way grip. **Vocabulary
size — not parameter count — is the binding constraint on browser payload**, which is the
generalisable lesson and should govern the next model search.

Unchanged: `onnx-community` declares no licence on any of these files, so the export must
be reproduced from the Apache-2.0 original regardless of which variant wins.

---

## CORRECTION: q4f16 DOES run in rten — my earlier note was wrong — 2026-08-22

Above I recorded "Open item #2 CLOSES NEGATIVE: rten cannot run `q4f16`", reasoning from
`impl Operator for MatMulNBits` declaring
`OutputType::Fixed(ValueType::Tensor(DataType::Float))`. **That reading was wrong.** That
declaration governs the operator's *output* dtype, not what the model file may *store*.
`inference-scout` tested it rather than accepting my source argument, and q4f16 loads,
runs, and produced byte-identical generated text against q4 on a matched prompt.

I have re-tested the consequence for both candidates. Every figure below is
`Model::load_file` on the unmodified published file:

| model | variant | size | load |
|---|---|---:|---|
| SmolLM2-360M | `model_q4.onnx` | 387.9 MB | OK 192 ms |
| **SmolLM2-360M** | **`model_q4f16.onnx`** | **272.7 MB** | OK (scout) |
| Qwen3-0.6B | `model_q4.onnx` | 919.1 MB | OK 831 ms |
| **Qwen3-0.6B** | **`model_q4f16.onnx`** | **569.8 MB** | **OK 594 ms** |
| Qwen3-0.6B | ORT `cpu-int4` (quantised embeddings) | 524.6 MB | **FAILS** — `GatherBlockQuantized` |

**The browser floor is 30–38% lower than recorded.** The `GatherBlockQuantized` failure
and the vocabulary analysis both still stand — 151,936 vs 49,152 is still why Qwen3 is
larger — but the flip's browser cost is **569.8 MB vs 272.7 MB (2.09x)**, not 919 vs 388.

### This materially strengthens the pin flip, on three independent counts
1. **569.8 MB is a shippable browser payload** where 919 MB was not.
2. **`rten-text` works on Qwen3 and FAILS on the SmolLM2 family** (`MissingVocabEntry("Ą")`
   at *load*, both 360M and 135M — family-wide, not one bad file). The tokenizer blocker
   is a SmolLM2 problem, not a Qwen3 one, so the flip removes it rather than inheriting it.
3. Qwen3-0.6B is the only sub-gig candidate with measured 4-way grip (0.467 vs SmolLM2's
   0.250 floor).

**Recommendation: flip the pin to Qwen3-0.6B on both targets** — native GGUF Q4_K_M 484 MB
via llama-cpp-2, browser `model_q4f16.onnx` 569.8 MB via rten direct-ONNX. The licence
obligation is unchanged and real: `onnx-community` declares no licence, so that export must
be reproduced from the Apache-2.0 original.

**Owed before shipping q4f16:** its numerical equivalence is verified on ONE short greedy
prompt. Identical output there does not establish equivalence across the distribution —
run the 4-way eval on q4f16 specifically before it ships.

---

# DECISION: the pin is Qwen3-0.6B — 2026-08-22 (Eric)

SmolLM2-360M-Instruct is **unpinned**. It reads floor at both granularities (12-way 0.060 —
statistically indistinguishable from chance and below the majority class; 4-way 0.250 —
exactly uniform chance), its collapse target moves with the prompt, and `rten-text` cannot
load its tokenizer at all. It is smaller and cleaner and it does not work.

## Pinned artifacts

| target | repo → file | size | licence |
|---|---|---:|---|
| **native** (llama-cpp-2) | `unsloth/Qwen3-0.6B-GGUF` → `Qwen3-0.6B-Q4_K_M.gguf` | **396.7 MB** | **apache-2.0**, ungated |
| **browser** (rten direct-ONNX) | `model_q4f16.onnx` — **must be self-produced**, see below | **569.8 MB** | — |
| **tokenizer** | `Qwen/Qwen3-0.6B` → `tokenizer.json` | 11.4 MB | apache-2.0 |

**Use unsloth's GGUF, not bartowski's.** The two disagreed in the scouting reports and both
were right about different files: `bartowski/Qwen_Qwen3-0.6B-GGUF` is **484.2 MB and declares
no licence**; `unsloth/Qwen3-0.6B-GGUF` is **396.7 MB and declares apache-2.0**. Smaller *and*
licensed — strictly better on both axes, so there is no trade to weigh.

**The browser ONNX must be reproduced by us.** `onnx-community/Qwen3-0.6B-ONNX` declares no
licence on any file, including the `q4f16` we measured. The upstream `Qwen/Qwen3-0.6B` LICENSE
is unmodified stock Apache-2.0 with no acceptable-use addendum, so the grant is real and the
gap is purely the re-publisher's undeclared artifact. Export from the Apache-2.0 original.

## Why this model
- **The only sub-gig candidate with measured grip.** 4-way surface: 0.467 vs SmolLM2's 0.250
  floor, McNemar p = 0.00119. Reaches 86% of a 14B control on the same task, unfine-tuned.
- **`rten-text` loads its tokenizer**, and fails family-wide on SmolLM2 (360M *and* 135M).
  The flip removes the tokenizer blocker rather than inheriting it.
- Loads in rten with **zero conversion** — `Model::load_file`, 594 ms, contrib ops included.

## Costs accepted, stated plainly
- **2.09x the browser payload** of SmolLM2 (569.8 vs 272.7 MB). Root cause measured and
  unavoidable: vocab 151,936 vs 49,152. `MatMulNBits` never touches the Gather-based
  embedding lookup, and ORT's 524.6 MB embedding-quantised export **fails to load**
  (`GatherBlockQuantized` unimplemented in rten). 569.8 MB is heavy for mobile Safari.
- **We own the ONNX export** in perpetuity.
- **q4f16 numerical equivalence rests on ONE short greedy prompt.** Run the 4-way eval on
  q4f16 specifically before it ships.

## Engine split, unchanged but for a new reason
llama-cpp-2 native / rten wasm still holds — but performance no longer decides it. With
prefix reuse the gap is **94 ms vs 111 ms (~15%)**, not 2x. Portability and tokenizer
support decide it now.

---

## The browser floor is 569.8 MB, and ONE op is why — 2026-08-22

Tested every published Qwen3-0.6B ONNX variant by execution (`Model::load_file`, rten 0.25).
The result is a clean dichotomy with a single cause:

| variant | size | embedding table | rten |
|---|---:|---|---|
| `onnx/model_q4.onnx` | 919.1 MB | **unquantised** (fp32) | **LOADS** 831 ms |
| `onnx/model_q4f16.onnx` | **569.8 MB** | **unquantised** (fp16) | **LOADS 594 ms** |
| ORT `cpu_and_mobile/cpu-int4-kld-block-128` | 524.6 MB | quantised | **FAILS** |
| `Qwen3-0.6B-DQ-ONNX` q4f16 (+ external data) | 355.5 MB | quantised | **FAILS** |

Both failures are the same node and the same op:
```
/model/embed_tokens/GatherBlockQuantized:
  com.microsoft/GatherBlockQuantized operator not supported or not enabled
```

**Every export small enough to want is small *because* it quantises the embedding table,
and every one of those needs `GatherBlockQuantized`, which rten does not implement.** The
two that load are exactly the two that leave the embedding unquantised. `model-scout`
independently confirmed the mechanism by range-fetching the 919MB graph and finding
`model.embed_tokens.weight` feeding a **plain `Gather` with no dequantise node**, and the
arithmetic closes: 919.1 − 569.8 = 349.3 MB ≈ a 151936×1024 embedding (155.6M params) in
fp32 plus ~13.9M fp32 MatMulNBits scales.

### Consequences
- **The browser payload is 569.8 MB today.** Not 524.6 and not 355.5 — those do not run.
  This supersedes the "~525 MB, about 1.35x" reading, which assumed the ORT export loads.
- **`GatherBlockQuantized` is a single, well-scoped unlock worth 214 MB (−38%).** Implementing
  it in rten — one contrib op, upstream contribution — takes the browser artifact to
  **355.5 MB**, below even SmolLM2's 387.9 MB q4. That is now the highest-leverage piece of
  engine work available, and its payoff is measured rather than estimated.
- The earlier note that q4f16 is "dead to rten" was **my error**, already corrected: q4f16
  runs. What kills these two files is the embedding op, not the f16 scales.

**Licence unchanged:** every `onnx-community` repo declares none, so we own the export
whichever variant wins.

---

## CORRECTION: it was never "exactly one operator" — 2026-08-22

The section above is titled "the browser floor is 569.8 MB, and ONE op is why". **That was
wrong, and the error is instructive.** I concluded "one blocking op" from the loader's error
message — but `Model::load_file` **fails at the first unsupported node and stops**. A load
error enumerates the first blocker, never the complete set. To enumerate blockers you must
implement the first one and re-run, or scan the whole graph for op types. I did neither and
generalised from a single message.

`sim-portability` implemented `GatherBlockQuantized`, re-ran, and hit the second wall:

| model | GatherBlockQuantized | MatMulNBits inputs |
|---|---|---|
| DQ 355.5 MB | 1 node | **197 nodes, ALL 4-input** (zero_points) |
| ORT int4 524.6 MB | 1 node | **197 nodes, ALL 4-input** |
| base 569.8 MB (works) | none | 196 nodes, all 3-input (symmetric) |

rten's `MatMulNBits` rejects `zero_points`, and that is **not** a small fix: the zero point 8
is hardcoded in rten-gemm's SIMD kernels in six places across three code paths, including an
int8 dot-product path where the correction term is a single constant times the LHS sum.
Making it per-block touches hot kernels on multiple ISAs.

So **adopting either published small export is off the table**, and that conclusion is
independent of the operator now implemented.

### The route that does work — and is a better experiment
The baseline that already runs holds its embedding as a plain fp16 `Gather` on
`model.embed_tokens.weight`, FLOAT16 [151936, 1024] = **311.2 MB of its 569.5 MB**.
Everything else in it is already symmetric 3-input `MatMulNBits` that rten runs today. So:
**quantise only the embedding of the working model and swap that one `Gather` for a
`GatherBlockQuantized`.** Expected ~350 MB — the full saving — needing only the op just
written and touching nothing else.

It is also strictly better evidence than comparing two unrelated exports: every other weight
stays **bit-identical to the baseline**, so the only difference between 569.8 MB and ~350 MB
is the embedding quantisation itself. A controlled comparison rather than a confounded one.

### The operator itself is verified, and separately from the payload
63 single-node cases against **ONNX Runtime 1.29 itself** — the reference implementation,
not a reading of the spec — sweeping bits {2,4,8} x block_size {16,32,128} x with/without
zero_points x f32/f16 scales x rank 2 and 3 x ragged block counts x negative and 2-D indices.
**All 63 bit-exact.** The suite was mutation-tested with five mutants, each caught by a count
matching the case design (nibble-order swap by exactly the 21 bits=4 cases; flat-index
zero-point addressing — the bug ORT's own source comments warn about — by 21; wrong default
zero point by exactly the 18 no-zero-point cases; negative-index wraparound by exactly the 18
negative-index cases).

**Op-level correctness and payload-level quantisation impact are being reported as two
separate numbers**, because the baseline holds the embedding in fp16 and the small one in
4-bit blocks — they are not the same model and their logits cannot match to fp16 precision.
The exact result must not be allowed to launder the approximate one.

---

## TWO RECORDED CLAIMS FALSIFIED — 2026-08-22

### 1. "`rten-text` works on Qwen3" is FALSE — it corrupts prompts silently
I recorded, as one of **three** stated reasons the pin flip was justified, that "`rten-text`
works on Qwen3 and FAILS family-wide on SmolLM2, so the flip removes the tokenizer blocker
rather than inheriting it." **The second half is wrong.** `rten-text` loads Qwen3's tokenizer
and then drops newlines on encode:

```
text      = "A\nB\nLabel: "
rten-text -> [32, 33, 2476, 25, 220]            (n=5)
tokenizers-> [32, 198, 33, 198, 2476, 25, 220]  (n=7)
probe "\n": rten=[]   hf=[198]
```

`encode("\n")` returns an **empty token list**. The model never sees the newlines. The first
working web build returned `Facts` for all four cases; unconstrained it emitted
`" ?\nA. Facts\nB. Rules\nC. Identity\nD. M"` — it was reading a run-on prompt as a quiz to
continue rather than a question to answer.

**This is worse than the SmolLM2 failure, not better.** SmolLM2 fails loudly at load. Qwen3
loads and degrades output invisibly. **The flip changed the blocker's FORM, from loud to
silent; it did not remove it.**

**Fixed** — not by the flip — by using the `tokenizers` crate on the web path
(`default-features = false, features = ["unstable_wasm"]`). Compiles to
`wasm32-unknown-unknown`, round-trips correctly, and the system prompt now tokenises to
**108 tokens on both backends, matching exactly.**

**Does the flip still stand? Yes, on two legs instead of three.** Capability is unaffected
(0.783 fine-tuned vs SmolLM2's floor at both granularities) and payload is unaffected. Only
the tokenizer leg is withdrawn.

### 2. rten prefix reuse of 2.78x was NOT prefix reuse
Also withdrawn, by its own author. That figure was measured with `append_prompt` in a loop —
a **growing conversation**, not a resettable prefix. `Generator` exposes only `with_prompt`,
`append_prompt`, `clear_prompt`, and `clear_prompt`'s own doc says it *"does not 'rewind' the
conversation"*. **rten cannot rewind a KV cache**, so genuinely independent calls must rebuild
the `Generator` and re-prefill every time.

**Prefix reuse — worth 6.83x on native — has no browser equivalent.** That is the browser
path's main structural disadvantage, and it is most of why browser is ~3x native per call.

## The bridge, both backends running
```
native   (Q4_K_M, prefix reused, constrained)   min 83.4   p50 ~102-110  p95 167 ms   8/8 well-formed
browser  (our q4f16, direct .onnx, no convert)  min 319.6  p50 336.7     max 451.9    4/4 well-formed
```
Browser constrained decoding uses **no llguidance**: the output language is four fixed
strings, so an exact token-level filter is ~30 lines, provably tight, and keeps llguidance
off the wasm dependency graph where it is unverified. **9/9 gates green.**

## The prefill thread is CLOSED — not anomalous, not tunable
Prompt-length scaling is the discriminator between fixed overhead and real compute:

| prompt tokens | 50 | 100 | 200 | 400 | 800 |
|---|---:|---:|---:|---:|---:|
| prefill p50 (ms) | 121.9 | 164.2 | 311.1 | 670.2 | 1448.3 |
| ms/token | 2.438 | 1.642 | 1.555 | 1.676 | 1.810 |

**Linear with a negligible intercept** — marginal 1.83 ms/token fitted over 100→800,
intercept ≈ −19 ms. There is no fixed overhead to remove; the 50-token row only looks
expensive because a small constant is spread over few tokens. With `n_batch`/`n_ubatch` worth
16% and `n_threads_batch` worth 4%, llama.cpp's prefill is simply what ~550–640 tok/s costs
on 8 P-cores at Q4 without AVX-512. **The ranking does not move; the architecture can harden.**

**Still owed:** label accuracy is unmeasured — 8/8 and 4/4 are structural validity only, and
the two backends already disagree on 1 of 4 shared cases (native `Rules`, web `Facts`), which
is plausibly real ambiguity plus a Q4_K_M-vs-q4f16 difference. **Run the 4-way eval against
the q4f16 artifact specifically.** And no wasm-runtime measurement exists: the web backend is
measured natively and has never executed in a browser.

---

## q4f16 GATE FAILED — HOLD the browser artifact — 2026-08-22

| arm | build | 4-way acc | agreement vs fp32 |
|---|---|---:|---:|
| B | ONNX fp32 (reference) | **0.772** | — |
| C | ONNX **q4f16** (the browser artifact) | **0.576** | **0.717** (66/92) |
| D | ONNX q4, fp32 activations | 0.576 | 0.717 |
| A | torch bf16 | 0.696 | 0.772 |

Decision-point mean |Δlogprob| fp32→q4f16 = **3.90** (median 3.56, max 14.9) against a
pre-registered band of ≤0.05. Pre-registered verdict: **NOT EQUIVALENT.** Every instrument
agrees, and the underpowered McNemar fired anyway (p=2.8e-04), which happens only because
the effect is large.

### Two clean separations
- **The fp16 half is exactly free.** q4 and q4f16 give **identical predictions on all 92
  items**. The earlier "rten cannot run q4f16" note was my error, and this confirms the f16
  conversion itself carries no risk at all.
- **All the loss is in 4-bit weight quantisation**, and it lands on the **fine-tuned** model:
  19.6 accuracy points, 26 of 92 predictions changed, collapsing toward the majority family
  (Facts 48 → 69 of 92). **Testing the base model would have given a false pass** — the
  fine-tune fragility was the whole risk.
- Hypothesis, untested: the task signal lives in small LoRA deltas that round-to-nearest
  destroys, while base capability lives in large-magnitude structure that survives.

### Scope: this condemns a QUANTISER, not the format
The quantiser was deliberately naive — RTN, symmetric, block 32, `accuracy_level=0`, chosen
to satisfy rten's `MatMulNBits` constraint rather than to preserve quality. Better methods
exist (`Qwen3-0.6B-DQ-ONNX` claims near-parity via arXiv:2501.06417). **A naive RTN 4-bit
export of these fine-tuned weights is not shippable; any replacement must be re-gated on
this same split.**

Note q4f16's 0.576 still nominally clears the 0.543 ship bar — but 0.717 agreement says it
is a **materially different model**, not a slightly degraded one. Shipping on the accuracy
number alone would have concealed that.

## The native artifact is UNTESTED, not unaffected — and it is what we ship
The report says "the native GGUF target is unaffected by this result." **That is too strong,
and the distinction matters.** Q4_K_M is also a 4-bit quantisation of the same fine-tuned
weights, and the proposed mechanism — small task-carrying deltas destroyed by 4-bit rounding
— applies to it in principle. Q4_K_M is a better quantiser than naive RTN (k-quant, per-block
scales, importance weighting), so it may well survive. **But that is a hypothesis, not a
measurement, and every demo number we have — the 108 ms bridge, 8/8 well-formed — runs on
Q4_K_M.** If it is degraded, the working system is degraded.

Required gate, designed to avoid the cross-runtime confound: compare **Q4_K_M against an F16
GGUF inside llama.cpp** — same runtime, same template, same sampling — so only quantisation
varies. Do NOT compare native against the ONNX arms: that spans runtime, format, template and
sampling, and this programme has already measured a **15-point swing from harness alone**
(ollama 0.467 vs transformers 0.315 on identical weights), larger than any plausible
quantisation effect.

## The 0.783 headline is a RANGE, not a point
Re-running the fine-tune — same code, same seed, same data, only GPU nondeterminism — gave
**0.696, not 0.783.** Both runs selected "best dev = 0.842", but dev has 19 items and several
epochs tie there, so the two runs picked different checkpoints. **Report 0.70–0.78.** The ship
decision stands (both clear 0.543 comfortably) but the point estimate does not.

Related: **torch bf16 costs 7.6 points against fp32** (0.696 vs 0.772). Earlier numbers were
measured under bf16 and may understate the model at full precision.

## Why prediction agreement replaced McNemar as the primary instrument
McNemar is **structurally uninformative at n=92**: it sees only discordant pairs, and the
smallest discordant total that can reach p<0.05 — even when every disagreement falls one way
— is **6**. A ship-worthy quantisation disagrees on 4–5 items, so in exactly the regime that
matters the test **cannot** return significance whatever the data say; passing would be
guaranteed in advance and would evidence nothing. My suggested criterion had this hole and
was correctly overruled. Agreement resolves at 1/92 = 0.011 per item, ~8x finer than accuracy,
and does not marginalise away per-item behaviour. **This split cannot certify equivalence
tighter than about ±9 accuracy points** — which is why accuracy could not be the instrument.

---

## Q4_K_M GATE FAILED TOO — the native artifact delivers ~0.64, not 0.70–0.78 — 2026-08-22

Same merged fine-tuned weights, same runtime, same template, same enum-masked decoding.
Only quantisation varies.

| arm | build | 4-way acc |
|---|---|---:|
| E | F16 GGUF | **0.783** |
| F | **Q4_K_M GGUF — what ships** | **0.641** |

| instrument | value | bar |
|---|---|---|
| prediction agreement | **0.793** (19/92 disagree) | ≥0.95 equiv / <0.90 **not equiv** |
| accuracy gap | **+0.141** | ≤0.03 equiv / >0.06 **not equiv** |
| McNemar | d=17 (15 vs 2), **p=0.00235** | underpowered below d=6 |

**Pre-registered verdict: NOT EQUIVALENT.** "Untested, not unaffected" was the right
correction and testing it found a real loss. Q4_K_M *is* better than naive RTN as predicted
— 14.1 points versus 19.6 — but better is not free.

**Same degradation signature as the ONNX failure:** collapse toward the majority family
(Facts 47→58) and Rules recall craters to **8/24 = 0.33**. Two different 4-bit quantisers,
two different runtimes, one failure shape. That strengthens the LoRA-delta hypothesis —
still untested.

**Scope:** 0.641 clears the 0.543 ship bar, so the system works. It is **14 points worse
than we have been saying.** Honest headline: the native artifact delivers **~0.64**.

**This is now ONE problem, not two:** 4-bit quantisation of these LoRA-fine-tuned weights
loses real quality in *every* quantiser tested. Next move is quantisation-aware or calibrated
methods (QAT, GPTQ/AWQ, imatrix), re-gated on this same split.

Cross-checks that make the number trustworthy: **F16 GGUF reproduces 0.783 exactly** — the
original fine-tune's test score through an entirely independent path (HF → GGUF →
llama.cpp), so the conversion is faithful; Q4_K_M lands at **396 MB**, matching the pinned
396.7 MB; and the same weights read **0.696 in torch bf16 vs 0.783 in F16**, a second
independent confirmation that **bf16 is the worst precision measured** (7-bit mantissa vs
F16's 10) and that several earlier numbers were taken under it.

## A silent 38-point hazard — and it is in OUR bridge too
`ollama show --template` on our imported GGUF returns `{{ .Prompt }}` — **bare passthrough**.
The GGUF itself carries `tokenizer.chat_template` correctly, but **ollama cannot execute a
Jinja template**; it needs a Go template and, for a custom GGUF import, **silently falls back
to passthrough rather than erroring**. Measured cost: **F16 scored 0.402 instead of 0.783 —
38 points, from a template that vanished without a warning.** Anyone importing our GGUF must
supply an explicit `TEMPLATE` in the Modelfile.

**Checking our own path on the strength of that finding: `ciris-nl` applies NO chat template
either.** `Session::session()` calls `str_to_token(system, AddBos::Always)` on a raw string,
and there is no `<|im_start|>` anywhere in the crate. It runs an *instruct* model as raw
completion.

What this does and does not invalidate:
- **Latency is unaffected** — 108 ms is timing, and timing does not care about tokens' meaning.
- **8/8 well-formed is unaffected** — that is structural validity, guaranteed by the enum mask
  regardless of prompt format.
- **Label quality is unmeasured and likely well below the model's capability.** That was
  already flagged as owed; this gives a specific reason to expect it is low.
- **It becomes a correctness defect the moment the fine-tuned weights are deployed.** The
  fine-tune was trained *with* the chat template; serving without it is train/serve skew and
  would break the learned output format. **Fix before the fine-tune lands in the bridge.**

---

## The fine-tune result is UNSTABLE — 0.696 to 0.880 across seeds — 2026-08-22

| run | 4-way |
|---|---:|
| original run 1 | 0.783 |
| original run 2 (same seed, GPU nondeterminism only) | 0.696 |
| seed 11 | **0.880** |
| seed 22 | 0.804 |

**Spread 0.696–0.880 — 18.4 points**, driven by checkpoint selection on a **19-item dev set**
where several epochs tie at the selection criterion. Two of four seeds are still running.

**This supersedes the 0.70–0.78 range recorded earlier, which was itself a correction.** Both
new seeds land *above* everything previously measured, so that range was two draws from a wide
distribution that happened to be low ones. **Quote no point estimate.** The honest statement
is: *highly variable across runs, roughly 0.70–0.88, driven by checkpoint selection on a
19-item dev set.*

### What this does and does not undermine — the distinction matters
- **It does NOT touch the quantisation gates.** Both compared the **same merged weights**
  quantised two ways — F16 0.783 vs Q4_K_M 0.641, and fp32 0.772 vs q4f16 0.576. Seed variance
  cancels in a within-weights comparison. **The 14.1-point and 19.6-point losses stand**, as do
  the agreement figures (0.793, 0.717) which are per-item and never involved a second training
  run.
- **It DOES undermine every absolute capability claim.** No single number is the model's
  ability, and the ship-bar comparison (0.543) should be read against the distribution, not
  against one draw.
- **The real defect is the selection procedure, not the model.** A 19-item dev set cannot
  discriminate between epochs that tie on it. That is fixable — more dev data, or averaging
  over seeds rather than selecting on a coin flip.

## Sharpened: the encoder collapse is monotone in quality
| build | Facts predictions /92 | accuracy |
|---|---:|---:|
| ONNX fp32 | 48 | 0.772 |
| GGUF F16 | 47 | 0.783 |
| GGUF Q4_K_M | 58 | 0.641 |
| ONNX q4f16 (RTN) | **69** | 0.576 |

Two independent quantisers, two runtimes, **monotone in the same direction**. A degraded
classifier falling back to the majority class is far more parsimonious than a domain shift
that happens to select the single most common training label. On harder out-of-domain text the
same mechanism running further plausibly reaches the observed **170/170**.

**Falsifiable and cheap:** if the fp32 re-encode returns a spread of families, quantisation
damage was the cause and the h3ere2 prereg's original substrate is viable after all.

### Seed variance, closed out over six runs — 2026-08-22
`0.663, 0.696, 0.750, 0.783, 0.804, 0.880` — **mean 0.763, sd 0.078.**

| statistic | value |
|---|---|
| 95% CI on the **mean** | 0.700 – 0.825 |
| expected range of any **single run** | **0.609 – 0.916** |
| **test items changing answer between runs** | **41.3% (38 of 92)** |

**The last row is the one that matters.** Four items in ten are coin-flips across runs, so a
single fine-tune's score is close to uninformative as a point estimate — it can land anywhere
from 0.61 to 0.92. **Anyone quoting one run is quoting a draw, not a capability.** This
supersedes both earlier corrections (0.70–0.78, then 0.70–0.88): those were low draws and a
partial sweep respectively.

**Honest headline: 0.76 ± 0.03 (mean of 6 runs).** Deployment recommendation follows from it:
**train several and keep the best on a held-out set**, rather than train once and ship the draw.

**It does not weaken the quantisation gates.** F16 and Q4_K_M were built from *identical*
weights and quantisation is deterministic, so training variance cancels. The 14.1-point loss
stands — but note the "F16 0.783" figure was itself one draw from this distribution, with its
Q4_K_M partner drawn alongside it.

### Prompt tuning: the pool I specified is contaminated, and the clean part is too small
I instructed that tuning happen on train/dev only, to keep the 92-item test frozen. That
protects the test split correctly and **has a hole**: the model was fitted on the train items
to loss 0.004, so scoring prompt variants there measures **memorisation, not classification.**

| variant | pool (138) | **CLEAN dev (n=18)** | memorised train (n=120) | Rules (all) |
|---|---|---|---|---|
| V0 baseline | 0.855 | **0.667** | 0.883 | 26/37 |
| V1 rules-hint | 0.935 | **0.722** | 0.967 | **34/37** |
| V2 member-glosses | 0.826 | **0.722** | 0.842 | 24/37 |

Pool reads 0.855 against 0.641 on test — **that entire gap is memorisation**, and a naive
tuning run would have optimised straight into it. Only the 18 dev items are clean, and
**n=18 cannot resolve a prompt improvement** (SE 0.108; V1's +0.055 is well inside noise).

**Fix, and it doubles as a test of the hypothesis:** sweep the variants on the **base**
Qwen3-0.6B, for which all 138 items are un-memorised — a 138-item clean selection signal
instead of 18. And if in-context signal is quantisation-robust *because it bypasses the
weights*, it should help a model with **no task weights at all**. Select on the base sweep plus
clean dev, then gate the single winner on test once.

**Watch item:** a prompt tuned on the base model may not transfer to the fine-tuned one, which
already encodes some of that signal in weights. The base sweep is a **selection** signal, not a
validation — the winner must still be gated on the fine-tuned Q4_K_M artifact, which is what
ships.

Early sign favours the lever: **V1's Rules recall goes 26/37 → 34/37**, aimed exactly at the
family Q4_K_M damages most (its recall craters to 8/24 there).

---

## Embedding-only 4-bit costs ESSENTIALLY NOTHING — 2026-08-22

Gate re-run on the **fine-tuned** weights (`ft_merged`, one fixed set quantised two ways, as
required so training variance cancels).

| | agreement vs unquantised | accuracy | size |
|---|---:|---:|---:|
| **all weights 4-bit** (their q4f16) | 0.7174 — 26 disagreements | 0.576, **−19.6 pts** | 569.8 MB |
| **4-bit EMBEDDING ONLY** | **0.9457** — 5 disagreements | 0.783, **+1.1 pts** | **346.1 MB** |

**Quantising only the tied embedding costs essentially nothing; quantising everything costs
twenty points.** Five times fewer disagreements, and all five were on items where the
unquantised arm itself had a margin under 0.5 — i.e. items it was not confident about anyway.
The quantised arm scores fractionally *higher* (0.7826 vs 0.7717), which is noise, but the
point is there is no measurable cost.

**Verdict: INCONCLUSIVE, one item short of EQUIVALENT.** 87/92 against a bar of 87.4, so 88
was needed. Clears NOT-EQUIVALENT comfortably.

**The control is what makes it citable:** arm A agrees with `model-scout`'s independent
`pred4_onnx_fp32` on **92/92 items**, same accuracy 0.7717, from a **PyTorch** harness against
their **ONNX** one. Two independent implementations, item-for-item identical — which also
independently confirms the earlier correction that the 0.337 base-model degeneracy was the
model, not the instrument.

**Flagged, not used:** the only failing criterion is mean |Δlogprob| ≤ 0.05, missed by 9.5x —
and that bar looks **unreachable for any 4-bit scheme**, since the all-weights arm has five
times the disagreements and must have larger deltas still. Reported as INCONCLUSIVE because
that is what the pre-registered rule returns; **a criterion is not relaxed after seeing the
result it governs.** But it should be reviewed before it gates anything else, against logprobs
already on disk.

**Not yet established:** these are measured *separately*. That embedding quantisation is free
on an otherwise-unquantised model does **not** prove it is free when composed with 4-bit
MatMul weights — quantisation errors can compound. The composed artifact needs its own gate.

**Consequence for the browser target:** the 39% reduction is available at no measurable quality
cost, which makes the embedding route strictly better than further weight quantisation. The
size problem and the quality problem are **separable**, and only one of them is expensive.

## The upstream branch does not build — and the blocker is 42 lines
`robertknight/rten`'s `gather-block-quantized` (41b811b4, 6 Aug, unmerged) merges cleanly
*textually* onto main 28 commits later, then **fails with 8 compile errors**, all in his file,
from two API changes that landed afterwards: `INVALID_INDEX_ERR` became
`invalid_index_err()`/`try_resolve_index()`, and `OpError` variants now take
`Cow<'static, str>` so bare `&str` literals need the `OpError::invalid_value()` /
`unsupported_value()` / `incompatible_input_shapes()` constructors.

Rebase prepared and verified: **one file, +22/−20**. After it — builds, fmt clean, **clippy 0
warnings, 507/507 tests, 63/63 bit-exact against ONNX Runtime**, wasm32 builds, and it loads
and runs the 346 MB artifact. One judgement call inside it: the out-of-range message adopts
main's newer wording and his two test expectations were updated to match, since that is what
main's own `Gather` now says.

Patch at `scratchpad/rten_gbq/REBASE-onto-main-of-upstream-branch.patch`. **Nothing sent
upstream — held for Eric**, since filing is a public action under his identity.

### The composition: the embedding is free even on top of quantised matmuls — 2026-08-22
Arms built on one fixed `ft_merged` weight set, differing **only** in the embedding:

| arm | accuracy | agreement vs REF | flips /92 |
|---|---:|---:|---:|
| REF untouched | 0.7717 | 1.0000 | 0 |
| M — all 196 matmuls 4-bit | 0.6739 | 0.8261 | 16 |
| C — matmuls **+ embedding** 4-bit | 0.6413 | 0.7935 | 19 |

**The matmuls do the damage; the embedding adds 3 flips of 92** (C-vs-M agreement 0.9674).
**The 39% size win is essentially free even in composition.**

**Both predictions were wrong, and the surprise I asked to be tested for did occur.** The
agent predicted 0.66–0.73 on the reasoning that independent errors ADD (5 isolated embedding
flips + 26 matmul flips ≈ 30); I predicted C ≈ the all-weights arm, ~0.72. Actual **0.7935**.
The embedding cost **5 flips in isolation but only 3 on top of quantised matmuls** — the errors
**partially absorb rather than add**. Prediction written before the run, in `PREDICTION.txt`.

### The mechanism behind every quantisation failure, now measured
The fine-tune moved the weights by **0.182% relative RMS overall — and the embedding by
exactly 0.0000%**. LoRA never touched the embedding table (which is why the embedding
quantisation is already byte-identical to what a fine-tuned artifact's would be, and why this
result transfers).

**That 0.182% carries +46 accuracy points.** The entire task capability lives in a perturbation
two parts in a thousand — which is precisely why 4-bit round-to-nearest destroys it, and it
quantifies the LoRA-delta hypothesis that had been offered as untested speculation. It also
explains why base models survive quantisation comfortably while fine-tuned ones do not: base
capability lives in large-magnitude structure, task capability lives in the noise floor.

### Control failed — absolutes withheld
Arms M and C landed ~10 points **high** against `model-scout`'s measured q4f16, outside the
pre-registered control band. Per its own rule the absolute numbers are **not reported as the
artifact's score**. Obvious causes eliminated: coverage exact (196/196, the same seven
projections × 28 layers the export quantises), `bits=4`, `block_size=32`, `accuracy_level`
unset so fp32 compute. Their fine-tuned export is not on disk. **Cause undetermined, and not
guessed at.** The within-run comparison stands because both arms shared the failure.

### LEAD, not a result: the published export may be leaving accuracy on the table
Plain **round-to-nearest symmetric 4-bit at block 32 — no calibration, no GPTQ, no AWQ —
scored ~10 points better** than the measured q4f16 on the same 196 tensors. If that reproduces
under an independent harness, the "better weight quantiser" unblock is **much cheaper than
calibrated methods**: the exporter, not the format, would be the problem. Flagged as a lead
precisely because the control failed — the check must run through `model-scout`'s ONNX path,
not the harness that produced it.

---

## CONFIRMED: the encoder collapse was quantisation damage — 2026-08-22

| encoder | surface distribution, same 170 wild items |
|---|---|
| **q4f16** (what all 2040 responses used) | **Facts 170 — total collapse** |
| **fp32** | **Facts 109, Manner 61** |
| fp32 + V1 Rules hint | Facts 102, Manner 63, Rules 5 |

Same corpus, same code, same prompt; **only the quantisation changed and the collapse
disappeared.** The domain-shift diagnosis was wrong, and it was only checkable because two
agents' reports sat side by side — one had the collapse, the other had the mechanism.

**The 2040 h3ere2 responses are VOID**, not merely narrowed. A constant surface makes arm C's
path byte-identical across all items, so the reasoning stage emits a constant and the
comparison degenerates to "does one fixed ordering beat another". Regenerate from
`encoded_fp32.jsonl`.

**Honest residual:** fp32 yields **two** families where four exist, and only 5 Rules on a
corpus containing federal-register regulations — so there is probably *some* domain shift on
top of the quantisation damage. But two families is the difference between per-item signal and
none at all.

## The prompt lever works — and it does NOT do what it was asked to do
Selected **V1** (baseline plus one paragraph: deadlines, thresholds, obligations, approvals and
procedural reordering are Rules, not Facts). Selection written before the gate; the frozen test
split read exactly once, after.

| Q4_K_M (ships) | overall | Facts | **Rules** | Identity | Manner |
|---|---|---|---|---|---|
| V0 | 0.641 | 32/34 | **8/24** | 4/10 | 15/24 |
| **V1** | **0.717** | 33/34 | **17/24** | 3/10 | 13/24 |

- **Rules: McNemar discordant 9, all 9 one-way, p = 0.00391** — and the hint never broke a
  Rules item it previously got right. The pre-declared target family, and the effect landed
  exactly where the mechanism predicted.
- Non-Rules 51 → 49, p = 0.625: no significant collateral damage.
- **Overall +7.6 points but p = 0.092 — NOT significant.** The target-family gain is confirmed;
  the overall gain is not.

**The correction, and it undercuts the task as framed.** The prompt was requested to
*compensate the quantisation loss*. **It does not — V1 lifts F16 by a similar amount:**

| | V0 | V1 |
|---|---|---|
| Q4_K_M | 0.641 | **0.717** |
| F16 | 0.783 | **0.837** |

Shipping V1 on both — which is correct — moves the quantisation gap only **0.141 → 0.120**.
**V1 raises the whole curve; it does not close the gap.** It recovers 54% of the damage only
against the *old* F16 baseline, which is not the honest comparison. **Both are true and must be
reported together: the prompt is a real, cheap, shippable improvement, and the quantisation
problem is untouched by it.** The answer to quantisation is still a better quantiser.

### Two corrections to my own advice
- **Few-shot examples HURT — badly.** Every few-shot variant scored 0.59–0.67 against a 0.855
  pool baseline. The fine-tune learned a single-turn format and exemplar turns break it. My
  prefix-reuse *cost* argument held, but the quality effect runs the other way: **this lever
  works through instructions, not examples.**
- **The base model is NOT a valid proxy for prompt selection**, which I had endorsed. The two
  models respond **oppositely**: on base, V1 predicts Rules 119 times of 138 and Facts recall
  collapses 32/52 → 3/52, dropping accuracy to 0.341. The same words are a bias sledgehammer on
  a model without task knowledge and a calibration nudge on one with it. Selection was made on
  the noisy-but-right model, with its pool contamination recorded.

**Deployment: ship V1 on both targets.** Native headline improves from ~0.64 to **0.717** —
while the F16 reference moves up too.

---

## WE CANNOT CURRENTLY MEASURE WHAT QUANTISATION COSTS — 2026-08-22

The RTN lead is **NOT CONFIRMED**, and the reason invalidates a whole class of our numbers.

**torch fp32 and ONNX fp32 — identical weights, identical tokenizer — agree on only 0.761 of
items and differ by 8.7 accuracy points.** That is *exactly* the same agreement as torch fp32
vs torch 4-bit (0.761).

**Two full-precision runtimes disagree as much as full precision disagrees with 4-bit
quantisation.** So any comparison that crosses runtimes crosses a gap as large as the effect
being measured. The RTN lead can be neither confirmed nor refuted that way — **and neither can
any other cross-runtime quantisation claim.**

**Mechanism:** the model sits densely on decision boundaries — 41.3% of items flip between
training runs — so ordinary runtime numerics flip about a quarter of them.

**And the shape of it matters:** every quantised build converges to **0.576–0.663 regardless of
path**, while the fp32 references differ (0.685 torch vs 0.772 ONNX). **Quantisation drags
everything toward ~0.6, so the measured "cost" is largely a function of where the baseline
sat** — a higher baseline simply has more correct answers to lose.

### What survives: within-runtime, within-tokenizer comparisons only
| comparison | cost |
|---|---|
| Q4_K_M vs F16, both in **llama.cpp** | **−14.1** (agreement 0.793) |
| q4f16 vs fp32, both in **ONNX** | **−19.6** (agreement 0.717) |
| RTN vs fp32, both in **torch** | **−2.2** (agreement 0.761) |

Three within-runtime measurements of "4-bit costs X" giving −2.2, −14.1 and −19.6 **is not
noise around a common value.** The cost is runtime-specific, and **a quantisation decision
cannot be made from a simulation in a runtime other than the one that ships.**

### The lm_head hypothesis: found, tested, rejected
The export has **197** `MatMulNBits` nodes, not 196 — the extra is `/lm_head/MatMul_Q4`, the
tied output projection that constrained decoding reads its logprobs from. That looked like the
whole answer. It is not: excluding it gives **+2.2 points (0.576 → 0.598)**, agreement 0.978
with the 197-node build — nearly a no-op — and costs **868 MB (648 → 1516 MB)**, putting the
browser artifact well over budget. **Do not exclude lm_head.**

### An unreconciled tension, stated rather than smoothed over
The embedding is free in composition; the output projection *sharing those weights* is nearly
free (+2.2); yet ONNX 4-bit still costs 19.6. The loss must be in the 196 layer projections —
and torch says those cost 2.2. **That does not reconcile unless the runtime gap is the dominant
term.** Recorded as unresolved.

### The instrument is the bottleneck, and this is the second finding to say so
Before any further quantiser work: **select the deployed model from several seeds rather than
one.** A model with 41% boundary-unstable items will keep producing measurements like these.
The seed-variance report reached the same conclusion independently. **The honest position is
that we have a working pipeline and a shippable prompt gain, and no trustworthy measurement of
quantisation cost — because our reference moves by as much as the effect.**
