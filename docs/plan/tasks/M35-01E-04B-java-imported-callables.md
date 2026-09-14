# M35-01E-04B — Consumer-owned Java dependency calls

- Status: complete
- Parent: [M35-01E-04](M35-01E-04-java-crate-bundles.md)
- Depends on: M35-01E-04A

## Implementation contract

Execute these focused checkpoints in order:

1. [M35-01E-04B-01](M35-01E-04B-01-dependency-spelling.md): shared typed spelling policy and C compatibility.
2. [M35-01E-04B-02](M35-01E-04B-02-java-consumer-bindings.md): Java scope, call mapping and certificate integration.
3. [M35-01E-04B-03](M35-01E-04B-03-java-dependency-native-proof.md): independent native consumers and adversarial closure.

- Introduce a Java-owned dependency binding scope with opaque consumer identity.
  Import a certificate-derived function through this scope; freeze its exact
  registrations into the original Java file/package projection before verification.
  Calls carry consumer-owned references, not raw foreign arena indices.
- The Java AST dependency-call variant retains that exact imported handle.
  Signature checks and package catalogue reconstruction authenticate it against
  the original scope; post-link verification repeats the reconstruction.
- Extend shared dependency spelling with an explicit enum: fixed imports retain
  existing C semantics; qualified references retain a typed language-owned name.
  Fixed-import collision rules must not reject distinct qualified Java owners.
- Java qualified static calls contain the certified RustCrate package, facade and
  resolved method identifier. Render structural qualification directly. No source
  strings, fabricated native aliases, unused static imports or foreign body copies.
- Derive used dependency owners and any import directives from actual typed uses.
  Fully qualified Java calls produce no import directive. Charge new names/scopes
  and dependency inventory against explicit target resource limits.

## Definition of done and tests

- Handwritten independent Java consumers compile and execute cross-package calls
  under Java 21 -Xlint:all -Werror, including identical member names in two owners.
- Wrong scope/session, owner, signature, private handle, omitted registration and
  altered post-link catalogue/name/import inventories reject before certification.
- Used/unused dependency tests prove exact owner selection and zero unnecessary
  import directives. Repeat rendering deterministically.
- Shared linker/C regression tests preserve fixed-import and collision behavior.
- Full relevant Bazel gates and fresh broad independent review pass.

## Completion evidence

All three child checkpoints are complete. B-03 records the independent native
Java proof, full gate `08de0334-1c70-4fd0-8dae-fe7116b77d23` (363/363 tests),
and clean fresh Sol Extra High review. E04C/E04D remain required before the
compiler-authenticated Java graph and production bundle claims are complete.
