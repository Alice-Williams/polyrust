# M35-01E-04C-01 — Typed Java foreign-call lowering

- Status: complete
- Parent: [M35-01E-04C](M35-01E-04C-java-compiler-graph.md)
- Depends on: M35-01E-04B

## Implementation contract

- Resolve ordinary calls to compiler DefId, separating reachable local bodies
  from foreign declarations. Never request a foreign HIR body or copy it into
  the consumer. Preserve bounded local discovery and deterministic identities.
- Resolve each foreign declaration through the same-analysis graph lookup,
  compare exact RustDeclarationId and compiler-mapped JavaMethodSignature with
  the immutable owner function, then import it into a consumer-owned scope.
- Keep local generated callable references and opaque dependency references in
  typed categories. The DirectCalls capability maps both while preserving argument
  evaluation order; signatures come from registered references, not names.
- Freeze the scope into the original compilation unit. Retain the existing
  single-crate adapter's rejection of foreign calls without authenticated owners.
- Keep foreign registration/signature joining separate from body assembly.

## Definition of done and tests

- Existing single-crate Java source, AST, compile-fail, negative and format gates
  pass without weakening their assertions.
- Actual compiler-backed foreign calls with zero and multiple scalar arguments
  lower to Dependency references; same-spelled owners remain distinct.
- Wrong declaration and signature proof substitutions reject before rendering.
- The graph/native evidence is provided by C02/C03 before parent C completes;
  compiling this internal bridge alone is not a claim of cross-crate completion.

## Evidence

- Single-crate source/AST/negative/capability/format gate
  `ba82cd94-1c9b-44ba-b89b-68b24cdc7e30`: 11/11 tests pass.
- Actual compiler graph admission gate `ef753bfc-39af-40db-b0b4-d0641c786fab`:
  6/6 targets pass, including two-crate/diamond, same-named owners, repeated aliases,
  unused declarations and source/documentation metadata disagreement.
- Mutation/capability/format gate `fe1a6b0d-b7de-465d-bc11-544fa70e5a97`:
  9/9 targets pass. Independently compiled test-only adapters substitute a different
  authenticated owner, a different public declaration, or an altered compiler-side
  scalar signature; all reject before output. No production mutation switch exists.
- The non-publishing graph adapter supplies the compiler harness for these joins;
  C02/C03 still own complete graph inventory, native comparison and final review.
