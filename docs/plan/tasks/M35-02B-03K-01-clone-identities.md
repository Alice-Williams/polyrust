# M35-02B-03K-01 — Standard scalar Box clone identities

- Status: complete
- Parent: [M35-02B-03K](M35-02B-03K-owned-clone.md)
- Depends on: M35-02B-03J

## Contract

First observe method autoref and explicit UFCS shared borrowing with the pinned
Rust compiler. Authenticate the Clone method and its resolved concrete standard
Box implementation, full Box<i32, Global> type and shared receiver. Do not select
an operation from a method's spelling or a matching function signature alone.
Private query-only evidence retains canonical source nodes and compiler types.

## Implementation

Use dedicated observation fixtures and Bazel targets before adding admission.
Then add a focused owned-source module and an optional consuming builder slot
using existing Capability, Mapping and Supports contracts. Keep legacy bindings
valid. Separate method autoref and explicit borrow source forms with an enum.

## Definition of done and tests

- Pinned compiler assertions distinguish standard clone, reference clone,
  same-named custom functions/methods, wrong payloads and clone_from.
- Positive tests compare actual resolved DefIds, instantiated signatures,
  full types, canonical owners/expressions and receiver adjustments.
- Missing/duplicate/wrong-capability/context/output/input registrations and
  private-field fabrication fail with exact compiler diagnostics.
- Both builder registration orders execute the mapping; missing clone support
  does not prevent existing Box-only consumers from compiling.
- Invalid Rust is rejected before success/output; fixture mutations demonstrate
  the oracle is sensitive to operation identity, not only successful compilation.
- Fresh broad read-only review and all historical/native/lint gates pass on the
  exact staged tree. Commit and push separately from whole-body correspondence.

## Implementation evidence

- The pinned probe establishes Clone trait/method language items, concrete
  Instance resolution, normalized implementation self type and two distinct
  receiver-borrow representations. Three-target observation/documentation gate
  `572fda33-2cc4-4cdf-93b9-982d85a69a51` passed in 14.939 seconds.
- `owned_source/cloning.rs` supplies a private BoxCloneInput, closed CloneForm
  and CloneError enums and read-only compiler-identity projections. The optional
  fifth builder slot supplies an executable Mapping through Supports; the full
  scalar Box type helper is shared with local-call admission without widening it.
- Expanded fixture gate `34e68973-994b-45fa-8982-22b717ad0dc8` passed all
  four runtime/format targets in 16.316 seconds. The complete 17-function
  inventory accepts four clone operations, executes eight clone bindings (both
  orders) and seven historical constructor bindings. It rejects 22 other calls.
- The runtime harness also rejects three invalid Rust programs and five
  valid-source oracle mutations, including replacement allocation and reference
  cloning. Fresh review and the full isolated historical/compile-negative/lint
  gate remain required before closure.
- Gate `5a2efad8-1f48-42e1-8022-c1d06ed78946` passed all twelve focused
  targets in 17.429 seconds: clone runtime/format, all eight exact compile-negative
  contracts, historical local-call registration and documentation. No lint or
  diagnostic expectation was relaxed.

## Review repair

Initial tree `f9c0ab7df39a74cbdf574affbe171f70155f3d44` passed isolated full
gate `cb321010-f937-459c-9646-ad8d35b66213`: 481/481 tests, 613 targets,
58.309 seconds, after verifying all 2,293 Git blobs and modes. Independent Sol
Extra High review `box_clone_identity_review` found one test/spec coverage gap,
also identified during the main-agent audit: the probe printed borrow staging
and counted drops without asserting exact places/order, and did not compare the
reference-clone implementation with the Box-clone implementation.

We agree. Production admission already checked the exact operation, but the
specification promises pinned observations too. Dedicated `adjustments.rs` and
`loans.rs` probe modules now assert HIR adjustment targets, actual MIR shared
borrow/reborrow edges, distinct clone/original destinations and ordered cleanup
places along normal CFG edges. The reference case authenticates its normalized
impl self type and distinct concrete method identity. No production admission
rule changes. Gate `a9688e69-fd97-41b3-8105-09aa845bf5e3` passed both probe
targets in 13.811 seconds; source mutation controls additionally exercise the
borrow form, replacement allocation, moved drop place and reference impl oracle.

## Closure evidence

The strengthened observation/capability/documentation gate
`50229660-2121-4e30-a5d1-be0eacf6ccea` passed all three targets in 14.676
seconds, including all four new valid-source observation mutation controls.
Repaired tree `7490768b8277dc8c9eda87518a1849df2cde4e5c` then passed full
isolated gate `93c516e6-d922-43f6-98ba-1f491b578900`: 481/481 tests across
613 targets in 25.668 seconds, with all 2,295 Git blobs/modes verified.
Fresh independent Sol Extra High reviewer `box_clone_identity_recheck` found no
core correctness, contract, privacy, compatibility or coverage issues.

This closes operation identity and executable registration only. K-02 still
must certify complete clone-body correspondence, and M35-02C/D must supply
typed C mapping and native cleanup/failure proof. Closure documentation receives
a final full isolated gate before commit; its exact tree and invocation are
recorded in the checkpoint commit.
