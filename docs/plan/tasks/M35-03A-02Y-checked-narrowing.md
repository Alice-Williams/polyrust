# M35-03A-02Y — Checked signed narrowing

- Status: in-progress
- Parent: [scalar parity](M35-03A-02-scalar-parity.md)
- Specification: [shared](../../specification/typed-generation/rust-checked-narrowing.md)

## Contract and order

Replace the legacy fallible i64-to-i32 conversion, not with a wrapping cast or
exception. Exact values in the inclusive i32 range succeed; all others return
an explicit error. This is one operation, not arbitrary TryFrom/Into admission.

1. [02Y-01 — Independent narrowing oracle](M35-03A-02Y-01-narrowing-oracle.md) — complete; 74,389 inputs, six executable fault families, all 1,036 release/lint tests and clean broad review.
2. Complete [05A — No-heap fallible scalar foundation](M35-03A-05A-scalar-results.md).
3. [02Y-02 — C checked narrowing](M35-03A-02Y-02-c-narrowing.md).
4. [02Y-03 — Java checked narrowing](M35-03A-02Y-03-java-narrowing.md).
5. [02Y-04 — Checked source integration](M35-03A-02Y-04-compiler-narrowing.md).

The independent oracle can run before result support. Target/source admission
cannot. Each checkpoint requires full Linux Bazel release/lint, fresh broad
GPT-6-Sol extra-high review, preserved unrelated work and its own tested commit.
No legacy retirement or full IntegerConversions claim follows from the oracle.
