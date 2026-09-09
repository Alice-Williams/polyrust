# M34A-11-02D-04C-01 — Actual allocation origins and raw lifecycle

- Status: complete
- Depends on: M34A-11-02D-04B

## Goal

Admit useful checked default-allocation/null/release flow without inventing
typed storage, custom callback effects or ownership-transfer certificates.

## Definition of done

- Borrow the numeric observation for the exact actual malloc call, retaining
  its authenticated function, graph point, requested byte extent and scope.
  Positive nonwrapping bytes are mandatory; a cloned call or registration is
  not evidence. Standard allocator alignment comes from the measured ABI.
- Track possible-null allocation, proved null/live success, release and
  conservative joins separately. Pointer tests refine the actual allocation
  origin, including copies, rather than a variable's spelling.
- A default free consumes only null or its exact unreleased allocation base.
  Reject foreign/local/subobject pointers, duplicate release and lost live
  allocations on every function exit. Release expires retained pointer copies.
- A repeated site never revives old aliases. This raw checkpoint may reject
  multiple simultaneously outstanding activations of one allocation site;
  04C-03 must supply the construction/work invariant needed by generated loops.
- Preserve every 04B storage guarantee. Typed allocation restoration remains
  explicitly unproved until 04C-02; custom/generated effects remain unproved.

## Tests and proof

- Checked constant/runtime sizes versus zero, possible-zero and wrapped sizes;
  actual-call identity and equal-clone/site substitution controls.
- Null/success branches, copies, unconditional null-safe default release,
  repeated release, release through a copied base and subsequent stale reads.
- Early return, lost pointer, branch/loop joins and repeated site activation;
  useful allocation/release positives, not blanket rejection.
- Private evidence compile-fail controls; full C units, Rustdoc, Clippy,
  Buildifier/docs, tracked/release and deterministic eight-target gates.
- Independent uncapped Sol Extra High review and explicit disposition of findings.

## Commit gate

Commit/push 04C-01 with exact evidence. Parent 04C and C compliance remain open.

## Implementation and evidence

Private allocation requests are derived from pointer-identical, site-validated
call observations over the same immutable numeric/control context. They retain
the original positive nonwrapping byte interval and measured default alignment.
The private function/opaque-point origin is authenticated against the actual
single root call, not supplied by the caller or inferred from a matching name.

Raw allocation state now distinguishes Possible, Live, Null, Released, Settled
and Unproved outcomes. Actual null tests refine all copies of the same origin.
Default release consumes null or the exact unreleased base and expires surviving
copies recursively; resource accounting survives overwritten or expired pointer
slots. Every function return and void fallthrough checks outstanding resources.
Repeated sites cannot revive aliases or overwrite an outstanding activation.
Typed restoration, other calls and raw global escapes remain explicit failures.

All 397 C units pass. Added suites cover original call/site/byte identity,
positive and wrapped sizes, null/polarity/copied guards, identical cloned calls
with distinct allocations, release via aliases, overwritten pointers, leaks,
early returns, fallthrough, cleanup jumps, one-branch cleanup, repeated sites,
expired previous-iteration copies, foreign stack pointers and raw escapes.
Record/Union crossed with scalar/array fields directly tests every aggregate
expiry/escape recursion arm. An exhaustive finite model checks join containment,
commutativity, associativity, idempotence and outstanding-resource detection.
Private allocation evidence has an additional compile-fail boundary control.

Final cached Linux Dev Container evidence:

- Five focused targets (C units, Rustdoc/compile-fail, Clippy, Buildifier, docs):
  `59789491-cce3-4021-985a-036b7765558d`.
- Full tracked graph: `ccd172d4-69dc-4a0f-be5c-fa0d74c159fe`
  (445 rules, 320 test targets pass, two executed).
- Release: `80ba8f6d-b769-41f9-a10c-5d191d8f5035`
  (257 test targets pass).
- Deterministic eight-target conformance:
  `4290de06-bd79-4c5a-ac50-5bb4ffd687a2`
  (50 cases plus one portable test; repeated manifests byte-identical).

Independent uncapped Sol Extra High review, c17_ast_construction_review,
completed PASS with no substantiated scoped defect or required proof gap. This
was a reused reviewer context, not represented as a fresh blind review. It
directly inspected current files and the complete delta, including exact origin
authentication, numeric history, lattice/flow/alias/exit composition, all tests
and privacy/Bazel wiring. Production/specification stayed frozen throughout.
One declared test-only localization added the aggregate recursion matrix before
the final test freeze and final gates above. No finding required rejection.

Development failures are not pass evidence:

- c13ed413-f270-41b8-8232-25779192b766: 379/383 units passed. Two leak tests found
  a genuine omitted void-fallthrough check because FunctionEnd has no successor
  edge. Explicit terminal checking repaired it. Two size controls were isolated
  from later poisoned reads so they test the primary size diagnostic directly.
- 8119d5cd-e203-45b2-be50-40bd8ee891b0: 388/389 units passed. A test negated an
  Int predicate without the AST-required Bool conversion; only the fixture changed.
- All intervening repair gates and the final matrix passed without weakening
  admitted storage or numeric checks.

The preceding storage checkpoint 230282ba5d8774b3cb2ad3d38de2372dd40c10db also
has exact-SHA hosted success in run 34319983308. This allocation checkpoint is
not a complete owning-value certificate or typed C cutover. Parent 04C remains
open for typed heap storage/dynamic extents and semantic owner transitions;
04D/05 still discharge custom incoming contracts and generated body effects.
