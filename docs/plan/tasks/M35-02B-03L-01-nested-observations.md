# M35-02B-03L-01 — Pinned nested-record observations

- Status: complete
- Parent: [M35-02B-03L](M35-02B-03L-nested-owned-records.md)
- Depends on: M35-02B-03K-02

## Contract

Observe the pinned compiler after successful analysis for a two-level record:
Inner owns two Box<i32> fields; Outer owns Inner and a third Box<i32>. Three
distinct scalar parameters construct the owners before the two aggregate
initializers. Exercise nested leaf extraction and whole-inner transfer.
This is test-only observation, not a public checked nested-input API.

## Definition of done and tests

- Exact nonempty source inventory includes first/second nested leaf, reversed
  source initializer order, multiple leaf moves and whole-inner moves.
- Assert actual nominal identities, full types, aggregate operands/staging,
  nested Place projections, final read owner and ordered Drop/Return locations.
  Debug printing alone is not evidence and cannot confer correspondence.
- Valid source mutations alter field/read/order/transfer structure and fail
  independent expected observations. Invalid Rust and empty inventory controls
  fail before the success marker.
- Pin PostCleanup and abort mode. No fabricated generic IDs, source-span matching
  or source-file text parsing to identify owners.
- Focused observation/format plus full isolated Linux/Bazel historical/native/
  lint gate, fresh independent review, documented evidence and commit/push.

## Pinned evidence and implementation

Initial inspection gate `4674f18b-73ff-49ca-bc02-e5d3b364d31d` passed both
observation and format targets in 14.322 seconds. PostCleanup stages Inner's
Box fields and Outer's Inner/Box operands through distinct move temporaries in
source initializer order, then supplies aggregate operands in declaration order.
Nested extraction has two typed Field projections. Whole-inner extraction moves
one Inner place and leaves only the spare Box under the old Outer binding.

The initial debug observations are now replaced by executable typed assertions
in focused `observation.rs` and test-only `trace.rs`. All normal blocks are
visited; loops, unwind and unexpected control forms fail. The tests establish
exact three constructor calls, two nominal aggregates, expected nested/whole
moves, scalar read and three ordered leaf drops before Return. They do not
publish a nested source certificate or claim complete generic-body admission.

Focused gate `695d0597-f2be-48ca-9b63-2d7e5c8e0905` passed both targets in
10.656 seconds. Five positive source bodies, three invalid Rust controls and
eight valid-source observation mutations are nonempty and independently checked.
Full isolated verification and fresh broad review remain pending.

## Review repairs

Initial exact tree `5a74c9c63d3e58052da4b13adc74be90edf4e02d` passed full
gate `edb05603-c573-4a53-b6df-e28737e54991`: 488/488 tests, 622 targets,
29.015 seconds; 2,321 exact Git blobs/modes were verified before extraction.

Self-audit found that the observer should independently assert the exact
box_new compiler diagnostic identity/arguments and local nongeneric record
identity. The independent Sol Extra High reviewer confirmed those gaps and
found the missing named-field-versus-tuple distinction. All findings were
accepted: the repaired observer requires exact constructor identity, empty
record arguments/no generic parameters and no tuple/unit constructor metadata.
Three additional valid-source controls replace Box::new with Box::from,
introduce Inner<T>, and convert Outer into a tuple struct. The tuple control
must fail specifically at the named-field assertion, not an unrelated change.

Focused gate `7641d629-ecda-4fa9-bb8d-d32cfa201aca` passed both targets in
14.559 seconds with three invalid Rust and eleven valid-source controls. The
repaired full isolated gate and fresh independent recheck remain required.

The repaired tree `225ddc99dbf388058003631723e7594939274bfe` passed full
gate `d00f5c70-f077-49df-b1fc-d43f585e5dfa`: 488/488 tests, 622 targets,
20.540 seconds, with 2,323 verified blobs/modes. Its fresh reviewer identified
three further fixture-contract gaps; all were accepted and repaired:

- Keep one actual compiler Ty/Adt pair across the five bodies. A same-shaped
  alternate nominal pair in just one function now fails that exact assertion.
- Replace the statement-removing whole-inner mutant with a same-count shared
  borrow plus direct nested extraction. It must fail the actual unique Move
  assertion rather than the earlier statement inventory.
- Require immutable simple binding modes for parameters and lets, with separate
  mutable-parameter/binding source controls that fail their specific assertions.

Gate `81107d4d-03b7-47ac-828a-e595a6b41bc0` passed runtime/format in
14.709 seconds: three invalid Rust and fourteen valid-source controls. A new
independent review and full isolated gate remain required before closure.

## Closure

- Exact repaired tree `0927a0ef13ab8f1707f40e72fcfa6d4964a15315` passed
  full gate `085bf2de-1ad3-4ed5-a01f-79dc1b2556f7`: 488/488 tests,
  622 targets, 19.075 seconds. All 2,323 Git blobs/modes were verified; cached
  test results remain enabled. All twenty preserved M34 files match baseline.
- A new independent Sol Extra High review found no remaining core defects.
  It confirmed all six repaired fixture-contract gaps and the scoped aggregate,
  move, read and cleanup observations and their non-vacuous controls.
- The final documentation-inclusive tree and gate are recorded in the commit.
  L-02/L-03 and 03M remain required; this checkpoint exposes no production nested
  capability, whole-body certificate or C/Java heap output.
