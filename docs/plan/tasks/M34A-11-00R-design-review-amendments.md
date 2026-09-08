# M34A-11-00R — Close the C design-review contract inventory

- Status: in-progress
- Depends on: M34A-11-01R, M34A-11-00P

## Goal

Resolve the immutable dc55311 design audit before admitting contextual C ASTs.
Specifications describe the target; passing documentation tests does not prove
that unimplemented target code exists or is correct.

## Definition of done

- Specify a deterministic callable ABI, allocator/lifecycle preconditions,
  transport/computational/value-result distinction, enum/tag representation,
  owned constant factories and the complete flat-vtable signature family.
- Enumerate all 42 capabilities and their closed inputs, plans, prerequisites,
  ownership/failure effects and structural output categories.
- Define macro reservations, observable name allocation, native system-link
  requirements and non-executable metadata ownership.
- Select the supported platform/FP environment, resource accounting/budget
  categories and exact proof targets. Probe concrete compiler budgets before
  certification is exposed in stage 04; no unmeasured limit is claimed proven.
- Make native assertions non-vacuous; enumerate historical replay; specify the
  exact compiler-negative shape and safe comment/literal preprocessing.
- Explicitly include interface fields in legacy payload-enum compatibility.
- Record every finding disposition below and review the amended contracts.

## Review disposition inventory

Review: Sol Extra High, read-only, immutable dc55311; no builds claimed.
The numbers follow the reviewer's final consolidated report.

| Finding | Root assessment and owner |
| --- | --- |
| 1: record/interface equality and list searches | Confirmed by compile controls; M34A-11-01R repairs typed and dynamic admission |
| 2: payload-enum interface nesting | Existing “all admitted type positions” already includes it; accept explicit proof-matrix clarification, not a newly missing representation feature |
| 3: Java/portable NaN expectation mismatch | Confirmed; M34A-10AB, preserving the established M27 NaN-class rule |
| 4: closed grammar inventory | Accepted; foundation commit 2e4c50b adds the normative inventory; stage 04 still owes exhaustive native evidence |
| 5: per-capability strategy inventory | Accepted contract gap; this amendment |
| 6: exact opaque ABI/pass modes | Accepted; choose one representation, not discretionary scalar-record optimization |
| 7: allocator/lifecycle conditions | Accepted; this amendment plus stage 02/06 proof |
| 8: valid-input size-overflow channel | Accepted; explicit transport capacity failure distinct from portable errors |
| 9: enum/tag ABI | Accepted; fixed-width public values and validated tags |
| 10: owned constant lifecycle | Accepted; allocator-parameterized fresh-value factories, no mutable global initialization |
| 11: preprocessor namespace | Accepted; this amendment and stage 03 collision proof |
| 12: native link dependencies | Accepted; typed system-library requirements derived from references |
| 13: F64 environment/contraction | Accepted; exact platform/caller/build preconditions and counterexamples |
| 14: resource limits | Accept missing accounting contract; measured numeric compiler budgets belong to stage 04 and cannot be asserted proven by prose |
| 15: assertion no-vacuity | Accepted; exact case/completion inventory and failing harness control |
| 16: exact negative shape | Accepted; one demonstrated incompatible aggregate initializer, isolated from production |
| 17: stale uncached gate | Accepted; shared layer corrected in M34A-11-01R to retain normal caches |
| 18: native command/target inventory | Accepted; explicit labels and pinned flags, clearly distinguishing future gates |
| 19: historical replay membership | Accepted; exact checked inventory plus per-port execution evidence |
| 20: build metadata ownership | Accepted; no generated executable BUILD/script text; external harness consumes typed requirements |
| 21: visible public-name allocation | Accepted; deterministic identity-to-symbol ABI map |
| 22: documentation preprocessing | Accepted; normalization and translation-phase adversarial cases |

These are not 22 demonstrated failures of an implemented C backend: most are
decisions/proofs missing from the design. C is still explicitly legacy/Fail.
Excluded syntax, additional platforms and representation optimizations remain
optional extensions rather than blockers.

## Tests and proof

- Documentation/link validation, Buildifier and the normal complete tracked/
  release/conformance baseline in the Linux development container.
- Machine-readable inventory checks arrive alongside their owning modules;
  planned tests and benchmark limits are not recorded as successful executions.
