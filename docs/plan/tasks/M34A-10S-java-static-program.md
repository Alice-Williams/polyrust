# M34A-10S — Generate Java from the static portable AST

- Status: in-progress — implementation and local proof complete; hosted CI pending
- Depends on: M34A-08S and M34A-10V
- Blocks: Java static-path review and M34A-11 static migration

## Goal

Make Java the first language to declare compile-time support for `StaticV1`
and generate every admitted static program through the certified structural
Java pipeline.

## Definition of done

- `JavaDialect` explicitly implements `Supports<StaticV1>`; there is no
  blanket or runtime support claim.
- The Java static entry point accepts only `StaticProgram<F>` with the bound
  `JavaDialect: Supports<F>`.
- No user-caused portable typing, target capability, Java syntax, import,
  constructor, or rendering diagnostic is possible after static construction.
- The existing Java linker remains the sole owner of names and imports.
- The existing Java post-link certificate and total renderer remain the sole
  executable-source path.
- A checked-in static example contains a typed record construction and a
  nested arithmetic expression whose fully parenthesized Java output is
  reviewed.
- The old dynamic Java API remains clearly labelled and all its evidence stays
  green.

## Tests

- Compile-fail proof that a non-static checked program cannot call the static
  entry point.
- Compile-fail proof that a language/profile without `Supports<F>` cannot use a
  static generation adapter.
- Three-generation byte determinism for the static example.
- Hermetic Java 21 compilation with `-Xlint:all -Werror` and execution of the
  generated record-construction and nested-expression behavior.
- Every M34A-08S compile-pass/fail test.
- Every existing Java verifier, linker, renderer, interface, conformance,
  public-consumer, negative, mutation-oracle, and snapshot test.
- Full tracked repository and release gate in the Linux development container.

## Commit gate

Commit and push `M34A-10S: generate Java from static programs` only after every
named proof passes. Verify local and remote SHAs and require all hosted CI jobs
for that exact commit before handing the Java static path to the user for
review.

## Exit evidence

- `JavaDialect` is the sole Java type implementing `Supports<StaticV1>`.
  `JavaBackend::generate_static` requires that evidence and accepts only
  `StaticProgram<F>`; two compile-fail doctests reject `CheckedProgram` and a
  generic profile without Java support.
- The checked-in `generate_static_v1.rs` authoring example generates the same
  six-file certified package as the dynamic pipeline. The generated
  `Generated.java` contains `Point(int x, int y)`, parenthesized addition,
  subtraction, and multiplication, and typed `new Point(x, y)` construction.
- Three generations have byte-identical canonical manifests. Focused Bazel
  invocation `78f59108-ce93-40ae-88d6-089c26f0b36b` passed the static
  compile-fail, backend determinism/source-shape, and separately compiled Java
  consumer tests without cached results.
- The consumer compiles with hermetic Java 21, `-Xlint:all`, and `-Werror`, then
  executes `computed() == 45` and verifies `make_point(3, 4)` field values.
- Complete Java/lint invocation `54afd897-4ec3-440b-9bba-d7ecdf86864f`
  passed all 24 tests without cached results, including every existing Java
  verifier, interface, public-boundary, negative-compiler, conformance, and
  mutation-oracle proof plus Rustfmt, Clippy, and Buildifier.
- Full tracked Bazel invocation `67b27c9d-c692-41c0-9838-80d81226711b`
  passed all 299 tests. The expanded release invocation
  `f9a3d081-240a-4d46-91c2-ab104148c200` passed all 236 tests without cached
  results, including Rustfmt, strict Clippy, Buildifier, documentation, source
  policy, dependency policy, and all generated-language compile/run gates.
- Deterministic all-target conformance invocation
  `68f050ad-8986-43d5-88b8-c027d6c50b16` passed: 50 cases, one portable test,
  the evaluator and all eight targets agree, and repeated manifests are
  byte-identical. Workspace Cargo tests and doctests passed with all features,
  strict Clippy passed, and `cargo audit` reported no known vulnerabilities.
- Implementation commit, remote SHA, and hosted CI evidence are pending.
