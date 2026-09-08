# M34A-11-00R — Close the C design-review contract inventory

- Status: in-progress
- Depends on: M34A-11-01R

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
