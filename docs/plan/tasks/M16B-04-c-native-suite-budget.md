# M16B-04 — C native suite execution budget

- Status: in-progress
- Parent: [native Linux CI](../milestones/M16B-native-linux-ci.md)
- Precedent: [Java native suite budget](M16B-03-java-native-suite-budget.md)

## Failure and scope

Hosted run 35954437869 executed all 1,041 release tests: 1,040 passed and
//crates/backend-c:portable_backend_c_unit_test timed out at exactly 300 seconds.
That target contains 884 cases after its eleven explicitly partitioned native
cases. The remaining suite still includes compiler/oracle executions and mutation
controls; its default medium classification is no longer appropriate. No Java
test or lint failed in that hosted run.

Classify this C suite as large, using Bazel's bounded long timeout (900 seconds),
as already done for Java. Do not alter global timeouts, test selection, assertions,
native partitions, retries or cached test results. A timeout is not a passing
result: execute the complete affected suite before committing the correction.

## Definition of done and tests

- Bazel resolves the C unit target to size large and timeout long.
- All 884 selected cases complete, without ignored cases or failures, including
  a container execution with four Rust test threads.
- All workspace/release, Rustfmt, Clippy, Buildifier, documentation, partition
  contract and CI policy tests pass on the isolated correction-only tree.
- Independent review finds no unresolved core defect.
- Commit and push only this BUILD classification and its evidence document,
  separately from the uncommitted Java nominal-import implementation.
- Inspect the replacement hosted run; do not claim CI green before it succeeds.

## Evidence

The isolated correction-only tree effc5b17e21f0edb65929a904a5c70c06d94c78c matches
its Git archive; Java import implementation and unrelated ownership work remain
outside this checkpoint. Bazel's XML query resolves size large and timeout long.

An execution with RUST_TEST_THREADS=4 passed all 884 selected cases: zero failed,
zero ignored and exactly eleven separately partitioned native cases filtered.
The Rust test process finished in 102.59s (Bazel reports 103.2s), invocation
ed0718b1-abce-4ba3-837d-348842e6fc6c. This was a real execution, not a cached result.

GPT-6-SOL extra-high read-only review found no defect: only the target's size
classification and this document change; test arguments, partitions, assertions
and cache behavior are unchanged.

The complete container command bazelisk --batch test //... //:release_gate
passed all 1,041 tests on the correction-only tree in 169.778s, invocation
a94329e5-8e47-4124-8af4-9e23b9542e06. Eight actions executed; unchanged test results
were cached. The C suite also passed with default thread settings in 97.5s.
Rustfmt, Clippy and Buildifier passed, and all 530 previously recorded generated
files retained their original hashes. Replacement hosted CI verification remains
pending; this is not a claim that the failed hosted run was green.
