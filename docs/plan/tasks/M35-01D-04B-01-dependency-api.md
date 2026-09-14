# M35-01D-04B-01 — Certified C dependency API

- Status: complete
- Depends on: [M35-01D-04A](M35-01D-04A-compiler-crate-identity.md)
- Parent: [M35-01D-04B](M35-01D-04B-certified-c-dependencies.md)
- Contract: [crate dependencies](../../specification/typed-generation/languages/c/rust-hir-crate-dependencies.md)

## Goal

Expose a read-only language-owned dependency API from a real C certificate,
retaining exact public references, linked names, header and checked resource
evidence. Do not yet admit imported calls.

## Definition of done

- Private evidence constructors require RenderReadyPackage<CDialect>, retain
  its immutable authority, and validate the exact one-crate public inventory.
- Lookup by compiler declaration identity returns only public scalar functions;
  private functions/fields, wrong owners and non-package certificates reject.
- Retain certificate-derived call-effect/stack evidence, never caller-authored
  purity or frame values. The API does not claim rustc analysis from metadata.
- No deserializer, raw header/symbol constructor, import registration or direct
  foreign-call bypass is added in this slice.

## Tests and proof

- Read-only API matches actual certified names/signatures/header/exports.
- Private/nonexistent/foreign lookup and malformed source/API provenance reject.
- Compile-negative tests prevent manual evidence construction and unchecked
  input to the certificate constructor.
- Resource bounds come from existing complete-package measurements; no separate
  estimator or zero-cost external-leaf convention.
- Full C/shared/Java/compile-fail/lint/docs gates and a fresh review pass.

## Evidence

- Added a focused certificate-owned API and separate inventory reconstruction
  module. It reuses CDefinedFunction, CGeneratedHeader, ScalarCalls and complete
  resource measurement. It introduces no imported-call bypass or deserializer.
- Reused a dedicated source-origin test fixture. Tests cover i32/bool, private
  record storage, exact certificate/name/signature/header/frame values, retained
  witness lifetime, private/missing/foreign lookup and cross-registry rejection.
  Coordinated missing/extra/private/foreign/namespace/reachability/visibility
  metadata changes cannot manufacture a dependency API from otherwise valid C.
- The real compiler public-package probe now proves exact dependency witness
  membership against compiler function/export identities, including private
  helpers, public aliases and immutable records/shared references.
- `ee7bca44-0ebb-42c1-b560-f0bf8c72763e`: focused 10/10 C, compiler,
  compile-fail, Clippy/rustfmt/buildifier/docs and source-policy targets pass.
- `48a14861-6e00-422d-ad2d-34b8bf6bd64e`: complete release, compiler,
  C/shared/Java and lint regression passes: 351 targets, 311/311 tests, all
  tests enabled (390 seconds; 70 executed, the remainder cached).
- Fresh independent Sol Extra High read-only review found no substantiated
  core errors. No imported-call or metadata-authentication claim is made.
