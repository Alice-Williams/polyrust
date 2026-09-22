# M35-03A-02W — Unicode scalar character values

- Status: in-progress
- Parent: [scalar parity](M35-03A-02-scalar-parity.md)
- Depends on: [signed-infinity constants](M35-03A-02V-infinite-f64-constants.md)
- Specification: [shared](../../specification/typed-generation/rust-character-values.md)

## Contract and order

Preserve Rust char as a Unicode scalar, not a byte, UTF-16 code unit, string,
grapheme, assigned-character database entry or normalized character.
Keep its source type distinct from integer types through checked mappings.

1. [02W-01 — Independent oracle](M35-03A-02W-01-character-oracle.md) — complete; all 1,015 release/lint targets pass and broad independent review is clean.
2. [02W-02 — C foundation](M35-03A-02W-02-c-characters.md) — complete; all 1,016 release/lint targets pass and fresh broad review is clean.
3. [02W-03 — Java foundation](M35-03A-02W-03-java-characters.md).
4. [02W-04 — Checked source integration](M35-03A-02W-04-compiler-characters.md).

Each checkpoint requires its own full Linux release/lint gate, independent
review and commit/push. Preserve old generated output and unrelated WIP.
This increment covers literal values, transport, immutable places, scalar
function parameters/results, conditionals and six same-type comparisons.
Character constants, casts, text encodings, classification/case conversion,
iteration, heap storage and pattern matching need separate contracts.
