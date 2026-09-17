# M35-03A-02G-02 — Checked Rust unit effects and publication

- Status: complete
- Parent: [unit results](M35-03A-02G-unit-results.md)
- Depends on: [target void foundation](M35-03A-02G-01-target-void.md)
- Specifications: [shared](../../specification/typed-generation/rust-unit-results.md),
  [C](../../specification/typed-generation/languages/c/rust-unit-results.md),
  [Java](../../specification/typed-generation/languages/java/rust-unit-results.md)

## Contract

Translate checked built-in Rust () function results into ordinary C/Java void.
Preserve typed executable capability registration, source order, crate identity,
visibility, docs and original dependency authority. Do not enable unit objects,
parameters, locals, constants, generic/indirect calls, early return, mutation,
panic, I/O or arbitrary external effects. Selected-entry ABI remains fn(i32)->i32.

## Ordered implementation

1. Add one target-free UnitEffects capability with a private checked UnitInput
   and a closed operation enum. Its constructor uses rustc TypeckResults and
   rejects non-unit, coercions, overloaded/unsupported forms and early returns.
   Distinguish empty tuple, unit block, direct unit call and unit conditional.
2. Register real C/Java mappings in consuming builder slots. Outputs are typed
   statements, never CValue or Java scalar Value. Factor direct-call resolution
   and ordered scalar argument materialization for reuse by value/effect calls;
   a void call itself has no temporary.
3. Extend FunctionSignatures only in its result position. Reuse lexical scope
   management, typed lets and existing budgets. Separate effect-only completion
   from function-return completion with an enum: a statement-position block or
   conditional must not accidentally return from its containing function.
   Missing else is permitted only for a checked unit conditional; condition
   preludes remain outside the branch, branch effects stay inside it.
4. Preserve every syntax node in admission: no silent erasure of trailing
   unsupported statements, adjustments or unit locals. Retain local and foreign
   call discovery, original witnesses, cycle/depth limits and resource costs.
5. Version unit-aware owner metadata independently of scalar-only schemas. C
   signatures derive from registered CFunctionType; Java result encoding is
   separate from scalar parameters/fields/constants. Standalone/bundle metadata
   must describe the same exact certified owners. Metadata never creates proof.
   Old scalar packages keep their existing versions and bytes.
6. Add genuine Rust producer -> relay -> consumer fixtures containing public,
   private and mixed scalar/unit functions, docs, empty/explicit unit,
   statement/tail calls, nested blocks and conditionals. Export reviewable
   generated artifacts under ignored generated/examples; do not commit outputs.

## Required proof and definition of done

- Independently cached target-free input probe and per-language registration
  controls: missing/duplicate/wrong capability/context/output/input and private
  field construction reject for the intended diagnostic. A positive complete
  builder proves these negatives are not failing because another slot is missing.
  Update earlier missing-slot probes so each still omits only its named slot.
- Typed AST probes assert actual void result/effect nodes, original callable
  handles, exactly-once ordered scalar argument temporaries, no void temporary,
  normal return completion and branch-local effects.
- Compile each generated C public header independently. Separately compile/link
  producers/consumers with GCC and Zig at O0/O2, and Java 21 with strict lint.
  Compare Rust/native scalar observations and reflect/check genuine void ABI.
- Since pure void calls have no observable result, use explicit test-only
  instrumentation on copied generated files to record arguments and call order.
  Compare fixed expected traces for multiple arguments, nested scalar calls,
  true/false branches, statement blocks followed by later work and tail effects.
  Also run unmodified outputs; instrumentation must not widen source admission.
- Reject unit storage/parameters, never/tuple results, indirect/generic calls,
  early return, unknown effects, wrong signatures, foreign/recertified owners and
  metadata disagreement. Preserve atomic publication/no-output-on-failure.
- Mixed constants/unit and transitive bundles use exact documented schema
  versions; independent artifact validators reject unknown emitted versions.
  Do not invent a manifest-input parser or treat descriptive JSON as authority. Test positive old scalar schemas unchanged.
- All Linux Bazel tests, release gate and Rust/Bazel linters pass on the exact
  isolated commit tree. Fresh independent Sol Extra High review has no core
  findings. Record evidence before the scoped commit and push.

## Scope boundary

This closes unit function results, not general Rust unit values or runtime-free
parity overall. Do not delete remaining legacy runtimes while other parity
features remain unimplemented. Keep focused production/test modules below the
repository file-size limits and separate Bazel targets at stable proof boundaries.

## Proof map

- `unit_contract_test`: fourteen intentionally invalid C/Java registrations
  (seven per backend), with exact compiler error categories; production adapters
  are the positive complete builders. `source_capabilities_test` retains the
  target-independent compiler input boundary.
- `unit_ast_test`: 22 unit operation observations and seven normal unit
  completions per backend. Source-derived temporary counts are correlated to
  argument locals in order; typed declarations cannot contain void results.
  Per-branch effect counts, no-return effect blocks and a scalar-call condition
  assert placement. Probe and production artifacts must be byte-identical.
- `unit_native_test`: three real crates, ten independent input/branch rows,
  Rust truth and separately compiled GCC/Zig O0/O2 plus Java 21 strict lint.
  Unmodified outputs preserve scalar continuations and ordinary void ABI;
  copied instrumentation records exact arguments, condition evaluation and
  branch order. Drop, duplicate and reordered-call mutants retain scalar truth
  but must fail the trace oracle. Unknown owner/index schemas reject.
- `unit_rejection_test`: thirteen unsupported valid-Rust cases in both targets,
  each with absent and existing destinations (52 atomic checks). Unsafe source
  is rejected by the compiler's forbid-unsafe policy; never results, unit
  storage/parameters, indirect/generic calls, early return and external effects
  retain their specific diagnostic.
- Existing target suites `shared_void_tests`, `shared_void_local`,
  `shared_void_native`, Java `unit_results` and `unit_results_native` provide
  original-versus-recertified owner, result/argument mismatch, invalid storage,
  dependency closure and source/call-height accounting controls.
- The old public-package negative now tests unit parameters, not newly supported
  unit results. No gate is disabled. All scalar-only fixtures remain enabled.
- Actual generated packages, handwritten clients and Rust input fixtures are
  exported as Bazel undeclared outputs and copied outside Docker to ignored
  `generated/examples/unit-results-597b335/`. Outputs are not committed.

## Release evidence

- Exact isolated implementation tree `597b335be434cbd4ddeb79758f3ff8ba99caffb5`
  passed Linux dev-container Bazel `//... //:release_gate`: **760/760 tests**,
  1,140 targets, 55 executed / 705 cached, 207.311 seconds. Invocation:
  `31933894-6bc3-4af3-8b95-d732e919df15`.
- Full gate retains Rust/Bazel linters, 351 Java unit tests, C target tests,
  all historical scalar/package/authority proofs and the new native unit suite.
- Initial review found no production defect but correctly identified missing
  condition-prelude, precise typed-AST completion/placement and never-result
  rejection evidence. All findings were accepted and closed with executable
  assertions and source/native fixtures; no production admission was broadened.
- All 19 separately preserved ownership files retain their original hashes.
  Neither those changes nor generated artifacts enter this checkpoint.
- Fresh independent Sol Extra High review of the exact implementation tree
  above found **no findings** across production and proof code. The final
  documentation-only completion tree is gated again before scoped commit/push.
