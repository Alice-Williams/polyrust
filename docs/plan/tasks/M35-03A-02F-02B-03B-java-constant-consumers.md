# M35-03A-02F-02B-03B — Authenticated Java constant consumers

- Status: complete
- Parent: [Java constant APIs](M35-03A-02F-02B-03-java-constant-api.md)
- Depends on: M35-03A-02F-02B-03A

## Contract

Complete the consumer half of the [Java constant specification](../../specification/typed-generation/languages/java/rust-public-constants.md).
Register only JavaDependencyConstant witnesses in the existing dependency scope.
Add JavaImportedValue and an explicit dependency-value variant to JavaValueRef.
Derive shared DependencyValueSpec rows and qualified field paths from retained
producer authority; never invent KnownField, free-form paths or synthetic calls.

Update exact scope freezing, references, typing, capability admission, dependency
catalogues, post-link reconstruction, renderer and resource/source accounting.
Traverse all retained value/function owners, including unused registrations and
transitive dependencies, with existing namespace and conflicting-authority rules.
Foreign fields never become owned declarations and cannot be assigned.

## Definition of done and tests

- Values-only and mixed independently compiled Java21 consumers agree with exact
  independent truth. Repeated reads and mixed calls retain one coherent producer
  scope, without Runtime output or boxing.
- Wrong scopes, stale certificates, incompatible primitive types, paths, owners,
  source IDs and coupled catalogue/reference changes reject. Public compile-
  negative tests prevent forged witnesses and unchecked renderer entry.
- Constants-only dependencies compose with call-bearing packages and diamonds;
  no zero-cost shortcut drops real invocation/source bounds. Unused references
  retain owner conflict checks without generating unnecessary code.
- Finality/native-write failures and compiling semantic mutants have independent
  oracles. Generated examples are inspected, ignored and preserved outside Docker.
- Fresh independent review, full isolated Bazel/lint proof and evidence precede a
  scoped push. Complete parent03 only after producer and consumer criteria pass;
  compiler/bundle children04/05 and legacy retirement remain pending.

## Implementation

Opaque JavaImportedValue witnesses retain their exact producer constant and
consumer scope. A distinct JavaValueRef::Dependency path feeds shared dependency
catalogues, expression typing, resolved-name rechecks and the existing renderer.
Both value and function registrations use one bounded scope. Frozen authority
retains every registered direct owner and its transitive proofs, even when unused;
only actual references are emitted. Foreign fields cannot become owned fields or
assignment targets.

Eleven added tests cover constants-only/mixed and used/unused consumers, exact
scopes and types, coupled catalogue/path tampering, shared-authority diamonds,
hidden unused-owner conflicts, source-byte reservations, combined binding/name/
owner limits, assignment rejection and independent native truth.

Java21 separately compiles each generated producer, generated consumer and
independent caller. All eight scalar values are checked, including both bools,
signed32/64 limits, 9007199254740993 and 62. All 32 attempted native writes fail.
Sixteen compiling wrong-reference mutants fail the independent oracle; restored
sources are recompiled and rerun before export. A three-crate diamond retains
shared constant authority and measures a real two-level invocation chain.

No runtime, boxing, raw syntax fallback or external dependency was added. No
legacy API or test was removed or disabled.

## Verification evidence

Implementation tree f313d104a8851395fe6f4b6ee8c9e664a2d64c77 passed the seven-target
preflight: Java unit/native and compile-negative suites, Rust Clippy, rustfmt,
buildifier, documentation and source policy. Invocation
0f7e2619-f135-4949-a86a-84aae9fec7aa; 140.697 seconds; 330 Java cases.

The same isolated tree passed all 624 full-gate tests across 816 targets in
208.639 seconds, invocation 0f607f6c-0173-464f-98f9-6ac98efe8abc. Release integration,
all rustc-frontend experiments, existing C/shared typed tests and lint gates
remained enabled, with test caching retained.

A retained-sandbox rerun passed all 330 Java cases in 117.613 seconds, invocation
8b1f63a8-bd86-4b44-ae0e-6fdd1cfa0da2. Actual generated sources and independent callers
were exported and inspected at ignored host directory
generated/examples/java-constant-consumers-f313d10: ConstantsOnlyUnused,
ConstantsOnlyRead, MixedUnused, MixedRead and Diamond. No generated code or
compiled classes are staged.

The independent Sol Extra High review found one contradictory old normative
paragraph about retaining unused owners. The finding was accepted and fixed:
the contract now explicitly preserves all registered authority while emitting
only used references. No implementation defect was reported. A fresh Sol Extra High recheck independently audited the full change and found
no actionable core defects; it confirmed the corrected retention contract.
Compiler lowering and bundle integration remain explicit later work.

The corrected-documentation tree 354166f68c774b6c3d9a6ba65fa82fe40c07dfe9 passed
all 624 tests again in 23.744 seconds, invocation
2bb42c32-f99d-427b-9f72-cf89416e716b. Completion bookkeeping is gated once more
before the scoped commit/push.
