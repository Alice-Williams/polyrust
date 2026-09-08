# C17 closed standard-call contracts

- Status: normative for M34A-11-02D-00 and 03
- Owner: dialect/catalogue; safety interpretation: ownership/

## Authority and admission

A known call has a closed identity distinct from a generated callable reference.
The catalogue owns its exact signature, header identity, permitted call form,
system-library requirement and operand-specific safety obligations. No name,
matching prototype or generated-contract registration grants standard semantics.
Catalogue metadata is immutable Rust data/enums; it cannot contain raw executable
source, arbitrary link flags or caller-authored effect booleans.

Construction and contextual reconstruction derive the same exact call relation.
Generated callable contracts continue to identify bodies that must be proved.
A known callable is not registered as a fake generated function/body, and a
macro operation is never available as a function address. The initial known
surface admits direct calls only; indirect standard function addresses require
an explicit later grammar extension and proof, not a signature-based shortcut.

## Initial operation inventory

The notation below is exact C type structure, not a string signature stored in
the AST. Void pointers preserve constness. Size means the pinned size_t and
Double means binary64 double. FILE is the existing opaque known object identity.

| Identity | C operation | Return | Ordered parameter types | Header | System library |
| --- | --- | --- | --- | --- | --- |
| Allocate | malloc | void* | Size | Stdlib | none |
| Release | free | void | void* | Stdlib | none |
| CopyBytes | memcpy | void* | void*, const void*, Size | String | none |
| CompareBytes | memcmp | Int | const void*, const void*, Size | String | none |
| FloatRemainder | fmod | Double | Double, Double | Math | Math |
| FloatTruncate | trunc | Double | Double | Math | Math |
| IsNan | isnan | Int | Double | Math | none |
| SignBit | signbit | Int | Double | Math | none |
| WriteBytes | fwrite | Size | const void*, Size, Size, FILE* | Stdio | none |
| StreamError | ferror | Int | FILE* | Stdio | none |

IsNan and SignBit are closed double-specialized macro operations, not arbitrary
C type-generic calls. Their result is zero or nonzero Int; nonzero is not promised
to equal one, minus one or a particular bit mask. CompareBytes returns an Int
whose sign has meaning, not necessarily -1/0/1. Explicit numeric conversion
establishes a portable Bool. No variadic printf-family operation is admitted.

This is an extensible inventory, not a claim to support the whole C library.
Additional entries require exact prototype, effects, dependency and independent
native controls. Known constants/objects retain their existing identity and
actual scalar types; metadata must not duplicate or reinterpret those types.

## Obligations, not proof flags

- Allocate requires a checked positive byte request. It returns null or fresh,
  disjoint, suitably aligned uninitialized storage. Only the actual default
  allocator implementation may use it to discharge the allocator protocol;
  a generated function merely named malloc cannot do so.
- Release requires null or the exact live base pointer from that allocator, with
  no outstanding borrow; it invalidates that allocation once. Custom allocator
  release uses the separately authenticated callback/context, not this identity.
- CopyBytes requires valid source/destination extents, initialized source bytes,
  writable destination and no overlap. Zero-length paths are handled without
  calling it on null pointers. The returned pointer aliases the destination;
  it is not fresh ownership. Initialized-byte transfer does not alone establish
  every object invariant; F64/U64 representation transfer uses the pinned model.
- CompareBytes requires initialized readable byte spans of the checked length.
  It is not portable structural equality and cannot compare arbitrary aggregate
  padding as semantic data. Zero-length null spans take the no-call path.
- Floating operations use the specified platform environment. Catalogue effects
  do not promise preservation of sticky FP flags or errno, which are outside
  portable observation. Portable-result and NaN-bit obligations remain mappings.
- WriteBytes requires checked element-size/count multiplication, readable
  initialized extent and a valid borrowed stream. Its result is the actual
  number of complete elements written, at most the requested count; partial
  output is possible and cannot be represented as transactional success.
- StreamError requires a valid borrowed stream; zero/nonzero Int reports its
  error indicator. It neither owns nor releases FILE.

The generated native harness may use Stdio operations; production packages must
not acquire test-only dependencies from unused catalogue entries. Stage 03 derives
headers/libraries from actual references and checks role/dependency isolation.
Stage 02D checks each actual operand and call context before applying an effect.
No effect or library label creates initialization, provenance, range or ownership
evidence by itself.

## Proof ownership

02D-00 adds distinct typed calls, exhaustive signature reconstruction and
independent pinned compiler probes. 02D-04/05 interprets the closed obligations
and proves actual generated allocator/callable bodies. Stage 03 resolves headers,
names and native-library metadata from these same entries. Stage 04 spells only
resolved certified call nodes; stage05/06 constructs actual structural runtime
bodies. Existing raw runtime text cannot supply any certificate in these stages.
