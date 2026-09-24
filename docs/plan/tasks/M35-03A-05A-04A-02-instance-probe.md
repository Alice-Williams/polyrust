# M35-03A-05A-04A-02 — Authenticated instance graph observation

- Status: complete
- Parent: [instance ownership](M35-03A-05A-04A-instance-ownership.md)
- Depends on: [identity model](M35-03A-05A-04A-01-instance-model.md)
- Specification: [canonical owners](../../specification/typed-generation/rust-canonical-type-owners.md)

## Contract

Extend the observation-only rustc experiment, not production source admission.
Retain original Result/error/variant/payload-field/core-root facts in a private
compiler-session witness. Reconcile owned stable facts in one graph-scoped
interner; no TyCtxt, arena DefId or Ty escapes a compiler session.

## Definition of done and tests

1. Actual checked signatures, including normalized aliases, derive all facts.
   Copy/no-drop and normalized standard error projection remain authenticated.
   Query the normalized error layout and reject query failure. Observation is
   not target-profile admission: pinned Rust 1.98 has a one-byte error kind.
   A test-only payload-free profile must reject the real error before output;
   a unit-layout control proves that this guard accepts an actual zero size.
   Lossless target state bindings remain a separate prerequisite, 04A-03.
2. Two independent producers and a fan-in consumer obtain one exact graph handle;
   traversal permutations and unrelated consumers preserve placement/facts.
3. Wrong enum/variant/field/error/core facts reject; equal names/shapes do not
   authenticate. Compile-fail tests reject forging the private witness.
   An independent rustc ADT/lang-item/parent-walk oracle audits all seven emitted
   roles. Seven compiling wrong-role mutations must fail that audit, even when
   all producers would otherwise agree on the same wrong descriptive facts.
4. Exact/one-over owner and use limits, conflicting late facts, missing owners
   and cycles reject before publication. Frozen observation cannot be mutated.
   Target-package certificates remain the responsibility of 04B/04C/04D.
5. Preserve scalar output and current Result source rejection. Full Linux Bazel
   release/lint, independent broad GPT-6-SOL review and scoped commit/push.

This proves a single merged checked graph. Cross-process trusted imports and
byte-exact filesystem TOCTOU protection are not claimed.

## Resource and failure contract

Allow at most 1,024 combined source-crate and canonical-instance owners and
100,000 encountered signature uses. Reusing an owner does not charge another
owner, but every registration charges a use. Test exact production limits and
one-over failures. Any registration failure poisons the interner, so ignoring an
error cannot produce a frozen successful prefix. Missing source owners and cycles
are rejected by the existing declared graph checker; no output is printed until
the entire compiler graph and original-handle closure succeed.

## Evidence

- Eight focused compiler/format/compile-fail targets pass; invocation
  `5b8d2c4f-a1a0-485f-849d-6c406730c3e7`, 23.046 seconds.
- Full Linux Bazel release/lint: all 1,047 targets pass; invocation
  `856c2dc7-e4ae-461e-a94f-96c98b247557`, 71.824 seconds. The code and normative
  contract tree is `f8aa012c0f7999d02d6646b5c6866910fed9bd09`.
- Fan-in, aliases, actual dependency traversal reversal, all seven wrong-fact
  controls, exact/one-over 1,024 owners and 100,000 uses, private construction,
  frozen mutation, late compiler failure and input/scratch restoration pass.
- All 530 recorded generated files remain byte-identical and 45 protected WIP
  files remain unchanged and excluded from the tested checkpoint.
- GPT-6-SOL exact-tree review is clean. The review/test loop found the pinned
  error is one byte, corrected an invalid zero-payload assumption, fixed test
  module paths and honored metadata publisher no-overwrite in scratch fixtures.
  The normal probe records layout without issuing a target certificate; a
  payload-free-profile negative plus unit-layout positive prove the distinction.

No production target frontend has been opened. Full original error-kind facts,
lossless state proof and version-2 target bindings remain 04A-03 and 04B–04E.
