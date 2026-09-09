# M34A-11-02D-04A — Actual index extent handoff

- Status: complete
- Depends on: M34A-11-02D-03

## Goal

Consume authenticated numeric index observations in the storage layer, proving
the declared array bound before the actual index operation is evaluated.

## Definition of done

- Expose a private read-only view of already site-validated Index obligations;
  do not export Number, State, mutable ranges or a caller proof constructor.
- Require a clean nonnegative integer range strictly below the actual array
  type's length, including address-only indexing. Never use a later guard.
- Derive the length from the exact indexed base, including nested arrays and
  member arrays. Another array's length is not evidence for this base.
- File-scope address initializers retain their actual initializer/place origin
  separately from runtime graph sites; constant wrap history is not discarded.
- Pointer indexing fails with an explicit unresolved-extent diagnostic until
  actual pointer provenance is implemented in 04B/04C. It is not skipped.
- Keep this diagnostic boundary explicitly narrower than complete storage
  safety: initialization, pointer dereference, active members, lifetime and
  allocation ownership remain later checkpoints. No rendering certificate.

## Tests and proof

- Zero/last-element positives; negative, exact length, larger, unknown and
  wrapped index negatives. Length-one and nested unequal-dimension controls.
- Actual runtime guard, wrong bound, reversed guard, missing lower bound,
  nondominating or stale guard, both-path versus one-path joins.
- Read/write/address occurrences, short-circuit and conditional reachability,
  loop pre-step versus post-step, and explicit pointer-index rejection.
- Static negative/out-of-bounds/wrapped address indices and cloned/missing
  initializer-site observations cannot bypass the same bound consumer.
- Existing private-site mutations remain mandatory; add a compile-fail check
  against the new observation view. Full C units, Rustdoc, Rust/Bazel lint,
  docs, tracked/release and deterministic eight-target gates pass.

## Commit gate

Record exact evidence and independent uncapped Sol Extra High review; commit
and push 04A separately. Do not mark parent 04 complete.

## Implementation and evidence

The storage extent consumer borrows private numeric observations and derives
each bound from the actual canonical array base. It checks the entire integer
domain and retained provenance. Pointer indices produce UnprovedPointerExtent,
not a guessed array bound. Diagnostic success remains explicitly narrower than
complete storage safety and cannot reach rendering.

File-scope address initializers are inventoried separately through the admitted
static grammar. Actual root-initializer and place references are retained and
revalidated, including inventory cardinality. Their constant operands reuse
the numeric transfer kernel in non-executing Derive mode, preserving wrap
history. No graph point is manufactured for a static initializer.

The C suite has 335 passing unit tests. New focused suites cover every listed
runtime extent/guard/loop case, nested static initializer families and qualifier
conversions, indexed member addresses, wrapped constants and private observation
substitution/deletion. New production modules are focused and below size limits.

Passing cached Linux Dev Container evidence:

- Five focused targets (C units, Rustdoc, Clippy, Buildifier, docs):
  `bdb30b64-d012-4391-abfc-b2013dd575a6`.
- Full tracked graph: `582e7253-5e73-4d70-a310-41208dcbb8e8`
  (445 rules, 320 test targets pass, 45 executed).
- Release: `140dd59d-4a3b-4c4a-9e9f-e44e2e5fd853`
  (257 test targets pass).
- Deterministic eight-target conformance:
  `b8ef6fc0-23b9-4016-a946-78f6a21777be`
  (50 cases plus one portable test agree; repeated manifests are byte-identical).

Independent uncapped Sol Extra High review, c17_contextual_final_review,
completed clean with no substantive defect or required semantic proof gap.
This was a new scope in an existing reviewer context. It covered the full 04A
production/spec delta, admitted static grammar, numeric/site handoff, constructors,
tests and Bazel/privacy wiring, reusing previously audited 03C prerequisites.
Production/specification were frozen throughout; only declared test-localization
additions preceded the final test freeze and complete gates. No finding needed
rejection or repair during that review. The user-owned stdlib-abs tree is excluded.

Development failures are not claimed as passes:

- b3d5ab62: 329/330 units passed; a logical-operand fixture omitted its required
  explicit Int-to-Bool normalization. Only the fixture was corrected.
- d057e8c3: a static fixture attempted private CSourceFile field access from
  outside the AST module. It now uses the immutable items accessor and builder.
- 0b615c33: both new static regressions ran red against the incomplete runtime-only
  consumer (out-of-bounds and wrapped indices returned Ok). Static observations
  fixed that real omission. fe3a49bc then passed all five focused targets before
  the final static-wrapper localization and evidence above.

This closes 04A only. 04B–04D initialization/provenance/lifetime/ownership and
05 generated-call summary obligations remain open; legacy C conformance is
regression evidence, not typed-C cutover evidence.
