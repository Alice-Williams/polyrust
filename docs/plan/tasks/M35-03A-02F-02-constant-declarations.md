# M35-03A-02F-02 — Public and block-local scalar constants

- Status: in-progress
- Parent: [M35-03A-02F](M35-03A-02F-scalar-constants.md)
- Depends on: M35-03A-02F-01

## Contract

Extend the certified package/dependency models with explicit constant
declarations and references. Preserve public/private and crate ownership,
same-spelling identities, aliases/re-exports and documentation attributes.
Specify the ordinary C declaration/definition and Java static-final-field
mappings before implementation. Do not silently fold away a public API name.

Admit block-local const items as compile-time declarations, without a runtime
statement or storage identity. Use compiler resolution, not a textual scope map.

## Definition of done and tests

Ordered children:

1. [02F-02A — Block-local declarations](M35-03A-02F-02A-local-constants.md) — complete.
2. [02F-02B — Public constant APIs](M35-03A-02F-02B-public-constants.md).

Each child has its own gate/review/commit. This parent is complete only after
both local declaration semantics and public/dependency APIs have evidence.

- Independently compiled consumers reference public constants through generated
  C headers and Java packages. Cross-crate aliases/re-exports retain identity.
- Private constants never enter the public API; public value/type/owner mutations
  fail certification or dependency checks. Documentation remains inspectable.
- Block-local shadowing resolves to the exact compiler declaration and has no
  runtime initialization effect.
- All 02F-01 regression gates remain green; explicitly update former rejection
  cases only when their source shape gains positive proof. Add declaration/reference contracts,
  native equality, stale-metadata negatives and atomic-publication tests.
- Per-language specification, independent reviews, exported examples and full
  isolated Bazel/lint gates precede the dedicated commit/push.
