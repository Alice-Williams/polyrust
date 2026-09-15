# Certified dependency values

- Status: normative; implemented and verified in M35-03A-02F-02B-01
- Parent: [public scalar constants](rust-public-constants.md)

## Typed shared boundary

A generated dependency value is neither a known standard-library field nor a
zero-argument function. Extend LinkerDialect with an associated DependencyValue
type and an executable dependency_value_spec hook. Add DependencyValueSpec and
TargetSymbolRef::DependencyValue; retain DependencyCallable as a distinct enum
variant. Do not overload a string ID, a callable signature, or KnownField.

DependencyValueSpec carries the opaque symbol, exact certified package owner,
typed name, TargetTypeRef, DependencySpelling and SourceRef. It is reconstructible
metadata, not authority. Each plugin reconstructs every field from its opaque
value witness. The witness can be minted only from that backend's independently
certified immutable package and complete export inventory.

The shared layer does not interpret scalar values or claim mutability semantics.
This first transport profile accepts context-free primitive types and
catalogue-known zero-arity types. C's existing i32/i64 representation uses
Known(CStdType), so it must not be replaced by a duplicate primitive model.
Unknown/generic known types, generated, parameter, runtime and constructed type
references reject; cross-package type authority needs its own later capability.
Known types must have built-in, prelude or standard-library origin and no external
package requirement. A required executable verify_dependency_value_type hook
additionally validates the exact type against the dialect's opaque value witness;
catalogue membership alone does not certify scalar admissibility.
The backend's constant witness proves its readonly declaration, exact type,
initializer and owner. Reuse GeneratedValueId for owned target declarations;
do not invent another shared arena for constants.

## Linking and authentication

SymbolCatalogue stores dependency values separately from callables. Validate
duplicate symbols, exact witness/spec equality, type validity and native-name
collisions. Fixed-import collisions are checked across callable and value
categories when their dialect namespaces coincide; a function and object cannot
silently acquire the same C native name. Qualified-name identity follows the
dialect's qualified spelling. Do not invent aliases for fixed ABI names.

A value reference uses value_namespace and its witness-derived fixed import or
qualified name. Qualified values allocate no local import alias. Imports are
derived from actual references; neither lowering nor rendering receives a
caller-authored list. The shared layer deduplicates repeated symbol bindings while
retaining distinct function/value bindings. Native directive deduplication belongs
to the backend projection: C child 02 must prove a shared owner header is emitted
once without dropping either binding. Shared bindings are not rendered directives.
Certified binding identity includes the typed namespace. Same-spelled callable
and value imports in different namespaces retain distinct import identities,
even when their directive kinds coincide. Post-link checking reconstructs exactly
one binding domain from every import's symbols; mixed certified namespaces and
empty inventories reject. Existing catalogue-known owner imports use their own
domain: e.g. a type and its constructor may share one type import even when their
expression namespaces differ. No certified dependency can join that domain.

Original-package reconstruction, catalogue verification, reference inventory,
resolved-binding verification and file-import verification all cover values.
Mutating both the linked catalogue and its references must not bypass the
original unresolved package authority. Complete dependency exports still
participate in collision checks, including unreferenced public constants.

## Compatibility and proof

Backends without dependency-value support use an uninhabited enum and an
exhaustive impossible match in the hook. This adds no false capability claim and
must not disable their current tests. Do not default the hook to a fabricated
value or a generic success result.

Dedicated tests use typed opaque test witnesses and exercise fixed/qualified
spelling, mixed callable/value namespaces, deduplication, unresolved witnesses,
type/owner/name/spelling mutation, duplicate symbols, linked-only and coupled
catalogue/reference tampering. Existing callable proofs remain unchanged.
Keep substantial new logic/tests in separate modules; shared linking.rs should
only gain the required variant, field and dispatch wiring.
