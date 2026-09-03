# M34A-10T — Generate Java from inferred capabilities

- Status: in-progress
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

The shared removal and Java's first consumer migration form one atomic
buildable checkpoint: `M34A-08T/M34A-10T: infer typed Java capabilities`.
Require hosted CI for that exact implementation checkpoint before marking this
task complete and handing Java back for review.

## Exit evidence

- `JavaDialect` explicitly implements all 18 initial `Supports<F>` witnesses;
  the typed entry point requires the recursively derived `SupportsAll<R>`.
- The checked-in example uses an exact three-argument function call, a
  three-field record construction, field-preserving public consumption, and
  nested left-to-right arithmetic producing `50`.
- Three generated manifests are byte-identical. Hermetic Java 21 compiles the
  generated API and separate consumer with `-Xlint:all -Werror`.
- Every Java Bazel target passed: 21/21
  (`5e6dd0c8-4ebb-4742-8280-05c17bd9c324`). The complete tracked and release
  evidence is recorded in M34A-08T and also includes all historical Java
  verifier, mutation, interface, native, snapshot, and conformance targets.
- The locked Cargo suite passed both Java compile-fail examples and all 75 Java
  unit tests.
- Pending: push the immutable implementation checkpoint and require hosted CI
  success for its exact SHA before user review.
