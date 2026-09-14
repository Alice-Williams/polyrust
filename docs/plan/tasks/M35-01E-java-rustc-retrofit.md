# M35-01E — Retrofit Java to compiler-checked Rust

- Status: in-progress
- Depends on: completed M35-01B, M35-01C and M35-01D
- Contract: [Java Rust HIR lowering](../../specification/typed-generation/languages/java/rust-hir-lowering.md)

## Goal

After the no-heap C integration is complete, feed the same compiler-checked
Rust source into the existing Java typed AST, linker, certifier and renderer.
Do not translate generated C or introduce another generic/Rust AST.

## Definition of done

- Isolate rustc inspection in the compiler adapter; backend-java has no rustc
  dependency and uses its existing typed references and capability builders.
- Share successful-analysis entry, compiler configuration and provenance
  extraction. Keep target admission and typed mappings language-owned.
- Use executable, feature-specific mapping slots. Do not advertise whole
  portable capabilities from a narrower Rust-source implementation.
- Implement the C milestone's admitted source subset with explicit Java value,
  immutable record and shared-reference representation mappings.
- Keep crate identity, logical modules, docs and declared/effective visibility;
  generate only actual public exports into the Java facade.
- Require the existing RenderReadyPackage<JavaDialect> before rendering.
- Derive imports from registered typed dependencies, never import strings.
- Keep historical Java builder/examples/conformance tests enabled; migrate
  their frontend only when their replacement evidence is present.
- Produce inspectable ignored Java sources in the workspace.
- Obtain a fresh independent review and resolve each demonstrated defect.

## Ordered implementation checkpoints

1. [M35-01E-01 — Shared compiler capability contracts](M35-01E-01-source-capabilities.md).
2. [M35-01E-02 — Java Rust-source identity and package model](M35-01E-02-java-source-identity.md).
3. [M35-01E-03 — Java HIR mappings and certified single-crate output](M35-01E-03-java-hir-mappings.md).
4. [M35-01E-04 — Certified Java dependencies and crate bundles](M35-01E-04-java-crate-bundles.md).
5. [M35-01E-05 — Native equivalence and migration closure](M35-01E-05-java-native-proof.md).

Each checkpoint retains its own evidence and review. A structural Java fixture
does not prove compiler admission; a single-crate fixture does not prove foreign
dependency support. All historical tests remain enabled throughout.

The representation-neutral source-contract extraction may proceed alongside
the final read-only C native-proof review, after the C implementation/native
gate passes. Java target-model and mapping implementation still require C
checkpoint closure. The extraction must preserve generated C bytes.

## Tests and proof

- The same source fixtures and input corpus agree between native Rust, C17
  O0/O2 and Java 21. Compare behavior, not source spelling.
- Invalid Rust fails before either backend; valid unsupported Rust has a
  target capability diagnostic and leaves artifacts absent or unchanged.
- AST assertions prove scalar types, operator binding, field/constructor
  identity, lexical scopes, qualifiers/reference representation and exports.
- Wrong-registry, cross-session, forged provenance and certificate-bypass
  regressions remain rejected at the relevant boundary.
- Doc escaping, keywords, symbol collisions, public-header/facade consumers,
  private implementation hiding and three-render determinism have coverage.
- All Java verification/resource/compile-negative tests, native Java lint,
  Rustfmt, Clippy, Buildifier, docs, release and cross-language gates pass.

## Commit and push gate

Keep C and Java completions in separate milestone commits. Do not push any
migration checkpoint until the full local Linux/Bazel test gate is green.
Do not disable failing tests to conceal a regression. No CI polling loop is
required while pushes are held.
