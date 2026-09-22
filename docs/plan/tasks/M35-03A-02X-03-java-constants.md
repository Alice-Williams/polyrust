# M35-03A-02X-03 — Java scalar constant foundation

- Status: complete
- Parent: [02X](M35-03A-02X-character-constants.md)
- Depends on: [C foundation](M35-03A-02X-02-c-constants.md)
- Specification: [Java21](../../specification/typed-generation/languages/java/rust-character-constants.md)

## Contract

Prove existing typed Int constant declarations/readers/imports across the
character constant corpus. Do not pretend target syntax distinguishes Rust
Char from I32. Keep this as focused reusable proof, with production changes
only if an actual target certification gap is demonstrated.

## Definition of done and tests

Certified fields, local readers and original-owner imported readers produce
the independent scalar values with strict separate Java21 compilation and
normal/-Xint execution. Compiling narrowing/replacement/value mutations are
detected after all dependents are recompiled. Exact type/value/owner/import and
resource negatives pass; negative Int controls remain target-valid. No Java
char/boxing/runtime or source admission. Full release/lint, fresh broad review,
preservation checks and separate commit/push are required.

## Implementation

Independent fixture preparation preceded the C gate's completion; Java
verification follows the separately committed C checkpoint. The proof uses
64-value batches to keep certification graphs bounded and avoid the shared
fixture's constant/reader declaration-ID overlap at 100 values. Every corpus
value gets a certified field, local reader and original-owner imported reader
through an alias-only package. Target-only Int controls are deliberately not
Unicode scalar witnesses. There are no production code changes.

## Verification history

The first isolated review/build snapshot accidentally reused an old temporary
Git index. Review and Bazel analysis both identified missing committed oracle
registration. This finding was accepted: the snapshot is discarded as evidence,
not repaired by weakening its tests. The snapshot procedure now initializes
from current HEAD on every run and rejects any changed path outside the exact
checkpoint list. Host files and the pushed C checkpoint were not changed.
The initial command also named a nonexistent lint target; the corrected run
uses rust_clippy_test. No Java native success is claimed from that attempt.

Compilation then caught an iterator borrow error in the repeated-read fixture;
the final return now clones the same typed read as the preceding local bindings.

## Focused evidence and reviews

The complete focused gate on 06206eefc11e0fe03b96643d918e0c3a94887461 now
passes all eight targets, including 436 Java unit cases, Rust/Bazel lint,
policy and partition checks. The native matrix passes in 369.2 seconds, covering
all 4,133 fields/local readers/imported readers, strict separate Java21
compilation, normal/-Xint, readonly consumers and four compiling faults after
dependent recompilation. Both independent broad Sol Extra High reviews are
clean across the complete ten-file implementation. Review's documentation-hygiene
note was accepted: superseded running-status statements are removed here.

## Completion evidence

Implementation/evidence tree f0e45041fd2281432f5ec8542bb4d27d7c68660d passes
all 1,029 Linux Bazel release/lint tests (11 executed, the remainder correctly
cached). Invocation: c9a52f72-3b20-40e5-8088-85a4a7009a3f; local receipt:
/tmp/m35-character-java-full.log. All earlier native Java regressions remain
enabled and pass. No timeout, assertion or production restriction was weakened.

All 530 previous generated-file hashes and 38 unrelated WIP hashes remain
unchanged. Final documentation is gated again before the separately scoped
commit/push. Java's target foundation is complete without production changes;
original Rust character constant admission remains the separate 02X-04 step.
