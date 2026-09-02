# M34A-06 — Expand interfaces and composition semantics

- Status: planned
- Depends on: M34A-02, M34A-05

## Goal

Make flat interfaces, explicit conformance, first-class polymorphic values, and
composition part of the portable checked model without exposing inheritance.

## Definition of done

- Unchecked/checked IR and CoreIR use a versioned first-class interface model;
  legacy v0 `contract` input remains readable through an explicit migration.
- Interfaces contain immutable method signatures only, have no inherited
  interfaces/default bodies/state, and use stable interface/method IDs.
- Records explicitly implement any number of independent interfaces and the
  checker proves exact method conformance without ambiguous duplicates.
- Interface types work in parameters, returns, fields, tagged payloads,
  lists/options/results, and local bindings.
- Static concrete calls and dynamic interface calls are distinct typed CoreIR
  operations; evaluator dispatch is deterministic.
- Interface values have immutable owned value semantics, no portable identity,
  downcast, equality, or mutation.
- Composition is ordinary typed fields plus explicit delegation and has no
  inherited lookup, promotion, `super`, or override relation.
- Portable schemas/builders cannot express inheritance. The target-only
  one-edge adapter policy has shared verifier support but no portable node.
- Canonical interface/conformance/composition fixtures and negative cases are
  reusable by every language task.

## Tests

- `bazel test //crates/ir:interface_model_test //crates/check:interface_test --nocache_test_results --test_output=errors`
- `bazel test //crates/core-ir:interface_test //crates/eval:interface_test --nocache_test_results --test_output=errors`
- Legacy-read/version, multiple-conformance, nested interface value,
  static/dynamic dispatch, explicit delegation, nonconformance, ambiguity,
  identity/equality exclusion, and no-inheritance schema tests.

## Commit gate

Commit and push `M34A-06: add first-class interfaces and composition` only
after IR/checker/evaluator/builder compatibility and canonical fixtures pass in
the dev container.
