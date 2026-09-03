# M34A-10 — Migrate Java 21 to typed generation

- Status: in-progress
- Depends on: M34A-09

The original implementation checkpoint was pushed as
`f4d9e1d539064ed70eb3c012537b99535ed344a0`. Independent review found gaps in
its claimed exit evidence. M34A-10 remains open until
[M34A-10R](M34A-10R-java-review-remediation.md) completes.

## Goal

Prove the complete architecture first in Java and eliminate the reported
opaque-runtime/manual-import design.

## Definition of done

- Java owns the complete `JavaType`, expression, statement, declaration,
  member, modifier, heritage, compilation-unit, package, and grammar/format
  enums in its language specification.
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
- Runtime declarations are Java AST and render through the ADR-0005
  render-ready certificate and total Java structural renderer.
- `JavaCode`, raw executable documents, `RUNTIME`, `require_java`, hard-coded
  import strings, and the legacy Java pipeline are deleted.
- The Java compliance row moves to **Pass** with exact source/test evidence.

## Tests

- `bazel test //crates/backend-java:all --nocache_test_results --test_output=errors`
- Java AST/verifier/catalogue/import/helper/certificate/total-renderer positive
  and negative matrices.
- Hermetic Java 21 lint-as-error compile, native/conformance tests, separate
  public consumer, invalid type fixtures, interface corpus, and three-generation
  determinism.
- All M17-M33 Java historical port targets.

## Commit gate

Commit and push `M34A-10: migrate Java to typed AST` only when all Java and
shared typed-generation gates pass in the dev container.

## Exit evidence

- Historical pre-ADR-0005 evidence: `crates/backend-java/src/ast.rs` owns the
  closed Java syntax, type-use, modifier, heritage, literal, operator, file,
  and former template model. The dialect
  catalogue and verifier are in `dialect.rs`; exhaustive CoreIR lowering is in
  `lower.rs`; structural helper declarations are in `runtime.rs`; and
  `render.rs` was a resolved-only strict Handlebars renderer; M34A-10V replaces
  that executable path before this task can complete.
- The legacy checked-in `Runtime.java`, raw Java document path, `JavaCode`,
  `RUNTIME`, `require_java`, and `serde_json` interpreter dependency are
  deleted. The typed-generation source policy now scans every production Java
  backend Rust source, while the dependency policy admits directives only in a
  certified typed import template.
- The shared linker coalesces one physical import used by multiple typed
  symbols, retains the exact symbol membership on that import, and rejects
  forged membership. Java tests lock the exact runtime import set and prove
  that generated files receive only reference-derived imports.
- The canonical interface corpus proves two flat interfaces on one immutable
  record, static and dynamic dispatch, first-class interface values in nested
  type positions, explicit final-field composition, and three-generation
  determinism. Invalid heritage, modifier, type-use, literal, operator,
  callable, constructor, and catalogue mutations fail closed.
- Hermetic Java 21 compilation uses `-Werror -Xlint:all`. Separate public
  consumer tests, native semantic tests, canonical conformance tests, and
  deliberate invalid-type compilation tests pass for both the base and
  interface packages.
- All 39 tracked historical Java generation/conformance/negative-type targets
  pass. The final uncached tracked-scope repository graph passes 288 of 288
  tests in the Linux development container; the user-owned untracked
  `examples/real-world/stdlib-abs` package is the only exclusion and was not
  modified.
