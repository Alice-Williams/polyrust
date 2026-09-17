# M35-03A-02I-03 — Checked Rust f64 value mappings

- Status: planned
- Parent: [binary64 values](M35-03A-02I-binary64-values.md)
- Depends on: [target foundation](M35-03A-02I-02-target-values.md)

## Contract

Extend the existing executable ObjectTypes, LiteralValues, ResolvedPlaces,
record, signature, call and ScalarComparisons mappings with exact f64 rules.
Obtain literal values from rustc's checked evaluation, not a duplicate decimal
parser. Preserve canonical source identities, documented public/private crate
boundaries and exactly-once left-to-right calls.

Literal construction initially accepts only finite results, including negative
zero. Runtime f64 parameters may carry any binary64 category under the specified
observable value/comparison contract. Public/local f64 constants, nonfinite
construction, casts, arithmetic and methods remain diagnosed until separately
specified and proven. Version metadata explicitly if new type spellings require
it; old scalar-only package bytes/schemas remain unchanged.

## Definition of done and tests

- Real multi-crate Rust and generated C/Java packages match independent exact
  finite-bit and nonfinite-category/comparison oracles, with native call traces.
- Compiler/AST probes prove original identity, exact type, literal bits, operand
  placement and identical probe/production bytes; counterfeit witnesses fail.
- Unsupported f32, casts, arithmetic, nonfinite construction, adjustments and
  mixed widths reject atomically after valid Rust analysis where applicable.
- Actual examples are exported outside Docker without committing generated
  output. Full Linux release/lint gate and a fresh independent review pass.
- No replacement-family completion or runtime retirement is claimed.
