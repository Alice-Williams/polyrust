# M35-03A-02U — Finite binary64 constants

- Status: in-progress
- Parent: [scalar parity](M35-03A-02-scalar-parity.md)
- Depends on: [signed widening](M35-03A-02T-signed-widening.md)
- Specification: [shared](../../specification/typed-generation/rust-finite-f64-constants.md)

## Contract

Extend the existing typed constant capabilities to exact finite f64 values.
Keep the constant domain separate from literal inputs. Preserve compiler-resolved
declaration identity, exact bits (including negative zero), original owner
authority, public/private visibility, documentation and alias bindings.
Do not admit NaN/infinity constants or generalize runtime NaN payload semantics.

## Ordered checkpoints

1. [02U-01 — Independent constant oracle](M35-03A-02U-01-constant-oracle.md) — complete; 991 release/lint targets pass, independent review clean.
2. [02U-02 — C constant foundation](M35-03A-02U-02-c-constants.md) — complete; 992 release/lint targets pass, independent review clean.
3. [02U-03 — Java constant foundation](M35-03A-02U-03-java-constants.md).
4. [02U-04 — Checked source integration](M35-03A-02U-04-compiler-constants.md).

Each checkpoint requires focused native/negative proof, the full Linux Bazel
release/lint gate, fresh independent review and its own commit/push. Source
admission follows both target foundations. All old packages and unrelated WIP
remain unchanged. This is partial constant parity, not legacy-removal approval.
