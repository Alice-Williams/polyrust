# M35-01E-02 — Java Rust-source identity and package model

- Status: complete
- Parent: [M35-01E](M35-01E-java-rustc-retrofit.md)
- Depends on: M35-01E-01

## Goal

Extend the existing Java AST and verification pipeline to retain Rust-source
ownership without pretending Rust declarations are CoreIR or runtime symbols.

## Implementation

The package/path/resource change is isolated in
[M35-01E-02A](M35-01E-02A-java-crate-packages.md). Source-origin/member/doc
authentication follows that slice, before HIR mappings. Registration/package
coherence is isolated in [M35-01E-02B](M35-01E-02B-java-source-registration.md);
record member identity is isolated in
[M35-01E-02C](M35-01E-02C-java-source-records.md). Documentation/export metadata
authentication follows in [M35-01E-02D](M35-01E-02D-java-source-documentation.md).

- Add a typed crate package identity and derived canonical source directory;
  retain the existing Generated package and exact Runtime restrictions.
- Register the crate facade as a synthesized PackageEntryPoint; genuine private
  nominal types, callables and fields carry RustSource origins. Derive declaration
  paths from actual registered owners. Modules are not fabricated nominal types.
- Add explicit immutable record member/constructor references where existing
  Core/runtime-only categories cannot express authenticated Rust members.
- Authenticate source fields by nominal owner plus Rust declaration identity,
  not the Structural name-only field alternative. Verify exact declaration,
  type, access, finality and constructor initialization.
- Add structured resolved-doc attachment and escaping, accounted for by the
  existing verifier/resource/spelling passes. No raw Java comments or code.
- Keep CoreIR conformance inventories separate; source records cannot mint
  portable interface conformance witnesses.
- Derive binary-name and constant-pool/resource costs from each actual typed
  package, replacing Generated-only prefix assumptions on the source path.

## Definition of done and tests

- A manually constructed structural target fixture passes the existing
  RenderReadyPackage gate and compiles under Java 21; this is target evidence
  only, not proof of compiler provenance.
- Wrong package/path, crate owner, registry, member owner, signature, visibility,
  duplicate declaration and forged witness mutations reject before rendering.
- Hostile docs, Java keywords, same-spelled cross-crate names, private nominal
  access and three-render determinism have positive/negative tests.
- Existing Java runtime, resource and builder gates stay green; fresh review
  resolves all core issues.

## Completion evidence

Children M35-01E-02A through M35-01E-02D are complete. Final full container gate
`1d9bfd30-4a69-4830-9406-7dca023ca0db` passed 343/343 tests; each child has
independent review evidence. This proves the target foundation, not yet a Java
compiler-source frontend; M35-01E-03 implements that adapter.
