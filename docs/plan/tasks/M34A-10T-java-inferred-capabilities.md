# M34A-10T — Generate Java from inferred capabilities

- Status: planned
- Depends on: M34A-08T and M34A-10S
- Blocks: Java inferred-path review and M34A-11

## Goal

Make Java the first dialect to advertise individual typed-program features and
generate an arbitrary-arity `TypedProgram<R>` only when
`JavaDialect: SupportsAll<R>`.

## Definition of done

- Java implements `Supports<F>` explicitly for every implemented initial
  feature and has no profile-wide or blanket support claim.
- Java's typed entry point accepts only `TypedProgram<R>` under
  `JavaDialect: SupportsAll<R>`.
- The checked-in Java example uses inferred requirements and contains a
  three-parameter function plus a three-field record.
- The existing Java linker, dependency resolver, post-link certificate, and
  total renderer remain the only executable-source path.
- The dynamic checked-input API remains separately fallible.

## Tests

- Compile-fail a generic requirement tree without the required Java proof and
  a concrete dialect lacking one used feature.
- Generate three byte-identical manifests.
- Compile generated Java with hermetic Java 21, `-Xlint:all -Werror`, and run a
  separate consumer proving the three-argument call and three-field record.
- Run every existing Java verifier, compiler oracle, mutation, interface,
  snapshot, and conformance test.
- Run Rustfmt, strict Clippy, Buildifier, documentation tests, full tracked
  Bazel tests, and the release gate in the Linux development container.

## Commit gate

Commit and push `M34A-10T: generate Java from inferred capabilities` only
after all named proof passes. Require hosted CI for the exact implementation
checkpoint before handing Java back for review.

## Exit evidence

Pending implementation.
