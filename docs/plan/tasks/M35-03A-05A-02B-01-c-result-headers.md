# M35-03A-05A-02B-01 — Owned C scalar-result headers and ABI

- Status: complete
- Parent: [public results](M35-03A-05A-02B-c-public-results.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-scalar-results.md)

## Contract

Permit the exact registered Bool/I32 struct in the owning public header before
its external function prototypes. Only such header-owned instances may appear
in public signatures; single-source public record signatures remain rejected.
Match type/member graph visibility to their defining public header. Internal
functions can use the package's public type without redefining it. Preserve
complete initialization, by-value calls, closed effects and resource checks.

Do not issue a CDependencyApi for any aggregate-bearing header yet, including
one whose selected scalar functions do not reference the struct. No unchecked
import or name reconstruction is introduced. C source consumers may include and
use the actual generated public header directly.

## Definition of done and tests

Certify and render the real header/source pair, compile a header-only translation
unit, and separately compile native consumers/producers with GCC14 and Zig at
O0/O2 plus UBSan. Exercise both tags, successful zero, I32 extremes, copies,
arguments, returns and branch observations. Mix compiler-produced objects.
Reject noncanonical public shapes, private/header ownership mismatches, late
declarations, incomplete values and aggregate dependency-API publication.
Mutation controls must compile and expose tag/payload faults. Verify bounds
and dynamic imports; preserve existing outputs/WIP. Full Linux release/lint and
fresh GPT-6-SOL review precede a separate commit/push. No source admission.

Public members use the existing typed `BindingScope::Type`, recovered from the
original projection's exact member/aggregate registration. A language hook
returns a registered owning type, not a scope string. The shared allocator
rejects absent owners and certification independently reconstructs the same
allocation. Ordinary package symbols remain separate. Source-private allocation
is unchanged for byte-for-byte compatibility. Tests cover repeated fields in
two public result structs, a field/function spelling collision, native consumers,
an absent owning type and post-link scope tampering.

## Review evidence

GPT-6-SOL at extra-high effort reviewed implementation tree
`186f154b63912689296c1b70668e170b7f65e98a` against the prior private-transport
checkpoint. Its final amended conclusion was clean: no actionable production
defect or required test gap.

The reviewer initially proposed a missing end-to-end noncanonical public-shape
test. This finding was not accepted: `shared_package_projection` and
`shared_dependency_api` already send an authentic one-I32-field header with
scalar-only public functions through `project_c_package` and require the exact
layout diagnostic. Removing the public-layout guard would fail their
`unwrap_err` checks; the separate result-signature check cannot mask that
regression. The reviewer inspected both cases and explicitly retracted the
finding. Broader shape enumeration remains in the registered-layout tests.

Native-harness corrections retain strict checking: public fixture keys use
production-compatible synthesized origins, and a GCC final link includes UBSan
for Zig-produced debug objects. The standalone-header and native-consumer
fixture helper is explicitly test-only; no production import-policy exception
was added. Final full-gate evidence is required before marking this task complete.

A second broad GPT-6-SOL review of `c25af6376c00bb40e94ac99dbaf52dda560dc429`
found a genuine naming defect despite all 1,039 full release/lint targets passing
on that snapshot: exported members were allocated as package-wide ordinary
names. Two valid aggregate owners could not reuse a field spelling. This finding
was accepted and the typed owner-scope change above addresses it. The reviewer
found no separate core defect. New regression/native tests and a fresh review
must pass before completion; the previous green suite is not evidence for the fix.

The fresh GPT-6-SOL extra-high review of `dd67a07fb6b173cab5ed459ab1b04e78920eb68d`
plus follow-up `9a59e1e7fe147388575eaef4a262103f48d64635` is clean: no actionable
production defect or required contract-test gap. It examined header ownership,
typed scopes and post-link reconstruction, graph references, dependency API
exclusion and native ABI/fault/resource checks. The follow-up fixes an exhaustive
test-fixture enum match and strengthens the scope-forgery test to mutate only
scope metadata. Checked Rust Result admission and certified nominal imports
remain explicitly outside this checkpoint.

## Completion evidence

The implementation snapshot `9a59e1e7fe147388575eaef4a262103f48d64635` passed all
1,039 Linux Bazel release/lint targets (166 executed, the rest valid cache hits).
This includes 154 shared-codegen cases, 868 C cases plus nine separately targeted
long suites, and 436 Java cases plus its separately targeted suites. The final
documentation snapshot is gated again before committing the exact tested tree.

The owned result ABI passes ten GCC14/Zig producer/consumer/optimization rounds,
including mixed objects and UBSan, with 786,504 observations per correct round.
Twenty independently compiled tag/payload mutations are detected. Standalone
double-inclusion headers, frame/linkage bounds, complete copies and returns,
canonical layout rejection and aggregate dependency-publication rejection pass.
Repeated field names across two public types and a field/function name collision
compile and run under both compilers at O0/O2 with UBSan. Generic tests reject
absent owners and scope-only post-link tampering.

All 530 preceding generated-file hashes and four newer character-constant
bundles are unchanged; all 45 unrelated WIP-file hashes remain unchanged.
No copied runtime, third-party dependency or source capability is introduced.
Certified nominal imports remain 02B-02, so the parent public-result task stays
in progress.
