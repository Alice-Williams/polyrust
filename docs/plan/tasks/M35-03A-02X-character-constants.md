# M35-03A-02X — Unicode scalar constants

- Status: in-progress
- Parent: [scalar parity](M35-03A-02-scalar-parity.md)
- Depends on: [character values](M35-03A-02W-character-values.md)
- Specification: [shared](../../specification/typed-generation/rust-character-constants.md)

## Contract and order

Extend the separate compiler-evaluated constant domain with Char(char), not
an unchecked integer. Preserve public/private/local/inherent declarations,
original owners, aliases and dependency reads through ordinary typed target
constants. Do not infer source Char from a Java Int or C U32 certificate.

1. [02X-01 — Independent constant oracle](M35-03A-02X-01-constant-oracle.md) — complete; 4,127 compile-time observations, all 1,027 release/lint tests and clean broad review.
2. [02X-02 — C constant foundation](M35-03A-02X-02-c-constants.md).
3. [02X-03 — Java constant foundation](M35-03A-02X-03-java-constants.md).
4. [02X-04 — Checked source integration](M35-03A-02X-04-compiler-constants.md).

Each checkpoint needs separate full Linux Bazel release/lint, fresh broad
review, preservation evidence and commit/push. No production source admission
is enabled by the oracle or target foundations. Existing legacy gates remain.
Character conversions, methods, references, text, generic/trait constants and
type-alias admission are outside this increment.
