# M35-03A-02F-02B-03A — Owned Java constants and producer APIs

- Status: complete
- Parent: [Java constant APIs](M35-03A-02F-02B-03-java-constant-api.md)
- Depends on: M35-03A-02F-02B-02

## Contract

Implement the producer half of the [Java constant specification](../../specification/typed-generation/languages/java/rust-public-constants.md).
Reuse generated value identities, JavaField and typed primitive literal
initializers. Admit public static final boolean/int/long fields only on the
registered crate facade, with exact source registration, visibility, owner,
type, initializer and resolved declared path agreement.

Add opaque JavaDependencyConstant witnesses derived only from the original
render-ready package. Reconcile one complete public function/constant export
inventory, including aliases and constants-only packages. Source descriptions
gain a distinct constant case; source-byte reservations include fields and their
generated value reads. Method-body admission accepts reads only from the
verified constant inventory, without enabling mutable globals or arbitrary fields.

## Implementation boundaries

- Isolate constant inventory validation and producer witnesses into focused
  dependency API modules; keep existing function and record checks intact.
- Reuse existing registration/declaration/finality/initializer checking and
  the shared generated-value reference path. No custom Runtime class, boxing,
  accessor methods, raw Java or alternative renderer.
- Charge field declarations, literal syntax and every qualified reference through
  the existing source/resource policy. No fabricated source method/call frame.
- Consumer dependency-value registration, source compiler admission and bundle
  publication remain separate children; no legacy entry point is removed.

## Definition of done and tests

- Constants-only and mixed typed facades compile under strict Java21 checks.
  Independent separately compiled consumers prove both booleans, int/long
  extremes, a wide exact integer and an ordinary value; final-field assignments
  fail compilation. A compiling value/reference mutant fails independent truth.
- Opaque witnesses retain exact certificate, source declaration, generated value,
  declared path, primitive type and literal. Clones retain authority; separate
  certificates with the same declaration remain distinct.
- Missing/extra/duplicate field identities, wrong modifier/type/value/owner/path,
  synthesized/private/disconnected exports and coordinated metadata mutations
  reject at the appropriate boundary. Compile-negative tests prevent forgery.
- Reads, aliases, documentation/descriptions and source bounds work without a
  first source method. Existing function/record/source tests remain enabled.
- Inspect actual generated sources outside the container, keep them ignored,
  update evidence/specification, obtain a fresh clean independent review and pass
  the isolated full Bazel/Rust/Bazel-lint gate before a scoped commit and push.

## Implementation and proof inventory

- JavaDependencyConstant is an opaque certificate-owned value witness with exact
  generated/source identity, declared path, primitive type and literal. Clones
  preserve authority; independently certified identical declarations stay distinct.
- Complete public function/constant export reconciliation admits constants-only
  facades, preserves aliases and rejects missing/extra/conflicting declarations.
  Registered public static final scalar literal fields use the existing AST and
  renderer. Method-body reads require the verified constant inventory.
- Borrowed Constant descriptions, original source/module documentation and
  conservative source reservations include field declarations and every qualified
  value read. Long repeated names and expanded documentation have direct bound
  tests. Existing JVM class/field/constant-pool policy remains enabled.
- Nine new tests cover exact constants-only/mixed APIs, witness lifetime,
  field/registration/export tampering, direct resolved path/type/literal checks,
  repeated name/resource accounting, typed assignment rejection and native proof.
- The native Java21 test separately compiles two producers and independent
  consumers: false/true, signed32/64 limits, 9007199254740993 and 62. All sixteen
  final-field assignments fail compilation. Sixteen deliberately wrong-value
  producers compile, but their freshly compiled independent consumers fail truth.
  Recompilation is essential because javac may inline constant variables.
- No new runtime, boxing, copied source, accessor helper or dependency was added.
  No legacy functionality/test was removed or disabled. Consumer imports and
  rustc/bundle integration remain later children, with explicit fail-closed
  Constant cases at the current bundle boundary.

## Verification evidence

Focused tree 5282894c9295ac1654fa430095a76fef1f3f9c72 passed all seven targets:
Java unit/native tests and compile-negative tests, Rust Clippy, rustfmt,
buildifier, documentation and source policy. Invocation
0f6bb405-bc42-4666-ac08-040713ed9904; 107.506 seconds.

Final implementation tree 456db9e4b3cf598f4145f946e3cec6621c3abc45 includes the
additional typed-assignment rejection test and clarified specification. All 2,564
archived Git blobs and executable modes were verified. The isolated full gate
passed 624 tests across 816 targets in 268.946 seconds, invocation
9e4135fe-e0dd-43c6-8401-69b6bb067f3c. It includes 319 ordinary Java tests,
Java compile-negative checks, existing C/shared codegen tests, all rustc-frontend
experiments, release integration and Rust/Bazel/source-policy lint gates.
Caching remained enabled.

A fresh independent Sol Extra High review of that exact implementation tree
found no actionable correctness, security, architecture or test defects. It
audited the full diff and surrounding certificate, source inventory, visibility,
type/literal, ownership, export, body-read, description and resource invariants.

Actual generated sources and independent consumers were exported and inspected
at ignored host directory generated/examples/java-constant-producers-456db9e,
with ConstantsOnly and Mixed subdirectories. A retained-sandbox Bazel rerun also
passed all 319 Java cases in 96.438 seconds overall, invocation
9e49aa68-b542-452d-ab97-fcaa7b8d9195. No compiled classes or generated source
is staged.

The documentation checkpoint is gated again before commit/push. This completes
only Java producer constants, not authenticated consumers or source integration.
