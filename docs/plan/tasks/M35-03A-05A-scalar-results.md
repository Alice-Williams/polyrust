# M35-03A-05A — No-heap fallible scalar foundation

- Status: planned
- Parent: [collections/results](M35-03A-05-collection-parity.md)
- Depends on: completed scalar values, unit results, calls and structured branches

## Dependency correction

This prerequisite is independent of full scalar parity, String/Vec and owned
heap mappings. Checked arithmetic, shifts and narrowing need an explicit
failure value; making them depend on the entire collection phase creates a
cycle. No-heap result work may precede completion of 03A-02 and 03A-03.

## Contract

Before implementation, split target and compiler work into separately specified
tasks. Define typed source variant/payload identities and a closed initial
Result<i32, TryFromIntError> domain, with scalar Option support as a separate
increment when required. Do not claim all enums, generic results or collections.
Preserve success versus error, including successful zero. C uses a certified
source-derived tagged value; Java uses certified ordinary source-derived types.
Neither relies on a shared Runtime type or string-tag dispatch in lowering.

Opaque standard-library error identity must remain distinct from arbitrary
empty types. Specify construction, transfer, return, match and inspection, and
which observations of TryFromIntError are admitted. Unsupported Debug/Display,
generic traits and heap payloads reject; error strings are not invented.

## Definition of done and tests

Native Rust/C/Java agree on success/error construction and observation,
branching, zero/min/max payloads, arguments, returns and cross-crate calls.
No Rust-owned heap payload is admitted; incidental Java allocation is not a
claim of Rust heap support. Wrong nominal instances, variant owner, inactive
payload access, incomplete matches and forged imports reject before output.
Prove operand effects, privacy/docs, standalone consumers, source boundaries,
atomic publication and capacity limits. Complete per-target specs and child
tasks before production edits, then full Linux release/lint and fresh reviews.
