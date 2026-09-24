# M35-03A-05A-02B-02 — Certified C scalar-result nominal imports

- Status: complete
- Parent: [public results](M35-03A-05A-02B-c-public-results.md)
- Depends on: [owned ABI](M35-03A-05A-02B-01-c-result-headers.md)

## Contract

Retain the original aggregate/member references, resolved names, complete
defining header and immutable certificate in an opaque target witness. Import
that witness, not a copied local declaration or caller-authored layout. Derive
the catalogue again from original package authority after linking.

The existing known-type catalogue supports CertifiedDependency origins and
FixedImport policy. First prove whether a language-owned referenced-type enum
(standard scalar versus certified aggregate) can use that path. Do not add a
parallel generic type-import framework without a demonstrated missing contract.
Member spellings must likewise come from the original certificate before
rendering, never from text matching or independent consumer name allocation.

Read-ahead confirms that `KnownFieldSpec` plus `DependencyPolicy::Member` can
carry an authenticated aggregate owner and field spelling without occupying the
ordinary import namespace. Select the owner's known-type reference structurally
as well, so member-only uses still introduce the correct defining header.
Generated public members already have typed owner scopes from 02B-01.

Synthesized target result structs need not be direct Rust type exports. Enumerate
their actual certificate-owned `CStructRef` witnesses and select them through
typed signatures/references; do not invent Rust declaration IDs or find them by
names. The compiler's canonical Result-instance ownership remains a separate
source-integration obligation. Keep standard scalar and certified nominal
known-type variants explicit; no arbitrary known-type strings are admitted.

## Implementation boundaries

1. Derive an opaque aggregate witness from the original certificate's actual
   header declaration and resolved type/member bindings. Retain its complete
   layout and certificate authority. Same spelling or layout is not identity.
2. Keep imported aggregates separate from owned registry declarations. Prefer
   retaining original foreign aggregate/member references so signatures and
   relay packages cannot accidentally manufacture a second nominal type.
   Membership and layout reads require the registered witness. Imported
   definitions remain immutable: adding members, redefining the aggregate or
   emitting its declaration in a consumer must fail. Registration failures
   must leave every registry inventory unchanged.
3. Extend the language-owned referenced-type vocabulary with standard and
   certified variants. Reconstruct its known-type catalogue from the consumer
   registry's immutable authorities. Use the existing fixed-import path and
   the C tag namespace for aggregate names; standard typedefs remain ordinary
   identifiers. Select imported types and members from structural file uses.
4. Resolve member spellings from the authenticated original type witness at
   link time, retaining typed member identity. Do not allocate consumer field
   names or look them up by strings in the renderer. Reference discovery must
   account for both member reads and aggregate initializers.
5. Reconcile entire included headers, including unused public types, across
   direct and transitive dependencies. Tag and ordinary namespaces stay
   distinct. Preserve the original header through a relay rather than copying
   a declaration or inventing an alias header. Charge closure/layout/name/frame
   costs before rendering and prove that type-only imports are not invisible.
6. Only after the above gates pass, replace the temporary aggregate-header
   publication rejection with exact certificate-derived API collection. Admit
   local/imported result signatures through the same registered shape proof.
   Keep compiler source capability admission separate from this target change.

The implementation must audit ownership-sensitive registry operations, not just
relax the central membership check: a reference valid for reading an imported
layout is not permission to mutate or declare that layout locally.

Reconcile all type names introduced by the complete dependency header, even
for scalar-only imports. Preserve C tag versus ordinary namespaces, original
owner through relays, transitive header dependencies, frame/layout bounds and
atomic registration. Reject conflicting nominal authorities rather than
treating same-shaped types as interchangeable. Source Result-instance identity
remains a separate compiler obligation.

## Definition of done and tests

Producer/relay/consumer packages certify independently, preserve exact original
types and fields, and agree natively across GCC14/Zig and optimization levels.
Type-only imports and mixed scalar/result APIs work. Cross-namespace equal names
behave correctly; same-namespace collisions, forged or missing authorities,
renamed members, header conflicts/cycles, missing transitive types, wrong
signature owners and over-budget imports reject. Mutation controls show that
metadata alone grants no authority. Full Linux release/lint, preserved existing
outputs/WIP and fresh review precede commit/push and removal of the temporary
aggregate-header dependency guard.

