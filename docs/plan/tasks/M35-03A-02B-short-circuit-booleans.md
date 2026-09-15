# M35-03A-02B — Short-circuit Boolean expressions

- Status: complete
- Parent: [M35-03A-02](M35-03A-02-scalar-parity.md)
- Depends on: M35-03A-02A
- Specification: [short-circuit mappings](../../specification/typed-generation/rust-short-circuit-booleans.md)

## Contract

Add a shared checked source capability for built-in bool `&&` and `||`, with
executable C and Java mappings. Preserve left-to-right evaluation and evaluate
the right operand only when required. The right operand's temporary statements
must remain inside that conditional branch, including recursively nested logic
and direct-call arguments. Hoisting those statements is not an optimization
permitted by this contract.

The admitted operand grammar initially matches the existing scalar source
expressions. This does not enable integer bitwise operators, overloaded traits,
general assignment, expression blocks, heap values or new source effects.

## Implementation order

1. Specify checked input constructors and a typed operator enum. Reject wrong
   operand/result types and implicit adjustments before lowering. Extend both
   consuming builders with the executable mapping and negative slot tests.
2. Add scoped evaluation support to each target reader: evaluate and retain the
   left value once; create a private initialized bool result temporary; lower
   the right prelude in a fresh child scope; assign the result only in the
   selected branch. Restore parent scope/prelude state on every path.
3. Admit precisely the resulting local scalar assignment shapes in target body,
   storage/effect, dependency, resource and rendering checks. Existing target
   construction, scope and definite-initialization checks remain mandatory.
   No writes through pointers, fields, parameters or foreign state are added.
4. Add source dispatch for lazy operators separately from scalar comparisons.
   Reuse typed target statements and Boolean-negation mapping where needed;
   introduce no raw snippets, support files, helper catalogue or runtime.
5. Prove behavior, update the partial-coverage inventory, obtain independent
   review and run the full isolated gate before the milestone commit/push.

## Definition of done and tests

- All four bool input combinations agree with an independent truth table for
  and/or; nested mixed operators and negation preserve Rust precedence.
- Operand calls, comparisons, fields, shared dereferences, shadowed bindings,
  argument positions, local initializers and branch conditions work in both
  generated targets. Calls occur in source order, once or zero times as required.
- Typed AST probes verify branch-local right preludes and non-escaping locals.
  Negative controls hoist, duplicate and reorder operand calls and must fail.
  A fixed-corpus native tracing oracle must observe skipped/taken calls, rather
  than inferring short-circuit behavior from pure final return values alone.
- Invalid and unsupported source cases reject atomically for absent and existing
  outputs. Missing, duplicate, incompatible mapping and forged input contracts
  fail to compile; older slot-isolation tests remain meaningful.
- Local writes outside the admitted private scalar shapes remain rejected by
  target dependency/effect profiles. Nested branch resource accounting includes
  every introduced local, statement and call without weakening existing budgets.
- Actual source-owned bundles compile with pinned Rust, strict Java 21 lint and
  GCC/Zig at O0/O2. Export/dependency identities and exact runtime-free inventories
  remain checked, and existing native/compile-negative/lint gates all pass.

This task is not complete until executable evaluation-order evidence passes.
Do not mark the entire legacy JavaBooleanLogic family complete solely because
operator names have registrations; audit its full admitted operand shapes too.

## Completion evidence

- Both executable mappings consume the checked private LazyBooleanInput and
  lower into scoped typed target locals, branches and assignments. Each builder
  has twelve required mappings; fourteen lazy registration/input negative
  contracts pass without weakening earlier slot-isolation tests.
- Eight Boolean triples across fourteen functions produce 112 exact results.
  Rust reference output and an independent truth oracle agree with Java and
  GCC/Zig at O0/O2. Native call traces separately prove taken/skipped evaluation
  order; eager, duplicate and reordered-call mutants retain truth but fail the
  trace oracle for every target/compiler configuration.
- Read-only AST probes check twenty-four lazy nodes per target, source-call
  counts, branch-local prerequisites and output equality with production.
  Actual bundles retain exact source-owned file/export/dependency inventories.
- Six invalid/unsupported source cases reject for two targets and absent/existing
  outputs (24 checks), with six successful publication controls. Initialized
  bool locals are admitted; parameter/non-bool writes remain rejected. Direct
  Java body tests additionally reject method-like and sibling-scope leakage.
- Integration testing exposed a masked Java unsupported-expression diagnostic,
  missing test state initializers, a probe comparison/publisher hookup, strict C
  consumer indentation and a negative test expecting a later rejection layer.
  These were repaired; the diagnostic preserves the original error after scope
  restoration, and the negative test now also exercises body admission directly.
  No test was disabled, and no production lint rule was weakened.
- Independent whole-scope review closed with no remaining findings on tree
  `9b4deecf4777f68131d66cc2a4f71f85d244adb7`. The isolated Linux Bazel/native/lint
  gate passed all 547 tests across 696 targets in 131.038 seconds, invocation
  `eeda4d8c-e8f4-4c35-ba36-f0cb71f3dd63`; archive validation checked 2,412 exact
  Git blobs and executable modes. Completion-document changes are gated again
  before commit/push.
- Tested output was exported to ignored `generated/m35-short-circuit-9b4/c`
  and `generated/m35-short-circuit-9b4/java` for inspection. Generated artifacts
  are not committed. Legacy runtimes/consumers remain until all parity work is
  complete; the inventory still marks JavaBooleanLogic as partial coverage.