- Fresh read-only review of the amended immutable specifications.

## Commit gate

Record amended specifications, actual gate results and every remaining proof
obligation. Commit and push M34A-11-00R separately; C remains Fail until cutover.

## Amended contract checkpoint (2026-09-08)

callable-abi.md selects opaque records, fixed-width enum/status values, exact
callable/allocator/lifecycle families, immutable observations, fresh constant
getters and flat table callbacks. capability-inventory.md enumerates all 42
capabilities with closed inputs, output categories and proof obligations.
platform-and-proof.md fixes platform/FP assumptions, preprocessing, native
dependencies, namespace/ABI mapping, target inventories and non-vacuous native
proof. The grammar and implementation tasks now refer to these contracts;
stage 02 is split into four bounded independently tested slices.

- `d0acaa2e-8249-4421-9b56-bc3a6b920b76`: docs and Buildifier pass.
- `ee4c792a-68c1-4900-ba3d-4090981ad54d`: all 436 tracked rules and 311 tests pass.
- `7f9ce9dc-3358-409e-b2b6-125e99b4224c`: all 248 release tests pass.
- `ba2b3ac2-ba50-4393-8036-9b21f9401a5d`: evaluator/eight-target agreement
  for 50 cases and one portable test; repeated manifests agree.

These green tests validate the existing regression baseline and documentation,
not implementation of the new C design. In particular, measured numeric
resource budgets, new native targets and all 42 C mappings remain mandatory
work at their owning stages. Fresh immutable design review is still pending.

## Second immutable design review and disposition

A fresh uncapped Sol Extra High read-only review of 684a0db examined every C
layer and the Stage 02 split. It found 13 contract defects, not 13 failures of
an implemented typed backend. Root independently evaluated all findings and
accepts the following repairs; none is dismissed as an optional feature.

| Finding | Accepted repair |
| --- | --- |
| 1: enum capability overlap | Enums owns typed top-level equality/branching exclusively; C explicitly assigns legacy payload cases there too. The latter is a C choice, not a shared mandate. Recursive aggregate helpers are dependencies, not second capability mappings. |
| 2: unspecified public ABI families | public-value-abi.md fixes every factory, observer, view, tag, borrowed output, lifetime, failure and outcome signature. Factories use status plus owning slot; observers never allocate/transfer. |
| 3: typedef-hidden categories/const | Expand actual registered aliases before signature/category proof; no array decay or pointee/element qualifier loss. |
| 4: no void-call grammar | Effect Call is separate from nonvoid Value Call and is consumed by Evaluate. |
| 5: allocator pointer conversions | ObjectToVoid and AllocationRestore retain qualifier/extent/provenance; interface adapters cannot substitute. |
| 6: unrepresentable constants | KnownConstant and Enumerator values retain catalogue/registration and actual constant-expression types. |
| 7: incomplete registry inventory | 02A owns enumerators, adapters/witnesses/tables, allocation and function-owned control identities. |
| 8: implicit array decay | Reject Read(array); use checked first-element address, with a separate empty-buffer path. |
| 9: nonliteral final literal nodes | ByteArray/F64Bits are mapping inputs expanded into typed storage/effects before final AST certification. |
| 10: empty C aggregates | Require nonempty definitions; portable empty records use private bookkeeping; incomplete forward tags remain valid. |
| 11: Bool/macro/int mismatch | Native _Bool, explicit 0/1 conversions, actual int logical/comparison results followed by explicit Bool conversion. |
| 12: FP status mutation | Preserve control modes; sticky exception flags may be raised and are outside portable observation. No implied whole-fenv save/restore. |
| 13: unspecified formatter | Explicit structural canonical format/no-diff target plus existing whitespace gate; no external full C formatter claim/dependency. Native compilers remain independent syntax oracles. |

Stage 02/03/04/06 tasks name the corresponding construction, mutation, compiler
and public-consumer obligations. Numeric resource budgets remain measured work
for Stage 04. All currently implemented C output still uses the legacy path;
these specification amendments do not turn that path into a certificate.

Documentation and Buildifier pass in invocation
920c6d1e-983d-4683-86c7-28a87d14e338. The earlier full baseline remains applicable:
only Markdown changed. A fresh review of the repaired immutable contract is
required before closing M34A-11-00R.

