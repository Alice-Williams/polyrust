# M35-01E-04A — Java certificate-derived owner APIs

- Status: complete
- Parent: [M35-01E-04](M35-01E-04-java-crate-bundles.md)
- Depends on: M35-01E-03

## Implementation contract

- Add a Java-owned resolved source-declaration inventory, derived from original
  registered declarations during linking and included in exact post-link
  rederivation. It retains the source identities/exports and typed signatures
  needed by an owner API. Do not expose or recover the unresolved AST from a
  LinkedTargetPackage; the existing shared phase boundary remains unchanged.

- JavaDependencyApi consumes an existing RenderReadyPackage<JavaDialect>.
  It exposes immutable JavaDependencyFunction handles, never constructors from
  source names, signatures, manifests, unchecked ASTs or provenance records.
- Retain the exact owner certificate with Arc identity. Copies share authority;
  independently certified packages with identical source IDs are distinct proofs.
  Pointer identity authenticates only and never affects emitted order or spelling.
- Reconstruct the complete public scalar API from certified declarations, resolved
  member names and the source export graph. Require one canonical RustCrate facade,
  exact owner/visibility/signature agreement and all public bindings represented.
  Private helpers and records must not receive public handles.
- A separate focused inventory/closed-body policy checks the admitted immutable
  scalar-call subset; do not derive purity merely from a function signature flag.
  This is target representation admission, not another Rust ownership checker.
- Preserve source aliases and docs as descriptive owner metadata, not authority.

## Definition of done and tests

- Positive certificates cover zero/four parameters, both scalars, private helpers,
  private records, same-spelled source declarations and public aliases.
- Reject incomplete public inventories, wrong facade/owner, non-scalar exports,
  inconsistent actual visibility, effectful/unadmitted body shapes and recursion.
- Compile-negative tests prohibit forging handles or accepting unchecked packages.
- Clones share exact proof identity; independently certified different bodies do
  not compare equal even if registration/source IDs and signatures coincide.
- Focused Bazel, Java units/compile-negative/lint gates and a fresh independent
  review pass before completion. This checkpoint does not yet emit foreign calls.

## Preparation evidence

Independent typed fixtures now cover public zero/four-parameter int/bool methods,
a private helper, alias metadata, docs, a nonconstructible facade and independently
certified different bodies with matching source IDs. Existing Java verification
and rendering accept them: Java suite 250/250 under `40161ab5-5284-4f07-8398-f175d50450ca`;
full/lint/docs gate `6d9d75bc-7588-4b40-8a5a-5a149c2e4f9b` passed 363/363.
These fixtures are target-only evidence, not rustc provenance or a dependency API.

## Implementation progress

The resolved source inventory, opaque JavaDependencyApi/Function/Package handles,
complete export reconciliation and bounded closed-body/record policy are implemented.
The compiler-backed public-package probe derives the owner API from its actual
certificate and compares every exported declaration against rustc; native Java 21
still compiles/runs the identical production output. Unit tests independently
exercise certified-but-unadmitted mutation, arithmetic, recursion, constructors,
missing/private/unmapped exports, non-scalar signatures and distinct owner proofs.

Focused gate `f05cce47-3c69-4d81-ad37-49aa25ae411b` passed 7/7 (Java units,
compile-negative contracts, compiler/native AST/API test, Rustfmt, Clippy,
Buildifier and docs). Full migration regression gate and independent review
remain required before completing this checkpoint.

The pre-review full gate `112e31bb-4575-4778-b50e-37380fde2501` passed 363/363,
including 272 Java units. Independent review nevertheless identified three
accepted defects: constructors needed the same body budget as methods, shared
export graphs needed allocation-aware comparison caching, and the registration
projection needed backend-private visibility. All three are fixed with mixed
constructor/method exact/one-over, instrumented graph comparison and external
compile-negative tests. The first two fixes passed focused gate
`32c0d9a8-df86-4dfd-96f2-f497a7f3ecd9` (7/7). Added compiler/native same-spelled
source functions within one crate and certified legacy/wrong-facade rejection
fixtures to close the reviewer-noted definition-of-done coverage gaps.
Final full gate and a fresh reviewer remain required.

The legacy negative fixture initially hit the stronger existing qualifier check
by calling a TestHarness-origin class Generated. It now uses the legitimate
PackageEntryPoint origin and proves that the resulting general Java certificate
still cannot acquire a Rust-source owner API without source declarations. No
verifier was relaxed. Final full gate `17d2913d-87b6-4d51-9d9f-b0258bb4a689`
passed 363/363; Java units passed 275/275 with none ignored or filtered.
The original reviewer confirmed all three fixes and no remaining core defects;
the fresh independent final review subsequently concluded clean: no core defects
or material test gaps. The reviewer independently checked exact owner authority,
export/signature/body admission, shared budgets, allocation memoization, private
projection, compiler/native coverage and phase boundaries. Documentation gate
`0841dfd0-3509-41d3-af59-8c7a26242094` also passed. E04A is complete; consumer
bindings and imported calls remain E04B. No tests were disabled and no push was made.
