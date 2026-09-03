# M34A-10V — Certify Java syntax and use a total structural renderer

- Status: planned
- Depends on: M34A-08R and the pushed M34A-10R round-14 baseline
- Blocks: resumption of M34A-10R blind review and M34A-11

## Goal

Make Java 21 the first concrete ADR-0005 language: a Java render-ready value is
an unforgeable proof capability, and every executable byte is emitted by a
total dependency-free structural renderer.

## Definition of done

- The Java post-link checker consumes the exact linked package, recomposes all
  runtime/user compilation units, checks the complete Java specification §10-11
  invariant set, and is the sole path to
  `RenderReadyPackage<JavaDialect>`.
- The Java renderer accepts only that certificate and has no fallible grammar
  branch; resource/path/manifest failures remain in their later shared phase.
- Every fixed Java keyword, punctuation mark, delimiter, separator,
  indentation/newline choice, literal escape, and precedence parenthesis is
  emitted by exhaustive Rust matches over closed Java enums.
- `JavaTemplateId`, Java executable Handlebars templates, serialized render
  views, and all Java template-engine calls are deleted.
- No third-party Java AST/code-generation/formatting library is added.
- Imports remain exclusively linker-derived typed references and are rendered
  structurally in the correct compilation-unit position.
- Runtime helpers and generated declarations pass through the identical
  post-link checker and renderer.
- Existing generated example snapshots remain byte-identical unless an
  intentional formatting-only change is documented and reviewed.

## Tests

- External Rust compile-fail cases prove Java callers cannot construct, mutate,
  deserialize, or render without the render-ready certificate.
- One positive and one negative verifier case for every Java AST variant plus
  contextual rules listed in Java specification §10-11.
- Deterministic structured mutation/compiler-oracle corpus: every accepted
  package renders infallibly and compiles with hermetic Java 21
  `-Werror -Xlint:all`.
- Paired native-negative fixtures for protected words, file/type identity,
  modifiers, annotations, heritage, interface methods, scopes, assignment,
  reachability, checked exceptions, generics, casts, arrays, wildcards,
  callables, constructors, statements, and expressions.
- Exact import, runtime composition, precedence, identifier/literal/comment,
  deterministic snapshot, public-consumer, native semantic, interface, and all
  historical Java port tests.
- Source-policy test proves no executable template, raw/token/source escape,
  wildcard renderer match, or string node dispatch exists.
- `bazel test //crates/backend-java:all --nocache_test_results --test_output=errors`
- Full tracked repository, release gate, fresh Cargo workspace, Rustfmt, Clippy,
  Buildifier, and documentation gates in the Linux development container.

## Commit gate

Commit and push `M34A-10V: certify total Java rendering` only after every named
proof passes. Verify local and remote SHAs, require hosted CI for that exact
commit, then resume the independent blind-review loop.

## Exit evidence

To be filled with exact targets, corpus sizes, generated examples, commit SHA,
remote SHA, hosted CI run, and reviewer disposition after implementation.