## Third immutable review and measured ABI closure

A fresh uncapped Sol Extra High read-only review of bd096ee found nine further
contract defects. Root independently accepts each and records the repair:

| Finding | Repair and owner |
| --- | --- |
| 1: undefined local declaration | Exact automatic local reference/optional initializer, const and initialization-state rules; 02B/02C |
| 2: isnan violates closed macro policy | Raw-bit NaN classification, not a function-like macro extension |
| 3: missing sequencing verifier | Conservative full-expression call roots and call-free operands/conditions; 02D, with mapping-level source-order/conditional-prefix mutations |
| 4: allocation restore cannot initialize | Preserve Uninitialized/Prefix(n), allow initialization writes, require initialized dominance for reads and complete commit for Live |
| 5: nonexistent interface marker | Shared Layer 6 now refers to the complete Interfaces capability, not a 43rd marker |
| 6: unstated native ABI facts | Exact abi-type-model.md table plus both pinned compilers at O0/O2 and an unsigned-char rejected configuration |
| 7: invalid source cannot be certified | Negative fixtures remain repository/native-oracle-only, outside plugin manifests; no production NegativeTestSource role |
| 8: prototype is not a callable contract | Private contract identity on function/callable-member references; generated body summaries derived at02D, known contracts catalogue-owned; wrong-contract substitution tests |
| 9: empty-source move unspecified | Null addresses InvalidInput first; empty/moved source, nonempty out and self-move InvalidState unchanged |

Native probe invocation f398dd9f-042c-436b-927e-482f82105523 passes under the
hermetic Zig compiler and GCC 14.2.0 at both optimization levels; the GCC
unsigned-char negative fails at the exact intended assertion. The ABI probes
are permanent tracked/release targets, not informal host compiler discovery.
They establish the type/layout model but do not implement the resource adapter.

