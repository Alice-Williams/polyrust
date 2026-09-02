# M34A-10 — Migrate Java 21 to typed generation

- Status: planned
- Depends on: M34A-09

## Goal

Prove the complete architecture first in Java and eliminate the reported
opaque-runtime/manual-import design.

## Definition of done

- Java owns the complete `JavaType`, expression, statement, declaration,
  member, modifier, heritage, compilation-unit, package, and template enums in
  its language specification.
- Every CoreIR feature has an exhaustive typed Java strategy and every known
  JDK type/method/field/constructor has one authoritative signature catalogue.
- Primitive/boxed generic contexts, evaluation order, sealed/tagged values,
  Unicode, checked arithmetic, exact F64 bits, immutable bytes/lists, and
  failures are verifier checked.
- Flat interfaces, multiple conformance, first-class/nested interface values,
  dynamic dispatch, and explicit final-field delegation pass.
- Optional target heritage accepts only a final one-edge external adapter and
  rejects generated chains/reuse; no portable fixture requires it.
- The resolver derives every package/import/qualification/helper/file from
  typed references, including all former `Runtime.java` dependencies.
- Runtime declarations are Java AST and render through strict Java Handlebars
  templates.
- `JavaCode`, raw executable documents, `RUNTIME`, `require_java`, hard-coded
  import strings, and the legacy Java pipeline are deleted.
- The Java compliance row moves to **Pass** with exact source/test evidence.

## Tests

- `bazel test //crates/backend-java:all --nocache_test_results --test_output=errors`
- Java AST/verifier/catalogue/import/helper/template positive and negative
  matrices.
- Hermetic Java 21 lint-as-error compile, native/conformance tests, separate
  public consumer, invalid type fixtures, interface corpus, and three-generation
  determinism.
- All M17-M33 Java historical port targets.

## Commit gate

Commit and push `M34A-10: migrate Java to typed AST` only when all Java and
shared typed-generation gates pass in the dev container.
