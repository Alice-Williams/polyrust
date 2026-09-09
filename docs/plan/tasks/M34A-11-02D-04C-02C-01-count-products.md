# M34A-11-02D-04C-02C-01 — Typed counts and allocation products

- Status: complete
- Depends on: M34A-11-02D-04C-02B

## Goal

Connect a typed immutable count binding and requested element shape to the
original actual allocation product, without granting dynamic access yet.

## Implementation

1. Split allocation registrations out of control registrations; add a closed
   requested-shape enum and private-field immutable size count reference.
2. Add typed count/buffer builders and contextual reconstruction/lexical checks.
3. Derive count/stride candidates from actual byte operands and immutable
   initializer chains in the existing product analysis. Retain numeric losses,
   exact call origins, checked layout and original count snapshots.
4. Admit private matched element-extent evidence only from a compatible actual
   request. Until checkpoint 02, reject dynamic restoration/access rather than
   treating it as a fixed object or accepting descriptor-only evidence.

## Definition of done and tests

- Count wrappers reject wrong type, mutability, registry/owner, missing actual
  declaration and lexical misuse; compile-fail controls reject arbitrary locals
  and direct private-field construction.
- Direct/reversed/immutable-materialized count products have matching positive
  cases; wrong count/stride, mutable/reassigned byte source, zero, overflow,
  late guard, cloned call and wrong graph/point cases reject.
- Fixed heap and standalone numeric/index behavior remain covered. No dynamic
  pointer capacity or initialized storage is invented by this checkpoint.
- Full cached focused/tracked/release/lint/conformance gates and uncapped
  independent review pass; document exact evidence, commit and push.

## Implementation checkpoint

Allocation registrations and immutable count bindings now have separate focused
modules. Registry reconstruction authenticates the full requested shape; lexical
checking requires the actual count declaration to dominate restoration.

A private product module traverses the actual allocation operand and immutable
size/ABI-equivalent initializer chains. It retains exact count/stride candidates
and original numeric bounds; joins intersect candidate identities and widen
their bounds. Measured element layout and original byte bounds must agree.
Mutable byte aliases and unsupported algebra yield no element evidence.

The production allocation-request path invokes this matching, but even a valid
match still returns the explicit unimplemented-dynamic-storage diagnostic.
Tests distinguish that boundary from a wrong-count/stride/byte request. No new
rendering path, dynamic access, initialized prefix or owner contract is admitted.
Full gates and independent review are required before this task can close.

## Completion evidence (2026-09-09)

- Focused invocation 27d0bf4f-0443-453d-8455-a7df0ed6253e passed 457 C
  units plus typed compile-fail, Clippy, documentation and Buildifier.
- Tracked invocation 965fc8fd-6537-41c1-a47e-594f59b2e0d5 passed 445 rules /
  320 tests. Release invocation 8f21e1cb-1829-4936-b900-0fecf2f4caab passed
  all 257 tests.
- Conformance invocation 63bcda90-024c-414d-b99a-dee43ba5ca7e passed 50
  cases plus one portable test: evaluator and eight targets agree, with
  byte-identical repeated manifests. C still uses legacy emission.
- An independent Sol Extra High reviewer directly inspected every changed/new
  scoped production, test, spec and plan file plus required prerequisites. Its
  uncapped consolidated verdict was PASS: no substantive defect or required
  proof gap. Coverage included private binding/shape reconstruction, actual
  declaration dominance, product/loss snapshots, exact sites, joins, layout,
  loop/scope interactions, Bazel inclusion and the fail-closed boundary.
- No review finding was dismissed or left unfixed. The production/test/spec
  tree stayed frozen through review and the full gates. Only closure documents
  changed afterward; documentation and Buildifier are rerun before commit.
- Dynamic access and count-activation relationships remain checkpoint 02;
  initialized prefixes remain 03. Parent 02C/02 and C render readiness stay open.
