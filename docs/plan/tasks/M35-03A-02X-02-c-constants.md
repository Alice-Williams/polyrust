# M35-03A-02X-02 — C U32 constant foundation

- Status: complete
- Parent: [02X](M35-03A-02X-character-constants.md)
- Depends on: [constant oracle](M35-03A-02X-01-constant-oracle.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-character-constants.md)

## Contract

Extend the closed certified constant inventory to exact U32 values using
existing typed syntax. Keep source character validation outside the target.
Preserve original object/dependency identity, qualifiers and resource bounds.

## Definition of done and tests

Test exact literal/inventory round trips, mismatched types/initializers,
readonly storage, aliases, original owners and forged imports. Native certified
objects/readers agree with the independent scalar corpus and U32 extrema under
strict GCC14/Zig O0/O2, UBSan, standalone headers and separate consumers.
Compiling narrowing/replacement/value faults must be detected. No raw source,
new runtime, broader integer capability or source admission. Full release/lint,
fresh broad review, old output/WIP preservation and separate commit/push.

## Implementation and verification history

The target inventory now has U32(u32), with exact typed unsigned literals and
public-header const storage. Existing compiler metadata matches are exhaustive
but do not admit new source constants. The old unsigned-object rejection case
now covers U8/U16/U64 and mutable U32, retaining its object-shape assertion.

Native proof is partitioned into a separate Bazel target and covers 4,133
certified objects plus imported readers through original-owner alias facades:
4,127 oracle values and six additional target-only U32 values. Strict separate
compilation, standalone headers, readonly consumers and four compiling faults
run under GCC14/Zig O0/O2 and GCC UBSan. All prior tests remain enabled.

The first focused run passes all 858 main C unit cases (nine expensive native
cases are separately partitioned), Rust/Bazel lint and the partition contract.
The new native corpus is still running. Fresh broad Sol Extra High review of
3ef9f443d3fe8981ff322bddc16580a61eaab120 found no core defect. Its diagnostic
polish finding is accepted: the exact dependency constant error must list U32
alongside the other supported target types. Full regression, preservation and
documentation closure remain pending; no completion or push is claimed.

The first native attempt spent over five minutes certifying 256-reader batches
before reaching native compilation. It was intentionally interrupted and the
batch size reduced to 64. All 4,133 objects and readers, original owners,
five compiler profiles and four fault controls remain; no timeout/assertion
was relaxed and no test was disabled. The complete focused run is repeated
on the smaller certification graphs and corrected diagnostic before closure.

Reviewed tree 846595979607499a41dffdb0384745c465edab57 passes all six focused
checks. All 858 C unit cases pass; the new native target passes in 211.2 seconds,
observing every object and imported reader under all five profiles. Actual
fault controls, standalone headers and readonly consumers pass. Rust/Bazel
lint and the partition contract pass. The same reviewer checked the diagnostic
and batching delta and found no remaining defect or coverage reduction.
The full 1,028-target Linux release/lint gate is now running on that exact tree.

The full gate identified missing source-policy registration for the new
handwritten native consumer harness. Add its exact test path to the existing
fixture catalogue and its adjacent-path failure-injection checks. Production
renderer restrictions are unchanged: neither similarly named source files nor
copies inherit permission. This is test-harness registration, not permission
to embed dependency directives in generated library bodies. The remaining
regressions continue before the corrected policy is gated in a new snapshot.

## Completion evidence

Corrected tree cc27295ad9a14b80214378562ce681b099062b24 passes all 1,028
Linux Bazel release/lint tests (five executed, the remainder correctly cached).
Invocation: 3f7467a6-a7d0-43f5-821c-362a99c3e840; local receipt:
/tmp/m35-u32-constant-c-corrected.log. The preceding complete run passed 1,027
tests and failed only the now-corrected harness registration. No assertion,
timeout or production restriction was weakened.

A fresh independent Sol Extra High whole-scope review of that corrected tree
found no core defects. It checked all 16 scoped files, exact dependency/source
authority, compiler admission, structural rendering, the complete native matrix,
partition wiring and exact-path policy negative controls. Earlier diagnostic
feedback was accepted and fixed; there are no disputed core findings.

All 530 pre-existing generated-file hashes and 38 unrelated WIP hashes are
unchanged. The documentation closure is gated again before the separately
scoped commit/push. Source character constants remain unsupported until 02X-04;
the next checkpoint proves Java's existing Int representation independently.
