# M35-03A-05A-04A-02 — Authenticated instance graph observation

- Status: in-progress
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

Implementation/test preparation began while 04A-01's isolated release gate ran.
This checkpoint is excluded from that candidate and cannot be committed before
04A-01 is complete. No production target frontend has been opened.
