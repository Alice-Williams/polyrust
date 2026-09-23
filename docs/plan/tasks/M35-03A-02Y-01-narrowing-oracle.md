# M35-03A-02Y-01 — Independent checked-narrowing oracle

- Status: complete
- Parent: [02Y](M35-03A-02Y-checked-narrowing.md)
- Depends on: [02X](M35-03A-02X-character-constants.md)
- Specification: [shared](../../specification/typed-generation/rust-checked-narrowing.md)

## Contract

Use independent unbounded integer range checks for i64-to-i32 success/error
truth. Native Rust TryFrom and TryInto are observations, never expected values.
The test-only protocol has distinct err and ok:<signed integer> tokens: error
must not become successful zero or another sentinel.

## Definition of done and tests

- Pin the unique corpus size, exhaust signed i16 values, cover both i32 limits
  and i64 extrema with neighbours, signed powers of two, deterministic full-i32
  success samples and full-i64 failure samples.
- Compare both standard conversions at O0/checks-on and O2/checks-off.
- Execute six faulty native conversions: wrapping, saturation, exclusive
  endpoints, missing lower bound, missing upper bound and zero success values.
  Each must exactly match its independent faulty model and differ from truth;
  the second standard conversion must remain correct.
- Malformed result protocols and invalid/out-of-domain reference input reject.
- Bazel owns native binaries and tests; Rust Clippy/rustfmt, Bazel lint, the
  full release gate and fresh broad review pass. Existing output and unrelated
  WIP remain unchanged. No source/target implementation or dependency is added.

## Execution

Tree a2022560a2e1c0982a30d08b921ceadf1b4c704d passes all six focused targets
and the full 1,036-target Linux Bazel release/lint gate (10 executed, the rest
cached). Local receipts: /tmp/m35-narrowing-oracle-focused.log and
/tmp/m35-narrowing-oracle-full.log. A fresh broad GPT-6-Sol extra-high review
of the exact 19-file scope is clean; no finding or test was dismissed.

The pinned 74,389 unique inputs contain 69,853 successes and 4,536 errors.
TryFrom and TryInto agree with independent truth at both native profiles;
all six actual mutant programs match their independent faulty models and
disagree with truth. Strict protocol, invalid-input and unknown-mode controls
pass. Rust Clippy/rustfmt and Bazel lint remain authoritative.

All 530 earlier generated files remain byte-identical, as do the four recent
character-constant bundles compared with their exported native artifacts.
All 45 unrelated WIP hashes are unchanged. No production mapper, backend or
source-admission code changes. Documentation closure is fully gated again
before the separately scoped commit/push.

Next is the bounded no-heap result prerequisite, not unchecked narrowing
admission. The dependency correction allows that foundation before full scalar
or collection parity without relaxing the heap/ownership gates.
