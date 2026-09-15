# M35-03A-04 — Text, Unicode and byte feature parity

- Status: planned
- Parent: [M35-03A](M35-03A-runtime-free-parity.md)
- Depends on: M35-02, M35-03A-02 and required nominal mappings

## Contract

Replace legacy string/byte storage, UTF-8 conversion/validation, scalar and
UTF-16 lengths, searching/slicing, concatenation, prefix/suffix operations,
replacement, trimming and UTF-8 byte truncation with admitted Rust-source
operations and ordinary typed C/Java implementations. No fixed runtime ABI.

## Definition of done and tests

- Distinguish bytes, Unicode scalars and UTF-16 units explicitly.
- Native differential tests include empty values, embedded NUL, supplementary
  characters, invalid UTF-8, Java surrogate boundaries and truncation edges.
- Ownership/aliasing/cleanup and allocation failure have complete evidence;
  C sanitizers and ordinary Java public consumers pass.
- Preserve literal replacement/search behavior and operation evaluation order.
- Standard-library calls use authenticated typed mappings only when equivalent.
- Every affected real-world text/byte case has replacement coverage before its
  legacy helper is removed; full gates/reviews and operation-specific commits.
