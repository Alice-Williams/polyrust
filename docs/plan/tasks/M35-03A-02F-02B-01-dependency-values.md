# M35-03A-02F-02B-01 — Shared certified dependency values

- Status: complete
- Parent: [public constants](M35-03A-02F-02B-public-constants.md)
- Depends on: M35-03A-02F-02A

## Contract

Implement the [shared dependency-value contract](../../specification/typed-generation/certified-dependency-values.md).
Add typed value witness/spec/reference hooks and complete linking/reconstruction
without conflating values with callables or known fields. Use an uninhabited
value type for current plugins until their certified constant APIs exist.

## Definition of done and tests

- Exact witness reconstruction and duplicate/type/owner/name/spelling checks.
- Fixed and qualified binding tests; mixed function/value collision and repeated
  symbol-binding deduplication tests; unresolved and coupled catalogue/reference
  mutations fail. Native directive deduplication is proved by C child 02.
- Existing C known scalar types retain their representation; shared context-free
  checks and the required dialect type-validation hook reject inadmissible types.
- Every existing dialect compiles without claiming new source support.
- Dedicated focused modules and tests; full isolated Bazel/lint gate, fresh
  evaluated review and scoped commit/push. No C/Java source support claim yet.

## Implementation

The shared linker now carries DependencyValueSpec and a distinct typed
DependencyValue reference. Dialects must reconstruct metadata from an opaque
witness and execute verify_dependency_value_type. The shared profile accepts
primitive types and catalogue-known zero-arity built-in/prelude/standard-library
types without package requirements; generated, runtime, parameter, constructed
and uncatalogued types reject. C's existing known i32/i64 types are preserved.

Certified imports retain a namespace-aware binding domain. Catalogue-known
owner imports retain their established domain so a type and constructor can
share a directive. Allocation and post-link reconstruction agree, including
mixed/empty-domain rejection. C and Java use uninhabited dependency-value types
until their child APIs exist; no new source capability is claimed.

New implementation, fixture and tests are in focused files. The shared linker
receives only variant/field/hook dispatch and domain-key integration.

## Proof and review record

- The shared unit target runs 125 passing cases, including 11 dedicated value/
  import cases. They cover exact witness metadata, fixed/qualified references,
  missing/unused symbols, collision namespaces, same/different-owner bindings,
  context-free/dialect type checks, original-reference authority, coupled
  catalogue/import/reference changes and catalogue-known coalescing.
- Preflight on tree 76f70a95633a6e6704173a9dbbbf45bb748da789 passed all five targets:
  shared units, Rust Clippy, rustfmt, buildifier and documentation. Invocation
  96bed019-b015-459e-bf82-301458459263; 56.834 seconds.
- Three independent Sol Extra High reviewers examined the evolving change.
  Accepted findings repaired C known-type admissibility, a binding/directive
  proof distinction, cross-namespace binding coalescing and its catalogue-known
  compatibility, a missing mixed-domain negative, and C ABI specification
  precedence. The final review found no remaining required fixes.
- Clippy rejected the first nested domain-key tuple. It is now a named Key
  struct; the lint remains enabled. The superseded full run was interrupted,
  not counted as a passing gate.
- Optional value-versus-value collision variants and a witness-sensitive test
  hook were not treated as release blockers: shared collision insertion already
  covers those categories, while exact metadata reconstruction plus independent
  type admissibility checks authenticate the current test witnesses. Concrete
  backend witness behavior belongs to children 02/03.
- Final implementation tree 76f70a95633a6e6704173a9dbbbf45bb748da789 passed the
  isolated full regression gate: 624 tests across 816 targets, 722.606 seconds,
  invocation 0651d4f5-bd0c-494c-87ca-8d775057ae2b. Its archive matched all 2,525
  Git blobs and executable modes. The gate includes release_gate, the complete
  rustc-frontend experiment, C/Java unit and compile-negative suites, shared
  codegen tests and documentation. Test caching remains enabled.

## Remaining boundary

Children 02–05 must still implement actual C/Java constant declarations, source
mapping and multi-crate publication. No custom runtime, legacy entry point or
existing test has been removed or disabled by this shared-layer step.