## Implementation and evidence

The candidate now derives original `CDependencyStruct` witnesses, separates
foreign read membership from ownership, and uses the shared known-type/field
catalogue for exact names and structural includes. Whole-header tags are
reconstructed directly from certificates, including scalar-only and type-only
dependencies. The former blanket aggregate API guard is replaced locally by
these checks. The full release/lint gate and fresh review have passed.

The corrected C unit suite (884 cases, ten separate native partitions) and
nominal-import native target pass on production/test candidate
`40b111d94e6589ada98938072b369d59050fd933`. Native proof executes 262,154 input/tag
cases in each of eight GCC14/Zig O0/O2 same/mixed compiler rounds. Same-compiler
rounds enable UBSan. Eight compiling tag/payload faults fail the actual value
oracle, not compilation or sanitizer checks. The test has its own Bazel
execution target and remains included by the partition contract/release gate.

Rejection tests cover original layout/name/member/certificate substitutions,
read-only import permissions and atomic failures, absent type witnesses,
resolved type/member tampering, whole unused tag collisions through relays,
header/output collisions, direct/transitive source cycles and distinct same-
shaped nominal references. Registration-order and unused-import output controls
pass, including the type-only cached-cost forgery and wrong authentic nominal
signature/argument tests. All 1,040 full repository release/lint targets pass;
fresh GPT-6-SOL Extra High review of that corrected candidate found no defects.
All 45 pre-existing unrelated WIP hashes remain unchanged.

## Review disposition

The first broad GPT-6-SOL Extra High review found no production correctness
defect, but identified a required missing regression: the suite must reject a
function signature when a different authentic same-shaped struct is registered,
not only when no struct is registered. Accepted; a direct atomic-registration
and wrong-argument-owner test is added, passing and independently reviewed.

The reviewer also confirmed a parent-level evidence gap: current value tests
cannot detect eager evaluation of an inactive pure call. Accepted and tracked
in [02C](M35-03A-05A-02C-c-result-branches.md), after this import checkpoint.
Neither the C transport parent nor public ABI parent will be marked complete
until that promised measured execution proof passes. This does not block the
bounded nominal import checkpoint once its own tests/review pass.

The fresh review found an API-scope issue: all registered foreign structs were
being republished, even if used only privately or unused. Accepted. Foreign API
witnesses are now selected from the exact exported function signatures; owned
public-header tags remain complete, and the original registry still retains
private dependencies for closure/resource verification. Regression cases cover
both an unused type and a body-local constructed type behind a scalar-only API.
The correction passed the unit suite and subsequent independent review; the
full release/lint gate also passes on the corrected code.

The third reviewer identified an incidental widening of local interface-witness
registration: its endpoint checks inherited imported read membership. Although
these references are not conformance proofs and the shared generation profile
does not admit interface inventory, the registration could attach a local
witness to a foreign file. Accepted as a registry ownership regression, not as
evidence of invalid generated code. Both endpoint checks now require ownership.
Tests reject either or both foreign endpoints without changing the registry,
then prove the same key/implementation remains available for owned endpoints.

The fourth fresh GPT-6-SOL Extra High review examined the complete immutable
diff, including ownership-sensitive operations, exact type publication, names,
header/dependency closure, resources and tests. No concrete defects were found.
This was a read-only source review, not a substitute for the full Linux gate.

## Completion checkpoint

The Linux dev-container run of `bazelisk --batch test //... //:release_gate`
(two jobs, the established isolated output root and lockfile mode off) passed
all 1,040 targets on production/test tree
`40b111d94e6589ada98938072b369d59050fd933`: 132 tests executed, the rest cached;
elapsed 1,479.383 seconds. This includes Rust Clippy/rustfmt, Bazel lint and the
separate native partitions. All 530 prior generated file hashes, four newer
character-constant bundles and 45 unrelated WIP hashes are unchanged.

The completion documentation is checked again on the final commit snapshot.
The parent C/public-result milestones remain in progress for 02C's measured
selected-arm proof. No Rust Result source capability, Java result transport or
legacy-runtime retirement is claimed by this bounded checkpoint.
