# M34A-11-02D-04C-02C-03 — Initialized buffer prefixes

- Status: complete
- Depends on: M34A-11-02D-04C-02C-02

## Goal

Prove ordered dynamic-buffer construction and safe partial reads/cleanup from
actual loop write coverage, then close parent 02C and allocated-storage 02.

## Implementation order

1. Add private prefix-bound/coverage states, separating absent evidence from a
   proved empty prefix and preserving conservative complete-element summaries.
2. Authenticate loop-entry, actual step and normal false-exit transitions using
   the existing graph/LoopEvidence, and compose them with numeric/memory solving
   and strict replay. Seed before header joins; grow only after successful steps.
3. Recheck prefix bounds and completeness at reads without storing stale proof
   flags on pointer paths. Retire/update coverage on writes, scope/lifetime
   changes, allocation restart and provisional errors.
4. Support direct immutable frontier snapshots for partial-read/cleanup loops.
   Preserve existing numeric loss fallbacks; prefix initialization alone does
   not certify arithmetic history or across-iteration numeric values.
5. Add actual-AST positive/negative coverage, privacy and transition tests;
   run full cached gates and independent review, then re-audit both parents.

## Definition of done

- Derive element/field coverage on every continuing actual loop path, including
  backedges and continues, and relate the prefix to the real counter phase.
- Complete-element writes grow only the proved prefix; joins intersect coverage.
  Skipped/conditional/reordered writes cannot establish a complete buffer.
- Early exits retain only actual initialized coverage. Release/reactivation and
  conflicting writes invalidate stale prefix and numeric observations.
- Parent definitions of done are re-audited; no remaining dynamic-storage
  obligation is deferred under a completed status.

## Tests and proof

- Runtime-length scalar/aggregate construction, guarded prefix reads and partial
  cleanup with zero, maximum-safe and nested-layout cases.
- Missing fields, skipped/reordered writes, continue/break/cleanup, wrong counter,
  stale count, join and expired/reactivated allocation controls.
- Full cached focused/tracked/release/lint/eight-target gates and uncapped
  independent review. Record exact evidence, close parents, commit and push.

## Implementation checkpoint

Private counter/snapshot/original-count states distinguish absent evidence from
a proved empty prefix. Actual graph entry seeds before the header join; complete
selected cells are captured before actual steps and committed only after both
numeric and memory transfers succeed. Only the actual normal false loop edge
can establish original-count coverage. Direct immutable frontier declarations
support later partial cleanup loops.

Prefix summaries participate in ordinary writes, joins, index mutation,
scope/activation retirement and allocation release. Reads and bounds recheck the
current product; pointer paths carry no cached prefix certificate. Non-exact
numeric prefix values retain the existing incomplete/loss fallback, while
current-element exact reaching facts remain unchanged.

A maintainer audit found that applying a pre-step captured summary could undo
the counter update's saved-address invalidation. The actual pointer-buffer
regression failed before repair (43d1a70e-684f-4ae2-bbcd-5d8e0a33719e):
the changing-index address was wrongly accepted while the fixed-address control
passed. Pending application now applies index retirement, and declaration
snapshots also preserve old-activation expiry, before restoring the summary.

Focused invocation 14670547-50c1-43ef-a5af-d268bf79a9b4 passes 512 C units
plus typed compile-fail, Clippy, documentation and Buildifier. Tests cover
runtime/full/partial construction, branch and continue must facts, missing and
reordered writes, unrelated bounds/indices, nested struct/union/array coverage,
zero/MAX bounds, cleanup jumps, frontier snapshots, numeric losses, release,
saved addresses, scope exit and repeated allocation activations. Two additional
privacy doctests reject public prefix-state/bound construction.

## Independent review and regression repairs

The first uncapped independent Sol Extra High review inspected the whole scoped
checkpoint and its prerequisite composition. It found one contract mismatch and
one required proof gap; both were accepted, with no dismissed findings.

- Snapshot capture also accepted a copy of an existing snapshot. This is not a
  demonstrated memory-unsafety case, but violates the direct-counter grammar.
  The actual-AST control failed before repair in invocation
  4b33691c-81bc-4315-83de-34fa89abd805 (512 passed, one failed). Capture now
  matches only the private Counter variant; direct captures remain accepted.
- Snapshot activation expiry needed composed coverage. The new graph tests
  reapply an actual declaration through the paired transfer over solver-derived
  pointer-buffer state, and cross an actual break/scope-exit edge. They check
  old-address rejection, snapshot-authority removal and unrelated live-address
  preservation, without constructing graph or prefix evidence. Temporarily
  removing the snapshot-specific expiry makes the declaration test wrongly
  accept the stale address: mutation invocation
  0a6e4127-4f05-4370-808e-8301abf4d0a8 fails as intended. The expiry rule was
  restored; no mutation is part of the implementation.

Focused invocation 731afde2-c276-4aad-823c-f83c36ffd04d passed all 515 units
and the four companion gates with these regressions. The second independent
uncapped Sol Extra High review then inspected the complete frozen checkpoint,
not only these repairs, and returned PASS with no substantiated defect or
required proof gap. It confirmed both parent definitions of done are covered.
Neither review is described as fresh blind: the independent reviewers retained
their earlier prerequisite-review context. C still uses legacy emission and is
not render-ready.

## Final evidence

- Focused: 9c89eff9-5c10-43c6-8e64-743a34d25e1f, all 515 C unit tests,
  typed compile-fail, Rust Clippy, documentation and Buildifier pass.
- Tracked graph: 1a2efd1b-4f2e-4528-b98a-102b8ea40a46, 445 rules
  (125 non-test and 320 test targets), all 320 tests pass; 45 executed.
- Release: 182498d6-e4e0-4739-8beb-7c77840a98f9, all 257 tests pass.
- Conformance: e2bd216c-8f38-4f81-9d56-2611163b3f91, 50 cases and one
  portable test; evaluator plus eight targets agree, repeated manifests are
  byte-identical. This still exercises legacy C, not migrated C emission.
- All builds/tests ran through cached Bazel in the Linux dev container.
  The user-owned untracked stdlib-abs work was excluded and remains untouched.

The parent audit below passed both maintainer and independent review. Close
02C and allocated-storage 02 with this checkpoint, then continue owner 04C-03.

## Parent audit checklist

- Parent 02C's original count/layout provenance is delivered by 01, guarded
  element identity/lifetime by 02, and ordered must-initialization by this task.
- Parent 02's fixed restore/type/layout and numeric-memory requirements are
  covered by 02A/02B; dynamic count/access/prefix requirements compose through
  02C. Existing regressions remain in the same full gate.
- Partial cleanup here means valid representation reads and raw allocation
  release. Semantic owning-child transfer/rollback, multiple live loop-produced
  owners, generated/custom call effects and incoming ABI contracts remain their
  explicit later checkpoints; none is certified by storage diagnostics.