The bd096ee checkpoint's hosted
[run 34229263594](https://github.com/Alice-Williams/polyrust/actions/runs/34229263594)
passes. Final amended contract review is still required; registry-only 02A work
may proceed without exposing verification, rendering or Supports claims.

The combined integration worktree (including concurrent 02A registry work)
passes all 439 tracked rules / 314 tests in d9b876b0, all 251 release tests in
71a46a7b, and deterministic eight-target conformance in 6e6b26af. The first
full run found only the missing exact native-oracle source-policy exemption;
the repair includes adjacent-path negative controls. Exact full IDs and C
foundation counts are recorded in 02A. This is not evidence that C00R alone
implements the concurrent registry work or certifies any generated C AST.

## Fourth immutable review: accepted recursive-graph repair

The fresh Sol Extra High read-only review of 5f5ba1ac identified a real conflict
between supported generic recursive interface shapes and blanket helper-cycle
rejection. Root accepts it: I plus R(child: Option<I>) implementing I admits
R(None), but clone/drop/table callbacks form a legal callable cycle.

Layer 7 and the C contracts now distinguish impossible complete-layout
prerequisites from legal callable components. Finite specialization identities
and prototypes are registered before bodies. Program-specific lifecycle/table
specializations belong to Implementation, preserving the baseline Runtime-to-user
dependency prohibition. No supported-shape rejection escape remains. Stages
03/06 own exact linking and R(None)/finite Some chain native clone/drop/fault/
sanitizer controls. This is a contract repair, not implemented recursive C AST
certification. The full review and fresh repaired-commit review remain open.

Further confirmed findings and root dispositions in this same review:

| Finding | Accepted repair |
| --- | --- |
| 2: combined invalid-input precedence | One short-circuit ABI ladder for every public family, including safe extent checks before traversal and pairwise combined-failure/output-preservation controls. Initially nonempty rejected outputs remain unchanged, not magically empty. |
| 3: missing switch identity | Switch carries its exact registration; a single structural occurrence, actual innermost Break target and innermost-loop Continue target are verified. |
| 4: no legal pointer-null guard | Closed internal PointerTest nullness variants and SameSlot for exact owning-slot self-move guards; ordinary binary/portable equality cannot compare live handle identity. |
| 5: missing known-object grammar | Distinct closed KnownObject identities for borrowed opaque FILE and complete MaxAlign; no generated-name substitution. Clarify 02B grammar versus 03 complete metadata/linking ownership. |
| 6: recursive runtime stack exhaustion | Depth-independent iterative lifecycle and comparison engines, allocation-free drop/rollback, checked temporary work and node counts; explicit transport effects and deep/fault/sanitizer evidence in runtime-traversal.md and Stage 06. |
| 7: missing block scope identity | Block carries its exact registered scope; function roots, actual child/parent ownership and one-to-one structural occurrence are verified, with sibling/parent/deletion/shadow matrices. |
| 8: resource check after certificate construction | Confirmed in shared source, affecting Java too. Separate [00P repair](M34A-11-00P-resource-certificate-boundary.md) checks syntax then resources before creating RenderReadyPackage; direct safe API and stage-order regressions required. |
| 9: public allocator aggregate omitted | Clarify that there are two view structs plus the separate complete three-member allocator protocol aggregate; exact public-header construction and catalogue proof at 03. |
| 10: finite native stack exhaustion | Explicit entry/callback preconditions and conservative linked native-call-path/frame admission in call-stack-resources.md; measured compiler/optimizer/sanitizer rules and boundary/one-over proof required at 04. Iterative value traversal remains a separate obligation. |
| 11: nested allocator substitution | One validated effective invocation descriptor must flow through all direct/interface/runtime/work allocations; source provenance governs only existing storage. Exact context/default substitutions and nested fault controls at 02D/06. |

Root independently checked the self-move requirement while evaluating finding
4: nullness alone would not implement the existing ABI, hence the narrow
SameSlot companion rather than arbitrary pointer equality. An initial reviewer
question about ordinary recursive user-call summaries was withdrawn after
checking v0's explicit recursion rejection; only synthesized lifecycle/helper
components are in scope. These are explicit dispositions, not extra features.

The completed round-4 report contains exactly these 11 findings, all accepted.
Finding 6 also changes the exact private lifecycle slot prototypes to take
distinct clone/drop work-engine pointers; unchanged public callbacks cannot
silently recreate recursion or rely on TLS. Engine identity, queued destination
lifetime and outer-driver commit are explicit proof obligations. The new stack
policy numbers are selected limits awaiting Stage 04 measurement, not proven
native bounds. A fresh repaired-commit review remains required before closure.

The amended documents pass documentation validation in
ef0f2da2-9ced-45ea-b34f-7ba132621e6f. The combined worktree also passed all
439 tracked rules / 314 tests (7bb71a7b), 251 release tests (c2c52272), and
deterministic eight-target conformance (10a6446a); the full IDs and shared
resource-code changes are recorded separately in M34A-11-00P. These baseline
results do not implement the new C contracts or measure the planned stack limits.

## Fifth immutable review: shared contract alignment and exact loops

Fresh Sol Extra High read-only review of 633b22ce7415d83d8d2ac324fc44504d90d76ff3
covered the complete C document/task inventory and relevant shared Layers 0–9.
It found three core contract defects; root independently accepts all three:

| Finding | Repair |
| --- | --- |
| 1: Layer 1 retained the old resource order | Align its prose, phase diagram and proof matrix with repaired shared certification. Root's adjacent-text audit also updates Java module ownership and future language tasks, and explicitly annotates the superseded historical 10AA checkpoint. |
| 2: Layer 5 still rejected every helper cycle | Distinguish impossible layout/definition prerequisites from legal finite callable components, preserving runtime direction rules and exact positive/negative closure tests. |
| 3: incomplete BoundedLoop payload | counted-loops.md selects one exact Size counted-while form with explicit condition/body and counter/bound/Step::One references. Real declarations and assignments establish initialization and one update per backedge; renderer adds nothing. 02B/02C/02D/04 own construction, control, range/progress and native proof respectively. |

The root also identified the missing loop payload during next-slice preparation.
The reviewer confirmed the fixed visit-counter form supports the admitted C
algorithms, including reverse/variable-width indexing and iterative work engines.
Actual step occurrences are rederived from all counter writes and verifier-owned
structural program points, not trusted caller IDs. This is a contract repair,
not a newly implemented or already certified loop node.

The review found no further substantiated core contradiction and ran no builds.
These amendments still require documentation validation and a fresh immutable
repair review before 00R is complete; existing full baseline proof is unchanged
because this checkpoint modifies documentation only.
