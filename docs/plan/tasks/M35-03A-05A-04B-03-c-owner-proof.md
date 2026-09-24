# M35-03A-05A-04B-03 — C canonical-owner native proof

- Status: complete
- Parent: [C type owner](M35-03A-05A-04B-c-type-owner.md)
- Depends on: [typed imports](M35-03A-05A-04B-02-c-owner-imports.md)
- Specification: [canonical owners](../../specification/typed-generation/rust-canonical-type-owners.md#c17-specification)

## Definition of done and tests

- Two producers use the same original type-only certificate. Compile separate C
  objects and a cross-producer consumer with strict GCC and Zig; exercise both
  variants, zero, signed extrema, payload forwarding and original member identity.
- Compare an independent expected-value oracle; include compiling wrong-tag,
  wrong-payload and duplicate-evaluation controls where executable composition
  occurs. No target text mutation counts as a typed generation success.
- Permuted registration and unrelated consumers preserve original package bytes.
- Exact/one-over owner, descriptor, identifier, source and closure limits fail
  at the expected boundary; no shadowed owner or foreign redeclaration passes.
- Export actual generated examples without committing generated files.
- Preserve recorded old outputs/WIP, pass full Linux Bazel release/lint and
  independent GPT-6-SOL review, then scoped commit/push.

This completes the C target owner foundation only. Graph publication and Rust HIR
Result admission remain 04D and 04E; no legacy-removal approval is implied.

## Concrete proof sequence

Start only after 04B-02's final release/lint gate, review and scoped commit.
Use its public certificate-derived dependency API and original type/member
witnesses; do not bypass the source-only compiler graph/manifest rejection.

1. Keep the canonical type fixture separate from executable source producers.
   Generate two independently certified source packages using the same original
   canonical certificate. Exercise construction, forwarding and selected-member
   reads through their real public signatures and an additional consumer.
2. Cover all six version-2 error-kind codes (0 through 5), not just an empty
   error or a single Err representative. Exercise successful zero, negative and
   positive values and signed extrema. The oracle independently states expected
   tag and active payload; it must not derive expected values by executing the
   same lowering or renderer under test.
3. Compile the generated owner and each producer/consumer into separate objects
   with strict GCC and Zig at O0/O2, then link a test-only foreign driver.
   Keep malformed native controls separate from accepted typed generation.
   Assert that compiling wrong-tag/payload controls fail the value oracle and
   reordered/duplicated effect controls fail an independent trace oracle.
4. Prove original header/member identity across both producers, repeated
   includes, consumer-registration permutations and unrelated instances.
   No producer emits another definition of the original record. Preserve all
   recorded preexisting generated outputs.
5. Map each boundary assertion to its actual policy: owner/edge budgets,
   fixed descriptor shape, generated identifier/header guard limits, and
   measured source/output bounds. Reuse existing exact/one-over tests where
   they test the same unchanged policy; add canonical-owner cases where they
   do not. Do not invent a new arbitrary limit just for the fixture.
6. Export the actual rendered examples to an ignored local output directory,
   record native results and negative controls, then complete the full gate and
   independent review before marking this checkpoint complete.

The target fixture's synthetic source identities are not rustc evidence.
These tests prove transport of the admitted target states, not arbitrary Rust
Result programs or source-level inspection of private standard-library fields.
Foreign C bytes are not implicitly authenticated Rust values: this checkpoint
does not open Rust FFI admission or claim that every I32 error field is a valid
error-kind code.

## Implemented proof and boundary map

The dedicated `//crates/backend-c:c_canonical_native_test` creates one original
type owner, two independently certified producers and two independently certified
consumers. Consumer 112 constructs with A and forwards through B; consumer 113
constructs with B and forwards through A. Both preserve the original record and
member witnesses. Independent foreign wrappers assert call traces 12 and 21;
the expected values and six error codes are written independently of lowering.

Each executable checks 8,203 states: six error codes, successful -4,096 through
4,096 inclusive, and four signed extrema/adjacent values. GCC/GCC, GCC/Zig,
Zig/GCC and Zig/Zig object combinations at O0/O2 give eight positive executions,
65,624 state checks and both generated call directions in every check.
Strict C17 warnings are errors; undefined-behavior sanitizers must remain silent.
Six typed, certified, separately compiled faulty alternatives per combination
give 48 negative executions: swapped tag, zeroed payload, and duplicated or
reordered calls in each consumer direction. Exact value/trace exit codes,
empty stderr and no success output distinguish detection from a crash,
compiler failure or sanitizer rejection.

Import-order and unrelated-owner permutations preserve both consumer directions'
bytes and the original owner's bytes. The owner, both producers and both
consumers export ten actual generated files from the Bazel test output under
`canonical-example`; host copies live in ignored
`generated/m35-canonical-c-owner-proof`. No generated files are committed.
All four canonical C test targets are also explicit members of
`//:release_gate`, in addition to the existing full `//...` CI invocation.

| Policy | Proof and scope |
| --- | --- |
| Original owner closure | New `c_canonical_owner_boundary_test` certifies an actual source consumer with 1,024 distinct same-core canonical owners; 1,025 rejects with the certificate-budget diagnostic. Unused imports remain charged and the source frame remains nonzero. |
| Traversal edge budget | Existing `resources/dependencies.rs::dependency_traversal_budgets_are_checked_at_the_boundary` checks 100,000/100,001 edges, 1,024/1,025 certificates and arithmetic overflow. This is a budget-unit proof, not a new 100,001-edge generated graph. |
| Fixed owner descriptor | `test/canonical/main.rs` rejects malformed inventories, extra/reversed members, wrong file/owner roles, foreign handles and source/type-profile mixing. `test/canonical/native.rs` separately compiles maximum-width keys and repeated headers. |
| Identifier/header limits | Existing `shared_capacity_policy.rs` exercises 256/257-byte identifiers; `shared_file_imports.rs::guard_identifier_budget_is_checked_after_injective_encoding` tests adjacent representable encoded guards (255 accepted, 257 rejected). Fixed canonical hex names remain within those policies even at maximum key width. |
| Source and frame bounds | Existing `shared_capacity_policy.rs::aggregate_capacity_boundaries_reject_exact_one_over` tests the unchanged measured node/source/frame policies. `shared_canonical_measurements.rs` checks actual canonical zero executable contribution and foreign-inventory rejection. The fixed two-member canonical profile cannot grow to the general source-byte limit without first violating its descriptor; no oversized canonical fixture or new limit is claimed. |
| Conflicts and original authority | `c_canonical_dependencies_test` covers replacement certificates, same-core instances, same-crate source conflicts, unused/diamond closure retention, header collisions and foreign declarations. |

The first blind GPT-6-SOL review found that reverse transport was exercised only
by the foreign driver. The generated reverse consumer and mirrored controls
above close that finding. Its follow-up review of tree
`f35a8abb529402e3df1732a1c1efa0a18e8fa428` found no remaining actionable issues.
That tree passes the five focused native/boundary/lint targets (invocation
`5dcbf933-afc2-48b6-8e7f-22d0c835da06`, 44.492 seconds). A fresh GPT-6-SOL
extra-high blind review of the same tree also found no actionable defect.
Both reviews are static evidence, separate from the executed tests.

Full Linux Bazel `test //... //:release_gate` passes all 1,056 test targets on
that code tree, including Rustfmt, Clippy, Buildifier and generated-language
compilation/execution (invocation `2081b38b-bdfc-45b0-8621-5eeac40f3d62`,
65.390 seconds; four tests executed and unchanged results reused).
All 530 recorded prior generated outputs are byte-identical and all 45 protected
WIP hashes are unchanged. The ten exported host artifacts match the actual Bazel
outputs by SHA-256. This closes 04B's target-owner foundation only; compiler
Result source admission remains closed.
