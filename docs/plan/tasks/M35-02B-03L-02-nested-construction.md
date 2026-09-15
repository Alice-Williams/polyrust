# M35-02B-03L-02 — Typed nested-record construction capability

- Status: complete
- Parent: [M35-02B-03L](M35-02B-03L-nested-owned-records.md)
- Depends on: M35-02B-03L-01
- Specification: [nested construction](../../specification/typed-generation/languages/c/rust-nested-construction.md)

## Contract

Authenticate one canonical complete nested-record construction operation after
successful compiler analysis. The layout uses actual local nongeneric AdtDef,
DefId, GenericArgsRef, Ty and FieldIdx. A closed field-kind enum distinguishes
the exact standard Box<i32, Global> leaf from a nested record layout. This is
operation identity, not ownership-flow or whole-body admission.

## Definition of done and tests

- Canonical owner/node checks, complete field membership, compiler-resolved
  initializer identity/type and absence of adjustments; no spelling authority.
- Finite default-representation named-field records, no custom Drop or cycles,
  no generic arguments/parameters, tuple/unit/union/enum/scalar/reference fields.
- Pin separate depth and aggregate-field budgets in the specification before
  implementing traversal. Repeated nominal types in distinct sibling fields
  are not themselves a recursive cycle; every field occurrence counts.
- Root construction must contain a nested record; existing flat input remains
  its own capability. Expose declaration order separately from initializer
  evaluation order and retain full leaf types including allocator arguments.
- Add one optional consuming builder slot with an executable Mapping signature;
  missing, duplicate, wrong capability/context/output/input and private evidence
  fail exact compile-negative tests. Historical registrations remain compatible.
- Positive nesting/repeated nominal/reversed initializer cases and negative
  types/shape/budgets/custom allocator/forged node controls are non-vacuous.
- Full isolated gate, fresh independent review, recorded evidence and a dedicated
  commit/push. No body certificate or target heap output is enabled.

## Implementation and focused evidence

The private nested input and bounded compiler layout reader are implemented in
separate source modules. Declaration-ordered fields retain compiler DefId,
FieldIdx and full Ty; canonical initializers retain their independent source
evaluation order. The sixth consuming builder slot provides the executable
Mapping, with all five historical slots preserved in forward/reverse order.

Focused Linux/Bazel gate `aaa1dcc9-224e-4c66-8301-2254ac2409df` passed all
13 targets in 22.118 seconds, including compiler-adapter Clippy and rustfmt.
The runtime inventory asserts eight admitted and twenty-one rejected root
operations, exact depth 8/9 and expanded-field 128/129 boundaries, repeated
nominal siblings, aliases and distinct same-spelled definitions. Isolated
compiler-type controls reject a substituted Box allocator and a repeated
ancestor identity. Each admitted operation rejects a copied HIR node and wrong
owner. Four invalid-Rust controls and four valid-source oracle controls reject
before the success marker. Eleven compile-negative API tests require their
exact error code/count, including private input/layout/field/initializer data.

Full isolated verification and fresh independent review remain required.

## First full gate and review repairs

Exact tree `67c7eb35acc642c95cbfe9ec1b1c75b06eab5240` passed full isolated
gate `55721b34-23bd-453c-9a41-1cbb1266e503`: 501/501 tests, 636 targets,
75.548 seconds, 2,336 verified Git blobs/modes. The fresh Sol Extra High
review found three contract/proof gaps. All were accepted:

- Clarify that explicit repr(Rust) is semantically default representation and
  accepted; add a positive with that attribute on both root and child. The
  implementation already used compiler semantic representation metadata.
- Isolate direct reference, external locality, packing and alignment rejection
  from generic-parameter or representation-flag rejection. Add nongeneric
  static-reference and Duration fields, packed/align records, and a raw pointer.
- Replace the allocator oracle's plain non-Allocator struct with actual
  std::alloc::System. The compiler type now denotes a real alternate allocator;
  only Box's allocator argument differs from the admitted standard Global type.

The repaired inventory has nine admitted and twenty-six rejected root
operations. The allocator oracle resolves the actual non-Sized allocator bound
from standard Box's compiler predicates and requires a concrete implementation
for the substituted System Ty. A fifth valid-source control substitutes
Duration and must fail that exact trait-implementation assertion.

Focused gate `b8213700-fd41-4a59-943d-963d33087069` passed runtime/format in
14.325 seconds. Repaired full verification and a new independent review remain
required; the added precise control-failure assertion will be checked by the
full gate before closure.

## Second review repair

Repaired tree `38f08f5bd0b6fdec48363bc6b83c0385d49104bf` passed full gate
`f1c36921-004d-446d-b67f-a3d6171d7932`: 501/501 tests, 636 targets,
37.308 seconds, 2,336 verified blobs/modes. The new independent review found
one remaining authentication gap in the test oracle: an exact trait/self-type
match in the impl inventory did not explicitly establish positive polarity.

We agree. The shared oracle predicates now require both a positive Box trait
clause and a positive concrete impl. Isolated controls preserve the actual
trait/self identities while substituting a negative clause and negative or
reservation impl polarity; all must reject. Actual System evidence still
passes through the same predicates using compiler-query polarity. Focused
gate `140ae689-69d4-463c-90c1-0ca0a5faeb0a` passed runtime/format in
17.716 seconds. Full repaired verification and a fresh review remain required.

## Closure

- Exact repaired tree `22de2a9ca5a7c55998ed15691ccaa114c7f942a8` passed
  full isolated gate `103ef922-1bbf-4218-b78a-1c9734623429`: 501/501 tests,
  636 targets, 28.211 seconds. All 2,336 Git blobs/modes were verified and test
  caching remained enabled. All four review findings were accepted and fixed.
- A new independent Sol Extra High review found no remaining core defects,
  confirming the non-vacuous positive-polarity repair, surrounding operation
  identity/layout/order, six-slot registration and private API boundaries.
- Nine admitted/twenty-six rejected operations, four invalid-Rust/five
  valid-source controls, isolated allocator/ancestor/polarity controls and
  eleven exact compile-negative tests remain permanent gates.
- The final documentation-inclusive tree/gate are recorded in the commit.
  L-03 and 03M remain required; this checkpoint enables no whole-body nested
  certificate or C/Java heap output.
