# M35-03A-05 — Collections and option/result feature parity

- Status: planned
- Parent: [M35-03A](M35-03A-runtime-free-parity.md)
- Depends on: M35-02, M35-03A-03 and M35-03A-04

The full collection milestone retains these dependencies. Its prerequisite
[05A — No-heap fallible scalar foundation](M35-03A-05A-scalar-results.md) does
not require heap/text or full scalar parity and runs earlier to support checked
numeric operations. This exception is bounded, not early collection admission.

## Contract

Replace runtime-backed list/byte conversions, copying, indexing, appending,
concatenation, containment and search plus option/result construction,
inspection, extraction, fallback and propagation. Emit ordinary source-derived
representations, not mandatory Runtime.Option/Result/List wrappers.

## Definition of done and tests

- Native C/Java match Rust for empty/nested values, out-of-range access,
  success/error propagation, short-circuit fallback and deep-equality policies
  explicitly implemented by the replacement source corpus.
- None differs from Some(empty); success/error payload ownership is preserved.
- Admitted generic instantiations and collection element identities are typed;
  C partial ABI support is not advertised as complete container lowering.
- Mutation/alias/cleanup and allocation-failure tests pass; unsupported element
  or control-flow combinations reject before output.
- Ordinary public consumers and migrated corpus pass without custom runtime
  dependencies; split child tasks, full gates, reviews and tested commits.
