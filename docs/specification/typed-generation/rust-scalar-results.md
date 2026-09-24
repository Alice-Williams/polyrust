# Checked Rust no-heap scalar results

- Status: compiler identity experiment and C target transport/proof complete;
  Java transport and compiler admission planned
- Plan: [05A](../../plan/tasks/M35-03A-05A-scalar-results.md)
- Targets: [C17](languages/c/rust-scalar-results.md), [Java21](languages/java/rust-scalar-results.md)

## Closed source semantics

Initial instance: core Result<i32, core::num::TryFromIntError>. Success carries
an exact I32; error carries the original opaque standard error identity.
Success(0) differs from error. Error formatting, equality, layout inspection
and arbitrary construction are not admitted. No Rust-owned heap, destructor,
reference payload, arbitrary generic instantiation or custom Result lookalike.

The compiler-only probe may observe normalized aliases for identity tests;
the production alias guard remains unchanged. The probe is not an admission
certificate and cannot issue a RenderReadyPackage.

## Compiler authority

Discover standard Result and Ok/Err via compiler diagnostic/language-item
identities. Derive the opaque error from the canonical standard
TryFrom<i64> for i32 associated Error projection. Verify the external core
anchor, exact I32 argument, original enum/variant/payload field identities,
normalization and absence of drop obligations. Names are diagnostic text only.
Keep witnesses private and tied to the compiler session.

## Shared source and target boundaries

Before source admission, add a bounded original nominal-instance model that
includes the defining declaration and exact type arguments; do not infer it
from target record/class names. A closed enum describes supported instance/
variant kinds. Dependency certificates retain original instance identity and
reconcile every target variant/member/signature. Serialized metadata alone
grants no authority. All resource and publication limits remain enforced.

Lower construction, transfer and exhaustive match through executable capability
bindings. Original HIR and checked field/variant identities authorize payload
access only in the corresponding match arm. Materialize the scrutinee once;
unselected arms have no effects. Renderers only consume certified target AST.

## Observations and scope

The Rust native driver can create both variants using the standard library and
call translated functions; this does not admit conversion into translated code.
Ok construction, propagation/reconstruction of an existing Err, signatures and
two-arm match suffice to prove result transport before narrowing admission.
Keep compile validity, compiler authority and functional tests distinct. No
full collection, general enum or ownership-migration claim follows.
