# M35-03A-02K — Built-in binary64 NaN classification

- Status: complete
- Parent: [scalar parity](M35-03A-02-scalar-parity.md)
- Depends on: [binary64 negation](M35-03A-02J-floating-negation.md)
- Specification: [shared and per-language contracts](../../specification/typed-generation/rust-floating-nan.md)

## Contract and implementation order

1. Authenticate the canonical standard inherent f64::is_nan method/associated
   call in a private source witness. Add a separate executable FloatingNaN
   builder slot; preserve every existing mapping and exact compile contract.
2. C and Java lower the original receiver once to a local and compare that local
   with itself using their existing typed inequality nodes. Normalize C Int
   to Bool. Do not add runtime files, raw fragments or new renderer semantics.
3. Add separate focused fixtures, compiler/AST probes, native integer-mask and
   call-trace oracles, compile-negative contracts and atomic rejection tests.
4. Run full Linux release/Rust/Bazel lint gates, export actual three-crate
   examples, obtain fresh independent Sol Extra High review, evaluate/fix core
   findings, document evidence, commit and push this checkpoint.

## Definition of done and tests

- Both by-value source spellings work; bool output distinguishes signed finite
  values/zeros/subnormals/infinities from quiet/signaling NaN encodings.
- Original receiver evaluated once, including imported calls, local and record
  reads and explicit shared dereference. No implicit adjustment is admitted.
- Private-input/canonical-context checks, exact slot contracts and target
  local identity/result/precedence probes pass. A disconnected call-result
  mutant must fail independently of call-count instrumentation.
- Rust, GCC14/Zig C17 O0/O2 and Java21 strict compilation agree with an
  independent integer-bit oracle; wrong-comparison/always-false and dropped/
  duplicated receiver faults are observable.
- Valid but unsupported f32, trait/custom methods, implicit autoref/deref,
  other float methods, casts and constants reject without partially replacing
  new or existing output directories.
- Prior source bundle bytes remain stable; real generated examples are
  exported; exact full release/lint gates and fresh independent review pass.
- Partial parity inventory only; no full floating-inspection or runtime
  retirement claim.

## Implementation and evidence

- Both call inventories now discover the authenticated primitive before ordinary
  callable resolution and still walk its receiver. The initial native gate
  caught the missing associated-call inventory integration; it was fixed, not
  bypassed. The f32 rejection now expects the earlier exact primitive diagnostic.
- Shared floating operand-dataflow test helpers return whether their deliberate
  detached-result fault was rejected. Negation and NaN probes own their separate
  observation labels; the existing negation proof continues to pass unchanged.
- Exact implementation tree 8a10350e57a947566ec3100aeb6226c512c3286e passed
  Linux Bazel test //... //:release_gate: 822/822 tests across 1,251 targets,
  67 executed and 755 cached, 248.942 seconds.
  Invocation: 9d4c468a-2580-4c25-bb02-f13aa04b61b8.
- Native proof checks 578 results over 72 raw binary64 inputs and two literal
  results, three original crates, GCC14/Zig C17 O0/O2 and Java21 strict lint.
  All wrong-comparison/constant-false/dropped/duplicated receiver controls fail
  their independent value or trace expectations. A free is_nan function remains
  an ordinary original-owner call, not a builtin selected by spelling.
- Per target, seven canonical target-AST observations (five method and two
  associated forms) plus two detached-call-result controls pass. Production and
  read-only probe output bytes match. Fourteen compile-negative mapping/input
  contracts and 48 atomic unsupported-source cases pass.
- All 138 files in the earlier 18 bundle baseline and both preceding
  floating-negation bundles are unchanged. Twenty-four actual example files
  are exported at generated/examples/nan-classification-8a10350e.
- Sol Extra High library_import_review completed a comprehensive independent
  review of the exact implementation tree with no core or required-proof findings.
  Fresh Sol Extra High binary64_targets_review independently found no core,
  authority, resource or required-proof findings. Its optional custom associated
  two-argument and imported-call detachment extensions are deferred: the closed
  direct-call boundary already rejects unsupported associated forms, and the
  original-owner inventory/native proof covers foreign authority. Neither is a
  missing contract requirement. Final documentation/parity gate precedes push.
