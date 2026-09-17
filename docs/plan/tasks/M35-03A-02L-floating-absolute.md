# M35-03A-02L — Built-in binary64 absolute value

- Status: complete
- Parent: [scalar parity](M35-03A-02-scalar-parity.md)
- Depends on: [NaN classification](M35-03A-02K-floating-nan.md)

## Ordered implementation

1. [02L-01 — binary64 conditional target foundation](M35-03A-02L-01-floating-conditional.md).
   Complete, with 822/822 Linux release tests and two clean independent reviews.
   Extend C's shared certified profile only for Bool condition and exact F64
   branch/result identity. Verify existing Java primitive conditional support.
2. [02L-02 — checked source absolute value](M35-03A-02L-02-compiler-absolute.md).
   Complete, with 840/840 Linux release tests, native/AST/atomic proof and
   reviewed ordinary-call preservation controls.
   Authenticate standard inherent f64::abs method/associated calls, add the
   executable typed mapping and complete native/AST/atomic proof.

## Required result

Runtime-free C and Java absolute value preserves every non-NaN magnitude
exactly, maps either signed zero to positive zero and either infinity to
positive infinity. The existing admitted NaN category-only observation
contract remains explicit; do not claim NaN payload/sign equivalence or enable
bit/sign inspection on this evidence. The original source receiver evaluates
once. No copied runtime, text template logic, unchecked output, binary
arithmetic or general cast admission is introduced.

Each child requires a full Linux release/Rust/Bazel lint gate, independent
review and its own scoped commit/push. Retain the legacy implementation until
the complete relevant parity inventory is satisfied.
