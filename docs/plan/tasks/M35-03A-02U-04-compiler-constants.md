# M35-03A-02U-04 — Checked finite-constant source integration

- Status: planned
- Parent: [02U](M35-03A-02U-finite-f64-constants.md)
- Depends on: [Java foundation](M35-03A-02U-03-java-constants.md)
- Specification: [shared](../../specification/typed-generation/rust-finite-f64-constants.md)

## Contract

Extend the distinct compiler constant domain with FiniteBinary64. Check f64
type/eight-byte representation before decoding bits, retain all original
definition/context/export/owner checks and map through existing executable
constant capabilities. Preserve signed zero and exact original producer values.

## Definition of done and tests

Original multi-crate Rust and normal C/Java packages agree on exact bits for
private/public/local/inherent constants, aliases and imported reads. Test
compiler-evaluated finite expressions without widening runtime admission.
Original APIs/docs/privacy and declaration identities remain intact. Typed
probes and negative contracts reject wrong domains, types, values, owners and
lookalike dependencies. Nonfinite/f32/generic/trait/alias-type/borrowed cases fail
atomically. Actual Bazel producer changes invalidate affected consumers and
restoration restores output; include positive/negative zero changes. Detect
compiling value faults, export real examples and update partial-only inventory.
Full gate, preservation and fresh independent review must pass before commit/push.
