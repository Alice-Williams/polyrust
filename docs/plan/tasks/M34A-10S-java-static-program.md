# M34A-10S — Generate Java from the static portable AST

- Status: planned
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

Pending implementation.

